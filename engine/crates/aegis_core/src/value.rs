//! Lenient JSON field access, mirroring Godot's `DictUtil`: Godot's JSON round trip turns ints into floats and
//! back, so every numeric read accepts either, and a missing or mistyped field gives the schema default.

use indexmap::IndexMap;
use serde_json::{Map, Value};

pub type Obj = Map<String, Value>;

pub fn to_int(v: &Value, default: i32) -> i32 {
    match v {
        Value::Number(n) => n.as_i64().map(|i| i as i32).or_else(|| n.as_f64().map(|f| f as i32)).unwrap_or(default),
        Value::Bool(b) => *b as i32,
        Value::String(s) => s.trim().parse::<f64>().map(|f| f as i32).unwrap_or(default),
        _ => default,
    }
}

pub fn to_float(v: &Value, default: f64) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(default),
        Value::Bool(b) => *b as i32 as f64,
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(default),
        _ => default,
    }
}

pub fn to_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

pub fn int(o: &Obj, key: &str, default: i32) -> i32 {
    o.get(key).map(|v| to_int(v, default)).unwrap_or(default)
}

pub fn float(o: &Obj, key: &str, default: f64) -> f64 {
    o.get(key).map(|v| to_float(v, default)).unwrap_or(default)
}

pub fn boolean(o: &Obj, key: &str, default: bool) -> bool {
    match o.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(default),
        _ => default,
    }
}

/// A string field; `default` when missing or null.
pub fn string(o: &Obj, key: &str, default: &str) -> String {
    match o.get(key) {
        Some(Value::Null) | None => default.to_string(),
        Some(v) => to_str(v),
    }
}

pub fn strings(o: &Obj, key: &str) -> Vec<String> {
    match o.get(key) {
        Some(Value::Array(a)) => a.iter().map(to_str).collect(),
        _ => Vec::new(),
    }
}

/// `{key: int}` in file order.
pub fn int_map(o: &Obj, key: &str) -> IndexMap<String, i32> {
    match o.get(key) {
        Some(Value::Object(m)) => m.iter().map(|(k, v)| (k.clone(), to_int(v, 0))).collect(),
        _ => IndexMap::new(),
    }
}

pub fn obj<'a>(o: &'a Obj, key: &str) -> Option<&'a Obj> {
    o.get(key).and_then(Value::as_object)
}

pub fn array<'a>(o: &'a Obj, key: &str) -> &'a [Value] {
    match o.get(key) {
        Some(Value::Array(a)) => a.as_slice(),
        _ => &[],
    }
}
