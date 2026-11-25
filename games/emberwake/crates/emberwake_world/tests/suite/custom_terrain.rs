//! LAYER-DOWN-2 (PR5-30): a new terrain type is data only. On a scratch copy of the shipped data, a test-only terrain
//! row (a new id with its glyph) is written through the content writer and the new rows add no problem to the folder check; a glyph is
//! required, unique and the built-in types keep theirs; an encounter's rows may use the new glyph (an unknown glyph is
//! still refused); then the battle board built from the catalog places the type and reads its row (move cost, cover,
//! walkable), with every other tile's terrain as before.

use aegis_core::json::{self, Dict};
use emberwake_battle::grid::{Grid, TerrainTable, GRID_H, GRID_W, V2};
use emberwake_content::{check, store};
use emberwake_world::Data;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const TERRAIN: &str = "t_quicksand";
const GLYPH: char = 'q';
const MOVE_COST: i64 = 3;
const COVER: f64 = 0.1;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_owned();
        if p.is_dir() {
            copy_dir(&p, &to.join(name));
        } else {
            std::fs::copy(&p, to.join(name)).unwrap();
        }
    }
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().expect("an object")
}

/// The test board: floor with the new glyph in the middle row's fourth tile.
fn rows() -> Vec<String> {
    (0..GRID_H).map(|y| (0..GRID_W).map(|x| if (x, y) == (3, 4) { GLYPH } else { '.' }).collect()).collect()
}

#[test]
fn a_new_terrain_type_validates_and_plays_without_kit_code() {
    let t = aegis_core::scratch::Scratch::vacant("emberwake_custom_terrain");
    let dir = t.join("data");
    copy_dir(&data_dir(), &dir);
    // the copy lies outside the game folder (no asset layers): the rows must add no problem to what it reports
    let before = store::check(&dir);
    let row = dict(json!({"id": TERRAIN, "glyph": GLYPH.to_string(), "walkable": true, "move_cost": MOVE_COST, "cover_guard": COVER}));
    // the glyph rules (checked before anything is written)
    let v = store::view(&dir);
    let floor_glyph = v.entries("terrain").iter().find(|r| json::s(r, "id") == "floor").map(|_| '.').expect("a floor row");
    let mut bare = row.clone();
    bare.shift_remove("glyph");
    assert!(check::validate_entry(&v, "terrain", &bare, "").iter().any(|m| m.contains("needs a one-character glyph")), "a new type needs a glyph");
    let mut taken = row.clone();
    taken.insert("glyph".into(), json!(floor_glyph.to_string()));
    assert!(check::validate_entry(&v, "terrain", &taken, "").iter().any(|m| m.starts_with("Glyph '.' is taken by terrain type 'floor'")), "a glyph is unique");
    let mut floor = v.entries("terrain").iter().find(|r| json::s(r, "id") == "floor").cloned().unwrap();
    floor.insert("glyph".into(), json!("F"));
    assert!(check::validate_entry(&v, "terrain", &floor, "floor").iter().any(|m| m.starts_with("Built-in terrain type 'floor' keeps its glyph")), "a built-in type keeps its glyph");
    assert!(check::validate_entry(&v, "terrain", &row, "").is_empty());
    if let Err(e) = store::upsert(&dir, None, "terrain", &row, None) {
        panic!("terrain: {e:?}");
    }
    let problems = store::check(&dir);
    assert_eq!(problems, before, "the new rows add no problem");
    // an encounter's rows may use the new glyph, not an unknown one
    let v = store::view(&dir);
    let enc = v.entries("encounters").first().cloned().expect("a catalog encounter");
    let id = json::s(&enc, "id");
    let with_rows = |rows: Vec<String>| {
        let mut e = enc.clone();
        e.insert("rows".into(), json!(rows));
        check::validate_entry(&v, "encounters", &e, &id)
    };
    assert!(!with_rows(rows()).iter().any(|m| m.starts_with("Rows are")), "the new glyph is a terrain glyph");
    let unknown: Vec<String> = rows().iter().map(|r| r.replace(GLYPH, "\u{2603}")).collect();
    assert!(with_rows(unknown).iter().any(|m| m.starts_with("Rows are")), "an unknown glyph is refused");
    // the battle board places it and reads its row
    let (d, errors) = Data::open(&dir).expect("the scratch data");
    assert!(errors.is_empty(), "{errors:?}");
    let table = TerrainTable::from_catalog(&d.cat);
    let shipped = TerrainTable::from_catalog(&Data::open(&data_dir()).expect("the shipped data").0.cat);
    assert_eq!(table.ids[..shipped.ids.len()], shipped.ids[..], "the shipped types keep their order (and indices)");
    assert_eq!(table.ids.last().map(String::as_str), Some(TERRAIN), "the new type follows them");
    let mut g = Grid::new(false, Vec::new(), table, 0);
    g.apply_board("stone_arena", &rows(), &d.cat.zones);
    let at = V2::new(3, 4);
    assert_eq!((g.terrain_id(at), g.glyph(at)), (TERRAIN, GLYPH));
    assert_eq!(g.move_cost(at), MOVE_COST as i32);
    assert!(g.is_walkable(at) && !g.stops_forced_move(at), "walkable as its row says");
    assert!((g.cover_guard(at) - COVER).abs() < 1e-9);
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            if V2::new(x, y) != at {
                assert_eq!(g.terrain_id(V2::new(x, y)), "floor");
            }
        }
    }
    let _ = std::fs::remove_dir_all(&t);
}
