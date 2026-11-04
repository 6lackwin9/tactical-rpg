//! VISCLAR1-D slice 1 (decision "Battle view answers, part 2"; mocks `rest_colour`, `intent_ring`, `party_ap`): a Rest block
//! is light slate with its icon; the intent ring replaces the side ring on every unit; the party cards' AP pips are one
//! colour, drain with the moves and refill at round start, with HP and poise numbers beside their rows; poise shows as 5
//! pips of 20 % each, display only; a banner names each new round; the next round's planning starts by itself, except while
//! watching a recording, under a setting. Fixtures are the catalog's encounters under the timeline ruleset; expectations
//! come from these rules, never shipped numbers.

use crate::battle::draw::card_rows;
use crate::battle::flow::next_round_by_itself;
use crate::battle::marks::{poise_pip_rect, poise_pips, POISE_PIPS};
use crate::battle::playback::{round_banner_alpha, ROUND_BANNER_MS};
use crate::battle::tips::{intent_ring_color, pip_fill, ring_intent, ApPips, AP_DRAIN_MS, AP_PIP, AP_PIP_EMPTY, AP_REFILL_MS, REST_BLOCK};
use crate::battle::{Action, BattleScreen, AP_PIPS, AUTO_CONFIRM, CONFIRM_GUARD_MS, GRID_X, GRID_Y, POISE, TILE};
use crate::screens::{Cx, Key, Screen};
use aegis_ui::canvas::{Rect, Rgba};
use aegis_ui::theme::th;
use aegis_ui::ui::{Gfx, H, W};
use emberwake_battle::grid::{GRID_H, GRID_W};
use emberwake_battle::intent::Intent;
use emberwake_battle::round_intent::RoundIntent;
use emberwake_battle::session::{Phase, Session};
use emberwake_rules::ruleset::TIMELINE;
use serde_json::{json, Value};
use std::time::Instant;

fn arena_in(view: &str, adjust: impl FnOnce(&mut Cx)) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!(view);
    adjust(&mut cx);
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.into()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn arena() -> (Cx, Gfx, BattleScreen) {
    arena_in("flat", |_| {})
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn draw(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
    b.draw(cx, g, (-500, -500), true);
}

/// One tick with the step pause and the impact gap over.
fn tick_ready(b: &mut BattleScreen, cx: &mut Cx) {
    b.next_at = Instant::now();
    b.gap_until = None;
    b.tick(cx);
}

/// Confirm (its guard after an advance over), then ticks until the round has played.
fn play_round(b: &mut BattleScreen, cx: &mut Cx) {
    b.advanced_at = None;
    b.act(cx, Action::Confirm);
    for _ in 0..600 {
        if s(b).phase == Phase::Replay {
            return;
        }
        tick_ready(b, cx);
    }
    panic!("the round did not end");
}

fn requests(b: &BattleScreen, cmd: &str) -> usize {
    b.host.history.iter().filter(|r| r["cmd"] == json!(cmd)).count()
}

fn card(b: &BattleScreen, cx: &Cx, a: usize) -> Value {
    b.state(cx)["party_cards"].as_array().unwrap().iter().find(|c| c["unit"] == json!(a)).cloned().expect("the member's card")
}

fn shown(b: &BattleScreen, cx: &Cx, a: usize) -> f32 {
    card(b, cx, a)["ap_shown"].as_f64().unwrap() as f32
}

fn widget_rect(b: &BattleScreen, id: &str) -> Rect {
    b.w.iter().find(|x| x.id == id).unwrap_or_else(|| panic!("no widget {id}")).rect
}

fn px(g: &Gfx, x: i32, y: i32) -> Rgba {
    let i = ((g.view.y(y as f32) * g.canvas.w + g.view.x(x as f32)) * 4) as usize;
    [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2], g.canvas.px[i + 3]]
}

fn count(g: &Gfx, r: Rect, col: Rgba) -> usize {
    let mut n = 0;
    for y in g.view.y(r.y as f32)..g.view.y((r.y + r.h) as f32) {
        for x in g.view.x(r.x as f32)..g.view.x((r.x + r.w) as f32) {
            let i = ((y * g.canvas.w + x) * 4) as usize;
            if g.canvas.px[i..i + 3] == col[..3] {
                n += 1;
            }
        }
    }
    n
}

/// A real 1280 x 720 canvas: at the test's 640 x 360 a thin ring is all antialiasing.
fn gfx_720() -> Gfx {
    let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).expect("font");
    Gfx { canvas: aegis_ui::canvas::Canvas::new(1280, 720), view: aegis_ui::painter::View::fit(1280, 720), fonts: aegis_ui::painter::Fonts::new(&font), kit: Default::default() }
}

// ---- The Rest block -----------------------------------------------------------------------------------------------

/// Rest reads light slate with its icon and no fade, as the leftover the planner rests (Auto-fill off, after a pick) and once
/// taken as picks; a kind other than Rest never takes that colour.
#[test]
fn a_rest_block_is_light_slate_with_its_icon() {
    let (mut cx, mut g, mut b) = arena_in("flat", |cx| cx.settings.v["auto_fill"] = json!(false));
    cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    b.tick(&mut cx);
    let h = b.selected.unwrap();
    let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.units[e].is_active() && s(&b).w.has(e)).expect("a foe on the field");
    b.act(&mut cx, Action::UnitClick(foe));
    let pick = b.popup.as_ref().and_then(|p| p.rows.iter().position(|r| r.valid && matches!(r.act, crate::battle::input::RowAct::Pick(_)))).expect("a pick");
    b.act(&mut cx, Action::Choose(pick));
    b.act(&mut cx, Action::ClosePopup);
    draw(&mut cx, &mut g, &mut b);
    let check = |b: &BattleScreen, g: &Gfx, cx: &Cx| {
        let sess = s(b);
        let spans: Vec<(i32, String)> = sess.plan.entries_for(&sess.w, h).iter().map(|e| (e.start_block, sess.w.cat.skills[e.skill].kind.clone())).collect();
        assert!(spans.iter().any(|x| x.1 == "REST"), "Auto-fill off rests the free blocks: {spans:?}");
        for (start, kind) in &spans {
            let row = super::visclar_c_tests::top_row(g, b, *start);
            if kind == "REST" {
                assert_eq!(row, [[REST_BLOCK[0], REST_BLOCK[1], REST_BLOCK[2]]], "the Rest at block {start} is one solid light slate");
            } else {
                assert_ne!(row[0][..], REST_BLOCK[..3], "{kind} at block {start}");
            }
        }
        let labels = b.state(cx)["strip_labels"].as_array().cloned().unwrap();
        let rests: Vec<&Value> = labels.iter().filter(|l| l["look"] == json!("rest")).collect();
        assert!(!rests.is_empty() && rests.iter().all(|l| l["icon"].is_string()), "every Rest carries its icon: {labels:?}");
    };
    let sess = s(&b);
    let ghosts = emberwake_battle::round_intent_plan::ghost_entries(&sess.w, &sess.plan, h);
    assert!(ghosts.contains(&true), "the leftover Rest is still the planner's");
    check(&b, &g, &cx);
    b.act(&mut cx, Action::FillGhost);
    draw(&mut cx, &mut g, &mut b);
    let sess = s(&b);
    assert!(!emberwake_battle::round_intent_plan::ghost_entries(&sess.w, &sess.plan, h).contains(&true), "F made them picks");
    check(&b, &g, &cx);
}

// ---- The intent ring ----------------------------------------------------------------------------------------------

fn with_intent(unit: usize, i: Option<RoundIntent>) -> Intent {
    let mut r: Intent = serde_like_default();
    r.unit = unit;
    r.round_intent = i;
    r
}

fn serde_like_default() -> Intent {
    let (_, _, b) = arena();
    b.host.session.as_ref().unwrap().round_intents().into_iter().next().expect("a round intent record")
}

/// The rule: a unit's ring is its intent's, on both sides; the side's where no intent shows: no record, the intent glyphs
/// off, an enemy while the tells are hidden.
#[test]
fn the_ring_is_the_intent_where_one_shows_else_the_side() {
    let a = RoundIntent::Assault;
    let rounds = vec![with_intent(0, Some(a)), with_intent(1, None)];
    assert_eq!(ring_intent(&rounds, 0, true, true, true), Some(a), "an ally");
    assert_eq!(ring_intent(&rounds, 0, false, true, true), Some(a), "an enemy, tells shown");
    assert_eq!(ring_intent(&rounds, 0, true, true, false), Some(a), "an ally's own stays with the tells hidden");
    assert_eq!(ring_intent(&rounds, 0, false, true, false), None, "an enemy's is a tell");
    assert_eq!(ring_intent(&rounds, 0, true, false, true), None, "glyphs off");
    assert_eq!(ring_intent(&rounds, 1, true, true, true), None, "no intent in the record");
    assert_eq!(ring_intent(&rounds, 2, true, true, true), None, "no record");
}

/// The five intents' rings are five colours.
#[test]
fn each_intent_has_its_own_ring() {
    let all = [RoundIntent::Assault, RoundIntent::Guard, RoundIntent::Support, RoundIntent::Maneuver, RoundIntent::Channel];
    for (k, i) in all.iter().enumerate() {
        for j in &all[k + 1..] {
            assert_ne!(intent_ring_color(*i), intent_ring_color(*j), "{i:?} and {j:?}");
        }
    }
}

/// On the real Stage: every unit on the field stands on its intent's ring, both sides; with the tells hidden an enemy's
/// ring gives nothing away.
#[test]
fn every_unit_stands_on_its_intent_ring() {
    let (mut cx, _, mut b) = arena_in("stage", |_| {});
    // the sprites, not the lettered plates: a plate hides the ring of the unit behind it
    cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    let mut g = gfx_720();
    b.tick(&mut cx);
    // a board of plain floor (this battle's copy), so no block stands in front of a foot
    let grid = &mut b.host.session.as_mut().unwrap().w.grid;
    grid.terrain.iter_mut().for_each(|t| *t = 0);
    draw(&mut cx, &mut g, &mut b);
    let rounds = s(&b).round_intents();
    let mut seen = (0, 0);
    for r in &rounds {
        let (Some(i), Some(_)) = (r.round_intent, s(&b).w.grid.pos(r.unit)) else { continue };
        // exact: the intent ring is opaque, the side ring blends with the ground
        let n = count(&g, widget_rect(&b, &format!("unit.{}", r.unit)), intent_ring_color(i));
        assert!(n > 8, "{}: no {i:?} ring ({n} px)", s(&b).w.units[r.unit].name);
        if s(&b).w.allies.contains(&r.unit) { seen.0 += 1 } else { seen.1 += 1 }
    }
    assert!(seen.0 > 0 && seen.1 > 0, "both sides checked: {seen:?}");
    cx.settings.v["show_tells"] = json!(false);
    draw(&mut cx, &mut g, &mut b);
    for r in rounds.iter().filter(|r| s(&b).w.enemies.contains(&r.unit)) {
        let (Some(i), Some(_)) = (r.round_intent, s(&b).w.grid.pos(r.unit)) else { continue };
        let foot = widget_rect(&b, &format!("unit.{}", r.unit));
        let at = (foot.y + foot.h - TILE).max(foot.y);
        let n = count(&g, Rect::new(foot.x, at, foot.w, foot.y + foot.h - at), intent_ring_color(i));
        assert_eq!(n, 0, "{}: hidden", s(&b).w.units[r.unit].name);
    }
}

/// The ring tells the intent, so the plate over a token waits for Alt, in both views; the cards keep theirs. A plate is the
/// intent's one letter drawn over the board.
#[test]
fn the_token_plate_waits_for_alt() {
    for view in ["flat", "stage"] {
        let (mut cx, mut g, mut b) = arena_in(view, |_| {});
        // the sprites: a token with no art draws its name's letter
        cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        b.tick(&mut cx);
        let plates = |cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen| {
            let recs = crate::hud::hud_tests::frame(b, cx, g, 1.0);
            let letters: Vec<String> = s(b).round_intents().iter().filter_map(|r| r.round_intent).map(|i| i.name().chars().take(1).collect()).collect();
            assert!(!letters.is_empty(), "units hold intents");
            let on_board = |x: f32| x >= GRID_X as f32 && x < (GRID_X + GRID_W * TILE) as f32;
            recs.iter().filter(|t| letters.contains(&t.text) && on_board(t.x)).count()
        };
        assert_eq!(plates(&mut cx, &mut g, &mut b), 0, "{view}: no plate on a token");
        cx.held.alt = true;
        let intents = s(&b).round_intents().iter().filter(|r| r.round_intent.is_some() && s(&b).w.grid.pos(r.unit).is_some()).count();
        assert_eq!(plates(&mut cx, &mut g, &mut b), intents, "{view}: Alt shows them all");
    }
}

/// The Flat view's foot ellipse takes the same colour rule: an ally on Assault stands on red, not its side's blue.
#[test]
fn the_flat_ellipse_is_the_intent_colour() {
    let redness = |glyphs: bool| {
        let (mut cx, _, mut b) = arena_in("flat", |cx| cx.settings.v["battle_overlays"]["glyphs"] = json!(glyphs));
        cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let mut g = gfx_720();
        b.tick(&mut cx);
        assert_eq!(b.send(&cx, json!({"cmd": "round_intent", "unit": 0, "intent": "assault", "target": s(&b).w.enemies[0]}))["ok"], json!(true));
        draw(&mut cx, &mut g, &mut b);
        let cell = crate::battle::board::cell_rect(s(&b).w.grid.pos(0).expect("on the field"));
        let (mut r, mut blue) = (0i64, 0i64);
        // the cell's lowest band, where the ellipse reaches past the sprite's feet
        for y in g.view.y((cell.y + TILE - 5) as f32)..g.view.y((cell.y + TILE) as f32) {
            for x in g.view.x(cell.x as f32)..g.view.x((cell.x + TILE) as f32) {
                let i = ((y * g.canvas.w + x) * 4) as usize;
                r += g.canvas.px[i] as i64;
                blue += g.canvas.px[i + 2] as i64;
            }
        }
        r - blue
    };
    assert!(redness(true) > redness(false), "the intent's red over the side's blue");
}

// ---- The party cards: AP ------------------------------------------------------------------------------------------

/// The pips as a function of time: they start where they were, move only toward the unit's AP and land on it.
#[test]
fn the_pips_are_a_pure_function_of_time() {
    let t0 = 1000;
    let start = ApPips::still(10.0, 0);
    assert_eq!(start.level(t0), 10.0);
    let drain = start.follow(6.0, t0, false);
    assert_eq!(drain.level(t0), 10.0, "a payment starts from what shows");
    assert_eq!(drain.level(t0 + AP_DRAIN_MS / 2), 8.0, "half way");
    assert_eq!(drain.level(t0 + AP_DRAIN_MS), 6.0, "lands");
    assert_eq!(drain.level(t0 + 10 * AP_DRAIN_MS), 6.0, "and stays");
    assert_eq!(drain.follow(6.0, t0 + 10, false), drain, "the same AP does not restart it");
    // a second payment mid-fall carries on from what shows
    let again = drain.follow(4.0, t0 + AP_DRAIN_MS / 2, false);
    assert_eq!(again.level(t0 + AP_DRAIN_MS / 2), 8.0);
    assert_eq!(again.level(t0 + AP_DRAIN_MS / 2 + AP_DRAIN_MS), 4.0);
    let refill = ApPips::still(4.0, 0).follow(12.0, t0, false);
    assert_eq!(refill.level(t0), 4.0, "the refill starts from the drained level");
    assert_eq!(refill.level(t0 + AP_REFILL_MS / 2), 8.0);
    assert_eq!(refill.level(t0 + AP_REFILL_MS), 12.0);
    assert!(AP_REFILL_MS > AP_DRAIN_MS, "the refill is the one to watch");
    assert_eq!(start.follow(3.0, t0, true).level(t0), 3.0, "skip animations and reduce motion land at once");
    // one pip holds `unit` AP; the one the level ends in is part full
    assert_eq!((pip_fill(5.0, 1, 4), pip_fill(5.0, 1, 5)), (1.0, 0.0));
    assert_eq!(pip_fill(4.5, 1, 4), 0.5);
    assert_eq!((pip_fill(5.0, 2, 1), pip_fill(5.0, 2, 2), pip_fill(5.0, 2, 3)), (1.0, 0.5, 0.0));
}

/// On the real round: whenever a move pays AP its member's pips fall from what they showed to the unit's AP over the drain
/// time; between payments they stand.
#[test]
fn the_pips_drain_with_the_moves_as_they_play() {
    let (mut cx, mut g, mut b) = arena();
    b.tick(&mut cx);
    draw(&mut cx, &mut g, &mut b);
    let allies = s(&b).w.allies.clone();
    assert_eq!(b.send(&cx, json!({"cmd": "confirm"}))["ok"], json!(true));
    draw(&mut cx, &mut g, &mut b);
    let mut paid = 0;
    for _ in 0..600 {
        if s(&b).phase == Phase::Replay {
            break;
        }
        let before: Vec<i32> = allies.iter().map(|&a| s(&b).w.units[a].ap).collect();
        assert_eq!(b.send(&cx, json!({"cmd": "step"}))["ok"], json!(true));
        draw(&mut cx, &mut g, &mut b);
        let fell: Vec<(usize, f32, f32)> = allies.iter().zip(&before).filter(|(a, was)| s(&b).w.units[**a].ap < **was).map(|(a, was)| (*a, *was as f32, s(&b).w.units[*a].ap as f32)).collect();
        for &(a, was, now) in &fell {
            assert_eq!(shown(&b, &cx, a), was, "the fall starts at the old AP");
            assert_eq!(card(&b, &cx, a)["ap"], json!(now as i32), "the number is the unit's at once");
        }
        b.clock.advance(AP_DRAIN_MS as f64 / 2.0, 1.0);
        draw(&mut cx, &mut g, &mut b);
        for &(a, was, now) in &fell {
            let mid = shown(&b, &cx, a);
            assert!(mid < was && mid > now, "falling: {was} > {mid} > {now}");
        }
        // (a rise, such as a refund, takes the refill's time)
        b.clock.advance(AP_REFILL_MS as f64, 1.0);
        draw(&mut cx, &mut g, &mut b);
        for &a in &allies {
            assert_eq!(shown(&b, &cx, a), s(&b).w.units[a].ap as f32, "landed");
        }
        paid += fell.len();
    }
    assert!(paid > 0, "someone paid AP this round");
}

/// A new round: the round start's AP rises over the refill time from the level the last round left; a round start in
/// which nothing rises is not animated.
#[test]
fn the_pips_refill_at_round_start() {
    let (mut cx, mut g, mut b) = arena();
    b.tick(&mut cx);
    draw(&mut cx, &mut g, &mut b);
    let a = b.selected.unwrap();
    assert_eq!(b.send(&cx, json!({"cmd": "resolve"}))["ok"], json!(true), "the round plays");
    b.host.session.as_mut().unwrap().w.units[a].ap = 0;
    draw(&mut cx, &mut g, &mut b);
    b.clock.advance(AP_DRAIN_MS as f64 * 2.0, 1.0);
    draw(&mut cx, &mut g, &mut b);
    assert_eq!(shown(&b, &cx, a), 0.0, "drained");
    b.act(&mut cx, Action::NextRound);
    let refilled = s(&b).w.units[a].ap as f32;
    assert!(refilled > 0.0, "the round start gives AP");
    draw(&mut cx, &mut g, &mut b);
    assert_eq!(shown(&b, &cx, a), 0.0, "the refill starts from the drained level");
    assert_eq!(card(&b, &cx, a)["ap"], json!(refilled as i32));
    b.clock.advance(AP_REFILL_MS as f64 / 2.0, 1.0);
    draw(&mut cx, &mut g, &mut b);
    let mid = shown(&b, &cx, a);
    assert!(mid > 0.0 && mid < refilled, "rising: {mid} of {refilled}");
    b.clock.advance(AP_REFILL_MS as f64, 1.0);
    draw(&mut cx, &mut g, &mut b);
    assert_eq!(shown(&b, &cx, a), refilled);
}

/// Reduce motion and skip animations show the AP at once.
#[test]
fn the_pips_land_at_once_without_animation() {
    for setting in ["skip_anim", "reduce_motion"] {
        let (mut cx, mut g, mut b) = arena();
        cx.settings.v[setting] = json!(true);
        b.tick(&mut cx);
        draw(&mut cx, &mut g, &mut b);
        let a = b.selected.unwrap();
        b.host.session.as_mut().unwrap().w.units[a].ap = 1;
        draw(&mut cx, &mut g, &mut b);
        assert_eq!(shown(&b, &cx, a), 1.0, "{setting}");
    }
}

/// The pips on the card: one colour, as many lit as the unit has AP (a pip per AP up to the cap, `ceil` above it); the
/// numbers stand beside their rows.
#[test]
fn a_card_shows_one_colour_of_pips_and_the_numbers_beside_the_rows() {
    let (mut cx, mut g, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    let a = b.selected.unwrap();
    let max = s(&b).w.units[a].max_ap;
    let per = aegis_ui::pips::unit(max, AP_PIPS);
    let slots = aegis_ui::pips::count(max, per);
    let lit = slots / 2;
    b.host.session.as_mut().unwrap().w.units[a].ap = lit * per;
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    let rows = card_rows(widget_rect(&b, &format!("party.{a}")), 0.0);
    for k in 0..slots {
        let pip = aegis_ui::pips::pip_rect(rows.ap.x, rows.ap.y + rows.ap.h / 2, k);
        let (x, y) = (pip.x + pip.w / 2, rows.ap.y + rows.ap.h / 2);
        assert_eq!(px(&g, x, y)[..3], if k < lit { AP_PIP } else { AP_PIP_EMPTY }[..3], "pip {k} of {slots}, {lit} lit");
    }
    // every number sits to the right of its row, on the row's line, inside the card
    let u = &s(&b).w.units[a];
    let st = emberwake_battle::poise::state(&s(&b).w, a).copied().expect("the timeline ruleset has poise");
    for (text, row) in [(format!("{}/{}", u.hp, u.max_hp), rows.hp), (format!("{}/{}", st.value, st.max), rows.poise), (format!("{}/{}", u.ap, u.max_ap), rows.ap)] {
        let t = recs.iter().find(|t| t.text == text && t.x >= (row.x + row.w) as f32 && t.x < rows.right as f32 && (t.y - row.y as f32).abs() < 8.0).unwrap_or_else(|| panic!("no {text} beside its row"));
        assert!(t.x + t.w <= rows.right as f32 + 0.5, "{text} stays inside the card");
    }
}

// ---- Poise pips ---------------------------------------------------------------------------------------------------

/// Five pips, each 20 %: a pip lights for every started 20 % of the poise, for any maximum; none means none left.
#[test]
fn poise_lights_a_pip_per_started_twenty_percent() {
    assert_eq!(POISE_PIPS, 5);
    for max in [1, 3, 5, 7, 60, 101, 137, 210] {
        for value in 0..=max {
            let expected = (value as f64 / max as f64 / 0.2 - 1e-9).ceil().max(0.0) as i32;
            assert_eq!(poise_pips(value, max, false), expected, "{value} of {max}");
        }
        assert_eq!(poise_pips(max, max, false), 5);
        assert_eq!(poise_pips(max * 2, max, false), 5, "a bonus over the max is still five");
        assert_eq!(poise_pips(-1, max, false), 0);
        assert_eq!(poise_pips(max, max, true), 0, "broken this round");
    }
    assert_eq!(poise_pips(0, 0, false), 0, "no poise at all");
    let r = Rect::new(10, 20, 76, 3);
    for k in 0..POISE_PIPS {
        let p = poise_pip_rect(r, k);
        assert!(p.w > 0 && p.h == r.h && p.y == r.y, "{p:?}");
        if k > 0 {
            assert_eq!(p.x, poise_pip_rect(r, k - 1).x + poise_pip_rect(r, k - 1).w + 1, "one unit apart");
        }
    }
    assert_eq!(poise_pip_rect(r, 0).x, r.x);
    assert_eq!(poise_pip_rect(r, 4).x + poise_pip_rect(r, 4).w, r.x + r.w, "the row is filled");
}

/// On the card: as many pips lit as the rule says, and drawing them changes no poise.
#[test]
fn the_card_draws_the_poise_pips() {
    let (mut cx, mut g, mut b) = arena();
    b.tick(&mut cx);
    let a = b.selected.unwrap();
    let st = emberwake_battle::poise::state(&s(&b).w, a).copied().expect("the timeline ruleset has poise");
    let value = st.max / 2;
    b.host.session.as_mut().unwrap().w.poise.get_mut(&a).unwrap().value = value;
    let (before, _) = (b.host.history.len(), ());
    draw(&mut cx, &mut g, &mut b);
    let rows = card_rows(widget_rect(&b, &format!("party.{a}")), 0.0);
    let lit = poise_pips(value, st.max, false);
    assert!((1..5).contains(&lit), "half the poise is some of the pips: {lit}");
    assert_eq!(card(&b, &cx, a)["poise_pips"], json!(lit));
    for k in 0..POISE_PIPS {
        let pip = poise_pip_rect(rows.poise, k);
        let on = px(&g, pip.x + pip.w / 2, pip.y + pip.h / 2)[..3] == POISE[..3];
        assert_eq!(on, k < lit, "pip {k}, {lit} lit");
    }
    assert_eq!(emberwake_battle::poise::state(&s(&b).w, a).unwrap().value, value, "display only");
    assert_eq!(b.host.history.len(), before, "no request");
}

// ---- The round banner ---------------------------------------------------------------------------------------------

/// The banner as a function of time: it comes up, holds, goes; without animation it holds as long and just goes.
#[test]
fn the_round_banner_is_a_pure_function_of_time() {
    let (fade_in, hold, fade_out) = ROUND_BANNER_MS;
    let total = fade_in + hold + fade_out;
    assert_eq!(round_banner_alpha(0, false), 0);
    assert_eq!(round_banner_alpha(fade_in, false), 255);
    assert_eq!(round_banner_alpha(fade_in + hold / 2, false), 255);
    let going = round_banner_alpha(fade_in + hold + fade_out / 2, false);
    assert!(going > 0 && going < 255, "{going}");
    assert_eq!(round_banner_alpha(total, false), 0);
    assert_eq!(round_banner_alpha(0, true), 255, "still: whole from the start");
    assert_eq!(round_banner_alpha(total - 1, true), 255);
    assert_eq!(round_banner_alpha(total, true), 0, "and gone as long after");
}

/// Each round's start shows its banner, then it goes; it takes no input (no widget of its own).
#[test]
fn a_new_round_shows_its_banner() {
    let (mut cx, mut g, mut b) = arena();
    b.tick(&mut cx);
    let total = ROUND_BANNER_MS.0 + ROUND_BANNER_MS.1 + ROUND_BANNER_MS.2;
    let board = Rect::new(GRID_X, GRID_Y, GRID_W * TILE, GRID_H * TILE);
    draw(&mut cx, &mut g, &mut b);
    b.clock.advance(ROUND_BANNER_MS.0 as f64, 1.0);
    draw(&mut cx, &mut g, &mut b);
    let widgets = b.w.len();
    assert_eq!(b.state(&cx)["round_banner"], json!({"round": 1, "alpha": 255}));
    let up = count(&g, board, th().gold);
    b.clock.advance(total as f64, 1.0);
    draw(&mut cx, &mut g, &mut b);
    assert_eq!(b.state(&cx)["round_banner"], Value::Null, "gone");
    assert_eq!(b.w.len(), widgets, "it adds no widget");
    assert!(count(&g, board, th().gold) < up, "its edge and name were drawn on the board");
    assert_eq!(b.send(&cx, json!({"cmd": "resolve"}))["ok"], json!(true));
    b.run(&mut cx, Action::NextRound);
    draw(&mut cx, &mut g, &mut b);
    b.clock.advance(ROUND_BANNER_MS.0 as f64, 1.0);
    draw(&mut cx, &mut g, &mut b);
    assert_eq!(b.state(&cx)["round_banner"], json!({"round": 2, "alpha": 255}));
}

// ---- The next round by itself -------------------------------------------------------------------------------------

/// The rule in full: only with a round to go; autoplay always; a locked round unless watching; else the setting, unless
/// watching a recording.
#[test]
fn the_next_round_rule() {
    for auto in [false, true] {
        for locked in [false, true] {
            for viewing in [false, true] {
                for setting in [false, true] {
                    let want = auto || (!viewing && (locked || setting));
                    assert_eq!(next_round_by_itself(true, auto, locked, viewing, setting), want, "auto {auto} locked {locked} viewing {viewing} setting {setting}");
                    assert!(!next_round_by_itself(false, auto, locked, viewing, setting), "no round to go");
                }
            }
        }
    }
    assert_eq!(crate::screens::default_settings()[AUTO_CONFIRM], json!(true), "on by default");
}

/// By default a played round goes on to the next round's planning by itself, with the ghosts drafted in the same tick; the
/// plan then waits for the player: however long, nothing confirms it.
#[test]
fn the_next_planning_starts_by_itself_and_waits_for_the_plan() {
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    assert!(b.pending_next_round, "due after the round-end pause");
    assert!(b.message.starts_with("Round over") && !b.message.contains("Space"), "no key to press: {}", b.message);
    tick_ready(&mut b, &mut cx);
    assert_eq!(s(&b).current_round, 2);
    assert!(b.planning());
    assert_eq!(b.ghosted, 2, "the planning start ran with the advance");
    assert_eq!(b.message, "Round 2: plan your party.");
    assert!(b.round_note.as_ref().is_some_and(|n| n.0 == b.message && n.1.starts_with("Round over")), "the summary stays readable: {:?}", b.round_note);
    let confirms = requests(&b, "confirm");
    for _ in 0..50 {
        tick_ready(&mut b, &mut cx);
    }
    assert!(b.planning() && s(&b).current_round == 2 && !b.playing, "the plan waits");
    assert_eq!(requests(&b, "confirm"), confirms, "nothing confirms it");
    assert_eq!(requests(&b, "advance"), 1);
}

/// Setting off: the round stays over until the player goes on.
#[test]
fn the_setting_off_waits_for_the_player() {
    let (mut cx, _, mut b) = arena_in("flat", |cx| cx.settings.v[AUTO_CONFIRM] = json!(false));
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    assert!(!b.pending_next_round);
    assert!(b.message.contains("Space"), "says how: {}", b.message);
    for _ in 0..20 {
        tick_ready(&mut b, &mut cx);
    }
    assert_eq!((s(&b).phase, s(&b).current_round), (Phase::Replay, 1));
    b.act(&mut cx, Action::NextRound);
    assert_eq!(s(&b).current_round, 2);
    assert!(b.planning());
}

/// Watching a recording: its rounds never go on by themselves, whatever the setting says.
#[test]
fn a_watched_recording_never_goes_on_by_itself() {
    let (mut cx, _, mut first) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    first.tick(&mut cx);
    let rec = first.recorded();
    let mut b = BattleScreen::playback(&cx, &rec, None);
    assert!(b.viewing, "{}", b.message);
    play_round(&mut b, &mut cx);
    assert!(!b.pending_next_round);
    for _ in 0..20 {
        tick_ready(&mut b, &mut cx);
    }
    assert_eq!((s(&b).phase, s(&b).current_round), (Phase::Replay, 1));
    assert_eq!(requests(&b, "advance"), 0);
}

/// The last round has no next: the round limit stands, nothing is pending.
#[test]
fn the_round_limit_does_not_go_on() {
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    b.host.session.as_mut().unwrap().def.max_rounds = 1;
    play_round(&mut b, &mut cx);
    assert!(!b.pending_next_round);
    tick_ready(&mut b, &mut cx);
    assert_eq!(requests(&b, "advance"), 0);
}

/// No input lost: a click right after the advance (the tick after has not run) keeps its target and its menu, as the planning
/// start ran with the advance and does not reset them.
#[test]
fn a_click_right_after_the_advance_is_kept() {
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    let h = b.selected.unwrap();
    play_round(&mut b, &mut cx);
    tick_ready(&mut b, &mut cx);
    assert!(b.planning() && s(&b).current_round == 2);
    let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.units[e].is_active() && s(&b).w.has(e)).expect("a foe on the field");
    b.selected = Some(h);
    b.act(&mut cx, Action::UnitClick(foe));
    assert!(b.player_target.contains(&h), "the click named the target");
    assert!(b.popup.is_some(), "its menu is open");
    tick_ready(&mut b, &mut cx);
    tick_ready(&mut b, &mut cx);
    assert!(b.player_target.contains(&h), "the target is still the player's");
    assert!(b.popup.is_some(), "the menu is still open");
}

/// Space (or Next round) during the pause goes on at once, once; the wait is over and the next round plays normally.
#[test]
fn going_on_during_the_pause_advances_once() {
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    assert!(b.pending_next_round);
    b.act(&mut cx, Action::Step);
    assert_eq!(s(&b).current_round, 2);
    assert!(!b.pending_next_round, "the wait is over");
    assert_eq!(requests(&b, "advance"), 1);
    tick_ready(&mut b, &mut cx);
    let message = b.message.clone();
    assert!(!message.contains("refused"), "{message}");
    assert_eq!(requests(&b, "advance"), 1);
    play_round(&mut b, &mut cx);
    assert_eq!(s(&b).current_round, 2, "round 2 played: the wait did not block it");
    assert!(b.pending_next_round);
}

/// A Space meant for the round's end that lands once the next round has come must not confirm its plan: the host's `step`
/// would; the plan keeps waiting for Confirm, and Enter still confirms it.
#[test]
fn a_late_space_does_not_confirm_the_next_plan() {
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    tick_ready(&mut b, &mut cx);
    assert!(b.planning() && s(&b).current_round == 2, "the next round came by itself");
    let (steps, confirms) = (requests(&b, "step"), requests(&b, "confirm"));
    let message = b.message.clone();
    b.key(&mut cx, Key::Space);
    assert!(b.planning() && !s(&b).plan_confirmed, "the plan still waits");
    assert_eq!((requests(&b, "step"), requests(&b, "confirm")), (steps, confirms), "no request");
    assert_eq!(b.message, message, "the round's summary stays");
    b.advanced_at = None;
    b.key(&mut cx, Key::Enter);
    assert_eq!(requests(&b, "confirm"), confirms + 1, "Enter is the plan's key");
}

/// The second Space, or the Next round click, right after a Space that played the round's last step (the screen's playback
/// still flagged on, its step not yet due): it advances, and nothing confirms the next plan.
#[test]
fn a_second_press_at_the_rounds_end_does_not_confirm_the_next_plan() {
    for by_click in [false, true] {
        let (mut cx, mut g, mut b) = arena();
        cx.settings.v["skip_anim"] = json!(true);
        b.tick(&mut cx);
        b.act(&mut cx, Action::Confirm);
        assert!(b.playing);
        for _ in 0..600 {
            if s(&b).phase == Phase::Replay {
                break;
            }
            b.key(&mut cx, Key::Space);
        }
        assert_eq!(s(&b).phase, Phase::Replay);
        assert!(b.playing, "the playback's flag is still on: no tick has run");
        draw(&mut cx, &mut g, &mut b);
        if by_click {
            b.click_id(&mut cx, "strip.confirm").expect("the Next round button");
        } else {
            b.key(&mut cx, Key::Space);
        }
        assert_eq!(s(&b).current_round, 2, "by click {by_click}");
        assert!(!b.playing, "the playback is over");
        let (steps, confirms) = (requests(&b, "step"), requests(&b, "confirm"));
        for _ in 0..5 {
            tick_ready(&mut b, &mut cx);
        }
        assert!(b.planning() && !s(&b).plan_confirmed, "the plan waits, by click {by_click}");
        assert_eq!((requests(&b, "step"), requests(&b, "confirm")), (steps, confirms), "no request confirms it");
    }
}

/// "Next round" and "Confirm round" are one button: for `CONFIRM_GUARD_MS` after an advance neither its click nor Enter takes
/// the new plan; after that both do. Autoplay's own Confirm does not wait.
#[test]
fn confirm_waits_just_after_an_advance() {
    for by_click in [false, true] {
        let (mut cx, mut g, mut b) = arena();
        cx.settings.v["skip_anim"] = json!(true);
        b.tick(&mut cx);
        play_round(&mut b, &mut cx);
        tick_ready(&mut b, &mut cx);
        assert!(b.planning() && s(&b).current_round == 2);
        draw(&mut cx, &mut g, &mut b);
        let confirms = requests(&b, "confirm");
        let press = |b: &mut BattleScreen, cx: &mut Cx| {
            if by_click {
                b.click_id(cx, "strip.confirm").expect("the Confirm button");
            } else {
                b.key(cx, Key::Enter);
            }
        };
        press(&mut b, &mut cx);
        assert_eq!(requests(&b, "confirm"), confirms, "by click {by_click}: too soon");
        assert!(b.planning());
        b.advanced_at = Some(Instant::now() - std::time::Duration::from_millis(CONFIRM_GUARD_MS + 1));
        press(&mut b, &mut cx);
        assert_eq!(requests(&b, "confirm"), confirms + 1, "by click {by_click}: the guard is over");
    }
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    let confirms = requests(&b, "confirm");
    b.auto = true;
    b.pending_next_round = false;
    b.run(&mut cx, Action::NextRound);
    assert_eq!(requests(&b, "confirm"), confirms + 1, "autoplay plans and confirms at once");
}

/// Classic keeps its old flow: no next round by itself, no ribbon, no guard, and Space in planning steps as it did.
#[test]
fn classic_keeps_its_old_flow() {
    let classic = || {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.settings.v["battle_view"] = json!("flat");
        cx.settings.v["skip_anim"] = json!(true);
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, None, "", false);
        assert!(!emberwake_battle::round_intent::on(&s(&b).w), "a catalog encounter plays classic");
        (cx, crate::hud::hud_tests::gfx(), b)
    };
    let (mut cx, mut g, mut b) = classic();
    cx.settings.v[AUTO_CONFIRM] = json!(true);
    b.tick(&mut cx);
    draw(&mut cx, &mut g, &mut b);
    assert_eq!(b.banner, None, "no ribbon");
    play_round(&mut b, &mut cx);
    assert!(!b.pending_next_round, "no next round by itself");
    assert!(b.message.contains("Space"), "says how: {}", b.message);
    for _ in 0..10 {
        tick_ready(&mut b, &mut cx);
    }
    assert_eq!((s(&b).phase, s(&b).current_round), (Phase::Replay, 1));
    b.key(&mut cx, Key::Space);
    assert_eq!(s(&b).current_round, 2, "Space goes on");
    assert_eq!(b.advanced_at, None, "no guard");
    draw(&mut cx, &mut g, &mut b);
    assert_eq!(b.banner, None);
    let steps = requests(&b, "step");
    b.key(&mut cx, Key::Space);
    assert_eq!(requests(&b, "step"), steps + 1, "Space steps from planning, as it did");

    let (mut cx, _, mut b) = classic();
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    b.act(&mut cx, Action::NextRound);
    let confirms = requests(&b, "confirm");
    b.key(&mut cx, Key::Enter);
    assert_eq!(requests(&b, "confirm"), confirms + 1, "Enter confirms at once");
}

/// The party cards at every text size: no two texts overlap, none passes the card or is cut, a long name included; the
/// Normal size keeps the card's height.
#[test]
fn the_card_numbers_fit_at_every_text_size() {
    for scale in [1.0, 1.15, 1.3] {
        let (mut cx, mut g, mut b) = arena();
        cx.settings.v["skip_anim"] = json!(true);
        b.tick(&mut cx);
        for a in s(&b).w.allies.clone() {
            b.host.session.as_mut().unwrap().w.units[a].name = "A Rather Long Party Member's Name".into();
        }
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
        let mut seen = 0;
        for a in s(&b).w.allies.clone() {
            let card = widget_rect(&b, &format!("party.{a}"));
            if scale == 1.0 {
                assert_eq!(card.h, 30, "Normal keeps its height");
            }
            let inside: Vec<_> = recs.iter().filter(|t| t.x >= card.x as f32 && t.x < (card.x + card.w) as f32 && t.y + t.h / 2.0 >= card.y as f32 && t.y + t.h / 2.0 < (card.y + card.h) as f32).cloned().collect();
            assert!(inside.len() >= 4, "{scale}: the name and three numbers: {}", inside.len());
            let v = aegis_ui::text_audit::issues(&inside, W as f32, H as f32, &[]);
            assert_eq!(v["counts"]["overlap"], json!(0), "{scale}: {v}");
            for t in &inside {
                assert!(t.y >= card.y as f32 && t.y + t.h <= (card.y + card.h) as f32, "{scale}: '{}' at {} to {} leaves {:?}", t.text, t.y, t.y + t.h, card);
                assert!(t.x + t.w <= (card.x + card.w) as f32, "{scale}: '{}' runs past the card", t.text);
                assert!(t.cut.is_none() || t.y < card.y as f32 + 8.0, "{scale}: only a name may be cut: {}", t.text);
                seen += 1;
            }
        }
        assert!(seen > 0);
    }
}

/// The next round's prompt stays whole however long the summary: the summary gives up its last clauses, all of them if it
/// must, and the message is not cut.
#[test]
fn the_prompt_survives_the_longest_summary() {
    let by_chars = |t: &str| t.len() as f32;
    let (summary, prompt) = ("Round over: 2 breaks (1 of 3 hits into a wind-up), 2 flanks, 6 follow-ups, Rest +1 AP.", "Round 2: plan your party.");
    assert_eq!(crate::battle::draw::summary_and_prompt(summary, prompt, 500.0, by_chars), format!("{summary} {prompt}"), "room for all");
    let some = crate::battle::draw::summary_and_prompt(summary, prompt, 90.0, by_chars);
    assert_eq!(some, format!("Round over: 2 breaks (1 of 3 hits into a wind-up), 2 flanks. {prompt}"), "whole clauses go");
    assert_eq!(crate::battle::draw::summary_and_prompt(summary, prompt, 30.0, by_chars), prompt, "only the prompt");

    let (mut cx, mut g, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    tick_ready(&mut b, &mut cx);
    let sum = emberwake_battle::round_intent::RoundSummary { breaks: 12, windup_breaks: 11, windup_hits: 13, flanks: 12, follow_ups: 16, rest_ap: 12 };
    let long = format!("Round over: {} and a little more, to pass the bar's width at the Normal text size, whatever else it holds.", emberwake_battle::round_intent::summary_text(&sum));
    let prompt = b.message.clone();
    b.round_note = Some((prompt.clone(), long));
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    let line = recs.iter().find(|t| t.text.ends_with(&prompt)).expect("the prompt is drawn whole");
    assert!(line.cut.is_none(), "not cut: {}", line.text);
    let status = b.widgets().into_iter().find(|w| w["id"] == "bar.status").unwrap();
    assert!(status["text"].as_str().unwrap().contains("Round over"), "the summary's first clause stays: {status}");
}

/// The open timeline view holds the round's end too; closing it lets the next round come.
#[test]
fn the_open_timeline_view_holds_the_next_round() {
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    b.act(&mut cx, Action::TimelineView);
    assert!(b.timeline_view_open());
    b.act(&mut cx, Action::Confirm);
    for _ in 0..800 {
        if s(&b).phase == Phase::Replay {
            break;
        }
        tick_ready(&mut b, &mut cx);
        if s(&b).phase != Phase::Replay && b.waiting.is_some() {
            b.act(&mut cx, Action::NextBlock);
        }
    }
    assert_eq!(s(&b).phase, Phase::Replay);
    assert!(b.pending_next_round);
    for _ in 0..5 {
        tick_ready(&mut b, &mut cx);
    }
    assert!(b.waiting.is_some() && s(&b).current_round == 1, "the view holds it");
    b.act(&mut cx, Action::TimelineView);
    tick_ready(&mut b, &mut cx);
    assert_eq!(s(&b).current_round, 2, "closed: it comes");
    assert!(b.planning());
}

/// A menu the player has open at the round's end stays open: the next round waits for it.
#[test]
fn an_open_menu_holds_the_next_round() {
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    b.tactic_menu(true);
    assert!(b.popup.is_some());
    for _ in 0..5 {
        tick_ready(&mut b, &mut cx);
    }
    assert_eq!((s(&b).phase, b.pending_next_round), (Phase::Replay, true));
    assert!(b.popup.is_some());
    b.act(&mut cx, Action::ClosePopup);
    tick_ready(&mut b, &mut cx);
    assert_eq!(s(&b).current_round, 2);
}

/// Block by block: the playback waits after the round's last block too; N goes on, and only then the next round comes.
#[test]
fn block_mode_holds_the_next_round_until_n() {
    let (mut cx, _, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    cx.settings.v["playback_speed"] = json!("block");
    b.tick(&mut cx);
    b.act(&mut cx, Action::Confirm);
    for _ in 0..800 {
        if s(&b).phase == Phase::Replay {
            break;
        }
        tick_ready(&mut b, &mut cx);
        if s(&b).phase != Phase::Replay && b.waiting.is_some() {
            b.act(&mut cx, Action::NextBlock);
        }
    }
    assert_eq!(s(&b).phase, Phase::Replay);
    for _ in 0..5 {
        tick_ready(&mut b, &mut cx);
    }
    assert!(b.waiting.is_some() && s(&b).current_round == 1, "it holds after the last block");
    b.act(&mut cx, Action::NextBlock);
    tick_ready(&mut b, &mut cx);
    assert_eq!(s(&b).current_round, 2);
    assert!(b.planning());
}

/// The Next round button stays live during the pause, so the wait can be cut short.
#[test]
fn the_next_round_button_works_during_the_pause() {
    let (mut cx, mut g, mut b) = arena();
    cx.settings.v["skip_anim"] = json!(true);
    b.tick(&mut cx);
    play_round(&mut b, &mut cx);
    assert!(b.pending_next_round);
    draw(&mut cx, &mut g, &mut b);
    assert!(b.w.iter().find(|x| x.id == "strip.confirm").unwrap().enabled);
}
