//! PR7-41 (PT-15) job fights on a scratch world: a fight that a job (a board quest) names as a `win_encounter` step is
//! offered only once a quest naming it is accepted; a fight no job names is always offered; accepting a quest whose
//! fight was already won (a `once` marker's done flag, a paid objective reward, a fired battle trigger) marks that step
//! met, so a one-fight job is ready to turn in; a save holding an active quest stuck on a won fight is repaired on
//! load. Expectations follow from the rules and the fixture.

use aegis_core::json::{self, Dict};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{explore, quest, save, Data, GameState};
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

/// World `ward`: job `raid` (on `clerk_board`) needs the fight `enc_mill`; job `two_step` needs `enc_mill` then
/// `enc_far`; quest `plain` (no board) needs `enc_side`; `enc_free` is named by no quest. The kiln holds a repeatable
/// marker of `enc_mill` (`once: false`, like a fight hidden by its own flag), a `once` marker of `enc_side` and one of
/// `enc_free`.
fn data() -> Data {
    build(false)
}

fn build(two_step: bool) -> Data {
    let mut quests = json!([
        {"id": "raid", "name": "Raid", "board": "clerk_board", "template": "clear_site", "steps": [{"text": "Wreck the mill", "encounter": "enc_mill"}]},
        {"id": "plain", "name": "Plain", "giver": "clerk", "template": "clear_site", "steps": [{"text": "Side work", "encounter": "enc_side"}]}
    ]);
    if two_step {
        quests.as_array_mut().expect("quests").push(json!({"id": "two_step", "name": "Two step", "board": "clerk_board", "template": "clear_two",
            "steps": [{"text": "Mill", "encounter": "enc_mill"}, {"text": "Far", "encounter": "enc_far"}]}));
    }
    let markers = json!([
        {"id": "mill", "kind": "encounter", "anchor": "a", "encounter": "enc_mill", "once": false},
        {"id": "side", "kind": "encounter", "anchor": "b", "encounter": "enc_side"},
        {"id": "free", "kind": "encounter", "anchor": "c", "encounter": "enc_free"}
    ]);
    let worlds = json!({
        "ward": {"manifest": {"id": "ward", "start_location": "kiln", "start_party": ["hero"]}, "own": {
            "locations": with_ids("ward", json!([{"id": "kiln", "name": "Kiln", "markers": markers}])),
            "characters": with_ids("ward", json!([{"id": "clerk", "name": "Clerk", "role": "npc"}])),
            "encounters": with_ids("ward", json!([
                {"id": "enc_mill", "name": "Mill", "enemies": []},
                {"id": "enc_far", "name": "Far", "enemies": []},
                {"id": "enc_side", "name": "Side", "enemies": []},
                {"id": "enc_free", "name": "Free", "enemies": []}
            ])),
            "boards": with_ids("ward", json!([{"id": "clerk_board", "name": "Notice board", "keeper": "clerk", "intro": "Work."}])),
            "quests": with_ids("ward", quests)
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let game = json!({"quests": [
        {"id": "clear_site", "objectives": [{"kind": "win_encounter"}], "rewards": {"xp": 5}},
        {"id": "clear_two", "objectives": [{"kind": "win_encounter"}, {"kind": "win_encounter"}], "rewards": {"xp": 5}}
    ]});
    Data::from_values(&catalog, &worlds, &game)
}

/// `data` plus job `two_step` (`clear_two`: `enc_mill` then `enc_far`).
fn data_two_step() -> Data {
    build(true)
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "ward", party);
    gs.location_id = "ward:kiln".into();
    gs
}

fn marker(d: &Data, id: &str) -> Dict {
    let loc = d.resolve("ward", "locations", "kiln");
    explore::markers_of(loc).into_iter().find(|m| json::s(m, "id") == id).expect("marker")
}

fn presence(d: &Data, gs: &GameState, id: &str) -> &'static str {
    explore::presence(d, gs, "ward:kiln", &marker(d, id))
}

fn win_once_marker(gs: &mut GameState, id: &str) {
    gs.set_flag(&explore::done_flag("ward:kiln", id), json!(true));
}

fn pay_objective(gs: &mut GameState, encounter_qid: &str) {
    gs.reward_paid.insert(encounter_qid.into(), json!(["objective"]));
}

#[test]
fn a_jobs_fight_is_offered_only_once_the_job_is_accepted() {
    let d = data();
    let mut gs = state(&d);
    assert_eq!(quest::status(&d, &gs, "ward:raid"), "available");
    assert!(quest::fight_waits_for_job(&d, &gs, "ward:enc_mill"));
    assert_eq!(presence(&d, &gs, "mill"), "absent", "the job's fight is hidden before the job is taken");
    assert!(quest::start(&d, &mut gs, "ward:raid").is_empty());
    assert!(!quest::fight_waits_for_job(&d, &gs, "ward:enc_mill"));
    assert_eq!(presence(&d, &gs, "mill"), "present", "taking the job offers its fight");
}

#[test]
fn a_fight_no_job_names_is_offered_and_a_non_board_quest_does_not_gate() {
    let d = data();
    let gs = state(&d);
    assert!(!quest::fight_waits_for_job(&d, &gs, "ward:enc_free"));
    assert_eq!(presence(&d, &gs, "free"), "present");
    // `plain` has no board: its fight is not a job's fight
    assert_eq!(quest::status(&d, &gs, "ward:plain"), "available");
    assert_eq!(presence(&d, &gs, "side"), "present");
    assert!(!quest::fight_waits_for_job(&d, &gs, ""));
}

#[test]
fn a_fight_shared_by_two_jobs_is_offered_once_either_is_accepted() {
    let d = data_two_step();
    let mut gs = state(&d);
    assert_eq!(presence(&d, &gs, "mill"), "absent");
    assert!(quest::start(&d, &mut gs, "ward:two_step").is_empty());
    assert_eq!(quest::status(&d, &gs, "ward:raid"), "available");
    assert_eq!(presence(&d, &gs, "mill"), "present");
}

#[test]
fn accepting_a_job_whose_fight_was_won_marks_it_ready_to_turn_in() {
    let d = data();
    // evidence 1: the fight's paid once objective reward
    let mut gs = state(&d);
    pay_objective(&mut gs, "ward:enc_mill");
    assert!(quest::encounter_won(&d, &gs, "ward:enc_mill"));
    assert!(quest::start(&d, &mut gs, "ward:raid").is_empty());
    assert_eq!(quest::status(&d, &gs, "ward:raid"), "ready");
    assert!(gs.flag_true(&quest::ready_flag("ward:raid")));
    // evidence 2: the done flag of a once marker placed with the fight
    let mut gs = state(&d);
    win_once_marker(&mut gs, "side");
    assert!(quest::encounter_won(&d, &gs, "ward:enc_side"));
    assert!(quest::start(&d, &mut gs, "ward:plain").is_empty());
    assert_eq!(quest::status(&d, &gs, "ward:plain"), "ready");
}

#[test]
fn accepting_a_job_whose_fight_was_not_won_leaves_it_active() {
    let d = data();
    let mut gs = state(&d);
    assert!(!quest::encounter_won(&d, &gs, "ward:enc_mill"));
    // a repeatable marker leaves no done flag: winning it again is the way, and a paid reward of another fight is not it
    pay_objective(&mut gs, "ward:enc_free");
    assert!(quest::start(&d, &mut gs, "ward:raid").is_empty());
    assert_eq!(quest::status(&d, &gs, "ward:raid"), "active");
    assert_eq!(json::s(&quest::current_objective(&d, &gs, "ward:raid"), "target"), "ward:enc_mill");
}

#[test]
fn accepting_a_two_fight_job_with_the_first_won_moves_to_the_second() {
    let d = data_two_step();
    let mut gs = state(&d);
    pay_objective(&mut gs, "ward:enc_mill");
    assert!(quest::start(&d, &mut gs, "ward:two_step").is_empty());
    assert_eq!(quest::status(&d, &gs, "ward:two_step"), "active");
    assert_eq!(json::s(&quest::current_objective(&d, &gs, "ward:two_step"), "target"), "ward:enc_far");
}

fn save_text(gs: &GameState) -> String {
    json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": gs.to_value()}).to_string()
}

#[test]
fn a_save_stuck_on_a_won_fight_is_repaired_on_load() {
    let d = data();
    // the stuck shape: the job active at its fight although that fight was won (won before the job gate existed)
    let mut gs = state(&d);
    gs.quests.insert("ward:raid".into(), json!({"state": "active", "objective": 0}));
    pay_objective(&mut gs, "ward:enc_mill");
    let loaded = save::decode(&d, &save_text(&gs)).expect("loads");
    assert_eq!(quest::status(&d, &loaded.state, "ward:raid"), "ready");
    assert!(loaded.state.flag_true(&quest::ready_flag("ward:raid")));
    assert_eq!(loaded.warnings, ["Raid: its fight was already won, so it is ready to turn in."]);
    // a sound save loads unchanged, with no warning
    let mut sound = state(&d);
    sound.quests.insert("ward:raid".into(), json!({"state": "active", "objective": 0}));
    let back = save::decode(&d, &save_text(&sound)).expect("loads");
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert_eq!(back.state, sound);
}
