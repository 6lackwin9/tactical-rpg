//! CLASS-BASICS (READY row R2-W2b, user 2026-11-08) in a battle on a scratch catalog built here (no shipped data): under
//! the rule `deck` every roster unit (hero or enemy) fights with its class's basics beside the ruleset's (no slot), so
//! the round-intent planner's auto order and hotkeys hold them after the ruleset's basics; a unit of another class does
//! not get them.

use indexmap::IndexMap;
use rpgkit_battle::round_intent::RoundIntent;
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, DECK, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const HERO: UnitId = 0;
const GUARDIAN: UnitId = 1;
const CUTTER: UnitId = 2;

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let class = |id: &str, basics: serde_json::Value| {
        json!({"id": id, "name": id, "base_stats": stats, "growth": {}, "proficiency": {"sword": {"level": 1}}, "skill_path": {"slash": 1}, "basics": basics})
    };
    let unit = |id: &str, class: &str, side: &str| json!({"id": id, "name": id, "class_id": class, "level": 3, "side": side, "skill_cap": 5});
    Catalog::from_value(&json!({
        "classes": [class("guardian", json!(["brace"])), class("cutter", json!(["flank_cut"]))],
        "units": [unit("p_hero", "guardian", "party"), unit("p_guard", "guardian", "enemy"), unit("p_cut", "cutter", "enemy")],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 3, "learned_only": true, "round_intent": "guard", "mitigation": 0.2},
            {"id": "flank_cut", "name": "Flank Cut", "kind": "ATTACK", "level": 1, "blocks": 1, "round_intent": "maneuver", "power_mult": 0.6},
            {"id": "slash", "name": "Slash", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0}
        ]
    }))
}

fn world() -> World {
    let rs = json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, DECK], "basics": ["strike", "guard"], "deck_slots": {"active": 2, "passive": 0, "keystone": 0}});
    let row = json!({"id": "cb_fight", "name": "Fight", "party": ["p_hero"], "enemies": ["p_guard", "p_cut"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 4, "rng_seed": 7});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(rs.as_object().unwrap());
    Session::start(Arc::new(catalog()), def, &IndexMap::new()).w
}

fn ids(w: &World, sids: &[usize]) -> Vec<String> {
    sids.iter().map(|&s| w.cat.skills[s].id.clone()).collect()
}

#[test]
fn an_enemy_fights_with_its_class_basic_after_the_ruleset_basics() {
    let w = world();
    let rule = w.units[GUARDIAN].deck_rule.as_ref().expect("the deck rule");
    assert_eq!(rule.basics, ["strike", "guard", "brace"], "the ruleset's basics, then the class's");
    assert_eq!(ids(&w, &rip::order(&w, GUARDIAN, RoundIntent::Guard)), ["guard", "brace"], "the auto Guard order ends with the class basic");
    assert!(!w.units[GUARDIAN].active_families().contains(&"brace".to_string()), "no active slot");
    assert!(ids(&w, &rip::hotkey_skills(&w, GUARDIAN)).contains(&"brace".to_string()));
}

#[test]
fn a_hero_gets_its_class_basic_and_another_class_does_not() {
    let w = world();
    assert!(ids(&w, &rip::order(&w, HERO, RoundIntent::Guard)).contains(&"brace".to_string()), "a hero of the class");
    assert!(!ids(&w, &rip::order(&w, CUTTER, RoundIntent::Guard)).contains(&"brace".to_string()), "another class");
    assert_eq!(ids(&w, &rip::order(&w, CUTTER, RoundIntent::Maneuver)), ["flank_cut"], "its own class basic");
    assert!(rip::order(&w, GUARDIAN, RoundIntent::Maneuver).is_empty(), "the cutter's basic is not the guardian's");
}
