//! A1.2-K3 (INTENT2 section 6, INTENT2-K3; A3.1 section 1 / IR9 for `affects`) on scratch data folders: the consumable
//! fields `area` (`NONE`, `PLUS`, `BURST`; default `NONE`) and `affects` ("" = the row's default, `all`, `enemies`,
//! `allies`) are schema fields of CONSUMABLE items, loaded into the typed catalog, written only when set (an item
//! written before they existed re-saves byte for byte), and checked: an area on an item that heals or buffs, or
//! `affects` on a single-target item, is one error; a shape outside the three options is an unknown value.

use aegis_content::schema::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Affects, ITEM_AREAS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_item_area_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn field_of(key: &str) -> Dict {
    fields(source::schema(), "content", "items").iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("content.items.{key}")).clone()
}

/// The consumable family, a potion and a single-target bomb (stored shape, no `area` / `affects`).
fn catalog(data: &Path) {
    let family = json!({"id": "consumable", "name": "Consumable", "description": "", "item_type": "CONSUMABLE", "off_hand_category": "", "base": {}, "bands": {}, "adjustable": [], "modifier_kinds": [], "max_modifiers": 0, "max_tiers": 0, "engine_rules": [], "damage_type": "", "art": ""});
    fs::write(data.join("catalog").join("item_families.json"), json!({"kind": "item_families", "version": 1, "entries": [family]}).to_string()).unwrap();
    let item = |id: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "family": "consumable", "rarity": "COMMON", "weight": 0.5, "value": 10, "description": "A test item.", "damage_type": "", "icon_id": "", "starter_stash": false, "consumable_type": "HEALTH_POTION", "heal": 0, "damage": 0, "buff_stat": "", "buff_pct": 0.0, "duration": 0, "use_blocks": 1, "use_reach": 1, "hazard": {}, "max_stack": 2});
        e.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        e
    };
    let items = json!([item("t_potion", json!({"heal": 40})), item("t_bomb", json!({"consumable_type": "THROWABLE_BOMB", "damage": 40, "use_reach": 3}))]);
    fs::write(data.join("catalog").join("items.json"), json!({"kind": "items", "version": 2, "entries": items}).to_string()).unwrap();
}

#[test]
fn the_schema_names_area_and_affects_for_consumables() {
    let area = field_of("area");
    assert_eq!((json::s(&area, "type"), json::s(&area, "default")), ("enum".to_string(), "NONE".to_string()));
    assert_eq!(json::aget(&area, "options").iter().map(gstr).collect::<Vec<_>>(), ITEM_AREAS, "NONE and the two bomb shapes");
    assert_eq!(json::aget(json::dget(&area, "when"), "in").iter().map(gstr).collect::<Vec<_>>(), ["CONSUMABLE"]);
    let affects = field_of("affects");
    let mut want = vec![String::new()];
    want.extend(Affects::IDS.iter().map(|s| s.to_string()));
    assert_eq!(json::aget(&affects, "options").iter().map(gstr).collect::<Vec<_>>(), want);
}

#[test]
fn area_loads_and_is_written_only_when_set() {
    let (_scratch, data) = scratch("load");
    catalog(&data);
    let text = fs::read_to_string(store::file_of(&data, None, "items")).unwrap();
    let before = store::resave_text(&data, None, "items").unwrap();
    let cat = rpgkit_data::Catalog::from_value(&source::load(&data).catalog);
    let bomb = cat.items.get("t_bomb").unwrap();
    assert_eq!((bomb.area.as_str(), bomb.affects, bomb.is_area()), ("NONE", None, false), "unset: single target");
    let mut entry = dict(serde_json::from_str::<Value>(&text).unwrap()["entries"][1].clone());
    entry.insert("area".into(), json!("PLUS"));
    store::upsert(&data, None, "items", &entry, None).expect("a bomb with an area");
    let stored: Value = serde_json::from_str(&fs::read_to_string(store::file_of(&data, None, "items")).unwrap()).unwrap();
    let row = |id: &str| stored["entries"].as_array().unwrap().iter().find(|e| e["id"] == id).unwrap().clone();
    assert_eq!(row("t_bomb")["area"], json!("PLUS"));
    assert!(row("t_potion").get("area").is_none() && row("t_potion").get("affects").is_none(), "unset fields are not written");
    let cat = rpgkit_data::Catalog::from_value(&source::load(&data).catalog);
    assert!(cat.items.get("t_bomb").unwrap().is_area());
    let now = fs::read_to_string(store::file_of(&data, None, "items")).unwrap();
    assert_eq!(store::resave_text(&data, None, "items").unwrap(), now, "a re-save without changes is byte-identical");
    assert!(before.contains("t_potion") && !before.contains("\"area\""), "the pre-K3 file re-saves without the new keys");
}

#[test]
fn each_broken_rule_is_one_error() {
    let (_scratch, data) = scratch("rules");
    catalog(&data);
    let v = store::view(&data);
    let errors = |id: &str, change: Value| {
        let mut e = v.get("items", id).clone();
        for (k, x) in change.as_object().unwrap() {
            e.insert(k.clone(), x.clone());
        }
        check::validate_entry(&v, "items", &e, id)
    };
    assert_eq!(errors("t_bomb", json!({})), Vec::<String>::new(), "the fixture is clean");
    for shape in ["PLUS", "BURST"] {
        assert_eq!(errors("t_bomb", json!({"area": shape, "affects": "all"})), Vec::<String>::new(), "{shape}");
    }
    let healing = errors("t_potion", json!({"area": "PLUS"}));
    assert!(healing.len() == 1 && healing[0].starts_with("Area: only a damage item"), "{healing:?}");
    assert_eq!(errors("t_bomb", json!({"affects": "enemies"})), ["Affects: a single-target item has no area to affect."]);
    let bogus = errors("t_bomb", json!({"area": "LINE"}));
    assert_eq!(bogus.len(), 1, "{bogus:?}");
}
