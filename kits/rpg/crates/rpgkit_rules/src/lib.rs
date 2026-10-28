//! Pure game rules ported from the Godot reference: stats, gear, unit building and the damage formula.
//! No I/O; every function is checked against Godot output by `tests/parity.rs`.

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
