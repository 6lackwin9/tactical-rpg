//! PK2-S3 (`docs/sessions/PK2_GRAFTS.md` sections 4, 5): the catalog kind `recipes`, a class proficiency entry's `role`
//! and an item's `theme`, on a scratch data folder built here (no shipped data). A missing file is an empty table; a
//! recipe row's checks (id, name, npc, target, cost, effects: known, their keys and target, their refs, the effects whose
//! system is not built yet) and across rows (unique ids); the world refs (the offering character, the `requires`
//! conditions) in the whole-catalog report and the Forge's check; the loader keeps the rows that pass and they reach
//! `Catalog::recipes`; the writer takes a row and re-saves it byte for byte.

use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, RECIPES_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_recipes_{tag}"));
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

/// The rows a recipe names: a class with proficiency roles, a skill, a unit meter, a graft, materials, two body slots
/// (a monster and a class slot), a unit track; a world `vale` with the offering character.
fn base_rows(data: &Path) {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    write_kind(data, "classes", json!([{"id": "adept", "name": "Adept", "base_stats": stats, "growth": stats, "skill_path": {}}]));
    let skills = json!([{"id": "t_claw", "name": "Claw", "kind": "ATTACK", "level": 1, "blocks": 2, "power_mult": 1.0, "channel": {"only": true}}]);
    fs::write(data.join("catalog/skills.json"), json!({"kind": "skills", "version": 2, "entries": skills}).to_string()).unwrap();
    write_kind(data, "unit_meters", json!([{"id": "t_gauge", "name": "Gauge", "short": "Gauge", "min": 0, "max": 100, "start": 0, "show": "bar"}]));
    write_kind(data, "item_families", json!([{"id": "t_graft", "name": "Graft", "item_type": "GRAFT"}, {"id": "t_mat", "name": "Material", "item_type": "MATERIAL"}]));
    let items = json!([
        {"id": "g_claw", "name": "Claw Graft", "family": "t_graft", "tier": 1, "body_role": "monster", "body_effects": [{"type": "skill", "skill": "t_claw"}]},
        {"id": "core", "name": "Core", "family": "t_mat"},
        {"id": "fine", "name": "Fine", "family": "t_mat"}
    ]);
    fs::write(data.join("catalog/items.json"), json!({"kind": "items", "version": 2, "entries": items}).to_string()).unwrap();
    write_kind(data, "body_slots", json!([{"id": "s_arm", "name": "Arm", "role": "monster"}, {"id": "s_head", "name": "Head", "role": "class"}]));
    write_kind(data, "unit_tracks", json!([{"id": "rot", "name": "Rot", "stages": [{"name": "One"}]}]));
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    fs::write(dir.join("characters.json"), json!({"kind": "characters", "version": 1, "entries": [{"id": "surgeon", "name": "Surgeon"}]}).to_string()).unwrap();
}

fn surgery(id: &str) -> Value {
    json!({"id": id, "name": "Surgery", "npc": "surgeon", "requires": [], "cost": {"gold": 10, "items": {"core": 1}}, "target": "member",
        "effects": [{"effect": "install", "item": "g_claw", "slot": "s_arm"}, {"effect": "track", "track": "rot", "delta": 1}]})
}

fn fx(effects: Value) -> Value {
    with(surgery("r_one"), "effects", effects)
}

#[test]
fn a_missing_file_is_an_empty_table() {
    let (_scratch, data) = scratch("missing");
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[RECIPES_KIND]).is_empty());
    assert!(src.errors.iter().all(|e| !e.contains(&format!("{RECIPES_KIND}.json"))), "{:?}", src.errors);
    assert!(Catalog::from_value(&src.catalog).recipes.is_empty());
}

#[test]
fn each_recipe_mistake_has_its_text() {
    let (_scratch, data) = scratch("texts");
    base_rows(&data);
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, RECIPES_KIND, &dict(e), "");
    assert_eq!(check(surgery("r_one")), Vec::<String>::new());
    let every = json!([
        {"effect": "remove", "slot": "s_arm"}, {"effect": "max_hp_pct", "pct": -15}, {"effect": "floor_offset", "delta": -10},
        {"effect": "train_proficiency", "role": "secondary", "to": 3}, {"effect": "doctrine", "axis": "flesh", "delta": 1}
    ]);
    assert_eq!(check(fx(every)), Vec::<String>::new(), "every built member effect");
    let salvage = json!({"id": "r_break", "name": "Break", "npc": "vale:surgeon", "target": "item", "effects": [{"effect": "salvage", "gold": 2, "tiers": {"RARE": "fine"}, "themes": {"order": "fine"}}]});
    assert_eq!(check(salvage.clone()), Vec::<String>::new(), "a salvage row; a qualified npc");
    assert_eq!(check(with(surgery("r_one"), "name", json!(""))), ["A recipe needs a name."]);
    assert_eq!(check(with(surgery("r_one"), "npc", json!(""))), ["A recipe needs the character who offers it (npc)."]);
    assert_eq!(check(with(surgery("r_one"), "npc", json!("nobody"))), ["Offered by: unknown character 'nobody'."]);
    assert_eq!(check(with(surgery("r_one"), "target", json!("party"))).first().map(String::as_str), Some("Target: 'party' is not one of member, weapon, item."));
    assert_eq!(check(with(surgery("r_one"), "cost", json!({"gold": -1}))), ["Cost gold must be a whole number 0..9999999."]);
    assert_eq!(check(with(surgery("r_one"), "cost", json!({"items": {"nope": 1}}))), ["Cost items: unknown item 'nope'."]);
    assert_eq!(check(with(surgery("r_one"), "cost", json!({"items": {"core": 0}}))), ["Cost items: core count must be a whole number 1..99."]);
    assert_eq!(check(with(surgery("r_one"), "cost", json!({"coins": 1}))), ["Cost: unknown key 'coins' (use gold, supplies, items)."]);
    assert_eq!(check(fx(json!([]))), ["Effects must be a list of 1-8 effects."]);
    assert!(check(fx(json!([{"effect": "heal"}])))[0].starts_with("Effect 1: unknown effect 'heal' (use install, remove,"));
    assert_eq!(check(fx(json!([{"effect": "remove", "slot": "s_arm", "item": "g_claw"}]))), ["Effect 1 (remove): unknown key 'item'."]);
    assert_eq!(check(fx(json!([{"effect": "remove", "slot": "s_leg"}]))), ["Effect 1 (remove): unknown body slot 's_leg'."]);
    assert_eq!(check(fx(json!([{"effect": "install", "item": "core", "slot": "s_arm"}]))), ["Effect 1 (install): 'core' is not a graft."]);
    assert_eq!(check(fx(json!([{"effect": "install", "item": "g_claw", "slot": "s_head"}]))), ["Effect 1 (install): body slot 's_head' does not take monster grafts."]);
    assert_eq!(check(fx(json!([{"effect": "track", "track": "nope", "delta": 1}]))), ["Effect 1 (track): unknown unit track 'nope'."]);
    assert_eq!(check(fx(json!([{"effect": "track", "track": "rot", "delta": 0}]))), ["Effect 1 (track): delta must be a whole number -100..100, not 0."]);
    assert_eq!(check(fx(json!([{"effect": "max_hp_pct", "pct": -95}]))), ["Effect 1 (max_hp_pct): pct must be a whole number -90..100, not 0."]);
    assert_eq!(check(fx(json!([{"effect": "train_proficiency", "role": "fifth", "to": 3}]))), ["Effect 1 (train_proficiency): role 'fifth' is not one of main, secondary, third, minor."]);
    assert_eq!(check(fx(json!([{"effect": "train_proficiency", "role": "minor", "to": 4}]))), ["Effect 1 (train_proficiency): to must be the level reached, a whole number 1..3."]);
    assert_eq!(check(fx(json!([{"effect": "doctrine", "axis": "", "delta": 1}]))), ["Effect 1 (doctrine): axis '' must be an id of 1-24 characters (a-z, 0-9 and _)."]);
    assert_eq!(check(fx(json!([{"effect": "salvage", "gold": 1}]))), ["Effect 1 (salvage): works on a item, but the recipe's target is member."]);
    assert_eq!(check(with(salvage.clone(), "effects", json!([{"effect": "salvage"}]))), ["Effect 1 (salvage): a salvage gives gold, tiers or themes."]);
    assert_eq!(check(with(salvage.clone(), "effects", json!([{"effect": "salvage", "tiers": {"SHINY": "fine"}}]))), ["Effect 1 (salvage): tiers key 'SHINY' is not one of COMMON, UNCOMMON, RARE, EPIC, LEGENDARY."]);
    assert_eq!(check(with(salvage, "effects", json!([{"effect": "salvage", "themes": {"order": "nope"}}]))), ["Effect 1 (salvage): themes order: unknown item 'nope'."]);
    let bad_cond = check(with(surgery("r_one"), "requires", json!([{"kind": "mood"}])));
    assert!(bad_cond.len() == 1 && bad_cond[0].starts_with("Requires 1: unknown condition kind 'mood'"), "the dialogue grammar: {bad_cond:?}");
}

#[test]
fn the_unbuilt_effects_are_refused_until_their_system_lands() {
    let (_scratch, data) = scratch("pending");
    base_rows(&data);
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, RECIPES_KIND, &dict(e), "");
    // PK2-S4: `drill` is built (weapon mastery): its xp is checked, nothing refuses it
    assert!(check(fx(json!([{"effect": "drill", "xp": 10}]))).is_empty());
    assert_eq!(check(fx(json!([{"effect": "drill", "xp": 0}]))), ["Effect 1 (drill): xp must be a whole number 1..999."]);
    assert_eq!(check(fx(json!([{"effect": "drill"}]))), ["Effect 1 (drill): xp must be a whole number 1..999."]);
    let weapon = |effects: Value| with(with(surgery("r_one"), "target", json!("weapon")), "effects", effects);
    assert_eq!(check(weapon(json!([{"effect": "repair"}]))), ["Effect 1 (repair): needs weapon instances, which this build does not have yet."]);
    assert_eq!(check(weapon(json!([{"effect": "reforge", "masterwork": true}]))), ["Effect 1 (reforge): needs weapon instances, which this build does not have yet."]);
}

#[test]
fn the_loader_keeps_the_recipes_that_pass_and_the_report_names_the_world_refs() {
    let (_scratch, data) = scratch("loader");
    base_rows(&data);
    let lost = with(surgery("r_lost"), "npc", json!("nobody"));
    write_kind(&data, RECIPES_KIND, json!([surgery("r_one"), surgery("r_one"), fx(json!([{"effect": "heal"}])), lost]));
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[RECIPES_KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(kept, ["r_one", "r_lost"], "the duplicate and the unknown effect are refused; a world ref is the report's");
    assert!(src.errors.iter().any(|e| e.ends_with("recipe 'r_one': ID 'r_one' is already used by another recipe.")), "{:?}", src.errors);
    let report = check::validate_all(&store::view(&data));
    assert_eq!(report.get("recipes:r_lost").cloned().unwrap_or_default(), ["Offered by: unknown character 'nobody'."]);
    assert!(!report.contains_key("recipes:r_one"));
    let cat = Catalog::from_value(&src.catalog);
    let r = &cat.recipes["r_one"];
    assert_eq!((r.npc.as_str(), r.target.as_str(), r.cost.gold, r.cost.items["core"], r.effect_names()), ("surgeon", "member", 10, 1, vec!["install".to_string(), "track".to_string()]));
}

#[test]
fn the_writer_takes_a_recipe() {
    let (_scratch, data) = scratch("writer");
    base_rows(&data);
    let file = data.join("catalog").join(format!("{RECIPES_KIND}.json"));
    let weapon = with(with(surgery("r_weapon"), "target", json!("weapon")), "effects", json!([{"effect": "repair"}]));
    let refused = store::upsert(&data, None, RECIPES_KIND, &dict(weapon), None).err().expect("a pending effect is refused");
    assert!(refused.iter().any(|e| e.ends_with("needs weapon instances, which this build does not have yet.")), "{refused:?}");
    store::upsert(&data, None, RECIPES_KIND, &dict(surgery("r_one")), None).expect("a valid row");
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, RECIPES_KIND).unwrap(), text, "a re-save without changes is byte-identical");
    assert_eq!(Catalog::from_value(&source::load(&data).catalog).recipes["r_one"].effects.len(), 2);
}

#[test]
fn a_class_proficiency_role_and_an_item_theme() {
    let (_scratch, data) = scratch("role_theme");
    base_rows(&data);
    let v = store::view(&data);
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let class = |role: &str| json!({"id": "adept", "name": "Adept", "base_stats": stats, "growth": stats, "skill_path": {}, "proficiency": {"sword": {"level": 2, "role": role}}});
    let errors = |role: &str| check::validate_entry(&v, "classes", &dict(class(role)), "adept");
    assert!(!errors("secondary").iter().any(|e| e.contains("role")), "{:?}", errors("secondary"));
    assert!(errors("spare").contains(&"Proficiency sword: role 'spare' is not one of main, secondary, third, minor.".to_string()), "{:?}", errors("spare"));
    let roles = rpgkit_data::ClassDef::from_obj(json::dict(&class("third"))).proficiency_roles;
    assert_eq!(roles.get("sword").map(String::as_str), Some("third"));
    let blade = json!({"id": "t_blade", "name": "Blade", "family": "t_mat", "theme": "order"});
    assert_eq!(check::validate_entry(&v, "items", &dict(blade.clone()), ""), Vec::<String>::new());
    assert_eq!(check::validate_entry(&v, "items", &dict(with(blade.clone(), "theme", json!("Order Arms"))), ""), ["Theme 'Order Arms' must be an id (lowercase a-z, 0-9 and _)."]);
    store::upsert(&data, None, "items", &dict(blade), None).expect("a themed item");
    let cat = Catalog::from_value(&source::load(&data).catalog);
    assert_eq!((cat.items["t_blade"].theme.as_str(), cat.items["core"].theme.as_str()), ("order", ""));
    let saved = fs::read_to_string(data.join("catalog/items.json")).unwrap();
    assert_eq!(saved.matches("\"theme\"").count(), 1, "only a themed item writes the key");
}
