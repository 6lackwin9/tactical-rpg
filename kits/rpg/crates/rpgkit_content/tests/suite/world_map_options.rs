//! MAP-HOOKS1 (AO-01 (4)): the world map's lantern flicker is a kit switch of the game's manifest,
//! `world_map_lantern_flicker` (`true` or `false`; missing reads off). A present value that is not a boolean reads off
//! and is a manifest problem; the loader carries it; the manifest writer sets it, refuses a bad value with the check's
//! own problem and removes it on a null. Scratch game folders only.

use aegis_core::json::Dict;
use rpgkit_content::source::{self, MANIFEST_KEYS, WORLD_MAP_LANTERN_FLICKER};
use rpgkit_content::store;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// A scratch game folder holding `game.json` (a bare manifest plus `extra`) and `data/` with empty catalog and worlds
/// folders.
fn scratch(tag: &str, extra: Value) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_world_map_options_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let mut m = json!({"id": "demo", "title": "Demo", "version": "0.1.0", "worlds": [], "start_worlds": [], "references": []});
    for (k, v) in extra.as_object().unwrap() {
        m[k] = v.clone();
    }
    fs::write(t.join("game.json"), serde_json::to_string_pretty(&m).unwrap()).unwrap();
    (t, data)
}

#[test]
fn the_lantern_flicker_switch_reads_off_unless_set_true() {
    assert!(MANIFEST_KEYS.contains(&WORLD_MAP_LANTERN_FLICKER), "a registered kit key");
    assert!(!source::lantern_flicker_of(&Dict::new()), "missing: off");
    assert!(source::lantern_flicker_problems(&Dict::new()).is_empty());
    for (v, on) in [(json!(true), true), (json!(false), false)] {
        let m = dict(json!({WORLD_MAP_LANTERN_FLICKER: v}));
        assert_eq!(source::lantern_flicker_of(&m), on, "{v}");
        assert!(source::manifest_problems(&m).is_empty(), "{v}");
    }
    let problem = format!("game.json: {WORLD_MAP_LANTERN_FLICKER} must be true or false.");
    for bad in [json!("yes"), json!(1), json!(null), json!({})] {
        let m = dict(json!({WORLD_MAP_LANTERN_FLICKER: bad}));
        assert!(!source::lantern_flicker_of(&m), "{bad}: off");
        assert_eq!(source::lantern_flicker_problems(&m), [problem.clone()], "{bad}");
        assert!(source::manifest_problems(&m).contains(&problem), "{bad}: a manifest problem");
    }
}

#[test]
fn the_loader_carries_it_and_the_writer_sets_checks_and_removes_it() {
    let (_t, off) = scratch("off", json!({}));
    assert!(!source::load(&off).world_map_lantern_flicker, "no key: off");
    let (t, data) = scratch("set", json!({}));
    let path = t.join("game.json");
    store::set_manifest(&path, &dict(json!({WORLD_MAP_LANTERN_FLICKER: true}))).unwrap();
    assert!(source::load(&data).world_map_lantern_flicker, "set on: the loader reads it");
    let before = fs::read_to_string(&path).unwrap();
    assert_eq!(store::set_manifest(&path, &dict(json!({WORLD_MAP_LANTERN_FLICKER: "on"}))).unwrap_err(), [format!("game.json: {WORLD_MAP_LANTERN_FLICKER} must be true or false.")]);
    assert_eq!(fs::read_to_string(&path).unwrap(), before, "nothing written");
    store::set_manifest(&path, &dict(json!({WORLD_MAP_LANTERN_FLICKER: null}))).unwrap();
    assert!(!source::load(&data).world_map_lantern_flicker, "removed: off");
    assert!(store::check(&data).iter().all(|l| !l.contains(WORLD_MAP_LANTERN_FLICKER)));
}
