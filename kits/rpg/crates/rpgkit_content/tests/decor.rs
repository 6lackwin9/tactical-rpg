//! Display-only encounter decor (DECOR1, PR7-81): the schema carries `decor` (json, default [], stored as no key while
//! empty) in both encounter sections; the writer stores and reloads it and a re-save is byte-identical; the typed reader
//! (`encounter::decor_of`, `EncounterDef::decor`) reads it; the battle board (terrain, walkability, deploy) is the same
//! with or without it; the checks refuse decor off the grid, on an exit, a deploy cell, a hold's objective tile or an
//! escape's stay-behind tile, two on one cell, and an art key that is not a decor key or names no art (a cover key in
//! the board's stage set: the zone's set, else the world's; none for a catalog encounter), in catalog and world
//! encounters. Scratch data folders only.

use aegis_content::schema::{fields, normalize};
use aegis_core::json::{self, Dict};
use indexmap::IndexMap;
use rpgkit_battle::encounter::{decor_of, Decor};
use rpgkit_battle::grid::{Grid, GRID_H, GRID_W, V2};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_content::art::{decor_art, decor_cover_sets, decor_known, DecorArt};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, UnitDef};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A scratch game folder `<tmp>/<tag>` with `data/` (empty catalog and worlds) and the fixture art under `assets/`:
/// a stage set's cover v0..v3, a zone set `zz_alley` (cover v0 only) that the stage set's `stage.fx.json` names for zone
/// `alley`, and a prop `zz_lamp` (one variant) in a region prop set.
fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_decor_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    fs::create_dir_all(t.join("assets/tilesets/zz_stage")).unwrap();
    fs::write(t.join("assets/tilesets/zz_stage/stage.fx.json"), json!({"zones": {"alley": {"set": "zz_alley"}}}).to_string()).unwrap();
    for rel in ["tilesets/zz_stage/cover/v0.png", "tilesets/zz_stage/cover/v1.png", "tilesets/zz_stage/cover/v2.png", "tilesets/zz_stage/cover/v3.png", "tilesets/zz_alley/cover/v0.png", "tilesets/zz_other/cover/v5.png", "props/zz_ward/zz_lamp/v0.png"] {
        let path = t.join("assets").join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"png").unwrap();
    }
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// Floor everywhere but a wall at (3, 1) (an inner block, so it shows a cover prop) and the outer bottom row.
fn rows() -> Vec<String> {
    let mut r: Vec<String> = (0..GRID_H).map(|_| ".".repeat(GRID_W as usize)).collect();
    r[1] = "...#........".into();
    r[(GRID_H - 1) as usize] = "#".repeat(GRID_W as usize);
    r
}

const WALL: V2 = V2 { x: 3, y: 1 };

/// A class, the system skills and a basic attack, one hero and one enemy, and the encounter `lit` on `rows()` with
/// `extra` merged in.
fn catalog(data: &Path, extra: Value) {
    let skill = |id: &str, kind: &str, more: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in more.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(data, "skills", 2, json!([skill("move", "MOVE", json!({"system": true})), skill("rest", "REST", json!({"system": true})), skill("strike", "ATTACK", json!({"basic": true}))]));
    write_kind(data, "classes", 1, json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2, "CONS": 2, "DEX": 1}}]));
    write_kind(data, "units", 1, json!([{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}, {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}]));
    let mut e = json!({"id": "lit", "name": "Lit", "party": ["hero"], "enemies": ["brute"], "rows": rows()});
    for (k, v) in extra.as_object().unwrap() {
        e[k] = v.clone();
    }
    write_kind(data, "encounters", 1, json!([e]));
}

fn battle(data: &Path) -> Session {
    let cat = Catalog::from_value(&source::load(data).catalog);
    let def = EncounterDef::from_obj(&cat.encounters["lit"]);
    Session::start(Arc::new(cat), def, &IndexMap::<String, UnitDef>::new())
}

/// A free floor cell: walkable, not an exit, no unit on it at the start of `s`.
fn free_cell(s: &Session) -> V2 {
    let taken: Vec<V2> = (0..s.w.units.len()).flat_map(|u| s.w.cells_of(u)).collect();
    (1..GRID_H - 1).flat_map(|y| (1..GRID_W - 1).map(move |x| V2::new(x, y))).find(|c| s.w.grid.is_walkable(*c) && !s.w.grid.is_exit(*c) && !taken.contains(c)).expect("a free cell")
}

#[test]
fn the_schema_carries_decor_in_both_encounter_sections() {
    for section in ["content", "world"] {
        let f = fields(source::schema(), section, "encounters").iter().map(json::dict).find(|f| json::s(f, "key") == "decor").unwrap_or_else(|| panic!("{section}.encounters.decor"));
        assert_eq!((json::s(f, "type"), f.get("default"), f.get("omit_default")), ("json".to_string(), Some(&json!([])), Some(&json!(true))), "{section}");
    }
    let e = normalize(fields(source::schema(), "content", "encounters"), &dict(json!({"id": "x"})), false);
    assert!(EncounterDef::from_obj(&e).decor.is_empty(), "no decor: none");
}

#[test]
fn the_decor_key_forms() {
    assert_eq!(decor_art("cover"), Some(DecorArt::Cover(None)));
    assert_eq!(decor_art("cover/v3"), Some(DecorArt::Cover(Some(3))));
    assert_eq!(decor_art("lantern_pole"), Some(DecorArt::Prop("lantern_pole".into(), None)));
    assert_eq!(decor_art("crate_stack/v1"), Some(DecorArt::Prop("crate_stack".into(), Some(1))));
    for bad in ["", "Cover", "cover/3", "cover/v", "cover/vx", "a", "props/common/tree", "tree/v1/v2", "tree/v1234"] {
        assert_eq!(decor_art(bad), None, "{bad:?}");
    }
    let (_s, data) = scratch("keys");
    let layers = aegis_core::assets::Layers::of(vec![data.parent().unwrap().join("assets")]);
    let stage = vec!["zz_stage".to_string()];
    for (key, known) in [("cover", true), ("cover/v3", true), ("cover/v4", false), ("cover/v5", false), ("zz_lamp", true), ("zz_lamp/v0", true), ("zz_lamp/v1", false), ("zz_none", false)] {
        assert_eq!(decor_known(&layers, key, &stage), known, "{key}");
    }
    assert!(!decor_known(&layers, "cover", &[]) && decor_known(&layers, "zz_lamp", &[]), "no stage set: no cover, props still");
    assert_eq!(decor_cover_sets(&layers, "", "alley"), Vec::<String>::new(), "a catalog encounter");
    assert_eq!(decor_cover_sets(&layers, "zz_stage", "yard"), ["zz_stage"]);
    let alley = decor_cover_sets(&layers, "zz_stage", "alley");
    assert_eq!(alley, ["zz_alley", "zz_stage"], "the zone's set first");
    assert!(decor_known(&layers, "cover/v0", &alley) && !decor_known(&layers, "cover/v3", &alley), "the zone set's cover folder is the one read");
}

#[test]
fn the_writer_stores_and_reloads_decor_and_leaves_it_out_when_empty() {
    let (_s, data) = scratch("writer");
    catalog(&data, json!({}));
    let base = dict(json!({"id": "lit", "name": "Lit", "party": ["hero"], "enemies": ["brute"], "rows": rows()}));
    store::upsert(&data, None, "encounters", &base, None).expect("no decor");
    let plain = fs::read_to_string(store::file_of(&data, None, "encounters")).unwrap();
    assert!(!plain.contains("\"decor\""), "empty decor is stored as no key");
    let s = battle(&data);
    let spot = free_cell(&s);
    let mut e = base.clone();
    e.insert("decor".into(), json!([{"art": "zz_lamp/v0", "x": WALL.x, "y": WALL.y}, {"art": "zz_lamp", "x": spot.x, "y": spot.y}]));
    store::upsert(&data, None, "encounters", &e, None).expect("valid decor");
    let cat = Catalog::from_value(&source::load(&data).catalog);
    let want = vec![Decor { art: "zz_lamp/v0".into(), at: WALL }, Decor { art: "zz_lamp".into(), at: spot }];
    assert_eq!(decor_of(&cat.encounters["lit"]), want);
    assert_eq!(EncounterDef::from_obj(&cat.encounters["lit"]).decor, want);
    let text = fs::read_to_string(store::file_of(&data, None, "encounters")).unwrap();
    assert_eq!(store::resave_text(&data, None, "encounters").unwrap(), text, "a re-save without changes is byte-identical");
    e.insert("decor".into(), json!([]));
    store::upsert(&data, None, "encounters", &e, None).expect("decor emptied");
    assert_eq!(fs::read_to_string(store::file_of(&data, None, "encounters")).unwrap(), plain, "emptied: the file as before");
}

#[test]
fn decor_leaves_the_board_and_the_deploy_unchanged() {
    let (_s, data) = scratch("board");
    catalog(&data, json!({}));
    let before = battle(&data);
    let spot = free_cell(&before);
    catalog(&data, json!({"decor": [{"art": "cover/v3", "x": WALL.x, "y": WALL.y}, {"art": "zz_lamp", "x": spot.x, "y": spot.y}]}));
    let after = battle(&data);
    assert_eq!(after.def.decor.len(), 2);
    let cells = |g: &Grid| (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).map(|c| (g.terrain_id(c).to_string(), g.is_walkable(c), g.is_exit(c))).collect::<Vec<_>>();
    assert_eq!(cells(&after.w.grid), cells(&before.w.grid), "terrain, walkability and exits");
    let at = |s: &Session| (0..s.w.units.len()).map(|u| s.w.grid.pos(u)).collect::<Vec<_>>();
    assert_eq!(at(&after), at(&before), "the units deploy where they did");
    assert!(after.w.grid.is_walkable(spot) && !after.w.grid.is_walkable(WALL));
}

/// The problems of `lit` (catalog) with `decor` set to `decor` and `extra` merged in, the decor ones only.
fn decor_errors(v: &check::View, decor: Value, extra: Value) -> Vec<String> {
    let mut e = v.get("encounters", "lit").clone();
    e.insert("decor".into(), decor);
    for (k, x) in extra.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, "encounters", &e, "lit").into_iter().filter(|p| p.starts_with("Decor")).collect()
}

#[test]
fn the_checks_refuse_bad_decor() {
    let (_s, data) = scratch("check");
    catalog(&data, json!({}));
    let v = store::view(&data);
    let s = battle(&data);
    let spot = free_cell(&s);
    let hero = s.w.grid.pos(0).expect("the hero deployed");
    let foe = s.w.grid.pos(1).expect("the enemy deployed");
    let one = |art: &str, c: [i32; 2]| json!([{"art": art, "x": c[0], "y": c[1]}]);
    assert_eq!(decor_errors(&v, json!([]), json!({})), Vec::<String>::new(), "none");
    assert_eq!(decor_errors(&v, json!([{"art": "zz_lamp", "x": WALL.x, "y": WALL.y}, {"art": "zz_lamp/v0", "x": spot.x, "y": spot.y}]), json!({})), Vec::<String>::new(), "valid");
    let exit = EncounterDef::from_obj(v.get("encounters", "lit")).exits[0];
    let cases: Vec<(&str, Value, Value, String)> = vec![
        ("off the grid", one("zz_lamp", [GRID_W, 2]), json!({}), format!("Decor 1 at ({GRID_W}, 2) is off the 12x9 grid.")),
        ("on the exit", one("zz_lamp", [exit.x, exit.y]), json!({}), format!("Decor 1 at {exit} is on an exit.")),
        ("on the hero's deploy cell", one("zz_lamp", [hero.x, hero.y]), json!({}), format!("Decor 1 at {hero} is on a deploy cell.")),
        ("on the enemy's deploy cell", one("zz_lamp", [foe.x, foe.y]), json!({}), format!("Decor 1 at {foe} is on a deploy cell.")),
        ("on a hold tile", one("zz_lamp", [spot.x, spot.y]), json!({"objective": "hold", "objective_tiles": [[spot.x, spot.y]]}), format!("Decor 1 at {spot} is on an objective tile.")),
        ("an unknown prop", one("zz_none", [spot.x, spot.y]), json!({}), "Decor 1: art 'zz_none' names no prop art.".into()),
        ("two on one cell", json!([{"art": "zz_lamp", "x": spot.x, "y": spot.y}, {"art": "zz_lamp", "x": spot.x, "y": spot.y}]), json!({}), format!("Decor 2 at {spot} shares its cell with decor 1.")),
        ("on a stay-behind tile", one("zz_lamp", [spot.x, spot.y]), json!({"objective": "escape", "stay_behind": {"mode": "hold", "tiles": [[spot.x, spot.y]]}}), format!("Decor 1 at {spot} is on an objective tile.")),
        ("cover on a catalog board", one("cover/v3", [WALL.x, WALL.y]), json!({}), "Decor 1: art 'cover/v3' needs a world stage set (a catalog encounter's board shows no cover).".into()),
        ("not a decor key", one("props/zz_ward/zz_lamp", [spot.x, spot.y]), json!({}), "Decor 1: art 'props/zz_ward/zz_lamp' is not a decor art key (cover, cover/v<N>, <prop> or <prop>/v<N>).".into()),
        ("no whole cell", json!([{"art": "zz_lamp", "x": 1.5, "y": 2}]), json!({}), "Decor 1: x and y must be whole cell numbers.".into()),
        ("an unknown key", json!([{"art": "zz_lamp", "x": spot.x, "y": spot.y, "z": 1}]), json!({}), "Decor 1: unknown keys [\"z\"] (decor holds art, x and y).".into()),
        ("not an entry", json!(["zz_lamp"]), json!({}), "Decor 1 is not an {art, x, y} entry.".into()),
        ("not a list", json!({"art": "zz_lamp"}), json!({}), "Decor must be a list of {art, x, y} entries.".into()),
    ];
    for (tag, decor, extra, want) in cases {
        assert_eq!(decor_errors(&v, decor, extra), vec![want], "{tag}");
    }
    // the ring is a hold's tile when it names none
    assert_eq!(decor_errors(&v, one("zz_lamp", [5, 4]), json!({"objective": "hold", "objective_tiles": []})).len(), 1, "the default ring");
}

#[test]
fn a_world_encounter_checks_its_decor() {
    let (_s, data) = scratch("world");
    catalog(&data, json!({}));
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1, "stage_set": "zz_stage"}).to_string()).unwrap();
    let decor = |art: &str, x: i32| json!([{"art": art, "x": x, "y": WALL.y}]);
    let entries = json!([
        {"id": "good", "name": "Good", "rows": rows(), "decor": decor("cover/v3", WALL.x)},
        {"id": "bad", "name": "Bad", "rows": rows(), "decor": decor("zz_lamp", GRID_W)},
        {"id": "past", "name": "Past", "rows": rows(), "decor": decor("cover/v5", WALL.x)},
        {"id": "alley_ok", "name": "Alley ok", "zone": "alley", "rows": rows(), "decor": decor("cover/v0", WALL.x)},
        {"id": "alley_bad", "name": "Alley bad", "zone": "alley", "rows": rows(), "decor": decor("cover/v3", WALL.x)}
    ]);
    fs::write(dir.join("encounters.json"), json!({"kind": "encounters", "version": 1, "entries": entries}).to_string()).unwrap();
    let found: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains("Decor")).collect();
    assert_eq!(found.len(), 3, "{found:?}");
    let of = |id: &str| found.iter().find(|p| p.contains(&format!("'{id}'")) || p.contains(&format!("{id}:")) || p.contains(&format!("/{id}"))).cloned().unwrap_or_default();
    assert!(of("bad").contains("off the 12x9 grid"), "{found:?}");
    assert!(of("past").contains("art 'cover/v5' names no cover in the board's stage set (zz_stage)"), "a variant another set has: {found:?}");
    assert!(of("alley_bad").contains("(zz_alley, zz_stage)"), "the zone's set is read first: {found:?}");
}
