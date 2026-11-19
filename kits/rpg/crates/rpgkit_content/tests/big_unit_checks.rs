//! BIGUNIT1 BU1-BU3 (docs/specs/BIGUNIT1.md section 5): the units schema carries `footprint` (int 1..2, default 1,
//! omitted at the default); an encounter naming a 2 x 2 unit is an error when the unit finds no square to deploy on whose
//! four cells are inside the arena, walkable, not an exit and clear of the other units; nothing for an encounter without
//! one, or when a square fits. Catalog and world encounters (a character plays as its base unit). Scratch data only.

use aegis_content::schema::fields;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_big_unit_checks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// System skills, a class, a hero, a one-cell brute and the 2 x 2 giant; the encounter `den` (hero against `enemies`).
fn catalog(data: &Path, enemies: Value) {
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
        json!([
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1},
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1},
            {"id": "giant", "name": "Giant", "side": "enemy", "class_id": "drill", "level": 1, "footprint": 2}
        ]),
    );
    write_kind(data, "encounters", 1, json!([{"id": "den", "name": "Den", "party": ["hero"], "enemies": enemies, "rows": open()}]));
}

/// Open floor; the default exit (11, 4).
fn open() -> Vec<String> {
    vec![".".repeat(12); 9]
}

/// Walls everywhere except the floor cells `floor`.
fn walled(floor: &[(usize, usize)]) -> Vec<String> {
    let mut rows: Vec<Vec<char>> = vec![vec!['#'; 12]; 9];
    for &(x, y) in floor {
        rows[y][x] = '.';
    }
    rows.into_iter().map(|r| r.into_iter().collect()).collect()
}

fn square(x: usize, y: usize) -> Vec<(usize, usize)> {
    vec![(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)]
}

const NO_SQUARE: &str = "fills 2 x 2 cells and finds no free 2 x 2 square to deploy on";

/// The footprint problems of `den` with `rows` and `enemies`.
fn errors(v: &check::View, rows: Vec<String>, enemies: Value) -> Vec<String> {
    let mut e = v.get("encounters", "den").clone();
    e.insert("rows".into(), json!(rows));
    e.insert("enemies".into(), enemies);
    check::validate_entry(v, "encounters", &e, "den").into_iter().filter(|p| p.contains(NO_SQUARE)).collect()
}

#[test]
fn the_schema_carries_footprint() {
    let schema: Value = serde_json::from_str(&fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schema.json")).unwrap()).unwrap();
    let f = fields(&schema, "content", "units").iter().find(|f| f["key"] == "footprint").expect("units.footprint");
    assert_eq!((&f["type"], &f["default"], &f["min"], &f["max"], &f["omit_default"]), (&json!("int"), &json!(1), &json!(1), &json!(2), &json!(true)));
    assert!(f["help"].as_str().is_some_and(|h| !h.is_empty()), "help text");
}

#[test]
fn a_big_unit_needs_a_free_square() {
    let (_t, data) = scratch("catalog");
    catalog(&data, json!(["giant"]));
    let v = store::view(&data);
    let one = |r: &str| vec![format!("'{r}' {NO_SQUARE} (all four cells inside the arena, walkable, not an exit, clear of other units).")];
    // a valid board: open floor; and no big unit: nothing to check
    assert_eq!(errors(&v, open(), json!(["giant", "brute"])), Vec::<String>::new(), "open floor");
    let corridor: Vec<(usize, usize)> = (0..12).map(|x| (x, 4)).collect();
    assert_eq!(errors(&v, walled(&corridor), json!(["brute"])), Vec::<String>::new(), "no 2 x 2 unit");
    // only a one-tile corridor: no square anywhere
    assert_eq!(errors(&v, walled(&corridor), json!(["giant"])), one("giant"), "no 2 x 2 walkable space");
    // the only square holds the exit (11, 4); a corridor on row 7 for the hero
    let mut exit: Vec<(usize, usize)> = square(10, 3);
    exit.extend((0..12).map(|x| (x, 7)));
    assert_eq!(errors(&v, walled(&exit), json!(["giant"])), one("giant"), "the only square holds an exit");
    // the same square one column in: it fits
    let mut inner: Vec<(usize, usize)> = square(9, 3);
    inner.extend((0..12).map(|x| (x, 7)));
    assert_eq!(errors(&v, walled(&inner), json!(["giant"])), Vec::<String>::new(), "a square clear of the exit");
    // the only floor is one square: the hero and the brute stand on it first
    assert_eq!(errors(&v, walled(&square(4, 3)), json!(["brute", "giant"])), one("giant"), "the only square is occupied");
    drop(_t);
}

#[test]
fn a_world_encounter_checks_its_big_characters() {
    let (_t, data) = scratch("world");
    catalog(&data, json!(["brute"]));
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    fs::write(dir.join("characters.json"), json!({"kind": "characters", "version": 1, "entries": [{"id": "hulk", "name": "Hulk", "base_unit": "giant"}]}).to_string()).unwrap();
    let corridor: Vec<(usize, usize)> = (0..12).map(|x| (x, 4)).collect();
    let entries = json!([
        {"id": "roomy", "name": "Roomy", "party": ["hero"], "enemies": ["hulk"], "rows": open()},
        {"id": "narrow", "name": "Narrow", "party": ["hero"], "enemies": ["hulk"], "rows": walled(&corridor)}
    ]);
    fs::write(dir.join("encounters.json"), json!({"kind": "encounters", "version": 1, "entries": entries}).to_string()).unwrap();
    let found: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains(NO_SQUARE)).collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("narrow") && found[0].contains("'hulk'"), "{found:?}");
    drop(_t);
}
