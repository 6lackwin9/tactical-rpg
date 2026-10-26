//! Portrait cue evaluation (PORTRAITFX1-S, docs/specs/PORTRAITFX1.md 5.2-5.3; engine-only, no Godot original): which
//! state rows of the cue table hold for a unit's look on a surface, which event rows a step's events fire, and the
//! events of one battle step (hit, crit, guard hit, heal, break) read from the HP before and after and the step's log
//! lines. Pure and read-only: no RNG, no state; the screens turn the rows into the engine's portrait cues.

use crate::session::Session;
use rpgkit_data::PortraitCue;
use rpgkit_rules::unit::Stance;

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
}

/// Whether state row `row`'s trigger matches `look` (event rows never do): `status` when the unit has the status,
/// `condition` when it carries the condition, `hp` when `max_hp` > 0 and `hp x 100 < below x max_hp`, `down` when
/// down, `derived` with `below` when the value with statuses is below it, with `drop_pct` when the value without
/// statuses is above 0 and `with x 100 <= without x (100 - drop_pct)`.
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
        _ => false,
    }
}

/// The state rows (every trigger but `event`) that hold for `look` on `surface`: the trigger matches, the row's
/// surfaces include `surface` and its rulesets `look.ruleset`. In row order.
pub fn holds<'a>(rows: &'a [PortraitCue], look: &Look, surface: &str) -> Vec<&'a PortraitCue> {
    rows.iter().filter(|r| !r.is_event() && r.on_surface(surface) && r.in_ruleset(&look.ruleset) && matches(r, look)).collect()
}

/// The event rows `events` fire on `surface` under `ruleset`: for each event in order, its rows in row order.
pub fn reactions<'a>(rows: &'a [PortraitCue], events: &[&str], surface: &str, ruleset: &str) -> Vec<&'a PortraitCue> {
    events.iter().flat_map(|ev| rows.iter().filter(move |r| r.is_event() && r.event == *ev && r.on_surface(surface) && r.in_ruleset(ruleset))).collect()
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
/// <action> is broken`, `poise::break_line`).
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
/// before the step, in unit order; units past its end are left out). Reads the session only.
pub fn session_step_events(s: &Session, hp_before: &[i32], lines: &[String]) -> Vec<(usize, &'static str)> {
    let units: Vec<StepUnit> = s.w.units.iter().zip(hp_before).map(|(u, &before)| StepUnit { name: &u.name, hp_before: before, hp_after: u.hp, guarded: u.guard > 0.0 || u.stance == Stance::Defensive }).collect();
    step_events(&units, lines)
}
