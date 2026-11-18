//! ROBUST1 (ENGINE-10): property tests of save migration (`aegis_save::migrate`, `open`). Envelopes of every version
//! migrate to the current one through each step once, in order; unknown keys are dropped with a warning; a newer, a
//! missing or a broken version is refused; random text and random JSON are refused or opened, never a panic.

use aegis_core::json::Dict;
use aegis_core::rng::GodotRng;
use aegis_save::{migrate, open, Format};
use serde_json::{json, Value};

const FMT: Format = Format { tag: "prop_save", version: 6, first_slot: 0, last_slot: 3, keys: &["party", "steps", "v2", "v3", "v4", "v5", "v6"] };

/// Step `v` adds the key `v{v+1}` and counts itself in `steps`.
fn step(v: i64, state: &Dict, _w: &mut Vec<String>) -> Option<Dict> {
    let mut s = state.clone();
    s.insert(format!("v{}", v + 1), json!(true));
    let steps = s.get("steps").and_then(Value::as_array).cloned().unwrap_or_default();
    s.insert("steps".into(), Value::Array(steps.into_iter().chain([json!(v)]).collect()));
    Some(s)
}

fn envelope(version: Value, state: Value) -> Dict {
    json!({"format": FMT.tag, "version": version, "saved_at": 0, "game_state": state}).as_object().unwrap().clone()
}

#[test]
fn every_version_migrates_through_each_step_once_in_order() {
    let mut r = GodotRng::new(5);
    for run in 0..400 {
        let from = r.randi_range(1, FMT.version as i32) as i64;
        let extra = (0..r.randi_range(0, 3)).map(|i| format!("junk{i}")).collect::<Vec<_>>();
        let mut state = json!({"party": r.randi_range(0, 9)});
        for k in &extra {
            state[k] = json!(1);
        }
        let (out, warnings) = migrate(&FMT, &envelope(json!(from), state), step).unwrap_or_else(|e| panic!("run {run}: {e:?}"));
        assert_eq!(out["version"], json!(FMT.version), "run {run}");
        let gs = out["game_state"].as_object().unwrap();
        let steps: Vec<i64> = gs.get("steps").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
        assert_eq!(steps, (from..FMT.version).collect::<Vec<_>>(), "run {run}: each step once, in order");
        for v in (from + 1)..=FMT.version {
            assert_eq!(gs.get(&format!("v{v}")), Some(&json!(true)), "run {run}: step to {v} ran");
        }
        assert!(gs.keys().all(|k| FMT.keys.contains(&k.as_str())), "run {run}: only the format's keys");
        assert_eq!(warnings.len(), extra.len(), "run {run}: one warning per dropped key: {warnings:?}");
        // A migrated envelope migrates again to itself.
        let (again, w2) = migrate(&FMT, &out, step).unwrap();
        assert_eq!((again, w2.len()), (out.clone(), 0), "run {run}");
    }
}

#[test]
fn a_newer_missing_or_broken_version_and_a_missing_step_are_refused() {
    for (version, why) in [(json!(7), "newer"), (json!(0), "invalid"), (json!(-3), "invalid"), (json!("2"), "no version"), (Value::Null, "no version")] {
        let e = migrate(&FMT, &envelope(version.clone(), json!({})), step).unwrap_err();
        assert!(e[0].contains(why), "{version}: {e:?}");
    }
    let gap = |v: i64, s: &Dict, w: &mut Vec<String>| if v == 3 { None } else { step(v, s, w) };
    assert!(migrate(&FMT, &envelope(json!(1), json!({})), gap).unwrap_err()[0].contains("no migration from save version 3"));
}

#[test]
fn random_text_and_json_are_refused_or_opened_never_a_panic() {
    let mut r = GodotRng::new(77);
    let pieces = ["{", "}", "\"format\"", ":", "\"prop_save\"", ",", "\"version\"", "3", "null", "[", "]", "\"game_state\"", "{}", "1e999", "-1", "\"x\""];
    for _ in 0..3000 {
        let text: String = (0..r.randi_range(0, 14)).map(|_| pieces[r.randi_range(0, pieces.len() as i32 - 1) as usize]).collect();
        if let Ok(env) = open(&FMT, &text) {
            assert_eq!(env.get("format"), Some(&json!(FMT.tag)));
            let _ = migrate(&FMT, &env, step);
        }
    }
    for state in [json!(null), json!([1, 2]), json!("s"), json!(3)] {
        let _ = migrate(&FMT, &envelope(json!(2), state), step);
    }
}
