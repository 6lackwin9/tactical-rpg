//! INPUT1-U slice 1 on a scratch catalog built here (no shipped data): the picks model of `round_intent_plan` under the
//! rule `round_intent` (with `ap_cost`, so skills cost their AP). Expectations come from the rules of INPUT1.md 2-5 and
//! 7: the menu rows by target kind before and after the first pick (IF2), the refusals and their words (IF6 a-e), the
//! intent and kind from the first pick (IF4), the ghost after the picks (GH1, GH3), Auto-fill (GH7), the step after
//! (GH4, ET4), atomic picks and a later member's dropped pick (GH5), the round start (GH6), the hotkey order (HK1),
//! Move here and the one regular move (ET2, ET3, ET5), the AP forecast on section 4's worked examples (AP1) and the
//! read-only preview (IF5).

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::Plan;
use rpgkit_battle::round_intent::{self, ManeuverKind, RoundIntent, RoundState, RoundTarget};
use rpgkit_battle::round_intent_plan::{self as rip, Pick, RegularMove};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, AP_COST, FLANK, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;
/// Section 4's worked examples: `ap_regen` 4.
const AP_REGEN: i32 = 4;
const BASICS: [&str; 5] = ["move", "rest", "strike", "active_guard", "flank_cut"];

/// Engine skills and the test kit's skills (RULESET1_TASKS 2.4 numbers; Flank Cut as INPUT1-MA writes it).
fn catalog() -> Catalog {
    let atk = |id: &str, intent: &str, blocks: i32, rec: i32, ap: i32, power: f64| {
        json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": intent, "level": 1, "blocks": blocks, "recovery_blocks": rec, "ap_cost": ap, "power_mult": power, "target_mode": "ENEMY"})
    };
    let mut flank_cut = atk("flank_cut", "maneuver", 1, 1, 2, 0.6);
    flank_cut["position"] = json!("flank");
    flank_cut["reach_max"] = json!(2);
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
        flank_cut,
        json!({"id": "tl_brace_counter", "name": "Brace and Counter", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 2, "mitigation": 0.3, "grant_status": "riposte", "status_rounds": 1}),
        json!({"id": "tl_field_dressing", "name": "Field Dressing", "kind": "SUPPORT", "round_intent": "support", "level": 1, "blocks": 2, "ap_cost": 2, "heal_base": 8, "heal_mult": 0.3, "target_mode": "ALLY_OR_SELF"}),
        atk("tl_heavy_cleave", "assault", 4, 1, 4, 2.0),
        atk("tl_shoulder_check", "assault", 1, 0, 1, 0.5),
        atk("tl_snap_shot", "assault", 1, 1, 2, 0.6),
        atk("tl_aimed_shot", "assault", 2, 1, 2, 1.1),
    ];
    let unit = |id: &str, side: &str, range: i32, deck: serde_json::Value| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": range, "skill_cap": 5, "deck": deck})
    };
    let units = vec![
        unit("tl_vanguard", "party", 1, json!({"order": {"assault": ["tl_heavy_cleave", "tl_shoulder_check", "strike"], "guard": ["tl_brace_counter", "active_guard"]}})),
        unit("tl_archer", "party", 3, json!({"order": {"assault": ["tl_aimed_shot", "tl_snap_shot"], "support": ["tl_field_dressing"]}})),
        unit("tl_spiker", "party", 1, json!({"order": {"assault": ["strike"], "maneuver": ["flank_cut"]}})),
        unit("t_decked", "party", 1, json!({"active": ["tl_shoulder_check", "tl_heavy_cleave", "tl_brace_counter"]})),
        unit("tl_footman", "enemy", 1, json!({})),
        unit("tl_footman_b", "enemy", 1, json!({})),
    ];
    Catalog::from_value(&json!({"skills": skills, "units": units}))
}

const VANGUARD: UnitId = 0;
const ARCHER: UnitId = 1;
const SPIKER: UnitId = 2;
const DECKED: UnitId = 3;
const FOOTMAN: UnitId = 4;
const FOOTMAN_B: UnitId = 5;

fn timeline() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, FLANK], "basics": BASICS}).as_object().unwrap())
}

/// Every unit parked in a corner or along the edges (tests place the ones they use); the enemies' round-1 plans are
/// dropped so only what a test plans acts.
fn session() -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["tl_vanguard", "tl_archer", "tl_spiker", "t_decked"],
        "enemies": ["tl_footman", "tl_footman_b"], "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": AP_REGEN, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = timeline();
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    let parked = [V2::new(0, 0), V2::new(0, 8), V2::new(2, 0), V2::new(2, 8), V2::new(11, 0), V2::new(11, 8)];
    for (c, cell) in parked.into_iter().enumerate() {
        place(&mut s.w, c, cell);
    }
    drop_enemy_plans(&mut s);
    s
}

fn drop_enemy_plans(s: &mut Session) {
    let enemies = s.w.enemies.clone();
    s.resolver.scheduled.retain(|a| !enemies.contains(&a.unit));
}

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn sk(s: &Session, id: &str) -> Pick {
    Pick::Skill { skill: sid(&s.w, id), kind: None }
}

fn ap_of(w: &World, c: UnitId, id: &str) -> i32 {
    skill::ap_cost_by(&w.cat.skills[sid(w, id)], Some(&w.units[c]), &w.ruleset)
}

fn need_of(w: &World, c: UnitId, id: &str) -> i32 {
    let s = &w.cat.skills[sid(w, id)];
    s.blocks + w.recovery_of(c, s)
}

fn pick(s: &mut Session, c: UnitId, at: RoundTarget, picks: &[Pick]) -> Result<(), String> {
    s.pick(c, at, picks).0
}

fn foe(t: UnitId) -> RoundTarget {
    RoundTarget::Unit(t)
}

/// (skill id, start, end, AP, tile after, reason, ghost) per entry of `c`'s plan.
type Row = (String, i32, i32, i32, V2, String, bool);

fn rows(w: &World, plan: &Plan, c: UnitId) -> Vec<Row> {
    let reasons = rip::reasons(w, plan, c);
    let ghost = rip::ghost_entries(w, plan, c);
    plan.entries_for(w, c).iter().zip(reasons).zip(ghost).map(|((e, r), g)| (w.cat.skills[e.skill].id.clone(), e.start_block, e.end_block(w), e.total_ap(), e.pos_after, r, g)).collect()
}

fn names(r: &[Row]) -> Vec<&str> {
    r.iter().map(|x| x.0.as_str()).collect()
}

fn ghosts(r: &[Row]) -> Vec<bool> {
    r.iter().map(|x| x.6).collect()
}

fn rest_gain(w: &World, blocks: i32) -> i32 {
    round_intent::rest_gain(&w.ruleset.numbers, blocks)
}

/// The AP the round's end gives (AP1) from `left` AP with `rest` planned Rest blocks.
fn next_of(w: &World, c: UnitId, left: i32, rest: i32) -> i32 {
    let u = &w.units[c];
    u.max_ap.min(u.max_ap.min(left + AP_REGEN + u.ap_regen_bonus) + rest_gain(w, rest))
}

/// Section 4's worked examples, picks solid and the ghost after them, with AP1's numbers (section 7's gauge table: now,
/// spent, left, next).
#[test]
fn the_worked_examples_and_their_ap_forecast() {
    // tl_vanguard, 10 AP, the footman 3 tiles away: click the footman, Heavy Cleave
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(2, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let hc = sk(&s, "tl_heavy_cleave");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[hc]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!(names(&r), ["move", "tl_heavy_cleave", "tl_shoulder_check"], "{r:?}");
    assert_eq!(ghosts(&r), [false, false, true], "the pick and its approach solid, the follow-up ghost");
    assert_eq!((r[0].1, r[0].3, r[0].5.as_str()), (1, 2, rip::APPROACH), "b1: the approach belongs to the pick");
    assert_eq!((r[1].1, r[1].2, r[1].5.as_str()), (2, 5, rip::OPENER));
    assert_eq!((r[2].1, r[2].5.as_str()), (6, "follow-up 2"), "the fill: Heavy Cleave does not fit, Shoulder Check does");
    assert_eq!(round_intent::intent_of(&s.w, VANGUARD), Some(RoundIntent::Assault), "the pick set the intent");
    assert_eq!(rip::ghost_from(&s.w, &s.plan, VANGUARD), 2);
    let f = rip::ap_forecast(&s.w, &s.plan, VANGUARD);
    let (hc_ap, sc_ap) = (ap_of(&s.w, VANGUARD, "tl_heavy_cleave"), ap_of(&s.w, VANGUARD, "tl_shoulder_check"));
    assert_eq!((f.now, f.picked, f.filled, f.spent), (UNIT_AP, 2 + hc_ap, sc_ap, 2 + hc_ap + sc_ap));
    assert_eq!((f.now, f.spent, f.left, f.next, f.max), (10, 7, 3, 7, 10), "the spec's row: 10 - 7 = 3, next 7");
    assert_eq!((f.gain, f.rest_gain), (f.next - f.left, 0));

    // the same, Auto-fill off: the free block rests
    rip::set_autofill(&mut s.w, &mut s.plan, false);
    let r = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!(names(&r), ["move", "tl_heavy_cleave", "rest"], "{r:?}");
    assert_eq!(ghosts(&r), [false, false, true]);
    let f = rip::ap_forecast(&s.w, &s.plan, VANGUARD);
    assert_eq!((f.spent, f.left, f.next), (6, 4, next_of(&s.w, VANGUARD, 4, 1)));
    assert_eq!((f.now, f.spent, f.left, f.next), (10, 6, 4, 8), "the spec's row: 10 - 6 = 4, next 8");

    // tl_archer, 10 AP, the footman adjacent: Aimed Shot, Snap Shot
    let mut s = session();
    place(&mut s.w, ARCHER, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let (aim, snap) = (sk(&s, "tl_aimed_shot"), sk(&s, "tl_snap_shot"));
    pick(&mut s, ARCHER, foe(FOOTMAN), &[aim]).unwrap();
    pick(&mut s, ARCHER, foe(FOOTMAN), &[snap]).unwrap();
    let r = rows(&s.w, &s.plan, ARCHER);
    assert_eq!(names(&r), ["move", "tl_aimed_shot", "tl_snap_shot", "tl_aimed_shot"], "{r:?}");
    assert_eq!(ghosts(&r), [false, false, false, true]);
    assert_eq!(r.iter().map(|x| (x.1, x.2)).collect::<Vec<_>>(), [(1, 1), (2, 3), (4, 4), (5, 6)], "b1 kite, b2-b3, b4, [b5-b6]");
    assert_eq!((r[0].5.as_str(), r[2].5.as_str()), (rip::KITE_IN_MELEE, "pick 2"));
    let f = rip::ap_forecast(&s.w, &s.plan, ARCHER);
    assert_eq!((f.now, f.spent, f.left, f.next), (10, 8, 2, 6), "the spec's row: 10 - 8 = 2, next 6");

    // tl_vanguard at 3 AP, the footman adjacent: the ghost is the auto plan; Heavy Cleave is refused, the ghost stays
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    s.w.units[VANGUARD].ap = 3;
    round_intent::set_intent(&mut s.w, VANGUARD, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut s.w, VANGUARD, foe(FOOTMAN)).unwrap();
    rip::plan_unit(&mut s.w, &mut s.plan, VANGUARD).unwrap();
    let before = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!(names(&before), ["tl_shoulder_check", "tl_shoulder_check", "tl_shoulder_check", "rest", "rest", "rest"]);
    assert!(ghosts(&before).iter().all(|g| *g), "with no pick the whole plan is ghost");
    let hc = sk(&s, "tl_heavy_cleave");
    assert_eq!(pick(&mut s, VANGUARD, foe(FOOTMAN), &[hc]), Err(format!("AP 3 < {}", ap_of(&s.w, VANGUARD, "tl_heavy_cleave"))));
    assert_eq!(rows(&s.w, &s.plan, VANGUARD), before, "a refused pick changes nothing");
    let f = rip::ap_forecast(&s.w, &s.plan, VANGUARD);
    assert_eq!((f.now, f.spent, f.left, f.next, f.rest_gain), (3, 3, 0, next_of(&s.w, VANGUARD, 0, 3), rest_gain(&s.w, 3)));
    assert_eq!((f.now, f.spent, f.left, f.next), (3, 3, 0, 5), "the spec's row: 3 - 3 = 0, next 5 (Rest 3 blocks: +1)");
}

/// IF2: the rows by target kind before the first pick (only skills that could act on that kind, disabled with a reason
/// when they cannot now, items, Rest last) and after it (only the round intent's rows that can still be picked).
#[test]
fn menus_by_target_kind_and_after_the_first_pick() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, ARCHER, V2::new(4, 6));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    place(&mut s.w, FOOTMAN_B, V2::new(8, 2));
    let intent_of = |o: &rip::PickOption| o.intent;
    let menu = |s: &Session, c: UnitId, at: RoundTarget| rip::pick_options(&s.w, &s.plan, c, at);

    let on_foe = menu(&s, VANGUARD, foe(FOOTMAN));
    assert!(on_foe.iter().all(|o| matches!(intent_of(o), Some(RoundIntent::Assault | RoundIntent::Maneuver) | None)), "an enemy takes Assault and Maneuver skills");
    assert!(!on_foe.iter().any(|o| intent_of(o) == Some(RoundIntent::Support) || intent_of(o) == Some(RoundIntent::Guard)), "never a Support or Guard row");
    assert_eq!(on_foe.last().map(|o| o.picks.clone()), Some(vec![Pick::Rest]), "Rest last");
    let hc_row = on_foe.iter().find(|o| o.picks == vec![sk(&s, "tl_heavy_cleave")]).expect("Heavy Cleave listed");
    assert_eq!((hc_row.problem.clone(), hc_row.ap, hc_row.blocks), (None, ap_of(&s.w, VANGUARD, "tl_heavy_cleave"), need_of(&s.w, VANGUARD, "tl_heavy_cleave")));
    let fc = on_foe.iter().filter(|o| matches!(&o.picks[..], [Pick::Skill { skill, .. }] if *skill == sid(&s.w, "flank_cut"))).map(|o| o.kind).collect::<Vec<_>>();
    assert_eq!(fc, [Some(ManeuverKind::Engage), Some(ManeuverKind::Disengage)], "a Maneuver attack once per kind, Engage first (MV11)");

    let on_ally = menu(&s, ARCHER, foe(VANGUARD));
    assert!(on_ally.iter().any(|o| o.picks == vec![sk(&s, "tl_field_dressing")]), "an ally takes Support skills");
    assert!(on_ally.iter().all(|o| matches!(intent_of(o), Some(RoundIntent::Support) | None)), "and only those: {on_ally:?}");

    let on_self = menu(&s, VANGUARD, RoundTarget::Myself);
    assert!(on_self.iter().any(|o| o.picks == vec![sk(&s, "tl_brace_counter")]) && on_self.iter().all(|o| intent_of(o) != Some(RoundIntent::Assault)), "self: Guard skills, no attack");

    let tile = V2::new(4, 2);
    let on_tile = menu(&s, VANGUARD, RoundTarget::Cell(tile));
    assert_eq!(on_tile[0].picks, vec![Pick::MoveHere(tile)], "the tile menu opens with Move here");
    assert!(on_tile.iter().any(|o| o.picks == vec![Pick::MoveHere(tile), sk(&s, "active_guard")]), "Move here, then a Guard skill");
    assert!(!on_tile.iter().any(|o| matches!(o.picks[..], [Pick::StepAfter(_)])), "no step after before a skill pick");

    // a disabled row says why: at 3 AP Heavy Cleave cannot be paid
    s.w.units[VANGUARD].ap = 3;
    let poor = menu(&s, VANGUARD, foe(FOOTMAN));
    let hc = poor.iter().find(|o| o.picks == vec![sk(&s, "tl_heavy_cleave")]).unwrap();
    assert_eq!(hc.problem.as_deref(), Some(format!("AP 3 < {}", ap_of(&s.w, VANGUARD, "tl_heavy_cleave")).as_str()));
    s.w.units[VANGUARD].ap = UNIT_AP;

    // after Heavy Cleave on the footman: only Assault rows that can still be picked, no Guard row, no disabled row
    let hc = sk(&s, "tl_heavy_cleave");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[hc]).unwrap();
    let after = menu(&s, VANGUARD, foe(FOOTMAN));
    assert!(!after.is_empty() && after.iter().all(|o| o.problem.is_none()), "no disabled row: {after:?}");
    assert!(after.iter().all(|o| matches!(intent_of(o), Some(RoundIntent::Assault) | None)), "only the round intent's rows");
    assert!(!after.iter().any(|o| o.picks == vec![sk(&s, "tl_heavy_cleave")]), "Heavy Cleave no longer fits the blocks left");
    let guard = sk(&s, "tl_brace_counter");
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Myself, &[guard]), Err("Only Assault skills this round (tl_heavy_cleave set it)".to_string()), "IF6 a");
    // one target per round: another enemy's menu has no row, and a pick on it is refused
    assert!(menu(&s, VANGUARD, foe(FOOTMAN_B)).is_empty(), "One target per round: tl_footman");
    let sc = sk(&s, "tl_shoulder_check");
    assert_eq!(pick(&mut s, VANGUARD, foe(FOOTMAN_B), &[sc]), Err("One target per round: tl_footman".to_string()), "IF6 b");
}

/// IF4: the first skill or item pick sets the intent, the round target and the Maneuver kind; neutral picks set none.
#[test]
fn the_first_pick_sets_intent_and_kind_neutral_picks_set_none() {
    let mut s = session();
    place(&mut s.w, DECKED, V2::new(3, 3));
    pick(&mut s, DECKED, RoundTarget::Myself, &[Pick::Rest]).unwrap();
    pick(&mut s, DECKED, RoundTarget::Myself, &[Pick::Rest]).unwrap();
    assert_eq!(round_intent::intent_of(&s.w, DECKED), None, "Rest sets no intent");
    assert_eq!(names(&rows(&s.w, &s.plan, DECKED)), ["rest"; BLOCKS as usize], "two Rest picks, then the ghost Rest (no intent: nothing to fill)");
    assert_eq!(ghosts(&rows(&s.w, &s.plan, DECKED))[..2], [false, false]);

    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(7, 4));
    place(&mut s.w, FOOTMAN, V2::new(6, 4));
    round_intent::set_intent(&mut s.w, FOOTMAN, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut s.w, FOOTMAN, foe(VANGUARD)).unwrap();
    place(&mut s.w, SPIKER, V2::new(5, 2));
    let fc = Pick::Skill { skill: sid(&s.w, "flank_cut"), kind: Some(ManeuverKind::Engage) };
    pick(&mut s, SPIKER, foe(FOOTMAN), &[fc]).unwrap();
    let rs = &s.w.round_states[&SPIKER];
    assert_eq!((rs.intent, rs.maneuver, rs.target), (Some(RoundIntent::Maneuver), Some(ManeuverKind::Engage), Some(foe(FOOTMAN))));
}

/// GH3: a pick goes at the first block after the last pick, replacing the ghost there, and the fill is rebuilt after it.
#[test]
fn a_pick_replaces_the_ghost_from_its_block() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    round_intent::set_intent(&mut s.w, VANGUARD, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut s.w, VANGUARD, foe(FOOTMAN)).unwrap();
    rip::plan_unit(&mut s.w, &mut s.plan, VANGUARD).unwrap();
    assert_eq!(names(&rows(&s.w, &s.plan, VANGUARD))[0], "tl_heavy_cleave", "the ghost opens with the order's first skill");
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc.clone()]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!((r[0].0.as_str(), r[0].1, r[0].6), ("tl_shoulder_check", 1, false), "b1 is the pick");
    assert_eq!((r[1].0.as_str(), r[1].1, r[1].6), ("tl_heavy_cleave", 2, true), "the fill restarts after it: the order's first that fits");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!(r.iter().map(|x| (x.0.as_str(), x.1, x.6)).take(3).collect::<Vec<_>>(), [("tl_shoulder_check", 1, false), ("tl_shoulder_check", 2, false), ("tl_heavy_cleave", 3, true)]);
    assert_eq!(rip::picks_of(&s.w, VANGUARD).len(), 2);
    // IF7: undo takes the last pick back, Clear every pick; with none left the plan is the auto plan again
    rip::unpick(&mut s.w, &mut s.plan, VANGUARD).0.unwrap();
    assert_eq!(rows(&s.w, &s.plan, VANGUARD)[1].0, "tl_heavy_cleave");
    rip::clear_picks(&mut s.w, &mut s.plan, VANGUARD).0.unwrap();
    let mut p2 = Plan::new();
    let mut w2 = s.w.clone();
    rip::plan_unit(&mut w2, &mut p2, VANGUARD).unwrap();
    assert_eq!(rows(&s.w, &s.plan, VANGUARD), rows(&w2, &p2, VANGUARD), "no pick: exactly plan_unit's plan");
    assert_eq!(s.w.round_states[&VANGUARD].target, Some(foe(FOOTMAN)), "the round target stays");
}

/// GH4, ET4: Step here after reserves the round's last block; later picks and the ghost go before it; a pick that would
/// need it is refused; one a round, after a skill pick, never after the round's regular move, one move block at most.
#[test]
fn the_step_after_takes_the_last_block() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let step = V2::new(3, 4);
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(step), &[Pick::StepAfter(step)]), Err("Step here after follows a skill or item pick".to_string()));
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc.clone()]).unwrap();
    pick(&mut s, VANGUARD, RoundTarget::Cell(step), &[Pick::StepAfter(step)]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    let last = r.last().unwrap();
    assert_eq!((last.0.as_str(), last.1, last.4, last.5.as_str(), last.6), ("move", BLOCKS, step, rip::STEP_AFTER, false), "the after-move on the last block, a pick: {r:?}");
    assert!(r[1..r.len() - 1].iter().all(|x| x.6 && x.2 < BLOCKS), "the ghost fills the blocks before it");
    assert_eq!(rip::regular_move(&s.w, &s.plan, VANGUARD), Some(RegularMove::StepAfter));
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    let hc = sk(&s, "tl_heavy_cleave");
    let need = need_of(&s.w, VANGUARD, "tl_heavy_cleave");
    assert_eq!(pick(&mut s, VANGUARD, foe(FOOTMAN), &[hc]), Err(format!("Blocks 3 < {need} (the step after takes B{BLOCKS})")), "IF6 c: b3-b5 are left");
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(step), &[Pick::StepAfter(step)]), Err("One step after a round".to_string()));

    // after a pick whose approach is the round's move: refused with the move's kind
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(2, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let hc = sk(&s, "tl_heavy_cleave");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[hc]).unwrap();
    assert_eq!(rip::regular_move(&s.w, &s.plan, VANGUARD), Some(RegularMove::Approach));
    let tile = V2::new(4, 3);
    let row = rip::pick_options(&s.w, &s.plan, VANGUARD, RoundTarget::Cell(tile));
    assert!(!row.iter().any(|o| o.picks == vec![Pick::StepAfter(tile)]), "not offered after the first pick when it cannot be picked");
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(tile), &[Pick::StepAfter(tile)]), Err("Already moved this round (approach)".to_string()));

    // too far for one move block
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    let mpb = s.w.units[VANGUARD].move_per_block();
    let far = V2::new(4 - mpb - 1, 4);
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(far), &[Pick::StepAfter(far)]), Err(format!("Too far for one move block: {} > {mpb}", mpb + 1)));
}

/// ET2, ET3, ET5: Move here is a plain move, only first and while no move is planned, within two move blocks; a pick
/// after it is measured from the new tile (no approach follows); "Move here, then Active Guard" holds the tile.
#[test]
fn move_here_and_the_one_regular_move() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(1, 4));
    place(&mut s.w, FOOTMAN, V2::new(10, 4));
    let most = s.plan.max_move_tiles(&s.w, VANGUARD);
    let far = V2::new(1 + most + 1, 4);
    let rows_far = rip::pick_options(&s.w, &s.plan, VANGUARD, RoundTarget::Cell(far));
    assert_eq!(rows_far[0].problem, Some(format!("Too far: {} tiles > {most} (two move blocks)", most + 1)), "Move here 5 tiles away");
    let near = V2::new(3, 4);
    pick(&mut s, VANGUARD, RoundTarget::Cell(near), &[Pick::MoveHere(near)]).unwrap();
    assert_eq!(round_intent::intent_of(&s.w, VANGUARD), None, "a plain move sets no intent");
    assert_eq!(rip::regular_move(&s.w, &s.plan, VANGUARD), Some(RegularMove::MoveHere));
    let sc = sk(&s, "tl_shoulder_check");
    assert_eq!(pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]), Err("Out of reach: 7 > 1".to_string()), "measured from the new tile, no approach");
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(V2::new(3, 5)), &[Pick::MoveHere(V2::new(3, 5))]), Err("Already moving this round".to_string()));

    // a pick in reach after the move: no approach entry follows the move
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(2, 4));
    place(&mut s.w, FOOTMAN, V2::new(6, 4));
    let to = V2::new(5, 4);
    pick(&mut s, VANGUARD, RoundTarget::Cell(to), &[Pick::MoveHere(to)]).unwrap();
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    let moves: Vec<&Row> = r.iter().filter(|x| x.0 == "move").collect();
    assert!(moves.iter().all(|x| x.5 == rip::MOVE_HERE && !x.6 && x.4.dist(to) <= 2), "only Move here's entries move: {r:?}");
    assert_eq!(r.iter().find(|x| x.0 == "tl_shoulder_check").map(|x| (x.4, x.6)), Some((to, false)));

    // Move here, then Active Guard: the move, then the Guard skill, the tile the round target
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(2, 4));
    let tile = V2::new(4, 4);
    let guard = sk(&s, "active_guard");
    pick(&mut s, VANGUARD, RoundTarget::Cell(tile), &[Pick::MoveHere(tile), guard]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!(names(&r)[..2], ["move", "active_guard"], "{r:?}");
    assert_eq!((r[0].4, r[0].6, r[1].6), (tile, false, false));
    assert_eq!(s.w.round_states[&VANGUARD].target, Some(RoundTarget::Cell(tile)));
    assert_eq!(round_intent::intent_of(&s.w, VANGUARD), Some(RoundIntent::Guard));
}

/// ET5: a positioned skill's dash is not the round's move, so Step here after is offered after it; the auto planner
/// never plans both a before-move and an after-move.
#[test]
fn a_dash_is_not_the_regular_move() {
    let mut s = session();
    place(&mut s.w, FOOTMAN, V2::new(6, 4));
    place(&mut s.w, VANGUARD, V2::new(7, 4));
    round_intent::set_intent(&mut s.w, FOOTMAN, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut s.w, FOOTMAN, foe(VANGUARD)).unwrap();
    place(&mut s.w, SPIKER, V2::new(5, 2));
    let fc = Pick::Skill { skill: sid(&s.w, "flank_cut"), kind: Some(ManeuverKind::Engage) };
    pick(&mut s, SPIKER, foe(FOOTMAN), &[fc]).unwrap();
    let e = s.plan.entries_for(&s.w, SPIKER).into_iter().find(|e| e.skill == sid(&s.w, "flank_cut")).unwrap();
    assert_eq!((e.pos_before, e.pos_after, e.path.len()), (V2::new(5, 2), V2::new(6, 3), 2), "the dash carries the hero 2 tiles");
    assert_eq!(rip::regular_move(&s.w, &s.plan, SPIKER), None, "the dash is no regular move");
    let tile = V2::new(5, 3);
    let offered = rip::pick_options(&s.w, &s.plan, SPIKER, RoundTarget::Cell(tile));
    assert!(offered.iter().any(|o| o.picks == vec![Pick::StepAfter(tile)] && o.problem.is_none()), "Step here after offered: {offered:?}");
    pick(&mut s, SPIKER, RoundTarget::Cell(tile), &[Pick::StepAfter(tile)]).unwrap();

    // the auto planner: at most one regular move a plan (an approach, a kite, a kite after)
    for (c, at) in [(VANGUARD, V2::new(1, 4)), (ARCHER, V2::new(5, 4)), (ARCHER, V2::new(2, 4))] {
        let mut s = session();
        place(&mut s.w, c, at);
        place(&mut s.w, FOOTMAN, V2::new(6, 4));
        round_intent::set_intent(&mut s.w, c, RoundIntent::Assault).unwrap();
        round_intent::set_target(&mut s.w, c, foe(FOOTMAN)).unwrap();
        rip::plan_unit(&mut s.w, &mut s.plan, c).unwrap();
        let mut steps: Vec<i32> = s.plan.entries_for(&s.w, c).iter().filter(|e| skill::kind(&s.w.cat.skills[e.skill]) == Kind::Move).map(|e| e.step_index).collect();
        steps.dedup();
        assert!(steps.len() <= 1, "{c} from {at}: one regular move at most: {:?}", rows(&s.w, &s.plan, c));
    }
}

/// GH5: a refused pick leaves the battle as it was; a later member's pick that no longer fits is dropped with the
/// message "<hero>: <skill> dropped: <reason>".
#[test]
fn refusals_are_atomic_and_later_picks_drop() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, ARCHER, V2::new(4, 6));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    let (r0, st0, ev0) = (rows(&s.w, &s.plan, VANGUARD), s.w.round_states.clone(), s.events.len());
    let guard = sk(&s, "active_guard");
    assert!(pick(&mut s, VANGUARD, RoundTarget::Myself, &[guard]).is_err());
    assert_eq!((rows(&s.w, &s.plan, VANGUARD), &s.w.round_states, s.events.len()), (r0, &st0, ev0), "nothing changed");

    // the faster vanguard leaves its tile in b1, so the archer may step onto it; the vanguard's undo claims it back
    let mut s = session();
    let start = V2::new(4, 4);
    place(&mut s.w, VANGUARD, start);
    place(&mut s.w, ARCHER, V2::new(4, 6));
    s.w.units[VANGUARD].speed = 20;
    let away = V2::new(4, 2);
    pick(&mut s, VANGUARD, RoundTarget::Cell(away), &[Pick::MoveHere(away)]).unwrap();
    pick(&mut s, ARCHER, RoundTarget::Cell(start), &[Pick::MoveHere(start)]).unwrap();
    let (result, later) = rip::unpick(&mut s.w, &mut s.plan, VANGUARD);
    result.unwrap();
    assert_eq!(later, vec![(ARCHER, "tl_archer: Move here dropped: No path".to_string())]);
    assert!(rip::picks_of(&s.w, ARCHER).is_empty(), "the dropped pick and every pick after it go");
}

/// GH6: the picks reset at the round start; the intent carries over.
#[test]
fn picks_reset_at_the_round_start() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    assert!(s.confirm_plan());
    drop_enemy_plans(&mut s);
    while s.phase != Phase::Replay {
        s.step_next();
    }
    s.advance_round();
    let rs: &RoundState = &s.w.round_states[&VANGUARD];
    assert!(rs.picks.is_empty() && rs.target.is_none(), "picks and the target reset");
    assert_eq!(rs.intent, Some(RoundIntent::Assault), "the intent carries over");
}

/// AP1 on a fight with no AP effects: `next` is the AP the hero has at the next planning (ap_regen, then the Rest gain).
#[test]
fn the_forecast_next_is_the_next_rounds_ap() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    s.w.units[VANGUARD].ap = 3;
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    let f = rip::ap_forecast(&s.w, &s.plan, VANGUARD);
    assert!(f.rest_gain > 0, "Rest blocks planned: {f:?}");
    assert!(s.confirm_plan());
    drop_enemy_plans(&mut s);
    while s.phase != Phase::Replay {
        s.step_next();
    }
    s.advance_round();
    assert_eq!(s.w.units[VANGUARD].ap, f.next, "{f:?}");
}

/// HK1: the deck's actives in slot order, then the basics that are not neutral, in `basics` order; never a neutral skill.
#[test]
fn hotkeys_follow_the_slots_then_the_basics() {
    let s = session();
    let ids = |c: UnitId| rip::hotkey_skills(&s.w, c).iter().map(|&k| s.w.cat.skills[k].id.clone()).collect::<Vec<_>>();
    let decked = ids(DECKED);
    assert_eq!(decked[..3], ["tl_shoulder_check", "tl_heavy_cleave", "tl_brace_counter"], "the actives in slot order: {decked:?}");
    let basics: Vec<&str> = BASICS.iter().copied().filter(|b| round_intent::skill_intent(&s.w.cat.skills[sid(&s.w, b)]) != "neutral").collect();
    assert_eq!(decked[3..], basics, "then the basics with an intent");
    for c in [VANGUARD, ARCHER, SPIKER] {
        assert!(rip::hotkey_skills(&s.w, c).iter().all(|&k| round_intent::skill_intent(&s.w.cat.skills[k]) != "neutral"));
        assert_eq!(ids(c)[ids(c).len() - basics.len()..], basics, "basics last");
    }
}

/// IF5: a preview (and the menu, which previews every row) changes neither the world nor the plan.
#[test]
fn previews_change_nothing() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(2, 4));
    place(&mut s.w, ARCHER, V2::new(4, 6));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    let snap = |s: &Session| (rows(&s.w, &s.plan, VANGUARD), rows(&s.w, &s.plan, ARCHER), s.w.round_states.clone(), s.w.units.iter().map(|u| (u.ap, u.hp)).collect::<Vec<_>>(), s.w.autofill);
    let before = snap(&s);
    let hc = sk(&s, "tl_heavy_cleave");
    let (steps, f) = rip::preview_pick(&s.w, &s.plan, VANGUARD, foe(FOOTMAN), &[hc]).unwrap();
    assert!(!steps.is_empty() && f.picked > rip::ap_forecast(&s.w, &s.plan, VANGUARD).picked, "the preview has the new pick");
    let _ = rip::pick_options(&s.w, &s.plan, VANGUARD, foe(FOOTMAN));
    let _ = rip::pick_options(&s.w, &s.plan, VANGUARD, RoundTarget::Cell(V2::new(1, 1)));
    assert_eq!(snap(&s), before);
}

/// IF9: the party tactic drafts every member with no pick and leaves a member with picks as the player left it.
#[test]
fn the_party_tactic_skips_members_with_picks() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, ARCHER, V2::new(2, 6));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[Pick::Rest]).unwrap();
    let kept = (rows(&s.w, &s.plan, VANGUARD), s.w.round_states[&VANGUARD].clone());
    assert!(s.plan.steps_of(ARCHER).is_empty(), "the archer has no plan yet");
    s.plan_apply_preset_all("balanced");
    assert_eq!((rows(&s.w, &s.plan, VANGUARD), s.w.round_states[&VANGUARD].clone()), kept, "the member with a pick is skipped");
    assert!(!s.plan.steps_of(ARCHER).is_empty(), "a member with no pick is drafted");
}
