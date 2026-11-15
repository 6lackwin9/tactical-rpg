//! GEAR-REWORK1 (PR5-51a pass 2, `docs/sessions/U3_GEAR_REWORK.md` 2-4) content checks on a scratch data folder built
//! here (no shipped data). An item family may name its hand category (`slot_rule`: 2h_solo, 2h_rig, 1h_heavy,
//! 1h_light, off_only; it fixes the family's hands), a `2h_rig` family the off-hand families it `pairs_with`, and an
//! off-hand family its `off_hand_skill`, ammunition rule (`empower_next` `{meter, cost, power_mult, throw_skill?}`, naming a unit
//! meter row and, pass 3, an ATTACK skill with its own reach as its Throw, with the family's Reload) and `power_stat_override`. An `off_only` family is an off-hand family whatever
//! its id ("dynamic OFF_CATEGORIES"); without the field the closed list holds. The loader carries the fields onto the
//! family's items, the writer stores no key while they are empty, and a unit loadout accepts a `1h_light` weapon in
//! the off hand and a two-handed weapon's paired off-hand item. A catalog without the fields checks as before.

use aegis_core::json::{self, Dict};
use aegis_core::scratch::Scratch;
use rpgkit_content::{check, source, store};
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const STATS: &str = r#"{"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10}"#;
/// The fragments of the hand rule checks' messages (a test reads only these).
const HAND_MSGS: [&str; 9] = ["hand category", "Hand category", "pairs_with", "off-hand family (hands = off) has", "empower_next", "names its Reload", "Off-hand families are", "not one of", "unit meter"];

fn write(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

fn base(hand: &str) -> Value {
    json!({hand: {"power": 10, "range": 1, "burden": 0, "mitigation": 0}})
}

fn family(id: &str, hand: &str, extra: Value) -> Value {
    let mut row = json!({"id": id, "name": id, "item_type": "WEAPON", "hands": [hand], "default_hand": hand, "scaling_stats": ["STR"], "damage_type": "slash", "base": base(hand)});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

fn bow() -> Value {
    family("t_bow", "main_2h", json!({"slot_rule": "2h_rig", "pairs_with": ["t_ammo"], "engine_rules": ["always_ranged"]}))
}

fn ammo(empower: Value) -> Value {
    family("t_ammo", "off", json!({"slot_rule": "off_only", "off_hand_skill": "t_reload", "empower_next": empower}))
}

fn quiver() -> Value {
    json!({"meter": "t_shots", "cost": 1, "power_mult": 1.5})
}

/// A scratch data folder: a 2h_rig bow pairing with ammo, an ammunition family (Reload `t_reload`, meter `t_shots`),
/// a 1h_light knife (band 1-2), a cloak (Fade `t_fade`), a sword without a hand category, a focus lending INT, their
/// items, the skills and the meter row.
fn scratch(tag: &str) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_hand_rules_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let skill = |id: &str| json!({"id": id, "name": id, "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "SELF", "heal_base": 1});
    let throw = json!({"id": "t_throw", "name": "Throw", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY", "reach_min": 2, "reach_max": 3});
    let lob = json!({"id": "t_lob", "name": "Lob", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"});
    write(&data, "skills", 2, json!([skill("t_reload"), skill("t_fade"), throw, lob]));
    write(&data, "unit_meters", 1, json!([{"id": "t_shots", "name": "Shots", "short": "Sh", "min": 0, "max": 3, "start": 0, "start_full": true, "show": "pips"}]));
    write(&data, "item_families", 1, json!([
        bow(),
        ammo(quiver()),
        family("t_knife", "main_1h", json!({"slot_rule": "1h_light", "range_band": {"min": 1, "max": 2}})),
        family("t_cloak", "off", json!({"slot_rule": "off_only", "off_hand_skill": "t_fade"})),
        family("t_sword", "main_1h", json!({})),
        family("t_focus", "off", json!({"slot_rule": "off_only", "power_stat_override": "INT"}))
    ]));
    write(&data, "items", 2, json!([
        {"id": "i_bow", "name": "Bow", "family": "t_bow"},
        {"id": "i_quiver", "name": "Quiver", "family": "t_ammo"},
        {"id": "i_knife", "name": "Knife", "family": "t_knife"},
        {"id": "i_mantle", "name": "Mantle", "family": "t_cloak"},
        {"id": "i_sword", "name": "Sword", "family": "t_sword"},
        {"id": "i_focus", "name": "Focus", "family": "t_focus"}
    ]));
    write(&data, "classes", 1, json!([]));
    (t, data)
}

fn hand_errors(found: Vec<String>) -> Vec<String> {
    found.into_iter().filter(|e| HAND_MSGS.iter().any(|m| e.contains(m))).collect()
}

#[test]
fn well_formed_hand_rules_pass_and_an_off_only_family_of_any_id_is_an_off_hand_family() {
    let (_t, data) = scratch("ok");
    let v = store::view(&data);
    for f in v.entries("item_families") {
        let id = json::s(f, "id");
        assert!(hand_errors(check::validate_entry(&v, "item_families", f, &id)).is_empty(), "{id}: {:?}", check::validate_entry(&v, "item_families", f, &id));
    }
    // without the hand category the closed list holds (today's rule)
    let mut plain = family("t_quiver2", "off", json!({}));
    plain.as_object_mut().unwrap().remove("slot_rule");
    let found = hand_errors(check::validate_entry(&v, "item_families", &dict(plain), ""));
    assert!(found.iter().any(|e| e.starts_with("Off-hand families are shield, orb, tome")), "{found:?}");
}

#[test]
fn a_hand_category_fixes_the_hands_and_the_off_hand_fields_belong_to_off_hand_families() {
    let (_t, data) = scratch("errors");
    let v = store::view(&data);
    let errors = |row: Value| hand_errors(check::validate_entry(&v, "item_families", &dict(row), ""));
    assert_eq!(errors(family("t_x", "main_1h", json!({"slot_rule": "2h_solo"}))), ["Hand category 2h_solo takes exactly the hand main_2h (hands = [main_2h])."]);
    assert_eq!(errors(family("t_x", "main_2h", json!({"slot_rule": "1h_light"}))), ["Hand category 1h_light takes exactly the hand main_1h (hands = [main_1h])."]);
    assert_eq!(errors(family("t_x", "main_1h", json!({"slot_rule": "off_only"}))), ["Hand category off_only takes exactly the hand off (hands = [off])."]);
    assert_eq!(errors(family("t_x", "main_2h", json!({"slot_rule": "2h_solo", "pairs_with": ["t_ammo"]}))), ["Only a 2h_rig family lists pairs_with (the off-hand families its two-handed weapons leave room for)."]);
    assert_eq!(errors(family("t_x", "main_2h", json!({"slot_rule": "2h_rig", "pairs_with": ["t_sword"]}))), ["pairs_with 't_sword' is not an off-hand family of the table (hands = off)."]);
    assert_eq!(errors(family("t_x", "main_1h", json!({"off_hand_skill": "t_fade"}))), ["Only an off-hand family (hands = off) has off_hand_skill."]);
    assert_eq!(errors(family("t_x", "main_1h", json!({"power_stat_override": "INT"}))), ["Only an off-hand family (hands = off) has power_stat_override."]);
    assert_eq!(errors(family("t_x", "main_1h", json!({"slot_rule": "huge"}))), ["Hand category: 'huge' is not one of , 2h_solo, 2h_rig, 1h_heavy, 1h_light, off_only."], "the schema's enum");
}

#[test]
fn an_ammunition_rule_names_a_meter_row_a_cost_a_factor_and_its_reload() {
    let (_t, data) = scratch("ammo");
    let v = store::view(&data);
    let errors = |row: Value| hand_errors(check::validate_entry(&v, "item_families", &dict(row), "t_ammo"));
    assert!(errors(ammo(quiver())).is_empty());
    assert!(errors(ammo(json!({"meter": "t_shots"}))).is_empty(), "cost and factor default");
    assert_eq!(errors(ammo(json!({"meter": "t_none"}))), ["empower_next meter 't_none' is not a unit meter row."]);
    assert_eq!(errors(ammo(json!({"meter": "t_shots", "cost": 0}))), ["empower_next cost 0 is below 1."]);
    assert_eq!(errors(ammo(json!({"meter": "t_shots", "power_mult": 0}))), ["empower_next power_mult 0.00 is not above 0."]);
    assert_eq!(errors(ammo(json!({"meter": "t_shots", "pips": 3}))), ["empower_next has no key 'pips' (meter, cost, power_mult, throw_skill)."]);
    assert_eq!(errors(ammo(json!({"cost": 1}))), ["empower_next names its unit meter (meter)."]);
    // PR5-51a pass 3: a Throw is an ATTACK skill row with its own reach
    assert!(errors(ammo(json!({"meter": "t_shots", "throw_skill": "t_throw"}))).is_empty(), "a 2-3 attack");
    assert_eq!(errors(ammo(json!({"meter": "t_shots", "throw_skill": "t_none"}))), ["empower_next throw_skill 't_none' is not a skill row."]);
    for bad in ["t_lob", "t_fade"] {
        assert_eq!(errors(ammo(json!({"meter": "t_shots", "throw_skill": bad}))), [format!("empower_next throw_skill '{bad}' needs an ATTACK skill with its own reach (reach_max above 0).")], "{bad}");
    }
    let mut silent = ammo(quiver());
    silent.as_object_mut().unwrap().remove("off_hand_skill");
    assert_eq!(errors(silent), ["An empower_next family names its Reload (off_hand_skill: the skill that refills the meter at 0)."]);
}

#[test]
fn the_loader_carries_the_hand_fields_to_the_items_and_the_writer_omits_empty_ones() {
    let (_t, data) = scratch("io");
    let cat = Catalog::from_value(&source::load(&data).catalog);
    let (bow, quiver, knife, sword) = (&cat.items["i_bow"], &cat.items["i_quiver"], &cat.items["i_knife"], &cat.items["i_sword"]);
    assert_eq!((bow.slot_rule.as_str(), bow.pairs_with.clone(), bow.two_handed), ("2h_rig", vec!["t_ammo".to_string()], true));
    assert_eq!((quiver.slot_rule.as_str(), quiver.off_hand_skill.as_str(), quiver.slot.as_str()), ("off_only", "t_reload", "OFF_HAND"));
    assert_eq!(quiver.empower_next, Some(rpgkit_data::EmpowerNext { meter: "t_shots".into(), cost: 1, power_mult: 1.5, throw_skill: String::new() }));
    assert_eq!((knife.slot_rule.as_str(), knife.family_reach_max), ("1h_light", 2));
    assert_eq!(cat.items["i_focus"].power_stat_override, "INT");
    assert!(sword.slot_rule.is_empty() && sword.pairs_with.is_empty() && sword.off_hand_skill.is_empty() && sword.empower_next.is_none() && sword.power_stat_override.is_empty(), "no keys, no fields");
    assert!(rpgkit_rules::gear::off_hand_ok(Some(bow), quiver) && !rpgkit_rules::gear::off_hand_ok(Some(bow), &cat.items["i_mantle"]));
    store::upsert(&data, None, "item_families", &dict(family("t_axe", "main_1h", json!({}))), None).expect("a family without the fields");
    let saved: Value = serde_json::from_str(&fs::read_to_string(data.join("catalog/item_families.json")).unwrap()).unwrap();
    let axe = saved["entries"].as_array().unwrap().iter().find(|e| e["id"] == "t_axe").expect("saved");
    for key in ["slot_rule", "pairs_with", "off_hand_skill", "empower_next", "power_stat_override"] {
        assert!(axe.get(key).is_none(), "{key} is left out while empty");
    }
    let bow_row = saved["entries"].as_array().unwrap().iter().find(|e| e["id"] == "t_bow").expect("kept");
    assert_eq!((bow_row["slot_rule"].clone(), bow_row["pairs_with"].clone()), (json!("2h_rig"), json!(["t_ammo"])), "set fields are stored");
    let items = fs::read_to_string(data.join("catalog/items.json")).unwrap();
    for key in [rpgkit_data::ITEM_FAMILY_SLOT_RULE, rpgkit_data::ITEM_FAMILY_PAIRS_WITH, rpgkit_data::ITEM_FAMILY_OFF_HAND_SKILL, rpgkit_data::ITEM_FAMILY_EMPOWER_NEXT, rpgkit_data::ITEM_FAMILY_POWER_STAT] {
        assert!(!items.contains(key), "the resolved key {key} is not stored");
    }
}

#[test]
fn a_loadout_takes_a_light_off_hand_weapon_and_a_paired_off_hand_item() {
    let (_t, data) = scratch("loadout");
    let v = store::view(&data);
    let stats: Value = serde_json::from_str(STATS).unwrap();
    let class = |main: &str, off: &str| json!({"id": "adept", "name": "Adept", "base_stats": stats, "growth": stats, "skill_path": {}, "proficiency": {"t_sword": {"level": 1}}, "main_hand": main, "off_hand": off});
    let gear_errors = |main: &str, off: &str| check::validate_entry(&v, "classes", &dict(class(main, off)), "adept").into_iter().filter(|e| e.contains("two-handed") || e.contains("can't be equipped")).collect::<Vec<_>>();
    assert!(gear_errors("i_sword", "i_knife").is_empty(), "a 1h_light weapon in the off hand");
    assert!(gear_errors("i_bow", "i_quiver").is_empty(), "the bow pairs with ammo");
    assert_eq!(gear_errors("i_bow", "i_mantle"), ["A two-handed weapon can't be paired with an off-hand item."], "a cloak is not in the bow's pairs_with");
    assert_eq!(gear_errors("i_quiver", ""), ["Quiver can't be equipped in Main Hand."], "off_only stays in the off hand");
}
