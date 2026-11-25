//! PK2-S5a (`docs/sessions/PK2_GRAFTS.md` 6-7) on scratch data built here (no shipped data): a WEAPON family's
//! `durability` (the full durability of its weapons) rides on its items' resolved rows (`family_durability`) and stays out
//! of the stored item form; a WEAPON item's own `durability` overrides it (`ItemDef::max_durability`); a family or item
//! without one stores as before (no key) and reads 0 (the weapon never wears); the writer holds a value at the schema's
//! top and leaves it off a non-weapon family, whose raw row the check refuses.

use aegis_core::json::Dict;
use aegis_core::scratch::Scratch;
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, DURABILITY_MAX, ITEM_DURABILITY, ITEM_FAMILY_DURABILITY};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const FAMILY: i64 = 30;
const OWN: i64 = 45;

fn weapon_family(id: &str, extra: Value) -> Value {
    let mut f = json!({"id": id, "name": id, "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"], "damage_type": "slash",
        "base": {"main_1h": {"power": 10, "range": 1, "burden": 0, "mitigation": 0}}});
    for (k, x) in extra.as_object().unwrap() {
        f[k] = x.clone();
    }
    f
}

fn write(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn scratch(tag: &str, families: Value, items: Value) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_pk2s5a_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    write(&data, "skills", 2, json!([]));
    write(&data, "item_families", 1, families);
    write(&data, "items", 2, items);
    write(&data, "classes", 1, json!([]));
    (t, data)
}

fn schema_max(kind: &str) -> i64 {
    let fields = source::schema()["content"][kind].as_array().unwrap();
    fields.iter().find(|f| f["key"] == ITEM_DURABILITY).and_then(|f| f["max"].as_i64()).expect("the durability field's max")
}

#[test]
fn the_familys_durability_rides_on_the_items_and_the_items_own_overrides_it() {
    let sword = weapon_family("t_sword", json!({ITEM_DURABILITY: FAMILY}));
    let axe = weapon_family("t_axe", json!({}));
    let items = json!([
        {"id": "i_sword", "name": "Sword", "family": "t_sword", "hand": "main_1h"},
        {"id": "i_heirloom", "name": "Heirloom", "family": "t_sword", "hand": "main_1h", ITEM_DURABILITY: OWN},
        {"id": "i_axe", "name": "Axe", "family": "t_axe", "hand": "main_1h"}
    ]);
    let (_t, data) = scratch("ride", json!([sword.clone(), axe.clone()]), items);
    let src = source::load(&data);
    assert!(src.errors.iter().all(|e| e.starts_with("XP curve file")), "{:?}", src.errors);
    let cat = Catalog::from_value(&src.catalog);
    let max = |id: &str| i64::from(cat.items[id].max_durability());
    assert_eq!((max("i_sword"), max("i_heirloom"), max("i_axe")), (FAMILY, OWN, 0), "the family's, the item's own over it, none");
    assert!(cat.items.values().all(|i| i.is_weapon()));
    let axe_row = src.catalog["items"].as_array().unwrap().iter().find(|r| r["id"] == "i_axe").unwrap().clone();
    assert!(axe_row.get(ITEM_FAMILY_DURABILITY).is_none(), "no key for a family without one: {axe_row}");
    let fams: Vec<Dict> = src.game["item_families"].as_array().unwrap().iter().filter_map(|f| f.as_object().cloned()).collect();
    let stored = store::catalog_storage("items", json!({"id": "i_sword", "name": "Sword", "family": "t_sword", "hand": "main_1h"}).as_object().unwrap(), &fams).unwrap();
    assert!(!stored.contains_key(ITEM_FAMILY_DURABILITY) && !stored.contains_key(ITEM_DURABILITY), "{stored:?}");
    let own = store::catalog_storage("items", json!({"id": "i_heirloom", "name": "Heirloom", "family": "t_sword", "hand": "main_1h", ITEM_DURABILITY: OWN}).as_object().unwrap(), &fams).unwrap();
    assert_eq!(own.get(ITEM_DURABILITY), Some(&json!(OWN)), "an item's own is stored");
    let with = store::catalog_storage("item_families", sword.as_object().unwrap(), &[]).unwrap();
    assert_eq!(with.get(ITEM_DURABILITY), Some(&json!(FAMILY)));
    let without = store::catalog_storage("item_families", axe.as_object().unwrap(), &[]).unwrap();
    assert!(!without.contains_key(ITEM_DURABILITY), "none stores as no key: {without:?}");
    let v = store::view(&data);
    for id in ["i_sword", "i_heirloom", "i_axe"] {
        let errors = check::validate_entry(&v, "items", v.get("items", id), id);
        assert!(errors.is_empty(), "{id}: {errors:?}");
    }
    let errors = check::validate_entry(&v, "item_families", v.get("item_families", "t_sword"), "t_sword");
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn a_durability_past_the_schema_range_is_held_at_it_and_needs_a_weapon() {
    let (fam_max, item_max) = (schema_max("item_families"), schema_max("items"));
    assert_eq!((fam_max, item_max), (DURABILITY_MAX, DURABILITY_MAX), "the schema's range is the check's");
    let sword = weapon_family("t_sword", json!({ITEM_DURABILITY: DURABILITY_MAX + 1}));
    let stored = store::catalog_storage("item_families", sword.as_object().unwrap(), &[]).unwrap();
    assert_eq!(stored.get(ITEM_DURABILITY), Some(&json!(DURABILITY_MAX)), "the writer holds a family's at the top");
    let fams = vec![weapon_family("t_sword", json!({})).as_object().unwrap().clone()];
    let item = store::catalog_storage("items", json!({"id": "i_h", "name": "H", "family": "t_sword", "hand": "main_1h", ITEM_DURABILITY: DURABILITY_MAX + 1}).as_object().unwrap(), &fams).unwrap();
    assert_eq!(item.get(ITEM_DURABILITY), Some(&json!(DURABILITY_MAX)), "and an item's own");
    // only weapons: the writer leaves it off another family, the check refuses it on a raw row
    let plate = json!({"id": "t_plate", "name": "t_plate", "item_type": "ARMOR", "slots": ["BODY"], ITEM_DURABILITY: 1});
    let stored = store::catalog_storage("item_families", plate.as_object().unwrap(), &[]).unwrap();
    assert!(!stored.contains_key(ITEM_DURABILITY), "{stored:?}");
    let (_t, data) = scratch("armor", json!([plate]), json!([]));
    let v = store::view(&data);
    let errors = check::validate_entry(&v, "item_families", v.get("item_families", "t_plate"), "t_plate");
    assert!(errors.contains(&"Durability needs a WEAPON (only weapons become instances).".to_string()), "{errors:?}");
}
