//! PR5-51b: a seed's remove step with `"if_exists": true` is idempotent (a missing entry is a no-op reported as
//! absent); without the flag a missing entry still fails the step. A minimal writer hook set over a temp folder.

use aegis_cli::content_step;
use aegis_content::file::{document, raw_entries};
use aegis_content::store::Hooks;
use aegis_core::json::Dict;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

struct Flat;

impl Hooks for Flat {
    fn kinds(&self, _world: Option<&str>) -> &'static [&'static str] {
        &["things"]
    }
    fn file_of(&self, data_dir: &Path, _world: Option<&str>, kind: &str) -> PathBuf {
        data_dir.join(format!("{kind}.json"))
    }
    fn validate(&self, _world: Option<&str>, _kind: &str, entry: &Dict) -> Result<Dict, Vec<String>> {
        Ok(entry.clone())
    }
    fn file_text(&self, _data_dir: &Path, _world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<String, String> {
        Ok(document(kind, 1, entries.iter().cloned().map(Value::Object).collect()))
    }
    fn check(&self, _data_dir: &Path) -> Vec<String> {
        Vec::new()
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aegis_content_step_{name}_{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(dir.join("things.json"), document("things", 1, vec![json!({"id": "a"}), json!({"id": "b"})])).expect("seed file");
    dir
}

fn ids(dir: &Path) -> Vec<String> {
    raw_entries(&dir.join("things.json")).iter().map(|e| e["id"].as_str().unwrap_or_default().to_string()).collect()
}

#[test]
fn if_exists_remove_runs_twice() {
    let dir = scratch("twice");
    let step = json!({"op": "remove", "kind": "things", "id": "a", "if_exists": true});
    let first = content_step(&Flat, &dir, &step).expect("first run removes");
    assert!(first.starts_with("removed things 'a'"), "{first}");
    assert_eq!(ids(&dir), ["b"]);
    let before = std::fs::read_to_string(dir.join("things.json")).expect("file");
    let second = content_step(&Flat, &dir, &step).expect("second run is a no-op");
    assert!(second.starts_with("absent things 'a'"), "{second}");
    assert_eq!(std::fs::read_to_string(dir.join("things.json")).expect("file"), before, "an absent remove writes nothing");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn plain_remove_of_a_missing_entry_still_fails() {
    let dir = scratch("plain");
    for step in [json!({"op": "remove", "kind": "things", "id": "zz"}), json!({"op": "remove", "kind": "things", "id": "zz", "if_exists": false})] {
        let err = content_step(&Flat, &dir, &step).expect_err("a missing entry fails without if_exists");
        assert_eq!(err, ["No things entry 'zz'."]);
    }
    assert_eq!(ids(&dir), ["a", "b"]);
    std::fs::remove_dir_all(&dir).ok();
}
