//! The game loop around the battles, ported from the Godot reference (scripts/game_state, progression,
//! exploration, quests, dialogue, travel): saved state, new game and saves, exploration, quests, dialogue, shops,
//! travel, rest and the battle bridge. Static and headless like the originals: every rule takes the data and the
//! GameState and returns Godot's result dictionaries. Checked against Godot by `tests/journey.rs`.

pub mod battle;
pub mod data;
pub mod dialogue;
pub mod explore;
pub mod new_game;
pub mod progression;
pub mod purse;
pub mod quest;
pub mod rolls;
pub mod save;
pub mod shop;
pub mod skill_text;
pub mod state;
pub mod travel;

/// The game's id (games/emberwake/game.json `id`): its folder, crate prefix and save folder.
pub const GAME_ID: &str = "emberwake";

pub use data::Data;
pub use state::{GameState, PartyState, UnitState};
