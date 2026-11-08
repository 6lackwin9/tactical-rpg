//! Portrait cue evaluation (PORTRAITFX1-S, docs/specs/PORTRAITFX1.md 5.2-5.3; engine-only, no Godot original): which
//! state rows of the cue table hold for a unit's look on a surface, which event rows a step's events fire, and the
//! events of one battle step (hit, crit, guard hit, heal, break) read from the HP before and after and the step's log
//! lines. PORTRAITFX2 5.3: the battle's own events (its start and an ambush, a round's start, a skill's wind-up, an ally
//! going down, the player's win) and the `unspent` state. Pure and read-only: no RNG, no state; the screens turn the
//! rows into the engine's portrait cues.

use crate::command::CmdKind;
use crate::session::Session;
use rpgkit_data::{PortraitCue, PORTRAIT_SKILL_EVENT};
use rpgkit_rules::unit::Stance;

/// PORTRAITFX2 5.3: the battle events (beside the step events of `step_events`).
pub const BATTLE_START: &str = "battle_start";
pub const AMBUSH: &str = "ambush";
pub const ROUND_START: &str = "round_start";
pub const SKILL: &str = PORTRAIT_SKILL_EVENT;
pub const ALLY_DOWN: &str = "ally_down";
pub const VICTORY: &str = "victory";
/// The encounter deployment that fires `ambush`.
pub const AMBUSH_DEPLOYMENT: &str = "ambush";

/// What the cue rows read of a unit (5.2), built by the game from a battle unit (with its member's conditions), a
/// party member (no statuses; HP as the save carries it) or a world character (no HP: `max_hp` 0, so no `hp` row
/// matches, and not `down`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Look {
    pub hp: i32,
    pub max_hp: i32,
    pub down: bool,
    pub statuses: Vec<String>,
    pub conditions: Vec<String>,
    /// Derived stats: (id, the value with status modifiers, the value without).
    pub derived: Vec<(String, i32, i32)>,
    pub ruleset: String,
    /// PORTRAITFX2 5.3: a party member's unspent points (`unspent` rows hold above 0); 0 in battle and for a world
    /// character.
    pub unspent: i64,
}

/// Whether state row `row`'s trigger matches `look` (event rows never do): `status` when the unit has the status,
/// `condition` when it carries the condition, `hp` when `max_hp` > 0 and `hp x 100 < below x max_hp`, `down` when
/// down, `derived` with `below` when the value with statuses is below it, with `drop_pct` when the value without
/// statuses is above 0 and `with x 100 <= without x (100 - drop_pct)`, `unspent` when the look has unspent points.
pub fn matches(row: &PortraitCue, look: &Look) -> bool {
    match row.trigger.as_str() {
        "status" => look.statuses.contains(&row.status),
        "condition" => look.conditions.contains(&row.condition),
        "hp" => look.max_hp > 0 && (look.hp as i64) * 100 < (row.below as i64) * (look.max_hp as i64),
        "down" => look.down,
        "derived" => look.derived.iter().find(|d| d.0 == row.stat).is_some_and(|&(_, with, without)| {
            if row.below > 0 {
                with < row.below
            } else {
                without > 0 && (with as i64) * 100 <= (without as i64) * (100 - row.drop_pct as i64)
            }
        }),
        "unspent" => look.unspent > 0,
        _ => false,
    }
}

/// The state rows (every trigger but `event`) that hold for `look` on `surface`: the trigger matches, the row's
/// surfaces include `surface` and its rulesets `look.ruleset`. In row order.
pub fn holds<'a>(rows: &'a [PortraitCue], look: &Look, surface: &str) -> Vec<&'a PortraitCue> {
    rows.iter().filter(|r| !r.is_event() && r.on_surface(surface) && r.in_ruleset(&look.ruleset) && matches(r, look)).collect()
}

/// The event rows `events` fire on `surface` under `ruleset`: for each event in order, its rows in row order (a
/// `skill` row only when its `min_level` is 0: `reactions_at` gives the skill's level).
pub fn reactions<'a>(rows: &'a [PortraitCue], events: &[&str], surface: &str, ruleset: &str) -> Vec<&'a PortraitCue> {
    reactions_at(rows, events, 0, surface, ruleset)
}

/// `reactions` for events of a skill of level `level` (PORTRAITFX2 5.3): a `skill` row fires when `level` is at least
/// its `min_level`; the other events' rows do not read it.
pub fn reactions_at<'a>(rows: &'a [PortraitCue], events: &[&str], level: i32, surface: &str, ruleset: &str) -> Vec<&'a PortraitCue> {
    events.iter().flat_map(|ev| rows.iter().filter(move |r| r.is_event() && r.event == *ev && (r.event != SKILL || level >= r.min_level) && r.on_surface(surface) && r.in_ruleset(ruleset))).collect()
}

/// One unit as a step left it, for `step_events`.
#[derive(Debug, Clone, PartialEq)]
pub struct StepUnit<'a> {
    /// The unit's name as the log lines name it.
    pub name: &'a str,
    pub hp_before: i32,
    pub hp_after: i32,
    /// Guarding when the step ended (the battle screen's `hurt_guarded` case: a guard up or a defensive stance).
    pub guarded: bool,
}

/// Whether `line` is an attack line that crits the unit named `name`: `struck <name> for <n> CRIT` (a single attack)
/// or `<name> <n> CRIT` as an area attack's part (after its opening parenthesis or a comma). Two units with the same
/// name cannot be told apart by the lines.
fn crit_on(line: &str, name: &str) -> bool {
    if name.is_empty() || !line.starts_with("[Attack]") {
        return false;
    }
    let mut from = 0;
    while let Some(at) = line[from..].find(name).map(|i| i + from) {
        from = at + name.len();
        let before = &line[..at];
        if !(before.ends_with("struck ") || before.ends_with('(') || before.ends_with(", ")) {
            continue;
        }
        let after = &line[from..];
        let Some(rest) = after.strip_prefix(" for ").or_else(|| after.strip_prefix(' ')) else { continue };
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits > 0 && rest[digits..].starts_with(" CRIT") {
            return true;
        }
    }
    false
}

/// Whether `line` is the break line of the unit named `name` (`[Break] <name> is broken` or `[Break] <name>'s
/// <action> is broken`, `poise::break_line`; either may end with " (Stagger queued)", R1-BAL).
fn break_on(line: &str, name: &str) -> bool {
    let rest = line.strip_prefix("[Break] ").and_then(|r| r.strip_prefix(name));
    !name.is_empty() && rest.is_some_and(|r| r.starts_with(" is broken") || r.starts_with("'s "))
}

/// The portrait events of one battle step (5.3), unit by unit in index order: a unit whose HP fell gets one of `crit`
/// (a step's attack line crits it), `guard_hit` (it was guarding) or `hit`; a unit whose HP rose gets `heal`; a unit a
/// `[Break]` line names also gets `break`. `lines` are the step's resolver log lines.
pub fn step_events(units: &[StepUnit], lines: &[String]) -> Vec<(usize, &'static str)> {
    let mut out = Vec::new();
    for (i, u) in units.iter().enumerate() {
        if u.hp_after < u.hp_before {
            let event = if lines.iter().any(|l| crit_on(l, u.name)) {
                "crit"
            } else if u.guarded {
                "guard_hit"
            } else {
                "hit"
            };
            out.push((i, event));
        } else if u.hp_after > u.hp_before {
            out.push((i, "heal"));
        }
        if lines.iter().any(|l| break_on(l, u.name)) {
            out.push((i, "break"));
        }
    }
    out
}

/// `step_events` over a session after a step: its units' names, HP and guard, against `hp_before` (each unit's HP
/// before the step, in unit order; units past its end are left out), then its `ally_down_events`. Reads the session
/// only.
pub fn session_step_events(s: &Session, hp_before: &[i32], lines: &[String]) -> Vec<(usize, &'static str)> {
    let units: Vec<StepUnit> = s.w.units.iter().zip(hp_before).map(|(u, &before)| StepUnit { name: &u.name, hp_before: before, hp_after: u.hp, guarded: u.guard > 0.0 || u.stance == Stance::Defensive }).collect();
    let mut out = step_events(&units, lines);
    out.extend(ally_down_events(s, hp_before));
    out
}

/// Whether unit `i` of `s` stands: alive and not withdrawn.
fn living(s: &Session, i: usize) -> bool {
    s.w.units.get(i).is_some_and(|u| u.hp > 0 && !u.is_withdrawn)
}

/// Whether unit `i` is on the player's side (the allies, guests included).
fn player_side(s: &Session, i: usize) -> bool {
    s.w.allies.contains(&i)
}

/// PORTRAITFX2 5.3 `battle_start` and `ambush`: every living unit gets `battle_start`; in an encounter whose
/// `deployment` is `ambush` the player side's living units then get `ambush`. Unit by unit in index order.
pub fn start_events(s: &Session) -> Vec<(usize, &'static str)> {
    let alive: Vec<usize> = (0..s.w.units.len()).filter(|&i| living(s, i)).collect();
    let mut out: Vec<(usize, &'static str)> = alive.iter().map(|&i| (i, BATTLE_START)).collect();
    if s.def.deployment == AMBUSH_DEPLOYMENT {
        out.extend(alive.iter().filter(|&&i| player_side(s, i)).map(|&i| (i, AMBUSH)));
    }
    out
}

/// PORTRAITFX2 5.3 `round_start` (decision Q7: classic only): every living unit when the session's round is past
/// `round_before`; none under the round intent rule (the timeline ruleset plays no visible rounds).
pub fn round_events(s: &Session, round_before: i32) -> Vec<(usize, &'static str)> {
    if crate::round_intent::on(&s.w) || s.current_round <= round_before {
        return Vec::new();
    }
    (0..s.w.units.len()).filter(|&i| living(s, i)).map(|i| (i, ROUND_START)).collect()
}

/// PORTRAITFX2 5.3 `skill`: the acting unit and its skill's level when the action the last step popped (this round's,
/// not cancelled) is a skill command (an attack, an area, a support or a focus whose skill row has the command's name,
/// so at its rank); None otherwise. The screen fires it at the step's wind-up with `reactions_at`.
pub fn skill_event(s: &Session) -> Option<(usize, i32)> {
    let (round, _, idx) = s.last_popped.filter(|p| p.0 == s.current_round)?;
    let act = s.resolver.scheduled.get(idx).filter(|a| !a.cancelled && round == s.current_round)?;
    if !matches!(act.command.kind, CmdKind::Attack { .. } | CmdKind::Area { .. } | CmdKind::Support { .. } | CmdKind::Focus { .. }) {
        return None;
    }
    let def = s.w.cat.skills.iter().find(|k| k.name == act.command.name)?;
    Some((act.unit, def.level))
}

/// PORTRAITFX2 5.3 `ally_down`: for each unit that went down in the step (HP above 0 in `hp_before`, 0 or less now),
/// every other living unit of its side, once per unit per step (index order).
pub fn ally_down_events(s: &Session, hp_before: &[i32]) -> Vec<(usize, &'static str)> {
    let fell: Vec<usize> = s.w.units.iter().zip(hp_before).enumerate().filter(|(_, (u, &before))| before > 0 && u.hp <= 0).map(|(i, _)| i).collect();
    (0..s.w.units.len()).filter(|&j| living(s, j) && fell.iter().any(|&f| f != j && player_side(s, f) == player_side(s, j))).map(|j| (j, ALLY_DOWN)).collect()
}

/// PORTRAITFX2 5.3 `victory`: the player side's living units when the step ended the battle in the player's win (the
/// outcome is `won` now and was `outcome_before` before the step).
pub fn victory_events(s: &Session, outcome_before: &str) -> Vec<(usize, &'static str)> {
    if s.outcome != "won" || outcome_before == "won" {
        return Vec::new();
    }
    (0..s.w.units.len()).filter(|&i| living(s, i) && player_side(s, i)).map(|i| (i, VICTORY)).collect()
}
