//! STORYUI1-W mission board on a scratch world (STORYUI1.md 4.4, 4.5, 5.5 B1-B10): the board is open while its keeper
//! is there and its flags hold; its quests sit in sections by status and track; the keeper's conversation offers the
//! board instead of its quests; Accept starts a quest given by the keeper or no one; the first open sets the board's
//! flag, after which the world map lists its jobs on their nodes. Expectations follow from the rules and the fixture.

use aegis_core::json::{self, Dict};
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{board, explore, quest, travel, Data, GameState};
use serde_json::{json, Value};

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

const KILN_STEP: &str = "Reach the kiln";

/// World `ward`: Clerk keeps `clerk_board` (open from act 1); the board lists a main mission, a second one locked behind
/// it (shown), one locked and hidden, a job, a job given by Smith, and a quest closed by a flag; Clerk also gives a quest
/// without a board. Three nodes on routes from the hub.
fn data() -> Data {
    let quests = json!([
        {"id": "m1", "name": "M1", "giver": "clerk", "board": "clerk_board", "track": "main", "template": "reach_and_clear",
         "steps": [{"text": KILN_STEP, "location": "kiln"}, {"text": "Raid the kiln", "location": "kiln", "encounter": "enc_a"}]},
        {"id": "m2", "name": "M2", "giver": "clerk", "board": "clerk_board", "track": "main", "template": "clear_site", "requires": ["m1"], "show_locked": true,
         "steps": [{"text": "Hold the watch", "encounter": "enc_w"}]},
        {"id": "secret", "name": "Secret", "giver": "clerk", "board": "clerk_board", "template": "clear_site", "requires": ["m1"],
         "steps": [{"text": "Hidden", "encounter": "enc_w"}]},
        {"id": "job1", "name": "Job One", "board": "clerk_board", "template": "clear_site", "steps": [{"text": "Clear the watch", "encounter": "enc_w"}]},
        {"id": "foreign", "name": "Foreign", "giver": "smith", "board": "clerk_board", "template": "clear_site", "steps": [{"text": "Smith's", "encounter": "enc_w"}]},
        {"id": "closing", "name": "Closing", "giver": "clerk", "board": "clerk_board", "template": "clear_site", "closed_text": "Too late.",
         "variants": [{"id": "late", "when_flags": [{"key": "late", "value": true}], "set": {"available": false}}],
         "steps": [{"text": "Before it closes", "encounter": "enc_w"}]},
        {"id": "plain", "name": "Plain", "giver": "clerk", "template": "clear_site", "steps": [{"text": "Plain work", "encounter": "enc_w"}]}
    ]);
    let worlds = json!({
        "ward": {"manifest": {"id": "ward", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": with_ids("ward", json!([{"id": "hub", "name": "Hub"}, {"id": "kiln", "name": "Kiln"}, {"id": "watch", "name": "Watch"}])),
            "routes": with_ids("ward", json!([{"id": "r1", "from": "hub", "to": "kiln", "time": 1}, {"id": "r2", "from": "hub", "to": "watch", "time": 1}])),
            "characters": with_ids("ward", json!([{"id": "clerk", "name": "Clerk", "role": "npc"}, {"id": "smith", "name": "Smith", "role": "npc"}])),
            "encounters": with_ids("ward", json!([{"id": "enc_a", "name": "A", "enemies": []}, {"id": "enc_w", "name": "W", "location": "watch", "enemies": []}])),
            "boards": with_ids("ward", json!([{"id": "clerk_board", "name": "Notice board", "keeper": "clerk", "intro": "Work.",
                "requires_flags": [{"key": "act", "op": ">=", "value": 1}]}])),
            "quests": with_ids("ward", quests)
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let game = json!({"quests": [
        {"id": "reach_and_clear", "objectives": [{"kind": "reach_location"}, {"kind": "win_encounter"}], "rewards": {"xp": 10}},
        {"id": "clear_site", "objectives": [{"kind": "win_encounter"}], "rewards": {"xp": 5}}
    ]});
    Data::from_values(&catalog, &worlds, &game)
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "ward", party);
    gs.set_flag(&explore::visited_flag("ward:hub"), json!(true));
    gs
}

fn act1(d: &Data) -> GameState {
    let mut gs = state(d);
    gs.set_flag("act", json!(1));
    gs
}

/// (section id, [quest qid, line]) as the board view lists them.
fn sections(view: &Dict) -> Vec<(String, Vec<(String, String)>)> {
    json::aget(view, "sections")
        .iter()
        .map(|s| {
            let s = json::dict(s);
            let rows = json::aget(s, "rows").iter().map(|r| (json::s(json::dict(r), "quest"), json::s(json::dict(r), "line"))).collect();
            (json::s(s, "id"), rows)
        })
        .collect()
}

fn row(q: &str, line: &str) -> (String, String) {
    (format!("ward:{q}"), line.to_string())
}

fn services(talk: &DialogueRunner, d: &Data, gs: &GameState) -> Vec<String> {
    json::aget(&talk.view(d, gs), "choices").iter().map(|c| json::s(json::dict(c), "service")).collect()
}

#[test]
fn the_board_is_open_while_its_keeper_is_there_and_its_flags_hold() {
    let d = data();
    let gs = state(&d);
    assert!(!board::is_open(&d, &gs, "clerk_board"), "B1: act 1 not reached");
    let gs = act1(&d);
    assert!(board::is_open(&d, &gs, "clerk_board"));
    assert_eq!(board::quests(&d, &gs, "ward:clerk_board"), ["ward:m1", "ward:m2", "ward:secret", "ward:job1", "ward:foreign", "ward:closing"], "B2: file order");
}

#[test]
fn quests_sit_in_sections_by_status_and_track() {
    let d = data();
    let mut gs = act1(&d);
    let view = board::view(&d, &gs, "clerk_board");
    assert_eq!((json::s(&view, "intro"), json::s(&view, "keeper_name")), ("Work.".to_string(), "Clerk".to_string()));
    assert_eq!(
        sections(&view),
        vec![
            ("main".to_string(), vec![row("m1", "")]),
            ("jobs".to_string(), vec![row("job1", ""), row("foreign", ""), row("closing", "")]),
            ("locked".to_string(), vec![row("m2", "Needs M1 (completed)")]),
        ],
        "B5: empty sections left out; a locked quest without show_locked is not listed"
    );
    // B6: a row's node is its first step's location, else its encounter's
    let rows: Vec<Dict> = json::aget(&view, "sections").iter().flat_map(|s| json::aget(json::dict(s), "rows").to_vec()).map(|r| json::dict(&r).clone()).collect();
    let node = |q: &str| rows.iter().find(|r| json::s(r, "quest") == format!("ward:{q}")).map(|r| (json::s(r, "node"), json::s(r, "node_name"))).unwrap();
    assert_eq!(node("m1"), ("ward:kiln".to_string(), "Kiln".to_string()));
    assert_eq!(node("job1"), ("ward:watch".to_string(), "Watch".to_string()));
    // active: the objective's text; ready: "Report to {keeper}"; closed with closed_text
    quest::start(&d, &mut gs, "m1");
    quest::start(&d, &mut gs, "job1");
    gs.quests.insert("ward:job1".into(), json!({"state": "ready", "objective": 1}));
    gs.set_flag("late", json!(true));
    let got = sections(&board::view(&d, &gs, "clerk_board"));
    assert_eq!(
        got,
        vec![
            ("jobs".to_string(), vec![row("foreign", "")]),
            ("under_way".to_string(), vec![row("m1", KILN_STEP), row("job1", "Report to Clerk")]),
            ("locked".to_string(), vec![row("m2", "Needs M1 (completed)")]),
            ("closed".to_string(), vec![row("closing", "Too late.")]),
        ]
    );
    // B7: the detail's steps by progress
    let detail = board::detail(&d, &gs, "ward:m1");
    let states: Vec<String> = json::aget(&detail, "steps").iter().map(|s| json::s(json::dict(s), "state")).collect();
    assert_eq!(states, ["current", "to_come"]);
    assert_eq!(json::dget(&detail, "rewards").get("xp"), Some(&json!(10)), "the template's rewards");
}

#[test]
fn the_keeper_offers_the_board_instead_of_its_quests() {
    let d = data();
    // closed board: no board choice; the quest without a board is offered as before, the board quests are not
    let mut gs = state(&d);
    let talk = DialogueRunner::open(&d, &mut gs, "clerk");
    assert_eq!(services(&talk, &d, &gs), ["offer", "farewell"], "B3: only `plain` is offered");
    assert_eq!(explore::npc_services(&d, &gs, "ward:clerk"), ["offer"], "B4: no board badge while closed");
    assert!(explore::npc_services(&d, &gs, "ward:smith").is_empty(), "B4: a board quest's giver offers nothing");
    // open board: one board choice after the offers (no turn-ins here), before the farewell
    let mut gs = act1(&d);
    let talk = DialogueRunner::open(&d, &mut gs, "clerk");
    assert_eq!(services(&talk, &d, &gs), ["offer", "board", "farewell"]);
    let choice = json::dict(&json::aget(&talk.view(&d, &gs), "choices")[1]).clone();
    assert_eq!(json::s(&choice, "text"), "Show me the board.", "the kit's default line");
    assert_eq!(explore::npc_services(&d, &gs, "ward:clerk"), ["offer", "board"]);
    // the marker badge follows: Smith's only quest is on the board, so his marker shows none; The clerk's shows the quest
    // badge while the board is open even when her own quest is taken
    let hub = d.resolve("ward", "locations", "hub").clone();
    let badge = |gs: &GameState, who: &str| json::s(&explore::marker_view(&d, gs, &hub, json::dict(&json!({"id": "m", "kind": "npc", "character": who}))), "badge");
    assert_eq!(badge(&gs, "smith"), "");
    quest::start(&d, &mut gs, "plain");
    assert_eq!(badge(&gs, "clerk"), "quest");
    let mut closed = state(&d);
    quest::start(&d, &mut closed, "plain");
    assert_eq!(badge(&closed, "clerk"), "", "board closed and her own quest taken: nothing on offer");
}

#[test]
fn accept_starts_a_board_quest_given_by_the_keeper_or_no_one() {
    let d = data();
    let mut gs = act1(&d);
    let mut talk = DialogueRunner::open(&d, &mut gs, "clerk");
    assert!(!gs.flag_true("board.clerk_board.opened"));
    let res = talk.choose(&d, &mut gs, 1);
    assert_eq!(json::s(&res, "board"), "ward:clerk_board");
    assert!(gs.flag_true("board.clerk_board.opened"), "B9");
    assert_eq!(json::s(&talk.view(&d, &gs), "board"), "ward:clerk_board");
    assert_eq!(json::sget(&talk.choose(&d, &mut gs, 0), "errors"), [board::BUSY_TEXT], "B3: the busy rule");
    // B8
    let res = talk.accept(&d, &mut gs, "m1");
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(json::sget(&res, "notes"), ["Quest taken: M1."]);
    assert_eq!(quest::status(&d, &gs, "m1"), "active");
    let res = talk.accept(&d, &mut gs, "job1");
    assert!(json::b(&res, "ok", false), "a quest without a giver: {res:?}");
    assert_eq!(json::sget(&talk.accept(&d, &mut gs, "foreign"), "errors"), ["Only Smith gives this quest."]);
    assert_eq!(quest::status(&d, &gs, "foreign"), "available");
    assert_eq!(json::sget(&talk.accept(&d, &mut gs, "m1"), "errors"), ["Quest 'ward:m1' can't start (it is active)."]);
    assert_eq!(json::sget(&talk.accept(&d, &mut gs, "plain"), "errors"), ["Quest 'plain' is not on this board."]);
    // B11: closing goes back to the conversation
    let res = talk.close_board(&d, &mut gs);
    assert!(json::b(&res, "ok", false) && talk.board_qid.is_empty() && !talk.closed);
    assert!(!talk.view(&d, &gs).contains_key("board"));
    assert_eq!(json::sget(&talk.accept(&d, &mut gs, "job1"), "errors"), ["No board is open."]);
}

fn jobs_by_node(d: &Data, gs: &GameState) -> Vec<(String, Vec<String>)> {
    json::aget(&travel::view(d, gs), "nodes")
        .iter()
        .map(json::dict)
        .filter(|n| n.contains_key("jobs"))
        .map(|n| (json::s(n, "qid"), json::sget(n, "jobs")))
        .collect()
}

#[test]
fn the_world_map_lists_jobs_only_after_the_board_was_opened() {
    let d = data();
    let mut gs = act1(&d);
    assert!(jobs_by_node(&d, &gs).is_empty(), "B10: no jobs key before the first open");
    board::mark_opened(&d, &mut gs, "clerk_board");
    assert_eq!(
        jobs_by_node(&d, &gs),
        vec![("ward:kiln".to_string(), vec!["M1".to_string()]), ("ward:watch".to_string(), vec!["Job One".to_string(), "Foreign".to_string(), "Closing".to_string()])]
    );
    // a taken job leaves the list (the active target keeps its quest marker)
    quest::start(&d, &mut gs, "m1");
    let view = travel::view(&d, &gs);
    let kiln = json::aget(&view, "nodes").iter().map(json::dict).find(|n| json::s(n, "qid") == "ward:kiln").unwrap().clone();
    assert!(!kiln.contains_key("jobs") && json::b(&kiln, "quest", false));
}
