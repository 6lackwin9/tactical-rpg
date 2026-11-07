//! ENCOUNTER2-C3 content side (docs/specs/ENCOUNTER2_C.md 5.1, 5.3): an encounter's `script` is empty or 1-3 round
//! scripts `{round, lock, others, banner, entries}` and each entry `{unit, target, picks, fill}` names its unit as SN1
//! (`ref`, `ref#n`, `party:<id>`, `party#n`), its target as SN3 and its picks as SN4. Each mistake gives its one error
//! (the errors a row has without the script are taken off first); a catalog encounter's script warns that it plays
//! only under the timeline ruleset; a world encounter's refs are characters (an enemy's skills are its base unit's).
//! Scratch data folders only.

use aegis_core::json::Dict;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_battle_script_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// The catalog: two heroes, a brute (it owns `cleave` only, plus the basic `strike`) and a guard (no skill list: it
/// knows every skill, as in battle); the encounter `scripted` (two guards, six rounds) with no script.
fn catalog(data: &Path) {
    let skill = |id: &str, kind: &str, extra: Value| {
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
        json!([skill("move", "MOVE", json!({"system": true})), skill("rest", "REST", json!({"system": true})), skill("strike", "ATTACK", json!({"basic": true})), skill("cleave", "ATTACK", json!({})), skill("jab", "ATTACK", json!({}))]),
    );
    write_kind(data, "classes", 1, json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2}}]));
    let unit = |id: &str, side: &str, extra: Value| {
        let mut u = json!({"id": id, "name": id, "side": side, "class_id": "drill", "level": 1});
        for (k, v) in extra.as_object().unwrap() {
            u[k] = v.clone();
        }
        u
    };
    write_kind(data, "units", 1, json!([unit("hero", "party", json!({})), unit("mate", "party", json!({})), unit("brute", "enemy", json!({"extra_skills": ["cleave"]})), unit("guard", "enemy", json!({}))]));
    let row = json!({"id": "scripted", "name": "Scripted", "party": ["hero", "mate"], "enemies": ["brute", "guard", "guard"], "max_rounds": 6, "total_blocks": 6});
    write_kind(data, "encounters", 1, json!([row]));
}

fn entry(unit: &str, target: Value, picks: Value) -> Value {
    json!({"unit": unit, "target": target, "picks": picks, "fill": "auto"})
}

fn round(n: i64, entries: Value) -> Value {
    json!({"round": n, "lock": true, "others": "auto", "banner": "", "entries": entries})
}

/// A script every rule accepts: round 2 locked (ref#2, party#2, party:<id>, a tile target, a move first, a step after,
/// rest), round 3 guided with Rest for the others.
fn good_script() -> Value {
    json!([
        round(2, json!([
            entry("brute", json!("party:hero"), json!([{"skill": "cleave"}])),
            entry("guard#2", json!("self"), json!(["rest"])),
            entry("party:hero", json!("brute"), json!([{"move": [3, 4]}, {"skill": "jab"}, {"step_after": [2, 4]}])),
            entry("party#2", json!([5, 5]), json!([{"skill": "strike", "kind": "utility"}]))
        ])),
        {"round": 3, "lock": false, "others": "rest", "banner": "Hold the line", "entries": [entry("guard", json!("party#1"), json!([{"skill": "strike"}]))]}
    ])
}

/// The errors `e` gains when its `script` is `script` (its errors without one taken off).
fn added(v: &check::View, e: &Dict, script: Value, check: &dyn Fn(&check::View, &Dict) -> Vec<String>) -> Vec<String> {
    let mut without = e.clone();
    without.shift_remove("script");
    let base = check(v, &without);
    let mut with = e.clone();
    with.insert("script".into(), script);
    check(v, &with).into_iter().filter(|x| !base.contains(x)).collect()
}

fn catalog_check(v: &check::View, e: &Dict) -> Vec<String> {
    check::validate_entry(v, "encounters", e, "scripted")
}

fn one(errors: Vec<String>, want: &str) {
    assert_eq!(errors, vec![want.to_string()]);
}

#[test]
fn every_script_mistake_gives_its_error_in_a_catalog_encounter() {
    let (_scratch, data) = scratch("catalog");
    catalog(&data);
    let v = store::view(&data);
    let e: Dict = v.get("encounters", "scripted").clone();
    let clean = Vec::<String>::new();
    assert_eq!(catalog_check(&v, &e), clean, "the row as written");
    assert_eq!(added(&v, &e, json!([]), &catalog_check), clean, "an empty script");
    assert_eq!(added(&v, &e, good_script(), &catalog_check), clean, "{}", good_script());
    let r2 = |entries: Value| json!([round(2, entries)]);
    let ok = || entry("guard", json!("self"), json!(["rest"]));
    // the round scripts
    one(added(&v, &e, json!([round(1, json!([ok()])), round(2, json!([ok()])), round(3, json!([ok()])), round(4, json!([ok()]))]), &catalog_check), "At most 3 scripted rounds.");
    one(added(&v, &e, json!([round(2, json!([ok()])), round(2, json!([ok()]))]), &catalog_check), "Round 2 is scripted twice.");
    one(added(&v, &e, json!([round(7, json!([ok()]))]), &catalog_check), "Script round must be 1..6.");
    one(added(&v, &e, json!([round(0, json!([ok()]))]), &catalog_check), "Script round must be 1..6.");
    one(added(&v, &e, json!({"round": 2}), &catalog_check), "Script must be a list of round scripts.");
    one(added(&v, &e, json!([{"round": 2, "lock": "yes", "entries": [ok()]}]), &catalog_check), "Script lock must be true or false.");
    one(added(&v, &e, json!([{"round": 2, "others": "fight", "entries": [ok()]}]), &catalog_check), "Script others is auto or rest.");
    one(added(&v, &e, json!([{"round": 2, "banner": "x".repeat(41), "entries": [ok()]}]), &catalog_check), "A script banner is text of at most 40 characters.");
    assert_eq!(added(&v, &e, json!([{"round": 2, "banner": "x".repeat(40), "entries": [ok()]}]), &catalog_check), clean, "a 40-character banner");
    one(added(&v, &e, json!([{"round": 2, "entries": []}]), &catalog_check), "A round script holds 1 to 16 entries.");
    // 17 entries (of one unit: its repeat is told once)
    assert_eq!(added(&v, &e, json!([{"round": 2, "entries": vec![ok(); 17]}]), &catalog_check), ["A round script holds 1 to 16 entries.", "guard has two entries in round 2."]);
    one(added(&v, &e, json!([{"round": 2, "pace": 1, "entries": [ok()]}]), &catalog_check), "Script round: unknown key 'pace'.");
    // the entries' units and targets (SN1, SN3)
    one(added(&v, &e, r2(json!([entry("ghost", json!("self"), json!(["rest"]))])), &catalog_check), "Script unit 'ghost' is not in this encounter.");
    one(added(&v, &e, r2(json!([entry("guard#3", json!("self"), json!(["rest"]))])), &catalog_check), "Script unit 'guard#3' is not in this encounter.");
    one(added(&v, &e, r2(json!([entry("party#3", json!("self"), json!(["rest"]))])), &catalog_check), "Script unit 'party#3' is not in this encounter.");
    one(added(&v, &e, r2(json!([entry("party:ghost", json!("self"), json!(["rest"]))])), &catalog_check), "Script hero 'ghost' is not in this encounter's party.");
    one(added(&v, &e, r2(json!([entry("guard", json!("ghost"), json!([{"skill": "strike"}]))])), &catalog_check), "Script unit 'ghost' is not in this encounter.");
    one(added(&v, &e, r2(json!([entry("guard", json!([12, 0]), json!([{"skill": "strike"}]))])), &catalog_check), "Script tile [12, 0] is off the grid.");
    one(added(&v, &e, r2(json!([ok(), entry("guard#1", json!("self"), json!(["rest"]))])), &catalog_check), "guard#1 has two entries in round 2.");
    one(added(&v, &e, r2(json!([{"unit": "guard", "picks": ["rest"], "wait": 1}])), &catalog_check), "Script entry: unknown key 'wait'.");
    one(added(&v, &e, r2(json!([{"unit": "guard", "picks": ["rest"], "fill": "all"}])), &catalog_check), "Script fill is auto or rest.");
    // the picks (SN2, SN4)
    let picks_text = "Script picks are 1-4 of skill, item, rest, move, step_after.";
    for picks in [json!([]), json!(["rest", "rest", "rest", "rest", "rest"]), json!([{"spell": "jab"}]), json!([{"skill": "jab", "item": "x"}]), json!([{"skill": "jab", "kind": "charge"}]), json!(["wait"])] {
        one(added(&v, &e, r2(json!([entry("guard", json!("self"), picks.clone())])), &catalog_check), picks_text);
    }
    one(added(&v, &e, r2(json!([entry("guard", json!("self"), json!([{"skill": "nope"}]))])), &catalog_check), "Unknown skill 'nope'.");
    one(added(&v, &e, r2(json!([entry("brute", json!("party:hero"), json!([{"skill": "jab"}]))])), &catalog_check), "brute has no skill 'jab'.");
    assert_eq!(added(&v, &e, r2(json!([entry("brute", json!("party:hero"), json!([{"skill": "strike"}]))])), &catalog_check), clean, "a basic is every unit's");
    one(added(&v, &e, r2(json!([entry("party:hero", json!("self"), json!([{"item": "potion"}]))])), &catalog_check), "Unknown item 'potion'.");
    one(added(&v, &e, r2(json!([entry("party:hero", json!("brute"), json!([{"skill": "jab"}, {"move": [3, 3]}]))])), &catalog_check), "A move is the first pick.");
    one(added(&v, &e, r2(json!([entry("party:hero", json!("self"), json!([{"move": [3, 9]}]))])), &catalog_check), "Script tile [3, 9] is off the grid.");
    // 5.3: a catalog script plays only under the timeline ruleset
    let mut with = e.clone();
    with.insert("script".into(), good_script());
    assert!(check::warnings_for(&v, "encounters", &with).contains(&"Plays scripted only under the timeline ruleset.".to_string()));
    assert!(!check::warnings_for(&v, "encounters", &e).iter().any(|w| w.contains("scripted")));
    drop_scratch(&data);
}

/// ENCOUNTER2 section 6: a hold escape picks its holder before round 1, so a locked round 1 is refused (a guided round
/// 1 and a locked round 2 are not).
#[test]
fn a_hold_escape_refuses_a_locked_round_one() {
    let (_scratch, data) = scratch("hold");
    catalog(&data);
    let v = store::view(&data);
    let mut e: Dict = v.get("encounters", "scripted").clone();
    e.insert("objective".into(), json!("escape"));
    e.insert("stay_behind".into(), json!({"mode": "hold", "tiles": [[6, 4]], "tiles_label": "the gap"}));
    let r = |n: i64, lock: bool| json!([{"round": n, "lock": lock, "entries": [entry("guard", json!("self"), json!(["rest"]))]}]);
    one(added(&v, &e, r(1, true), &catalog_check), "A hold escape picks its holder before round 1: script round 2 or later.");
    assert_eq!(added(&v, &e, r(1, false), &catalog_check), Vec::<String>::new(), "a guided round 1");
    assert_eq!(added(&v, &e, r(2, true), &catalog_check), Vec::<String>::new(), "a locked round 2");
    drop_scratch(&data);
}

/// A world encounter's refs are characters: `party:<id>` names a party character, and an enemy's skills are its base
/// unit's.
#[test]
fn a_world_script_names_characters() {
    let (_scratch, data) = scratch("world");
    catalog(&data);
    let ch = |id: &str, role: &str, base: &str| json!({"id": id, "name": id, "role": role, "base_unit": base});
    let row = json!({"id": "scripted", "name": "Scripted", "party": ["hero_c", "mate_c"], "enemies": ["brute_c", "guard_c", "guard_c"], "max_rounds": 6, "total_blocks": 6});
    let kinds = json!({"characters": [ch("hero_c", "party", "hero"), ch("mate_c", "party", "mate"), ch("brute_c", "enemy", "brute"), ch("guard_c", "enemy", "guard")], "encounters": [row]});
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "version": 1}).to_string()).unwrap();
    for (kind, entries) in kinds.as_object().unwrap() {
        fs::write(dir.join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
    }
    let v = store::view(&data);
    let e: Dict = v.worlds.own_entries("vale", "encounters")[0].clone();
    let world_check = |v: &check::View, e: &Dict| check::world::validate_entry(v, "vale", "encounters", e, "scripted");
    let r2 = |entries: Value| json!([round(2, entries)]);
    assert_eq!(added(&v, &e, r2(json!([entry("party:hero_c", json!("guard_c#2"), json!([{"skill": "jab"}])), entry("brute_c", json!("party#2"), json!([{"skill": "cleave"}]))])), &world_check), Vec::<String>::new());
    one(added(&v, &e, r2(json!([entry("brute_c", json!("party:hero_c"), json!([{"skill": "jab"}]))])), &world_check), "brute_c has no skill 'jab'.");
    one(added(&v, &e, r2(json!([entry("party:hero", json!("self"), json!(["rest"]))])), &world_check), "Script hero 'hero' is not in this encounter's party.");
    assert!(!check::warnings_for(&v, "encounters", &e).iter().any(|w| w.contains("scripted")));
    drop_scratch(&data);
}
