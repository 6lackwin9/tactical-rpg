//! ENGINE-1 (HELLO-GAME1): the sample game's rules, save, command host and files. The screens are walked end to end by
//! the scenario `hello_walk` (`tools/fc.sh gate`, step `scenarios_all`).

use aegis_core::scratch::Scratch;
use hello_game::walk::{self, Walk};
use hello_game::{save, GAME_ID};
use serde_json::json;
use std::path::Path;

#[test]
fn a_walk_steps_onto_free_tiles_and_bumps_into_walls_and_stones() {
    let mut w = Walk::default();
    assert_eq!((w.x, w.y, w.steps), (walk::START.0, walk::START.1, 0));
    assert!(w.step(-1, 0), "one step left is free");
    assert!(!w.step(-1, 0), "the outer ring is wall");
    assert_eq!((w.x, w.y, w.steps), (1, 5, 1), "a bump changes nothing");
    for _ in 0..4 {
        assert!(w.step(1, 0));
    }
    assert!(!w.step(1, 0), "(6, 5) is a stone");
    assert!(!w.step(0, 0) && !w.step(2, 0) && !w.step(0, -2), "a step is one tile");
    assert_eq!((w.x, w.y, w.steps), (5, 5, 5));
    for (x, y) in walk::STONES {
        assert!(walk::blocked(*x, *y));
        assert!((1..walk::COLS - 1).contains(x) && (1..walk::ROWS - 1).contains(y), "stones lie inside the walls");
    }
    assert!(!walk::blocked(walk::START.0, walk::START.1), "the start is free");
}

/// WORLD-PRIM1 (ENGINE-9, WP4): a walk to a tile takes the shortest free route (the engine's grid search), round the
/// stones, and refuses a stone or the wall.
#[test]
fn a_walk_to_a_tile_takes_the_shortest_free_route() {
    let w = Walk::default();
    assert_eq!(w.route(walk::START), Some(Vec::new()), "already there");
    assert_eq!(w.route((6, 4)), None, "a stone");
    assert_eq!(w.route((0, 0)), None, "the wall");
    let to = (8, 4);
    let steps = w.route(to).expect("free");
    assert_eq!(steps.len() as i64, (to.0 - w.x).abs() + (to.1 - w.y).abs() + 2, "round the stone column at x 6, two steps more than straight");
    let mut walker = w;
    for (dx, dy) in &steps {
        assert!(walker.step(*dx, *dy), "every step is onto a free tile");
    }
    assert_eq!((walker.x, walker.y), to);
}

#[test]
fn a_walk_reads_back_its_saved_form_and_refuses_a_broken_one() {
    let w = Walk { x: 4, y: 3, steps: 9 };
    assert_eq!(Walk::from_value(&w.to_value()), Ok(w));
    assert!(Walk::from_value(&json!({"x": 4, "y": 3})).unwrap_err().contains("steps"));
    assert!(Walk::from_value(&json!({"x": 6, "y": 4, "steps": 1})).unwrap_err().contains("not a free tile"));
    assert!(Walk::from_value(&json!({"x": 4, "y": 3, "steps": -1})).is_err());
    assert!(Walk::from_value(&json!({"x": 4.5, "y": 3, "steps": 1})).is_err());
}

#[test]
fn the_slot_round_trips_through_the_engine_save_framework() {
    let t = Scratch::new("hello_save");
    let dir = t.path().join("saves");
    assert!(!save::exists(&dir));
    assert!(save::read(&dir).is_err());
    let w = Walk { x: 7, y: 2, steps: 12 };
    assert_eq!(save::write(&dir, w), Vec::<String>::new());
    assert!(save::exists(&dir));
    assert_eq!(save::read(&dir), Ok(w));
    let text = std::fs::read_to_string(aegis_save::path_for(&dir, save::SLOT)).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!((envelope["format"].as_str(), envelope["version"].as_i64()), (Some("hello_save"), Some(1)));
    // A save whose walk stands on a stone is refused, not loaded.
    std::fs::write(aegis_save::path_for(&dir, save::SLOT), text.replace("\"x\": 7", "\"x\": 6").replace("\"y\": 2", "\"y\": 4")).unwrap();
    assert!(save::read(&dir).unwrap_err()[0].contains("broken"));
}

#[test]
fn the_command_host_records_moves_and_replays_them() {
    use aegis_host::{Driver, Host};
    let mut h = Host::new(walk::Commands::default());
    assert_eq!(h.send(&json!({"cmd": "new"}))["ok"], json!(true));
    assert_eq!(h.send(&json!({"cmd": "move", "dx": 1, "dy": 0}))["walk"], json!({"x": 3, "y": 5, "steps": 1}));
    let bump = h.send(&json!({"cmd": "move", "dx": 0, "dy": -9}));
    assert_eq!((bump["ok"].clone(), bump["error"].clone()), (json!(false), json!("blocked")));
    assert_eq!(h.send(&json!({"cmd": "move", "dx": 0, "dy": 1}))["ok"], json!(true));
    assert_eq!(h.history().len(), 3, "new and the two moves that went through");
    assert_eq!(h.checkpoints().last().unwrap()["state"], json!({"x": 3, "y": 6, "steps": 2}));
    let mut again = Host::new(walk::Commands::default());
    for req in h.history().to_vec() {
        again.send(&req);
    }
    assert_eq!(again.checkpoints(), h.checkpoints(), "the history replays the same walk");
    assert_eq!(h.send(&json!({"cmd": "fly"}))["ok"], json!(false));
}

fn game_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_game_folder_holds_its_manifest_sprite_and_scenarios() {
    let g = game_dir();
    let m = aegis_content::manifest::Manifest::read(&g.join("game.json")).unwrap();
    assert_eq!((m.id.as_str(), m.kit.as_str()), (GAME_ID, ""), "the sample game names no kit");
    assert!(g.join("assets").join(hello_game::screens::SPRITE).is_file());
    let files = aegis_cli::scenario::files_in(&g.join("scenarios"));
    assert!(!files.is_empty());
    let problems = aegis_cli::scenario::check_files(&files);
    assert!(problems.is_empty(), "{problems:#?}");
    // The sounds it plays are the engine's, found through the asset layers.
    let layers = aegis_core::assets::Layers::for_game(&g);
    for rel in [hello_game::sound::PRESS, hello_game::sound::BUMP, hello_game::sound::SAVED, hello_game::sound::BACK] {
        assert!(layers.has(&format!("audio/{rel}")), "{rel}");
    }
}

/// The point of the sample game: it builds on the engine alone. No dependency of its crate is a kit or another game.
#[test]
fn the_sample_game_depends_on_engine_crates_only() {
    let toml = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let paths: Vec<&str> = toml.lines().filter_map(|l| l.split_once("{ path = \"").map(|(_, p)| p.trim_end_matches(['"', '}', ' ']))).collect();
    assert!(!paths.is_empty());
    for p in paths {
        assert!(p.starts_with("../../../../engine/crates/aegis_"), "{p} is not an engine crate");
    }
}
