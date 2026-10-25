//! The game's sim crate: a façade over the RPG kit crate rpgkit_sim (kits/rpg/crates/rpgkit_sim, KIT1). KIT1-SIM: the
//! kit names no world; `game()` tells it Emberwake's default world and story run, and `parse` and `run` pass them.
pub use rpgkit_sim::*;

use emberwake_world::Data;

/// The Ashlands: the default world of `world=` and `extra=`, and the story the `campaign` mode plays.
pub const WORLD: &str = "ashlands";
/// The flag of the "bar the gate" choice that branches the story: arms = ashmaw_stand (full finale), gate =
/// ashmaw_stand_barred.
pub const STEEL_FLAG: &str = "ashlands.steel_to_gate";

/// Emberwake's sim description: the Ashlands as default world and story run, branched on the steel choice.
pub fn game() -> SimGame {
    SimGame {
        default_world: WORLD.into(),
        story: Some(StoryRun {
            world: WORLD.into(),
            branch_flag: STEEL_FLAG.into(),
            choice: "steel".into(),
            branches: vec![("arms".into(), false), ("gate".into(), true)],
            usage: "       campaign [level]   Ashlands story fights in quest order, HP carried over (level = party level, default 5): all 10 trios, both steel choices".into(),
            extra_example: "4:ash_raiders".into(),
        }),
    }
}

/// Parses the sim command line `argv` (mode first) on `d` for Emberwake: `world=` defaults to the Ashlands and the
/// campaign mode plays its story run. Err = the message after "SIM argument error: ".
pub fn parse(d: &Data, argv: &[String]) -> Result<args::SimArgs, String> {
    args::SimArgs::parse_for(d, argv, &game())
}

/// Runs the sim command line `argv` (mode first, as `tools/fc.sh sim` takes it) on `d` for Emberwake; the printed
/// lines.
pub fn run(d: &Data, argv: &[String]) -> Vec<String> {
    rpgkit_sim::run(d, argv, &game())
}
