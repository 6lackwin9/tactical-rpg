//! CAMPAIGN1-C on scratch worlds (CAMPAIGN1.md K1-K5, C1-C3, P2, P3, M7, R1-R4, STORYUI1 B6, B10): days pass only
//! through legs, rests and retreats and tick the campaign once per leg or step; countdowns start, set and stop without
//! false crossings, warn in order and restart or end the run at 0; checkpoints are met, warned and missed once; upkeep
//! is spent per day and never refused; zero upkeep and band modifiers move the road-fight chance and the start AP; the
//! rest rule and the retreat replace Camp and the wake; the board and the world map show the open deadlines. A world
//! without `campaign` keeps today's results. Every expectation follows from the rule and the fixture's own numbers.

use aegis_core::json::{self, Dict};
use rpgkit_world::campaign::{self, checkpoint, purse, Cause};
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{battle, board, explore, progression, quest, travel, Data, GameState};
use serde_json::{json, Value};

const CLOCK_MAX: i64 = 99;
const WARN_AT: i64 = 7;
const RESTART: i64 = 14;
const DEADLINE: i64 = 14;
const LOCKDOWN: i64 = 71;
const LOCKDOWN_PCT: i64 = 150;
const LONG_LEG: i64 = 7;
const HURT: i64 = 5;

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

/// The countdown: a warning at 7; at 0 the `zero` rule (soft: a flag, a vignette, a restart; hard: the end).
fn clock(hard: bool) -> Value {
    let zero = if hard {
        json!({"id": "zero", "at": 0, "effects": [{"effect": "end", "kind": "fail", "card": "card_clock"}]})
    } else {
        json!({"id": "zero", "at": 0, "effects": [{"effect": "add_flag", "key": "lesser", "delta": 1}], "then": [{"vignette": "v_lesser"}], "restart_days": RESTART})
    };
    json!({"id": "clock", "name": "The Clock", "kind": "countdown", "min": 0, "max": CLOCK_MAX, "start": 0, "unit": "days",
           "bands": [{"id": "due", "name": "Due", "from": 0, "tone": "danger"}, {"id": "far", "name": "Far", "from": 8, "tone": "calm"}],
           "rules": [{"id": "week", "at": WARN_AT, "warn": "A week left."}, zero]})
}

/// `days` counts the day inputs, `legs` the leg inputs; `alert` holds a Lockdown band that scales road fights on
/// routes tagged `main` and adds `enc_hunt` to their pool.
fn gauges() -> Vec<Value> {
    let band = json!([{"id": "low", "name": "Low", "from": 0, "tone": "calm"}]);
    vec![
        json!({"id": "days", "name": "Days", "max": 999, "bands": band, "inputs": [{"on": "day", "delta": 1}]}),
        json!({"id": "legs", "name": "Legs", "max": 999, "bands": band, "inputs": [{"on": "leg", "delta": 1}]}),
        json!({"id": "alert", "name": "Alert", "bands": [{"id": "calm", "name": "Calm", "from": 0, "tone": "calm"}, {"id": "lockdown", "name": "Lockdown", "from": LOCKDOWN, "tone": "danger"}],
               "modifiers": [{"bands": ["lockdown"], "road_pct": LOCKDOWN_PCT, "route_tags": ["main"], "road_encounters": [["enc_hunt", 1]]}]}),
    ]
}

/// The world's manifest besides `id` and the start (each test sets the blocks it reads).
#[derive(Clone)]
struct Setup {
    campaign: bool,
    hard: bool,
    blocks: Value,
    checkpoint: Value,
}

impl Default for Setup {
    fn default() -> Self {
        Self { campaign: true, hard: false, blocks: json!({}), checkpoint: json!({}) }
    }
}

/// World `w`: the hub (a rest place) with routes hub-a-b-c (1 day each), hub-far (7 days, tagged `main`) and
/// hub-street (1 day, `street` is no rest place); `ruin` (off the map, its map missing: it cannot be entered); the quest `job` (win `enc_job`) on Clerk's board, node `a`; the
/// checkpoint `cp` on `job` (deadline 14, warnings 7, 3, 1, `end_run`) merged with `setup.checkpoint`.
fn data(setup: &Setup) -> Data {
    let w = "w";
    let mut manifest = json!({"id": w, "start_location": "hub", "start_party": ["hero"]});
    if setup.campaign {
        manifest["campaign"] = json!({"start_act": 1});
    }
    for (k, v) in json::dict(&setup.blocks) {
        manifest[k] = v.clone();
    }
    let mut cp = json!({"id": "cp", "name": "The Job", "act": 1, "quest": "job", "deadline": DEADLINE, "warn_days": [7, 3, 1], "mode": "end_run", "card": "bad_end_job", "text": "Too late."});
    for (k, v) in json::dict(&setup.checkpoint) {
        cp[k] = v.clone();
    }
    let mut meters = vec![clock(setup.hard)];
    meters.extend(gauges());
    let worlds = json!({
        w: {"manifest": manifest, "own": {
            "locations": with_ids(w, json!([{"id": "hub", "name": "Hub", "rest": true}, {"id": "a", "name": "A"}, {"id": "b", "name": "B"}, {"id": "c", "name": "C"},
                                              {"id": "far", "name": "Far"}, {"id": "street", "name": "Street"}, {"id": "ruin", "name": "Ruin", "map": "no_such_map"}])),
            "routes": with_ids(w, json!([
                {"id": "r_a", "from": "hub", "to": "a", "time": 1}, {"id": "r_b", "from": "a", "to": "b", "time": 1}, {"id": "r_c", "from": "b", "to": "c", "time": 1},
                {"id": "r_far", "from": "hub", "to": "far", "time": LONG_LEG, "tags": ["main"], "encounter_pct": 40, "encounters": [["enc_road", 1]]},
                {"id": "r_street", "from": "hub", "to": "street", "time": 1}])),
            "encounters": with_ids(w, json!([{"id": "enc_job", "name": "Job", "location": "a", "enemies": []}, {"id": "enc_road", "name": "Road", "enemies": []},
                                               {"id": "enc_hunt", "name": "Hunt", "enemies": []}])),
            "characters": with_ids(w, json!([{"id": "clerk", "name": "Clerk", "role": "npc", "dialogue": "talk"}])),
            "dialogues": with_ids(w, json!([{"id": "talk", "start": "n", "services": false, "nodes": [{"id": "n", "text": "Rest?", "choices": [{"text": "Rest a day.", "effects": [{"effect": "rest"}]}]}]}])),
            "boards": with_ids(w, json!([{"id": "clerk_board", "name": "Board", "keeper": "clerk"}])),
            "quests": with_ids(w, json!([{"id": "job", "name": "The Job", "board": "clerk_board", "template": "clear_site", "steps": [{"text": "Clear A", "encounter": "enc_job"}]}])),
            "meters": with_ids(w, Value::Array(meters)),
            "checkpoints": with_ids(w, json!([cp]))
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let game = json!({"quests": [{"id": "clear_site", "objectives": [{"kind": "win_encounter"}], "rewards": {}}]});
    Data::from_values(&catalog, &worlds, &game)
}

/// A new game at the hub with every node visited (so every trip is known), `supplies` upkeep.
fn state(d: &Data, supplies: i64) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    gs.party.supplies = supplies;
    for n in ["hub", "a", "b", "c", "far", "street"] {
        gs.set_flag(&explore::visited_flag(&format!("w:{n}")), json!(true));
    }
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

/// Moves the clock to day `day` (time day - 1) as if a tick had ended there.
fn at_day(gs: &mut GameState, day: i64) {
    gs.time = day - 1;
    gs.campaign.as_mut().unwrap().sync(gs.time);
}

fn effect(d: &Data, gs: &mut GameState, fx: Value) -> Vec<Value> {
    let mut events = Vec::new();
    campaign::apply_effect(d, gs, json::dict(&fx), &mut events);
    events.extend(campaign::tick(d, gs, &Cause::Effects));
    events
}

fn value(gs: &GameState, id: &str) -> i64 {
    gs.campaign.as_ref().unwrap().value(id, gs.time)
}

fn events_of(res: &Dict) -> Vec<Dict> {
    json::aget(res, "campaign").iter().map(|e| json::dict(e).clone()).collect()
}

fn field(events: &[Dict], key: &str) -> Vec<String> {
    events.iter().map(|e| json::gstr(e.get(key).unwrap_or(&Value::Null))).collect()
}

fn set_alert(gs: &mut GameState, v: i64) {
    let c = gs.campaign.as_mut().unwrap();
    c.meters.get_mut("alert").unwrap().value = v;
    c.sync(gs.time);
}

// ---- days and the clock (K1-K5) ----

#[test]
fn a_started_clock_counts_down_one_tick_per_leg() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    let start = 42;
    effect(&d, &mut gs, json!({"effect": "clock_start", "meter": "clock", "days": start}));
    assert_eq!(gs.read_flag("meter.clock.running"), Some(json!(true)));
    assert!(gs.campaign.as_ref().unwrap().meters["clock"].visible, "K1: started and shown");
    let res = travel::travel(&d, &mut gs, "c");
    assert_eq!(json::s(&res, "status"), "arrived", "{res:?}");
    let legs = json::i(&res, "legs_done", 0);
    assert_eq!(legs, 3);
    assert_eq!(value(&gs, "clock"), start - legs, "K2: one day per leg");
    assert_eq!((value(&gs, "days"), value(&gs, "legs")), (legs, legs), "the days tick and the leg tick ran once per leg");
    assert_eq!(gs.read_flag("campaign.day"), Some(json!(legs + 1)));
}

#[test]
fn clock_start_does_not_fire_the_rules_it_jumps_over() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    assert_eq!(value(&gs, "clock"), CLOCK_MAX, "M1: a stopped clock reads its max");
    let events = effect(&d, &mut gs, json!({"effect": "clock_start", "meter": "clock", "days": WARN_AT - 2}));
    assert!(events.is_empty(), "the jump from max to the days left is no crossing: {events:?}");
    assert_eq!(value(&gs, "clock"), WARN_AT - 2);
}

#[test]
fn a_leg_past_a_rule_point_warns_once() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    effect(&d, &mut gs, json!({"effect": "clock_start", "meter": "clock", "days": WARN_AT + 1}));
    let mut events = Vec::new();
    for _ in 0..2 {
        gs.time += 1;
        events.extend(campaign::tick(&d, &mut gs, &Cause::Days { n: 1, why: "travel".into() }));
    }
    let events: Vec<Dict> = events.iter().map(|e| json::dict(e).clone()).collect();
    assert_eq!(value(&gs, "clock"), WARN_AT - 1);
    assert_eq!(field(&events, "rule"), ["week"], "K5: the warning at 7, once");
}

#[test]
fn a_soft_clock_at_zero_runs_its_rule_and_restarts() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    effect(&d, &mut gs, json!({"effect": "clock_start", "meter": "clock", "days": 1}));
    let res = travel::travel(&d, &mut gs, "a");
    assert_eq!(json::s(&res, "status"), "arrived");
    assert_eq!(gs.flag("lesser"), Some(&json!(1)), "K4: the effects ran");
    assert_eq!(gs.campaign.as_ref().unwrap().pending, vec![json::obj([("vignette", "v_lesser".into())])], "the vignette is queued");
    assert_eq!(value(&gs, "clock"), RESTART, "restarted with restart_days left");
    assert!(!gs.campaign.as_ref().unwrap().is_over());
}

#[test]
fn a_hard_clock_at_zero_ends_the_run_and_stops_the_trip() {
    let d = data(&Setup { hard: true, ..Setup::default() });
    let mut gs = state(&d, 10);
    effect(&d, &mut gs, json!({"effect": "clock_start", "meter": "clock", "days": 2}));
    let res = travel::travel(&d, &mut gs, "c");
    assert_eq!(json::s(&res, "status"), "over", "{res:?}");
    assert_eq!(json::i(&res, "legs_done", -1), 1, "the second leg's tick ended the run");
    assert_eq!(gs.location_id, "w:a", "the party stays where the last leg ended");
    let events = events_of(&res);
    assert_eq!(field(&events, "event"), ["run_over"]);
    assert_eq!(field(&events, "card"), ["card_clock"]);
    assert!(!json::aget(&res, "events").iter().any(|e| json::s(json::dict(e), "kind") == "autosave"), "F3: no autosave");
}

#[test]
fn clock_set_relabels_and_clock_stop_reads_max() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    effect(&d, &mut gs, json!({"effect": "clock_start", "meter": "clock", "days": 30}));
    let set_to = 10;
    effect(&d, &mut gs, json!({"effect": "clock_set", "meter": "clock", "days": set_to, "label": "The Great Clock"}));
    assert_eq!(value(&gs, "clock"), set_to);
    assert_eq!(gs.campaign.as_ref().unwrap().meters["clock"].label, "The Great Clock", "K3: the HUD label");
    effect(&d, &mut gs, json!({"effect": "clock_stop", "meter": "clock"}));
    assert_eq!(gs.read_flag("meter.clock"), Some(json!(CLOCK_MAX)), "M1: stopped reads max");
    assert_eq!(gs.read_flag("meter.clock.running"), Some(json!(false)));
}

// ---- checkpoints (C1-C3) ----

fn start_job(d: &Data, gs: &mut GameState) {
    assert!(quest::start(d, gs, "job").is_empty());
}

#[test]
fn a_checkpoint_done_on_its_deadline_day_is_met() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    start_job(&d, &mut gs);
    at_day(&mut gs, DEADLINE);
    battle::settle(&d, &mut gs, "w:enc_job", "won", None);
    assert_eq!(quest::status(&d, &gs, "job"), "ready");
    assert_eq!(gs.campaign.as_ref().unwrap().checkpoints.get("cp").map(String::as_str), Some("met"), "C1");
    let res = travel::travel(&d, &mut gs, "a");
    assert_eq!(json::s(&res, "status"), "arrived");
    assert!(!gs.campaign.as_ref().unwrap().is_over(), "a met checkpoint is never missed");
}

#[test]
fn a_checkpoint_met_by_an_untick_action_counts_on_its_day() {
    // a quest finished between ticks (here: the stored state moves without a tick) is met on the day it happened, even
    // when the next tick is a leg past the deadline
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    start_job(&d, &mut gs);
    at_day(&mut gs, DEADLINE);
    let c = gs.campaign.take();
    battle::settle(&d, &mut gs, "w:enc_job", "won", None);
    gs.campaign = c;
    let res = travel::travel(&d, &mut gs, "a");
    assert_eq!(json::s(&res, "status"), "arrived");
    assert_eq!(gs.campaign.as_ref().unwrap().checkpoints.get("cp").map(String::as_str), Some("met"));
}

#[test]
fn a_missed_end_run_checkpoint_ends_the_run_once() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    start_job(&d, &mut gs);
    at_day(&mut gs, DEADLINE);
    let res = travel::travel(&d, &mut gs, "a");
    assert_eq!(json::s(&res, "status"), "over");
    let events = events_of(&res);
    assert_eq!(field(&events, "event"), ["run_over"]);
    assert_eq!((json::s(&events[0], "kind"), json::s(&events[0], "card"), json::s(&events[0], "text")), ("bad_end".into(), "bad_end_job".into(), "Too late.".into()), "C3");
    let c = gs.campaign.as_ref().unwrap();
    assert_eq!(c.checkpoints.get("cp").map(String::as_str), Some("missed"));
    assert!(campaign::tick(&d, &mut gs, &Cause::Days { n: 1, why: "rest".into() }).is_empty(), "recorded once");
}

#[test]
fn a_missed_go_on_checkpoint_runs_its_effects() {
    let d = data(&Setup { checkpoint: json!({"mode": "go_on", "effects": [{"effect": "set_flag", "key": "cages_lost"}]}), ..Setup::default() });
    let mut gs = state(&d, 10);
    start_job(&d, &mut gs);
    at_day(&mut gs, DEADLINE);
    let res = travel::travel(&d, &mut gs, "a");
    assert_eq!(json::s(&res, "status"), "arrived");
    assert!(gs.flag_true("cages_lost"), "C3: effects ran");
    assert!(!gs.campaign.as_ref().unwrap().is_over());
    assert_eq!(field(&events_of(&res), "event"), ["checkpoint_missed"]);
}

#[test]
fn one_trip_past_every_warning_day_warns_each_once_in_order() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    start_job(&d, &mut gs);
    let from = DEADLINE - LONG_LEG - 1;
    at_day(&mut gs, from);
    // the leg's tick runs before its road roll: the warnings come with an arrival or a road fight alike
    let res = travel::travel(&d, &mut gs, "far");
    assert!(json::b(&res, "ok", false) && json::s(&res, "status") != "over", "{res:?}");
    let events = events_of(&res);
    assert_eq!(field(&events, "event"), ["checkpoint_warning"; 3]);
    assert_eq!(field(&events, "warn"), ["7", "3", "1"], "C2: in order");
    assert!(events.iter().all(|e| json::i(e, "left", -1) == DEADLINE - (from + LONG_LEG) && json::i(e, "day", 0) == DEADLINE));
}

#[test]
fn the_board_and_the_map_show_an_open_deadline() {
    let d = data(&Setup::default());
    let mut gs = state(&d, 10);
    gs.set_flag(&rpgkit_data::board_opened_flag("clerk_board"), json!(true));
    let day = 4;
    at_day(&mut gs, day);
    assert_eq!(checkpoint::checkpoint(&d, &gs, "job"), Some((DEADLINE, DEADLINE - day)), "B6 reader");
    let view = board::view(&d, &gs, "clerk_board");
    let row = json::dict(&json::aget(json::dict(&json::aget(&view, "sections")[0]), "rows")[0]).clone();
    assert_eq!(json::s(json::dget(&row, "deadline"), "text"), board::deadline_text(DEADLINE, DEADLINE - day));
    assert_eq!(json::s(json::dget(&board::detail(&d, &gs, "w:job"), "deadline"), "text"), board::deadline_text(DEADLINE, DEADLINE - day), "B7");
    let map = travel::view(&d, &gs);
    let node_a = json::aget(&map, "nodes").iter().map(json::dict).find(|n| json::s(n, "qid") == "w:a").unwrap().clone();
    assert_eq!(json::s(json::dict(&json::aget(&node_a, "deadlines")[0]), "text"), board::checkpoint_text(DEADLINE), "B10");
    assert!(json::aget(&map, "nodes").iter().map(json::dict).filter(|n| json::s(n, "qid") != "w:a").all(|n| !n.contains_key("deadlines")), "absent when empty");
    // met: the lines go
    start_job(&d, &mut gs);
    battle::settle(&d, &mut gs, "w:enc_job", "won", None);
    assert_eq!(checkpoint::checkpoint(&d, &gs, "job"), None);
    assert!(board::detail(&d, &gs, "w:job").get("deadline").is_none());
    // and a world without a campaign shows none
    let classic = data(&Setup { campaign: false, ..Setup::default() });
    let gs = state(&classic, 10);
    assert_eq!(checkpoint::checkpoint(&classic, &gs, "job"), None);
    assert!(board::deadlines(&classic, &gs).is_empty());
}

// ---- upkeep (P2, P3, M7) ----

#[test]
fn a_leg_spends_its_days_times_per_day_and_is_never_refused() {
    let per_day = 2;
    let d = data(&Setup { blocks: json!({"purse": {"upkeep": {"per_day": per_day}}}), ..Setup::default() });
    let mut gs = state(&d, 10);
    let trip = travel::trip(&d, &gs, "w:hub", "w:c");
    assert_eq!(json::i(&trip, "supplies", 0), 3 * per_day, "the preview spends the same");
    let res = travel::travel(&d, &mut gs, "c");
    assert_eq!(json::i(&res, "supplies_spent", 0), 3 * per_day, "P2");
    gs.party.supplies = 0;
    let res = travel::travel(&d, &mut gs, "b");
    assert_eq!((json::s(&res, "status"), json::i(&res, "supplies_spent", -1)), ("arrived".into(), 0), "never refused at 0");
}

#[test]
fn zero_upkeep_and_band_modifiers_scale_the_road_chance() {
    let d = data(&Setup { blocks: json!({"purse": {"upkeep": {"at_zero": {"road_pct": 100}}}}), ..Setup::default() });
    let mut gs = state(&d, 0);
    let far = travel::route_in(&d, "w", "w:r_far");
    let base = json::i(&far, "encounter_pct", 0);
    assert_eq!(purse::road_chance(&d, &gs, &far, base, true), base, "P3: road_pct 100 leaves it");
    set_alert(&mut gs, LOCKDOWN);
    let scaled = (base * LOCKDOWN_PCT / 100).min(100);
    assert_eq!(purse::road_chance(&d, &gs, &far, base, true), scaled, "M7: Lockdown x1.5");
    assert_eq!(purse::road_chance(&d, &gs, &far, 80, false), 100, "capped at 100");
    let pool: Vec<String> = purse::road_encounters(&d, &gs, &far).iter().map(|p| json::gstr(&json::arr(p)[0])).collect();
    assert_eq!(pool, ["w:enc_hunt"], "M7: road_encounters join the pool");
    let street = travel::route_in(&d, "w", "w:r_street");
    assert_eq!(purse::road_chance(&d, &gs, &street, 40, false), 40, "a route without the tag is not scaled");
    assert!(purse::road_encounters(&d, &gs, &street).is_empty());
    // the defaults are today's x1.5 at zero
    let d = data(&Setup::default());
    let gs = state(&d, 0);
    assert_eq!(purse::road_chance(&d, &gs, &far, base, true), travel::starved_chance(base));
}

#[test]
fn zero_upkeep_lowers_the_start_ap_not_below_zero() {
    let d = data(&Setup { blocks: json!({"purse": {"upkeep": {"name": "Rations", "at_zero": {"start_ap": -2}}}}), ..Setup::default() });
    let gs = state(&d, 0);
    let mods = purse::battle_mods(&d, &gs);
    assert_eq!(mods.start_ap, -2);
    assert!(mods.note.contains("Rations"), "W6: {}", mods.note);
    assert_eq!((purse::start_ap(3, &mods), purse::start_ap(1, &mods)), (1, 0), "not below 0");
    let gs = state(&d, 1);
    assert_eq!(purse::battle_mods(&d, &gs), purse::BattleMods::default(), "with upkeep: nothing");
}

// ---- rest (R1-R3) ----

fn rest_blocks() -> Value {
    json!({"rest": {"days": 1, "cost": 1, "where": "rest_places", "on_enter": false, "needs_upkeep": false}})
}

fn hurt(gs: &mut GameState) {
    gs.party.members[0].hp_lost = HURT;
}

#[test]
fn a_rest_at_a_rest_place_heals_and_costs_a_day_and_upkeep() {
    let d = data(&Setup { blocks: rest_blocks(), ..Setup::default() });
    let mut gs = state(&d, 5);
    assert_eq!(progression::can_camp(&d, &gs), progression::REFUSE_RESTED);
    hurt(&mut gs);
    let day = gs.time + 1;
    let res = progression::make_camp(&d, &mut gs);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!((gs.party.members[0].hp_lost, gs.time + 1, gs.party.supplies), (0, day + 1, 4), "R2: full, a day, 1 upkeep");
    assert_eq!(value(&gs, "days"), 1, "the days tick ran (rest)");
    // on a street node: refused
    travel::travel(&d, &mut gs, "street");
    hurt(&mut gs);
    assert_eq!(progression::can_camp(&d, &gs), purse::REFUSE_NOT_REST_PLACE);
    assert!(!json::b(&progression::make_camp(&d, &mut gs), "ok", true));
}

#[test]
fn the_rest_effect_rests_and_its_days_tick_after_the_list() {
    let d = data(&Setup { blocks: rest_blocks(), ..Setup::default() });
    let mut gs = state(&d, 0);
    hurt(&mut gs);
    let mut talk = DialogueRunner::open(&d, &mut gs, "clerk");
    // CAMPAIGN1-U W3: the game reads a reply's effects before picking it (the ask before a rest that would end the run)
    let fx = talk.choice_effects(&d, &gs, 0);
    assert_eq!(fx.iter().map(|e| json::s(e, "effect")).collect::<Vec<_>>(), ["rest"]);
    assert!(talk.choice_effects(&d, &gs, 1).is_empty() && talk.choice_effects(&d, &gs, -1).is_empty(), "no such reply: none");
    assert_eq!((gs.party.members[0].hp_lost, gs.time), (HURT, 0), "reading changes nothing");
    let res = talk.choose(&d, &mut gs, 0);
    assert!(json::b(&res, "ok", false), "needs_upkeep false: rests at 0 upkeep: {res:?}");
    assert_eq!((gs.party.members[0].hp_lost, gs.time, value(&gs, "days")), (0, 1, 1));
    // at full HP the precheck refuses with R2's text
    let mut talk = DialogueRunner::open(&d, &mut gs, "clerk");
    let res = talk.choose(&d, &mut gs, 0);
    assert_eq!(json::sget(&res, "errors"), [progression::REFUSE_RESTED]);
}

#[test]
fn entering_a_rest_place_heals_only_when_on_enter() {
    let d = data(&Setup { blocks: rest_blocks(), ..Setup::default() });
    let mut gs = state(&d, 5);
    hurt(&mut gs);
    explore::enter(&d, &mut gs, "hub", explore::SPAWN);
    assert_eq!(gs.party.members[0].hp_lost, HURT, "R1: on_enter false");
    let d = data(&Setup::default());
    let mut gs = state(&d, 5);
    hurt(&mut gs);
    explore::enter(&d, &mut gs, "hub", explore::SPAWN);
    assert_eq!(gs.party.members[0].hp_lost, 0, "the default heals as today");
}

// ---- retreat (R4) ----

type LostPath = fn(&Data, &mut GameState) -> Dict;

/// The three `after_fight` paths, each settling a lost battle (the party stands at `a`).
fn lost_paths() -> [(&'static str, LostPath); 3] {
    [
        ("explore", |d, gs| explore::after_fight(d, gs, &json::obj([("encounter", "w:enc_job".into()), ("marker", "m".into())]), "lost", None)),
        ("travel", |d, gs| {
            let fight = json::obj([("encounter", "w:enc_road".into()), ("route", "w:r_b".into()), ("source", "road".into()), ("from", "w:a".into()), ("to", "w:b".into())]);
            travel::after_fight(d, gs, &fight, "lost", None)
        }),
        ("dialogue", |d, gs| rpgkit_world::dialogue::after_fight(d, gs, &json::obj([("encounter", "w:enc_job".into())]), "lost", None)),
    ]
}

fn defeat_event(res: &Dict) -> Dict {
    json::aget(res, "events").iter().map(json::dict).find(|e| json::s(e, "kind") == "defeat").cloned().unwrap_or_default()
}

#[test]
fn a_lost_battle_retreats_to_the_hub_on_every_path() {
    let per_day = 1;
    let days = 1;
    let d = data(&Setup { blocks: json!({"defeat": {"mode": "retreat", "to": "hub", "days": days}}), ..Setup::default() });
    for (tag, run) in lost_paths() {
        let mut gs = state(&d, 5);
        start_job(&d, &mut gs);
        travel::travel(&d, &mut gs, "a");
        let (time, supplies) = (gs.time, gs.party.supplies);
        let res = run(&d, &mut gs);
        assert!(json::b(&res, "ok", false), "{tag}: {res:?}");
        assert_eq!(gs.location_id, "w:hub", "{tag}: at the hub");
        assert_eq!(quest::status(&d, &gs, "job"), "active", "{tag}: the encounter stays undone");
        assert_eq!((gs.time, gs.party.supplies), (time + days, supplies - days * per_day), "{tag}: a day and its upkeep (CMQ4)");
        assert_eq!(json::s(&defeat_event(&res), "mode"), "retreat", "{tag}");
    }
}

#[test]
fn a_hub_that_cannot_be_entered_falls_back_to_the_wake_with_a_note() {
    let d = data(&Setup { blocks: json!({"defeat": {"mode": "retreat", "to": "ruin", "days": 1}}), ..Setup::default() });
    for (tag, run) in lost_paths() {
        let mut gs = state(&d, 5);
        travel::travel(&d, &mut gs, "a");
        hurt(&mut gs);
        let time = gs.time;
        let res = run(&d, &mut gs);
        assert!(json::b(&res, "ok", false), "{tag}: {res:?}");
        let events = json::aget(&res, "events");
        let first = json::dict(&events[0]);
        assert_eq!(json::s(first, "kind"), "retreat_failed", "{tag}: the note comes first");
        assert!(json::s(first, "text").contains("no_such_map"), "{tag}: the note names why: {}", json::s(first, "text"));
        let mode = json::s(&defeat_event(&res), "mode");
        assert!(!mode.is_empty() && mode != "retreat", "{tag}: the classic wake ({mode})");
        assert_ne!(gs.location_id, "w:ruin", "{tag}");
        assert_eq!((gs.time, gs.party.members[0].hp_lost), (time, 0), "{tag}: no day passes; the wake heals as today");
    }
}

// ---- classic worlds ----

#[test]
fn a_world_without_campaign_travels_rests_and_wakes_as_today() {
    let d = data(&Setup { campaign: false, blocks: json!({"rest": {"days": 1, "cost": 1}, "defeat": {"mode": "retreat", "to": "hub", "days": 1}, "purse": {"upkeep": {"per_day": 3}}}), ..Setup::default() });
    let mut gs = state(&d, 10);
    assert!(gs.campaign.is_none());
    let res = travel::travel(&d, &mut gs, "c");
    assert!(!res.contains_key("campaign"), "X3: no result key");
    assert_eq!(json::i(&res, "supplies_spent", 0), 3, "1 per time unit, as today");
    hurt(&mut gs);
    let res = progression::make_camp(&d, &mut gs);
    assert_eq!((json::i(&res, "spent", 0), gs.time), (progression::CAMP_COST, 3), "today's Camp: 3 supplies, no time");
    let res = explore::after_fight(&d, &mut gs, &json::obj([("encounter", "w:enc_job".into()), ("marker", "m".into())]), "lost", None);
    let defeat = json::aget(&res, "events").iter().map(json::dict).find(|e| json::s(e, "kind") == "defeat").cloned().unwrap_or_default();
    assert_ne!(json::s(&defeat, "mode"), "retreat", "the wake, as today");
    assert_eq!(purse::battle_mods(&d, &state(&d, 0)), purse::BattleMods::default());
}
