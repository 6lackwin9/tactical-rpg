//! Ruleset rows (RULESET1-R, docs/specs/RULESET1_TASKS.md 1.2; engine-only, no Godot original): the checks of one row
//! of the kind `rulesets` (id from the code-owned list, name, rules from the register, intents, basics, the poise stat,
//! deck slots, origin) and across rows (unique ids). The numbers' ranges are the schema's min and max, which normalizing
//! a row applies (as for every kind). The loader keeps the rows that pass.

use super::View;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_rules::ruleset::{self, DECK, INTENTS, NEUTRAL, POISE, REGISTER, ROUND_INTENT, ROW_IDS};
use serde_json::Value;

/// The catalog kind (file data/catalog/rulesets.json).
pub const KIND: &str = ruleset::KIND;
const NAME_MAX: usize = 24;
/// The deck slot kinds and their ranges (RULESET1 9 D1).
const SLOTS: [(&str, i64, i64); 3] = [("active", 1, 8), ("passive", 0, 4), ("keystone", 0, 2)];

/// What a row's checks read from the rest of the catalog: the round intent of a tier-1 skill family (None = no such
/// family) and the derived-stat ids.
pub struct Refs<'a> {
    pub skill_intent: &'a dyn Fn(&str) -> Option<String>,
    pub derived: &'a [String],
}

/// The intent of a skill row (RULESET1_INTENT IN4): its `round_intent` when set, else its kind's default.
pub fn intent_of(skill: &Dict) -> String {
    let set = json::s(skill, "round_intent");
    if set.is_empty() {
        ruleset::kind_intent(&json::s(skill, "kind")).to_string()
    } else {
        set
    }
}

/// Refs over loaded catalog rows ({kind: [entry]}): skill families are the tier-1 skill rows.
pub fn skill_intent_in(skills: &[Value], id: &str) -> Option<String> {
    skills.iter().map(json::dict).find(|e| json::s(e, "id") == id && json::i(e, "tier", 1) == 1).map(intent_of)
}

fn listed(e: &Dict, key: &str, label: &str, allowed: &[&str], out: &mut Vec<String>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for v in json::aget(e, key).iter().map(gstr) {
        if !allowed.contains(&v.as_str()) {
            out.push(format!("{label}: '{v}' is not one of {}.", allowed.join(", ")));
        } else if seen.contains(&v) {
            out.push(format!("{label} lists '{v}' twice."));
        }
        seen.push(v);
    }
    seen
}

/// The own problems of a (normalized) row.
pub fn row_errors(e: &Dict, refs: &Refs) -> Vec<String> {
    let mut out = Vec::new();
    let id = json::s(e, "id");
    if id.is_empty() {
        out.push("ID is required.".into());
    } else if !ROW_IDS.contains(&id.as_str()) {
        out.push(format!("ID: '{id}' is not one of {} (the ruleset ids are code-owned; classic has no row).", ROW_IDS.join(", ")));
    }
    let name = json::s(e, "name");
    if name.trim().is_empty() {
        out.push("Name is required.".into());
    } else if name.chars().count() > NAME_MAX {
        out.push(format!("Name is {} characters (max {NAME_MAX}).", name.chars().count()));
    }
    let ids: Vec<&str> = REGISTER.iter().map(|r| r.id).collect();
    let rules = listed(e, "rules", "Rules", &ids, &mut out);
    let has = |r: &str| rules.iter().any(|x| x == r);
    let intents = listed(e, "intents", "Intents", &INTENTS, &mut out);
    if intents.is_empty() {
        out.push("Intents: list at least one.".into());
    }
    let default_intents: Vec<String> = ruleset::DEFAULT_INTENTS.iter().map(|s| s.to_string()).collect();
    if intents != default_intents && !has(ROUND_INTENT) {
        out.push(format!("Intents need {ROUND_INTENT} in Rules."));
    }
    let basics = json::sget(e, "basics");
    let mut seen: Vec<&String> = Vec::new();
    for b in &basics {
        if seen.contains(&b) {
            out.push(format!("Basics lists '{b}' twice."));
        }
        seen.push(b);
        match (refs.skill_intent)(b) {
            None => out.push(format!("Basics: unknown skill family '{b}'.")),
            Some(intent) if intent != NEUTRAL && !intents.contains(&intent) => out.push(format!("Basics: '{b}' is of intent {intent}, which Intents does not list.")),
            Some(_) => {}
        }
    }
    if has(DECK) && basics.is_empty() {
        out.push(format!("Basics: {DECK} needs at least one basic."));
    }
    let poise_stat = json::s(e, "poise_stat");
    if !poise_stat.is_empty() && !refs.derived.contains(&poise_stat) {
        out.push(format!("Poise stat: unknown derived stat '{poise_stat}'."));
    } else if poise_stat.is_empty() && has(POISE) {
        out.push(format!("Poise stat: {POISE} needs a derived stat."));
    }
    deck_slot_errors(json::get(e, "deck_slots"), has(DECK), &mut out);
    let origin = json::s(e, "origin");
    if origin != super::ORIGIN_SYSTEM && origin != "user" {
        out.push(format!("Origin: '{origin}' is not one of system, user."));
    }
    out
}

fn deck_slot_errors(v: &Value, deck: bool, out: &mut Vec<String>) {
    let Some(slots) = v.as_object() else {
        out.push("Deck slots must be an object of slot kind to count.".into());
        return;
    };
    let kinds: Vec<&str> = SLOTS.iter().map(|s| s.0).collect();
    for k in slots.keys().filter(|k| !kinds.contains(&k.as_str())) {
        out.push(format!("Deck slots: '{k}' is not one of {}.", kinds.join(", ")));
    }
    for (k, lo, hi) in SLOTS {
        match slots.get(k) {
            None if deck && k == "active" => out.push(format!("Deck slots: {DECK} needs an active count ({lo}..{hi}).")),
            None => {}
            Some(n) if !n.as_f64().is_some_and(|x| x.fract() == 0.0 && (lo as f64..=hi as f64).contains(&x)) => {
                out.push(format!("Deck slots: {k} must be a whole number {lo}..{hi}."));
            }
            Some(_) => {}
        }
    }
}

/// The checks of rows as the loader read them (normalized): each row's own problems and the ids used twice.
pub fn check_rows(rows: &[Dict], refs: &Refs) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for (i, e) in rows.iter().enumerate() {
        let mut found = row_errors(e, refs);
        let id = json::s(e, "id");
        if !id.is_empty() && rows[..i].iter().any(|r| json::s(r, "id") == id) {
            found.push(format!("ID '{id}' is already used by another ruleset."));
        }
        out.push(found);
    }
    out
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry) among the loaded rows.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    let skills: Vec<Value> = v.entries("skills").iter().cloned().map(Value::Object).collect();
    let skill_intent = |id: &str| skill_intent_in(&skills, id);
    let derived = v.derived_ids();
    check_rows(&rows, &Refs { skill_intent: &skill_intent, derived: &derived }).pop().unwrap_or_default()
}
