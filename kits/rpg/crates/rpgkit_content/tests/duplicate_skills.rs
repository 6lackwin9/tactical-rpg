//! PK2-FOLLOWUP (PR5-36, FB-214 (e); `docs/sessions/PK2_GRAFTS.md` 5) on scratch data built here (no shipped data): a
//! unit owns each skill once across its class path, its `extra_skills` and the techniques (and keystone) of its class's
//! masterable families. A class whose path holds a technique of a family it may master is refused on the class row; a
//! derived unit whose extra skills repeat a path skill or such a technique is refused on the unit row (a fixed unit owns
//! its extra skills alone; skills are compared by family, so any rank of a path family counts). A technique of a family
//! the class cannot master (proficiency 1) is no clash. The writer refuses such a row.

use aegis_core::json::{self, Dict};
use aegis_core::scratch::Scratch;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn write(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn attack(id: &str) -> Value {
    json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0})
}

fn weapon_family(id: &str, extra: Value) -> Value {
    let mut f = json!({"id": id, "name": id, "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"], "damage_type": "slash",
        "base": {"main_1h": {"power": 10, "range": 1, "burden": 0, "mitigation": 0}}});
    for (k, x) in extra.as_object().unwrap() {
        f[k] = x.clone();
    }
    f
}

fn class(id: &str, path: Value) -> Value {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    json!({"id": id, "name": id, "base_stats": stats, "growth": stats, "skill_path": path, "proficiency": {"f_main": {"level": 3}, "f_minor": {"level": 1}}})
}

/// Skills: `p_one` (a path skill), `x_free`, technique heads `t_main`, `t_key` (family `f_main`'s technique and
/// keystone) and `t_minor` (family `f_minor`'s, proficiency 1 in every class here).
fn scratch(tag: &str) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_dupskills_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    write(&data, "skills", 2, json!([attack("p_one"), attack("x_free"), attack("t_main"), attack("t_key"), attack("t_minor")]));
    write(&data, "item_families", 1, json!([
        weapon_family("f_main", json!({"techniques": [{"skill": "t_main"}], "keystone": "t_key"})),
        weapon_family("f_minor", json!({"techniques": [{"skill": "t_minor"}]}))
    ]));
    write(&data, "items", 2, json!([]));
    write(&data, "classes", 1, json!([class("c_clean", json!({"p_one": 1})), class("c_clash", json!({"p_one": 1, "t_main": 2})), class("c_minor", json!({"p_one": 1, "t_minor": 2}))]));
    (t, data)
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

fn dup_errors(data: &Path, kind: &str, row: Value) -> Vec<String> {
    let v = store::view(data);
    let id = json::s(json::dict(&row), "id");
    check::validate_entry(&v, kind, &dict(row), &id).into_iter().filter(|e| e.contains("a unit owns each skill once")).collect()
}

fn unit(extra: &[&str]) -> Value {
    json!({"id": "u_one", "name": "One", "side": "party", "class_id": "c_clean", "level": 1, "extra_skills": extra})
}

#[test]
fn a_class_path_holding_a_technique_it_may_master_is_refused() {
    let (_t, data) = scratch("class");
    let v = store::view(&data);
    let class_row = |id: &str| v.get("classes", id).clone();
    assert!(dup_errors(&data, "classes", Value::Object(class_row("c_clean"))).is_empty());
    assert_eq!(
        dup_errors(&data, "classes", Value::Object(class_row("c_clash"))),
        ["Skill 't_main' is in both the class path and a technique (family 't_main'); a unit owns each skill once."]
    );
    assert!(dup_errors(&data, "classes", Value::Object(class_row("c_minor"))).is_empty(), "a family at proficiency 1 is not masterable: no technique owned");
}

#[test]
fn a_unit_whose_extra_skills_repeat_its_path_or_a_technique_is_refused() {
    let (_t, data) = scratch("unit");
    assert!(dup_errors(&data, "units", unit(&["x_free"])).is_empty());
    assert_eq!(dup_errors(&data, "units", unit(&["p_one"])), ["Skill 'p_one' is in both the extra skills and the class path (family 'p_one'); a unit owns each skill once."]);
    assert_eq!(dup_errors(&data, "units", unit(&["t_key"])), ["Skill 't_key' is in both the extra skills and a technique (family 't_key'); a unit owns each skill once."], "the family keystone counts");
    assert!(dup_errors(&data, "units", unit(&["t_minor"])).is_empty(), "a technique of a family the class cannot master");
    let mut fixed = unit(&["p_one"]);
    fixed["stat_mode"] = json!("fixed");
    assert!(dup_errors(&data, "units", fixed).is_empty(), "a fixed unit owns its extra skills alone (no class path)");
    let refused = store::upsert(&data, None, "units", &dict(unit(&["t_main"])), None).err().expect("the writer refuses the clash");
    assert!(refused.iter().any(|e| e.contains("'t_main' is in both the extra skills and a technique")), "{refused:?}");
}

/// AUDIT-05 (PR5-40): a world manifest's class overlay that replaces a class's path (`classes.<id>.skill_path`) is
/// checked like the class row: an overlay path holding a technique the class may master is refused on the manifest
/// (`classes_errors`), a clean overlay path or one holding a technique of a family at proficiency 1 is not.
#[test]
fn a_world_class_overlay_path_holding_a_technique_it_may_master_is_refused() {
    let (_t, data) = scratch("overlay");
    let v = store::view(&data);
    let overlay = |path: Value| dict(json!({"id": "w", "classes": {"c_clean": {"name": "Clean", "skill_path": path}}}));
    let dups = |path: Value| check::hooks::classes_errors(&v, &overlay(path)).into_iter().filter(|e| e.contains("a unit owns each skill once")).collect::<Vec<_>>();
    assert!(dups(json!({"p_one": 1})).is_empty(), "the class's own clean path");
    assert!(dups(json!({})).is_empty(), "no path");
    assert_eq!(
        dups(json!({"p_one": 1, "t_key": 3})),
        ["Class names 'c_clean': Skill 't_key' is in both the class path and a technique (family 't_key'); a unit owns each skill once."]
    );
    assert!(dups(json!({"t_minor": 2})).is_empty(), "a family at proficiency 1 is not masterable");
}
