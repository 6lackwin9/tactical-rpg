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
/// (HISTORY 2026-10-22) a unit's and a character's `palette` and `portrait_tier`; PROPS1 (HISTORY 2026-10-23) a map
/// template's `props`; RULESET1-R (HISTORY 2026-10-25) a world manifest's `ruleset` and `plan_reveal_flag`; RULESET1-S
/// (HISTORY 2026-10-25) a unit's `poise`, `deck` and `default_intent` and a passive's `slot` (a skill's timeline fields
/// are left out at their defaults, as every skill field); ENCOUNTER2-T (HISTORY 2026-10-25) an encounter's `rows`
/// (catalog and world encounters); HOOKS1-S (HISTORY 2026-10-27) a unit's `tactics` and an encounter's `arena_hazards`
/// (catalog and world encounters; a skill's hook fields are left out at their defaults, a consumable's `hazard` is
/// ITEMS_AFTER_GODOT); ENCOUNTER2-H (HISTORY 2026-10-27) an encounter's `unkillable`, `timeout_outcome` and
/// `stay_behind` (catalog and world encounters); ROSTER1-L (HISTORY 2026-10-31) a character's `keepsake` and a world
/// encounter's `party_min`.
const AFTER_GODOT: [&str; 20] = [
    "\"loot\": []",
    "\"key_loot\": []",
    "\"objective_reward\": {}",
    "\"palette\": \"\"",
    "\"portrait_tier\": \"\"",
    "\"props\": []",
    "\"ruleset\": \"\"",
    "\"plan_reveal_flag\": \"\"",
    "\"poise\": -1",
    "\"deck\": {}",
    "\"default_intent\": \"\"",
    "\"slot\": \"\"",
    "\"rows\": []",
    "\"tactics\": []",
    "\"arena_hazards\": []",
    "\"unkillable\": []",
    "\"timeout_outcome\": \"fled\"",
    "\"stay_behind\": {}",
    "\"keepsake\": \"\"",
    "\"party_min\": 0",
];

/// Retired parity, the catalog items file only (catalog/items.json): HOOKS1-S (HISTORY 2026-10-27) a consumable's
/// `hazard`, written as `{}`.
const ITEMS_AFTER_GODOT: [&str; 1] = ["\"hazard\": {}"];

/// Retired parity, world manifests only (world.json): CAMPAIGN1-S (HISTORY 2026-10-25) a manifest's `campaign`,
/// `purse`, `rest` and `defeat`, written as `{}` (an encounter's `purse` line is still compared); STORYUI1-S (HISTORY
/// 2026-10-26) its `start_sequence`, written as ""; HOOKS1-S (HISTORY 2026-10-27) its `classes`, written as `{}`;
/// ROSTER1-L (HISTORY 2026-10-31) its `roster`, written as `{}`.
const MANIFEST_AFTER_GODOT: [&str; 7] = ["\"campaign\": {}", "\"purse\": {}", "\"rest\": {}", "\"defeat\": {}", "\"start_sequence\": \"\"", "\"classes\": {}", "\"roster\": {}"];

/// Retired parity, world quests only (worlds/<w>/quests.json): STORYUI1-S (HISTORY 2026-10-26) a quest's board
/// fields `board`, `track`, `show_locked` and `closed_text`, written with their defaults (a dialogue choice's
/// `show_locked` line is still compared).
const QUEST_AFTER_GODOT: [&str; 4] = ["\"board\": \"\"", "\"track\": \"side\"", "\"show_locked\": false", "\"closed_text\": \"\""];

/// Retired parity, kind level: CAMPAIGN1-S (HISTORY 2026-10-25) the world kinds `meters`, `checkpoints` and
/// `triggers`; STORYUI1-S (HISTORY 2026-10-26) the world kinds `vignettes`, `sequences` and `boards` (Godot's
/// WorldCatalog.save_all wrote no such files).
const WORLD_KINDS_AFTER_GODOT: [&str; 6] = ["meters", "checkpoints", "triggers", "vignettes", "sequences", "boards"];

/// `text` without the AFTER_GODOT field lines, and without the file's own retired lines (`extra`: a world manifest's
/// MANIFEST_AFTER_GODOT, a world quests file's QUEST_AFTER_GODOT, the catalog items file's ITEMS_AFTER_GODOT); a
/// dropped last field takes the comma off the line before it.
fn at_freeze(text: &str, extra: &[&str]) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        let field = t.trim_end_matches(',');
        if AFTER_GODOT.contains(&field) || extra.contains(&field) {
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

/// Retired parity (PHASEFOLD, HISTORY 2026-10-22): the single reinforcement wave became an arrival phase, so the
/// writer stores no `wave_*` keys. Godot's saved encounters files are parsed, every entry folded by the engine's
/// `fold_legacy_wave` and printed by the same document writer before comparing; every other file stays byte for byte.
fn godot_encounters_folded(text: &str) -> String {
    let doc: serde_json::Value = serde_json::from_str(text).expect("Godot's encounters file");
    let kind = doc["kind"].as_str().unwrap_or("encounters").to_string();
    let version = doc["version"].as_i64().unwrap_or(1);
    let (entries, _) = aegis_content::file::entries_of(doc);
    let folded = entries
        .into_iter()
        .map(|mut e| {
            if let Some(o) = e.as_object_mut() {
                emberwake_battle::encounter::fold_legacy_wave(o);
            }
            e
        })
        .collect();
    aegis_content::file::document(&kind, version, folded)
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
        let mut want = std::fs::read_to_string(after.join(&rel)).unwrap_or_default().replace("\r\n", "\n");
        if rel.file_name().is_some_and(|n| n == "encounters.json") && !want.is_empty() {
            want = godot_encounters_folded(&want);
        }
        let in_world = rel.starts_with("worlds");
        let extra: &[&str] = match rel.file_name().and_then(|n| n.to_str()) {
            Some("world.json") => &MANIFEST_AFTER_GODOT,
            Some("quests.json") if in_world => &QUEST_AFTER_GODOT,
            Some("items.json") if !in_world => &ITEMS_AFTER_GODOT,
            _ => &[],
        };
        match text {
            Err(e) => diffs.push(format!("{}: {e}", path.display())),
            Ok(t) if at_freeze(&t, extra) != want => {
                let t = at_freeze(&t, extra);
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
        for kind in source::WORLD_KINDS.into_iter().filter(|k| !WORLD_KINDS_AFTER_GODOT.contains(k)) {
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
