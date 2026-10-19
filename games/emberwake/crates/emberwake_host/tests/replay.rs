//! The command log replays a battle exactly: the same requests on a fresh host give the same state.

use emberwake_data::Catalog;
use emberwake_host::{Battle, Host};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

fn catalog() -> Arc<Catalog> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/parity/out/catalog.json");
    Arc::new(Catalog::load_resolved(&p).expect("missing catalog.json - run engine/parity/export.sh"))
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
