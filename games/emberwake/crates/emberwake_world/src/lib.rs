//! The game's world crate: a façade over the RPG kit crate rpgkit_world (kits/rpg/crates/rpgkit_world, KIT1), plus
//! the game's id and its data-folder lookup.
pub use rpgkit_world::*;

use std::path::{Path, PathBuf};

/// The game's id (games/emberwake/game.json `id`): its folder, crate prefix and save folder.
pub const GAME_ID: &str = "emberwake";

/// The game's data/ folder (games/emberwake/data) found from `start` (the game folder, the repository, or a folder
/// inside either).
pub fn find_data_dir(start: &Path) -> Option<PathBuf> {
    Data::find_game_data_dir(start, GAME_ID)
}
