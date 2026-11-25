//! ENGINE-20: consolidated tests for `schema::field_names`, `Hooks::known_fields`, `Hooks::check_types`,
//! `store::merge_fields`, and `store::upsert_fields` (validation order, closed vs open field sets, sorted extra-key
//! reporting, type checks, schema validation, zero-write dry-run, world entries, and blocking rollback).

use aegis_content::file::{document, raw_entries};
use aegis_content::schema;
use aegis_content::store::{self, Hooks};
use aegis_core::json::Dict;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

fn test_schema() -> Value {
    json!({
        "content": {
            "things": [
                {"key": "id", "type": "id", "default": "", "label": "ID"},
                {"key": "name", "type": "string", "default": "", "label": "Name"},
                {"key": "cost", "type": "int", "default": 0, "label": "Cost"},
                {"key": "mode", "type": "enum", "default": "a", "options": ["a", "b"], "label": "Mode"},
                {"key": "tag", "type": "string", "default": "", "label": "Tag"}
            ]
        },
        "world": {
            "notes": [
                {"key": "id", "type": "id", "default": "", "label": "ID"},
                {"key": "title", "type": "string", "default": "", "label": "Title"},
                {"key": "rank", "type": "int", "default": 1, "label": "Rank"}
            ]
        }
    })
}

struct SchemaHooks {
    schema: Value,
    closed: bool,
    fail_blocking: AtomicBool,
}

impl SchemaHooks {
    fn closed() -> Self {
        Self { schema: test_schema(), closed: true, fail_blocking: AtomicBool::new(false) }
    }
    fn open() -> Self {
        Self { schema: test_schema(), closed: false, fail_blocking: AtomicBool::new(false) }
    }
}

impl Hooks for SchemaHooks {
    fn kinds(&self, world: Option<&str>) -> &'static [&'static str] {
        if world.is_some() { &["notes"] } else { &["things"] }
    }

    fn file_of(&self, data_dir: &Path, world: Option<&str>, kind: &str) -> PathBuf {
        match world {
            Some(w) => aegis_content::world::kind_path(data_dir, w, kind),
            None => data_dir.join(format!("{kind}.json")),
        }
    }

    fn validate(&self, world: Option<&str>, kind: &str, entry: &Dict) -> Result<Dict, Vec<String>> {
        let section = if world.is_some() { "world" } else { "content" };
        let full = schema::normalize(schema::fields(&self.schema, section, kind), entry, world.is_some());
        let errors = schema::field_errors(&self.schema, section, kind, &full);
        if errors.is_empty() { Ok(full) } else { Err(errors) }
    }

    fn file_text(&self, _data_dir: &Path, _world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<String, String> {
        Ok(document(kind, 1, entries.iter().cloned().map(Value::Object).collect()))
    }

    fn check(&self, _data_dir: &Path) -> Vec<String> {
        if self.fail_blocking.load(Ordering::SeqCst) {
            vec!["blocking check failed".into()]
        } else {
            Vec::new()
        }
    }

    fn known_fields(&self, world: Option<&str>, kind: &str) -> Option<Vec<String>> {
        if !self.closed {
            return None;
        }
        let section = if world.is_some() { "world" } else { "content" };
        let names = schema::field_names(&self.schema, section, kind);
        (!names.is_empty()).then_some(names)
    }

    fn check_types(&self, world: Option<&str>, kind: &str, entry: &Dict) -> Vec<String> {
        let section = if world.is_some() { "world" } else { "content" };
        schema::type_errors(schema::fields(&self.schema, section, kind), entry)
    }
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn fields(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| (*s).to_string()).collect()
}

fn seed_dir(name: &str) -> aegis_core::scratch::Scratch {
    let dir = aegis_core::scratch::Scratch::new(name);
    std::fs::write(
        dir.join("things.json"),
        document("things", 1, vec![json!({"id": "alpha", "name": "Alpha", "cost": 3, "mode": "a", "tag": "old"})]),
    )
    .unwrap();
    let world_dir = dir.join("worlds").join("demo_world");
    std::fs::create_dir_all(&world_dir).unwrap();
    std::fs::write(world_dir.join("world.json"), "{\"id\": \"demo_world\"}\n").unwrap();
    std::fs::write(
        world_dir.join("notes.json"),
        document("notes", 1, vec![json!({"id": "note_one", "title": "First", "rank": 2})]),
    )
    .unwrap();
    dir
}

#[test]
fn schema_field_names_lists_keys_or_empty_for_unknown() {
    let s = test_schema();
    assert_eq!(schema::field_names(&s, "content", "things"), vec!["id", "name", "cost", "mode", "tag"]);
    assert_eq!(schema::field_names(&s, "world", "notes"), vec!["id", "title", "rank"]);
    assert!(schema::field_names(&s, "content", "missing").is_empty());
    assert!(schema::field_names(&s, "other", "things").is_empty());
}

#[test]
fn merge_fields_validates_kind_world_and_id_first() {
    let dir = seed_dir("fields_first");
    let hooks = SchemaHooks::closed();

    let err = store::merge_fields(&hooks, dir.path(), None, "bogus", &dict(json!({"id": "alpha", "cost": 1})), &fields(&["cost"])).unwrap_err();
    assert!(err[0].contains("Unknown kind 'bogus'"), "{err:?}");

    let err = store::merge_fields(&hooks, dir.path(), Some("no_world"), "notes", &dict(json!({"id": "note_one", "rank": 3})), &fields(&["rank"])).unwrap_err();
    assert!(err[0].contains("Unknown world 'no_world'"), "{err:?}");

    for bad_entry in [json!({"cost": 1}), json!({"id": "", "cost": 1}), json!({"id": "   ", "cost": 1}), json!({"id": 42, "cost": 1})] {
        let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(bad_entry), &fields(&["cost"])).unwrap_err();
        assert!(err[0].contains("non-empty string 'id'"), "{err:?}");
    }
}

#[test]
fn merge_fields_rejects_bad_field_specs_and_sorts_all_extra_keys() {
    let dir = seed_dir("fields_spec");
    let hooks = SchemaHooks::closed();

    let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha"})), &[]).unwrap_err();
    assert!(err.iter().any(|e| e.contains("a list of")), "{err:?}");

    let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha"})), &fields(&["  "])).unwrap_err();
    assert!(err.iter().any(|e| e.contains("cannot be blank")), "{err:?}");

    let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha"})), &fields(&["id"])).unwrap_err();
    assert!(err.iter().any(|e| e.contains("never names 'id'")), "{err:?}");

    let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha", "cost": 5})), &fields(&["cost", "cost"])).unwrap_err();
    assert!(err.iter().any(|e| e.contains("duplicate field 'cost'")), "{err:?}");

    let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha", "cots": 5})), &fields(&["cots"])).unwrap_err();
    assert!(err.iter().any(|e| e.contains("unknown things field 'cots'")), "{err:?}");

    // Multiple extra keys outside {"id"} U fields are ALL reported in deterministic sorted order.
    let err = store::merge_fields(
        &hooks,
        dir.path(),
        None,
        "things",
        &dict(json!({"id": "alpha", "cost": 5, "zebra": 1, "apple": 2, "middle": 3})),
        &fields(&["cost"]),
    )
    .unwrap_err();
    assert_eq!(err.len(), 3, "{err:?}");
    assert!(err[0].contains("holds 'apple'"), "{err:?}");
    assert!(err[1].contains("holds 'middle'"), "{err:?}");
    assert!(err[2].contains("holds 'zebra'"), "{err:?}");

    // Open hooks allow custom field names when known_fields returns None.
    let open = SchemaHooks::open();
    let merged = store::merge_fields(&open, dir.path(), None, "things", &dict(json!({"id": "alpha", "custom": "ok"})), &fields(&["custom"])).unwrap();
    assert_eq!(merged.get("custom"), Some(&json!("ok")));
}

#[test]
fn merge_fields_dry_run_checks_types_and_validate_without_writing() {
    let dir = seed_dir("fields_dry");
    let hooks = SchemaHooks::closed();
    let before = std::fs::read_to_string(dir.join("things.json")).unwrap();

    // Valid dry-run: updates cost, removes tag, preserves name/mode, writes nothing.
    let merged = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha", "cost": 9})), &fields(&["cost", "tag"])).unwrap();
    assert_eq!(merged.get("name"), Some(&json!("Alpha")));
    assert_eq!(merged.get("cost"), Some(&json!(9)));
    assert_eq!(merged.get("mode"), Some(&json!("a")));
    assert_eq!(merged.get("tag"), None);
    assert_eq!(std::fs::read_to_string(dir.join("things.json")).unwrap(), before, "merge_fields must not write");

    // Wrong JSON type for an int field (string or non-whole float) caught by check_types.
    for bad_cost in [json!("nine"), json!(4.5), json!(true)] {
        let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha", "cost": bad_cost})), &fields(&["cost"])).unwrap_err();
        assert!(err.iter().any(|e| e.contains("not a whole number")), "{err:?}");
    }

    // Schema validation error (invalid enum option) caught by validate.
    let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha", "mode": "z"})), &fields(&["mode"])).unwrap_err();
    assert!(err.iter().any(|e| e.contains("not one of a, b")), "{err:?}");

    // Removing a required non-blank name fails validate.
    let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha"})), &fields(&["name"])).unwrap_err();
    assert!(err.iter().any(|e| e.contains("Name is required")), "{err:?}");

    // Missing target entry reports clear error.
    let err = store::merge_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "missing", "cost": 1})), &fields(&["cost"])).unwrap_err();
    assert!(err[0].contains("no things entry 'missing'"), "{err:?}");

    assert_eq!(std::fs::read_to_string(dir.join("things.json")).unwrap(), before);
}

#[test]
fn upsert_fields_persists_base_and_world_and_rolls_back_on_blocking() {
    let dir = seed_dir("fields_upsert");
    let hooks = SchemaHooks::closed();

    let out = store::upsert_fields(&hooks, dir.path(), None, "things", &dict(json!({"id": "alpha", "cost": 7})), &fields(&["cost", "tag"])).unwrap();
    assert!(out.replaced);
    let stored = raw_entries(&dir.join("things.json")).into_iter().find(|e| e["id"] == "alpha").unwrap();
    assert_eq!(stored.get("name"), Some(&json!("Alpha")));
    assert_eq!(stored.get("cost").and_then(Value::as_f64), Some(7.0));
    assert_eq!(stored.get("tag"), None);

    // World entry fields-scoped upsert.
    let wout = store::upsert_fields(&hooks, dir.path(), Some("demo_world"), "notes", &dict(json!({"id": "note_one", "rank": 5})), &fields(&["rank"])).unwrap();
    assert!(wout.replaced);
    let note = store::get(&hooks, dir.path(), Some("demo_world"), "notes", "note_one").unwrap();
    assert_eq!(note.get("title"), Some(&json!("First")));
    assert_eq!(note.get("rank").and_then(Value::as_f64), Some(5.0));

    // Blocking failure rolls back disk cleanly.
    let before = std::fs::read_to_string(dir.join("things.json")).unwrap();
    let rollback_hooks = SchemaHooks::closed();
    // Pre-write check sees no problems; trigger blocking after write by using a custom wrapper.
    struct FlakyBlocking(SchemaHooks, AtomicBool);
    impl Hooks for FlakyBlocking {
        fn kinds(&self, w: Option<&str>) -> &'static [&'static str] { self.0.kinds(w) }
        fn file_of(&self, d: &Path, w: Option<&str>, k: &str) -> PathBuf { self.0.file_of(d, w, k) }
        fn validate(&self, w: Option<&str>, k: &str, e: &Dict) -> Result<Dict, Vec<String>> { self.0.validate(w, k, e) }
        fn file_text(&self, d: &Path, w: Option<&str>, k: &str, e: &[Dict]) -> Result<String, String> { self.0.file_text(d, w, k, e) }
        fn check(&self, _d: &Path) -> Vec<String> {
            if self.1.swap(true, Ordering::SeqCst) { vec!["post-write rule broke".into()] } else { Vec::new() }
        }
        fn known_fields(&self, w: Option<&str>, k: &str) -> Option<Vec<String>> { self.0.known_fields(w, k) }
        fn check_types(&self, w: Option<&str>, k: &str, e: &Dict) -> Vec<String> { self.0.check_types(w, k, e) }
    }
    rollback_hooks.fail_blocking.store(false, Ordering::SeqCst);
    let flaky = FlakyBlocking(rollback_hooks, AtomicBool::new(false));
    let err = store::upsert_fields(&flaky, dir.path(), None, "things", &dict(json!({"id": "alpha", "cost": 99})), &fields(&["cost"])).unwrap_err();
    assert_eq!(err, vec!["post-write rule broke"]);
    assert_eq!(std::fs::read_to_string(dir.join("things.json")).unwrap(), before, "file rolled back");
}
