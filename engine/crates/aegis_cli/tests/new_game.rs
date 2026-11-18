//! NEW-GAME1 (ENGINE-6): `aegis new game`. The files it writes and the refusals; that a made game builds, passes its
//! tests and plays its scenario is `tools/fc.sh newgame` (in the gate).

use aegis_cli::new_game;
use aegis_core::scratch::Scratch;

#[test]
fn an_id_is_lower_snake_case_and_not_an_engine_name() {
    for ok in ["star_forge", "ab", "g2"] {
        assert_eq!(new_game::id_problem(ok), None, "{ok}");
    }
    for bad in ["a", "Star", "2d", "x-y", "x_", "x__y", "aegis", "kit", &"a".repeat(33)] {
        assert!(new_game::id_problem(bad).is_some(), "{bad}");
    }
    assert_eq!(new_game::type_name("star_forge"), "StarForge");
    assert_eq!(new_game::default_title("star_forge"), "Star Forge");
}

#[test]
fn the_files_carry_no_placeholder_and_name_the_game() {
    let files = new_game::files("star_forge", "Star Forge");
    let paths: Vec<String> = files.iter().map(|(p, _)| p.to_string_lossy().replace('\\', "/")).collect();
    for want in ["game.json", ".gitignore", "assets/.gitkeep", "scenarios/star_forge_start.json", "crates/star_forge_game/Cargo.toml", "crates/star_forge_game/src/lib.rs", "crates/star_forge_game/src/bin/star_forge_cli.rs", "crates/star_forge_game/tests/star_forge.rs"] {
        assert!(paths.iter().any(|p| p == want), "{want} in {paths:?}");
    }
    for (p, text) in &files {
        assert!(!text.contains("__ID__") && !text.contains("__TITLE__") && !text.contains("__TYPE__"), "{p:?} keeps a placeholder");
        assert!(!p.to_string_lossy().contains("__"), "{p:?}");
    }
    let manifest: serde_json::Value = serde_json::from_str(&files.iter().find(|(p, _)| p.ends_with("game.json")).unwrap().1).unwrap();
    assert_eq!((manifest["id"].as_str(), manifest["title"].as_str(), manifest["kit"].as_str()), (Some("star_forge"), Some("Star Forge"), Some("")));
    let lib = &files.iter().find(|(p, _)| p.ends_with("lib.rs")).unwrap().1;
    assert!(lib.contains("pub struct StarForge") && lib.contains("pub const GAME_ID: &str = \"star_forge\";"));
    let toml = &files.iter().find(|(p, _)| p.ends_with("Cargo.toml")).unwrap().1;
    let deps: Vec<&str> = toml.lines().filter_map(|l| l.split_once("{ path = \"").map(|(_, p)| p.trim_end_matches(['"', '}', ' ']))).collect();
    assert!(!deps.is_empty() && deps.iter().all(|p| p.starts_with("../../../../engine/crates/aegis_")), "engine crates only: {deps:?}");
}

#[test]
fn it_writes_the_folder_once_and_refuses_what_it_cannot_make() {
    let t = Scratch::new("new_game");
    let root = t.path();
    assert!(new_game::create(root, "star_forge", "Star Forge").unwrap_err().to_string().contains("not the workspace root"));
    std::fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
    std::fs::create_dir_all(root.join("engine/crates")).unwrap();
    let written = new_game::create(root, "star_forge", "Star Forge").unwrap();
    assert_eq!(written.len(), new_game::files("star_forge", "Star Forge").len());
    assert!(root.join("games/star_forge/crates/star_forge_game/src/screens.rs").is_file());
    let scenarios = aegis_cli::scenario::files_in(&root.join("games/star_forge/scenarios"));
    assert_eq!(aegis_cli::scenario::check_files(&scenarios), Vec::<String>::new(), "the scenario file is valid");
    assert!(new_game::create(root, "star_forge", "Star Forge").unwrap_err().to_string().contains("exists already"));
    assert!(new_game::create(root, "Bad", "x").is_err());
    assert!(new_game::create(root, "quoted", "a \"b\"").unwrap_err().to_string().contains("no quote"));
    assert_eq!(new_game::find_root(&root.join("games/star_forge/crates")).as_deref(), Some(root));
}
