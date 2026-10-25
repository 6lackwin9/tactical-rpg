//! Weapon type options added after the Godot freeze (RULESET1-S `polearm`, `scourge`; CAST1-S `censer`, `bone_saw`,
//! `sprayer`): each is an items `weapon_type` option, a gear token and an optional one (no family needed until data
//! uses it); a family named after it passes the gear token rule; every other gear token still needs its family.
//! Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, gstr};
use rpgkit_content::check::{self, item, skill};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

/// The CAST1-S weapon types (CAST1_GEAR section 1).
const NEW_TYPES: [&str; 3] = ["censer", "bone_saw", "sprayer"];

fn weapon_type_options() -> Vec<String> {
    let f = fields(source::schema(), "content", "items").iter().map(json::dict).find(|f| json::s(f, "key") == "weapon_type").expect("content.items.weapon_type").clone();
    json::aget(&f, "options").iter().map(gstr).collect()
}

#[test]
fn each_new_type_is_an_option_a_gear_token_and_optional() {
    let options = weapon_type_options();
    for t in NEW_TYPES {
        assert!(options.iter().any(|o| o == t), "{t}: an items weapon_type option");
        assert!(skill::GEAR_TYPES.contains(&t), "{t}: a gear token");
        assert!(skill::OPTIONAL_GEAR_TYPES.contains(&t), "{t}: needs no family until data uses it");
    }
    // the schema options are "" then the gear tokens in the code's order
    let mut want = vec![String::new()];
    want.extend(skill::GEAR_TYPES.iter().map(|g| g.to_string()));
    assert_eq!(options, want, "the schema and the code list the same gear tokens in the same order");
}

#[test]
fn only_the_required_gear_tokens_need_a_family() {
    let family = |id: &str| json!({"id": id}).as_object().cloned().unwrap();
    let required: Vec<&str> = skill::GEAR_TYPES.iter().copied().filter(|g| !skill::OPTIONAL_GEAR_TYPES.contains(g)).collect();
    let all: Vec<_> = required.iter().map(|g| family(g)).collect();
    assert!(item::family_set_errors(&all).is_empty(), "the new types are not asked for");
    let missing = required[0];
    let fewer: Vec<_> = required[1..].iter().map(|g| family(g)).collect();
    assert_eq!(item::family_set_errors(&fewer), [format!("Gear token '{missing}' has no item family (every Proficiency gear type needs one).")]);
}

/// The three families as CAST1_GEAR section 1 gives them (hand, scaling stats, damage type, base power, range, block
/// cost, the sprayer's always_ranged rule).
fn families() -> Value {
    let fam = |id: &str, hand: &str, stats: Value, dmg: &str, power: i64, range: i64, block: i64, rules: Value| {
        json!({"id": id, "name": id, "item_type": "WEAPON", "hands": [hand], "default_hand": hand, "scaling_stats": stats, "damage_type": dmg,
               "base": {hand: {"power": power, "range": range, "block_cost": block, "burden": 1, "mitigation": 0}}, "engine_rules": rules})
    };
    json!([
        fam("censer", "main_1h", json!(["INT", "STR"]), "blunt", 11, 1, 1, json!([])),
        fam("bone_saw", "main_1h", json!(["DEX", "STR"]), "slash", 14, 1, 0, json!([])),
        fam("sprayer", "main_2h", json!(["INT"]), "fire", 18, 3, 1, json!(["always_ranged"])),
    ])
}

#[test]
fn a_family_named_after_a_new_type_passes_the_gear_token_rule() {
    let t = std::env::temp_dir().join(format!("rpgkit_weapon_types_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data: PathBuf = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    fs::write(data.join("catalog/item_families.json"), json!({"kind": "item_families", "version": 1, "entries": families()}).to_string()).unwrap();
    let v = store::view(&data);
    for id in NEW_TYPES {
        let e = v.get("item_families", id).clone();
        assert!(!e.is_empty(), "fixture {id}");
        let errors = check::validate_entry(&v, "item_families", &e, id);
        assert!(!errors.iter().any(|m| m.contains("gear token")), "{id}: {errors:?}");
    }
    // a family named after no gear token is still refused
    let mut bogus = v.get("item_families", "censer").clone();
    bogus.insert("id".into(), json!("thurible"));
    let errors = check::validate_entry(&v, "item_families", &bogus, "thurible");
    assert!(errors.iter().any(|m| m.starts_with("A WEAPON family is named after its gear token")), "{errors:?}");
    let _ = fs::remove_dir_all(&t);
}
