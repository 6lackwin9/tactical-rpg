//! ENGINE-43 (TERRAIN-PARTIAL1): a game's terrain table lists only the terrains it changes. The others take the kit's
//! code defaults quietly (no problem and no warning per missing row), and a listed row is still checked whole.

use aegis_core::json::Dict;
use rpgkit_battle::grid::{TerrainTable, TERRAIN_IDS};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_terrain_partial_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn terrain_lines(lines: &[String]) -> Vec<&String> {
    lines.iter().filter(|l| l.starts_with("terrain")).collect()
}

#[test]
fn a_one_row_terrain_table_loads_with_defaults_for_the_rest_and_no_warnings() {
    let (_t, data) = scratch("one_row");
    let none_before = rpgkit_content::store::warnings(&data);
    assert!(terrain_lines(&none_before).is_empty(), "no terrain file: no warning per terrain ({none_before:?})");
    let floor = dict(json!({"id": "floor", "move_cost": 2, "cover_guard": 0.25}));
    let written = rpgkit_content::store::upsert(&data, None, "terrain", &floor, None);
    assert!(written.as_ref().err().is_none(), "one row is a whole table now: {:?}", written.err());
    let (problems, warnings) = (rpgkit_content::store::check(&data), rpgkit_content::store::warnings(&data));
    assert!(terrain_lines(&problems).is_empty(), "{problems:?}");
    assert!(terrain_lines(&warnings).is_empty(), "{warnings:?}");
    let src = rpgkit_content::source::load(&data);
    let table = TerrainTable::from_catalog(&rpgkit_data::Catalog::from_value(&src.catalog));
    let defaults = TerrainTable::builtin();
    assert_eq!(table.ids, defaults.ids, "every kit terrain type is there");
    let at = |t: &TerrainTable, id: &str| t.ids.iter().position(|x| x == id).unwrap();
    let floor = &table.rows[at(&table, "floor")];
    assert_eq!((floor.move_cost, floor.cover_guard), (2, 0.25), "the listed row's own values");
    let numbers = |r: &rpgkit_battle::grid::TerrainRow| (r.walkable, r.stops_forced_move, r.move_cost, r.cover_guard, r.hazard.is_some());
    for id in TERRAIN_IDS.iter().filter(|id| **id != "floor") {
        assert_eq!(numbers(&table.rows[at(&table, id)]), numbers(&defaults.rows[at(&defaults, id)]), "{id}: the code default");
    }
    assert!(!table.rows[at(&table, "wall")].walkable, "an unlisted wall still blocks");
}

#[test]
fn a_listed_terrain_row_is_still_checked_whole() {
    let (_t, data) = scratch("whole_row");
    let bad = dict(json!({"id": "floor", "move_cost": 5}));
    let refused = rpgkit_content::store::upsert(&data, None, "terrain", &bad, None);
    assert!(refused.is_err(), "a listed row out of range is refused");
}
