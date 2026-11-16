//! PK3 revive checks on `skills.revive` and `items.revive` using a scratch data directory (story-free).

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{self, skill};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_revive_checks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn catalog(data: &Path) {
    let one = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(
        data,
        "skills",
        2,
        json!([
            one("move", "MOVE", json!({"system": true})),
            one("rest", "REST", json!({"system": true})),
            one("strike", "ATTACK", json!({"basic": true})),
            one("jab", "ATTACK", json!({})),
            one("t_revive", "SUPPORT", json!({"target_mode": "ALLY", "heal_base": 12, "revive": true}))
        ]),
    );
    let band = |lo: i32, hi: i32| json!({"min": lo, "max": hi});
    write_kind(
        data,
        "item_families",
        1,
        json!([
            {"id": "t_sword", "name": "Sword", "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"],
             "base": {"main_1h": {"power": 14, "range": 1, "block_cost": 1, "burden": 0, "mitigation": 0}},
             "bands": {"main_1h": {"power": band(10, 23), "range": band(1, 2), "block_cost": band(1, 3), "burden": band(0, 4), "mitigation": band(0, 0), "bonus_value": band(0, 4)}},
             "adjustable": ["power"], "modifier_kinds": ["adjust"], "max_modifiers": 4, "max_tiers": 3, "damage_type": "slash"},
            {"id": "consumable", "name": "Consumable", "item_type": "CONSUMABLE"}
        ]),
    );
    write_kind(
        data,
        "items",
        2,
        json!([
            {"id": "t_salts", "name": "Revive Salts", "family": "consumable", "consumable_type": "HEALTH_POTION", "heal": 15, "damage": 0, "use_blocks": 1, "use_reach": 1, "revive": true, "value": 15},
            {"id": "t_blade", "name": "Short Blade", "family": "t_sword", "hand": "main_1h", "value": 10}
        ]),
    );
    write_kind(
        data,
        "classes",
        1,
        json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {}, "proficiency": {}, "skill_path": {"jab": 1, "t_revive": 1}}]),
    );
}

fn field_of(kind: &str, key: &str) -> Dict {
    fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("content.{kind}.{key}")).clone()
}

#[test]
fn schema_and_skill_fields_agree_on_revive() {
    let sf = field_of("skills", "revive");
    assert_eq!(json::s(&sf, "type"), "bool");
    assert_eq!(json::get(&sf, "default"), &json!(false));
    assert!(!json::b(&sf, "rank", false));
    assert_eq!(skill::KINDS.iter().copied().filter(|k| skill::has_field(k, "revive")).collect::<Vec<_>>(), ["SUPPORT"]);

    let itf = field_of("items", "revive");
    assert_eq!(json::s(&itf, "type"), "bool");
    assert_eq!(json::get(&itf, "default"), &json!(false));
}

#[test]
fn skill_and_item_revive_checks_and_store_roundtrip() {
    let (_s, data) = scratch("valid");
    catalog(&data);
    let v = store::view(&data);

    let mut sk = v.get("skills", "t_revive").clone();
    assert_eq!(check::validate_entry(&v, "skills", &sk, "t_revive"), Vec::<String>::new());

    sk.insert("target_mode".into(), json!("SELF"));
    assert!(!check::validate_entry(&v, "skills", &sk, "t_revive").is_empty(), "SELF cannot revive");

    let mut no_heal = v.get("skills", "t_revive").clone();
    no_heal.remove("heal_base");
    assert!(!check::validate_entry(&v, "skills", &no_heal, "t_revive").is_empty(), "revive skill requires a heal");

    let mut atk = v.get("skills", "jab").clone();
    atk.insert("revive".into(), json!(true));
    assert!(!check::validate_entry(&v, "skills", &atk, "jab").is_empty(), "ATTACK rejects revive");

    let mut salts = v.get("items", "t_salts").clone();
    assert_eq!(check::validate_entry(&v, "items", &salts, "t_salts"), Vec::<String>::new());

    salts.insert("use_reach".into(), json!(0));
    assert!(!check::validate_entry(&v, "items", &salts, "t_salts").is_empty(), "revive item needs use_reach >= 1");

    let mut dmg = v.get("items", "t_salts").clone();
    dmg.insert("damage".into(), json!(5));
    assert!(!check::validate_entry(&v, "items", &dmg, "t_salts").is_empty(), "revive item cannot deal damage");

    let mut gear = v.get("items", "t_blade").clone();
    gear.insert("revive".into(), json!(true));
    assert!(!check::validate_entry(&v, "items", &gear, "t_blade").is_empty(), "non-consumable rejects revive");

    let salts_row = json!({"id": "t_salts", "name": "Revive Salts", "family": "consumable", "consumable_type": "HEALTH_POTION", "heal": 15, "damage": 0, "use_blocks": 1, "use_reach": 1, "revive": true, "value": 15});
    let plain_row = json!({"id": "t_plain_pot", "name": "Plain Pot", "family": "consumable", "consumable_type": "HEALTH_POTION", "heal": 15, "damage": 0, "use_blocks": 1, "use_reach": 1, "revive": false, "value": 15});
    let _ = store::upsert(&data, None, "items", salts_row.as_object().unwrap(), None).unwrap();
    let _ = store::upsert(&data, None, "items", plain_row.as_object().unwrap(), None).unwrap();
    let raw = fs::read_to_string(data.join("catalog/items.json")).unwrap();
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    let entries = parsed["entries"].as_array().unwrap();
    assert_eq!(entries.iter().find(|e| e["id"] == "t_salts").unwrap().get("revive"), Some(&json!(true)));
    assert_eq!(entries.iter().find(|e| e["id"] == "t_plain_pot").unwrap().get("revive"), None, "default false omitted");
}
