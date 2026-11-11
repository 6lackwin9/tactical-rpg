//! The game loop around the battles, ported from the Godot reference (scripts/game_state, progression,
//! exploration, quests, dialogue, travel): saved state, new game and saves, exploration, quests, dialogue, shops,
//! travel, rest and the battle bridge, and the term resolver (`terms`, TERMS1). Static and headless like the originals: every rule takes the data and the
//! GameState and returns Godot's result dictionaries. Checked against Godot by `tests/journey.rs`.

pub mod battle;
pub mod board;
pub mod campaign;
pub mod data;
pub mod dialogue;
pub mod explore;
pub mod mastery;
pub mod new_game;
pub mod party_ops;
pub mod progression;
pub mod purse;
pub mod quest;
pub mod recipe;
pub mod rolls;
pub mod roster;
pub mod save;
pub mod shop;
pub mod skill_text;
pub mod state;
pub mod story;
pub mod terms;
pub mod travel;
pub mod weapons;

pub use data::Data;
pub use state::{GameState, PartyState, UnitState};
