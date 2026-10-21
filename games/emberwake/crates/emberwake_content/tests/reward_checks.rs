//! REWARD1 validation (spec section 10): every message of a unit's loot table, a world encounter's key loot and
//! objective reward, and the retired purse loot, on entries of the shipped data with one field set wrong.

use aegis_core::json::{self, Dict};
use emberwake_content::check::{self, reward, View};
use emberwake_content::store;
use serde_json::{json, Value};
use std::path::PathBuf;

fn view() -> View {
    store::view(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"))
}

fn with(base: &Dict, key: &str, value: Value) -> Dict {
    let mut e = base.clone();
    e.insert(key.into(), value);
    e
}

fn unit_errors(v: &View, loot: Value) -> Vec<String> {
    let e = with(v.get("units", "goblin_brawler"), "loot", loot);
    check::validate_entry(v, "units", &e, "goblin_brawler")
}

#[test]
fn a_loot_table_is_checked_row_by_row() {
    let v = view();
    assert!(unit_errors(&v, json!([{"item": "wpn_knuckles", "chance": 3}, {"item": "arm_leather", "chance": 3, "count": 2}])).is_empty());
    assert_eq!(unit_errors(&v, json!({"item": "wpn_knuckles"})), ["Loot must be a list of rows."]);
    let bad = unit_errors(&v, json!([{"item": "nope", "chance": 0, "count": 10, "odds": 1}, {"item": "wpn_knuckles", "chance": 60}, {"item": "wpn_knuckles", "chance": 50}]));
    for m in [
        "Loot row 1: key 'odds' is not allowed.",
        "Loot row 1: item 'nope' does not resolve.",
        "Loot row 1: chance must be a whole number 1..100.",
        "Loot row 1: count must be a whole number 1..9.",
        "Loot item 'wpn_knuckles' is listed twice.",
        "Loot chances add up to 110 (at most 100).",
    ] {
        assert!(bad.contains(&m.to_string()), "{m} in {bad:?}");
    }
}

#[test]
fn loot_warnings_name_party_units_special_items_and_rich_tables() {
    let v = view();
    let special = v.entries("items").iter().find(|i| json::b(i, "special", false)).map(|i| json::s(i, "id")).expect("a special item ships");
    let party = with(v.get("units", "warrior"), "loot", json!([{"item": "wpn_knuckles", "chance": 3}]));
    assert!(reward::loot_warnings(&v, &party).contains(&"Party-side unit 'warrior' has a loot table; party units never drop.".to_string()));
    let rich = with(v.get("units", "goblin_brawler"), "loot", json!([{"item": special, "chance": 100}]));
    let w = reward::loot_warnings(&v, &rich);
    assert!(w.contains(&format!("Loot drops special item '{special}'.")), "{w:?}");
    let value = json::i(v.get("items", &special), "value", 0);
    if value > reward::loot_ev_soft("minion") {
        assert!(w.contains(&format!("Loot is worth {value} gold per defeat on average (soft cap 5 for minion).")), "{w:?}");
    }
    let plain = with(v.get("units", "goblin_brawler"), "loot", json!([{"item": "wpn_knuckles", "chance": 3}]));
    assert!(reward::loot_warnings(&v, &plain).is_empty(), "a small table of plain items is quiet");
}

fn encounter_errors(v: &View, key: &str, value: Value) -> Vec<String> {
    let e = with(v.worlds.resolve("ashlands", "encounters", "ash_patrol"), key, value);
    check::world::validate_entry(v, "ashlands", "encounters", &e, "ash_patrol")
}

#[test]
fn key_loot_needs_an_item_reward_of_the_world_and_a_roster_kill() {
    let v = view();
    assert!(encounter_errors(&v, "key_loot", json!([{"loot": "keep_ring", "kill": "ash_warchief"}])).is_empty());
    assert_eq!(encounter_errors(&v, "key_loot", json!({"loot": "keep_ring"})), ["Key loot must be a list of rows."]);
    let bad = encounter_errors(&v, "key_loot", json!([{"loot": "nope", "kill": "nobody", "x": 1}, {"loot": "wardens_drill", "kill": "ash_warchief"}, {"loot": "keep_ring", "kill": "ash_warchief"}, {"loot": "keep_ring", "kill": "ash_berserker"}]));
    for m in [
        "Key loot row 1: key 'x' is not allowed.",
        "Key loot 'nope' does not resolve.",
        "Key loot kill 'nobody' is not an enemy of this encounter.",
        "Key loot 'wardens_drill' must be an item reward.",
        "Key loot 'keep_ring' is listed twice.",
    ] {
        assert!(bad.contains(&m.to_string()), "{m} in {bad:?}");
    }
}

#[test]
fn key_loot_warns_of_a_repeated_kill_and_a_quest_pool() {
    let v = view();
    let raiders = v.worlds.resolve("ashlands", "encounters", "ash_raiders");
    let pooled = v.worlds.merged_entries("ashlands", "quests").iter().flat_map(|q| json::sget(q, "loot_pool")).next().expect("a quest pool ships");
    let mut e = with(raiders, "enemies", json!(["ash_brawler", "ash_brawler", "ash_shaman"]));
    e.insert("key_loot".into(), json!([{"loot": pooled, "kill": "ash_brawler"}]));
    let w = reward::key_loot_warnings(&v, "ashlands", &e);
    assert!(w.contains(&"Key loot kill 'ash_brawler' is listed 2 times: every one must be slain.".to_string()), "{w:?}");
    assert!(w.contains(&format!("Key loot '{pooled}' is also in a quest pool: whichever claims it first wins.")), "{w:?}");
}

#[test]
fn an_objective_reward_is_gold_and_an_item_reward() {
    let v = view();
    assert!(encounter_errors(&v, "objective_reward", json!({"gold": 10, "loot": "keep_ring", "once": true})).is_empty());
    let bad = encounter_errors(&v, "objective_reward", json!({"gold": 600, "loot": "wardens_drill", "x": 1}));
    for m in ["Objective reward key 'x' is not allowed.", "Objective reward gold must be a whole number 0..500.", "Objective loot 'wardens_drill' must be an item reward."] {
        assert!(bad.contains(&m.to_string()), "{m} in {bad:?}");
    }
    assert!(encounter_errors(&v, "objective_reward", json!({"loot": "nope"})).contains(&"Objective loot 'nope' does not resolve.".to_string()));
    let rich = json::obj([("objective_reward", json!({"gold": 150}))]);
    assert_eq!(reward::objective_reward_warnings(&rich), ["Objective reward gold above 100: check the budget."]);
    let empty = json::obj([("objective_reward", json!({"once": true}))]);
    assert_eq!(reward::objective_reward_warnings(&empty), ["Empty objective reward."]);
}

#[test]
fn purse_loot_is_retired_on_catalog_and_world_encounters() {
    let v = view();
    let moved = "Purse key 'loot' moved to key_loot (REWARD1).".to_string();
    assert!(encounter_errors(&v, "purse", json!({"gold_min": 5, "loot": "keep_ring"})).contains(&moved));
    let cat = v.entries("encounters").first().expect("a catalog encounter").clone();
    let id = json::s(&cat, "id");
    let e = with(&cat, "purse", json!({"gold_min": 5, "loot": "ashlands:keep_ring"}));
    assert!(check::validate_entry(&v, "encounters", &e, &id).contains(&moved));
    assert!(encounter_errors(&v, "purse", json!({"gold_min": 5, "gold_max": 9, "once": false})).is_empty(), "the gold keys stay");
}
