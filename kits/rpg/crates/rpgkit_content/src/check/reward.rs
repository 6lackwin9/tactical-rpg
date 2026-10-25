//! REWARD1 checks (spec section 10): a unit's loot table, a world encounter's key loot and objective reward. The purse
//! itself stays in `encounter.rs` (PurseRules), which refuses the retired `purse.loot`.

use super::encounter::world_loot_check;
use super::View;
use aegis_core::json::{self, gstr, Dict};
use serde_json::Value;

/// Most items one loot row drops.
pub const LOOT_COUNT_MAX: i64 = 9;
/// The chances of a table add up to at most this (the rest drops nothing).
pub const LOOT_CHANCE_TOTAL: i64 = 100;
/// Most an objective reward pays (`purse::GOLD_CAP`), and the gold above which it warns.
const OBJECTIVE_GOLD_MAX: i64 = 500;
const OBJECTIVE_GOLD_SOFT: i64 = 100;
const LOOT_ROW_KEYS: [&str; 3] = ["item", "chance", "count"];
const KEY_LOOT_KEYS: [&str; 2] = ["loot", "kill"];
const OBJECTIVE_KEYS: [&str; 3] = ["gold", "loot", "once"];

/// Soft cap on a table's average item value per defeat, by the unit's tier (warning only).
pub fn loot_ev_soft(tier: &str) -> i64 {
    match tier {
        "minion" => 5,
        "standard" => 10,
        "elite" => 20,
        "boss" => 30,
        _ => 0,
    }
}

fn whole_in(v: &Value, lo: i64, hi: i64) -> bool {
    let x = json::normalize(v);
    json::is_int(&x) && (lo..=hi).contains(&json::to_int(&x))
}

/// A unit's `loot` rows: shape, items, chances (1..100, at most 100 in all), counts (1..9), no item twice.
pub fn loot_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(raw) = e.get("loot") else { return errors };
    let Some(rows) = raw.as_array() else {
        errors.push("Loot must be a list of rows.".into());
        return errors;
    };
    let (mut total, mut seen): (i64, Vec<String>) = (0, Vec::new());
    for (i, row) in rows.iter().enumerate() {
        let n = i + 1;
        let Some(row) = row.as_object() else {
            errors.push("Loot must be a list of rows.".into());
            continue;
        };
        for key in row.keys().filter(|k| !LOOT_ROW_KEYS.contains(&k.as_str())) {
            errors.push(format!("Loot row {n}: key '{key}' is not allowed."));
        }
        let item = gstr(row.get("item").unwrap_or(&Value::Null));
        if v.get("items", &item).is_empty() {
            errors.push(format!("Loot row {n}: item '{item}' does not resolve."));
        }
        match row.get("chance") {
            Some(c) if whole_in(c, 1, LOOT_CHANCE_TOTAL) => total += json::to_int(c),
            _ => errors.push(format!("Loot row {n}: chance must be a whole number 1..{LOOT_CHANCE_TOTAL}.")),
        }
        if row.get("count").is_some_and(|c| !whole_in(c, 1, LOOT_COUNT_MAX)) {
            errors.push(format!("Loot row {n}: count must be a whole number 1..{LOOT_COUNT_MAX}."));
        }
        if !item.is_empty() && seen.contains(&item) {
            errors.push(format!("Loot item '{item}' is listed twice."));
        }
        seen.push(item);
    }
    if total > LOOT_CHANCE_TOTAL {
        errors.push(format!("Loot chances add up to {total} (at most {LOOT_CHANCE_TOTAL})."));
    }
    errors
}

/// A party-side unit with a table, a special item, an average value past the tier's soft cap.
pub fn loot_warnings(v: &View, e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let rows: Vec<&Dict> = json::aget(e, "loot").iter().filter_map(Value::as_object).collect();
    if rows.is_empty() {
        return out;
    }
    if json::s(e, "side") == "party" {
        out.push(format!("Party-side unit '{}' has a loot table; party units never drop.", json::s(e, "id")));
    }
    let mut ev_x100 = 0;
    for row in &rows {
        let item = v.get("items", &json::s(row, "item"));
        if item.is_empty() {
            continue;
        }
        if json::b(item, "special", false) {
            out.push(format!("Loot drops special item '{}'.", json::s(item, "id")));
        }
        ev_x100 += json::i(row, "chance", 0) * json::i(item, "value", 0) * json::i(row, "count", 1);
    }
    let tier = json::s_or(e, "tier", "standard");
    let (ev, soft) = (ev_x100 / 100, loot_ev_soft(&tier));
    if ev > soft {
        out.push(format!("Loot is worth {ev} gold per defeat on average (soft cap {soft} for {tier})."));
    }
    out
}

/// The refs of a world encounter's roster (REWARD1 2.1): enemies, then every phase wave in list order (the arrival,
/// the reinforcements, first: PHASEFOLD 3), resolving ones only.
fn roster_refs(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut refs = json::sget(e, "enemies");
    for phase in json::aget(e, "phases") {
        for fx in json::aget(json::dict(phase), "effects") {
            let fx = json::dict(fx);
            if json::s(fx, "type") == "wave" {
                refs.extend(json::sget(fx, "units"));
            }
        }
    }
    refs.retain(|r| !v.worlds.resolve(w, "characters", r).is_empty());
    refs
}

fn loot_problem(v: &View, w: &str, r: &str, label: &str) -> Option<String> {
    match world_loot_check(v, w, r) {
        "missing" => Some(format!("{label} '{r}' does not resolve.")),
        "kind" => Some(format!("{label} '{r}' must be an item reward.")),
        _ => None,
    }
}

/// A world encounter's `key_loot` rows: shape, the loot (an item reward of the world), the kill (a roster ref), no
/// loot twice.
pub fn key_loot_problems(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(raw) = e.get("key_loot") else { return errors };
    let Some(rows) = raw.as_array() else {
        errors.push("Key loot must be a list of rows.".into());
        return errors;
    };
    let roster = roster_refs(v, w, e);
    let mut seen: Vec<String> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let Some(row) = row.as_object() else {
            errors.push("Key loot must be a list of rows.".into());
            continue;
        };
        for key in row.keys().filter(|k| !KEY_LOOT_KEYS.contains(&k.as_str())) {
            errors.push(format!("Key loot row {}: key '{key}' is not allowed.", i + 1));
        }
        let loot = gstr(row.get("loot").unwrap_or(&Value::Null));
        errors.extend(loot_problem(v, w, &loot, "Key loot"));
        let kill = gstr(row.get("kill").unwrap_or(&Value::Null));
        if !roster.contains(&kill) {
            errors.push(format!("Key loot kill '{kill}' is not an enemy of this encounter."));
        }
        let hit = v.worlds.resolve(w, "loot", &loot);
        let key = if hit.is_empty() { loot.clone() } else { json::s(hit, "_qid") };
        if seen.contains(&key) {
            errors.push(format!("Key loot '{loot}' is listed twice."));
        }
        seen.push(key);
    }
    errors
}

/// A kill ref listed several times (every one must be slain), a key loot entry also in a quest pool.
pub fn key_loot_warnings(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let rows: Vec<&Dict> = json::aget(e, "key_loot").iter().filter_map(Value::as_object).collect();
    if rows.is_empty() {
        return out;
    }
    let roster = roster_refs(v, w, e);
    let mut pools: Vec<String> = Vec::new();
    for q in v.worlds.merged_entries(w, "quests") {
        let mut lists = vec![json::get(q, "loot_pool").clone()];
        lists.extend(json::aget(q, "variants").iter().map(|x| json::get(json::dict(x), "loot_pool").clone()));
        for r in lists.iter().flat_map(json::arr) {
            pools.push(json::s(v.worlds.resolve(w, "loot", &gstr(r)), "_qid"));
        }
    }
    for row in rows {
        let kill = json::s(row, "kill");
        let n = roster.iter().filter(|r| **r == kill).count();
        if n > 1 {
            out.push(format!("Key loot kill '{kill}' is listed {n} times: every one must be slain."));
        }
        let loot = json::s(row, "loot");
        let qid = json::s(v.worlds.resolve(w, "loot", &loot), "_qid");
        if !qid.is_empty() && pools.contains(&qid) {
            out.push(format!("Key loot '{loot}' is also in a quest pool: whichever claims it first wins."));
        }
    }
    out
}

/// A world encounter's `objective_reward`: its keys, gold 0..500, the loot (an item reward of the world).
pub fn objective_reward_problems(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(raw) = e.get("objective_reward") else { return errors };
    let Some(o) = raw.as_object() else {
        errors.push("Objective reward must be an object {gold, loot, once}.".into());
        return errors;
    };
    for key in o.keys().filter(|k| !OBJECTIVE_KEYS.contains(&k.as_str())) {
        errors.push(format!("Objective reward key '{key}' is not allowed."));
    }
    if o.get("gold").is_some_and(|g| !whole_in(g, 0, OBJECTIVE_GOLD_MAX)) {
        errors.push(format!("Objective reward gold must be a whole number 0..{OBJECTIVE_GOLD_MAX}."));
    }
    let loot = json::s(o, "loot");
    if !loot.is_empty() {
        errors.extend(loot_problem(v, w, &loot, "Objective loot"));
    }
    if o.get("once").is_some_and(|x| !x.is_boolean()) {
        errors.push("Objective reward once must be true or false.".into());
    }
    errors
}

/// Gold above the soft cap, a reward that pays nothing.
pub fn objective_reward_warnings(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let Some(o) = e.get("objective_reward").and_then(Value::as_object) else { return out };
    let gold = json::i(o, "gold", 0);
    if gold > OBJECTIVE_GOLD_SOFT {
        out.push(format!("Objective reward gold above {OBJECTIVE_GOLD_SOFT}: check the budget."));
    }
    if !o.is_empty() && gold <= 0 && json::s(o, "loot").is_empty() {
        out.push("Empty objective reward.".into());
    }
    out
}
