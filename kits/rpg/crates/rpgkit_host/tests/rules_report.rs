//! BASERULES1-M (docs/specs/BASERULES1.md 4.1-4.2): `aegis rules` and `aegis rules --contract` on a fixture. The report
//! has its sections and a row's notes (an error for a rule without what it requires, "ok" for a clean row); the contract
//! has one tunable line per ruleset field of the kit schema, one line per rule and per locked constant, and equals its
//! golden line for line (`tests/fixtures/rules_contract_golden.json`). A contract change (a field, a default, a range,
//! a requirement, a locked constant) fails it; re-record only when intended, named in the commit and a docs/HISTORY.md
//! row, and tell the other silos: `RECORD=1 tools/fc.sh test --test rules_report`.

use rpgkit_host::rules;
use rpgkit_rules::ruleset::{Ruleset, LOCKED, REGISTER};
use serde_json::{json, Value};
use std::path::PathBuf;

fn kit_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The ruleset rows' schema fields of the kit.
fn fields() -> Vec<Value> {
    let text = std::fs::read_to_string(kit_dir().join("schema.json")).expect("kits/rpg/schema.json");
    let schema: Value = serde_json::from_str(&text).unwrap();
    schema["content"]["rulesets"].as_array().expect("the rulesets fields").clone()
}

fn row(v: Value) -> Ruleset {
    Ruleset::from_row(v.as_object().unwrap())
}

#[test]
fn the_report_lists_the_sections_and_each_rows_notes() {
    let rows = [Ruleset::classic(), row(json!({"id": "loose", "rules": ["flank"]}))];
    let out = rules::report(&fields(), &rows);
    for section in ["Rules (", "Fields (", "Bundles:", "Tested sets (", "Locked (", "Rows:"] {
        assert!(out.iter().any(|l| l.starts_with(section)), "section {section}");
    }
    for r in REGISTER.iter() {
        assert!(out.iter().any(|l| l.trim_start().starts_with(r.id) && l.contains(r.does)), "rule {}", r.id);
    }
    let at = |id: &str| out.iter().position(|l| l == &format!("  {id}: {}", if id == "classic" { "no rules" } else { "flank" })).unwrap_or_else(|| panic!("row {id}: {out:#?}"));
    assert_eq!(out[at("classic") + 1], "    ok");
    assert_eq!(out[at("loose") + 1], "    error: flank needs round_intent in Rules: without it the rule does nothing.");
    assert!(rules::report(&fields(), &[]).iter().any(|l| l.contains("built-in classic values")), "no rows");
}

#[test]
fn the_contract_lists_every_field_rule_and_locked_constant_and_holds_its_golden() {
    let out = rules::contract(&fields());
    for f in fields() {
        let key = f["key"].as_str().unwrap();
        if ["id", "name", "origin"].contains(&key) {
            continue;
        }
        assert_eq!(out.iter().filter(|l| l.starts_with(&format!("tunable {key} "))).count(), 1, "field {key}");
    }
    assert!(out.iter().any(|l| l.starts_with("tunable flank_power_mult float default=1.0 range=1.0..2.0 needs=flank")), "{out:#?}");
    for r in REGISTER.iter() {
        assert_eq!(out.iter().filter(|l| l.starts_with(&format!("rule {} ", r.id))).count(), 1, "rule {}", r.id);
    }
    assert!(out.contains(&"rule flank extension system=positioning bundle=positioning requires=round_intent conflicts=-".to_string()));
    for l in LOCKED.iter() {
        assert!(out.contains(&format!("locked {}::{} {}", l.file, l.name, l.system)), "locked {}", l.name);
    }
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rules_contract_golden.json");
    if std::env::var("RECORD").is_ok_and(|v| v == "1") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&json!({"contract": out})).unwrap() + "\n").unwrap();
        return;
    }
    let golden: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("the contract golden (RECORD=1 to write it)")).unwrap();
    let want: Vec<String> = golden["contract"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    for (i, (a, b)) in out.iter().zip(&want).enumerate() {
        assert_eq!(a, b, "contract line {i} changed (a contract change: re-record only when intended)");
    }
    assert_eq!(out.len(), want.len(), "the contract's line count changed");
}
