//! Pure game rules ported from the Godot reference: stats, gear, unit building and the damage formula.
//! No I/O; unit building and the damage formula are pinned by an engine-recorded unit and damage golden in the game's
//! rules façade tests (CLN-38; the Godot comparison is retired).

pub mod bands;
pub mod budget;
pub mod carry;
pub mod damage;
pub mod derived;
pub mod gear;
pub mod ruleset;
pub mod stats;
pub mod status;
pub mod unit;

pub use unit::{build_unit, build_unit_in, DeckRule, Stance, Unit};
