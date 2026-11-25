//! BATTLE-UI2 (the battle screen contract 2 and 4): the top bar's goal and protect lines (`Objectives::goal_of`,
//! `Objectives::protect_of`) and the foe panel's reinforcement tag (`Phases::next_reinforcement`), on rows built
//! here (no shipped data). Expectations come from the contract's texts ("Rout them", "Survive 6 rounds", "Reach the
//! exit", "Protect A") and the phase rule (the first round-triggered phase not fired or dropped that brings units).

use rpgkit_battle::encounter::{Objectives, Phases, RETREAT};
use serde_json::{json, Value};

fn obj(v: Value) -> aegis_core::value::Obj {
    v.as_object().cloned().expect("an object")
}

fn name_of(r: &str) -> String {
    format!("<{r}>")
}

#[test]
fn the_goal_line_reads_as_the_contract() {
    assert_eq!(Objectives::goal_of(&obj(json!({})), name_of), "Rout them");
    assert_eq!(Objectives::goal_of(&obj(json!({"objective": "rout"})), name_of), "Rout them");
    assert_eq!(Objectives::goal_of(&obj(json!({"objective": "hold", "objective_rounds": 6})), name_of), "Survive 6 rounds");
    assert_eq!(Objectives::goal_of(&obj(json!({"objective": "escape"})), name_of), "Reach the exit");
    assert_eq!(Objectives::goal_of(&obj(json!({"objective": "slay", "objective_unit": "boss"})), name_of), "Defeat <boss>");
    assert_eq!(Objectives::goal_of(&obj(json!({"objective": RETREAT, "objective_unit": "boss"})), name_of), "Drive <boss> back");
}

#[test]
fn the_protect_line_names_every_protect_target() {
    assert_eq!(Objectives::protect_of(&obj(json!({})), name_of), "");
    assert_eq!(Objectives::protect_of(&obj(json!({"protect": ["a"]})), name_of), "Protect <a>");
    assert_eq!(Objectives::protect_of(&obj(json!({"protect": ["a", "b"]})), name_of), "Protect <a> and <b>");
}

#[test]
fn the_reinforcement_tag_is_the_next_round_phase_with_units() {
    let raw = vec![
        json!({"id": "late", "trigger": {"kind": "round", "round": 5}}),
        json!({"id": "soon", "trigger": {"kind": "round", "round": 3}}),
        json!({"id": "empty", "trigger": {"kind": "round", "round": 2}}),
        json!({"id": "hurt", "trigger": {"kind": "hp_below", "unit": "boss", "pct": 50}}),
    ];
    let mut p = Phases::create(&raw);
    assert_eq!(p.next_reinforcement(), None, "no phase brings units yet");
    for (id, unit) in [("late", 10), ("soon", 11), ("soon", 12), ("hurt", 13)] {
        p.reserve(id, unit);
    }
    // "empty" brings no units and "hurt" has no round: the soonest with units is "soon" (2 units, round 3)
    assert_eq!(p.next_reinforcement(), Some((2, 3)));
    p.fired.insert("soon".into(), (3, 0));
    assert_eq!(p.next_reinforcement(), Some((1, 5)));
    p.dropped.push("late".into());
    assert_eq!(p.next_reinforcement(), None);
}
