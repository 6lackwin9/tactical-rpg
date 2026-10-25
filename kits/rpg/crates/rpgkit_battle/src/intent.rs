//! Enemy intent tells (IntentRules, scripts/combat/session/intent_rules.gd; INTENT1, decision combat "DR1"): a pure
//! read-out of the enemies' scheduled actions for the planning screen. Enemy plans are drafted before the party plans
//! and never re-planned, so a tell always names the plan that resolves. Display only: nothing here changes planning,
//! AI drafts, resolution or the sim.
//!
//! One record per active enemy on the grid (`intents`): the kind of its first scheduled action (with none: idle,
//! "Holding", for an enemy that was on the grid when the round's enemies were scheduled, else arriving, a wave unit
//! placed later), its target (attack and support targets, the approach target of an Engage move) or cell (an area's
//! aimed tile, the goal of a move without a target), the blocks and the skill; revealed when a foresight holder is
//! targeted by any of its actions this round or the enemy is enraged (PACE1), and then its whole plan.

use crate::command::{CmdKind, Command};
use crate::grid::{V2, NO_CELL};
use crate::resolver::ScheduledAction;
use crate::world::{UnitId, World};

pub const ARRIVING: &str = "arriving";
pub const IDLE: &str = "idle";
/// Every kind a record can carry (the screen's chips map these; "act" is any other command).
pub const KINDS: [&str; 14] = ["attack", "area", "support", "guard", "focus", "rest", "retreat", "move", "flee", "switch", "stunned", "act", ARRIVING, IDLE];

/// One plan line: {kind, target, cell, blocks, skill}.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub kind: &'static str,
    pub target: Option<UnitId>,
    /// NO_CELL when the line aims at no tile.
    pub cell: V2,
    /// (start block, end block); (0, 0) when arriving or idle.
    pub blocks: (i32, i32),
    /// The action's display name ("" when arriving or idle).
    pub skill: String,
}

/// The record of one enemy.
#[derive(Debug, Clone, PartialEq)]
pub struct Intent {
    pub unit: UnitId,
    pub line: Line,
    pub revealed: bool,
    pub enraged: bool,
    /// The foresight holders it targets (empty unless revealed by foresight).
    pub readers: Vec<UnitId>,
    /// Revealed: every scheduled action in block order; else empty.
    pub plan: Vec<Line>,
    /// MORALE1 rule 13: the state word and the group (never the value or the flee ratio); "" and -1 without morale.
    pub morale_state: &'static str,
    pub morale_group: i32,
}

/// The tell kind of one command.
pub fn kind_of(w: &World, cmd: &Command) -> &'static str {
    match &cmd.kind {
        CmdKind::Area { .. } => "area",
        CmdKind::Attack { .. } => "attack",
        CmdKind::Support { .. } => "support",
        CmdKind::Defend { .. } => "guard",
        CmdKind::Focus { .. } => "focus",
        CmdKind::Rest { .. } => "rest",
        CmdKind::Disengage { .. } => "retreat",
        CmdKind::Walk { goal, .. } => {
            if *goal != NO_CELL && w.grid.is_exit(*goal) {
                "flee"
            } else {
                "move"
            }
        }
        CmdKind::Target { .. } => "switch",
        CmdKind::StatusBlock { .. } => "stunned",
        _ => "act",
    }
}

/// The unit a command aims at: attack and support targets, the approach target of an Engage move.
pub fn target_of(cmd: &Command) -> Option<UnitId> {
    match &cmd.kind {
        CmdKind::Attack { target, .. } => *target,
        CmdKind::Support { target, .. } => Some(*target),
        CmdKind::Walk { approach, .. } => *approach,
        _ => None,
    }
}

/// The tile a command aims at: an area's aimed tile (its first cell when the aim is unknown), the goal of a move
/// without a target; else NO_CELL.
pub fn cell_of(cmd: &Command) -> V2 {
    match &cmd.kind {
        CmdKind::Area { cells, aim, .. } => {
            if *aim != NO_CELL {
                *aim
            } else {
                cells.first().copied().unwrap_or(NO_CELL)
            }
        }
        CmdKind::Walk { goal, approach, .. } if approach.is_none() => *goal,
        _ => NO_CELL,
    }
}

pub fn line_of(w: &World, a: &ScheduledAction) -> Line {
    Line { kind: kind_of(w, &a.command), target: target_of(&a.command), cell: cell_of(&a.command), blocks: (a.start, a.end), skill: a.command.name.clone() }
}

/// `unit`'s live (not cancelled) scheduled actions in block order.
pub fn actions_of(unit: UnitId, scheduled: &[ScheduledAction]) -> Vec<&ScheduledAction> {
    let mut out: Vec<&ScheduledAction> = scheduled.iter().filter(|a| a.unit == unit && !a.cancelled).collect();
    out.sort_by_key(|a| a.start);
    out
}

/// The action aims at `holder`: a single-target attack or support (an enemy's support on a hero is a debuff) with the
/// holder as target, or an area whose cells include the holder's tile.
pub fn targets_unit(a: &ScheduledAction, holder: UnitId, holder_cell: V2) -> bool {
    match &a.command.kind {
        CmdKind::Area { cells, .. } => holder_cell != NO_CELL && cells.contains(&holder_cell),
        CmdKind::Attack { .. } | CmdKind::Support { .. } => target_of(&a.command) == Some(holder),
        _ => false,
    }
}

/// The party's active foresight holders on the grid that any of `actions` targets (empty = not revealed).
pub fn readers_of(w: &World, actions: &[&ScheduledAction]) -> Vec<UnitId> {
    w.allies
        .iter()
        .copied()
        .filter(|&h| !w.units[h].foresight_source.is_empty() && w.units[h].is_active() && w.has(h))
        .filter(|&h| {
            let cell = w.pos(h);
            actions.iter().any(|a| targets_unit(a, h, cell))
        })
        .collect()
}

/// The record of one enemy (morale filled by the caller).
pub fn intent_of(w: &World, unit: UnitId, scheduled: &[ScheduledAction], on_grid_at_schedule: &[UnitId], enraged: bool) -> Intent {
    let actions = actions_of(unit, scheduled);
    if actions.is_empty() {
        let kind = if on_grid_at_schedule.contains(&unit) { IDLE } else { ARRIVING };
        return Intent { unit, line: Line { kind, target: None, cell: NO_CELL, blocks: (0, 0), skill: String::new() }, revealed: enraged, enraged, readers: Vec::new(), plan: Vec::new(), morale_state: "", morale_group: -1 };
    }
    let readers = readers_of(w, &actions);
    let revealed = !readers.is_empty() || enraged;
    let plan = if revealed { actions.iter().map(|a| line_of(w, a)).collect() } else { Vec::new() };
    Intent { unit, line: line_of(w, actions[0]), revealed, enraged, readers, plan, morale_state: "", morale_group: -1 }
}

/// One record per active enemy on the grid, in the enemies' order.
pub fn intents(w: &World, scheduled: &[ScheduledAction], on_grid_at_schedule: &[UnitId], enraged: &[UnitId]) -> Vec<Intent> {
    w.enemies.iter().copied().filter(|&e| w.units[e].is_active() && w.has(e)).map(|e| intent_of(w, e, scheduled, on_grid_at_schedule, enraged.contains(&e))).collect()
}

/// The tells in the trace shape the Godot recorder writes (parity/godot/trace.gd `intents_state`): per enemy
/// [name, line, revealed, enraged, [reader names], [plan lines], morale state, morale group]; a line is
/// [kind, target name or "", cell x, cell y, start block, end block, skill].
pub fn trace(w: &World, intents: &[Intent]) -> serde_json::Value {
    let name = |u: Option<UnitId>| u.map(|u| w.units[u].name.clone()).unwrap_or_default();
    let line = |l: &Line| serde_json::json!([l.kind, name(l.target), l.cell.x, l.cell.y, l.blocks.0, l.blocks.1, l.skill]);
    serde_json::Value::Array(
        intents
            .iter()
            .map(|i| {
                serde_json::json!([
                    w.units[i.unit].name,
                    line(&i.line),
                    i.revealed,
                    i.enraged,
                    i.readers.iter().map(|r| w.units[*r].name.clone()).collect::<Vec<_>>(),
                    i.plan.iter().map(line).collect::<Vec<_>>(),
                    i.morale_state,
                    i.morale_group
                ])
            })
            .collect(),
    )
}
