//! U3.2-S4 (docs/sessions/U3_GEAR.md Level 2 and Level 3, "Fixed vs parameter" `level2_bonus`, `level3_passive`; FB-35):
//! the family table's Level-2 bonus and Level-3 passive ride on the resolved item row (`ITEM_FAMILY_LEVEL2`,
//! `ITEM_FAMILY_LEVEL3`) and stay out of the stored forms; an empty one stores as no key, so a family written before them
//! re-saves as before; the family check reads their shape (test-only values, scratch data). Items may carry `skill` and
//! `stat` effects now (taken off with the item); `proficiency` stays refused.

use aegis_core::json::Dict;
use aegis_core::scratch::Scratch;
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, ITEM_FAMILY_LEVEL2, ITEM_FAMILY_LEVEL3};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn family(id: &str, item_type: &str, extra: Value) -> Value {
    let mut f = json!({"id": id, "name": id, "item_type": item_type, "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"], "damage_type": "slash",
                       "base": {"main_1h": {"power": 10, "range": 1, "burden": 1, "mitigation": 0}}});
    if item_type == "ACCESSORY" {
        f = json!({"id": id, "name": id, "item_type": item_type, "slots": ["ACCESSORY_1", "ACCESSORY_2"], "base": {}});
    }
    for (k, x) in extra.as_object().unwrap() {
        f[k] = x.clone();
    }
    f
}

fn write(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn scratch(tag: &str, families: Value, items: Value) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_u32s4_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    write(&data, "item_families", 1, families);
    write(&data, "items", 2, items);
    write(&data, "skills", 2, json!([{"id": "t_flourish", "name": "Flourish", "kind": "ATTACK"}]));
    write(&data, "passives", 1, json!([
        {"id": "p_mastery", "name": "Mastery", "slot": "", "effects": [{"type": "stat", "stat": "DEX", "value": 1}]},
        {"id": "p_slotted", "name": "Slotted", "slot": "passive", "effects": [{"type": "stat", "stat": "DEX", "value": 1}]},
        {"id": "p_unlock", "name": "Unlock", "slot": "", "effects": [{"type": "unlock", "family": "t_flourish", "levels": 1}]},
        {"id": "p_drill", "name": "Drill", "slot": "", "effects": [{"type": "proficiency", "gear": "t_blade", "levels": 1}]}
    ]));
    (t, data)
}

fn bonus_errors(data: &Path, id: &str) -> Vec<String> {
    let v = store::view(data);
    check::validate_entry(&v, "item_families", v.get("item_families", id), id).into_iter().filter(|e| e.contains("Level 2") || e.contains("Level 3")).collect()
}

#[test]
fn the_family_bonuses_ride_on_the_item_and_stay_out_of_the_stored_forms() {
    let (_t, data) = scratch(
        "ride",
        json!([family("t_blade", "WEAPON", json!({"level2_bonus": {"crit": 3}, "level3_passive": "p_mastery"})), family("t_plain", "WEAPON", json!({}))]),
        json!([{"id": "i_blade", "name": "Blade", "family": "t_blade"}, {"id": "i_plain", "name": "Plain", "family": "t_plain"}]),
    );
    let src = source::load(&data);
    let cat = Catalog::from_value(&src.catalog);
    let (blade, plain) = (&cat.items["i_blade"], &cat.items["i_plain"]);
    assert_eq!((blade.level2_bonus.crit, blade.level2_bonus.burden, blade.level3_passive.as_str()), (3.0, 0, "p_mastery"));
    assert!(plain.level2_bonus.is_empty() && plain.level3_passive.is_empty());
    let plain_row = src.catalog["items"].as_array().unwrap().iter().find(|r| r["id"] == "i_plain").unwrap().clone();
    assert!(plain_row.get(ITEM_FAMILY_LEVEL2).is_none() && plain_row.get(ITEM_FAMILY_LEVEL3).is_none(), "a family without them adds no key: {plain_row}");
    let fams: Vec<Dict> = src.game["item_families"].as_array().unwrap().iter().filter_map(|f| f.as_object().cloned()).collect();
    let stored = store::catalog_storage("items", json!({"id": "i_blade", "name": "Blade", "family": "t_blade"}).as_object().unwrap(), &fams).unwrap();
    assert!(!stored.contains_key(ITEM_FAMILY_LEVEL2) && !stored.contains_key(ITEM_FAMILY_LEVEL3), "{stored:?}");
    let with = store::catalog_storage("item_families", family("t_blade", "WEAPON", json!({"level2_bonus": {"crit": 3}, "level3_passive": "p_mastery"})).as_object().unwrap(), &[]).unwrap();
    assert_eq!((with["level2_bonus"].clone(), with["level3_passive"].clone()), (json!({"crit": 3}), json!("p_mastery")));
    let without = store::catalog_storage("item_families", family("t_plain", "WEAPON", json!({})).as_object().unwrap(), &[]).unwrap();
    assert!(!without.contains_key("level2_bonus") && !without.contains_key("level3_passive"), "empty ones store as no key: {without:?}");
    assert!(bonus_errors(&data, "t_blade").is_empty() && bonus_errors(&data, "t_plain").is_empty(), "{:?}", bonus_errors(&data, "t_blade"));
}

#[test]
fn the_family_check_reads_the_bonus_shapes() {
    let cases: [(&str, Value, &str); 10] = [
        ("WEAPON", json!({"level2_bonus": {"burden": 2}}), ""),
        ("WEAPON", json!({"level2_bonus": {"crit": 3, "burden": 1}}), "Level 2 bonus is one of crit or burden, not both."),
        ("WEAPON", json!({"level2_bonus": {"crit": 11}}), "Level 2 crit must be 1-10."),
        ("WEAPON", json!({"level2_bonus": {"burden": 1.5}}), "Level 2 burden must be a whole number 1-3."),
        ("WEAPON", json!({"level2_bonus": {"power": 2}}), "Level 2 bonus: unknown key 'power' (crit or burden)."),
        ("ACCESSORY", json!({"level2_bonus": {"crit": 2}}), "Level 2 and 3 bonuses need a family a unit trains (WEAPON, SHIELD, ARMOR)."),
        ("WEAPON", json!({"level3_passive": "p_slotted"}), "Level 3 passive 'p_slotted' is slottable (passive); a mastery passive is always on (slot \"\")."),
        ("WEAPON", json!({"level3_passive": "p_unlock"}), "Level 3 passive 'p_unlock' has an effect worn gear can't apply (unlock, foresight, stagger_taken, meter or proficiency)."),
        ("WEAPON", json!({"level3_passive": "p_drill"}), "Level 3 passive 'p_drill' has an effect worn gear can't apply (unlock, foresight, stagger_taken, meter or proficiency)."),
        ("WEAPON", json!({"level3_passive": "p_mastery"}), ""),
    ];
    for (i, (t, extra, want)) in cases.into_iter().enumerate() {
        let (_t, data) = scratch(&format!("check{i}"), json!([family("t_fam", t, extra.clone())]), json!([]));
        let errors = bonus_errors(&data, "t_fam");
        if want.is_empty() {
            assert!(errors.is_empty(), "{extra}: {errors:?}");
        } else {
            assert!(errors.iter().any(|e| e == want), "{extra}: {errors:?}");
        }
    }
}

#[test]
fn items_may_carry_skill_and_stat_effects_but_not_proficiency() {
    let effect = |e: Value| json!({"kind": "effect", "effect": e});
    let items = json!([
        {"id": "i_skill", "name": "Skill", "family": "t_blade", "modifiers": [effect(json!({"type": "skill", "skill": "t_flourish"}))]},
        {"id": "i_stat", "name": "Stat", "family": "t_blade", "modifiers": [effect(json!({"type": "stat", "stat": "STR", "value": 2}))]},
        {"id": "i_prof", "name": "Prof", "family": "t_blade", "modifiers": [effect(json!({"type": "proficiency", "gear": "t_blade", "levels": 1}))]}
    ]);
    let (_t, data) = scratch("effects", json!([family("t_blade", "WEAPON", json!({}))]), items);
    let v = store::view(&data);
    let not_allowed = |id: &str| check::validate_entry(&v, "items", v.get("items", id), id).into_iter().filter(|e| e.contains("not allowed on items")).collect::<Vec<String>>();
    assert!(not_allowed("i_skill").is_empty() && not_allowed("i_stat").is_empty(), "{:?} {:?}", not_allowed("i_skill"), not_allowed("i_stat"));
    assert!(!not_allowed("i_prof").is_empty(), "proficiency stays refused");
}
