//! PR5-49 (MAP-POLISH1) on scratch data folders and asset layers: the world map fields of world data. A region's
//! `map_reveal` (bool) and a location's landmark `art` (string) are schema fields stored only when set
//! (`omit_default`), so files written before them re-save byte for byte; set, they are written and loaded. A location's
//! `art` names its landmark picture (`art::landmark`: `map/landmarks/<art>.png`, else its local id); a bad name is an
//! error, a name no layer holds a warning, and the layout sidecar check looks for the picture by the same rule.

use aegis_content::schema::fields;
use aegis_core::assets::Layers;
use aegis_core::json::{self, Dict};
use aegis_core::scratch::Scratch;
use rpgkit_content::check::{assets, map, View};
use rpgkit_content::{art, source, store, world_map};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn field(kind: &str, key: &str) -> Dict {
    fields(source::schema(), "world", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("{kind}.{key}")).clone()
}

/// A scratch data folder with world `w` (its manifest, regions and locations files as fixtures, the writer's input).
fn scratch(tag: &str) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_world_map_fields_{tag}"));
    let data = t.join("data");
    let w = data.join("worlds/w");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(&w).unwrap();
    fs::write(w.join("world.json"), json!({"id": "w", "name": "W"}).to_string()).unwrap();
    let file = |kind: &str, entries: Value| fs::write(w.join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
    file("regions", json!([{"id": "quarter", "name": "Quarter"}, {"id": "outer", "name": "Outer"}]));
    file("locations", json!([{"id": "hub", "name": "Hub", "kind": "town", "region": "quarter"}, {"id": "gate", "name": "Gate", "kind": "gate", "region": "quarter"}]));
    (t, data)
}

fn stored(data: &Path, kind: &str) -> Vec<Dict> {
    let v: Value = serde_json::from_str(&fs::read_to_string(store::file_of(data, Some("w"), kind)).unwrap()).unwrap();
    json::aget(json::dict(&v), "entries").iter().map(|e| json::dict(e).clone()).collect()
}

fn row(rows: &[Dict], id: &str) -> Dict {
    rows.iter().find(|r| json::s(r, "id") == id).cloned().unwrap_or_else(|| panic!("{id}"))
}

#[test]
fn the_fields_are_schema_fields_stored_only_when_set() {
    let reveal = field("regions", "map_reveal");
    assert_eq!((json::s(&reveal, "type"), json::get(&reveal, "default").clone()), ("bool".to_string(), json!(false)));
    let look = field("locations", "art");
    assert_eq!((json::s(&look, "type"), json::get(&look, "default").clone()), ("string".to_string(), json!("")));
    assert!(json::b(&reveal, "omit_default", false) && json::b(&look, "omit_default", false), "both stored only when set");
}

#[test]
fn files_without_the_fields_resave_byte_for_byte_and_set_fields_round_trip() {
    let (_t, data) = scratch("round_trip");
    for kind in ["regions", "locations"] {
        let path = store::file_of(&data, Some("w"), kind);
        let text = store::resave_text(&data, Some("w"), kind).unwrap();
        fs::write(&path, &text).unwrap();
        assert_eq!(store::resave_text(&data, Some("w"), kind).unwrap(), text, "{kind}: stable");
        let key = if kind == "regions" { "map_reveal" } else { "\"art\"" };
        assert!(!text.contains(key), "{kind}: nothing written for rows without it");
    }
    let before = fs::read_to_string(store::file_of(&data, Some("w"), "locations")).unwrap();
    // set: written on that row only; the loaded world carries them
    store::upsert(&data, Some("w"), "regions", json::dict(&json!({"id": "quarter", "name": "Quarter", "map_reveal": true})), None).unwrap();
    store::upsert(&data, Some("w"), "locations", json::dict(&json!({"id": "gate", "name": "Gate", "kind": "gate", "region": "quarter", "art": "shared_gate"})), None).unwrap();
    let (regions, locations) = (stored(&data, "regions"), stored(&data, "locations"));
    assert_eq!(row(&regions, "quarter").get("map_reveal"), Some(&json!(true)));
    assert!(row(&regions, "outer").get("map_reveal").is_none(), "the other region keeps its shape");
    assert_eq!(row(&locations, "gate").get("art"), Some(&json!("shared_gate")));
    assert!(row(&locations, "hub").get("art").is_none());
    let loaded = store::view(&data);
    assert_eq!(loaded.worlds.resolve("w", "regions", "quarter").get("map_reveal"), Some(&json!(true)), "loaded");
    assert_eq!(json::s(loaded.worlds.resolve("w", "locations", "gate"), "art"), "shared_gate", "loaded");
    // back to the defaults: left out again, the file as it was
    store::upsert(&data, Some("w"), "locations", json::dict(&json!({"id": "gate", "name": "Gate", "kind": "gate", "region": "quarter", "art": ""})), None).unwrap();
    assert_eq!(fs::read_to_string(store::file_of(&data, Some("w"), "locations")).unwrap(), before, "a default is not stored: byte-identical");
    store::upsert(&data, Some("w"), "regions", json::dict(&json!({"id": "quarter", "name": "Quarter", "map_reveal": false})), None).unwrap();
    assert!(row(&stored(&data, "regions"), "quarter").get("map_reveal").is_none());
}

#[test]
fn a_locations_art_names_its_landmark_picture() {
    assert_eq!(art::landmark("w:gate", ""), "map/landmarks/gate.png", "empty: the local id");
    assert_eq!(art::landmark("w:gate", "shared_gate"), "map/landmarks/shared_gate.png");
    let v = View::new(&json!({}), &json!({}), &json!({}), Layers::default());
    let errs = |a: &str| map::location_errors(&v, "w", json::dict(&json!({"id": "gate", "name": "Gate", "art": a})));
    assert!(errs("").is_empty() && errs("shared_gate").is_empty(), "{:?}", errs("shared_gate"));
    for bad in ["Gate", "a", "x/y", "gate.png"] {
        assert_eq!(errs(bad).len(), 1, "'{bad}': one error");
        assert!(errs(bad)[0].contains(&format!("'{bad}'")), "{:?}", errs(bad));
    }
}

/// A scratch asset layer holding `files` (stand-in contents).
fn layer(s: &Scratch, files: &[&str]) -> PathBuf {
    let root = s.path().join("assets");
    for rel in files {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, "png").unwrap();
    }
    root
}

#[test]
fn a_missing_art_picture_warns_and_the_sidecar_check_looks_by_the_same_rule() {
    let s = Scratch::new("wmf_art");
    let root = layer(&s, &["map/landmarks/shared_gate.png", "map/landmarks/hub.png"]);
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {
        "regions": [{"id": "quarter", "name": "Quarter"}],
        "locations": [
            {"id": "hub", "name": "Hub", "kind": "town", "region": "quarter"},
            {"id": "gate", "name": "Gate", "kind": "gate", "region": "quarter", "art": "shared_gate"},
            {"id": "gate2", "name": "Gate Two", "kind": "gate", "region": "quarter", "art": "shared_gate"},
            {"id": "lost", "name": "Lost", "kind": "wild", "region": "quarter", "art": "no_such_art"}
        ],
        "routes": [{"id": "road", "from": "hub", "to": "gate"}],
    }}});
    let v = View::new(&json!({}), &worlds, &json!({}), Layers::of(vec![root]));
    assert_eq!(assets::landmark_art_warnings(&v, "w"), vec!["Location 'lost' landmark art 'no_such_art': no file map/landmarks/no_such_art.png in any layer; the world map draws its kind pin.".to_string()]);
    let none = View::new(&json!({}), &worlds, &json!({}), Layers::default());
    assert!(assets::landmark_art_warnings(&none, "w").is_empty(), "skipped without asset layers");
    // the sidecar: gate and gate2 share one picture (no warning, no gate.png needed); lost warns by its art's name
    let layout = json!({"landmarks": {"hub": {"pos": [0.5, 0.5]}, "gate": {"pos": [0.2, 0.2]}, "gate2": {"pos": [0.3, 0.3]}, "lost": {"pos": [0.4, 0.4]}}});
    let f = world_map::check(&v.worlds, &v.assets, "quarter", &layout);
    assert!(f.errors.is_empty(), "{:?}", f.errors);
    assert_eq!(f.warnings, vec!["Landmark 'lost' has no map/landmarks/no_such_art.png: the map draws its pin ui/pins/wild.png.".to_string()]);
}
