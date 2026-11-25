//! W.2-S3 (FB-139) content side on fixture rows (no shipped data): a unit's `growth_per_stage` is an int field of the
//! units table (default 0, 0-`GROWTH_PER_STAGE_MAX`); the writer stores 0 as no key (a unit written before the field
//! existed re-saves byte for byte) and any other value as written; the check refuses a value that is not a whole number
//! in range (the field table would clamp it silently); the loader reads it (0 when absent).

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{self, unit::{GROWTH_PER_STAGE, GROWTH_PER_STAGE_MAX}};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn unit_row(growth: Option<Value>) -> Dict {
    let mut row = json::dict(&json!({"id": "wolf", "name": "Wolf", "side": "enemy", "class_id": "drill", "level": 2, "tier": "standard"})).clone();
    if let Some(g) = growth {
        row.insert(GROWTH_PER_STAGE.into(), g);
    }
    row
}

#[test]
fn the_units_table_has_the_field_with_its_range() {
    let f = fields(source::schema(), "content", "units").iter().map(json::dict).find(|f| json::s(f, "key") == GROWTH_PER_STAGE).cloned().expect("the units table has growth_per_stage");
    assert_eq!(json::s(&f, "type"), "int");
    assert_eq!((json::i(&f, "default", -1), json::i(&f, "min", -1), json::i(&f, "max", -1)), (0, 0, GROWTH_PER_STAGE_MAX));
}

#[test]
fn the_writer_stores_zero_as_no_key_and_any_other_value_as_written() {
    for growth in [None, Some(json!(0))] {
        let row = store::catalog_storage("units", &unit_row(growth.clone()), &[]).unwrap();
        assert!(!row.contains_key(GROWTH_PER_STAGE), "{growth:?}: stored as no key");
    }
    let row = store::catalog_storage("units", &unit_row(Some(json!(3))), &[]).unwrap();
    assert_eq!(row.get(GROWTH_PER_STAGE), Some(&json!(3)));
}

#[test]
fn the_check_refuses_a_value_out_of_range_or_not_whole() {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_growth_per_stage");
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    let base = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    write_kind(&data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": base, "growth": {"STR": 2, "CONS": 2, "DEX": 1}}]));
    write_kind(&data, "units", json!([]));
    let v = store::view(&data);
    let msg = format!("Growth per stage must be a whole number 0-{GROWTH_PER_STAGE_MAX}.");
    let refused = |growth: Option<Value>| check::validate_entry(&v, "units", &unit_row(growth), "").contains(&msg);
    for ok in [None, Some(json!(0)), Some(json!(2)), Some(json!(GROWTH_PER_STAGE_MAX))] {
        assert!(!refused(ok.clone()), "{ok:?} is allowed");
    }
    for bad in [json!(-1), json!(GROWTH_PER_STAGE_MAX + 1), json!(1.5), json!("2")] {
        assert!(refused(Some(bad.clone())), "{bad} is refused");
    }
}

#[test]
fn the_loader_reads_the_field_and_zero_when_absent() {
    assert_eq!(rpgkit_data::UnitDef::from_obj(&unit_row(None)).growth_per_stage, 0);
    assert_eq!(rpgkit_data::UnitDef::from_obj(&unit_row(Some(json!(2)))).growth_per_stage, 2);
}
