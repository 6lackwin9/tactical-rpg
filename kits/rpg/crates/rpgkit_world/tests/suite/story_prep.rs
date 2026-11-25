//! PR6-10 (`docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` 3) on a scratch world built here (no shipped data): the story step
//! `camp_prep` (its free rest heals with no clock tick, once; going on moves the story), a fight step's
//! `retry_day_cost` (each retry passes its days through the campaign tick, which can end the run) and `echo_fallen`
//! (a retry fields up to `ECHO_MAX` echo units of departed members, the latest first, never roster enemies), and the
//! `story.running` save keys `tries` and `rested` (written only when set). Expectations follow from the rules and the
//! fixture's own numbers.

use aegis_core::json::{self, Dict};
use rpgkit_data::{echo_ref, Step, ECHO_MAX, ECHO_NAME_PREFIX};
use rpgkit_world::campaign::{self, Cause};
use rpgkit_world::state::{Departed, PartyState, UnitState};
use rpgkit_world::story::{self, Running, Story};
use rpgkit_world::{battle, Data, GameState};
use serde_json::{json, Value};

/// The prep's retry cost (days) in the fixture.
const COST: i64 = 2;
const HURT: i64 = 5;

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

/// World `w` (a campaign: `days` counts day inputs; `clock` a hard countdown) with sequence `dream` (a camp prep with a
/// keeper, a fight that retries at `COST` days with echoes, an end) and sequence `plain` (a fight that retries with no
/// new field).
fn data() -> Data {
    let band = json!([{"id": "low", "name": "Low", "from": 0, "tone": "calm"}]);
    let clock = json!({"id": "clock", "name": "Clock", "kind": "countdown", "min": 0, "max": 99, "start": 0, "unit": "days",
        "bands": [{"id": "due", "name": "Due", "from": 0, "tone": "danger"}],
        "rules": [{"id": "zero", "at": 0, "effects": [{"effect": "end", "kind": "fail"}]}]});
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["lead"], "campaign": {"start_act": 1}}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub"}])),
        "characters": with_ids("w", json!([
            {"id": "lead", "name": "Lead", "role": "party", "base_unit": "hero"},
            {"id": "smith", "name": "Smith", "role": "npc"},
            {"id": "rat", "name": "Rat", "role": "npc", "base_unit": "rat"}
        ])),
        "encounters": with_ids("w", json!([{"id": "core", "name": "Core", "enemies": ["rat"]}])),
        "meters": with_ids("w", json!([{"id": "days", "name": "Days", "max": 999, "bands": band, "inputs": [{"on": "day", "delta": 1}]}, clock])),
        "sequences": with_ids("w", json!([
            {"id": "dream", "name": "Dream", "steps": [
                {"kind": "camp_prep", "keeper": "smith"},
                {"kind": "fight", "encounter": "core", "on_lost": "retry", "on_fled": "retry", "retry_day_cost": COST, "echo_fallen": true},
                {"kind": "end"}
            ]},
            {"id": "plain", "name": "Plain", "steps": [{"kind": "fight", "encounter": "core", "on_lost": "retry"}, {"kind": "end"}]}
        ]))
    }}});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 3},
            {"id": "rat", "name": "Rat", "side": "enemy", "class_id": "drill", "level": 1}
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn gone(id: &str, life: &str) -> Departed {
    let mut member = UnitState::create(id, "hero");
    member.name = id.to_string();
    Departed { member, life: life.into(), cause: "downed".into(), ..Departed::default() }
}

/// Lead active at the hub, `departed` added (oldest first), the campaign synced.
fn state(d: &Data, departed: Vec<Departed>) -> GameState {
    let members = vec![UnitState::create("lead", "hero")];
    let mut gs = GameState::create(d, "w", PartyState { members, active_ids: vec!["lead".into()], departed, ..PartyState::default() });
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

fn ok(r: &Dict) -> bool {
    json::b(r, "ok", false)
}

fn days(gs: &GameState) -> i64 {
    gs.campaign.as_ref().unwrap().value("days", gs.time)
}

fn running(gs: &GameState) -> Running {
    gs.story.running.clone().expect("a sequence runs")
}

#[test]
fn a_prep_step_rests_once_with_no_clock_tick_then_goes_on() {
    let d = data();
    let mut gs = state(&d, Vec::new());
    assert!(ok(&story::start(&d, &mut gs, "dream")));
    assert_eq!(story::current(&d, &gs), Some(Step::CampPrep { text: String::new(), keeper: "w:smith".into() }), "the keeper as a qid");
    assert!(story::fight(&d, &gs).is_none());
    gs.party.members[0].hp_lost = HURT;
    let (time, day_count) = (gs.time, days(&gs));
    assert!(story::prep_rest_open(&d, &gs));
    let r = story::prep_rest(&d, &mut gs);
    assert!(ok(&r), "{r:?}");
    assert_eq!(json::sget(&r, "notes"), [story::PREP_REST_TEXT]);
    assert_eq!(gs.party.members[0].hp_lost, 0, "every member back to full");
    assert_eq!((gs.time, days(&gs)), (time, day_count), "no time passes, no tick");
    assert!(running(&gs).rested && !story::prep_rest_open(&d, &gs));
    let again = story::prep_rest(&d, &mut gs);
    assert_eq!(json::sget(&again, "errors"), [story::PREP_RESTED_TEXT], "once per prep step");
    // the taken rest is saved and read back
    let saved = gs.story.to_value();
    assert_eq!(saved["running"]["rested"], json!(true));
    let mut problems = Vec::new();
    assert_eq!(Story::from_value(&saved, &mut problems), gs.story);
    assert!(problems.is_empty());

    let r = story::prep_done(&d, &mut gs);
    assert!(ok(&r), "{r:?}");
    assert!(matches!(story::current(&d, &gs), Some(Step::Fight { retry_day_cost: 2, echo_fallen: true, .. })));
    assert!(!running(&gs).rested, "the next step starts fresh");
    assert_eq!(gs.time, time, "going on passes no time");
    // the prep calls are refused at any other step
    assert_eq!(json::sget(&story::prep_rest(&d, &mut gs), "errors"), [story::NOT_PREP_TEXT]);
    assert_eq!(json::sget(&story::prep_done(&d, &mut gs), "errors"), [story::NOT_PREP_TEXT]);
    assert!(!story::prep_rest_open(&d, &gs));
}

#[test]
fn a_retry_passes_its_days_counts_the_tries_and_a_win_clears_them() {
    let d = data();
    let mut gs = state(&d, Vec::new());
    story::start(&d, &mut gs, "dream");
    story::prep_done(&d, &mut gs);
    let fight = story::fight(&d, &gs).expect("at the fight");
    assert!(!fight.contains_key(story::ECHOES_KEY), "the first try fields no echo");
    let (time, day_count) = (gs.time, days(&gs));
    gs.party.members[0].hp_lost = HURT;
    let r = story::after_fight(&d, &mut gs, &fight, "lost", None);
    assert!(ok(&r), "{r:?}");
    assert_eq!(json::sget(&r, "notes"), [story::RETRY_TEXT.to_string(), story::retry_days_text(COST)]);
    assert_eq!((gs.time, days(&gs)), (time + COST, day_count + COST), "the retry's days pass through the day tick");
    assert_eq!(gs.party.members[0].hp_lost, 0, "the retry heals");
    assert_eq!(running(&gs).tries, 1);
    assert_eq!(gs.story.to_value()["running"]["tries"], json!(1), "tries saved once set");
    story::after_fight(&d, &mut gs, &fight, "fled", None);
    assert_eq!((running(&gs).tries, gs.time), (2, time + 2 * COST), "a flight retries at the same cost");
    story::after_fight(&d, &mut gs, &fight, "won", None);
    assert_eq!(story::current(&d, &gs), Some(Step::End { to: "map".into() }));
    assert_eq!(running(&gs).tries, 0, "a won fight moves on with no tries");
    assert_eq!(gs.time, time + 2 * COST, "a win costs nothing");
    assert_eq!(story::retry_days_text(1), "1 day passes.");
}

#[test]
fn a_retry_whose_days_empty_the_clock_ends_the_run() {
    let d = data();
    let mut gs = state(&d, Vec::new());
    let mut events = Vec::new();
    campaign::apply_effect(&d, &mut gs, json::dict(&json!({"effect": "clock_start", "meter": "clock", "days": COST})), &mut events);
    campaign::tick(&d, &mut gs, &Cause::Effects);
    assert!(campaign::save_refusal(&gs).is_empty(), "the run goes on before the retry");
    story::start(&d, &mut gs, "dream");
    story::prep_done(&d, &mut gs);
    let fight = story::fight(&d, &gs).expect("at the fight");
    story::after_fight(&d, &mut gs, &fight, "lost", None);
    assert!(!campaign::save_refusal(&gs).is_empty(), "the retry's days run the clock out: the run is over");
}

#[test]
fn a_retried_echo_fight_fields_the_latest_fallen_as_echo_units() {
    let d = data();
    let fallen = vec![gone("oldest", "dead"), gone("benched", "retired"), gone("taken", "captured"), gone("missing", "lost")];
    let mut gs = state(&d, fallen);
    story::start(&d, &mut gs, "dream");
    story::prep_done(&d, &mut gs);
    let first = story::fight(&d, &gs).expect("at the fight");
    let f = battle::begin_world(&d, &mut gs, &first).expect("starts");
    assert_eq!(f.session.w.enemies.len(), 1, "no echo before a retry");
    story::after_fight(&d, &mut gs, &first, "lost", None);
    let fight = story::fight(&d, &gs).expect("still at the fight");
    assert_eq!(json::i(&fight, story::ECHOES_KEY, 0), ECHO_MAX as i64, "three qualify (retired does not): at most ECHO_MAX");
    assert_eq!(story::echo_count(&gs), ECHO_MAX);
    let f = battle::begin_world(&d, &mut gs, &fight).expect("starts with echoes");
    assert_eq!(f.session.w.enemies.len(), 1 + ECHO_MAX, "the encounter's enemy, then the echoes");
    let names: Vec<String> = (1..=ECHO_MAX).map(|i| f.session.w.units[f.session.units_by_ref[&echo_ref(i)][0]].name.clone()).collect();
    assert_eq!(names, [format!("{ECHO_NAME_PREFIX}missing"), format!("{ECHO_NAME_PREFIX}taken")], "the latest departed first");
    let (_, notes) = f.tally().fates_of(&battle::roster(&d, "w", "w:core"));
    assert!(notes.is_empty(), "echo units are never roster enemies: {notes:?}");
    // the echoes carry over: the next retry fields them again
    story::after_fight(&d, &mut gs, &fight, "lost", None);
    assert_eq!(json::i(&story::fight(&d, &gs).unwrap(), story::ECHOES_KEY, 0), ECHO_MAX as i64);
    // one fallen fields one echo
    let mut gs = state(&d, vec![gone("only", "dead")]);
    story::start(&d, &mut gs, "dream");
    story::prep_done(&d, &mut gs);
    let fight = story::fight(&d, &gs).unwrap();
    story::after_fight(&d, &mut gs, &fight, "lost", None);
    let fight = story::fight(&d, &gs).unwrap();
    assert_eq!(json::i(&fight, story::ECHOES_KEY, 0), 1);
    assert_eq!(battle::begin_world(&d, &mut gs, &fight).expect("starts").session.w.enemies.len(), 2);
}

#[test]
fn a_fight_step_without_the_new_fields_and_its_save_are_as_before() {
    let d = data();
    let mut gs = state(&d, vec![gone("fallen", "dead")]);
    story::start(&d, &mut gs, "plain");
    assert!(matches!(story::current(&d, &gs), Some(Step::Fight { retry_day_cost: 0, echo_fallen: false, .. })));
    let saved = gs.story.to_value();
    assert_eq!(saved, json!({"running": {"sequence": "w:plain", "step": 0}}), "no new key while unset");
    let fight = story::fight(&d, &gs).unwrap();
    let time = gs.time;
    let r = story::after_fight(&d, &mut gs, &fight, "lost", None);
    assert_eq!(json::sget(&r, "notes"), [story::RETRY_TEXT], "no day line");
    assert_eq!(gs.time, time, "no days pass");
    assert_eq!(gs.story.to_value(), saved, "a plain retry saves the old shape (no tries)");
    let fight = story::fight(&d, &gs).unwrap();
    assert_eq!(fight, json::obj([("source", story::FIGHT_SOURCE.into()), ("encounter", "w:core".into())]), "no echo without echo_fallen");
    // a damaged running block is a load problem
    let mut problems = Vec::new();
    Story::from_value(&json!({"running": {"sequence": "w:plain", "step": 0, "tries": "1"}}), &mut problems);
    Story::from_value(&json!({"running": {"sequence": "w:plain", "step": 0, "rested": 1}}), &mut problems);
    assert_eq!(problems.len(), 2, "{problems:?}");
    let bad = Story { running: Some(Running { sequence: "w:plain".into(), step: 0, tries: -1, rested: false }), ..Story::default() };
    assert_eq!(bad.validate(), ["story.running.tries must be >= 0."]);
}
