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
//!
//! RULESET1 section 10 (rule `plan_badges`; engine-only, no Godot original): each record also carries its unit's round
//! intent, round target and Maneuver kind (V1, V5, intent UI5) and an impact badge per live action but a move or Rest
//! (V1: block and coarse kind only); the battle's `plan_reveal` reveals every enemy (V2) and a foresight `range` reveals
//! every enemy within it (V4); `round_intents` adds party records from their plans (V5). With the rule off (classic)
//! none of it is filled and the trace keeps the Godot shape.
//!
//! HOOKS1 AH3 (rule `arena_hazards`; engine-only): `arena_tells` adds the round's scheduled arena hazards (cells and
//! blocks), known from the round's start; none in classic.

use crate::command::{CmdKind, Command};
use crate::grid::{V2, NO_CELL};
use crate::plan::Plan;
use crate::resolver::ScheduledAction;
use crate::round_intent::{self, ManeuverKind, RoundIntent, RoundTarget};
use crate::world::{UnitId, World};
use rpgkit_rules::ruleset;

pub const ARRIVING: &str = "arriving";
pub const IDLE: &str = "idle";
/// RULESET1 V1: the impact badge kinds (TQ7: coarse, no skill, target or start): an Attack or Area lands, or anything
/// else but a move or Rest.
pub const HIT: &str = "hit";
pub const OTHER: &str = "other";
/// Every kind a record can carry (the screen's chips map these; "act" is any other command).
pub const KINDS: [&str; 14] = ["attack", "area", "support", "guard", "focus", "rest", "retreat", "move", "flee", "switch", "stunned", "act", ARRIVING, IDLE];

/// One plan line: {kind, target, cell, blocks, skill}.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub kind: &'static str,
    pub target: Option<UnitId>,
    /// NO_CELL when the line aims at no tile.
    pub cell: V2,
    /// (start block, impact block: where it lands); (0, 0) when arriving or idle. Classic: the impact is the end.
    pub blocks: (i32, i32),
    /// RULESET1 T1: the recovery blocks booked after the impact (0 in classic; not in the trace).
    pub recovery: i32,
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
    /// RULESET1 V5 (`plan_badges` with `round_intent`): the unit's round intent (None without one or with a rule off).
    pub round_intent: Option<RoundIntent>,
    /// The round target: the set one, else the first skill or item command's, else the lock (None as above).
    pub round_target: Option<RoundTarget>,
    /// The kind of a Maneuver round (None for the other intents).
    pub maneuver: Option<ManeuverKind>,
    /// RULESET1 V1 (`plan_badges`): (impact block, `HIT` or `OTHER`) of every live action but a move or Rest, in block
    /// order.
    pub impacts: Vec<(i32, &'static str)>,
}

/// Whether the battle shows impact badges and the round intent in tells (rule `plan_badges`; never in classic).
pub fn badges_on(w: &World) -> bool {
    w.ruleset.has(ruleset::PLAN_BADGES)
}

/// The id of a Maneuver kind ("engage", "disengage", "utility").
pub fn maneuver_id(k: ManeuverKind) -> &'static str {
    match k {
        ManeuverKind::Engage => "engage",
        ManeuverKind::Disengage => "disengage",
        ManeuverKind::Utility => "utility",
    }
}

/// RULESET1 V1: the badge kind of a command: None for a move (walks and flights) and for Rest (user 2026-10-26: "No
/// badge on Rest"), `HIT` for an Attack or Area, else `OTHER` (Defend, Disengage, status blocks).
pub fn badge_kind(w: &World, cmd: &Command) -> Option<&'static str> {
    match kind_of(w, cmd) {
        "move" | "flee" | "rest" => None,
        "attack" | "area" => Some(HIT),
        _ => Some(OTHER),
    }
}

/// RULESET1 V1: the impact badges of `actions` (live, block order).
pub fn impacts_of(w: &World, actions: &[&ScheduledAction]) -> Vec<(i32, &'static str)> {
    actions.iter().filter_map(|a| badge_kind(w, &a.command).map(|k| (a.impact, k))).collect()
}

/// RULESET1_INTENT TG1: the round target a command acts on (attacks: their unit; areas: their tile; support and items:
/// their unit; Defend and Focus: the user); moves and the rest none.
pub fn command_target(c: UnitId, cmd: &Command) -> Option<RoundTarget> {
    let t = match &cmd.kind {
        CmdKind::Attack { target: Some(t), .. } => RoundTarget::Unit(*t),
        CmdKind::Area { .. } => {
            let p = cell_of(cmd);
            if p == NO_CELL {
                return None;
            }
            RoundTarget::Cell(p)
        }
        CmdKind::Support { target, .. } | CmdKind::Item { target, .. } => RoundTarget::Unit(*target),
        CmdKind::Defend { .. } | CmdKind::Focus { .. } => RoundTarget::Myself,
        _ => return None,
    };
    Some(round_intent::normalize(c, t))
}

/// RULESET1 V5: fills a record's round intent, round target and Maneuver kind (under `plan_badges` and
/// `round_intent`) and its impact badges (under `plan_badges`). `planned` is the unit's round target from its plan
/// (party records), `cmds` its commands in block order.
fn fill_round(w: &World, rec: &mut Intent, planned: Option<RoundTarget>, cmds: &[&Command], impacts: Vec<(i32, &'static str)>) {
    if !badges_on(w) {
        return;
    }
    rec.impacts = impacts;
    if !round_intent::on(w) {
        return;
    }
    let c = rec.unit;
    let live = |t: UnitId| w.units.get(t).is_some_and(|u| u.is_active()) && w.has(t);
    let set = w.round_states.get(&c).and_then(|r| r.target);
    rec.round_target = set.or(planned).or_else(|| cmds.iter().find_map(|cmd| command_target(c, cmd))).or_else(|| w.units[c].active_target.filter(|&t| live(t)).map(RoundTarget::Unit)).map(|t| round_intent::normalize(c, t));
    rec.round_intent = round_intent::intent_of(w, c);
    rec.maneuver = (rec.round_intent == Some(RoundIntent::Maneuver)).then(|| round_intent::maneuver_of(w, c));
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
    Line { kind: kind_of(w, &a.command), target: target_of(&a.command), cell: cell_of(&a.command), blocks: (a.start, a.impact), recovery: a.end - a.impact, skill: a.command.name.clone() }
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

/// RULESET1 V4: the party's active foresight holders on the grid with a `range` that covers `unit`'s tile now (every
/// enemy within N tiles; only under `plan_badges`). The tells read it as measured when the round's enemy plan was
/// scheduled (`range_reveals`), so the round's playback (units moving, holders falling) never changes a reveal.
pub fn range_readers(w: &World, unit: UnitId) -> Vec<UnitId> {
    if !badges_on(w) || !w.has(unit) {
        return Vec::new();
    }
    w.allies
        .iter()
        .copied()
        .filter(|&h| {
            let u = &w.units[h];
            !u.foresight_source.is_empty() && u.foresight_range > 0 && u.is_active() && w.has(h) && w.dist(h, unit) <= u.foresight_range
        })
        .collect()
}

/// RULESET1 V4 at scheduling: each active enemy on the grid that a foresight range covers now, with its range readers
/// (empty without `plan_badges` or a holder with a range, so classic measures nothing).
pub fn range_reveals(w: &World) -> Vec<(UnitId, Vec<UnitId>)> {
    w.enemies.iter().copied().filter(|&e| w.units[e].is_active()).map(|e| (e, range_readers(w, e))).filter(|(_, r)| !r.is_empty()).collect()
}

/// The record of one enemy (morale filled by the caller). `ranged`: the range reveals measured when the round's enemy
/// plan was scheduled (`range_reveals`).
pub fn intent_of(w: &World, unit: UnitId, scheduled: &[ScheduledAction], on_grid_at_schedule: &[UnitId], ranged: &[(UnitId, Vec<UnitId>)], enraged: bool) -> Intent {
    let actions = actions_of(unit, scheduled);
    // RULESET1 V2: the battle's plan_reveal shows every enemy (plan_badges only)
    let reveal_all = badges_on(w) && w.plan_reveal;
    let mut rec = if actions.is_empty() {
        let kind = if on_grid_at_schedule.contains(&unit) { IDLE } else { ARRIVING };
        Intent { unit, line: Line { kind, target: None, cell: NO_CELL, blocks: (0, 0), recovery: 0, skill: String::new() }, revealed: enraged || reveal_all, enraged, readers: Vec::new(), plan: Vec::new(), morale_state: "", morale_group: -1, round_intent: None, round_target: None, maneuver: None, impacts: Vec::new() }
    } else {
        // RULESET1 V4: a foresight range (as measured at scheduling) reveals on top of today's rule (holders in the
        // allies' order)
        let mut readers = readers_of(w, &actions);
        let ranged: &[UnitId] = ranged.iter().find(|r| r.0 == unit).map_or(&[], |r| r.1.as_slice());
        if !ranged.is_empty() {
            readers = w.allies.iter().copied().filter(|h| readers.contains(h) || ranged.contains(h)).collect();
        }
        let revealed = !readers.is_empty() || enraged || reveal_all;
        let plan = if revealed { actions.iter().map(|a| line_of(w, a)).collect() } else { Vec::new() };
        Intent { unit, line: line_of(w, actions[0]), revealed, enraged, readers, plan, morale_state: "", morale_group: -1, round_intent: None, round_target: None, maneuver: None, impacts: Vec::new() }
    };
    let cmds: Vec<&Command> = actions.iter().map(|a| &a.command).collect();
    fill_round(w, &mut rec, None, &cmds, impacts_of(w, &actions));
    rec
}

/// RULESET1 V5, intent UI5: the record of a party-side unit (its own plan, so always revealed). While planning
/// (`plan` given) it reads the unit's plan entries; after the plan is committed, its scheduled actions.
pub fn party_intent_of(w: &World, unit: UnitId, plan: Option<&Plan>, scheduled: &[ScheduledAction]) -> Intent {
    let idle = || Line { kind: IDLE, target: None, cell: NO_CELL, blocks: (0, 0), recovery: 0, skill: String::new() };
    let mut rec = Intent { unit, line: idle(), revealed: true, enraged: false, readers: Vec::new(), plan: Vec::new(), morale_state: "", morale_group: -1, round_intent: None, round_target: None, maneuver: None, impacts: Vec::new() };
    match plan {
        Some(p) => {
            let st = p.compile(w, unit);
            let cmds: Vec<Command> = st.entries.iter().map(|e| e.build_command(w, unit)).collect();
            rec.plan = st
                .entries
                .iter()
                .zip(&cmds)
                .map(|(e, cmd)| Line { kind: kind_of(w, cmd), target: target_of(cmd), cell: cell_of(cmd), blocks: (e.start_block, e.end_block(w)), recovery: e.recovery, skill: cmd.name.clone() })
                .collect();
            let impacts = st.entries.iter().zip(&cmds).filter_map(|(e, cmd)| badge_kind(w, cmd).map(|k| (e.end_block(w), k))).collect();
            let refs: Vec<&Command> = cmds.iter().collect();
            fill_round(w, &mut rec, st.round_target, &refs, impacts);
        }
        None => {
            let actions = actions_of(unit, scheduled);
            rec.plan = actions.iter().map(|a| line_of(w, a)).collect();
            let cmds: Vec<&Command> = actions.iter().map(|a| &a.command).collect();
            fill_round(w, &mut rec, None, &cmds, impacts_of(w, &actions));
        }
    }
    if let Some(first) = rec.plan.first() {
        rec.line = first.clone();
    }
    rec
}

/// One record per active enemy on the grid, in the enemies' order (`ranged` as `intent_of`).
pub fn intents(w: &World, scheduled: &[ScheduledAction], on_grid_at_schedule: &[UnitId], ranged: &[(UnitId, Vec<UnitId>)], enraged: &[UnitId]) -> Vec<Intent> {
    w.enemies.iter().copied().filter(|&e| w.units[e].is_active() && w.has(e)).map(|e| intent_of(w, e, scheduled, on_grid_at_schedule, ranged, enraged.contains(&e))).collect()
}

/// RULESET1 V5, intent UI5: one record per active unit on the grid, both sides (the threat intel panel, strip rows and
/// board glyphs): the party in the allies' order (`party_intent_of`), then the enemies as `intents`. Empty without
/// `plan_badges` (classic).
pub fn round_intents(w: &World, plan: Option<&Plan>, scheduled: &[ScheduledAction], on_grid_at_schedule: &[UnitId], ranged: &[(UnitId, Vec<UnitId>)], enraged: &[UnitId]) -> Vec<Intent> {
    if !badges_on(w) {
        return Vec::new();
    }
    let mut out: Vec<Intent> = w.allies.iter().copied().filter(|&a| w.units[a].is_active() && w.has(a)).map(|a| party_intent_of(w, a, plan, scheduled)).collect();
    out.extend(intents(w, scheduled, on_grid_at_schedule, ranged, enraged));
    out
}

/// HOOKS1 AH3 (rule `arena_hazards`; engine-only): one scheduled arena hazard due this round, known from its start: its
/// id, name, struck cells and the blocks at whose end it strikes (the board marks the cells, the strip the blocks).
#[derive(Debug, Clone, PartialEq)]
pub struct ArenaTell {
    pub id: String,
    pub name: String,
    pub cells: Vec<V2>,
    pub blocks: Vec<i32>,
}

/// HOOKS1 AH3: the arena tells of the round, in the encounter's order (empty without the rule, so classic has none).
/// Both sides see them whatever `plan_reveal` says: the arena hides nothing.
pub fn arena_tells(w: &World) -> Vec<ArenaTell> {
    crate::arena::due(w).into_iter().map(|h| ArenaTell { id: h.id.clone(), name: h.name.clone(), cells: h.cells.clone(), blocks: h.blocks.clone() }).collect()
}

/// HOOKS1 AH3: the arena tells as JSON: [{id, name, cells: [[x, y], ...], blocks: [...]}, ...].
pub fn arena_json(tells: &[ArenaTell]) -> serde_json::Value {
    serde_json::Value::Array(
        tells.iter().map(|t| serde_json::json!({"id": t.id, "name": t.name, "cells": t.cells.iter().map(|c| serde_json::json!([c.x, c.y])).collect::<Vec<_>>(), "blocks": t.blocks})).collect(),
    )
}

/// The round target in the trace shape: [kind ("", "self", "unit", "cell"), unit name or "", cell x, cell y].
fn target_trace(w: &World, t: Option<RoundTarget>) -> serde_json::Value {
    match t {
        None => serde_json::json!(["", "", NO_CELL.x, NO_CELL.y]),
        Some(RoundTarget::Myself) => serde_json::json!(["self", "", NO_CELL.x, NO_CELL.y]),
        Some(RoundTarget::Unit(u)) => serde_json::json!(["unit", w.units[u].name, NO_CELL.x, NO_CELL.y]),
        Some(RoundTarget::Cell(p)) => serde_json::json!(["cell", "", p.x, p.y]),
    }
}

/// The tells in the trace shape the retired Godot recorder wrote (trace.gd `intents_state`): per enemy
/// [name, line, revealed, enraged, [reader names], [plan lines], morale state, morale group]; a line is
/// [kind, target name or "", cell x, cell y, start block, end block, skill]. RULESET1 V5: under `plan_badges` each
/// record adds [round intent id or "", Maneuver kind id or "", round target (`target_trace`), [[impact block, badge
/// kind], ...]] and each line its recovery blocks; classic keeps the Godot shape.
pub fn trace(w: &World, intents: &[Intent]) -> serde_json::Value {
    let badges = badges_on(w);
    let name = |u: Option<UnitId>| u.map(|u| w.units[u].name.clone()).unwrap_or_default();
    let line = |l: &Line| {
        if badges {
            serde_json::json!([l.kind, name(l.target), l.cell.x, l.cell.y, l.blocks.0, l.blocks.1, l.skill, l.recovery])
        } else {
            serde_json::json!([l.kind, name(l.target), l.cell.x, l.cell.y, l.blocks.0, l.blocks.1, l.skill])
        }
    };
    serde_json::Value::Array(
        intents
            .iter()
            .map(|i| {
                let mut rec = serde_json::json!([
                    w.units[i.unit].name,
                    line(&i.line),
                    i.revealed,
                    i.enraged,
                    i.readers.iter().map(|r| w.units[*r].name.clone()).collect::<Vec<_>>(),
                    i.plan.iter().map(line).collect::<Vec<_>>(),
                    i.morale_state,
                    i.morale_group
                ]);
                if badges {
                    let arr = rec.as_array_mut().unwrap();
                    arr.push(serde_json::json!(i.round_intent.map_or("", |r| r.id())));
                    arr.push(serde_json::json!(i.maneuver.map_or("", maneuver_id)));
                    arr.push(target_trace(w, i.round_target));
                    arr.push(serde_json::json!(i.impacts.iter().map(|(b, k)| serde_json::json!([b, k])).collect::<Vec<_>>()));
                }
                rec
            })
            .collect(),
    )
}
