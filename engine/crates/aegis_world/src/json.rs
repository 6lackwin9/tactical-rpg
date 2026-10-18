//! GDScript Variant semantics over serde_json values (DictUtil, JsonUtil, str()). The world layer keeps its entries
//! and blobs as JSON objects like Godot keeps Dictionaries, so the ports read line by line like the originals.
//! serde_json runs with `preserve_order`: object order is insertion order (Godot's Dictionary order); erase with
//! `shift_remove` to keep it.

use serde_json::{Map, Number, Value};
use std::sync::OnceLock;

pub type Dict = Map<String, Value>;

pub fn empty() -> &'static Dict {
    static E: OnceLock<Dict> = OnceLock::new();
    E.get_or_init(Dict::new)
}

pub const NULL: Value = Value::Null;

pub fn is_int(v: &Value) -> bool {
    matches!(v, Value::Number(n) if n.is_i64() || n.is_u64())
}

pub fn is_float(v: &Value) -> bool {
    matches!(v, Value::Number(n) if n.is_f64())
}

pub fn int(i: i64) -> Value {
    Value::Number(Number::from(i))
}

/// Godot's `roundi`: half away from zero.
pub fn roundi(x: f64) -> i64 {
    x.round() as i64
}

/// DictUtil.to_int.
pub fn to_int_or(v: &Value, fallback: i64) -> i64 {
    match v {
        Value::Number(n) => n.as_i64().unwrap_or_else(|| roundi(n.as_f64().unwrap_or(0.0))),
        Value::Bool(b) => *b as i64,
        Value::String(s) => s.parse::<f64>().map(roundi).unwrap_or(fallback),
        _ => fallback,
    }
}

pub fn to_int(v: &Value) -> i64 {
    to_int_or(v, 0)
}

/// DictUtil.to_float.
pub fn to_float_or(v: &Value, fallback: f64) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(fallback),
        Value::String(s) => s.parse::<f64>().unwrap_or(fallback),
        _ => fallback,
    }
}

/// DictUtil.to_bool.
pub fn to_bool_or(v: &Value, fallback: bool) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        Value::String(s) => matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"),
        _ => fallback,
    }
}

pub fn to_bool(v: &Value) -> bool {
    to_bool_or(v, false)
}

/// GDScript `str(v)`.
pub fn gstr(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "<null>".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => match n.as_i64() {
            Some(i) => i.to_string(),
            None => {
                let f = n.as_f64().unwrap_or(0.0);
                if f == f.floor() && f.abs() < 1e15 {
                    format!("{f:.1}")
                } else {
                    f.to_string()
                }
            }
        },
        Value::Array(a) => format!("[{}]", a.iter().map(gstr).collect::<Vec<_>>().join(", ")),
        Value::Object(o) => {
            format!("{{ {} }}", o.iter().map(|(k, v)| format!("\"{k}\": {}", gstr(v))).collect::<Vec<_>>().join(", "))
        }
    }
}

/// `d.get(key, null)`.
pub fn get<'a>(d: &'a Dict, key: &str) -> &'a Value {
    d.get(key).unwrap_or(&NULL)
}

/// `str(d.get(key, ""))`.
pub fn s(d: &Dict, key: &str) -> String {
    match d.get(key) {
        None => String::new(),
        Some(v) => gstr(v),
    }
}

/// `str(d.get(key, fallback))`.
pub fn s_or(d: &Dict, key: &str, fallback: &str) -> String {
    match d.get(key) {
        None => fallback.to_string(),
        Some(v) => gstr(v),
    }
}

/// `DictUtil.to_int(d.get(key, fallback), fallback)`.
pub fn i(d: &Dict, key: &str, fallback: i64) -> i64 {
    d.get(key).map(|v| to_int_or(v, fallback)).unwrap_or(fallback)
}

/// `DictUtil.to_bool(d.get(key, fallback))`.
pub fn b(d: &Dict, key: &str, fallback: bool) -> bool {
    d.get(key).map(|v| to_bool_or(v, fallback)).unwrap_or(fallback)
}

pub fn f(d: &Dict, key: &str, fallback: f64) -> f64 {
    d.get(key).map(|v| to_float_or(v, fallback)).unwrap_or(fallback)
}

/// DictUtil.dict_of.
pub fn dict(v: &Value) -> &Dict {
    v.as_object().unwrap_or_else(|| empty())
}

/// DictUtil.dict_of(d.get(key, {})).
pub fn dget<'a>(d: &'a Dict, key: &str) -> &'a Dict {
    d.get(key).map(dict).unwrap_or_else(|| empty())
}

/// DictUtil.array_of.
pub fn arr(v: &Value) -> &[Value] {
    match v {
        Value::Array(a) => a.as_slice(),
        _ => &[],
    }
}

pub fn aget<'a>(d: &'a Dict, key: &str) -> &'a [Value] {
    d.get(key).map(arr).unwrap_or(&[])
}

/// JsonUtil.strings.
pub fn strings(v: &Value) -> Vec<String> {
    arr(v).iter().map(gstr).collect()
}

pub fn sget(d: &Dict, key: &str) -> Vec<String> {
    d.get(key).map(strings).unwrap_or_default()
}

/// JsonUtil.ints.
pub fn ints(v: &Value) -> indexmap::IndexMap<String, i64> {
    match v {
        Value::Object(o) => o.iter().map(|(k, v)| (k.clone(), to_int(v))).collect(),
        _ => indexmap::IndexMap::new(),
    }
}

/// JsonUtil.normalize: a deep copy with whole floats turned into ints.
pub fn normalize(v: &Value) -> Value {
    match v {
        Value::Number(n) if n.is_f64() => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.is_finite() && f == f.floor() && f.abs() < 9007199254740992.0 {
                int(f as i64)
            } else {
                v.clone()
            }
        }
        Value::Array(a) => Value::Array(a.iter().map(normalize).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), normalize(v))).collect()),
        _ => v.clone(),
    }
}

pub fn normalize_dict(d: &Dict) -> Dict {
    d.iter().map(|(k, v)| (k.clone(), normalize(v))).collect()
}

/// A deep copy with every object's keys sorted: Godot's JSON.stringify sorts keys by default, so a value written by
/// a Godot save (and read back) has its dictionaries in key order.
pub fn sorted(v: &Value) -> Value {
    match v {
        Value::Array(a) => Value::Array(a.iter().map(sorted).collect()),
        Value::Object(o) => {
            let mut keys: Vec<&String> = o.keys().collect();
            keys.sort();
            Value::Object(keys.into_iter().map(|k| (k.clone(), sorted(&o[k]))).collect())
        }
        _ => v.clone(),
    }
}

/// JsonUtil.deep_equal: 2 == 2.0, object key order ignored.
pub fn deep_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| deep_equal(p, q)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| deep_equal(v, w))),
        _ => a == b,
    }
}

/// JsonUtil.id_ok: non-empty [a-z0-9_].
pub fn id_ok(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// `typeof(a) == typeof(b) and a == b` (strict GDScript equality; 1 is not 1.0 or true).
pub fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let xi = x.is_i64() || x.is_u64();
            let yi = y.is_i64() || y.is_u64();
            xi == yi && x.as_f64() == y.as_f64()
        }
        _ => std::mem::discriminant(a) == std::mem::discriminant(b) && a == b,
    }
}

pub fn strs(list: &[String]) -> Value {
    Value::Array(list.iter().map(|x| Value::String(x.clone())).collect())
}

/// Builds an object from (key, value) pairs in order.
pub fn obj<const N: usize>(pairs: [(&str, Value); N]) -> Dict {
    let mut d = Dict::new();
    for (k, v) in pairs {
        d.insert(k.to_string(), v);
    }
    d
}

pub fn ok_result(ok: bool, errors: &[String]) -> Dict {
    obj([("ok", Value::Bool(ok)), ("errors", strs(errors))])
}

pub fn fail(message: &str) -> Dict {
    ok_result(false, &[message.to_string()])
}
