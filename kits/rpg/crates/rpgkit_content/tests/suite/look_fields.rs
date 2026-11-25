//! PR5-33 (LAYER-UP-DATA) on scratch data folders: the presentation fields of skills (`fx`, `clip`), item families
//! (`fx`, `fx_far`, `attack_clips`, `area_clips`) and terrain (`relief`, `relief_depth`, `prop`, `color`, `face`) are
//! schema fields stored only when set (`omit_default`), so a file written before they existed re-saves byte for byte; a
//! skill's look is an entry field every kind has, so its rank rows load it too; the loaded terrain table carries each
//! type's look; and the fields add no check message.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_battle::grid::TerrainTable;
use rpgkit_content::{check, source, store};
use rpgkit_data::looks::{Face, Relief};
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const SKILL_KEYS: [&str; 2] = ["fx", "clip"];
const FAMILY_KEYS: [&str; 4] = ["fx", "fx_far", "attack_clips", "area_clips"];
const TERRAIN_KEYS: [&str; 5] = ["relief", "relief_depth", "prop", "color", "face"];

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_look_fields_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

/// Writes a catalog file as a fixture (the file is the test's own input; the writer is tested below); skills in their
/// ranked (version 2) form.
fn write_kind(data: &Path, kind: &str, entries: Value) {
    let version = if kind == "skills" { 2 } else { 1 };
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn field(kind: &str, key: &str) -> Dict {
    fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("{kind}.{key}")).clone()
}

fn loaded(data: &Path, kind: &str) -> Vec<Dict> {
    let src = source::load(data);
    json::aget(json::dict(&src.catalog), kind).iter().map(|v| json::dict(v).clone()).collect()
}

#[test]
fn the_look_fields_are_schema_fields_stored_only_when_set() {
    for (kind, keys) in [("skills", &SKILL_KEYS[..]), ("item_families", &FAMILY_KEYS[..]), ("terrain", &TERRAIN_KEYS[..])] {
        for key in keys {
            let f = field(kind, key);
            assert!(json::b(&f, "omit_default", false), "{kind}.{key} is stored only when set");
            let d = json::get(&f, "default");
            assert!(d == &json!("") || d == &json!(0) || d == &json!([]), "{kind}.{key}: an empty default ({d})");
        }
    }
    for key in SKILL_KEYS {
        assert!(rpgkit_content::check::skill::KINDS.iter().all(|k| rpgkit_content::check::skill::has_field(k, key)), "{key}: every skill kind has it");
        assert_eq!(json::get(&field("skills", key), "rank"), &json!(false), "{key} is an entry field, the same at every rank");
    }
}

/// Terrain: a file of rows without the fields re-saves byte for byte with none written; an upsert stores the given
/// fields on its row only; the loaded table carries them as typed looks.
#[test]
fn terrain_rows_store_and_load_their_looks() {
    let (_t, data) = scratch("terrain");
    write_kind(&data, "terrain", json!([{"id": "grass", "move_cost": 1, "cover_guard": 0.0}, {"id": "lava", "move_cost": 1, "cover_guard": 0.0}]));
    let file = data.join("catalog/terrain.json");
    let text = store::resave_text(&data, None, "terrain").unwrap();
    fs::write(&file, &text).unwrap();
    assert_eq!(store::resave_text(&data, None, "terrain").unwrap(), text, "stable");
    assert!(TERRAIN_KEYS.iter().all(|k| !text.contains(&format!("\"{k}\""))), "no look keys written for rows without them");
    let lava = json!({"id": "lava", "move_cost": 1, "cover_guard": 0.0, "relief": "sunken", "relief_depth": 5, "color": "#963214", "face": "lava"});
    store::upsert(&data, None, "terrain", json::dict(&lava), None).unwrap();
    let stored: Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
    let row = |id: &str| stored["entries"].as_array().unwrap().iter().find(|r| r["id"] == json!(id)).unwrap().clone();
    assert_eq!((row("lava")["relief"].clone(), row("lava")["relief_depth"].clone(), row("lava")["face"].clone()), (json!("sunken"), json!(5), json!("lava")));
    assert!(row("lava").get("prop").is_none(), "a field left at its default is not stored");
    assert!(TERRAIN_KEYS.iter().all(|k| row("grass").get(*k).is_none()), "the other rows keep their shape");
    let after = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, "terrain").unwrap(), after, "a re-save without changes is byte-identical");
    let src = source::load(&data);
    let table = TerrainTable::from_catalog(&Catalog::from_value(&src.catalog));
    let look = table.look("lava");
    assert_eq!((look.relief, look.depth, look.color, look.face), (Relief::Sunken, 5, Some([150, 50, 20]), Face::Lava));
    assert_eq!(table.look("grass"), table.look("no_such_terrain"), "no look fields: the default look");
}

/// Skills: a ranked entry's `fx` and `clip` are stored once on the entry (never in a rank modifier), and every rank row
/// loads them.
#[test]
fn a_ranked_skill_stores_its_look_once_and_every_rank_loads_it() {
    let (_t, data) = scratch("skills");
    let entry = json!({"id": "lob", "name": "Lob", "kind": "ATTACK", "level": 1, "blocks": 2, "description": "A lob.", "power_mult": 1.0,
        "fx": "fire_lob", "clip": "throw", "ranks": [{"level": 3, "power_mult": 1.2}]});
    let plain = json!({"id": "jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "description": "A jab.", "power_mult": 1.0});
    write_kind(&data, "skills", json!([entry, plain]));
    let text = store::resave_text(&data, None, "skills").unwrap();
    let stored: Value = serde_json::from_str(&text).unwrap();
    let lob = stored["entries"].as_array().unwrap().iter().find(|r| r["id"] == json!("lob")).unwrap();
    let jab = stored["entries"].as_array().unwrap().iter().find(|r| r["id"] == json!("jab")).unwrap();
    assert_eq!((lob["fx"].clone(), lob["clip"].clone()), (json!("fire_lob"), json!("throw")));
    assert!(lob["ranks"].as_array().unwrap().iter().all(|m| m.get("fx").is_none() && m.get("clip").is_none()), "not a rank field");
    assert!(jab.get("fx").is_none() && jab.get("clip").is_none(), "unset: not stored");
    let rows = loaded(&data, "skills");
    let lobs: Vec<(String, String, String)> = rows.iter().filter(|r| json::s(r, "id").starts_with("lob")).map(|r| (json::s(r, "id"), json::s(r, "fx"), json::s(r, "clip"))).collect();
    assert_eq!(lobs, [("lob".into(), "fire_lob".into(), "throw".into()), ("lob_2".into(), "fire_lob".into(), "throw".into())]);
}

/// Item families: the look fields are stored when set and left out otherwise; the world data's catalog carries them.
#[test]
fn a_family_stores_its_looks_only_when_set() {
    let (_t, data) = scratch("families");
    let base = |id: &str| json!({"id": id, "name": id, "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h"});
    let mut pike = base("pike");
    pike["fx"] = json!("stab");
    pike["attack_clips"] = json!(["thrust"]);
    write_kind(&data, "item_families", json!([pike, base("club")]));
    let text = store::resave_text(&data, None, "item_families").unwrap();
    let stored: Value = serde_json::from_str(&text).unwrap();
    let row = |id: &str| stored["entries"].as_array().unwrap().iter().find(|r| r["id"] == json!(id)).unwrap().clone();
    assert_eq!((row("pike")["fx"].clone(), row("pike")["attack_clips"].clone()), (json!("stab"), json!(["thrust"])));
    assert!(row("pike").get("fx_far").is_none() && row("pike").get("area_clips").is_none(), "unset fields are not stored");
    assert!(FAMILY_KEYS.iter().all(|k| row("club").get(*k).is_none()), "a family without looks keeps its shape");
    let src = source::load(&data);
    let mut cat = Catalog::from_value(&src.catalog);
    assert!(cat.family_looks.is_empty(), "the resolved catalog leaves the family rows out");
    let rows: Vec<Dict> = json::aget(json::dict(&src.game), "item_families").iter().map(|v| json::dict(v).clone()).collect();
    cat.set_family_looks(rows.iter());
    assert_eq!(cat.family_looks.get("pike").map(|f| (f.fx.as_str(), f.attack_clips.clone())), Some(("stab", vec!["thrust".to_string()])));
    assert_eq!(cat.family_looks.get("club").map(|f| f.fx.as_str()), Some(""));
}

/// A terrain colour is empty or `#rrggbb`; anything else is refused by name.
#[test]
fn a_terrain_colour_is_checked() {
    let (_t, data) = scratch("colour");
    write_kind(&data, "terrain", json!([{"id": "grass", "move_cost": 1, "cover_guard": 0.0}]));
    let v = store::view(&data);
    let errors = |c: &str| check::validate_entry(&v, "terrain", json::dict(&json!({"id": "grass", "color": c})), "grass");
    let refused = |c: &str| errors(c).iter().any(|m| m == &format!("Colour '{c}' must be #rrggbb (or empty for the default)."));
    assert!(!refused("") && !refused("#3c6838") && !refused("#3C6838"));
    for bad in ["3c6838", "#3c683", "#3c68zz", "green"] {
        assert!(refused(bad), "{bad}");
    }
}

/// The look fields add no check message: an entry checks the same with and without them.
#[test]
fn the_look_fields_add_no_check_message() {
    let (_t, data) = scratch("checks");
    write_kind(&data, "terrain", json!([{"id": "grass", "move_cost": 1, "cover_guard": 0.0}]));
    let v = store::view(&data);
    let errors = |kind: &str, e: Value| check::validate_entry(&v, kind, json::dict(&e), &json::s(json::dict(&e), "id"));
    assert_eq!(
        errors("terrain", json!({"id": "grass", "relief": "block", "relief_depth": 3, "prop": "post", "color": "#000000", "face": "wall"})),
        errors("terrain", json!({"id": "grass"}))
    );
    assert_eq!(
        errors("skills", json!({"id": "jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "fx": "any_look", "clip": "any_clip"})),
        errors("skills", json!({"id": "jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1}))
    );
}
