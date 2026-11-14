//! MAP-HOOKS1 on scratch worlds: `travel::paths` offers up to `PATHS_MAX` simple ways over open legs between known
//! nodes, each at most `PATHS_DAYS_SLACK` days longer than the cheapest, ranked by days, upkeep, legs and route qids,
//! priced by `campaign::purse::leg_upkeep`, labelled by display names only; `travel::travel_via` walks a chosen way and
//! refuses one that is not a chain of open legs to the place; `travel::travel` still takes `travel::path`. Every
//! expectation follows from the rules and the fixture's own numbers.

use aegis_core::json;
use rpgkit_world::campaign::purse;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::travel::{self, TravelPath, PATHS_DAYS_SLACK, PATHS_MAX};
use rpgkit_world::{explore, Data, GameState};
use serde_json::{json, Value};

const SUPPLIES: i64 = 40;
/// The upkeep a day in the campaign fixture (`purse.upkeep.per_day`).
const PER_DAY: i64 = 2;
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

/// A route row: `id` from `a` to `b` taking `time` days, named "<Id> Road", no road fights.
fn road(id: &str, a: &str, b: &str, time: i64) -> Value {
    json!({"id": id, "name": format!("{} Road", id.to_uppercase()), "from": a, "to": b, "time": time, "encounter_pct": 0})
}

/// World `w` with places hub, a, b, c (named "Hub", "Place A", ...) over `routes`; `campaign` runs a campaign with
/// `PER_DAY` upkeep a day.
fn data(routes: Vec<Value>, campaign: bool) -> Data {
    let w = "w";
    let mut manifest = json!({"id": w, "start_location": "hub", "start_party": ["hero"]});
    if campaign {
        manifest["campaign"] = json!({"start_act": 1});
        manifest["purse"] = json!({"upkeep": {"per_day": PER_DAY}});
    }
    let worlds = json!({
        w: {"manifest": manifest, "own": {
            "locations": with_ids(w, json!([{"id": "hub", "name": "Hub"}, {"id": "a", "name": "Place A"}, {"id": "b", "name": "Place B"}, {"id": "c", "name": "Place C"}])),
            "routes": with_ids(w, Value::Array(routes))
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A new game at the hub with every node visited (every way is known).
fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    gs.party.supplies = SUPPLIES;
    for n in ["hub", "a", "b", "c"] {
        gs.set_flag(&explore::visited_flag(&format!("w:{n}")), json!(true));
    }
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

fn routes_of(ps: &[TravelPath]) -> Vec<Vec<String>> {
    ps.iter().map(|p| p.routes.clone()).collect()
}

fn q(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|i| format!("w:{i}")).collect()
}

/// hub-b direct (2 days), hub-a-b (1 + 1), hub-c-b (1 + 2), hub-c-a-b (1 + 1 + 1), hub-a-c-b (1 + 1 + 2).
fn diamond() -> Vec<Value> {
    vec![road("direct", "hub", "b", 2), road("ha", "hub", "a", 1), road("ab", "a", "b", 1), road("hc", "hub", "c", 1), road("cb", "c", "b", 2), road("ac", "a", "c", 1)]
}

#[test]
fn at_most_three_ways_ranked_by_days_then_legs() {
    let d = data(diamond(), false);
    let gs = state(&d);
    let ps = travel::paths(&d, &gs, "w:hub", "w:b");
    assert_eq!(ps.len(), PATHS_MAX, "five ways fit the slack; only the best three are offered");
    // days: direct 2 (1 leg), ha+ab 2 (2 legs), hc+cb 3 (2 legs) before hc+ac+ab 3 (3 legs) and ha+ac+cb 4
    assert_eq!(routes_of(&ps), [q(&["direct"]), q(&["ha", "ab"]), q(&["hc", "cb"])]);
    assert_eq!(ps.iter().map(|p| p.days).collect::<Vec<_>>(), [2, 1 + 1, 1 + 2]);
    assert_eq!(ps[1].nodes, q(&["a", "b"]), "the nodes a way reaches, the place last");
    assert_eq!(ps[0].routes, travel::path(&d, &gs, "w:hub", "w:b"), "the first way is the one travel takes");
    let cheapest = ps[0].days;
    assert!(ps.iter().all(|p| p.days <= cheapest + PATHS_DAYS_SLACK));
    assert_eq!(travel::paths(&d, &gs, "w:hub", "w:hub"), [], "no way to where the party stands");
    assert_eq!(travel::paths(&d, &gs, "w:hub", "w:nowhere"), []);
}

#[test]
fn labels_name_what_sets_a_way_apart_by_display_names() {
    let d = data(diamond(), false);
    let gs = state(&d);
    let labels: Vec<String> = travel::paths(&d, &gs, "w:hub", "w:b").into_iter().map(|p| p.label).collect();
    // the direct road passes no stop; the others each pass a stop no other way passes
    assert_eq!(labels, ["by DIRECT Road", "via Place A", "via Place C"]);
    // from a: the one-leg way is named by its road
    let from_a: Vec<String> = travel::paths(&d, &gs, "w:a", "w:b").into_iter().map(|p| p.label).collect();
    assert_eq!(from_a[0], "by AB Road");
    assert!(from_a.iter().all(|l| !l.contains("w:") && !l.contains('_')), "no ids in a label: {from_a:?}");
}

#[test]
fn the_slack_and_closed_roads_limit_the_ways() {
    // hub-b 1 day; hub-a-b 2 days; hub-c-b 4 days (> 1 + slack); a closed parallel road hub-b until the gate flag
    let mut routes = vec![road("direct", "hub", "b", 1), road("ha", "hub", "a", 1), road("ab", "a", "b", 1), road("hc", "hub", "c", 2), road("cb", "c", "b", 2)];
    let mut locked = road("locked", "hub", "b", 1);
    locked["requires_flags"] = json!([{"key": GATE, "value": true}]);
    routes.push(locked);
    let d = data(routes, false);
    let mut gs = state(&d);
    let ps = travel::paths(&d, &gs, "w:hub", "w:b");
    assert_eq!(routes_of(&ps), [q(&["direct"]), q(&["ha", "ab"])], "the 4-day way is past the slack; the closed road is not offered");
    assert!(4 > ps[0].days + PATHS_DAYS_SLACK);
    gs.set_flag(GATE, json!(true));
    let ps = travel::paths(&d, &gs, "w:hub", "w:b");
    assert_eq!(routes_of(&ps), [q(&["direct"]), q(&["locked"]), q(&["ha", "ab"])], "an equal way ties on its route qid");
    assert_eq!(ps.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(), ["by DIRECT Road", "by LOCKED Road", "via Place A"]);
}

#[test]
fn upkeep_is_priced_by_the_purse() {
    for campaign in [false, true] {
        let d = data(diamond(), campaign);
        let gs = state(&d);
        for p in travel::paths(&d, &gs, "w:hub", "w:b") {
            let legs: i64 = p.routes.iter().map(|rq| purse::leg_upkeep(&d, "w", travel::leg_days(&travel::route_in(&d, "w", rq), travel::mode_days(&d, &gs)))).sum();
            assert_eq!(p.supplies, legs, "{:?}", p.routes);
            let per_day = if campaign { PER_DAY } else { 1 };
            assert_eq!(p.supplies, p.days * per_day, "{campaign}: each day's upkeep");
        }
    }
}

#[test]
fn travel_via_walks_the_chosen_way() {
    let d = data(diamond(), true);
    let mut gs = state(&d);
    let ps = travel::paths(&d, &gs, "w:hub", "w:b");
    let chosen = ps[2].clone();
    let res = travel::travel_via(&d, &mut gs, "b", &chosen.routes);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(json::s(&res, "status"), "arrived");
    assert_eq!(gs.location_id, "w:b");
    assert_eq!(json::i(&res, "legs_done", 0), chosen.legs() as i64);
    assert_eq!(json::i(&res, "time", 0), chosen.days);
    assert_eq!(json::i(&res, "supplies_spent", 0), chosen.supplies, "the party could pay every leg");
    assert_eq!(gs.party.supplies, SUPPLIES - chosen.supplies);
    for rq in &chosen.routes {
        assert!(gs.flag_true(&travel::traversed_flag(rq)), "{rq} walked");
    }
    assert!(!gs.flag_true(&travel::traversed_flag("w:direct")), "the cheapest road was not taken");
}

#[test]
fn travel_via_refuses_a_way_that_is_not_a_chain_of_open_legs() {
    let d = data(diamond(), false);
    let mut gs = state(&d);
    let before = gs.clone();
    for bad in [q(&[]), q(&["ab"]), q(&["ha"]), q(&["ha", "nowhere"]), q(&["ha", "ab", "cb"])] {
        let res = travel::travel_via(&d, &mut gs, "b", &bad);
        assert!(!json::b(&res, "ok", true), "{bad:?}: {res:?}");
        assert_eq!(gs.location_id, before.location_id, "{bad:?}: nothing moves");
        assert_eq!(gs.time, before.time);
    }
}

#[test]
fn travel_still_takes_the_cheapest_path() {
    let d = data(diamond(), false);
    let mut gs = state(&d);
    let way = travel::path(&d, &gs, "w:hub", "w:b");
    let res = travel::travel(&d, &mut gs, "b");
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(json::i(&res, "legs_done", 0), way.len() as i64);
    for rq in &way {
        assert!(gs.flag_true(&travel::traversed_flag(rq)));
    }
}
