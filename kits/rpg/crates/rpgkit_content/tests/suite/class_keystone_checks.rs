//! CLASSKEY1 (`PR6-14`) content checks and schema persistence on scratch data (no shipped data): a class's optional
//! `keystone` is `{id, level}` (`id` a passive with `slot: "keystone"`, `level` a whole number of 1 or more), omitted
//! from the file when unset, read by the loader as `ClassDef::keystone`.

use aegis_core::json::{self, Dict};
use aegis_core::scratch::Scratch;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn setup(tag: &str) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_class_keystone_checks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let kit_body = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/units/dummy/base");
    let body = t.join("assets/units/dummy/base");
    fs::create_dir_all(&body).unwrap();
    for f in ["clips.json", "sheet.png"] {
        fs::copy(kit_body.join(f), body.join(f)).unwrap();
    }
    write_kind(&data, "item_families", 1, json!([
        {"id": "sword", "name": "Sword", "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"], "damage_type": "slash",
         "base": {"main_1h": {"power": 10, "range": 1, "burden": 0, "mitigation": 0}}}
    ]));
    write_kind(&data, "skills", 2, json!([
        {"id": "slash", "name": "Slash", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "target_mode": "ENEMY", "weapon_types": ["sword"]}
    ]));
    write_kind(&data, "passives", 1, json!([
        {"id": "p_key", "name": "Key", "slot": "keystone", "effects": [{"type": "stat", "stat": "CONS", "value": 2}]},
        {"id": "p_slotted", "name": "Slotted", "slot": "passive", "effects": [{"type": "stat", "stat": "STR", "value": 1}]},
        {"id": "p_always", "name": "Always", "slot": "", "effects": [{"type": "stat", "stat": "DEX", "value": 1}]}
    ]));
    write_kind(&data, "classes", 1, json!([]));
    (t, data)
}

fn class(id: &str, keystone: Value) -> Dict {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let growth = json!({"STR": 1, "DEX": 1, "INT": 1, "CONS": 1, "PERC": 1, "KNOW": 0});
    let mut o = dict(json!({"id": id, "name": id, "art": "dummy", "base_stats": stats, "growth": growth,
        "proficiency": {"sword": {"level": 1, "scaling": 1.5}}, "skill_path": {"slash": 1}}));
    if !keystone.is_null() {
        o.insert("keystone".into(), keystone);
    }
    o
}

#[test]
fn a_class_keystone_is_stored_only_when_set_and_read_by_the_loader() {
    let (_t, data) = setup("store");
    store::upsert(&data, None, "classes", &class("plain", Value::Null), None).unwrap();
    let text = fs::read_to_string(data.join("catalog/classes.json")).unwrap();
    assert!(!text.contains("\"keystone\""), "an unset class keystone is omitted: {text}");
    assert_eq!(store::resave_text(&data, None, "classes").unwrap(), text);

    store::upsert(&data, None, "classes", &class("keyed", json!({"id": "p_key", "level": 6})), None).unwrap();
    let text = fs::read_to_string(data.join("catalog/classes.json")).unwrap();
    assert!(text.contains("\"keystone\""), "a set class keystone is saved: {text}");
    assert_eq!(store::resave_text(&data, None, "classes").unwrap(), text);

    let v = store::view(&data);
    let k = v.typed.classes["keyed"].keystone.clone().expect("keyed has a keystone");
    assert_eq!((k.id.as_str(), k.level), ("p_key", 6));
    assert!(v.typed.classes["plain"].keystone.is_none());
}

#[test]
fn a_class_keystone_names_a_keystone_passive_at_level_one_or_more() {
    let (_t, data) = setup("checks");
    let v = store::view(&data);
    let errs = |k: Value| check::validate_entry(&v, "classes", &class("test_cls", k), "test_cls");
    let has = |k: Value, part: &str| errs(k).iter().any(|e| e.contains("Class keystone") && e.contains(part));

    for ok in [json!({"id": "p_key", "level": 6}), json!({"id": "p_key", "level": 1}), json!({}), Value::Null] {
        assert!(!errs(ok.clone()).iter().any(|e| e.contains("Class keystone")), "{ok}: {:?}", errs(ok.clone()));
    }
    assert!(has(json!({"id": "p_slotted", "level": 6}), "slot 'passive'"));
    assert!(has(json!({"id": "p_always", "level": 6}), "slot ''"));
    assert!(has(json!({"id": "no_such", "level": 6}), "no_such"));
    assert!(has(json!({"level": 6}), "id must name"));
    assert!(has(json!({"id": "p_key", "level": 0}), "level"));
    assert!(has(json!({"id": "p_key"}), "level"));
    assert!(has(json!({"id": "p_key", "level": 2.5}), "level"));
    assert!(has(json!({"id": "p_key", "level": 6, "tier": 1}), "unknown key 'tier'"));
    assert!(has(json!(["p_key"]), "object"));
}
