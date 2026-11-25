//! PR5-51b: a seed's remove step with `"if_exists": true` is idempotent (a missing entry is a no-op reported as
//! absent); without the flag a missing entry still fails the step. A minimal writer hook set over a temp folder.

use aegis_cli::{check_content_step, content_step};
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

struct StrictFlat;

impl Hooks for StrictFlat {
    fn kinds(&self, world: Option<&str>) -> &'static [&'static str] {
        Flat.kinds(world)
    }
    fn file_of(&self, data_dir: &Path, world: Option<&str>, kind: &str) -> PathBuf {
        Flat.file_of(data_dir, world, kind)
    }
    fn validate(&self, world: Option<&str>, kind: &str, entry: &Dict) -> Result<Dict, Vec<String>> {
        Flat.validate(world, kind, entry)
    }
    fn file_text(&self, data_dir: &Path, world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<String, String> {
        Flat.file_text(data_dir, world, kind, entries)
    }
    fn check(&self, data_dir: &Path) -> Vec<String> {
        Flat.check(data_dir)
    }
    fn known_fields(&self, _world: Option<&str>, kind: &str) -> Option<Vec<String>> {
        (kind == "things").then(|| vec!["id".into(), "name".into(), "cost".into(), "tag".into()])
    }
    fn check_types(&self, _world: Option<&str>, _kind: &str, entry: &Dict) -> Vec<String> {
        match entry.get("cost") {
            Some(v) if !aegis_core::json::is_int(v) => vec![format!("Cost: {v} is not a whole number.")],
            _ => Vec::new(),
        }
    }
    fn sidecar_ops(&self) -> &'static [&'static str] {
        &["layout"]
    }
    fn sidecar(&self, _data_dir: &Path, op: &str, name: &str, _edits: Option<&Value>) -> Result<String, Vec<String>> {
        Ok(format!("wrote {op} {name}"))
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

/// ENGINE-20 (TOOLS-HELP3): an upsert with `"fields"` changes only the named fields of the stored entry (one left out
/// of the entry is removed) and keeps the rest, so it never undoes another seed's fields; it is refused for a missing
/// entry, a field it does not name, `id`, or a list that is not names.
#[test]
fn an_upsert_with_fields_changes_only_those() {
    let dir = scratch("fields");
    let whole = json!({"op": "upsert", "kind": "things", "entry": {"id": "a", "name": "Alpha", "cost": 3, "tag": "x"}});
    content_step(&Flat, &dir, &whole).expect("a whole upsert");
    let step = json!({"op": "upsert", "kind": "things", "fields": ["cost", "tag"], "entry": {"id": "a", "cost": 5}});
    let line = content_step(&Flat, &dir, &step).expect("fields");
    assert!(line.starts_with("replaced things 'a'"), "{line}");
    let a = raw_entries(&dir.join("things.json")).into_iter().find(|e| e["id"] == "a").expect("a");
    assert_eq!((a.get("name"), a.get("cost").and_then(Value::as_f64), a.get("tag")), (Some(&json!("Alpha")), Some(5.0), None), "name kept, cost changed, tag removed");
    for (bad, why) in [
        (json!({"op": "upsert", "kind": "things", "fields": ["cost"], "entry": {"id": "zz", "cost": 1}}), "no things entry 'zz'"),
        (json!({"op": "upsert", "kind": "things", "fields": ["cost"], "entry": {"id": "a", "cost": 1, "name": "B"}}), "holds 'name'"),
        (json!({"op": "upsert", "kind": "things", "fields": ["id"], "entry": {"id": "a"}}), "never names 'id'"),
        (json!({"op": "upsert", "kind": "things", "fields": "cost", "entry": {"id": "a"}}), "a list of"),
        (json!({"op": "upsert", "kind": "things", "fields": [], "entry": {"id": "a"}}), "a list of"),
    ] {
        let err = content_step(&Flat, &dir, &bad).expect_err("refused").join(" ");
        assert!(err.contains(why), "{err}");
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn content_step_rejects_malformed_shapes_unknown_keys_and_replace_with_fields() {
    let dir = scratch("hardened");
    let before = std::fs::read_to_string(dir.join("things.json")).expect("file");

    for (bad, why) in [
        (json!("not an object"), "must be a JSON object"),
        (json!([1, 2]), "must be a JSON object"),
        (json!({"op": 99, "kind": "things", "entry": {"id": "a"}}), "'op' must be a string"),
        // Typo "field" instead of "fields" must fail instead of silently doing a whole upsert!
        (json!({"op": "upsert", "kind": "things", "field": ["cost"], "entry": {"id": "a", "cost": 5}}), "unknown key(s): field"),
        // Typo "worlds" instead of "world" must fail immediately!
        (json!({"op": "upsert", "kind": "things", "worlds": "demo", "entry": {"id": "a"}}), "unknown key(s): worlds"),
        // Multiple unknown keys are reported sorted.
        (json!({"op": "remove", "kind": "things", "id": "a", "zzz": 1, "aaa": 2}), "unknown key(s): aaa, zzz"),
        (json!({"op": "world", "id": "demo", "fields": {}, "extra": 1}), "unknown key(s): extra"),
        (json!({"op": "layout", "name": "m", "bogus": 1}), "unknown key(s): bogus"),
        // Combining "replace" and "fields" is refused.
        (json!({"op": "upsert", "kind": "things", "fields": ["cost"], "replace": "a", "entry": {"id": "a", "cost": 1}}), "cannot combine 'fields' with 'replace'"),
        // Non-string/non-bool optional fields are refused.
        (json!({"op": "upsert", "kind": "things", "world": 123, "entry": {"id": "a"}}), "'world' must be a string"),
        (json!({"op": "remove", "kind": "things", "id": "a", "if_exists": "yes"}), "'if_exists' must be a boolean"),
        (json!({"op": "upsert", "kind": "things", "fields": [1, "cost"], "entry": {"id": "a", "cost": 1}}), "a list of"),
    ] {
        let err = content_step(&StrictFlat, &dir, &bad).expect_err("must be refused").join(" ");
        assert!(err.contains(why), "expected '{why}' in '{err}'");
        let chk = check_content_step(&StrictFlat, &dir, &bad).expect_err("check must refuse too").join(" ");
        assert!(chk.contains(why), "expected '{why}' in check '{chk}'");
    }

    // Closed field list and check_types wired through store::upsert_fields.
    let err = content_step(&StrictFlat, &dir, &json!({"op": "upsert", "kind": "things", "fields": ["cots"], "entry": {"id": "a", "cots": 1}}))
        .expect_err("unknown field refused")
        .join(" ");
    assert!(err.contains("unknown things field 'cots'"), "{err}");

    let err = content_step(&StrictFlat, &dir, &json!({"op": "upsert", "kind": "things", "fields": ["cost"], "entry": {"id": "a", "cost": "bad"}}))
        .expect_err("type mismatch refused")
        .join(" ");
    assert!(err.contains("not a whole number"), "{err}");

    assert_eq!(std::fs::read_to_string(dir.join("things.json")).expect("file"), before, "failed steps wrote nothing");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn check_content_step_validates_without_writing_to_disk() {
    let dir = scratch("check_mode");
    let before = std::fs::read_to_string(dir.join("things.json")).expect("file");

    let ok_fields = check_content_step(
        &StrictFlat,
        &dir,
        &json!({"op": "upsert", "kind": "things", "fields": ["cost"], "entry": {"id": "a", "cost": 9}}),
    )
    .expect("valid fields check");
    assert!(ok_fields.contains("ok upsert things 'a'"), "{ok_fields}");

    let ok_whole = check_content_step(
        &StrictFlat,
        &dir,
        &json!({"op": "upsert", "kind": "things", "entry": {"id": "c", "name": "C"}}),
    )
    .expect("valid whole upsert check");
    assert!(ok_whole.contains("ok upsert things 'c'"), "{ok_whole}");

    let ok_rm = check_content_step(&StrictFlat, &dir, &json!({"op": "remove", "kind": "things", "id": "a"})).expect("valid remove check");
    assert!(ok_rm.contains("ok remove things 'a'"), "{ok_rm}");

    let ok_absent = check_content_step(
        &StrictFlat,
        &dir,
        &json!({"op": "remove", "kind": "things", "id": "missing", "if_exists": true}),
    )
    .expect("if_exists absent check");
    assert!(ok_absent.contains("ok absent things 'missing'"), "{ok_absent}");

    let err_rm = check_content_step(&StrictFlat, &dir, &json!({"op": "remove", "kind": "things", "id": "missing"})).expect_err("missing remove fails check");
    assert_eq!(err_rm, ["No things entry 'missing'."]);

    let err_rep = check_content_step(
        &StrictFlat,
        &dir,
        &json!({"op": "upsert", "kind": "things", "replace": "missing", "entry": {"id": "a"}}),
    )
    .expect_err("missing replace target fails check");
    assert!(err_rep[0].contains("No things entry 'missing' to replace"), "{err_rep:?}");

    assert_eq!(std::fs::read_to_string(dir.join("things.json")).expect("file"), before, "check_content_step must never mutate disk");
    std::fs::remove_dir_all(&dir).ok();
}
