//! ROLLTABLE1 (docs/specs/ACTS23.md PK1 hook a): the content check of the `roll_table` trigger effect. A table with
//! weighted rows of trigger effects checks clean wherever trigger effects go (a trigger row, a meter rule with
//! `meter_set`); each broken rule is one error naming the row: no rows, a weight below 1 (or not whole), an unknown or
//! invalid nested effect, a nested table, a nested redirect, an unknown row key. The schema names the effect in its help.
//! No catalog: the checks run on a context built here.

use aegis_content::schema::fields;
use aegis_core::json;
use rpgkit_content::check::hooks::{effects_errors, Ctx};
use rpgkit_content::source;
use rpgkit_data::{ROLL_TABLE, ROLL_WEIGHT, TRIGGER_EFFECTS};
use serde_json::{json, Value};

fn ctx() -> Ctx {
    Ctx { meters: vec![("strain".into(), 0, 100)], ..Ctx::default() }
}

/// The errors of a trigger row's `effects` holding `table` (a `hit_taken` row; `meter_set` as a meter rule's list).
fn errors(table: Value, meter_set: bool) -> Vec<String> {
    effects_errors(&json!([table]), &ctx(), "Trigger 1", Some("hit_taken"), meter_set)
}

fn table(rows: Value) -> Value {
    json!({"type": ROLL_TABLE, "rows": rows})
}

#[test]
fn roll_table_is_a_trigger_effect_with_rows() {
    let keys = TRIGGER_EFFECTS.iter().find(|(k, _)| *k == ROLL_TABLE).map(|(_, keys)| keys.to_vec());
    assert_eq!(keys, Some(vec!["rows"]));
    assert!(ROLL_WEIGHT.0 == 1 && ROLL_WEIGHT.1 > 1, "weights start at 1");
}

#[test]
fn a_weighted_table_checks_clean() {
    let ok = table(json!([
        {"weight": 1, "effects": [{"type": "crit", "value": 10}]},
        {"weight": 3, "effects": [{"type": "damage", "value": 5, "to": "other"}, {"type": "inflict", "status": "bleed", "potency": 100, "rounds": 3}]},
        {"weight": ROLL_WEIGHT.1, "effects": [{"type": "meter", "meter": "strain", "delta": 5}]},
    ]));
    assert_eq!(errors(ok, false), Vec::<String>::new());
    // a meter rule's table may set the meter, as its own list may
    let rule = table(json!([{"weight": 1, "effects": [{"type": "meter_set", "meter": "strain", "value": 75}]}]));
    assert_eq!(errors(rule.clone(), true), Vec::<String>::new());
    let off = errors(rule, false);
    assert_eq!(off.len(), 1, "{off:?}");
    assert!(off[0].contains("row 1 effect 1: unknown effect type 'meter_set'"), "{off:?}");
}

#[test]
fn each_broken_table_rule_is_one_error() {
    let crit = json!([{"type": "crit", "value": 10}]);
    let cases: Vec<(Value, &str)> = vec![
        (json!({"type": ROLL_TABLE}), "Trigger 1 effect 1: roll_table needs rows"),
        (table(json!([])), "Trigger 1 effect 1: roll_table needs rows"),
        (table(json!({"weight": 1})), "Trigger 1 effect 1: roll_table needs rows"),
        (table(json!([{"weight": 0, "effects": crit}])), "row 1: weight must be a whole number 1-"),
        (table(json!([{"weight": -2, "effects": crit}])), "row 1: weight must be a whole number 1-"),
        (table(json!([{"weight": 1.5, "effects": crit}])), "row 1: weight must be a whole number 1-"),
        (table(json!([{"weight": ROLL_WEIGHT.1 + 1, "effects": crit}])), "row 1: weight must be a whole number 1-"),
        (table(json!([{"effects": crit}])), "row 1: weight is required"),
        (table(json!([{"weight": 1, "effects": crit}, 7])), "row 2 must be an object {weight, effects}."),
        (table(json!([{"weight": 1, "effects": crit, "odds": 2}])), "row 1: unknown key 'odds'"),
        (table(json!([{"weight": 1, "effects": []}])), "row 1: effects holds 1-4 effects."),
        (table(json!([{"weight": 1}])), "row 1: effects holds 1-4 effects."),
        (table(json!([{"weight": 1, "effects": [{"type": "explode"}]}])), "row 1 effect 1: unknown effect type 'explode'"),
        (table(json!([{"weight": 1, "effects": [{"type": "crit", "value": 0}]}])), "row 1 effect 1: value must be a whole number 1-100."),
        (table(json!([{"weight": 1, "effects": [{"type": "grant", "status": "bleed"}]}])), "row 1 effect 1: grant 'bleed' is not a buff"),
        (table(json!([{"weight": 1, "effects": [{"type": "meter", "meter": "rage", "delta": 1}]}])), "row 1 effect 1: unknown unit meter 'rage'."),
        (table(json!([{"weight": 1, "effects": [{"type": "redirect", "pct": 100}]}])), "row 1 effect 1: a roll_table cannot redirect."),
        (table(json!([{"weight": 2, "effects": [table(json!([{"weight": 1, "effects": crit}]))]}])), "row 1 effect 1: a roll_table cannot hold a roll_table."),
        (json!({"type": ROLL_TABLE, "rows": [{"weight": 1, "effects": crit}], "to": "other"}), "Trigger 1 effect 1: unknown key 'to'"),
    ];
    for (t, want) in cases {
        let got = errors(t.clone(), false);
        assert_eq!(got.len(), 1, "{t}: {got:?}");
        assert!(got[0].contains(want), "{t}: {got:?} wants {want}");
    }
}

#[test]
fn the_schema_help_names_the_table() {
    for (kind, key) in [("skills", "triggers"), (rpgkit_data::UNIT_METERS_KIND, "rules")] {
        let f = fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("{kind}.{key}"));
        assert!(json::s(&f, "help").contains(ROLL_TABLE), "{kind}.{key}: help names roll_table");
    }
}
