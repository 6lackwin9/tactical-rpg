//! PR5-49 (MAP-POLISH1, FB-257 (a)) on scratch worlds: a region's `map_reveal`. While the party is in a revealing region
//! (its region, or one up the `parent` chain), the world map shows every place of that region a route touches
//! (`travel::shown`, the view's nodes), the ones it cannot travel to now locked (`travel::locked`) and marked
//! `revealed` when only the region shows them; a place no route touches never shows, nor a place of another region
//! that is not known; the view's routes are those between places shown. Off (or the party outside the region), the map
//! is as before: the known places, no `revealed` key.

use aegis_core::json;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::travel::{self, MAP_REVEAL};
use rpgkit_world::{explore, Data, GameState};
use serde_json::{json, Value};

const GATE: &str = "w.gate_open";

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

/// World `w`: region `quarter` (revealing when `reveal`) holds district `ward` (never revealing itself); `outer` is
/// another region. Places: hub (ward), a, b, c (quarter), lone (quarter, no route), far (outer), wild (no region).
/// Routes: hub-a open, a-b and b-c behind `GATE`, a-far open, far-wild open.
fn data(reveal: bool) -> Data {
    let w = "w";
    let mut quarter = json!({"id": "quarter", "name": "Quarter", "parent": ""});
    if reveal {
        quarter[MAP_REVEAL] = json!(true);
    }
    let gated = |id: &str, a: &str, b: &str| json!({"id": id, "from": a, "to": b, "time": 1, "requires_flags": [{"key": GATE, "value": true}]});
    let worlds = json!({
        w: {"manifest": {"id": w, "start_location": "hub", "start_party": ["hero"]}, "own": {
            "regions": with_ids(w, json!([quarter, {"id": "ward", "name": "Ward", "parent": "quarter"}, {"id": "outer", "name": "Outer", "parent": ""}])),
            "locations": with_ids(w, json!([
                {"id": "hub", "name": "Hub", "region": "ward"}, {"id": "a", "name": "Place A", "region": "quarter"},
                {"id": "b", "name": "Place B", "region": "quarter"}, {"id": "c", "name": "Place C", "region": "quarter"},
                {"id": "lone", "name": "Lone", "region": "quarter"}, {"id": "far", "name": "Far", "region": "outer"},
                {"id": "wild", "name": "Wild"}
            ])),
            "routes": with_ids(w, json!([
                {"id": "r_hub_a", "from": "hub", "to": "a", "time": 1}, gated("r_a_b", "a", "b"), gated("r_b_c", "b", "c"),
                {"id": "r_a_far", "from": "a", "to": "far", "time": 1}, {"id": "r_far_wild", "from": "far", "to": "wild", "time": 1}
            ]))
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A new game at `at` with `visited` been to.
fn state(d: &Data, at: &str, visited: &[&str]) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    for n in visited {
        gs.set_flag(&explore::visited_flag(&q(n)), json!(true));
    }
    explore::enter(d, &mut gs, at, explore::SPAWN);
    gs
}

fn q(id: &str) -> String {
    format!("w:{id}")
}

fn qs(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|i| q(i)).collect()
}

fn view_nodes(d: &Data, gs: &GameState) -> Vec<(String, bool)> {
    json::aget(&travel::view(d, gs), "nodes").iter().map(json::dict).map(|n| (json::s(n, "qid"), json::b(n, "revealed", false))).collect()
}

fn view_routes(d: &Data, gs: &GameState) -> Vec<String> {
    json::aget(&travel::view(d, gs), "routes").iter().map(|r| json::s(json::dict(r), "qid")).collect()
}

#[test]
fn the_region_chain_runs_up_the_parents() {
    let d = data(true);
    assert_eq!(travel::region_chain(&d, "w", &q("hub")), qs(&["ward", "quarter"]));
    assert_eq!(travel::region_chain(&d, "w", &q("far")), qs(&["outer"]));
    assert!(travel::region_chain(&d, "w", &q("wild")).is_empty(), "a place in no region");
}

#[test]
fn off_the_map_shows_the_known_places_as_before() {
    let d = data(false);
    let gs = state(&d, "hub", &["hub"]);
    assert_eq!(travel::reveal_region(&d, &gs), None);
    assert_eq!(travel::shown(&d, &gs), travel::known(&d, &gs), "shown is known");
    assert_eq!(travel::known(&d, &gs), qs(&["hub", "a"]), "the hub and one open road past it");
    let view = travel::view(&d, &gs);
    assert!(json::aget(&view, "nodes").iter().all(|n| !json::dict(n).contains_key("revealed")), "no revealed key");
    assert_eq!(view_routes(&d, &gs), qs(&["r_hub_a"]));
}

#[test]
fn on_every_routed_place_of_the_region_shows_the_unreachable_ones_locked() {
    let d = data(true);
    let gs = state(&d, "hub", &["hub"]);
    assert_eq!(travel::reveal_region(&d, &gs), Some(q("quarter")), "the hub's district sits in the revealing quarter");
    let shown = travel::shown(&d, &gs);
    assert_eq!(shown, qs(&["hub", "a", "b", "c"]), "every routed place of the quarter, node order");
    assert!(!shown.contains(&q("lone")), "a place no route touches never shows");
    assert!(!shown.contains(&q("far")) && !shown.contains(&q("wild")), "places of other regions only when known");
    assert_eq!(view_nodes(&d, &gs), vec![(q("hub"), false), (q("a"), false), (q("b"), true), (q("c"), true)], "revealed marks the places only the region shows");
    let locked: Vec<(String, bool)> = shown.iter().map(|n| (n.clone(), travel::locked(&d, &gs, n))).collect();
    assert_eq!(locked, vec![(q("hub"), false), (q("a"), false), (q("b"), true), (q("c"), true)], "the party's place and a place it can travel to are not locked");
    assert_eq!(view_routes(&d, &gs), qs(&["r_hub_a", "r_a_b", "r_b_c"]), "the routes between places shown, shut ones too");
    // a route opening and a visit lift the lock; far shows once known (one open road past a visited place)
    let mut gs = state(&d, "hub", &["hub", "a"]);
    gs.set_flag(GATE, json!(true));
    assert!(!travel::locked(&d, &gs, &q("b")), "b: an open way over known places");
    assert!(travel::locked(&d, &gs, &q("c")), "c: not known yet, so no trip");
    assert_eq!(travel::shown(&d, &gs), qs(&["hub", "a", "b", "c", "far"]), "far is known now");
    assert_eq!(view_nodes(&d, &gs).iter().filter(|n| n.1).map(|n| n.0.clone()).collect::<Vec<_>>(), qs(&["c"]));
}

#[test]
fn outside_the_revealing_region_the_map_is_as_before() {
    let d = data(true);
    let gs = state(&d, "far", &["far"]);
    assert_eq!(travel::reveal_region(&d, &gs), None, "far's region does not reveal");
    assert_eq!(travel::shown(&d, &gs), travel::known(&d, &gs));
    let gs = state(&d, "wild", &["wild"]);
    assert_eq!(travel::reveal_region(&d, &gs), None, "a place in no region");
    assert_eq!(travel::shown(&d, &gs), travel::known(&d, &gs));
    // the same game with the region off and on differs only by the revealed places
    let (off, on) = (data(false), data(true));
    let (g_off, g_on) = (state(&off, "a", &["hub", "a"]), state(&on, "a", &["hub", "a"]));
    let known: Vec<(String, bool)> = travel::known(&off, &g_off).into_iter().map(|n| (n, false)).collect();
    assert_eq!(view_nodes(&on, &g_on).into_iter().filter(|n| !n.1).collect::<Vec<_>>(), known, "the known places, unmarked, either way");
}
