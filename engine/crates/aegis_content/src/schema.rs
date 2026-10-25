//! The schema-driven entry rules (ContentSchema / WorldSchema): a game's schema is
//! {"content": {kind: [field]}, "world": {kind: [field]}}, each field {key, type, default, min, max, options, when,
//! internal, ref, label}. normalize coerces an entry to its fields, to_storage gives the stored form, field_errors and
//! ref_problems are the schema checks.

use aegis_core::json::{self, gstr, int, to_bool_or, to_float_or, to_int_or, Dict};
use serde_json::Value;

/// The fields of `kind` in `section` ("content" or "world"); none for an unknown kind.
pub fn fields<'a>(schema: &'a Value, section: &str, kind: &str) -> &'a [Value] {
    schema[section][kind].as_array().map(|a| a.as_slice()).unwrap_or(&[])
}

pub fn has_field(schema: &Value, section: &str, kind: &str, key: &str) -> bool {
    fields(schema, section, kind).iter().any(|f| json::s(json::dict(f), "key") == key)
}

/// A field read as GDScript `str()`, or `fallback` when missing.
pub fn sget(d: &Dict, key: &str, fallback: &str) -> String {
    d.get(key).map(gstr).unwrap_or_else(|| fallback.to_string())
}

/// ContentSchema._json_ints: whole floats inside free-form values become ints.
fn json_ints(v: &Value) -> Value {
    match v {
        Value::Number(n) if n.is_f64() => {
            let x = n.as_f64().unwrap_or(0.0);
            if x == x.floor() {
                int(json::roundi(x))
            } else {
                v.clone()
            }
        }
        Value::Array(a) => Value::Array(a.iter().map(json_ints).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), json_ints(v))).collect()),
        _ => v.clone(),
    }
}

/// ContentSchema._copy: collections get whole numbers back as ints; scalars stay as they are.
pub fn copy(v: &Value) -> Value {
    if v.is_array() || v.is_object() {
        json_ints(v)
    } else {
        v.clone()
    }
}

/// Math::snapped.
pub fn snapped(x: f64, step: f64) -> f64 {
    (x / step + 0.5).floor() * step
}

/// ContentSchema.coerce.
fn coerce(f: &Dict, value: &Value) -> Value {
    let fallback = json::get(f, "default");
    match json::s(f, "type").as_str() {
        "int" => {
            let mut n = to_int_or(value, to_int_or(fallback, 0));
            if let Some(min) = f.get("min") {
                n = n.max(json::to_int(min));
            }
            if let Some(max) = f.get("max") {
                n = n.min(json::to_int(max));
            }
            int(n)
        }
        "float" => {
            let mut x = to_float_or(value, to_float_or(fallback, 0.0));
            if let Some(min) = f.get("min") {
                x = x.max(to_float_or(min, 0.0));
            }
            if let Some(max) = f.get("max") {
                x = x.min(to_float_or(max, 0.0));
            }
            Value::from(snapped(x, 0.0001))
        }
        "bool" => Value::Bool(to_bool_or(value, to_bool_or(fallback, false))),
        "refs" | "strings" | "multi_pick" => {
            let list: Vec<String> = value.as_array().map(|a| a.iter().map(gstr).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
            json::strs(&list)
        }
        "stats" => {
            let mut stats = Dict::new();
            if let Value::Object(src) = value {
                for (k, raw) in src {
                    if raw.is_number() {
                        let amount = json::to_int(raw);
                        if amount != 0 {
                            stats.insert(k.clone(), int(amount));
                        }
                    }
                }
            }
            Value::Object(stats)
        }
        "json" | "modifier_table" | "tier_table" | "hand_grid" => copy(if value.is_array() || value.is_object() { value } else { fallback }),
        "table" => match value {
            Value::Object(src) => Value::Object(src.iter().map(|(k, v)| (k.clone(), whole_level(v))).collect()),
            Value::Array(_) => copy(value),
            _ => copy(fallback),
        },
        "dict_pick" => copy(if value.is_object() { value } else { fallback }),
        _ => Value::from(if value.is_null() { gstr(fallback) } else { gstr(value) }),
    }
}

/// ClassArchetype.whole_level: a whole float becomes an int; anything else stays.
fn whole_level(v: &Value) -> Value {
    match v.as_f64() {
        Some(x) if v.is_f64() && x == x.floor() => int(json::roundi(x)),
        _ => v.clone(),
    }
}

/// ContentSchema.normalize (`world` false) / WorldSchema.normalize (`world` true: "_" keys are dropped).
pub fn normalize(fields: &[Value], entry: &Dict, world: bool) -> Dict {
    let mut out = Dict::new();
    for f in fields {
        let f = json::dict(f);
        let key = json::s(f, "key");
        let value = entry.get(&key).unwrap_or_else(|| json::get(f, "default"));
        out.insert(key, coerce(f, value));
    }
    for (k, v) in entry {
        if !out.contains_key(k) && !(world && k.starts_with('_')) {
            out.insert(k.clone(), copy(v));
        }
    }
    out
}

/// ContentSchema.is_visible: a field with a `when` {key, in | not_in} shows only for matching entries.
pub fn visible(f: &Dict, full: &Dict) -> bool {
    let Some(cond) = f.get("when").and_then(Value::as_object) else { return true };
    let value = sget(full, &json::s(cond, "key"), "");
    if let Some(allowed) = cond.get("in").and_then(Value::as_array) {
        return allowed.iter().any(|v| gstr(v) == value);
    }
    if let Some(denied) = cond.get("not_in").and_then(Value::as_array) {
        return !denied.iter().any(|v| gstr(v) == value);
    }
    true
}

/// ContentSchema.to_storage of a normalized row: hidden and internal fields left out, and the fields `omit(key,
/// value)` names (the game's own rules); unknown keys kept.
pub fn to_storage(schema: &Value, section: &str, kind: &str, full: &Dict, omit: impl Fn(&str, &Value) -> bool) -> Dict {
    let mut out = Dict::new();
    for f in fields(schema, section, kind) {
        let f = json::dict(f);
        let key = json::s(f, "key");
        if !visible(f, full) || json::b(f, "internal", false) {
            continue;
        }
        let v = json::get(full, &key);
        if omit(&key, v) {
            continue;
        }
        out.insert(key, v.clone());
    }
    for (k, v) in full {
        if !out.contains_key(k) && !has_field(schema, section, kind, k) {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

/// WorldSchema.to_storage of a raw entry (normalized first; an empty `override` is left out).
pub fn world_storage(schema: &Value, kind: &str, entry: &Dict) -> Dict {
    let full = normalize(fields(schema, "world", kind), entry, true);
    let mut out = Dict::new();
    for f in fields(schema, "world", kind) {
        let key = json::s(json::dict(f), "key");
        if key == "override" && sget(&full, &key, "").is_empty() {
            continue;
        }
        out.insert(key.clone(), json::get(&full, &key).clone());
    }
    for (k, v) in &full {
        if !out.contains_key(k) && !has_field(schema, "world", kind, k) {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

/// An entry id: lowercase snake_case starting with a letter.
pub fn is_id(id: &str) -> bool {
    let mut c = id.chars();
    c.next().is_some_and(|f| f.is_ascii_lowercase()) && c.all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '_')
}

/// The values of `entry` (raw, before `normalize`) whose JSON type does not fit their field's type (normalize would
/// coerce them silently): ints, numbers, bools, text, lists of text, objects. Keys without a field are not checked.
pub fn type_errors(fields: &[Value], entry: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    for f in fields {
        let f = json::dict(f);
        let key = json::s(f, "key");
        let Some(v) = entry.get(&key) else { continue };
        let (ok, wants) = match json::s(f, "type").as_str() {
            "int" => (json::is_int(v), "a whole number"),
            "float" => (v.is_number(), "a number"),
            "bool" => (v.is_boolean(), "true or false"),
            "id" | "string" | "text" | "ref" | "enum" => (v.is_string(), "text"),
            "refs" | "strings" | "multi_pick" => (v.as_array().is_some_and(|a| a.iter().all(Value::is_string)), "a list of text"),
            "stats" | "dict_pick" => (v.is_object(), "an object"),
            "json" => (v.is_object() || v.is_array(), "an object or a list"),
            _ => (true, ""),
        };
        if !ok {
            errors.push(format!("{}: {v} is not {wants}.", json::s_or(f, "label", &key)));
        }
    }
    errors
}

/// Schema checks of a normalized entry (ContentSchema.field_errors / WorldSchema shape): id, name, enum options.
pub fn field_errors(schema: &Value, section: &str, kind: &str, full: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let id = sget(full, "id", "");
    if id.is_empty() {
        errors.push("ID is required.".into());
    } else if !is_id(&id) {
        errors.push(format!("ID '{id}' must be lowercase snake_case (a-z, 0-9, _)."));
    }
    if has_field(schema, section, kind, "name") && sget(full, "name", "").trim().is_empty() {
        errors.push("Name is required.".into());
    }
    for f in fields(schema, section, kind) {
        let f = json::dict(f);
        if json::s(f, "type") != "enum" || !visible(f, full) {
            continue;
        }
        if let Some(options) = f.get("options").and_then(Value::as_array) {
            let value = sget(full, &json::s(f, "key"), "");
            if !options.iter().any(|o| gstr(o) == value) {
                errors.push(format!("{}: '{value}' is not one of {}.", json::s(f, "label"), options.iter().map(gstr).collect::<Vec<_>>().join(", ")));
            }
        }
    }
    errors
}

/// References (`ref` / `refs` fields of `section`) in `rows` ({kind: [entry]}) that name no entry: `ids(target)`
/// gives the known ids of a target kind, None for a kind that is not checked.
pub fn ref_problems(schema: &Value, section: &str, rows: &Dict, ids: impl Fn(&str) -> Option<Vec<String>>) -> Vec<String> {
    let mut out = Vec::new();
    for (kind, list) in rows {
        for f in fields(schema, section, kind) {
            let f = json::dict(f);
            let target = json::s(f, "ref");
            if target.is_empty() || !matches!(json::s(f, "type").as_str(), "ref" | "refs") {
                continue;
            }
            let Some(known) = ids(&target) else { continue };
            let key = json::s(f, "key");
            for e in json::arr(list) {
                let e = json::dict(e);
                if !visible(f, e) {
                    continue;
                }
                let v = json::get(e, &key);
                let refs: Vec<String> = if v.is_array() { json::arr(v).iter().map(gstr).collect() } else { vec![gstr(v)] };
                for r in refs.into_iter().filter(|r| !r.is_empty() && r != "<null>" && r != "none") {
                    if !known.contains(&r) {
                        out.push(format!("{kind} '{}': {} names the unknown {target} '{r}'.", json::s(e, "id"), json::s(f, "label")));
                    }
                }
            }
        }
    }
    out
}
