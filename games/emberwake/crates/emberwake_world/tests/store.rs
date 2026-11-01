//! The content writer (emberwake_content::store) against the shipped data: every shipped data file re-saves byte for
//! byte through the writer (CLN-37, the writer's storage form and JSON format), the data folder checks clean, and
//! upsert / remove on a scratch copy of data/ validate, write and roll back as documented.

use aegis_content::store::Hooks;
use aegis_core::json::Dict;
use emberwake_content::{source, store};
use serde_json::json;
use std::path::{Path, PathBuf};

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// Every JSON file under `dir`, recursively, sorted.
fn json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            json_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
}

/// The text a save of the writer-owned file `rel` (relative to the data folder) writes now: a base catalog or
/// game-data kind (`resave_text`), a world kind (`resave_text` in the world), or a world manifest (the untouched
/// `set_world` text, fields the file leaves out at their defaults staying out). None for a file the writer does not own.
fn writer_text(dir: &Path, rel: &Path) -> Option<Result<String, String>> {
    let parts: Vec<String> = rel.iter().map(|c| c.to_string_lossy().into_owned()).collect();
    let stem = rel.file_stem()?.to_string_lossy().into_owned();
    match parts.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["worlds", w, "world.json"] => Some(store::world_manifest_text(dir, w, &Dict::new(), false).map_err(|e| e.join("; "))),
        ["worlds", w, _] if source::WORLD_KINDS.contains(&stem.as_str()) => Some(store::resave_text(dir, Some(w), &stem)),
        ["catalog", _] if store::RpgContent.kinds(None).contains(&stem.as_str()) => Some(store::resave_text(dir, None, &stem)),
        _ => store::GAME_KINDS.iter().find(|(_, file, ..)| Path::new(file) == rel).map(|(kind, ..)| store::resave_text(dir, None, kind)),
    }
}

/// Files the writer does not own: `seeds/` holds patch files that `aegis content apply` reads and never writes;
/// `progression/xp_curve.json` is read by the loader (`source::xp_curve`) but is no writer kind (no upsert, no save).
const NOT_WRITER_OWNED: [&str; 2] = ["seeds", "progression/xp_curve.json"];

/// CLN-37 (replaces the Godot resave comparison and its AFTER_GODOT lists): re-saving any shipped data file without
/// changes writes it back byte for byte. Every JSON file under data/ must be one the writer owns (catalog kinds, the
/// engine-only base kinds, game-data kinds, world kinds and world manifests) or one of NOT_WRITER_OWNED. Read-only:
/// the texts are computed, not written.
#[test]
fn every_shipped_file_resaves_byte_for_byte() {
    let dir = data_dir();
    let mut files = Vec::new();
    json_files(&dir, &mut files);
    let mut diffs = Vec::new();
    let mut compared = 0;
    for path in &files {
        let rel = path.strip_prefix(&dir).unwrap();
        let key = rel.to_string_lossy().replace('\\', "/");
        if NOT_WRITER_OWNED.iter().any(|n| key == *n || key.starts_with(&format!("{n}/"))) {
            continue;
        }
        let Some(text) = writer_text(&dir, rel) else {
            diffs.push(format!("{key}: not a file the writer owns"));
            continue;
        };
        compared += 1;
        let want = std::fs::read_to_string(path).unwrap();
        match text {
            Err(e) => diffs.push(format!("{key}: {e}")),
            Ok(t) if t != want => {
                let line = t.lines().zip(want.lines()).position(|(a, b)| a != b).unwrap_or_else(|| t.lines().count().min(want.lines().count()));
                diffs.push(format!("{key} line {}:\n  file   {:?}\n  writer {:?}", line + 1, want.lines().nth(line), t.lines().nth(line)));
            }
            Ok(_) => {}
        }
    }
    eprintln!("shipped files re-saved: {compared}");
    assert!(compared > 0 && files.iter().any(|p| p.ends_with("world.json")), "no shipped data found under {}", dir.display());
    assert!(diffs.is_empty(), "{} files differ; first:\n{}", diffs.len(), diffs.iter().take(5).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
fn shipped_data_checks_clean() {
    let problems = store::check(&data_dir());
    assert!(problems.is_empty(), "{problems:?}");
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else if p.extension().is_some_and(|x| x == "json") {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

#[test]
fn upsert_and_remove_validate_write_and_roll_back() {
    let tmp = std::env::temp_dir().join(format!("aegis_store_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    for sub in ["catalog", "worlds", "maps", "quests", "dialogue", "progression"] {
        copy_dir(&data_dir().join(sub), &tmp.join(sub));
    }
    let units = store::file_of(&tmp, None, "units");
    // the copy in the writer's own form first (a schema field added since the data was last written shows up here)
    std::fs::write(&units, store::resave_text(&tmp, None, "units").unwrap()).unwrap();
    let original = std::fs::read_to_string(&units).unwrap();

    let unit = json!({"id": "test_recruit", "name": "Test Recruit", "side": "party", "class_id": "warrior", "level": 2});
    let out = store::upsert(&tmp, None, "units", unit.as_object().unwrap(), None).expect("valid unit");
    assert!(!out.replaced);
    let d = emberwake_world::Data::open(&tmp).unwrap().0;
    assert_eq!(d.entry("units", "test_recruit")["level"], json!(2));

    let bad = json!({"id": "test_bad", "name": "Bad", "class_id": "no_such_class"});
    let errors = store::upsert(&tmp, None, "units", bad.as_object().unwrap(), None).err().expect("unknown class refused");
    assert!(errors.iter().any(|e| e.contains("no_such_class")), "{errors:?}");
    assert!(!emberwake_world::Data::open(&tmp).unwrap().0.has("units", "test_bad"), "rolled back");

    let bad_id = json!({"id": "Bad Id", "name": "x"});
    assert!(store::upsert(&tmp, None, "units", bad_id.as_object().unwrap(), None).is_err());

    store::remove(&tmp, None, "units", "test_recruit").expect("remove");
    assert_eq!(std::fs::read_to_string(&units).unwrap(), original, "upsert + remove leaves the file as it was");

    let lore = json!({"id": "test_note", "name": "Test Note", "category": "history", "text": "A line."});
    store::upsert(&tmp, Some("ashlands"), "lore", lore.as_object().unwrap(), None).expect("world entry");
    let d = emberwake_world::Data::open(&tmp).unwrap().0;
    assert_eq!(aegis_core::json::s(d.resolve("ashlands", "lore", "test_note"), "text"), "A line.");

    // A ported content rule refuses a catalog write: a class whose base stats miss the class budget.
    let mut class = d.entry("classes", "warrior").clone();
    class.insert("id".into(), json!("test_class"));
    class.insert("base_stats".into(), json!({"STR": 18, "DEX": 18, "INT": 18, "CONS": 18, "PERC": 18, "KNOW": 18}));
    let errors = store::upsert(&tmp, None, "classes", &class, None).err().expect("over budget refused");
    assert!(errors.iter().any(|e| e.contains("Base stats total 108")), "{errors:?}");

    // A world write may name an entry that does not exist yet (WorldCatalog.upsert: worlds are full of mutual
    // references); the check still reports it.
    let region = json!({"id": "test_region", "name": "Test Region", "faction": "test_faction"});
    store::upsert(&tmp, Some("ashlands"), "regions", region.as_object().unwrap(), None).expect("forward reference kept");
    assert!(store::check(&tmp).iter().any(|p| p.contains("unknown faction 'test_faction'")));
    let bad_shape = json!({"id": "test_band", "name": "Test Band", "relations": {"ember_wardens": "friendly"}});
    let errors = store::upsert(&tmp, Some("ashlands"), "factions", bad_shape.as_object().unwrap(), None).err().expect("bad stance refused");
    assert!(errors.iter().any(|e| e.contains("Relation stance 'friendly'")), "{errors:?}");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// PROPS1 1-2: a map template's `props` through the writer, on a test-only template in a scratch copy: no `props`
/// loads and stores as `[]`, a prop given with only id, art, x and y is stored whole in key order, and an upsert whose
/// standing prop lies on an open tile (P9) is refused and leaves maps.json as it was.
#[test]
fn map_props_are_stored_whole_and_checked_on_upsert() {
    let tmp = std::env::temp_dir().join(format!("aegis_store_props_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    for sub in ["catalog", "worlds", "maps", "quests", "dialogue", "progression"] {
        copy_dir(&data_dir().join(sub), &tmp.join(sub));
    }
    let maps = store::file_of(&tmp, None, "maps");
    std::fs::write(&maps, store::resave_text(&tmp, None, "maps").unwrap()).unwrap();
    let original = std::fs::read_to_string(&maps).unwrap();
    let rows = ["########", "#......#", "#......#", "#......#", "#......#", "#......#", "#......#", "########"];
    let base = json!({"id": "test_props_map", "name": "Test Props", "rows": rows, "zones": [], "zone_legend": {}, "anchors": [{"id": "spawn", "x": 3, "y": 3}], "paths": {}, "tags": []});
    let stored = |dir: &Path| -> serde_json::Value {
        let doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(store::file_of(dir, None, "maps")).unwrap()).unwrap();
        doc["entries"].as_array().unwrap().iter().find(|e| e["id"] == "test_props_map").cloned().expect("stored")
    };
    store::upsert(&tmp, None, "maps", base.as_object().unwrap(), None).expect("a template without props");
    let loaded = source::load(&tmp).game["maps"].as_array().unwrap().iter().find(|e| e["id"] == "test_props_map").cloned().unwrap();
    assert_eq!(loaded["props"], json!([]));
    let keys: Vec<String> = stored(&tmp).as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, ["id", "name", "rows", "zones", "zone_legend", "anchors", "paths", "props", "tags"]);

    let mut with_post = base.clone();
    with_post["props"] = json!([{"y": 0, "x": 2, "art": "signpost", "id": "road_sign"}]);
    store::upsert(&tmp, None, "maps", with_post.as_object().unwrap(), None).expect("a prop on the wall");
    let prop = stored(&tmp)["props"][0].clone();
    let prop_keys: Vec<String> = prop.as_object().unwrap().keys().cloned().collect();
    assert_eq!(prop_keys, source::PROP_KEYS);
    assert_eq!(prop, json!({"id": "road_sign", "art": "signpost", "x": 2, "y": 0, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false}));

    let before = std::fs::read_to_string(&maps).unwrap();
    let mut on_floor = base.clone();
    on_floor["props"] = json!([{"id": "road_sign", "art": "signpost", "x": 2, "y": 1}]);
    let errors = store::upsert(&tmp, None, "maps", on_floor.as_object().unwrap(), None).err().expect("P9 refused");
    let p9 = "Prop 'road_sign': a standing prop needs wall tiles under its whole footprint; (2, 1) is '.'.";
    assert!(errors.iter().any(|e| e.contains(p9)), "{errors:?}");
    assert_eq!(std::fs::read_to_string(&maps).unwrap(), before, "a refused upsert leaves maps.json unchanged");

    store::remove(&tmp, None, "maps", "test_props_map").expect("remove");
    assert_eq!(std::fs::read_to_string(&maps).unwrap(), original, "upsert + remove leaves the file as it was");
    let _ = std::fs::remove_dir_all(&tmp);
}
