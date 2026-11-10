//! PR5-28 (docs/sessions/U3_GEAR.md FB-36c, A3_SHAPES.md: "range_band.max on the family replaces the hardcoded
//! REACH_WEAPON_REACH = 2"): a family's `range_band` `{min, max}` rides on the resolved item row as its max
//! (`ITEM_FAMILY_REACH_MAX`) and stays out of the item's stored form; an empty one stores as no key, so a family written
//! before it re-saves as before and its items read the fallback reach (`REACH_WEAPON_REACH`); the family check reads its
//! shape (test-only values, scratch data). The schema no longer offers the retired gear number `block_cost`.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use aegis_core::scratch::Scratch;
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, ITEM_FAMILY_REACH_MAX};
use rpgkit_rules::gear::Equipment;
use rpgkit_rules::unit::REACH_WEAPON_REACH;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn family(id: &str, extra: Value) -> Value {
    let mut f = json!({"id": id, "name": id, "item_type": "WEAPON", "hands": ["main_2h"], "default_hand": "main_2h", "scaling_stats": ["STR"], "damage_type": "pierce",
                       "base": {"main_2h": {"power": 20, "range": 1, "burden": 1, "mitigation": 0}}});
    for (k, x) in extra.as_object().unwrap() {
        f[k] = x.clone();
    }
    f
}

fn write(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn scratch(tag: &str, families: Value, items: Value) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_pr528_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    write(&data, "item_families", 1, families);
    write(&data, "items", 2, items);
    (t, data)
}

fn band_errors(data: &Path, id: &str) -> Vec<String> {
    let v = store::view(data);
    check::validate_entry(&v, "item_families", v.get("item_families", id), id).into_iter().filter(|e| e.contains("Range band")).collect()
}

#[test]
fn the_range_band_max_rides_on_the_item_and_stays_out_of_the_stored_forms() {
    let reach = json!({"engine_rules": ["reach"], "range_band": {"min": 1, "max": 3}});
    let (_t, data) = scratch(
        "ride",
        json!([family("t_pole", reach.clone()), family("t_spear", json!({"engine_rules": ["reach"]}))]),
        json!([{"id": "i_pole", "name": "Pole", "family": "t_pole"}, {"id": "i_spear", "name": "Spear", "family": "t_spear"}]),
    );
    let src = source::load(&data);
    assert!(src.errors.iter().all(|e| !e.contains("ange band") && !e.contains("i_pole") && !e.contains("i_spear")), "{:?}", src.errors);
    let cat = Catalog::from_value(&src.catalog);
    let (pole, spear) = (&cat.items["i_pole"], &cat.items["i_spear"]);
    assert_eq!((pole.family_reach_max, spear.family_reach_max), (3, 0));
    assert_eq!(Equipment::new(pole).reach_max(), 3, "the family's range_band.max");
    assert_eq!(Equipment::new(spear).reach_max(), REACH_WEAPON_REACH, "no range_band: the fallback");
    assert!(Equipment::new(pole).has_reach() && Equipment::new(spear).has_reach());
    let spear_row = src.catalog["items"].as_array().unwrap().iter().find(|r| r["id"] == "i_spear").unwrap().clone();
    assert!(spear_row.get(ITEM_FAMILY_REACH_MAX).is_none(), "a family without a range_band adds no key: {spear_row}");
    let fams: Vec<Dict> = src.game["item_families"].as_array().unwrap().iter().filter_map(|f| f.as_object().cloned()).collect();
    let stored = store::catalog_storage("items", json!({"id": "i_pole", "name": "Pole", "family": "t_pole"}).as_object().unwrap(), &fams).unwrap();
    assert!(!stored.contains_key(ITEM_FAMILY_REACH_MAX), "{stored:?}");
    let with = store::catalog_storage("item_families", family("t_pole", reach).as_object().unwrap(), &[]).unwrap();
    assert_eq!(with["range_band"], json!({"min": 1, "max": 3}));
    let without = store::catalog_storage("item_families", family("t_spear", json!({"engine_rules": ["reach"]})).as_object().unwrap(), &[]).unwrap();
    assert!(!without.contains_key("range_band"), "an empty one stores as no key: {without:?}");
    assert!(band_errors(&data, "t_pole").is_empty() && band_errors(&data, "t_spear").is_empty(), "{:?}", band_errors(&data, "t_pole"));
}

#[test]
fn the_family_check_reads_the_range_band_shape() {
    let reach = |band: Value| json!({"engine_rules": ["reach"], "range_band": band});
    let cases: [(Value, &str); 8] = [
        (reach(json!({})), ""),
        (reach(json!({"min": 1, "max": 2})), ""),
        (reach(json!({"min": 1, "max": 6})), ""),
        (json!({"range_band": {"min": 1, "max": 2}}), "Range band needs the engine rule reach (the reach rule's melee band)."),
        (reach(json!({"min": 2, "max": 3})), "Range band min must be 1 (a melee band starts next to the target)."),
        (reach(json!({"min": 1, "max": 7})), "Range band max must be a whole number 2-6."),
        (reach(json!({"min": 1, "max": 2, "far": 1})), "Range band: unknown key 'far' (min, max)."),
        (reach(json!([1, 2])), "Range band must be an object: {} or {\"min\": 1, \"max\": n}."),
    ];
    for (i, (extra, want)) in cases.into_iter().enumerate() {
        let (_t, data) = scratch(&format!("check{i}"), json!([family("t_fam", extra.clone())]), json!([]));
        let errors = band_errors(&data, "t_fam");
        if want.is_empty() {
            assert!(errors.is_empty(), "{extra}: {errors:?}");
        } else {
            assert!(errors.iter().any(|e| e == want), "{extra}: {errors:?}");
        }
    }
}

/// PR5-28 (3): the schema documents the family field and the engine rule `reach`, and no longer offers the retired gear
/// number `block_cost` (U3.2-S1, `source::RETIRED_NUMBERS`) as an adjustable number.
#[test]
fn the_schema_documents_range_band_and_drops_block_cost() {
    let field = |key: &str| fields(source::schema(), "content", "item_families").iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned();
    let band = field("range_band").expect("item_families.range_band");
    assert_eq!(json::get(&band, "default"), &json!({}));
    assert!(json::s(&field("engine_rules").unwrap(), "help").contains("reach"));
    let adjustable = field("adjustable").unwrap();
    for key in ["default", "options"] {
        assert!(!json::get(&adjustable, key).to_string().contains("block_cost"), "adjustable {key}");
    }
    assert!(!json::s(&field("base").unwrap(), "help").contains("block cost"));
}
