//! Derived-stat rows (DERIVE1-S, docs/specs/DERIVE1.md 4.1; engine-only, no Godot original): the checks of one row (id,
//! name, label, formula and the names it uses, constants, caps, status modifiers, rulesets, screens, suffix, origin)
//! and across rows (unique ids and labels, formula cycles, rows naming a row that fails). The loader keeps the rows
//! that pass, in topological order (a row after the rows its formula names; ties in file order).

use super::View;
use crate::source::fields;
use aegis_content::schema::is_id;
use aegis_core::formula::{self, Formula};
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{LEVEL_INPUT, STATS};
use rpgkit_rules::status;
use serde_json::Value;
use std::collections::VecDeque;

/// The catalog kind (file data/catalog/derived_stats.json).
pub const KIND: &str = "derived_stats";
const ID_MIN: usize = 2;
const ID_MAX: usize = 24;
const NAME_MAX: usize = 24;
const LABEL_MAX: usize = 16;
const FLAT_LIMIT: f64 = 100.0;
const PERCENT_LIMIT: f64 = 1.0;
const MODIFIER_KEYS: [&str; 3] = ["status", "flat", "percent"];

/// The problems of each row (same order as the rows) and the rows that load, as indices in load order.
pub struct Checked {
    pub errors: Vec<Vec<String>>,
    pub order: Vec<usize>,
}

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// A base stat in any case (`know` collides with `KNOW`).
fn is_base_stat(s: &str) -> bool {
    STATS.iter().any(|b| b.eq_ignore_ascii_case(s))
}

/// The schema options of a field of the kind (rulesets, show_on, suffix, origin).
fn options(key: &str) -> Vec<String> {
    fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// A whole number within `limit` (JSON numbers load as floats).
fn whole_within(v: &Value, limit: f64) -> bool {
    v.as_f64().is_some_and(|x| x.fract() == 0.0 && x.abs() <= limit)
}

/// The checks of rows as the loader read them (normalized), on their own and against each other.
pub fn check_rows(rows: &[Dict]) -> Checked {
    let ids: Vec<String> = rows.iter().map(|r| json::s(r, "id")).collect();
    let mut errors: Vec<Vec<String>> = Vec::new();
    let mut parsed: Vec<Option<Formula>> = Vec::new();
    for i in 0..rows.len() {
        let (found, f) = row_errors(rows, &ids, i);
        errors.push(found);
        parsed.push(f);
    }
    // the graph: a row -> the rows its formula names (first row of an id)
    let deps: Vec<Vec<usize>> = parsed
        .iter()
        .map(|f| f.as_ref().map(|f| f.names().iter().filter_map(|n| ids.iter().position(|id| !id.is_empty() && id == n)).collect()).unwrap_or_default())
        .collect();
    for (i, found) in errors.iter_mut().enumerate() {
        if let Some(path) = cycle_from(i, &deps) {
            let names: Vec<&str> = path.iter().map(|&j| ids[j].as_str()).collect();
            found.push(format!("Formula: cycle {}.", names.join(" -> ")));
        }
    }
    let mut ok: Vec<bool> = errors.iter().map(Vec::is_empty).collect();
    loop {
        let mut changed = false;
        for i in 0..rows.len() {
            if !ok[i] {
                continue;
            }
            if let Some(&j) = deps[i].iter().find(|&&j| !ok[j]) {
                ok[i] = false;
                changed = true;
                errors[i].push(format!("Formula: derived stat '{}' has errors, so this row does not load either.", ids[j]));
            }
        }
        if !changed {
            break;
        }
    }
    let mut placed = vec![false; rows.len()];
    let mut order = Vec::new();
    while let Some(i) = (0..rows.len()).find(|&i| ok[i] && !placed[i] && deps[i].iter().all(|&j| placed[j])) {
        placed[i] = true;
        order.push(i);
    }
    Checked { errors, order }
}

/// The shortest path from row `i` back to itself over `deps` (`[i, ..., i]`), if the row is in a cycle.
fn cycle_from(i: usize, deps: &[Vec<usize>]) -> Option<Vec<usize>> {
    let mut parent = vec![usize::MAX; deps.len()];
    let mut seen = vec![false; deps.len()];
    seen[i] = true;
    let mut queue = VecDeque::from([i]);
    while let Some(x) = queue.pop_front() {
        for &y in &deps[x] {
            if y == i {
                let mut back = Vec::new();
                let mut c = x;
                while c != i {
                    back.push(c);
                    c = parent[c];
                }
                back.reverse();
                let mut path = vec![i];
                path.extend(back);
                path.push(i);
                return Some(path);
            }
            if !seen[y] {
                seen[y] = true;
                parent[y] = x;
                queue.push_back(y);
            }
        }
    }
    None
}

/// The own problems of row `i` (its fields, its names against the other rows' ids) and its parsed formula.
fn row_errors(rows: &[Dict], ids: &[String], i: usize) -> (Vec<String>, Option<Formula>) {
    let e = &rows[i];
    let mut out = Vec::new();
    let id = &ids[i];
    if id.is_empty() {
        out.push("ID is required.".into());
    } else if !is_id(id) {
        out.push(format!("ID '{id}' must be lowercase snake_case (a-z, 0-9, _)."));
    } else if !(ID_MIN..=ID_MAX).contains(&chars(id)) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters long."));
    } else if is_base_stat(id) {
        out.push(format!("ID '{id}' is a base stat."));
    } else if id == LEVEL_INPUT {
        out.push(format!("ID '{id}' is the unit level input."));
    } else if formula::is_function(id) {
        out.push(format!("ID '{id}' is a formula function."));
    }
    if !id.is_empty() && ids[..i].contains(id) {
        out.push(format!("ID '{id}' is already used by another derived stat."));
    }
    let name = json::s(e, "name");
    if name.trim().is_empty() {
        out.push("Name is required.".into());
    } else if chars(&name) > NAME_MAX {
        out.push(format!("Name is {} characters (max {NAME_MAX}).", chars(&name)));
    }
    let label = json::s(e, "label");
    if chars(&label) > LABEL_MAX {
        out.push(format!("Label is {} characters (max {LABEL_MAX}).", chars(&label)));
    }
    if let Some(j) = (0..i).find(|&j| !label.is_empty() && json::s(&rows[j], "label") == label) {
        out.push(format!("Label '{label}' is already used by derived stat '{}'.", ids[j]));
    }
    let constants = constant_errors(json::get(e, "constants"), ids, &mut out);
    let parsed = match Formula::parse(&json::s(e, "formula")) {
        Err(err) => {
            out.push(format!("Formula: {err}."));
            None
        }
        Ok(f) => {
            for n in f.names() {
                let known = STATS.contains(&n.as_str()) || n == LEVEL_INPUT || constants.contains(n) || ids.contains(n);
                if known {
                    continue;
                }
                if is_base_stat(n) {
                    out.push(format!("Formula: base stats are upper case ('{}', not '{n}').", n.to_ascii_uppercase()));
                } else {
                    out.push(format!("Formula: unknown name '{n}' (a base stat {}, {LEVEL_INPUT}, a derived stat id or a constant).", STATS.join(", ")));
                }
            }
            Some(f)
        }
    };
    let (min, max) = (json::i(e, "min", 0), json::i(e, "max", 100));
    if min > max {
        out.push(format!("Min {min} is above max {max}."));
    }
    status_modifier_errors(json::get(e, "status_modifiers"), &mut out);
    for (key, label) in [("rulesets", "Rulesets"), ("show_on", "Show on")] {
        let allowed = options(key);
        let mut seen: Vec<String> = Vec::new();
        for v in json::sget(e, key) {
            if !allowed.contains(&v) {
                out.push(format!("{label}: '{v}' is not one of {}.", allowed.join(", ")));
            } else if seen.contains(&v) {
                out.push(format!("{label} lists '{v}' twice."));
            }
            seen.push(v);
        }
    }
    let suffix = json::s(e, "suffix");
    if !options("suffix").contains(&suffix) {
        out.push(format!("Suffix must be empty or '%' (not '{suffix}')."));
    }
    let origin = json::s(e, "origin");
    let origins = options("origin");
    if !origins.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", origins.join(", ")));
    }
    (out, parsed)
}

/// The constants' problems; returns the names that may be used in the formula.
fn constant_errors(v: &Value, ids: &[String], out: &mut Vec<String>) -> Vec<String> {
    let Some(map) = v.as_object() else {
        out.push("Constants must be an object of name to number.".into());
        return Vec::new();
    };
    let mut names = Vec::new();
    for (k, x) in map {
        if !formula::is_name(k) {
            out.push(format!("Constants: '{k}' is not a name (a letter or _ first, then letters, digits or _; not a function)."));
        } else if is_base_stat(k) {
            out.push(format!("Constants: '{k}' is a base stat."));
        } else if k == LEVEL_INPUT {
            out.push(format!("Constants: '{k}' is the unit level input."));
        } else if ids.contains(k) {
            out.push(format!("Constants: '{k}' is a derived stat id."));
        }
        if !x.as_f64().is_some_and(|n| n.is_finite() && n.abs() <= formula::MAX_NUMBER) {
            out.push(format!("Constants: '{k}' must be a number up to {} in size.", formula::MAX_NUMBER));
        }
        names.push(k.clone());
    }
    names
}

fn status_modifier_errors(v: &Value, out: &mut Vec<String>) {
    let Some(list) = v.as_array() else {
        out.push("Status modifiers must be a list.".into());
        return;
    };
    let mut seen: Vec<String> = Vec::new();
    for (i, raw) in list.iter().enumerate() {
        let n = i + 1;
        let Some(m) = raw.as_object() else {
            out.push(format!("Status modifier {n} is not an object."));
            continue;
        };
        for k in m.keys().filter(|k| !MODIFIER_KEYS.contains(&k.as_str())) {
            out.push(format!("Status modifier {n}: key '{k}' is not allowed ({}).", MODIFIER_KEYS.join(", ")));
        }
        let st = json::s(m, "status");
        if status::row_of(&st).is_none() {
            out.push(format!("Status modifier {n}: unknown status '{st}'."));
        } else if seen.contains(&st) {
            out.push(format!("Status modifier {n}: status '{st}' is listed twice."));
        }
        seen.push(st);
        let flat = m.get("flat").cloned().unwrap_or(Value::from(0));
        let percent = m.get("percent").cloned().unwrap_or(Value::from(0.0));
        let flat_ok = whole_within(&flat, FLAT_LIMIT);
        let percent_ok = percent.as_f64().is_some_and(|x| x.abs() <= PERCENT_LIMIT);
        if !flat_ok {
            out.push(format!("Status modifier {n}: flat must be a whole number -{FLAT_LIMIT}..{FLAT_LIMIT}."));
        }
        if !percent_ok {
            out.push(format!("Status modifier {n}: percent must be -{PERCENT_LIMIT:.1}..{PERCENT_LIMIT:.1}."));
        }
        if flat_ok && percent_ok && flat.as_f64() == Some(0.0) && percent.as_f64() == Some(0.0) {
            out.push(format!("Status modifier {n}: flat and percent are both 0; remove it."));
        }
    }
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): every row problem it has among
/// the loaded rows, cycles included. The entry is checked last (its old row taken out), so the duplicate id and label
/// checks, which look at earlier rows, see every other row.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    check_rows(&rows).errors.pop().unwrap_or_default()
}
