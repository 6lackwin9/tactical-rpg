//! BUNKS1 BK1-BK3, BK9-BK12 (spec BUNKS1 sections 1 and 3): the hub's bunks and idle spots.
//! Bunk props (`bunk_<n>`, number order) bind to the members housed in bunks or crypts in roster order and show
//! empty, benched, active or crypt; the condition word reads wounded, else ready (active) or resting (benched); active
//! members idle at the table spots (`idle_*` anchors, then around the `table*` props), benched ones in front of their
//! bunks; passing the lead trades the two places and moves no one else. Expectations follow from the fixture map.

use aegis_core::json::{self, Dict};
use rpgkit_world::bunks::{self, BunkState};
use rpgkit_world::{new_game, roster, Data, GameState};
use serde_json::{json, Value};

fn with_ids(w: &str, rows: Value) -> Value {
    Value::Array(
        json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = json::s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect(),
    )
}

/// World `w` (with or without a roster block) and five party characters `m1`..`m5`.
fn data(roster: bool) -> Data {
    let ch = |id: &str| json!({"id": id, "name": id, "role": "party", "base_unit": "hero"});
    let mut manifest = json!({"id": "w", "start_location": "hub", "start_party": ["hero"], "party_max": 3});
    if roster {
        manifest["roster"] = json!({"bunks": 6});
    }
    let worlds = json!({"w": {"manifest": manifest, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub", "rest": true}])),
        "characters": with_ids("w", json!([ch("m1"), ch("m2"), ch("m3"), ch("m4"), ch("m5")]))
    }}});
    let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]});
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A 12 x 8 room: bunks along the top wall (row 1) listed out of order, a 2 x 1 table at (5, 5), one `idle_` anchor.
fn map() -> Dict {
    let rows = ["############", "#..........#", "#..........#", "#..........#", "#..........#", "#....##....#", "#..........#", "############"];
    json::dict(&json!({"id": "room", "rows": rows,
        "anchors": [{"id": "spawn", "x": 1, "y": 6}, {"id": "idle_a", "x": 9, "y": 6}],
        "props": [{"id": "bunk_2", "art": "cot", "x": 4, "y": 0, "w": 2, "h": 1}, {"id": "bunk_1", "art": "cot", "x": 1, "y": 0, "w": 2, "h": 1},
                  {"id": "bunk_3", "art": "cot", "x": 7, "y": 0, "w": 2, "h": 1}, {"id": "bunk_4", "art": "cot", "x": 9, "y": 0, "w": 2, "h": 1},
                  {"id": "table", "art": "table_stools", "x": 5, "y": 5, "w": 2, "h": 1}]})).clone()
}

fn loc() -> Dict {
    Dict::new()
}

/// Members m1..m4 (m1..m3 active), m5 left out unless asked.
fn game(d: &Data, n: usize) -> GameState {
    let mut gs = new_game::create_for_world(d, "w", 1).expect("a new game");
    gs.party.members = ["m1", "m2", "m3", "m4", "m5"][..n].iter().map(|c| new_game::hero_from_character(d, "w", c).expect("a character")).collect();
    gs.party.active_ids = vec!["m1".into(), "m2".into(), "m3".into()];
    gs
}

#[test]
fn bunks_bind_in_number_order_and_show_the_four_states() {
    let d = data(true);
    let t = map();
    let mut gs = game(&d, 3);
    assert!(bunks::is_hub(&gs, &t));
    assert!(!bunks::is_hub(&game(&data(false), 3), &t), "no roster block: no hub");
    gs.party.active_ids = vec!["m1".into(), "m3".into()];
    gs.party.members[2].housing = "crypt".into();
    let b = bunks::bunks(&gs, &t);
    assert_eq!(b.iter().map(|b| b.prop.as_str()).collect::<Vec<_>>(), vec!["bunk_1", "bunk_2", "bunk_3", "bunk_4"]);
    assert_eq!(b.iter().map(|b| (b.member.as_str(), b.state)).collect::<Vec<_>>(), vec![("m1", BunkState::Active), ("m2", BunkState::Benched), ("m3", BunkState::Crypt), ("", BunkState::Empty)]);
    assert_eq!(bunks::BUNK_STATES.iter().map(|s| s.name()).collect::<Vec<_>>(), vec!["empty", "benched", "active", "crypt"]);
    // a crypt_only member takes no bunk: the next member moves up
    gs.party.members[1].housing = roster::CRYPT_ONLY.into();
    let b = bunks::bunks(&gs, &t);
    assert_eq!((b[1].member.as_str(), b[2].member.as_str()), ("m3", ""));
    assert_eq!(b[0].spot(), (1, 1), "in front of the bunk");
}

#[test]
fn the_condition_word_reads_wounded_then_ready_or_resting() {
    let d = data(true);
    let mut gs = game(&d, 4);
    let word = |gs: &GameState, i: usize| bunks::condition(&d, gs, &gs.party.members[i]);
    assert_eq!(word(&gs, 0), bunks::READY_WORD);
    assert_eq!(word(&gs, 3), bunks::RESTING);
    gs.party.members[3].add_condition(roster::WOUNDED);
    assert_eq!(word(&gs, 3), bunks::WOUNDED_WORD);
    assert!(!bunks::strain_high(&d, &gs, &gs.party.members[0]), "a world without the channel rule has no Strain");
}

#[test]
fn active_members_idle_at_the_tables_and_benched_ones_sleep_in_their_bunks() {
    let d = data(true);
    let t = map();
    let mut gs = game(&d, 5);
    // the table spots: the idle anchor, then west, east, south and north of the table
    assert_eq!(bunks::table_spots(&loc(), &t), vec![(9, 6), (4, 5), (7, 5), (5, 6), (6, 6), (5, 4), (6, 4)]);
    let spots = bunks::place(&gs, &loc(), &t, "m1");
    // HUB-BUNKS2 (4): the benched m4 sleeps in bunk_4 (idles nowhere); m5 has no bunk prop: not shown
    assert_eq!(spots, vec![("m2".to_string(), (9, 6)), ("m3".to_string(), (4, 5))]);
    let asleep = bunks::sleepers(&gs, &t);
    assert_eq!(asleep.iter().map(|b| (b.prop.as_str(), b.member.as_str())).collect::<Vec<_>>(), vec![("bunk_4", "m4")]);
    assert!(asleep.iter().all(|b| b.state == BunkState::Benched));
    // a crypt member is not shown
    gs.party.members[3].housing = "crypt".into();
    assert!(!bunks::place(&gs, &loc(), &t, "m1").iter().any(|(id, _)| id == "m4"));
    assert!(bunks::sleepers(&gs, &t).is_empty(), "a crypt member does not sleep in a bunk");
}

#[test]
fn passing_the_lead_trades_the_two_places_and_moves_no_one_else() {
    let d = data(true);
    let t = map();
    let mut gs = game(&d, 4);
    gs.position = (1, 6);
    let mut spots = bunks::place(&gs, &loc(), &t, "m1");
    let before = spots.clone();
    let m3_spot = spots.iter().find(|(id, _)| id == "m3").unwrap().1;
    assert!(bunks::pass_lead(&mut gs, &t, &mut spots, "m1", "m3"));
    // m3 stands up onto the first open free cell beside its spot (up first); m1 takes the spot
    assert_eq!(gs.position, (m3_spot.0, m3_spot.1 - 1));
    assert_eq!(spots.iter().find(|(id, _)| id == "m1").map(|s| s.1), Some(m3_spot));
    assert!(!spots.iter().any(|(id, _)| id == "m3"));
    for (id, p) in before.iter().filter(|(id, _)| id != "m3") {
        assert_eq!(spots.iter().find(|(i, _)| i == id).map(|s| s.1), Some(*p), "{id} did not move");
    }
    assert!(!bunks::pass_lead(&mut gs, &t, &mut spots, "m3", "nobody"), "someone who idles nowhere takes no lead here");
    assert_ne!(bunks::signature(&gs, "m1"), bunks::signature(&gs, "m3"));
}
