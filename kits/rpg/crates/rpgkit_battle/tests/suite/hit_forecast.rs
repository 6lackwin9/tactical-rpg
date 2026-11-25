//! PR7-41 (PT-20) on a scratch catalog built here (no shipped data): the one forecast convention of the battle screen.
//! A hit is forecast per hit, at the block it lands, with the target's guard at that block (`intent::hits_on`,
//! `intent::guard_at`, `Plan::forecast_entry`). Expectations come from the resolver itself: the forecast of each hit
//! equals what the round then deals (no crit on this seed), a move before the hit puts the landing block after it,
//! and a guard planned before the impact lowers the hit to the guarded amount.

use indexmap::IndexMap;
use rpgkit_battle::command::CmdKind;
use rpgkit_battle::grid::V2;
use rpgkit_battle::intent;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const FOE: UnitId = 1;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_thrust", "name": "Thrust", "kind": "ATTACK", "level": 1, "blocks": 2, "power_mult": 1.5, "target_mode": "ENEMY"}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 100, "armor": 10, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("f_hero", "party"), unit("f_foe", "enemy")]}))
}

/// The hero at (4, 4), the foe two tiles off at (7, 4) (it walks two tiles, then thrusts), in the plain timeline.
fn session(seed: i32) -> Session {
    let row = json!({"id": "f_fight", "name": "Fight", "party": ["f_hero"], "enemies": ["f_foe"], "total_blocks": BLOCKS, "unit_ap": 12, "ap_regen": 0, "max_rounds": 8, "rng_seed": seed});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": []}).as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (FOE, V2::new(7, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn add(plan: &mut Plan, w: &World, c: UnitId, step: rpgkit_battle::plan::Step) {
    assert_eq!(plan.add_step(w, c, step, true), "", "step for {}", w.units[c].name);
}

/// The foe walks two tiles west, then thrusts at the hero twice (blocks 1-2 moves, impacts 4 and 6); scheduled now.
fn schedule_foe(s: &mut Session) {
    let mut plan = Plan::new();
    let w = &s.w;
    add(&mut plan, w, FOE, PlanStep::mv(sid(w, "move"), V2::new(-1, 0), 2));
    add(&mut plan, w, FOE, PlanStep::engage(sid(w, "t_thrust"), HERO));
    add(&mut plan, w, FOE, PlanStep::engage(sid(w, "t_thrust"), HERO));
    s.resolver.scheduled.clear();
    s.resolver.register(HERO);
    s.resolver.register(FOE);
    plan.commit(&s.w, &mut s.resolver);
}

/// The hero's plan: a Guard at block 1 (`guard`) or Rest.
fn hero_plan(s: &Session, guard: bool) -> Plan {
    let mut plan = Plan::new();
    add(&mut plan, &s.w, HERO, PlanStep::self_skill(sid(&s.w, if guard { "guard" } else { "rest" })));
    plan
}

/// Commits the hero's plan and runs the round, returning the hero's HP loss at each block a hit lands and the log.
fn run(s: &mut Session, plan: &Plan) -> (Vec<(i32, i32)>, Vec<String>) {
    plan.commit(&s.w, &mut s.resolver);
    let mut losses = Vec::new();
    let mut log = Vec::new();
    for b in 1..=BLOCKS {
        let before = s.w.units[HERO].hp;
        let resolving = s.resolver.prepare_block(&mut s.w, b);
        for i in resolving {
            s.resolver.execute_single(&mut s.w, i);
            log.push(s.resolver.scheduled[i].command.log.clone());
        }
        if s.w.units[HERO].hp != before {
            losses.push((b, before - s.w.units[HERO].hp));
        }
    }
    (losses, log)
}

/// PT-20: the forecast names each hit at the block it lands (after the moves, never the first action's block) and
/// its amount is what the round deals, unguarded.
#[test]
fn each_hit_is_forecast_at_its_landing_block_and_matches_the_round() {
    let mut s = session(1337);
    schedule_foe(&mut s);
    let plan = hero_plan(&s, false);
    let hits = intent::hits_on(&s.w, &intent::planned_schedule(&s.w, &s.resolver.scheduled, &plan), FOE, HERO);
    let impacts: Vec<i32> = s.resolver.scheduled.iter().filter(|a| a.unit == FOE && matches!(a.command.kind, CmdKind::Attack { .. })).map(|a| a.impact).collect();
    assert_eq!(hits.iter().map(|h| h.block).collect::<Vec<_>>(), impacts, "{hits:?}");
    let first_start = s.resolver.scheduled.iter().filter(|a| a.unit == FOE).map(|a| a.start).min().unwrap();
    assert!(hits.iter().all(|h| h.block > first_start && !h.guarded), "lands after the moves: {hits:?}");
    let (losses, log) = run(&mut s, &plan);
    assert!(!log.iter().any(|l| l.contains("CRIT")), "seed without a crit: {log:?}");
    assert_eq!(losses, hits.iter().map(|h| (h.block, h.damage)).collect::<Vec<_>>(), "{log:?}");
}

/// PT-20: with a Guard planned before the impact the forecast counts the guard, lower than unguarded, and equals what
/// the guarded round deals.
#[test]
fn a_planned_guard_is_counted_at_the_block_the_hit_lands() {
    let mut s = session(1337);
    schedule_foe(&mut s);
    let open = intent::hits_on(&s.w, &s.resolver.scheduled, FOE, HERO);
    let plan = hero_plan(&s, true);
    let round = intent::planned_schedule(&s.w, &s.resolver.scheduled, &plan);
    let hits = intent::hits_on(&s.w, &round, FOE, HERO);
    assert!(hits.iter().zip(&open).all(|(g, o)| g.guarded && g.damage < o.damage), "{hits:?} vs {open:?}");
    assert_eq!(intent::guard_at(&s.w, &round, HERO, hits[0].block), 0.3, "the Guard's authored fraction");
    let (losses, log) = run(&mut s, &plan);
    assert!(!log.iter().any(|l| l.contains("CRIT")), "seed without a crit: {log:?}");
    assert_eq!(losses, hits.iter().map(|h| (h.block, h.damage)).collect::<Vec<_>>(), "{log:?}");
}

/// PT-20: a guard counts only from the block its wind-up starts through the round's end; before it the hero is
/// unguarded.
#[test]
fn a_guard_counts_from_its_start_block() {
    let s = session(1337);
    let mut plan = Plan::new();
    add(&mut plan, &s.w, HERO, PlanStep::self_skill(sid(&s.w, "rest")));
    add(&mut plan, &s.w, HERO, PlanStep::self_skill(sid(&s.w, "rest")));
    add(&mut plan, &s.w, HERO, PlanStep::self_skill(sid(&s.w, "guard")));
    let round = intent::planned_schedule(&s.w, &[], &plan);
    let start = round.iter().find(|a| matches!(a.command.kind, CmdKind::Defend { .. })).expect("the guard").start;
    assert_eq!(intent::guard_at(&s.w, &round, HERO, start - 1), 0.0);
    assert_eq!(intent::guard_at(&s.w, &round, HERO, start), 0.3);
    assert_eq!(intent::guard_at(&s.w, &round, HERO, BLOCKS), 0.3);
    assert_eq!(intent::guard_at(&s.w, &round, FOE, start), 0.0, "another unit's guard");
}

/// PT-20: the party's planned hit on a guarding foe (`Plan::forecast_entry`) takes the foe's guard at its impact; on
/// an unguarded foe it is the entry's own estimate.
#[test]
fn a_planned_hit_on_a_guarding_foe_counts_its_guard() {
    let mut s = session(1337);
    s.w.grid.positions[FOE] = Some(V2::new(5, 4));
    let mut foe = Plan::new();
    add(&mut foe, &s.w, FOE, PlanStep::self_skill(sid(&s.w, "guard")));
    s.resolver.scheduled.clear();
    s.resolver.register(FOE);
    foe.commit(&s.w, &mut s.resolver);
    let mut plan = Plan::new();
    add(&mut plan, &s.w, HERO, PlanStep::engage(sid(&s.w, "t_thrust"), FOE));
    let e = plan.compile(&s.w, HERO).entries.into_iter().find(|e| e.target == Some(FOE)).expect("the thrust");
    let guarded = plan.forecast_entry(&s.w, &s.resolver.scheduled, HERO, &e);
    assert!(e.est_damage > 0 && guarded < e.est_damage, "{guarded} vs {}", e.est_damage);
    assert_eq!(plan.forecast_entry(&s.w, &[], HERO, &e), e.est_damage, "no guard: the entry's estimate");
}
