//! PR6-8 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 3-4): the `facilities` world kind's field table, loader and checks,
//! the `facility` service line, and a shop's `supply_price_mult_by_act` (B2). A facility written to section 4's shape
//! checks clean and each field set wrong adds exactly one error; the writer stores it and re-saves byte for byte; a shop
//! without the new key re-saves unchanged (the key is stored only when set). Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::facility::{FACILITIES_KIND, SUPPLY_MULT_KEY};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const W: &str = "ward";

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn write_catalog(data: &Path, kind: &str, entries: Value) {
    let version = if kind == "items" { 2 } else { 1 };
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(store::file_of(data, Some(W), kind), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

/// A facility written to section 4's shape (three levels, each effect once).
fn hearth() -> Value {
    json!({"id": "hearth", "name": "Hearth", "keeper_character": "smith", "levels": [
        {"level": 1, "name": "Salvage", "scrap_cost": 0, "materials": {}, "requires_flags": [], "effects": []},
        {"level": 2, "name": "Drills", "scrap_cost": 40, "materials": {"timber": 2}, "requires_flags": [{"key": "act2", "value": true}],
         "effects": [{"effect": "recipe_pct", "pct": 85, "recipe_tags": ["repair"]}, {"effect": "set_flag", "key": "hub.bunks_more", "value": true}]},
        {"level": 3, "name": "Rites", "scrap_cost": 90, "effects": [{"effect": "shop_pct", "pct": 75}, {"effect": "rest_upkeep", "delta": -1}]}
    ]})
}

fn stall() -> Value {
    json!({"id": "stall", "name": "Stall", "keeper": "smith", "stock": [{"item": "timber", "count": -1, "price": -1, "requires_flags": []}], "loot": [],
        "buy_mult": 1.0, "supply_price": 8, "sell_mult": 0.5, "buys": ["MATERIAL"], "requires_flags": [], "tags": []})
}

/// A scratch world: a class, a party unit, a material, the manifest, a location, Smith, his stall and the hearth.
fn world(tag: &str) -> PathBuf {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_facility_checks_{tag}"));
    let data = t.join("data");
    t.until_thread_end();
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    write_catalog(&data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2}}]));
    write_catalog(&data, "units", json!([{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}]));
    write_catalog(&data, "items", json!([{"id": "timber", "name": "Timber", "item_type": "MATERIAL", "value": 2}]));
    fs::write(aegis_content::world::manifest_path(&data, W), json!({"id": W, "name": "Ward", "extends": [], "connects": [], "start_location": "hub", "version": 1}).to_string()).unwrap();
    write_kind(&data, "locations", json!([{"id": "hub", "name": "Hub", "rest": true}]));
    write_kind(&data, "characters", json!([{"id": "smith", "name": "Smith", "role": "npc"}]));
    write_kind(&data, "shops", json!([stall()]));
    write_kind(&data, FACILITIES_KIND, json!([hearth()]));
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn errors_of(v: &check::View, kind: &str, entry: &Value) -> Vec<String> {
    check::world::validate_entry(v, W, kind, entry.as_object().unwrap(), &json::s(entry.as_object().unwrap(), "id"))
}

fn added(v: &check::View, kind: &str, good: &Value, bad: &Value) -> Vec<String> {
    let before = errors_of(v, kind, good);
    errors_of(v, kind, bad).into_iter().filter(|e| !before.contains(e)).collect()
}

fn with(entry: &Value, path: &[&str], value: Value) -> Value {
    let mut out = entry.clone();
    let mut at = &mut out;
    for p in path {
        at = match p.parse::<usize>() {
            Ok(n) => &mut at[n],
            Err(_) => &mut at[*p],
        };
    }
    *at = value;
    out
}

fn one(found: Vec<String>, needle: &str, tag: &str) {
    assert_eq!(found.len(), 1, "{tag}: {found:?}");
    assert!(found[0].contains(needle), "{tag}: {found:?} (wanted '{needle}')");
}

#[test]
fn the_schema_carries_the_facilities_kind_and_the_shop_key() {
    let schema = source::schema();
    let field = |kind: &str, key: &str| fields(schema, "world", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("world.{kind}.{key}"));
    assert_eq!((json::s(&field(FACILITIES_KIND, "keeper_character"), "type"), json::s(&field(FACILITIES_KIND, "keeper_character"), "ref")), ("ref".to_string(), "characters".to_string()));
    assert_eq!((json::s(&field(FACILITIES_KIND, "levels"), "type"), json::get(&field(FACILITIES_KIND, "levels"), "default")), ("json".to_string(), &json!([])));
    let m = field("shops", SUPPLY_MULT_KEY);
    assert_eq!((json::s(&m, "type"), json::get(&m, "default"), json::b(&m, "omit_default", false)), ("json".to_string(), &json!({}), true), "stored only when set");
    assert!(source::WORLD_KINDS.contains(&FACILITIES_KIND) && check::world::KINDS.contains(&FACILITIES_KIND), "a loaded and checked world kind");
    let data = world("schema");
    let src = source::load(&data);
    let row = src.worlds[W]["own"][FACILITIES_KIND][0].as_object().cloned().expect("the loaded row");
    let levels = rpgkit_data::facility::levels(&row);
    assert_eq!(levels.iter().map(|l| (l.level, l.scrap_cost)).collect::<Vec<_>>(), [(1, 0), (2, 40), (3, 90)]);
    assert_eq!(levels[1].materials.get("timber").copied(), Some(2));
    drop_scratch(&data);
}

#[test]
fn a_facility_checks_clean_and_each_bad_field_is_one_error() {
    let data = world("rows");
    let v = store::view(&data);
    let good = hearth();
    let found = errors_of(&v, FACILITIES_KIND, &good);
    assert!(found.is_empty(), "{found:?}");
    let cases: Vec<(&str, Value, &str)> = vec![
        ("no keeper", with(&good, &["keeper_character"], json!("")), "choose the keeper"),
        ("an unknown keeper", with(&good, &["keeper_character"], json!("ghost")), "unknown character 'ghost'"),
        ("no levels", with(&good, &["levels"], json!([])), "levels need at least level 1"),
        ("a level out of order", with(&good, &["levels", "1", "level"], json!(3)), "level must be 2"),
        ("a nameless level", with(&good, &["levels", "2", "name"], json!(" ")), "name is required"),
        ("a cost too high", with(&good, &["levels", "1", "scrap_cost"], json!(100000)), "scrap_cost must be a whole number 0..99999"),
        ("an unknown material", with(&good, &["levels", "1", "materials"], json!({"ghost": 1})), "unknown material item 'ghost'"),
        ("a material count of 0", with(&good, &["levels", "1", "materials", "timber"], json!(0)), "material 'timber' count must be 1..99"),
        ("an unknown level key", with(&good, &["levels", "2", "text"], json!("x")), "unknown key 'text'"),
        ("an unknown effect", with(&good, &["levels", "2", "effects", "0", "effect"], json!("bunks")), "unknown effect 'bunks'"),
        ("a pct past 100", with(&good, &["levels", "2", "effects", "0", "pct"], json!(120)), "pct must be a whole number 10..100"),
        ("a rest delta too large", with(&good, &["levels", "2", "effects", "1", "delta"], json!(10)), "delta must be a whole number -9..9"),
        ("a bad flag key", with(&good, &["levels", "1", "effects", "1", "key"], json!("Bad Key")), "must be a flag key"),
        ("a bad flag value", with(&good, &["levels", "1", "effects", "1", "value"], json!([1])), "value must be a bool, int or text"),
        ("an unknown effect key", with(&good, &["levels", "1", "effects", "0", "who"], json!(1)), "unknown key 'who'"),
    ];
    for (tag, bad, needle) in cases {
        one(added(&v, FACILITIES_KIND, &good, &bad), needle, tag);
    }
    let mut four = good.clone();
    four["levels"].as_array_mut().unwrap().push(json!({"level": 4, "name": "Too far", "scrap_cost": 1}));
    assert!(errors_of(&v, FACILITIES_KIND, &four).iter().any(|e| e.contains("at most 3 levels")));
    let blocking = check::world::blocking_errors(&v, W, FACILITIES_KIND, &dict(with(&good, &["levels"], json!({}))), "hearth");
    assert!(blocking.iter().any(|e| e.contains("must be a list")), "{blocking:?}");
    drop_scratch(&data);
}

#[test]
fn the_supply_multiplier_by_act_is_checked() {
    let data = world("supply");
    let v = store::view(&data);
    let good = stall();
    assert!(errors_of(&v, "shops", &good).is_empty(), "{:?}", errors_of(&v, "shops", &good));
    let set = with(&good, &[SUPPLY_MULT_KEY], json!({"3": 2.0, "2": 1.25}));
    assert!(errors_of(&v, "shops", &set).is_empty(), "{:?}", errors_of(&v, "shops", &set));
    let cases: Vec<(&str, Value, &str)> = vec![
        ("a list", with(&good, &[SUPPLY_MULT_KEY], json!([2.0])), "must be an object {act: mult}"),
        ("an act past 9", with(&good, &[SUPPLY_MULT_KEY], json!({"10": 2.0})), "'10' must be an act 0..9"),
        ("a mult of 0", with(&good, &[SUPPLY_MULT_KEY], json!({"3": 0.0})), "mult must be a number 0.1..10.0"),
        ("three decimals", with(&good, &[SUPPLY_MULT_KEY], json!({"3": 1.125})), "at most two decimals"),
        ("no supplies sold", with(&with(&good, &["supply_price"], json!(0)), &[SUPPLY_MULT_KEY], json!({"3": 2.0})), "needs a supply price"),
    ];
    for (tag, bad, needle) in cases {
        one(added(&v, "shops", &good, &bad), needle, tag);
    }
    drop_scratch(&data);
}

#[test]
fn the_writer_stores_facilities_and_shops_and_resaves_them_byte_for_byte() {
    let data = world("writer");
    fs::remove_file(store::file_of(&data, Some(W), FACILITIES_KIND)).unwrap();
    store::upsert(&data, Some(W), FACILITIES_KIND, &dict(hearth()), None).expect("a facility");
    let file = store::file_of(&data, Some(W), FACILITIES_KIND);
    assert!(file.ends_with("facilities.json"), "{file:?}");
    let stored = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, Some(W), FACILITIES_KIND).unwrap(), stored, "a re-save without changes is byte-identical");
    // a shop without the new key is written without it; with it, the key is kept
    store::upsert(&data, Some(W), "shops", &dict(stall()), None).expect("the stall");
    let shops = fs::read_to_string(store::file_of(&data, Some(W), "shops")).unwrap();
    assert!(!shops.contains(SUPPLY_MULT_KEY), "omitted while unset");
    assert_eq!(store::resave_text(&data, Some(W), "shops").unwrap(), shops);
    store::upsert(&data, Some(W), "shops", &dict(with(&stall(), &[SUPPLY_MULT_KEY], json!({"3": 2.0}))), None).expect("the stall with a multiplier");
    let shops = fs::read_to_string(store::file_of(&data, Some(W), "shops")).unwrap();
    assert!(shops.contains(SUPPLY_MULT_KEY), "kept when set");
    let problems: Vec<String> = store::check(&data).into_iter().filter(|p| p.starts_with(&format!("{W}/facilities")) || p.starts_with(&format!("{W}/shops"))).collect();
    assert!(problems.is_empty(), "{problems:?}");
    let refused = store::upsert(&data, Some(W), FACILITIES_KIND, &dict(with(&hearth(), &["levels"], json!({}))), None).err().expect("a shape problem blocks");
    assert!(refused.iter().any(|e| e.contains("must be a list")), "{refused:?}");
    drop_scratch(&data);
}
