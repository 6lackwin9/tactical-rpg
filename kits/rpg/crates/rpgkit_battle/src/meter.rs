//! Unit meters (HOOKS1 9, UM1-UM7, rule `unit_meters`; engine-only, no Godot original). A unit holds a meter (a
//! `unit_meters` row) while a passive's `meter` effect applies; meters live in battle state only (`World::meters`, no
//! save key): each battle starts a meter at the larger of its `start` and the holder's floor (UM1). A skill's `meters`
//! change it at the command's start after the AP and HP cost (a negative delta the value does not cover fails the
//! command unpaid; the planners refuse it), the trigger effect `meter` and a rule's `meter_set` change it, and its
//! `decay` rows run at round end (UM2, UM3); values clamp to floor..max. The band holding the value (the shared
//! `rpgkit_rules::bands` helper) adds its `effects` and holds its `statuses` (granted for 1 round at each round start
//! and on entering, removed on leaving; UM4); a move across a rule's point runs the rule's effects (the trigger effects
//! plus `meter_set`; UM5), whose own meter changes cross no further rule (depth 1). The row's `skills` are open to a
//! holder without a slot while their cost is met (UM6). Every entry point reads the rule first, so classic battles hold
//! no meter and log, draw or record nothing new. U1.2-S5 (a pool from data): a ruleset's `unit_pools` meters are held by
//! every unit; a row's `max_stat` makes each holder's max its value of that derived stat (fixed at battle start; a
//! band may start at a percent of it, `from_pct`), `start_full` starts it full, `delta_max` bounds a `meter` effect, and
//! its `triggers` are held by every holder (`held_triggers`, rule triggers).

use crate::resolver::Resolver;
use crate::trigger::{self, Event, Fired, Holder};
use crate::world::{UnitId, World};
use aegis_core::value::{self, Obj};
use rpgkit_data::{MeterDelta, TriggerRow, UnitMeterDef};
use rpgkit_rules::bands;
use rpgkit_rules::derived;
use rpgkit_rules::pool::PoolView;
use rpgkit_rules::ruleset;
use rpgkit_rules::status;
use rpgkit_rules::unit::BandApplied;
use serde_json::Value;

/// The holder key prefix of a meter's own lines (the log tags them `[Meter]`).
pub const METER_KEY: &str = "meter:";

/// Whether the battle plays by unit meters.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::UNIT_METERS)
}

/// One meter one unit holds this battle: its value, floor and max (U1.2-S5: the row's max, or the holder's `max_stat`
/// value), whether a skill raised it this round (UM3), the band it is in and what that band's effects added (UM4), and
/// the `once` rules that fired (UM5).
#[derive(Debug, Clone, PartialEq)]
pub struct Gauge {
    pub unit: UnitId,
    pub meter: String,
    pub value: i32,
    pub floor: i32,
    pub max: i32,
    pub gained: bool,
    pub band: Option<usize>,
    applied: BandApplied,
    fired: Vec<String>,
}

/// The battle's meters (empty in classic and without the rule), holders in unit order, each unit's meters in the
/// order its passives gave them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Meters {
    pub gauges: Vec<Gauge>,
}

/// UM7: one meter as the screens show it (the unit card, inspect and the strip row): pips up to 10, else a bar.
#[derive(Debug, Clone, PartialEq)]
pub struct Shown {
    pub id: String,
    pub name: String,
    pub short: String,
    pub value: i32,
    pub min: i32,
    pub max: i32,
    /// One of `rpgkit_data::METER_SHOW` (`pips` above `METER_PIPS_MAX` reads as `bar`).
    pub show: String,
    /// The band's name ("" = none).
    pub band: String,
}

fn find(w: &World, c: UnitId, meter: &str) -> Option<usize> {
    w.meters.gauges.iter().position(|g| g.unit == c && g.meter == meter)
}

/// `c`'s value of `meter` (None when it holds none).
pub fn value(w: &World, c: UnitId, meter: &str) -> Option<i32> {
    find(w, c, meter).map(|i| w.meters.gauges[i].value)
}

/// `c`'s meters and their values now (the planner starts from them; a snapshot records them).
pub fn values(w: &World, c: UnitId) -> Vec<(String, i32)> {
    w.meters.gauges.iter().filter(|g| g.unit == c).map(|g| (g.meter.clone(), g.value)).collect()
}

/// UM4, U1.2-S5: where band `b` starts on a gauge whose max is `max`: its `from`, or its `from_pct` of the max.
fn band_from(b: &Obj, max: i32) -> i64 {
    if b.contains_key("from_pct") {
        return bands::pct_point(value::int(b, "from_pct", 0) as i64, max as i64);
    }
    value::int(b, "from", 0) as i64
}

/// U1.2-S5: `c`'s max of meter `def`: its value of the row's `max_stat` (clamped to the row's min..max), else the
/// row's max (also when the derived row is absent in the battle's ruleset).
fn max_of(w: &World, c: UnitId, def: &UnitMeterDef) -> i32 {
    if def.max_stat.is_empty() {
        return def.max;
    }
    derived::value_for(&w.cat, &w.ruleset, &w.units[c], &def.max_stat).map_or(def.max, |v| v.clamp(def.min, def.max))
}

/// U1.2-S5 (FB-04's generic store): `c`'s meter `meter` as a pool view (floor to max; None when it holds none).
pub fn pool(w: &World, c: UnitId, meter: &str) -> Option<PoolView> {
    find(w, c, meter).map(|i| {
        let g = &w.meters.gauges[i];
        PoolView { current: g.value, min: g.floor, max: g.max }
    })
}

fn band_name(b: &Obj) -> String {
    let n = value::string(b, "name", "");
    if n.is_empty() {
        value::string(b, "id", "")
    } else {
        n
    }
}

/// UM4: a band's held statuses (`{status, magnitude}`).
fn band_statuses(b: &Obj) -> Vec<(String, i32)> {
    value::array(b, "statuses").iter().filter_map(Value::as_object).map(|s| (value::string(s, "status", ""), value::int(s, "magnitude", 0))).filter(|(s, _)| !s.is_empty()).collect()
}

/// UM1: battle start (rule unit_meters): each unit's passive `meter` effects become its gauges (a second effect on
/// the same meter can only raise the floor), the rows' meter skills open to their holders (UM6), and each gauge enters
/// the band of its start value (its statuses wait for the round start). Nothing in classic.
pub fn battle_start(w: &mut World) {
    if !on(w) {
        return;
    }
    for c in 0..w.units.len() {
        unit_start(w, c);
    }
    for i in 0..w.meters.gauges.len() {
        settle_band(w, i, false);
    }
}

/// UM1 for one unit `c` (rule on): its passive `meter` effects and the ruleset's pools become its gauges, its meter
/// skills open (band entry is the caller's).
fn unit_start(w: &mut World, c: UnitId) {
    let cat = w.cat.clone();
    let pools = w.ruleset.numbers.unit_pools.clone();
    // U1.2-S5: the ruleset's pools after the passives' meters (start -1: the row's start)
    let held = w.units[c].passive_meters.clone().into_iter().map(|h| (h.meter, h.start, h.floor));
    for (meter, start, floor) in held.chain(pools.iter().map(|m| (m.clone(), -1, None))) {
        let Some(def) = cat.unit_meters.get(&meter) else { continue };
        let max = max_of(w, c, def);
        let floor = floor.unwrap_or(def.min).clamp(def.min, max);
        if let Some(i) = find(w, c, &meter) {
            let g = &mut w.meters.gauges[i];
            g.floor = g.floor.max(floor);
            g.value = g.value.max(g.floor);
            continue;
        }
        let start = match (start < 0, def.start_full) {
            (true, true) => max,
            (true, false) => def.start,
            _ => start,
        };
        let value = start.clamp(def.min, max).max(floor);
        w.meters.gauges.push(Gauge { unit: c, meter: meter.clone(), value, floor, max, gained: false, band: None, applied: BandApplied::default(), fired: Vec::new() });
        for s in def.skills.iter().filter(|s| cat.skill(s).is_some()) {
            if !w.units[c].meter_skills.contains(s) {
                w.units[c].meter_skills.push(s.clone());
            }
        }
    }
}

/// PHASEFX1: unit `c` became another row (a phase swap; rule unit_meters): the old row's gauges go, the new row's
/// start as at the battle start (UM1, each in the band of its start value). Nothing in classic.
pub fn restart_unit(w: &mut World, c: UnitId) {
    if !on(w) {
        return;
    }
    w.meters.gauges.retain(|g| g.unit != c);
    let first = w.meters.gauges.len();
    unit_start(w, c);
    for i in first..w.meters.gauges.len() {
        settle_band(w, i, false);
    }
}

/// UM4: the gauge `i` takes the band of its value: leaving one takes its effects off and removes its statuses,
/// entering one adds its effects and (with `grant`) grants its statuses for 1 round. Returns the line of a band change.
fn settle_band(w: &mut World, i: usize, grant: bool) -> Option<String> {
    let cat = w.cat.clone();
    let (c, meter, v, old, max) = {
        let g = &w.meters.gauges[i];
        (g.unit, g.meter.clone(), g.value, g.band, g.max)
    };
    let def = cat.unit_meters.get(&meter)?;
    let new = bands::band_index(def.bands.iter().map(|b| band_from(b, max)), v as i64);
    if new == old {
        return None;
    }
    let applied = std::mem::take(&mut w.meters.gauges[i].applied);
    // No band held before: nothing to take off (an empty source would match unrelated modifiers).
    if old.is_some() {
        w.units[c].remove_band(&applied);
    }
    if let Some(o) = old {
        for (st, _) in band_statuses(&def.bands[o]) {
            status::band_release(&mut w.units[c], &st);
        }
    }
    let mut applied = BandApplied::default();
    if let Some(n) = new {
        let b = &def.bands[n];
        applied = w.units[c].apply_band(&cat, value::array(b, "effects"), &format!("{METER_KEY}{meter}"));
        if grant {
            for (st, mag) in band_statuses(b) {
                status::band_hold(&mut w.units[c], &st, mag, &def.name);
            }
        }
    }
    let g = &mut w.meters.gauges[i];
    g.band = new;
    g.applied = applied;
    let name = &w.units[c].name;
    Some(match (new, old) {
        (Some(n), _) => format!("{name}'s {} enters {}", def.name, band_name(&def.bands[n])),
        (None, Some(o)) => format!("{name}'s {} leaves {}", def.name, band_name(&def.bands[o])),
        (None, None) => return None,
    })
}

/// UM4: round start (rule unit_meters), before the `round_start` triggers: each held band's statuses are granted for
/// this round.
pub fn round_start(w: &mut World) {
    if !on(w) {
        return;
    }
    let cat = w.cat.clone();
    for g in &w.meters.gauges {
        let (Some(n), Some(def)) = (g.band, cat.unit_meters.get(&g.meter)) else { continue };
        if !w.units[g.unit].is_active() {
            continue;
        }
        for (st, mag) in band_statuses(&def.bands[n]) {
            status::band_hold(&mut w.units[g.unit], &st, mag, &def.name);
        }
    }
}

/// The holder of a meter's own lines (`[Meter] <unit>'s <source>: ...`; `source` the skill or the meter's name).
fn holder(c: UnitId, source: &str, key: &str) -> Holder {
    let row = TriggerRow { on: "meter".into(), when: Obj::new(), effects: Vec::new(), limit: "none".into() };
    Holder { unit: c, source: source.to_string(), key: format!("{METER_KEY}{key}"), row }
}

fn delta_text(d: i32) -> String {
    if d > 0 {
        format!("+{d}")
    } else {
        d.to_string()
    }
}

/// UM2-UM5: moves the gauge `i` to `to` (clamped to floor..max). A move logs `<what> (old -> new)` under `h`, then the
/// band change, then (with `rules`) each crossed rule's effects in crossing order (`once` rules once a battle), their
/// own meter changes crossing no further rule. Returns whether the value moved.
#[allow(clippy::too_many_arguments)]
fn move_to(res: &mut Resolver, w: &mut World, i: usize, to: i32, h: &Holder, what: &str, rules: bool, block: i32, fired: &mut Fired) -> bool {
    let cat = w.cat.clone();
    let (meter, old, floor, max) = {
        let g = &w.meters.gauges[i];
        (g.meter.clone(), g.value, g.floor, g.max)
    };
    let Some(def) = cat.unit_meters.get(&meter) else { return false };
    let new = to.clamp(floor, max.max(floor));
    if new == old {
        return false;
    }
    w.meters.gauges[i].value = new;
    let l = trigger::line(w, h, &format!("{what} ({old} -> {new})"));
    res.log.push(l.clone());
    fired.push(l, 0, Vec::new());
    if let Some(b) = settle_band(w, i, true) {
        let l = trigger::line(w, h, &b);
        res.log.push(l.clone());
        fired.push(l, 0, Vec::new());
    }
    if rules {
        run_rules(res, w, i, def, old as i64, new as i64, block, fired);
    }
    true
}

/// UM5: the rules a move from `old` to `new` crosses (as CAMPAIGN1 M5: `at` a band id or a number, `dir` up by
/// default, `once` per battle) run their effects for the gauge's holder.
#[allow(clippy::too_many_arguments)]
fn run_rules(res: &mut Resolver, w: &mut World, i: usize, def: &UnitMeterDef, old: i64, new: i64, block: i32, fired: &mut Fired) {
    let max = w.meters.gauges[i].max;
    let bands_at: Vec<(&str, i64)> = def.bands.iter().map(|b| (b.get("id").and_then(Value::as_str).unwrap_or(""), band_from(b, max))).collect();
    let mut crossed: Vec<(i64, &Obj)> = def
        .rules
        .iter()
        .filter_map(|r| bands::point(r.get("at").unwrap_or(&Value::Null), bands_at.iter().copied()).map(|at| (at, r)))
        .filter(|(at, r)| bands::crosses(*at, value::string(r, "dir", "up") != "down", old, new))
        .collect();
    bands::crossing_order(&mut crossed, old, new, |(at, _)| *at);
    let c = w.meters.gauges[i].unit;
    for (_, r) in crossed {
        let id = value::string(r, "id", "");
        if value::boolean(r, "once", false) {
            if w.meters.gauges[i].fired.contains(&id) {
                continue;
            }
            w.meters.gauges[i].fired.push(id.clone());
        }
        if !w.units[c].is_active() {
            continue;
        }
        let h = holder(c, &def.name, &format!("{}#{id}", def.id));
        let effects: Vec<Obj> = value::array(r, "effects").iter().filter_map(Value::as_object).cloned().collect();
        let ev = Event::new("meter", c, None);
        trigger::run_effects(res, w, &ev, &h, &effects, block, false, fired);
    }
}

/// 8.2 `meter` and UM5 `meter_set` (run by `trigger::run_effects` for `to`): a trigger's meter change on a meter `to`
/// holds (`delta` within the row's `delta_max`, 10 by default), or a rule's set (`value`); the line is the holder's;
/// with `rules` the move may cross rules.
#[allow(clippy::too_many_arguments)]
pub fn from_effect(res: &mut Resolver, w: &mut World, h: &Holder, to: UnitId, e: &Obj, rules: bool, block: i32, fired: &mut Fired) {
    if !on(w) {
        return;
    }
    let meter = value::string(e, "meter", "");
    let Some(i) = find(w, to, &meter) else { return };
    let Some(def) = w.cat.unit_meters.get(&meter).cloned() else { return };
    let cur = w.meters.gauges[i].value;
    let (target, what) = if value::string(e, "type", "") == rpgkit_data::METER_SET {
        let v = value::int(e, "value", cur);
        (v, format!("{}'s {} set to {v}", w.units[to].name, def.name))
    } else {
        let d = value::int(e, "delta", 0).clamp(-def.delta_max, def.delta_max);
        if d == 0 {
            return;
        }
        (cur + d, format!("{}'s {} {}", w.units[to].name, def.name, delta_text(d)))
    };
    move_to(res, w, i, target, h, &what, rules, block, fired);
}

/// PK1-S1 (3.6): a hazard named `source` changes `c`'s meter by `d` (rule unit_meters; nothing when `c` holds no such
/// meter): within the row's `delta_max` like a trigger's `meter` effect, not a skill's gain (UM3's `unless_gained` is
/// untouched); its line is `[Meter] <unit>'s <source>: ...`, its move may cross rules. Its Fired joins the triggers the
/// session logs next.
pub fn from_hazard(res: &mut Resolver, w: &mut World, c: UnitId, d: &MeterDelta, source: &str, block: i32) {
    if !on(w) || !w.units[c].is_active() {
        return;
    }
    let Some(i) = find(w, c, &d.meter) else { return };
    let Some(def) = w.cat.unit_meters.get(&d.meter).cloned() else { return };
    let delta = d.delta.clamp(-def.delta_max, def.delta_max);
    if delta == 0 {
        return;
    }
    let h = holder(c, source, &format!("hazard:{source}"));
    let mut fired = Fired::new(c, "meter", None, source);
    let to = w.meters.gauges[i].value + delta;
    move_to(res, w, i, to, &h, &format!("{}'s {} {}", w.units[c].name, def.name, delta_text(delta)), true, block, &mut fired);
    if !fired.lines.is_empty() || !fired.moves.is_empty() {
        res.triggers.fired.push(fired);
    }
}

/// UM2: why `deltas` cannot be paid from the values `value_of` gives ("" = they can): a negative delta needs a value at
/// least its size (a meter the unit does not hold counts 0): `<meter name> 0 < 1`.
pub fn cost_problem(w: &World, deltas: &[MeterDelta], value_of: impl Fn(&str) -> Option<i32>) -> String {
    for d in deltas.iter().filter(|d| d.delta < 0) {
        let have = value_of(&d.meter).unwrap_or(0);
        if have < -d.delta {
            let name = w.cat.unit_meters.get(&d.meter).map_or_else(|| d.meter.clone(), |m| m.name.clone());
            return format!("{name} {have} < {}", -d.delta);
        }
    }
    String::new()
}

/// PK1-S2a (PK1 4.3 item 5, `requires_meter`): why a skill gated on `gate` cannot start at the values `value_of`
/// gives ("" = it can, or no gate): the meter below `min` (a meter the unit does not hold counts 0): `<meter name> 45
/// < 50`. Read only under the rule unit_meters.
pub fn gate_problem(w: &World, gate: Option<&rpgkit_data::MeterGate>, value_of: impl Fn(&str) -> Option<i32>) -> String {
    let Some(g) = gate.filter(|_| on(w)) else { return String::new() };
    let have = value_of(&g.meter).unwrap_or(0);
    if have >= g.min {
        return String::new();
    }
    let name = w.cat.unit_meters.get(&g.meter).map_or_else(|| g.meter.clone(), |m| m.name.clone());
    format!("{name} {have} < {}", g.min)
}

/// UM2 for the planners: `c`'s planned meter values after paying `deltas` (clamped to its floor..max; a meter it does
/// not hold stays out).
pub fn spend(w: &World, c: UnitId, values: &mut [(String, i32)], deltas: &[MeterDelta]) {
    for d in deltas {
        let Some(i) = find(w, c, &d.meter) else { continue };
        let (floor, max) = (w.meters.gauges[i].floor, w.meters.gauges[i].max);
        if let Some(v) = values.iter_mut().find(|(m, _)| *m == d.meter) {
            v.1 = (v.1 + d.delta).clamp(floor, max.max(floor));
        }
    }
}

/// UM2: the action `i` starts (after its AP and HP cost): its skill's meter changes in list order (a rise marks the
/// meter gained this round, UM3). Logged `[Meter] <unit>'s <skill>: <meter> -1 (1 -> 0)`.
pub fn pay(res: &mut Resolver, w: &mut World, i: usize) {
    let (c, deltas, name, block) = {
        let a = &res.scheduled[i];
        (a.unit, a.command.meters.clone(), a.command.name.clone(), a.start)
    };
    if deltas.is_empty() || !on(w) {
        return;
    }
    let h = holder(c, &name, &name);
    let mut fired = Fired::new(c, "meter", None, &name);
    for d in deltas {
        let Some(g) = find(w, c, &d.meter) else { continue };
        if d.delta > 0 {
            w.meters.gauges[g].gained = true;
        }
        let meter = w.cat.unit_meters.get(&d.meter).map_or_else(|| d.meter.clone(), |m| m.name.clone());
        let to = w.meters.gauges[g].value + d.delta;
        move_to(res, w, g, to, &h, &format!("{meter} {}", delta_text(d.delta)), true, block, &mut fired);
    }
    if !fired.lines.is_empty() || !fired.moves.is_empty() {
        res.triggers.fired.push(fired);
    }
}

/// UM2: the action `i` is about to start: when its unit cannot pay its meter cost now it fails unpaid (no AP, HP or
/// meter is paid; the action is cancelled and logged). Returns whether it failed.
pub fn unpaid(res: &mut Resolver, w: &World, i: usize) -> bool {
    let a = &res.scheduled[i];
    if (a.command.meters.is_empty() && a.command.gate.is_none()) || !on(w) {
        return false;
    }
    let c = a.unit;
    // PK1-S2a: a meter gate (`requires_meter`) is tested first, at the start, before any cost
    let gate = gate_problem(w, a.command.gate.as_ref(), |m| value(w, c, m));
    let why = if gate.is_empty() { cost_problem(w, &a.command.meters, |m| value(w, c, m)) } else { gate };
    if why.is_empty() {
        return false;
    }
    let name = a.command.name.clone();
    res.scheduled[i].cancelled = true;
    let h = holder(c, &name, &name);
    let mut fired = Fired::new(c, "meter", None, &name);
    let l = trigger::line(w, &h, &format!("fails unpaid ({why})"));
    res.log.push(l.clone());
    fired.push(l, 0, Vec::new());
    res.triggers.fired.push(fired);
    true
}

/// UM3: whether each unit guarded this round (a Defend, or a held guard), read before the `round_end` triggers end the
/// held guards (PR5-26: the decay runs after them and reads this).
pub fn guarded_now(res: &Resolver, w: &World) -> Vec<bool> {
    (0..w.units.len()).map(|c| res.scheduled.iter().any(|a| a.unit == c && !a.cancelled && a.command.is_defend()) || w.units[c].held_guard.is_some()).collect()
}

/// UM3: round end (rule unit_meters), after the ticks and ground hazards and (PR5-26, FB-191 Option A) after the
/// `round_end` triggers, so a meter's own `round_end` row (a crisis `at_least` its max) reads the round's value before
/// it decays: each active holder's decay rows in order (`unless_gained` skips a row when a skill raised the meter this
/// round, `if_guarded` applies it only when the unit guarded this round: a Defend, or a held guard, `guarded` read by
/// `guarded_now` before the triggers); then the gained marks clear. PK1-S2a (PK1 4.3 items 5, 8): a unit's Channel round
/// skips every decay row of the channel meter (it counts as a gain, and its `if_guarded` rows skip too).
pub fn round_end(res: &mut Resolver, w: &mut World, guarded: &[bool]) {
    if !on(w) {
        return;
    }
    let cat = w.cat.clone();
    let block = w.total_blocks;
    for i in 0..w.meters.gauges.len() {
        let (c, meter, gained) = {
            let g = &w.meters.gauges[i];
            (g.unit, g.meter.clone(), g.gained)
        };
        w.meters.gauges[i].gained = false;
        let channel_round = crate::channel::is_channel(w, c) && meter == crate::channel::meter_id(w);
        let Some(def) = cat.unit_meters.get(&meter) else { continue };
        if def.decay.is_empty() || !w.units[c].is_active() || channel_round {
            continue;
        }
        let guarded = guarded.get(c).copied().unwrap_or(false);
        let h = holder(c, &def.name, &def.id);
        let mut fired = Fired::new(c, "meter", None, &def.name);
        for d in &def.decay {
            let delta = value::int(d, "delta", 0);
            if delta == 0 || (value::boolean(d, "unless_gained", false) && gained) || (value::boolean(d, "if_guarded", false) && !guarded) {
                continue;
            }
            let to = w.meters.gauges[i].value + delta;
            move_to(res, w, i, to, &h, &format!("decays {}", delta_text(delta)), true, block, &mut fired);
        }
        if !fired.lines.is_empty() || !fired.moves.is_empty() {
            res.triggers.fired.push(fired);
        }
    }
}

/// U1.2-S5: the trigger rows of the meters `c` holds (a row's `triggers`, in gauge then list order; key
/// `meter:<meter>#<n>`, source the meter's name). Empty without the rule.
pub fn held_triggers(w: &World, c: UnitId) -> Vec<Holder> {
    if !on(w) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for g in w.meters.gauges.iter().filter(|g| g.unit == c) {
        let Some(def) = w.cat.unit_meters.get(&g.meter) else { continue };
        for (n, row) in def.triggers.iter().enumerate() {
            out.push(Holder { unit: c, source: def.name.clone(), key: format!("{METER_KEY}{}#{n}", def.id), row: row.clone() });
        }
    }
    out
}

/// UM6: `c`'s meter skills whose cost its meters meet now (offered without a slot; empty without the rule).
pub fn offered(w: &World, c: UnitId) -> Vec<String> {
    let u = &w.units[c];
    u.meter_skills
        .iter()
        .filter(|id| w.cat.skill(id).is_some_and(|s| cost_problem(w, crate::skill::meters_by(s, &w.ruleset), |m| value(w, c, m)).is_empty()))
        .cloned()
        .collect()
}

/// UM7: `c`'s meters as the screens show them (none without the rule; `hidden` meters left out).
pub fn shown(w: &World, c: UnitId) -> Vec<Shown> {
    let mut out = Vec::new();
    for g in w.meters.gauges.iter().filter(|g| g.unit == c) {
        let Some(def) = w.cat.unit_meters.get(&g.meter) else { continue };
        if def.show == "hidden" {
            continue;
        }
        let show = if def.show == "pips" && g.max - def.min > rpgkit_data::METER_PIPS_MAX { "bar".to_string() } else { def.show.clone() };
        let band = g.band.and_then(|n| def.bands.get(n)).map(band_name).unwrap_or_default();
        out.push(Shown { id: def.id.clone(), name: def.name.clone(), short: def.short.clone(), value: g.value, min: def.min, max: g.max, show, band });
    }
    out
}
