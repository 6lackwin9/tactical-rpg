//! PR4-2 (FB-146): consumables have no `buff_stat` any more (read by nothing; a buff item grants `empower` at its
//! `buff_pct`). The schema has no such field; stored data that still names it loads without it (the loader ignores the
//! key, `source::RETIRED_ITEM_KEYS`) and the writer's stored form leaves it out, so a resave drops it instead of keeping
//! it as an unknown key; the item checks raise nothing about it. Scratch data only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const RETIRED: &str = "buff_stat";

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_pr42_buff_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn family() -> Value {
    json!({"id": "consumable", "name": "Consumable", "description": "", "item_type": "CONSUMABLE", "off_hand_category": "", "base": {}, "bands": {}, "adjustable": [], "modifier_kinds": [], "max_modifiers": 0, "max_tiers": 0, "engine_rules": [], "damage_type": "", "art": ""})
}

/// A buff tonic stored before PR4-2: `buff_stat` set beside its `buff_pct` (or without it: `with` false).
fn tonic(with: bool) -> Dict {
    let mut e = json!({"id": "t_tonic", "name": "Tonic", "family": "consumable", "rarity": "COMMON", "weight": 0.5, "value": 10, "description": "A test item.", "damage_type": "", "icon_id": "", "starter_stash": false, "consumable_type": "FOCUS_INCENSE", "heal": 0, "damage": 0, "buff_pct": 0.2, "duration": 2, "use_blocks": 1, "use_reach": 0, "hazard": {}, "max_stack": 2});
    if with {
        let o = e.as_object_mut().unwrap();
        // where the writer stored it (after `damage`)
        let at = o.keys().position(|k| k == "damage").unwrap() + 1;
        o.shift_insert(at, RETIRED.into(), json!("STR"));
    }
    e.as_object().cloned().unwrap()
}

fn write(data: &Path, item: Dict) {
    fs::write(data.join("catalog").join("item_families.json"), json!({"kind": "item_families", "version": 1, "entries": [family()]}).to_string()).unwrap();
    fs::write(data.join("catalog").join("items.json"), json!({"kind": "items", "version": 2, "entries": [item]}).to_string()).unwrap();
}

#[test]
fn the_schema_has_no_buff_stat() {
    assert!(fields(source::schema(), "content", "items").iter().map(json::dict).all(|f| json::s(f, "key") != RETIRED));
}

#[test]
fn a_stored_buff_stat_is_ignored_by_the_loader_and_the_checks() {
    let (_scratch, data) = scratch("load");
    write(&data, tonic(true));
    let src = source::load(&data);
    assert!(src.errors.iter().all(|e| !e.contains(RETIRED)), "no load problem names buff_stat: {:?}", src.errors);
    let item = src.catalog["items"].as_array().unwrap().iter().find(|r| r["id"] == "t_tonic").expect("item").clone();
    assert!(item.get(RETIRED).is_none(), "the loaded item names no buff_stat: {item}");
    let v = store::view(&data);
    let errors = check::validate_entry(&v, "items", v.get("items", "t_tonic"), "t_tonic");
    assert!(errors.iter().all(|e| !e.to_lowercase().contains("buff stat")), "no check names buff_stat: {errors:?}");
}

#[test]
fn the_writer_drops_buff_stat_on_resave() {
    let (_scratch, data) = scratch("write");
    write(&data, tonic(false));
    let src = source::load(&data);
    let fams: Vec<Dict> = src.game["item_families"].as_array().unwrap().iter().filter_map(|f| f.as_object().cloned()).collect();
    let old = store::catalog_storage("items", &tonic(true), &fams).expect("stored form");
    let clean = store::catalog_storage("items", &tonic(false), &fams).expect("stored form");
    assert!(!old.contains_key(RETIRED), "stored: {old:?}");
    assert_eq!(Value::Object(old), Value::Object(clean), "the same stored row, key for key and in order, as an item without it");
}

/// An item whose family is unknown is kept as stored (a load problem names the family), but without a retired key.
#[test]
fn an_item_of_an_unknown_family_loads_without_buff_stat() {
    let (_scratch, data) = scratch("orphan");
    let mut item = tonic(true);
    item.insert("family".into(), json!("t_gone"));
    write(&data, item);
    let src = source::load(&data);
    assert!(src.errors.iter().any(|e| e.contains("t_tonic") && e.contains("t_gone")), "the unknown family is a load problem: {:?}", src.errors);
    let row = src.catalog["items"].as_array().unwrap().iter().find(|r| r["id"] == "t_tonic").expect("kept as stored").clone();
    assert_eq!(row["family"], json!("t_gone"), "kept as stored: {row}");
    assert!(row.get(RETIRED).is_none(), "the kept row names no buff_stat: {row}");
}
