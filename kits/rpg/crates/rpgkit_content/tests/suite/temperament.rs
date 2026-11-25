//! U4.2-S3 (docs/sessions/U4_PASSIVES.md, FB-29) on fixture rows (no shipped data): a character's `temperament` names
//! a known always-on passive (slot ""); its `default_intent` is one some ruleset lists (the unit row's rule) and its
//! `ai_role` and `default_intent` take the unit field's vocabularies; the three keys store as no key while empty, so a
//! character written before them re-saves as before.

use aegis_core::assets::Layers;
use aegis_core::json::Dict;
use aegis_content::schema::fields;
use rpgkit_content::check::{world, View};
use rpgkit_content::{source, store};
use serde_json::{json, Value};

fn view(character: Value) -> View {
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter", "skill_path": {}, "proficiency": {"sword": {"level": 1}}}],
        "units": [{"id": "plain", "name": "Plain", "side": "enemy", "class_id": "fighter", "level": 1, "deck": {}, "tactics": []}],
        "passives": [
            {"id": "bold", "name": "Bold", "slot": "", "effects": [{"type": "stat", "stat": "STR", "value": 1}]},
            {"id": "stance", "name": "Stance", "slot": "passive", "effects": [{"type": "stat", "stat": "STR", "value": 1}]}
        ],
        "rulesets": [{"id": "classic", "name": "Classic", "rules": [], "intents": ["assault", "guard"]}]
    });
    let mut c = character;
    c["_world"] = json!("w");
    c["_qid"] = json!("w:c");
    let worlds = json!({"w": {"manifest": {"id": "w", "start_party": []}, "own": {"characters": [c]}}});
    View::new(&catalog, &worlds, &json!({}), Layers::default())
}

fn character(extra: Value) -> Value {
    let mut c = json!({"id": "c", "name": "C", "role": "enemy", "base_unit": "plain"});
    for (k, x) in extra.as_object().unwrap() {
        c[k] = x.clone();
    }
    c
}

fn errors(extra: Value) -> Vec<String> {
    let v = view(character(extra));
    let e: Dict = v.worlds.own_entries("w", "characters")[0].clone();
    world::validate_entry(&v, "w", "characters", &e, "c")
}

#[test]
fn a_temperament_is_a_known_always_on_passive() {
    assert!(!errors(json!({"temperament": "bold"})).iter().any(|e| e.contains("emperament")), "{:?}", errors(json!({"temperament": "bold"})));
    assert!(errors(json!({"temperament": "stance"})).iter().any(|e| e == "Temperament 'stance' is slottable (passive); a temperament is always on (slot \"\")."));
    let unknown = errors(json!({"temperament": "nobody"}));
    assert_eq!(unknown.iter().filter(|e| e.contains("nobody")).collect::<Vec<_>>(), ["Temperament: unknown base passive 'nobody'."], "the field table's ref check, once");
    assert!(!errors(json!({})).iter().any(|e| e.contains("emperament")));
}

#[test]
fn the_default_intent_is_one_a_ruleset_lists() {
    assert!(!errors(json!({"default_intent": "guard"})).iter().any(|e| e.contains("Default intent")));
    assert!(errors(json!({"default_intent": "maneuver"})).iter().any(|e| e.starts_with("Default intent: no ruleset lists 'maneuver'")), "{:?}", errors(json!({"default_intent": "maneuver"})));
}

#[test]
fn the_ai_keys_take_the_unit_fields_vocabularies() {
    let options = |section: &str, kind: &str, key: &str| -> Value {
        fields(source::schema(), section, kind).iter().find(|f| f["key"] == key).unwrap_or_else(|| panic!("{kind}.{key}"))["options"].clone()
    };
    for key in ["default_intent", "ai_role"] {
        assert_eq!(options("world", "characters", key), options("content", "units", key), "{key}");
    }
    let intents: Vec<Value> = std::iter::once(json!("")).chain(rpgkit_rules::ruleset::DEFAULT_INTENTS.iter().map(|i| json!(i))).collect();
    assert_eq!(options("world", "characters", "default_intent"), Value::Array(intents), "the round intent ids");
    assert!(errors(json!({"ai_role": "BOSS"})).iter().any(|e| e.starts_with("AI role: 'BOSS' is not one of")));
}

#[test]
fn the_keys_store_as_no_key_while_empty() {
    let plain = store::world_storage("characters", character(json!({})).as_object().unwrap());
    for key in ["temperament", "default_intent", "ai_role"] {
        assert!(!plain.contains_key(key), "{key}: {plain:?}");
    }
    let set = store::world_storage("characters", character(json!({"temperament": "bold", "default_intent": "guard", "ai_role": "SUPPORT"})).as_object().unwrap());
    assert_eq!((set["temperament"].clone(), set["default_intent"].clone(), set["ai_role"].clone()), (json!("bold"), json!("guard"), json!("SUPPORT")));
}
