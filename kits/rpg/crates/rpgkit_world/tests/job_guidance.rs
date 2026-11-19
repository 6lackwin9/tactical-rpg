//! PR7-37 (PT-02, PT-08, PT-24, PT-31) on a scratch world (no shipped data): what the game tells about a job. A fight
//! not won leaves its job open (`quest::open_on_fight`: the active quests whose current step is to win it); the world
//! map's legend lists every active quest, with its place when its step has one (`travel::legend_quests`); a place no
//! road reaches names the way there over roads and places' exits (`travel::way_toward`, `way_words`); a locked job
//! names the quest that opens it (`board::unlock_words`). Expectations follow from the fixture's rules.

use aegis_core::json;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::travel::WayStep;
use rpgkit_world::{board, explore, quest, travel, Data, GameState};
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

fn exit(id: &str, to: &str, label: &str) -> Value {
    json!({"id": id, "kind": "exit", "anchor": id, "to": to, "label": label})
}

/// World `w`: roads hub - grate and grate - yard; the grate's exit "West drains" leads into the drains, whose exit
/// leads on to the junction (a place no road reaches); the junction's way back is the world map. Quests: `sweep` (win
/// `enc_sweep` at the junction), `flagged` (a flag step, no place), `opener` (sets `opened` once done) and `later`
/// (on the board, locked until `opened`).
fn data() -> Data {
    let w = "w";
    let locations = json!([
        {"id": "hub", "name": "Hub"},
        {"id": "grate", "name": "The Grate", "markers": [exit("west", "drains", "West drains"), exit("map", "world_map", "World map")]},
        {"id": "drains", "name": "The Drains", "markers": [exit("on", "junction", "")]},
        {"id": "junction", "name": "The Junction", "markers": [exit("map", "world_map", "World map")]},
        {"id": "yard", "name": "The Yard"}
    ]);
    let quests = json!([
        {"id": "sweep", "name": "The Sweep", "template": "clear_site", "steps": [{"text": "Break the sweep", "encounter": "enc_sweep"}]},
        {"id": "flagged", "name": "Flagged", "template": "flag_job", "steps": [{"text": "Find the cipher", "flag": "cipher"}]},
        {"id": "opener", "name": "The Opener", "template": "clear_site", "complete_flags": [{"key": "opened", "value": true}], "steps": [{"text": "Open it", "encounter": "enc_yard"}]},
        {"id": "later", "name": "Later", "template": "clear_site", "board": "b", "show_locked": true, "requires_flags": [{"key": "opened", "value": true}], "steps": [{"text": "Later on", "encounter": "enc_yard"}]},
        {"id": "riddle", "name": "Riddle", "template": "clear_site", "board": "b", "show_locked": true, "requires_flags": [{"key": "nobody_sets_this", "value": true}], "steps": [{"text": "?", "encounter": "enc_yard"}]}
    ]);
    let worlds = json!({w: {"manifest": {"id": w, "start_location": "hub", "start_party": ["hero"]}, "own": {
        "locations": with_ids(w, locations),
        "routes": with_ids(w, json!([{"id": "r1", "from": "hub", "to": "grate", "time": 1}, {"id": "r2", "from": "grate", "to": "yard", "time": 1}])),
        "characters": with_ids(w, json!([{"id": "clerk", "name": "Clerk", "role": "npc"}])),
        "encounters": with_ids(w, json!([{"id": "enc_sweep", "name": "Sweep", "location": "junction", "enemies": []}, {"id": "enc_yard", "name": "Yard", "location": "yard", "enemies": []}])),
        "boards": with_ids(w, json!([{"id": "b", "name": "Board", "keeper": "clerk"}])),
        "quests": with_ids(w, quests)
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let game = json!({"quests": [
        {"id": "clear_site", "objectives": [{"kind": "win_encounter"}], "rewards": {"xp": 5}},
        {"id": "flag_job", "objectives": [{"kind": "flag"}], "rewards": {"xp": 5}}
    ]});
    Data::from_values(&catalog, &worlds, &game)
}

/// At the hub (visited), `sweep` and `flagged` under way.
fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    for q in ["w:sweep", "w:flagged"] {
        gs.quests.insert(q.into(), json!({"state": "active", "objective": 0}));
    }
    gs
}

fn name<'a>(d: &'a Data, gs: &GameState) -> impl Fn(&str) -> String + 'a {
    let gs = gs.clone();
    move |q: &str| json::s_or(explore::resolve_location(d, &gs, q), "name", q)
}

#[test]
fn a_fight_not_won_leaves_the_job_whose_step_it_is_open() {
    let d = data();
    let mut gs = state(&d);
    assert_eq!(quest::open_on_fight(&d, &gs, "enc_sweep"), ["w:sweep"], "the active job whose current step is the fight");
    assert_eq!(quest::open_on_fight(&d, &gs, "w:enc_sweep"), ["w:sweep"], "by qid too");
    assert!(quest::open_on_fight(&d, &gs, "enc_yard").is_empty(), "no active job fights it");
    assert!(quest::open_on_fight(&d, &gs, "").is_empty());
    gs.quests.insert("w:sweep".into(), json!({"state": "ready", "objective": 1}));
    assert!(quest::open_on_fight(&d, &gs, "enc_sweep").is_empty(), "a job past the step is not left open");
    assert_eq!(quest::quests_fighting(&d, &gs, "w:enc_yard"), ["w:opener", "w:later", "w:riddle"], "every quest with the fight as a step");
}

#[test]
fn the_legend_lists_every_active_quest_with_its_place_when_it_has_one() {
    let d = data();
    let gs = state(&d);
    let rows: Vec<(String, String)> = travel::legend_quests(&d, &gs).into_iter().map(|(n, t, _)| (n, t)).collect();
    assert_eq!(rows, [("w:junction".to_string(), "The Sweep".to_string()), (String::new(), "Flagged".to_string())], "a flag step has no place, and is still listed");
    let places: Vec<String> = travel::quest_places(&d, &gs).into_iter().map(|(_, t)| t).collect();
    assert_eq!(places, ["The Sweep"], "the places list (the map's) keeps the quests with a place only");
}

#[test]
fn a_place_no_road_reaches_names_the_way_over_roads_and_exits() {
    let d = data();
    let mut gs = state(&d);
    assert!(travel::path(&d, &gs, "w:hub", "w:junction").is_empty(), "no road to the junction");
    let way = travel::way_toward(&d, &gs, "w:hub", "w:junction");
    assert_eq!(
        way,
        [
            WayStep::Road { to: "w:grate".into(), known: true },
            WayStep::Exit { at: "w:grate".into(), label: "West drains".into(), to: "w:drains".into() },
            WayStep::Exit { at: "w:drains".into(), label: String::new(), to: "w:junction".into() }
        ]
    );
    let words = travel::way_words(&way, "w:junction", &name(&d, &gs));
    assert_eq!(words, "Travel to The Grate, enter it and take West drains, then on toward The Junction.");
    // in the grate: enter it and take the exit
    explore::enter(&d, &mut gs, "grate", explore::SPAWN);
    let way = travel::way_toward(&d, &gs, "w:grate", "w:junction");
    assert_eq!(travel::way_words(&way, "w:junction", &name(&d, &gs)), "Enter The Grate and take West drains, then on toward The Junction.");
    // from the drains the unlabelled exit names where it goes
    let way = travel::way_toward(&d, &gs, "w:drains", "w:junction");
    assert_eq!(travel::way_words(&way, "w:junction", &name(&d, &gs)), "Enter The Drains and take the way to The Junction.");
    assert!(travel::way_toward(&d, &gs, "w:junction", "w:junction").is_empty(), "already there");
    assert_eq!(travel::way_words(&[], "w:junction", &name(&d, &gs)), "", "no way: no words");
}

#[test]
fn a_road_past_an_unknown_place_says_to_travel_to_the_known_one_first() {
    let d = data();
    let gs = state(&d);
    // the yard is two legs out: the grate is known (a leg from the visited hub), the yard is not yet
    assert!(!travel::known(&d, &gs).contains(&"w:yard".to_string()));
    let way = travel::way_toward(&d, &gs, "w:hub", "w:yard");
    assert_eq!(way, [WayStep::Road { to: "w:grate".into(), known: true }, WayStep::Road { to: "w:yard".into(), known: false }]);
    assert_eq!(travel::way_words(&way, "w:yard", &name(&d, &gs)), "Travel to The Grate first: the road on to The Yard shows from there.");
}

#[test]
fn a_locked_job_names_the_quest_that_opens_it() {
    let d = data();
    let mut gs = state(&d);
    assert_eq!(quest::status(&d, &gs, "w:later"), "locked");
    assert_eq!(board::unlock_words(&d, &gs, "w:later"), "Opens after The Opener.", "the quest whose complete_flags set the missing flag");
    assert_eq!(board::unlock_words(&d, &gs, "w:riddle"), "", "nothing known sets its flag: no words (the board keeps its own)");
    gs.set_flag("opened", json!(true));
    assert_eq!(board::unlock_words(&d, &gs, "w:later"), "", "the flag holds: nothing missing");
}
