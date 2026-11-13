//! ACT1-HOOKS2 (FB-244 (2)) on a scratch campaign world: a meter rule's effects run in order, each under its `when`
//! (flag conditions read when the effect is reached) and `else` (runs only when no effect of its chain ran yet; an
//! effect without `else` starts a chain), so one rule writes if / else if / else; an ungated effect beside the chain
//! runs every time; a negative `add_flag` with `min` 0 never goes below 0. Each firing below is one crossing of the
//! rule's point by a dialogue choice's `meter_add`, and every expectation follows from the chain's own conditions.

use aegis_core::json;
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{Data, GameState};
use serde_json::{json, Value};

const AT: i64 = 10;
const COUNT: &str = "w.count";
const FIRST_GUARD: &str = "w.guard_a";
const FIRST: &str = "w.first";
const SECOND_GUARD: &str = "w.guard_b";
const SECOND: &str = "w.second";
const POOL: &str = "w.pool";
const DROP: i64 = 2;

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

fn unset(key: &str) -> Value {
    json!({"key": key, "op": "!=", "value": true})
}

/// The gauge's rule at `AT`: count every firing; if neither the first guard nor `FIRST` holds, set `FIRST`; else if
/// neither the second guard nor `SECOND` holds, set `SECOND`; else take `DROP` from the pool, floored at 0.
fn data() -> Data {
    let effects = json!([
        {"effect": "add_flag", "key": COUNT, "delta": 1},
        {"effect": "set_flag", "key": FIRST, "when": [unset(FIRST_GUARD), unset(FIRST)]},
        {"effect": "set_flag", "key": SECOND, "else": true, "when": [unset(SECOND_GUARD), unset(SECOND)]},
        {"effect": "add_flag", "key": POOL, "delta": -DROP, "min": 0, "else": true}
    ]);
    let gauge = json!({"id": "gauge", "name": "Gauge", "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}],
        "rules": [{"id": "r_cross", "at": AT, "effects": effects}]});
    let worlds = json!({"w": {"manifest": {"id": "w", "campaign": {"start_act": 1}}, "own": {
        "meters": with_ids("w", json!([gauge])),
        "characters": with_ids("w", json!([{"id": "clerk", "name": "Clerk", "role": "npc", "dialogue": "talk"}])),
        "dialogues": with_ids("w", json!([{"id": "talk", "start": "a", "services": false, "nodes": [{"id": "a", "text": "Well?",
            "choices": [{"text": "Push.", "effects": [{"effect": "meter_add", "meter": "gauge", "delta": AT}]}]}]}]))
    }}});
    let catalog = json!({"classes": [{"id": "drill", "name": "Drill"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}]});
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data, flags: &[(&str, Value)]) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    for (k, v) in flags {
        gs.set_flag(k, v.clone());
    }
    gs
}

/// One crossing: the gauge back to 0, then the choice pushes it over the rule's point.
fn fire(d: &Data, gs: &mut GameState) {
    let c = gs.campaign.as_mut().expect("a campaign");
    c.meters.get_mut("gauge").unwrap().value = 0;
    c.sync(gs.time);
    let mut r = DialogueRunner::open(d, gs, "clerk");
    let res = r.choose(d, gs, 0);
    assert!(json::b(&res, "ok", false), "{res:?}");
}

fn flag(gs: &GameState, key: &str) -> Option<Value> {
    gs.flag(key).cloned()
}

#[test]
fn the_first_branch_whose_when_holds_runs_and_the_rest_of_its_chain_does_not() {
    let d = data();
    // nothing set: the if branch
    let mut gs = state(&d, &[(POOL, json!(5))]);
    fire(&d, &mut gs);
    assert_eq!((flag(&gs, FIRST), flag(&gs, SECOND), flag(&gs, POOL)), (Some(json!(true)), None, Some(json!(5))));
    // the if guard holds: the else-if branch
    let mut gs = state(&d, &[(FIRST_GUARD, json!(true)), (POOL, json!(5))]);
    fire(&d, &mut gs);
    assert_eq!((flag(&gs, FIRST), flag(&gs, SECOND), flag(&gs, POOL)), (None, Some(json!(true)), Some(json!(5))));
    // both guards hold: the else branch
    let mut gs = state(&d, &[(FIRST_GUARD, json!(true)), (SECOND_GUARD, json!(true)), (POOL, json!(5))]);
    fire(&d, &mut gs);
    assert_eq!((flag(&gs, FIRST), flag(&gs, SECOND), flag(&gs, POOL)), (None, None, Some(json!(5 - DROP))));
    assert_eq!(flag(&gs, COUNT), Some(json!(1)), "the ungated effect ran once");
}

/// Each branch sets the flag its own `when` excludes, so successive crossings walk the chain once each, then keep
/// taking the else branch, whose floored `add_flag` stops at 0; the ungated effect runs at every crossing.
#[test]
fn successive_crossings_walk_the_chain_and_the_floor_holds_at_zero() {
    let d = data();
    let start = 3;
    let mut gs = state(&d, &[(POOL, json!(start))]);
    let mut pool = start;
    for n in 1..=5 {
        fire(&d, &mut gs);
        if n >= 3 {
            pool = (pool - DROP).max(0);
        }
        assert_eq!(flag(&gs, FIRST), Some(json!(true)), "crossing {n}");
        assert_eq!(flag(&gs, SECOND), (n >= 2).then(|| json!(true)), "crossing {n}");
        assert_eq!(flag(&gs, POOL), Some(json!(pool)), "crossing {n}");
        assert_eq!(flag(&gs, COUNT), Some(json!(n)), "crossing {n}");
    }
    assert_eq!(pool, 0, "the floor was reached");
    // an unset pool reads 0 and stays at the floor
    let mut gs = state(&d, &[(FIRST, json!(true)), (SECOND, json!(true))]);
    fire(&d, &mut gs);
    assert_eq!(flag(&gs, POOL), Some(json!(0)));
}
