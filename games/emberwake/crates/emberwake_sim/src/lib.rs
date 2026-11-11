//! The game's sim crate: a façade over the RPG kit crate rpgkit_sim (kits/rpg/crates/rpgkit_sim, KIT1). KIT1-SIM: the
//! kit names no world; `game()` tells it Emberwake's default world and story run, and `parse` and `run` pass them.
//! PR5-33 (LAYER-UP-DATA): that description is data, `games/emberwake/sim.json` (the default world, the story run and
//! its steel choice, the `feel` encounters, the classes, roles, role loadouts, generated rosters and role limit tests),
//! read by the kit (`rpgkit_sim::config`); this crate names no world, encounter, class or skill id.
pub use rpgkit_sim::*;

use emberwake_world::Data;
use std::sync::OnceLock;

/// The sim description file (embedded at build time, so every sim run and test reads the same one).
pub const CONFIG: &str = include_str!("../../../sim.json");

/// Emberwake's sim description (`CONFIG` read once per process by `rpgkit_sim::config::game_from_json`).
pub fn game() -> SimGame {
    static GAME: OnceLock<SimGame> = OnceLock::new();
    GAME.get_or_init(|| config::game_from_json(CONFIG).unwrap_or_else(|e| panic!("games/emberwake/sim.json: {e}"))).clone()
}

/// Parses the sim command line `argv` (mode first) on `d` for Emberwake: `world=` defaults to the game's default world
/// and the campaign mode plays its story run. Err = the message after "SIM argument error: ".
pub fn parse(d: &Data, argv: &[String]) -> Result<args::SimArgs, String> {
    args::SimArgs::parse_for(d, argv, &game())
}

/// Runs the sim command line `argv` (mode first, as `tools/fc.sh sim` takes it) on `d` for Emberwake; the printed
/// lines.
pub fn run(d: &Data, argv: &[String]) -> Vec<String> {
    rpgkit_sim::run(d, argv, &game())
}
