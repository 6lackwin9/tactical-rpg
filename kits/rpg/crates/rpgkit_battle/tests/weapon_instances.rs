//! PK2-S5a (`docs/sessions/PK2_GRAFTS.md` 6) on a scratch catalog built here (no shipped data): the rule `durability`
//! switches weapon instances on in the world (`rpgkit_world::weapons`, tests in `rpgkit_world/tests/weapon_instances.rs`);
//! the battle reads nothing of it yet (the durability spend is PK2-S5b), so a fight under a timeline row with the rule
//! plays the same log and HP as the same row without it.

use indexmap::IndexMap;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{self, Ruleset, DURABILITY, POISE, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const ROUNDS: i32 = 4;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_cut", "name": "Cut", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 300, "power": 40, "speed": 12, "attack_range": 1, "skill_cap": 5, "skills": ["t_cut"]});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("t_hero", "party"), unit("t_foe", "enemy")]}))
}

fn row(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// The fight's log and each unit's HP after up to `ROUNDS` auto-planned rounds.
fn play(r: Ruleset) -> (Vec<String>, Vec<i32>) {
    let enc = json!({"id": "t_fight", "name": "Fight", "party": ["t_hero"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(enc.as_object().unwrap());
    def.ruleset = r;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for _ in 0..ROUNDS {
        s.plan_apply_preset_all("balanced");
        if !s.confirm_plan() {
            break;
        }
        while s.phase != Phase::Replay {
            s.step_next();
        }
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    (s.resolver.log.clone(), s.w.units.iter().map(|u| u.hp).collect())
}

#[test]
fn the_rule_is_a_registered_extension_that_needs_nothing() {
    let r = ruleset::rule(DURABILITY).expect("the register lists durability");
    assert_eq!(r.kind, ruleset::RuleKind::Extension, "inert without data, so an extension");
    assert!(r.requires.is_empty(), "the world reads it alone");
    assert!(row(&[ROUND_INTENT, DURABILITY]).has(DURABILITY));
    assert!(row(&[DURABILITY]).notes().errors.is_empty(), "a row may list it alone");
}

#[test]
fn a_fight_plays_the_same_with_or_without_the_rule() {
    let base = [ROUND_INTENT, POISE];
    let (log, hp) = play(row(&base));
    assert!(!log.is_empty(), "the fight resolved some blocks");
    let with: Vec<&str> = base.iter().copied().chain([DURABILITY]).collect();
    assert_eq!(play(row(&with)), (log, hp), "the battle reads nothing of durability yet");
    assert_eq!(play(row(&[DURABILITY])), play(row(&[])), "nor over a row with no other rule");
}
