//! The data file format: what Godot's JSON.parse reads (every number a float) and JSON.stringify(doc, "\t", false)
//! writes, so a file the writer rewrites without changes stays byte-identical. Files hold
//! {"kind", "version", "entries": [...]} (or a bare list).

use aegis_core::json::{self, to_int_or, Dict};
use aegis_core::vfs;
use serde_json::Value;
use std::path::Path;

/// JSON.parse: every number is a float.
fn floats(v: Value) -> Value {
    match v {
        Value::Number(n) => Value::from(n.as_f64().unwrap_or(0.0)),
        Value::Array(a) => Value::Array(a.into_iter().map(floats).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, v)| (k, floats(v))).collect()),
        other => other,
    }
}

/// A data file as JSON.parse reads it; None when missing, or unreadable (reported in `errors`).
pub fn read(path: &Path, errors: &mut Vec<String>) -> Option<Value> {
    let text = vfs::read_to_string(path).ok()?;
    match serde_json::from_str::<Value>(&text) {
        Ok(v) => Some(floats(v)),
        Err(e) => {
            errors.push(format!("{}: JSON error on line {}: {e}", path.display(), e.line()));
            None
        }
    }
}

/// A file's entry list: {"version", "entries": [...]} (or a bare list) -> (entries, version).
pub fn entries_of(doc: Value) -> (Vec<Value>, i64) {
    match doc {
        Value::Object(mut o) => {
            let version = to_int_or(json::get(&o, "version"), 1);
            match o.shift_remove("entries") {
                Some(Value::Array(a)) => (a, version),
                _ => (Vec::new(), version),
            }
        }
        Value::Array(a) => (a, 1),
        _ => (Vec::new(), 1),
    }
}

/// The raw entries of a data file (numbers as JSON.parse reads them); empty when missing or unreadable.
pub fn raw_entries(path: &Path) -> Vec<Dict> {
    let mut errors = Vec::new();
    read(path, &mut errors).map(|d| entries_of(d).0).unwrap_or_default().iter().filter_map(Value::as_object).cloned().collect()
}

/// A whole data file: {"kind", "version", "entries"} in the file format.
pub fn document(kind: &str, version: i64, entries: Vec<Value>) -> String {
    let mut doc = Dict::new();
    doc.insert("kind".into(), Value::from(kind));
    doc.insert("version".into(), json::int(version));
    doc.insert("entries".into(), Value::Array(entries));
    stringify(&Value::Object(doc))
}

// ---- JSON.stringify(doc, "\t", false) ------------------------------------------------------------------------

/// Godot's JSON.stringify with a tab indent (keys in insertion order) plus the trailing newline the save adds.
pub fn stringify(v: &Value) -> String {
    let mut out = String::new();
    write(&mut out, v, 0);
    out.push('\n');
    out
}

fn write(out: &mut String, v: &Value, depth: usize) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => match n.as_i64() {
            Some(i) => out.push_str(&i.to_string()),
            None => out.push_str(&float_text(n.as_f64().unwrap_or(0.0))),
        },
        Value::String(s) => quote(out, s),
        Value::Array(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                indent(out, depth + 1);
                write(out, x, depth + 1);
            }
            out.push('\n');
            indent(out, depth);
            out.push(']');
        }
        Value::Object(o) => {
            if o.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, x)) in o.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                indent(out, depth + 1);
                quote(out, k);
                out.push_str(": ");
                write(out, x, depth + 1);
            }
            out.push('\n');
            indent(out, depth);
            out.push('}');
        }
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push('\t');
    }
}

/// String::json_escape.
fn quote(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// JSON::stringify of a float: 14 significant digits (`String::num`, trailing zeros trimmed to one decimal).
fn float_text(x: f64) -> String {
    if x == 0.0 {
        return "0.0".into();
    }
    let precision = (14 - x.abs().log10().floor() as i64).max(1) as usize;
    let mut t = format!("{x:.precision$}");
    if t.contains('.') {
        while t.ends_with('0') && !t.ends_with(".0") {
            t.pop();
        }
    }
    t
}
