//! Pure game rules ported from the Godot reference: stats, gear, unit building and the damage formula.
//! No I/O; every function is checked against Godot output by `tests/parity.rs`.

pub mod damage;
pub mod gear;
pub mod ghash;
pub mod rng;
pub mod sort;
pub mod stats;
pub mod status;
pub mod unit;

pub use unit::{build_unit, Stance, Unit};
