//! A4.2-K1b (INTENT2 IR15, IR21) on fixture rows (no shipped data): a member's saved deck order reads the skills'
//! derived intent rows when a deck ruleset plays `field_intent` (a dual skill fits Field and Assault, a hazard throw
//! only Field, and `field` is an order key); without the rule the legacy single intent holds (`skill_intent`).

use rpgkit_world::state::UnitState;
use rpgkit_world::Data;
use serde_json::{json, Value};

fn data(rules: Value) -> Data {
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter", "skill_path": {}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true},
            {"id": "d_burst", "name": "Burst", "kind": "ATTACK", "level": 1, "blocks": 2, "reach_min": 2, "reach_max": 3, "area": "BURST"},
            {"id": "d_pot", "name": "Pot", "kind": "ATTACK", "level": 1, "blocks": 2, "reach_min": 2, "reach_max": 3, "area": "BURST", "hazard": {"id": "d_fire", "shape": "SKILL"}}
        ],
        "hazards": [{"id": "d_fire", "name": "Fire", "on": "round_end", "damage": 1, "duration": 1}],
        "rulesets": [{"id": "timeline", "name": "Timeline", "rules": rules, "basics": ["strike"], "deck_slots": {"active": 2, "passive": 0, "keystone": 0}}],
        "units": [{"id": "plain", "name": "Plain", "side": "party", "class_id": "fighter", "level": 1, "extra_skills": ["d_burst", "d_pot"]}]
    });
    Data::from_values(&catalog, &json!({}), &json!({}))
}

fn order_errors(d: &Data, order: Value) -> Vec<String> {
    let mut m = UnitState::create("m", "plain");
    m.deck.active = vec!["d_burst".into(), "d_pot".into()];
    for (k, v) in order.as_object().unwrap() {
        m.deck.order.insert(k.clone(), v.as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect());
    }
    m.deck_errors(d, "m").into_iter().filter(|e| e.contains("deck order")).collect()
}

#[test]
fn a_saved_deck_order_reads_the_derived_rows_under_field_intent() {
    let on = data(json!(["round_intent", "deck", "field_intent"]));
    let off = data(json!(["round_intent", "deck"]));
    let dual = json!({"field": ["d_burst"], "assault": ["d_burst", "strike"]});
    assert_eq!(order_errors(&on, dual.clone()), Vec::<String>::new(), "a dual skill in both rows; field is a key");
    assert_eq!(order_errors(&off, dual), ["m: deck order: 'field' is not an intent (one of assault, guard, support, maneuver, channel)."]);
    assert_eq!(order_errors(&on, json!({"assault": ["d_pot"]})), ["m: deck order assault: 'd_pot' is of intent field."], "a hazard throw is Field only");
    assert_eq!(order_errors(&off, json!({"assault": ["d_pot", "d_burst"]})), Vec::<String>::new(), "legacy: ATTACK is Assault");
}
