//! The content writer (emberwake_content::store) against the shipped data: re-saving any data file without changes writes it
//! back byte for byte (Godot's storage form and JSON format), the data folder checks clean, and upsert / remove on a
//! scratch copy of data/ validate, write and roll back as documented.

use emberwake_content::{source, store};
use serde_json::json;
use std::path::{Path, PathBuf};

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

const CATALOG: [&str; 8] = ["classes", "units", "skills", "items", "item_families", "passives", "encounters", "terrain"];

/// Retired parity: fields added after the Godot freeze, written with their empty defaults by the engine's writer:
/// REWARD1 (HISTORY 2026-10-21) a unit's `loot`, a world encounter's `key_loot` and `objective_reward`; ASSETORG2
/// (HISTORY 2026-10-22) a unit's and a character's `palette` and `portrait_tier`.
const AFTER_GODOT: [&str; 5] = ["\"loot\": []", "\"key_loot\": []", "\"objective_reward\": {}", "\"palette\": \"\"", "\"portrait_tier\": \"\""];

/// `text` without the AFTER_GODOT field lines (a dropped last field takes the comma off the line before it).
fn at_freeze(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        let field = t.trim_end_matches(',');
        if AFTER_GODOT.contains(&field) {
            if !t.ends_with(',') {
                if let Some(prev) = out.last_mut() {
                    if prev.ends_with(',') {
                        prev.pop();
                    }
                }
            }
            continue;
        }
        out.push(line.to_string());
    }
    let mut s = out.join("\n");
    if text.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Godot's saves of the same files: `parity/export.sh only=resave` copies the catalog and worlds to resave/before and
/// has ContentCatalog.save_all / WorldCatalog.save_all write them into resave/after. The writer, given resave/before,
/// must write every file of resave/after byte for byte (field order, storage form, empty kinds).
#[test]
fn writer_saves_exactly_what_godot_saves() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out/resave");
    if !root.join("after").is_dir() {
        eprintln!("no parity/out/resave: run games/emberwake/parity/export.sh");
        return;
    }
    let dir = root.join("before");
    let after = root.join("after");
    let mut diffs = Vec::new();
    let mut files = 0;
    let mut compare = |path: PathBuf, text: Result<String, String>| {
        files += 1;
        let rel = path.strip_prefix(&dir).unwrap().to_path_buf();
        let want = std::fs::read_to_string(after.join(&rel)).unwrap_or_default().replace("\r\n", "\n");
        match text {
            Err(e) => diffs.push(format!("{}: {e}", path.display())),
            Ok(t) if at_freeze(&t) != want => {
                let t = at_freeze(&t);
                let line = t.lines().zip(want.lines()).position(|(a, b)| a != b).unwrap_or(0);
                diffs.push(format!("{} line {}:\n  file   {:?}\n  writer {:?}", path.display(), line + 1, want.lines().nth(line), t.lines().nth(line)));
            }
            Ok(_) => {}
        }
    };
    for kind in CATALOG {
        compare(store::file_of(&dir, None, kind), store::resave_text(&dir, None, kind));
    }
    for w in ["ashlands", "iron_reaches", "tidewrack"] {
        compare(dir.join("worlds").join(w).join("world.json"), store::manifest_text(&dir, w));
        for kind in source::WORLD_KINDS {
            compare(store::file_of(&dir, Some(w), kind), store::resave_text(&dir, Some(w), kind));
        }
    }
    for (kind, ..) in store::GAME_KINDS {
        compare(store::file_of(&dir, None, kind), store::resave_text(&dir, None, kind));
    }
    eprintln!("files compared with Godot's save: {files}");
    assert!(diffs.is_empty(), "{} files differ; first:\n{}", diffs.len(), diffs.iter().take(3).cloned().collect::<Vec<_>>().join("\n"));
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
