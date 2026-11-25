//! ACT1-HOOKS2 item 3 (`FB-244` (4-6)): the content checks, schema and writer of the gear and consumable hooks, on a
//! scratch data folder built here (no shipped data). Each new key checks clean in its valid form and each broken rule is
//! one error:
//!
//! - the effect `skill_block_cost` {skill, delta} (an item's `effect` modifier, a passive's effect): a known skill, a
//!   whole delta in `SKILL_BLOCK_COST_DELTA`, not 0, no other key;
//! - a consumable's `meter_delta` {meter, delta}: a known unit meter, a whole delta -100..100, not 0, no other key; an
//!   item with only a meter change is a valid use (no heal, damage or buff needed, as `unmark`);
//! - a consumable's `armor_piercing`: only on a damage item (an error otherwise, so the flag never sits idle).
//!
//! The loader reads them into the typed rows; the writer stores an empty `meter_delta` and a false `armor_piercing` as
//! no key (a row written before them re-saves as it was), and the schema help names every key.

use aegis_content::schema::fields;
use aegis_core::json;
use rpgkit_content::{check, source, store};
use rpgkit_data::{MeterDelta, ITEM_ARMOR_PIERCING, ITEM_METER_DELTA, SKILL_BLOCK_COST, SKILL_BLOCK_COST_DELTA, UNIT_METERS_KIND};
use rpgkit_rules::grants::{self, Grant};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const HEAVY: &str = "t_heavy";

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_hooks2_gear_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn block_cost(skill: &str, delta: Value) -> Value {
    json!({"type": SKILL_BLOCK_COST, "skill": skill, "delta": delta})
}

/// Skills (a 3-block one), a unit meter, a passive, a consumable family and an accessory family; a ring with a stat
/// bonus and a `skill_block_cost`, a vial with only a `meter_delta`, an armour-piercing dart.
fn catalog(data: &Path) {
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
        json!([one("move", "MOVE", json!({"system": true})), one("rest", "REST", json!({"system": true})), one("strike", "ATTACK", json!({"basic": true})), one(HEAVY, "ATTACK", json!({"blocks": 3}))]),
    );
    write_kind(data, UNIT_METERS_KIND, 1, json!([{"id": "t_gauge", "name": "Gauge", "short": "Ga", "min": 0, "max": 100, "show": "bar"}]));
    write_kind(data, "passives", 1, json!([{"id": "t_quick", "name": "Quick", "slot": "", "effects": [block_cost(HEAVY, json!(-1))]}]));
    write_kind(
        data,
        "item_families",
        1,
        json!([
            {"id": "t_pouch", "name": "Pouch", "item_type": "CONSUMABLE"},
            {"id": "t_band", "name": "Band", "item_type": "ACCESSORY", "slots": ["ACCESSORY_1", "ACCESSORY_2"], "modifier_kinds": ["stat_bonus", "effect"], "max_modifiers": 4}
        ]),
    );
    write_kind(
        data,
        "items",
        2,
        json!([
            {"id": "t_ring", "name": "Ring", "family": "t_band", "slot": "ACCESSORY_1", "modifiers": [
                {"kind": "stat_bonus", "stat": "PERC", "value": 1},
                {"kind": "effect", "effect": block_cost(HEAVY, json!(-1))}]},
            {"id": "t_vial", "name": "Vial", "family": "t_pouch", "consumable_type": "FOCUS_INCENSE", "use_blocks": 1, "use_reach": 0, ITEM_METER_DELTA: {"meter": "t_gauge", "delta": -25}},
            {"id": "t_dart", "name": "Dart", "family": "t_pouch", "consumable_type": "THROWABLE_BOMB", "damage": 32, "use_blocks": 2, "use_reach": 3, ITEM_ARMOR_PIERCING: true}
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
    let items = &v.typed.items;
    assert_eq!(items["t_vial"].meter_delta, Some(MeterDelta { meter: "t_gauge".into(), delta: -25 }));
    assert!(!items["t_vial"].armor_piercing && items["t_vial"].offered_in_battle() && !items["t_vial"].aims_at_foes(), "a self-use item");
    assert!(items["t_dart"].armor_piercing && items["t_dart"].meter_delta.is_none());
    assert_eq!(items["t_ring"].stat_bonuses.get("PERC"), Some(&1));
    let want = Grant::SkillBlockCost { skill: HEAVY.into(), delta: -1 };
    assert_eq!(grants::read(&items["t_ring"].effects), vec![want.clone()], "the ring's effect, read as a gear grant");
    assert!(want.gear_ok() && !want.band_ok());
    assert_eq!(grants::read(&v.typed.passives["t_quick"].effects), vec![want]);
    let report = check::validate_all(&v);
    for key in ["items:t_ring", "items:t_vial", "items:t_dart", "passives:t_quick"] {
        assert!(!report.contains_key(key), "{key}: {:?}", report.get(key));
    }
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn each_broken_rule_is_one_error() {
    let (_scratch, data) = scratch("rules");
    catalog(&data);
    let v = store::view(&data);
    let (lo, hi) = SKILL_BLOCK_COST_DELTA;
    let ring = |fx: Value| json!({"modifiers": [{"kind": "stat_bonus", "stat": "PERC", "value": 1}, {"kind": "effect", "effect": fx}]});
    let passive = |fx: Value| json!({"effects": [fx]});
    let range = format!("skill_block_cost delta must be a whole number {lo}..{hi}, not 0.");
    let cases: Vec<(&str, &str, Value, String)> = vec![
        ("items", "t_ring", ring(block_cost(HEAVY, json!(hi))), String::new()),
        ("items", "t_ring", ring(block_cost("t_nope", json!(-1))), "skill_block_cost: unknown skill 't_nope'.".into()),
        ("items", "t_ring", ring(block_cost(HEAVY, json!(0))), range.clone()),
        ("items", "t_ring", ring(block_cost(HEAVY, json!(lo - 1))), range.clone()),
        ("items", "t_ring", ring(block_cost(HEAVY, json!(hi + 1))), range.clone()),
        ("items", "t_ring", ring(block_cost(HEAVY, json!(-1.5))), range.clone()),
        ("items", "t_ring", ring(json!({"type": SKILL_BLOCK_COST, "skill": HEAVY, "delta": -1, "to": "self"})), "skill_block_cost takes only skill and delta (not 'to').".into()),
        ("passives", "t_quick", passive(block_cost(HEAVY, json!(2))), String::new()),
        ("passives", "t_quick", passive(block_cost("t_nope", json!(-1))), "skill_block_cost: unknown skill 't_nope'.".into()),
        ("passives", "t_quick", passive(block_cost(HEAVY, json!(0))), range),
        ("items", "t_vial", json!({ITEM_METER_DELTA: {"meter": "t_gauge", "delta": 100}}), String::new()),
        ("items", "t_vial", json!({ITEM_METER_DELTA: {}}), "A consumable should heal, damage or buff.".into()),
        ("items", "t_vial", json!({ITEM_METER_DELTA: {"meter": "t_rage", "delta": -5}}), "Meter change: unknown unit meter 't_rage'.".into()),
        ("items", "t_vial", json!({ITEM_METER_DELTA: {"meter": "t_gauge", "delta": 0}}), "Meter change: delta must be a whole number -100..100, not 0.".into()),
        ("items", "t_vial", json!({ITEM_METER_DELTA: {"meter": "t_gauge", "delta": -101}}), "Meter change: delta must be a whole number -100..100, not 0.".into()),
        ("items", "t_vial", json!({ITEM_METER_DELTA: {"delta": -5}}), "Meter change: meter is required".into()),
        ("items", "t_vial", json!({ITEM_METER_DELTA: {"meter": "t_gauge", "delta": -5, "to": "x"}}), "Meter change: unknown key 'to'".into()),
        // the schema's json field reads a non-object as its default ({}): no meter change left
        ("items", "t_vial", json!({ITEM_METER_DELTA: 5}), "A consumable should heal, damage or buff.".into()),
        ("items", "t_vial", json!({ITEM_ARMOR_PIERCING: true}), "Armour piercing: only a damage item pierces armour".into()),
        ("items", "t_dart", json!({ITEM_ARMOR_PIERCING: false}), String::new()),
        ("items", "t_dart", json!({ITEM_METER_DELTA: {"meter": "t_gauge", "delta": 10}}), String::new()),
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
fn the_writer_stores_the_unset_keys_as_no_key() {
    let (_scratch, data) = scratch("writer");
    catalog(&data);
    let v = store::view(&data);
    let families: Vec<json::Dict> = v.entries("item_families").to_vec();
    let stored = |row: Value| store::catalog_storage("items", row.as_object().unwrap(), &families).unwrap();
    let plain = json!({"id": "t_tonic", "name": "Tonic", "family": "t_pouch", "consumable_type": "HEALTH_POTION", "heal": 20});
    let before = stored(plain.clone());
    assert!(!before.contains_key(ITEM_METER_DELTA) && !before.contains_key(ITEM_ARMOR_PIERCING), "a row written before the fields: {before:?}");
    let mut unset = plain.clone();
    unset[ITEM_METER_DELTA] = json!({});
    unset[ITEM_ARMOR_PIERCING] = json!(false);
    assert_eq!(stored(unset), before, "the defaults re-save as no key");
    let mut set = plain;
    set[ITEM_METER_DELTA] = json!({"meter": "t_gauge", "delta": -25});
    set[ITEM_ARMOR_PIERCING] = json!(true);
    let after = stored(set);
    assert_eq!(after.get(ITEM_METER_DELTA), Some(&json!({"meter": "t_gauge", "delta": -25})));
    assert_eq!(after.get(ITEM_ARMOR_PIERCING), Some(&json!(true)));
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn the_schema_holds_the_keys_and_names_them_in_its_help() {
    let field = |kind: &str, key: &str| fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("{kind}.{key}"));
    for (key, default) in [(ITEM_METER_DELTA, json!({})), (ITEM_ARMOR_PIERCING, json!(false))] {
        let f = field("items", key);
        assert_eq!(json::get(&f, "default"), &default, "{key}");
        assert!(json::b(&f, "omit_default", false), "{key} is stored only when set");
        assert_eq!(json::get(&f, "when"), &json!({"key": "item_type", "in": ["CONSUMABLE"]}), "{key}");
    }
    assert!(json::s(&field("items", ITEM_METER_DELTA), "help").contains("\"delta\""));
    assert!(json::s(&field("items", "effects"), "help").contains(SKILL_BLOCK_COST));
    assert!(json::s(&field("passives", "effects"), "help").contains(SKILL_BLOCK_COST));
}
