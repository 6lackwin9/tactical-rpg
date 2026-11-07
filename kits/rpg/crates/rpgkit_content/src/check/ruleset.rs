//! Ruleset rows (RULESET1-R, docs/specs/RULESET1_TASKS.md 1.2; engine-only, no Godot original): the checks of one row
//! of the kind `rulesets` (a snake_case id, open since CLN-44, name, rules from the register, intents, basics, the
//! poise stat, deck slots, the switches' values (CLN-45), origin) and across rows (unique ids). The numbers' ranges are
//! the schema's min and max, which normalizing a row applies (as for every kind). The loader keeps the rows that pass.

use super::View;
use aegis_core::json::{self, gstr, Dict};
use aegis_content::schema::is_id;
use rpgkit_rules::ruleset::{self, DECK, INTENTS, NEUTRAL, POISE, REGISTER, ROUND_INTENT};
use serde_json::Value;

/// The catalog kind (file data/catalog/rulesets.json).
pub const KIND: &str = ruleset::KIND;
const NAME_MAX: usize = 24;
/// The id length bounds (the schema's `min_len` and `max_len`).
const ID_MIN: usize = 2;
const ID_MAX: usize = 24;
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
    } else if !is_id(&id) || !(ID_MIN..=ID_MAX).contains(&id.chars().count()) {
        // CLN-44: the ids are open (classic included: the default ruleset is a row too)
        out.push(format!("ID '{id}' must be lowercase snake_case (a-z, 0-9, _), {ID_MIN}-{ID_MAX} characters."));
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
    // BASERULES1-M: a listed rule the register says would be silently inert (`flank` without `round_intent`)
    out.extend(ruleset::Ruleset::from_row(e).notes().errors);
    // CLN-45: each switch is "" (by the rules) or one of its values
    for key in ruleset::BY_RULES_FIELDS {
        let v = json::s(e, key);
        let values = ruleset::by_rules_values(key);
        if !v.is_empty() && !values.contains(&v.as_str()) {
            out.push(format!("{key}: '{v}' is not one of {} (or empty: by the rules).", values.join(", ")));
        }
    }
    // S2.2-S2: a row-only value field (`poise_mode`) is absent or one of its ids (else it would read as its default)
    for f in ruleset::FIELD_RULES.iter().filter(|f| !f.values.is_empty()) {
        let v = json::s(e, f.key);
        if e.contains_key(f.key) && !f.values.contains(&v.as_str()) {
            out.push(format!("{}: '{v}' is not one of {}.", f.key, f.values.join(", ")));
        }
    }
    // U1.2-S5: `unit_pools` (row-only) is a list of unit meter ids, none twice; whether each names a row is
    // `missing_pools` (the meters load after the rulesets)
    if let Some(raw) = e.get("unit_pools") {
        let mut seen: Vec<String> = Vec::new();
        match raw.as_array() {
            None => out.push("unit_pools must be a list of unit meter ids.".into()),
            Some(list) => {
                for v in list {
                    let id = v.as_str().unwrap_or_default().to_string();
                    if !is_id(&id) {
                        out.push(format!("unit_pools: '{}' must be a unit meter id (lowercase snake_case).", json::gstr(v)));
                    } else if seen.contains(&id) {
                        out.push(format!("unit_pools lists '{id}' twice."));
                    }
                    seen.push(id);
                }
            }
        }
    }
    let origin = json::s(e, "origin");
    if origin != super::ORIGIN_SYSTEM && origin != "user" {
        out.push(format!("Origin: '{origin}' is not one of system, user."));
    }
    out
}

/// CLN-44: the shape of another row's `rulesets` list (derived stats, portrait cues): ruleset ids, none twice. Whether
/// each names a ruleset row is `missing_rows` (the rows load after the derived stats).
pub fn list_errors(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for v in json::sget(e, "rulesets") {
        if !is_id(&v) {
            out.push(format!("Rulesets: '{v}' must be a ruleset id (lowercase snake_case)."));
        } else if seen.contains(&v) {
            out.push(format!("Rulesets lists '{v}' twice."));
        }
        seen.push(v);
    }
    out
}

/// CLN-44: the ids of a row's `rulesets` list that name no loaded ruleset row.
pub fn missing_rows(v: &View, e: &Dict) -> Vec<String> {
    json::sget(e, "rulesets")
        .into_iter()
        .filter(|r| is_id(r) && !v.has(KIND, r))
        .map(|r| format!("Rulesets: '{r}' has no row in {KIND} (data/catalog/{KIND}.json)."))
        .collect()
}

/// U1.2-S5: the ids of a ruleset row's `unit_pools` that name no loaded unit meter row.
pub fn missing_pools(v: &View, e: &Dict) -> Vec<String> {
    json::sget(e, "unit_pools")
        .into_iter()
        .filter(|m| is_id(m) && !v.has(rpgkit_data::UNIT_METERS_KIND, m))
        .map(|m| format!("unit_pools: '{m}' has no row in {0} (data/catalog/{0}.json).", rpgkit_data::UNIT_METERS_KIND))
        .collect()
}

/// INPUT1 1.7 (MR9): the soft problems of one row (normalized): a `flank_protected_ap` set while the cost is not `fixed`
/// (it is not read), then the register metadata's warnings (BASERULES1-M, `Ruleset::notes`). CLN-45: `melee_diagonal` maneuver without `round_intent` is no longer a warning (it reads as set).
pub fn warnings(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let ap = e.get("flank_protected_ap").map(|v| json::to_int(v));
    if ap.is_some_and(|ap| ap != i64::from(ruleset::FlankCost::FIXED_AP)) && json::s(e, "flank_protected_cost") != "fixed" {
        out.push("flank_protected_ap is read only with fixed".into());
    }
    // BASERULES1-M: the register metadata's warnings (a set field nothing reads, a conflict, a split bundle, an
    // untested rule set)
    out.extend(ruleset::Ruleset::from_row(e).notes().warnings);
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
