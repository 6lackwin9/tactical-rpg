//! CLASSPASS1 (`PR5-62a`) content checks, schema persistence, and loader derivations on scratch data (no shipped data).

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
    let t = Scratch::vacant(&format!("rpgkit_classpass_checks_{tag}"));
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
         "base": {"main_1h": {"power": 10, "range": 1, "burden": 0, "mitigation": 0}}},
        {"id": "bow", "name": "Bow", "item_type": "WEAPON", "hands": ["main_2h"], "default_hand": "main_2h", "scaling_stats": ["DEX"], "damage_type": "pierce",
         "engine_rules": ["always_ranged"], "base": {"main_2h": {"power": 10, "range": 4, "burden": 0, "mitigation": 0}}}
    ]));
    write_kind(&data, "skills", 2, json!([
        {"id": "slash", "name": "Slash", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "target_mode": "ENEMY", "weapon_types": ["sword"]},
        {"id": "shot", "name": "Shot", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "target_mode": "ENEMY", "weapon_types": ["bow"]},
        {"id": "mixed", "name": "Mixed", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "target_mode": "ENEMY", "weapon_types": ["bow", "sword"]},
        {"id": "chan", "name": "Chan", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "target_mode": "ENEMY", "channel": {"only": true}}
    ]));
    write_kind(&data, "passives", 1, json!([
        {"id": "p_always", "name": "Always", "slot": "", "effects": [{"type": "crit", "value": 8.0, "when": {"dual_wield": "1h_light"}}]},
        {"id": "p_slotted", "name": "Slotted", "slot": "passive", "effects": [{"type": "stat", "stat": "STR", "value": 1}]}
    ]));
    write_kind(&data, "classes", 1, json!([]));
    (t, data)
}

#[test]
fn class_passives_store_with_omit_default_and_require_empty_slot() {
    let (_t, data) = setup("class_passives");
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let growth = json!({"STR": 1, "DEX": 1, "INT": 1, "CONS": 1, "PERC": 1, "KNOW": 0});
    let mk = |id: &str, passives: Value| {
        let mut o = dict(json!({"id": id, "name": id, "art": "dummy", "base_stats": stats, "growth": growth,
            "proficiency": {"sword": {"level": 1, "scaling": 1.5}}, "skill_path": {"slash": 1}}));
        if !passives.is_null() {
            o.insert("passives".into(), passives);
        }
        o
    };

    store::upsert(&data, None, "classes", &mk("plain", Value::Null), None).unwrap();
    let text = fs::read_to_string(data.join("catalog/classes.json")).unwrap();
    assert!(!text.contains("\"passives\""), "empty class.passives is omitted: {text}");
    assert_eq!(store::resave_text(&data, None, "classes").unwrap(), text);

    store::upsert(&data, None, "classes", &mk("with_p", json!(["p_always"])), None).unwrap();
    let text = fs::read_to_string(data.join("catalog/classes.json")).unwrap();
    assert!(text.contains("\"passives\""), "non-empty class.passives is saved: {text}");
    assert_eq!(store::resave_text(&data, None, "classes").unwrap(), text);

    let v = store::view(&data);
    assert_eq!(v.typed.classes["with_p"].passives, ["p_always"]);
    assert!(v.typed.classes["plain"].passives.is_empty());

    let errs = |p: Value| check::validate_entry(&v, "classes", &mk("test_cls", p), "test_cls");
    assert!(errs(json!(["p_slotted"])).iter().any(|e| e.contains("slot 'passive'")));
    assert!(errs(json!(["p_always", "p_always"])).iter().any(|e| e.contains("twice")));
    assert!(errs(json!(["no_such"])).iter().any(|e| e.contains("no_such")));
}

#[test]
fn always_ranged_derives_on_load_and_is_never_saved() {
    let (_t, data) = setup("always_ranged");
    let v = store::view(&data);
    let skill = |id: &str| v.typed.skills.iter().find(|s| s.id == id).unwrap();
    assert!(skill("shot").always_ranged);
    assert!(!skill("slash").always_ranged);
    assert!(!skill("mixed").always_ranged);

    let mut row = dict(json!({"id": "new_bow", "name": "New Bow", "kind": "ATTACK", "level": 1, "blocks": 1, "min_proficiency": 1, "power_mult": 1.0, "target_mode": "ENEMY", "weapon_types": ["bow"]}));
    row.insert("always_ranged".into(), json!(true));
    store::upsert(&data, None, "skills", &row, None).unwrap();
    let text = fs::read_to_string(data.join("catalog/skills.json")).unwrap();
    assert!(!text.contains("\"always_ranged\""), "derived always_ranged is never written to skills.json: {text}");
}

#[test]
fn crit_channel_ranged_and_when_dual_wield_check_cleanly() {
    let (_t, data) = setup("effects");
    let v = store::view(&data);
    let hooks = check::hooks::Ctx::of(&v);
    let has_skill = |s: &str| v.typed.family_of(s) != "" && v.typed.skills.iter().any(|r| r.id == s);
    let fx_errs = |items_only: bool, rows: Value| check::simple::effect_errors(&rows, &has_skill, &[], false, items_only, &hooks);

    assert!(fx_errs(false, json!([{"type": "crit", "value": 8.0, "when": {"dual_wield": "1h_light"}}])).is_empty());
    assert!(fx_errs(false, json!([{"type": "skill_potency", "group": "channel", "value": 0.25}])).is_empty());
    assert!(fx_errs(false, json!([{"type": "skill_reach", "reach": 1, "ranged": true}])).is_empty());

    assert!(fx_errs(false, json!([{"type": "crit", "value": 0.0}])).iter().any(|e| e.contains("crit bonus")));
    assert!(fx_errs(false, json!([{"type": "crit", "value": 150.0}])).iter().any(|e| e.contains("crit bonus")));
    assert!(fx_errs(true, json!([{"type": "crit", "value": 5.0}])).iter().any(|e| e.contains("not allowed on items")));
    assert!(fx_errs(false, json!([{"type": "skill_reach", "reach": 1, "ranged": "yes"}])).iter().any(|e| e.contains("true or false")));
    assert!(fx_errs(false, json!([{"type": "crit", "value": 5.0, "when": {"dual_wield": "bad_rule"}}])).iter().any(|e| e.contains("bad_rule")));
    assert!(fx_errs(false, json!([{"type": "crit", "value": 5.0, "when": {"pct": 50}}])).iter().any(|e| e.contains("hp_below")));
}
