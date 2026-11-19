//! PR6-6 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 1-2, B3-B5), the typed side of the campaign hooks: a unit track row's
//! `signed` (absent = false; a signed row runs -TRACK_SIGNED_MAX..TRACK_SIGNED_MAX with no stage, passive or
//! departure), a recipe row's `tags` (absent = none) and its doctrine sum, and the kit tables the schema mirrors: the
//! trigger cause `recipe`, the `hero` filter keys and `clock_set`'s `keep_longer`.

use rpgkit_data::recipe::RecipeDef;
use rpgkit_data::{UnitTrackDef, CAMPAIGN_EFFECTS, HERO_FILTER_KEYS, TRACK_SIGNED_MAX, TRIGGER_ON};
use serde_json::{json, Value};

fn obj(v: Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap()
}

/// The kit schema's field `key` of world or content kind `kind`.
fn schema_field(section: &str, kind: &str, key: &str) -> Value {
    let raw = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema.json")).expect("the kit schema");
    let schema: Value = serde_json::from_str(&raw).expect("json");
    schema[section][kind].as_array().expect("a kind").iter().find(|f| f["key"] == key).cloned().unwrap_or_else(|| panic!("{kind}.{key}"))
}

#[test]
fn a_signed_track_runs_both_ways_with_no_stage() {
    let staged = UnitTrackDef::from_obj(&obj(json!({"id": "rot", "name": "Rot", "stages": [{"name": "One"}, {"name": "Two", "departure": "lost"}]})));
    assert!(!staged.signed, "absent = false");
    assert_eq!((staged.bottom(), staged.top(), staged.clamp(-5), staged.clamp(9)), (0, 2, 0, 2));
    let lean = UnitTrackDef::from_obj(&obj(json!({"id": "lean", "name": "Lean", "signed": true})));
    assert!(lean.signed);
    assert_eq!((lean.bottom(), lean.top()), (-TRACK_SIGNED_MAX, TRACK_SIGNED_MAX));
    assert_eq!((lean.clamp(-9), lean.clamp(-2), lean.clamp(9)), (-TRACK_SIGNED_MAX, -2, TRACK_SIGNED_MAX));
    for n in -TRACK_SIGNED_MAX..=TRACK_SIGNED_MAX {
        assert!(lean.passives_to(n).is_empty() && lean.departure_at(n).is_empty() && lean.stage(n).is_none(), "stage {n}");
    }
}

#[test]
fn a_recipe_reads_its_tags_and_sums_its_doctrine() {
    let plain = RecipeDef::from_obj(&obj(json!({"id": "a", "effects": [{"effect": "drill", "xp": 5}]})));
    assert!(plain.tags.is_empty());
    assert_eq!(plain.doctrine(), 0);
    let tagged = RecipeDef::from_obj(&obj(json!({"id": "b", "tags": ["slab"], "effects": [
        {"effect": "doctrine", "axis": "doctrine", "delta": 4}, {"effect": "install", "item": "g", "slot": "s"}, {"effect": "doctrine", "axis": "doctrine", "delta": 4}]})));
    assert_eq!(tagged.tags, ["slab"]);
    assert_eq!(tagged.doctrine(), 8);
}

#[test]
fn the_schema_mirrors_the_kit_tables() {
    let on: Vec<String> = schema_field("world", "triggers", "on")["options"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    assert_eq!(on, TRIGGER_ON);
    assert!(TRIGGER_ON.contains(&"recipe"));
    assert_eq!(HERO_FILTER_KEYS, ["strain_peak", "strain_end", "channel_actions"]);
    let clock_set = CAMPAIGN_EFFECTS.iter().find(|(k, _)| *k == "clock_set").unwrap().1;
    assert!(clock_set.contains(&"keep_longer"));
    // every new field is left out of a row while it holds its default
    for (section, kind, key) in [
        ("content", "unit_tracks", "signed"),
        ("content", "recipes", "tags"),
        ("world", "triggers", "hero"),
        ("world", "triggers", "recipe"),
        ("world", "triggers", "recipe_tags"),
        ("world", "checkpoints", "after_flag"),
        ("world", "checkpoints", "offset_days"),
    ] {
        assert_eq!(schema_field(section, kind, key)["omit_default"], json!(true), "{kind}.{key}");
    }
}
