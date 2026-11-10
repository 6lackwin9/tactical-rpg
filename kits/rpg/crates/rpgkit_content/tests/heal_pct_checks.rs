//! HEALPCT1 (`FB-177` Option A): the schema, content checks, writer and `{heal}` token of a SUPPORT skill's `heal_pct`
//! (a percent of the target's max HP), on a scratch data folder built here (no shipped data). The schema and the code
//! name the same field (SUPPORT only, not per rank, default 0); a skill with only `heal_pct` is a valid support skill;
//! out of 0-100 is one error; on another kind it is the belongs-to error; the writer stores it only when set; the
//! `{heal}` token names the percent after the formula.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::hooks::HEAL_PCT;
use rpgkit_content::check::{self, skill};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_heal_pct_checks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// Fixture files: engine skills, Strike, a Mend heal by formula, a Feed (self, a heal by `heal_pct` only) and a class.
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
            one("mend", "SUPPORT", json!({"target_mode": "ALLY", "heal_base": 10})),
            one("feed", "SUPPORT", json!({"target_mode": "SELF", "heal_pct": 4}))
        ]),
    );
    write_kind(
        data,
        "classes",
        1,
        json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {},
                "proficiency": {}, "skill_path": {"jab": 1, "mend": 1, "feed": 1}}]),
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

/// The schema and the code name the same field: SUPPORT only (`when` and the support section), not per rank, an int
/// with default 0 (none); the range is the checks' `HEAL_PCT`, 0-100.
#[test]
fn the_schema_and_the_code_name_heal_pct() {
    let f = field_of("heal_pct");
    assert!(!json::b(&f, "rank", false), "not per rank");
    assert_eq!(json::s(&f, "type"), "int");
    assert_eq!(json::get(&f, "default"), &json!(0));
    let users: Vec<&str> = skill::KINDS.iter().copied().filter(|k| skill::has_field(k, "heal_pct")).collect();
    assert_eq!(users, ["SUPPORT"]);
    let shown: Vec<String> = json::aget(json::dget(&f, "when"), "in").iter().map(json::gstr).collect();
    assert_eq!(shown, ["SUPPORT"]);
    let section: Vec<String> = json::aget(&f, "section").iter().map(json::gstr).collect();
    assert_eq!(section, ["support"]);
    assert_eq!(HEAL_PCT, (0, 100));
}

/// A skill with only `heal_pct` is a support skill with a heal; 0, 1 and 100 check clean; the formula and the percent
/// together check clean.
#[test]
fn the_valid_forms_check_clean() {
    let (_s, data) = scratch("valid");
    catalog(&data);
    let v = store::view(&data);
    for change in [json!({}), json!({"heal_pct": 1}), json!({"heal_pct": HEAL_PCT.1}), json!({"heal_pct": 0, "heal_base": 3})] {
        assert_eq!(errors_of(&v, "feed", &change), Vec::<String>::new(), "feed {change}");
    }
    assert_eq!(errors_of(&v, "mend", &json!({"heal_pct": 4})), Vec::<String>::new(), "mend: formula and percent");
    assert!(store::check(&data).iter().all(|e| !e.contains("feed")), "{:?}", store::check(&data));
}

/// Out of range is one error naming the field; a SUPPORT skill with neither heal nor grant nor cleanse keeps the old
/// message (its words unchanged); another kind's `heal_pct` is the belongs-to error.
#[test]
fn each_broken_rule_is_one_error() {
    let (_s, data) = scratch("broken");
    catalog(&data);
    let v = store::view(&data);
    // Mend keeps its formula, so the range error stands alone (a negative percent is no heal for the support check)
    for (pct, want) in [(-1, "Heal % must be 0-100 (is -1)."), (101, "Heal % must be 0-100 (is 101).")] {
        assert_eq!(errors_of(&v, "mend", &json!({"heal_pct": pct})), [want], "{pct}");
    }
    assert_eq!(errors_of(&v, "feed", &json!({"heal_pct": 101})), ["Heal % must be 0-100 (is 101)."], "Feed: out of range is still a heal");
    assert_eq!(errors_of(&v, "feed", &json!({"heal_pct": 0})), ["A support skill needs a heal (base or per-stat), a cleanse or a granted status."]);
    assert_eq!(errors_of(&v, "jab", &json!({"heal_pct": 4})), ["Heal % belongs to SUPPORT skills, not ATTACK."]);
}

/// The writer stores `heal_pct` only when set (a row without it re-saves without the key) and keeps a set one.
#[test]
fn the_writer_stores_heal_pct_only_when_set() {
    let row = |extra: Value| {
        let mut r = json!({"id": "feed", "name": "Feed", "kind": "SUPPORT", "level": 1, "blocks": 1, "description": "", "target_mode": "SELF", "heal_base": 2});
        for (k, v) in extra.as_object().unwrap() {
            r[k] = v.clone();
        }
        r.as_object().cloned().unwrap()
    };
    let stored = store::catalog_storage("skills", &row(json!({})), &[]).unwrap();
    assert!(!stored.contains_key("heal_pct"), "{stored:?}");
    let stored = store::catalog_storage("skills", &row(json!({"heal_pct": 0})), &[]).unwrap();
    assert!(!stored.contains_key("heal_pct"), "0 is no key");
    let stored = store::catalog_storage("skills", &row(json!({"heal_pct": 4})), &[]).unwrap();
    assert_eq!(json::get(&stored, "heal_pct"), &json!(4));
}

/// The `{heal}` token: the formula as before, then the percent of max HP; a percent alone; nothing without either.
#[test]
fn the_heal_token_names_the_percent() {
    let row = |v: Value| v.as_object().cloned().unwrap();
    let render = |r: Value| source::render("{heal}", "SUPPORT", &row(r), 1, true);
    assert_eq!(render(json!({"heal_base": 10})), "10", "unchanged without heal_pct");
    assert_eq!(render(json!({"heal_pct": 4})), "4% of max HP");
    assert_eq!(render(json!({"heal_base": 10, "heal_pct": 4})), "10 + 4% of max HP");
    assert_eq!(render(json!({})), "");
}
