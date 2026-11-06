//! U3.2-S1 (docs/sessions/U3_GEAR.md, FB-37): gear has no `block_cost` any more (priced in the budget, never read by
//! combat; the wind-up belongs to the skill). Stored data that still names it loads: the loader ignores the key in a
//! family's base and band rows and its adjustable list, an item's adjust modifier on it and a flat key; the checks raise
//! nothing about it; the writer's stored form leaves it out, so a resave drops it; the budget prices nothing for it.
//! Scratch data only.

use aegis_core::json::Dict;
use rpgkit_content::{check, source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const RETIRED: &str = "block_cost";

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_u32s1_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn band(lo: i64, hi: i64) -> Value {
    json!({"min": lo, "max": hi})
}

/// A family stored before U3.2-S1: the retired number in its base row, its band row and its adjustable list.
fn old_family() -> Value {
    json!({"id": "t_blade", "name": "Blade", "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"],
           "base": {"main_1h": {"power": 14, "range": 1, RETIRED: 2, "burden": 0, "mitigation": 0}},
           "bands": {"main_1h": {"power": band(10, 23), "range": band(1, 2), RETIRED: band(1, 3), "burden": band(0, 4), "mitigation": band(0, 0), "bonus_value": band(0, 4)}},
           "adjustable": ["power", RETIRED, "burden"], "modifier_kinds": ["adjust", "stat_bonus", "requirement"], "max_modifiers": 3})
}

/// An item stored before U3.2-S1: a flat retired key and an adjust modifier on it beside a power one.
fn old_item() -> Value {
    json!({"id": "t_sabre", "name": "Sabre", "family": "t_blade", "hand": "main_1h", RETIRED: 3,
           "modifiers": [{"kind": "adjust", "field": RETIRED, "delta": 2}, {"kind": "adjust", "field": "power", "delta": 3}], "tiers": []})
}

fn names_retired(v: &Value) -> bool {
    v.to_string().contains(RETIRED)
}

#[test]
fn stored_block_cost_is_ignored_by_the_loader_and_the_checks() {
    let (_scratch, data) = scratch("load");
    write_kind(&data, "item_families", 1, json!([old_family()]));
    write_kind(&data, "items", 2, json!([old_item()]));
    let src = source::load(&data);
    let item = src.catalog["items"].as_array().unwrap().iter().find(|r| r["id"] == "t_sabre").expect("item").clone();
    assert!(!names_retired(&item), "the resolved item names no block_cost: {item}");
    assert_eq!(item["base_power"], json!(14 + 3), "the other adjust still applies to the family base");
    assert_eq!(item["modifiers"].as_array().map(Vec::len), Some(1), "only the power adjust is kept");
    let family = src.game["item_families"].as_array().unwrap().iter().find(|f| f["id"] == "t_blade").expect("family").clone();
    assert!(!names_retired(&family), "the derived family names no block_cost: {family}");
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert_eq!(cat.items["t_sabre"].base_power, 17);
    let v = store::view(&data);
    for (kind, id) in [("item_families", "t_blade"), ("items", "t_sabre")] {
        let errors = check::validate_entry(&v, kind, v.get(kind, id), id);
        assert!(errors.iter().all(|e| !e.contains(RETIRED)), "{kind} {id}: no check names block_cost: {errors:?}");
    }
    assert!(src.errors.iter().all(|e| !e.contains(RETIRED)), "no load problem names block_cost: {:?}", src.errors);
}

#[test]
fn the_writer_drops_block_cost_on_resave() {
    let family = old_family().as_object().cloned().unwrap();
    let stored_family = store::catalog_storage("item_families", &family, &[]).expect("family stored form");
    assert!(!names_retired(&Value::Object(stored_family.clone())), "stored family: {stored_family:?}");
    assert_eq!(stored_family["adjustable"], json!(["power", "burden"]));
    let (_scratch, data) = scratch("write");
    write_kind(&data, "item_families", 1, json!([old_family()]));
    let src = source::load(&data);
    let fams: Vec<Dict> = src.game["item_families"].as_array().unwrap().iter().filter_map(|f| f.as_object().cloned()).collect();
    let stored_item = store::catalog_storage("items", old_item().as_object().unwrap(), &fams).expect("item stored form");
    assert!(!names_retired(&Value::Object(stored_item.clone())), "stored item: {stored_item:?}");
    assert_eq!(stored_item["modifiers"], json!([{"kind": "adjust", "field": "power", "delta": 3}]));
}

#[test]
fn the_budget_prices_nothing_for_block_cost() {
    use rpgkit_rules::budget::{item_value, modifier_value};
    assert_eq!(modifier_value(&json!({"kind": "adjust", "field": RETIRED, "delta": 2})), 0.0);
    let row = |extra: Value| {
        let mut o = json!({"id": "t_w", "item_type": "WEAPON", "slot": "MAIN_HAND", "base_power": 14, "burden": 1}).as_object().cloned().unwrap();
        o.extend(extra.as_object().cloned().unwrap());
        rpgkit_data::ItemDef::from_obj(&o)
    };
    assert_eq!(item_value(&row(json!({RETIRED: 3}))), item_value(&row(json!({}))), "a stored block_cost changes no item value");
}
