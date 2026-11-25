//! PR7-47 (PR7-37 PT-31) on a scratch world (no shipped data): the New road notice. The first look (no saved list)
//! only learns the roads open then; a road a flag opens later is told once, as its two places, and saved as told in
//! the optional `explore.roads_seen` key, so a road opened before a save is still told after the load; a save without
//! the key loads as before (a new optional key, no version bump), a damaged key is refused by `explore_errors`.

use aegis_core::json;
use aegis_core::scratch::Scratch;
use rpgkit_world::state::{explore_errors, PartyState, UnitState, ROADS_SEEN_KEY};
use rpgkit_world::{save, travel, Data, GameState};
use serde_json::{json, Value};

const GATE: &str = "w.gate_open";

fn row(id: &str, extra: Value) -> Value {
    let mut r = json::dict(&extra).clone();
    r.insert("id".into(), id.into());
    r.insert("_world".into(), "w".into());
    r.insert("_qid".into(), format!("w:{id}").into());
    Value::Object(r)
}

/// World `w`: places hub, a, b; road `open` hub-a always open, road `gated` hub-b open once `GATE` is set.
fn data() -> Data {
    let worlds = json!({
        "w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": [row("hub", json!({"name": "Hub"})), row("a", json!({"name": "Place A"})), row("b", json!({"name": "Place B"}))],
            "routes": [
                row("open", json!({"name": "Open Road", "from": "hub", "to": "a", "time": 1, "encounter_pct": 0})),
                row("gated", json!({"name": "Gated Road", "from": "hub", "to": "b", "time": 1, "encounter_pct": 0, "requires_flags": [{"key": GATE, "value": true}]}))
            ]
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, "w", party)
}

/// The (from, to) of the first open leg over route `qid` (`travel::legs`' own order).
fn leg_of(d: &Data, gs: &GameState, qid: &str) -> (String, String) {
    travel::legs(d, gs, "w").into_iter().find(|(r, _, _)| json::s(r, "_qid") == qid).map(|(_, a, b)| (a, b)).expect("an open leg")
}

#[test]
fn the_first_look_learns_and_a_later_road_is_told_once() {
    let d = data();
    let mut gs = state(&d);
    assert_eq!(travel::roads_seen(&gs), None, "a new game has no saved roads");
    assert!(travel::note_new_roads(&d, &mut gs).is_empty(), "the first look only learns");
    assert_eq!(travel::roads_seen(&gs), Some(travel::open_roads(&d, &gs)));
    assert_eq!(travel::open_roads(&d, &gs), vec!["w:open".to_string()], "only the open road");
    let before = gs.to_value();
    assert!(travel::note_new_roads(&d, &mut gs).is_empty(), "no new road");
    assert_eq!(gs.to_value(), before, "no change writes nothing");
    gs.set_flag(GATE, json!(true));
    let told = travel::note_new_roads(&d, &mut gs);
    assert_eq!(told, vec![leg_of(&d, &gs, "w:gated")], "the new road, as its two places");
    assert!(travel::note_new_roads(&d, &mut gs).is_empty(), "told once");
}

#[test]
fn an_untold_road_survives_a_save_and_a_load() {
    let d = data();
    let mut gs = state(&d);
    travel::note_new_roads(&d, &mut gs);
    gs.set_flag(GATE, json!(true));
    let dir = Scratch::new("rpgkit_road_notice_trip");
    assert!(save::write(&d, dir.path(), &gs, 1, 5).is_empty());
    let mut back = save::read(&d, dir.path(), 1).expect("reads back").state;
    assert_eq!(back, gs, "the saved roads come back");
    assert_eq!(travel::note_new_roads(&d, &mut back), vec![leg_of(&d, &back, "w:gated")], "told after the load");
    assert!(save::write(&d, dir.path(), &back, 1, 6).is_empty());
    let mut again = save::read(&d, dir.path(), 1).expect("reads back").state;
    assert!(travel::note_new_roads(&d, &mut again).is_empty(), "a told road stays told after the next save");
}

#[test]
fn a_save_without_the_key_loads_and_a_damaged_one_is_refused() {
    let d = data();
    let gs = state(&d);
    let mut raw = gs.to_value();
    assert!(raw["explore"].get(ROADS_SEEN_KEY).is_none(), "no look writes no key (older saves read the same)");
    let mut back = GameState::from_dict(&d, raw.as_object().unwrap());
    assert!(back.load_problems.is_empty() && explore_errors(&back.explore).is_empty());
    assert!(travel::note_new_roads(&d, &mut back).is_empty(), "an older save's first look only learns");
    assert!(explore_errors(&back.explore).is_empty(), "the learnt key is valid");
    for bad in [json!("w:open"), json!([3]), json!(["Not A Qid"])] {
        raw["explore"][ROADS_SEEN_KEY] = bad.clone();
        let e = raw["explore"].as_object().unwrap().clone();
        assert!(!explore_errors(&e).is_empty(), "explore.roads_seen {bad} is refused");
    }
}
