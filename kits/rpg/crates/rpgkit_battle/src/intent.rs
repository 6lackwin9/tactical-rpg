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
//!
//! PK1-S2b (rule `channel`, PK1 4.3 item 6; engine-only): while a unit's round flag is set (a Channel round) its record
//! shows the intent Channel (`round_intent`, term `intent_channel`) with `channel` set and the round's own row in
//! `row_intent`, and each line of a channeled version names it with the channeled mark (`channeled`, term `channeled_mark`). Without
//! the rule (or a Channel round) every record and the trace are unchanged.

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
    /// The action's display name ("" when arriving or idle); PK1-S2b: a channeled version's carries the channeled mark
    /// (`channel::channeled_name`: "Cleave (Channeled)").
    pub skill: String,
    /// PK1-S2b (PK1 4.3 item 6, rule `channel`): the action runs a channeled version (false outside a Channel round).
    pub channeled: bool,
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
    /// RULESET1 V5 (`plan_badges` with `round_intent`): the unit's round intent as the tells show it (None without one
    /// or with a rule off); PK1-S2b: `RoundIntent::Channel` while the round is a Channel round (`channel::tell_intent`).
    pub round_intent: Option<RoundIntent>,
    /// PK1-S2b (rule `channel`, under `plan_badges` with `round_intent`): the round is a Channel round (the round flag
    /// `RoundState::channel`; false without the rule).
    pub channel: bool,
    /// PK1-S2b: the round's own intent (the first skill's row, which the rules read; `round_intent::intent_of`): equal
    /// to `round_intent` outside a Channel round (None as `round_intent`).
    pub row_intent: Option<RoundIntent>,
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
        CmdKind::Item { cell: Some(p), .. } => RoundTarget::Cell(*p),
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
    // PK1-S2b (PK1 4.3 item 6): a Channel round shows Channel from the toggle on; the Maneuver kind follows the row
    rec.row_intent = round_intent::intent_of(w, c);
    rec.channel = crate::channel::is_channel(w, c);
    rec.round_intent = crate::channel::tell_intent(w, c);
    rec.maneuver = (rec.row_intent == Some(RoundIntent::Maneuver)).then(|| round_intent::maneuver_of(w, c));
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

/// PK1-S2b (PK1 4.3 item 6): the name a command shows in the tells: a channeled version's with the channeled mark.
pub fn skill_name(w: &World, cmd: &Command) -> String {
    if cmd.channeled {
        crate::channel::channeled_name(&w.cat, &cmd.name)
    } else {
        cmd.name.clone()
    }
}

pub fn line_of(w: &World, a: &ScheduledAction) -> Line {
    Line { kind: kind_of(w, &a.command), target: target_of(&a.command), cell: cell_of(&a.command), blocks: (a.start, a.impact), recovery: a.end - a.impact, skill: skill_name(w, &a.command), channeled: a.command.channeled }
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
        Intent { unit, line: Line { kind, target: None, cell: NO_CELL, blocks: (0, 0), recovery: 0, skill: String::new(), channeled: false }, revealed: enraged || reveal_all, enraged, readers: Vec::new(), plan: Vec::new(), morale_state: "", morale_group: -1, round_intent: None, channel: false, row_intent: None, round_target: None, maneuver: None, impacts: Vec::new() }
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
        Intent { unit, line: line_of(w, actions[0]), revealed, enraged, readers, plan, morale_state: "", morale_group: -1, round_intent: None, channel: false, row_intent: None, round_target: None, maneuver: None, impacts: Vec::new() }
    };
    let cmds: Vec<&Command> = actions.iter().map(|a| &a.command).collect();
    fill_round(w, &mut rec, None, &cmds, impacts_of(w, &actions));
    rec
}

/// RULESET1 V5, intent UI5: the record of a party-side unit (its own plan, so always revealed). While planning
/// (`plan` given) it reads the unit's plan entries; after the plan is committed, its scheduled actions.
pub fn party_intent_of(w: &World, unit: UnitId, plan: Option<&Plan>, scheduled: &[ScheduledAction]) -> Intent {
    let idle = || Line { kind: IDLE, target: None, cell: NO_CELL, blocks: (0, 0), recovery: 0, skill: String::new(), channeled: false };
    let mut rec = Intent { unit, line: idle(), revealed: true, enraged: false, readers: Vec::new(), plan: Vec::new(), morale_state: "", morale_group: -1, round_intent: None, channel: false, row_intent: None, round_target: None, maneuver: None, impacts: Vec::new() };
    match plan {
        Some(p) => {
            let st = p.compile(w, unit);
            let cmds: Vec<Command> = st.entries.iter().map(|e| e.build_command(w, unit)).collect();
            rec.plan = st
                .entries
                .iter()
                .zip(&cmds)
                .map(|(e, cmd)| Line { kind: kind_of(w, cmd), target: target_of(cmd), cell: cell_of(cmd), blocks: (e.start_block, e.end_block(w)), recovery: e.recovery, skill: skill_name(w, cmd), channeled: cmd.channeled })
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

/// PR7-41 (PT-20): the round as it will run: the scheduled actions `sched` (the enemies', drafted first) with `plan`'s
/// entries booked after them, as `Plan::commit` books them on confirm (planning reads the plan, not the resolver).
pub fn planned_schedule(w: &World, sched: &[ScheduledAction], plan: &Plan) -> Vec<ScheduledAction> {
    // ENGINE-46: once per scope on this battle (`memo`)
    memo::round(w, sched, plan, || planned_schedule_now(w, sched, plan))
}

fn planned_schedule_now(w: &World, sched: &[ScheduledAction], plan: &Plan) -> Vec<ScheduledAction> {
    let mut res = crate::resolver::Resolver::new(w.total_blocks, 0);
    res.scheduled = sched.to_vec();
    plan.commit(w, &mut res);
    res.scheduled
}

/// PT-20: the guard mitigation `c` has at `block`, as `Resolver::prepare_block` raises it: the largest of each live
/// Defend that has started by then (S1.2-S4: held from the wind-up start to the round's end; a broken guard gives
/// none) and the held guard or `guard` status (`trigger::held_at`). Read only.
pub fn guard_at(w: &World, sched: &[ScheduledAction], c: UnitId, block: i32) -> f64 {
    let running = sched.iter().filter(|a| a.unit == c && !a.cancelled && a.start <= block).filter_map(|a| match &a.command.kind {
        CmdKind::Defend { mitigation, guard_broken: false, .. } => Some(*mitigation),
        _ => None,
    });
    running.fold(crate::trigger::held_at(w, c, block).unwrap_or(0.0), f64::max)
}

/// PT-20: the stance `c` holds when a hit lands at `block`, as the resolver changes it: each live action's start stance
/// from its start block on (`Resolver::begin_actions`), each end stance (a flow) once its impact has passed
/// (`apply_combo`); else the stance it holds now.
pub fn stance_at(w: &World, sched: &[ScheduledAction], c: UnitId, block: i32) -> rpgkit_rules::unit::Stance {
    let mut changes: Vec<(i32, i32, rpgkit_rules::unit::Stance)> = Vec::new();
    for a in actions_of(c, sched) {
        if let Some(st) = a.command.stance_on_start.filter(|_| a.start <= block) {
            changes.push((a.start, 0, st));
        }
        if let Some(st) = a.command.stance_on_end.filter(|_| a.impact < block) {
            changes.push((a.impact, 1, st));
        }
    }
    changes.sort_by_key(|x| (x.0, x.1));
    changes.last().map_or(w.units[c].stance, |x| x.2)
}

/// PT-20: one hit a scheduled action lands on a unit, forecast by the one convention every damage number on the battle
/// screen follows: per hit, at the block it lands, with the target's guard at that block (no crit roll).
#[derive(Debug, Clone, PartialEq)]
pub struct HitOn {
    /// The action's tell name (`skill_name`).
    pub skill: String,
    /// The block the hit lands (the action's impact).
    pub block: i32,
    pub damage: i32,
    /// The target guards at that block (a guard-bypassing hit counts it as unguarded).
    pub guarded: bool,
    /// The action's place among its unit's live actions in block order (`actions_of`; 0 = its first, the tell's
    /// line), so a screen can keep an unrevealed plan's later actions hidden; `hit_on` leaves it 0.
    pub nth: usize,
    /// HITCALC1 (PR7-45): how the hit fails at its impact (None: it lands). `damage` is what reaches the target: 0
    /// for a dodge, out of reach or a protected flank; the part an intercept leaves.
    pub fails: Option<HitFail>,
}

/// HITCALC1 (PR7-45): how a planned hit fails at its impact, as the resolver decides it (`Command::execute`,
/// `trigger::ally_hit`), so a screen shows it as such instead of a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitFail {
    /// ATTACKLOCK1: the target left the tile the attack locked on before it lands.
    Dodged,
    /// The target is outside the attack's reach band at the impact (after the chase or dash it may take as it starts).
    OutOfReach,
    /// INPUT1 MR8 c: a carried Engage finds the target's flank protected and misses.
    FlankProtected,
    /// HOOKS1 8.4: a friend of the target takes `pct` percent of the hit (`trigger::intercept_of`).
    Intercepted { by: UnitId, pct: i32 },
    /// PR7-87 (PT-67, GD5): the target holds stealth and has moved this round by the impact, so a single-target attack
    /// or a blow shaped on it finds nothing (`hidden_at`).
    Hidden,
}

/// PR7-87 (PT-67): `round_intent::hidden` read at `block` over the round as it will run: the target holds `stealth`
/// (now, or granted by its own action landing earlier) and has moved this round (already, or by a move, disengage,
/// carried Engage or retreat of its own landing earlier).
pub fn hidden_at(w: &World, sched: &[ScheduledAction], t: UnitId, block: i32) -> bool {
    if !round_intent::on(w) {
        return false;
    }
    let earlier: Vec<&ScheduledAction> = actions_of(t, sched).into_iter().filter(|a| a.impact < block && !a.landed).collect();
    let stealth = w.units[t].has_status("stealth") || earlier.iter().any(|a| matches!(&a.command.kind, CmdKind::Defend { grant_status, .. } if grant_status == "stealth"));
    let moved = w.round_states.get(&t).is_some_and(|r| r.moved)
        || earlier.iter().any(|a| match &a.command.kind {
            CmdKind::Walk { path, .. } => !path.is_empty(),
            CmdKind::Disengage { .. } => true,
            _ => a.command.carry != crate::command::Carry::None,
        });
    stealth && moved
}

/// PT-20, PT-17: where `c` stands when a hit lands at `block`: the end tile of its last live move that lands earlier
/// (`command::walk_preview`), else where it stands now. `sched` is the round as it will run (`planned_schedule` while
/// planning), so a party member's planned moves count.
///
/// PR7-90 (PT-85): each step of a walk is read on the board as it stands when the walk runs (`walk_end_at`): the
/// tiles the other units hold then (their moves that land earlier, or in the same block ahead of it), a friend's tile
/// passed through as ESCAPE-PLAN1 walks it, a foe that steps in front stopping it.
pub fn position_at(w: &World, sched: &[ScheduledAction], c: UnitId, block: i32) -> V2 {
    position_in(w, sched, c, block, None, true)
}

/// `position_at`, also counting the moves landing in `block` itself whose command acts before initiative `ahead_of`
/// (the resolver's order in a block); `aware`: a walk reads the others' tiles then (`walk_end_at`), else the board
/// now (`walk_preview`, the others' own reads inside `walk_end_at`).
fn position_in(w: &World, sched: &[ScheduledAction], c: UnitId, block: i32, ahead_of: Option<i32>, aware: bool) -> V2 {
    // the inner reads (the board now) are cheap: only a read that plays the walks is kept
    if !aware {
        return position_read(w, sched, c, block, ahead_of, aware);
    }
    memo::tile(w, sched, (c, block, ahead_of, aware), || position_read(w, sched, c, block, ahead_of, aware))
}

fn position_read(w: &World, sched: &[ScheduledAction], c: UnitId, block: i32, ahead_of: Option<i32>, aware: bool) -> V2 {
    let mut at = w.grid.pos(c).unwrap_or(NO_CELL);
    let earlier = |a: &ScheduledAction| a.impact < block || ahead_of.is_some_and(|i| a.impact == block && a.command.initiative(w) > i);
    // HITCALC1: a move that already landed is in the tile it stands on now (mid-round reads)
    for a in actions_of(c, sched).into_iter().filter(|a| (a.start < block || earlier(a)) && !a.landed) {
        // PR7-87 (PT-67): a carried Engage moves its unit as it starts
        if let crate::command::Carry::Engage(cell) = a.command.carry {
            if !a.command.started && cell != NO_CELL {
                at = cell;
            }
        }
        if !earlier(a) {
            continue;
        }
        match &a.command.kind {
            // each move steps on from where the earlier ones end (the preview's start is the chain's tile)
            CmdKind::Walk { path, expected_start, goal, .. } if aware => at = walk_end_at(w, sched, c, a, path, (*expected_start, *goal), at),
            CmdKind::Walk { path, .. } => {
                if let Some(end) = crate::command::walk_preview(w, c, path, at).last() {
                    at = *end;
                }
            }
            // PR7-87 (PT-67): a disengage lands on its tile at its impact
            CmdKind::Disengage { dest, .. } if *dest != NO_CELL => at = *dest,
            _ => {}
        }
        // PR7-87 (PT-67): a carried retreat follows its hit
        if let crate::command::Carry::Retreat(cell) = a.command.carry {
            if cell != NO_CELL {
                at = cell;
            }
        }
    }
    // PR7-87 (PT-69): a push or pull of another unit's attack on `c` landing earlier moves it by the forecast steps
    // (`forced::forecast` from the tile it stands on now), applied after its own moves
    if crate::forced::on(w) && w.has(c) {
        let mut shoves: Vec<&ScheduledAction> = sched.iter().filter(|a| a.unit != c && !a.cancelled && !a.landed && a.impact < block && a.command.forced.is_some() && aims_at(w, a, c)).collect();
        shoves.sort_by_key(|a| a.impact);
        for a in shoves {
            let Some(src_pos) = w.grid.pos(a.unit) else { continue };
            if let Some(f) = crate::forced::forecast(w, sched, a.unit, src_pos, &a.command, a.impact, &[c]).into_iter().next() {
                at = at + (f.to - f.from);
            }
        }
    }
    at
}

/// PR7-90 (PT-85): the tile walk `a` of `c` (steps `path`, from `from`) ends on, as `command::walk` takes it when it
/// runs: the other units stand where their moves landing earlier (or in the same block, ahead of it) leave them
/// (`position_in`, one level: their own walks read the board now); a step onto a foe's tile, a wall or the edge stops
/// the walk; a party member passes a friend's tile mid-route and steps back to the last tile it held alone when the
/// walk ends on one (ESCAPE-PLAN1 (2)); an exit it leaves by and a trap it springs end it. A walk that starts off its
/// planned start (an earlier one was cut short) re-paths to its goal around the others, as `walk` does. A big unit
/// keeps `walk_preview`.
fn walk_end_at(w: &World, sched: &[ScheduledAction], c: UnitId, a: &ScheduledAction, path: &[V2], (expected, goal): (V2, V2), from: V2) -> V2 {
    let preview = || crate::command::walk_preview(w, c, path, from).last().copied().unwrap_or(from);
    if w.size_of(c) > 1 || !w.units[c].is_active() || !w.has(c) || from == NO_CELL {
        return preview();
    }
    let init = a.command.initiative(w);
    let others: Vec<(UnitId, V2)> = (0..w.units.len())
        .filter(|&u| u != c && w.units[u].is_active() && w.has(u))
        .map(|u| (u, position_in(w, sched, u, a.impact, Some(init), false)))
        .filter(|(_, p)| *p != NO_CELL)
        .collect();
    let holder = |cell: V2| others.iter().find(|(u, p)| crate::grid::square_covers(*p, w.size_of(*u), cell)).map(|(u, _)| *u);
    // `command::walk`'s re-path: off its planned start, the route to its goal around the others, cut to its budget
    let repath: Vec<V2>;
    let path = if expected != NO_CELL && goal != NO_CELL && from != expected {
        let blocked: crate::path::Cells = others.iter().flat_map(|(u, p)| crate::grid::square_cells(*p, w.size_of(*u))).collect();
        let goals: crate::path::Cells = [goal].into_iter().collect();
        let full = crate::path::find_path_sized(&w.grid, from, &goals, &blocked, w.grid.is_exit(goal), &crate::path::Hooks::default(), 1);
        let budget = w.units[c].move_per_block().max(crate::path::path_cost_sized(&w.grid, expected, path, 1));
        repath = crate::path::fit_blocks_sized(&w.grid, from, &full, budget, 1, 1).into_iter().take(path.len()).collect();
        &repath[..]
    } else {
        path
    };
    let passes = crate::plan::escape_plan1() && w.allies.contains(&c);
    let friends = w.friends_of(c);
    let (mut at, mut alone) = (from, from);
    let n = path.len();
    for (i, d) in path.iter().enumerate() {
        let next = at + *d;
        if w.grid.is_exit(next) || (w.grid.open && !w.grid.is_valid(next)) {
            if w.hold.as_ref().is_some_and(|g| !g.lets_out(w, c)) {
                break;
            }
            if w.grid.is_valid(next) {
                at = next;
                alone = next;
            }
            break;
        }
        if !w.grid.is_valid(next) || !w.grid.is_walkable(next) {
            break;
        }
        match holder(next) {
            Some(o) if passes && i + 1 < n && friends.contains(&o) => at = next,
            Some(_) => break,
            None => {
                at = next;
                alone = next;
            }
        }
        if crate::command::newly_covered(w, c, at - *d, at).iter().any(|&cell| crate::ground::springs(w, c, cell).is_some()) {
            break;
        }
    }
    if holder(at).is_some() {
        at = alone;
    }
    at
}

/// PR7-87 (PT-69): scheduled action `a` is locked on `c`: a single-target attack on it (or its lock), or a blow shaped
/// on it.
fn aims_at(w: &World, a: &ScheduledAction, c: UnitId) -> bool {
    match &a.command.kind {
        CmdKind::Attack { target, .. } => target.or(w.units[a.unit].active_target) == Some(c),
        CmdKind::Area { target, .. } => *target == Some(c),
        _ => false,
    }
}

/// PR7-41 (PT-17): the tiles scheduled area action `a` strikes when it lands, the one area its tell marks. A tile
/// area (no target) strikes the tiles fixed when it was planned; a shaped Assault row (IR5, IR7, IR17: locked on its
/// target) is centred where the target stands at the impact, read from where its user stands then (`position_at`, as
/// `Command::execute` re-reads it). Its planned tiles when the target is gone or off the grid; empty for any other
/// action. Read only.
pub fn area_cells(w: &World, sched: &[ScheduledAction], a: &ScheduledAction) -> Vec<V2> {
    let CmdKind::Area { cells, target, shape, .. } = &a.command.kind else { return Vec::new() };
    let Some(t) = target.filter(|&t| w.units.get(t).is_some_and(|u| u.is_active()) && w.has(t)) else { return cells.clone() };
    let (from, to) = (position_at(w, sched, a.unit, a.impact), position_at(w, sched, t, a.impact));
    if from == NO_CELL || to == NO_CELL {
        return cells.clone();
    }
    crate::skill::shape_cells(*shape, from, to).into_iter().filter(|c| w.grid.is_valid(*c) && *c != from).collect()
}

/// PT-20: the hit scheduled action `a` lands on `t` (an Attack on it, or an Area whose tiles hold it or that is
/// locked on it), forecast as `HitOn`; None for any other action or a target it misses. HITCALC1 (PR7-45): the damage
/// comes from the resolver's own hit-input builder (`command::hit_terms`) on the board at the impact (`scene_at`: the
/// tiles both units stand on then, `guard_at` the impact, both stances then), and a hit that fails at its impact says
/// how (`HitOn::fails`: a dodge, out of reach, a protected flank or an intercept). `sched` is the round as it will run
/// (`planned_schedule` while planning).
pub fn hit_on(w: &World, sched: &[ScheduledAction], a: &ScheduledAction, t: UnitId) -> Option<HitOn> {
    hit_on_with(w, sched, a, t, false)
}

/// `hit_on`, the hit forced to crit when `crit` (BATTLE-UI2: the foe card's crit star; `HitInput::force_crit`).
fn hit_on_with(w: &World, sched: &[ScheduledAction], a: &ScheduledAction, t: UnitId, crit: bool) -> Option<HitOn> {
    hit_on_adjusted(w, sched, a, t, crit, &Adjust::NONE)
}

/// STATUSVIS1 SV7 (PR7-57): what a what-if forecast takes away from a hit (`hit_on_adjusted`): `power` multiplies
/// the command's power (1.0: as booked; the reciprocal of an attacker's status factor takes that status out), `strip`
/// lists statuses the target is forecast without, `guard` false forecasts it unguarded.
#[derive(Debug, Clone, Copy)]
pub struct Adjust<'a> {
    pub power: f64,
    pub strip: &'a [&'a str],
    pub guard: bool,
}

impl Adjust<'_> {
    /// The hit as booked (`hit_on`).
    pub const NONE: Adjust<'static> = Adjust { power: 1.0, strip: &[], guard: true };
}

/// HITCALC1 (PR7-45): where target `t` stands at `block` for an attacker on tile `from` then (`position_at`); when both
/// are planned onto one tile (a forecast clash: one of them stops short when the round runs), `t` where it stands now
/// (the rule `status_view::counter_on` reads too). Read only.
pub fn target_at(w: &World, sched: &[ScheduledAction], t: UnitId, from: V2, block: i32) -> V2 {
    Some(position_at(w, sched, t, block)).filter(|p| *p != from || from == NO_CELL).unwrap_or_else(|| w.grid.pos(t).unwrap_or(NO_CELL))
}

/// HITCALC1 (PR7-45): the board scheduled action `a`'s hit on `t` is read on at its impact (`command::HitScene`): the
/// attacker's tile then (`position_at`; a carried Engage that has not started strikes from its planned tile), the
/// target's tile then, its guard then (`guard_at`; 0 for a guard-bypassing command) and both stances then
/// (`stance_at`). Read only.
pub fn scene_at(w: &World, sched: &[ScheduledAction], a: &ScheduledAction, t: UnitId) -> crate::command::HitScene {
    let from = match a.command.carry {
        crate::command::Carry::Engage(cell) if !a.command.started && cell != NO_CELL => cell,
        _ => position_at(w, sched, a.unit, a.impact),
    };
    let guard = if a.command.ignore_guard { 0.0 } else { guard_at(w, sched, t, a.impact) };
    crate::command::HitScene { from, at: target_at(w, sched, t, from, a.impact), guard: Some(guard), attacker_stance: Some(stance_at(w, sched, a.unit, a.impact)), target_stance: Some(stance_at(w, sched, t, a.impact)) }
}

/// HITCALC1 (PR7-45): how scheduled attack `a` on `t` fails at its impact, in `Command::execute`'s order: a protected
/// flank (decided as it starts, from its tile then), an ATTACKLOCK1 dodge (the lock it holds, or before it starts the
/// one it takes: the target's tile at its start when in reach then and the ruleset locks the band) and its reach band
/// at the impact (before it starts, plus the chase or dash tiles it may take). `scene`: the board at the impact.
fn attack_fail(w: &World, sched: &[ScheduledAction], a: &ScheduledAction, t: UnitId, scene: &crate::command::HitScene) -> Option<HitFail> {
    let CmdKind::Attack { min_range, max_range, follow_tiles, dash_tiles, .. } = &a.command.kind else { return None };
    let (src, cmd) = (a.unit, &a.command);
    let start_tile = position_at(w, sched, src, a.start);
    if crate::command::flank_protected_from(w, cmd, start_tile) {
        return Some(HitFail::FlankProtected);
    }
    let band = crate::command::reach_band(w, src, *min_range, *max_range);
    let (sa, st) = (w.size_of(src), w.size_of(t));
    let gap = |from: V2, at: V2| crate::grid::square_band(cmd.diagonal, from, sa, at, st);
    let t0 = target_at(w, sched, t, start_tile, a.start);
    let in_band = |d: i32, extra: i32| d >= band.0 && d <= band.1 + extra;
    let known = start_tile != NO_CELL && t0 != NO_CELL;
    // the chase or dash it may take as it starts: only toward a target out of its band then
    let extra = if cmd.started || cmd.carry != crate::command::Carry::None || (known && in_band(gap(start_tile, t0), 0)) { 0 } else { (*follow_tiles).max(*dash_tiles).max(0) };
    let lock = if cmd.started {
        cmd.locked
    } else if w.ruleset.attack_lock().locks(round_intent::melee_band(band)) {
        (known && in_band(gap(start_tile, t0), extra)).then_some((t, t0))
    } else {
        None
    };
    if lock.is_some_and(|(l, cell)| l == t && scene.at != cell) {
        return Some(HitFail::Dodged);
    }
    if scene.from == NO_CELL || scene.at == NO_CELL {
        return None;
    }
    let d = gap(scene.from, scene.at);
    (d < band.0 || d > band.1 + extra).then_some(HitFail::OutOfReach)
}

/// `hit_on_with` under `adj` (`Adjust::NONE`: the same hit, the same numbers). Read only.
pub fn hit_on_adjusted(w: &World, sched: &[ScheduledAction], a: &ScheduledAction, t: UnitId, crit: bool, adj: &Adjust) -> Option<HitOn> {
    use rpgkit_rules::damage;
    let src = a.unit;
    if a.cancelled || src == t || !w.has(t) {
        return None;
    }
    let mut scene = scene_at(w, sched, a, t);
    let mut skip: Vec<UnitId> = Vec::new();
    let (power, offhand, ranged, mut fails) = match &a.command.kind {
        CmdKind::Attack { target, skill_mult, use_offhand, .. } if target.or(w.units[src].active_target) == Some(t) && *skill_mult > 0.0 => {
            (*skill_mult, *use_offhand, w.units[src].attack_range > 1 && !use_offhand, attack_fail(w, sched, a, t, &scene))
        }
        CmdKind::Area { skill_mult, target, splash, affects, band, shape, .. } if *skill_mult > 0.0 && crate::skill::affects_unit(*affects, w.friends_of(src).contains(&t)) => {
            // PT-17: the tiles it strikes when it lands, the target where it stands then (BIGUNIT1 BU9: any of its cells)
            let struck = area_cells(w, sched, a);
            if !w.cells_at(t, scene.at).iter().any(|c| struck.contains(c)) {
                return None;
            }
            // HOOKS1 8.4: a unit the area also strikes takes no redirect
            skip = w.units.iter().enumerate().filter(|&(v, _)| w.has(v) && w.cells_at(v, position_at(w, sched, v, a.impact)).iter().any(|c| struck.contains(c))).map(|(v, _)| v).collect();
            // A4.2-K1b (IR17): a shaped Assault row strikes no tile when its target is out of its band at the impact (it
            // may chase 1 tile as it starts)
            let extra = i32::from(!a.command.started);
            let reach = |d: i32| d >= band.0 && d <= band.1 + extra;
            let out = target.is_some_and(|p| p == t) && scene.from != NO_CELL && !reach(crate::grid::square_band(crate::skill::diagonal_shape(*shape), scene.from, w.size_of(src), scene.at, w.size_of(t)));
            (*skill_mult * if target.is_none_or(|p| p == t) { 1.0 } else { *splash }, false, w.units[src].attack_range > 1, out.then_some(HitFail::OutOfReach))
        }
        _ => return None,
    };
    // PR7-87 (PT-67): a stealthed target that has moved is lost by an attack on it, or a blow shaped on it
    let aimed = match &a.command.kind {
        CmdKind::Attack { .. } => true,
        CmdKind::Area { target, .. } => target.is_some(),
        _ => false,
    };
    if fails.is_none() && aimed && hidden_at(w, sched, t, a.impact) {
        fails = Some(HitFail::Hidden);
    }
    if !adj.guard {
        scene.guard = Some(0.0);
    }
    let guard = scene.guard.unwrap_or(0.0);
    let mut input = crate::command::hit_terms(w, &a.command, src, t, &scene, ranged, offhand, power).input;
    input.force_crit = crit;
    if adj.power != 1.0 {
        input.skill_mult *= adj.power;
    }
    let damage = if adj.strip.is_empty() {
        damage::compute(&w.units[src], &w.units[t], input).final_dmg
    } else {
        let mut target = w.units[t].clone();
        for s in adj.strip {
            target.st.active.shift_remove(*s);
        }
        damage::compute(&w.units[src], &target, input).final_dmg
    };
    if fails.is_none() {
        fails = crate::trigger::intercept_of(w, sched, a.impact, src, t, ranged, &a.command.family, &skip).map(|(by, pct)| HitFail::Intercepted { by, pct });
    }
    let damage = match fails {
        None => damage,
        Some(HitFail::Intercepted { pct, .. }) if pct < 100 => crate::trigger::part(damage, 100 - pct),
        Some(_) => 0,
    };
    Some(HitOn { skill: skill_name(w, &a.command), block: a.impact, damage, guarded: guard > 0.0, nth: 0, fails })
}

/// PT-20: every hit `attacker`'s live scheduled actions land on `t` this round, in block order (`hit_on`), each with
/// its action's place (`HitOn::nth`).
pub fn hits_on(w: &World, sched: &[ScheduledAction], attacker: UnitId, t: UnitId) -> Vec<HitOn> {
    actions_of(attacker, sched).into_iter().enumerate().filter_map(|(n, a)| hit_on(w, sched, a, t).map(|h| HitOn { nth: n, ..h })).collect()
}

/// BATTLE-UI2 (`SCREEN_CONTRACT_BATTLE.md` 5, the docked foe card): one hit a party unit's live action lands on a foe,
/// read from the round as it will run (never a separate forecast list): its skill and blocks as booked, the damage by
/// the one convention (`hit_on`), the pips it breaks (each hit settles on its own, `poise::forecast_hits`) and the
/// status it applies.
#[derive(Debug, Clone, PartialEq)]
pub struct CardHit {
    pub unit: UnitId,
    /// The command's skill id (its icon) and tell name.
    pub skill: String,
    pub name: String,
    /// The action's first block and the block it lands (`ScheduledAction::start`, `impact`).
    pub start: i32,
    pub impact: i32,
    pub damage: i32,
    /// The same hit as a crit (x `CRIT_MULT`) and the chance it crits (0 to 1, its unit's `crit_chance`).
    pub crit_damage: i32,
    pub crit_chance: f64,
    /// The pips it breaks (0 to the per-hit cap; 0 without a bar).
    pub pips: i32,
    /// It empties the bar (the split shield in place of its pips).
    pub breaks: bool,
    /// The bar is already broken when it lands (shown dimmed).
    pub on_broken: bool,
    /// The status it applies (none, or the command's `status_effect`).
    pub statuses: Vec<String>,
    /// It lands after the forecast KO block (the soft "after KO" tag).
    pub after_ko: bool,
    /// HITCALC1 (PR7-45): how it fails at its impact (`HitOn::fails`; None: it lands).
    pub fails: Option<HitFail>,
}

/// BATTLE-UI2 (`SCREEN_CONTRACT_BATTLE.md` 6, the heal preview): the HP the round's planned heals restore on `unit`
/// over `sched` (the round as it will run): each live Support action of its side aimed at it (`heal_amount_on`),
/// summed, after its `heal_taken` and within its missing HP now (`effective_heal`). 0 without one. Read only.
pub fn heal_on(w: &World, sched: &[ScheduledAction], unit: UnitId) -> i32 {
    let Some(u) = w.units.get(unit) else { return 0 };
    let side = |c: UnitId| w.allies.contains(&c);
    let total: i32 = sched
        .iter()
        .filter(|a| !a.cancelled && side(a.unit) == side(unit))
        .filter_map(|a| match &a.command.kind {
            CmdKind::Support { target, skill, .. } if *target == unit => Some(crate::command::heal_amount_on(w, a.unit, unit, *skill)),
            _ => None,
        })
        .sum();
    if total <= 0 { 0 } else { crate::command::effective_heal(u, total) }
}

/// BATTLE-UI2: the foe card's read of the round on foe `foe` (`foe_card`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FoeCard {
    pub hp: i32,
    pub max_hp: i32,
    /// Every planned party hit on the foe, in block order (then start, then booking order).
    pub hits: Vec<CardHit>,
    /// The block the base damage (no crits) takes it to 0 HP; None when the plan does not.
    pub ko_block: Option<i32>,
    /// The block a hit empties its poise bar; None when none does.
    pub break_block: Option<i32>,
    /// The crit star: the earliest block before `ko_block` by which the foe is down with a chance of 50 % or more over
    /// every crit outcome of the planned hits (each crits on its own); None otherwise (and without a KO block).
    pub crit_star: Option<i32>,
    /// The bar: the pip threshold T, the pips now and the pips of a full bar (all 0 without a bar).
    pub threshold: i32,
    pub pips: i32,
    pub max_pips: i32,
}

impl FoeCard {
    /// The total base damage of `unit`'s hits (None: every party unit's).
    pub fn total(&self, unit: Option<UnitId>) -> i32 {
        self.hits.iter().filter(|h| unit.is_none_or(|u| h.unit == u)).map(|h| h.damage).sum()
    }

    /// The blocks the hits land in, in order, each with its hits (the card's rows with no hero selected).
    pub fn by_block(&self) -> Vec<(i32, Vec<&CardHit>)> {
        let mut out: Vec<(i32, Vec<&CardHit>)> = Vec::new();
        for h in &self.hits {
            match out.last_mut() {
                Some(row) if row.0 == h.impact => row.1.push(h),
                _ => out.push((h.impact, vec![h])),
            }
        }
        out
    }
}

/// BOSSUI1 BU15, BU16 (PR7-54): where the plan breaks a phase on the card's foe. With the phase's threshold `floor`
/// (`Session::phase_bar`; 0 for the last phase: no break) the hits land in the card's order until one takes the foe
/// to or below it: that hit (`hit`, its index in `FoeCard::hits`) lands in block `block` and stops at the threshold
/// (`landed`: the HP it takes), and the round ends there (PHASEBREAK1: no later hit lands). None when no hit reaches
/// it. Read only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseCut {
    pub hit: usize,
    pub block: i32,
    pub landed: i32,
}

/// BOSSUI1 BU15: the card's phase break on threshold `floor` (`PhaseCut`).
pub fn phase_cut(card: &FoeCard, floor: i32) -> Option<PhaseCut> {
    if floor < 1 || card.hp <= floor {
        return None;
    }
    let mut hp = card.hp;
    for (k, h) in card.hits.iter().enumerate() {
        if hp - h.damage <= floor {
            return Some(PhaseCut { hit: k, block: h.impact, landed: (hp - floor).max(0) });
        }
        hp -= h.damage;
    }
    None
}

/// The chance `hp` HP of base hits `hits` (each `(damage, crit damage, crit chance)`) are all taken, over every crit
/// outcome (each hit crits on its own): an exact distribution of the damage so far, capped at `hp`.
fn down_chance(hp: i32, hits: &[(i32, i32, f64)]) -> f64 {
    let cap = hp.max(0) as usize;
    let mut dist = vec![0.0f64; cap + 1];
    dist[0] = 1.0;
    for &(base, crit, p) in hits {
        let p = p.clamp(0.0, 1.0);
        let mut next = vec![0.0f64; cap + 1];
        for (d, &q) in dist.iter().enumerate() {
            if q == 0.0 {
                continue;
            }
            if d == cap {
                next[cap] += q;
                continue;
            }
            next[(d + base.max(0) as usize).min(cap)] += q * (1.0 - p);
            next[(d + crit.max(0) as usize).min(cap)] += q * p;
        }
        dist = next;
    }
    dist[cap]
}

/// BATTLE-UI2 (`SCREEN_CONTRACT_BATTLE.md` 5): the foe card of `foe` over `sched`, the round as it will run
/// (`planned_schedule` while planning, so the rows are the live plan's: the same skills, blocks and booking): each
/// party unit's live hit on it (`hit_on`, its crit too), the pips each breaks on the bar the earlier ones leave
/// (`poise::forecast_hits`), the KO and BREAK blocks, the after-KO tags and the crit star. Read only.
pub fn foe_card(w: &World, sched: &[ScheduledAction], foe: UnitId) -> FoeCard {
    use crate::poise;
    let Some(u) = w.units.get(foe) else { return FoeCard::default() };
    let mut card = FoeCard { hp: u.hp, max_hp: u.max_hp, ..FoeCard::default() };
    let n = &w.ruleset.numbers;
    let bar = poise::state(w, foe).filter(|st| st.max > 0).copied();
    if let Some(st) = bar {
        (card.threshold, card.pips, card.max_pips) = (poise::pip_threshold(n, &st), poise::pips_now(n, &st), poise::full_pips(n, &st));
    }
    let mut acts: Vec<(usize, &ScheduledAction)> = sched.iter().enumerate().filter(|(_, a)| !a.cancelled && a.unit != foe && w.allies.contains(&a.unit)).collect();
    acts.sort_by_key(|(k, a)| (a.impact, a.start, *k));
    let landed: Vec<(&ScheduledAction, HitOn, i32)> = acts
        .into_iter()
        .filter_map(|(_, a)| {
            let h = hit_on(w, sched, a, foe)?;
            let crit = hit_on_with(w, sched, a, foe, true).map_or(h.damage, |c| c.damage);
            Some((a, h, crit))
        })
        .collect();
    // HITCALC1: a hit that fails at its impact (or that a friend takes whole) staggers nothing
    let staggers = |h: &HitOn| match h.fails {
        None => true,
        Some(HitFail::Intercepted { pct, .. }) => pct < 100,
        Some(_) => false,
    };
    let planned: Vec<poise::PlannedHit> = landed
        .iter()
        .filter(|(_, h, _)| staggers(h))
        .map(|(a, _, _)| {
            let flank = round_intent::flank_bonus_at(w, a.unit, scene_at(w, sched, a, foe).from, foe);
            poise::PlannedHit { target: foe, block: a.impact, impact: a.command.impact, blocks: a.command.block_cost, flank, ignore_guard: a.command.ignore_guard }
        })
        .collect();
    let fcs_kept = if bar.is_some() { poise::forecast_hits(w, sched, &planned) } else { Vec::new() };
    let mut kept = fcs_kept.into_iter();
    let fcs: Vec<Option<poise::Forecast>> = if bar.is_some() { landed.iter().map(|(_, h, _)| if staggers(h) { kept.next().flatten() } else { None }).collect() } else { Vec::new() };
    let mut broken = bar.is_some_and(|st| st.broken);
    let mut taken = 0;
    for (k, (a, h, crit)) in landed.iter().enumerate() {
        let fc = fcs.get(k).copied().flatten();
        let breaks = fc.is_some_and(|f| matches!(f.verdict, poise::Verdict::Breaks(_)));
        let on_broken = broken || fc.is_some_and(|f| f.verdict == poise::Verdict::AlreadyBroken);
        let pips = if on_broken || breaks { 0 } else { fc.map_or(0, |f| (f.pips_before - f.pips_after).max(0)) };
        if breaks && card.break_block.is_none() {
            card.break_block = Some(a.impact);
        }
        broken |= breaks;
        let after_ko = card.ko_block.is_some_and(|b| a.impact > b);
        taken += h.damage;
        if card.ko_block.is_none() && card.hp > 0 && taken >= card.hp {
            card.ko_block = Some(a.impact);
        }
        let statuses = if a.command.status_effect.is_empty() { Vec::new() } else { vec![a.command.status_effect.clone()] };
        let chance = (rpgkit_rules::damage::crit_chance(&w.units[a.unit]) / 100.0).clamp(0.0, 1.0);
        card.hits.push(CardHit { unit: a.unit, skill: a.command.skill.clone(), name: h.skill.clone(), start: a.start, impact: a.impact, damage: h.damage, crit_damage: *crit, crit_chance: chance, pips, breaks, on_broken, statuses, after_ko, fails: h.fails });
    }
    if let Some(ko) = card.ko_block {
        let blocks: Vec<i32> = card.by_block().into_iter().map(|(b, _)| b).filter(|b| *b < ko).collect();
        card.crit_star = blocks.into_iter().find(|&b| {
            let by: Vec<(i32, i32, f64)> = card.hits.iter().filter(|h| h.impact <= b).map(|h| (h.damage, h.crit_damage, h.crit_chance)).collect();
            down_chance(card.hp, &by) >= 0.5
        });
    }
    card
}

/// ENGINE-46 (FEEL-FIX1): a screen that reads many forecasts over one battle in a row (a frame of the battle while
/// planning: the plates, the foe card, the strip, the tips, the marks) opens a scope on that battle's world, scheduled
/// actions and plan (`memo::scope`). While it lives (it borrows them, so none of them can change), the round as it will
/// run (`planned_schedule`), each member's compiled plan (`Plan::compile`) and each tile read (`position_at`) over that
/// world are worked out once. Reads over any other world (a preview's copy, a planner's trial) are worked out as before,
/// and with no scope open (the resolver, the sims, the goldens) nothing is kept: the answers are the same either way.
pub mod memo {
    use super::{CmdKind, ScheduledAction, UnitId, World, V2};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};

    /// A quick hasher for a round's print (FNV-1a over 8-byte words): the tile reads are many and cheap, so SipHash's
    /// cost would outweigh what the scope saves.
    #[derive(Default)]
    struct Quick(u64);

    impl Hasher for Quick {
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, bytes: &[u8]) {
            let mut h = if self.0 == 0 { 0xcbf2_9ce4_8422_2325 } else { self.0 };
            for chunk in bytes.chunks(8) {
                let mut word = [0u8; 8];
                word[..chunk.len()].copy_from_slice(chunk);
                h ^= u64::from_le_bytes(word);
                h = h.wrapping_mul(0x0000_0100_0000_01b3).rotate_left(5);
            }
            self.0 = h;
        }
    }

    type Quickly = std::hash::BuildHasherDefault<Quick>;

    #[derive(Default)]
    struct Memo {
        /// The open scopes' battles, innermost last: (world, scheduled actions and their count, plan) addresses.
        open: Vec<(usize, (usize, usize), usize)>,
        tiles: HashMap<(u64, (UnitId, i32, Option<i32>, bool)), V2, Quickly>,
        round: Option<Vec<ScheduledAction>>,
        compiled: HashMap<UnitId, crate::plan::SimState, Quickly>,
    }

    thread_local!(static MEMO: RefCell<Memo> = RefCell::new(Memo::default()));

    /// While it lives, forecast reads over its battle are kept (see the module).
    pub struct Scope<'a> {
        _battle: std::marker::PhantomData<(&'a World, &'a [ScheduledAction], &'a crate::plan::Plan)>,
    }

    fn id(w: &World, sched: &[ScheduledAction], plan: &crate::plan::Plan) -> (usize, (usize, usize), usize) {
        (w as *const World as usize, (sched.as_ptr() as usize, sched.len()), plan as *const crate::plan::Plan as usize)
    }

    /// A scope on battle `w` with its scheduled actions `sched` (the enemies', drafted) and the party's `plan`.
    pub fn scope<'a>(w: &'a World, sched: &'a [ScheduledAction], plan: &'a crate::plan::Plan) -> Scope<'a> {
        MEMO.with(|m| {
            let mut m = m.borrow_mut();
            let battle = id(w, sched, plan);
            if m.open.last() != Some(&battle) {
                m.tiles.clear();
                m.round = None;
                m.compiled.clear();
            }
            m.open.push(battle);
        });
        Scope { _battle: std::marker::PhantomData }
    }

    impl Drop for Scope<'_> {
        fn drop(&mut self) {
            MEMO.with(|m| {
                let mut m = m.borrow_mut();
                let gone = m.open.pop();
                if m.open.last() != gone.as_ref() {
                    m.tiles.clear();
                    m.round = None;
                    m.compiled.clear();
                }
            });
        }
    }

    fn open_on(w: &World) -> bool {
        MEMO.with(|m| m.borrow().open.last().is_some_and(|b| b.0 == w as *const World as usize))
    }

    /// Whether a scope is open on this thread.
    pub fn open() -> bool {
        MEMO.with(|m| !m.borrow().open.is_empty())
    }

    /// A round's print: what a tile read looks at (each action's timing and flags, its walk, disengage, carried move,
    /// whom it aims at and whether it shoves).
    fn round_print(sched: &[ScheduledAction]) -> u64 {
        let mut h = Quick::default();
        sched.len().hash(&mut h);
        for a in sched {
            (a.unit, a.start, a.impact, a.end, a.cancelled, a.landed, a.broken, a.command.started, &a.command.skill, a.command.priority).hash(&mut h);
            std::mem::discriminant(&a.command.kind).hash(&mut h);
            match &a.command.kind {
                CmdKind::Walk { path, expected_start, goal, .. } => (path, expected_start, goal).hash(&mut h),
                CmdKind::Disengage { dest, .. } => dest.hash(&mut h),
                CmdKind::Attack { target, .. } | CmdKind::Area { target, .. } => target.hash(&mut h),
                _ => {}
            }
            match a.command.carry {
                crate::command::Carry::None => 0u8.hash(&mut h),
                crate::command::Carry::Engage(c) => (1u8, c).hash(&mut h),
                crate::command::Carry::Retreat(c) => (2u8, c).hash(&mut h),
            }
            a.command.forced.is_some().hash(&mut h);
        }
        h.finish()
    }

    pub(super) fn tile(w: &World, sched: &[ScheduledAction], q: (UnitId, i32, Option<i32>, bool), work: impl FnOnce() -> V2) -> V2 {
        if !open_on(w) {
            return work();
        }
        let key = (round_print(sched), q);
        if let Some(v) = MEMO.with(|m| m.borrow().tiles.get(&key).copied()) {
            return v;
        }
        let v = work();
        MEMO.with(|m| m.borrow_mut().tiles.insert(key, v));
        v
    }

    pub(super) fn round(w: &World, sched: &[ScheduledAction], plan: &crate::plan::Plan, work: impl FnOnce() -> Vec<ScheduledAction>) -> Vec<ScheduledAction> {
        if MEMO.with(|m| m.borrow().open.last() != Some(&id(w, sched, plan))) {
            return work();
        }
        if let Some(r) = MEMO.with(|m| m.borrow().round.clone()) {
            return r;
        }
        let r = work();
        MEMO.with(|m| m.borrow_mut().round = Some(r.clone()));
        r
    }

    pub(crate) fn compiled(w: &World, plan: &crate::plan::Plan, c: UnitId, work: impl FnOnce() -> crate::plan::SimState) -> crate::plan::SimState {
        let here = MEMO.with(|m| m.borrow().open.last().is_some_and(|b| b.0 == w as *const World as usize && b.2 == plan as *const crate::plan::Plan as usize));
        if !here {
            return work();
        }
        if let Some(st) = MEMO.with(|m| m.borrow().compiled.get(&c).cloned()) {
            return st;
        }
        let st = work();
        MEMO.with(|m| m.borrow_mut().compiled.insert(c, st.clone()));
        st
    }
}
