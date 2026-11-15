//! GEAR-REWORK1 (PR5-51a, `docs/sessions/U3_GEAR_REWORK.md` 1, 5) content checks on a scratch data folder built here
//! (no shipped data). A class proficiency entry's `role` is one of main, secondary, third, minor or the alias `primary`
//! (read as main; an unknown name is an error, a level other than the role's seeded one a warning). An ARMOR family may
//! name its `armor_weight` (heavy, medium, light: covering HEAD and BODY, one family per weight; light weighs nothing);
//! the loader carries it onto the family's items and the writer stores no key while it is empty. The strict gear rule's
//! catalog checks are gated on the classes (`check::unit::strict_gear`: some class tags a role): only then must every
//! HEAD or BODY armour family name its weight, and an accessory carry at most 2 bonus modifiers or 1 passive hook (a
//! trigger effect), requirements aside. A catalog without roles (today's) checks as before.

use aegis_core::json::{self, Dict};
use aegis_core::scratch::Scratch;
use rpgkit_content::{check, source, store};
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const STATS: &str = r#"{"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10}"#;

fn write(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

fn class(proficiency: Value) -> Value {
    let stats: Value = serde_json::from_str(STATS).unwrap();
    json!({"id": "adept", "name": "Adept", "base_stats": stats, "growth": stats, "skill_path": {}, "proficiency": proficiency})
}

fn armour(id: &str, weight: &str, slots: Value, burden: i64) -> Value {
    let mut row = json!({"id": id, "name": id, "item_type": "ARMOR", "slots": slots, "base": {"none": {"power": 0, "range": 1, "burden": burden, "mitigation": 4}},
                         "adjustable": ["mitigation", "burden"], "modifier_kinds": ["adjust", "stat_bonus", "requirement", "effect"], "max_modifiers": 6, "max_tiers": 2});
    if !weight.is_empty() {
        row["armor_weight"] = json!(weight);
    }
    row
}

/// A scratch data folder: a sword family, a plate family (`plate_weight`, "" = none), an accessory family, a body armour
/// and an accessory item, and one class (with roles when `roles`).
fn scratch(tag: &str, roles: bool, plate_weight: &str) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_gear_rework_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    write(&data, "item_families", 1, json!([
        {"id": "t_sword", "name": "Sword", "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"], "damage_type": "slash",
         "base": {"main_1h": {"power": 10, "range": 1, "burden": 0, "mitigation": 0}}},
        armour("t_plate", plate_weight, json!(["HEAD", "BODY"]), 2),
        {"id": "t_acc", "name": "Accessory", "item_type": "ACCESSORY", "slots": ["ACCESSORY_1", "ACCESSORY_2"], "base": {"none": {"power": 0, "range": 1, "burden": 0, "mitigation": 0}},
         "adjustable": ["burden"], "modifier_kinds": ["stat_bonus", "effect", "requirement"], "max_modifiers": 4, "max_tiers": 0}
    ]));
    write(&data, "items", 2, json!([
        {"id": "i_plate", "name": "Plate", "family": "t_plate", "slot": "BODY"},
        {"id": "i_ring", "name": "Ring", "family": "t_acc", "slot": "ACCESSORY_1"}
    ]));
    let role = |level: i64, role: &str| if roles { json!({"level": level, "role": role}) } else { json!({"level": level}) };
    write(&data, "classes", 1, json!([class(json!({"t_sword": role(3, "main"), "t_plate": role(3, "main")}))]));
    (t, data)
}

#[test]
fn a_role_is_a_known_name_or_the_primary_alias_and_its_level_is_the_seeded_one() {
    let (_t, data) = scratch("roles", false, "");
    let v = store::view(&data);
    let errors = |role: &str| check::validate_entry(&v, "classes", &dict(class(json!({"t_sword": {"level": 3, "role": role}}))), "adept");
    for role in ["main", "secondary", "third", "minor", "primary"] {
        assert!(!errors(role).iter().any(|e| e.contains("role")), "{role}: {:?}", errors(role));
    }
    assert!(errors("captain").contains(&"Proficiency t_sword: role 'captain' is not one of main, secondary, third, minor (or primary = main).".to_string()), "{:?}", errors("captain"));
    let primary = rpgkit_data::ClassDef::from_obj(&dict(class(json!({"t_sword": {"level": 3, "role": "primary"}}))));
    assert_eq!(primary.proficiency_roles.get("t_sword").map(String::as_str), Some("main"), "primary reads as main");
    let warns = |level: i64, role: &str| check::warnings_for(&v, "classes", &dict(class(json!({"t_sword": {"level": level, "role": role}}))));
    let off = |w: &Vec<String>| w.iter().filter(|w| w.contains("is seeded at level")).cloned().collect::<Vec<_>>();
    for (role, level) in rpgkit_data::PROF_ROLE_LEVELS {
        assert!(off(&warns(i64::from(level), role)).is_empty(), "{role} at {level}");
    }
    assert_eq!(off(&warns(2, "primary")), ["Proficiency t_sword: role main is seeded at level 3; this entry is level 2."]);
}

#[test]
fn an_armour_weight_is_known_on_an_armour_family_over_head_and_body_once() {
    let (_t, data) = scratch("weights", false, "heavy");
    let v = store::view(&data);
    let errors = |row: Value| check::validate_entry(&v, "item_families", &dict(row), "");
    let weight_errors = |row: Value| errors(row).into_iter().filter(|e| e.contains("rmour weight") || e.contains("armour family")).collect::<Vec<_>>();
    assert!(weight_errors(armour("t_robe", "light", json!(["HEAD", "BODY"]), 0)).is_empty());
    assert!(weight_errors(armour("t_robe", "", json!(["HEAD", "BODY"]), 0)).is_empty(), "no weight: fine without the strict rule");
    assert_eq!(weight_errors(armour("t_robe", "huge", json!(["HEAD", "BODY"]), 0)), ["Armour weight: 'huge' is not one of , heavy, medium, light."], "the schema's enum");
    assert_eq!(weight_errors(armour("t_robe", "light", json!(["BODY"]), 0)), ["A light armour family covers both HEAD and BODY (one proficiency key per weight)."]);
    assert_eq!(weight_errors(armour("t_robe", "heavy", json!(["HEAD", "BODY"]), 0)), ["Armour weight 'heavy' is already family 't_plate's (one family per weight)."]);
    let mut sword = armour("t_blade", "medium", json!([]), 0);
    sword["item_type"] = json!("WEAPON");
    assert!(errors(sword).contains(&"Only an ARMOR family has an armour weight (this is a WEAPON family).".to_string()));
    let light = |burden: i64| check::warnings_for(&v, "item_families", &dict(armour("t_robe", "light", json!(["HEAD", "BODY"]), burden)));
    assert!(light(1).contains(&"Light armour carries 0 burden; this family's base burden is 1.".to_string()), "{:?}", light(1));
    assert!(!light(0).iter().any(|w| w.contains("Light armour")));
}

#[test]
fn the_loader_carries_the_weight_to_the_items_and_the_writer_omits_an_empty_one() {
    let (_t, data) = scratch("weight_io", false, "heavy");
    let cat = Catalog::from_value(&source::load(&data).catalog);
    assert_eq!(cat.items["i_plate"].armor_weight, "heavy");
    assert_eq!(cat.items["i_ring"].armor_weight, "");
    store::upsert(&data, None, "item_families", &dict(armour("t_robe", "", json!(["HEAD", "BODY"]), 0)), None).expect("a family without a weight");
    store::upsert(&data, None, "item_families", &dict(armour("t_mail", "medium", json!(["HEAD", "BODY"]), 1)), None).expect("a medium family");
    let saved = fs::read_to_string(data.join("catalog/item_families.json")).unwrap();
    assert_eq!(saved.matches("\"armor_weight\"").count(), 2, "heavy and medium store the key, the empty one none");
    assert!(!fs::read_to_string(data.join("catalog/items.json")).unwrap().contains(rpgkit_data::ITEM_FAMILY_ARMOR_WEIGHT), "the resolved key is not stored");
}

#[test]
fn the_strict_rule_needs_a_weight_on_every_head_and_body_armour_family() {
    for (roles, want) in [(false, 0), (true, 1)] {
        let (_t, data) = scratch(&format!("gate_{roles}"), roles, "");
        let v = store::view(&data);
        assert_eq!(check::unit::strict_gear(&v), roles);
        let found = check::validate_entry(&v, "item_families", &dict(armour("t_plate", "", json!(["HEAD", "BODY"]), 2)), "t_plate");
        let n = found.iter().filter(|e| e.contains("names its armour weight")).count();
        assert_eq!(n, want, "roles {roles}: {found:?}");
    }
}

#[test]
fn under_the_strict_rule_an_accessory_carries_two_bonuses_or_one_hook() {
    let stat = |s: &str| json!({"kind": "stat_bonus", "stat": s, "value": 1});
    let need = json!({"kind": "requirement", "stat": "KNOW", "value": 12});
    let hook = json!({"kind": "effect", "effect": {"type": "trigger", "on": "hit_taken", "limit": "battle", "effects": [{"type": "stat", "stat": "DEX", "value": 1}]}});
    let ring = |mods: Vec<Value>| json!({"id": "i_ring", "name": "Ring", "family": "t_acc", "slot": "ACCESSORY_1", "modifiers": mods});
    let cases: [(Vec<Value>, Option<&str>); 6] = [
        (vec![stat("DEX"), stat("STR")], None),
        (vec![stat("DEX"), stat("STR"), need.clone()], None),
        (vec![hook.clone(), need.clone()], None),
        (vec![stat("DEX"), stat("STR"), stat("INT")], Some("An accessory carries at most 2 bonus modifiers (requirements aside) or 1 passive hook; this one has 3.")),
        (vec![hook.clone(), stat("DEX")], Some("An accessory with a passive hook carries no other bonus (requirements aside); this one has 1 more.")),
        (vec![hook.clone(), hook.clone()], Some("An accessory holds at most 1 passive hook (a trigger effect); this one has 2.")),
    ];
    for roles in [false, true] {
        let (_t, data) = scratch(&format!("acc_{roles}"), roles, "heavy");
        let v = store::view(&data);
        for (mods, want) in &cases {
            let found: Vec<String> = check::validate_entry(&v, "items", &dict(ring(mods.clone())), "i_ring").into_iter().filter(|e| e.starts_with("An accessory")).collect();
            let want: Vec<String> = want.filter(|_| roles).map(str::to_string).into_iter().collect();
            assert_eq!(found, want, "roles {roles}: {mods:?}");
        }
    }
}
