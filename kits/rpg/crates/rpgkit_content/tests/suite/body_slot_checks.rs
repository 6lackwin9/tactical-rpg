//! PK2-S2 (`docs/sessions/PK2_GRAFTS.md` section 3): the catalog kind `body_slots`, the item type `GRAFT` and a meter
//! effect's `floors`, on a scratch data folder built here (no shipped data). A missing file is an empty table; a body
//! slot row's checks (id, name, role, origin) and across rows (unique ids); the loader keeps the rows that pass and they
//! reach `Catalog::body_slots`; the writer takes a row and re-saves it byte for byte. A `GRAFT` item checks its body
//! role (the schema's enum), keystone mark (a tier 3 class piece), class and `body_effects` (the passive grammar
//! without a member grant), with or without item families, and reaches `ItemDef::graft`; a meter effect's `floors` is
//! 1-8 whole floors in the meter's range, never falling.

use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, BODY_SLOTS_KIND, BODY_SLOTS_MAX, METER_FLOORS_MAX};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_body_slots_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

fn with(mut row: Value, key: &str, v: Value) -> Value {
    row[key] = v;
    row
}

fn slot(id: &str, role: &str) -> Value {
    json!({"id": id, "name": "Test Slot", "role": role})
}

/// The rows a graft names: a class, a skill, a passive and a unit meter.
fn base_rows(data: &Path) {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    write_kind(data, "classes", json!([{"id": "adept", "name": "Adept", "base_stats": stats, "growth": stats, "skill_path": {}}]));
    // a version 2 skills file (families and ranks)
    let skills = json!([{"id": "t_claw", "name": "Claw", "kind": "ATTACK", "level": 1, "blocks": 2, "power_mult": 1.0, "channel": {"only": true}}]);
    fs::write(data.join("catalog/skills.json"), json!({"kind": "skills", "version": 2, "entries": skills}).to_string()).unwrap();
    write_kind(data, "unit_meters", json!([{"id": "t_gauge", "name": "Gauge", "short": "Gauge", "min": 0, "max": 100, "start": 0, "show": "bar"}]));
}

fn graft(id: &str) -> Value {
    json!({"id": id, "name": "Test Graft", "item_type": "GRAFT", "tier": 1, "body_role": "monster",
        "body_effects": [{"type": "skill", "skill": "t_claw"}, {"type": "meter", "meter": "t_gauge", "floors": [5, 15, 30, 50]}]})
}

#[test]
fn a_missing_file_is_an_empty_table() {
    let (_scratch, data) = scratch("missing");
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[BODY_SLOTS_KIND]).is_empty());
    assert!(src.errors.iter().all(|e| !e.contains(&format!("{BODY_SLOTS_KIND}.json"))), "{:?}", src.errors);
    assert!(Catalog::from_value(&src.catalog).body_slots.is_empty());
}

#[test]
fn each_body_slot_mistake_has_its_text() {
    let (_scratch, data) = scratch("slot_texts");
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, BODY_SLOTS_KIND, &dict(e), "");
    assert_eq!(check(slot("s_one", "monster")), Vec::<String>::new());
    assert_eq!(check(slot("s_one", "class")), Vec::<String>::new());
    assert_eq!(check(slot("s_one", "spare")), ["Role: 'spare' is not one of monster, class."]);
    assert_eq!(check(with(slot("s_one", "monster"), "name", json!(""))), ["A body slot needs a name."]);
    assert_eq!(check(with(slot("s_one", "monster"), "name", json!("x".repeat(33)))), ["Name is 33 characters (max 32)."]);
    let bad_id = check(slot("S", "monster"));
    assert!(bad_id.iter().any(|e| e.starts_with("ID 'S'")), "{bad_id:?}");
}

#[test]
fn the_loader_keeps_the_body_slots_that_pass() {
    let (_scratch, data) = scratch("slot_loader");
    let mut rows = vec![slot("s_head", "class"), slot("s_arm", "monster"), slot("s_arm", "monster"), slot("s_bad", "spare")];
    rows.extend((0..BODY_SLOTS_MAX).map(|i| slot(&format!("s_extra{i}"), "monster")));
    write_kind(&data, BODY_SLOTS_KIND, Value::Array(rows));
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[BODY_SLOTS_KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    // the duplicate and the bad role are refused; past the cap (counted over every row in file order) too
    let mut want = vec!["s_head".to_string(), "s_arm".to_string()];
    want.extend((0..BODY_SLOTS_MAX - 4).map(|i| format!("s_extra{i}")));
    assert_eq!(kept, want);
    assert!(src.errors.iter().any(|e| e.ends_with("body slot 's_arm': ID 's_arm' is already used by another body slot.")), "{:?}", src.errors);
    assert!(src.errors.iter().any(|e| e.ends_with(&format!("body slot 's_extra{}': A catalog holds at most {BODY_SLOTS_MAX} body slots.", BODY_SLOTS_MAX - 4))), "{:?}", src.errors);
    let cat = Catalog::from_value(&src.catalog);
    assert!(cat.body_slots["s_head"].is_class() && !cat.body_slots["s_arm"].is_class());
}

#[test]
fn the_writer_takes_a_body_slot() {
    let (_scratch, data) = scratch("slot_writer");
    let file = data.join("catalog").join(format!("{BODY_SLOTS_KIND}.json"));
    let refused = store::upsert(&data, None, BODY_SLOTS_KIND, &dict(slot("s_one", "spare")), None).err().expect("a bad row is refused");
    assert!(refused.iter().any(|e| e.ends_with("Role: 'spare' is not one of monster, class.")), "{refused:?}");
    store::upsert(&data, None, BODY_SLOTS_KIND, &dict(slot("s_one", "class")), None).expect("a valid row");
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, BODY_SLOTS_KIND).unwrap(), text, "a re-save without changes is byte-identical");
    assert_eq!(Catalog::from_value(&source::load(&data).catalog).body_slots["s_one"].role, "class");
}

fn item_errors(v: &check::View, e: Value) -> Vec<String> {
    check::validate_entry(v, "items", &dict(e), "")
}

/// Without item families (flat rows).
#[test]
fn each_graft_mistake_has_its_text() {
    let (_scratch, data) = scratch("graft_texts");
    base_rows(&data);
    let v = store::view(&data);
    assert_eq!(item_errors(&v, graft("g_one")), Vec::<String>::new());
    let keystone = with(with(with(with(graft("g_key"), "body_role", json!("class")), "keystone", json!(true)), "class_id", json!("adept")), "tier", json!(3));
    assert_eq!(item_errors(&v, keystone.clone()), Vec::<String>::new(), "a tier 3 class keystone piece");
    for tier in [1, 2] {
        assert_eq!(item_errors(&v, with(keystone.clone(), "tier", json!(tier))), ["A keystone graft is a tier 3 graft (it needs 3 grafts first, as every tier 3 graft)."], "tier {tier}");
    }
    assert_eq!(item_errors(&v, with(graft("g_one"), "body_role", json!("spare"))), ["Body role: 'spare' is not one of monster, class."]);
    assert_eq!(item_errors(&v, with(with(graft("g_one"), "keystone", json!(true)), "tier", json!(3))), ["A keystone graft is a class piece (body role class)."]);
    assert_eq!(item_errors(&v, with(graft("g_one"), "class_id", json!("adept"))), ["Only a class piece (body role class) names a class."]);
    assert_eq!(item_errors(&v, with(with(graft("g_one"), "body_role", json!("class")), "class_id", json!("nope"))), ["Class: unknown classe 'nope'."], "the schema's ref check");
    let member = item_errors(&v, with(graft("g_one"), "body_effects", json!([{"type": "pierce_melee"}])));
    assert_eq!(member, ["Body effects: Effect 1: 'pierce_melee' is a passive's member grant; a graft can't carry it."]);
    let unknown = item_errors(&v, with(graft("g_one"), "body_effects", json!([{"type": "skill", "skill": "t_nope"}])));
    assert_eq!(unknown, ["Body effects: Effect 1: unknown skill 't_nope'."]);
}

/// With item families: a `GRAFT` family resolves the type; the graft reaches the catalog with its body fields.
#[test]
fn a_graft_resolves_through_its_family() {
    let (_scratch, data) = scratch("graft_family");
    base_rows(&data);
    let family = json!({"id": "t_graft", "name": "Graft", "item_type": "GRAFT", "base": {}, "bands": {}, "adjustable": [], "modifier_kinds": [], "max_modifiers": 0, "max_tiers": 0});
    write_kind(&data, "item_families", json!([family]));
    let mut row = graft("g_one");
    row.as_object_mut().unwrap().remove("item_type");
    row["family"] = json!("t_graft");
    store::upsert(&data, None, "items", &dict(row.clone()), None).expect("a valid graft");
    let src = source::load(&data);
    assert!(src.errors.iter().all(|e| e.contains("xp_curve")), "{:?}", src.errors);
    let cat = Catalog::from_value(&src.catalog);
    let item = &cat.items["g_one"];
    let g = item.graft.as_ref().expect("a graft");
    assert_eq!((item.item_type.as_str(), g.tier, g.role.as_str(), g.keystone, g.effects.len()), ("GRAFT", 1, "monster", false, 2));
    assert!(item.effects.is_empty() && !item.is_equipment(), "no gear effect, never worn");
    let refused = store::upsert(&data, None, "items", &dict(with(with(row, "keystone", json!(true)), "tier", json!(3))), None).err().expect("a monster keystone is refused");
    assert!(refused.iter().any(|e| e.ends_with("A keystone graft is a class piece (body role class).")), "{refused:?}");
}

#[test]
fn a_meter_effect_checks_its_floors() {
    let (_scratch, data) = scratch("floors");
    base_rows(&data);
    let v = store::view(&data);
    let floors = |f: Value| item_errors(&v, with(graft("g_one"), "body_effects", json!([{"type": "meter", "meter": "t_gauge", "floors": f}])));
    assert_eq!(floors(json!([5, 15, 30, 50])), Vec::<String>::new());
    assert_eq!(floors(json!([10, 10])), Vec::<String>::new(), "a floor may stay");
    let list = format!("Body effects: Effect 1: floors is a list of 1-{METER_FLOORS_MAX} floors (by graft count: 1, 2, ...).");
    assert_eq!(floors(json!([])), [list.clone()]);
    assert_eq!(floors(json!(5)), [list.clone()]);
    assert_eq!(floors(Value::Array(vec![json!(1); METER_FLOORS_MAX + 1])), [list]);
    assert_eq!(floors(json!([5, 101])), ["Body effects: Effect 1: each of floors must be a whole number 0-100."]);
    assert_eq!(floors(json!([15, 5])), ["Body effects: Effect 1: floors never falls as the graft count grows."]);
    // a passive's meter may name a table too (no graft count: no table floor)
    let passive = check::validate_entry(&v, "passives", &dict(json!({"id": "p_one", "name": "One", "effects": [{"type": "meter", "meter": "t_gauge", "floors": [5]}]})), "");
    assert_eq!(passive, Vec::<String>::new());
}
