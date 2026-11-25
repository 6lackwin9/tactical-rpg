//! The shake schema slice (FX1-S, spec FX1 5.1; checks S1-S3 of FX1_TASKS 1.2): a
//! vignette card's `shake` (S1), the phase effect `{"type": "shake", "preset"}` (S2) and its parse, which keeps the
//! preset and changes nothing in battle (S3). Scratch data folders only.

use aegis_core::json::{self, Dict};
use indexmap::IndexMap;
use rpgkit_battle::encounter::parse_phases;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, UnitDef, SHAKE_PRESETS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const W: &str = "ward";

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_shake_schema_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn write_catalog(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn write_world_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(store::file_of(data, Some(W), kind), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

/// A world with one location and a vignette of two text cards.
fn story_world(tag: &str) -> PathBuf {
    let (scratch_dir, data) = scratch(tag);
    scratch_dir.until_thread_end();
    let manifest = json!({"id": W, "name": "Ward", "extends": [], "connects": [], "start_location": "hub", "version": 1});
    fs::write(aegis_content::world::manifest_path(&data, W), manifest.to_string()).unwrap();
    write_world_kind(&data, "locations", json!([{"id": "hub", "name": "Hub"}]));
    write_world_kind(&data, "vignettes", json!([reel()]));
    data
}

fn reel() -> Value {
    json!({"id": "reel", "name": "Reel", "cards": [{"style": "text", "text": "First."}, {"style": "text", "text": "Second."}]})
}

fn reel_with_shake(shake: Value) -> Value {
    let mut r = reel();
    r["cards"][0]["shake"] = shake;
    r
}

fn vignette_errors(v: &check::View, entry: &Value) -> Vec<String> {
    check::world::validate_entry(v, W, "vignettes", entry.as_object().unwrap(), "reel")
}

#[test]
fn a_card_shake_is_a_preset_or_nothing_and_unset_writes_no_key() {
    let data = story_world("card");
    let v = store::view(&data);
    assert_eq!(vignette_errors(&v, &reel()), Vec::<String>::new(), "the fixture checks clean");
    for preset in SHAKE_PRESETS.iter().chain([&""]) {
        assert_eq!(vignette_errors(&v, &reel_with_shake(json!(preset))), Vec::<String>::new(), "shake '{preset}'");
    }
    for bad in [json!("boom"), json!(3)] {
        let errors = vignette_errors(&v, &reel_with_shake(bad.clone()));
        assert_eq!(errors.len(), 1, "{bad}: {errors:?}");
        assert!(errors[0].starts_with("Card 1: shake ") && errors[0].contains(&SHAKE_PRESETS.join(", ")), "{bad}: {errors:?}");
    }
    // the writer: a card without shake stores no key; a card with one keeps it
    store::upsert(&data, Some(W), "vignettes", &dict(json!({"id": "plain", "name": "Plain", "cards": [{"style": "text", "text": "Calm."}]})), None).expect("a card without shake");
    store::upsert(&data, Some(W), "vignettes", &dict(json!({"id": "toll", "name": "Toll", "cards": [{"style": "text", "text": "Bells.", "shake": "toll"}]})), None).expect("a toll card");
    let file = fs::read_to_string(store::file_of(&data, Some(W), "vignettes")).unwrap();
    let stored: Value = serde_json::from_str(&file).unwrap();
    let card = |id: &str| -> Dict {
        let e = json::aget(stored.as_object().unwrap(), "entries").iter().map(json::dict).find(|e| json::s(e, "id") == id).cloned().unwrap_or_else(|| panic!("{id} stored"));
        json::dict(&json::aget(&e, "cards")[0]).clone()
    };
    assert_eq!(card("plain").get("shake"), None, "no shake key on a card without one");
    assert_eq!(card("toll").get("shake"), Some(&json!("toll")));
    drop_scratch(&data);
}

/// A class, the system skills and a basic attack, one hero and one enemy, and the encounter `quaking` with `phases`.
fn catalog(data: &Path, phases: Value) {
    let skill = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_catalog(data, "skills", 2, json!([skill("move", "MOVE", json!({"system": true})), skill("rest", "REST", json!({"system": true})), skill("strike", "ATTACK", json!({"basic": true}))]));
    write_catalog(
        data,
        "classes",
        1,
        json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2, "CONS": 2, "DEX": 1}}]),
    );
    write_catalog(
        data,
        "units",
        1,
        json!([{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}, {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}]),
    );
    write_catalog(data, "encounters", 1, json!([{"id": "quaking", "name": "Quaking", "party": ["hero"], "enemies": ["brute"], "phases": phases}]));
}

/// The phase `tremor` at `round`: a line, then `extra` effects.
fn tremor(round: i64, extra: &[Value]) -> Value {
    let mut effects = vec![json!({"type": "line", "text": "The ground rolls."})];
    effects.extend(extra.iter().cloned());
    json!([{"id": "tremor", "name": "Tremor", "trigger": {"kind": "round", "round": round}, "effects": effects}])
}

fn encounter_errors(v: &check::View, phases: Value) -> Vec<String> {
    let mut e = v.get("encounters", "quaking").clone();
    assert!(!e.is_empty(), "fixture encounter");
    e.insert("phases".into(), phases);
    check::validate_entry(v, "encounters", &e, "quaking")
}

#[test]
fn a_phase_shake_needs_one_known_preset_and_comes_once() {
    let (_scratch, data) = scratch("phase");
    catalog(&data, tremor(2, &[]));
    let v = store::view(&data);
    let base = encounter_errors(&v, tremor(2, &[]));
    assert_eq!(base, Vec::<String>::new(), "the fixture checks clean");
    for preset in SHAKE_PRESETS {
        assert_eq!(encounter_errors(&v, tremor(2, &[json!({"type": "shake", "preset": preset})])), Vec::<String>::new(), "preset {preset}");
    }
    let needs = format!("Phase 'tremor': a shake needs a preset, one of {}.", SHAKE_PRESETS.join(", "));
    let cases: Vec<(Value, String)> = vec![
        (json!({"type": "shake"}), needs.clone()),
        (json!({"type": "shake", "preset": "boom"}), needs.clone()),
        (json!({"type": "shake", "preset": ""}), needs.clone()),
        (json!({"type": "shake", "preset": 2}), needs.clone()),
        (json!({"type": "shake", "preset": "toll", "px": 4}), "Phase 'tremor': unknown shake key 'px'.".to_string()),
    ];
    for (fx, want) in cases {
        assert_eq!(encounter_errors(&v, tremor(2, std::slice::from_ref(&fx))), vec![want], "{fx}");
    }
    let two = encounter_errors(&v, tremor(2, &[json!({"type": "shake", "preset": "toll"}), json!({"type": "shake", "preset": "tremor"})]));
    assert_eq!(two, vec!["Phase 'tremor': one shake per phase.".to_string()]);
    drop_scratch(&data);
}

/// The battle of `quaking` with `phases`. The battle fixtures put the phase at round 1 (the check asks for 2 or later,
/// the session fires whatever it is given), so it fires as the session starts.
fn battle(phases: Value) -> Session {
    let (scratch_dir, data) = scratch("battle");
    scratch_dir.until_thread_end();
    catalog(&data, phases);
    let src = source::load(&data);
    drop_scratch(&data);
    let cat = Catalog::from_value(&src.catalog);
    let def = EncounterDef::from_obj(&cat.encounters["quaking"]);
    Session::start(Arc::new(cat), def, &IndexMap::<String, UnitDef>::new())
}

#[test]
fn the_parser_keeps_the_preset_and_a_shake_changes_nothing_in_battle() {
    let raw = tremor(1, &[json!({"type": "shake", "preset": "tremor"})]);
    let parsed = parse_phases(raw.as_array().unwrap());
    let shake = parsed[0].effects.iter().find(|fx| json::s(fx, "type") == "shake").expect("the shake effect is kept");
    assert_eq!(shake, &dict(json!({"type": "shake", "preset": "tremor"})));
    let missing = parse_phases(tremor(1, &[json!({"type": "shake"})]).as_array().unwrap());
    assert_eq!(json::s(&missing[0].effects[1], "preset"), "", "a missing preset reads as \"\"");

    let plain = battle(tremor(1, &[]));
    let shaken = battle(raw);
    assert!(plain.phases.fired.contains_key("tremor") && shaken.phases.fired.contains_key("tremor"), "the round-1 phase fired in both");
    assert!(!plain.log.is_empty() && !plain.events.is_empty(), "the phase wrote its log entry and event");
    assert_eq!(shaken.log, plain.log, "the log entries are the same");
    assert_eq!(shaken.events, plain.events, "the event texts are the same");
    assert_eq!(format!("{:?}", shaken.w.units), format!("{:?}", plain.w.units), "no unit changed");
}
