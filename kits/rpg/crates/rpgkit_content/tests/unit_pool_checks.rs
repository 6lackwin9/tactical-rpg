//! U1.2-S5 (docs/sessions/U1_STATS.md "Adding a derived value without a logic change", FB-08): the checks and the
//! writer of a pool from data, on scratch data folders only. A unit meter row's pool fields (`max_stat` a derived stat,
//! `start_full`, `delta_max` 1-100, `triggers` as trigger rows) and its bands' `from_pct` (0-100, instead of `from`,
//! ascending at the row's max) are checked, a band may hold any kit status, a `meter` effect's delta lies within its
//! meter's `delta_max` (10 by default), a Rest may move a meter, and a ruleset's row-only `unit_pools` names unit meter
//! rows and is read only with unit_meters. The writer stores the pool fields only when they differ from the defaults.

use aegis_core::json::Dict;
use rpgkit_content::check;
use rpgkit_content::{source, store};
use rpgkit_data::UNIT_METERS_KIND;
use rpgkit_rules::ruleset;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_unit_pool_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// The composure acceptance row (U1): `KNOW x 2 + CONS` (0..100) as each holder's max, full each fight, -10 when a
/// friend within 2 is knocked, +5 on Rest, weaken below half, a named band at 0.
fn composure() -> Value {
    json!({"id": "composure", "name": "Composure", "short": "Comp", "min": 0, "max": 100, "show": "bar", "max_stat": "composure", "start_full": true,
        "bands": [
            {"id": "shaken", "name": "Shaken", "from": 0},
            {"id": "rattled", "name": "Rattled", "from": 1, "statuses": [{"status": "weaken", "magnitude": 10}]},
            {"id": "steady", "name": "Steady", "from_pct": 50}],
        "triggers": [{"on": "ally_downed", "when": {"range": 2}, "effects": [{"type": "meter", "meter": "composure", "delta": -10}], "limit": "none"}]})
}

fn catalog(data: &Path, pools: Value) {
    let one = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(
        data,
        "skills",
        2,
        json!([
            one("move", "MOVE", json!({"system": true})),
            one("rest", "REST", json!({"system": true, "meters": [{"meter": "composure", "delta": 5}]})),
            one("strike", "ATTACK", json!({"basic": true}))
        ]),
    );
    write_kind(
        data,
        "derived_stats",
        1,
        json!([{"id": "composure", "name": "Composure", "formula": "KNOW * per_know + CONS * per_cons", "constants": {"per_know": 2, "per_cons": 1}, "min": 0, "max": 100}]),
    );
    write_kind(data, UNIT_METERS_KIND, 1, json!([composure(), {"id": "phial", "name": "Bile Phial", "short": "Phial", "max": 3}]));
    write_kind(data, ruleset::KIND, 1, json!([{"id": "timeline", "name": "Timeline", "rules": ["unit_meters", "triggers"], "unit_pools": pools}]));
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
fn the_composure_row_loads_and_checks_clean() {
    let (_scratch, data) = scratch("clean");
    catalog(&data, json!(["composure"]));
    let src = source::load(&data);
    let catalog_errors: Vec<&String> = src.errors.iter().filter(|e| e.contains("catalog")).collect();
    assert!(catalog_errors.is_empty(), "{catalog_errors:?}");
    let v = store::view(&data);
    let m = &v.typed.unit_meters["composure"];
    assert_eq!((m.max_stat.as_str(), m.start_full, m.delta_max, m.triggers.len()), ("composure", true, rpgkit_data::METER_DELTA_MAX, 1));
    assert_eq!(v.typed.skill("rest").unwrap().meters, vec![rpgkit_data::MeterDelta { meter: "composure".into(), delta: 5 }]);
    let report = check::validate_all(&v);
    for key in ["unit_meters:composure", "unit_meters:phial", "rulesets:timeline", "skills:rest"] {
        assert!(!report.contains_key(key), "{key}: {:?}", report.get(key));
    }
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn each_broken_pool_field_is_one_error() {
    let (_scratch, data) = scratch("rules");
    catalog(&data, json!(["composure"]));
    let v = store::view(&data);
    let meter_fx = |delta: i64| json!([{"on": "ally_downed", "when": {"range": 2}, "effects": [{"type": "meter", "meter": "composure", "delta": delta}]}]);
    let cases: Vec<(&str, &str, Value, &str)> = vec![
        (UNIT_METERS_KIND, "composure", json!({}), ""),
        (UNIT_METERS_KIND, "composure", json!({"max_stat": "grit"}), "Max stat: unknown derived stat 'grit'."),
        (UNIT_METERS_KIND, "composure", json!({"delta_max": 0}), "delta_max) must be a whole number 1-100."),
        (UNIT_METERS_KIND, "composure", json!({"delta_max": 101}), "delta_max) must be a whole number 1-100."),
        // bands: from or from_pct (ascending at the row's max), any kit status
        (UNIT_METERS_KIND, "composure", json!({"bands": [{"id": "a", "from": 0}, {"id": "b", "from": 10, "from_pct": 20}]}), "Band 'b': set from or from_pct, not both."),
        (UNIT_METERS_KIND, "composure", json!({"bands": [{"id": "a", "from": 0}, {"id": "b", "from_pct": 120}]}), "Band 'b': from_pct must be a whole number 0-100."),
        (UNIT_METERS_KIND, "composure", json!({"bands": [{"id": "a", "from_pct": 0}, {"id": "b", "from_pct": 50}, {"id": "c", "from": 60}]}), ""),
        (UNIT_METERS_KIND, "composure", json!({"bands": [{"id": "a", "from": 0}, {"id": "b", "from": 60}, {"id": "c", "from_pct": 50}]}), "Band 'c': bands run in ascending from."),
        (UNIT_METERS_KIND, "composure", json!({"bands": [{"id": "a", "from_pct": 10}]}), "Band 'a': the first band starts at min (0)."),
        (UNIT_METERS_KIND, "composure", json!({"bands": [{"id": "a", "from": 0, "statuses": [{"status": "stagger"}]}]}), ""),
        (UNIT_METERS_KIND, "composure", json!({"bands": [{"id": "a", "from": 0, "statuses": [{"status": "weaken", "magnitude": 30}]}]}), "Band 'a' status 1: weaken needs a magnitude 1-25."),
        // triggers: trigger rows; a meter effect within delta_max (10 by default)
        (UNIT_METERS_KIND, "composure", json!({"triggers": meter_fx(-25)}), "Trigger 1 effect 1: delta must be a whole number -10..10, not 0."),
        (UNIT_METERS_KIND, "composure", json!({"triggers": meter_fx(-25), "delta_max": 30}), ""),
        (UNIT_METERS_KIND, "composure", json!({"triggers": [{"on": "nap", "effects": [{"type": "ap", "value": 1}]}]}), "Trigger 1: on 'nap' is not one of"),
        (UNIT_METERS_KIND, "composure", json!({"triggers": [{"on": "hit_taken", "when": {"range": 2}, "effects": [{"type": "ap", "value": 1}]}]}), "range is for ally_hit and ally_downed only."),
        (UNIT_METERS_KIND, "composure", json!({"triggers": [{"on": "ally_downed", "when": {"range": 7}, "effects": [{"type": "ap", "value": 1}]}]}), "range must be"),
        // a Rest's meter change is a meter cost like any other
        ("skills", "rest", json!({"meters": [{"meter": "grit", "delta": 5}]}), "Meter cost 1: unknown unit meter 'grit'."),
        // the ruleset's list is a list of ids
        (ruleset::KIND, "timeline", json!({"unit_pools": "composure"}), "unit_pools must be a list of unit meter ids."),
        (ruleset::KIND, "timeline", json!({"unit_pools": ["composure", "composure"]}), "unit_pools lists 'composure' twice."),
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

/// A pool the ruleset names must be a unit meter row (the full check, the meters load after the rulesets); set
/// without the rule unit_meters, the list warns that nothing reads it.
#[test]
fn unit_pools_name_meter_rows_and_need_the_rule() {
    let (_scratch, data) = scratch("pools");
    catalog(&data, json!(["composure", "grit"]));
    let v = store::view(&data);
    let report = check::validate_all(&v);
    assert_eq!(report.get("rulesets:timeline").cloned().unwrap_or_default(), vec!["unit_pools: 'grit' has no row in unit_meters (data/catalog/unit_meters.json).".to_string()]);
    let mut row: Dict = v.get(ruleset::KIND, "timeline").clone();
    row.insert("rules".into(), json!(["triggers"]));
    let warnings = check::warnings_for(&v, ruleset::KIND, &row);
    assert!(warnings.iter().any(|w| w == "unit_pools is set but read only with unit_meters."), "{warnings:?}");
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

/// The writer stores a meter row's pool fields only when they differ from their defaults: a row written before they
/// existed keeps its keys, the composure row keeps its own.
#[test]
fn the_writer_stores_pool_fields_only_when_set() {
    let (_scratch, data) = scratch("store");
    catalog(&data, json!([]));
    let v = store::view(&data);
    let pool_keys = ["max_stat", "start_full", "delta_max", "triggers"];
    let stored = |id: &str| store::catalog_storage(UNIT_METERS_KIND, v.get(UNIT_METERS_KIND, id), &[]).unwrap();
    let phial = stored("phial");
    assert!(pool_keys.iter().all(|k| !phial.contains_key(*k)), "{phial:?}");
    let comp = stored("composure");
    assert_eq!(pool_keys.iter().filter(|k| comp.contains_key(**k)).count(), 3, "all but delta_max (at its default): {comp:?}");
    assert!(!comp.contains_key("delta_max"));
    let text = store::resave_text(&data, None, UNIT_METERS_KIND).unwrap();
    fs::write(data.join("catalog").join(format!("{UNIT_METERS_KIND}.json")), &text).unwrap();
    assert_eq!(store::resave_text(&data, None, UNIT_METERS_KIND).unwrap(), text, "a re-save without changes is byte-identical");
    let _ = fs::remove_dir_all(data.parent().unwrap());
}
