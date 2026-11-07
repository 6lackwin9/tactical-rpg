//! U4.2-S3 (docs/sessions/U4_PASSIVES.md ROSTER1-T fold-in, FB-29; FB-28 gather order; FB-63 (3) RECRUIT1) on fixture
//! rows (no shipped data): a character's `temperament` names an always-on passive that joins its gathered sources right
//! after its own passives (then faction, disciplines, named items; each id once), heroes too; a hero's save does not
//! keep it, so every build re-gathers it and a cleared temperament is gone. The character's `ai_role` and
//! `default_intent` override its base unit's (and the class's role) where those are read: the world roster's unit entry
//! (the session's AI profile and start intent of a fielded character) and a hero's build definition (its AI profile).

use rpgkit_battle::session::Roster;
use rpgkit_battle::world::{AiProfile, Role};
use rpgkit_world::battle::WorldRoster;
use rpgkit_world::new_game::hero_from_character;
use rpgkit_world::progression::{build_with, member_passives};
use rpgkit_world::Data;
use serde_json::{json, Value};

const BOLD_STR: i32 = 2;

fn data(characters: Value) -> Data {
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter", "ai_role": "AGGRESSOR", "skill_path": {}, "proficiency": {"sword": {"level": 1}}}],
        "passives": [
            {"id": "own_trait", "name": "Own", "slot": "", "effects": [{"type": "resist", "status": "bleed", "value": 10}]},
            {"id": "bold", "name": "Bold", "slot": "", "effects": [{"type": "stat", "stat": "STR", "value": BOLD_STR}]},
            {"id": "guild_grant", "name": "Guild", "slot": "", "effects": [{"type": "ap_regen", "value": 1}]},
            {"id": "school_way", "name": "School", "slot": "", "effects": [{"type": "resist", "status": "burn", "value": 10}]}
        ],
        "units": [{"id": "plain", "name": "Plain", "side": "party", "class_id": "fighter", "level": 1, "ai_role": "SNIPER"}]
    });
    let mut chars = characters.as_array().unwrap().clone();
    for c in &mut chars {
        let id = c["id"].as_str().unwrap().to_string();
        c["_world"] = json!("w");
        c["_qid"] = json!(format!("w:{id}"));
    }
    let worlds = json!({
        "w": {"manifest": {"id": "w", "start_party": ["hero"]}, "own": {
            "characters": chars,
            "factions": [{"id": "guild", "name": "Guild", "grants": ["guild_grant", "bold"], "_world": "w", "_qid": "w:guild"}],
            "disciplines": [{"id": "school", "name": "School", "passives": ["school_way"], "_world": "w", "_qid": "w:school"}]
        }}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn character(id: &str, role: &str, extra: Value) -> Value {
    let mut c = json!({"id": id, "name": id, "role": role, "base_unit": "plain", "passives": ["own_trait"]});
    for (k, x) in extra.as_object().unwrap() {
        c[k] = x.clone();
    }
    c
}

#[test]
fn the_temperament_comes_after_the_own_passives_and_before_the_other_sources_once() {
    let d = data(json!([character("npc", "npc", json!({"temperament": "bold", "faction": "guild", "disciplines": ["school"]}))]));
    let c = d.resolve("w", "characters", "npc");
    assert_eq!(d.character_passives(c, "w"), ["own_trait", "bold", "guild_grant", "school_way"], "the faction's 'bold' is not gathered twice");
    let none = data(json!([character("npc", "npc", json!({"faction": "guild"}))]));
    assert_eq!(none.character_passives(none.resolve("w", "characters", "npc"), "w"), ["own_trait", "guild_grant", "bold"], "without one, the faction's order");
}

#[test]
fn a_heroes_temperament_is_gathered_at_every_build_and_gone_when_cleared() {
    let d = data(json!([character("hero", "party", json!({"temperament": "bold"}))]));
    let hero = hero_from_character(&d, "w", "hero").expect("the hero joins (RECRUIT1 allows a temperament)");
    assert_eq!(hero.passives, ["own_trait"], "the save does not keep the temperament");
    assert_eq!(member_passives(&d, &hero), ["own_trait", "bold"]);
    let built = build_with(&d, &hero, "w", true).expect("it builds").unit;
    assert_eq!(built.passives.iter().filter(|p| *p == "bold").count(), 1, "applied once");
    let plain = data(json!([character("hero", "party", json!({}))]));
    let without = build_with(&plain, &hero, "w", true).expect("it builds").unit;
    assert_eq!(built.stats.stat("STR", None), without.stats.stat("STR", None) + BOLD_STR, "the temperament's stat");
    assert_eq!(member_passives(&plain, &hero), ["own_trait"], "cleared in the data, gone from the build");
    assert!(!without.passives.contains(&"bold".to_string()));
}

#[test]
fn a_fielded_characters_ai_role_and_default_intent_override_its_base_unit() {
    let d = data(json!([
        character("calm", "enemy", json!({"ai_role": "SUPPORT", "default_intent": "guard"})),
        character("plain_npc", "enemy", json!({}))
    ]));
    let roster = WorldRoster { d: &d, world: "w".into() };
    let calm = roster.entry(&d.cat, "calm").expect("the entry");
    assert_eq!((calm.ai_role.as_str(), calm.default_intent.as_str()), ("SUPPORT", "guard"));
    assert_eq!(AiProfile::for_unit(&d.cat, &calm).role, Role::Support);
    let plain = roster.entry(&d.cat, "plain_npc").expect("the entry");
    assert_eq!((plain.ai_role.as_str(), plain.default_intent.as_str()), ("SNIPER", ""), "empty keys keep the base unit's");
    assert_eq!(AiProfile::for_unit(&d.cat, &plain).role, Role::Sniper);
}

#[test]
fn a_heroes_ai_role_and_default_intent_reach_its_build_definition() {
    let d = data(json!([character("hero", "party", json!({"ai_role": "SKIRMISHER", "default_intent": "maneuver"}))]));
    let hero = hero_from_character(&d, "w", "hero").unwrap();
    let def = build_with(&d, &hero, "w", true).expect("it builds").def;
    assert_eq!((def.ai_role.as_str(), def.default_intent.as_str()), ("SKIRMISHER", "maneuver"));
    assert_eq!(AiProfile::for_unit(&d.cat, &def).role, Role::Skirmisher);
    let plain = data(json!([character("hero", "party", json!({}))]));
    assert_eq!(build_with(&plain, &hero, "w", true).unwrap().def.ai_role, "SNIPER", "the base unit's without one");
}
