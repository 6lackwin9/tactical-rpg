//! BASERULES1-M (docs/specs/BASERULES1.md 4.1, BASERULES1_LIBRARY.md 7; engine-only, no Godot original): the tweak
//! report a game's CLI prints as `aegis rules`, generated from the ruleset rows' schema fields (the kit's
//! `schema.json`), the rule register's metadata (`rpgkit_rules::ruleset`: `REGISTER`, `FIELD_RULES`, `BUNDLES`,
//! `TESTED_SETS`) and the `LOCKED` table, plus the game's own rows and their notes; `aegis rules --contract` prints only
//! the stable surface (every ruleset field as tunable with its default, range and needs; every rule with its
//! requirements; every locked constant), one line each, which a golden holds.

use rpgkit_rules::ruleset::{self, BundleKind, RuleKind, Ruleset, BUNDLES, FIELD_RULES, LOCKED, PROMOTED, REGISTER, TESTED_SETS};
use serde_json::Value;

/// Ruleset row fields that name the row rather than tune it.
const IDENTITY: [&str; 3] = ["id", "name", "origin"];

fn s<'a>(f: &'a Value, k: &str) -> &'a str {
    f.get(k).and_then(Value::as_str).unwrap_or("")
}

fn list(items: &[&str]) -> String {
    if items.is_empty() {
        "-".into()
    } else {
        items.join(", ")
    }
}

fn kind_id(k: RuleKind) -> &'static str {
    match k {
        RuleKind::Option => "option",
        RuleKind::Extension => "extension",
    }
}

/// One schema field's range or values ("" when it has neither).
fn range(f: &Value) -> String {
    if let Some(o) = f.get("options").and_then(Value::as_array) {
        let ids: Vec<String> = o.iter().map(|v| v.as_str().map(|x| if x.is_empty() { "\"\"".to_string() } else { x.to_string() }).unwrap_or_default()).collect();
        return format!("values={}", ids.join("|"));
    }
    match (f.get("min"), f.get("max")) {
        (Some(lo), Some(hi)) => format!("range={lo}..{hi}"),
        _ => String::new(),
    }
}

/// The tunable lines: each schema field of the rows (bar identity), then each metadata field the schema lacks.
fn tunables(fields: &[Value]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for f in fields {
        let key = s(f, "key");
        if key.is_empty() || IDENTITY.contains(&key) {
            continue;
        }
        seen.push(key);
        let default = f.get("default").map(Value::to_string).unwrap_or_default();
        let mut line = format!("tunable {key} {} default={default}", s(f, "type"));
        let r = range(f);
        if !r.is_empty() {
            line.push_str(&format!(" {r}"));
        }
        if let Some(m) = ruleset::field_rule_of(key) {
            line.push_str(&meta(m));
        }
        out.push(line);
    }
    for m in FIELD_RULES.iter().filter(|m| !seen.contains(&m.key)) {
        let default = ruleset::Numbers::default().field(m.key).unwrap_or_default();
        out.push(format!("tunable {} row-only default={default}{} (no schema entry)", m.key, meta(m)));
    }
    out
}

fn meta(m: &ruleset::FieldRule) -> String {
    let mut out = String::new();
    if !m.needs.is_empty() {
        out.push_str(&format!(" needs={}", m.needs.join("+")));
        if m.refused {
            out.push_str("(refused)");
        }
    }
    if !m.untested_with.is_empty() {
        out.push_str(&format!(" untested_with={}", m.untested_with.join("+")));
    }
    out
}

fn rule_lines() -> Vec<String> {
    REGISTER
        .iter()
        .map(|r| format!("rule {} {} system={} bundle={} requires={} conflicts={}", r.id, kind_id(r.kind), r.system, if r.bundle.is_empty() { "-" } else { r.bundle }, list(r.requires), list(r.conflicts)))
        .collect()
}

fn locked_lines() -> Vec<String> {
    let mut out: Vec<String> = LOCKED.iter().map(|l| format!("locked {}::{} {}", l.file, l.name, l.system)).collect();
    out.extend(PROMOTED.iter().map(|(name, key)| format!("promoted {name} -> {key}")));
    out
}

/// `aegis rules --contract`: the surface other silos read, one line each (`fields`: the ruleset rows' schema fields).
pub fn contract(fields: &[Value]) -> Vec<String> {
    let mut out = tunables(fields);
    out.extend(rule_lines());
    out.extend(locked_lines());
    out
}

/// `aegis rules`: the tweak report, by section: rules by system, row fields, bundles, tested sets, locked, then the
/// game's rows `rows` with their notes ("ok" when none).
pub fn report(fields: &[Value], rows: &[Ruleset]) -> Vec<String> {
    let mut out = vec!["Rules (the register: kind, requires, conflicts, bundle, tested in):".to_string()];
    let mut systems: Vec<&str> = Vec::new();
    for r in REGISTER.iter() {
        if !systems.contains(&r.system) {
            systems.push(r.system);
        }
    }
    for sys in systems {
        out.push(format!("  [{sys}]"));
        for r in REGISTER.iter().filter(|r| r.system == sys) {
            out.push(format!(
                "    {:<15} {:<9} requires {}; conflicts {}; bundle {}; tested {}: {}",
                r.id,
                kind_id(r.kind),
                list(r.requires),
                list(r.conflicts),
                if r.bundle.is_empty() { "-" } else { r.bundle },
                list(r.tested),
                r.does
            ));
        }
    }
    out.push("Fields (tunable per row; needs: read only when it holds):".into());
    out.extend(tunables(fields).into_iter().map(|l| format!("  {}", l.trim_start_matches("tunable "))));
    out.push("Bundles:".into());
    for b in BUNDLES.iter() {
        let kind = if b.kind == BundleKind::MustShip { "must ship" } else { "works well" };
        let when = if b.anchor.is_empty() { String::new() } else { format!(" when {}", b.anchor.join("+")) };
        let note = if b.note.is_empty() { String::new() } else { format!(" ({})", b.note) };
        out.push(format!("  {:<12} {kind}{when}: {}{note}", b.id, b.holds.join(", ")));
    }
    out.push("Tested sets (a row listing another set is untested together):".into());
    for t in TESTED_SETS.iter() {
        out.push(format!("  [{}]", if t.len() == REGISTER.len() { "all".to_string() } else { t.join(", ") }));
    }
    out.push("Locked (stays code; LOCKED in rpgkit_rules::ruleset):".into());
    out.extend(LOCKED.iter().map(|l| format!("  {:<11} {:<23} {:<11} {}", l.file, l.name, l.system, l.why)));
    out.extend(PROMOTED.iter().map(|(name, key)| format!("  promoted    {name:<23} the default of {key}")));
    out.push("Rows:".into());
    if rows.is_empty() {
        out.push("  none (battles play the built-in classic values)".into());
    }
    for r in rows {
        let n = r.notes();
        let rules = if r.rules.is_empty() { "no rules".to_string() } else { r.rules.join(", ") };
        out.push(format!("  {}: {rules}", r.id));
        if n.errors.is_empty() && n.warnings.is_empty() {
            out.push("    ok".into());
        }
        out.extend(n.errors.iter().map(|e| format!("    error: {e}")));
        out.extend(n.warnings.iter().map(|w| format!("    warning: {w}")));
    }
    out
}
