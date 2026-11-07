//! E2.2-S1 (FB-53a,b, E_ECONOMY.md section 2): the game-wide purse caps (`supplies_max`, `purse_gold_cap`) are optional
//! keys of the game's manifest (`game.json` beside the data folder). Missing, they read today's caps (the defaults);
//! a present key that is not a whole number 1 or more reads the default and is a manifest problem; the loader and the
//! check view carry them, and the world checks bound by them: start gold (also at most `START_GOLD_MAX`), start supplies,
//! the inherited default start supplies (`SUPPLIES_START`) when no world in the lineage sets one, and a chest's supplies.
//! Scratch data folders only; every expectation follows from the rule and the fixture's own numbers.

use aegis_core::json::Dict;
use rpgkit_content::check::{map, world, View};
use rpgkit_content::source::{self, PurseCaps, START_GOLD_MAX, SUPPLIES_START};
use rpgkit_content::store;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const SUPPLIES: i64 = 20;
const GOLD: i64 = 500;

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// A scratch game folder holding `game.json` (`manifest`, when given) and `data/` with empty catalog and worlds folders.
fn scratch(tag: &str, manifest: Option<Value>) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_purse_caps_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    if let Some(m) = manifest {
        fs::write(t.join("game.json"), serde_json::to_string_pretty(&m).unwrap()).unwrap();
    }
    (t, data)
}

fn game(extra: Value) -> Value {
    let mut m = json!({"id": "demo", "title": "Demo", "version": "0.1.0", "worlds": [], "start_worlds": [], "references": []});
    for (k, v) in extra.as_object().unwrap() {
        m[k] = v.clone();
    }
    m
}

#[test]
fn the_defaults_are_todays_caps() {
    let d = PurseCaps::default();
    assert_eq!((d.supplies_max, d.purse_gold_cap), (PurseCaps::SUPPLIES_MAX, PurseCaps::PURSE_GOLD_CAP));
    assert_eq!((PurseCaps::SUPPLIES_MAX, PurseCaps::PURSE_GOLD_CAP), (99, 9_999_999), "E_ECONOMY.md section 4 defaults");
    assert_eq!(PurseCaps::from_manifest(&Dict::new()), d, "a manifest without the keys");
    assert!(PurseCaps::problems(&Dict::new()).is_empty());
}

#[test]
fn a_manifest_sets_each_cap_and_a_bad_value_reads_the_default() {
    let set = PurseCaps::from_manifest(&dict(json!({"supplies_max": SUPPLIES, "purse_gold_cap": GOLD})));
    assert_eq!((set.supplies_max, set.purse_gold_cap), (SUPPLIES, GOLD));
    let floats = PurseCaps::from_manifest(&dict(json!({"supplies_max": SUPPLIES as f64, "purse_gold_cap": GOLD as f64})));
    assert_eq!(floats, set, "a file's whole floats read as whole numbers");
    for bad in [json!(0), json!(-3), json!(2.5), json!("many"), json!(null)] {
        let m = dict(json!({"supplies_max": bad, "purse_gold_cap": bad}));
        assert_eq!(PurseCaps::from_manifest(&m), PurseCaps::default(), "{bad}: the default");
        assert_eq!(
            PurseCaps::problems(&m),
            PurseCaps::KEYS.iter().map(|k| format!("game.json: {k} must be a whole number 1 or more.")).collect::<Vec<_>>(),
            "{bad}: a problem per key"
        );
    }
}

#[test]
fn the_loader_and_the_check_view_read_the_manifest_beside_the_data() {
    let (_t, data) = scratch("load", Some(game(json!({"supplies_max": SUPPLIES, "purse_gold_cap": GOLD}))));
    let want = PurseCaps { supplies_max: SUPPLIES, purse_gold_cap: GOLD };
    assert_eq!(source::load(&data).purse, want);
    assert_eq!(store::view(&data).purse, want);
    assert!(!store::check(&data).iter().any(|l| l.contains("supplies_max") || l.contains("purse_gold_cap")), "valid caps are no problem");
    let (_u, bare) = scratch("bare", None);
    assert_eq!(source::load(&bare).purse, PurseCaps::default(), "no manifest: the defaults");
    let (_v, plain) = scratch("plain", Some(game(json!({}))));
    assert_eq!(source::load(&plain).purse, PurseCaps::default(), "a manifest without the keys: the defaults");
}

#[test]
fn a_bad_cap_is_a_manifest_problem_of_the_data_check() {
    let (_t, data) = scratch("bad", Some(game(json!({"supplies_max": 0, "purse_gold_cap": "lots"}))));
    let lines = store::check(&data);
    for k in PurseCaps::KEYS {
        let want = format!("game.json: {k} must be a whole number 1 or more.");
        assert!(lines.iter().any(|l| l.ends_with(&want)), "{want} in {lines:?}");
    }
    assert_eq!(source::load(&data).purse, PurseCaps::default(), "and the loader reads the defaults");
}

/// The `prefix` errors of world `w`'s manifest holding `start` under `caps`.
fn start_errors(start: Value, caps: PurseCaps, prefix: &str) -> Vec<String> {
    let mut manifest = json!({"id": "w"});
    for (k, x) in start.as_object().unwrap() {
        manifest[k] = x.clone();
    }
    let worlds = json!({"w": {"manifest": manifest, "own": {}}});
    let mut v = View::new(&json!({}), &worlds, &json!({}), Default::default());
    v.purse = caps;
    world::manifest_errors(&v, "w").into_iter().filter(|e| e.starts_with(prefix)).collect()
}

fn manifest_errors(start_supplies: i64, caps: PurseCaps) -> Vec<String> {
    start_errors(json!({"start_supplies": start_supplies}), caps, "Start supplies")
}

#[test]
fn start_supplies_fit_the_games_supply_cap() {
    let low = PurseCaps { supplies_max: SUPPLIES, ..PurseCaps::default() };
    let max = PurseCaps::SUPPLIES_MAX;
    assert!(manifest_errors(max, PurseCaps::default()).is_empty(), "today's cap");
    assert_eq!(manifest_errors(max + 1, PurseCaps::default()), [format!("Start supplies must be -1 (inherited) or 0..{max}.")], "today's text");
    assert!(manifest_errors(SUPPLIES, low).is_empty(), "at the lower cap");
    assert_eq!(manifest_errors(SUPPLIES + 1, low), [format!("Start supplies must be -1 (inherited) or 0..{SUPPLIES}.")]);
    assert!(manifest_errors(-1, low).is_empty(), "inherited");
}

#[test]
fn start_gold_fits_the_games_gold_cap() {
    let gold = |g: i64, caps: PurseCaps| start_errors(json!({"start_gold": g}), caps, "Start gold");
    let low = PurseCaps { purse_gold_cap: GOLD, ..PurseCaps::default() };
    assert!(gold(START_GOLD_MAX, PurseCaps::default()).is_empty(), "today's bound");
    assert_eq!(gold(START_GOLD_MAX + 1, PurseCaps::default()), [format!("Start gold must be -1 (inherited) or 0..{START_GOLD_MAX}.")], "today's text");
    assert!(gold(GOLD, low).is_empty(), "at the lower cap");
    assert_eq!(gold(GOLD + 1, low), [format!("Start gold must be -1 (inherited) or 0..{GOLD}.")]);
    assert!(gold(-1, low).is_empty(), "inherited (the default start gold is 0)");
}

#[test]
fn the_inherited_default_start_supplies_fit_the_cap() {
    let inherit = |caps: PurseCaps| start_errors(json!({}), caps, "Start supplies");
    assert!(inherit(PurseCaps::default()).is_empty(), "today's cap holds the default start");
    assert!(inherit(PurseCaps { supplies_max: SUPPLIES_START, ..PurseCaps::default() }).is_empty(), "a cap equal to the default start");
    let tight = SUPPLIES_START - 1;
    assert_eq!(
        inherit(PurseCaps { supplies_max: tight, ..PurseCaps::default() }),
        [format!("Start supplies inherit the default {SUPPLIES_START}, above the game's supply cap {tight}; set start_supplies 0..{tight}.")]
    );
    assert!(manifest_errors(tight, PurseCaps { supplies_max: tight, ..PurseCaps::default() }).is_empty(), "a world that sets its start inside the cap");
}

/// The supplies errors of a location holding one chest with `supplies` under `caps`.
fn chest_errors(supplies: i64, caps: PurseCaps) -> Vec<String> {
    let template = json!({"id": "m", "name": "M", "rows": ["...."], "zones": [], "zone_legend": {}, "anchors": [{"id": "a1", "x": 1, "y": 0}], "paths": {}, "props": [], "tags": []});
    let worlds = json!({"w": {"manifest": {"id": "w"}, "own": {}}});
    let mut v = View::new(&json!({}), &worlds, &json!({"maps": [template]}), Default::default());
    v.purse = caps;
    let location = dict(json!({"id": "hub", "map": "m", "markers": [{"id": "c1", "kind": "chest", "anchor": "a1", "supplies": supplies}]}));
    map::location_errors(&v, "w", &location).into_iter().filter(|e| e.contains("supplies must")).collect()
}

#[test]
fn a_chest_holds_no_more_supplies_than_the_cap() {
    let max = PurseCaps::SUPPLIES_MAX;
    let low = PurseCaps { supplies_max: SUPPLIES, ..PurseCaps::default() };
    assert!(chest_errors(max, PurseCaps::default()).is_empty(), "today's cap");
    assert_eq!(chest_errors(max + 1, PurseCaps::default()), [format!("Marker 'c1': supplies must be a whole number 0..{max}.")], "today's text");
    assert!(chest_errors(SUPPLIES, low).is_empty(), "at the lower cap");
    assert_eq!(chest_errors(SUPPLIES + 1, low), [format!("Marker 'c1': supplies must be a whole number 0..{SUPPLIES}.")]);
}
