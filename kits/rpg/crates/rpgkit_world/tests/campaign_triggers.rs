//! CAMPAIGN1-T on scratch worlds (CAMPAIGN1.md T1-T3, F2, W3, X4, R2): triggers fire after their cause (battle with
//! its encounter and outcome, enter with its location, day, act, any) when `when` holds, once or again after `when`
//! stopped holding; their effects run through the shared effect body and their `then` items queue in `pending` (saved);
//! a queued battle and a queued scene are played through `trigger::fight` / `after_fight` and
//! `DialogueRunner::open_scene`; an act set inside a tick runs the act triggers in the same action; the previews list
//! the end a battle or some days would fire without touching the state; a rest inside a tick (a missed `go_on`
//! checkpoint's effects) ticks its days in the same action. A world without `campaign` runs no trigger. Every
//! expectation follows from the rule and the fixture's own numbers.

use aegis_core::json::{self, Dict};
use rpgkit_world::campaign::{self, trigger, Cause};
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{battle, explore, save, travel, Data, GameState};
use serde_json::{json, Value};

const BREACH: i64 = 100;
const LOUD_WON: i64 = 8;
const LOUD_LOST: i64 = 4;
const DEADLINE: i64 = 5;
const CLOCK_DAYS: i64 = 42;

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

/// `alert`: a gauge whose Breach (100) ends the run in act 1 and act 2, moved by loud battles (won +8, lost or fled +4);
/// `days`: +1 per day passed; `clock`: a countdown.
fn meters() -> Value {
    json!([
        {"id": "alert", "name": "Alert", "max": BREACH,
         "bands": [{"id": "guarded", "name": "Guarded", "from": 0, "tone": "calm"}, {"id": "breach", "name": "Breach", "from": BREACH, "tone": "fatal"}],
         "rules": [{"id": "a_breach", "at": "breach", "acts": [1, 2], "effects": [{"effect": "end", "kind": "fail", "card": "c_breach", "text": "Found."}]}],
         "inputs": [{"on": "battle", "tags": ["loud"], "outcomes": ["won"], "delta": LOUD_WON}, {"on": "battle", "tags": ["loud"], "outcomes": ["lost", "fled"], "delta": LOUD_LOST}]},
        {"id": "days", "name": "Days", "max": 999, "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}], "inputs": [{"on": "day", "delta": 1}]},
        {"id": "clock", "name": "The Clock", "kind": "countdown", "min": 0, "max": 99, "unit": "days", "bands": [{"id": "due", "name": "Due", "from": 0, "tone": "danger"}]}
    ])
}

/// World `w` (a campaign from act 1 unless `campaign` is false; a rest costs 1 day and nothing else): the hub (a rest
/// place), `a` and `b` on a line of 1-day routes, the encounters `enc_x` and `enc_loud` (tagged loud), Clerk and the
/// dialogue `scene`, the meters above, and the given trigger and checkpoint rows.
fn data(campaign: bool, triggers: Value, checkpoints: Value) -> Data {
    let w = "w";
    let mut manifest = json!({"id": w, "start_location": "hub", "start_party": ["hero"],
                              "rest": {"days": 1, "cost": 0, "where": "rest_places", "on_enter": false, "needs_upkeep": false}});
    if campaign {
        manifest["campaign"] = json!({"start_act": 1});
    }
    let worlds = json!({
        w: {"manifest": manifest, "own": {
            "locations": with_ids(w, json!([{"id": "hub", "name": "Hub", "rest": true}, {"id": "a", "name": "A"}, {"id": "b", "name": "B"}])),
            "routes": with_ids(w, json!([{"id": "r_a", "from": "hub", "to": "a", "time": 1}, {"id": "r_b", "from": "a", "to": "b", "time": 1}])),
            "encounters": with_ids(w, json!([{"id": "enc_x", "name": "X", "enemies": []}, {"id": "enc_loud", "name": "Loud", "tags": ["loud"], "enemies": []}])),
            "characters": with_ids(w, json!([{"id": "clerk", "name": "Clerk", "role": "npc"}])),
            "dialogues": with_ids(w, json!([{"id": "scene", "start": "n", "nodes": [{"id": "n", "text": "A scene.", "end": true}]}])),
            "meters": with_ids(w, meters()),
            "checkpoints": with_ids(w, checkpoints),
            "triggers": with_ids(w, triggers)
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A new game at the hub with every node visited (so every trip is known).
fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    gs.party.supplies = 10;
    for n in ["hub", "a", "b"] {
        gs.set_flag(&explore::visited_flag(&format!("w:{n}")), json!(true));
    }
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

fn value(gs: &GameState, id: &str) -> i64 {
    gs.campaign.as_ref().unwrap().value(id, gs.time)
}

fn set_alert(gs: &mut GameState, v: i64) {
    let c = gs.campaign.as_mut().unwrap();
    c.meters.get_mut("alert").unwrap().value = v;
    c.sync(gs.time);
}

fn dicts(events: &[Value]) -> Vec<Dict> {
    events.iter().map(|e| json::dict(e).clone()).collect()
}

fn campaign_events(res: &Dict) -> Vec<Dict> {
    dicts(json::aget(res, "campaign"))
}

/// `key` of each event, as text ("" when absent).
fn field(events: &[Dict], key: &str) -> Vec<String> {
    events.iter().map(|e| json::gstr(e.get(key).unwrap_or(&Value::Null))).collect()
}

fn fired(events: &[Dict]) -> Vec<String> {
    events.iter().filter(|e| json::s(e, "event") == "trigger_fired").map(|e| json::s(e, "trigger")).collect()
}

fn pending(gs: &GameState) -> Vec<Dict> {
    gs.campaign.as_ref().unwrap().pending.clone()
}

fn day_tick(d: &Data, gs: &mut GameState) -> Vec<Dict> {
    gs.time += 1;
    dicts(&campaign::tick(d, gs, &Cause::Days { n: 1, why: "travel".into() }))
}

// ---- T1, T2 ----

#[test]
fn a_battle_trigger_runs_its_effects_and_queues_its_then_items() {
    let then = json!([{"vignette": "v_after"}]);
    let d = data(true, json!([{"id": "t_won", "name": "After X", "on": "battle", "encounter": "enc_x", "outcome": "won",
                                "effects": [{"effect": "set_flag", "key": "x_won"}], "then": then}]), json!([]));
    let mut gs = state(&d);
    let lost = battle::settle(&d, &mut gs, "w:enc_x", "lost", None);
    assert!(fired(&campaign_events(&lost)).is_empty() && !gs.flag_true("x_won"), "T1: the outcome filter");
    let other = battle::settle(&d, &mut gs, "w:enc_loud", "won", None);
    assert!(fired(&campaign_events(&other)).is_empty(), "T1: the encounter filter");
    let won = battle::settle(&d, &mut gs, "w:enc_x", "won", None);
    assert_eq!(fired(&campaign_events(&won)), ["t_won"]);
    assert!(gs.flag_true("x_won"), "T2: effects at once");
    let queued: Vec<Dict> = json::arr(&then).iter().map(|i| json::dict(i).clone()).collect();
    assert_eq!(pending(&gs), queued, "T2: then items queued");
    let again = battle::settle(&d, &mut gs, "w:enc_x", "won", None);
    assert!(fired(&campaign_events(&again)).is_empty() && pending(&gs) == queued, "once by default");
    // the queue and the fired record are saved (11.5)
    let dir = std::env::temp_dir().join(format!("rpgkit_campaign_triggers_{}", std::process::id()));
    assert!(save::write(&d, &dir, &gs, 1, 1).is_empty());
    let back = save::read(&d, &dir, 1).expect("reads back");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert_eq!(back.state, gs);
    assert!(back.state.campaign.as_ref().unwrap().fired.contains(&"trigger:t_won".to_string()));
}

#[test]
fn a_once_trigger_fires_once_and_a_repeatable_one_again_after_its_when_stopped_holding() {
    let hot = json!([{"key": "hot", "value": true}]);
    let d = data(
        true,
        json!([{"id": "t_once", "on": "day", "when": hot, "effects": [{"effect": "add_flag", "key": "once_n", "delta": 1}]},
               {"id": "t_again", "on": "day", "once": false, "when": hot, "effects": [{"effect": "add_flag", "key": "again_n", "delta": 1}]}]),
        json!([]),
    );
    let mut gs = state(&d);
    let counts = |gs: &GameState| (gs.flag("once_n").cloned().unwrap_or(json!(0)), gs.flag("again_n").cloned().unwrap_or(json!(0)));
    day_tick(&d, &mut gs);
    assert_eq!(counts(&gs), (json!(0), json!(0)), "when does not hold");
    gs.set_flag("hot", json!(true));
    day_tick(&d, &mut gs);
    assert_eq!(counts(&gs), (json!(1), json!(1)), "both fire when it holds");
    day_tick(&d, &mut gs);
    assert_eq!(counts(&gs), (json!(1), json!(1)), "still holding: neither fires again");
    assert_eq!(gs.campaign.as_ref().unwrap().armed, ["t_again"]);
    gs.set_flag("hot", json!(false));
    day_tick(&d, &mut gs);
    assert!(gs.campaign.as_ref().unwrap().armed.is_empty(), "the repeatable one re-arms when when stops holding");
    gs.set_flag("hot", json!(true));
    day_tick(&d, &mut gs);
    assert_eq!(counts(&gs), (json!(1), json!(2)), "T1: once fires once; repeatable fires twice");
    // an `on: day` trigger is not checked after another cause
    gs.set_flag("hot", json!(false));
    day_tick(&d, &mut gs);
    gs.set_flag("hot", json!(true));
    campaign::tick(&d, &mut gs, &Cause::Effects);
    assert_eq!(counts(&gs), (json!(1), json!(2)));
}

#[test]
fn a_wiped_cell_ends_the_run() {
    let d = data(
        true,
        json!([{"id": "t_wiped", "on": "any", "when": [{"key": "roster.living", "op": "<", "value": 1}],
                "effects": [{"effect": "end", "kind": "fail", "card": "c_wiped"}, {"effect": "set_flag", "key": "after_end"}]}]),
        json!([]),
    );
    let mut gs = state(&d);
    assert!(campaign::tick(&d, &mut gs, &Cause::Effects).is_empty(), "a living member: nothing");
    gs.party.members.clear();
    let events = dicts(&campaign::tick(&d, &mut gs, &Cause::Effects));
    assert_eq!(field(&events, "event"), ["trigger_fired", "run_over"]);
    assert_eq!(json::s(&events[1], "card"), "c_wiped", "F2");
    assert!(!gs.flag_true("after_end"), "F1: the list stops at the end");
    assert_eq!(campaign::save_refusal(&gs), campaign::RUN_OVER_TEXT, "F3");
}

#[test]
fn an_enter_trigger_fires_on_arrival_and_its_act_change_runs_the_act_triggers() {
    let d = data(
        true,
        json!([{"id": "t_act", "on": "enter", "location": "a", "when": [{"key": "prologue_done"}],
                "effects": [{"effect": "set_act", "act": 2}, {"effect": "meter_show", "meter": "alert"}, {"effect": "clock_start", "meter": "clock", "days": CLOCK_DAYS}],
                "then": [{"vignette": "v_act"}]},
               {"id": "t_act_seen", "on": "act", "when": [{"key": "campaign.act", "value": 2}], "effects": [{"effect": "set_flag", "key": "act_seen"}]}]),
        json!([]),
    );
    let mut gs = state(&d);
    let res = travel::travel(&d, &mut gs, "a");
    assert_eq!(json::s(&res, "status"), "arrived");
    assert!(fired(&campaign_events(&res)).is_empty(), "when does not hold yet");
    travel::travel(&d, &mut gs, "hub");
    gs.set_flag("prologue_done", json!(true));
    let res = travel::travel(&d, &mut gs, "a");
    assert_eq!(json::s(&res, "status"), "arrived", "{res:?}");
    let events = campaign_events(&res);
    assert_eq!(field(&events, "event"), ["trigger_fired", "act_start", "trigger_fired"], "{events:?}");
    assert_eq!(fired(&events), ["t_act", "t_act_seen"], "the act tick follows in the same action");
    let c = gs.campaign.as_ref().unwrap();
    assert_eq!(c.act, 2);
    assert!(c.meters["alert"].visible && gs.flag_true("act_seen"));
    assert_eq!(value(&gs, "clock"), CLOCK_DAYS);
    assert_eq!(pending(&gs), vec![json::obj([("vignette", "v_act".into())])]);
    // an explore entry ticks the same way (the location filter: the hub fires nothing)
    let entered = explore::enter(&d, &mut gs, "hub", explore::SPAWN);
    assert!(!entered.contains_key("campaign"), "{entered:?}");
}

#[test]
fn an_enter_trigger_that_ends_the_run_stops_the_trip_without_an_autosave() {
    let d = data(true, json!([{"id": "t_trap", "on": "enter", "location": "a", "effects": [{"effect": "end", "kind": "fail", "card": "c_trap"}]}]), json!([]));
    let mut gs = state(&d);
    let res = travel::travel(&d, &mut gs, "b");
    assert_eq!((json::s(&res, "status"), json::i(&res, "legs_done", -1)), ("over".into(), 1), "{res:?}");
    assert_eq!(gs.location_id, "w:a");
    assert_eq!(field(&campaign_events(&res), "event"), ["trigger_fired", "run_over"]);
    assert!(!json::aget(&res, "events").iter().any(|e| json::s(json::dict(e), "kind") == "autosave"), "F3");
}

// ---- the queue (T2) and the shared effect body ----

#[test]
fn queued_battles_and_scenes_are_played_through_the_kit() {
    let d = data(
        true,
        json!([{"id": "t_x", "on": "battle", "encounter": "enc_x", "outcome": "won", "effects": [{"effect": "give", "gold": 5}, {"effect": "rest"}],
                "then": [{"battle": "enc_loud"}, {"dialogue": "scene", "speaker": "clerk"}]}]),
        json!([]),
    );
    let mut gs = state(&d);
    let (gold, time, days) = (gs.party.gold, gs.time, value(&gs, "days"));
    let res = battle::settle(&d, &mut gs, "w:enc_x", "won", None);
    let ev = campaign_events(&res);
    assert_eq!(fired(&ev), ["t_x"]);
    assert_eq!(gs.party.gold, gold + json::i(&res, "gold", 0) + 5, "the give effect");
    let notes = json::sget(&ev[0], "notes");
    assert!(notes.len() == 1 && notes[0].starts_with("The party rests"), "the rest's note reaches the log: {ev:?}");
    let rest_days = 1;
    assert_eq!((gs.time, value(&gs, "days")), (time + rest_days, days + rest_days), "R2: the rest's day ticked in the same action");
    let fight = trigger::fight(&d, &gs).expect("a battle at the head");
    assert_eq!((json::s(&fight, "source"), json::s(&fight, "encounter")), (trigger::FIGHT_SOURCE.to_string(), "w:enc_loud".to_string()));
    campaign::pop_pending(&mut gs);
    let alert = value(&gs, "alert");
    let settled = trigger::after_fight(&d, &mut gs, &fight, "won", None);
    assert!(json::b(&settled, "ok", false), "{settled:?}");
    assert_eq!(value(&gs, "alert"), alert + LOUD_WON, "settled like a dialogue fight: the battle tick ran");
    assert!(trigger::fight(&d, &gs).is_none(), "the head is the scene now");
    let head = campaign::pop_pending(&mut gs).unwrap();
    let mut talk = DialogueRunner::open_scene(&d, &mut gs, &json::s(&head, "dialogue"), &json::s(&head, "speaker"));
    let view = talk.view(&d, &gs);
    assert_eq!((json::s(&view, "text"), json::s(&view, "speaker")), ("A scene.".to_string(), "w:clerk".to_string()), "{view:?}");
    assert!(json::aget(&view, "choices").is_empty(), "a scene offers no services");
    assert!(json::b(&talk.advance(&d, &mut gs), "ok", false));
    let bad = DialogueRunner::open_scene(&d, &mut gs, "nope", "");
    assert!(bad.closed && bad.errors == ["Unknown dialogue 'nope'."], "{:?}", bad.errors);
}

#[test]
fn every_effect_list_stops_at_the_first_end() {
    let d = data(true, json!([]), json!([]));
    let mut gs = state(&d);
    let list = vec![json::obj([("effect", "end".into()), ("kind", "fail".into()), ("card", "c".into())]), json::obj([("effect", "set_flag".into()), ("key", "late".into())])];
    let (mut notes, mut events) = (Vec::new(), Vec::new());
    rpgkit_world::dialogue::apply_effects(&d, &mut gs, &list, &mut notes, &mut events);
    assert!(gs.campaign.as_ref().unwrap().is_over() && !gs.flag_true("late"));
    // a classic world has no run to end: the whole list runs
    let d = data(false, json!([]), json!([]));
    let mut gs = state(&d);
    rpgkit_world::dialogue::apply_effects(&d, &mut gs, &list, &mut notes, &mut events);
    assert!(gs.flag_true("late"));
}

// ---- previews (W3) ----

#[test]
fn preview_battle_lists_the_end_a_loud_fight_would_fire() {
    let d = data(true, json!([]), json!([]));
    let mut gs = state(&d);
    let high = 95;
    assert!(high + LOUD_WON >= BREACH && high + LOUD_LOST < BREACH, "the fixture: only a win crosses Breach");
    set_alert(&mut gs, high);
    let before = gs.clone();
    let ends = dicts(&campaign::preview_battle(&d, &gs, "enc_loud"));
    assert_eq!(field(&ends, "case"), ["won"]);
    assert_eq!((json::s(&ends[0], "kind"), json::s(&ends[0], "card")), ("fail".into(), "c_breach".into()));
    assert_eq!(gs, before, "the state is untouched");
    assert!(!gs.campaign.as_ref().unwrap().is_over());
    set_alert(&mut gs, 50);
    assert!(campaign::preview_battle(&d, &gs, "enc_loud").is_empty());
    assert!(campaign::preview_battle(&d, &gs, "enc_x").is_empty(), "a quiet fight moves nothing");
}

#[test]
fn preview_days_lists_a_missed_end_run_deadline() {
    let d = data(true, json!([]), json!([{"id": "cp", "name": "The Job", "act": 1, "met_when": [{"key": "job_done"}], "deadline": DEADLINE, "mode": "end_run", "card": "bad_end_cp"}]));
    let gs = state(&d);
    let day = gs.time + 1;
    assert!(campaign::preview_days(&d, &gs, DEADLINE - day).is_empty(), "arriving on the deadline day is in time");
    let ends = dicts(&campaign::preview_days(&d, &gs, DEADLINE - day + 1));
    assert_eq!((field(&ends, "case"), field(&ends, "kind"), field(&ends, "card")), (vec!["days".to_string()], vec!["bad_end".to_string()], vec!["bad_end_cp".to_string()]));
    assert!(gs.campaign.as_ref().unwrap().checkpoints.is_empty(), "the state is untouched");
    let classic = data(false, json!([]), json!([]));
    let gs = state(&classic);
    assert!(campaign::preview_days(&classic, &gs, 99).is_empty() && campaign::preview_battle(&classic, &gs, "enc_loud").is_empty());
}

// ---- a rest inside a tick (CAMPAIGN1-C edge case) ----

#[test]
fn a_missed_go_on_checkpoint_that_rests_ticks_its_days_in_the_same_action() {
    let never = json!([{"key": "never"}]);
    let d = data(
        true,
        json!([]),
        json!([{"id": "cp_rest", "act": 1, "met_when": never, "deadline": DEADLINE, "mode": "go_on", "effects": [{"effect": "rest"}]},
               {"id": "cp_late", "act": 1, "met_when": never, "deadline": DEADLINE + 3, "warn_days": [1], "mode": "go_on", "effects": [{"effect": "set_flag", "key": "late"}]}]),
    );
    let mut gs = state(&d);
    gs.time = DEADLINE - 1;
    gs.campaign.as_mut().unwrap().sync(gs.time);
    let rest_days = 1;
    let res = travel::travel(&d, &mut gs, "a");
    assert_eq!(json::s(&res, "status"), "arrived", "{res:?}");
    assert_eq!(gs.time, DEADLINE + rest_days, "the leg's day and the rest's day passed");
    assert_eq!(value(&gs, "days"), 1 + rest_days, "M3: the day input ran for the rest's day in the same action");
    let events = campaign_events(&res);
    assert_eq!(field(&events, "event"), ["checkpoint_missed", "checkpoint_warning"], "{events:?}");
    assert_eq!((json::s(&events[1], "checkpoint"), json::i(&events[1], "left", -1)), ("cp_late".into(), 1), "C2 across the rest's day");
    assert_eq!(gs.campaign.as_ref().unwrap().owed_days, 0, "nothing is owed to the next cause");
    assert!(campaign::tick(&d, &mut gs, &Cause::Effects).is_empty(), "the next cause runs nothing twice");
    assert_eq!(value(&gs, "days"), 1 + rest_days);
}

// ---- classic worlds ----

#[test]
fn a_world_without_campaign_runs_no_trigger() {
    let d = data(false, json!([{"id": "t", "on": "enter", "effects": [{"effect": "set_flag", "key": "entered"}]}]), json!([]));
    let mut gs = state(&d);
    let res = travel::travel(&d, &mut gs, "a");
    assert!(!res.contains_key("campaign") && !gs.flag_true("entered"), "X1, X3");
    let entered = explore::enter(&d, &mut gs, "hub", explore::SPAWN);
    assert!(!entered.contains_key("campaign"));
    assert!(json::aget(&entered, "events").iter().any(|e| json::s(json::dict(e), "kind") == "autosave"), "the autosave as today");
}
