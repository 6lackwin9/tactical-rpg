//! The campaign checks (CAMPAIGN1 11.1-11.4, kit hook; engine-only, no Godot original): a world manifest's `campaign`,
//! `purse`, `rest` and `defeat` blocks, the rows of the world kinds `meters`, `checkpoints` and `triggers`, the
//! campaign effects (called by the dialogue effect checker for every effect list), the `then` items, and the read keys
//! (`meter.<id>`, `campaign.act`, ...) in any condition of a world entry. Every rule is inert data until a world sets
//! `campaign` (X1); these checks only fire on the new fields and kinds, so worlds without them report as before.

use super::quest::{condition_errors, SET_FLAG_LISTS};
use super::world::ref_errors;
use super::{dialogue, map, View};
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{CAMPAIGN_READ_KEYS, DEFEAT_MODES, END_KINDS, INPUT_ON, METER_TONES, OUTCOMES, REST_WHERE, RULE_DIRS};
use serde_json::Value;

/// Acts are numbered 0 (before the first) to 9.
pub const ACT_MAX: i64 = 9;
const METER_NAME_MAX: usize = 32;
const PURSE_NAME_MAX: usize = 24;
const HUD_SHORT_MAX: usize = 16;
const LABEL_MAX: usize = 32;
const TEXT_MAX: usize = 1000;
const PER_DAY_MAX: i64 = 5;
const ROAD_PCT_MIN: i64 = 100;
const ROAD_PCT_MAX: i64 = 300;
const START_AP_MIN: i64 = -5;
const REST_DAYS_MAX: i64 = 3;
const REST_COST_MAX: i64 = 10;
const DEFEAT_DAYS_MAX: i64 = 3;
const BANDS_MAX: usize = 6;
const RULES_MAX: usize = 12;
const INPUTS_MAX: usize = 12;
/// A countdown's longest run and a checkpoint's latest deadline (days).
pub const DAYS_MAX: i64 = 999;
const WARN_DAY_MAX: i64 = 30;
/// A band-modifier's road-fight percentage (100 = unchanged).
const MODIFIER_PCT_MAX: i64 = 300;
const CAMPAIGN_KEYS: [&str; 2] = ["start_act", "act_saves"];
const PURSE_KEYS: [&str; 2] = ["currency", "upkeep"];
const CURRENCY_KEYS: [&str; 3] = ["name", "icon", "tip"];
const UPKEEP_KEYS: [&str; 6] = ["name", "short", "icon", "tip", "per_day", "at_zero"];
const AT_ZERO_KEYS: [&str; 2] = ["road_pct", "start_ap"];
const REST_KEYS: [&str; 5] = ["days", "cost", "where", "on_enter", "needs_upkeep"];
const DEFEAT_KEYS: [&str; 3] = ["mode", "to", "days"];
const BAND_KEYS: [&str; 4] = ["id", "name", "from", "tone"];
const RULE_KEYS: [&str; 9] = ["id", "at", "dir", "acts", "once", "warn", "effects", "then", "restart_days"];
const MODIFIER_KEYS: [&str; 4] = ["bands", "road_pct", "road_encounters", "route_tags"];
const FLOOR_KEYS: [&str; 2] = ["from_act", "min"];
const HUD_KEYS: [&str; 3] = ["short", "icon", "order"];
const THEN_KEYS: [&str; 4] = ["vignette", "battle", "dialogue", "speaker"];

/// The keys an input of `on` takes.
fn input_keys(on: &str) -> &'static [&'static str] {
    match on {
        "battle" => &["on", "tags", "outcomes", "delta", "acts"],
        "day" => &["on", "delta", "acts"],
        _ => &["on", "route_tags", "delta", "acts"],
    }
}

fn int_in(v: &Value, lo: i64, hi: i64) -> bool {
    json::is_int(v) && (lo..=hi).contains(&json::to_int(v))
}

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// An unknown key of `d` (one error each).
fn unknown_keys(d: &Dict, allowed: &[&str], label: &str) -> Vec<String> {
    d.keys().filter(|k| !allowed.contains(&k.as_str())).map(|k| format!("{label}: unknown key '{k}'.")).collect()
}

/// `d[key]`, when present, is a whole number in lo..hi.
fn int_key(d: &Dict, key: &str, lo: i64, hi: i64, label: &str) -> Option<String> {
    d.get(key).filter(|x| !int_in(x, lo, hi)).map(|_| format!("{label}: {key} must be a whole number {lo}..{hi}."))
}

/// `d[key]`, when present, is true or false.
fn bool_key(d: &Dict, key: &str, label: &str) -> Option<String> {
    d.get(key).filter(|x| !x.is_boolean()).map(|_| format!("{label}: {key} must be true or false."))
}

/// `d[key]`, when present, is text of at most `max` characters (and not blank when `named`).
fn text_key(d: &Dict, key: &str, max: usize, named: bool, label: &str) -> Option<String> {
    let x = d.get(key)?;
    match x.as_str() {
        None => Some(format!("{label}: {key} must be text.")),
        Some(t) if named && t.trim().is_empty() => Some(format!("{label}: {key} must not be empty.")),
        Some(t) if chars(t) > max => Some(format!("{label}: {key} is longer than {max} characters.")),
        Some(_) => None,
    }
}

/// `d[key]`, when present, is one of `options`.
fn enum_key(d: &Dict, key: &str, options: &[&str], label: &str) -> Option<String> {
    let x = d.get(key)?;
    (!x.as_str().is_some_and(|s| options.contains(&s))).then(|| format!("{label}: {key} '{}' is not one of {}.", gstr(x), options.join(", ")))
}

/// `d[key]`, when present, is a list of text.
fn strings_key(d: &Dict, key: &str, label: &str) -> Option<String> {
    let x = d.get(key)?;
    (!x.as_array().is_some_and(|a| a.iter().all(Value::is_string))).then(|| format!("{label}: {key} must be a list of text."))
}

/// `d[key]`, when present, is a list of acts (whole numbers 0..ACT_MAX).
fn acts_key(d: &Dict, key: &str, label: &str) -> Option<String> {
    let x = d.get(key)?;
    (!x.as_array().is_some_and(|a| a.iter().all(|n| int_in(n, 0, ACT_MAX)))).then(|| format!("{label}: {key} must be a list of acts 0..{ACT_MAX}."))
}

/// The object a manifest block or a json field holds (None and an error when it holds something else).
fn object<'a>(raw: &'a Value, label: &str, errors: &mut Vec<String>) -> Option<&'a Dict> {
    let d = raw.as_object();
    if d.is_none() {
        errors.push(format!("{label} must be an object."));
    }
    d
}

/// The list a json field holds (None and an error when it holds something else).
fn list<'a>(raw: &'a Value, label: &str, errors: &mut Vec<String>) -> Option<&'a [Value]> {
    let a = raw.as_array().map(Vec::as_slice);
    if a.is_none() {
        errors.push(format!("{label} must be a list."));
    }
    a
}

// ---- 11.1 manifest blocks ----

/// The problems of a world manifest's campaign blocks (11.1); absent blocks are today's rules.
pub fn manifest_errors(v: &View, w: &str, m: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(c) = m.get("campaign").and_then(|raw| object(raw, "Campaign", &mut errors)) {
        errors.extend(unknown_keys(c, &CAMPAIGN_KEYS, "Campaign"));
        errors.extend(int_key(c, "start_act", 0, ACT_MAX, "Campaign"));
        errors.extend(bool_key(c, "act_saves", "Campaign"));
    }
    if let Some(p) = m.get("purse").and_then(|raw| object(raw, "Purse names", &mut errors)) {
        errors.extend(unknown_keys(p, &PURSE_KEYS, "Purse names"));
        if let Some(c) = p.get("currency").and_then(|raw| object(raw, "Purse names currency", &mut errors)) {
            errors.extend(unknown_keys(c, &CURRENCY_KEYS, "Purse names currency"));
            errors.extend(text_key(c, "name", PURSE_NAME_MAX, true, "Purse names currency"));
            errors.extend(text_key(c, "icon", TEXT_MAX, false, "Purse names currency"));
            errors.extend(text_key(c, "tip", TEXT_MAX, false, "Purse names currency"));
        }
        if let Some(u) = p.get("upkeep").and_then(|raw| object(raw, "Purse names upkeep", &mut errors)) {
            let label = "Purse names upkeep";
            errors.extend(unknown_keys(u, &UPKEEP_KEYS, label));
            errors.extend(text_key(u, "name", PURSE_NAME_MAX, true, label));
            errors.extend(text_key(u, "short", PURSE_NAME_MAX, true, label));
            errors.extend(text_key(u, "icon", TEXT_MAX, false, label));
            errors.extend(text_key(u, "tip", TEXT_MAX, false, label));
            errors.extend(int_key(u, "per_day", 0, PER_DAY_MAX, label));
            if let Some(z) = u.get("at_zero").and_then(|raw| object(raw, "Purse names upkeep at_zero", &mut errors)) {
                errors.extend(unknown_keys(z, &AT_ZERO_KEYS, "Purse names upkeep at_zero"));
                errors.extend(int_key(z, "road_pct", ROAD_PCT_MIN, ROAD_PCT_MAX, "Purse names upkeep at_zero"));
                errors.extend(int_key(z, "start_ap", START_AP_MIN, 0, "Purse names upkeep at_zero"));
            }
        }
    }
    if let Some(r) = m.get("rest").and_then(|raw| object(raw, "Rest rule", &mut errors)) {
        errors.extend(unknown_keys(r, &REST_KEYS, "Rest rule"));
        errors.extend(int_key(r, "days", 0, REST_DAYS_MAX, "Rest rule"));
        errors.extend(int_key(r, "cost", 0, REST_COST_MAX, "Rest rule"));
        errors.extend(enum_key(r, "where", &REST_WHERE, "Rest rule"));
        errors.extend(bool_key(r, "on_enter", "Rest rule"));
        errors.extend(bool_key(r, "needs_upkeep", "Rest rule"));
    }
    if let Some(d) = m.get("defeat").and_then(|raw| object(raw, "Defeat rule", &mut errors)) {
        errors.extend(unknown_keys(d, &DEFEAT_KEYS, "Defeat rule"));
        errors.extend(enum_key(d, "mode", &DEFEAT_MODES, "Defeat rule"));
        errors.extend(text_key(d, "to", TEXT_MAX, false, "Defeat rule"));
        errors.extend(int_key(d, "days", 0, DEFEAT_DAYS_MAX, "Defeat rule"));
        let to = json::s(d, "to");
        if json::s(d, "mode") == "retreat" {
            if to.is_empty() {
                errors.push("Defeat rule: a retreat needs \"to\", the location the party retreats to.".into());
            } else {
                errors.extend(ref_errors(v, w, "Defeat rule to", "locations", &to));
            }
        }
    }
    errors
}

/// The soft problems of world `w`'s campaign data: purse and HUD icons that no asset layer holds.
pub fn warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = Vec::new();
    if v.assets.roots.is_empty() {
        return out;
    }
    let mut icon = |icon: String, what: String| {
        if !icon.is_empty() && !v.assets.has(&icon) {
            out.push(format!("{what}: icon '{icon}' is in no asset folder."));
        }
    };
    let purse = json::dget(v.worlds.manifest(w), "purse");
    for part in ["currency", "upkeep"] {
        icon(json::s(json::dget(purse, part), "icon"), format!("{w}/world: purse {part}"));
    }
    for e in v.worlds.own_entries(w, "meters") {
        icon(json::s(json::dget(e, "hud"), "icon"), format!("{}: HUD", json::s(e, "_qid")));
    }
    out
}

// ---- 11.2 meters ----

/// A meter's band ids (the well-formed ones), for rules and modifiers.
fn band_ids(e: &Dict) -> Vec<String> {
    json::aget(e, "bands").iter().map(|b| json::s(json::dict(b), "id")).filter(|id| !id.is_empty()).collect()
}

/// The problems of a meter row (11.2, M1-M9).
pub fn meter_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if chars(&json::s(e, "name")) > METER_NAME_MAX {
        errors.push(format!("Name is longer than {METER_NAME_MAX} characters."));
    }
    if json::s(e, "scope") == "unit" {
        errors.push("Scope unit is a doctrine lean: CAMPAIGN1-DOC (not built yet).".into());
    }
    let countdown = json::s(e, "kind") == "countdown";
    let (lo, hi, start) = (json::i(e, "min", 0), json::i(e, "max", 100), json::i(e, "start", 0));
    if lo >= hi {
        // every other number of the row is checked against the range
        errors.push("Min must be below max.".into());
        return errors;
    }
    if !(lo..=hi).contains(&start) {
        errors.push(format!("Start must be in min..max ({lo}..{hi})."));
    }
    if countdown {
        if lo != 0 {
            errors.push("A countdown's min is 0.".into());
        }
        if !(1..=DAYS_MAX).contains(&hi) {
            errors.push(format!("A countdown's max must be 1..{DAYS_MAX} days."));
        }
        if json::s(e, "unit") != "days" {
            errors.push("A countdown needs the unit days.".into());
        }
    }
    errors.extend(bands_errors(json::get(e, "bands"), lo, hi));
    let bands = band_ids(e);
    if let Some(rules) = list(json::get(e, "rules"), "Rules", &mut errors) {
        if rules.len() > RULES_MAX {
            errors.push(format!("A meter holds at most {RULES_MAX} rules (has {}).", rules.len()));
        }
        let mut seen: Vec<String> = Vec::new();
        for (i, raw) in rules.iter().enumerate() {
            errors.extend(rule_errors(v, w, i, raw, &bands, (lo, hi), countdown, &mut seen));
        }
    }
    if let Some(inputs) = list(json::get(e, "inputs"), "Inputs", &mut errors) {
        if countdown && !inputs.is_empty() {
            errors.push("Inputs move gauges only; a countdown runs on the travel clock.".into());
        }
        if inputs.len() > INPUTS_MAX {
            errors.push(format!("A meter holds at most {INPUTS_MAX} inputs (has {}).", inputs.len()));
        }
        for (i, raw) in inputs.iter().enumerate() {
            errors.extend(input_errors(i, raw, hi - lo));
        }
    }
    if let Some(mods) = list(json::get(e, "modifiers"), "Modifiers", &mut errors) {
        for (i, raw) in mods.iter().enumerate() {
            errors.extend(modifier_errors(v, w, i, raw, &bands));
        }
    }
    if let Some(floors) = list(json::get(e, "floors"), "Floors", &mut errors) {
        for (i, raw) in floors.iter().enumerate() {
            let label = format!("Floor {}", i + 1);
            let Some(f) = object(raw, &label, &mut errors) else { continue };
            errors.extend(unknown_keys(f, &FLOOR_KEYS, &label));
            if !int_in(json::get(f, "from_act"), 0, ACT_MAX) {
                errors.push(format!("{label}: from_act must be a whole number 0..{ACT_MAX}."));
            }
            if !int_in(json::get(f, "min"), lo, hi) {
                errors.push(format!("{label}: min must be a whole number in min..max ({lo}..{hi})."));
            }
        }
    }
    if let Some(hud) = object(json::get(e, "hud"), "HUD", &mut errors) {
        errors.extend(unknown_keys(hud, &HUD_KEYS, "HUD"));
        errors.extend(text_key(hud, "short", HUD_SHORT_MAX, true, "HUD"));
        errors.extend(text_key(hud, "icon", TEXT_MAX, false, "HUD"));
        if hud.get("order").is_some_and(|o| !json::is_int(o)) {
            errors.push("HUD: order must be a whole number.".into());
        }
    }
    errors
}

/// M2: 1-6 bands {id, name, from, tone}, the first from at `lo`, ascending, ids unique, tones known.
fn bands_errors(raw: &Value, lo: i64, hi: i64) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(bands) = list(raw, "Bands", &mut errors) else { return errors };
    if bands.is_empty() || bands.len() > BANDS_MAX {
        errors.push(format!("A meter needs 1-{BANDS_MAX} bands (has {}).", bands.len()));
    }
    let mut seen: Vec<String> = Vec::new();
    let mut last: Option<i64> = None;
    for (i, raw) in bands.iter().enumerate() {
        let label = format!("Band {}", i + 1);
        let Some(b) = object(raw, &label, &mut errors) else { continue };
        errors.extend(unknown_keys(b, &BAND_KEYS, &label));
        let id = json::s(b, "id");
        if !aegis_content::schema::is_id(&id) {
            errors.push(format!("{label}: id '{id}' must be lowercase snake_case (a-z, 0-9, _)."));
        } else if seen.contains(&id) {
            errors.push(format!("{label}: id '{id}' is used twice."));
        }
        seen.push(id);
        if json::s(b, "name").trim().is_empty() || chars(&json::s(b, "name")) > METER_NAME_MAX {
            errors.push(format!("{label}: name must be 1-{METER_NAME_MAX} characters."));
        }
        if !METER_TONES.contains(&json::s(b, "tone").as_str()) {
            errors.push(format!("{label}: tone '{}' is not one of {}.", json::s(b, "tone"), METER_TONES.join(", ")));
        }
        let from = json::get(b, "from");
        if !int_in(from, lo, hi) {
            errors.push(format!("{label}: from must be a whole number in min..max ({lo}..{hi})."));
            continue;
        }
        let from = json::to_int(from);
        match last {
            None if from != lo => errors.push(format!("The first band must start at min ({lo}).")),
            Some(prev) if from <= prev => errors.push(format!("{label}: bands must rise (from {from} after {prev}).")),
            _ => {}
        }
        last = Some(from);
    }
    errors
}

/// M5: one meter rule.
#[allow(clippy::too_many_arguments)]
fn rule_errors(v: &View, w: &str, i: usize, raw: &Value, bands: &[String], (lo, hi): (i64, i64), countdown: bool, seen: &mut Vec<String>) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(r) = object(raw, &format!("Rule {}", i + 1), &mut errors) else { return errors };
    let id = json::s(r, "id");
    let label = if aegis_content::schema::is_id(&id) { format!("Rule '{id}'") } else { format!("Rule {}", i + 1) };
    if !aegis_content::schema::is_id(&id) {
        errors.push(format!("{label}: id '{id}' must be lowercase snake_case (a-z, 0-9, _)."));
    } else if seen.contains(&id) {
        errors.push(format!("{label}: id '{id}' is used twice."));
    }
    seen.push(id);
    errors.extend(unknown_keys(r, &RULE_KEYS, &label));
    match r.get("at") {
        Some(Value::String(band)) if !bands.contains(band) => errors.push(format!("{label}: at '{band}' names no band of this meter.")),
        Some(Value::String(_)) => {}
        Some(n) if int_in(n, lo, hi) => {}
        _ => errors.push(format!("{label}: at must be a band id or a whole number in min..max ({lo}..{hi}).")),
    }
    errors.extend(enum_key(r, "dir", &RULE_DIRS, &label));
    errors.extend(acts_key(r, "acts", &label));
    errors.extend(bool_key(r, "once", &label));
    errors.extend(text_key(r, "warn", TEXT_MAX, false, &label));
    errors.extend(effects_errors(v, w, r.get("effects").unwrap_or(&Value::Array(Vec::new())), &label));
    errors.extend(then_errors(v, w, r.get("then").unwrap_or(&Value::Array(Vec::new())), &label));
    if let Some(days) = r.get("restart_days") {
        if !countdown {
            errors.push(format!("{label}: restart_days is for countdowns only."));
        } else if !int_in(days, 1, DAYS_MAX) {
            errors.push(format!("{label}: restart_days must be a whole number 1..{DAYS_MAX}."));
        }
    }
    errors
}

/// M3: one gauge input (`span`: max - min, the largest move).
fn input_errors(i: usize, raw: &Value, span: i64) -> Vec<String> {
    let mut errors = Vec::new();
    let label = format!("Input {}", i + 1);
    let Some(d) = object(raw, &label, &mut errors) else { return errors };
    let on = json::s(d, "on");
    if !INPUT_ON.contains(&on.as_str()) {
        errors.push(format!("{label}: on '{on}' is not one of {}.", INPUT_ON.join(", ")));
        return errors;
    }
    errors.extend(unknown_keys(d, input_keys(&on), &label));
    let delta = json::get(d, "delta");
    if !json::is_int(delta) || json::to_int(delta) == 0 || json::to_int(delta).abs() > span {
        errors.push(format!("{label}: delta must be a whole number from -{span} to {span}, not 0."));
    }
    errors.extend(acts_key(d, "acts", &label));
    errors.extend(strings_key(d, "tags", &label));
    errors.extend(strings_key(d, "route_tags", &label));
    if let Some(x) = d.get("outcomes") {
        if !x.as_array().is_some_and(|a| a.iter().all(|o| o.as_str().is_some_and(|s| OUTCOMES.contains(&s)))) {
            errors.push(format!("{label}: outcomes must be a list of {}.", OUTCOMES.join(", ")));
        }
    }
    errors
}

/// M7: one band modifier.
fn modifier_errors(v: &View, w: &str, i: usize, raw: &Value, bands: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    let label = format!("Modifier {}", i + 1);
    let Some(d) = object(raw, &label, &mut errors) else { return errors };
    errors.extend(unknown_keys(d, &MODIFIER_KEYS, &label));
    let named = json::strings(json::get(d, "bands"));
    if !json::get(d, "bands").as_array().is_some_and(|a| !a.is_empty() && a.iter().all(Value::is_string)) {
        errors.push(format!("{label}: bands must be a list of this meter's band ids."));
    }
    for b in named.iter().filter(|b| !bands.contains(b)) {
        errors.push(format!("{label}: band '{b}' is not a band of this meter."));
    }
    errors.extend(int_key(d, "road_pct", 0, MODIFIER_PCT_MAX, &label));
    if let Some(pool) = d.get("road_encounters") {
        errors.extend(map::pair_list_errors(v, w, &label, "road_encounters", pool));
    }
    errors.extend(strings_key(d, "route_tags", &label));
    errors
}

// ---- 11.2 checkpoints and triggers ----

/// The problems of a checkpoint row (C1-C3).
pub fn checkpoint_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if !int_in(json::get(e, "act"), 1, ACT_MAX) {
        errors.push(format!("Act must be 1..{ACT_MAX}."));
    }
    if !int_in(json::get(e, "deadline"), 1, DAYS_MAX) {
        errors.push(format!("Deadline day must be 1..{DAYS_MAX}."));
    }
    let met_when = json::get(e, "met_when");
    match (json::s(e, "quest").is_empty(), json::arr(met_when).is_empty()) {
        (true, true) => errors.push("Set a quest or Met when.".into()),
        (false, false) => errors.push("Set a quest or Met when, not both.".into()),
        _ => {}
    }
    errors.extend(condition_errors(met_when, "Met when"));
    if let Some(days) = list(json::get(e, "warn_days"), "Warn days", &mut errors) {
        let ok = days.iter().all(|d| int_in(d, 1, WARN_DAY_MAX)) && days.windows(2).all(|p| json::to_int(&p[0]) > json::to_int(&p[1]));
        if !ok {
            errors.push(format!("Warn days must be whole numbers 1..{WARN_DAY_MAX}, descending."));
        }
    }
    let effects = json::get(e, "effects");
    match json::s(e, "mode").as_str() {
        "end_run" if json::s(e, "card").trim().is_empty() => errors.push("A checkpoint that ends the run needs a card.".into()),
        "go_on" if json::arr(effects).is_empty() => errors.push("A go_on checkpoint needs effects.".into()),
        _ => {}
    }
    if chars(&json::s(e, "text")) > TEXT_MAX {
        errors.push(format!("Text is longer than {TEXT_MAX} characters."));
    }
    errors.extend(effects_errors(v, w, effects, "Effects"));
    errors
}

/// The problems of a trigger row (T1-T2).
pub fn trigger_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let on = json::s(e, "on");
    if !json::s(e, "encounter").is_empty() && on != "battle" {
        errors.push("Encounter is for on battle only.".into());
    }
    if json::s(e, "outcome") != "any" && on != "battle" {
        errors.push("Outcome is for on battle only.".into());
    }
    if !json::s(e, "location").is_empty() && on != "enter" {
        errors.push("Location is for on enter only.".into());
    }
    errors.extend(condition_errors(json::get(e, "when"), "When"));
    let (effects, then) = (json::get(e, "effects"), json::get(e, "then"));
    if json::arr(effects).is_empty() && json::arr(then).is_empty() {
        errors.push("A trigger needs effects or then.".into());
    }
    errors.extend(effects_errors(v, w, effects, "Effects"));
    errors.extend(then_errors(v, w, then, "Then"));
    errors
}

/// T2: a `then` list: items with exactly one of `vignette` (an id), `battle` (an encounter), `dialogue` (a dialogue,
/// with an optional `speaker` character).
pub fn then_errors(v: &View, w: &str, raw: &Value, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(items) = raw.as_array() else { return vec![format!("{label}: then must be a list.")] };
    for (i, item) in items.iter().enumerate() {
        let il = format!("{label} then {}", i + 1);
        let Some(d) = item.as_object() else {
            errors.push(format!("{il} must be an object {{\"vignette\"}}, {{\"battle\"}} or {{\"dialogue\"}}."));
            continue;
        };
        errors.extend(unknown_keys(d, &THEN_KEYS, &il));
        let set: Vec<&str> = ["vignette", "battle", "dialogue"].into_iter().filter(|k| d.contains_key(*k)).collect();
        if set.len() != 1 {
            errors.push(format!("{il} needs exactly one of vignette, battle or dialogue."));
            continue;
        }
        let r = json::s(d, set[0]);
        match set[0] {
            "vignette" if !aegis_content::schema::is_id(&r) => errors.push(format!("{il}: vignette '{r}' must be an id.")),
            "battle" => errors.extend(ref_errors(v, w, &format!("{il} battle"), "encounters", &r)),
            "dialogue" => errors.extend(ref_errors(v, w, &format!("{il} dialogue"), "dialogues", &r)),
            _ => {}
        }
        let speaker = json::s(d, "speaker");
        if speaker.is_empty() {
            continue;
        }
        if set[0] == "dialogue" {
            errors.extend(ref_errors(v, w, &format!("{il} speaker"), "characters", &speaker));
        } else {
            errors.push(format!("{il}: speaker is for a dialogue only."));
        }
    }
    errors
}

// ---- 11.3 effects ----

/// The effect list of a meter rule, a checkpoint or a trigger (T2: the dialogue effects that need no screen and the
/// campaign effects), checked by the dialogue effect checker.
pub fn effects_errors(v: &View, w: &str, raw: &Value, label: &str) -> Vec<String> {
    dialogue::list_errors(v, w, raw, label, dialogue::EffectList::Campaign)
}

/// The meter `fx.meter` names, with the problems of the name (unknown, or not of the wanted kind).
fn meter_of<'a>(v: &'a View, w: &str, fx: &Dict, el: &str, countdown: Option<bool>, errors: &mut Vec<String>) -> Option<&'a Dict> {
    let r = json::s(fx, "meter");
    if r.is_empty() {
        errors.push(format!("{el}: choose a meter."));
        return None;
    }
    let found = ref_errors(v, w, &format!("{el} meter"), "meters", &r);
    if !found.is_empty() {
        errors.extend(found);
        return None;
    }
    let m = v.worlds.resolve(w, "meters", &r);
    match countdown {
        Some(true) if json::s(m, "kind") != "countdown" => errors.push(format!("{el}: meter '{r}' is not a countdown.")),
        Some(false) if json::s(m, "kind") == "countdown" => errors.push(format!("{el}: meter '{r}' is not a gauge.")),
        _ => {}
    }
    Some(m)
}

/// The problems of one campaign effect `fx` (11.3) written in world `w` (`el`: the effect's label).
pub fn effect_errors(v: &View, w: &str, effect: &str, fx: &Dict, el: &str) -> Vec<String> {
    let mut errors = Vec::new();
    match effect {
        "meter_add" => {
            meter_of(v, w, fx, el, None, &mut errors);
            let delta = json::get(fx, "delta");
            if !json::is_int(delta) || json::to_int(delta) == 0 || json::to_int(delta).abs() > DAYS_MAX {
                errors.push(format!("{el}: delta must be a whole number from -{DAYS_MAX} to {DAYS_MAX}, not 0."));
            }
        }
        "meter_set" => {
            let m = meter_of(v, w, fx, el, Some(false), &mut errors);
            let (lo, hi) = m.map_or((i64::MIN, i64::MAX), |m| (json::i(m, "min", 0), json::i(m, "max", 100)));
            if !int_in(json::get(fx, "value"), lo, hi) {
                errors.push(format!("{el}: value must be a whole number in the meter's min..max."));
            }
        }
        "meter_show" => {
            meter_of(v, w, fx, el, None, &mut errors);
            errors.extend(bool_key(fx, "visible", el));
        }
        "clock_start" | "clock_set" | "clock_stop" => {
            meter_of(v, w, fx, el, Some(true), &mut errors);
            let lo = if effect == "clock_start" { 1 } else { 0 };
            if effect != "clock_stop" && !int_in(json::get(fx, "days"), lo, DAYS_MAX) {
                errors.push(format!("{el}: days must be a whole number {lo}..{DAYS_MAX}."));
            }
            errors.extend(text_key(fx, "label", LABEL_MAX, false, el));
        }
        "set_act" => {
            if !int_in(json::get(fx, "act"), 0, ACT_MAX) {
                errors.push(format!("{el}: act must be a whole number 0..{ACT_MAX}."));
            }
        }
        "end" => {
            let kind = json::s(fx, "kind");
            if !END_KINDS.contains(&kind.as_str()) {
                errors.push(format!("{el}: kind '{kind}' is not one of {}.", END_KINDS.join(", ")));
            }
            errors.extend(text_key(fx, "card", TEXT_MAX, false, el));
            errors.extend(text_key(fx, "text", TEXT_MAX, false, el));
            if json::s(fx, "card").trim().is_empty() && json::s(fx, "text").trim().is_empty() {
                errors.push(format!("{el}: an end needs a card or a text."));
            }
        }
        _ => {}
    }
    errors
}

// ---- 11.4 read keys ----

/// The problem of reading flag key `key` (11.4): a reserved key must be a read key, and a meter read key must name a
/// meter of world `w` (through its lineage). "" = fine (or not a reserved key).
pub fn read_key_error(v: &View, w: &str, key: &str) -> String {
    if !rpgkit_data::reserved_key(key) {
        return String::new();
    }
    if key.starts_with("meter.") {
        return match rpgkit_data::meter_of_read_key(key) {
            None => format!("'{key}' is not a read key (meter.<id>, meter.<id>.band, meter.<id>.running)."),
            Some(id) if v.worlds.resolve(w, "meters", id).is_empty() => format!("'{key}' names no meter of this world."),
            Some(_) => String::new(),
        };
    }
    if CAMPAIGN_READ_KEYS.contains(&key) {
        String::new()
    } else {
        format!("'{key}' is not a read key ({}).", CAMPAIGN_READ_KEYS.join(", "))
    }
}

/// The reserved-key problem of writing flag key `key` ("" = fine).
pub fn write_key_error(key: &str) -> String {
    if rpgkit_data::reserved_key(key) {
        format!("flag key '{key}' is reserved (meter., campaign. and roster. keys are read-only).")
    } else {
        String::new()
    }
}

/// Every read-key problem in the conditions of world entry `e` (any list of {"key", ...} conditions at any depth, and
/// quest flag steps), and every reserved key in its flag-setting lists (`complete_flags`, `sets_flags`). Effects are
/// the effect checker's.
pub fn read_key_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    fn walk(v: &View, w: &str, raw: &Value, list_key: &str, out: &mut Vec<String>) {
        match raw {
            Value::Object(d) if d.contains_key("effect") => {}
            Value::Object(d) => {
                for (k, x) in d {
                    match (k.as_str(), x.as_str()) {
                        ("key" | "flag", Some(key)) => {
                            let why = if SET_FLAG_LISTS.contains(&list_key) { write_key_error(key) } else { read_key_error(v, w, key) };
                            if !why.is_empty() && !out.iter().any(|o| o.ends_with(&why)) {
                                out.push(format!("{}: {why}", if list_key.is_empty() { k.as_str() } else { list_key }));
                            }
                        }
                        _ => walk(v, w, x, k, out),
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|x| walk(v, w, x, list_key, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for (k, x) in e {
        walk(v, w, x, k, &mut out);
    }
    out
}
