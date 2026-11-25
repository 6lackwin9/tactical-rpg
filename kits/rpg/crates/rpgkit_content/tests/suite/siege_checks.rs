//! PR6-9 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 5, B6): the `sieges` world kind's field table, loader and checks,
//! the effect `start_siege`, and an encounter's objective `retreat` with its `hp_floor`. A siege written to section 5's
//! shape checks clean and each field set wrong adds exactly one error; the writer stores it and re-saves byte for byte;
//! an encounter without `hp_floor` is written without it (stored only when set). Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::facility::FACILITIES_KIND;
use rpgkit_data::siege::{SiegeDef, SIEGES_KIND};
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

/// A siege written to section 5's shape: three rooms, the second guarding the vault, win and loss effects.
fn sluice() -> Value {
    json!({"id": "sluice", "name": "Sluice", "rooms": [
        {"encounter": "grate", "name": "Grate"}, {"encounter": "lock", "name": "Lock", "facility": "vault"}, {"encounter": "hearth", "name": "Hearth"}],
        "carry_hp": true, "carry_downed": true, "carry_pouch": true, "no_camp": true, "restocks": 1,
        "win_effects": [{"effect": "set_flag", "key": "hub.held", "value": true}],
        "loss_effects": [{"effect": "set_flag", "key": "hub.fell", "value": true}]})
}

/// A retreat encounter: the captain withdraws at 25% HP.
fn gate() -> Value {
    json!({"id": "gate", "name": "Gate", "party": ["lead"], "enemies": ["captain", "rat"], "objective": "retreat", "objective_unit": "captain", "hp_floor": 25})
}

/// A scratch world: a class, a party unit, two enemy units, the manifest, a location, the characters, the rooms'
/// encounters, the vault facility and the siege.
fn world(tag: &str) -> PathBuf {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_siege_checks_{tag}"));
    let data = t.join("data");
    t.until_thread_end();
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    write_catalog(&data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2}}]));
    write_catalog(&data, "units", json!([
        {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1},
        {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}
    ]));
    fs::write(aegis_content::world::manifest_path(&data, W), json!({"id": W, "name": "Ward", "extends": [], "connects": [], "start_location": "hub", "version": 1}).to_string()).unwrap();
    write_kind(&data, "locations", json!([{"id": "hub", "name": "Hub", "rest": true}]));
    write_kind(&data, "characters", json!([
        {"id": "lead", "name": "Lead", "role": "party", "base_unit": "hero"},
        {"id": "keeper", "name": "Keeper", "role": "npc"},
        {"id": "captain", "name": "Captain", "role": "enemy", "base_unit": "brute"},
        {"id": "rat", "name": "Rat", "role": "enemy", "base_unit": "brute"}
    ]));
    write_kind(&data, "encounters", json!([
        {"id": "grate", "name": "Grate", "party": ["lead"], "enemies": ["rat"]},
        {"id": "lock", "name": "Lock", "party": ["lead"], "enemies": ["rat"]},
        {"id": "hearth", "name": "Hearth", "party": ["lead"], "enemies": ["rat"]},
        gate()
    ]));
    write_kind(&data, FACILITIES_KIND, json!([{"id": "vault", "name": "Vault", "keeper_character": "keeper", "levels": [{"level": 1, "name": "Bare", "scrap_cost": 0}]}]));
    write_kind(&data, SIEGES_KIND, json!([sluice()]));
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

fn without(entry: &Value, key: &str) -> Value {
    let mut out = entry.clone();
    out.as_object_mut().unwrap().shift_remove(key);
    out
}

fn one(found: Vec<String>, needle: &str, tag: &str) {
    assert_eq!(found.len(), 1, "{tag}: {found:?}");
    assert!(found[0].contains(needle), "{tag}: {found:?} (wanted '{needle}')");
}

#[test]
fn the_schema_carries_the_sieges_kind_and_the_retreat_fields() {
    let schema = source::schema();
    let field = |kind: &str, key: &str| fields(schema, "world", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("world.{kind}.{key}"));
    assert_eq!((json::s(&field(SIEGES_KIND, "rooms"), "type"), json::get(&field(SIEGES_KIND, "rooms"), "default")), ("json".to_string(), &json!([])));
    for key in ["carry_hp", "carry_downed", "carry_pouch", "no_camp"] {
        assert_eq!((json::s(&field(SIEGES_KIND, key), "type"), json::get(&field(SIEGES_KIND, key), "default")), ("bool".to_string(), &json!(true)), "{key}");
    }
    let floor = field("encounters", "hp_floor");
    assert_eq!((json::get(&floor, "default"), json::get(&floor, "min"), json::get(&floor, "max"), json::b(&floor, "omit_default", false)), (&json!(25), &json!(1), &json!(99), true));
    assert!(json::sget(&field("encounters", "objective"), "options").contains(&"retreat".to_string()));
    assert!(source::WORLD_KINDS.contains(&SIEGES_KIND) && check::world::KINDS.contains(&SIEGES_KIND), "a loaded and checked world kind");
    let data = world("schema");
    let src = source::load(&data);
    let row = src.worlds[W]["own"][SIEGES_KIND][0].as_object().cloned().expect("the loaded row");
    let def = SiegeDef::from_obj(&row);
    assert_eq!(def.rooms.iter().map(|r| r.encounter.as_str()).collect::<Vec<_>>(), ["grate", "lock", "hearth"]);
    assert_eq!((def.rooms[1].facility.as_str(), def.restocks, def.carry_hp, def.no_camp), ("vault", 1, true, true));
    // the defaults of a bare row: every carry rule on, one re-stock
    let bare = SiegeDef::from_obj(&dict(json!({"id": "bare", "rooms": [{"encounter": "grate"}]})));
    assert!(bare.carry_hp && bare.carry_downed && bare.carry_pouch && bare.no_camp);
    assert_eq!(bare.restocks, rpgkit_data::siege::RESTOCKS_DEFAULT);
    drop_scratch(&data);
}

#[test]
fn a_siege_checks_clean_and_each_bad_field_is_one_error() {
    let data = world("rows");
    let v = store::view(&data);
    let good = sluice();
    let found = errors_of(&v, SIEGES_KIND, &good);
    assert!(found.is_empty(), "{found:?}");
    let cases: Vec<(&str, Value, &str)> = vec![
        ("no rooms", with(&good, &["rooms"], json!([])), "1..4 rooms"),
        ("a room without an encounter", with(&good, &["rooms", "0", "encounter"], json!("")), "choose the encounter"),
        ("an unknown encounter", with(&good, &["rooms", "0", "encounter"], json!("ghost")), "'ghost'"),
        ("an unknown facility", with(&good, &["rooms", "1", "facility"], json!("ghost")), "'ghost'"),
        ("an unknown room key", with(&good, &["rooms", "2", "boss"], json!("x")), "unknown key 'boss'"),
        ("a room that is not an object", with(&good, &["rooms", "2"], json!("hearth")), "must be an object"),
        ("an unknown win effect", with(&good, &["win_effects", "0", "effect"], json!("boom")), "unknown effect 'boom'"),
        ("a screen effect on loss", with(&good, &["loss_effects", "0"], json!({"effect": "open_shop", "shop": "x"})), "open_shop"),
    ];
    for (tag, bad, needle) in cases {
        one(added(&v, SIEGES_KIND, &good, &bad), needle, tag);
    }
    let mut five = good.clone();
    for _ in 0..2 {
        five["rooms"].as_array_mut().unwrap().push(json!({"encounter": "grate"}));
    }
    assert!(errors_of(&v, SIEGES_KIND, &five).iter().any(|e| e.contains("1..4 rooms")));
    let blocking = check::world::blocking_errors(&v, W, SIEGES_KIND, &dict(with(&good, &["rooms"], json!({}))), "sluice");
    assert!(blocking.iter().any(|e| e.contains("must be a list")), "{blocking:?}");
    drop_scratch(&data);
}

#[test]
fn start_siege_names_a_siege_of_the_world() {
    let data = world("effect");
    let v = store::view(&data);
    let trigger = |siege: &str| json!({"id": "breach", "name": "Breach", "on": "any", "effects": [{"effect": "start_siege", "siege": siege}]});
    let found = errors_of(&v, "triggers", &trigger("sluice"));
    assert!(found.is_empty(), "{found:?}");
    one(added(&v, "triggers", &trigger("sluice"), &trigger("ghost")), "'ghost'", "an unknown siege");
    one(added(&v, "triggers", &trigger("sluice"), &trigger("")), "choose a siege", "no siege");
    drop_scratch(&data);
}

#[test]
fn a_retreat_needs_its_unit_and_a_floor_in_range() {
    let data = world("retreat");
    let v = store::view(&data);
    let good = gate();
    let found = errors_of(&v, "encounters", &good);
    assert!(found.is_empty(), "{found:?}");
    one(added(&v, "encounters", &good, &with(&good, &["objective_unit"], json!(""))), "Retreat needs a unit from enemies", "no unit");
    one(added(&v, "encounters", &good, &with(&good, &["objective_unit"], json!("keeper"))), "Retreat needs a unit from enemies", "not an enemy");
    assert!(errors_of(&v, "encounters", &without(&good, "hp_floor")).is_empty(), "the floor defaults");
    drop_scratch(&data);
}

#[test]
fn the_writer_stores_sieges_and_omits_an_unset_floor() {
    let data = world("writer");
    fs::remove_file(store::file_of(&data, Some(W), SIEGES_KIND)).unwrap();
    store::upsert(&data, Some(W), SIEGES_KIND, &dict(sluice()), None).expect("a siege");
    let file = store::file_of(&data, Some(W), SIEGES_KIND);
    assert!(file.ends_with("sieges.json"), "{file:?}");
    let stored = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, Some(W), SIEGES_KIND).unwrap(), stored, "a re-save without changes is byte-identical");
    store::upsert(&data, Some(W), "encounters", &dict(json!({"id": "plain", "name": "Plain", "party": ["lead"], "enemies": ["rat"]})), None).expect("a rout");
    store::upsert(&data, Some(W), "encounters", &dict(with(&gate(), &["hp_floor"], json!(40))), None).expect("a retreat at 40%");
    let encounters = fs::read_to_string(store::file_of(&data, Some(W), "encounters")).unwrap();
    let rows: Value = serde_json::from_str(&encounters).unwrap();
    let row = |id: &str| rows["entries"].as_array().unwrap().iter().find(|e| e["id"] == id).cloned().unwrap();
    assert!(row("plain").get("hp_floor").is_none(), "an unset floor is not stored");
    assert_eq!(row("gate")["hp_floor"], json!(40), "a set floor is kept");
    assert_eq!(store::resave_text(&data, Some(W), "encounters").unwrap(), encounters);
    let problems: Vec<String> = store::check(&data).into_iter().filter(|p| p.starts_with(&format!("{W}/sieges")) || p.starts_with(&format!("{W}/encounters"))).collect();
    assert!(problems.is_empty(), "{problems:?}");
    drop_scratch(&data);
}
