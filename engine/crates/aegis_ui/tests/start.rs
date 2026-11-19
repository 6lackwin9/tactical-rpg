//! ENGINE-11 (GAME-START1): the engine's game start. A game folder's manifest, layers, saves and options come out the
//! same for every game; a missing manifest is a notice and the defaults; the manifest names the game's CLI.

use aegis_content::manifest::Manifest;
use aegis_core::scratch::Scratch;
use aegis_ui::start::{arg, Start, FONT};
use std::path::Path;

fn args(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[test]
fn a_game_folder_gives_the_manifest_the_options_and_the_folders() {
    let t = Scratch::new("start_game");
    std::fs::write(t.path().join("game.json"), r#"{"id": "salve", "title": "Salve", "subtitle": "", "icon": "ui/i.png", "window_sizes": ["1280x720"], "version": "0.1.0", "kit": "", "engine": ">=0.1", "worlds": [], "start_worlds": [], "references": []}"#).unwrap();
    let s = Start::new("salve", Some(t.path_buf()), Path::new("eng"), &args(&[("port", "47811"), ("saves", "my_saves")]), None);
    assert!(s.notice.is_empty(), "{:?}", s.notice);
    assert_eq!((s.manifest.title.as_str(), s.options.title.as_str()), ("Salve", "Aegis - Salve"));
    assert_eq!((s.options.font.clone(), s.options.port, s.options.icon.as_str()), (Path::new("eng").join(FONT), 47811, "ui/i.png"));
    assert_eq!(s.options.window_sizes, serde_json::json!(["1280x720"]));
    assert_eq!((s.save_dir.as_path(), s.shot_dir.clone()), (Path::new("my_saves"), t.path().join(".shots")));
    assert_eq!(s.manifest.cli_name(), "salve_cli", "no cli key: <id>_cli");
    let d = Start::new("salve", Some(t.path_buf()), Path::new("eng"), &[], None);
    assert_eq!(d.options.port, aegis_ui::bridge::DEFAULT_PORT);
    assert!(d.save_dir.ends_with("saves"));
    assert_eq!(arg(&args(&[("a", "1")]), "a"), Some("1"));
}

#[test]
fn no_game_folder_is_a_notice_and_the_defaults() {
    let s = Start::new("salve", None, Path::new("eng"), &[], None);
    assert_eq!(s.notice, ["Game folder games/salve (game.json) not found."]);
    assert_eq!((s.manifest.id.as_str(), s.options.title.as_str()), ("salve", "Aegis - salve"));
    let t = Scratch::new("start_broken");
    std::fs::write(t.path().join("game.json"), "[1]").unwrap();
    assert_eq!(Start::new("salve", Some(t.path_buf()), Path::new("eng"), &[], None).notice.len(), 1, "a broken manifest is a notice");
}

#[test]
fn the_manifest_names_the_game_cli() {
    let m = Manifest { id: "salve".into(), cli: "aegis".into(), title: "S".into(), ..Manifest::default() };
    assert_eq!(m.cli_name(), "aegis");
    assert_eq!(m.to_dict()["cli"], serde_json::json!("aegis"));
    assert!(Manifest { cli: "a b".into(), ..m.clone() }.problems().iter().any(|p| p.contains("cli 'a b'")));
    assert!(!Manifest { cli: String::new(), ..m }.to_dict().contains_key("cli"), "no key when it names none");
}
