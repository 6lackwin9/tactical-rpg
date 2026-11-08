//! INTENT2-K1a (A4.2, IR21, `docs/sessions/A4_EFFECTS.md` section 2): each precedence step of `intent::derive` on
//! test-only rows, the first match winning over the later steps, and the rule gate (`field_intent` off = the legacy
//! single intent, as classic reads today; FB-106, the gate's own tests: `rpgkit_battle/tests/field_intent.rs`).

use rpgkit_rules::intent::{self, IntentInput, ASSAULT, FIELD, MANEUVER, STAND, SUPPORT};
use rpgkit_rules::ruleset::{Ruleset, FIELD_INTENT, NEUTRAL, ROUND_INTENT};
use serde_json::{json, Value};

fn row(v: Value) -> IntentInput {
    IntentInput::of_row(v.as_object().expect("an object"))
}

fn derive(v: Value) -> Vec<&'static str> {
    intent::derive(&row(v))
}

#[test]
fn engine_kinds_are_neutral_and_unknown_kinds_derive_nothing() {
    for kind in ["MOVE", "REST", "TARGET", "STATUS"] {
        assert_eq!(derive(json!({"kind": kind})), [NEUTRAL], "{kind}");
    }
    assert!(derive(json!({"kind": "BOGUS"})).is_empty());
}

#[test]
fn defend_focus_stand_even_with_a_hazard() {
    assert_eq!(derive(json!({"kind": "DEFEND"})), [STAND]);
    assert_eq!(derive(json!({"kind": "FOCUS"})), [STAND]);
    assert_eq!(derive(json!({"kind": "DEFEND", "hazard": {"id": "ash", "shape": "BURST"}})), [STAND], "step 2 before any hazard rule");
    assert_eq!(derive(json!({"kind": "DISENGAGE", "dash_tiles": 2})), [MANEUVER]);
}

#[test]
fn support_adds_stand_only_when_it_may_target_its_user() {
    assert_eq!(derive(json!({"kind": "SUPPORT"})), [SUPPORT, STAND], "self or a friend");
    assert_eq!(derive(json!({"kind": "SUPPORT", "target_mode": "ALLY"})), [SUPPORT], "another friend only");
    assert_eq!(derive(json!({"kind": "SUPPORT", "target_mode": "SELF"})), [STAND], "its user only");
}

#[test]
fn attack_gap_close_is_maneuver_before_any_shape() {
    assert_eq!(derive(json!({"kind": "ATTACK", "dash_tiles": 3})), [MANEUVER]);
    assert_eq!(derive(json!({"kind": "ATTACK", "position": "flank"})), [MANEUVER]);
    assert_eq!(derive(json!({"kind": "ATTACK", "dash_tiles": 2, "area": "LINE"})), [MANEUVER], "step 5 before step 6");
}

#[test]
fn shaped_attacks_by_shape_reach_and_hazard() {
    let hz = json!({"id": "fire", "shape": "SKILL", "at": "target", "rounds": 2});
    assert_eq!(derive(json!({"kind": "ATTACK", "area": "RING"})), [STAND]);
    assert_eq!(derive(json!({"kind": "ATTACK", "area": "RING", "hazard": hz})), [STAND], "RING is Stand whatever it places");
    for shape in ["ARC", "LINE", "CONE"] {
        assert_eq!(derive(json!({"kind": "ATTACK", "area": shape})), [FIELD, ASSAULT], "{shape} without a hazard");
        assert_eq!(derive(json!({"kind": "ATTACK", "area": shape, "reach_max": 1})), [FIELD, ASSAULT], "{shape}: reach not read");
        assert_eq!(derive(json!({"kind": "ATTACK", "area": shape, "hazard": hz})), [FIELD], "{shape} with a hazard");
    }
    for shape in ["BURST", "PLUS"] {
        assert_eq!(derive(json!({"kind": "ATTACK", "area": shape, "reach_max": 2})), [FIELD, ASSAULT], "{shape} reach 2");
        assert_eq!(derive(json!({"kind": "ATTACK", "area": shape, "reach_max": 1})), [FIELD], "{shape} reach 1");
        assert_eq!(derive(json!({"kind": "ATTACK", "area": shape})), [FIELD], "{shape} default reach");
        assert_eq!(derive(json!({"kind": "ATTACK", "area": shape, "reach_max": 3, "hazard": hz})), [FIELD], "{shape} with a hazard");
    }
}

#[test]
fn unshaped_attacks_hazard_field_else_assault_and_unknown_areas_are_unshaped() {
    let hz = json!({"id": "snare", "shape": "NONE", "at": "target", "rounds": 2});
    assert_eq!(derive(json!({"kind": "ATTACK", "hazard": hz})), [FIELD], "step 7 before step 8");
    assert_eq!(derive(json!({"kind": "ATTACK"})), [ASSAULT]);
    assert_eq!(derive(json!({"kind": "ATTACK", "area": "NONE"})), [ASSAULT]);
    assert_eq!(derive(json!({"kind": "ATTACK", "hazard": {}})), [ASSAULT], "an empty hazard places nothing");
    assert_eq!(derive(json!({"kind": "ATTACK", "area": "SWEEP"})), [ASSAULT], "the retired SWEEP (W2-4) is no IR21 shape");
    assert_eq!(derive(json!({"kind": "ATTACK", "area": "SWEEP", "hazard": hz})), [FIELD]);
}

#[test]
fn authored_round_intent_is_not_read() {
    assert_eq!(derive(json!({"kind": "ATTACK", "area": "BURST", "reach_max": 3, "round_intent": "assault"})), [FIELD, ASSAULT]);
    assert_eq!(derive(json!({"kind": "ATTACK", "round_intent": "maneuver"})), [ASSAULT]);
}

#[test]
fn rule_gate_keeps_the_legacy_single_intent_without_field_intent() {
    let classic = Ruleset::classic();
    assert!(!classic.has(ROUND_INTENT));
    let line = row(json!({"kind": "ATTACK", "area": "LINE"}));
    assert_eq!(intent::allowed(&classic, &line, ""), [ASSAULT], "kind default");
    assert_eq!(intent::allowed(&classic, &line, "maneuver"), [MANEUVER], "authored");
    assert_eq!(intent::allowed(&classic, &row(json!({"kind": "FOCUS"})), ""), [SUPPORT], "classic FOCUS stays support");
    let on = Ruleset { rules: vec![ROUND_INTENT, FIELD_INTENT], ..Ruleset::classic() };
    assert_eq!(intent::allowed(&on, &line, "maneuver"), [FIELD, ASSAULT], "derived under field_intent (FB-106)");
    assert!(intent::row_allows(&intent::derive(&line), ASSAULT));
    assert!(!intent::row_allows(&intent::derive(&line), STAND));
}
