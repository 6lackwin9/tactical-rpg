//! ASSETORG2 asset checks A1-A6 (docs/specs/ASSETORG2.md section 6), the body options and the skipped enum check, on
//! the shipped data over a scratch asset layer built here (bodies, a short sheet, a bad clips.json, portraits, a palette).

use aegis_core::assets::Layers;
use aegis_core::json::{self, Dict};
use emberwake_content::check::{self, assets, View};
use emberwake_content::store;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// A file whose header says it is a `w` x `h` PNG (the checks read only the header).
fn png(path: &Path, w: u32, h: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut b = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    b.extend(w.to_be_bytes());
    b.extend(h.to_be_bytes());
    std::fs::write(path, b).unwrap();
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

const CLIPS: &str = r##"{"frame": [104, 104], "palette": {"main": "#a83028"}, "clips": [{"id": "idle", "frames": 1, "ms": [1000]}]}"##;

fn layer(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_asset_checks_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for body in ["brawler/base", "brawler/goblin", "brawler/short", "brawler/broken"] {
        let dir = root.join("units").join(body);
        png(&dir.join("sheet.png"), if body.ends_with("short") { 50 } else { 104 }, 104);
        write(&dir.join("clips.json"), if body.ends_with("broken") { r#"{"frame": [4, 4], "clips": []}"# } else { CLIPS });
    }
    png(&root.join("portraits/common/brawler_base.png"), 240, 240);
    png(&root.join("items/sword/greatsword.png"), 96, 96);
    write(&root.join("units/brawler/goblin/palettes/red.json"), r##"{"slots": {"main": "#ff0000"}}"##);
    write(&root.join("units/brawler/goblin/palettes/odd.json"), r##"{"slots": {"cape": "#ff0000"}}"##);
    root
}

fn view(root: &Path) -> View {
    let mut v = store::view(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
    v.assets = Layers::of(vec![root.to_path_buf()]);
    v
}

fn with(base: &Dict, fields: Value) -> Dict {
    let mut e = base.clone();
    for (k, x) in fields.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    e
}

#[test]
fn bodies_and_icons_are_checked_against_the_layers() {
    let root = layer("a1");
    let v = view(&root);
    assert!(assets::body_errors(&v, "brawler/goblin").is_empty() && assets::body_errors(&v, "").is_empty());
    assert_eq!(assets::body_errors(&v, "brawler/nope"), ["Art 'brawler/nope' isn't a body (units/<class>/<variant>/ in the game, kit or engine)."]);
    assert_eq!(assets::body_errors(&v, "brawler/short"), ["Body 'brawler/short': the sheet is 50x104; its clips need at least 104x104."]);
    let broken = assets::body_errors(&v, "brawler/broken");
    assert!(broken[0].starts_with("Body 'brawler/broken': frame must be [w, h], each 8..256."), "{broken:?}");
    let sword = json::obj([("art", json!(""))]);
    let item = |icon: &str| json::obj([("family", json!("sword")), ("icon_id", json!(icon))]);
    assert!(assets::icon_errors(&v, &item("greatsword"), &sword).is_empty());
    assert_eq!(assets::icon_errors(&v, &item("claymore"), &sword), ["Icon 'claymore' isn't in items/sword/ (game, kit or engine)."]);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn portraits_and_palettes_warn() {
    let root = layer("a3");
    let v = view(&root);
    let unit = json::obj([("id", json!("goblin_brawler")), ("name", json!("Goblin Brawler")), ("class_id", json!("brawler")), ("tier", json!("minion")), ("art", json!("brawler/goblin"))]);
    let skarn = json::obj([("id", json!("skarn_ironjaw")), ("name", json!("Skarn Ironjaw")), ("role", json!("enemy")), ("art", json!("")), ("portrait_tier", json!("elite"))]);
    let w = assets::portrait_and_palette_warnings(&v, &skarn, &unit);
    assert_eq!(w, ["Skarn Ironjaw (elite) has no portrait at portraits/elites/skarn_ironjaw.png; it shows the common portrait brawler_goblin."], "A3");
    let mob = with(&skarn, json!({"id": "ash_brawler", "name": "Ash Brawler", "portrait_tier": ""}));
    assert_eq!(assets::portrait_and_palette_warnings(&v, &mob, &unit), ["Body 'brawler/goblin' has no common portrait; Ash Brawler shows brawler_base."], "A5");
    let plain = with(&mob, json!({"art": "brawler"}));
    assert!(assets::portrait_and_palette_warnings(&v, &plain, &unit).is_empty(), "the class body's common portrait is there");
    let red = with(&plain, json!({"art": "brawler/goblin", "palette": "red"}));
    assert_eq!(assets::portrait_and_palette_warnings(&v, &red, &unit).len(), 1, "only A5: the red palette fits");
    let odd = with(&red, json!({"palette": "odd"}));
    assert!(assets::portrait_and_palette_warnings(&v, &odd, &unit).contains(&"Palette 'odd' of Ash Brawler: slot 'cape' is not in the body's palette.".to_string()), "A4");
    let gone = with(&red, json!({"palette": "blue"}));
    assert!(assets::portrait_and_palette_warnings(&v, &gone, &unit).contains(&"Palette 'blue' of Ash Brawler: no file units/brawler/goblin/palettes/blue.json.".to_string()), "A4");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn art_offers_the_bodies_and_skips_the_enum_check() {
    let root = layer("options");
    let v = view(&root);
    let field = json::obj([("key", json!("art")), ("type", json!("enum")), ("options_from", json!("body_inherit"))]);
    assert_eq!(check::options_of(&v, &field), ["", "brawler", "brawler/broken", "brawler/goblin", "brawler/short"]);
    assert!(!check::enum_checked(&field));
    let unit = with(v.get("units", "goblin_brawler"), json!({"art": "brawler/goblin"}));
    let errors = check::validate_entry(&v, "units", &unit, "goblin_brawler");
    assert!(!errors.iter().any(|e| e.contains("is not one of")), "{errors:?}");
    let class = with(v.get("classes", "brawler"), json!({"art": ""}));
    assert!(check::validate_entry(&v, "classes", &class, "brawler").iter().any(|e| e.starts_with("Art '' isn't a body")), "a class names its body");
    let _ = std::fs::remove_dir_all(&root);
}

/// ASSETCHK1: every palette file in the game's layers (game, kit, engine) parses with the loader's rules, whether a
/// unit references it or not.
#[test]
fn every_shipped_palette_file_parses() {
    let layers = Layers::for_game(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    assert!(!layers.roots.is_empty());
    assert_eq!(aegis_core::assets::palette_problems(&layers), Vec::<String>::new());
}
