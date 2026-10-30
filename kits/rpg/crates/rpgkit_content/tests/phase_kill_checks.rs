//! ENCOUNTER2-K content side (docs/specs/ENCOUNTER2.md section 6, P1, P2, P5): the phase `kill` effect (keys `type,
//! units, by, within`; 1-6 refs, each an enemy, a phase-wave ref or a guest, never a hero or a protected guest; `by` ""
//! or a unit of the encounter; `within` 0-11 and above 0 only with a `by`), the phase key `group` (`[a-z0-9_]{1,24}`) and
//! an hp_below trigger on a phase-wave ref: one error per mistake, in catalog and world encounters alike; a row without
//! them checks as before. Scratch data folders only.

use aegis_core::json::Dict;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_phase_kill_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// The cull: a hero against a brute and an ogre, an imp arriving as a phase wave, a captive guest and a protected
/// ward; `break` kills the brute, the imp and the captive near the ogre, and shares the group `break` with `early`.
fn cull() -> Value {
    json!({"id": "cull", "name": "Cull", "party": ["hero"], "enemies": ["brute", "ogre"], "guests": ["captive", "ward"], "protect": ["ward"], "max_rounds": 6,
        "phases": [
            {"id": "late", "name": "Late", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": ["imp"]}]},
            {"id": "break", "name": "Break", "group": "break", "trigger": {"kind": "hp_below", "unit": "ogre", "pct": 50, "or_round": 4},
                "effects": [{"type": "kill", "units": ["brute", "imp", "captive"], "by": "ogre", "within": 3}]},
            {"id": "early", "name": "Early", "group": "break", "trigger": {"kind": "hp_below", "unit": "imp", "pct": 40},
                "effects": [{"type": "kill", "units": ["brute"]}]}
        ]})
}

fn catalog(data: &Path) {
    let skill = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(data, "skills", 2, json!([skill("move", "MOVE", json!({"system": true})), skill("rest", "REST", json!({"system": true})), skill("strike", "ATTACK", json!({"basic": true}))]));
    write_kind(data, "classes", 1, json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2}}]));
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "class_id": "drill", "level": 1});
    write_kind(data, "units", 1, json!([unit("hero", "party"), unit("captive", "party"), unit("ward", "party"), unit("brute", "enemy"), unit("ogre", "enemy"), unit("imp", "enemy")]));
    write_kind(data, "encounters", 1, json!([cull()]));
}

/// The errors of `cull` with the phase `id` changed by `edit` (merged over the phase; a key set to null is removed).
fn phase_errors(v: &check::View, id: &str, edit: Value) -> Vec<String> {
    let mut e: Dict = v.get("encounters", "cull").clone();
    let phases = e.get_mut("phases").and_then(Value::as_array_mut).unwrap();
    let p = phases.iter_mut().find(|p| p["id"] == id).unwrap().as_object_mut().unwrap();
    for (k, val) in edit.as_object().unwrap() {
        if val.is_null() {
            p.shift_remove(k);
        } else {
            p.insert(k.clone(), val.clone());
        }
    }
    check::validate_entry(v, "encounters", &e, "cull")
}

/// The errors of `cull` with `kill` merged over the break's kill effect.
fn kill_errors(v: &check::View, kill: Value) -> Vec<String> {
    let mut fx = json!({"type": "kill", "units": ["brute", "imp", "captive"], "by": "ogre", "within": 3});
    for (k, val) in kill.as_object().unwrap() {
        if val.is_null() {
            fx.as_object_mut().unwrap().shift_remove(k);
        } else {
            fx[k] = val.clone();
        }
    }
    phase_errors(v, "break", json!({"effects": [fx]}))
}

fn one(errors: Vec<String>, want: &str) {
    assert_eq!(errors, vec![want.to_string()]);
}

#[test]
fn a_valid_cull_and_a_row_without_the_keys_check_clean() {
    let data = scratch("valid");
    catalog(&data);
    let v = store::view(&data);
    assert_eq!(phase_errors(&v, "break", json!({})), Vec::<String>::new(), "the cull");
    assert_eq!(kill_errors(&v, json!({"by": null, "within": null})), Vec::<String>::new(), "a kill without by and within");
    assert_eq!(kill_errors(&v, json!({"by": "", "within": 0})), Vec::<String>::new(), "by empty, anywhere");
    assert_eq!(kill_errors(&v, json!({"by": "hero", "within": 11})), Vec::<String>::new(), "any unit of the encounter may be the by unit");
    assert_eq!(phase_errors(&v, "break", json!({"group": null})), Vec::<String>::new(), "no group");
    assert_eq!(phase_errors(&v, "break", json!({"group": "a_1"})), Vec::<String>::new(), "another group");
    assert_eq!(phase_errors(&v, "break", json!({"group": "g".repeat(24)})), Vec::<String>::new(), "24 characters");
    drop_scratch(&data);
}

#[test]
fn each_kill_mistake_is_one_error() {
    let data = scratch("kill");
    catalog(&data);
    let v = store::view(&data);
    one(kill_errors(&v, json!({"units": ["brute", "hero"]})), "Kill names hero 'hero'.");
    one(kill_errors(&v, json!({"units": ["ward"]})), "Kill names the protected guest 'ward'.");
    one(kill_errors(&v, json!({"units": vec!["brute"; 7]})), "Phase 'break': a kill names 1 to 6 units.");
    one(kill_errors(&v, json!({"units": []})), "Phase 'break': a kill names 1 to 6 units.");
    one(kill_errors(&v, json!({"units": ["brute", "ghost"]})), "Phase 'break': kill unit 'ghost' is not in this encounter.");
    one(kill_errors(&v, json!({"by": "ghost"})), "Phase 'break': kill by 'ghost' is not in this encounter.");
    one(kill_errors(&v, json!({"within": 12})), "Phase 'break': within must be 0..11.");
    one(kill_errors(&v, json!({"within": -1})), "Phase 'break': within must be 0..11.");
    one(kill_errors(&v, json!({"within": 2, "by": ""})), "Phase 'break': a kill within tiles needs a by unit.");
    one(kill_errors(&v, json!({"rewards": false})), "Phase 'break': unknown kill key 'rewards'.");
    drop_scratch(&data);
}

#[test]
fn each_group_mistake_is_one_error_and_hp_below_may_watch_a_wave_unit() {
    let data = scratch("group");
    catalog(&data);
    let v = store::view(&data);
    let want = "Phase 'break': the group is 1 to 24 characters a-z, 0-9 or _.";
    for bad in [json!("Break!"), json!(""), json!("g".repeat(25)), json!(5)] {
        one(phase_errors(&v, "break", json!({"group": bad})), want);
    }
    // P5: `early` watches the imp, a phase-wave ref (clean above); a ref outside the encounter keeps Godot's message
    one(phase_errors(&v, "early", json!({"trigger": {"kind": "hp_below", "unit": "ghost", "pct": 40}})), "Phase 'early': trigger unit 'ghost' is not in enemies.");
    drop_scratch(&data);
}

#[test]
fn a_world_encounter_checks_its_kills_too() {
    let data = scratch("world");
    catalog(&data);
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    let enc = |id: &str, units: Value| {
        json!({"id": id, "name": id, "party": ["hero"], "enemies": ["brute"], "max_rounds": 6,
            "phases": [{"id": "cull", "name": "Cull", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "kill", "units": units}]}]})
    };
    let entries = json!([enc("good", json!(["brute"])), enc("bad", json!(["hero"]))]);
    fs::write(dir.join("encounters.json"), json!({"kind": "encounters", "version": 1, "entries": entries}).to_string()).unwrap();
    let problems: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains("Kill names") || p.contains("kill unit") || p.contains("kill by")).collect();
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("bad") && problems[0].contains("Kill names hero 'hero'."), "{problems:?}");
    drop_scratch(&data);
}
