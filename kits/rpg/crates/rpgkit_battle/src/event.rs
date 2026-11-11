//! Typed battle events (PR5-32 `LAYER-UP-EVENTS`, audit 2026-11-10 2.2 item 2): engine-only, no Godot original. The
//! resolver's log stays prose (`Resolver::log`, which the sim and the goldens print byte for byte); alongside it the
//! resolver records one `BattleEvent` for each line whose outcome a screen reacts to (a hit and whether it crit or
//! dealt nothing, a miss, a dodge, a status landed or resisted, a guard raised or broken, a focus, a support, a move,
//! a poise break), keyed by the line's index in the log. Screens pick their sounds and impact effects from these
//! fields instead of reading words in the prose. Nothing here is serialized: no trace, save or golden reads it.

use crate::world::UnitId;

/// What one logged line did, in fields (the line's own words stay in `Resolver::log`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    /// An attack that struck (a single attack's `[Attack]` line or an area's): `crit` = a critical hit among its hits;
    /// `zero` = it took no HP in total.
    Hit { crit: bool, zero: bool },
    /// An attack that found nobody (its target out of combat, an area on empty tiles).
    Miss,
    /// ATTACKLOCK1: the target left the tile the attack committed to.
    Dodged,
    /// A no-damage attack's status roll: `status` the status id ("" = it rolls none), `landed` false when resisted.
    Status { status: String, landed: bool },
    /// A unit loses the block to `status` (stun, stagger).
    LostBlock { status: String },
    /// A Defend landed: its guard raised, or (`broken`) the guard was broken before it landed.
    Guard { broken: bool },
    /// A Focus landed.
    Focus,
    /// A Support skill landed.
    Support,
    /// A unit moved: a walk (cut short or a withdrawal too), a Disengage jump, or a dash or chase at a command's start.
    Move,
    /// RULESET1 P9: `unit` is broken (rule poise).
    Break { unit: UnitId },
}

/// One typed event: `line` is the index of its line in `Resolver::log`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleEvent {
    pub line: usize,
    pub kind: EventKind,
}

/// The events of the lines from `from` on (the lines one step added when `from` was the log's length before it).
pub fn since(events: &[BattleEvent], from: usize) -> impl Iterator<Item = &EventKind> {
    events.iter().filter(move |e| e.line >= from).map(|e| &e.kind)
}
