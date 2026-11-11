//! The command log replays a battle exactly: the same requests on a fresh host give the same state. Runs on the
//! shipped data (games/emberwake/data, read only; CLN-36: no Godot export).

use emberwake_data::Catalog;
use emberwake_host::{Battle, Host};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

/// The shipped catalog (catalog encounters start through `new` by id).
fn catalog() -> Arc<Catalog> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    emberwake_world::Data::open(&dir).expect("the shipped data opens").0.cat.clone()
}

#[test]
fn history_replays_to_the_same_state() {
    let cat = catalog();
    let mut host = Host::new(Battle::new(cat.clone()));
    let script = [
        json!({"cmd": "new", "encounter": "goblin_ambush", "seed": 99}),
        json!({"cmd": "plan", "unit": 0, "steps": [{"mode": "self", "skill": "guard"}, {"mode": "engage", "skill": "strike", "target": "Goblin Brawler"}]}),
        json!({"cmd": "preset", "unit": 1, "preset": "ranged_kite"}),
        json!({"cmd": "step"}),
        json!({"cmd": "state"}),
        json!({"cmd": "resolve"}),
        json!({"cmd": "advance"}),
        json!({"cmd": "autoplay", "preset": "aggressive"}),
    ];
    for r in &script {
        assert_eq!(host.handle(r)["ok"], json!(true), "request failed: {r}");
    }
    let end: Value = host.handle(&json!({"cmd": "state"}))["state"].clone();
    let history = host.handle(&json!({"cmd": "history"}))["history"].as_array().unwrap().clone();
    assert!(history.iter().all(|r| r["cmd"] != json!("state")), "read-only requests are not recorded");
    let mut again = Host::new(Battle::new(cat));
    for r in &history {
        again.handle(r);
    }
    assert_eq!(again.handle(&json!({"cmd": "state"}))["state"], end);
}

/// A battle played round by round: preset, confirm, resolve, advance.
fn play(host: &mut Host, start: Value, rounds: usize) {
    assert_eq!(host.handle(&start)["ok"], json!(true));
    for _ in 0..rounds {
        for r in [json!({"cmd": "preset", "preset": "balanced"}), json!({"cmd": "confirm"}), json!({"cmd": "resolve"})] {
            assert_eq!(host.handle(&r)["ok"], json!(true), "{r}");
        }
        if host.handle(&json!({"cmd": "state"}))["state"]["over"] == json!(true) {
            break;
        }
        assert_eq!(host.handle(&json!({"cmd": "advance"}))["ok"], json!(true));
    }
}

/// PLAYTEST1 acceptance on the engine: a recorded battle replays checkpoint for checkpoint; a changed HP diverges at
/// its round and later rounds are unverified; a take-over stops in the planning of a recorded round; a seed sweep's
/// recorded seed gives the recorded end.
#[test]
fn a_recorded_battle_replays_round_by_round() {
    use aegis_host::playtest::{self, Recording, Session, Verdict};
    let cat = catalog();
    let mut host = Host::new(Battle::new(cat.clone()));
    play(&mut host, json!({"cmd": "new", "encounter": "goblin_ambush", "seed": 7}), 4);
    let s = Session::of(&host);
    assert!(s.checkpoints.len() >= 3, "a checkpoint per concluded round: {}", s.checkpoints.len());
    assert_eq!(s.checkpoints[0]["state"]["round"], json!(1));
    let mut rec = Recording::new("20261020-000000", "agent", json!({}));
    rec.sessions.push(s.clone());
    let back = Recording::from_json(&rec.to_json()).unwrap();
    let report = playtest::replay(&mut Host::new(Battle::new(cat.clone())), &back.sessions[0], None);
    assert!(report.matched(), "{report:?}");

    let mut changed = s.clone();
    changed.checkpoints[1]["state"]["units"][0]["hp"] = json!(-5);
    let report = playtest::replay(&mut Host::new(Battle::new(cat.clone())), &changed, None);
    assert_eq!(report.checks[0].verdict, Verdict::Match);
    assert!(matches!(&report.checks[1].verdict, Verdict::Diverge { path, .. } if path == "units.0.hp"), "{:?}", report.checks[1]);
    assert!(report.checks[2..].iter().all(|c| c.verdict == Verdict::Unverified));

    let until = s.checkpoints[1]["at"].as_u64().unwrap() as usize + 1;
    let mut taken = Host::new(Battle::new(cat.clone()));
    assert!(playtest::replay(&mut taken, &s, Some(until)).matched());
    let state = taken.handle(&json!({"cmd": "state"}))["state"].clone();
    assert_eq!((state["round"].clone(), state["phase"].clone()), (json!(3), json!("planning")), "after round 2's advance");
    assert_eq!(taken.handle(&json!({"cmd": "preset"}))["ok"], json!(true), "the player goes on live");

    assert_eq!(playtest::sweep_one(&mut Host::new(Battle::new(cat.clone())), &s, "seed", 7, None), s.result());
}

/// World encounters start through `new` ("<world>:<id>") on a host with the world data, and replay.
#[test]
fn a_world_encounter_records_and_replays() {
    use aegis_host::playtest::{self, Session};
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = Arc::new(emberwake_world::Data::open(&dir).unwrap().0);
    let encounter = format!("ashlands:{}", aegis_core::json::s(&data.own_entries("ashlands", "encounters")[0], "id"));
    let mut host = Host::new(Battle::with_data(data.clone()));
    play(&mut host, json!({"cmd": "new", "encounter": encounter, "seed": 3}), 2);
    let s = Session::of(&host);
    assert!(!s.checkpoints.is_empty());
    assert!(playtest::replay(&mut Host::new(Battle::with_data(data)), &s, None).matched());
    let refused = Host::new(Battle::new(catalog())).handle(&json!({"cmd": "new", "encounter": encounter}));
    assert!(refused["error"].as_str().unwrap().contains("no world data"));
}
