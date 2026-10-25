//! RULESET1 V3 on scratch worlds: the world manifest's `plan_reveal_flag` names a game-state flag; the bridge
//! (`battle::encounter_def_in`) sets the battle's `plan_reveal` when the flag holds (default value true), read along
//! the lineage like the ruleset (the world's own, else the nearest parent's); no flag, or the flag unset, reveals
//! nothing; a row alone never sets it.

use rpgkit_world::state::GameState;
use rpgkit_world::{battle, Data};
use serde_json::{json, Value};

/// A names `a.scouted` and owns `a_fight`; B extends A with nothing (inherits); C extends A and names its own flag;
/// D names none.
fn data() -> Data {
    let worlds = json!({
        "a": {"manifest": {"id": "a", "ruleset": "timeline", "plan_reveal_flag": "a.scouted"}, "own": {"encounters": [{"id": "a_fight", "name": "A fight", "enemies": []}]}},
        "b": {"manifest": {"id": "b", "extends": ["a"]}, "own": {}},
        "c": {"manifest": {"id": "c", "extends": ["a"], "plan_reveal_flag": "c_spies"}, "own": {}},
        "d": {"manifest": {"id": "d"}, "own": {}}
    });
    Data::from_values(&json!({"rulesets": [{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "plan_badges"]}]}), &worlds, &json!({}))
}

#[test]
fn the_world_flag_sets_the_battle_reveal() {
    let d = data();
    let flags: Vec<String> = ["a", "b", "c", "d"].iter().map(|w| battle::plan_reveal_flag(&d, w)).collect();
    assert_eq!(flags, ["a.scouted", "a.scouted", "c_spies", ""]);
    let entry = d.resolve("b", "encounters", "a_fight").clone();
    let mut gs = GameState::default();
    assert!(!battle::encounter_def_in(&d, &gs, "b", &entry).plan_reveal, "the flag is not set yet");
    assert_eq!(gs.set_flag("a.scouted", Value::Bool(true)), "");
    for (world, reveals) in [("a", true), ("b", true), ("c", false), ("d", false)] {
        let def = battle::encounter_def_in(&d, &gs, world, &entry);
        assert_eq!(def.plan_reveal, reveals, "{world}");
        assert_eq!(def.ruleset, d.ruleset(world), "{world}: the ruleset as encounter_def gives it");
    }
    assert!(!battle::encounter_def(&d, "a", &entry).plan_reveal, "encounter_def alone reads no game state");
    assert_eq!(gs.set_flag("a.scouted", Value::Bool(false)), "");
    assert!(!battle::plan_reveal(&d, &gs, "a"), "a false flag reveals nothing");
}
