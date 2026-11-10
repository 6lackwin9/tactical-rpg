//! WEBREL1-G1: `aegis bundle` packs a web build's bundle parts as the game's manifest names them: the paths relative
//! to `root` are the keys, an exclude drops a path or a file ending, the parts follow the manifest's order, and
//! `bundle.json` lists them with their file counts and sizes. A file in two parts, a missing include or an empty part
//! stops it.

use aegis_cli::bundle::{self, Written};
use aegis_core::scratch::Scratch;
use aegis_core::vfs::Table;
use serde_json::{json, Value};
use std::path::Path;

fn tree(dir: &Path) {
    for (rel, text) in [
        ("game/game.json", "{}"),
        ("game/data/catalog/units.json", "[]"),
        ("game/data/README.md", "notes"),
        ("game/data/seeds/s1.json", "{}"),
        ("game/assets/ui/a.png", "png"),
        ("game/assets/audio/theme.ogg", "ogg"),
        ("engine/assets/fonts/f.ttf", "ttf"),
        ("engine/assets/audio/click.wav", "wav"),
        ("engine/crates/x/src/lib.rs", "code"),
    ] {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
}

fn manifest(dir: &Path, v: Value) -> std::path::PathBuf {
    let p = dir.join("game/web_bundle.json");
    std::fs::write(&p, v.to_string()).unwrap();
    p
}

fn keys(t: &Table) -> Vec<String> {
    t.iter().map(|(k, _)| k.to_string()).collect()
}

#[test]
fn the_manifest_names_the_parts_and_their_files() {
    let dir = Scratch::new("bundle_parts");
    tree(&dir);
    let m = manifest(&dir, json!({"root": "..", "name": "game", "exclude": ["*.md", "game/data/seeds"], "parts": [
        {"file": "game_1.pak", "include": ["game/game.json", "game/data", "game/assets", "engine/assets"], "exclude": ["game/assets/audio", "engine/assets/audio"]},
        {"file": "game_2.pak", "include": ["game/assets/audio", "engine/assets/audio"]}
    ]}));
    let (name, parts) = bundle::collect(&m).unwrap();
    assert_eq!(name, "game");
    assert_eq!(parts.iter().map(|p| p.0.as_str()).collect::<Vec<_>>(), ["game_1.pak", "game_2.pak"], "load order");
    assert_eq!(keys(&parts[0].1), ["engine/assets/fonts/f.ttf", "game/assets/ui/a.png", "game/data/catalog/units.json", "game/game.json"]);
    assert_eq!(keys(&parts[1].1), ["engine/assets/audio/click.wav", "game/assets/audio/theme.ogg"]);
    assert_eq!(parts[0].1.read(Path::new("game/game.json")).unwrap(), b"{}");

    let out = dir.join("out");
    let written = bundle::write(&m, &out).unwrap();
    for (w, (file, table)) in written.iter().zip(&parts) {
        let bytes = std::fs::read(out.join(file)).unwrap();
        assert_eq!(*w, Written { file: file.clone(), files: table.len(), bytes: bytes.len() });
        assert_eq!(keys(&Table::from_pack(&bytes).unwrap()), keys(table), "the part reads back as the same files");
    }
    let index: Value = serde_json::from_str(&std::fs::read_to_string(out.join(bundle::INDEX)).unwrap()).unwrap();
    assert_eq!(index["name"], "game");
    assert_eq!(index["parts"][1], json!({"file": "game_2.pak", "files": 2, "bytes": written[1].bytes}));
}

#[test]
fn a_bad_manifest_stops_the_bundle() {
    let dir = Scratch::new("bundle_bad");
    tree(&dir);
    let err = |v: Value| bundle::collect(&manifest(&dir, v)).unwrap_err().to_string();
    assert!(err(json!({"root": "..", "parts": [{"file": "a.pak", "include": ["game/assets"]}, {"file": "b.pak", "include": ["game/assets/audio"]}]})).contains("in two parts"));
    assert!(err(json!({"root": "..", "parts": [{"file": "a.pak", "include": ["game/none"]}]})).contains("does not exist"));
    assert!(err(json!({"root": "..", "parts": [{"file": "a.pak", "include": ["game/data/seeds"], "exclude": ["game/data/seeds"]}]})).contains("no file"));
    assert!(err(json!({"root": "..", "parts": []})).contains("parts"));
}
