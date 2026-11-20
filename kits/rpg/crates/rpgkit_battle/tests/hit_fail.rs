//! HITCALC1 (PR7-45) on a scratch catalog built here (no shipped data; the board and kit of `attack_lock.rs`): a
//! planned hit that fails at its impact is forecast as such (`intent::HitOn::fails`), each kind as the resolver then
//! decides it: an ATTACKLOCK1 dodge, a target out of reach, a protected flank (INPUT1 MR8 c) and an intercept (HOOKS1
//! 8.4, a redirecting `ally_hit` trigger). Each forecast is read on the round as it will run before the round, then
//! the round is resolved and the hit's outcome checked against it.

use indexmap::IndexMap;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::grid::{V2, NO_CELL};
use rpgkit_battle::intent::{self, HitFail};
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, ManeuverKind, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, AP_COST, ARENA_HAZARDS, FLANK, FORCED_MOVE, ROUND_INTENT, TIMELINE, TRIGGERS};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const ARCHER: UnitId = 2;
const FOE: UnitId = 3;
const FOE_B: UnitId = 4;

/// Engine skills and the test kit: melee attacks of one, two and three blocks, a Maneuver attack with a position, a band
/// (1, 2) as a reach weapon gives, a band (1, 3) that reaches past it, a ranged shot and an area.
fn catalog() -> Catalog {
    let atk = |id: &str, blocks: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": blocks, "ap_cost": 2, "power_mult": 1.0, "target_mode": "ENEMY"});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let skills = vec![
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        atk("tl_cut", 1, json!({})),
        atk("tl_heavy", 2, json!({"ap_cost": 3})),
        atk("tl_long", 3, json!({})),
        atk("tl_slip_strike", 2, json!({"round_intent": "maneuver", "position": "flank", "reach_max": 2})),
        atk("flank_cut", 1, json!({"round_intent": "maneuver", "position": "flank", "reach_max": 2})),
        atk("tl_thrust", 2, json!({"reach_min": 1, "reach_max": 2})),
        atk("tl_roar", 2, json!({"power_mult": 0.0, "reach_min": 1, "reach_max": 2})),
        atk("tl_hook", 2, json!({"reach_min": 1, "reach_max": 3})),
        atk("tl_aimed_shot", 2, json!({})),
        atk("tl_burst", 2, json!({"area": "BURST", "reach_min": 1, "reach_max": 3})),
        atk("tl_shove", 1, json!({"forced_move": {"kind": "push", "tiles": 1}})),
        json!({"id": "tl_hook_guard", "name": "tl_hook_guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3,
            "triggers": [{"on": "ally_hit", "when": {"range": 1, "melee": true}, "effects": [{"type": "redirect", "pct": 100}, {"type": "move", "kind": "pull", "tiles": 1, "to": "other"}], "limit": "round"}]}),
    ];
    let unit = |id: &str, side: &str, range: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": range, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("tl_vanguard", "party", 1), unit("tl_ally", "party", 1), unit("tl_archer", "party", 3), unit("tl_footman", "enemy", 1), unit("tl_footman_b", "enemy", 1)]}))
}

/// A timeline row with the round intent, `ap_cost`, the flank rule and what moves a unit by force; `lock` as `attack_lock`
/// (a row value).
fn timeline(lock: &str) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, FLANK, FORCED_MOVE, TRIGGERS, ARENA_HAZARDS], "attack_lock": lock}).as_object().unwrap())
}

/// The board: tl_vanguard (4, 4) next to tl_footman (5, 4); the others away. No planned round (tests book every action).
fn world(r: Ruleset) -> World {
    session(r, json!([])).w
}

fn session(r: Ruleset, hazards: Value) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["tl_vanguard", "tl_ally", "tl_archer"], "enemies": ["tl_footman", "tl_footman_b"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337, "arena_hazards": hazards});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = r;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(0, 8)), (ARCHER, V2::new(0, 0)), (FOE, V2::new(5, 4)), (FOE_B, V2::new(11, 8))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn hit(w: &World, c: UnitId, t: UnitId, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, PlanStep::engage(sid(w, skill), t), true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

/// `c` takes Maneuver Engage on `t`: its attacks with a position carry their dash.
fn engage(w: &mut World, c: UnitId, t: UnitId) {
    round_intent::set_intent(w, c, RoundIntent::Maneuver).unwrap();
    round_intent::set_target(w, c, RoundTarget::Unit(t)).unwrap();
    round_intent::set_maneuver(w, c, ManeuverKind::Engage).unwrap();
}

/// A Walk of the given one-tile steps, landing in its one block.
fn walk(c: UnitId, dirs: &[V2]) -> Command {
    let mut cmd = Command::new(c, 0, 1, CmdKind::Walk { path: dirs.to_vec(), expected_start: NO_CELL, goal: NO_CELL, approach: None, moved_tiles: 0, cut_short: false, blocked_by: String::new() });
    cmd.name = "Move".into();
    cmd
}

fn v(x: i32, y: i32) -> V2 {
    V2::new(x, y)
}

fn book(res: &mut Resolver, c: UnitId, cmd: Command, start: i32) -> usize {
    assert!(res.schedule(c, cmd, start), "booked at {start}");
    res.scheduled.len() - 1
}

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

fn forecast(w: &World, res: &Resolver, a: usize, t: UnitId) -> intent::HitOn {
    intent::hit_on(w, &res.scheduled, &res.scheduled[a], t).expect("the action aims at the unit")
}

/// ATTACKLOCK1: a Walk landing in the wind-up leaves the locked tile: forecast Dodged (no damage) before the round
/// and once the lock is taken, and the hit is dodged.
#[test]
fn a_dodge_is_forecast_as_a_dodge() {
    let mut w = world(timeline("melee"));
    let hp = w.units[FOE].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&w, HERO, FOE, "tl_heavy"), 1);
    book(&mut res, FOE, walk(FOE, &[v(0, -1), v(-1, 0)]), 1);
    let before = forecast(&w, &res, a, FOE);
    assert_eq!((before.fails, before.damage), (Some(HitFail::Dodged), 0), "{before:?}");
    run(&mut w, &mut res, 1, 1);
    assert_eq!(forecast(&w, &res, a, FOE).fails, Some(HitFail::Dodged), "with the lock taken");
    run(&mut w, &mut res, 2, 2);
    assert_eq!(res.scheduled[a].command.dodged, Some(FOE), "{:?}", res.log);
    assert_eq!(w.units[FOE].hp, hp);
}

/// Without a lock (`attack_lock` none) the target walking out of the band in the wind-up is out of reach at the
/// impact: forecast so, and the hit ends out of range. A target that stays is forecast to land, for what it deals.
#[test]
fn out_of_reach_is_forecast_as_out_of_reach() {
    let mut w = world(timeline("none"));
    let hp = w.units[FOE].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&w, HERO, FOE, "tl_heavy"), 1);
    book(&mut res, FOE, walk(FOE, &[v(1, 0), v(1, 0)]), 1);
    let f = forecast(&w, &res, a, FOE);
    assert_eq!((f.fails, f.damage), (Some(HitFail::OutOfReach), 0), "{f:?}");
    run(&mut w, &mut res, 1, 2);
    assert!(res.scheduled[a].command.log.starts_with("[Attack Out of Range]"), "{:?}", res.log);
    assert_eq!(w.units[FOE].hp, hp);
    // the same swing on a target that stays lands for its forecast
    let mut w = world(timeline("none"));
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&w, HERO, FOE, "tl_heavy"), 1);
    let f = forecast(&w, &res, a, FOE);
    assert_eq!(f.fails, None);
    let hp = w.units[FOE].hp;
    run(&mut w, &mut res, 1, 2);
    assert!(!res.scheduled[a].command.log.contains("CRIT"), "seed without a crit: {:?}", res.log);
    assert_eq!(hp - w.units[FOE].hp, f.damage, "{:?}", res.log);
}

/// INPUT1 MR8 c: a carried Engage planned while the flank was open, whose position tiles are all taken by the time it
/// starts, is forecast Flank protected and misses so.
#[test]
fn a_protected_flank_is_forecast_as_protected() {
    let mut w = world(timeline("melee"));
    // the foe on the top row facing the hero below it: its only position tiles are the two beside it
    w.grid.positions[FOE] = Some(v(5, 0));
    w.grid.positions[HERO] = Some(v(5, 2));
    round_intent::set_intent(&mut w, FOE, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut w, FOE, RoundTarget::Unit(HERO)).unwrap();
    engage(&mut w, HERO, FOE);
    let cut = hit(&w, HERO, FOE, "flank_cut");
    assert!(matches!(cut.carry, rpgkit_battle::command::Carry::Engage(_)), "a carried Engage: {:?}", cut.carry);
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cut, 1);
    assert_eq!(forecast(&w, &res, a, FOE).fails, None, "open flank");
    // both are taken by the time it starts
    for (c, cell) in [(ALLY, v(4, 0)), (ARCHER, v(6, 0))] {
        w.grid.positions[c] = Some(cell);
    }
    let f = forecast(&w, &res, a, FOE);
    run(&mut w, &mut res, 1, 1);
    let log = res.scheduled[a].command.log.clone();
    assert_eq!((f.fails, f.damage), (Some(HitFail::FlankProtected), 0), "{f:?}; the round: {log}");
    assert!(log.starts_with("[Flank Protected]") || res.log.iter().any(|l| l.starts_with("[Flank Protected]")), "{:?}", res.log);
}

/// HOOKS1 8.4: a friend holding a redirecting `ally_hit` guard next to the target takes the whole hit: forecast as an
/// intercept by it (nothing reaches the target), and the round moves the hit to it.
#[test]
fn an_intercept_is_forecast_as_an_intercept() {
    let mut w = world(timeline("melee"));
    w.grid.positions[ALLY] = Some(v(5, 3));
    w.grid.positions[ARCHER] = Some(v(6, 3));
    round_intent::set_target(&mut w, FOE, RoundTarget::Unit(ALLY)).unwrap();
    let cut = hit(&w, FOE, ALLY, "tl_cut");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, ARCHER, PlanStep::self_skill(sid(&w, "tl_hook_guard")), true), "");
    let guard = plan.entries_for(&w, ARCHER)[0].build_command(&w, ARCHER);
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, ARCHER, guard, 1);
    let a = book(&mut res, FOE, cut, 1);
    let f = forecast(&w, &res, a, ALLY);
    assert_eq!((f.fails, f.damage), (Some(HitFail::Intercepted { by: ARCHER, pct: 100 }), 0), "{f:?}");
    let (ally_hp, archer_hp) = (w.units[ALLY].hp, w.units[ARCHER].hp);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(w.units[ALLY].hp, ally_hp, "{:?}", res.log);
    assert!(w.units[ARCHER].hp < archer_hp, "the holder took it: {:?}", res.log);
}
