//! PR5-33 (LAYER-UP-DATA): a game's sim description read from JSON (`rpgkit_sim::config`): every key the game used to
//! write as constants (default world, story run, feel encounters, classes, roles and loadouts, generated rosters, role
//! limit tests, the S2 roster) reads into `SimGame` as given; a missing key is the `SimGame` default; a malformed one is
//! refused with its key. Checked on a made-up game (classes `knight`, `archer`).

use rpgkit_sim::config::{game_from_json, game_from_value};
use rpgkit_sim::{roles, scenarios};
use serde_json::json;

fn full() -> serde_json::Value {
    json!({
        "_doc": "a made-up game",
        "default_world": "vale",
        "story": {"world": "vale", "branch_flag": "vale.choice", "choice": "path", "branches": [["left", false], ["right", true]],
                  "usage": "       campaign [level]   the vale story", "extra_example": "2:e_side"},
        "feel_encounters": ["t_one", "t_two"],
        "classes": {
            "ids": ["knight", "archer"], "default_comp": ["knight", "archer"], "trio_lead": "knight", "elite": "knight", "boss": "archer",
            "support_baseline": "archer",
            "roles": [
                {"class": "knight", "primary": "tank", "secondary": ["support"],
                 "loadouts": [{"role": "tank", "skills": ["bash", "wall"]}, {"role": "support", "skills": ["shout"]}],
                 "l7_swaps": [{"role": "tank", "skill": "wall", "with": "fortress"}]},
                {"class": "archer", "primary": "ranged", "secondary": [], "loadouts": [{"role": "ranged", "skills": ["shot"]}], "l7_swaps": []}
            ],
            "generated": [{"id": "mob", "enemies": [["knight", "minion"], ["archer", "minion"]], "wave": [["@boss", "boss"]]}],
            "scenarios": [
                {"id": "tank_hold", "role": "tank", "allies": ["archer"], "enemies": [["knight", "standard"]], "priority": false, "block": [2, 3], "tier_until": null},
                {"id": "striker_priority", "role": "striker", "allies": [], "enemies": [["archer", "elite"]], "priority": true, "block": [0, 0],
                 "tier_until": {"level": 3, "slot": 0, "tier": "standard"}}
            ],
            "s2_enemies": [["archer", "standard"]]
        }
    })
}

#[test]
fn every_key_reads_as_given() {
    let g = game_from_value(&full()).unwrap();
    assert_eq!(g.default_world, "vale");
    assert_eq!(g.feel_encounters, ["t_one", "t_two"]);
    let s = g.story.as_ref().unwrap();
    assert_eq!((s.world.as_str(), s.branch_flag.as_str(), s.choice.as_str(), s.extra_example.as_str()), ("vale", "vale.choice", "path", "2:e_side"));
    assert_eq!(s.branches, [("left".to_string(), false), ("right".to_string(), true)]);
    let c = g.classes;
    assert_eq!((c.ids, c.default_comp, c.trio_lead, c.elite, c.boss, c.support_baseline), (&["knight", "archer"][..], &["knight", "archer"][..], "knight", "knight", "archer", "archer"));
    assert_eq!(roles::primary_role(&c, "knight"), "tank");
    assert_eq!(roles::secondary_roles(&c, "knight"), ["support"]);
    assert_eq!(c.roles[0].loadouts, [("tank", &["bash", "wall"][..]), ("support", &["shout"][..])]);
    assert_eq!(c.roles[0].l7_swaps, [("tank", "wall", "fortress")]);
    assert_eq!(scenarios::generated(&c, "mob"), Some((&[("knight", "minion"), ("archer", "minion")][..], &[("@boss", "boss")][..])));
    let hold = scenarios::scenario_def(&c, "tank_hold").unwrap();
    assert_eq!((hold.allies, hold.enemies, hold.priority, hold.block, hold.tier_until), (&["archer"][..], &[("knight", "standard")][..], false, (2, 3), None));
    assert_eq!(scenarios::scenario_def(&c, "striker_priority").unwrap().tier_until, Some((3, 0, "standard")));
    assert_eq!(c.s2_enemies, [("archer", "standard")]);
    let text = full().to_string();
    assert_eq!(format!("{:?}", game_from_json(&text).unwrap().classes), format!("{c:?}"), "the text reads the same");
}

#[test]
fn a_missing_key_is_the_default() {
    let g = game_from_value(&json!({})).unwrap();
    assert_eq!((g.default_world.as_str(), g.story.is_none(), g.feel_encounters.is_empty(), g.classes.ids.is_empty()), ("", true, true, true));
    let g = game_from_value(&json!({"classes": {"ids": ["knight"]}})).unwrap();
    assert!(g.classes.roles.is_empty() && g.classes.scenarios.is_empty() && g.classes.trio_lead.is_empty());
}

#[test]
fn a_malformed_key_is_refused_by_name() {
    let err = |v: serde_json::Value| game_from_value(&v).unwrap_err();
    assert!(err(json!([])).contains("not an object"));
    assert!(err(json!({"default_world": 3})).contains("`default_world`"));
    assert!(err(json!({"feel_encounters": "t_one"})).contains("`feel_encounters`"));
    assert!(err(json!({"classes": {"s2_enemies": [["archer"]]}})).contains("`s2_enemies`"));
    assert!(err(json!({"story": {"branches": [["left", "yes"]]}})).contains("story branch"));
    assert!(err(json!({"classes": {"scenarios": [{"id": "x", "block": [1]}]}})).contains("block"));
    assert!(game_from_json("{").unwrap_err().starts_with("sim config:"));
}
