//! PR5-41 (ACT1-DATA1): the HUD chips of the Act 1 world (`sunk_ward`) draw their own icons: every chip the HUD shows
//! in a new game (the currency and upkeep of `world.json`'s `purse`, each visible meter's `hud.icon` in
//! `meters.json`) names an icon, and that file loads through the game's asset layers at the HUD's icon size (no
//! drawn stand-in).

use aegis_ui::art::Art;
use emberwake_world::{campaign, new_game, Data};
use std::path::{Path, PathBuf};

const WORLD: &str = "sunk_ward";

/// The HUD's icon box (`hud.rs`, 14 px).
const ICON: i32 = 14;

fn game_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

#[test]
fn every_act1_hud_chip_loads_its_icon() {
    let dir = emberwake_world::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let d = Data::open(&dir).expect("data").0;
    let gs = new_game::create_for_world(&d, WORLD, 1).expect("a new sunk_ward game");
    let chips = campaign::hud(&d, &gs);
    let shown = d.merged_entries(WORLD, "meters").into_iter().filter(|m| m.get("visible").and_then(|v| v.as_bool()).unwrap_or(true)).count();
    assert!(shown > 0, "the world shows meters");
    assert_eq!(chips.len(), 2 + shown, "the purse's two chips and each visible meter's");
    let mut art = Art::new(Some(game_dir()));
    for chip in &chips {
        assert!(!chip.icon.is_empty(), "chip '{}' names an icon", chip.id);
        assert!(art.image(&chip.icon, ICON, ICON).is_some(), "chip '{}': {} loads from the game's asset layers", chip.id, chip.icon);
    }
}
