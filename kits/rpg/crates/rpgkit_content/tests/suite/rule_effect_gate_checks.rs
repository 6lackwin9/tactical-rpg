//! ACT1-HOOKS2 (FB-244 (2)), content side of a meter rule's effect gates: each effect of a rule may carry `when` (flag
//! conditions, checked as any condition list) and `else` (true or false); an `else` effect needs an effect before it
//! and is refused after an effect of its chain that always runs (it could never run); both keys belong to meter rules
//! only (a trigger's effect refuses them as unknown keys). A negative `add_flag` floored by `min` passes in a rule.

use aegis_core::json::{self, Dict};
use rpgkit_content::check::{campaign, View};
use serde_json::{json, Value};

const RULE: &str = "Rule 'r_zero'";

fn view() -> View {
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W", "campaign": {"start_act": 1}}, "own": {}}});
    View::new(&json!({}), &worlds, &json!({}), Default::default())
}

fn meter(effects: Value) -> Dict {
    json::dict(&json!({"id": "gauge", "name": "Gauge", "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}],
        "rules": [{"id": "r_zero", "at": 10, "dir": "down", "effects": effects}], "inputs": [], "modifiers": [], "floors": [], "hud": {}}))
    .clone()
}

fn errors(effects: Value) -> Vec<String> {
    campaign::meter_errors(&view(), "w", &meter(effects))
}

fn set(key: &str) -> Value {
    json!({"effect": "set_flag", "key": key})
}

fn not_set(key: &str) -> Value {
    json!({"key": key, "op": "!=", "value": true})
}

#[test]
fn an_if_else_chain_passes() {
    let chain = json!([
        {"effect": "add_flag", "key": "w.count", "delta": 1},
        {"effect": "set_flag", "key": "w.first", "when": [not_set("w.a"), not_set("w.first")]},
        {"effect": "set_flag", "key": "w.second", "else": true, "when": [not_set("w.b"), not_set("w.second")]},
        {"effect": "add_flag", "key": "w.pool", "delta": -2, "min": 0, "else": true}
    ]);
    assert_eq!(errors(chain), Vec::<String>::new());
    assert!(errors(json!([{"effect": "add_flag", "key": "w.pool", "delta": -2, "min": 0}])).is_empty(), "a floored negative add_flag");
    assert!(errors(json!([{"effect": "set_flag", "key": "w.x", "when": [], "else": false}])).is_empty(), "empty when, else false");
}

#[test]
fn a_bad_gate_names_its_problem() {
    let one = |fx: Value| errors(json!([fx]));
    assert_eq!(one(json!({"effect": "set_flag", "key": "w.x", "else": true})), [format!("{RULE} effect 1: else needs an effect before it.")]);
    assert_eq!(one(json!({"effect": "set_flag", "key": "w.x", "else": "yes"})), [format!("{RULE} effect 1: else must be true or false.")]);
    assert_eq!(one(json!({"effect": "set_flag", "key": "w.x", "when": {"key": "w.a"}})), [format!("{RULE} effect 1: when must be a list of {{\"key\", \"value\"}} conditions.")]);
    assert_eq!(one(json!({"effect": "set_flag", "key": "w.x", "when": [{"key": "w.a", "op": "~"}]})), [format!("{RULE} effect 1: when 1: op '~' must be one of == != >= <= > <.")]);
    // an else after an effect that always runs never runs
    let dead = errors(json!([set("w.x"), {"effect": "set_flag", "key": "w.y", "else": true}]));
    assert_eq!(dead, [format!("{RULE} effect 2: else never runs (an earlier effect of its chain has no when, so it always runs).")]);
    let dead_late = errors(json!([{"effect": "set_flag", "key": "w.x", "when": [not_set("w.a")]}, {"effect": "set_flag", "key": "w.y", "else": true}, {"effect": "set_flag", "key": "w.z", "else": true}]));
    assert_eq!(dead_late, [format!("{RULE} effect 3: else never runs (an earlier effect of its chain has no when, so it always runs).")]);
}

#[test]
fn the_gates_are_meter_rule_keys_only() {
    let trigger = json::dict(&json!({"id": "t", "on": "any", "effects": [{"effect": "set_flag", "key": "w.x", "when": [not_set("w.a")]}]})).clone();
    let got = campaign::trigger_errors(&view(), "w", &trigger);
    assert!(got.iter().any(|e| e.ends_with("unknown key 'when'.")), "{got:?}");
    // the effect itself is still checked under its gate
    assert_eq!(errors(json!([{"effect": "add_flag", "key": "w.pool", "delta": 0, "when": [not_set("w.a")]}])), [format!("{RULE} effect 1 (add_flag): delta must be a whole number from -100 to 100, not 0.")]);
}
