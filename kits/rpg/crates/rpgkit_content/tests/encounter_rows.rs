//! Authored terrain rows on encounters (ENCOUNTER2-T, docs/specs/ENCOUNTER2.md R0 and section 6): the schema carries
//! `rows` (json, default []) in both encounter sections; a battle whose encounter sets valid rows plays on exactly those
//! tiles (every glyph its terrain type), one without rows on its zone's rows as before; the checks give one error per
//! bad `rows` (wrong count, wrong length, unknown glyph, not text) in catalog and world encounters; a hold tile is
//! checked against the authored board; the writer stores and reloads the rows. Scratch data folders only.

use aegis_content::schema::{fields, normalize};
use aegis_core::json::{self, Dict};
use indexmap::IndexMap;
use rpgkit_battle::grid::{self, zone_rows, Grid, GLYPHS, GRID_H, GRID_W, TERRAIN_IDS, V2};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, UnitDef};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A scratch data folder `<tmp>/<tag>/data` with empty catalog and worlds folders.
fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_encounter_rows_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

/// Writes a catalog file as a fixture (the files are the test's own input; the writer is tested below).
fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// The zone the fixtures name: its rows differ from the authored ones, so a board built from the zone shows.
const ZONE: &str = "ruined_courtyard";

/// Valid rows using every terrain glyph: the glyphs on the edge columns, floor in the middle (the deployment columns),
/// the default exit (11, 4) on grass.
fn authored() -> Vec<String> {
    ["#,:......o*~", "WmL......T=r", "fpi......#,:", "s..........#", "...........,", "#..........#", "#..........#", "#..........#", "############"]
        .iter()
        .map(|r| r.to_string())
        .collect()
}

/// A class, the system skills and a basic attack, one hero and one enemy, and the encounter `walled` with `rows`.
fn catalog(data: &Path, rows: Value) {
    let skill = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(data, "skills", 2, json!([skill("move", "MOVE", json!({"system": true})), skill("rest", "REST", json!({"system": true})), skill("strike", "ATTACK", json!({"basic": true}))]));
    write_kind(
        data,
        "classes",
        1,
        json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2, "CONS": 2, "DEX": 1}}]),
    );
    write_kind(
        data,
        "units",
        1,
        json!([{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}, {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}]),
    );
    write_kind(data, "encounters", 1, json!([{"id": "walled", "name": "Walled", "zone": ZONE, "party": ["hero"], "enemies": ["brute"], "rows": rows}]));
}

/// The battle of `walled` on the fixture catalog.
fn battle(data: &Path) -> Session {
    let src = source::load(data);
    let cat = Catalog::from_value(&src.catalog);
    let def = EncounterDef::from_obj(&cat.encounters["walled"]);
    Session::start(Arc::new(cat), def, &IndexMap::<String, UnitDef>::new())
}

/// The terrain id of glyph `ch` (GLYPHS and TERRAIN_IDS share their order).
fn terrain_of(ch: char) -> &'static str {
    TERRAIN_IDS[GLYPHS.iter().position(|g| *g == ch).expect("a known glyph")]
}

/// Every tile's terrain id, read from `rows`.
fn board_of<'a>(rows: impl IntoIterator<Item = &'a str>) -> Vec<&'static str> {
    rows.into_iter().flat_map(|r| r.chars().map(terrain_of).collect::<Vec<_>>()).collect()
}

/// Every tile's terrain id on `g`, row-major.
fn board(g: &Grid) -> Vec<&'static str> {
    (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).map(|c| g.terrain_id(c)).collect()
}

#[test]
fn the_schema_carries_rows_in_both_encounter_sections() {
    for section in ["content", "world"] {
        let f = fields(source::schema(), section, "encounters").iter().map(json::dict).find(|f| json::s(f, "key") == "rows").unwrap_or_else(|| panic!("{section}.encounters.rows"));
        assert_eq!((json::s(f, "type"), f.get("default")), ("json".to_string(), Some(&json!([]))), "{section}");
    }
    let e = normalize(fields(source::schema(), "content", "encounters"), &dict(json!({"id": "x"})), false);
    assert_eq!(e.get("rows"), Some(&json!([])), "a row without rows reads the default");
    assert!(EncounterDef::from_obj(&e).rows.is_empty());
}

#[test]
fn authored_rows_are_the_battle_board() {
    let data = scratch("board");
    let rows = authored();
    assert!(GLYPHS.iter().all(|g| rows.iter().any(|r| r.contains(*g))), "the fixture uses every glyph");
    catalog(&data, json!(rows));
    let s = battle(&data);
    let want = board_of(rows.iter().map(String::as_str));
    assert_eq!(board(&s.w.grid), want, "terrain equals the rows");
    assert_ne!(want, board_of(zone_rows(ZONE).unwrap()), "the rows are not the zone's");
    // walls block (Terrain.is_walkable), so a wall glyph is no tile to stand on; every unit deployed on a walkable tile
    assert!(!s.w.grid.is_walkable(V2::new(0, 0)) && !grid::terrain_walkable("wall"));
    for u in 0..s.w.units.len() {
        let c = s.w.grid.pos(u).expect("deployed");
        assert!(s.w.grid.is_walkable(c), "unit {u} on {c}");
    }
    drop_scratch(&data);
}

#[test]
fn without_valid_rows_the_zone_builds_the_board() {
    let zone = board_of(zone_rows(ZONE).unwrap());
    let mut eight = authored();
    eight.pop();
    for (tag, rows) in [("none", json!([])), ("eight", json!(eight))] {
        let data = scratch(tag);
        catalog(&data, rows);
        assert_eq!(board(&battle(&data).w.grid), zone, "{tag}: the zone's rows, as before rows existed");
        drop_scratch(&data);
    }
    let data = scratch("absent");
    catalog(&data, json!([]));
    let mut e = Catalog::from_value(&source::load(&data).catalog).encounters["walled"].clone();
    e.shift_remove("rows");
    assert!(EncounterDef::from_obj(&e).rows.is_empty(), "no rows key: none");
    drop_scratch(&data);
}

/// The `rows` problems of `walled` with `rows` set to `value`.
fn row_errors(v: &check::View, value: Value) -> Vec<String> {
    let mut e = v.get("encounters", "walled").clone();
    e.insert("rows".into(), value);
    check::validate_entry(v, "encounters", &e, "walled")
}

#[test]
fn each_bad_rows_value_is_one_error() {
    let data = scratch("check");
    catalog(&data, json!(authored()));
    let v = store::view(&data);
    let text = format!("Rows are {GRID_H} strings of {GRID_W} terrain glyphs.");
    assert_eq!(row_errors(&v, json!(authored())), Vec::<String>::new(), "valid rows");
    assert_eq!(row_errors(&v, json!([])), Vec::<String>::new(), "no rows");
    let mut eight = authored();
    eight.pop();
    let mut short = authored();
    short[3].pop();
    let mut unknown = authored();
    unknown[4] = format!("X{}", &unknown[4][1..]);
    let mut number = json!(authored());
    number[2] = json!(7);
    for (tag, bad) in [("8 rows", json!(eight)), ("11 glyphs", json!(short)), ("unknown glyph", json!(unknown)), ("a number", number), ("an object", json!({"rows": 9}))] {
        assert_eq!(row_errors(&v, bad), vec![text.clone()], "{tag}");
    }
    drop_scratch(&data);
}

#[test]
fn a_world_encounter_checks_its_rows() {
    let data = scratch("world");
    catalog(&data, json!([]));
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    let mut eight = authored();
    eight.pop();
    let entries = json!([{"id": "good", "name": "Good", "rows": authored()}, {"id": "bad", "name": "Bad", "rows": eight}]);
    fs::write(dir.join("encounters.json"), json!({"kind": "encounters", "version": 1, "entries": entries}).to_string()).unwrap();
    let rows: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains("Rows are")).collect();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(rows[0].contains("bad"), "{rows:?}");
    drop_scratch(&data);
}

#[test]
fn a_hold_tile_is_checked_on_the_authored_board() {
    let data = scratch("hold");
    catalog(&data, json!(authored()));
    let v = store::view(&data);
    // (0, 2) is burning ground in the rows (walkable) and a wall in the zone; (11, 3) is a wall in the rows
    let hold = |rows: Value, tile: [i32; 2]| {
        let mut e = v.get("encounters", "walled").clone();
        e.insert("rows".into(), rows);
        e.insert("objective".into(), json!("hold"));
        e.insert("objective_tiles".into(), json!([tile]));
        check::validate_entry(&v, "encounters", &e, "walled")
    };
    let zone = zone_rows(ZONE).unwrap();
    let at = |rows: &[&str], x: usize, y: usize| rows[y].chars().nth(x).unwrap();
    let mine: Vec<String> = authored();
    let mine: Vec<&str> = mine.iter().map(String::as_str).collect();
    assert!(grid::terrain_walkable(terrain_of(at(&mine, 0, 2))) && !grid::terrain_walkable(terrain_of(at(&zone, 0, 2))), "fixture");
    assert!(!grid::terrain_walkable(terrain_of(at(&mine, 11, 3))), "fixture");
    assert_eq!(hold(json!(authored()), [0, 2]), Vec::<String>::new(), "walkable on the rows");
    assert_eq!(hold(json!([]), [0, 2]), vec!["Objective tile (0, 2) is off the grid, blocked or an exit.".to_string()], "blocked on the zone");
    assert_eq!(hold(json!(authored()), [11, 3]), vec!["Objective tile (11, 3) is off the grid, blocked or an exit.".to_string()], "blocked on the rows");
    drop_scratch(&data);
}

#[test]
fn the_writer_stores_and_reloads_rows() {
    let data = scratch("writer");
    catalog(&data, json!([]));
    let mut e = dict(json!({"id": "walled", "name": "Walled", "zone": ZONE, "party": ["hero"], "enemies": ["brute"]}));
    e.insert("rows".into(), json!(authored()));
    store::upsert(&data, None, "encounters", &e, None).expect("valid rows");
    let cat = Catalog::from_value(&source::load(&data).catalog);
    assert_eq!(EncounterDef::from_obj(&cat.encounters["walled"]).rows, authored());
    let text = fs::read_to_string(store::file_of(&data, None, "encounters")).unwrap();
    assert_eq!(store::resave_text(&data, None, "encounters").unwrap(), text, "a re-save without changes is byte-identical");
    let mut eight = authored();
    eight.pop();
    e.insert("rows".into(), json!(eight));
    let errors = store::upsert(&data, None, "encounters", &e, None).err().expect("bad rows refused");
    assert!(errors.iter().any(|x| x.contains("Rows are")), "{errors:?}");
    assert_eq!(fs::read_to_string(store::file_of(&data, None, "encounters")).unwrap(), text, "rolled back");
    drop_scratch(&data);
}
