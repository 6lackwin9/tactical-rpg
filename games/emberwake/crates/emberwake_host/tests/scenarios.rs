//! HEADLESS1: the game's scenario files parse, so a change to the format or a bad edit fails the gate. No game starts here
//! (running them is `tools/fc.sh scenario all`).

use aegis_cli::scenario::{check_files, files_in};
use std::path::Path;

#[test]
fn the_shipped_scenarios_parse() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios");
    let files = files_in(&folder);
    assert!(!files.is_empty(), "no scenario files in {}", folder.display());
    let problems = check_files(&files);
    assert!(problems.is_empty(), "{problems:#?}");
}

/// ENGINE-5: the launch table is the game's (the engine's closed list moved here): the menu, the arena and the runner,
/// and a ruleset only with the arena, under either of its names.
#[test]
fn the_game_opens_on_its_launch_screens_and_plays_a_ruleset_only_in_the_arena() {
    use aegis_cli::scenario::{load, parse};
    use aegis_cli::Game;
    use emberwake_host::cli::Emberwake;
    use serde_json::json;
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios");
    for f in files_in(&folder) {
        let s = load(&f).expect("parses");
        assert_eq!(Emberwake.launch_problems(&s.launch), Vec::<String>::new(), "{}", f.display());
    }
    let launch = |l: serde_json::Value| parse("x", &json!({"id": "x", "launch": l, "steps": [{"step": "back"}]})).expect("parses").launch;
    let problems = |l| Emberwake.launch_problems(&launch(l)).join(" | ");
    assert!(problems(json!({"screen": "arena", "scene": "e1", "ruleset": "r1"})).is_empty());
    assert!(problems(json!({"screen": "runner", "ruleset": "r1"})).contains("ruleset"));
    assert!(problems(json!({"ruleset": "r1"})).contains("ruleset"), "the default screen is not the arena");
    assert!(problems(json!({"screen": "runner", "rules": "r1"})).contains("ruleset"), "the alias is held to the same rule");
    assert!(problems(json!({"screen": "credits"})).contains("not one this game opens on (menu, arena, runner)"));
}
