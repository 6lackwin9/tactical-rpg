//! PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5, FB-167 Q7 (a)) on scratch data built here (no shipped data): a weapon
//! family's `techniques` (0-3 `{skill, grip}`, each skill once) and `keystone` ride on its items' resolved rows and stay
//! out of the stored forms (a family without them stores as before); the family check refuses them off a main-hand
//! WEAPON family, past 3, with an unknown skill, a bad grip or a skill twice; a weapon's own `technique` is checked the
//! same way; a `MASTERY_BOOK` names a main-hand weapon family; a class with more than 3 masterable families (main-hand
//! weapon families at proficiency 2+) warns; the manifest's `durability` {from_act} is read beside the data, checked,
//! and taken by the manifest writer.

use aegis_core::json::{self, Dict};
use aegis_core::scratch::Scratch;
use rpgkit_content::source::{self, DURABILITY, FROM_ACT, FROM_ACT_MAX};
use rpgkit_content::{check, store};
use rpgkit_data::mastery::{ITEM_FAMILY_KEYSTONE, ITEM_FAMILY_TECHNIQUES, MASTERABLE_MAX, MASTERY_MAX};
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn weapon_family(id: &str, hands: Value, extra: Value) -> Value {
    let hand = hands[0].clone();
    let mut f = json!({"id": id, "name": id, "item_type": "WEAPON", "hands": hands, "default_hand": hand, "scaling_stats": ["STR"], "damage_type": "slash",
        "base": {hand.as_str().unwrap(): {"power": 10, "range": 1, "burden": 0, "mitigation": 0}}});
    for (k, x) in extra.as_object().unwrap() {
        f[k] = x.clone();
    }
    f
}

fn write(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn attack(id: &str) -> Value {
    json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0})
}

/// A scratch game folder (`game.json` when `manifest` is given) with the skills `t_a`..`t_d` and `t_key`, the families,
/// items and classes given.
fn scratch(tag: &str, manifest: Option<Value>, families: Value, items: Value, classes: Value) -> (Scratch, PathBuf) {
    let t = Scratch::vacant(&format!("rpgkit_pk2s4_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    if let Some(m) = manifest {
        let mut g = json!({"id": "demo", "title": "Demo", "version": "0.1.0", "worlds": [], "start_worlds": [], "references": []});
        for (k, v) in m.as_object().unwrap() {
            g[k] = v.clone();
        }
        fs::write(t.join("game.json"), serde_json::to_string_pretty(&g).unwrap()).unwrap();
    }
    write(&data, "skills", 2, json!([attack("t_a"), attack("t_b"), attack("t_c"), attack("t_d"), attack("t_key")]));
    write(&data, "item_families", 1, families);
    write(&data, "items", 2, items);
    write(&data, "classes", 1, classes);
    (t, data)
}

fn techniques(list: &[(&str, &str)]) -> Value {
    Value::Array(list.iter().map(|(s, g)| if g.is_empty() { json!({"skill": s}) } else { json!({"skill": s, "grip": g}) }).collect())
}

fn family_errors(data: &Path, id: &str) -> Vec<String> {
    let v = store::view(data);
    check::validate_entry(&v, "item_families", v.get("item_families", id), id).into_iter().filter(|e| e.contains("echnique") || e.contains("keystone")).collect()
}

#[test]
fn the_techniques_ride_on_the_items_and_stay_out_of_the_stored_forms() {
    let fam = weapon_family("t_sword", json!(["main_1h", "main_2h"]), json!({"techniques": techniques(&[("t_a", ""), ("t_b", "2h")]), "keystone": "t_key"}));
    let plain = weapon_family("t_axe", json!(["main_1h"]), json!({}));
    let items = json!([{"id": "i_sword", "name": "Sword", "family": "t_sword", "hand": "main_1h", "technique": {"skill": "t_c", "grip": "1h"}}, {"id": "i_axe", "name": "Axe", "family": "t_axe", "hand": "main_1h"}]);
    let (_t, data) = scratch("ride", None, json!([fam.clone(), plain.clone()]), items, json!([]));
    let src = source::load(&data);
    assert!(src.errors.iter().all(|e| e.starts_with("XP curve file")), "{:?}", src.errors);
    let cat = Catalog::from_value(&src.catalog);
    let (sword, axe) = (&cat.items["i_sword"], &cat.items["i_axe"]);
    let skills: Vec<(&str, &str)> = sword.family_techniques.iter().map(|t| (t.skill.as_str(), t.grip.as_str())).collect();
    assert_eq!(skills, [("t_a", ""), ("t_b", "2h")]);
    assert_eq!((sword.family_keystone.as_str(), sword.technique.as_ref().map(|t| (t.skill.as_str(), t.grip.as_str()))), ("t_key", Some(("t_c", "1h"))));
    assert!(axe.family_techniques.is_empty() && axe.family_keystone.is_empty() && axe.technique.is_none());
    let axe_row = src.catalog["items"].as_array().unwrap().iter().find(|r| r["id"] == "i_axe").unwrap().clone();
    assert!(axe_row.get(ITEM_FAMILY_TECHNIQUES).is_none() && axe_row.get(ITEM_FAMILY_KEYSTONE).is_none(), "{axe_row}");
    let fams: Vec<Dict> = src.game["item_families"].as_array().unwrap().iter().filter_map(|f| f.as_object().cloned()).collect();
    let stored = store::catalog_storage("items", json!({"id": "i_sword", "name": "Sword", "family": "t_sword", "hand": "main_1h"}).as_object().unwrap(), &fams).unwrap();
    assert!(!stored.contains_key(ITEM_FAMILY_TECHNIQUES) && !stored.contains_key(ITEM_FAMILY_KEYSTONE) && !stored.contains_key("technique"), "{stored:?}");
    let with = store::catalog_storage("item_families", fam.as_object().unwrap(), &[]).unwrap();
    assert_eq!((&with["techniques"], &with["keystone"]), (&techniques(&[("t_a", ""), ("t_b", "2h")]), &json!("t_key")));
    let without = store::catalog_storage("item_families", plain.as_object().unwrap(), &[]).unwrap();
    assert!(!without.contains_key("techniques") && !without.contains_key("keystone"), "none stores as no key: {without:?}");
    assert!(family_errors(&data, "t_sword").is_empty(), "{:?}", family_errors(&data, "t_sword"));
    let v = store::view(&data);
    let sword_errors = check::validate_entry(&v, "items", v.get("items", "i_sword"), "i_sword");
    assert!(sword_errors.is_empty(), "{sword_errors:?}");
}

#[test]
fn the_family_check_reads_the_techniques() {
    let four: Vec<(&str, &str)> = vec![("t_a", ""), ("t_b", ""), ("t_c", ""), ("t_d", "")];
    let cases: Vec<(Value, Value, String)> = vec![
        (json!(["main_1h"]), json!({"techniques": techniques(&four[..3])}), String::new()),
        (json!(["main_1h"]), json!({"techniques": techniques(&four)}), format!("A family lists at most {MASTERY_MAX} techniques (one per mastery level); it lists 4.")),
        (json!(["off"]), json!({"techniques": techniques(&four[..1])}), "Techniques and a keystone need a main-hand WEAPON family (only those gain mastery).".into()),
        (json!(["off"]), json!({"keystone": "t_key"}), "Techniques and a keystone need a main-hand WEAPON family (only those gain mastery).".into()),
        (json!(["main_1h"]), json!({"techniques": techniques(&[("t_zz", "")])}), "Technique 1: unknown skill 't_zz'.".into()),
        (json!(["main_1h"]), json!({"techniques": techniques(&[("t_a", "3h")])}), "Technique 1: grip '3h' is not one of \"\", 1h, 2h.".into()),
        (json!(["main_1h"]), json!({"techniques": techniques(&[("t_a", ""), ("t_a", "1h")])}), "Technique 2: 't_a' is listed twice.".into()),
        (json!(["main_1h"]), json!({"techniques": [{"skill": "t_a", "hand": "x"}]}), "Technique 1: unknown key 'hand' (skill, grip).".into()),
    ];
    for (i, (hands, extra, want)) in cases.into_iter().enumerate() {
        let id = if hands[0] == "off" { "orb" } else { "t_fam" };
        let (_t, data) = scratch(&format!("check{i}"), None, json!([weapon_family(id, hands, extra.clone())]), json!([]), json!([]));
        let errors = family_errors(&data, id);
        if want.is_empty() {
            assert!(errors.is_empty(), "{extra}: {errors:?}");
        } else {
            assert!(errors.iter().any(|e| *e == want), "{extra}: {errors:?}");
        }
    }
}

#[test]
fn a_book_names_a_main_hand_weapon_family_and_a_weapon_technique_is_checked() {
    let fams = json!([weapon_family("t_sword", json!(["main_1h"]), json!({})), weapon_family("orb", json!(["off"]), json!({})), {"id": "t_manual", "name": "Manual", "item_type": "MASTERY_BOOK"}]);
    let items = json!([
        {"id": "b_sword", "name": "Sword Book", "family": "t_manual", "mastery_family": "t_sword", "mastery_xp": 30},
        {"id": "b_orb", "name": "Orb Book", "family": "t_manual", "mastery_family": "orb"},
        {"id": "b_none", "name": "Blank Book", "family": "t_manual"},
        {"id": "i_bad", "name": "Bad", "family": "t_sword", "hand": "main_1h", "technique": {"skill": "t_zz", "grip": "2h"}}
    ]);
    let (_t, data) = scratch("book", None, fams, items, json!([]));
    let v = store::view(&data);
    let errors = |id: &str| check::validate_entry(&v, "items", v.get("items", id), id);
    assert!(errors("b_sword").is_empty(), "{:?}", errors("b_sword"));
    let cat = Catalog::from_value(&source::load(&data).catalog);
    assert_eq!(cat.items["b_sword"].book.as_ref().map(|b| (b.family.clone(), b.xp)), Some(("t_sword".to_string(), 30)));
    // PK2-FOLLOWUP (PR5-36): a book naming no XP gives the default (+150), the schema's and the kit's alike
    let book_xp = aegis_content::schema::fields(source::schema(), "content", "items").iter().map(json::dict).find(|f| json::s(f, "key") == rpgkit_data::mastery::BOOK_XP_KEY).map(|f| json::i(f, "default", 0));
    assert_eq!(book_xp, Some(i64::from(rpgkit_data::mastery::BOOK_XP_DEFAULT)));
    assert_eq!(cat.items["b_orb"].book.as_ref().map(|b| b.xp), Some(i64::from(rpgkit_data::mastery::BOOK_XP_DEFAULT)));
    assert!(errors("b_orb").contains(&"Mastery family 'orb' is not a main-hand weapon family (only those gain mastery).".to_string()), "{:?}", errors("b_orb"));
    assert!(errors("b_none").contains(&"A mastery book names the weapon family it trains (mastery_family).".to_string()), "{:?}", errors("b_none"));
    assert!(errors("i_bad").contains(&"Masterwork technique: unknown skill 't_zz'.".to_string()), "{:?}", errors("i_bad"));
}

#[test]
fn a_class_past_three_masterable_families_warns() {
    let fams: Vec<Value> = ["f_a", "f_b", "f_c", "f_d"].iter().map(|f| weapon_family(f, json!(["main_1h"]), json!({}))).chain(std::iter::once(weapon_family("orb", json!(["off"]), json!({})))).collect();
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let class = |id: &str, prof: Value| json!({"id": id, "name": id, "base_stats": stats, "growth": stats, "skill_path": {}, "proficiency": prof});
    let three = json!({"f_a": {"level": 3}, "f_b": {"level": 2}, "f_c": {"level": 2}, "f_d": {"level": 1}, "orb": {"level": 2}});
    let four = json!({"f_a": {"level": 3}, "f_b": {"level": 2}, "f_c": {"level": 2}, "f_d": {"level": 2}});
    let (_t, data) = scratch("class", None, Value::Array(fams), json!([]), json!([class("c_three", three), class("c_four", four)]));
    let v = store::view(&data);
    let warns = |id: &str| check::warnings_for(&v, "classes", v.get("classes", id)).into_iter().filter(|w| w.contains("masterable")).collect::<Vec<_>>();
    assert!(warns("c_three").is_empty(), "a proficiency 1 family and an off-hand focus do not count: {:?}", warns("c_three"));
    assert_eq!(warns("c_four"), [format!("The class has 4 masterable weapon families (f_a, f_b, f_c, f_d); a class masters at most {MASTERABLE_MAX} (main-hand weapon families at proficiency 2+).")]);
}

#[test]
fn the_manifest_names_the_act_mastery_starts_in() {
    let (_t, data) = scratch("from_act", Some(json!({DURABILITY: {FROM_ACT: 2}})), json!([]), json!([]), json!([]));
    assert_eq!(source::load(&data).from_act, 2);
    let (_n, none) = scratch("no_block", Some(json!({})), json!([]), json!([]), json!([]));
    assert_eq!(source::load(&none).from_act, 0, "no block: never");
    let m = |v: Value| -> Dict { json::dict(&json!({DURABILITY: v})).clone() };
    assert!(source::durability_problems(&m(json!({FROM_ACT: 1}))).is_empty());
    assert!(source::durability_problems(&Dict::new()).is_empty());
    let range = format!("game.json: {DURABILITY}.{FROM_ACT} must be a whole number 1-{FROM_ACT_MAX}.");
    for bad in [json!(0), json!(FROM_ACT_MAX + 1), json!("two"), json!(1.5)] {
        assert_eq!(source::durability_problems(&m(json!({FROM_ACT: bad.clone()}))), [range.clone()], "{bad}");
        assert_eq!(source::from_act_of(&m(json!({FROM_ACT: bad.clone()}))), 0, "{bad} reads as none");
    }
    assert_eq!(source::durability_problems(&m(json!(2))), [format!("game.json: {DURABILITY} must be an object {{{FROM_ACT}}}.")]);
    assert_eq!(source::durability_problems(&m(json!({FROM_ACT: 2, "max": 9}))), [format!("game.json: {DURABILITY}: unknown key 'max' (known: {FROM_ACT}).")]);
    let (t, _data) = scratch("writer", Some(json!({})), json!([]), json!([]), json!([]));
    let path = t.join("game.json");
    store::set_manifest(&path, json::dict(&json!({DURABILITY: {FROM_ACT: 3}}))).expect("the writer takes the block");
    let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(written[DURABILITY], json!({FROM_ACT: 3}));
    assert_eq!(store::set_manifest(&path, json::dict(&json!({DURABILITY: {FROM_ACT: 0}}))).unwrap_err(), [range]);
}
