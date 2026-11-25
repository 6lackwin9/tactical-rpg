//! BATTLE-UI2 (the battle screen contract 6, the heal preview) on a scratch catalog built here (no shipped data):
//! `intent::heal_on` reads the round as it will run (`planned_schedule`): the planned heals aimed at a unit, as the
//! command heals (`heal_amount_on`), within its missing HP (`effective_heal`); nothing on a unit no heal is aimed at.

use indexmap::IndexMap;
use rpgkit_battle::command;
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

fn session() -> Session {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "c_mend", "name": "Mend", "kind": "SUPPORT", "round_intent": "support", "level": 1, "blocks": 2, "ap_cost": 2, "heal_base": 30, "heal_mult": 0.3, "target_mode": "ALLY_OR_SELF"}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 500, "power": 100, "armor": 10, "speed": speed, "attack_range": 1, "skill_cap": 5});
    let cat = Catalog::from_value(&json!({"skills": skills, "units": [unit("c_hero", "party", 14), unit("c_ally", "party", 12), unit("c_foe", "enemy", 10)]}));
    let row = json!({"id": "c_fight", "name": "Fight", "party": ["c_hero", "c_ally"], "enemies": ["c_foe"], "total_blocks": 6, "unit_ap": 12, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": []}).as_object().unwrap());
    let mut s = Session::start(Arc::new(cat), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(6, 4)), (ALLY, V2::new(7, 4)), (FOE, V2::new(9, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.resolver.scheduled.clear();
    for c in [HERO, ALLY, FOE] {
        s.resolver.register(c);
    }
    s
}

fn mend(w: &World) -> usize {
    w.book().find("c_mend").expect("the heal skill")
}

fn plan(w: &World) -> Plan {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(mend(w), ALLY), true), "", "the hero mends the ally");
    plan
}

fn heal_on(s: &Session, plan: &Plan, u: UnitId) -> i32 {
    intent::heal_on(&s.w, &intent::planned_schedule(&s.w, &s.resolver.scheduled, plan), u)
}

#[test]
fn a_planned_heal_previews_what_the_command_heals_within_the_missing_hp() {
    let mut s = session();
    let full = command::heal_amount_on(&s.w, HERO, ALLY, mend(&s.w));
    assert!(full > 1, "the heal heals: {full}");
    // far below full HP: the whole heal
    let max = s.w.units[ALLY].max_hp;
    s.w.units[ALLY].hp = max - full - 50;
    let p = plan(&s.w);
    assert_eq!(heal_on(&s, &p, ALLY), command::effective_heal(&s.w.units[ALLY], full));
    assert_eq!(heal_on(&s, &p, ALLY), full);
    // just below full HP: only the missing HP
    s.w.units[ALLY].hp = max - 1;
    assert_eq!(heal_on(&s, &p, ALLY), 1);
    // nobody else gains
    assert_eq!((heal_on(&s, &p, HERO), heal_on(&s, &p, FOE)), (0, 0));
    assert_eq!(heal_on(&s, &Plan::new(), ALLY), 0, "no plan, no heal");
}
