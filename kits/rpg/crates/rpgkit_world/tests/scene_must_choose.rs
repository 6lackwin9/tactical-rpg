//! ACT1-HOOKS1 (FB-241 a): a queued scene (`DialogueRunner::open_scene`) has no farewell reply, so the game must not let
//! it be dismissed while a reply is still to pick (`DialogueRunner::must_choose`, read by the GUI's Back and Esc). The
//! rule reads the tree alone: an open reply on the node shown, or any reply on a node its `next` lines lead to; a node
//! whose replies are all locked may be left (never a dead end), and so may a scene that only ends, a closed one, or the
//! same tree talked to as a visit (`open`, which has a farewell).

use aegis_core::json;
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{Data, GameState};
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

const LOCKED_BY: &str = "w.never";
/// The gold the `toll` scene's only reply takes.
const TOLL: i64 = 30;

/// World `w`: the guide (whose visit tree is `ask`) and the trees `ask` (a choice at the start), `lead_in` (two lines,
/// then a choice), `only_lines` (a line, then the end), `locked` (one locked reply shown), `services_on` (a choice,
/// services on) and `toll` (a line, then one reply that takes `TOLL` gold).
fn data() -> Data {
    let pick = json!([{"text": "Yes.", "to": ""}, {"text": "No.", "to": ""}]);
    let dialogues = json!([
        {"id": "ask", "start": "a", "nodes": [{"id": "a", "text": "Choose.", "choices": pick}]},
        {"id": "lead_in", "start": "l1", "services": false, "nodes": [
            {"id": "l1", "text": "One.", "next": "l2"},
            {"id": "l2", "text": "Two.", "next": "l3"},
            {"id": "l3", "text": "Choose.", "choices": pick}
        ]},
        {"id": "only_lines", "start": "o1", "services": false, "nodes": [
            {"id": "o1", "text": "A line.", "next": "o2"},
            {"id": "o2", "text": "The end.", "end": true}
        ]},
        {"id": "locked", "start": "k", "services": false, "nodes": [
            {"id": "k", "text": "Nothing to pick.", "choices": [{"text": "Later.", "to": "", "show_locked": true, "requires": [{"kind": "flag", "key": LOCKED_BY, "value": true}]}]}
        ]},
        {"id": "services_on", "start": "s", "services": true, "nodes": [{"id": "s", "text": "Choose.", "choices": pick}]},
        {"id": "toll", "start": "t0", "services": false, "nodes": [
            {"id": "t0", "text": "The gate.", "next": "t"},
            {"id": "t", "text": "Pay.", "choices": [{"text": "Pay.", "to": "", "effects": [{"effect": "take", "gold": TOLL}]}]}
        ]}
    ]);
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero"]}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub"}])),
        "characters": with_ids("w", json!([{"id": "hero", "name": "Hero", "role": "party"}, {"id": "guide", "name": "Guide", "role": "npc", "dialogue": "ask"}])),
        "dialogues": with_ids("w", dialogues)
    }}});
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

fn has_farewell(d: &Data, gs: &GameState, r: &DialogueRunner) -> bool {
    json::aget(&r.view(d, gs), "choices").iter().any(|c| json::s(json::dict(c), "service") == "farewell")
}

#[test]
fn a_scene_with_an_open_reply_must_be_answered_and_a_visit_never() {
    let d = data();
    let mut gs = state(&d);
    let scene = DialogueRunner::open_scene(&d, &mut gs, "ask", "guide");
    assert!(!scene.closed && scene.scene, "{:?}", scene.errors);
    assert!(!has_farewell(&d, &gs, &scene), "a scene offers no farewell");
    assert!(scene.must_choose(&d, &gs), "an open reply on the node shown");
    let visit = DialogueRunner::open(&d, &mut gs, "guide");
    assert!(!visit.closed && !visit.scene);
    assert!(has_farewell(&d, &gs, &visit) && !visit.must_choose(&d, &gs), "the same tree as a visit has a farewell and may be left");
    // a tree with services on, queued as a scene: still no farewell, still answered
    let with_services = DialogueRunner::open_scene(&d, &mut gs, "services_on", "");
    assert!(!has_farewell(&d, &gs, &with_services) && with_services.must_choose(&d, &gs));
}

#[test]
fn the_lines_before_a_choice_hold_and_the_choice_releases_the_scene() {
    let d = data();
    let mut gs = state(&d);
    let mut r = DialogueRunner::open_scene(&d, &mut gs, "lead_in", "");
    for node in ["l1", "l2"] {
        assert_eq!(r.node_id, node);
        assert!(json::aget(&r.view(&d, &gs), "choices").is_empty(), "{node}: a line, no reply yet");
        assert!(r.must_choose(&d, &gs), "{node}: a reply waits ahead on its next lines");
        let res = r.advance(&d, &mut gs);
        assert!(json::b(&res, "ok", false), "{res:?}");
    }
    assert_eq!(r.node_id, "l3");
    assert!(r.must_choose(&d, &gs), "the choice itself");
    let res = r.choose(&d, &mut gs, 0);
    assert!(json::b(&res, "ok", false) && r.closed, "{res:?}");
    assert!(!r.must_choose(&d, &gs), "answered: closed, nothing holds it");
}

#[test]
fn a_scene_with_nothing_to_pick_may_be_left() {
    let d = data();
    let mut gs = state(&d);
    let lines = DialogueRunner::open_scene(&d, &mut gs, "only_lines", "");
    assert!(!lines.closed && !lines.must_choose(&d, &gs), "lines that only end: no reply anywhere ahead");
    let locked = DialogueRunner::open_scene(&d, &mut gs, "locked", "");
    let view = locked.view(&d, &gs);
    assert!(json::aget(&view, "choices").iter().all(|c| json::b(json::dict(c), "locked", false)) && !json::aget(&view, "choices").is_empty(), "{view:?}");
    assert!(!locked.must_choose(&d, &gs), "every reply locked: never a dead end");
    gs.set_flag(LOCKED_BY, Value::Bool(true));
    assert!(locked.must_choose(&d, &gs), "the reply opens: it must be answered");
    let refused = DialogueRunner::open_scene(&d, &mut gs, "nope", "");
    assert!(refused.closed && !refused.must_choose(&d, &gs), "a scene that did not open");
}

/// Review H1: an open reply the choice would refuse (its precheck: here a `take` the party cannot afford) is not an
/// answer, so a scene whose only reply is refused may be left; once affordable it must be answered. The same reading
/// holds for the reply ahead on the scene's `next` lines.
#[test]
fn a_reply_the_choice_would_refuse_does_not_hold_the_scene() {
    let d = data();
    let mut gs = state(&d);
    gs.party.gold = TOLL - 1;
    let mut r = DialogueRunner::open_scene(&d, &mut gs, "toll", "");
    assert!(!r.closed && !r.must_choose(&d, &gs), "on the line: the reply ahead cannot be afforded");
    gs.party.gold = TOLL;
    assert!(r.must_choose(&d, &gs), "on the line: the reply ahead can be paid");
    assert!(json::b(&r.advance(&d, &mut gs), "ok", false));
    assert!(r.must_choose(&d, &gs), "affordable: it must be answered");
    gs.party.gold = TOLL - 1;
    let view = r.view(&d, &gs);
    assert!(json::aget(&view, "choices").iter().all(|c| !json::b(json::dict(c), "locked", true)), "the reply shows open: {view:?}");
    assert!(!json::b(&r.choose(&d, &mut gs, 0), "ok", true), "and the choice refuses it");
    assert!(!r.must_choose(&d, &gs), "every reply refused: the scene may be left");
}
