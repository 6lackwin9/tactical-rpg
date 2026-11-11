//! RULESET1-U slice 5 (RULESET1 5, 12; RULESET1_INTENT UI2, MV8; the slice 4 review; preference "Battle overlays and
//! the clarity pass"): poise bars and BREAK, the strip's wind-up / impact / recovery, the hit forecasts, the round
//! summary, the overlay groups' flags, the revealed moves shown for the hovered enemy or with Alt, and the reach band of
//! a real Disengage plan. Expectations are kit queries (`poise::forecast`, `round_intent::round_summary`,
//! `skill::attack_band`, the plan), never shipped numbers; the Disengage plan runs on a scratch catalog built here.

use crate::battle::board::cell_rect;
use crate::battle::input::pick_detail;
use crate::battle::marks::{broken_units, hit_forecasts, round_summary_text, HitForecast};
use crate::battle::overlays::{overlay_of, strike_tile, Layers, MoveKind, OVERLAY_GROUPS};
use crate::battle::{BattleScreen, Mode, TILE};
use crate::battle::tips::{intent_glyph, unit_tip};
use aegis_ui::canvas::Rect;
use crate::intent_text;
use crate::screens::{Cx, Screen};
use aegis_ui::painter::Painter;
use aegis_ui::ui::Gfx;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::round_intent::{self as ri, ManeuverKind, RoundIntent, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use aegis_ui::art::Art;
use emberwake_battle::skill::Kind;
use serde_json::{json, Value};
use emberwake_rules::ruleset::{Ruleset, AP_COST, FLANK, POISE, ROUND_INTENT, TIMELINE};

fn arena(reveal: bool) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.into()), "", reveal);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn assault(cx: &Cx, b: &mut BattleScreen) -> (usize, usize) {
    let h = b.selected.unwrap();
    let foe = s(b).w.enemies.iter().copied().find(|&e| s(b).w.units[e].is_active() && s(b).w.has(e)).unwrap();
    let r = b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
    assert_eq!(r["ok"], json!(true), "{r}");
    (h, foe)
}

/// The preference (user 2026-10-30): every group is at its default (on, but VISCLAR1's `all_bars` and the
/// testing groups); the settings key switches one off and the board stops drawing it; the state lists the flags.
#[test]
fn each_overlay_group_has_its_flag() {
    let all = Layers::of(&Value::Null);
    assert_eq!(all.json().as_object().unwrap().len(), Layers::IDS.len());
    assert!(OVERLAY_GROUPS.iter().all(|g| all.json()[g.0] == json!(g.3)));
    let off = Layers::of(&json!({"band": false, "walks": false}));
    assert!(!off.band && !off.walks && off.front && off.poise);
    let (mut cx, mut g, mut b) = arena(false);
    assault(&cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let o = overlay_of(s(&b), b.selected, true).unwrap();
    assert!(!o.band.is_empty() && o.shown(&off, None, None, true).band.is_empty() && o.shown(&all, None, None, true).band == o.band);
    cx.settings.v["battle_overlays"] = json!({"band": false});
    assert_eq!(b.state(&cx)["overlay_layers"]["band"], json!(false));
}

/// The scratch fight of MV8: a dancer next to a footman, Maneuver Disengage with AP for one Slip Strike, which then
/// carries the retreat after its hit.
fn disengage_session() -> (Session, usize, usize) {
    scratch_session(true)
}

/// The scratch fight: Maneuver Disengage (`disengage`) or Assault (with AP for two Cleaves) on the footman.
fn scratch_session(disengage: bool) -> (Session, usize, usize) {
    scratch_session_with(disengage, json!([]))
}

/// `scratch_session` with the catalog's `terms` rows (TERMS1 5.1).
fn scratch_session_with(disengage: bool, terms: Value) -> (Session, usize, usize) {
    let atk = |id: &str, intent: &str, blocks: i32, ap: i32, dash: i32| json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": intent, "level": 1, "blocks": blocks, "ap_cost": ap, "power_mult": 0.8, "dash_tiles": dash, "target_mode": "ENEMY"});
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true}),
        atk("g_slip", "maneuver", 2, 2, 2),
        json!({"id": "g_cut", "name": "Cleave", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 2, "ap_cost": 2, "power_mult": 0.5, "stagger": 12, "target_mode": "ENEMY"}),
        json!({"id": "g_heavy", "name": "Heavy", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 4, "ap_cost": 1, "power_mult": 0.5, "target_mode": "ENEMY"}),
    ];
    let unit = |id: &str, side: &str, order: Value| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5, "poise": 40, "deck": {"order": order}});
    let cat = emberwake_data::Catalog::from_value(&json!({"skills": skills, "units": [unit("g_dancer", "party", json!({"maneuver": ["g_slip"], "assault": ["g_cut"]})), unit("g_footman", "enemy", json!({"assault": ["g_heavy"]}))], "terms": terms}));
    let rules = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, FLANK, POISE], "basics": ["move", "rest", "strike"]}).as_object().unwrap());
    let row = json!({"id": "g_fight", "name": "Fight", "party": ["g_dancer"], "enemies": ["g_footman"], "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8});
    let mut def = emberwake_battle::session::EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules;
    let mut sess = Session::start(std::sync::Arc::new(cat), def, &indexmap::IndexMap::new());
    let (d, f) = (0, 1);
    sess.w.grid.positions[d] = Some(V2::new(4, 4));
    sess.w.grid.positions[f] = Some(V2::new(5, 4));
    sess.w.units[d].ap = if disengage { 2 } else { 4 };
    ri::set_intent(&mut sess.w, d, if disengage { RoundIntent::Maneuver } else { RoundIntent::Assault }).unwrap();
    ri::set_target(&mut sess.w, d, RoundTarget::Unit(f)).unwrap();
    if disengage {
        ri::set_maneuver(&mut sess.w, d, ManeuverKind::Disengage).unwrap();
    }
    let (w, plan) = (&mut sess.w, &mut sess.plan);
    rip::plan_unit(w, plan, d).unwrap_or_else(|e| panic!("the round: {e}"));
    (sess, d, f)
}

/// TERMS1 TU5: the battle screen's intent names read the game's `terms` table through the battle's catalog (the
/// round head of a record, a pick's detail, the glyph's initial, the kit's intent texts); with no row the defaults.
#[test]
fn intent_names_read_the_games_terms() {
    let renamed = json!([{"id": "intent_assault", "name": "Onslaught", "origin": "test"}, {"id": "intent_maneuver", "name": "Shift", "origin": "test"}]);
    for (terms, assault, maneuver) in [(json!([]), "Assault", "Maneuver"), (renamed, "Onslaught", "Shift")] {
        let (sess, d, f) = scratch_session_with(false, terms);
        let w = &sess.w;
        assert_eq!(RoundIntent::Assault.label(&w.cat), assault);
        assert_eq!(emberwake_world::party_ops::intent_names(&w.cat).get("assault").map(String::as_str), Some(assault), "the deck screens' names (party, Dojo)");
        let line = emberwake_battle::intent::Line { kind: "", target: None, cell: V2::new(0, 0), blocks: (0, 0), recovery: 0, skill: String::new(), channeled: false };
        let rec = emberwake_battle::intent::Intent { unit: d, line, revealed: false, enraged: false, readers: Vec::new(), plan: Vec::new(), morale_state: "", morale_group: -1,
            round_intent: Some(RoundIntent::Assault), channel: false, row_intent: Some(RoundIntent::Assault), round_target: Some(RoundTarget::Unit(f)), maneuver: None, impacts: Vec::new() };
        assert_eq!(intent_text::round_head(w, &rec), Some(format!("{assault}: {}", w.units[f].name)));
        let opt = |i: RoundIntent, kind: Option<ManeuverKind>| rip::PickOption { picks: Vec::new(), intent: Some(i), kind, ap: 2, blocks: 2, problem: None, reasons: Vec::new(), ap_short: None, drops: Vec::new(), at: RoundTarget::Myself };
        assert_eq!(pick_detail(&w.cat, &opt(RoundIntent::Assault, None)), format!("2 blocks, 2 AP; {assault}"));
        assert_eq!(pick_detail(&w.cat, &opt(RoundIntent::Maneuver, Some(ManeuverKind::Engage))), format!("2 blocks, 2 AP; {maneuver} (Engage)"));
        assert_eq!(ri::target_problem(w, d, RoundIntent::Assault, RoundTarget::Myself), format!("{assault} targets an enemy or a tile."), "the kit's intent text");
        let mut g = crate::hud::hud_tests::gfx();
        g.fonts.audit = Some(Vec::new());
        let view = g.view;
        intent_glyph(&mut Painter { c: &mut g.canvas, v: view, fonts: &mut g.fonts }, &w.cat, RoundIntent::Assault, Rect::new(0, 0, 10, 10));
        let letters: Vec<String> = g.fonts.audit.take().unwrap_or_default().into_iter().map(|r| r.text).collect();
        assert_eq!(letters, [assault[..1].to_string()], "the glyph's initial");
    }
}

/// The slice 3 review on a real plan: under a Disengage round the reach band centres on the tile the Slip Strike
/// hits from (before its retreat), and the retreat is drawn as an after-move.
#[test]
fn a_disengage_retreat_keeps_the_band_on_the_strike_tile() {
    let (sess, d, _) = disengage_session();
    let w = &sess.w;
    let st = sess.plan.compile(w, d);
    let first = st.entries.iter().find(|e| emberwake_battle::skill::kind(&w.cat.skills[e.skill]) == Kind::Attack).expect("the Slip Strike");
    assert!(matches!(first.carry, emberwake_battle::command::Carry::Retreat(_)) && first.pos_before != first.pos_after, "it carries the retreat: {:?}", first.carry);
    let o = overlay_of(&sess, Some(d), true).unwrap();
    let (lo, hi) = emberwake_battle::skill::attack_band(&w.units[d], &w.ruleset);
    let around = |at: V2| (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && (lo..=hi).contains(&c.dist(at))).collect::<Vec<_>>();
    assert_eq!(o.band, around(first.pos_before), "the strike tile");
    assert_ne!(o.band, around(first.pos_after), "not the retreat tile");
    assert!(o.moves.iter().any(|(t, k)| *k == MoveKind::After && t.last() == Some(&first.pos_after)), "{:?}", o.moves);
}

/// RULESET1 5, 12: under the rule poise the hover card tells the bar; a planned hit's strip tip carries the kit's
/// forecast of it (`poise::forecast` on the target's bar now, flank from its strike tile); a unit broken this round
/// is listed for its BREAK plate.
#[test]
fn poise_breaks_parts_and_forecasts_follow_the_kit() {
    let (mut cx, mut g, mut b) = arena(false);
    assert!(s(&b).w.ruleset.has(POISE), "the timeline row plays poise");
    let (h, foe) = assault(&cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    let bar = *emberwake_battle::poise::state(w, foe).unwrap();
    let tip = unit_tip(s(&b), foe, Some(h), &[], &[], &[]);
    // S2.2-S3: in pips under poise_mode pips
    let n = &w.ruleset.numbers;
    let want = if emberwake_battle::poise::pips_mode(n) { format!("Poise pips {}/{}", bar.pips, n.poise_pips) } else { format!("Poise {}/{}", bar.value, bar.max) };
    assert!(tip.iter().any(|t| t.0.starts_with(&want)), "{tip:?}");
    assert!(broken_units(w, &Layers::of(&Value::Null)).is_empty(), "nobody broke yet");
    b.host.session.as_mut().unwrap().w.poise.get_mut(&foe).unwrap().broken = true;
    assert_eq!(broken_units(&s(&b).w, &Layers::of(&Value::Null)), vec![foe]);
    assert!(broken_units(&s(&b).w, &Layers::of(&json!({"breaks": false}))).is_empty(), "behind its flag");
    assert!(unit_tip(s(&b), foe, Some(h), &[], &[], &[]).iter().any(|t| t.0.contains("BROKEN")));
}

/// RULESET1 12 (forecasts) on the scratch fight: two planned Cleaves (stagger 12) into the footman's wind-up: each
/// is the kit's `poise::forecast` on the bar the earlier one left (the first on the bar as it stands); a hit with
/// nothing to say (no stagger, flank or pierce) has none; the strip block's tip and the intel panel carry them.
#[test]
fn forecasts_follow_the_kit_hit_by_hit() {
    let (mut sess, d, f) = scratch_session(false);
    // the footman winds up a 4-block Heavy from block 1
    let mut fp = emberwake_battle::plan::Plan::new();
    assert_eq!(fp.add_step(&sess.w, f, emberwake_battle::plan::PlanStep::engage(sess.w.book().find("g_heavy").unwrap(), d), false), "");
    sess.resolver.scheduled.clear();
    fp.commit(&sess.w, &mut sess.resolver);
    let w = &sess.w;
    let st = sess.plan.compile(w, d);
    let fcs = hit_forecasts(&sess, d);
    assert_eq!(fcs.len(), st.entries.len());
    let hits: Vec<(usize, &HitForecast)> = fcs.iter().enumerate().filter_map(|(k, x)| x.as_ref().map(|x| (k, x))).collect();
    assert!(hits.len() >= 2, "two Cleaves: {}", hits.len());
    let mut bar = *emberwake_battle::poise::state(w, f).unwrap();
    for (k, h) in &hits {
        let e = &st.entries[*k];
        let cmd = e.build_command(w, d);
        let want = emberwake_battle::poise::forecast(w, &sess.resolver.scheduled, f, bar, e.end_block(w), &cmd.impact, cmd.block_cost, ri::flank_bonus_from(w, d, strike_tile(e), f), cmd.ignore_guard);
        assert_eq!(h.fc, want, "entry {k}");
        bar.value = want.after;
        if emberwake_battle::poise::pips_mode(&w.ruleset.numbers) {
            bar.pips = want.pips_after;
        }
        if matches!(want.verdict, emberwake_battle::poise::Verdict::Breaks(_)) {
            (bar.value, bar.pips, bar.broken) = (0, 0, true);
        }
    }
    assert!(hits[0].1.fc.windup, "the first lands in the footman's wind-up");
    // on the screen: the strip tip and the intel panel
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let mut g = crate::hud::hud_tests::gfx();
    let mut b = BattleScreen::blank(&cx, Mode::Arena);
    b.host.begin(sess);
    b.after_start("");
    b.selected = Some(d);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let e0 = &b.host.session.as_ref().unwrap().plan.compile(&b.host.session.as_ref().unwrap().w, d).entries[hits[0].0].clone();
    let block = b.w.iter().find(|x| x.id == format!("strip.block{}", e0.start_block)).unwrap();
    assert!(block.tip.iter().any(|t| t.0 == "Forecast on g_footman"), "{:?}", block.tip);
    assert!(block.tip.iter().any(|t| t.0.starts_with("Stagger 12 x in wind-up")), "{:?}", block.tip);
}

/// R1-BAL (FB-88): a break forecast (breaks or may break) under `poise_mode` pips adds the kit's
/// `poise::break_stagger_note` ("queues Stagger next round (locks B1, B4)") after its verdict; a hit that holds, a
/// `steadfast` target and `bar` add none.
#[test]
fn a_break_forecast_under_pips_names_the_queued_stagger() {
    use emberwake_battle::poise::{self, BreakCause, Verdict};
    let (mut sess, d, _) = scratch_session(false);
    let mut fcs = hit_forecasts(&sess, d);
    let h = fcs.iter_mut().flatten().find(|h| h.lines(&sess.w).len() > 2).expect("a planned hit with a verdict");
    sess.w.ruleset.numbers.poise_mode = emberwake_rules::ruleset::PoiseMode::Pips;
    let note = poise::break_stagger_note(&sess.w.ruleset.numbers, &sess.w.units[h.target]).expect("pips: a note");
    assert!(note.starts_with("queues Stagger next round"), "{note}");
    let shows = |s: &Session, h: &HitForecast| {
        let lines = h.lines(&s.w);
        let at = lines.iter().position(|l| l.0 == note);
        // right after the verdict line
        at.is_some_and(|k| k >= 2 && (lines[k - 1].0.starts_with("Breaks") || lines[k - 1].0.starts_with("May break")))
    };
    for v in [Verdict::Breaks(BreakCause::Poise), Verdict::Chance(40)] {
        h.fc.verdict = v;
        assert!(shows(&sess, h), "{v:?}: {:?}", h.lines(&sess.w));
    }
    h.fc.verdict = Verdict::Holds;
    assert!(!h.lines(&sess.w).iter().any(|l| l.0 == note), "a hit that holds");
    h.fc.verdict = Verdict::Breaks(BreakCause::Poise);
    let mut steady = sess.w.clone();
    emberwake_rules::status::grant_now(&mut steady.units[h.target], poise::STEADFAST, 1, 0, "test");
    assert!(!h.lines(&steady).iter().any(|l| l.0 == note), "steadfast");
    sess.w.ruleset.numbers.poise_mode = emberwake_rules::ruleset::PoiseMode::Bar;
    assert!(!h.lines(&sess.w).iter().any(|l| l.0 == note), "bar");
}

/// RULESET1 12: once the round is over, the state's summary is the kit's `round_summary` and the message leads
/// with it; classic has neither.
#[test]
fn the_round_summary_reads_the_kit() {
    let (mut cx, mut g, mut b) = arena(false);
    assault(&cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let r = b.send(&cx, json!({"cmd": "resolve"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    let sum = ri::round_summary(&s(&b).w, &s(&b).resolver.scheduled);
    let st = b.state(&cx);
    assert_eq!(st["round_summary"]["breaks"], json!(sum.breaks));
    assert_eq!(st["round_summary"]["follow_ups"], json!(sum.follow_ups));
    let text = round_summary_text(s(&b), &Layers::of(&Value::Null)).unwrap();
    assert!(text.starts_with(if sum.breaks == 0 { "no breaks" } else { "1 break" }) || sum.breaks > 1, "{text}");
    assert!(round_summary_text(s(&b), &Layers::of(&json!({"summary": false}))).is_none());
}

/// The slice 4 review: during the playback a hostile card's glyph and its token's agree (both from the round
/// records): the intent's initial is drawn in the card's glyph square for every enemy with an intent.
#[test]
fn card_and_token_glyphs_agree_in_playback() {
    let (mut cx, mut g, mut b) = arena(false);
    assault(&cx, &mut b);
    let r = b.send(&cx, json!({"cmd": "confirm"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    // VISCLAR1-D: the token's plate shows while Alt is held (the ring tells the intent otherwise)
    cx.held.alt = true;
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    let rounds = s(&b).round_intents();
    let mut checked = 0;
    for r in rounds.iter().filter(|r| s(&b).w.enemies.contains(&r.unit)) {
        let Some(i) = r.round_intent else { continue };
        let card = b.w.iter().find(|x| x.id == format!("foe.{}", r.unit)).unwrap().rect;
        let letter = &i.name()[..1];
        let in_card = recs.iter().any(|t| t.text == letter && t.x >= card.x as f32 && t.x <= (card.x + 13) as f32 && t.y >= card.y as f32 && t.y <= (card.y + 12) as f32);
        let at = s(&b).w.pos(r.unit);
        let tile = cell_rect(at);
        let on_token = recs.iter().any(|t| t.text == letter && t.x >= (tile.x + TILE - 12) as f32 && t.x <= (tile.x + TILE) as f32 && t.y >= tile.y as f32 && t.y <= (tile.y + 12) as f32);
        assert!(in_card && on_token, "{}: card {in_card}, token {on_token}", s(&b).w.units[r.unit].name);
        checked += 1;
    }
    assert!(checked > 0, "the enemies hold intents in the playback");
}
