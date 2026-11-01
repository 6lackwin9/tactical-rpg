//! The story checks (STORYUI1-S, docs/specs/STORYUI1.md 4.1-4.7) on the game's data: every shipped world manifest
//! resaves byte-identical through the manifest writer (WORLDW1) now that the schema has `start_sequence`. The kit's own
//! tests are kits/rpg/crates/rpgkit_content/tests/story_checks.rs. (The one-off "frozen Godot data reports nothing new"
//! check went with CLN-36.)

use aegis_core::json::{self, Dict};
use emberwake_content::store;
use serde_json::json;
use std::path::{Path, PathBuf};

fn game_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
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
