//! `PR7-8` (`RELIC-SIG1`): the content checks and schema help of the relic signature hooks, on a scratch data folder
//! built here (no shipped data). Each new key checks clean in its valid form and each broken rule is one error:
//!
//! - the effect `meter_floor` {meter, floor} (an item's `effect` modifier, a passive's effect): a known unit meter, a
//!   whole floor in that meter's min..max, no other key but `when`; it reads as `Grant::MeterFloor`, a gear grant;
//! - the `skill_potency` group `attack`;
//! - the `when` keys `from_act` (a whole number in `WHEN_FROM_ACT`) and `holds_meter` (a known unit meter), on an
//!   effect row and on a trigger;
//! - the trigger effect `shockwave` {tiles, pct_of_hit}: both required, in `SHOCKWAVE_TILES` / `SHOCKWAVE_PCT`, on a
//!   `hit_dealt` or `kill` trigger only.

use aegis_content::schema::fields;
use aegis_core::json;
use rpgkit_content::{check, source, store};
use rpgkit_data::{METER_FLOOR, SHOCKWAVE, SHOCKWAVE_PCT, SHOCKWAVE_TILES, UNIT_METERS_KIND, WHEN_FROM_ACT, WHEN_HOLDS_METER};
use rpgkit_rules::grants::{self, Grant};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const GAUGE_MAX: i64 = 100;

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_relic_hooks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn floor(meter: &str, floor: Value) -> Value {
    json!({"type": METER_FLOOR, "meter": meter, "floor": floor})
}

fn wave(on: &str, tiles: Value, pct: Value) -> Value {
    json!({"type": "trigger", "on": on, "limit": "none", "effects": [{"type": SHOCKWAVE, "tiles": tiles, "pct_of_hit": pct}]})
}

/// Skills, a unit meter, a passive and an accessory family; a ring with a floor from act 2 and an attack bonus while
/// the meter is held, a band with a shockwave.
fn catalog(data: &Path) {
    let one = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(data, "skills", 2, json!([one("move", "MOVE", json!({"system": true})), one("rest", "REST", json!({"system": true})), one("strike", "ATTACK", json!({"basic": true}))]));
    write_kind(data, UNIT_METERS_KIND, 1, json!([{"id": "t_gauge", "name": "Gauge", "short": "Ga", "min": 0, "max": GAUGE_MAX, "show": "bar"}]));
    write_kind(data, "passives", 1, json!([{"id": "t_steady", "name": "Steady", "slot": "", "effects": [floor("t_gauge", json!(10))]}]));
    write_kind(
        data,
        "item_families",
        1,
        json!([{"id": "t_band", "name": "Band", "item_type": "ACCESSORY", "slots": ["ACCESSORY_1", "ACCESSORY_2"], "modifier_kinds": ["stat_bonus", "effect"], "max_modifiers": 4}]),
    );
    let ring = json!([
        {"kind": "effect", "effect": {"type": "skill_potency", "group": "attack", "value": 0.3, "when": {WHEN_HOLDS_METER: "t_gauge"}}},
        {"kind": "effect", "effect": {"type": METER_FLOOR, "meter": "t_gauge", "floor": 20, "when": {"from_act": 2}}}
    ]);
    write_kind(
        data,
        "items",
        2,
        json!([
            {"id": "t_ring", "name": "Ring", "family": "t_band", "slot": "ACCESSORY_1", "modifiers": ring},
            {"id": "t_wave", "name": "Wave", "family": "t_band", "slot": "ACCESSORY_1", "modifiers": [{"kind": "effect", "effect": wave("hit_dealt", json!(2), json!(50))}]}
        ]),
    );
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
    let want = Grant::MeterFloor { meter: "t_gauge".into(), floor: 10 };
    assert_eq!(grants::read(&v.typed.passives["t_steady"].effects), vec![want.clone()]);
    assert!(want.gear_ok() && !want.band_ok(), "worn gear applies it; a band does not");
    let (always, cond) = grants::read_with_when(&v.typed.items["t_ring"].effects);
    assert!(always.is_empty() && cond.len() == 2, "both ring effects are conditional");
    assert_eq!(cond[1].1, vec![Grant::MeterFloor { meter: "t_gauge".into(), floor: 20 }]);
    let report = check::validate_all(&v);
    for key in ["items:t_ring", "items:t_wave", "passives:t_steady"] {
        assert!(!report.contains_key(key), "{key}: {:?}", report.get(key));
    }
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn each_broken_rule_is_one_error() {
    let (_scratch, data) = scratch("rules");
    catalog(&data);
    let v = store::view(&data);
    let ring = |fx: Value| json!({"modifiers": [{"kind": "effect", "effect": fx}]});
    let passive = |fx: Value| json!({"effects": [fx]});
    let (alo, ahi) = WHEN_FROM_ACT;
    let (tlo, thi) = SHOCKWAVE_TILES;
    let (plo, phi) = SHOCKWAVE_PCT;
    let floor_range = format!("meter_floor floor must be a whole number 0-{GAUGE_MAX}");
    let potency = |when: Value| json!({"type": "skill_potency", "group": "attack", "value": 0.3, "when": when});
    let cases: Vec<(&str, &str, Value, String)> = vec![
        ("items", "t_ring", ring(floor("t_gauge", json!(GAUGE_MAX))), String::new()),
        ("items", "t_ring", ring(floor("t_nope", json!(5))), "meter_floor: unknown unit meter 't_nope'.".into()),
        ("items", "t_ring", ring(floor("t_gauge", json!(GAUGE_MAX + 1))), floor_range.clone()),
        ("items", "t_ring", ring(floor("t_gauge", json!(-1))), floor_range.clone()),
        ("items", "t_ring", ring(floor("t_gauge", json!(2.5))), floor_range.clone()),
        ("items", "t_ring", ring(json!({"type": METER_FLOOR, "meter": "t_gauge", "floor": 5, "to": "self"})), "meter_floor takes only meter and floor (not 'to').".into()),
        ("passives", "t_steady", passive(floor("t_nope", json!(5))), "meter_floor: unknown unit meter 't_nope'.".into()),
        ("passives", "t_steady", passive(potency(json!({}))), String::new()),
        ("items", "t_ring", ring(json!({"type": "skill_potency", "group": "fists", "value": 0.3})), "unknown skill group 'fists'".into()),
        ("items", "t_ring", ring(potency(json!({"from_act": ahi}))), String::new()),
        ("items", "t_ring", ring(potency(json!({"from_act": alo - 1}))), format!("from_act must be a whole number {alo}-{ahi}.")),
        ("items", "t_ring", ring(potency(json!({"from_act": ahi + 1}))), format!("from_act must be a whole number {alo}-{ahi}.")),
        ("items", "t_ring", ring(potency(json!({WHEN_HOLDS_METER: "t_nope"}))), "holds_meter: unknown unit meter 't_nope'.".into()),
        ("items", "t_wave", ring(wave("kill", json!(thi), json!(phi))), String::new()),
        ("items", "t_wave", ring(json!({"type": "trigger", "on": "hit_dealt", "when": {"from_act": 2, WHEN_HOLDS_METER: "t_gauge"}, "effects": [{"type": "meter", "meter": "t_gauge", "delta": -10}]})), String::new()),
        ("items", "t_wave", ring(json!({"type": "trigger", "on": "hit_dealt", "when": {"from_act": 0}, "effects": [{"type": "crit", "value": 1}]})), format!("from_act must be a whole number {alo}-{ahi}.")),
        ("items", "t_wave", ring(wave("hit_dealt", json!(tlo - 1), json!(plo))), format!("tiles must be a whole number {tlo}-{thi}.")),
        ("items", "t_wave", ring(wave("hit_dealt", json!(thi + 1), json!(plo))), format!("tiles must be a whole number {tlo}-{thi}.")),
        ("items", "t_wave", ring(wave("hit_dealt", json!(tlo), json!(phi + 1))), format!("pct_of_hit must be a whole number {plo}-{phi}.")),
        ("items", "t_wave", ring(json!({"type": "trigger", "on": "hit_dealt", "effects": [{"type": SHOCKWAVE, "tiles": 2}]})), "pct_of_hit is required".into()),
        ("items", "t_wave", ring(wave("hit_taken", json!(2), json!(50))), "shockwave needs a hit_dealt or kill trigger".into()),
    ];
    let mut failures = Vec::new();
    for (kind, id, change, want) in &cases {
        let errors = errors_of(&v, kind, id, change);
        let ok = if want.is_empty() { errors.is_empty() } else { errors.len() == 1 && errors[0].contains(want.as_str()) };
        if !ok {
            failures.push(format!("{kind} {id} {change}: want {want:?}, got {errors:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn the_schema_help_names_the_hooks() {
    let field = |kind: &str, key: &str| fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("{kind}.{key}"));
    for kind in ["items", "passives"] {
        let help = json::s(&field(kind, "effects"), "help");
        for word in [METER_FLOOR, SHOCKWAVE, "from_act", WHEN_HOLDS_METER, "attack"] {
            assert!(help.contains(word), "{kind}.effects help names {word}");
        }
    }
}
