//! Weapon type options added after the Godot freeze (RULESET1-S `polearm`, `scourge`; CAST1-S `censer`, `bone_saw`,
//! `sprayer`) and LAYER-DOWN-2 (the family table names the gear tokens): each listed type is an items `weapon_type`
//! option and a pre-family gear token; the field reads its options from the family table; a family named after any id
//! (a listed type or a new one) passes the gear token rule, an id that cannot be a token does not. Scratch data folders
//! only.

use aegis_content::schema::fields;
use aegis_core::json::{self, gstr};
use rpgkit_content::check::{self, skill};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

/// The CAST1-S weapon types (CAST1_GEAR section 1).
const NEW_TYPES: [&str; 3] = ["censer", "bone_saw", "sprayer"];

fn weapon_type_field() -> json::Dict {
    fields(source::schema(), "content", "items").iter().map(json::dict).find(|f| json::s(f, "key") == "weapon_type").expect("content.items.weapon_type").clone()
}

#[test]
fn each_new_type_is_an_option_and_a_pre_family_gear_token() {
    let f = weapon_type_field();
    let options: Vec<String> = json::aget(&f, "options").iter().map(gstr).collect();
    for t in NEW_TYPES {
        assert!(options.iter().any(|o| o == t), "{t}: an items weapon_type option");
        assert!(skill::schema_gear_tokens().iter().any(|g| g == t), "{t}: a pre-family gear token");
    }
    // the schema options are "" then the pre-family gear tokens; with a family table the field reads the table
    let mut want = vec![String::new()];
    want.extend(skill::schema_gear_tokens());
    assert_eq!(options, want, "the field's options are the pre-family gear tokens in their order");
    assert_eq!(json::s(&f, "options_from"), "gear_types");
    assert_eq!(skill::gear_tokens(&[]), skill::schema_gear_tokens(), "no family table: the schema's tokens");
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
fn a_family_names_its_own_gear_token() {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_weapon_types");
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
    // LAYER-DOWN-2: a family named after no listed type adds its own token
    let mut thurible = v.get("item_families", "censer").clone();
    thurible.insert("id".into(), json!("thurible"));
    let errors = check::validate_entry(&v, "item_families", &thurible, "");
    assert!(!errors.iter().any(|m| m.contains("gear token")), "{errors:?}");
    // an id that cannot be a gear token is refused, the table's tokens listed (the schema's order first)
    for bad in ["", "Bad Id!", "unarmed"] {
        let mut row = thurible.clone();
        row.insert("id".into(), json!(bad));
        let errors = check::validate_entry(&v, "item_families", &row, "");
        let want = format!("A WEAPON family is named after its gear token (one of {}).", skill::listed_gear_tokens(v.entries("item_families")).join(", "));
        assert!(errors.contains(&want), "{bad}: {errors:?}");
    }
    assert_eq!(skill::listed_gear_tokens(v.entries("item_families")), ["censer", "bone_saw", "sprayer"], "the schema's order, then the table's others");
    let _ = fs::remove_dir_all(&t);
}
