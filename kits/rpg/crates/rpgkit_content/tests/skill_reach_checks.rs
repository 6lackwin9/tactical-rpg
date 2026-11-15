//! PR5-50 (`KIT-HOOKS-CLEANUP`, FB-204) content side, on a scratch data folder built here (no shipped data): the passive
//! and item effect `skill_reach {reach 1-2, intent?, kind?}` (intent one of the round intents IR21 derives, kind an
//! authorable skill kind or SPELL, no other key) checks clean and each mistake is one error; the retired `spell_reach
//! {value}` still checks (a load alias); the unknown-type lists name `skill_reach`. The writer: a re-save without changes
//! is byte-identical (a stored `spell_reach` stays until its row is written), and a row the write changes stores its
//! `spell_reach` effects as `skill_reach {reach, kind: SPELL}` (a passive's `effects`, an item's `effect` modifiers),
//! leaving the untouched rows as they were.

use aegis_core::json::Dict;
use rpgkit_content::check::simple::{effect_errors, SKILL_REACH_INTENTS, SKILL_REACH_KINDS};
use rpgkit_content::{check, store};
use rpgkit_data::{SKILL_REACH, SKILL_REACH_SPELL, SPELL_REACH_ALIAS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_pr550_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().unwrap().clone()
}

fn reach(extra: Value) -> Value {
    let mut row = json!({"type": SKILL_REACH});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

fn old(value: i64) -> Value {
    json!({"type": SPELL_REACH_ALIAS, "value": value})
}

/// The errors of a one-effect list, as a passive's (`items_only` false) and as gear's (true).
fn errors(fx: Value) -> (Vec<String>, Vec<String>) {
    let (_scratch, data) = scratch("ctx");
    let v = store::view(&data);
    let hooks = check::hooks::Ctx::of(&v);
    let list = json!([fx]);
    let run = |items: bool| effect_errors(&list, &|_| true, &[], false, items, &hooks);
    (run(false), run(true))
}

fn one(fx: Value, want: &str) {
    let (p, i) = errors(fx);
    assert_eq!(p, vec![want.to_string()], "as a passive");
    assert_eq!(i, vec![want.to_string()], "as gear");
}

#[test]
fn skill_reach_checks_clean_with_any_filters_and_the_alias_still_checks() {
    let clean = (Vec::<String>::new(), Vec::<String>::new());
    assert_eq!(errors(reach(json!({"reach": 1}))), clean, "no filter");
    for kind in SKILL_REACH_KINDS {
        assert_eq!(errors(reach(json!({"reach": 2, "kind": kind}))), clean, "kind {kind}");
    }
    for intent in SKILL_REACH_INTENTS {
        assert_eq!(errors(reach(json!({"reach": 1, "intent": intent, "kind": SKILL_REACH_SPELL}))), clean, "intent {intent}");
    }
    assert_eq!(errors(reach(json!({"reach": 1, "intent": "", "kind": ""}))), clean, "empty filters match every skill");
    assert_eq!(errors(old(1)), clean, "the alias");
    one(old(3), "Effect 1: spell reach is 1-2 tiles.");
}

#[test]
fn each_skill_reach_mistake_is_one_error() {
    one(reach(json!({"reach": 0})), "Effect 1: skill_reach reach must be a whole number 1-2 (tiles).");
    one(reach(json!({"reach": 3})), "Effect 1: skill_reach reach must be a whole number 1-2 (tiles).");
    one(reach(json!({"reach": 1.5})), "Effect 1: skill_reach reach must be a whole number 1-2 (tiles).");
    one(reach(json!({})), "Effect 1: skill_reach reach must be a whole number 1-2 (tiles).");
    one(reach(json!({"reach": 1, "intent": "channel"})), &format!("Effect 1: skill_reach intent 'channel' is not one of {}.", SKILL_REACH_INTENTS.join(", ")));
    one(reach(json!({"reach": 1, "kind": "MOVE"})), &format!("Effect 1: skill_reach kind 'MOVE' is not one of {}.", SKILL_REACH_KINDS.join(", ")));
    one(reach(json!({"reach": 1, "value": 1})), "Effect 1: skill_reach takes only reach, intent, kind and ranged (not 'value').");
    let (p, i) = errors(json!({"type": "zz"}));
    assert!(p[0].contains(SKILL_REACH) && !p[0].contains(SPELL_REACH_ALIAS), "the passive list names skill_reach: {p:?}");
    assert!(i[0].contains(SKILL_REACH) && !i[0].contains(SPELL_REACH_ALIAS), "the item list names skill_reach: {i:?}");
}

/// A focus family, two items and two passives holding the alias (one of each to be written, one of each untouched).
fn catalog(data: &Path) {
    write_kind(data, "item_families", 1, json!([
        {"id": "orb", "name": "Orb", "item_type": "WEAPON", "hands": ["off"], "default_hand": "off", "scaling_stats": ["INT"],
         "base": {"off": {"power": 0, "range": 1, "burden": 0, "mitigation": 0}},
         "bands": {"off": {"power": {"min": 0, "max": 4}, "range": {"min": 1, "max": 1}, "burden": {"min": 0, "max": 2}, "mitigation": {"min": 0, "max": 0}, "bonus_value": {"min": 0, "max": 6}}},
         "adjustable": ["power"], "modifier_kinds": ["adjust", "stat_bonus", "requirement", "effect"], "max_modifiers": 4, "max_tiers": 1, "damage_type": "magic"}
    ]));
    let item = |id: &str| json!({"id": id, "name": id, "family": "orb", "hand": "off", "modifiers": [{"kind": "effect", "effect": old(1)}], "tiers": []});
    write_kind(data, "items", 2, json!([item("t_orb"), item("t_lens")]));
    let passive = |id: &str| json!({"id": id, "name": id, "effects": [{"type": "stat", "stat": "INT", "value": 1}, old(1)]});
    write_kind(data, "passives", 1, json!([passive("p_reach"), passive("p_keep")]));
}

/// The folder problems naming the rows here (the scratch folder has no engine skills, so it has others).
fn ours(problems: &[String]) -> Vec<String> {
    problems.iter().filter(|p| ["orb", "t_orb", "t_lens", "p_reach", "p_keep", "reach"].iter().any(|k| p.contains(k))).cloned().collect()
}

fn stored(data: &Path, kind: &str, id: &str) -> Value {
    let doc: Value = serde_json::from_str(&fs::read_to_string(data.join("catalog").join(format!("{kind}.json"))).unwrap()).unwrap();
    doc["entries"].as_array().unwrap().iter().find(|e| e["id"] == id).cloned().unwrap_or_else(|| panic!("{kind} {id}"))
}

#[test]
fn the_writer_keeps_untouched_rows_and_stores_a_written_row_as_skill_reach() {
    let (_scratch, data) = scratch("writer");
    catalog(&data);
    for kind in ["items", "passives"] {
        let text = store::resave_text(&data, None, kind).unwrap();
        fs::write(data.join("catalog").join(format!("{kind}.json")), &text).unwrap();
        assert_eq!(store::resave_text(&data, None, kind).unwrap(), text, "{kind}: a re-save without changes is byte-identical");
        assert!(text.contains(SPELL_REACH_ALIAS), "{kind}: the stored alias stays until its row is written");
    }
    assert_eq!(ours(&store::check(&data)), Vec::<String>::new());
    // write one item and one passive as they are stored (the alias in them)
    let new_fx = json!({"type": SKILL_REACH, "reach": 1, "kind": SKILL_REACH_SPELL});
    for (kind, id) in [("items", "t_orb"), ("passives", "p_reach")] {
        let row = dict(stored(&data, kind, id));
        let out = store::upsert(&data, None, kind, &row, None);
        assert!(out.is_ok(), "{kind} {id}: {:?}", out.err());
    }
    assert_eq!(stored(&data, "items", "t_orb")["modifiers"][0]["effect"], new_fx, "the written item's modifier");
    assert_eq!(stored(&data, "passives", "p_reach")["effects"], json!([{"type": "stat", "stat": "INT", "value": 1}, new_fx]), "the written passive");
    assert_eq!(stored(&data, "items", "t_lens")["modifiers"][0]["effect"], old(1), "an untouched item keeps its stored form");
    assert_eq!(stored(&data, "passives", "p_keep")["effects"][1], old(1), "an untouched passive too");
    // the written rows re-save byte for byte and check clean; the item still prices the same
    for kind in ["items", "passives"] {
        let text = fs::read_to_string(data.join("catalog").join(format!("{kind}.json"))).unwrap();
        assert_eq!(store::resave_text(&data, None, kind).unwrap(), text, "{kind}: re-saves byte for byte after the write");
    }
    assert_eq!(ours(&store::check(&data)), Vec::<String>::new());
    let v = store::view(&data);
    let price = |id: &str| rpgkit_rules::budget::item_value(&v.typed.items[id]);
    assert_eq!(price("t_orb"), price("t_lens"), "the new form and the alias price the same");
    // a row given in the new form is stored as given
    let row = dict(stored(&data, "items", "t_orb"));
    assert!(store::upsert(&data, None, "items", &row, None).is_ok());
    assert_eq!(stored(&data, "items", "t_orb")["modifiers"][0]["effect"], new_fx);
}
