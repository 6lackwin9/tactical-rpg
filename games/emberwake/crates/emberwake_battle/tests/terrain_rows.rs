//! P1.2-S1 (FB-51) on the game's terrain rows: the shipped rows (read only, through the loader) and the frozen kit's
//! rows (`tests/fixtures/timeline_kit.json`, the rules / timeline / sim goldens' catalog) read the same `walkable`,
//! `stops_forced_move`, move cost, cover and hazard as the code's old rule: walkable unless the type is one the old
//! fallback blocks (`grid::terrain_walkable`), forced moves stopped exactly where not walkable, whether a row carries
//! the keys or leaves them out (the keys are stripped to compare). Expectations come from the rule, not shipped numbers.
//!
//! The terrain rows are game data, so the test lives in the game's façade crate (the kit names no game).

use emberwake_battle::grid::{terrain_walkable, TerrainTable, TERRAIN_IDS};
use emberwake_data::Catalog;
use emberwake_world::Data;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn data_dir() -> PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn frozen() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/timeline_kit.json");
    serde_json::from_str(&std::fs::read_to_string(&path).expect("timeline_kit.json")).unwrap()
}

/// The catalog with every terrain row's `walkable` and `stops_forced_move` taken out (the missing-key path).
fn stripped(catalog: &Value) -> Value {
    let mut out = catalog.clone();
    for row in out["terrain"].as_array_mut().expect("terrain rows") {
        let o = row.as_object_mut().unwrap();
        o.shift_remove("walkable");
        o.shift_remove("stops_forced_move");
    }
    out
}

fn assert_old_rule(what: &str, catalog: &Value) {
    let given = TerrainTable::from_catalog(&Catalog::from_value(catalog));
    let fallback = TerrainTable::from_catalog(&Catalog::from_value(&stripped(catalog)));
    assert_eq!(catalog["terrain"].as_array().map(Vec::len), Some(TERRAIN_IDS.len()), "{what}: a row per type");
    for (i, id) in TERRAIN_IDS.iter().enumerate() {
        let (g, f) = (&given.rows[i], &fallback.rows[i]);
        assert_eq!(g.walkable, terrain_walkable(id), "{what}: {id} walkable");
        assert_eq!(g.stops_forced_move, !g.walkable, "{what}: {id} stops forced moves");
        assert_eq!((g.walkable, g.stops_forced_move, g.move_cost, g.cover_guard, &g.hazard), (f.walkable, f.stops_forced_move, f.move_cost, f.cover_guard, &f.hazard), "{what}: {id} with and without the keys");
    }
}

#[test]
fn shipped_rows_read_the_old_rule() {
    let src = emberwake_content::source::load(&data_dir());
    assert!(src.errors.is_empty(), "{:?}", src.errors);
    assert_old_rule("shipped", &src.catalog);
}

#[test]
fn frozen_rows_read_the_old_rule() {
    assert_old_rule("frozen", &frozen());
}
