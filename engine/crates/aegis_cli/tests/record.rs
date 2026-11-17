//! REPLAY-CI1: what `aegis record` plays: one list of requests is one session, a list of lists is one session each, a
//! playtest recording gives its sessions' requests (recorded again); anything else is refused.

use aegis_cli::record_sessions;
use serde_json::json;

#[test]
fn a_list_is_one_session_and_a_list_of_lists_one_each() {
    let one = record_sessions(&json!([{"cmd": "new"}, {"cmd": "step"}])).unwrap();
    assert_eq!(one, vec![vec![json!({"cmd": "new"}), json!({"cmd": "step"})]]);
    let two = record_sessions(&json!([[{"cmd": "new"}], [{"cmd": "new", "seed": 2}, {"cmd": "step"}]])).unwrap();
    assert_eq!(two.len(), 2);
    assert_eq!(two[1].len(), 2);
}

#[test]
fn a_recording_gives_its_sessions_requests() {
    let rec = json!({"format": "playtest", "version": 2, "id": "r", "content": {}, "sessions": [
        {"requests": [{"cmd": "new"}, {"cmd": "step"}], "checkpoints": [{"at": 2, "state": {}}], "end": {"result": "won", "at": 2}},
        {"requests": [{"cmd": "new"}], "checkpoints": []}
    ]});
    let s = record_sessions(&rec).unwrap();
    assert_eq!(s, vec![vec![json!({"cmd": "new"}), json!({"cmd": "step"})], vec![json!({"cmd": "new"})]]);
}

#[test]
fn other_shapes_are_refused() {
    for bad in [json!({}), json!([]), json!([[]]), json!([{"no": "cmd"}]), json!([[{"cmd": "new"}], []]), json!({"format": "playtest", "version": 1})] {
        assert!(record_sessions(&bad).is_err(), "{bad}");
    }
}
