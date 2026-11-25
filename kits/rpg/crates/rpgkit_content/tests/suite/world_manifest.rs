//! The world manifest writer (WORLDW1): `store::set_world` creates or updates data/worlds/<id>/world.json from a patch
//! of `world.world` fields, merged over the stored manifest (over the schema defaults for a new world), checked against
//! the schema (field types, enum options) and kept only when the data folder adds no blocking or manifest problem
//! (else the file, and a folder the write made, are taken back). Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// A scratch data folder `<tmp>/<tag>/data` with empty catalog and worlds folders.
fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_world_manifest_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn manifest_path(data: &Path, id: &str) -> PathBuf {
    aegis_content::world::manifest_path(data, id)
}

fn stored(data: &Path, id: &str) -> Dict {
    dict(serde_json::from_str(&fs::read_to_string(manifest_path(data, id)).unwrap()).unwrap())
}

/// The schema default of a `world.world` field.
fn default_of(key: &str) -> Value {
    let f = fields(source::schema(), "world", "world").iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("world.world.{key}"));
    json::get(f, "default").clone()
}

/// The keys a new manifest holds: every field in schema order except the engine-only `omit_default` ones (LOOK1-K),
/// which are written only once they hold a value.
fn schema_keys() -> Vec<String> {
    fields(source::schema(), "world", "world").iter().filter(|f| !json::b(json::dict(f), "omit_default", false)).map(|f| json::s(json::dict(f), "key")).collect()
}

#[test]
fn a_new_world_is_written_whole_over_the_schema_defaults() {
    let (_scratch, data) = scratch("new");
    let out = store::set_world(&data, "vale", &dict(json!({"name": "The Vale", "tagline": "Quiet."})), false).expect("new world");
    assert!(!out.replaced);
    assert_eq!(out.file, manifest_path(&data, "vale"));
    let m = stored(&data, "vale");
    let mut keys = schema_keys();
    keys.push("version".into());
    assert_eq!(m.keys().cloned().collect::<Vec<_>>(), keys, "every field in schema order, then the version");
    assert_eq!(m["id"], json!("vale"));
    assert_eq!(m["name"], json!("The Vale"));
    assert_eq!(m["tagline"], json!("Quiet."));
    for key in schema_keys().iter().filter(|k| !["id", "name", "tagline"].contains(&k.as_str())) {
        assert_eq!(m[key.as_str()], default_of(key), "{key} takes its schema default");
    }
    assert_eq!(fs::read_to_string(&out.file).unwrap(), store::manifest_text(&data, "vale").unwrap(), "written as manifest_text renders it");
    assert!(source::load(&data).worlds.get("vale").is_some(), "the loader reads the new world");
    drop_scratch(&data);
}

#[test]
fn an_update_merges_one_field_and_keeps_the_others() {
    let (_scratch, data) = scratch("update");
    store::set_world(&data, "vale", &dict(json!({"name": "The Vale", "description": "Hills.", "start_gold": 50})), false).unwrap();
    let before = stored(&data, "vale");
    let out = store::set_world(&data, "vale", &dict(json!({"tagline": "Still quiet."})), false).expect("update");
    assert!(out.replaced);
    let after = stored(&data, "vale");
    assert_eq!(after["tagline"], json!("Still quiet."));
    for (k, v) in before.iter().filter(|(k, _)| k.as_str() != "tagline") {
        assert_eq!(&after[k.as_str()], v, "{k} kept");
    }
    // the id may be named when it is the world's own; another id is refused
    // (CLN-44: a world's `ruleset` names a rulesets row, and this scratch catalog has none)
    store::set_world(&data, "vale", &dict(json!({"id": "vale", "description": "Valleys."})), false).expect("own id");
    assert_eq!(stored(&data, "vale")["description"], json!("Valleys."));
    let errors = store::set_world(&data, "vale", &dict(json!({"id": "glen"})), false).err().expect("other id refused");
    assert!(errors.iter().any(|e| e.contains("glen")), "{errors:?}");
    drop_scratch(&data);
}

#[test]
fn a_hand_written_manifest_resaves_byte_identical_and_full_writes_every_field() {
    let (_scratch, data) = scratch("trim");
    let dir = data.join("worlds").join("glen");
    fs::create_dir_all(&dir).unwrap();
    // a manifest in the writer's format that leaves default fields out (a world written before a field existed)
    let text = aegis_content::file::stringify(&json!({"id": "glen", "name": "Glen", "extends": [], "connects": [], "version": 1}));
    fs::write(dir.join("world.json"), &text).unwrap();
    store::set_world(&data, "glen", &Dict::new(), false).expect("resave");
    assert_eq!(fs::read_to_string(dir.join("world.json")).unwrap(), text, "an untouched resave is byte-identical");
    store::set_world(&data, "glen", &dict(json!({"tagline": "Low."})), false).unwrap();
    let m = stored(&data, "glen");
    assert_eq!(m.keys().cloned().collect::<Vec<_>>(), ["id", "name", "tagline", "extends", "connects", "version"], "a new field takes its schema place");
    store::set_world(&data, "glen", &dict(json!({"party_max": default_of("party_max")})), false).unwrap();
    assert!(stored(&data, "glen").contains_key("party_max"), "a field the patch names is written even at its default");
    store::set_world(&data, "glen", &Dict::new(), true).unwrap();
    assert_eq!(fs::read_to_string(dir.join("world.json")).unwrap(), store::manifest_text(&data, "glen").unwrap(), "full: every field");
    drop_scratch(&data);
}

#[test]
fn bad_values_are_refused_and_nothing_is_written() {
    let (_scratch, data) = scratch("bad");
    for patch in [
        json!({"name": "Moor", "ruleset": "no_such_ruleset"}),
        json!({"name": "Moor", "party_max": "three"}),
        json!({"name": "Moor", "extends": "vale"}),
        json!({"name": "Moor", "no_such_field": 1}),
        json!({"name": ""}),
    ] {
        let errors = store::set_world(&data, "moor", &dict(patch.clone()), false).err().unwrap_or_else(|| panic!("{patch} refused"));
        assert!(!errors.is_empty());
        assert!(!data.join("worlds").join("moor").exists(), "{patch}: no folder written");
    }
    let errors = store::set_world(&data, "moor", &dict(json!({"ruleset": "no_such_ruleset"})), false).err().unwrap();
    assert!(errors.iter().any(|e| e.contains("no_such_ruleset")), "{errors:?}");
    assert!(store::set_world(&data, "Bad Id", &dict(json!({"name": "x"})), false).is_err());
    store::set_world(&data, "moor", &dict(json!({"name": "Moor"})), false).unwrap();
    let text = fs::read_to_string(manifest_path(&data, "moor")).unwrap();
    assert!(store::set_world(&data, "moor", &dict(json!({"ruleset": "no_such_ruleset"})), false).is_err());
    assert_eq!(fs::read_to_string(manifest_path(&data, "moor")).unwrap(), text, "an existing manifest is left as it was");
    drop_scratch(&data);
}

#[test]
fn a_change_that_breaks_the_data_folder_rolls_back() {
    let (_scratch, data) = scratch("rollback");
    // a new world naming an unknown parent: refused, and its folder taken back
    let errors = store::set_world(&data, "fen", &dict(json!({"name": "Fen", "extends": ["no_such_world"]})), false).err().expect("unknown parent refused");
    assert!(errors.iter().any(|e| e.contains("no_such_world")), "{errors:?}");
    assert!(!data.join("worlds").join("fen").exists(), "the new folder is taken back");
    // an update that makes the inheritance circular: the old file is put back
    store::set_world(&data, "vale", &dict(json!({"name": "Vale"})), false).unwrap();
    store::set_world(&data, "glen", &dict(json!({"name": "Glen", "extends": ["vale"]})), false).unwrap();
    let vale = fs::read_to_string(manifest_path(&data, "vale")).unwrap();
    let errors = store::set_world(&data, "vale", &dict(json!({"extends": ["glen"]})), false).err().expect("cycle refused");
    assert!(errors.iter().any(|e| e.contains("circular")), "{errors:?}");
    assert_eq!(fs::read_to_string(manifest_path(&data, "vale")).unwrap(), vale);
    // a ruleset whose row the catalog lacks
    let errors = store::set_world(&data, "vale", &dict(json!({"ruleset": "timeline"})), false).err().expect("ruleset without its row refused");
    assert!(errors.iter().any(|e| e.contains("timeline")), "{errors:?}");
    assert_eq!(fs::read_to_string(manifest_path(&data, "vale")).unwrap(), vale);
    // a problem that was there before the write does not block it
    fs::write(manifest_path(&data, "glen"), aegis_content::file::stringify(&json!({"id": "glen", "name": "Glen", "extends": ["vale", "gone"], "version": 1}))).unwrap();
    store::set_world(&data, "vale", &dict(json!({"tagline": "Up."})), false).expect("an older problem elsewhere does not block");
    drop_scratch(&data);
}
