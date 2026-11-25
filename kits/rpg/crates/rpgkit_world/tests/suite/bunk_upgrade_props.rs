//! BUNKS-UPGRADE-PROPS1 (`PR7-50`): a map prop may hold a bunk's place (`until_bunk`, the reverse of the bunk's cap
//! gate): it shows only while that bunk is past the hub's bunk cap, and the bunk shows only once the cap reaches it;
//! other props, and every prop outside a hub, always show. Expectations follow from the fixture map and the cap rule
//! (`bunks` plus the `bunks_flag` value, at most `bunks_max`).

use aegis_core::json::{self, Dict};
use rpgkit_world::bunks;
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

/// World `w`: two bunks, up to four with the flag `w.beds` (the upgrade), or no roster block.
fn data(roster: bool) -> Data {
    let ch = |id: &str| json!({"id": id, "name": id, "role": "party", "base_unit": "hero"});
    let mut manifest = json!({"id": "w", "start_location": "hub", "start_party": ["hero"], "party_max": 3});
    if roster {
        manifest["roster"] = json!({"bunks": 2, "bunks_max": 4, "bunks_flag": "w.beds"});
    }
    let worlds = json!({"w": {"manifest": manifest, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub", "rest": true}])),
        "characters": with_ids("w", json!([ch("m1"), ch("m2")]))
    }}});
    let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]});
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// Four bunks on the top wall; bunks 3 and 4's places held by a timber stack and crates; a table holds nothing.
fn map() -> Dict {
    let rows = ["############", "#..........#", "#..........#", "#...##.##..#", "############"];
    json::dict(&json!({"id": "room", "rows": rows, "anchors": [{"id": "spawn", "x": 1, "y": 2}],
        "props": [{"id": "bunk_1", "art": "cot", "x": 1, "y": 0, "w": 2, "h": 1}, {"id": "bunk_2", "art": "cot", "x": 4, "y": 0, "w": 2, "h": 1},
                  {"id": "bunk_3", "art": "cot", "x": 4, "y": 3, "w": 2, "h": 1}, {"id": "bunk_4", "art": "cot", "x": 7, "y": 3, "w": 2, "h": 1},
                  {"id": "spare_timber", "art": "bed_timber_stack", "x": 4, "y": 3, "w": 2, "h": 1, "until_bunk": "bunk_3"},
                  {"id": "spare_crates", "art": "crate_stack", "x": 7, "y": 3, "w": 2, "h": 1, "until_bunk": "bunk_4"},
                  {"id": "table", "art": "table_stools", "x": 8, "y": 1, "w": 1, "h": 1}]})).clone()
}

fn game(d: &Data) -> GameState {
    let mut gs = new_game::create_for_world(d, "w", 1).expect("a new game");
    gs.party.members = ["m1", "m2"].iter().map(|c| new_game::hero_from_character(d, "w", c).expect("a character")).collect();
    gs.party.active_ids = vec!["m1".into()];
    gs
}

fn shown(gs: &GameState, t: &Dict) -> Vec<String> {
    let t2 = bunks::shown_props(gs, t);
    json::aget(t2.as_ref().unwrap_or(t), "props").iter().map(|p| json::s(json::dict(p), "id")).collect()
}

#[test]
fn before_the_upgrade_the_props_hold_the_bunks_places_and_after_it_the_bunks_show() {
    let d = data(true);
    let t = map();
    let mut gs = game(&d);
    assert!(bunks::is_hub(&gs, &t));
    let cap = roster::bunk_cap(&gs);
    assert_eq!(bunks::open_bunk_ids(&gs, &t).len() as i64, cap);
    assert_eq!(shown(&gs, &t), ["bunk_1", "bunk_2", "spare_timber", "spare_crates", "table"], "before: the props, not bunks 3 and 4");
    // one more bunk: bunk 3 comes in and its timber goes; bunk 4 stays held
    gs.set_flag("w.beds", json!(1));
    assert_eq!(roster::bunk_cap(&gs), cap + 1);
    assert_eq!(shown(&gs, &t), ["bunk_1", "bunk_2", "bunk_3", "spare_crates", "table"]);
    // the full upgrade: both bunks, no props
    gs.set_flag("w.beds", json!(2));
    assert_eq!(shown(&gs, &t), ["bunk_1", "bunk_2", "bunk_3", "bunk_4", "table"], "after: the bunks, not the props");
    assert!(bunks::shown_props(&gs, &json::dict(&json!({"props": [{"id": "bunk_1"}], "rows": ["#"]})).clone()).is_none(), "nothing hidden: None");
}

#[test]
fn a_bunk_no_prop_holds_shows_past_the_cap_and_outside_a_hub_every_prop_shows() {
    let d = data(true);
    let mut t = map();
    t.insert("props".into(), json!([{"id": "bunk_1", "art": "cot", "x": 1, "y": 0}, {"id": "bunk_2", "art": "cot", "x": 4, "y": 0}, {"id": "bunk_3", "art": "cot", "x": 7, "y": 0}]));
    let gs = game(&d);
    assert!(bunks::shown_props(&gs, &t).is_none(), "a bunk past the cap that nothing holds draws empty, as before");
    let out = game(&data(false));
    assert!(!bunks::is_hub(&out, &map()));
    assert!(bunks::shown_props(&out, &map()).is_none(), "no roster block: no hub, every prop shows");
}
