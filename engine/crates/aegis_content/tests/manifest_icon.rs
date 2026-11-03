//! ICON1: the manifest's optional `icon`: absent stays absent in the file, a path's form is checked.

use aegis_content::manifest::{Manifest, KEYS};
use serde_json::{json, Value};

fn manifest(icon: &str) -> Manifest {
    Manifest { id: "demo".into(), title: "Demo".into(), icon: icon.into(), ..Manifest::default() }
}

fn patch(v: Value) -> aegis_core::json::Dict {
    v.as_object().unwrap().clone()
}

#[test]
fn a_manifest_without_an_icon_writes_no_key() {
    assert!(!manifest("").to_dict().contains_key("icon"));
    assert!(!manifest("").to_dict().contains_key("canvas") && !manifest("").to_dict().contains_key("window_sizes"), "the same for the canvas and the window sizes");
    assert_eq!(Manifest::from_dict(&patch(json!({"id": "demo", "title": "Demo"}))).icon, "");
    assert!(manifest("").problems().is_empty());
}

#[test]
fn the_icon_keeps_its_place_and_round_trips() {
    let m = Manifest { canvas: json!({"policy": "fixed"}), window_sizes: json!(["1280x720"]), ..manifest("ui/mark.png") };
    let d = m.to_dict();
    let back = Manifest::from_dict(&d);
    assert_eq!((back.icon.as_str(), &back.canvas, &back.window_sizes), ("ui/mark.png", &m.canvas, &m.window_sizes));
    let order: Vec<&str> = d.keys().map(String::as_str).collect();
    assert_eq!(order, KEYS, "the file holds the keys in KEYS order");
}

#[test]
fn set_keeps_the_canvas_and_window_sizes_as_written_and_null_removes_them() {
    let dir = std::env::temp_dir().join(format!("aegis_manifest_canvas_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("game.json");
    Manifest::set(&path, &patch(json!({"id": "demo", "title": "Demo"}))).unwrap();
    assert!(!std::fs::read_to_string(&path).unwrap().contains("canvas"), "absent stays absent");
    let (canvas, sizes) = (json!({"policy": "fit", "w": 1280, "h": 720}), json!(["1280x720", "1920x1080"]));
    Manifest::set(&path, &patch(json!({"canvas": canvas, "window_sizes": sizes}))).unwrap();
    let back = Manifest::read(&path).unwrap();
    assert_eq!((&back.canvas, &back.window_sizes), (&canvas, &sizes));
    assert_eq!(back.canvas["w"].as_i64(), Some(1280), "a whole number stays one, as the shell reads it");
    Manifest::set(&path, &patch(json!({"canvas": null, "window_sizes": null}))).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(!text.contains("canvas") && !text.contains("window_sizes"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn only_a_png_path_inside_the_layers_is_an_icon() {
    assert!(manifest("ui/mark.png").problems().is_empty());
    for bad in ["../outside.png", "ui/../../outside.png", "ui\\..\\outside.png", "ui/mark.ico", "/ui/mark.png", "\\ui\\mark.png", "C:/ui/mark.png", "C:mark.png"] {
        assert_eq!(manifest(bad).problems().len(), 1, "{bad}");
    }
}

#[test]
fn set_writes_the_icon_and_empty_removes_it() {
    let dir = std::env::temp_dir().join(format!("aegis_manifest_icon_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("game.json");
    Manifest::set(&path, &patch(json!({"id": "demo", "title": "Demo"}))).unwrap();
    let without = std::fs::read_to_string(&path).unwrap();
    assert!(!without.contains("icon"));
    assert_eq!(Manifest::set(&path, &patch(json!({"icon": "ui/mark.png"}))).unwrap().icon, "ui/mark.png");
    assert!(std::fs::read_to_string(&path).unwrap().contains("\"icon\": \"ui/mark.png\""));
    assert!(Manifest::set(&path, &patch(json!({"icon": "../mark.png"}))).is_err(), "a bad path is refused");
    Manifest::set(&path, &patch(json!({"icon": ""}))).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), without, "removing the icon restores the file");
    let _ = std::fs::remove_dir_all(dir);
}
