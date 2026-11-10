//! PK1-S1 (docs/sessions/PK1_CORRUPTION.md section 3, FB-167): the content checks, schema and writer of the PK1 hooks,
//! on a scratch data folder and contexts built here (no shipped data). Each new key checks clean in its valid form and
//! each broken rule is one error: the trigger effects `ap` `set` (with `rounds`), `damage` `pct_of_hit` (a hit event
//! or a rider only), `strike_nearest` and `mark`; the band-only effect `heal_taken` (not a passive effect: the passive
//! effect list, a checks_golden message, is unchanged); a hazard `meter` on an arena effect and a ground hazard row (a
//! known unit meter, a whole non-zero delta); a consumable's `unmark`. The writer stores an empty hazard `meter` and
//! an empty `unmark` as no key, and the schema names every key in its help.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::hooks::{arena_hazards_errors, effects_errors, Ctx};
use rpgkit_content::check::{self, hook_kinds};
use rpgkit_content::{source, store};
use rpgkit_data::{HAZARDS_KIND, HEAL_TAKEN, MARK, STRIKE_NEAREST, TRIGGER_EFFECTS, UNIT_METERS_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn ctx() -> Ctx {
    Ctx { meters: vec![("tension".into(), 0, 100)], ..Ctx::default() }
}

/// The errors of one effect in a trigger row answering `event` (None: a rider).
fn effect(e: Value, event: Option<&str>) -> Vec<String> {
    effects_errors(&json!([e]), &ctx(), "Trigger 1", event, false)
}

#[test]
fn the_new_trigger_effects_and_keys_are_listed() {
    let keys = |t: &str| TRIGGER_EFFECTS.iter().find(|(k, _)| *k == t).map(|(_, keys)| keys.to_vec()).unwrap_or_default();
    assert_eq!(keys("ap"), vec!["value", "set", "rounds", "to"]);
    assert_eq!(keys("damage"), vec!["value", "pct_of_hit", "to"]);
    assert_eq!(keys(STRIKE_NEAREST), vec!["power", "then_status", "then_rounds"]);
    assert_eq!(keys(MARK), vec!["mark", "value", "to"]);
    assert!(rpgkit_data::BAND_EFFECTS.contains(&HEAL_TAKEN));
    assert!(!check::hooks::PASSIVE_EFFECTS.contains(&HEAL_TAKEN), "band-only");
}

#[test]
fn the_valid_forms_check_clean() {
    let ok: Vec<(Value, Option<&str>)> = vec![
        (json!({"type": "ap", "set": 0, "rounds": 1}), Some("round_end")),
        (json!({"type": "ap", "set": 2}), Some("round_end")),
        (json!({"type": "ap", "value": 1}), Some("round_end")),
        (json!({"type": "damage", "pct_of_hit": 10}), Some("hit_dealt")),
        (json!({"type": "damage", "pct_of_hit": 100, "to": "self"}), Some("hit_taken")),
        (json!({"type": "damage", "pct_of_hit": 10, "to": "self"}), None),
        (json!({"type": "damage", "value": 5}), Some("round_end")),
        (json!({"type": STRIKE_NEAREST}), Some("round_end")),
        (json!({"type": STRIKE_NEAREST, "power": 1.0, "then_status": "stun", "then_rounds": 1}), Some("round_end")),
        (json!({"type": MARK, "mark": "calcify", "value": 1}), Some("round_end")),
        (json!({"type": MARK, "mark": "calcify", "to": "other"}), Some("kill")),
    ];
    for (e, event) in ok {
        assert_eq!(effect(e.clone(), event), Vec::<String>::new(), "{e} on {event:?}");
    }
    // the crisis shape: a round_end table of strike_nearest, ap set and mark rows, in a meter row's trigger
    let table = json!({"type": "roll_table", "rows": [
        {"weight": 1, "effects": [{"type": STRIKE_NEAREST, "power": 1.0, "then_status": "stun", "then_rounds": 1}]},
        {"weight": 1, "effects": [{"type": "ap", "set": 0, "rounds": 1}, {"type": "inflict", "status": "bleed", "potency": 100, "rounds": 3}]},
        {"weight": 1, "effects": [{"type": MARK, "mark": "calcify", "value": 1}]}]});
    assert_eq!(effect(table, Some("round_end")), Vec::<String>::new());
}

#[test]
fn each_broken_trigger_rule_is_one_error() {
    let cases: Vec<(Value, Option<&str>, &str)> = vec![
        (json!({"type": "ap", "set": 0, "value": 1}), Some("round_end"), "ap takes value or set, not both."),
        (json!({"type": "ap", "set": 5}), Some("round_end"), "set must be a whole number 0-4."),
        (json!({"type": "ap", "set": 0, "rounds": 6}), Some("round_end"), "rounds must be a whole number 1-5."),
        (json!({"type": "ap", "value": 1, "rounds": 2}), Some("round_end"), "rounds goes with set."),
        (json!({"type": "damage", "pct_of_hit": 10, "value": 5}), Some("hit_dealt"), "damage takes value or pct_of_hit, not both."),
        (json!({"type": "damage", "pct_of_hit": 0}), Some("hit_dealt"), "pct_of_hit must be a whole number 1-100."),
        (json!({"type": "damage", "pct_of_hit": 101}), Some("hit_dealt"), "pct_of_hit must be a whole number 1-100."),
        (json!({"type": "damage", "pct_of_hit": 10}), Some("round_end"), "pct_of_hit needs a hit event"),
        (json!({"type": "damage", "pct_of_hit": 10}), Some("ally_hit"), "pct_of_hit needs a hit event"),
        (json!({"type": "damage", "pct_of_hit": 10, "to": "other"}), Some("hit_dealt"), "pct_of_hit is recoil on the holder (to: self)."),
        (json!({"type": "damage", "pct_of_hit": 10}), None, "pct_of_hit is recoil on the holder: a rider sets to: self."),
        (json!({"type": STRIKE_NEAREST, "power": 4.0}), Some("round_end"), "power must be a number 0.1-3.0."),
        (json!({"type": STRIKE_NEAREST, "then_status": "nap"}), Some("round_end"), "then_status 'nap' is not a status."),
        (json!({"type": STRIKE_NEAREST, "then_rounds": 2}), Some("round_end"), "then_rounds needs a then_status."),
        (json!({"type": STRIKE_NEAREST, "then_status": "stun", "then_rounds": 9}), Some("round_end"), "then_rounds must be a whole number 1-5."),
        (json!({"type": STRIKE_NEAREST, "to": "other"}), Some("round_end"), "unknown key 'to'"),
        (json!({"type": MARK}), Some("round_end"), "mark needs an id of 1-24 characters"),
        (json!({"type": MARK, "mark": "Calcify"}), Some("round_end"), "mark needs an id of 1-24 characters"),
        (json!({"type": MARK, "mark": "calcify", "value": 0}), Some("round_end"), "value must be a whole number 1-99."),
    ];
    for (e, event, want) in cases {
        let got = effect(e.clone(), event);
        assert_eq!(got.len(), 1, "{e}: {got:?}");
        assert!(got[0].contains(want), "{e}: {got:?} wants {want}");
    }
}

#[test]
fn an_arena_hazard_meter_names_a_known_meter_and_a_delta() {
    let meters = vec!["tension".to_string()];
    let row = |effect: Value| {
        let e = json!({"max_rounds": 6, "total_blocks": 6, "arena_hazards": [{"id": "vent", "name": "Vent", "cells": [[1, 1]], "blocks": [1], "effect": effect}]});
        arena_hazards_errors(e.as_object().unwrap(), &meters)
    };
    assert_eq!(row(json!({"meter": {"meter": "tension", "delta": 5}})), Vec::<String>::new(), "a meter alone is an effect");
    assert_eq!(row(json!({"damage": 3, "meter": {"meter": "tension", "delta": -100}})), Vec::<String>::new());
    let cases: Vec<(Value, &str)> = vec![
        (json!({"meter": {"meter": "rage", "delta": 5}}), "meter: unknown unit meter 'rage'."),
        (json!({"meter": {"meter": "tension", "delta": 0}}), "meter: delta must be a whole number -100..100, not 0."),
        (json!({"meter": {"meter": "tension", "delta": 2.5}}), "meter: delta must be a whole number -100..100, not 0."),
        (json!({"meter": {"delta": 5}}), "meter: meter is required"),
        (json!({"meter": {"meter": "tension", "delta": 5, "to": "x"}}), "meter: unknown key 'to'"),
        (json!({"damage": 3, "meter": 5}), "meter must be an object"),
    ];
    for (effect, want) in cases {
        let got = row(effect.clone());
        assert_eq!(got.len(), 1, "{effect}: {got:?}");
        assert!(got[0].contains(want), "{effect}: {got:?} wants {want}");
    }
}

// ---- rows of the catalog (scratch data) ------------------------------------------------------------------------------

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_pk1_hooks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// Skills, the tension meter (a heal_taken band), a hazard with a meter, a consumable with an unmark.
fn catalog(data: &Path) {
    let one = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(data, "skills", 2, json!([one("move", "MOVE", json!({"system": true})), one("rest", "REST", json!({"system": true})), one("strike", "ATTACK", json!({"basic": true}))]));
    write_kind(data, UNIT_METERS_KIND, 1, json!([{"id": "tension", "name": "Tension", "short": "Te", "min": 0, "max": 100, "show": "bar",
        "bands": [{"id": "calm", "from": 0}, {"id": "tense", "from": 25, "effects": [{"type": HEAL_TAKEN, "value": -25}]}]}]));
    write_kind(data, HAZARDS_KIND, 1, json!([{"id": "miasma", "name": "Miasma", "meter": {"meter": "tension", "delta": 5}}, {"id": "smoke", "name": "Smoke"}]));
    write_kind(data, "item_families", 1, json!([{"id": "pouch", "name": "Pouch", "item_type": "CONSUMABLE"}]));
    write_kind(data, "items", 2, json!([{"id": "salt", "name": "Salt", "family": "pouch", "consumable_type": "HEALTH_POTION", "unmark": {"mark": "calcify"}}]));
}

fn errors_of(v: &check::View, kind: &str, id: &str, change: &Value) -> Vec<String> {
    let mut e = v.get(kind, id).clone();
    assert!(!e.is_empty(), "fixture {kind} {id}");
    for (k, x) in change.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, kind, &e, id)
}

#[test]
fn the_rows_load_and_check_clean() {
    let (_scratch, data) = scratch("clean");
    catalog(&data);
    let src = source::load(&data);
    let catalog_errors: Vec<&String> = src.errors.iter().filter(|e| e.contains("catalog")).collect();
    assert!(catalog_errors.is_empty(), "{catalog_errors:?}");
    let v = store::view(&data);
    assert_eq!(v.typed.items["salt"].unmark, "calcify");
    let report = check::validate_all(&v);
    for key in ["unit_meters:tension", "hazards:miasma", "hazards:smoke", "items:salt"] {
        assert!(!report.contains_key(key), "{key}: {:?}", report.get(key));
    }
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn each_broken_row_rule_is_one_error() {
    let (_scratch, data) = scratch("rules");
    catalog(&data);
    let v = store::view(&data);
    let band = |fx: Value| json!({"bands": [{"id": "calm", "from": 0}, {"id": "tense", "from": 25, "effects": [fx]}]});
    let cases: Vec<(&str, &str, Value, &str)> = vec![
        (UNIT_METERS_KIND, "tension", band(json!({"type": HEAL_TAKEN, "value": 50})), ""),
        (UNIT_METERS_KIND, "tension", band(json!({"type": HEAL_TAKEN, "value": 0})), "heal_taken needs a value -100..100, not 0."),
        (UNIT_METERS_KIND, "tension", band(json!({"type": HEAL_TAKEN, "value": -101})), "heal_taken needs a value -100..100, not 0."),
        (UNIT_METERS_KIND, "tension", band(json!({"type": HEAL_TAKEN, "value": -25, "to": "self"})), "unknown key 'to' (keys: type, value)."),
        (HAZARDS_KIND, "miasma", json!({"meter": {}}), ""),
        (HAZARDS_KIND, "miasma", json!({"meter": {"meter": "rage", "delta": 5}}), "Hazard meter: unknown unit meter 'rage'."),
        (HAZARDS_KIND, "miasma", json!({"meter": {"meter": "tension", "delta": 0}}), "Hazard meter: delta must be a whole number -100..100, not 0."),
        ("items", "salt", json!({"unmark": {}}), "A consumable should heal, damage or buff."),
        ("items", "salt", json!({"unmark": {"mark": ""}}), "Unmark: mark needs an id"),
        ("items", "salt", json!({"unmark": {"mark": "calcify", "all": true}}), "Unmark: unknown key 'all'"),
        ("items", "salt", json!({"damage": 5}), "Unmark: an unmark item is used on allies (no damage)."),
    ];
    let mut failures = Vec::new();
    for (kind, id, change, want) in &cases {
        let errors = errors_of(&v, kind, id, change);
        let ok = if want.is_empty() { errors.is_empty() } else { errors.len() == 1 && errors[0].contains(want) };
        if !ok {
            failures.push(format!("{kind} {id} {change}: want {want:?}, got {errors:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

/// The loader reads the hazards before the meters: a hazard's meter shape is checked there, its id once the meters
/// are known (the row check).
#[test]
fn the_loader_checks_a_hazard_meter_shape_before_the_meters_load() {
    let row: Dict = json!({"id": "miasma", "name": "Miasma", "origin": "system", "meter": {"meter": "rage", "delta": 5}}).as_object().unwrap().clone();
    fn found(row: &Dict, meters: Option<&[String]>) -> Vec<String> {
        let refs = hook_kinds::Refs { skills: &[], derived: &[], families: &[], meters };
        hook_kinds::check_rows(HAZARDS_KIND, std::slice::from_ref(row), &refs).remove(0)
    }
    let known = vec!["tension".to_string()];
    let unknown_id = |found: &[String]| found.iter().any(|e| e.contains("unknown unit meter 'rage'"));
    assert!(!unknown_id(&found(&row, None)));
    assert!(unknown_id(&found(&row, Some(&known))));
}

#[test]
fn the_writer_stores_an_empty_meter_and_unmark_as_no_key() {
    let hazard = |meter: Value| json!({"id": "smoke", "name": "Smoke", "meter": meter}).as_object().unwrap().clone();
    assert!(!store::catalog_storage(HAZARDS_KIND, &hazard(json!({})), &[]).unwrap().contains_key("meter"));
    let set = json!({"meter": "tension", "delta": 5});
    assert_eq!(store::catalog_storage(HAZARDS_KIND, &hazard(set.clone()), &[]).unwrap().get("meter"), Some(&set));
    let plain = json!({"id": "smoke", "name": "Smoke"}).as_object().unwrap().clone();
    assert!(!store::catalog_storage(HAZARDS_KIND, &plain, &[]).unwrap().contains_key("meter"), "a row written before the field re-saves as it was");
}

#[test]
fn the_schema_help_names_every_new_key() {
    let help = |section: &str, kind: &str, key: &str| {
        let f = fields(source::schema(), section, kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("{kind}.{key}"));
        json::s(&f, "help")
    };
    let triggers = help("content", "skills", "triggers");
    for k in ["\"set\"", "pct_of_hit", STRIKE_NEAREST, "then_status", "\"mark\""] {
        assert!(triggers.contains(k), "skills.triggers help names {k}");
    }
    assert!(help("content", UNIT_METERS_KIND, "bands").contains(HEAL_TAKEN));
    assert!(help("content", HAZARDS_KIND, "meter").contains("delta"));
    assert!(help("content", "items", "unmark").contains("mark"));
    for section in ["content", "world"] {
        assert!(help(section, "encounters", "arena_hazards").contains("\"meter\""), "{section} arena_hazards help names meter");
    }
}
