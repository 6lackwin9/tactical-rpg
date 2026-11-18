//! PR6-11 (`docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` 4): the `endings` world kind and the `end` effect's kind
//! `ending`. The ordered branch (`campaign::ending::pick`) finishes on a named row while its gates hold, else on the
//! first row in order whose flag and meter gates hold; the end records the run as completed with the ending (runtime
//! only: the save refusal stays, a save written before holds no trace of it); the card evaluator
//! (`campaign::ending::epilogue`) fills the outcome lines by their flags and one fate card per member, then per departed
//! member (the track stage, the cause line, the keepsake). Every expectation follows from the fixture's own constants.

use aegis_core::json::{self, Dict};
use rpgkit_world::campaign::ending;
use rpgkit_world::{campaign, new_game, progression, roster, Data, GameState};
use serde_json::{json, Value};

/// The gate between the first two endings: the first holds at `SPLIT` or less, the second above it with `FLAG`.
const SPLIT: i64 = 40;
const FLAG: &str = "w.pact";
const TALLY: &str = "w.saved";
const TALLY_N: i64 = 3;
const TRACK: &str = "stone";
const STAGE: i64 = 2;
const CAUSE: &str = "story";

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

/// The three endings in order: `first` (doctrine <= SPLIT), `second` (doctrine > SPLIT and FLAG), `last` (no gates).
fn endings() -> Value {
    let fates = json!({"track": TRACK, "member": "{name} lives on ({stage}).", "departed": "{name} left ({cause}).", "causes": {CAUSE: "{name} became stone."}});
    json!([
        {"id": "first", "name": "First", "title": "The First", "tone": "calm", "requires_meter": [{"id": "doctrine", "op": "<=", "value": SPLIT}],
         "epilogue_paragraphs": ["One.", "Two."], "world_outcomes": [{"flag": FLAG, "text_if_true": "Pact kept.", "text_if_false": "No pact."},
                                                                      {"flag": TALLY, "op": ">=", "value": 1, "text_if_true": "{value} were saved.", "text_if_false": ""}],
         "member_fates_template": fates},
        {"id": "second", "name": "Second", "title": "The Second", "tone": "danger", "requires_flags": [{"key": FLAG, "value": true}],
         "requires_meter": [{"id": "doctrine", "op": ">", "value": SPLIT}], "epilogue_paragraphs": ["Two."], "member_fates_template": fates},
        {"id": "last", "name": "Last", "epilogue_paragraphs": ["Last."]}
    ])
}

/// World `w` (a campaign with the gauge `doctrine`), two party units, a locket item and a staged unit track; `rows`
/// are its endings.
fn data_with(rows: Value) -> Data {
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero", "mate"], "campaign": {"start_act": 1}}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub", "rest": true}])),
        "meters": with_ids("w", json!([{"id": "doctrine", "name": "Doctrine", "kind": "gauge", "min": 0, "max": 100, "start": 0}])),
        "endings": with_ids("w", rows)
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}, {"id": "mate", "name": "Mate", "side": "party", "class_id": "fighter", "level": 1}],
        "items": [{"id": "locket", "name": "Locket", "item_type": "ACCESSORY", "value": 1}],
        "unit_tracks": [{"id": TRACK, "name": "Stone", "mark": "", "cause": "", "stages": [{"name": "Grey"}, {"name": "Hard"}, {"name": "Still"}]}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn game(d: &Data) -> GameState {
    new_game::create_for_world(d, "w", 1).expect("a new game")
}

fn set_doctrine(gs: &mut GameState, v: i64) {
    gs.campaign.as_mut().unwrap().meters.get_mut("doctrine").unwrap().value = v;
}

fn end_fx(named: &str) -> Dict {
    let mut fx = json::dict(&json!({"effect": "end", "kind": "ending"})).clone();
    if !named.is_empty() {
        fx.insert("ending".into(), named.into());
    }
    fx
}

#[test]
fn the_ordered_branch_picks_the_named_row_while_it_holds_else_the_first_that_holds() {
    let d = data_with(endings());
    let mut gs = game(&d);
    let cases: [(i64, bool, &str, &str); 7] = [
        (SPLIT, false, "", "first"),
        (SPLIT + 1, false, "", "last"),
        (SPLIT + 1, true, "", "second"),
        (SPLIT, true, "", "first"),
        (SPLIT, true, "second", "first"),
        (SPLIT, false, "last", "last"),
        (SPLIT + 1, true, "w:last", "last"),
    ];
    for (doctrine, flag, named, want) in cases {
        set_doctrine(&mut gs, doctrine);
        gs.set_flag(FLAG, json!(flag));
        assert_eq!(ending::pick(&d, &gs, named), want, "doctrine {doctrine}, flag {flag}, named '{named}'");
    }
    // no row holds: none
    let d = data_with(json!([endings()[0].clone()]));
    let mut gs = game(&d);
    set_doctrine(&mut gs, SPLIT + 1);
    assert_eq!(ending::pick(&d, &gs, ""), "");
}

#[test]
fn an_ending_completes_the_run_with_its_row_and_saves_stay_refused() {
    let d = data_with(endings());
    let mut gs = game(&d);
    set_doctrine(&mut gs, SPLIT + 1);
    gs.set_flag(FLAG, json!(true));
    let before = gs.to_value().to_string();
    let mut events = Vec::new();
    campaign::apply_effects(&d, &mut gs, &[end_fx("")], &mut events);
    let over = gs.campaign.as_ref().and_then(|c| c.over.clone()).expect("the run is over");
    assert!(over.completed() && over.ending == "second", "{over:?}");
    assert_eq!(campaign::save_refusal(&gs), campaign::RUN_OVER_TEXT, "F3 holds for a completed run");
    let ev = campaign::tick(&d, &mut gs, &campaign::Cause::Effects);
    let told: Vec<&Value> = ev.iter().filter(|e| json::s(json::dict(e), "event") == "run_over").collect();
    assert_eq!(told.len(), 1, "{ev:?}");
    let e = json::dict(told[0]);
    assert_eq!((json::s(e, "kind"), json::b(e, "completed", false), json::s(e, "ending")), ("ending".to_string(), true, "second".to_string()));
    // a later end changes nothing
    campaign::apply_effects(&d, &mut gs, &[json::dict(&json!({"effect": "end", "kind": "fail", "text": "x"})).clone()], &mut Vec::new());
    assert_eq!(gs.campaign.as_ref().and_then(|c| c.over.clone()).map(|o| o.ending), Some("second".into()));
    // the run-over state is runtime only: the campaign block reads as before the end
    let mut back = gs.clone();
    back.campaign.as_mut().unwrap().over = None;
    assert_eq!(back.to_value().to_string(), before, "the save holds no ending");
}

#[test]
fn a_lost_run_keeps_its_old_event_shape() {
    let d = data_with(endings());
    let mut gs = game(&d);
    campaign::apply_effects(&d, &mut gs, &[json::dict(&json!({"effect": "end", "kind": "fail", "card": "", "text": "Lost."})).clone()], &mut Vec::new());
    let over = gs.campaign.as_ref().and_then(|c| c.over.clone()).unwrap();
    assert!(!over.completed() && over.ending.is_empty());
    let e = over.event();
    assert_eq!(json::dict(&e).keys().cloned().collect::<Vec<_>>(), ["event", "kind", "card", "text"], "no completed or ending key");
}

#[test]
fn the_card_evaluator_fills_outcomes_and_one_fate_card_per_member_then_per_departed() {
    let d = data_with(endings());
    let mut gs = game(&d);
    gs.set_flag(TALLY, json!(TALLY_N));
    let ids: Vec<String> = gs.party.members.iter().map(|m| m.id.clone()).collect();
    assert_eq!(ids.len(), 2);
    gs.party.members[0].tracks.insert(TRACK.into(), STAGE);
    roster::depart(&d, &mut gs, &ids[1], "dead", CAUSE, "").expect("departs");
    gs.party.departed.last_mut().unwrap().keepsake = "locket".into();
    let ep = ending::epilogue(&d, &gs, "first").expect("the first ending");
    assert_eq!((ep.title.as_str(), ep.tone.as_str(), ep.paragraphs.len()), ("The First", "calm", 2));
    assert_eq!(ep.outcomes, ["No pact.".to_string(), format!("{TALLY_N} were saved.")]);
    let alive = progression::display_name(&d, &gs.party.members[0]);
    let gone = progression::display_name(&d, &gs.party.departed[0].member);
    let fates: Vec<(String, bool, i64, String, String, String)> = ep.fates.iter().map(|f| (f.name.clone(), f.departed, f.stage, f.stage_name.clone(), f.keepsake.clone(), f.text.clone())).collect();
    assert_eq!(
        fates,
        [
            (alive.clone(), false, STAGE, "Hard".to_string(), String::new(), format!("{alive} lives on (Hard).")),
            (gone.clone(), true, 0, String::new(), "Locket".to_string(), format!("{gone} became stone.")),
        ]
    );
    // the flag set: the other line; the cause line falls back to `departed` for another cause
    gs.set_flag(FLAG, json!(true));
    gs.party.departed[0].cause = "stayed".into();
    let ep = ending::epilogue(&d, &gs, "first").unwrap();
    assert_eq!(ep.outcomes[0], "Pact kept.");
    assert_eq!(ep.fates[1].text, format!("{gone} left (stayed)."));
    // a row without a template: cards with no line; an unknown ending: none
    let last = ending::epilogue(&d, &gs, "last").unwrap();
    assert_eq!((last.tone.as_str(), last.fates.len()), ("calm", 2));
    assert!(last.fates.iter().all(|f| f.text.is_empty()) && last.outcomes.is_empty());
    assert!(ending::epilogue(&d, &gs, "ghost").is_none() && ending::epilogue(&d, &gs, "").is_none());
}

#[test]
fn an_ending_with_no_holding_row_still_completes_the_run() {
    let d = data_with(json!([]));
    let mut gs = game(&d);
    campaign::apply_effects(&d, &mut gs, &[end_fx("")], &mut Vec::new());
    let over = gs.campaign.as_ref().and_then(|c| c.over.clone()).unwrap();
    assert!(over.completed() && over.ending.is_empty());
}
