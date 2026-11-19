//! Facility rules of the world checks (PR6-8, `docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 4; engine-only, no Godot
//! original): a `facilities` row's keeper and level rows (levels `LEVEL_MIN` up to at most `LEVEL_MAX`, each once and
//! in order, with a name, a purse cost, known materials, flag conditions and known effects), and a shop row's
//! `supply_price_mult_by_act` (B2).

use super::campaign::{int_in, int_key, unknown_keys};
use super::quest::condition_errors;
use super::View;
use aegis_core::json::{self, Dict};
use rpgkit_data::facility::{self as f, LEVEL_EFFECTS_MAX, LEVEL_KEYS, LEVEL_MAX, LEVEL_MIN, MATERIAL_COUNT_MAX, SCRAP_COST_MAX};
use serde_json::Value;

/// The problems of a `facilities` row beyond its field table (the keeper ref resolves there).
pub fn facility_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if json::s(e, "keeper_character").is_empty() {
        errors.push("Facilities: choose the keeper.".into());
    }
    let Some(levels) = json::get(e, "levels").as_array() else {
        errors.push("Facilities: levels must be a list of level rows.".into());
        return errors;
    };
    if levels.is_empty() {
        errors.push(format!("Facilities: levels need at least level {LEVEL_MIN}."));
    }
    if levels.len() as i64 > LEVEL_MAX - LEVEL_MIN + 1 {
        errors.push(format!("Facilities: at most {} levels ({LEVEL_MIN}..{LEVEL_MAX}).", LEVEL_MAX - LEVEL_MIN + 1));
    }
    for (i, raw) in levels.iter().enumerate() {
        let label = format!("Facilities level {}", i + 1);
        let Some(lv) = raw.as_object() else {
            errors.push(format!("{label}: must be an object {{{}}}.", LEVEL_KEYS.join(", ")));
            continue;
        };
        errors.extend(unknown_keys(lv, &LEVEL_KEYS, &label));
        let want = LEVEL_MIN + i as i64;
        if !lv.get("level").is_some_and(|l| int_in(l, want, want)) {
            errors.push(format!("{label}: level must be {want} (levels count up from {LEVEL_MIN})."));
        }
        if json::s(lv, "name").trim().is_empty() {
            errors.push(format!("{label}: name is required."));
        }
        errors.extend(int_key(lv, "scrap_cost", 0, SCRAP_COST_MAX, &label));
        match lv.get("materials") {
            None => {}
            Some(Value::Object(m)) => {
                for (item, n) in m {
                    if !v.has("items", item) {
                        errors.push(format!("{label}: unknown material item '{item}'."));
                    }
                    if !int_in(n, 1, MATERIAL_COUNT_MAX) {
                        errors.push(format!("{label}: material '{item}' count must be 1..{MATERIAL_COUNT_MAX}."));
                    }
                }
            }
            Some(_) => errors.push(format!("{label}: materials must be an object {{item id: count}}.")),
        }
        if let Some(c) = lv.get("requires_flags") {
            errors.extend(condition_errors(c, &format!("{label} requires flags")));
        }
        errors.extend(effect_errors(lv.get("effects"), &label));
    }
    errors
}

/// The effects of a level row: a list of at most `LEVEL_EFFECTS_MAX` known effects with their keys.
fn effect_errors(raw: Option<&Value>, label: &str) -> Vec<String> {
    let Some(raw) = raw else { return Vec::new() };
    let Some(list) = raw.as_array() else {
        return vec![format!("{label}: effects must be a list of {{effect, ...}}.")];
    };
    let mut errors = Vec::new();
    if list.len() > LEVEL_EFFECTS_MAX {
        errors.push(format!("{label}: at most {LEVEL_EFFECTS_MAX} effects."));
    }
    let names: Vec<&str> = f::FACILITY_EFFECTS.iter().map(|x| x.0).collect();
    for (i, fx) in list.iter().enumerate() {
        let el = format!("{label} effect {}", i + 1);
        let Some(fx) = fx.as_object() else {
            errors.push(format!("{el}: must be an object {{effect, ...}}."));
            continue;
        };
        let name = json::s(fx, "effect");
        let Some((_, keys)) = f::FACILITY_EFFECTS.iter().find(|x| x.0 == name) else {
            errors.push(format!("{el}: unknown effect '{name}' (use {}).", names.join(", ")));
            continue;
        };
        let mut allowed = vec!["effect"];
        allowed.extend(keys.iter());
        errors.extend(unknown_keys(fx, &allowed, &el));
        match name.as_str() {
            f::FX_SHOP_PCT | f::FX_RECIPE_PCT => {
                let (lo, hi) = f::PCT_RANGE;
                if !fx.get("pct").is_some_and(|p| int_in(p, lo, hi)) {
                    errors.push(format!("{el}: pct must be a whole number {lo}..{hi}."));
                }
                if fx.get("recipe_tags").is_some_and(|t| !t.as_array().is_some_and(|a| a.iter().all(Value::is_string))) {
                    errors.push(format!("{el}: recipe_tags must be a list of tags."));
                }
            }
            f::FX_REST_UPKEEP => {
                let (lo, hi) = f::REST_DELTA;
                if !fx.get("delta").is_some_and(|d| int_in(d, lo, hi)) {
                    errors.push(format!("{el}: delta must be a whole number {lo}..{hi}."));
                }
            }
            f::FX_SET_FLAG => {
                let key = json::s(fx, "key");
                if !super::quest::flag_key_ok(&key) {
                    errors.push(format!("{el}: key '{key}' must be a flag key [a-z0-9_.]."));
                } else if rpgkit_data::reserved_key(&key) {
                    errors.push(format!("{el}: key '{key}' is a read-only key."));
                }
                if !fx.get("value").is_some_and(|x| x.is_boolean() || x.is_string() || json::is_int(x)) {
                    errors.push(format!("{el}: value must be a bool, int or text."));
                }
            }
            _ => {}
        }
    }
    errors
}

/// PR6-8 (B2): a shop row's `supply_price_mult_by_act` `{"<act>": mult}`: an object whose keys are acts 0..9 and whose
/// values are numbers in `SUPPLY_MULT_RANGE` with at most two decimals ({} = none).
pub fn supply_mult_errors(e: &Dict) -> Vec<String> {
    let Some(raw) = e.get(f::SUPPLY_MULT_KEY) else { return Vec::new() };
    let Some(map) = raw.as_object() else {
        return vec!["Supply price by act must be an object {act: mult}.".into()];
    };
    let (lo, hi) = f::SUPPLY_MULT_RANGE;
    let mut errors = Vec::new();
    for (act, m) in map {
        if !act.parse::<i64>().is_ok_and(|a| (0..=9).contains(&a)) || act.starts_with('+') || (act.len() > 1 && act.starts_with('0')) {
            errors.push(format!("Supply price by act: '{act}' must be an act 0..9."));
        }
        match m.as_f64() {
            Some(x) if x >= lo - 0.00001 && x <= hi + 0.00001 && (x * 100.0 - (x * 100.0).round()).abs() <= 0.001 => {}
            _ => errors.push(format!("Supply price by act {act}: mult must be a number {lo:.1}..{hi:.1} with at most two decimals.")),
        }
    }
    if json::i(e, "supply_price", 0) == 0 && !map.is_empty() {
        errors.push("Supply price by act needs a supply price (the shop sells no supplies).".into());
    }
    errors
}
