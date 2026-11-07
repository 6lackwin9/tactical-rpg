//! Finding a game's data folder (KIT1-N): the kit's `Data::find_data_dir` takes the nearest game folder at or above
//! the start (game.json, data/catalog/, data/worlds/); `Data::find_game_data_dir` finds a named game from anywhere in
//! its repository. Scratch folders only.

use rpgkit_world::Data;
use std::fs;

/// A scratch tree `t/games/demo/` with game.json, data/catalog/ and data/worlds/, plus a folder inside the game.
fn scratch(tag: &str) -> aegis_core::scratch::Scratch {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_data_dir_{tag}"));
    let _ = fs::remove_dir_all(&t);
    let game = t.join("games").join("demo");
    fs::create_dir_all(game.join("data").join("catalog")).unwrap();
    fs::create_dir_all(game.join("data").join("worlds")).unwrap();
    fs::create_dir_all(game.join("crates").join("demo_world")).unwrap();
    fs::write(game.join("game.json"), "{\"id\": \"demo\"}").unwrap();
    t
}

#[test]
fn the_nearest_game_folder_gives_its_data_folder() {
    let t = scratch("near");
    let data = t.join("games").join("demo").join("data");
    assert_eq!(Data::find_data_dir(&t.join("games").join("demo")).as_deref(), Some(data.as_path()));
    assert_eq!(Data::find_data_dir(&t.join("games").join("demo").join("crates").join("demo_world")).as_deref(), Some(data.as_path()));
    assert_eq!(Data::find_data_dir(&t), None, "the repository root is no game folder");
    let _ = fs::remove_dir_all(&t);
}

#[test]
fn a_named_game_is_found_from_its_repository() {
    let t = scratch("named");
    let data = t.join("games").join("demo").join("data");
    assert_eq!(Data::find_game_data_dir(&t, "demo").as_deref(), Some(data.as_path()));
    assert_eq!(Data::find_game_data_dir(&data, "demo").as_deref(), Some(data.as_path()));
    assert_eq!(Data::find_game_data_dir(&t, "other"), None);
    let _ = fs::remove_dir_all(data.join("worlds"));
    assert_eq!(Data::find_game_data_dir(&t, "demo"), None, "a data folder needs catalog/ and worlds/");
    assert_eq!(Data::find_data_dir(&t.join("games").join("demo")), None);
    let _ = fs::remove_dir_all(&t);
}
