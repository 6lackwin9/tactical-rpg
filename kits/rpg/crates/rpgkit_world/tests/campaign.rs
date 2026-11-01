//! CAMPAIGN1-M on scratch worlds (CAMPAIGN1.md X1-X5, M1-M8, F1, F3, 11.3-11.5): a world without `campaign` stays
//! inert; read keys are reserved; gauges move by effects and battle inputs, clamp to floor..max, cross bands and fire
//! rules (warnings, effects, `then` items, the run over) in crossing order, once per crossing; the act raises floors;
//! read keys answer conditions; the campaign block saves and loads (a v12 save gets a fresh one) and no save is
//! written while the run is over. Every expectation follows from the rule and the fixture's own numbers.

use aegis_core::json::{self, Dict};
use rpgkit_world::campaign::{self, Cause, RUN_OVER_TEXT};
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{battle, quest, save, Data, GameState};
use serde_json::{json, Value};

/// Alert's numbers in the fixture (the rules below are written against them).
const SUSPICIOUS: i64 = 31;
const LOCKDOWN: i64 = 71;
const LAST_WARNING: i64 = 90;
const BREACH: i64 = 100;
const LOUD_WON: i64 = 8;
const FLOOR_ACT2: i64 = 30;
const CLOCK_MAX: i64 = 99;

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

fn alert() -> Value {
    json!({
        "id": "alert", "name": "Alert", "kind": "gauge", "min": 0, "max": BREACH, "start": 0, "unit": "%",
        "bands": [
            {"id": "guarded", "name": "Guarded", "from": 0, "tone": "calm"},
            {"id": "suspicious", "name": "Suspicious", "from": SUSPICIOUS, "tone": "warn"},
            {"id": "lockdown", "name": "Lockdown", "from": LOCKDOWN, "tone": "danger"},
            {"id": "breach", "name": "Breach", "from": BREACH, "tone": "fatal"}
        ],
        "rules": [
            {"id": "a_suspicious", "at": "suspicious", "warn": "Suspicious."},
            {"id": "a_lockdown", "at": "lockdown", "warn": "Lockdown."},
            {"id": "a_last", "at": LAST_WARNING, "acts": [1], "warn": "One more."},
            {"id": "a_breach_1", "at": "breach", "acts": [1], "effects": [{"effect": "end", "kind": "fail", "card": "extinguished_breach"}], "then": [{"vignette": "v_never"}]}
        ],
        "inputs": [{"on": "battle", "tags": ["loud"], "outcomes": ["won"], "delta": LOUD_WON}],
        "floors": [{"from_act": 2, "min": FLOOR_ACT2}]
    })
}

fn clock() -> Value {
    json!({
        "id": "doom_clock", "name": "The Clock", "kind": "countdown", "min": 0, "max": CLOCK_MAX, "start": 0, "unit": "days",
        "bands": [{"id": "due", "name": "Due", "from": 0, "tone": "danger"}, {"id": "far", "name": "Far", "from": 8, "tone": "calm"}]
    })
}

/// `heat` at 50 pushes `cool` up by 10; `cool` at 10 sets a flag and queues a vignette: one move crosses both.
fn pair() -> Vec<Value> {
    vec![
        json!({"id": "heat", "name": "Heat", "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}],
               "rules": [{"id": "h50", "at": 50, "effects": [{"effect": "meter_add", "meter": "cool", "delta": 10}]}]}),
        json!({"id": "cool", "name": "Cool", "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}],
               "rules": [{"id": "c10", "at": 10, "effects": [{"effect": "add_flag", "key": "cooled", "delta": 1}], "then": [{"vignette": "v_cool"}]}]}),
    ]
}

fn encounters(w: &str) -> Value {
    with_ids(w, json!([{"id": "loud_job", "name": "Loud", "tags": ["loud"], "enemies": []}, {"id": "quiet_job", "name": "Quiet", "tags": ["quiet"], "enemies": []}]))
}

/// A talker whose one choice runs `effects`.
fn talk(w: &str, effects: Value) -> (Value, Value) {
    let dialogues = with_ids(w, json!([{"id": "talk", "start": "a", "services": false, "nodes": [{"id": "a", "text": "Well?", "choices": [{"text": "Do it.", "effects": effects}]}]}]));
    let characters = with_ids(w, json!([{"id": "tess", "name": "Tess", "role": "npc", "dialogue": "talk"}]));
    (dialogues, characters)
}

/// World `classic` (no campaign) and `camp` (a campaign from act 1 with Alert, the clock and the pair); both share the
/// encounters and a talker that runs `effects`.
fn data_with(effects: Value) -> Data {
    let mut meters = vec![alert(), clock()];
    meters.extend(pair());
    let (cd, cc) = talk("classic", effects.clone());
    let (kd, kc) = talk("camp", effects);
    let worlds = json!({
        "classic": {"manifest": {"id": "classic"}, "own": {"encounters": encounters("classic"), "dialogues": cd, "characters": cc}},
        "camp": {"manifest": {"id": "camp", "campaign": {"start_act": 1}}, "own": {
            "meters": with_ids("camp", Value::Array(meters)), "encounters": encounters("camp"), "dialogues": kd, "characters": kc}}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn data() -> Data {
    data_with(json!([{"effect": "meter_add", "meter": "alert", "delta": 10}]))
}

fn state(d: &Data, w: &str) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, w, party)
}

fn set_alert(gs: &mut GameState, value: i64) {
    let c = gs.campaign.as_mut().expect("a campaign");
    c.meters.get_mut("alert").unwrap().value = value;
    c.sync(gs.time);
}

fn alert_of(gs: &GameState) -> i64 {
    gs.campaign.as_ref().unwrap().value("alert", gs.time)
}

fn events_of(res: &Dict) -> Vec<Dict> {
    json::aget(res, "campaign").iter().map(|e| json::dict(e).clone()).collect()
}

fn kinds(events: &[Dict]) -> Vec<String> {
    events.iter().map(|e| json::s(e, "event")).collect()
}

fn cond(key: &str, op: &str, value: Value) -> Dict {
    json::obj([("key", key.into()), ("op", op.into()), ("value", value)])
}

#[test]
fn a_world_without_campaign_stays_inert() {
    let d = data();
    let mut gs = state(&d, "classic");
    assert!(gs.campaign.is_none() && !campaign::is_on(&d, "classic"));
    assert!(!gs.to_value().as_object().unwrap().contains_key("campaign"), "no saved key (X3)");
    let res = battle::settle(&d, &mut gs, "classic:loud_job", "won", None);
    assert!(json::b(&res, "ok", false) && !res.contains_key("campaign"), "no result key (X3): {res:?}");
    let mut r = DialogueRunner::open(&d, &mut gs, "tess");
    let res = r.choose(&d, &mut gs, 0);
    assert!(json::b(&res, "ok", false) && !res.contains_key("campaign"), "a campaign effect does nothing: {res:?}");
    assert!(campaign::tick(&d, &mut gs, &Cause::Effects).is_empty());
    for key in ["meter.alert", "campaign.act", "campaign.day"] {
        assert_eq!(gs.read_flag(key), None, "no read key answered: {key}");
    }
    // ROSTER1 RK1: the roster's read keys are the party's, answered in every world
    assert_eq!(gs.read_flag("roster.living"), Some(json!(gs.party.members.len())));
    assert!(campaign::save_refusal(&gs).is_empty());
}

#[test]
fn read_keys_are_reserved_in_every_world() {
    let d = data_with(json!([{"effect": "set_flag", "key": "meter.alert", "value": 5}]));
    for w in ["classic", "camp"] {
        let mut gs = state(&d, w);
        assert_eq!(gs.set_flag("meter.alert", json!(5)), campaign::reserved_text("meter.alert"));
        assert_eq!(quest::set_flag(&d, &mut gs, "campaign.act", json!(3)), campaign::reserved_text("campaign.act"));
        assert_eq!(quest::add_flag(&d, &mut gs, "roster.living", 1, &Value::Null, &Value::Null), campaign::reserved_text("roster.living"));
        let mut r = DialogueRunner::open(&d, &mut gs, "tess");
        let res = r.choose(&d, &mut gs, 0);
        assert!(!json::b(&res, "ok", true), "{w}: the choice is refused");
        assert_eq!(json::sget(&res, "errors"), vec![campaign::reserved_text("meter.alert")]);
        assert!(gs.flags.is_empty());
    }
}

#[test]
fn a_meter_add_moves_the_gauge_into_its_band_with_one_warning() {
    let d = data();
    let mut gs = state(&d, "camp");
    let start = SUSPICIOUS - 6;
    set_alert(&mut gs, start);
    let mut r = DialogueRunner::open(&d, &mut gs, "tess");
    let res = r.choose(&d, &mut gs, 0);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(alert_of(&gs), start + 10);
    assert_eq!(gs.read_flag("meter.alert.band"), Some(json!("suspicious")));
    let events = events_of(&res);
    assert_eq!(kinds(&events), ["meter_warning"]);
    assert_eq!((json::s(&events[0], "rule"), json::s(&events[0], "text"), json::s(&events[0], "tone")), ("a_suspicious".into(), "Suspicious.".into(), "warn".into()));
}

#[test]
fn breach_in_act_1_ends_the_run_once_and_refuses_saves() {
    let d = data();
    let mut gs = state(&d, "camp");
    set_alert(&mut gs, BREACH - 5);
    let mut r = DialogueRunner::open(&d, &mut gs, "tess");
    let res = r.choose(&d, &mut gs, 0);
    assert_eq!(alert_of(&gs), BREACH, "clamped at max (M4)");
    let events = events_of(&res);
    assert_eq!(kinds(&events), ["run_over"], "95 is past the 90 warning; the breach rule fires and ends the run");
    assert_eq!((json::s(&events[0], "kind"), json::s(&events[0], "card")), ("fail".into(), "extinguished_breach".into()));
    let c = gs.campaign.as_ref().unwrap();
    assert!(c.is_over() && c.pending.is_empty(), "the tick stops at the end: no then item queued (F1)");
    assert!(campaign::tick(&d, &mut gs, &Cause::Effects).is_empty(), "nothing runs once the run is over");
    let dir = std::env::temp_dir().join(format!("rpgkit_campaign_over_{}", std::process::id()));
    assert_eq!(save::write(&d, &dir, &gs, 1, 1), vec![RUN_OVER_TEXT.to_string()], "F3");
    assert!(!save::exists(&dir, 1));
    assert!(!gs.to_value()["campaign"].as_object().unwrap().contains_key("over"), "over is never saved");
}

#[test]
fn an_act_1_rule_does_not_fire_in_act_2() {
    let d = data();
    let mut gs = state(&d, "camp");
    let mut events = Vec::new();
    campaign::apply_effect(&d, &mut gs, &json::obj([("effect", "set_act".into()), ("act", 2.into())]), &mut events);
    campaign::tick(&d, &mut gs, &Cause::Effects);
    set_alert(&mut gs, BREACH - 5);
    let mut r = DialogueRunner::open(&d, &mut gs, "tess");
    let res = r.choose(&d, &mut gs, 0);
    assert_eq!(alert_of(&gs), BREACH);
    assert!(events_of(&res).is_empty(), "the breach rule is act 1 only (M5 acts)");
    assert!(!gs.campaign.as_ref().unwrap().is_over());
}

#[test]
fn a_new_act_raises_the_gauge_to_its_floor() {
    let d = data_with(json!([{"effect": "set_act", "act": 2}]));
    let mut gs = state(&d, "camp");
    let start = FLOOR_ACT2 - 18;
    set_alert(&mut gs, start);
    let mut r = DialogueRunner::open(&d, &mut gs, "tess");
    let res = r.choose(&d, &mut gs, 0);
    assert_eq!(alert_of(&gs), FLOOR_ACT2, "M4: the value rises to the floor");
    assert_eq!(gs.read_flag("campaign.act"), Some(json!(2)));
    let events = events_of(&res);
    assert_eq!(kinds(&events), ["act_start"]);
    let run = gs.campaign.as_ref().unwrap().run.clone();
    assert_eq!(json::s(&events[0], "save"), campaign::act_save_key(2, &run), "act_saves defaults on (F5)");
    let mut ev = Vec::new();
    campaign::apply_effect(&d, &mut gs, &json::obj([("effect", "meter_add".into()), ("meter", "alert".into()), ("delta", (-20).into())]), &mut ev);
    assert_eq!(alert_of(&gs), FLOOR_ACT2, "the floor holds for the rest of the act");
}

#[test]
fn battle_inputs_follow_the_tags_and_the_outcome() {
    let d = data();
    let mut gs = state(&d, "camp");
    let res = battle::settle(&d, &mut gs, "camp:loud_job", "won", None);
    assert!(json::b(&res, "ok", false));
    assert_eq!(alert_of(&gs), LOUD_WON);
    let res = battle::settle(&d, &mut gs, "camp:loud_job", "lost", None);
    assert_eq!(alert_of(&gs), LOUD_WON, "no lost input: +0");
    assert!(!res.contains_key("campaign"), "no event, no key");
    battle::settle(&d, &mut gs, "camp:quiet_job", "won", None);
    assert_eq!(alert_of(&gs), LOUD_WON, "a quiet fight has no input here");
    set_alert(&mut gs, SUSPICIOUS - 1);
    let res = battle::settle(&d, &mut gs, "camp:loud_job", "won", None);
    assert_eq!(kinds(&events_of(&res)), ["meter_warning"], "the input crosses suspicious");
    assert_eq!(kinds(&events_of(&battle::result_keys(&res))), ["meter_warning"], "after_fight results carry the events");
}

#[test]
fn one_move_crossing_two_points_fires_them_in_order() {
    let d = data();
    let mut gs = state(&d, "camp");
    set_alert(&mut gs, SUSPICIOUS - 1);
    let mut events = Vec::new();
    campaign::apply_effect(&d, &mut gs, &json::obj([("effect", "meter_set".into()), ("value", LOCKDOWN.into()), ("meter", "alert".into())]), &mut events);
    events.extend(campaign::tick(&d, &mut gs, &Cause::Effects));
    let rules: Vec<String> = events.iter().map(|e| json::s(json::dict(e), "rule")).collect();
    assert_eq!(rules, ["a_suspicious", "a_lockdown"]);
}

#[test]
fn a_rule_whose_effect_crosses_another_fires_both_once() {
    let d = data();
    let mut gs = state(&d, "camp");
    let mut events = Vec::new();
    campaign::apply_effect(&d, &mut gs, &json::obj([("effect", "meter_add".into()), ("meter", "heat".into()), ("delta", 50.into())]), &mut events);
    campaign::tick(&d, &mut gs, &Cause::Effects);
    let c = gs.campaign.as_ref().unwrap();
    assert_eq!((c.value("heat", 0), c.value("cool", 0)), (50, 10));
    assert_eq!(gs.flag("cooled"), Some(&json!(1)), "c10 fired once");
    let pending: Vec<Dict> = gs.campaign.as_ref().unwrap().pending.clone();
    assert_eq!(pending, vec![json::obj([("vignette", "v_cool".into())])]);
    assert!(campaign::tick(&d, &mut gs, &Cause::Effects).is_empty() && gs.flag("cooled") == Some(&json!(1)), "no move, no rule");
    assert_eq!(campaign::pending_head(&gs), pending.first());
    assert_eq!(campaign::pop_pending(&mut gs), pending.first().cloned());
    assert_eq!(campaign::pop_pending(&mut gs), None);
}

#[test]
fn read_keys_answer_conditions_as_the_values_move() {
    let d = data();
    let mut gs = state(&d, "camp");
    let at_least = cond("meter.alert", ">=", json!(SUSPICIOUS));
    let lockdown = cond("meter.alert.band", "==", json!("lockdown"));
    let act1 = cond("campaign.act", "==", json!(1));
    assert!(!quest::condition_met(&gs, &at_least) && !quest::condition_met(&gs, &lockdown) && quest::condition_met(&gs, &act1));
    set_alert(&mut gs, SUSPICIOUS);
    assert!(quest::condition_met(&gs, &at_least) && !quest::condition_met(&gs, &lockdown));
    set_alert(&mut gs, LOCKDOWN);
    assert!(quest::condition_met(&gs, &lockdown));
    assert_eq!(gs.read_flag("campaign.day"), Some(json!(gs.time + 1)), "X2: day 1 at time 0");
    assert_eq!(gs.read_flag("roster.living"), Some(json!(gs.party.members.len())));
    assert_eq!(gs.read_flag("meter.doom_clock"), Some(json!(CLOCK_MAX)), "a stopped countdown reads its max (M1)");
    assert_eq!(gs.read_flag("meter.doom_clock.running"), Some(json!(false)));
    assert_eq!(gs.read_flag("meter.nope"), None);
}

#[test]
fn the_campaign_block_saves_and_loads() {
    let d = data();
    let mut gs = state(&d, "camp");
    set_alert(&mut gs, 42);
    let mut events = Vec::new();
    campaign::apply_effect(&d, &mut gs, &json::obj([("effect", "meter_show".into()), ("meter", "alert".into())]), &mut events);
    campaign::apply_effect(&d, &mut gs, &json::obj([("effect", "meter_add".into()), ("meter", "heat".into()), ("delta", 50.into())]), &mut events);
    campaign::tick(&d, &mut gs, &Cause::Effects);
    {
        let c = gs.campaign.as_mut().unwrap();
        c.fired.push("rule:alert/a_last".into());
        c.fired.sort();
        assert!(!c.pending.is_empty());
    }
    let dir = std::env::temp_dir().join(format!("rpgkit_campaign_save_{}", std::process::id()));
    assert!(save::write(&d, &dir, &gs, 1, 1).is_empty());
    let back = save::read(&d, &dir, 1).expect("reads back");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert_eq!(back.state, gs);
    let c = back.state.campaign.as_ref().unwrap();
    assert!(c.meters["alert"].visible && c.value("alert", 0) == 42 && c.fired == ["rule:alert/a_last"]);
}

#[test]
fn a_save_without_campaign_of_a_campaign_world_gets_a_fresh_campaign() {
    let d = data();
    let gs = state(&d, "camp");
    let mut old = gs.to_value();
    old.as_object_mut().unwrap().shift_remove("campaign");
    let text = json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": old}).to_string();
    let loaded = save::decode(&d, &text).expect("loads");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let fresh = campaign::Campaign::fresh(&d, "camp", gs.rng_seed).unwrap();
    assert_eq!(loaded.state.campaign.as_ref(), Some(&fresh));
    // and a save of a classic world loads with none
    let classic = state(&d, "classic").to_value();
    let text = json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": classic}).to_string();
    assert!(save::decode(&d, &text).expect("loads").state.campaign.is_none());
}

#[test]
fn a_damaged_campaign_block_is_refused_not_coerced() {
    let d = data();
    let gs = state(&d, "camp");
    let mut v = gs.to_value();
    let c = v["campaign"].as_object_mut().unwrap();
    c.insert("act".into(), json!(12));
    c.insert("pending".into(), json!([{"vignette": "v", "battle": "b"}]));
    c["meters"].as_object_mut().unwrap().insert("ghost".into(), json!({"value": 1}));
    let text = json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": v}).to_string();
    let errors = save::decode(&d, &text).err().expect("refused");
    assert_eq!(errors.len(), 3, "{errors:?}");
    // a non-object meters or checkpoints, a non-text label or checkpoint state, an unknown key in the block or a
    // meter's state: one problem each, never dropped silently
    let damaged: Vec<(&str, Box<dyn Fn(&mut Dict)>, &str)> = vec![
        ("meters not an object", Box::new(|c: &mut Dict| { c.insert("meters".into(), json!([1])); }), "campaign.meters must be an object."),
        ("checkpoints not an object", Box::new(|c: &mut Dict| { c.insert("checkpoints".into(), json!("met")); }), "campaign.checkpoints must be an object."),
        ("a checkpoint state not text", Box::new(|c: &mut Dict| { c.insert("checkpoints".into(), json!({"cp": 1})); }), "campaign.checkpoints.cp must be met or missed."),
        ("a label not text", Box::new(|c: &mut Dict| { c["meters"]["alert"]["label"] = json!(5); }), "campaign.meters.alert.label must be text."),
        ("a hand-written over", Box::new(|c: &mut Dict| { c.insert("over".into(), json!({"kind": "fail"})); }), "campaign: unknown key 'over'."),
        ("an unknown meter state key", Box::new(|c: &mut Dict| { c["meters"]["alert"]["hot"] = json!(true); }), "campaign.meters.alert: unknown key 'hot'."),
    ];
    for (tag, damage, want) in damaged {
        let mut v = gs.to_value();
        damage(v["campaign"].as_object_mut().unwrap());
        let text = json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": v}).to_string();
        let errors = save::decode(&d, &text).err().unwrap_or_else(|| panic!("{tag}: refused"));
        assert_eq!(errors, vec![want.to_string()], "{tag}");
    }
}

/// Meters `m0`..`m<last>` written in reverse data order, so each pass fires one link: `m<i>` at 1 adds 1 to
/// `m<i+1>`; the last one sets the flag `chain_end`.
fn chain_data(last: usize) -> Data {
    let band = json!([{"id": "low", "name": "Low", "from": 0, "tone": "calm"}]);
    let meters: Vec<Value> = (0..=last)
        .rev()
        .map(|i| {
            let fx = if i == last { json!({"effect": "set_flag", "key": "chain_end"}) } else { json!({"effect": "meter_add", "meter": format!("m{}", i + 1), "delta": 1}) };
            json!({"id": format!("m{i}"), "name": format!("M{i}"), "bands": band, "rules": [{"id": "r", "at": 1, "effects": [fx]}]})
        })
        .collect();
    let worlds = json!({"chain": {"manifest": {"id": "chain", "campaign": {"start_act": 1}}, "own": {"meters": with_ids("chain", Value::Array(meters))}}});
    Data::from_values(&json!({}), &worlds, &json!({}))
}

/// Starts the chain (m0 + 1) and ticks once.
fn run_chain(d: &Data) -> GameState {
    let mut gs = GameState::create(d, "chain", PartyState::default());
    let mut events = Vec::new();
    campaign::apply_effect(d, &mut gs, &json::obj([("effect", "meter_add".into()), ("meter", "m0".into()), ("delta", 1.into())]), &mut events);
    campaign::tick(d, &mut gs, &Cause::Effects);
    gs
}

#[test]
fn rules_re_enter_at_most_eight_times_per_tick() {
    // the first pass fires m0; re-entries 1..8 fire m1..m8
    let d = chain_data(campaign::REENTRIES_MAX);
    let gs = run_chain(&d);
    assert!(gs.flag_true("chain_end"), "a chain of exactly {} re-entries completes", campaign::REENTRIES_MAX);
    // one link more: m9 moves on the 8th re-entry, its rule would need a 9th
    let d = chain_data(campaign::REENTRIES_MAX + 1);
    let mut gs = run_chain(&d);
    let last = format!("m{}", campaign::REENTRIES_MAX + 1);
    assert_eq!(gs.campaign.as_ref().unwrap().value(&last, 0), 1, "the 8th re-entry moved it");
    assert!(!gs.flag_true("chain_end"), "a 9th re-entry is not run");
    assert!(campaign::tick(&d, &mut gs, &Cause::Effects).is_empty() && !gs.flag_true("chain_end"), "the left-over crossing is dropped, not fired later");
}
