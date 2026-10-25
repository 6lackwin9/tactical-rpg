//! The game's host crate: a façade over the RPG kit's battle command set rpgkit_host (kits/rpg/crates/rpgkit_host,
//! KIT1), plus the game's own CLI commands (`cli`: `sim <mode>`, `battle`) and the `aegis` binary.
pub use rpgkit_host::*;

pub mod cli;
