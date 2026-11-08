//! CLASS-BASICS (READY row R2-W2b, user 2026-11-08) content side, scratch data folders only: a class row's `basics`
//! (skill ids) is a schema field stored only when set, so a class without it re-saves byte for byte; it reads into the
//! typed `ClassDef::basics`. Each id names a known tier 1 skill that is not an engine skill, none twice. Under a deck
//! ruleset a unit's class basics are basics: never a deck active (no slot), and its deck order may name them, while a
//! unit of another class may not. A class basic also in the class's skill path is warned (it takes a path slot).

use aegis_core::json::Dict;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().unwrap().clone()
}

fn stats() -> Value {
    json!({"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10})
}

fn growth() -> Value {
    json!({"STR": 2, "CONS": 2, "DEX": 1})
}

fn skills() -> Value {
    json!([
        {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 2, "basic": true},
        {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
        {"id": "brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 3, "learned_only": true, "round_intent": "guard", "mitigation": 0.2},
        {"id": "flank_cut", "name": "Flank Cut", "kind": "ATTACK", "level": 1, "blocks": 1, "round_intent": "maneuver", "power_mult": 0.6},
        {"id": "slash", "name": "Slash", "kind": "ATTACK", "level": 1, "blocks": 2, "power_mult": 1.0, "ranks": [{"level": 3, "power_mult": 1.2}]}
    ])
}

/// A scratch game with the kit's dummy body (a class must name one) and the skills above.
fn scratch(name: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(name);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    let kit_body = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/units/dummy/base");
    let body = t.join("assets/units/dummy/base");
    fs::create_dir_all(&body).unwrap();
    for f in ["clips.json", "sheet.png"] {
        fs::copy(kit_body.join(f), body.join(f)).unwrap();
    }
    // a version 2 skills file (families and ranks)
    fs::write(data.join("catalog/skills.json"), json!({"kind": "skills", "version": 2, "entries": skills()}).to_string()).unwrap();
    (t, data)
}

fn class(id: &str, extra: Value) -> Dict {
    let mut row = dict(json!({"id": id, "name": id, "art": "dummy", "base_stats": stats(), "growth": growth(), "proficiency": {"sword": {"level": 1, "scaling": 1.5}}, "skill_path": {"slash": 1}}));
    row.extend(dict(extra));
    row
}

#[test]
fn class_basics_are_stored_only_when_set() {
    let (_t, data) = scratch("rpgkit_class_basics_store");
    store::upsert(&data, None, "classes", &class("plain", json!({})), None).unwrap_or_else(|e| panic!("saves: {e:?}"));
    let text = fs::read_to_string(data.join("catalog/classes.json")).unwrap();
    assert!(!text.contains("\"basics\""), "no basics key while unset: {text}");
    assert_eq!(store::resave_text(&data, None, "classes").unwrap(), text, "re-saves byte for byte");

    store::upsert(&data, None, "classes", &class("guard", json!({"basics": ["brace"]})), None).unwrap_or_else(|e| panic!("saves: {e:?}"));
    let text = fs::read_to_string(data.join("catalog/classes.json")).unwrap();
    let saved: Value = serde_json::from_str(&text).unwrap();
    let rows = saved["entries"].as_array().unwrap();
    let stored = rows.iter().find(|e| e["id"] == "guard").unwrap();
    assert_eq!(stored["basics"], json!(["brace"]), "{stored}");
    assert!(rows.iter().find(|e| e["id"] == "plain").unwrap().get("basics").is_none());
    assert_eq!(store::resave_text(&data, None, "classes").unwrap(), text, "re-saves byte for byte with basics");
    let v = store::view(&data);
    assert_eq!(v.typed.classes["guard"].basics, ["brace"]);
    assert!(v.typed.classes["plain"].basics.is_empty());
}

#[test]
fn class_basics_name_known_tier_one_skills_once() {
    let (_t, data) = scratch("rpgkit_class_basics_check");
    store::upsert(&data, None, "classes", &class("plain", json!({})), None).unwrap();
    let v = store::view(&data);
    let errors = |basics: Value| check::validate_entry(&v, "classes", &class("drill", json!({"basics": basics})), "drill");
    let base = errors(json!([]));
    let added = |basics: Value| errors(basics).into_iter().filter(|e| !base.contains(e)).collect::<Vec<_>>();
    assert!(added(json!(["brace", "strike"])).is_empty(), "a skill and a catalog basic are fine");
    assert_eq!(added(json!(["brace", "brace"])), ["Class basics lists 'brace' twice."], "the schema's list check");
    assert_eq!(added(json!(["slash_2"])), ["Class basics: 'slash_2' is a higher tier; list the tier 1 skill of the family."]);
    assert_eq!(added(json!(["move"])), ["Class basics: 'move' can't be used here."], "the schema's filter (no engine skill)");
    let unknown = store::upsert(&data, None, "classes", &class("drill", json!({"basics": ["no_such_skill"]})), None).err().expect("an unknown skill is refused");
    assert!(unknown.iter().any(|e| e.contains("no_such_skill")), "{unknown:?}");
}

#[test]
fn a_class_basic_in_the_path_is_warned() {
    let (_t, data) = scratch("rpgkit_class_basics_warn");
    let v = store::view(&data);
    let warned = |row: Dict| check::warnings_for(&v, "classes", &row).into_iter().filter(|w| w.starts_with("Class basic ")).collect::<Vec<_>>();
    assert!(warned(class("drill", json!({"basics": ["brace"]}))).is_empty());
    let both = class("drill", json!({"basics": ["brace"], "skill_path": {"slash": 1, "brace": 1}}));
    assert_eq!(warned(both), ["Class basic 'brace' is also in the skill path, where it takes a path slot (an auto loadout fills slots in path order)."]);
}

#[test]
fn under_a_deck_ruleset_a_class_basic_takes_no_slot_and_may_be_ordered() {
    let (_t, data) = scratch("rpgkit_class_basics_deck");
    write_kind(&data, "classes", json!([class("cutter", json!({"basics": ["flank_cut"]})), class("plain", json!({}))]));
    write_kind(&data, "rulesets", json!([{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "deck"], "basics": ["strike"], "deck_slots": {"active": 2, "passive": 0, "keystone": 0}}]));
    let v = store::view(&data);
    let unit = |class: &str, deck: Value| dict(json!({"id": "u1", "name": "U", "class_id": class, "level": 1, "side": "enemy", "deck": deck}));
    let deck_errors = |row: Dict| check::validate_entry(&v, "units", &row, "u1").into_iter().filter(|e| e.starts_with("Deck")).collect::<Vec<_>>();
    let none: Vec<String> = Vec::new();
    assert_eq!(deck_errors(unit("cutter", json!({"order": {"maneuver": ["flank_cut"]}}))), none, "its class basic may be ordered");
    assert_eq!(deck_errors(unit("cutter", json!({"active": ["flank_cut"]}))), ["Deck active: 'flank_cut' is a basic (open to every unit, no slot)."]);
    let other = deck_errors(unit("plain", json!({"order": {"maneuver": ["flank_cut"]}})));
    assert!(other.iter().any(|e| e.contains("flank_cut")), "another class may not order it: {other:?}");
}
