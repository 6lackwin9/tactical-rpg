//! Pure game rules ported from the Godot reference: stats, gear, unit building and the damage formula.
//! No I/O; unit building and the damage formula are pinned by an engine-recorded unit and damage golden in the game's
//! rules façade tests (CLN-38; the Godot comparison is retired).

pub mod bands;
pub mod budget;
pub mod carry;
pub mod damage;
pub mod derived;
pub mod gear;
pub mod grants;
pub mod pipeline;
pub mod pool;
pub mod ruleset;
pub mod stats;
pub mod status;
pub mod unit;
/// U2.2-S1: the unit budget and class shares (lives in rpgkit_data, so the class loader can compute its view).
pub use rpgkit_data::unit_budget;

pub use unit::{build_unit, build_unit_in, DeckRule, Stance, Unit};
