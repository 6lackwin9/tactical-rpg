//! The game's web bundle (`games/emberwake/web_bundle.json`, WEBREL1-G1) ships what the themed UI draws with: part 1
//! (fetched before the main menu) holds every theme file and every frame file (`ui/frames/<role>.png` with its
//! sidecar) of every asset layer, and each font a theme names, so the web build draws the same frames as the desktop.

use aegis_cli::bundle;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..").canonicalize().unwrap()
}

/// Every file under `dir` (relative to the repository, `/`-separated).
fn files_under(root: &Path, dir: &str, out: &mut Vec<String>) {
    let Ok(read) = std::fs::read_dir(root.join(dir)) else { return };
    for e in read.flatten() {
        let rel = format!("{dir}/{}", e.file_name().to_string_lossy());
        if e.path().is_dir() {
            files_under(root, &rel, out);
        } else if !rel.ends_with(".md") {
            out.push(rel);
        }
    }
}

#[test]
fn part_one_holds_every_theme_frame_and_font() {
    let root = repo();
    let (_, parts) = bundle::collect(&root.join("games/emberwake/web_bundle.json")).unwrap();
    let first = &parts[0].1;
    let mut wanted: Vec<String> = Vec::new();
    for layer in ["engine/assets", "kits/rpg/assets", "games/emberwake/assets"] {
        let theme = format!("{layer}/ui/theme.json");
        if root.join(&theme).is_file() {
            let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(&theme)).unwrap()).unwrap();
            for face in v["fonts"].as_object().into_iter().flat_map(|o| o.values()) {
                let file = face["file"].as_str().unwrap();
                let found = ["games/emberwake/assets", "kits/rpg/assets", "engine/assets"].iter().map(|l| format!("{l}/{file}")).find(|p| root.join(p).is_file());
                wanted.push(found.unwrap_or_else(|| panic!("{theme}: font {file} is in no layer")));
            }
            wanted.push(theme);
        }
        files_under(&root, &format!("{layer}/ui/frames"), &mut wanted);
    }
    assert!(wanted.iter().any(|f| f.ends_with("/ui/frames/panel.png")), "the game has frame art to check: {wanted:?}");
    let missing: Vec<&String> = wanted.iter().filter(|f| !first.is_file(Path::new(f.as_str()))).collect();
    assert!(missing.is_empty(), "not in {}: {missing:?}", parts[0].0);
}
