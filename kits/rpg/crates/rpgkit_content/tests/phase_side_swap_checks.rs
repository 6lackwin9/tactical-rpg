//! PHASEFX1 content side (docs/specs/ACTS23.md PK6 hooks l and m): the phase `side` effect (keys `type, units, to`; 1-6
//! refs, each an enemy, a phase-wave ref or a guest, never a hero, a protected guest or the slay target; `to`
//! hostile_all, party or enemy) and the phase `swap` effect (keys `type, unit, to, hp`; `unit` an enemy, a phase-wave ref or a guest, never a
//! hero; `to` a unit row of the swapped unit's side, a guest's being the party side; `hp` keep_pct or full): one error
//! per mistake, in catalog and world encounters alike. Scratch data folders only.

use aegis_core::json::Dict;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_phase_side_swap_{tag}"));
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

/// The turn: a hero against a founder and an ogre, an imp arriving as a phase wave, a captive guest and a protected
/// ward; `turn` turns the founder and the imp on everyone and swaps the ogre into the beast.
fn turn() -> Value {
    json!({"id": "turn", "name": "Turn", "party": ["hero"], "enemies": ["founder", "ogre"], "guests": ["captive", "ward"], "protect": ["ward"], "max_rounds": 6,
        "phases": [
            {"id": "late", "name": "Late", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": ["imp"]}]},
            {"id": "turn", "name": "Turn", "trigger": {"kind": "hp_below", "unit": "ogre", "pct": 50},
                "effects": [{"type": "side", "units": ["founder", "imp"], "to": "hostile_all"}, {"type": "swap", "unit": "ogre", "to": "beast", "hp": "keep_pct"}]}
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
    write_kind(
        data,
        "units",
        1,
        json!([unit("hero", "party"), unit("captive", "party"), unit("ward", "party"), unit("pet", "party"), unit("founder", "enemy"), unit("ogre", "enemy"), unit("imp", "enemy"), unit("beast", "enemy")]),
    );
    write_kind(data, "encounters", 1, json!([turn()]));
}

/// The errors of `turn` with its `turn` phase's effect of `kind` changed by `edit` (merged over it; null removes a key).
fn effect_errors(v: &check::View, kind: &str, edit: Value) -> Vec<String> {
    let mut e: Dict = v.get("encounters", "turn").clone();
    let phases = e.get_mut("phases").and_then(Value::as_array_mut).unwrap();
    let p = phases.iter_mut().find(|p| p["id"] == "turn").unwrap();
    let fx = p["effects"].as_array_mut().unwrap().iter_mut().find(|fx| fx["type"] == kind).unwrap().as_object_mut().unwrap();
    for (k, val) in edit.as_object().unwrap() {
        if val.is_null() {
            fx.shift_remove(k);
        } else {
            fx.insert(k.clone(), val.clone());
        }
    }
    check::validate_entry(v, "encounters", &e, "turn")
}

fn one(errors: Vec<String>, want: &str) {
    assert_eq!(errors, vec![want.to_string()]);
}

#[test]
fn a_valid_turn_checks_clean() {
    let (_scratch, data) = scratch("valid");
    catalog(&data);
    let v = store::view(&data);
    assert_eq!(effect_errors(&v, "side", json!({})), Vec::<String>::new(), "the turn");
    for to in ["party", "enemy", "hostile_all"] {
        assert_eq!(effect_errors(&v, "side", json!({"to": to})), Vec::<String>::new(), "side to {to}");
    }
    assert_eq!(effect_errors(&v, "side", json!({"units": ["captive"]})), Vec::<String>::new(), "a guest may turn");
    assert_eq!(effect_errors(&v, "swap", json!({"hp": null})), Vec::<String>::new(), "hp defaults to keep_pct");
    assert_eq!(effect_errors(&v, "swap", json!({"hp": "full"})), Vec::<String>::new(), "hp full");
    assert_eq!(effect_errors(&v, "swap", json!({"unit": "imp"})), Vec::<String>::new(), "a phase-wave unit may swap");
    assert_eq!(effect_errors(&v, "swap", json!({"unit": "captive", "to": "pet"})), Vec::<String>::new(), "a guest swaps onto a party row");
    drop_scratch(&data);
}

#[test]
fn each_side_mistake_is_one_error() {
    let (_scratch, data) = scratch("side");
    catalog(&data);
    let v = store::view(&data);
    one(effect_errors(&v, "side", json!({"units": ["founder", "hero"]})), "Side names hero 'hero'.");
    one(effect_errors(&v, "side", json!({"units": ["ward"]})), "Side names the protected guest 'ward'.");
    one(effect_errors(&v, "side", json!({"units": ["founder", "ghost"]})), "Phase 'turn': side unit 'ghost' is not in this encounter.");
    one(effect_errors(&v, "side", json!({"units": []})), "Phase 'turn': a side names 1 to 6 units.");
    one(effect_errors(&v, "side", json!({"units": vec!["founder"; 7]})), "Phase 'turn': a side names 1 to 6 units.");
    one(effect_errors(&v, "side", json!({"to": "neutral"})), "Phase 'turn': side to 'neutral' must be one of hostile_all, party, enemy.");
    one(effect_errors(&v, "side", json!({"to": null})), "Phase 'turn': side to '<null>' must be one of hostile_all, party, enemy.");
    one(effect_errors(&v, "side", json!({"by": "ogre"})), "Phase 'turn': unknown side key 'by'.");
    drop_scratch(&data);
}

#[test]
fn a_side_never_turns_the_slay_target() {
    let (_scratch, data) = scratch("slay");
    catalog(&data);
    let v = store::view(&data);
    let mut e: Dict = v.get("encounters", "turn").clone();
    e.insert("objective".into(), json!("slay"));
    e.insert("objective_unit".into(), json!("founder"));
    assert_eq!(check::validate_entry(&v, "encounters", &e, "turn"), vec!["Side names the slay target 'founder'.".to_string()]);
    e.insert("objective_unit".into(), json!("ogre"));
    assert_eq!(check::validate_entry(&v, "encounters", &e, "turn"), Vec::<String>::new(), "another slay target");
    drop_scratch(&data);
}

#[test]
fn each_swap_mistake_is_one_error() {
    let (_scratch, data) = scratch("swap");
    catalog(&data);
    let v = store::view(&data);
    one(effect_errors(&v, "swap", json!({"unit": "hero"})), "Swap names hero 'hero'.");
    one(effect_errors(&v, "swap", json!({"unit": "ghost"})), "Phase 'turn': swap unit 'ghost' is not in this encounter.");
    one(effect_errors(&v, "swap", json!({"to": "ghost"})), "Phase 'turn': swap to 'ghost' is not a unit.");
    one(effect_errors(&v, "swap", json!({"to": null})), "Phase 'turn': swap to '<null>' is not a unit.");
    one(effect_errors(&v, "swap", json!({"to": "pet"})), "Phase 'turn': swap to 'pet' is on the party side; 'ogre' fights on the enemy side.");
    one(effect_errors(&v, "swap", json!({"unit": "captive"})), "Phase 'turn': swap to 'beast' is on the enemy side; 'captive' fights on the party side.");
    one(effect_errors(&v, "swap", json!({"hp": "half"})), "Phase 'turn': swap hp must be keep_pct or full.");
    one(effect_errors(&v, "swap", json!({"units": ["ogre"]})), "Phase 'turn': unknown swap key 'units'.");
    drop_scratch(&data);
}

#[test]
fn a_world_encounter_checks_its_sides_and_swaps_too() {
    let (_scratch, data) = scratch("world");
    catalog(&data);
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    let ch = |id: &str, base: &str, role: &str| json!({"id": id, "name": id, "base_unit": base, "role": role});
    let chars = json!([ch("brute", "ogre", "enemy"), ch("monster", "beast", "enemy"), ch("friend", "pet", "party")]);
    fs::write(dir.join("characters.json"), json!({"kind": "characters", "version": 1, "entries": chars}).to_string()).unwrap();
    let enc = |id: &str, fx: Value| {
        json!({"id": id, "name": id, "party": ["hero"], "enemies": ["brute"], "max_rounds": 6,
            "phases": [{"id": "turn", "name": "Turn", "trigger": {"kind": "round", "round": 2}, "effects": [fx]}]})
    };
    let entries = json!([
        enc("good", json!({"type": "swap", "unit": "brute", "to": "monster"})),
        enc("good_side", json!({"type": "side", "units": ["brute"], "to": "hostile_all"})),
        enc("bad_side", json!({"type": "swap", "unit": "brute", "to": "friend"})),
        enc("bad_to", json!({"type": "swap", "unit": "brute", "to": "nobody"})),
        enc("bad_turn", json!({"type": "side", "units": ["brute"], "to": "everyone"})),
    ]);
    fs::write(dir.join("encounters.json"), json!({"kind": "encounters", "version": 1, "entries": entries}).to_string()).unwrap();
    let problems: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains("swap to") || p.contains("side to")).collect();
    assert_eq!(problems.len(), 3, "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("bad_side") && p.contains("swap to 'friend' is on the party side; 'brute' fights on the enemy side.")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("bad_to") && p.contains("swap to 'nobody' is not a unit.")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("bad_turn") && p.contains("side to 'everyone' must be one of")), "{problems:?}");
    drop_scratch(&data);
}
