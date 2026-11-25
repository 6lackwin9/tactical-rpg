//! FOOTLOCKER1: a bunk has a footlocker (its member's loadout over the shared stash) when its member's loadout can
//! change: an active or a benched member's bunk has one, a crypt member's (the loadout locked) and an empty bunk none.
//! Expectations follow from the fixture roster (one member in each state) and the rule.

use aegis_core::json::{self, Dict};
use rpgkit_world::bunks::{self, BunkState};
use rpgkit_world::{new_game, Data, GameState};
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

/// World `w` with a roster block and three party characters.
fn data() -> Data {
    let ch = |id: &str| json!({"id": id, "name": id, "role": "party", "base_unit": "hero"});
    let manifest = json!({"id": "w", "start_location": "hub", "start_party": ["hero"], "party_max": 3, "roster": {"bunks": 4, "bunks_max": 4}});
    let worlds = json!({"w": {"manifest": manifest, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub", "rest": true}])),
        "characters": with_ids("w", json!([ch("m1"), ch("m2"), ch("m3")]))
    }}});
    let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]});
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// Four bunks on the top wall.
fn map() -> Dict {
    let rows = ["##########", "#........#", "##########"];
    let bunk = |n: i64| json!({"id": format!("bunk_{n}"), "art": "cot", "x": 2 * n - 1, "y": 0, "w": 2, "h": 1});
    json::dict(&json!({"id": "room", "rows": rows, "anchors": [{"id": "spawn", "x": 1, "y": 1}], "props": [bunk(1), bunk(2), bunk(3), bunk(4)]})).clone()
}

/// m1 active, m2 benched, m3 in the crypts; the fourth bunk empty.
fn game(d: &Data) -> GameState {
    let mut gs = new_game::create_for_world(d, "w", 1).expect("a new game");
    gs.party.members = ["m1", "m2", "m3"].iter().map(|c| new_game::hero_from_character(d, "w", c).expect("a character")).collect();
    gs.party.active_ids = vec!["m1".into()];
    gs.party.members[2].housing = "crypt".into();
    gs
}

#[test]
fn active_and_benched_members_have_a_footlocker_crypt_members_and_empty_bunks_none() {
    let d = data();
    let gs = game(&d);
    let t = map();
    let all = bunks::bunks(&gs, &t);
    let states: Vec<BunkState> = all.iter().map(|b| b.state).collect();
    assert_eq!(states, [BunkState::Active, BunkState::Benched, BunkState::Crypt, BunkState::Empty], "the fixture: one bunk in each state");
    let with: Vec<bool> = all.iter().map(bunks::has_locker).collect();
    assert_eq!(with, [true, true, false, false]);
    let members: Vec<String> = bunks::lockers(&gs, &t).into_iter().map(|b| b.member).collect();
    assert_eq!(members, ["m1", "m2"], "the lockers, in bunk order");
}

#[test]
fn a_member_leaving_the_crypts_gets_a_footlocker_back() {
    let d = data();
    let mut gs = game(&d);
    let t = map();
    assert!(!bunks::lockers(&gs, &t).iter().any(|b| b.member == "m3"));
    gs.party.members[2].housing = "bunk".into();
    assert!(bunks::lockers(&gs, &t).iter().any(|b| b.member == "m3"), "back in a bunk: the loadout opens again");
}
