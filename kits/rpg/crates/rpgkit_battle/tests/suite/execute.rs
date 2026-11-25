//! HOOKS1-X on a scratch catalog built here (no shipped data): execute riders, an `exploit` entry
//! `{hp_below, power_mult}` under the rule `execute` (HOOKS1 8.6 EX1). Expectations come from the rule: the entry
//! matches when the target's HP share is at or below `hp_below` before the hit's damage; the best entry wins (x2.0 beats
//! a staggered x1.5); a consuming skill removes nothing for it; the hit's log cell names it. With the rule off (and in
//! classic) the entry never matches and the battle draws the same random numbers as a plain hit.

use indexmap::IndexMap;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::skill;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::ruleset::{Ruleset, EXECUTE, POISE, TIMELINE};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HP: i32 = 5000;
const PCT: i32 = 35;
const HERO: UnitId = 0;
const FOE: UnitId = 1;

/// The exploit list of HOOKS1_TASKS 3's row X: a staggered target x1.5, an execute under 35% x2.0.
fn exploit() -> Value {
    json!([{"status": "stagger", "power_mult": 1.5}, {"hp_below": PCT, "power_mult": 2.0}])
}

/// Engine skills, a plain jab (x0.5) and the carve (x0.5 with the exploit list; `t_carve_c` also consumes).
fn catalog() -> Catalog {
    let attack = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY"});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        attack("t_jab", json!({})),
        attack("t_carve", json!({"exploit": exploit()})),
        attack("t_carve_c", json!({"exploit": exploit(), "exploit_consume": true})),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("p_hero", "party", 14), unit("p_foe", "enemy", 10)]}))
}

fn timeline(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// Hero (4,4) faces the foe (5,4).
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn hit(w: &World, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(sid(w, skill), FOE), true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, HERO).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, HERO)
}

/// The foe at `pct`% of its HP (staggered: the status active).
fn foe_at(w: &mut World, pct: i32, staggered: bool) {
    w.units[FOE].hp = w.units[FOE].max_hp * pct / 100;
    if staggered {
        status::grant_now(&mut w.units[FOE], "stagger", 1, 0, "test");
        assert!(w.units[FOE].has_status("stagger"));
    }
}

/// The hero's `skill` on the foe at block 1: the world, the resolver and the action.
fn strike(s: &Session, skill: &str, pct: i32, staggered: bool) -> (World, World, Resolver, usize) {
    let mut w = s.w.clone();
    foe_at(&mut w, pct, staggered);
    let before = w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(HERO, hit(&w, skill), 1));
    for i in res.prepare_block(&mut w, 1) {
        res.execute_single(&mut w, i);
    }
    (before, w, res, 0)
}

fn result(res: &Resolver, a: usize) -> (i32, bool) {
    match &res.scheduled[a].command.kind {
        CmdKind::Attack { last_damage, last_crit, .. } => (*last_damage, *last_crit),
        _ => panic!("an attack"),
    }
}

/// EX1, HOOKS1_TASKS 3 row X: the foe at 36% / 35% / 35% staggered / 36% staggered: x1.0 / x2.0 / x2.0 / x1.5 (the best
/// entry wins); rule off (and classic): x1.0 at 35%, x1.5 staggered.
#[test]
fn the_entry_matches_at_or_below_its_share() {
    let on = session(timeline(&[EXECUTE]));
    for (pct, staggered, label, mult) in [(PCT + 1, false, "", 1.0), (PCT, false, skill::EXECUTE, 2.0), (PCT, true, skill::EXECUTE, 2.0), (PCT + 1, true, "stagger", 1.5)] {
        let mut w = on.w.clone();
        foe_at(&mut w, pct, staggered);
        let got = skill::exploit_match_by(exploit().as_array().unwrap(), &w.units[FOE], &[], false, &w.ruleset);
        assert_eq!((got.0.as_str(), got.1), (label, mult), "{pct}% staggered {staggered}");
    }
    for rules in [timeline(&[POISE]), Ruleset::classic()] {
        let s = session(rules.clone());
        for (staggered, mult) in [(false, 1.0), (true, 1.5)] {
            let mut w = s.w.clone();
            foe_at(&mut w, PCT, staggered);
            assert_eq!(skill::exploit_match_by(exploit().as_array().unwrap(), &w.units[FOE], &[], false, &w.ruleset).1, mult, "{}", rules.id);
        }
    }
}

/// EX1 in a battle: the carve on the foe at 35% deals the hit of power x0.5 x2.0 (the HP share read before the damage),
/// records the exploit combo `execute` and says so in its log cell; at 36% x0.5. A consuming carve on a staggered foe
/// at 35% executes and leaves the stagger. Rule off: x0.5 and the same random draws as a plain jab.
#[test]
fn a_hit_executes_and_consumes_nothing() {
    let on = session(timeline(&[EXECUTE]));
    for (pct, mult) in [(PCT, 2.0), (PCT + 1, 1.0)] {
        let (before, w, res, a) = strike(&on, "t_carve", pct, false);
        let (dealt, crit) = result(&res, a);
        let want = damage::compute(&w.units[HERO], &before.units[FOE], HitInput { skill_mult: 0.5 * mult, force_crit: crit, ..HitInput::default() }).final_dmg;
        assert_eq!(dealt, want, "{pct}%");
        let executed = res.scheduled[a].command.exploited.iter().any(|(t, l)| *t == FOE && l == skill::EXECUTE);
        assert_eq!(executed, mult > 1.0, "{pct}%");
        let cell = rpgkit_battle::log::from_action(&w, &res.scheduled[a], 1, 1);
        assert_eq!(cell.detail.contains(&format!("Execute x2.00 (HP {PCT}% or less)")), mult > 1.0, "{}", cell.detail);
    }

    let (_, w, res, a) = strike(&on, "t_carve_c", PCT, true);
    assert!(res.scheduled[a].command.exploited.iter().any(|(_, l)| l == skill::EXECUTE), "x2.0 beats the stagger's x1.5");
    assert!(w.units[FOE].has_status("stagger"), "nothing consumed for an execute");

    for rules in [timeline(&[POISE]), Ruleset::classic()] {
        let s = session(rules.clone());
        let (before, w, mut res, a) = strike(&s, "t_carve", PCT, false);
        let (_, _, mut plain, _) = strike(&s, "t_jab", PCT, false);
        let (dealt, crit) = result(&res, a);
        let want = damage::compute(&w.units[HERO], &before.units[FOE], HitInput { skill_mult: 0.5, force_crit: crit, ..HitInput::default() }).final_dmg;
        assert_eq!(dealt, want, "{}: no execute", rules.id);
        assert!(res.scheduled[a].command.exploited.is_empty(), "{}", rules.id);
        assert_eq!(res.rng.randi(), plain.rng.randi(), "{}: no new draw", rules.id);
    }
}

/// EX1 in the plan: the planner's expected hit counts the execute when the foe's HP share meets it now (x2.0 at 35%,
/// x1.0 at 36%); rule off it never does.
#[test]
fn the_estimate_counts_the_execute() {
    for (rules, doubles) in [(timeline(&[EXECUTE]), true), (timeline(&[POISE]), false)] {
        let s = session(rules.clone());
        let est = |pct: i32| {
            let mut w = s.w.clone();
            foe_at(&mut w, pct, false);
            let mut plan = Plan::new();
            assert_eq!(plan.add_step(&w, HERO, PlanStep::engage(sid(&w, "t_carve"), FOE), true), "");
            let id = sid(&w, "t_carve");
            plan.entries_for(&w, HERO).into_iter().find(|e| e.skill == id).unwrap().est_damage
        };
        let (low, high) = (est(PCT), est(PCT + 1));
        assert!(high > 0);
        assert_eq!(low > high, doubles, "{}: {low} vs {high}", rules.id);
        assert_eq!(low == high, !doubles, "{}: {low} vs {high}", rules.id);
    }
}
