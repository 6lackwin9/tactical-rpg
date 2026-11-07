//! VISCLAR1-C (decision "Battle view answers, part 2"): plan strip blocks are one solid colour and a ghost fades with no
//! outline; a dash faces its target from its first frame in the side, up and down views; the ghost shows only while
//! planning; the skill menu's Fill does what key F does. Fixtures are the catalog's encounters; expectations come from the
//! rules, never shipped numbers.

use crate::battle::input::RowAct;
use crate::battle::tips::kind_color;
use crate::battle::tokens::Tokens;
use crate::battle::{Action, BattleScreen, DASH_MS};
use crate::screens::{Cx, Screen};
use aegis_core::assets::{Facing, View as Dir};
use aegis_ui::ui::Gfx;
use emberwake_battle::command::{CmdKind, MoveKind};
use emberwake_battle::grid::V2;
use emberwake_battle::round_intent::RoundTarget;
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use emberwake_rules::ruleset::TIMELINE;
use serde_json::{json, Value};

fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn foe_of(b: &BattleScreen) -> usize {
    s(b).w.enemies.iter().copied().find(|&e| s(b).w.units[e].is_active() && s(b).w.has(e)).expect("a foe on the field")
}

/// The colours on block `start`'s top row of pixels, between its rounded corners (the icon and name sit lower).
pub(super) fn top_row(g: &Gfx, b: &BattleScreen, start: i32) -> Vec<[u8; 3]> {
    let r = b.w.iter().find(|x| x.id == format!("strip.block{start}")).unwrap().rect;
    let y = g.view.y(r.y as f32);
    let (x0, x1) = (g.view.x(r.x as f32 + 3.0), g.view.x((r.x + r.w) as f32 - 3.0));
    let mut out: Vec<[u8; 3]> = (x0..x1).map(|x| {
        let i = ((y * g.canvas.w + x) * 4) as usize;
        [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2]]
    }).collect();
    out.dedup();
    out
}

/// A chosen span, a long one included, is its kind's colour and nothing else along its top: no impact bar, no edge. A
/// ghost span is one fainter colour with no dashes.
#[test]
fn blocks_are_one_solid_colour_and_a_ghost_only_fades() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    b.tick(&mut cx);
    let h = b.selected.unwrap();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let session = s(&b);
    let ghosts = rip::ghost_entries(&session.w, &session.plan, h);
    let spans: Vec<(i32, String)> = session.plan.entries_for(&session.w, h).iter().map(|e| (e.start_block, session.w.cat.skills[e.skill].kind.clone())).collect();
    let mut faded = 0;
    for (k, (start, kind)) in spans.iter().enumerate().filter(|(_, x)| x.1 != "REST") {
        assert!(ghosts[k], "nothing is picked yet");
        let row = top_row(&g, &b, *start);
        assert_eq!(row.len(), 1, "one colour along the ghost's top, no dashes: {row:?}");
        assert_ne!(row[0][..], kind_color(kind)[..3], "a ghost is fainter than its kind");
        faded += 1;
    }
    assert!(faded > 0, "the tactic's ghost has a span that is not Rest");
    let foe = foe_of(&b);
    assert_eq!(b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}))["ok"], json!(true));
    assert_eq!(b.send(&cx, json!({"cmd": "accept_ghost", "unit": h}))["ok"], json!(true));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let session = s(&b);
    let spans: Vec<(i32, i32, String)> = session.plan.entries_for(&session.w, h).iter().map(|e| (e.start_block, e.end_block(&session.w), session.w.cat.skills[e.skill].kind.clone())).collect();
    assert!(spans.iter().any(|x| x.1 > x.0 && x.2 != "REST"), "a span of several blocks: {spans:?}");
    for (start, _, kind) in spans.iter().filter(|x| x.2 != "REST") {
        assert_eq!(top_row(&g, &b, *start), [[kind_color(kind)[0], kind_color(kind)[1], kind_color(kind)[2]]], "{kind} at block {start}");
    }
}

/// A dash that runs across its target's line shows, from the slide's first moment, the facing it lands with; the side,
/// up and down views alike, with its action popping in the same step or later.
#[test]
fn a_dash_faces_its_target_as_it_starts() {
    let (mut cx, _, mut b) = arena(Some(TIMELINE));
    b.tick(&mut cx);
    assert_eq!(b.send(&cx, json!({"cmd": "confirm"}))["ok"], json!(true));
    let s = b.host.session.as_mut().unwrap();
    let (idx, actor, target) = s.resolver.scheduled.iter().enumerate().find_map(|(idx, a)| match a.command.kind {
        CmdKind::Attack { target: Some(t), .. } if !a.cancelled && t != a.unit && s.w.grid.pos(t).is_some() && s.w.grid.pos(a.unit).is_some() => Some((idx, a.unit, t)),
        _ => None,
    }).expect("an attack");
    let tp = V2::new(6, 4);
    s.w.grid.positions[target] = Some(tp);
    s.resolver.scheduled[idx].start = s.current_block;
    s.resolver.scheduled[idx].command.moved = Some(MoveKind::Dash);
    let walk = 300;
    // (where it lands, where it dashes from, the view it must show): each dash runs at right angles to its target
    let cases = [
        (tp + V2::new(-1, 0), tp + V2::new(-1, 2), Dir::Side),
        (tp + V2::new(1, 0), tp + V2::new(1, -2), Dir::Side),
        (tp + V2::new(0, 1), tp + V2::new(2, 1), Dir::Up),
        (tp + V2::new(0, -1), tp + V2::new(-2, -1), Dir::Down),
    ];
    for (land, from, view) in cases {
        for pops in [false, true] {
            s.last_popped = None;
            s.w.grid.positions[actor] = Some(from);
            let mut t = Tokens::default();
            t.catch_up(s, 0, false, walk, true);
            s.w.grid.positions[actor] = Some(land);
            if pops {
                s.last_popped = Some((s.current_round, s.current_block, idx));
            }
            t.catch_up(s, 1000, false, walk, true);
            let a = &t.anims[actor];
            assert_eq!(a.slide, Some((from, 1000)), "the dash runs");
            let first: Facing = a.faces(1000, walk);
            assert_eq!(first.view, view, "from {from} to {land}, target on {tp}, pops {pops}");
            assert_eq!(first, a.faces(1000 + DASH_MS, walk), "no turn when it lands: from {from} to {land}, pops {pops}");
        }
    }
    // a plain walk still faces its step
    s.resolver.scheduled[idx].command.moved = None;
    s.last_popped = None;
    let (from, land) = (tp + V2::new(-1, 2), tp + V2::new(-1, 1));
    s.w.grid.positions[actor] = Some(from);
    let mut t = Tokens::default();
    t.catch_up(s, 0, false, walk, true);
    s.w.grid.positions[actor] = Some(land);
    t.catch_up(s, 1000, false, walk, true);
    assert_eq!(t.anims[actor].faces(1000, walk).view, Dir::Up, "the walk goes up");
}

fn strip_looks(b: &BattleScreen, cx: &Cx) -> Vec<Value> {
    b.state(cx)["strip_labels"].as_array().cloned().unwrap_or_default()
}

/// The strip's ghost look lasts while the party plans; a round being played, and the round over, show none, on any step.
#[test]
fn the_ghost_shows_only_in_planning() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    b.tick(&mut cx);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(strip_looks(&b, &cx).iter().any(|l| l["look"] == json!("ghost")), "planning shows it");
    assert_eq!(b.send(&cx, json!({"cmd": "confirm"}))["ok"], json!(true));
    let mut steps = 0;
    loop {
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(strip_looks(&b, &cx).iter().all(|l| l["look"] != json!("ghost")), "step {steps}");
        let session = b.host.session.as_mut().unwrap();
        if session.phase == Phase::Replay || steps > 400 {
            break;
        }
        session.step_next();
        steps += 1;
    }
    assert!(steps > 0 && s(&b).phase == Phase::Replay, "the round played to its end");
}

/// Fill on the skill menu leaves the plan key F leaves, and closes the menu; a round with no ghost left has no Fill.
#[test]
fn the_menu_fill_does_what_f_does() {
    let open = |cx: &mut Cx, b: &mut BattleScreen| {
        b.tick(cx);
        let foe = foe_of(b);
        b.act(cx, Action::UnitClick(foe));
    };
    let plan_of = |b: &BattleScreen, h: usize| crate::battle::input::plan_rows(&s(b).w, &s(b).plan, h);
    let (mut cx, _, mut by_row) = arena(Some(TIMELINE));
    let h = { by_row.tick(&mut cx); by_row.selected.unwrap() };
    open(&mut cx, &mut by_row);
    let rows = by_row.popup.as_ref().expect("the skill menu").rows.clone();
    let k = rows.iter().position(|r| matches!(r.act, RowAct::Fill)).expect("a Fill row while a ghost shows");
    assert!(rows[k].valid);
    // VISCLAR1-D: the tip's wording, as "contains F" passes on the word Fill alone
    let tip: Vec<&str> = rows[k].tip.iter().map(|t| t.0.as_str()).collect();
    assert_eq!(tip, ["Fill (F)", "Accepts the auto-fill for the rest of the round."], "it names the key and what it takes");
    by_row.act(&mut cx, Action::Choose(k));
    assert!(by_row.popup.is_none(), "Fill closes the menu");
    assert!(!rip::picks_of(&s(&by_row).w, h).is_empty(), "the ghost became picks");

    let (mut cx2, _, mut by_key) = arena(Some(TIMELINE));
    by_key.tick(&mut cx2);
    let foe = foe_of(&by_key);
    // the same click, so the same target and ghost, then key F's action
    by_key.act(&mut cx2, Action::UnitClick(foe));
    by_key.act(&mut cx2, Action::FillGhost);
    assert_eq!(plan_of(&by_row, h), plan_of(&by_key, h), "the row and key F agree");

    // no ghost left: no Fill. VISCLAR1-D: the rows themselves, as a menu with no row at all does not open and passed any code
    let foe = foe_of(&by_row);
    let (_, after) = by_row.menu_rows(h, RoundTarget::Unit(foe));
    assert!(!after.iter().any(|r| r.valid && matches!(r.act, RowAct::Fill)), "the round is full of picks: nothing to fill");
    // the same with rows to pick from: Auto-fill off, a hero with no plan yet has skills to pick and no ghost to fill
    let (mut cx3, _, mut bare) = arena(Some(TIMELINE));
    cx3.settings.v["auto_fill"] = json!(false);
    bare.tick(&mut cx3);
    let (h3, foe3) = (bare.selected.unwrap(), foe_of(&bare));
    assert!(!rip::ghost_entries(&s(&bare).w, &s(&bare).plan, h3).contains(&true), "no ghost yet");
    let (_, rows3) = bare.menu_rows(h3, RoundTarget::Unit(foe3));
    assert!(rows3.iter().any(|r| matches!(r.act, RowAct::Pick(_))), "skills to pick");
    assert!(!rows3.iter().any(|r| r.valid && matches!(r.act, RowAct::Fill)), "no ghost: Fill stays in its place, faded");
}

/// VISCLAR1-D (carried from VISCLAR1-C): Fill with Auto-fill off: after a pick the free blocks are the planner's Rest, the row
/// offers them, and taking it leaves what key F leaves: those Rests as picks, no follow-up skills, no ghost left.
#[test]
fn fill_with_auto_fill_off_takes_the_rest_blocks() {
    let planned = |b: &BattleScreen, h: usize| -> Vec<(i32, String, bool)> {
        let session = s(b);
        let ghosts = rip::ghost_entries(&session.w, &session.plan, h);
        session.plan.entries_for(&session.w, h).iter().zip(ghosts).map(|(e, g)| (e.start_block, session.w.cat.skills[e.skill].kind.clone(), g)).collect()
    };
    // one screen to the point the row or the key acts on: the hero's first valid skill on the first foe
    let start = || {
        let (mut cx, _, mut b) = arena(Some(TIMELINE));
        cx.settings.v["auto_fill"] = json!(false);
        b.tick(&mut cx);
        let (h, foe) = (b.selected.unwrap(), foe_of(&b));
        b.act(&mut cx, Action::UnitClick(foe));
        let pick = b.popup.as_ref().and_then(|p| p.rows.iter().position(|r| r.valid && matches!(r.act, RowAct::Pick(_)))).expect("a skill to pick");
        b.act(&mut cx, Action::Choose(pick));
        (cx, b, h)
    };
    let (mut cx, mut by_row, h) = start();
    let before = planned(&by_row, h);
    let free: Vec<_> = before.iter().filter(|e| e.2).collect();
    assert!(!free.is_empty() && free.iter().all(|e| e.1 == "REST"), "Auto-fill off: the free blocks Rest: {before:?}");
    let rows = by_row.popup.as_ref().expect("the menu stays open after the pick").rows.clone();
    let k = rows.iter().position(|r| matches!(r.act, RowAct::Fill)).expect("a Fill row for the Rest blocks");
    by_row.act(&mut cx, Action::Choose(k));
    assert!(by_row.popup.is_none(), "Fill closes the menu");
    let after = planned(&by_row, h);
    assert_eq!(after.len(), before.len(), "no block changes");
    for (a, b) in after.iter().zip(&before) {
        assert_eq!((a.0, &a.1), (b.0, &b.1), "the same blocks and kinds: no follow-up skill appears");
    }
    assert!(after.iter().all(|e| !e.2), "no ghost left: {after:?}");

    let (mut cx2, mut by_key, h2) = start();
    by_key.act(&mut cx2, Action::FillGhost);
    assert_eq!(planned(&by_key, h2), after, "the row and key F agree");
}
