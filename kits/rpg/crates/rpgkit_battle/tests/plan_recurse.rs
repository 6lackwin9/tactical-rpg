//! PLAN-RECURSE1 (`PR7-61`, `FB-372`) on a scratch catalog built here (no shipped data): a unit whose first plan step is
//! an Engage with an area skill that `affects` `all` plans with friends active, where the planner once recursed
//! without end (`first_move` -> `apply` -> `apply_engage` -> `apply_area` -> `planned_position` -> `ally_moves` ->
//! `first_move`). The count of friends the area hits still reads each friend's planned tile.

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::intent;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;

/// The hero three tiles west of the foe, the ally beside the hero, under the timeline.
fn session() -> Session {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "c_ring", "name": "Ring", "kind": "ATTACK", "level": 1, "blocks": 2, "ap_cost": 2, "power_mult": 1.0, "area": "RING", "affects": "all", "target_mode": "ENEMY"}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 500, "power": 100, "armor": 10, "speed": 12, "attack_range": 1, "skill_cap": 5});
    let cat = Catalog::from_value(&json!({"skills": skills, "units": [unit("c_hero", "party"), unit("c_ally", "party"), unit("c_foe", "enemy")]}));
    let row = json!({"id": "c_fight", "name": "Fight", "party": ["c_hero", "c_ally"], "enemies": ["c_foe"], "total_blocks": 6, "unit_ap": 12, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": []}).as_object().unwrap());
    let mut s = Session::start(Arc::new(cat), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(7, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.resolver.scheduled.clear();
    for c in [HERO, ALLY, FOE] {
        s.resolver.register(c);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

#[test]
fn an_engage_with_an_area_on_everyone_plans_with_friends_active() {
    let s = session();
    let ring = sid(&s.w, "c_ring");
    // the hero alone, then with the ally's own Engage planned after it, then both in either order
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&s.w, HERO, PlanStep::engage(ring, FOE), true), "", "the hero walks in and rings the foe");
    assert_eq!(plan.add_step(&s.w, ALLY, PlanStep::engage(ring, FOE), true), "", "the ally does the same");
    let sched = intent::planned_schedule(&s.w, &[], &plan);
    assert!(sched.iter().any(|a| a.unit == HERO), "the hero's actions are scheduled");
    assert!(sched.iter().any(|a| a.unit == ALLY), "the ally's actions are scheduled");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&s.w, ALLY, PlanStep::engage(ring, FOE), true), "");
    assert_eq!(plan.add_step(&s.w, HERO, PlanStep::engage(ring, FOE), true), "");
    let _ = plan.compile(&s.w, HERO);
    let _ = plan.compile(&s.w, ALLY);
}

#[test]
fn the_area_still_counts_a_friend_on_its_planned_tile() {
    let s = session();
    let ring = sid(&s.w, "c_ring");
    // the hero stands next to the foe and rings it; the ally walks next to the hero (into the ring) first
    let mut w = s.w.clone();
    w.grid.positions[HERO] = Some(V2::new(6, 4));
    w.grid.positions[ALLY] = Some(V2::new(6, 7));
    let mv = w.book().find("move").unwrap();
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, ALLY, PlanStep::go_to(mv, V2::new(6, 5)), true), "", "the ally walks beside the hero");
    assert_eq!(plan.add_step(&w, HERO, PlanStep::engage(ring, FOE), true), "");
    let hits = plan.compile(&w, HERO).entries.iter().map(|e| e.hits_allies).sum::<i32>();
    assert_eq!(hits, 1, "the ring reaches the ally where it will stand, not where it stands now");
}
