//! RULESET1-PL on a scratch catalog built here (no shipped data): the round intent auto planner of RULESET1_INTENT 4, 7
//! and 8 under the rule `round_intent` (with `ap_cost`, so the skills cost their RULESET1_TASKS 2.4 AP). Expectations
//! come from the rules: the four worked examples of section 4, determinism and reason tags (PL1, PL6), refusals (MV7,
//! MV8, PL7), the reserved kite step (AS3), Guard and Support rounds (GD1, GD2, SP1, SP2), the order lists (FU4), the
//! Disengage retreat and jump (MV8), the auto kind (MV11), the rebuild of later members (PL8), the proposed target
//! (TG6) and manual mode (MM1-MM4).

use indexmap::IndexMap;
use rpgkit_battle::command::Carry;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanEntry, PlanStep};
use rpgkit_battle::round_intent::{self, ManeuverKind, Opener, RoundIntent, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, ItemDef};
use rpgkit_rules::ruleset::{Ruleset, AP_COST, FLANK, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::rc::Rc;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;
const BASICS: [&str; 5] = ["move", "rest", "strike", "active_guard", "flank_cut"];

/// Engine skills and the test kit's skills (RULESET1_TASKS 2.4 numbers; `tl_backflip` is a DISENGAGE jump for MV8).
fn catalog() -> Catalog {
    let atk = |id: &str, intent: &str, blocks: i32, rec: i32, ap: i32, power: f64, dash: i32| {
        json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": intent, "level": 1, "blocks": blocks, "recovery_blocks": rec, "ap_cost": ap, "power_mult": power, "dash_tiles": dash, "target_mode": "ENEMY"})
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
        atk("flank_cut", "maneuver", 1, 1, 2, 0.6, 2),
        json!({"id": "tl_brace_counter", "name": "Brace and Counter", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 2, "mitigation": 0.3, "grant_status": "riposte", "status_rounds": 1}),
        json!({"id": "tl_field_dressing", "name": "Field Dressing", "kind": "SUPPORT", "round_intent": "support", "level": 1, "blocks": 2, "ap_cost": 2, "heal_base": 8, "heal_mult": 0.3, "target_mode": "ALLY_OR_SELF"}),
        atk("tl_hamstring_cut", "maneuver", 1, 1, 2, 0.8, 3),
        atk("tl_slip_strike", "maneuver", 2, 1, 2, 0.9, 2),
        atk("tl_heavy_cleave", "assault", 4, 1, 4, 2.0, 0),
        atk("tl_shoulder_check", "assault", 1, 0, 1, 0.5, 0),
        atk("tl_snap_shot", "assault", 1, 1, 2, 0.6, 0),
        atk("tl_aimed_shot", "assault", 2, 1, 2, 1.1, 0),
        json!({"id": "tl_backflip", "name": "Backflip", "kind": "DISENGAGE", "round_intent": "maneuver", "level": 1, "blocks": 1, "ap_cost": 1, "jump_tiles": 2}),
    ];
    let unit = |id: &str, side: &str, range: i32, order: serde_json::Value| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": range, "skill_cap": 5, "deck": {"order": order}})
    };
    let units = vec![
        unit("tl_vanguard", "party", 1, json!({"assault": ["tl_heavy_cleave", "tl_shoulder_check", "strike"], "guard": ["tl_brace_counter", "active_guard"]})),
        unit("tl_archer", "party", 3, json!({"assault": ["tl_aimed_shot", "tl_snap_shot"], "support": ["tl_field_dressing"]})),
        unit("tl_spiker", "party", 1, json!({"assault": ["strike"], "maneuver": ["tl_hamstring_cut", "tl_slip_strike", "flank_cut"]})),
        unit("t_longbow", "party", 5, json!({"assault": ["tl_aimed_shot", "tl_snap_shot"]})),
        unit("t_plain", "party", 1, json!({})),
        unit("t_dancer", "party", 1, json!({"maneuver": ["tl_slip_strike", "tl_backflip"]})),
        unit("tl_footman", "enemy", 1, json!({})),
        unit("tl_footman_b", "enemy", 1, json!({})),
    ];
    Catalog::from_value(&json!({"skills": skills, "units": units}))
}

const VANGUARD: UnitId = 0;
const ARCHER: UnitId = 1;
const SPIKER: UnitId = 2;
const LONGBOW: UnitId = 3;
const PLAIN: UnitId = 4;
const DANCER: UnitId = 5;
const FOOTMAN: UnitId = 6;
const FOOTMAN_B: UnitId = 7;

/// A timeline ruleset with the round intent, `ap_cost` and the flank bonus, and the test kit's basics (2.1).
fn timeline() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, FLANK], "basics": BASICS}).as_object().unwrap())
}

/// Every unit placed off in a corner or along the edges (tests place the ones they use).
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["tl_vanguard", "tl_archer", "tl_spiker", "t_longbow", "t_plain", "t_dancer"],
        "enemies": ["tl_footman", "tl_footman_b"], "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": 0, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    let parked = [V2::new(0, 0), V2::new(0, 8), V2::new(2, 0), V2::new(2, 8), V2::new(4, 0), V2::new(4, 8), V2::new(11, 0), V2::new(11, 8)];
    for (c, cell) in parked.into_iter().enumerate() {
        place(&mut s.w, c, cell);
    }
    s
}

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn intent(w: &mut World, c: UnitId, i: RoundIntent, t: RoundTarget) {
    round_intent::set_intent(w, c, i).expect("intent");
    round_intent::set_target(w, c, t).expect("target");
}

fn kind_of(w: &World, e: &PlanEntry) -> Kind {
    skill::kind(&w.cat.skills[e.skill])
}

fn ap_of(w: &World, c: UnitId, id: &str) -> i32 {
    skill::ap_cost_by(&w.cat.skills[sid(w, id)], Some(&w.units[c]), &w.ruleset)
}

/// (skill id, start, end, AP, tile after, reason) per entry of `c`'s plan.
fn rows(w: &World, plan: &Plan, c: UnitId) -> Vec<(String, i32, i32, i32, V2, String)> {
    let reasons = rip::reasons(w, plan, c);
    plan.entries_for(w, c).iter().zip(reasons).map(|(e, r)| (w.cat.skills[e.skill].id.clone(), e.start_block, e.end_block(w), e.total_ap(), e.pos_after, r)).collect()
}

fn auto(s: &mut Session, c: UnitId) {
    let (w, plan) = (&mut s.w, &mut s.plan);
    rip::plan_unit(w, plan, c).unwrap_or_else(|e| panic!("planned: {e}"));
}

/// Section 4's first example: tl_vanguard, 10 AP, the footman 3 tiles away, Assault: b1 approach 2 tiles (2 AP),
/// b2-b5 Heavy Cleave (4), b6 Shoulder Check (1); 3 AP left. The third: 3 AP, the footman adjacent: Shoulder Check in
/// b1-b3 (Heavy Cleave unaffordable), Rest in b4-b6 with the first refused entry's cause, 0 AP, then the Rest gain.
#[test]
fn worked_examples_vanguard() {
    let mut s = session(timeline());
    let foe_at = V2::new(5, 4);
    place(&mut s.w, VANGUARD, V2::new(2, 4));
    place(&mut s.w, FOOTMAN, foe_at);
    intent(&mut s.w, VANGUARD, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
    auto(&mut s, VANGUARD);
    let w = &s.w;
    let r = rows(w, &s.plan, VANGUARD);
    let names: Vec<&str> = r.iter().map(|x| x.0.as_str()).collect();
    assert_eq!(names, ["move", "tl_heavy_cleave", "tl_shoulder_check"], "{r:?}");
    assert_eq!((r[0].1, r[0].3, r[0].4.dist(foe_at), r[0].5.as_str()), (1, 2, 1, rip::APPROACH), "b1: approach 2 tiles into band");
    assert_eq!((r[1].1, r[1].2, r[1].5.as_str()), (2, 5, rip::OPENER));
    assert_eq!((r[2].1, r[2].5.as_str()), (6, "follow-up 2"), "the first order entry that fits: Heavy Cleave does not");
    let st = s.plan.compile(w, VANGUARD);
    assert_eq!(st.ap_left, UNIT_AP - 2 - ap_of(w, VANGUARD, "tl_heavy_cleave") - ap_of(w, VANGUARD, "tl_shoulder_check"));
    assert_eq!(st.ap_left, 3, "the spec's AP left");

    let mut s = session(timeline());
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, foe_at);
    s.w.units[VANGUARD].ap = 3;
    intent(&mut s.w, VANGUARD, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
    auto(&mut s, VANGUARD);
    let w = &s.w;
    let r = rows(w, &s.plan, VANGUARD);
    let names: Vec<&str> = r.iter().map(|x| x.0.as_str()).collect();
    assert_eq!(names, ["tl_shoulder_check", "tl_shoulder_check", "tl_shoulder_check", "rest", "rest", "rest"], "{r:?}");
    assert_eq!(r.iter().map(|x| x.1).collect::<Vec<_>>(), [1, 2, 3, 4, 5, 6]);
    let heavy = ap_of(w, VANGUARD, "tl_heavy_cleave");
    let need = w.cat.skills[sid(w, "tl_heavy_cleave")].blocks;
    assert_eq!(r[3].5, format!("rest: blocks 3 < {need}"), "Heavy Cleave, the first entry, needs more blocks than are left");
    assert!(heavy > 3, "and was unaffordable as the opener");
    assert_eq!(s.plan.compile(w, VANGUARD).ap_left, 0);
    assert_eq!(round_intent::rest_gain(&w.ruleset.numbers, 3), 1, "then Rest +1 at round end");
    // UI2: the battle screen's Rest gain counts the planned Rest blocks; no poise without the rule poise
    let rest = rip::rest_preview(w, &s.plan, VANGUARD);
    assert_eq!(rest, rip::RestGain { blocks: 3, ap: round_intent::rest_gain(&w.ruleset.numbers, 3), poise: 0 });
}

/// Section 4's second example: tl_archer, 10 AP, the footman adjacent, Assault: b1 kite 2 tiles (2 AP) to the farthest
/// band tile, b2-b3 and b4-b5 Aimed Shot (2 each), b6 Snap Shot (2); 2 AP left.
#[test]
fn worked_example_archer_kites() {
    let mut s = session(timeline());
    let foe_at = V2::new(5, 4);
    place(&mut s.w, ARCHER, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, foe_at);
    intent(&mut s.w, ARCHER, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
    auto(&mut s, ARCHER);
    let w = &s.w;
    let r = rows(w, &s.plan, ARCHER);
    let names: Vec<&str> = r.iter().map(|x| x.0.as_str()).collect();
    assert_eq!(names, ["move", "tl_aimed_shot", "tl_aimed_shot", "tl_snap_shot"], "{r:?}");
    let band = skill::reach_for(&w.cat.skills[sid(w, "tl_aimed_shot")], &w.units[ARCHER]);
    assert_eq!((r[0].1, r[0].3, r[0].4.dist(foe_at), r[0].5.as_str()), (1, 2, band.1, rip::KITE_IN_MELEE), "kite to the farthest band tile");
    assert_eq!(r.iter().skip(1).map(|x| (x.1, x.2)).collect::<Vec<_>>(), [(2, 3), (4, 5), (6, 6)]);
    assert_eq!(r.iter().skip(1).map(|x| x.5.as_str()).collect::<Vec<_>>(), [rip::OPENER, "follow-up 1", "follow-up 2"]);
    assert_eq!(s.plan.compile(w, ARCHER).ap_left, 2);
}

/// Section 4's fourth example: tl_spiker, 6 AP, the footman 3 tiles away and engaged with tl_vanguard, Maneuver Engage:
/// b1 Hamstring Cut carried to a flank tile (2), b2 and b3 Hamstring Cut from there (2 each), b4-b6 Rest; 0 AP.
#[test]
fn worked_example_spiker_engages() {
    let mut s = session(timeline());
    let foe_at = V2::new(6, 4);
    let start = V2::new(4, 3);
    place(&mut s.w, SPIKER, start);
    place(&mut s.w, VANGUARD, V2::new(5, 4));
    place(&mut s.w, FOOTMAN, foe_at);
    s.w.units[SPIKER].ap = 6;
    intent(&mut s.w, FOOTMAN, RoundIntent::Assault, RoundTarget::Unit(VANGUARD));
    intent(&mut s.w, SPIKER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    round_intent::set_maneuver(&mut s.w, SPIKER, ManeuverKind::Engage).unwrap();
    assert_eq!(start.dist(foe_at), 3);
    auto(&mut s, SPIKER);
    let w = &s.w;
    let entries = s.plan.entries_for(w, SPIKER);
    let r = rows(w, &s.plan, SPIKER);
    let names: Vec<&str> = r.iter().map(|x| x.0.as_str()).collect();
    assert_eq!(names, ["tl_hamstring_cut", "tl_hamstring_cut", "tl_hamstring_cut", "rest", "rest", "rest"], "{r:?}");
    let ham = &w.cat.skills[sid(w, "tl_hamstring_cut")];
    let flank = entries[0].pos_after;
    assert!(entries[0].carry == Carry::Engage(flank) && entries[0].path.len() as i32 <= ham.dash_tiles, "carried in its own block");
    assert!(flank.dist(foe_at) == 1 && !round_intent::in_front(w, FOOTMAN, flank), "a flank tile in band: {flank}");
    assert_eq!(r[0].5, format!("{}; {}", rip::OPENER, rip::FLANK_TILE));
    assert!(entries[1..3].iter().all(|e| e.pos_after == flank && e.path.is_empty()), "the follow-ups stay: the tile still meets the condition");
    assert_eq!((r[1].5.as_str(), r[2].5.as_str()), ("follow-up 1", "follow-up 1"));
    assert_eq!(r[3].5, format!("rest: AP 0 < {}", ap_of(w, SPIKER, "tl_hamstring_cut")));
    assert_eq!(s.plan.compile(w, SPIKER).ap_left, 0);
}

/// PL1, PL6: the same world and inputs give the same steps, entries and reason tags.
#[test]
fn the_same_inputs_give_the_same_plan() {
    let build = || {
        let mut s = session(timeline());
        place(&mut s.w, VANGUARD, V2::new(2, 4));
        place(&mut s.w, ARCHER, V2::new(4, 4));
        place(&mut s.w, FOOTMAN, V2::new(5, 4));
        intent(&mut s.w, VANGUARD, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
        intent(&mut s.w, ARCHER, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
        let (w, plan) = (&mut s.w, &mut s.plan);
        let _ = rip::replan(w, plan, VANGUARD);
        (rows(&s.w, &s.plan, VANGUARD), rows(&s.w, &s.plan, ARCHER))
    };
    let (a, b) = (build(), build());
    assert!(!a.0.is_empty() && !a.1.is_empty());
    assert_eq!(a, b);
}

/// MV7, MV8, PL7: Engage with no flank tile within range (even after the plain move first, MV9 a) and Disengage with
/// the target out of band are not offered: the reason names the kind and the target, the plan stays empty, and a
/// preview changes nothing.
#[test]
fn engage_and_disengage_out_of_range_are_not_offered() {
    let mut s = session(timeline());
    place(&mut s.w, SPIKER, V2::new(0, 4));
    place(&mut s.w, FOOTMAN, V2::new(11, 4));
    intent(&mut s.w, SPIKER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    round_intent::set_maneuver(&mut s.w, SPIKER, ManeuverKind::Engage).unwrap();
    let (w, plan) = (&mut s.w, &mut s.plan);
    let e = rip::plan_unit(w, plan, SPIKER).unwrap_err();
    assert_eq!(e, "No Engage skill reaches tl_footman this round.");
    assert!(s.plan.steps_of(SPIKER).is_empty());
    // Disengage: the target 3 tiles away, outside every Maneuver skill's band from the unit's tile
    place(&mut s.w, FOOTMAN, V2::new(3, 4));
    round_intent::set_maneuver(&mut s.w, SPIKER, ManeuverKind::Disengage).unwrap();
    let before = s.w.round_states.clone();
    let p = rip::preview(&s.w, &s.plan, SPIKER, RoundIntent::Maneuver, Some(ManeuverKind::Disengage));
    assert_eq!(p.unwrap_err(), "No Disengage skill reaches tl_footman this round.");
    assert_eq!(s.w.round_states, before, "a preview changes nothing");
    let (w, plan) = (&mut s.w, &mut s.plan);
    assert!(rip::plan_unit(w, plan, SPIKER).is_err());
    assert!(s.plan.steps_of(SPIKER).is_empty());
}

/// AS3: a ranged unit in band of its target and threatened by it (distance at most the target's 2-block move plus its
/// reach) keeps the last block for a step to a farther band tile after its attacks; the fill stops before it.
#[test]
fn a_threatened_ranged_unit_kites_after_its_attacks() {
    let mut s = session(timeline());
    let foe_at = V2::new(6, 4);
    place(&mut s.w, LONGBOW, V2::new(2, 4));
    place(&mut s.w, FOOTMAN, foe_at);
    let here = V2::new(2, 4).dist(foe_at);
    let f = &s.w.units[FOOTMAN];
    assert!(here <= f.move_per_block() * 2 + f.attack_range, "the footman threatens the tile");
    intent(&mut s.w, LONGBOW, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
    auto(&mut s, LONGBOW);
    let w = &s.w;
    let entries = s.plan.entries_for(w, LONGBOW);
    let r = rows(w, &s.plan, LONGBOW);
    let band = skill::reach_for(&w.cat.skills[sid(w, "tl_aimed_shot")], &w.units[LONGBOW]);
    assert!(here >= band.0 && here <= band.1, "in band: no approach");
    let last = entries.last().unwrap();
    assert_eq!((kind_of(w, last), last.start_block, r.last().unwrap().5.as_str()), (Kind::Move, BLOCKS, rip::KITE_THREATENED), "{r:?}");
    let d = last.pos_after.dist(foe_at);
    assert!(d > here && d <= band.1, "a farther band tile: {}", last.pos_after);
    assert!(entries[..entries.len() - 1].iter().all(|e| kind_of(w, e) == Kind::Attack && e.last_block(w) < BLOCKS), "the attacks come first");
    // a melee unit never kites after (no band tile is farther)
    let mut s = session(timeline());
    place(&mut s.w, VANGUARD, V2::new(5, 4));
    place(&mut s.w, FOOTMAN, foe_at);
    intent(&mut s.w, VANGUARD, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
    auto(&mut s, VANGUARD);
    assert!(s.plan.entries_for(&s.w, VANGUARD).iter().all(|e| kind_of(&s.w, e) != Kind::Move));
}

/// GD1, GD2: Guard on self holds in place and fills from the Guard order (an unaffordable entry falls to the next;
/// Rest after with the cause); the auto order repeats Active Guard so 3 AP cover six blocks; a tile target moves there
/// first.
#[test]
fn guard_holds_or_moves_to_its_tile_and_repeats_its_order() {
    let mut s = session(timeline());
    place(&mut s.w, VANGUARD, V2::new(3, 4));
    place(&mut s.w, FOOTMAN, V2::new(6, 4));
    s.w.units[VANGUARD].ap = 3;
    intent(&mut s.w, VANGUARD, RoundIntent::Guard, RoundTarget::Myself);
    auto(&mut s, VANGUARD);
    let w = &s.w;
    let r = rows(w, &s.plan, VANGUARD);
    let names: Vec<&str> = r.iter().map(|x| x.0.as_str()).collect();
    assert_eq!(names, ["tl_brace_counter", "active_guard", "rest", "rest"], "{r:?}");
    assert!(r.iter().all(|x| x.4 == V2::new(3, 4)), "holds in place");
    assert_eq!(r[2].5, format!("rest: AP 0 < {}", ap_of(w, VANGUARD, "tl_brace_counter")));
    // the auto order (no deck): Active Guard, 2 blocks for 1 AP, three times
    let mut s = session(timeline());
    place(&mut s.w, PLAIN, V2::new(3, 4));
    s.w.units[PLAIN].known_skills = vec!["active_guard".into(), "strike".into()];
    s.w.units[PLAIN].ap = 3;
    intent(&mut s.w, PLAIN, RoundIntent::Guard, RoundTarget::Myself);
    auto(&mut s, PLAIN);
    let r = rows(&s.w, &s.plan, PLAIN);
    assert_eq!(r.iter().map(|x| (x.0.as_str(), x.1, x.2)).collect::<Vec<_>>(), [("active_guard", 1, 2), ("active_guard", 3, 4), ("active_guard", 5, 6)]);
    assert_eq!(s.plan.compile(&s.w, PLAIN).ap_left, 0);
    // a tile target: the standard move there first
    let mut s = session(timeline());
    place(&mut s.w, VANGUARD, V2::new(3, 4));
    let tile = V2::new(3, 6);
    intent(&mut s.w, VANGUARD, RoundIntent::Guard, RoundTarget::Cell(tile));
    auto(&mut s, VANGUARD);
    let r = rows(&s.w, &s.plan, VANGUARD);
    assert_eq!((r[0].0.as_str(), r[0].4, r[0].5.as_str()), ("move", tile, rip::COVER_TILE), "{r:?}");
    assert_eq!((r[1].0.as_str(), r[1].5.as_str()), ("tl_brace_counter", rip::OPENER));
}

/// SP1, SP2, FU2 (e): Support on an ally out of reach approaches into the skill's reach first, then heals and follows
/// up with Support skills on it; a pouch item opener is used once, never as a follow-up.
#[test]
fn support_approaches_and_spends_only_the_item_it_was_told() {
    let mut s = session(timeline());
    let ally_at = V2::new(6, 4);
    place(&mut s.w, ARCHER, V2::new(0, 4));
    place(&mut s.w, VANGUARD, ally_at);
    intent(&mut s.w, ARCHER, RoundIntent::Support, RoundTarget::Unit(VANGUARD));
    auto(&mut s, ARCHER);
    let w = &s.w;
    let entries = s.plan.entries_for(w, ARCHER);
    let r = rows(w, &s.plan, ARCHER);
    let reach = skill::support_reach(&w.cat.skills[sid(w, "tl_field_dressing")], &w.units[ARCHER]);
    let moves: Vec<_> = entries.iter().zip(&r).filter(|(e, _)| kind_of(w, e) == Kind::Move).collect();
    assert!(!moves.is_empty() && moves.iter().all(|(_, x)| x.5 == rip::APPROACH), "{r:?}");
    assert!(moves.last().unwrap().0.pos_after.dist(ally_at) <= reach.1, "into reach");
    let heals: Vec<_> = r.iter().filter(|x| x.0 == "tl_field_dressing").collect();
    assert!(heals.len() >= 2 && heals[0].5 == rip::OPENER && heals[1].5 == "follow-up 1", "{r:?}");
    // an item opener: once
    let mut s = session(timeline());
    place(&mut s.w, ARCHER, V2::new(4, 4));
    place(&mut s.w, VANGUARD, V2::new(5, 4));
    let potion = ItemDef::from_obj(json!({"id": "t_potion", "name": "Potion", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 20}).as_object().unwrap());
    for _ in 0..2 {
        assert!(s.w.units[ARCHER].inventory.add_to_pouch(&potion));
    }
    s.w.add_item_skills();
    intent(&mut s.w, ARCHER, RoundIntent::Support, RoundTarget::Unit(VANGUARD));
    rip::set_opener(&mut s.w, ARCHER, Some(Opener::Item("t_potion".into()))).unwrap();
    auto(&mut s, ARCHER);
    let w = &s.w;
    let entries = s.plan.entries_for(w, ARCHER);
    let r = rows(w, &s.plan, ARCHER);
    assert!(entries[0].item.is_some() && r[0].5 == rip::OPENER, "{r:?}");
    assert_eq!(entries.iter().filter(|e| e.item.is_some()).count(), 1, "the second potion stays in the pouch");
    assert!(r[1..].iter().any(|x| x.0 == "tl_field_dressing"), "follow-ups are skills");
}

/// MV10: a Utility round opened with a foe-aimed pouch item: items carry no move, so a standard approach into the
/// item's reach comes first; the follow-ups are Maneuver skills within range. Under Engage the item is refused.
#[test]
fn utility_throws_the_item_after_an_approach() {
    let mut s = session(timeline());
    let foe_at = V2::new(7, 4);
    place(&mut s.w, SPIKER, V2::new(1, 4));
    place(&mut s.w, FOOTMAN, foe_at);
    let bomb = ItemDef::from_obj(json!({"id": "t_bomb", "name": "Bomb", "item_type": "CONSUMABLE", "consumable_type": "THROWABLE_BOMB", "damage": 45}).as_object().unwrap());
    assert!(s.w.units[SPIKER].inventory.add_to_pouch(&bomb));
    s.w.add_item_skills();
    intent(&mut s.w, SPIKER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    rip::set_opener(&mut s.w, SPIKER, Some(Opener::Item("t_bomb".into()))).unwrap();
    round_intent::set_maneuver(&mut s.w, SPIKER, ManeuverKind::Engage).unwrap();
    let (w, plan) = (&mut s.w, &mut s.plan);
    assert_eq!(rip::plan_unit(w, plan, SPIKER).unwrap_err(), "Bomb opens only a Utility round.");
    round_intent::set_maneuver(w, SPIKER, ManeuverKind::Utility).unwrap();
    rip::plan_unit(w, plan, SPIKER).unwrap();
    let w = &s.w;
    let entries = s.plan.entries_for(w, SPIKER);
    let r = rows(w, &s.plan, SPIKER);
    let thrown = entries.iter().position(|e| e.item.is_some()).expect("the bomb");
    assert!(thrown > 0 && entries[..thrown].iter().all(|e| kind_of(w, e) == Kind::Move) && r[..thrown].iter().all(|x| x.5 == rip::APPROACH), "{r:?}");
    assert!(entries[thrown].pos_before.dist(foe_at) <= bomb.battle_reach(), "within the item's reach");
    assert_eq!(r[thrown].5, rip::OPENER);
    assert!(entries[thrown + 1..].iter().all(|e| kind_of(w, e) == Kind::Rest || round_intent::skill_intent(&w.cat.skills[e.skill]) == "maneuver"), "{r:?}");
}

/// FU4: a member's `deck.order` for the intent, else the auto order: its usable families of the intent that are not
/// basics, in catalog order, then the ruleset's basics of the intent in their order.
#[test]
fn the_order_is_the_deck_order_else_auto() {
    let mut s = session(timeline());
    let ids = |w: &World, l: &[usize]| l.iter().map(|&i| w.cat.skills[i].id.clone()).collect::<Vec<_>>();
    let w = &s.w;
    assert_eq!(ids(w, &rip::order(w, VANGUARD, RoundIntent::Assault)), ["tl_heavy_cleave", "tl_shoulder_check", "strike"]);
    assert_eq!(ids(w, &rip::order(w, VANGUARD, RoundIntent::Guard)), ["tl_brace_counter", "active_guard"]);
    let known = ["tl_shoulder_check", "tl_heavy_cleave", "tl_hamstring_cut", "strike", "active_guard", "flank_cut"];
    s.w.units[PLAIN].known_skills = known.iter().map(|k| k.to_string()).collect();
    let w = &s.w;
    for i in [RoundIntent::Assault, RoundIntent::Guard, RoundIntent::Maneuver] {
        let of = |id: &str| round_intent::skill_intent(w.cat.skill(id).unwrap()) == i.id();
        let mut want: Vec<String> = w.cat.skills.iter().filter(|s| known.contains(&s.id.as_str()) && !BASICS.contains(&s.id.as_str()) && of(&s.id)).map(|s| s.id.clone()).collect();
        want.extend(BASICS.iter().filter(|b| known.contains(b) && of(b)).map(|b| b.to_string()));
        assert_eq!(ids(w, &rip::order(w, PLAIN, i)), want, "{}", i.id());
    }
}

/// MV8: in a Disengage round the hits come from the unit's tile and the last Maneuver attack carries the retreat; a
/// DISENGAGE skill in the order goes last instead (its blocks kept back) and retreats with its jump.
#[test]
fn disengage_retreats_with_the_last_hit_or_the_jump() {
    let mut s = session(timeline());
    let (hero_at, foe_at) = (V2::new(4, 4), V2::new(5, 4));
    place(&mut s.w, SPIKER, hero_at);
    place(&mut s.w, FOOTMAN, foe_at);
    intent(&mut s.w, SPIKER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    round_intent::set_maneuver(&mut s.w, SPIKER, ManeuverKind::Disengage).unwrap();
    auto(&mut s, SPIKER);
    let w = &s.w;
    let entries = s.plan.entries_for(w, SPIKER);
    let r = rows(w, &s.plan, SPIKER);
    let hits: Vec<usize> = (0..entries.len()).filter(|&i| kind_of(w, &entries[i]) == Kind::Attack).collect();
    let last = *hits.last().unwrap();
    assert!(hits.len() >= 2 && hits[..hits.len() - 1].iter().all(|&i| entries[i].carry == Carry::None && entries[i].pos_after == hero_at), "{r:?}");
    let Carry::Retreat(cell) = entries[last].carry else { panic!("the last hit retreats: {r:?}") };
    assert!(cell.dist(foe_at) > hero_at.dist(foe_at));
    assert!(r[last].5.ends_with(rip::RETREAT), "{}", r[last].5);
    // with a jump in the order
    let mut s = session(timeline());
    place(&mut s.w, DANCER, hero_at);
    place(&mut s.w, FOOTMAN, foe_at);
    intent(&mut s.w, DANCER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    round_intent::set_maneuver(&mut s.w, DANCER, ManeuverKind::Disengage).unwrap();
    auto(&mut s, DANCER);
    let w = &s.w;
    let entries = s.plan.entries_for(w, DANCER);
    let r = rows(w, &s.plan, DANCER);
    let names: Vec<&str> = r.iter().map(|x| x.0.as_str()).collect();
    assert_eq!(names, ["tl_slip_strike", "tl_slip_strike", "tl_backflip", "rest"], "{r:?}");
    assert!(entries[..2].iter().all(|e| e.carry == Carry::None && e.pos_after == hero_at), "no carried retreat before a jump");
    assert_eq!((r[2].1, r[2].5.as_str()), (5, rip::RETREAT));
    assert!(entries[2].pos_after.dist(foe_at) > hero_at.dist(foe_at));
    let slip = &w.cat.skills[sid(w, "tl_slip_strike")];
    assert_eq!(r[3].5, format!("rest: blocks 1 < {}", slip.blocks));
    // the jump belongs to Disengage: an Engage round never plans it
    round_intent::set_maneuver(&mut s.w, DANCER, ManeuverKind::Engage).unwrap();
    auto(&mut s, DANCER);
    assert!(rows(&s.w, &s.plan, DANCER).iter().all(|x| x.0 != "tl_backflip"));
}

/// MV11: a Maneuver round without a picked kind takes Engage, else Disengage, else Utility; with none possible the
/// intent is refused and the kind left unpicked.
#[test]
fn the_auto_kind_is_engage_else_disengage() {
    let mut s = session(timeline());
    place(&mut s.w, SPIKER, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    intent(&mut s.w, SPIKER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    auto(&mut s, SPIKER);
    assert_eq!(round_intent::maneuver_of(&s.w, SPIKER), ManeuverKind::Engage);
    assert!(s.w.round_states[&SPIKER].maneuver_auto);
    // the footman in a corner facing the spiker, its one side tile taken: no flank tile, so Disengage
    let mut s = session(timeline());
    place(&mut s.w, SPIKER, V2::new(1, 0));
    place(&mut s.w, VANGUARD, V2::new(6, 6));
    place(&mut s.w, FOOTMAN, V2::new(0, 0));
    place(&mut s.w, FOOTMAN_B, V2::new(0, 1));
    intent(&mut s.w, FOOTMAN, RoundIntent::Assault, RoundTarget::Unit(SPIKER));
    intent(&mut s.w, SPIKER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    auto(&mut s, SPIKER);
    assert_eq!(round_intent::maneuver_of(&s.w, SPIKER), ManeuverKind::Disengage);
    // nothing possible
    let mut s = session(timeline());
    place(&mut s.w, SPIKER, V2::new(0, 4));
    place(&mut s.w, FOOTMAN, V2::new(11, 4));
    intent(&mut s.w, SPIKER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    let (w, plan) = (&mut s.w, &mut s.plan);
    assert!(rip::plan_unit(w, plan, SPIKER).unwrap_err().contains("Engage"));
    assert_eq!(s.w.round_states[&SPIKER].maneuver, None);
}

/// PL8: rebuilding a member rebuilds every later member of its side that plans automatically, in roster order, so a
/// later approach never ends on an earlier claim; a manual member keeps its steps.
#[test]
fn later_members_are_rebuilt_around_earlier_claims() {
    let mut s = session(timeline());
    let foe_at = V2::new(6, 4);
    place(&mut s.w, VANGUARD, V2::new(3, 4));
    place(&mut s.w, SPIKER, V2::new(3, 5));
    place(&mut s.w, DANCER, V2::new(9, 4));
    place(&mut s.w, FOOTMAN, foe_at);
    for c in [VANGUARD, SPIKER] {
        intent(&mut s.w, c, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
    }
    intent(&mut s.w, DANCER, RoundIntent::Maneuver, RoundTarget::Unit(FOOTMAN));
    let (w, plan) = (&mut s.w, &mut s.plan);
    rip::set_manual(w, plan, DANCER, true).unwrap();
    plan.set_steps(DANCER, vec![PlanStep::self_skill(sid(w, "rest"))]);
    let manual_steps = plan.steps_of(DANCER);
    let (first, refused) = rip::replan(w, plan, VANGUARD);
    assert!(first.is_ok() && refused.is_empty(), "{first:?} {refused:?}");
    let w = &s.w;
    let end = |c: UnitId| s.plan.entries_for(w, c).iter().filter(|e| kind_of(w, e) == Kind::Move).last().map(|e| e.pos_after);
    let (a, b) = (end(VANGUARD).expect("vanguard approaches"), end(SPIKER).expect("spiker approaches"));
    assert!(a != b && a.dist(foe_at) == 1 && b.dist(foe_at) == 1, "{a} {b}");
    let kept = s.plan.steps_of(DANCER);
    assert!(kept.len() == manual_steps.len() && Rc::ptr_eq(&kept[0], &manual_steps[0]), "the manual member is left alone");
}

/// TG6: the planner proposes last round's target while it is valid.
#[test]
fn last_rounds_target_is_proposed() {
    let mut s = session(timeline());
    place(&mut s.w, VANGUARD, V2::new(3, 4));
    place(&mut s.w, FOOTMAN_B, V2::new(4, 4));
    intent(&mut s.w, VANGUARD, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
    round_intent::round_start(&mut s.w);
    assert_eq!(s.w.round_states[&VANGUARD].target, None, "the target does not persist");
    assert_eq!(rip::proposed_target(&s.w, VANGUARD, RoundIntent::Assault), Some(RoundTarget::Unit(FOOTMAN)), "but is proposed");
    s.w.units[FOOTMAN].hp = 0;
    assert_eq!(rip::proposed_target(&s.w, VANGUARD, RoundIntent::Assault), Some(RoundTarget::Unit(FOOTMAN_B)), "a downed one is not: the nearest foe");
    assert_eq!(rip::proposed_target(&s.w, VANGUARD, RoundIntent::Guard), Some(RoundTarget::Myself));
}

/// MM1-MM4: a manual member keeps its steps (a delayed opener: Rest, Rest, then Heavy Cleave landing in b6); "fill the
/// rest" places follow-ups and Rest after the last manual step; Manual off rebuilds the auto plan.
#[test]
fn manual_mode_keeps_the_steps_and_fills_on_request() {
    let mut s = session(timeline());
    place(&mut s.w, VANGUARD, V2::new(4, 4));
    place(&mut s.w, FOOTMAN, V2::new(5, 4));
    intent(&mut s.w, VANGUARD, RoundIntent::Assault, RoundTarget::Unit(FOOTMAN));
    let (w, plan) = (&mut s.w, &mut s.plan);
    rip::set_manual(w, plan, VANGUARD, true).unwrap();
    let rest = sid(w, "rest");
    for _ in 0..2 {
        assert_eq!(plan.add_step(w, VANGUARD, PlanStep::self_skill(rest), false), "");
    }
    assert_eq!(plan.add_step(w, VANGUARD, PlanStep::self_skill(sid(w, "tl_brace_counter")), false), "Only Assault skills this round.");
    assert_eq!(plan.add_step(w, VANGUARD, PlanStep::engage(sid(w, "tl_heavy_cleave"), FOOTMAN), false), "");
    let heavy = plan.entries_for(w, VANGUARD).pop().unwrap();
    assert_eq!(heavy.end_block(w), BLOCKS, "a delayed opener lands in b6");
    // fill the rest after two manual Rests
    plan.set_steps(VANGUARD, plan.steps_of(VANGUARD)[..2].to_vec());
    rip::fill_rest(w, plan, VANGUARD).unwrap();
    let r = rows(w, plan, VANGUARD);
    assert_eq!(r.iter().map(|x| (x.0.as_str(), x.1, x.5.as_str())).collect::<Vec<_>>(), [("rest", 1, ""), ("rest", 2, ""), ("tl_heavy_cleave", 3, "follow-up 1")]);
    // Manual off: the auto plan
    rip::set_manual(w, plan, VANGUARD, false).unwrap();
    let r = rows(w, plan, VANGUARD);
    assert_eq!(r.iter().map(|x| (x.0.as_str(), x.5.as_str())).collect::<Vec<_>>(), [("tl_heavy_cleave", rip::OPENER), ("tl_shoulder_check", "follow-up 2"), ("tl_shoulder_check", "follow-up 2")]);
}

/// The planner runs only under the round intent: classic has none.
#[test]
fn classic_has_no_planner() {
    let mut s = session(Ruleset::classic());
    let (w, plan) = (&mut s.w, &mut s.plan);
    assert!(rip::plan_unit(w, plan, VANGUARD).is_err());
    assert!(rip::set_manual(w, plan, VANGUARD, true).is_err());
    assert!(s.w.round_states.is_empty());
}
