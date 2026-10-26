//! RULESET1-K (docs/specs/RULESET1.md 8 K1) in the loader and the writer: an item of a family with the engine rule
//! `reach` resolves with the row key `reach_rule` (true), which the typed catalog reads as `ItemDef.reach_rule`; an item
//! of any other family resolves without the key (the Godot shape); the stored form never carries it. Scratch data only.

use aegis_core::json::Dict;
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_reach_rule_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// Two melee families, one with the engine rule `reach`, and one stored item of each.
fn families() -> Value {
    let base = json!({"main_2h": {"power": 26, "range": 1, "block_cost": 1, "burden": 2, "mitigation": 0}});
    json!([
        {"id": "t_pole", "name": "Pole", "item_type": "WEAPON", "hands": ["main_2h"], "default_hand": "main_2h", "scaling_stats": ["STR"], "base": base, "engine_rules": ["reach"]},
        {"id": "t_blade", "name": "Blade", "item_type": "WEAPON", "hands": ["main_2h"], "default_hand": "main_2h", "scaling_stats": ["STR"], "base": base, "engine_rules": []}
    ])
}

fn item(id: &str, family: &str) -> Value {
    json!({"id": id, "name": id, "family": family, "hand": "main_2h", "modifiers": [], "tiers": []})
}

#[test]
fn a_reach_family_marks_its_items_and_the_stored_form_leaves_it_out() {
    let data = scratch("load");
    write_kind(&data, "item_families", 1, families());
    write_kind(&data, "items", 2, json!([item("t_hook", "t_pole"), item("t_sabre", "t_blade")]));
    let src = source::load(&data);
    let rows: Vec<&Dict> = src.catalog["items"].as_array().unwrap().iter().filter_map(Value::as_object).collect();
    let row = |id: &str| rows.iter().find(|r| r["id"] == id).copied().unwrap_or_else(|| panic!("item {id}"));
    assert_eq!(row("t_hook").get(rpgkit_data::ITEM_REACH_RULE), Some(&json!(true)), "K1: the reach family's item carries the rule");
    assert!(row("t_sabre").get(rpgkit_data::ITEM_REACH_RULE).is_none(), "any other family's item keeps its shape");
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert!(cat.items["t_hook"].reach_rule);
    assert!(!cat.items["t_sabre"].reach_rule);
    // the writer: the stored form of a reach item (resolved, then the derived keys left out) has no rule key
    let fams: Vec<Dict> = src.game["item_families"].as_array().unwrap().iter().filter_map(|f| f.as_object().cloned()).collect();
    let stored = store::catalog_storage("items", item("t_hook", "t_pole").as_object().unwrap(), &fams).expect("stored form");
    assert!(stored.get(rpgkit_data::ITEM_REACH_RULE).is_none(), "the rule is derived from the family, never stored");
    assert_eq!(stored.get("family"), Some(&json!("t_pole")));
    let _ = fs::remove_dir_all(data.parent().unwrap());
}
