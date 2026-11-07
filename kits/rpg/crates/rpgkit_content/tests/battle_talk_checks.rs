//! ENCOUNTER2-C1 content side (docs/specs/ENCOUNTER2_C.md 5.2): the phase `talk` effect (keys `type, dialogue,
//! speaker`; one per phase; world encounters only; its dialogue resolves in the world and runs only set_flag / add_flag;
//! its speaker "" or a character there), a `round` trigger's `block` (0 to total_blocks; round 1 allowed), the `downed`
//! trigger (`unit` an encounter ref, `party` or `party:<id>`; `count` 1 to the units the name covers) and the `broken`
//! trigger (a world encounter needs the poise rule in its world's ruleset). One error per mistake; a row without them
//! checks as before. Scratch data folders only.

use aegis_core::json::Dict;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_battle_talk_{tag}"));
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

fn write_world(data: &Path, w: &str, manifest: Value, kinds: &Value) {
    let dir = data.join("worlds").join(w);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), manifest.to_string()).unwrap();
    for (kind, entries) in kinds.as_object().unwrap() {
        fs::write(dir.join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
    }
}

/// The catalog: a hero, a brute and a guard; the encounter `talky` (two guards, six blocks, six rounds) with one phase.
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
    write_kind(data, "units", 1, json!([unit("hero", "party"), unit("mate", "party"), unit("brute", "enemy"), unit("guard", "enemy")]));
    write_kind(data, "derived_stats", 1, json!([{"id": "max_poise", "name": "Max Poise", "formula": "CONS * 2"}]));
    write_kind(data, "rulesets", 1, json!([{"id": "timeline", "name": "Timeline", "rules": ["poise"], "poise_stat": "max_poise"}]));
    let talky = json!({"id": "talky", "name": "Talky", "party": ["hero", "mate"], "enemies": ["brute", "guard", "guard"], "max_rounds": 6, "total_blocks": 6,
        "phases": [{"id": "turn", "name": "Turn", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "line", "text": "Now."}]}]});
    write_kind(data, "encounters", 1, json!([talky]));
}

/// The catalog encounter's errors with its phase's `trigger` and `effects` replaced (null keeps the row's).
fn catalog_errors(v: &check::View, trigger: Value, effects: Value) -> Vec<String> {
    let mut e: Dict = v.get("encounters", "talky").clone();
    let p = e.get_mut("phases").and_then(Value::as_array_mut).unwrap()[0].as_object_mut().unwrap();
    if !trigger.is_null() {
        p.insert("trigger".into(), trigger);
    }
    if !effects.is_null() {
        p.insert("effects".into(), effects);
    }
    check::validate_entry(v, "encounters", &e, "talky")
}

fn one(errors: Vec<String>, want: &str) {
    assert_eq!(errors, vec![want.to_string()]);
}

fn line() -> Value {
    json!([{"type": "line", "text": "Now."}])
}

#[test]
fn round_blocks_and_event_triggers_check_in_a_catalog_encounter() {
    let (_scratch, data) = scratch("catalog");
    catalog(&data);
    let v = store::view(&data);
    let clean = Vec::<String>::new();
    assert_eq!(catalog_errors(&v, Value::Null, Value::Null), clean, "the row as written");
    for trigger in [
        json!({"kind": "round", "round": 1}),
        json!({"kind": "round", "round": 1, "block": 0}),
        json!({"kind": "round", "round": 3, "block": 6}),
        json!({"kind": "downed", "unit": "guard", "count": 2}),
        json!({"kind": "downed", "unit": "brute"}),
        json!({"kind": "downed", "unit": "party", "count": 2}),
        json!({"kind": "downed", "unit": "party:mate"}),
        json!({"kind": "broken", "unit": "guard"}),
        json!({"kind": "broken", "unit": "party:hero"}),
    ] {
        assert_eq!(catalog_errors(&v, trigger.clone(), Value::Null), clean, "{trigger}");
    }
    one(catalog_errors(&v, json!({"kind": "round", "round": 0}), Value::Null), "Phase 'turn': round must be 1..6.");
    // the hp_below or_round floor stays 2 (Godot's)
    one(catalog_errors(&v, json!({"kind": "hp_below", "unit": "brute", "pct": 50, "or_round": 1}), Value::Null), "Phase 'turn': or_round must be 2..6.");
    assert_eq!(catalog_errors(&v, json!({"kind": "hp_below", "unit": "brute", "pct": 50, "or_round": 2}), Value::Null), clean);
    one(catalog_errors(&v, json!({"kind": "round", "round": 2, "block": 7}), Value::Null), "Phase 'turn': block must be 0..6.");
    one(catalog_errors(&v, json!({"kind": "round", "round": 2, "block": -1}), Value::Null), "Phase 'turn': block must be 0..6.");
    one(catalog_errors(&v, json!({"kind": "downed", "unit": "ghost"}), Value::Null), "Phase 'turn': trigger unit 'ghost' is not in this encounter.");
    one(catalog_errors(&v, json!({"kind": "downed", "unit": "guard#2"}), Value::Null), "Phase 'turn': trigger unit 'guard#2' is not in this encounter.");
    one(catalog_errors(&v, json!({"kind": "downed", "unit": "guard", "count": 3}), Value::Null), "Phase 'turn': count must be 1..2.");
    one(catalog_errors(&v, json!({"kind": "downed", "unit": "party", "count": 0}), Value::Null), "Phase 'turn': count must be 1..2.");
    one(catalog_errors(&v, json!({"kind": "downed", "unit": "party:ghost"}), Value::Null), "Phase 'turn': trigger hero 'ghost' is not in this encounter's party.");
    one(catalog_errors(&v, json!({"kind": "broken", "unit": "ghost"}), Value::Null), "Phase 'turn': trigger unit 'ghost' is not in this encounter.");
    one(catalog_errors(&v, json!({"kind": "broken", "unit": "guard", "count": 1}), Value::Null), "Phase 'turn': unknown trigger key 'count'.");
    one(catalog_errors(&v, json!({"kind": "downed", "unit": "guard", "pct": 5}), Value::Null), "Phase 'turn': unknown trigger key 'pct'.");
    // a talk is a world encounter's
    let talk = json!([{"type": "line", "text": "Now."}, {"type": "talk", "dialogue": "dlg_ok", "speaker": ""}]);
    one(catalog_errors(&v, Value::Null, talk), "A talk needs a world encounter (dialogue trees live in worlds).");
    drop_scratch(&data);
}

/// Two worlds over the catalog: `vale` (classic) and `ridge` (the timeline row: rule poise), each with the hero and the
/// mate, a brute and a guard, the trees `dlg_ok` (set_flag on a node, add_flag on a choice) and `dlg_bad` (a give on
/// node `a`, a start_quest on a choice of node `b`), and the encounter `talky` with one phase.
fn worlds(data: &Path) {
    let ch = |id: &str, role: &str, base: &str| json!({"id": id, "name": id, "role": role, "base_unit": base});
    let node = |id: &str, extra: Value| {
        let mut n = json!({"id": id, "speaker": "hero_c", "text": "Steady."});
        for (k, v) in extra.as_object().unwrap() {
            n[k] = v.clone();
        }
        n
    };
    let tree = |id: &str, nodes: Value| json!({"id": id, "start": "a", "services": false, "speaker": "hero_c", "nodes": nodes});
    let ok = tree("dlg_ok", json!([
        node("a", json!({"effects": [{"effect": "set_flag", "key": "met", "value": true}], "next": "b"})),
        node("b", json!({"choices": [{"text": "Go.", "to": "", "effects": [{"effect": "add_flag", "key": "nerve", "delta": 1}]}]}))
    ]));
    let bad = tree("dlg_bad", json!([
        node("a", json!({"effects": [{"effect": "give", "gold": 5}], "next": "b"})),
        node("b", json!({"choices": [{"text": "Go.", "to": "", "effects": [{"effect": "start_quest", "quest": "q"}]}]}))
    ]));
    let talky = json!({"id": "talky", "name": "Talky", "party": ["hero_c", "mate_c"], "enemies": ["brute_c", "guard_c", "guard_c"], "max_rounds": 6, "total_blocks": 6,
        "phases": [{"id": "turn", "name": "Turn", "trigger": {"kind": "round", "round": 1}, "effects": [{"type": "talk", "dialogue": "dlg_ok", "speaker": "hero_c"}]}]});
    let kinds = json!({
        "characters": [ch("hero_c", "party", "hero"), ch("mate_c", "party", "mate"), ch("brute_c", "enemy", "brute"), ch("guard_c", "enemy", "guard")],
        "dialogues": [ok, bad],
        "encounters": [talky],
    });
    write_world(data, "vale", json!({"id": "vale", "name": "Vale", "version": 1}), &kinds);
    write_world(data, "ridge", json!({"id": "ridge", "name": "Ridge", "ruleset": "timeline", "version": 1}), &kinds);
}

/// World `w`'s encounter's errors with its phase's `trigger` and `effects` replaced (null keeps the row's).
fn world_errors(v: &check::View, w: &str, trigger: Value, effects: Value) -> Vec<String> {
    let mut e: Dict = v.worlds.own_entries(w, "encounters")[0].clone();
    let p = e.get_mut("phases").and_then(Value::as_array_mut).unwrap()[0].as_object_mut().unwrap();
    if !trigger.is_null() {
        p.insert("trigger".into(), trigger);
    }
    if !effects.is_null() {
        p.insert("effects".into(), effects);
    }
    check::world::validate_entry(v, w, "encounters", &e, "talky")
}

fn talk(dialogue: &str, speaker: &str) -> Value {
    json!({"type": "talk", "dialogue": dialogue, "speaker": speaker})
}

#[test]
fn a_world_talk_resolves_its_tree_and_speaker_and_runs_only_flags() {
    let (_scratch, data) = scratch("world");
    catalog(&data);
    worlds(&data);
    let v = store::view(&data);
    let clean = Vec::<String>::new();
    assert_eq!(world_errors(&v, "vale", Value::Null, Value::Null), clean, "a round-1 talk with a speaker");
    assert_eq!(world_errors(&v, "vale", Value::Null, json!([talk("dlg_ok", "")])), clean, "no speaker");
    assert_eq!(world_errors(&v, "vale", Value::Null, json!([{"type": "line", "text": "Now."}, talk("dlg_ok", ""), {"type": "shake", "preset": "jolt"}])), clean, "among other effects");
    one(world_errors(&v, "vale", Value::Null, json!([talk("dlg_none", "")])), "Unknown dialogue 'dlg_none'.");
    one(world_errors(&v, "vale", Value::Null, json!([talk("", "")])), "Phase 'turn': a talk needs a dialogue.");
    one(world_errors(&v, "vale", Value::Null, json!([talk("dlg_ok", ""), talk("dlg_ok", "")])), "Phase 'turn': one talk per phase.");
    one(world_errors(&v, "vale", Value::Null, json!([{"type": "talk", "dialogue": "dlg_ok", "speaker": "", "line": "x"}])), "Phase 'turn': unknown talk key 'line'.");
    let speaker = world_errors(&v, "vale", Value::Null, json!([talk("dlg_ok", "nobody")]));
    assert!(speaker.len() == 1 && speaker[0].starts_with("Phase 'turn': talk speaker") && speaker[0].contains("nobody"), "{speaker:?}");
    assert_eq!(
        world_errors(&v, "vale", Value::Null, json!([talk("dlg_bad", "")])),
        ["Battle talk 'dlg_bad' node 'a': effect 'give' cannot run in a battle.", "Battle talk 'dlg_bad' node 'b': effect 'start_quest' cannot run in a battle."]
    );
    drop_scratch(&data);
}

#[test]
fn a_world_broken_trigger_needs_the_poise_rule() {
    let (_scratch, data) = scratch("broken");
    catalog(&data);
    worlds(&data);
    let v = store::view(&data);
    let broken = json!({"kind": "broken", "unit": "guard_c"});
    one(world_errors(&v, "vale", broken.clone(), line()), "A broken trigger needs the poise rule.");
    assert_eq!(world_errors(&v, "ridge", broken, line()), Vec::<String>::new(), "the timeline row lists poise");
    assert_eq!(world_errors(&v, "vale", json!({"kind": "downed", "unit": "party:mate_c"}), line()), Vec::<String>::new(), "downed needs no rule");
    assert_eq!(world_errors(&v, "vale", json!({"kind": "downed", "unit": "guard_c", "count": 2}), line()), Vec::<String>::new());
    one(world_errors(&v, "vale", json!({"kind": "downed", "unit": "guard_c", "count": 3}), line()), "Phase 'turn': count must be 1..2.");
    drop_scratch(&data);
}
