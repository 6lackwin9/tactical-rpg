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
fn the_step_after_replaces_the_last_block() {
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
    assert_eq!(pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]), Err(rip::ONLY_REST.to_string()), "VISCLAR1-D: only Rest follows it");
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
    s.w.units[VANGUARD].initiative = 20;
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
    s.plan_apply_preset(VANGUARD, "balanced");
    assert_eq!((rows(&s.w, &s.plan, VANGUARD), s.w.round_states[&VANGUARD].clone()), kept, "also one by one");
}

/// HK2, HK3 (`hotkey_target`): a self-only skill (a Defend kind, or `target_mode` SELF) picked by its key first aims at
/// self; once a pick holds the round target it goes on that target; a skill aimed at others takes the round target,
/// else the proposed one for the intent (TG6), else none.
#[test]
fn hotkeys_aim_a_self_only_skill_at_self_first() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let rs = s.w.round_states.entry(VANGUARD).or_default();
    rs.intent = None;
    rs.target = None;
    let (guard, cleave, dressing) = (sid(&s.w, "active_guard"), sid(&s.w, "tl_heavy_cleave"), sid(&s.w, "tl_field_dressing"));
    assert!(rip::self_only(&s.w.cat.skills[guard]) && !rip::self_only(&s.w.cat.skills[cleave]) && !rip::self_only(&s.w.cat.skills[dressing]), "a Defend kind is self-only; an attack and an ally-or-self heal are not");
    assert_eq!(rip::hotkey_target(&s.w, VANGUARD, guard), Some(RoundTarget::Myself), "no pick yet: self");
    assert_eq!(rip::hotkey_target(&s.w, VANGUARD, cleave), None, "no target and no intent: none");
    s.set_round_intent(VANGUARD, RoundIntent::Assault.id()).unwrap();
    assert_eq!(rip::hotkey_target(&s.w, VANGUARD, cleave), rip::proposed_target(&s.w, VANGUARD, RoundIntent::Assault), "the proposed target (TG6)");
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc]).unwrap();
    assert_eq!(rip::hotkey_target(&s.w, VANGUARD, guard), Some(foe(FOOTMAN)), "a pick holds the target: the self-only skill goes there too (and is refused there)");
    assert_eq!(rip::hotkey_target(&s.w, VANGUARD, cleave), Some(foe(FOOTMAN)));
}

/// GH2 b, IN6 timing: `preview_round` makes, on copies, exactly what `Session::set_round` then makes (the battle screen
/// draws a waiting round change from the one and Confirm sends the other) and changes nothing; a refused field leaves the
/// battle as it was.
#[test]
fn a_round_change_previews_what_it_makes() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(2, 4));
    place(&mut s.w, ARCHER, V2::new(2, 6));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    s.set_round_intent(ARCHER, RoundIntent::Assault.id()).unwrap();
    s.set_round(ARCHER, &rip::RoundSet { target: Some(foe(FOOTMAN)), ..Default::default() }).unwrap();
    let party = s.w.allies.clone();
    let snap = |s: &Session| (party.iter().map(|&c| rows(&s.w, &s.plan, c)).collect::<Vec<_>>(), s.w.round_states.clone(), s.events.len());
    let before = snap(&s);
    let set = rip::RoundSet { intent: Some(RoundIntent::Assault), target: Some(foe(FOOTMAN)), ..Default::default() };
    let (w2, p2) = rip::preview_round(&s.w, &s.plan, VANGUARD, &set).unwrap();
    assert_eq!(snap(&s), before, "the preview changes nothing");
    let done = s.set_round(VANGUARD, &set).unwrap();
    assert!(done.planned.is_ok(), "{:?}", done.planned);
    for &c in &party {
        assert_eq!(rows(&w2, &p2, c), rows(&s.w, &s.plan, c), "member {c}: the preview is the change");
    }
    assert_eq!(w2.round_states, s.w.round_states);
    let after = snap(&s);
    let refused = rip::RoundSet { intent: Some(RoundIntent::Assault), target: Some(RoundTarget::Myself), ..Default::default() };
    assert!(s.set_round(VANGUARD, &refused).is_err() && rip::preview_round(&s.w, &s.plan, VANGUARD, &refused).is_err(), "Assault takes no self target");
    assert_eq!(snap(&s), after, "a refused field leaves the battle as it was");
}

/// INPUT1-AP (AP3, AP8): a row refused for its AP carries `ap_short`, its cost less the AP free for it as the refusal
/// counts it (the AP left after the earlier picks); a row refused for another reason or a payable row carries none.
/// After a picked Move here the free AP is what the move left, so the shortfall is the cost less that (here 1), not the
/// cost less the AP held (here 0).
#[test]
fn an_ap_refusal_carries_its_shortfall() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let hc_cost = ap_of(&s.w, VANGUARD, "tl_heavy_cleave");
    s.w.units[VANGUARD].ap = hc_cost;
    // the first-pick menu at full cost: payable, no shortfall
    let hc = sk(&s, "tl_heavy_cleave");
    let row = |s: &Session| rip::pick_options(&s.w, &s.plan, VANGUARD, foe(FOOTMAN)).into_iter().find(|o| o.picks == vec![hc.clone()]).expect("Heavy Cleave listed");
    let r = row(&s);
    assert_eq!((r.problem.clone(), r.ap_short), (None, None), "{r:?}");
    // a picked Move here one tile (off the footman) spends 1 AP: Heavy Cleave is now 1 short
    let tile = V2::new(4, 5);
    pick(&mut s, VANGUARD, RoundTarget::Cell(tile), &[Pick::MoveHere(tile)]).unwrap();
    let f = rip::ap_forecast(&s.w, &s.plan, VANGUARD);
    assert_eq!(f.picked, 1, "the move is a pick");
    let r = row(&s);
    let free = hc_cost - f.picked;
    assert_eq!(r.problem.as_deref(), Some(format!("AP {free} < {hc_cost}").as_str()));
    assert_eq!(r.ap_short, Some(hc_cost - free), "cost less the AP free after the move: {r:?}");
    assert_eq!(r.ap_short, Some(1));
    assert_ne!(r.ap_short, Some(hc_cost - f.now), "not the cost less the AP held");
    // a row refused for a reason other than AP: no shortfall
    let other = rip::pick_options(&s.w, &s.plan, VANGUARD, foe(FOOTMAN)).into_iter().filter(|o| o.problem.as_deref().is_some_and(|p| !p.starts_with("AP "))).collect::<Vec<_>>();
    assert!(other.iter().all(|o| o.ap_short.is_none()), "{other:?}");
}

/// INPUT1-AP (AP3, AP8): Move here (a move row) at too little AP carries its shortfall, the route's AP less the AP
/// free; with an AP debt the debt counts too (the free AP below 0; the words show it from 0), and the "Move here, then"
/// rows carry the move's shortfall.
#[test]
fn a_move_row_carries_its_shortfall_and_a_debt_counts() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    let tile = V2::new(4, 2);
    let menu = |s: &Session| rip::pick_options(&s.w, &s.plan, VANGUARD, RoundTarget::Cell(tile));
    let route = menu(&s).into_iter().find(|o| o.picks == vec![Pick::MoveHere(tile)] && o.problem.is_none()).expect("Move here payable at full AP").ap;
    assert!(route >= 1);
    for now in [route - 1, -2] {
        s.w.units[VANGUARD].ap = now;
        let rows = menu(&s);
        let mv = rows.iter().find(|o| o.picks == vec![Pick::MoveHere(tile)]).expect("Move here listed");
        assert_eq!(mv.problem.as_deref(), Some(format!("AP {} < {route}", now.max(0)).as_str()), "at {now} AP");
        assert_eq!(mv.ap_short, Some(route - now), "at {now} AP: {mv:?}");
        for then in rows.iter().filter(|o| o.picks.len() == 2 && o.picks[0] == Pick::MoveHere(tile)) {
            assert_eq!(then.ap_short, Some(route - now), "Move here, then: {then:?}");
        }
    }
}

/// The ghost entries' skill ids after the picks (the fill and its Rest).
fn ghost_names(s: &Session, c: UnitId) -> Vec<String> {
    rows(&s.w, &s.plan, c).into_iter().filter(|r| r.6).map(|r| r.0).collect()
}

fn fams(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|x| x.to_string()).collect()
}

/// INPUT1 FL2 (INPUT1.md 9's acceptance) on tl_vanguard with `order.assault` ranking Shoulder Check (A) before Heavy
/// Cleave (B): after a Strike pick the fill takes A from the order; with the list `strike -> [B]` it takes B where the
/// order would place A; when B cannot be placed (AP, then blocks) the block takes A from the order. The list is the
/// opener's (a later pick's list is not read), and a ghost-only plan reads the list of PL3's opener.
#[test]
fn the_fill_reads_the_openers_follow_up_list_first_then_the_order() {
    let setup = || {
        let mut s = session();
        place(&mut s.w, VANGUARD, V2::new(4, 4));
        place(&mut s.w, FOOTMAN, V2::new(5, 4));
        s.w.units[VANGUARD].deck.order.insert("assault".into(), fams(&["tl_shoulder_check", "tl_heavy_cleave", "strike"]));
        s
    };
    let (a, b) = ("tl_shoulder_check", "tl_heavy_cleave");
    // without a list: the order
    let mut s = setup();
    let strike = sk(&s, "strike");
    assert_eq!(rip::follow_order(&s.w, VANGUARD, RoundIntent::Assault, Some(&round_intent::Opener::Skill(sid(&s.w, "strike")))), rip::order(&s.w, VANGUARD, RoundIntent::Assault), "no list: the order");
    pick(&mut s, VANGUARD, foe(FOOTMAN), std::slice::from_ref(&strike)).unwrap();
    let plain = ghost_names(&s, VANGUARD);
    assert_eq!(plain.first().map(String::as_str), Some(a), "the order's first entry fills: {plain:?}");

    // the list strike -> [B]: B where the order would place A
    let mut s = setup();
    s.w.units[VANGUARD].deck.follow_ups.insert("strike".into(), fams(&[b]));
    let want = [sid(&s.w, b), sid(&s.w, a), sid(&s.w, "strike")];
    assert_eq!(rip::follow_order(&s.w, VANGUARD, RoundIntent::Assault, Some(&round_intent::Opener::Skill(sid(&s.w, "strike")))), want, "the list, then the order, each once");
    pick(&mut s, VANGUARD, foe(FOOTMAN), std::slice::from_ref(&strike)).unwrap();
    let listed = ghost_names(&s, VANGUARD);
    assert_eq!(listed.first().map(String::as_str), Some(b), "the list's entry fills first: {listed:?}");
    let r = rows(&s.w, &s.plan, VANGUARD);
    let first_ghost = r.iter().find(|x| x.6).unwrap();
    assert_eq!(first_ghost.5, "follow-up 1", "tagged by its place in the list then the order");

    // B unaffordable: the block takes A from the order
    let mut s = setup();
    s.w.units[VANGUARD].deck.follow_ups.insert("strike".into(), fams(&[b]));
    let strike_ap = ap_of(&s.w, VANGUARD, "strike");
    s.w.units[VANGUARD].ap = strike_ap + ap_of(&s.w, VANGUARD, b) - 1;
    assert!(ap_of(&s.w, VANGUARD, a) <= ap_of(&s.w, VANGUARD, b) - 1, "A stays affordable");
    pick(&mut s, VANGUARD, foe(FOOTMAN), std::slice::from_ref(&strike)).unwrap();
    let poor = ghost_names(&s, VANGUARD);
    assert_eq!(poor.first().map(String::as_str), Some(a), "AP: B refused, A from the order: {poor:?}");

    // B does not fit the blocks left: A from the order
    let mut s = setup();
    s.w.units[VANGUARD].deck.follow_ups.insert("strike".into(), fams(&[b]));
    let picks = vec![strike.clone(); (BLOCKS - need_of(&s.w, VANGUARD, b) + 1) as usize];
    assert!(BLOCKS - picks.len() as i32 >= need_of(&s.w, VANGUARD, a), "A fits what is left");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &picks).unwrap();
    let tight = ghost_names(&s, VANGUARD);
    assert_eq!(tight.first().map(String::as_str), Some(a), "blocks: B refused, A from the order: {tight:?}");

    // the opener's list, not a later pick's: Shoulder Check's own list is not read after a Strike opener
    let mut s = setup();
    s.w.units[VANGUARD].deck.follow_ups.insert(a.into(), fams(&["strike"]));
    let sc = sk(&s, a);
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[strike.clone(), sc]).unwrap();
    let later = ghost_names(&s, VANGUARD);
    assert_eq!(later.first().map(String::as_str), Some(a), "Strike opened: the order, not Shoulder Check's list: {later:?}");

    // a ghost-only plan: PL3's opener (the order's first that fits, A) chooses the list
    let mut s = setup();
    s.w.units[VANGUARD].deck.follow_ups.insert(a.into(), fams(&["strike"]));
    round_intent::set_intent(&mut s.w, VANGUARD, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut s.w, VANGUARD, foe(FOOTMAN)).unwrap();
    rip::plan_unit(&mut s.w, &mut s.plan, VANGUARD).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!((r[0].0.as_str(), r[0].5.as_str()), (a, rip::OPENER), "{r:?}");
    assert_eq!(r[1].0, "strike", "after A its list's Strike: {r:?}");
}

/// FL2: the reader keeps only skills of the round intent the unit can use in this battle (an unknown family and a
/// Guard family in an Assault opener's list are skipped); an item opener and no opener read the order alone.
#[test]
fn the_follow_up_reader_skips_what_the_unit_cannot_use_here() {
    let mut s = session();
    let order = rip::order(&s.w, VANGUARD, RoundIntent::Assault);
    s.w.units[VANGUARD].deck.follow_ups.insert("tl_heavy_cleave".into(), fams(&["t_gone", "tl_brace_counter", "strike"]));
    let hc = round_intent::Opener::Skill(sid(&s.w, "tl_heavy_cleave"));
    let got = rip::follow_order(&s.w, VANGUARD, RoundIntent::Assault, Some(&hc));
    let mut want = vec![sid(&s.w, "strike")];
    want.extend(order.iter().copied().filter(|x| *x != sid(&s.w, "strike")));
    assert_eq!(got, want);
    assert_eq!(rip::follow_order(&s.w, VANGUARD, RoundIntent::Assault, None), order);
    assert_eq!(rip::follow_order(&s.w, VANGUARD, RoundIntent::Assault, Some(&round_intent::Opener::Item("t_potion".into()))), order, "an item opener reads the order");
    // an empty list is no list
    s.w.units[VANGUARD].deck.follow_ups.insert("tl_heavy_cleave".into(), Vec::new());
    assert_eq!(rip::follow_order(&s.w, VANGUARD, RoundIntent::Assault, Some(&hc)), order);
}

/// INPUT2: accepting the ghost turns it into picks with the same plan: the whole ghost (no ghost entry left, every
/// entry where it was), or up to one ghost entry (the entries up to it solid, the fill rebuilt after them); a pick
/// already made stays first; a non-ghost entry or a hero with nothing auto-filled is refused and changes nothing.
#[test]
fn accepting_the_ghost_makes_the_same_plan_solid() {
    let setup = |vanguard: V2, archer: V2| {
        let mut s = session();
        place(&mut s.w, VANGUARD, vanguard);
        place(&mut s.w, ARCHER, archer);
        place(&mut s.w, FOOTMAN, V2::new(5, 4));
        for c in [VANGUARD, ARCHER] {
            round_intent::set_intent(&mut s.w, c, RoundIntent::Assault).unwrap();
            round_intent::set_target(&mut s.w, c, foe(FOOTMAN)).unwrap();
            rip::plan_unit(&mut s.w, &mut s.plan, c).unwrap();
        }
        s
    };
    let shape = |r: &[Row]| r.iter().map(|x| (x.0.clone(), x.1, x.2, x.3, x.4)).collect::<Vec<_>>();
    // next to the footman, and an archer that must approach first (its move is the shot's implied move)
    for c in [VANGUARD, ARCHER] {
        let mut s = setup(V2::new(4, 4), V2::new(1, 4));
        let before = rows(&s.w, &s.plan, c);
        assert!(!before.is_empty() && before.iter().all(|r| r.6), "{c}: no pick, every entry ghost: {before:?}");
        if c == ARCHER {
            assert_eq!(names(&before)[0], "move", "the archer approaches first: {before:?}");
        }
        let (r, _) = rip::accept_ghost(&mut s.w, &mut s.plan, c, None);
        let n = r.unwrap_or_else(|e| panic!("{c}: {e}"));
        let after = rows(&s.w, &s.plan, c);
        assert_eq!(rip::picks_of(&s.w, c).len(), n);
        assert!(after.iter().all(|r| !r.6), "{c}: nothing ghost after: {after:?}");
        assert_eq!(shape(&after), shape(&before), "{c}: the same plan, solid");
    }
    // up to one ghost entry: the first two entries solid, the fill after them
    let mut s = setup(V2::new(4, 4), V2::new(1, 4));
    let before = rows(&s.w, &s.plan, VANGUARD);
    if before.len() >= 3 {
        rip::accept_ghost(&mut s.w, &mut s.plan, VANGUARD, Some(1)).0.unwrap();
        let after = rows(&s.w, &s.plan, VANGUARD);
        assert_eq!(shape(&after[..2]), shape(&before[..2]), "the entries up to the clicked one keep their place");
        assert_eq!(ghosts(&after)[..2], [false, false]);
        assert!(after[2..].iter().all(|r| r.6), "the fill after them stays ghost: {after:?}");
        // a solid entry is not a ghost entry: refused, nothing changes
        let picks = rip::picks_of(&s.w, VANGUARD);
        let (r, _) = rip::accept_ghost(&mut s.w, &mut s.plan, VANGUARD, Some(0));
        assert_eq!(r, Err("No auto-filled block there.".into()));
        assert_eq!(rip::picks_of(&s.w, VANGUARD), picks);
    }
    // a pick first, then the rest of the ghost: the pick stays first, the plan keeps its shape
    let mut s = setup(V2::new(4, 4), V2::new(1, 4));
    let sc = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[sc.clone()]).unwrap();
    let before = rows(&s.w, &s.plan, VANGUARD);
    rip::accept_ghost(&mut s.w, &mut s.plan, VANGUARD, None).0.unwrap();
    assert_eq!(rip::picks_of(&s.w, VANGUARD)[0], sc);
    assert_eq!(shape(&rows(&s.w, &s.plan, VANGUARD)), shape(&before));
    // nothing auto-filled left: refused
    let (r, _) = rip::accept_ghost(&mut s.w, &mut s.plan, VANGUARD, None);
    assert_eq!(r, Err("Nothing auto-filled to accept.".into()));
}

// ---- VISCLAR1-D: the menu's Move, which replaces the last move or filled block; the icon menu's cells ------------------

/// `c`'s Assault order is `fam` alone (the fill's follow-ups).
fn assault_order(s: &mut Session, c: UnitId, fam: &str) {
    s.w.units[c].deck.order.insert("assault".into(), vec![fam.to_string()]);
}

/// A clear board round VANGUARD on (4, 4) with FOOTMAN beside it.
fn side_by_side() -> Session {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    s
}

/// A Move after a skill takes the first slot of the last filled block, here Heavy Cleave, at 1 AP a tile; Rest fills after it,
/// the note names what it replaced, the block's AP comes back; undo gives the block back.
#[test]
fn the_move_takes_the_slot_of_the_last_filled_block() {
    let mut s = side_by_side();
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener.clone(), opener]).unwrap();
    let before = rows(&s.w, &s.plan, VANGUARD);
    let need = need_of(&s.w, VANGUARD, "tl_heavy_cleave");
    let last = before.last().unwrap();
    assert_eq!((last.0.as_str(), last.1, last.2, last.6), ("tl_heavy_cleave", BLOCKS - need + 1, BLOCKS, true), "the fill ends in the large block: {before:?}");
    assert_eq!(rip::move_note(&s.plan, VANGUARD), None);

    let step = V2::new(3, 4);
    pick(&mut s, VANGUARD, RoundTarget::Cell(step), &[Pick::StepAfter(step)]).unwrap();
    let after = rows(&s.w, &s.plan, VANGUARD);
    let kept = before.len() - 1;
    assert_eq!(after[..kept], before[..kept], "the fill before it stays");
    let mv = &after[kept];
    assert_eq!((mv.0.as_str(), mv.1, mv.3, mv.4, mv.6), ("move", before[kept].1, 1, step, false), "a picked move in the block's first slot at 1 AP a tile: {after:?}");
    let spare = &after[kept + 1..];
    assert_eq!(spare.len() as i32, need - 1, "the spare blocks: {after:?}");
    assert!(spare.iter().all(|x| x.0 == "rest" && x.6 && x.5.starts_with(rip::MOVE_REPLACED)), "ghost Rest after it, tagged: {after:?}");
    assert_eq!(rip::move_note(&s.plan, VANGUARD), Some(format!("Move replaces tl_heavy_cleave: {} blocks left as Rest", need - 1)));
    let spent = rip::ap_forecast(&s.w, &s.plan, VANGUARD).spent;
    assert_eq!(spent, 2 * ap_of(&s.w, VANGUARD, "tl_shoulder_check") + 1, "the cleave's AP came back; the move costs its tile");

    rip::unpick(&mut s.w, &mut s.plan, VANGUARD).0.unwrap();
    assert_eq!(rows(&s.w, &s.plan, VANGUARD), before, "undo restores the block");
    assert_eq!(rip::move_note(&s.plan, VANGUARD), None);
}

/// A block no larger than the move: it goes alone, with no Rest and no note, and the fill before it stays.
#[test]
fn the_move_over_one_block_leaves_no_rest() {
    let mut s = side_by_side();
    assault_order(&mut s, VANGUARD, "tl_shoulder_check");
    assert_eq!(need_of(&s.w, VANGUARD, "tl_shoulder_check"), 1, "the fixture's one-block skill");
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    let before = rows(&s.w, &s.plan, VANGUARD);
    assert!(before.iter().all(|x| x.0 == "tl_shoulder_check") && before.last().unwrap().2 == BLOCKS, "{before:?}");
    let step = V2::new(3, 4);
    pick(&mut s, VANGUARD, RoundTarget::Cell(step), &[Pick::StepAfter(step)]).unwrap();
    let after = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!(after[..before.len() - 1], before[..before.len() - 1]);
    assert_eq!((after.len(), after.last().unwrap().0.as_str(), after.last().unwrap().1), (before.len(), "move", BLOCKS));
    assert_eq!(rip::move_note(&s.plan, VANGUARD), None);
}

/// The Move's blocks follow from the distance, up to the rule's cap at the move's own pace: at the cap's reach it plans, one
/// tile further it is refused, and a cap of one holds it to one block. A Move as the opener keeps its two.
#[test]
fn the_move_runs_up_to_the_cap() {
    let mpb = |s: &Session| s.w.units[VANGUARD].move_per_block();
    let cap = |s: &Session| s.w.ruleset.numbers.move_after_blocks;
    let open = |s: &mut Session| {
        place(&mut s.w, VANGUARD, V2::new(1, 4));
        place(&mut s.w, FOOTMAN, V2::new(2, 4));
        assault_order(s, VANGUARD, "tl_shoulder_check");
        let opener = sk(s, "tl_shoulder_check");
        pick(s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    };
    let mut s = session();
    open(&mut s);
    assert!(cap(&s) >= 2, "a cap above one block");
    let far = V2::new(1, 4 - mpb(&s).min(4));
    pick(&mut s, VANGUARD, RoundTarget::Cell(far), &[Pick::StepAfter(far)]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!(r.iter().filter(|x| x.0 == "move").count(), 1, "{r:?}");

    // the cap's reach: straight along an open row, `cap` blocks of `mpb` tiles
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(0, 4));
    place(&mut s.w, FOOTMAN, V2::new(0, 5));
    assault_order(&mut s, VANGUARD, "tl_shoulder_check");
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    let tiles = mpb(&s) * cap(&s);
    let at_cap = V2::new(tiles, 4);
    pick(&mut s, VANGUARD, RoundTarget::Cell(at_cap), &[Pick::StepAfter(at_cap)]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    let moves: Vec<&Row> = r.iter().filter(|x| x.0 == "move").collect();
    assert_eq!(moves.len() as i32, cap(&s), "one entry a move block: {r:?}");
    assert_eq!(moves.iter().map(|x| x.3).sum::<i32>(), tiles, "1 AP a tile: {r:?}");
    assert_eq!(moves.last().unwrap().4, at_cap);
    rip::unpick(&mut s.w, &mut s.plan, VANGUARD).0.unwrap();
    let past = V2::new(tiles + 1, 4);
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(past), &[Pick::StepAfter(past)]), Err(format!("Too far: {} move blocks > {}", cap(&s) + 1, cap(&s))));

    // the rule value holds it
    s.w.ruleset.numbers.move_after_blocks = 1;
    let two_blocks = V2::new(mpb(&s) + 1, 4);
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(two_blocks), &[Pick::StepAfter(two_blocks)]), Err("Too far: 2 move blocks > 1".to_string()));

    // the opener Move stays at two blocks
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(0, 4));
    let most = s.plan.max_move_tiles(&s.w, VANGUARD);
    let beyond = V2::new(most + 1, 4);
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(beyond), &[Pick::MoveHere(beyond)]), Err(format!("Too far: {} tiles > {most} (two move blocks)", most + 1)));
}

/// The cap gives way to the blocks left after the picks, and to the AP left, with no debt.
#[test]
fn the_move_is_capped_by_blocks_and_ap() {
    let start = |ap: i32, picks: usize| {
        let mut s = session();
        place(&mut s.w, VANGUARD, V2::new(0, 4));
        place(&mut s.w, FOOTMAN, V2::new(0, 5));
        assault_order(&mut s, VANGUARD, "tl_shoulder_check");
        s.w.units[VANGUARD].ap = ap;
        for _ in 0..picks {
            let opener = sk(&s, "tl_shoulder_check");
            pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
        }
        s
    };
    let sc = ap_of(&start(UNIT_AP, 0).w, VANGUARD, "tl_shoulder_check");
    let mpb = start(UNIT_AP, 0).w.units[VANGUARD].move_per_block();
    let three = V2::new(2 * mpb + 1, 4);
    // two free blocks left: a tile that takes three is refused
    let mut s = start(UNIT_AP, BLOCKS as usize - 2);
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(three), &[Pick::StepAfter(three)]), Err("Blocks 2 < 3 (the picks fill the round)".to_string()));
    // the AP left pays for every tile, or the tile is refused
    let tiles = three.x;
    let mut s = start(sc + tiles - 1, 1);
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(three), &[Pick::StepAfter(three)]), Err(format!("AP {} < {tiles}", tiles - 1)));
    let mut s = start(sc + tiles, 1);
    pick(&mut s, VANGUARD, RoundTarget::Cell(three), &[Pick::StepAfter(three)]).unwrap();
    assert_eq!(rip::ap_forecast(&s.w, &s.plan, VANGUARD).left, 0, "every AP spent and no debt");
}

/// After the Move only Rest: a skill, an item, a second Move and the Fill are refused with a short reason, and the Rest picked
/// is a pick, in the block after the Move.
#[test]
fn only_rest_follows_the_move() {
    let mut s = side_by_side();
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener.clone(), opener.clone()]).unwrap();
    let step = V2::new(3, 4);
    pick(&mut s, VANGUARD, RoundTarget::Cell(step), &[Pick::StepAfter(step)]).unwrap();
    assert_eq!(pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]), Err(rip::ONLY_REST.to_string()));
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(V2::new(3, 3)), &[Pick::StepAfter(V2::new(3, 3))]), Err("One step after a round".to_string()));
    assert_eq!(rip::accept_ghost(&mut s.w, &mut s.plan, VANGUARD, None).0, Err(rip::ONLY_REST.to_string()));
    let move_block = rows(&s.w, &s.plan, VANGUARD).iter().find(|x| x.0 == "move").map(|x| x.1).unwrap();
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[Pick::Rest]).unwrap();
    let r = rows(&s.w, &s.plan, VANGUARD);
    let rest = r.iter().find(|x| x.1 == move_block + 1).unwrap();
    assert_eq!((rest.0.as_str(), rest.6), ("rest", false), "a picked Rest after the Move: {r:?}");
    let cells = rip::menu_options(&s.w, &s.plan, VANGUARD, foe(FOOTMAN));
    assert!(cells.iter().filter(|o| o.picks[0].holds_target()).all(|o| o.problem.is_some()), "every skill fades: {cells:?}");
    assert!(cells.iter().any(|o| o.picks == [Pick::Rest] && o.problem.is_none()), "Rest stays");
}

/// When the Move needs two blocks and takes two fills back for them, the note names both, in round order.
#[test]
fn the_note_names_every_block_it_replaced() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(0, 4));
    place(&mut s.w, FOOTMAN, V2::new(0, 5));
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    let before = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!(before[1..].iter().map(|x| x.0.as_str()).collect::<Vec<_>>(), ["tl_heavy_cleave", "tl_shoulder_check"], "the fixture's fill");
    let two = V2::new(s.w.units[VANGUARD].move_per_block() + 1, 4);
    // The tile says so before the click: a far tile can cost more than the last block
    let near = V2::new(1, 4);
    let tiles = rip::move_tiles(&s.w, &s.plan, VANGUARD);
    let tile = |at: V2| tiles.iter().find(|t| t.cell == at).unwrap().drops.clone();
    assert_eq!(tile(two), "tl_heavy_cleave and tl_shoulder_check", "{tiles:?}");
    assert_eq!(tile(near), "tl_shoulder_check", "{tiles:?}");
    pick(&mut s, VANGUARD, RoundTarget::Cell(two), &[Pick::StepAfter(two)]).unwrap();
    let free = BLOCKS - before[0].2 - 2;
    assert_eq!(rip::move_note(&s.plan, VANGUARD), Some(format!("Move replaces tl_heavy_cleave and tl_shoulder_check: {free} blocks left as Rest")));
}

/// The longest Move plays as it previews: three move blocks after the opener end on the picked tile, with no AP debt.
#[test]
fn a_three_block_move_plays_as_it_previews() {
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(0, 4));
    place(&mut s.w, FOOTMAN, V2::new(0, 5));
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    let far = V2::new(2 * s.w.units[VANGUARD].move_per_block() + 1, 4);
    let tiles = rip::move_tiles(&s.w, &s.plan, VANGUARD);
    let tile = tiles.iter().find(|t| t.cell == far).unwrap_or_else(|| panic!("{far} in {tiles:?}"));
    assert_eq!(tile.blocks, 3, "{tile:?}");
    pick(&mut s, VANGUARD, RoundTarget::Cell(far), &[Pick::StepAfter(far)]).unwrap();
    assert!(s.confirm_plan());
    while s.phase != rpgkit_battle::session::Phase::Replay {
        s.step_next();
    }
    assert_eq!(s.w.pos(VANGUARD), far);
    assert!(s.w.units[VANGUARD].ap >= 0);
}

/// The AP for the move comes back from the blocks it takes: with the fill spending every AP the last block goes though a free
/// block was there.
#[test]
fn the_move_takes_a_block_back_for_its_ap() {
    let mut s = side_by_side();
    let unit = ap_of(&s.w, VANGUARD, "tl_shoulder_check");
    assault_order(&mut s, VANGUARD, "tl_shoulder_check");
    s.w.units[VANGUARD].ap = 3 * unit;
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    let before: Vec<String> = rows(&s.w, &s.plan, VANGUARD).iter().map(|x| x.0.clone()).collect();
    assert_eq!(before.iter().filter(|x| *x == "tl_shoulder_check").count(), 3, "the fill runs out of AP: {before:?}");
    let step = V2::new(3, 4);
    pick(&mut s, VANGUARD, RoundTarget::Cell(step), &[Pick::StepAfter(step)]).unwrap();
    let f = rip::ap_forecast(&s.w, &s.plan, VANGUARD);
    assert!(f.left >= 0, "no debt: {f:?}");
    let after = rows(&s.w, &s.plan, VANGUARD);
    assert!(after.iter().filter(|x| x.0 == "tl_shoulder_check").count() < 3, "a block went for the move: {after:?}");
    assert!(rip::move_note(&s.plan, VANGUARD).is_some());
}

/// The last move: a ranged unit's auto-filled kite step (AS3) is replaced in place; the fill before it stays.
#[test]
fn the_move_replaces_the_auto_filled_kite() {
    let mut s = session();
    place(&mut s.w, ARCHER, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(6, 4));
    let opener = sk(&s, "tl_aimed_shot");
    pick(&mut s, ARCHER, foe(FOOTMAN), &[opener]).unwrap();
    let before = rows(&s.w, &s.plan, ARCHER);
    let kite = before.iter().position(|x| x.5 == rip::KITE_THREATENED).unwrap_or_else(|| panic!("the kite step is planned: {before:?}"));
    assert_eq!(rip::move_refusal(&s.w, &s.plan, ARCHER), None, "the planner's own move does not count as moved");
    let step = V2::new(4, 3);
    pick(&mut s, ARCHER, RoundTarget::Cell(step), &[Pick::StepAfter(step)]).unwrap();
    let after = rows(&s.w, &s.plan, ARCHER);
    assert_eq!(after[..kite], before[..kite], "the fill before it stays");
    assert_eq!((after[kite].0.as_str(), after[kite].1, after[kite].4, after[kite].5.as_str(), after[kite].6), ("move", before[kite].1, step, rip::STEP_AFTER, false), "the pick in its block: {after:?}");
    assert_eq!(rip::move_note(&s.plan, ARCHER), None);
}

/// Move's validity: free before an opener and after an opener that stays; refused once the unit has moved (the opener's own
/// approach, a picked move), after a Rest first, when the picks fill the round and at no AP.
#[test]
fn the_menu_move_is_valid_only_if_the_unit_has_not_moved() {
    let mut s = side_by_side();
    assert_eq!(rip::move_refusal(&s.w, &s.plan, VANGUARD), None, "no pick yet");
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    assert_eq!(rip::move_refusal(&s.w, &s.plan, VANGUARD), None, "the opener stayed");
    let step = V2::new(3, 4);
    pick(&mut s, VANGUARD, RoundTarget::Cell(step), &[Pick::StepAfter(step)]).unwrap();
    assert_eq!(rip::move_refusal(&s.w, &s.plan, VANGUARD).as_deref(), Some("One step after a round"));

    // the opener walks: Move fades, on a unit's menu and a tile's alike, in the same words as the pick's refusal
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(2, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    let why = "Already moved this round (approach)";
    assert_eq!(rip::move_refusal(&s.w, &s.plan, VANGUARD).as_deref(), Some(why));
    let tile = V2::new(2, 3);
    assert_eq!(pick(&mut s, VANGUARD, RoundTarget::Cell(tile), &[Pick::StepAfter(tile)]), Err(why.to_string()));
    let cell = rip::menu_options(&s.w, &s.plan, VANGUARD, RoundTarget::Cell(tile)).into_iter().find(|o| o.picks == [Pick::StepAfter(tile)]).expect("the Move stays in the menu");
    assert_eq!(cell.problem.as_deref(), Some(why));

    // Rest first, then Move: as Move here's own rule
    let mut s = session();
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[Pick::Rest]).unwrap();
    assert_eq!(rip::move_refusal(&s.w, &s.plan, VANGUARD).as_deref(), Some("Move here comes first in a round"));

    // the picks fill the round
    let mut s = side_by_side();
    assault_order(&mut s, VANGUARD, "tl_shoulder_check");
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    rip::accept_ghost(&mut s.w, &mut s.plan, VANGUARD, None).0.unwrap();
    assert!(rip::move_refusal(&s.w, &s.plan, VANGUARD).is_some_and(|w| w.starts_with("Blocks 0")));

    // no AP left after the picks
    let mut s = side_by_side();
    s.w.units[VANGUARD].ap = ap_of(&s.w, VANGUARD, "tl_shoulder_check");
    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    assert_eq!(rip::move_refusal(&s.w, &s.plan, VANGUARD).as_deref(), Some("AP 0 < 1"));
}

/// The icon menu's cells are one list: the same picks in the same order before and after an opener. A cell is lit only when
/// `pick_options` has it on the clicked target: on a foe a Guard skill is grey and says what it needs, on the unit itself it is
/// lit; once the opener fixes the intent it fades with why.
#[test]
fn the_menu_cells_are_one_list_lit_by_the_target() {
    let mut s = side_by_side();
    let key = |os: &[rip::PickOption]| -> Vec<String> { os.iter().map(|o| format!("{:?}", o.picks.last())).collect() };
    let open = rip::menu_options(&s.w, &s.plan, VANGUARD, foe(FOOTMAN));
    for sid in rip::hotkey_skills(&s.w, VANGUARD) {
        assert!(open.iter().any(|o| matches!(o.picks[..], [Pick::Skill { skill, .. }] if skill == sid)), "every skill has a cell");
    }
    assert!(open.iter().any(|o| o.picks == [Pick::Rest] && o.problem.is_none()));
    let offered = rip::pick_options(&s.w, &s.plan, VANGUARD, foe(FOOTMAN));
    for row in &offered {
        let o = open.iter().find(|o| o.picks == row.picks).expect("the kit's row has its cell");
        assert_eq!((&o.problem, o.ap, o.blocks, o.at), (&row.problem, row.ap, row.blocks, foe(FOOTMAN)), "the same answer on the same target");
    }
    for o in open.iter().filter(|o| o.problem.is_none()) {
        assert!(offered.iter().any(|r| r.picks == o.picks), "lit only with a row of `pick_options`: {o:?}");
    }
    let guard = sid(&s.w, "tl_brace_counter");
    let is_guard = |o: &&rip::PickOption| matches!(o.picks[..], [Pick::Skill { skill, .. }] if skill == guard);
    let cell = open.iter().find(is_guard).expect("a Guard cell on a foe's menu");
    assert_eq!((cell.intent, cell.at, cell.problem.as_deref()), (Some(RoundIntent::Guard), foe(FOOTMAN), Some("Target yourself to use this")), "grey on a foe");
    let on_self = rip::menu_options(&s.w, &s.plan, VANGUARD, RoundTarget::Myself);
    assert_eq!(on_self.iter().find(is_guard).unwrap().problem, None, "lit on the unit itself");
    assert_eq!(key(&on_self), key(&open), "the same cells on another target");

    let opener = sk(&s, "tl_shoulder_check");
    pick(&mut s, VANGUARD, foe(FOOTMAN), &[opener]).unwrap();
    let fixed = rip::menu_options(&s.w, &s.plan, VANGUARD, foe(FOOTMAN));
    assert_eq!(key(&fixed), key(&open), "the same cells in the same order");
    let cell = fixed.iter().find(is_guard).unwrap();
    assert!(cell.problem.as_deref().is_some_and(|p| p.starts_with("Only ")), "faded with why: {:?}", cell.problem);
    assert!(rip::menu_options(&s.w, &s.plan, VANGUARD, foe(FOOTMAN_B)).is_empty(), "another unit than the round target: no menu, as `pick_options`");
    // a tile's menu lists the same skills, Rest and then the move
    let tile = V2::new(3, 4);
    let on_tile = rip::menu_options(&s.w, &s.plan, VANGUARD, RoundTarget::Cell(tile));
    assert_eq!(key(&on_tile[..open.len()]), key(&open));
    assert_eq!(on_tile.last().map(|o| o.picks.clone()), Some(vec![Pick::StepAfter(tile)]));
}
