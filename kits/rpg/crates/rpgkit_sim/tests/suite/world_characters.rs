//! C02-A (FB-83): a world encounter's story characters play in the sim as the game's world battle spawns them: a
//! character with its own temperament, AI role or default intent is a sandbox copy of its base unit that carries the
//! AI role and default intent (U4.2-S3; AUDIT-04 `Prebuilt::default_intent`) and spawns with the temperament passive
//! (`character_overrides`); a character without extras plays as its base unit. Checked on a scratch world built here.

use rpgkit_battle::world::{AiProfile, Role};
use rpgkit_sim::args::SimArgs;
use rpgkit_sim::runner::Runner;
use rpgkit_sim::world;
use rpgkit_world::Data;
use serde_json::json;

/// Base unit `t_base` (no AI role, no default intent) and world `vale` with three characters on it: `hero` (temperament,
/// AI role, default intent), `scout` (an AI role only) and `plain` (nothing of its own), all in encounter `e_one`.
fn data() -> Data {
    let catalog = json!({
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}
        ],
        "units": [{"id": "t_base", "name": "Base", "side": "enemy", "stat_mode": "fixed", "hp": 60, "speed": 10, "attack_range": 1, "skill_cap": 3}],
    });
    let worlds = json!({
        "vale": {
            "manifest": {"id": "vale"},
            "own": {
                "characters": [
                    {"id": "hero", "name": "Hero", "base_unit": "t_base", "temperament": "t_bold", "ai_role": "SNIPER", "default_intent": "guard"},
                    {"id": "scout", "name": "Scout", "base_unit": "t_base", "ai_role": "SUPPORT"},
                    {"id": "plain", "name": "Plain", "base_unit": "t_base"}
                ],
                "encounters": [{"id": "e_one", "name": "One", "enemies": ["hero", "scout", "plain"]}]
            }
        }
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

#[test]
fn a_sim_built_character_gets_its_temperament_ai_role_and_default_intent() {
    let d = data();
    let a = SimArgs::default();
    let mut r = Runner::new(&d, &a);
    let enc = world::encounter(&mut r, "e_one", 1, false, "vale", false).expect("the scratch encounter");
    assert_eq!(enc.enemies.len(), 3, "{:?}", enc.enemies);
    let (hero, scout, plain) = (enc.enemies[0].clone(), enc.enemies[1].clone(), enc.enemies[2].clone());
    assert_ne!(hero, "t_base", "a character with a temperament, AI role and default intent is a sandbox copy");
    assert_ne!(scout, "t_base", "an AI role alone makes a sandbox copy");
    assert_eq!(plain, "t_base", "a character with nothing of its own plays as its base unit");

    let (row, over) = r.spawn_row(&hero).expect("the hero writes");
    assert_eq!((row.ai_role.as_str(), row.default_intent.as_str()), ("SNIPER", "guard"), "the character's AI role and default intent");
    assert!(over.passives.iter().any(|p| p == "t_bold"), "the temperament passive spawns with it: {:?}", over.passives);
    assert_eq!(AiProfile::for_unit(&d.cat, &row).role, Role::Sniper, "the battle's AI profile reads the character's role");

    let (row, over) = r.spawn_row(&scout).expect("the scout writes");
    assert_eq!((row.ai_role.as_str(), row.default_intent.as_str()), ("SUPPORT", ""), "its AI role over the base unit's; no intent");
    assert!(over.passives.is_empty(), "{:?}", over.passives);

    let (row, over) = r.spawn_row(&plain).expect("the base unit");
    assert_eq!((row.ai_role.as_str(), row.default_intent.as_str()), ("", ""), "the base unit's own (none)");
    assert!(over.passives.is_empty());
}

#[test]
fn a_level_shifted_character_keeps_its_ai_fields() {
    let d = data();
    let a = SimArgs::default();
    let mut r = Runner::new(&d, &a);
    let enc = world::encounter(&mut r, "e_one", 4, true, "vale", false).expect("the scratch encounter");
    let (row, over) = r.spawn_row(&enc.enemies[0]).expect("the hero writes");
    assert_eq!((row.level, row.ai_role.as_str(), row.default_intent.as_str()), (4, "SNIPER", "guard"));
    assert!(over.passives.iter().any(|p| p == "t_bold"));
}
