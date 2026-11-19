//! BATTLE-UI2 (decision "Enemy short names") on fixture rows (no shipped data): the units table has an optional
//! `short_name` (default "", stored as no key), the loader reads it and the plate name falls back to `name`; the content
//! check warns on an enemy with no short name whose name is longer than a plate holds (`PLATE_NAME_MAX_CHARS`), never
//! on one with a short name, a short enough name, or a party unit.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::unit::plate_name_warnings;
use rpgkit_content::{source, store};
use rpgkit_data::{plate_name, UnitDef, PLATE_NAME_MAX_CHARS};
use serde_json::json;

fn row(name: &str, short: Option<&str>, side: &str) -> Dict {
    let mut row = json::dict(&json!({"id": "wolf", "name": name, "side": side, "class_id": "drill", "level": 2, "tier": "standard"})).clone();
    if let Some(s) = short {
        row.insert("short_name".into(), json!(s));
    }
    row
}

fn long_name() -> String {
    "W".repeat(PLATE_NAME_MAX_CHARS + 1)
}

#[test]
fn the_units_table_has_an_optional_short_name_stored_as_no_key_when_empty() {
    let f = fields(source::schema(), "content", "units").iter().map(json::dict).find(|f| json::s(f, "key") == "short_name").cloned().expect("the units table has short_name");
    assert_eq!((json::s(&f, "type"), json::s(&f, "default")), ("string".to_string(), String::new()));
    let stored = store::catalog_storage("units", &row("Wolf", Some(""), "enemy"), &[]).unwrap();
    assert!(!stored.contains_key("short_name"), "an empty short name is stored as no key");
    let stored = store::catalog_storage("units", &row("Grey Wolf", Some("Wolf"), "enemy"), &[]).unwrap();
    assert_eq!(stored.get("short_name"), Some(&json!("Wolf")));
}

#[test]
fn the_plate_name_is_the_short_name_or_falls_back_to_the_name() {
    assert_eq!(UnitDef::from_obj(&row("Grey Wolf", Some("Wolf"), "enemy")).plate_name(), "Wolf");
    assert_eq!(UnitDef::from_obj(&row("Grey Wolf", None, "enemy")).plate_name(), "Grey Wolf");
    assert_eq!(UnitDef::from_obj(&row("Grey Wolf", Some("  "), "enemy")).plate_name(), "Grey Wolf", "a blank short name falls back");
    assert_eq!(plate_name(" Wolf ", "Grey Wolf"), "Wolf");
}

#[test]
fn an_enemy_whose_long_name_has_no_short_name_is_warned() {
    let long = long_name();
    let warned = plate_name_warnings(&row(&long, None, "enemy"));
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(warned[0].contains(&long) && warned[0].contains("short name"), "{warned:?}");
    assert!(plate_name_warnings(&row(&long, Some("Wolf"), "enemy")).is_empty(), "a short name answers it");
    assert!(plate_name_warnings(&row(&"W".repeat(PLATE_NAME_MAX_CHARS), None, "enemy")).is_empty(), "a name that fits");
    assert!(plate_name_warnings(&row(&long, None, "party")).is_empty(), "heroes have their own plates");
}

#[test]
fn an_enemy_short_name_longer_than_the_plate_is_warned() {
    let long = long_name();
    let warned = plate_name_warnings(&row("Grey Wolf", Some(&long), "enemy"));
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(warned[0].contains(&long) && warned[0].contains("Short name"), "{warned:?}");
    assert!(plate_name_warnings(&row("Grey Wolf", Some(&"W".repeat(PLATE_NAME_MAX_CHARS)), "enemy")).is_empty(), "a short name that fits");
    assert!(plate_name_warnings(&row("Grey Wolf", Some(&long), "party")).is_empty(), "heroes have their own plates");
}
