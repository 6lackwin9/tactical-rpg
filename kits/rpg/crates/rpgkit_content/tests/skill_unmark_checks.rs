//! PK2-S1 (PK1 3.7, `FB-174` (6)): the schema, content checks and writer of a skill's `unmark` (the consumable's effect
//! as a SUPPORT skill, the priest's Inoculated keystone) and `uses_per_battle` (how often a unit uses the skill in one
//! battle), on a scratch data folder built here (no shipped data). The schema and the code name the same fields; a
//! SUPPORT skill with only `unmark` is a valid support skill; each mistake is one error; another kind's `unmark` is the
//! belongs-to error; the writer stores both only when set.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::hooks::USES_PER_BATTLE;
use rpgkit_content::check::{self, skill};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_skill_unmark_checks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// Fixture files: engine skills, Strike, a Jab, an unmark-only Inoculate (once a battle) and a class.
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
        json!([
            one("move", "MOVE", json!({"system": true})),
            one("rest", "REST", json!({"system": true})),
            one("strike", "ATTACK", json!({"basic": true})),
            one("jab", "ATTACK", json!({})),
            one("inoculate", "SUPPORT", json!({"target_mode": "ALLY", "unmark": {"mark": "calcify"}, "uses_per_battle": 1}))
        ]),
    );
    write_kind(
        data,
        "classes",
        1,
        json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {},
                "proficiency": {}, "skill_path": {"jab": 1, "inoculate": 1}}]),
    );
}

fn errors_of(v: &check::View, id: &str, change: &Value) -> Vec<String> {
    let mut e = v.get("skills", id).clone();
    assert!(!e.is_empty(), "fixture skills {id}");
    for (k, x) in change.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, "skills", &e, id)
}

fn field_of(key: &str) -> Dict {
    fields(source::schema(), "content", "skills").iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("content.skills.{key}")).clone()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().map(json::gstr).collect()).unwrap_or_default()
}

/// The schema and the code name the same fields: `unmark` SUPPORT only (the support section, json, {} = none, stored
/// only when set), `uses_per_battle` every authored kind (the cost section, an int, 0 = no limit); neither per rank.
#[test]
fn the_schema_and_the_code_name_the_fields() {
    let f = field_of("unmark");
    assert!(!json::b(&f, "rank", false) && json::b(&f, "omit_default", false));
    assert_eq!((json::s(&f, "type"), json::get(&f, "default")), ("json".to_string(), &json!({})));
    assert_eq!(skill::KINDS.iter().copied().filter(|k| skill::has_field(k, "unmark")).collect::<Vec<_>>(), ["SUPPORT"]);
    assert_eq!(strings(json::get(json::dget(&f, "when"), "in")), ["SUPPORT"]);
    assert_eq!(strings(json::get(&f, "section")), ["support"]);
    let f = field_of("uses_per_battle");
    assert!(!json::b(&f, "rank", false));
    assert_eq!((json::s(&f, "type"), json::get(&f, "default")), ("int".to_string(), &json!(0)));
    let users: Vec<&str> = skill::KINDS.iter().copied().filter(|k| skill::has_field(k, "uses_per_battle")).collect();
    assert_eq!(users, ["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE"]);
    assert_eq!(strings(json::get(&f, "section")), ["cost"]);
    assert_eq!(USES_PER_BATTLE, (0, 9));
}

/// An unmark-only SUPPORT skill is a support skill; the limits 0 and 9 check clean; a Jab may carry a limit.
#[test]
fn the_valid_forms_check_clean() {
    let (_s, data) = scratch("valid");
    catalog(&data);
    let v = store::view(&data);
    for change in [json!({}), json!({"uses_per_battle": 0}), json!({"uses_per_battle": USES_PER_BATTLE.1})] {
        assert_eq!(errors_of(&v, "inoculate", &change), Vec::<String>::new(), "inoculate {change}");
    }
    assert_eq!(errors_of(&v, "jab", &json!({"uses_per_battle": 2})), Vec::<String>::new());
    assert!(store::check(&data).iter().all(|e| !e.contains("inoculate")), "{:?}", store::check(&data));
    let cat = rpgkit_data::Catalog::from_value(&source::load(&data).catalog);
    let s = cat.skill("inoculate").unwrap();
    assert_eq!((s.unmark.as_str(), s.uses_per_battle), ("calcify", 1));
}

/// Each mistake is one error; a SUPPORT skill with no use at all keeps the old message; another kind's `unmark` is the
/// belongs-to error.
#[test]
fn each_broken_rule_is_one_error() {
    let (_s, data) = scratch("broken");
    catalog(&data);
    let v = store::view(&data);
    assert_eq!(errors_of(&v, "inoculate", &json!({"uses_per_battle": 10})), ["Uses per battle must be 0-9 (is 10; 0 = no limit)."]);
    assert_eq!(errors_of(&v, "inoculate", &json!({"uses_per_battle": -1})), ["Uses per battle must be 0-9 (is -1; 0 = no limit)."]);
    assert_eq!(errors_of(&v, "inoculate", &json!({"unmark": {"mark": ""}})), ["Unmark: mark needs an id of 1-24 characters (a-z, 0-9, _, a letter first)."]);
    assert_eq!(errors_of(&v, "inoculate", &json!({"unmark": {"mark": "calcify", "value": 1}})), ["Unmark: unknown key 'value' (keys: mark)."]);
    assert_eq!(errors_of(&v, "inoculate", &json!({"unmark": {}})), ["A support skill needs a heal (base or per-stat), a cleanse or a granted status."]);
    assert_eq!(errors_of(&v, "jab", &json!({"unmark": {"mark": "calcify"}})), ["Unmark belongs to SUPPORT skills, not ATTACK."]);
}

/// The writer stores `unmark` and `uses_per_battle` only when set.
#[test]
fn the_writer_stores_the_fields_only_when_set() {
    let row = |extra: Value| {
        let mut r = json!({"id": "inoculate", "name": "Inoculate", "kind": "SUPPORT", "level": 1, "blocks": 1, "description": "", "target_mode": "ALLY", "heal_base": 2});
        for (k, v) in extra.as_object().unwrap() {
            r[k] = v.clone();
        }
        r.as_object().cloned().unwrap()
    };
    let stored = store::catalog_storage("skills", &row(json!({"unmark": {}, "uses_per_battle": 0})), &[]).unwrap();
    assert!(!stored.contains_key("unmark") && !stored.contains_key("uses_per_battle"), "{stored:?}");
    let stored = store::catalog_storage("skills", &row(json!({"unmark": {"mark": "calcify"}, "uses_per_battle": 1})), &[]).unwrap();
    assert_eq!((json::get(&stored, "unmark"), json::get(&stored, "uses_per_battle")), (&json!({"mark": "calcify"}), &json!(1)));
}
