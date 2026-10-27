//! The story checks (STORYUI1-S, docs/specs/STORYUI1.md 4.1-4.7) on the game's data: the frozen Godot data
//! (legacy/data: the Ashlands, Iron Reaches and Tidewrack) sets none of the story fields or kinds, so the story checks
//! add nothing to its reports (TS2; checks_parity.rs compares the whole reports with Godot's), and every shipped world
//! manifest resaves byte-identical through the manifest writer (WORLDW1) now that the schema has `start_sequence`. The
//! kit's own tests are kits/rpg/crates/rpgkit_content/tests/story_checks.rs.

use aegis_core::json::{self, Dict};
use emberwake_content::{check, store};
use serde_json::json;
use std::path::{Path, PathBuf};

fn game_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_classic_worlds_report_nothing_new() {
    let frozen = game_dir().join("../../legacy/data");
    if !frozen.is_dir() {
        eprintln!("no legacy/data");
        return;
    }
    let v = store::view(&frozen);
    let worlds = v.worlds.world_ids();
    assert!(!worlds.is_empty());
    for w in worlds {
        let m = v.worlds.manifest(&w);
        assert_eq!(json::get(m, "start_sequence"), &json!(""), "{w}: no start sequence (the map, as before)");
        assert_eq!(check::story::manifest_errors(&v, &w, m), Vec::<String>::new(), "{w}");
        assert_eq!(check::story::warnings(&v, &w), Vec::<String>::new(), "{w}");
        for kind in emberwake_data::STORY_KINDS {
            assert!(v.worlds.own_entries(&w, kind).is_empty(), "{w} has no {kind}");
        }
        for q in v.worlds.own_entries(&w, "quests") {
            assert_eq!((json::s(q, "board"), json::s(q, "track")), (String::new(), "side".to_string()), "{w}: {} is on no board", json::s(q, "id"));
            assert_eq!(check::board::quest_board_errors(&v, &w, q), Vec::<String>::new(), "{w}");
        }
    }
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

/// WORLDW1 with the new manifest field: an untouched resave of every shipped world manifest (a scratch copy of data/)
/// writes it back byte for byte, `start_sequence` left out while it holds its default.
#[test]
fn shipped_manifests_resave_byte_identical() {
    let tmp = std::env::temp_dir().join(format!("emberwake_story_manifests_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let data = tmp.join("data");
    copy_dir(&game_dir().join("data"), &data);
    let worlds: Vec<String> = std::fs::read_dir(data.join("worlds")).unwrap().filter_map(|e| e.ok()).filter(|e| e.path().join("world.json").is_file()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    assert!(!worlds.is_empty());
    for w in &worlds {
        let path = data.join("worlds").join(w).join("world.json");
        let before = std::fs::read_to_string(&path).unwrap();
        store::set_world(&data, w, &Dict::new(), false).unwrap_or_else(|e| panic!("{w}: {e:?}"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before, "{w}: an untouched resave is byte-identical");
        assert_eq!(before.contains("start_sequence"), json::get(store::view(&data).worlds.manifest(w), "start_sequence") != &json!(""), "{w}");
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
