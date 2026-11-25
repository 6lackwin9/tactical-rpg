//! ASSETORG4 (FB-227): whether a game mounts the engine's opt-in portrait overlay layer is a kit switch of its manifest,
//! `portrait_overlays` (`true` or `false`; missing reads off, so a new game opts in). A present value that is not a
//! boolean reads off and is a manifest problem; the loader carries it; the manifest writer sets it, refuses a bad value
//! with the check's own problem and removes it on a null. Scratch game folders only.

use aegis_core::json::Dict;
use rpgkit_content::source::{self, MANIFEST_KEYS, PORTRAIT_OVERLAYS};
use rpgkit_content::store;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// A scratch game folder holding a bare `game.json` and `data/` with empty catalog and worlds folders.
fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_portrait_overlay_switch_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let m = json!({"id": "demo", "title": "Demo", "version": "0.1.0", "worlds": [], "start_worlds": [], "references": []});
    fs::write(t.join("game.json"), serde_json::to_string_pretty(&m).unwrap()).unwrap();
    (t, data)
}

#[test]
fn the_portrait_overlay_switch_reads_off_unless_set_true() {
    assert!(MANIFEST_KEYS.contains(&PORTRAIT_OVERLAYS), "a registered kit key");
    assert_eq!(PORTRAIT_OVERLAYS, "portrait_overlays");
    assert!(!source::portrait_overlays_of(&Dict::new()), "missing: off (a game opts in)");
    assert!(source::portrait_overlays_problems(&Dict::new()).is_empty());
    for (v, on) in [(json!(true), true), (json!(false), false)] {
        let m = dict(json!({PORTRAIT_OVERLAYS: v}));
        assert_eq!(source::portrait_overlays_of(&m), on, "{v}");
        assert!(source::manifest_problems(&m).is_empty(), "{v}");
    }
    let problem = format!("game.json: {PORTRAIT_OVERLAYS} must be true or false.");
    for bad in [json!("yes"), json!(1), json!({})] {
        let m = dict(json!({PORTRAIT_OVERLAYS: bad}));
        assert!(!source::portrait_overlays_of(&m), "{bad}: off");
        assert_eq!(source::portrait_overlays_problems(&m), [problem.clone()], "{bad}");
        assert!(source::manifest_problems(&m).contains(&problem), "{bad}: a manifest problem");
    }
}

#[test]
fn the_loader_carries_it_and_the_writer_sets_checks_and_removes_it() {
    let (t, data) = scratch("set");
    assert!(!source::load(&data).portrait_overlays, "no key: off");
    let path = t.join("game.json");
    store::set_manifest(&path, &dict(json!({PORTRAIT_OVERLAYS: true}))).unwrap();
    assert!(source::load(&data).portrait_overlays, "set on: the loader reads it");
    let before = fs::read_to_string(&path).unwrap();
    assert_eq!(store::set_manifest(&path, &dict(json!({PORTRAIT_OVERLAYS: "on"}))).unwrap_err(), [format!("game.json: {PORTRAIT_OVERLAYS} must be true or false.")]);
    assert_eq!(fs::read_to_string(&path).unwrap(), before, "nothing written");
    store::set_manifest(&path, &dict(json!({PORTRAIT_OVERLAYS: null}))).unwrap();
    assert!(!source::load(&data).portrait_overlays, "removed: off");
    assert!(store::check(&data).iter().all(|l| !l.contains(PORTRAIT_OVERLAYS)));
}
