//! LAYER-ENG-KEYS (PR5-29a): a recording's sessions are written under `sessions`; a file with the old name `battles`
//! still loads (and is written back with the new name), and a file with both names is refused.

use aegis_host::playtest::{Recording, Session, SESSIONS_ALIAS, SESSIONS_KEY};
use serde_json::{json, Value};

fn session(n: i64) -> Value {
    json!({"index": 0, "requests": [{"cmd": "new", "seed": n}, {"cmd": "add", "n": n}], "checkpoints": [{"at": 1, "state": {"total": 0}}], "end": {"result": "won", "at": 2}})
}

fn file(key: &str) -> Value {
    json!({"format": "playtest", "version": 2, "id": "20261111-000000", "origin": "user", "parent": null, key: [session(1), session(2)]})
}

#[test]
fn sessions_are_written_under_the_new_key() {
    let mut rec = Recording::new("20261111-000000", "agent", Value::Null);
    rec.sessions.push(Session { requests: vec![json!({"cmd": "new"})], checkpoints: Vec::new(), end: json!({"result": "won", "at": 1}) });
    let out = rec.to_json();
    assert_eq!(out[SESSIONS_KEY].as_array().map(Vec::len), Some(1), "{out}");
    assert!(out.get(SESSIONS_ALIAS).is_none(), "the old name is not written: {out}");
    assert_eq!(SESSIONS_KEY, "sessions");
}

#[test]
fn the_old_battles_key_still_loads_and_saves_as_sessions() {
    let new = Recording::from_json(&file(SESSIONS_KEY)).unwrap();
    let old = Recording::from_json(&file(SESSIONS_ALIAS)).unwrap();
    assert_eq!(new.sessions.len(), 2);
    assert_eq!(old.sessions, new.sessions, "the old name reads as the new one");
    assert!(old.extra.is_empty(), "the old key is not kept as an unknown one: {:?}", old.extra);
    let back = old.to_json();
    assert!(back.get(SESSIONS_ALIAS).is_none() && back[SESSIONS_KEY] == new.to_json()[SESSIONS_KEY], "{back}");
}

#[test]
fn a_file_with_both_names_is_refused() {
    let mut both = file(SESSIONS_KEY);
    both[SESSIONS_ALIAS] = json!([]);
    let e = Recording::from_json(&both).unwrap_err();
    assert!(e.contains(SESSIONS_KEY) && e.contains(SESSIONS_ALIAS), "{e}");
}

#[test]
fn a_broken_session_under_either_name_is_refused() {
    for key in [SESSIONS_KEY, SESSIONS_ALIAS] {
        let bad = json!({"format": "playtest", "version": 2, key: [{"requests": [{"cmd": "new"}], "checkpoints": [{"at": 5}]}]});
        assert!(Recording::from_json(&bad).unwrap_err().contains("past its requests"), "{key}");
    }
}
