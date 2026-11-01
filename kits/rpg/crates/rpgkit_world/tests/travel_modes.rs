//! CAMPAIGN1-TM on scratch worlds: travel modes (manifest `travel_modes`, CMQ2). The default mode is the party's
//! until it picks another; a mode's `days` are added to every leg (path search, trip cost, the clock, upkeep and the
//! days tick agree), its `road_pct` scales the road-fight chance, its `sure_when` makes the road fight sure and its
//! `road_encounters` join the pool; leg inputs may count only some modes. The pick is saved only when it is not the
//! default (older saves load with the default; CAMPAIGN1-TM2: a saved mode the world no longer has loads in the default
//! with a load note). A mode's leg days are its `days` floored at 0 (`TravelMode::leg_days`). A world without modes, and a world without `campaign`, travel as
//! before. Every expectation follows from the rule and the fixture's own numbers.

use aegis_core::json::{self, Dict};
use rpgkit_world::campaign::{self, purse};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{explore, save, travel, Data, GameState};
use serde_json::{json, Value};

/// The stealth mode: one day more a leg, half the road-fight chance.
const STEALTH_DAYS: i64 = 1;
const STEALTH_PCT: i64 = 50;
/// The forced march: Alert per leg; a sure road fight from this Alert on.
const FORCED_ALERT: i64 = 5;
const SURE_AT: i64 = 50;
/// The direct hub - b road (the other way is hub - a - b, a day each leg).
const DIRECT: i64 = 3;
/// The wild road's road-fight chance.
const WILD_PCT: i64 = 40;
const BREACH: i64 = 100;
const SUPPLIES: i64 = 20;

fn with_ids(w: &str, rows: Value) -> Value {
    let rows = json::arr(&rows)
        .iter()
        .map(|r| {
            let mut r = json::dict(r).clone();
            let id = json::s(&r, "id");
            r.insert("_world".into(), w.into());
            r.insert("_qid".into(), format!("{w}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

fn modes() -> Value {
    json!({"default": "stealth", "modes": [
        {"id": "forced", "name": "Forced march", "sure_when": [{"key": "meter.alert", "op": ">=", "value": SURE_AT}], "road_encounters": [["enc_roam", 1]]},
        {"id": "stealth", "name": "Stealth", "tip": "Under the soot.", "days": STEALTH_DAYS, "road_pct": STEALTH_PCT}
    ]})
}

/// World `w`: hub - a - b (1 day each), hub - b direct (DIRECT days), hub - wild (1 day, WILD_PCT road fights from
/// `enc_road`); the gauge `alert` (+FORCED_ALERT per forced leg, the run ends at BREACH) and `legs` (+1 per leg, any
/// mode). `campaign` off = a classic world; `travel_modes` = the manifest block (none when null).
fn data(campaign: bool, travel_modes: Value) -> Data {
    let w = "w";
    let mut manifest = json!({"id": w, "start_location": "hub", "start_party": ["hero"]});
    if campaign {
        manifest["campaign"] = json!({"start_act": 1});
    }
    if !travel_modes.is_null() {
        manifest["travel_modes"] = travel_modes;
    }
    let band = |id: &str| json!([{"id": id, "name": id, "from": 0, "tone": "calm"}]);
    let meters = json!([
        {"id": "alert", "name": "Alert", "bands": band("calm"), "inputs": [{"on": "leg", "modes": ["forced"], "delta": FORCED_ALERT}],
         "rules": [{"id": "breach", "at": BREACH, "effects": [{"effect": "end", "kind": "fail", "card": "card_breach", "text": "Found."}]}]},
        {"id": "legs", "name": "Legs", "max": 999, "bands": band("low"), "inputs": [{"on": "leg", "delta": 1}]}
    ]);
    let worlds = json!({
        w: {"manifest": manifest, "own": {
            "locations": with_ids(w, json!([{"id": "hub", "name": "Hub"}, {"id": "a", "name": "A"}, {"id": "b", "name": "B"}, {"id": "wild", "name": "Wild"}])),
            "routes": with_ids(w, json!([
                {"id": "r_a", "from": "hub", "to": "a", "time": 1, "encounter_pct": 0}, {"id": "r_b", "from": "a", "to": "b", "time": 1, "encounter_pct": 0},
                {"id": "r_direct", "from": "hub", "to": "b", "time": DIRECT, "encounter_pct": 0},
                {"id": "r_wild", "from": "hub", "to": "wild", "time": 1, "encounter_pct": WILD_PCT, "encounters": [["enc_road", 1]]}])),
            "encounters": with_ids(w, json!([{"id": "enc_road", "name": "Road", "enemies": []}, {"id": "enc_roam", "name": "Roamers", "enemies": []}])),
            "meters": with_ids(w, meters)
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A new game at the hub with every node visited (every trip is known).
fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    gs.party.supplies = SUPPLIES;
    for n in ["hub", "a", "b", "wild"] {
        gs.set_flag(&explore::visited_flag(&format!("w:{n}")), json!(true));
    }
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

fn value(gs: &GameState, id: &str) -> i64 {
    gs.campaign.as_ref().unwrap().value(id, gs.time)
}

fn set_alert(gs: &mut GameState, v: i64) {
    let c = gs.campaign.as_mut().unwrap();
    c.meters.get_mut("alert").unwrap().value = v;
    c.sync(gs.time);
}

fn pick(d: &Data, gs: &mut GameState, id: &str) {
    let res = purse::set_travel_mode(d, gs, id);
    assert!(json::b(&res, "ok", false), "{res:?}");
}

fn mode_id(d: &Data, gs: &GameState) -> Option<String> {
    purse::travel_mode(d, gs).map(|m| m.id)
}

#[test]
fn the_default_mode_adds_its_days_to_every_leg_and_the_path_search_counts_them() {
    let d = data(true, modes());
    let mut gs = state(&d);
    assert_eq!(mode_id(&d, &gs).as_deref(), Some("stealth"), "the manifest's default, not the first row");
    assert_eq!(travel::mode_days(&d, &gs), STEALTH_DAYS);
    // hub - a - b is 2 legs of 1 day; the direct road DIRECT days: with STEALTH_DAYS more a leg the two tie
    // (2 + 2 = 3 + 1), and the tie goes to the fewer legs
    let two_legs = 2 * (1 + STEALTH_DAYS);
    let direct = DIRECT + STEALTH_DAYS;
    assert_eq!(two_legs, direct, "the fixture's tie");
    assert_eq!(travel::path(&d, &gs, "w:hub", "w:b"), ["w:r_direct"]);
    let trip = travel::trip(&d, &gs, "w:hub", "w:b");
    assert_eq!((json::i(&trip, "time", 0), json::i(&trip, "supplies", 0)), (direct, direct), "P2: a mode's days cost upkeep as the route's own");
    let all = travel::all_legs(&d, "w");
    let nodes = travel::nodes(&d, "w");
    assert_eq!(travel::best_path_with(&all, &nodes, "w:hub", "w:b", STEALTH_DAYS).map(|b| b.0), Some(direct), "the sim's path search agrees");
    assert_eq!(travel::best_path(&all, &nodes, "w:hub", "w:b").map(|b| b.0), Some(2), "without a mode: the two 1-day legs");
    let res = travel::travel(&d, &mut gs, "b");
    assert_eq!(json::s(&res, "status"), "arrived", "{res:?}");
    assert_eq!((json::i(&res, "time", 0), gs.time, json::i(&res, "supplies_spent", 0)), (direct, direct, direct));
    assert_eq!(gs.party.supplies, SUPPLIES - direct);
    assert_eq!((value(&gs, "legs"), value(&gs, "alert")), (1, 0), "one leg; the forced-only input did not count");
    let view = travel::view(&d, &gs);
    assert_eq!(json::s(&view, "mode"), "stealth");
    let r = json::aget(&view, "routes").iter().map(json::dict).find(|r| json::s(r, "qid") == "w:r_direct").cloned().unwrap();
    assert_eq!((json::i(&r, "time", 0), json::i(&r, "days", 0)), (DIRECT, direct));
}

#[test]
fn a_forced_march_adds_no_day_and_its_legs_move_the_inputs_that_name_it() {
    let d = data(true, modes());
    let mut gs = state(&d);
    pick(&d, &mut gs, "forced");
    assert_eq!(travel::mode_days(&d, &gs), 0);
    assert_eq!(travel::path(&d, &gs, "w:hub", "w:b"), ["w:r_a", "w:r_b"], "the fewest days again");
    let res = travel::travel(&d, &mut gs, "b");
    assert_eq!(json::s(&res, "status"), "arrived", "{res:?}");
    assert_eq!(gs.time, 2);
    assert_eq!((value(&gs, "legs"), value(&gs, "alert")), (2, 2 * FORCED_ALERT));
}

#[test]
fn the_mode_scales_the_road_chance_and_a_forced_march_meets_a_sure_fight() {
    let d = data(true, modes());
    let mut gs = state(&d);
    let wild = travel::route_in(&d, "w", "w:r_wild");
    assert_eq!(purse::road_chance(&d, &gs, &wild, WILD_PCT, false), purse::scaled(WILD_PCT, STEALTH_PCT), "stealth");
    assert!(purse::road_encounters(&d, &gs, &wild).is_empty());
    pick(&d, &mut gs, "forced");
    assert_eq!(purse::road_chance(&d, &gs, &wild, WILD_PCT, false), WILD_PCT, "forced below SURE_AT: unchanged");
    let pool: Vec<String> = purse::road_encounters(&d, &gs, &wild).iter().map(|p| json::gstr(&json::arr(p)[0])).collect();
    assert_eq!(pool, ["w:enc_roam"], "the mode's roamers join the pool");
    set_alert(&mut gs, SURE_AT - FORCED_ALERT);
    // the leg's own input lifts Alert to SURE_AT before the road roll: the fight is sure
    let res = travel::travel(&d, &mut gs, "wild");
    assert_eq!(json::s(&res, "status"), "fight", "{res:?}");
    assert_eq!(value(&gs, "alert"), SURE_AT);
    assert_eq!(purse::road_chance(&d, &gs, &wild, 0, false), 100, "sure even on a road without a chance of its own");
}

#[test]
fn the_pick_is_saved_only_when_it_is_not_the_default() {
    let d = data(true, modes());
    let mut gs = state(&d);
    assert!(gs.to_value()["campaign"].get("mode").is_none(), "the default writes nothing");
    pick(&d, &mut gs, "forced");
    assert_eq!(gs.to_value()["campaign"]["mode"], json!("forced"));
    let dir = std::env::temp_dir().join(format!("rpgkit_travel_modes_{}", std::process::id()));
    assert!(save::write(&d, &dir, &gs, 1, 1).is_empty());
    let back = save::read(&d, &dir, 1).expect("reads back");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert_eq!(back.state, gs);
    assert_eq!(mode_id(&d, &back.state).as_deref(), Some("forced"));
    pick(&d, &mut gs, "stealth");
    assert!(gs.to_value()["campaign"].get("mode").is_none(), "back to the default: the key goes");
    // a save without `mode` loads in the default mode
    let mut forced = gs.clone();
    pick(&d, &mut forced, "forced");
    let mut old = forced.to_value();
    old["campaign"].as_object_mut().unwrap().shift_remove("mode");
    let text = json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": old}).to_string();
    let loaded = save::decode(&d, &text).expect("loads");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(mode_id(&d, &loaded.state).as_deref(), Some("stealth"));
    // CAMPAIGN1-TM2 (decision "Saves and data edits"): a mode the world no longer has loads in the default mode with
    // a load note, not dropped silently and not refused
    let mut v = forced.to_value();
    v["campaign"]["mode"] = json!("gallop");
    let text = json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": v}).to_string();
    let loaded = save::decode(&d, &text).expect("loads in the default mode");
    assert_eq!(loaded.warnings, [campaign::mode_dropped_text("gallop")]);
    assert!(loaded.warnings[0].contains(&purse::unknown_mode_text("gallop")), "the note names the dropped mode");
    assert_eq!(mode_id(&d, &loaded.state).as_deref(), Some("stealth"));
    assert!(loaded.state.campaign.as_ref().unwrap().load_notes.is_empty(), "decode takes the notes");
    assert!(loaded.state.to_value()["campaign"].get("mode").is_none(), "saved again: the default writes no key");
}

#[test]
fn a_modes_leg_days_floor_its_days_at_zero_and_every_reader_uses_them() {
    let row = |days: i64| rpgkit_data::TravelMode::from_obj(json::dict(&json!({"id": "m", "days": days})));
    assert_eq!((row(-2).leg_days(), row(0).leg_days(), row(STEALTH_DAYS).leg_days()), (0, 0, STEALTH_DAYS));
    assert_eq!(rpgkit_data::TravelMode::from_obj(json::dict(&json!({"id": "m"}))).leg_days(), 0, "no days: 0");
    // a mode below zero (the content check refuses it; the runtime floors it): legs take the route's own days
    let d = data(true, json!({"modes": [{"id": "slack", "days": -2}]}));
    let mut gs = state(&d);
    assert_eq!(mode_id(&d, &gs).as_deref(), Some("slack"));
    assert_eq!(travel::mode_days(&d, &gs), 0);
    assert_eq!(json::i(&travel::trip(&d, &gs, "w:hub", "w:b"), "time", 0), 2, "hub - a - b, a day each");
    let view = travel::view(&d, &gs);
    let r = json::aget(&view, "routes").iter().map(json::dict).find(|r| json::s(r, "qid") == "w:r_direct").cloned().unwrap();
    assert_eq!(json::i(&r, "days", 0), DIRECT);
    let res = travel::travel(&d, &mut gs, "b");
    assert_eq!((json::s(&res, "status").as_str(), gs.time), ("arrived", 2), "{res:?}");
}

#[test]
fn a_pick_is_refused_without_modes_for_an_unknown_mode_and_once_the_run_is_over() {
    let d = data(true, modes());
    let mut gs = state(&d);
    let refused = |res: Dict| json::sget(&res, "errors");
    assert_eq!(refused(purse::set_travel_mode(&d, &mut gs, "gallop")), [purse::unknown_mode_text("gallop")]);
    let plain = data(true, Value::Null);
    let mut gs2 = state(&plain);
    assert_eq!(refused(purse::set_travel_mode(&plain, &mut gs2, "stealth")), [purse::REFUSE_NO_MODES]);
    let classic = data(false, modes());
    let mut gs3 = state(&classic);
    assert_eq!(refused(purse::set_travel_mode(&classic, &mut gs3, "stealth")), [purse::REFUSE_NO_MODES]);
    set_alert(&mut gs, BREACH - 1);
    let mut events = Vec::new();
    campaign::apply_effect(&d, &mut gs, json::dict(&json!({"effect": "meter_add", "meter": "alert", "delta": 1})), &mut events);
    campaign::tick(&d, &mut gs, &campaign::Cause::Effects);
    assert!(gs.campaign.as_ref().unwrap().is_over());
    assert_eq!(refused(purse::set_travel_mode(&d, &mut gs, "forced")), [campaign::RUN_OVER_TEXT]);
}

#[test]
fn worlds_without_modes_or_without_a_campaign_travel_as_before() {
    for (tag, d) in [("campaign, no modes", data(true, Value::Null)), ("classic world with a modes block", data(false, modes()))] {
        let mut gs = state(&d);
        assert!(purse::travel_mode(&d, &gs).is_none(), "{tag}");
        assert_eq!(travel::mode_days(&d, &gs), 0, "{tag}");
        assert_eq!(travel::path(&d, &gs, "w:hub", "w:b"), ["w:r_a", "w:r_b"], "{tag}");
        let view = travel::view(&d, &gs);
        assert!(!view.contains_key("mode") && json::aget(&view, "routes").iter().all(|r| !json::dict(r).contains_key("days")), "{tag}: no new view keys");
        let wild = travel::route_in(&d, "w", "w:r_wild");
        if gs.campaign.is_some() {
            assert_eq!(purse::road_chance(&d, &gs, &wild, WILD_PCT, false), WILD_PCT, "{tag}");
        }
        let res = travel::travel(&d, &mut gs, "b");
        assert_eq!((json::s(&res, "status").as_str(), gs.time), ("arrived", 2), "{tag}");
    }
}

#[test]
fn the_trip_preview_runs_the_legs_in_the_mode_now() {
    let d = data(true, modes());
    let mut gs = state(&d);
    set_alert(&mut gs, BREACH - FORCED_ALERT);
    assert!(campaign::preview_trip(&d, &gs, "b").is_empty(), "stealth legs move no Alert");
    pick(&d, &mut gs, "forced");
    let ends = campaign::preview_trip(&d, &gs, "b");
    assert_eq!(ends.len(), 1, "{ends:?}");
    assert_eq!(json::s(json::dict(&ends[0]), "card"), "card_breach", "the first forced leg reaches Breach");
    let trip = travel::trip(&d, &gs, "w:hub", "w:b");
    assert!(campaign::preview_days(&d, &gs, json::i(&trip, "time", 0)).is_empty(), "the days alone would not show it");
    assert_eq!(value(&gs, "alert"), BREACH - FORCED_ALERT, "the preview leaves the state alone");
    assert!(campaign::preview_trip(&d, &gs, "nowhere").is_empty(), "no known way: nothing to preview");
}
