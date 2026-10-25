//! RULESET1 3.1.1-3.1.3 on scratch worlds: a world's ruleset is its own `ruleset`, else the nearest parent's, else
//! classic; a battle built by the world bridge (`battle::encounter_def`) plays by the ruleset of the world that
//! resolves the encounter, whichever world wrote the row; a timeline world without its row plays classic (the content
//! check reports it).

use rpgkit_rules::ruleset::{Ruleset, CLASSIC, ROUND_INTENT, TIMELINE};
use rpgkit_world::{battle, Data};
use serde_json::{json, Value};

/// A timeline; B extends A with "" (inherits); C sets nothing; D extends C and A (nearest parent last in `extends`
/// wins, as for every inherited manifest field); E sets classic under A. A owns encounter `a_fight`.
fn worlds() -> Value {
    json!({
        "a": {"manifest": {"id": "a", "ruleset": "timeline"}, "own": {"encounters": [{"id": "a_fight", "name": "A fight", "enemies": []}]}},
        "b": {"manifest": {"id": "b", "extends": ["a"], "ruleset": ""}, "own": {}},
        "c": {"manifest": {"id": "c"}, "own": {}},
        "d": {"manifest": {"id": "d", "extends": ["c", "a"]}, "own": {}},
        "e": {"manifest": {"id": "e", "extends": ["a"], "ruleset": "classic"}, "own": {}}
    })
}

fn data(with_row: bool) -> Data {
    let rows = if with_row { json!([{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "poise"]}]) } else { json!([]) };
    Data::from_values(&json!({"rulesets": rows}), &worlds(), &json!({}))
}

#[test]
fn a_world_inherits_its_ruleset_along_the_lineage_else_classic() {
    let d = data(true);
    let ids: Vec<String> = ["a", "b", "c", "d", "e"].iter().map(|w| d.ruleset_id(w)).collect();
    assert_eq!(ids, [TIMELINE, TIMELINE, CLASSIC, TIMELINE, CLASSIC]);
    assert_eq!(d.ruleset_id("nowhere"), CLASSIC, "an unknown world names nothing");
    let r = d.ruleset("b");
    assert_eq!((r.id.as_str(), r.has(ROUND_INTENT), r.stances()), (TIMELINE, true, false));
    assert_eq!(d.ruleset("c"), Ruleset::classic());
}

#[test]
fn the_bridge_gives_the_battle_the_ruleset_of_the_resolving_world() {
    let d = data(true);
    let entry = d.resolve("b", "encounters", "a_fight").clone();
    assert!(!entry.is_empty(), "b sees a's encounter");
    assert_eq!(battle::encounter_def(&d, "b", &entry).ruleset, d.ruleset("b"));
    assert_eq!(battle::encounter_def(&d, "e", &entry).ruleset, Ruleset::classic(), "the same row resolved in a classic world");
    assert_eq!(rpgkit_battle::session::EncounterDef::from_obj(&entry).ruleset, Ruleset::classic(), "a row alone never sets the ruleset");
}

#[test]
fn a_timeline_world_without_its_row_plays_classic() {
    let d = data(false);
    assert_eq!(d.ruleset_id("a"), TIMELINE);
    assert_eq!(d.ruleset("a"), Ruleset::classic());
}
