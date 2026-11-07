//! P1.2-S1 (FB-51) and P2.2-S1 (FB-40b) on scratch data folders: terrain `walkable` and `stops_forced_move`, and the
//! ruleset number `move_after_blocks`, are schema fields. A terrain row without the keys loads with its type's fallback
//! (`grid::terrain_walkable`, then not walkable), not the schema default; a row with them loads them as given; the
//! terrain checks read the row's `walkable`. The writer stores the keys only when the row gives them, so a file written
//! before they existed re-saves byte for byte and an explicit value is kept. `move_after_blocks` has the code default
//! (`ruleset::MOVE_AFTER_BLOCKS`) and the spec's range 1..4, reads as set, and is stored only when given.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_battle::grid::{terrain_walkable, TERRAIN_IDS};
use rpgkit_content::{check, source, store};
use rpgkit_rules::ruleset::{self, Ruleset, MOVE_AFTER_BLOCKS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_terrain_fields_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

/// Writes a catalog file as a fixture (the file is the test's own input; the writer is tested below).
fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// Every terrain type's row as written before the fields existed (the frozen kit's shape).
fn bare_rows() -> Vec<Value> {
    TERRAIN_IDS.iter().map(|id| json!({"id": id, "move_cost": 1, "cover_guard": 0.0})).collect()
}

fn loaded(data: &Path) -> Vec<Dict> {
    let src = source::load(data);
    json::aget(json::dict(&src.catalog), "terrain").iter().map(|v| json::dict(v).clone()).collect()
}

fn field(section: &str, kind: &str, key: &str) -> Dict {
    fields(source::schema(), section, kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("{kind}.{key}")).clone()
}

#[test]
fn the_terrain_fields_are_schema_fields() {
    for key in source::TERRAIN_FALLBACK_KEYS {
        assert_eq!(json::s(&field("content", "terrain", key), "type"), "bool", "{key}");
    }
}

/// A row without the keys loads its type's fallback; a row with them loads them as given.
#[test]
fn a_row_without_the_keys_loads_the_fallback() {
    let (_t, data) = scratch("load");
    write_kind(&data, "terrain", Value::Array(bare_rows()));
    let rows = loaded(&data);
    assert_eq!(rows.len(), TERRAIN_IDS.len());
    for r in &rows {
        let id = json::s(r, "id");
        let walkable = terrain_walkable(&id);
        assert_eq!((json::to_bool(&r["walkable"]), json::to_bool(&r["stops_forced_move"])), (walkable, !walkable), "{id}");
    }
    let mut rows = bare_rows();
    rows[TERRAIN_IDS.iter().position(|t| *t == "wall").unwrap()]["walkable"] = json!(true);
    rows[TERRAIN_IDS.iter().position(|t| *t == "grass").unwrap()]["stops_forced_move"] = json!(true);
    write_kind(&data, "terrain", Value::Array(rows));
    let rows = loaded(&data);
    let at = |id: &str| rows.iter().find(|r| json::s(r, "id") == id).map(|r| (json::to_bool(&r["walkable"]), json::to_bool(&r["stops_forced_move"]))).unwrap();
    assert_eq!(at("wall"), (true, false), "walkable as given, stops follows it");
    assert_eq!(at("grass"), (true, true), "stops as given");
}

/// The terrain checks read the row's `walkable`: a hazard on a row made unwalkable is refused, a move cost outside 1-3
/// on a row made walkable is refused, and neither on the type's fallback.
#[test]
fn the_checks_read_the_row() {
    let (_t, data) = scratch("checks");
    write_kind(&data, "terrain", Value::Array(bare_rows()));
    let v = store::view(&data);
    let errors = |e: Value| {
        let e = dict(e);
        check::validate_entry(&v, "terrain", &e, &json::s(&e, "id"))
    };
    let hazard = "Only walkable terrain can carry a hazard.";
    let cost = "Move cost must be 1-3.";
    assert!(!errors(json!({"id": "grass", "hazard_status": "slow"})).iter().any(|m| m == hazard));
    assert!(errors(json!({"id": "grass", "walkable": false, "hazard_status": "slow"})).iter().any(|m| m == hazard));
    assert!(errors(json!({"id": "wall", "hazard_status": "slow"})).iter().any(|m| m == hazard), "the fallback blocks walls");
    assert!(!errors(json!({"id": "wall", "move_cost": 5})).iter().any(|m| m == cost));
    assert!(errors(json!({"id": "wall", "walkable": true, "move_cost": 5})).iter().any(|m| m == cost));
}

/// The writer keeps a file without the keys byte for byte and stores an explicit value (upsert), and only that one.
#[test]
fn the_writer_stores_the_keys_only_when_given() {
    let (_t, data) = scratch("store");
    write_kind(&data, "terrain", Value::Array(bare_rows()));
    let file = data.join("catalog/terrain.json");
    let text = store::resave_text(&data, None, "terrain").unwrap();
    fs::write(&file, &text).unwrap();
    assert_eq!(store::resave_text(&data, None, "terrain").unwrap(), text, "stable");
    assert!(!text.contains("walkable") && !text.contains("stops_forced_move"), "no keys written for rows without them");
    store::upsert(&data, None, "terrain", &dict(json!({"id": "deep_water", "walkable": false, "stops_forced_move": true, "move_cost": 1, "cover_guard": 0.0})), None).unwrap();
    let stored: Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
    let row = |id: &str| stored["entries"].as_array().unwrap().iter().find(|r| r["id"] == json!(id)).unwrap().clone();
    assert_eq!((row("deep_water")["walkable"].clone(), row("deep_water")["stops_forced_move"].clone()), (json!(false), json!(true)));
    assert!(row("floor").get("walkable").is_none(), "the other rows keep their shape");
    let after = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, "terrain").unwrap(), after, "a re-save without changes is byte-identical");
}

/// `move_after_blocks`: the schema's default is the code's and its range the spec's (1..4); a row reads it as set; the
/// writer stores it only when the row gives it.
#[test]
fn move_after_blocks_is_a_schema_field() {
    let f = field("content", ruleset::KIND, "move_after_blocks");
    assert_eq!((json::s(&f, "type"), json::to_int(&f["default"]), json::to_int(&f["min"]), json::to_int(&f["max"])), ("int".to_string(), i64::from(MOVE_AFTER_BLOCKS), 1, 4));
    let read = |v: Value| Ruleset::from_row(&aegis_content::schema::normalize(fields(source::schema(), "content", ruleset::KIND), &dict(v), false)).numbers.move_after_blocks;
    assert_eq!(read(json!({"id": "classic", "name": "Classic"})), MOVE_AFTER_BLOCKS, "left out: the default");
    assert_eq!(read(json!({"id": "classic", "name": "Classic", "move_after_blocks": 2})), 2, "as set");

    let (_t, data) = scratch("move_after");
    write_kind(&data, ruleset::KIND, json!([{"id": "classic", "name": "Classic"}]));
    let file = data.join("catalog").join(format!("{}.json", ruleset::KIND));
    let text = store::resave_text(&data, None, ruleset::KIND).unwrap();
    assert!(!text.contains("move_after_blocks"), "a row without it is stored without it");
    fs::write(&file, &text).unwrap();
    store::upsert(&data, None, ruleset::KIND, &dict(json!({"id": "classic", "name": "Classic", "move_after_blocks": 2})), None).unwrap();
    let stored: Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(stored["entries"][0]["move_after_blocks"], json!(2), "a given value is stored");
}
