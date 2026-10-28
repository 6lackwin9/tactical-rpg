//! Zones as data (HOOKS1-Z, docs/specs/HOOKS1.md section 13, ZN1-ZN3): the encounter `zone` options are the 8 code
//! zones, then the loaded `zones` rows (catalog and world encounters); a row reusing a code id is one load error and
//! the code zone keeps its code rows; the board is the encounter's `rows`, else the code zone's, else the data zone
//! row's (empty or unknown: floor); a hold tile is checked on that board; the backdrop key is the row's `backdrop`,
//! else `backgrounds/battle/<id>.png`. A catalog without a `zones` file plays as before. Scratch data folders only.

use aegis_core::json::{self, Dict};
use indexmap::IndexMap;
use rpgkit_battle::grid::{zone_rows, Grid, GLYPHS, GRID_H, GRID_W, TERRAIN_IDS, V2, ZONE_IDS};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_content::art::{battle_backdrop, zone_backdrop};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, UnitDef, ZONES_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A scratch data folder `<tmp>/<tag>/data` with empty catalog and worlds folders.
fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_zones_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

/// Writes a catalog file as a fixture (the files are the test's own input).
fn write_kind(data: &Path, kind: &str, entries: Value) {
    let version = if kind == "skills" { 2 } else { 1 };
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn strs(rows: &[&str]) -> Vec<String> {
    rows.iter().map(|r| r.to_string()).collect()
}

/// The data zone's own layout: a wall ring with floor inside, the default exit (11, 4) open; (0, 2) is a wall.
fn zone_layout() -> Vec<String> {
    strs(&["############", "#..........#", "#..........#", "#..........#", "............", "#..........#", "#..........#", "#..........#", "############"])
}

/// Authored encounter rows, different from the zone's: grass everywhere but a tree at (5, 0).
fn authored() -> Vec<String> {
    let mut rows = vec![",,,,,,,,,,,,".to_string(); GRID_H as usize];
    rows[0] = ",,,,,T,,,,,,".into();
    rows
}

/// A class, the system skills and a basic attack, one hero and one enemy, the data zones `alley` (its layout) and
/// `yard` (no rows), and the encounter `fight` in `zone` with `rows`.
fn catalog(data: &Path, zone: &str, rows: Value) {
    let skill = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(data, "skills", json!([skill("move", "MOVE", json!({"system": true})), skill("rest", "REST", json!({"system": true})), skill("strike", "ATTACK", json!({"basic": true}))]));
    write_kind(data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2, "CONS": 2, "DEX": 1}}]));
    write_kind(data, "units", json!([{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}, {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}]));
    write_kind(data, ZONES_KIND, json!([{"id": "alley", "name": "Cinder Alley", "rows": zone_layout(), "ground": "stone"}, {"id": "yard", "name": "Yard", "ground": "dirt", "backdrop": "backgrounds/battle/yard_dusk.png"}]));
    encounter(data, zone, rows);
}

fn encounter(data: &Path, zone: &str, rows: Value) {
    write_kind(data, "encounters", json!([{"id": "fight", "name": "Fight", "zone": zone, "party": ["hero"], "enemies": ["brute"], "rows": rows}]));
}

/// The battle of `fight` on the fixture catalog.
fn battle(data: &Path) -> Session {
    let cat = Catalog::from_value(&source::load(data).catalog);
    let def = EncounterDef::from_obj(&cat.encounters["fight"]);
    Session::start(Arc::new(cat), def, &IndexMap::<String, UnitDef>::new())
}

/// The terrain id of glyph `ch` (GLYPHS and TERRAIN_IDS share their order).
fn terrain_of(ch: char) -> &'static str {
    TERRAIN_IDS[GLYPHS.iter().position(|g| *g == ch).expect("a known glyph")]
}

fn board_of<'a>(rows: impl IntoIterator<Item = &'a str>) -> Vec<&'static str> {
    rows.into_iter().flat_map(|r| r.chars().map(terrain_of).collect::<Vec<_>>()).collect()
}

fn board(g: &Grid) -> Vec<&'static str> {
    (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).map(|c| g.terrain_id(c)).collect()
}

fn floor() -> Vec<&'static str> {
    vec![TERRAIN_IDS[0]; (GRID_W * GRID_H) as usize]
}

/// The encounter `zone` field of a schema section.
fn zone_field(section: &str) -> Dict {
    aegis_content::schema::fields(source::schema(), section, "encounters").iter().map(json::dict).find(|f| json::s(f, "key") == "zone").expect("encounters.zone").clone()
}

#[test]
fn the_zone_options_are_the_code_zones_then_the_rows() {
    let data = scratch("options");
    catalog(&data, "alley", json!([]));
    let v = store::view(&data);
    let mut want = strs(&ZONE_IDS);
    want.extend(strs(&["alley", "yard"]));
    for section in ["content", "world"] {
        assert_eq!(check::options_of(&v, &zone_field(section)), want, "{section}");
    }
    let fight = v.get("encounters", "fight").clone();
    assert_eq!(check::validate_entry(&v, "encounters", &fight, "fight"), Vec::<String>::new(), "a data zone is a zone");
    let mut lost = fight.clone();
    lost.insert("zone".into(), json!("nowhere"));
    let errors = check::validate_entry(&v, "encounters", &lost, "fight");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].starts_with("Zone: 'nowhere' is not one of stone_arena,") && errors[0].ends_with(", alley, yard."), "{errors:?}");
    drop_scratch(&data);
}

#[test]
fn without_zone_rows_the_options_are_the_code_zones() {
    let data = scratch("classic");
    catalog(&data, "forest_clearing", json!([]));
    fs::remove_file(data.join("catalog").join(format!("{ZONES_KIND}.json"))).unwrap();
    let v = store::view(&data);
    assert_eq!(check::options_of(&v, &zone_field("content")), strs(&ZONE_IDS));
    assert_eq!(board(&battle(&data).w.grid), board_of(zone_rows("forest_clearing").unwrap()), "a code zone's board as before");
    drop_scratch(&data);
}

#[test]
fn a_world_encounter_may_name_a_data_zone() {
    let data = scratch("world");
    catalog(&data, "alley", json!([]));
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    let entries = json!([{"id": "good", "name": "Good", "zone": "yard"}, {"id": "bad", "name": "Bad", "zone": "nowhere"}]);
    fs::write(dir.join("encounters.json"), json!({"kind": "encounters", "version": 1, "entries": entries}).to_string()).unwrap();
    let zones: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains("Zone: '")).collect();
    assert_eq!(zones.len(), 1, "{zones:?}");
    assert!(zones[0].contains("bad") && zones[0].contains("'nowhere'"), "{zones:?}");
    drop_scratch(&data);
}

#[test]
fn a_road_fight_may_name_a_data_zone() {
    let data = scratch("route");
    catalog(&data, "alley", json!([]));
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    let file = |kind: &str, entries: Value| fs::write(dir.join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
    file("locations", json!([{"id": "town", "name": "Town"}, {"id": "mill", "name": "Mill"}]));
    let route = |id: &str, zone: &str| json!({"id": id, "name": id, "from": "town", "to": "mill", "time": 2, "zone": zone});
    file("routes", json!([route("good", "yard"), route("code", "swamp_bog"), route("bad", "nowhere")]));
    let zones: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains("Unknown zone")).collect();
    assert_eq!(zones.len(), 1, "{zones:?}");
    assert!(zones[0].contains("bad") && zones[0].contains("Unknown zone 'nowhere'."), "{zones:?}");
    drop_scratch(&data);
}

#[test]
fn a_data_zone_without_encounter_rows_plays_on_its_own_rows() {
    let data = scratch("zone_rows");
    catalog(&data, "alley", json!([]));
    let s = battle(&data);
    assert_eq!(board(&s.w.grid), board_of(zone_layout().iter().map(String::as_str)), "the zone row's layout");
    for u in 0..s.w.units.len() {
        let c = s.w.grid.pos(u).expect("deployed");
        assert!(s.w.grid.is_walkable(c), "unit {u} on {c}");
    }
    drop_scratch(&data);
}

#[test]
fn encounter_rows_win_over_the_data_zone() {
    let data = scratch("enc_rows");
    catalog(&data, "alley", json!(authored()));
    let want = board_of(authored().iter().map(String::as_str));
    assert_ne!(want, board_of(zone_layout().iter().map(String::as_str)), "fixture");
    assert_eq!(board(&battle(&data).w.grid), want, "the encounter's rows");
    drop_scratch(&data);
}

#[test]
fn a_data_zone_without_rows_or_an_unknown_zone_is_floor() {
    for (tag, zone) in [("empty", "yard"), ("unknown", "nowhere")] {
        let data = scratch(tag);
        catalog(&data, zone, json!([]));
        assert_eq!(board(&battle(&data).w.grid), floor(), "{tag}: 9 rows of floor");
        drop_scratch(&data);
    }
}

#[test]
fn a_row_named_like_a_code_zone_is_one_error_and_the_code_zone_stays() {
    let data = scratch("code_id");
    catalog(&data, "stone_arena", json!([]));
    write_kind(&data, ZONES_KIND, json!([{"id": "alley", "name": "Cinder Alley", "rows": zone_layout()}, {"id": "stone_arena", "name": "Walled Arena", "rows": zone_layout()}]));
    let src = source::load(&data);
    let errors: Vec<&String> = src.errors.iter().filter(|e| e.contains("catalog")).collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("zone 'stone_arena': ID 'stone_arena' is a code zone"), "{errors:?}");
    assert_eq!(board(&battle(&data).w.grid), board_of(zone_rows("stone_arena").unwrap()), "the code zone's rows, not the row's");
    drop_scratch(&data);
}

/// The same bad row reaching a view or a catalog unchecked (the loader drops it; a value built by hand keeps it): the
/// options list the code id once, and the code zone ignores the row for its board and backdrop key.
#[test]
fn an_unchecked_code_id_row_never_stands_in_for_the_code_zone() {
    let rows = json!([{"id": "alley", "name": "Cinder Alley"}, {"id": "stone_arena", "name": "Walled Arena", "rows": zone_layout(), "backdrop": "backgrounds/battle/walled.png"}]);
    let mut catalog = json!({});
    catalog[ZONES_KIND] = rows;
    let v = check::View::new(&catalog, &json!({}), &json!({}), aegis_core::assets::Layers::default());
    let mut want = strs(&ZONE_IDS);
    want.push("alley".into());
    assert_eq!(check::zone_ids(&v), want, "each id once");
    assert_eq!(check::options_of(&v, &zone_field("content")), want);
    let cat = Catalog::from_value(&catalog);
    assert!(cat.zones.contains_key("stone_arena"), "fixture: the row is in the catalog");
    assert_eq!(rpgkit_battle::grid::data_zone("stone_arena", &cat.zones), None);
    assert_eq!(rpgkit_battle::grid::data_zone("alley", &cat.zones).map(|z| z.id.as_str()), Some("alley"));
    assert_eq!(zone_backdrop("stone_arena", rpgkit_battle::grid::data_zone("stone_arena", &cat.zones)), battle_backdrop("stone_arena"), "the code zone's key");
    let mut g = Grid::new(false, Vec::new(), rpgkit_battle::grid::TerrainTable::from_catalog(&cat), 0);
    g.apply_board("stone_arena", &[], &cat.zones);
    assert_eq!(board(&g), board_of(zone_rows("stone_arena").unwrap()), "the code zone's rows");
}

#[test]
fn a_hold_tile_is_checked_on_the_data_zone_board() {
    let data = scratch("hold");
    catalog(&data, "alley", json!([]));
    let v = store::view(&data);
    let hold = |zone: &str, tile: [i32; 2]| {
        let mut e = v.get("encounters", "fight").clone();
        e.insert("zone".into(), json!(zone));
        e.insert("objective".into(), json!("hold"));
        e.insert("objective_tiles".into(), json!([tile]));
        check::validate_entry(&v, "encounters", &e, "fight")
    };
    let wall = |rows: &[String], x: usize, y: usize| !rpgkit_battle::grid::terrain_walkable(terrain_of(rows[y].chars().nth(x).unwrap()));
    assert!(wall(&zone_layout(), 0, 2) && !wall(&zone_layout(), 3, 2), "fixture");
    assert_eq!(hold("alley", [3, 2]), Vec::<String>::new(), "floor in the zone");
    assert_eq!(hold("alley", [0, 2]), vec!["Objective tile (0, 2) is off the grid, blocked or an exit.".to_string()], "a wall in the zone");
    assert_eq!(hold("yard", [0, 2]), Vec::<String>::new(), "a zone without rows is floor");
    drop_scratch(&data);
}

#[test]
fn the_backdrop_key_is_the_row_s_else_the_zone_path() {
    let data = scratch("backdrop");
    catalog(&data, "alley", json!([]));
    let cat = Catalog::from_value(&source::load(&data).catalog);
    assert_eq!(zone_backdrop("alley", cat.zones.get("alley")), battle_backdrop("alley"), "no backdrop: backgrounds/battle/<id>.png");
    assert_eq!(battle_backdrop("alley"), "backgrounds/battle/alley.png");
    assert_eq!(zone_backdrop("yard", cat.zones.get("yard")), "backgrounds/battle/yard_dusk.png", "the row's key");
    assert_eq!(zone_backdrop("forest_clearing", cat.zones.get("forest_clearing")), battle_backdrop("forest_clearing"), "a code zone has no row");
    assert_eq!((cat.zones["alley"].ground.as_str(), cat.zones["yard"].ground.as_str()), ("stone", "dirt"), "the gradient's ground");
    drop_scratch(&data);
}
