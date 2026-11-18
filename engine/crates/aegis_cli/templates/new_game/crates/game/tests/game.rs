//! Made by `aegis new game __ID__`: the game folder, its scenarios and the command host. The screens are walked end to
//! end by the scenario `__ID___start` (`__ID___cli scenario run all`).

use __ID___game::GAME_ID;
use serde_json::json;
use std::path::Path;

fn game_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_game_folder_holds_its_manifest_and_valid_scenarios() {
    let g = game_dir();
    let m = aegis_content::manifest::Manifest::read(&g.join("game.json")).unwrap();
    assert_eq!(m.id, GAME_ID);
    let files = aegis_cli::scenario::files_in(&g.join("scenarios"));
    assert!(!files.is_empty());
    let problems = aegis_cli::scenario::check_files(&files);
    assert!(problems.is_empty(), "{problems:#?}");
    let layers = aegis_core::assets::Layers::for_game(&g);
    for rel in [__ID___game::sound::PRESS, __ID___game::sound::BACK] {
        assert!(layers.has(&format!("audio/{rel}")), "{rel}");
    }
}

#[test]
fn the_command_host_starts_a_game_and_replays_it() {
    use aegis_host::{Driver, Host};
    let mut h = Host::new(__ID___game::Commands::default());
    assert_eq!(h.send(&json!({"cmd": "state"}))["started"], json!(false));
    assert_eq!(h.send(&json!({"cmd": "new"}))["started"], json!(true));
    assert_eq!(h.history().len(), 1);
    assert_eq!(h.send(&json!({"cmd": "fly"}))["ok"], json!(false));
}
