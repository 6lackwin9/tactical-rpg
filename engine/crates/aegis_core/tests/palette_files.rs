//! ASSETCHK1: every palette file of every asset layer is parsed with the loader's rules (`assets::parse_palette`),
//! referenced or not; a file that fails is reported with its path and the reason. Scratch layers under the temp dir.

use aegis_core::assets::{self, Layers};
use std::path::Path;

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn scratch(name: &str) -> aegis_core::scratch::Scratch {
    let root = aegis_core::scratch::Scratch::vacant(&format!("palette_files_{name}"));
    root
}

fn shown(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

#[test]
fn good_palettes_pass() {
    let root = scratch("good");
    write(&root.join("units/warrior/base/palettes/red.json"), r##"{"slots": {"main": "#ff0000", "trim": "#E0B43C"}}"##);
    write(&root.join("units/warrior/goblin/palettes/blue.json"), r##"{"slots": {"main": "#2040c0"}}"##);
    write(&root.join("units/warrior/goblin/palettes/notes.txt"), "not a palette");
    assert_eq!(assets::palette_problems(&Layers::of(vec![root.to_path_buf()])), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_hex_value_without_its_hash_is_reported_with_its_path() {
    let root = scratch("hash");
    let file = root.join("units/warrior/base/palettes/red.json");
    write(&file, r##"{"slots": {"main": "ff0000"}}"##);
    let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    let reason = assets::parse_palette(&raw).unwrap_err();
    let want: Vec<String> = reason.iter().map(|r| format!("{}: {r}", shown(&file))).collect();
    assert_eq!(assets::palette_problems(&Layers::of(vec![root.to_path_buf()])), want, "the loader's reason, prefixed by the path");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unreferenced_bad_palettes_in_every_layer_are_reported_in_order() {
    let game = scratch("game");
    let engine = scratch("engine");
    // No clips.json anywhere: nothing is a body, so nothing references these files.
    write(&game.join("units/rogue/base/palettes/zeta.json"), r##"{"slots": {"main": "12345"}}"##);
    write(&game.join("units/rogue/base/palettes/alpha.json"), r##"{"main": "#123456"}"##);
    write(&game.join("units/brawler/goblin/palettes/torn.json"), r##"{"slots": {"main": "#1"##);
    write(&engine.join("units/rogue/base/palettes/zeta.json"), r##"{"slots": {"main": "#12345g"}}"##);
    write(&engine.join("units/rogue/base/palettes/fine.json"), r##"{"slots": {"main": "#123456"}}"##);
    let layers = Layers::of(vec![game.to_path_buf(), engine.to_path_buf()]);
    assert!(layers.bodies().is_empty(), "no body references the palettes");
    let problems = assets::palette_problems(&layers);
    let files: Vec<String> = problems.iter().map(|p| p.split(".json: ").next().unwrap().to_string() + ".json").collect();
    let want: Vec<String> = [
        game.join("units/brawler/goblin/palettes/torn.json"),
        game.join("units/rogue/base/palettes/alpha.json"),
        game.join("units/rogue/base/palettes/zeta.json"),
        engine.join("units/rogue/base/palettes/zeta.json"),
    ]
    .iter()
    .map(|p| shown(p))
    .collect();
    assert_eq!(files, want, "{problems:?}");
    assert!(problems[0].contains(": not JSON ("), "{}", problems[0]);
    let reason = |text: &str| assets::parse_palette(&serde_json::from_str(text).unwrap()).unwrap_err();
    assert_eq!(problems[1..3], [format!("{}: {}", want[1], reason(r##"{"main": "#123456"}"##)[0]), format!("{}: {}", want[2], reason(r##"{"slots": {"main": "12345"}}"##)[0])]);
    let _ = std::fs::remove_dir_all(&game);
    let _ = std::fs::remove_dir_all(&engine);
}
