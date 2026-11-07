//! E2.2-S1 (FB-53a, E_ECONOMY.md section 2): the party reads the game's purse caps (`Data::purse`, from `game.json`):
//! gold and supply grants stop at them, the save check bounds by them, the kit's cap sentences name them, and a save made
//! under higher caps loads clamped to the game's (never failing the check), and so does a new game's start purse; a
//! damaged value below 0 is still a problem.
//! Without the manifest keys the caps are today's (`GOLD_MAX`, `SUPPLIES_MAX`). Every expectation follows from the
//! rule and the fixture's own numbers.

use aegis_core::json;
use rpgkit_world::state::{PartyState, PurseCaps, UnitState, GOLD_MAX, SUPPLIES_MAX, SUPPLIES_START};
use rpgkit_world::{new_game, terms, Data, GameState};
use serde_json::{json, Value};
use std::fs;

const SUPPLIES: i64 = 20;
const GOLD: i64 = 500;

fn data_with(caps: PurseCaps) -> Data {
    data_starting(caps, json!({}))
}

/// World `w` (a hub, the hero) whose manifest also holds `start` (`start_gold` / `start_supplies`, or none).
fn data_starting(caps: PurseCaps, start: Value) -> Data {
    let mut manifest = json!({"id": "w", "start_location": "hub", "start_party": ["hero"]});
    for (k, v) in json::dict(&start) {
        manifest[k] = v.clone();
    }
    let worlds = json!({"w": {"manifest": manifest, "own": {"locations": [{"id": "hub", "name": "Hub", "_world": "w", "_qid": "w:hub"}]}}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let mut d = Data::from_values(&catalog, &worlds, &json!({}));
    d.purse = caps;
    d
}

fn low() -> PurseCaps {
    PurseCaps { supplies_max: SUPPLIES, purse_gold_cap: GOLD }
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, "w", party)
}

fn purse_errors(d: &Data, gs: &GameState) -> Vec<String> {
    gs.party.validate(d).into_iter().filter(|e| e.starts_with("gold") || e.starts_with("supplies")).collect()
}

fn reload(d: &Data, gs: &GameState) -> GameState {
    GameState::from_dict(d, json::dict(&gs.to_value()))
}

#[test]
fn without_manifest_keys_the_caps_are_todays() {
    let d = data_with(PurseCaps::default());
    let mut gs = state(&d);
    assert_eq!((gs.party.purse.supplies_max, gs.party.purse.purse_gold_cap), (SUPPLIES_MAX, GOLD_MAX));
    assert_eq!(gs.party.add_supplies(SUPPLIES_MAX + 5), SUPPLIES_MAX, "a grant stops at today's cap");
    assert_eq!(gs.party.add_gold(GOLD_MAX + 5), GOLD_MAX);
    assert!(purse_errors(&d, &gs).is_empty());
}

#[test]
fn grants_stop_at_the_games_caps_and_the_check_bounds_by_them() {
    let d = data_with(low());
    let mut gs = state(&d);
    assert_eq!(gs.party.purse, low(), "a new game takes the game's caps");
    let (s0, g0) = (gs.party.supplies, gs.party.gold);
    assert_eq!(gs.party.add_supplies(SUPPLIES * 2), SUPPLIES - s0, "only what fits is added");
    assert_eq!(gs.party.supplies, SUPPLIES);
    assert_eq!(gs.party.add_gold(GOLD * 2), GOLD - g0);
    assert_eq!(gs.party.gold, GOLD);
    assert!(purse_errors(&d, &gs).is_empty(), "at the caps is valid");
    gs.party.supplies = SUPPLIES + 1;
    gs.party.gold = GOLD + 1;
    assert_eq!(purse_errors(&d, &gs), [format!("gold must be 0..{GOLD}."), format!("supplies must be 0..{SUPPLIES}.")]);
}

#[test]
fn the_cap_sentences_name_the_games_caps() {
    let d = data_with(low());
    assert_eq!(terms::upkeep_carry_max(&d, "w"), format!("You can carry at most {SUPPLIES} supplies."));
    assert_eq!(terms::upkeep_capped(&d, "w", 3), format!("Supplies are capped at {SUPPLIES} (3 left behind)."));
    assert_eq!(terms::currency_capped(&d, "w", 7), format!("Gold is capped at {GOLD} (7 not added)."));
}

#[test]
fn a_save_made_under_higher_caps_loads_clamped() {
    let high = data_with(PurseCaps::default());
    let mut gs = state(&high);
    gs.party.supplies = SUPPLIES + 30;
    gs.party.gold = GOLD + 1000;
    let same = reload(&high, &gs);
    assert_eq!((same.party.supplies, same.party.gold), (SUPPLIES + 30, GOLD + 1000), "under the same caps nothing moves");
    let lowered = data_with(low());
    let loaded = reload(&lowered, &gs);
    assert_eq!(loaded.party.purse, low());
    assert_eq!((loaded.party.supplies, loaded.party.gold), (SUPPLIES, GOLD), "clamped to the game's caps");
    assert!(purse_errors(&lowered, &loaded).is_empty(), "the clamped save passes the check");
    assert_eq!(reload(&lowered, &loaded), loaded, "a clamped save reloads unchanged");
}

#[test]
fn a_damaged_purse_value_is_not_clamped_away() {
    let d = data_with(low());
    let mut gs = state(&d);
    gs.party.gold = -1;
    let loaded = reload(&d, &gs);
    assert_eq!(loaded.party.gold, -1);
    assert_eq!(purse_errors(&d, &loaded), [format!("gold must be 0..{GOLD}.")]);
}

#[test]
fn data_open_reads_the_caps_from_the_games_manifest() {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_world_purse_caps_open");
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let manifest: Value = json!({"id": "demo", "title": "Demo", "supplies_max": SUPPLIES, "purse_gold_cap": GOLD});
    fs::write(t.join("game.json"), serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    let (d, _) = Data::open(&data).expect("a data folder");
    assert_eq!(d.purse, low());
    let bare = aegis_core::scratch::Scratch::vacant("rpgkit_world_purse_caps_bare");
    fs::create_dir_all(bare.join("data/catalog")).unwrap();
    fs::create_dir_all(bare.join("data/worlds")).unwrap();
    assert_eq!(Data::open(&bare.join("data")).expect("a data folder").0.purse, PurseCaps::default(), "no manifest: today's caps");
}

#[test]
fn a_new_game_starts_inside_the_caps() {
    let tight = PurseCaps { supplies_max: SUPPLIES_START - 1, purse_gold_cap: GOLD };
    let inherited = data_starting(tight, json!({}));
    let gs = new_game::create_for_world(&inherited, "w", 7).expect("a valid new game under a cap below the default start");
    assert_eq!(gs.party.supplies, tight.supplies_max, "the default start {SUPPLIES_START} clamped to the cap");
    let authored = data_starting(tight, json!({"start_gold": GOLD + 100, "start_supplies": SUPPLIES_START + 5}));
    let gs = new_game::create_for_world(&authored, "w", 7).expect("a valid new game with start values above the caps");
    assert_eq!((gs.party.gold, gs.party.supplies), (GOLD, tight.supplies_max), "authored starts clamped to the caps");
    let roomy = data_starting(PurseCaps::default(), json!({"start_gold": GOLD + 100}));
    let gs = new_game::create_for_world(&roomy, "w", 7).expect("today's caps");
    assert_eq!((gs.party.gold, gs.party.supplies), (GOLD + 100, SUPPLIES_START), "under today's caps the starts are kept");
}
