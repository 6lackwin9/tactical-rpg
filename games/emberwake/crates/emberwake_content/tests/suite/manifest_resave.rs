//! E2.2-S2 (FB-110 Option A): the shipped game manifest (`game.json`) re-saves byte for byte through the kit's manifest
//! writer (`store::set_manifest`, what `aegis content manifest` runs), and setting then removing a kit key
//! (`supplies_max`) gives the same bytes back. Works on a scratch copy (a workspace holding `engine/assets/` and the
//! kit's manifest), never on the shipped file.

use emberwake_content::source::PurseCaps;
use emberwake_content::store;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn game_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_shipped_manifest_resaves_byte_for_byte() {
    let shipped = fs::read_to_string(game_dir().join("game.json")).expect("game.json");
    let m = aegis_content::manifest::Manifest::read(&game_dir().join("game.json")).unwrap();
    let kit = m.kit.clone();
    let root = aegis_core::scratch::Scratch::vacant("emberwake_manifest_resave");
    fs::create_dir_all(root.join("engine/assets")).unwrap();
    if !kit.is_empty() {
        let kit_json = game_dir().join("../../kits").join(&kit).join("kit.json");
        fs::create_dir_all(root.join("kits").join(&kit)).unwrap();
        fs::copy(kit_json, root.join("kits").join(&kit).join("kit.json")).unwrap();
    }
    let dir = root.join("games").join(&m.id);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("game.json");
    fs::write(&path, &shipped).unwrap();

    let empty = serde_json::Map::new();
    store::set_manifest(&path, &empty).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), shipped, "an empty patch re-saves the shipped bytes");

    let caps = store::set_manifest(&path, json!({"supplies_max": PurseCaps::SUPPLIES_MAX}).as_object().unwrap()).unwrap();
    assert_eq!(caps.extra.get("supplies_max"), Some(&json!(PurseCaps::SUPPLIES_MAX)));
    assert_ne!(fs::read_to_string(&path).unwrap(), shipped);
    store::set_manifest(&path, json!({"supplies_max": null}).as_object().unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), shipped, "set then removed: the shipped bytes again");
}
