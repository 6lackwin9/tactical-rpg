//! KIT1-SIM: the kit sim names no world; the game passes its default world and story run (`SimGame`). Checked on a
//! scratch world built here: the story chain and its branch flag, the campaign mode without a story run, and the
//! usage and `extra=` lines that carry the game's values.

use rpgkit_sim::{world, SimGame, StoryRun};
use rpgkit_world::Data;
use serde_json::json;

/// World `vale`: an auto-start root quest whose step flag `vale.raid` unlocks a scripted route raid, a quest that
/// requires it with a variant on the choice flag `vale.choice`, and a side quest outside the chain.
fn data() -> Data {
    let worlds = json!({
        "vale": {
            "manifest": {"id": "vale"},
            "own": {
                "quests": [
                    {"id": "q_root", "auto_start": true, "steps": [{"encounter": "e_root", "flag": "vale.raid"}]},
                    {"id": "q_side", "steps": [{"encounter": "e_side"}]},
                    {"id": "q_next", "requires": ["q_root"], "steps": [{"encounter": "e_next"}],
                     "variants": [{"when_flags": [{"key": "vale.choice"}], "set": {"steps": {"0": {"encounter": "e_next_alt"}}}}]}
                ],
                "routes": [{"id": "r1", "scripted": [{"requires_flags": [{"key": "vale.raid"}], "encounter": "vale:e_raid"}]}]
            }
        }
    });
    Data::from_values(&json!({}), &worlds, &json!({}))
}

fn story() -> StoryRun {
    StoryRun {
        world: "vale".into(),
        branch_flag: "vale.choice".into(),
        choice: "path".into(),
        branches: vec![("left".into(), false), ("right".into(), true)],
        usage: "       campaign [level]   the vale story".into(),
        extra_example: "2:e_side".into(),
    }
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

#[test]
fn the_story_is_the_auto_start_chain_branched_on_the_choice_flag() {
    let d = data();
    let names = |on: bool| world::story_fights(&d, &story(), on).iter().map(|f| f.encounter.clone()).collect::<Vec<_>>();
    assert_eq!(names(false), ["e_root", "e_raid", "e_next"], "root, its flag's scripted raid, then the quest requiring it");
    assert_eq!(names(true), ["e_root", "e_raid", "e_next_alt"], "the variant on the choice flag replaces the step");
    let quests: Vec<String> = world::story_quests(&d, "vale").iter().map(|q| aegis_core::json::s(q, "id")).collect();
    assert_eq!(quests, ["q_root", "q_next"], "the side quest is not in the chain");
}

#[test]
fn campaign_without_a_story_run_says_so() {
    let d = data();
    let out = rpgkit_sim::run(&d, &argv("campaign 5 1"), &SimGame::default());
    assert!(out[0].starts_with("ARGS mode campaign"), "{out:?}");
    assert!(out.iter().any(|l| l == "CAMPAIGN: this game has no story run"), "{out:?}");
}

#[test]
fn usage_and_extra_carry_the_games_values() {
    let d = data();
    let game = SimGame { default_world: "vale".into(), story: Some(story()) };
    let out = rpgkit_sim::run(&d, &argv("fly"), &game);
    assert_eq!(out[0], "SIM argument error: unknown mode 'fly'");
    let sweep = out.iter().position(|l| l.trim_start().starts_with("sweep [level]")).expect("the sweep line");
    assert_eq!(out[sweep + 1], story().usage, "the story's campaign line follows the sweep line");
    assert!(out.iter().any(|l| l.trim_start().starts_with("world=<id> (vale): ")), "{out:?}");
    let none = rpgkit_sim::run(&d, &argv("fly"), &SimGame::default());
    assert!(none[sweep + 1].contains("this game has none"), "{}", none[sweep + 1]);
    assert_eq!(none.len(), out.len(), "the same help text, line for line");
    let err = rpgkit_sim::run(&d, &argv("campaign 5 1 extra=2:nowhere"), &game);
    assert_eq!(err[0], "SIM argument error: extra must be <battle>:<vale encounter>[,..], e.g. extra=2:e_side (got '2:nowhere')");
}
