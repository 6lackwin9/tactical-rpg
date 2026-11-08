//! FB-110 (E2.2-S2): the game manifest's extra keys. A game's content hooks register keys beyond `manifest::KEYS`
//! (`Hooks::manifest_keys`) and check them (`Hooks::manifest_problems`); `store::set_manifest` writes them after the
//! engine keys, refuses a key nobody registered and writes nothing when the hooks find a problem. Extra keys already in
//! a file survive a read, a re-save and a plain `Manifest::set`. The hook here is a test-only fake ("speed", 1..9).

use aegis_content::manifest::{Manifest, FILE, KEYS};
use aegis_content::store::{self, Hooks};
use aegis_core::json::Dict;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

struct Fake;

impl Hooks for Fake {
    fn kinds(&self, _world: Option<&str>) -> &'static [&'static str] {
        &[]
    }
    fn file_of(&self, data_dir: &Path, _world: Option<&str>, kind: &str) -> PathBuf {
        data_dir.join(kind)
    }
    fn validate(&self, _world: Option<&str>, _kind: &str, entry: &Dict) -> Result<Dict, Vec<String>> {
        Ok(entry.clone())
    }
    fn file_text(&self, _data_dir: &Path, _world: Option<&str>, _kind: &str, _entries: &[Dict]) -> Result<String, String> {
        Ok(String::new())
    }
    fn check(&self, _data_dir: &Path) -> Vec<String> {
        Vec::new()
    }
    fn manifest_keys(&self) -> &'static [&'static str] {
        &["speed", "mode"]
    }
    fn manifest_problems(&self, m: &Dict) -> Vec<String> {
        match m.get("speed") {
            Some(v) if !v.as_i64().is_some_and(|n| (1..=9).contains(&n)) => vec!["game.json: speed must be 1..9.".into()],
            _ => Vec::new(),
        }
    }
}

fn patch(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn keys(path: &Path) -> Vec<String> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    v.as_object().unwrap().keys().cloned().collect()
}

#[test]
fn a_registered_key_is_written_after_the_engine_keys_and_checked() {
    let root = aegis_core::scratch::Scratch::vacant("manifest_extra");
    std::fs::create_dir_all(root.path()).unwrap();
    let path = root.join(FILE);
    let m = store::set_manifest(&Fake, &path, &patch(json!({"id": "demo", "title": "Demo", "mode": "fast", "speed": 3}))).unwrap();
    assert_eq!(m.extra.keys().cloned().collect::<Vec<_>>(), ["speed", "mode"], "in the hooks' order");
    let back = Manifest::read(&path).unwrap();
    assert_eq!((back.extra.get("speed"), back.extra.get("mode")), (Some(&json!(3)), Some(&json!("fast"))), "a whole number stays whole");
    let written = keys(&path);
    assert_eq!(&written[written.len() - 2..], ["speed", "mode"], "after the engine keys");
    assert!(written[..written.len() - 2].iter().all(|k| KEYS.contains(&k.as_str())));

    let before = std::fs::read_to_string(&path).unwrap();
    assert_eq!(store::set_manifest(&Fake, &path, &patch(json!({"speed": 12}))).unwrap_err(), ["game.json: speed must be 1..9."]);
    assert!(store::set_manifest(&Fake, &path, &patch(json!({"colour": 1}))).unwrap_err()[0].contains("no key 'colour'"), "an unregistered key is refused");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before, "nothing written on a problem");

    // the engine alone keeps the extra keys of the file but sets none
    assert!(Manifest::set(&path, &patch(json!({"speed": 4}))).unwrap_err()[0].contains("no key 'speed'"));
    assert_eq!(Manifest::set(&path, &patch(json!({"title": "Demo Two"}))).unwrap().extra, back.extra, "a plain set keeps them");

    // null removes an extra key
    let m = store::set_manifest(&Fake, &path, &patch(json!({"mode": null}))).unwrap();
    assert_eq!(m.extra.keys().cloned().collect::<Vec<_>>(), ["speed"]);
    assert!(!keys(&path).contains(&"mode".to_string()));

    // an extra key the game does not register (a hand-edited typo) is never kept silently
    let mut raw: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    raw.as_object_mut().unwrap().insert("sped".into(), json!(5));
    std::fs::write(&path, serde_json::to_string_pretty(&raw).unwrap()).unwrap();
    let err = store::set_manifest(&Fake, &path, &patch(json!({"title": "Demo Three"}))).unwrap_err();
    assert!(err.iter().any(|e| e.contains("key 'sped' is not a game.json key")), "{err:?}");
    let m = store::set_manifest(&Fake, &path, &patch(json!({"sped": null}))).unwrap();
    assert!(!m.extra.contains_key("sped"), "null removes it");
}

#[test]
fn a_manifest_with_extra_keys_resaves_as_read() {
    let d = patch(json!({"id": "demo", "title": "Demo", "subtitle": "", "version": "0.1.0", "kit": "", "engine": "", "worlds": [], "start_worlds": [], "references": [], "speed": 2.0}));
    let m = Manifest::from_dict(&d);
    assert_eq!(m.extra.get("speed"), Some(&json!(2)), "file floats read whole");
    assert_eq!(Manifest::from_dict(&m.to_dict()).to_dict(), m.to_dict());
    assert!(Manifest::from_dict(&patch(json!({"id": "demo"}))).extra.is_empty());
}
