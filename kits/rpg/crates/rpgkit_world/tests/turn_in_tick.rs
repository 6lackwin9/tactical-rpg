//! ACT1-HOOKS2 (FB-246 (a)) on a scratch campaign world: a turn-in (`DialogueRunner::confirm_turn_in`) is a campaign
//! tick even when no effect follows it, so a trigger whose `when` the turn-in satisfies (its `complete_flags`) fires
//! right after it, in the turn-in's result; an `on: "any"` trigger with a `location` fires only while the party is at
//! that location (after a turn-in, a day, an entry), and checked again on later ticks a `once` trigger never fires
//! twice nor re-runs its effects; `on: "enter"` keeps its location filter.

use aegis_core::json::{self, Dict};
use rpgkit_world::campaign::{self, Cause};
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{explore, quest, Data, GameState};
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

/// World `w` (a campaign from act 1): the hub and the yard, one 1-day route between them; the clerk takes the turn-ins
/// of `job_a` (sets `a_in`) and `job_b` (sets `b_in`); the given triggers.
fn data(triggers: Value) -> Data {
    let w = "w";
    let manifest = json!({"id": w, "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1}});
    let job = |id: &str, flag: &str| json!({"id": id, "name": id, "template": "job", "turn_in": "clerk", "complete_flags": [{"key": flag, "value": true}]});
    let worlds = json!({w: {"manifest": manifest, "own": {
        "locations": with_ids(w, json!([{"id": "hub", "name": "Hub"}, {"id": "yard", "name": "Yard"}])),
        "routes": with_ids(w, json!([{"id": "r_yard", "from": "hub", "to": "yard", "time": 1}])),
        "characters": with_ids(w, json!([{"id": "hero", "name": "Hero", "role": "party"}, {"id": "clerk", "name": "Clerk", "role": "npc"}])),
        "quests": with_ids(w, json!([job("job_a", "a_in"), job("job_b", "b_in")])),
        "triggers": with_ids(w, triggers)
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let game = json!({"quests": [{"id": "job", "objectives": [{"kind": "win_encounter"}], "rewards": {"gold": 1}}]});
    Data::from_values(&catalog, &worlds, &game)
}

/// A new game at the hub with both jobs ready to turn in.
fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    for n in ["hub", "yard"] {
        gs.set_flag(&explore::visited_flag(&format!("w:{n}")), json!(true));
    }
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    for q in ["w:job_a", "w:job_b"] {
        gs.quests.insert(q.into(), json!({"state": "ready", "objective": 1}));
    }
    gs
}

/// Turns in `quest` to the clerk through the runner (its turn-in service); the confirm result.
fn turn_in(d: &Data, gs: &mut GameState, quest: &str) -> Dict {
    let mut r = DialogueRunner::open(d, gs, "clerk");
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    let services: Vec<String> = json::aget(&r.view(d, gs), "choices").iter().map(|c| json::s(json::dict(c), "service")).collect();
    for (i, svc) in services.iter().enumerate() {
        if svc != "turn_in" {
            continue;
        }
        let picked = r.choose(d, gs, i as i64);
        if json::s(&picked, "quest") != quest {
            r.cancel();
            continue;
        }
        let res = r.confirm_turn_in(d, gs, "", &[]);
        assert!(json::b(&res, "ok", false), "{res:?}");
        return res;
    }
    panic!("no turn-in of {quest} at the clerk: {services:?}");
}

fn fired(res: &Dict) -> Vec<String> {
    json::aget(res, "campaign").iter().map(json::dict).filter(|e| json::s(e, "event") == "trigger_fired").map(|e| json::s(e, "trigger")).collect()
}

fn count(gs: &GameState, key: &str) -> i64 {
    json::to_int(gs.flag(key).unwrap_or(&json!(0)))
}

/// Both turn-ins done: `t_after` (on any, at the hub) counts once.
fn both_in(on: &str, location: &str) -> Value {
    json!([{"id": "t_after", "name": "After", "on": on, "location": location,
            "when": [{"key": "a_in", "value": true}, {"key": "b_in", "value": true}], "effects": [{"effect": "add_flag", "key": "after_n", "delta": 1}]}])
}

#[test]
fn a_turn_in_ticks_the_campaign_and_fires_a_trigger_it_satisfies() {
    let d = data(both_in("any", ""));
    let mut gs = state(&d);
    let first = turn_in(&d, &mut gs, "w:job_a");
    assert!(fired(&first).is_empty() && count(&gs, "after_n") == 0, "one of two: the when does not hold");
    let second = turn_in(&d, &mut gs, "w:job_b");
    assert_eq!(quest::status(&d, &gs, "w:job_b"), "completed");
    assert_eq!(fired(&second), ["t_after"], "the turn-in with no effect after it still ticks: {second:?}");
    assert_eq!(count(&gs, "after_n"), 1);
    // later ticks with the when still holding: once means once, the effects do not run again
    campaign::tick(&d, &mut gs, &Cause::Effects);
    gs.time += 1;
    campaign::tick(&d, &mut gs, &Cause::Days { n: 1, why: "travel".into() });
    assert_eq!(count(&gs, "after_n"), 1, "a once trigger fires once");
    assert!(gs.campaign.as_ref().unwrap().fired.contains(&"trigger:t_after".to_string()));
}

#[test]
fn an_any_trigger_with_a_location_fires_only_while_the_party_is_there() {
    let d = data(both_in("any", "hub"));
    let mut gs = state(&d);
    turn_in(&d, &mut gs, "w:job_a");
    // away from the hub the second turn-in's tick does not fire it
    gs.location_id = "w:yard".into();
    let away = turn_in(&d, &mut gs, "w:job_b");
    assert!(fired(&away).is_empty() && count(&gs, "after_n") == 0, "not at the hub: {away:?}");
    gs.time += 1;
    campaign::tick(&d, &mut gs, &Cause::Days { n: 1, why: "travel".into() });
    assert_eq!(count(&gs, "after_n"), 0, "a day away: still not");
    // back at the hub any tick fires it (here the entry), once
    explore::enter(&d, &mut gs, "hub", explore::SPAWN);
    assert_eq!(count(&gs, "after_n"), 1, "fires at the hub");
    campaign::tick(&d, &mut gs, &Cause::Effects);
    explore::enter(&d, &mut gs, "hub", explore::SPAWN);
    assert_eq!(count(&gs, "after_n"), 1, "once: not again");
}

#[test]
fn an_any_trigger_at_the_location_fires_on_the_turn_in_there() {
    let d = data(both_in("any", "hub"));
    let mut gs = state(&d);
    turn_in(&d, &mut gs, "w:job_a");
    let res = turn_in(&d, &mut gs, "w:job_b");
    assert_eq!(fired(&res), ["t_after"], "at the hub: the turn-in fires it without walking out: {res:?}");
    assert_eq!(count(&gs, "after_n"), 1);
}

#[test]
fn an_enter_trigger_keeps_waiting_for_the_entry() {
    let d = data(both_in("enter", "hub"));
    let mut gs = state(&d);
    turn_in(&d, &mut gs, "w:job_a");
    let res = turn_in(&d, &mut gs, "w:job_b");
    assert!(fired(&res).is_empty() && count(&gs, "after_n") == 0, "on enter: the turn-in tick is not an entry");
    explore::enter(&d, &mut gs, "yard", explore::SPAWN);
    assert_eq!(count(&gs, "after_n"), 0, "another location");
    explore::enter(&d, &mut gs, "hub", explore::SPAWN);
    assert_eq!(count(&gs, "after_n"), 1, "entering the hub");
}
