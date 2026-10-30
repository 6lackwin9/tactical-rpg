//! The battle report (BattleReport, scripts/combat/session/battle_report.gd; BREPORT1-L 1.4, rules 4-7): the per-hero
//! table and the best-moment line of the results screen, built once from a finished BattleMetrics. Rows come from the
//! party side only (heroes, then guests); units that never deployed are not in the battle.

use crate::metrics::{counts_json, BattleMetrics, Detail, UnitRecord, COMBO_KINDS};
use crate::session::Session;
use rpgkit_rules::status;
use indexmap::IndexMap;
use serde_json::{json, Value};

/// A JSON key and the type check its value must pass (problems).
type KeyCheck<'a> = (&'a str, fn(&Value) -> bool);

pub const VERSION: i64 = 1;
/// Table order (spec section "What the report shows").
pub const COLUMNS: [&str; 6] = ["name", "dealt", "taken", "healed", "effects", "combos"];
/// Rule 5: the other columns show only when some row is non-zero.
pub const ALWAYS: [&str; 3] = ["name", "dealt", "taken"];
pub const NUMBERS: [&str; 10] = ["dealt", "dealt_dot", "ff_dealt", "taken", "taken_dot", "healed", "regen", "effects", "combos", "kills"];
pub const STATES: [&str; 3] = ["standing", "down", "left"];

pub fn combo_label(kind: &str) -> &'static str {
    match kind {
        "exploit" => "Exploit",
        "break" => "Stance break",
        _ => "Flow",
    }
}

/// One hero's row.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The hero's stable key (Session::hero_key; "" for a guest).
    pub key: String,
    pub name: String,
    pub guest: bool,
    /// "standing", "down" or "left".
    pub state: &'static str,
    pub dealt: i32,
    pub dealt_dot: i32,
    pub ff_dealt: i32,
    pub taken: i32,
    pub taken_dot: i32,
    pub healed: i32,
    pub regen: i32,
    pub effects: i32,
    pub combos: i32,
    pub kills: i32,
    pub status_counts: IndexMap<String, i32>,
    pub combo_counts: IndexMap<String, i32>,
}

impl Row {
    /// A column's number (0 for "name" and unknown ids).
    pub fn number(&self, id: &str) -> i32 {
        match id {
            "dealt" => self.dealt,
            "dealt_dot" => self.dealt_dot,
            "ff_dealt" => self.ff_dealt,
            "taken" => self.taken,
            "taken_dot" => self.taken_dot,
            "healed" => self.healed,
            "regen" => self.regen,
            "effects" => self.effects,
            "combos" => self.combos,
            "kills" => self.kills,
            _ => 0,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({"ref": self.key, "name": self.name, "guest": self.guest, "state": self.state, "dealt": self.dealt, "dealt_dot": self.dealt_dot,
            "ff_dealt": self.ff_dealt, "taken": self.taken, "taken_dot": self.taken_dot, "healed": self.healed, "regen": self.regen,
            "effects": self.effects, "combos": self.combos, "kills": self.kills, "status_counts": counts_json(&self.status_counts), "combo_counts": counts_json(&self.combo_counts)})
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub outcome: String,
    pub rounds: i32,
    pub rows: Vec<Row>,
    pub columns: Vec<&'static str>,
    /// The best-moment line ("" = none).
    pub best: String,
}

impl Report {
    /// The report of the fight `m` followed in `s`.
    pub fn build(m: &BattleMetrics, s: &Session, outcome: &str) -> Self {
        let blank_rec = UnitRecord::default();
        let blank_det = Detail::default();
        let mut rows = Vec::new();
        let mut details = Vec::new();
        for &c in &s.w.allies {
            let u = &s.w.units[c];
            let rec = m.units.get(&u.name).unwrap_or(&blank_rec);
            let det = m.detail.get(&u.name);
            let state = if u.is_withdrawn {
                "left"
            } else if u.is_alive() {
                "standing"
            } else {
                "down"
            };
            let d = det.unwrap_or(&blank_det);
            rows.push(Row {
                key: s.hero_key(c),
                name: u.name.clone(),
                guest: s.guests.contains(&c),
                state,
                dealt: rec.dmg_dealt + rec.dot_dealt,
                dealt_dot: rec.dot_dealt,
                ff_dealt: rec.ff_dealt,
                taken: rec.dmg_taken + rec.dot_taken + rec.ff_taken,
                taken_dot: rec.dot_taken,
                healed: rec.heal_done,
                regen: d.regen_done,
                effects: rec.buffs_applied + rec.debuffs_applied,
                combos: rec.combos,
                kills: rec.kills,
                // A unit with no detail has empty counts (Godot read {} for it).
                status_counts: det.map(|d| d.status_counts.clone()).unwrap_or_default(),
                combo_counts: det.map(|d| d.combo_counts.clone()).unwrap_or_default(),
            });
            details.push(det.cloned().unwrap_or_default());
        }
        let columns = columns_for(&rows);
        let best = best_moment(&rows, &details);
        Self { outcome: outcome.to_string(), rounds: s.current_round, rows, columns, best }
    }

    pub fn to_json(&self) -> Value {
        json!({"version": VERSION, "outcome": self.outcome, "rounds": self.rounds, "rows": self.rows.iter().map(Row::to_json).collect::<Vec<_>>(),
            "columns": self.columns, "best": self.best})
    }
}

/// Rule 5: name, dealt, taken always; healed, effects, combos only when some row has a non-zero value.
pub fn columns_for(rows: &[Row]) -> Vec<&'static str> {
    COLUMNS.iter().copied().filter(|id| ALWAYS.contains(id) || rows.iter().any(|r| r.number(id) != 0)).collect()
}

/// Rule 6: a save, else a combo finish, else the biggest hit, else "". `details[i]` belongs to `rows[i]`. Within a rule
/// the strongest candidate wins (save: the lowest HP before the heal; hit: the largest amount), then the earliest.
pub fn best_moment(rows: &[Row], details: &[Detail]) -> String {
    let pairs = || rows.iter().zip(details);
    // (score, order, who) per candidate: a higher score wins, then the lower order.
    fn pick<T: Clone>(cands: impl Iterator<Item = (i32, i32, String, T)>) -> Option<(String, T)> {
        let mut best: Option<(i32, i32, String, T)> = None;
        for c in cands {
            if best.as_ref().is_none_or(|b| c.0 > b.0 || (c.0 == b.0 && c.1 < b.1)) {
                best = Some(c);
            }
        }
        best.map(|b| (b.2, b.3))
    }
    if let Some((who, s)) = pick(pairs().filter_map(|(r, d)| d.best_save.clone().map(|s| (-s.hp, s.order, r.name.clone(), s)))) {
        return format!("{who} healed {} at {} HP, and {} made it through.", s.ally, s.hp, s.ally);
    }
    if let Some((who, f)) = pick(pairs().filter_map(|(r, d)| d.best_finish.clone().map(|f| (0, f.order, r.name.clone(), f)))) {
        // HOOKS1 EX1: an execute entry (its label is the rule id, no status) reads "executed"
        let how = match (f.combo.as_str(), f.status.as_str()) {
            ("break", _) => "broke the guard".to_string(),
            (_, crate::skill::EXECUTE) => "executed".to_string(),
            _ => format!("exploited {}", status::display_name(&f.status)),
        };
        return format!("{who} finished {} with {} ({how}).", f.target, f.skill);
    }
    if let Some((who, h)) = pick(pairs().filter_map(|(r, d)| d.best_hit.clone().map(|h| (h.amount, h.order, r.name.clone(), h)))) {
        if h.amount >= 1 {
            return format!("{who}'s {} hit {} for {}.", h.skill, h.target, h.amount);
        }
    }
    String::new()
}

// ---- Tips (shown on hover) -------------------------------------------------------------------------------------------

pub fn dealt_tip(r: &Row) -> String {
    let mut tip = format!("{} from hits, {} from bleed, poison and burn.", r.dealt - r.dealt_dot, r.dealt_dot);
    if r.ff_dealt > 0 {
        tip += &format!(" Hit allies for {}.", r.ff_dealt);
    }
    tip
}

pub fn taken_tip(r: &Row) -> String {
    format!("{} from hits, {} from effects.", r.taken - r.taken_dot, r.taken_dot)
}

pub fn healed_tip(r: &Row) -> String {
    if r.regen > 0 {
        format!("Includes {} from regeneration.", r.regen)
    } else {
        String::new()
    }
}

/// "Bleed x2, Slow x1, Haste on allies x1": every status put on, buffs marked "on allies".
pub fn effects_tip(r: &Row) -> String {
    let buffs = status::buffs();
    r.status_counts
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(s, n)| format!("{}{} x{n}", status::display_name(s), if buffs.contains(&s.as_str()) { " on allies" } else { "" }))
        .collect::<Vec<_>>()
        .join(", ")
}

/// "Exploit x1, Stance break x1, Flow x2" (kinds with a count only).
pub fn combos_tip(r: &Row) -> String {
    COMBO_KINDS
        .iter()
        .filter_map(|k| r.combo_counts.get(*k).filter(|n| **n > 0).map(|n| format!("{} x{n}", combo_label(k))))
        .collect::<Vec<_>>()
        .join(", ")
}

// ---- Validation (1.4) ------------------------------------------------------------------------------------------------

/// Problems of a report read back as JSON (a save, a results screen): missing keys, wrong types, negative numbers,
/// unknown column ids or states; empty = usable.
pub fn problems(report: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let Some(o) = report.as_object() else { return vec!["the report is not an object".into()] };
    let top: [KeyCheck; 6] =
        [("version", Value::is_i64), ("outcome", Value::is_string), ("rounds", Value::is_i64), ("rows", Value::is_array), ("columns", Value::is_array), ("best", Value::is_string)];
    for (key, ok) in top {
        match o.get(key) {
            None => out.push(format!("missing key '{key}'")),
            Some(v) if !ok(v) => out.push(format!("'{key}' has the wrong type")),
            _ => {}
        }
    }
    if !out.is_empty() {
        return out;
    }
    if o["rounds"].as_i64().unwrap_or(0) < 0 {
        out.push("'rounds' is negative".into());
    }
    for c in o["columns"].as_array().into_iter().flatten() {
        if !c.as_str().is_some_and(|s| COLUMNS.contains(&s)) {
            out.push(format!("unknown column '{}'", c.as_str().map_or_else(|| c.to_string(), str::to_string)));
        }
    }
    let mut row_types: Vec<KeyCheck> =
        vec![("ref", Value::is_string), ("name", Value::is_string), ("guest", Value::is_boolean), ("state", Value::is_string), ("status_counts", Value::is_object), ("combo_counts", Value::is_object)];
    for id in NUMBERS {
        row_types.push((id, Value::is_i64));
    }
    for (i, raw) in o["rows"].as_array().into_iter().flatten().enumerate() {
        let Some(row) = raw.as_object() else {
            out.push(format!("row {i} is not a dictionary"));
            continue;
        };
        for (key, ok) in &row_types {
            match row.get(*key) {
                None => out.push(format!("row {i}: missing key '{key}'")),
                Some(v) if !ok(v) => out.push(format!("row {i}: '{key}' has the wrong type")),
                Some(v) if NUMBERS.contains(key) && v.as_i64().unwrap_or(0) < 0 => out.push(format!("row {i}: '{key}' is negative")),
                _ => {}
            }
        }
        if let Some(st) = row.get("state").and_then(Value::as_str) {
            if !STATES.contains(&st) {
                out.push(format!("row {i}: unknown state '{st}'"));
            }
        }
    }
    out
}
