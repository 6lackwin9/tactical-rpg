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

/// The kit's saves (`rpgkit_world::save`) with Emberwake's renamed deck keys: `read`, `read_keyed` and `decode` here
/// load an older save that holds a deck key under its old name (LAYER-DOWN-1).
pub mod save {
    pub use rpgkit_world::save::*;

    use rpgkit_world::Data;
    use std::path::Path;

    /// LAYER-DOWN-1: PK1-S2a saved the channeled marks as the deck key `marrow`; the kit's key is `channeled`.
    pub const DECK_KEY_ALIASES: &[KeyAlias] = &[("marrow", "channeled")];

    /// `rpgkit_world::save::read` with `DECK_KEY_ALIASES`.
    pub fn read(d: &Data, dir: &Path, slot: i64) -> Result<Loaded, Vec<String>> {
        read_with(d, dir, slot, DECK_KEY_ALIASES)
    }

    /// `rpgkit_world::save::read_keyed` with `DECK_KEY_ALIASES`.
    pub fn read_keyed(d: &Data, dir: &Path, key: &str) -> Result<Loaded, Vec<String>> {
        read_keyed_with(d, dir, key, DECK_KEY_ALIASES)
    }

    /// `rpgkit_world::save::decode` with `DECK_KEY_ALIASES`.
    pub fn decode(d: &Data, text: &str) -> Result<Loaded, Vec<String>> {
        decode_with(d, text, DECK_KEY_ALIASES)
    }
}
