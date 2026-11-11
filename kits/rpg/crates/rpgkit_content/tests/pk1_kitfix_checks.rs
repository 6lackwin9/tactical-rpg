//! PK1-KITFIX (a), (b) (FB-184 Option A; `docs/sessions/PK1_CORRUPTION.md` 2.1, the crisis): the content checks and
//! schema of the three trigger-effect fixes, on contexts and a scratch data folder built here (no shipped data). A
//! unit meter row's `triggers` take `meter_set` (as its `rules` do; other trigger rows still refuse it); `inflict`
//! takes `rounds: -1` for the rest of the battle (as `grant` does); the new trigger effect `rating_set` `{rating, value,
//! rounds}` names one of `RATING_SET_RANGES` with a value in its range and 1-5 rounds or -1. Each valid form checks
//! clean, each broken rule is one error, and the schema help names every new key.

use aegis_content::schema::fields;
use aegis_core::json;
use rpgkit_content::check::hooks::{effects_errors, Ctx};
use rpgkit_content::check::{self};
use rpgkit_content::{source, store};
use rpgkit_data::{RATING_SET, RATING_SET_RANGES, TRIGGER_EFFECTS, UNIT_METERS_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn ctx() -> Ctx {
    Ctx { meters: vec![("strain".into(), 0, 100)], ..Ctx::default() }
}

/// The errors of one effect in a trigger row answering `event`; `meter_set` as a meter row's list.
fn effect(e: Value, event: &str, meter_set: bool) -> Vec<String> {
    effects_errors(&json!([e]), &ctx(), "Trigger 1", Some(event), meter_set)
}

#[test]
fn rating_set_is_a_trigger_effect_with_its_keys() {
    let keys = TRIGGER_EFFECTS.iter().find(|(k, _)| *k == RATING_SET).map(|(_, keys)| keys.to_vec());
    assert_eq!(keys, Some(vec!["rating", "value", "rounds", "to"]));
    // the ratings are the status table's rating words
    for (r, _) in RATING_SET_RANGES {
        assert!(rpgkit_rules::status::Rating::from_id(r).is_some(), "{r} is a status rating");
    }
}

#[test]
fn inflict_takes_minus_one_for_the_battle() {
    for rounds in [json!(-1), json!(1), json!(5)] {
        assert_eq!(effect(json!({"type": "inflict", "status": "bleed", "potency": 100, "rounds": rounds}), "round_end", false), Vec::<String>::new(), "rounds {rounds}");
    }
    for rounds in [json!(0), json!(-2), json!(6), json!(1.5)] {
        let got = effect(json!({"type": "inflict", "status": "bleed", "potency": 100, "rounds": rounds}), "round_end", false);
        assert_eq!(got, vec!["Trigger 1 effect 1: rounds must be -1 (the battle) or 1-5.".to_string()], "rounds {rounds}");
    }
}

#[test]
fn rating_set_valid_forms_check_clean() {
    let mut ok = vec![json!({"type": RATING_SET, "rating": "armor", "value": 0, "rounds": -1}), json!({"type": RATING_SET, "rating": "armor", "value": 0, "to": "other"})];
    for (r, (lo, hi)) in RATING_SET_RANGES {
        for v in [lo, hi] {
            for rounds in [1, 5, -1] {
                ok.push(json!({"type": RATING_SET, "rating": r, "value": v, "rounds": rounds}));
            }
        }
    }
    for e in ok {
        assert_eq!(effect(e.clone(), "round_end", false), Vec::<String>::new(), "{e}");
    }
}

#[test]
fn each_broken_rating_set_rule_is_one_error() {
    let names = RATING_SET_RANGES.iter().map(|(r, _)| *r).collect::<Vec<_>>().join(", ");
    let (lo, hi) = RATING_SET_RANGES.iter().find(|(r, _)| *r == "armor").map(|(_, range)| *range).unwrap();
    let cases: Vec<(Value, String)> = vec![
        (json!({"type": RATING_SET, "value": 0}), format!("rating_set needs a rating ({names}).")),
        (json!({"type": RATING_SET, "rating": "guard", "value": 0}), format!("rating_set needs a rating ({names}).")),
        (json!({"type": RATING_SET, "rating": "luck", "value": 0}), format!("rating_set needs a rating ({names}).")),
        (json!({"type": RATING_SET, "rating": "armor"}), format!("value is required ({lo}-{hi}).")),
        (json!({"type": RATING_SET, "rating": "armor", "value": hi + 1}), format!("value must be a whole number {lo}-{hi}.")),
        (json!({"type": RATING_SET, "rating": "armor", "value": lo - 1}), format!("value must be a whole number {lo}-{hi}.")),
        (json!({"type": RATING_SET, "rating": "armor", "value": 0, "rounds": 0}), "rounds must be -1 (the battle) or 1-5.".into()),
        (json!({"type": RATING_SET, "rating": "armor", "value": 0, "rounds": 6}), "rounds must be -1 (the battle) or 1-5.".into()),
        (json!({"type": RATING_SET, "rating": "armor", "value": 0, "magnitude": 3}), "unknown key 'magnitude'".into()),
        (json!({"type": RATING_SET, "rating": "armor", "value": 0, "to": "both"}), "to".into()),
    ];
    for (e, want) in cases {
        let got = effect(e.clone(), "round_end", false);
        assert_eq!(got.len(), 1, "{e}: {got:?}");
        assert!(got[0].contains(&want), "{e}: {got:?} wants {want}");
    }
}

#[test]
fn meter_set_needs_a_meter_list() {
    let set = json!({"type": "meter_set", "meter": "strain", "value": 50});
    assert_eq!(effect(set.clone(), "round_end", true), Vec::<String>::new(), "a meter row's list takes it");
    let got = effect(set, "round_end", false);
    assert!(got.len() == 1 && got[0].contains("unknown effect type 'meter_set'"), "other trigger rows refuse it: {got:?}");
    let got = effect(json!({"type": "meter_set", "meter": "strain", "value": 101}), "round_end", true);
    assert_eq!(got, vec!["Trigger 1 effect 1: value must be a whole number 0-100.".to_string()]);
}

// ---- a meter row (scratch data) ---------------------------------------------------------------------------------------

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_pk1_kitfix_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// The crisis shape (PK1 2.1): a round_end trigger at Strain 100 rolling a table whose row holds the armour at 0 and
/// a bleed for the battle, then setting Strain to 50.
fn crisis(set_value: i64) -> Value {
    json!({"on": "round_end", "when": {"meter": {"id": "strain", "at_least": 100}}, "limit": "none", "effects": [
        {"type": "roll_table", "rows": [
            {"weight": 1, "effects": [{"type": RATING_SET, "rating": "armor", "value": 0, "rounds": -1}, {"type": "inflict", "status": "bleed", "potency": 100, "rounds": 3}]},
            {"weight": 1, "effects": [{"type": "inflict", "status": "bleed", "potency": 100, "rounds": -1}]}]},
        {"type": "meter_set", "meter": "strain", "value": set_value}]})
}

fn meter_row(set_value: i64) -> Value {
    json!({"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": 100, "show": "bar", "bands": [{"id": "calm", "from": 0}], "triggers": [crisis(set_value)]})
}

#[test]
fn a_meter_rows_trigger_takes_meter_set_and_the_crisis_shape() {
    let (_scratch, data) = scratch("crisis");
    write_kind(&data, UNIT_METERS_KIND, 1, json!([meter_row(50)]));
    let src = source::load(&data);
    let meter_errors: Vec<&String> = src.errors.iter().filter(|e| e.contains(UNIT_METERS_KIND)).collect();
    assert!(meter_errors.is_empty(), "{meter_errors:?}");
    let v = store::view(&data);
    assert!(v.typed.unit_meters.contains_key("strain"), "the row loads");
    assert!(!check::validate_all(&v).contains_key("unit_meters:strain"));
    // the set is checked like a rule's: its value within the meter's range
    let mut bad = v.get(UNIT_METERS_KIND, "strain").clone();
    bad.insert("triggers".into(), json!([crisis(150)]));
    let errors = check::validate_entry(&v, UNIT_METERS_KIND, &bad, "strain");
    assert_eq!(errors, vec!["Trigger 1 effect 2: value must be a whole number 0-100.".to_string()]);
}

#[test]
fn the_schema_help_names_every_new_key() {
    let help = |kind: &str, key: &str| {
        let f = fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("{kind}.{key}"));
        json::s(&f, "help")
    };
    let triggers = help("skills", "triggers");
    for k in [RATING_SET, "\"rating\"", "\"rounds\": -1"] {
        assert!(triggers.contains(k), "skills.triggers help names {k}");
    }
    for (r, _) in RATING_SET_RANGES {
        assert!(triggers.contains(r), "skills.triggers help names {r}");
    }
    assert!(help(UNIT_METERS_KIND, "triggers").contains("meter_set"));
}
