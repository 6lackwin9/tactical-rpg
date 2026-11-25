//! PR7-42 (PACE1, PT-39) on the shipped data (read only): the second story fight of a new game says what it asks.
//! Its hint asks for no escape unless the objective is one, and every exit tile it keeps can be reached: walkable and
//! on a path from a party unit's start (an exit removed leaves nothing to reach). Nothing about the fight is named.

use aegis_core::json;
use emberwake_battle::grid::V2;
use emberwake_battle::path::{self, Cells, Hooks};
use emberwake_battle::session::Session;
use emberwake_data::Step;
use emberwake_world::{battle, new_game, story, Data};
use std::path::Path;

const WORLD: &str = "sunk_ward";
/// Words that tell the player to leave the field.
const ESCAPE_WORDS: [&str; 4] = ["slip away", "escape", "flee", "get out"];

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

/// The `n`th story fight of a new game (each earlier one won): (its encounter entry, the battle at its start).
fn story_fight(d: &Data, n: usize) -> (aegis_core::json::Dict, Session) {
    let mut gs = new_game::create_for_world(d, WORLD, 1).expect("a new game");
    let mut seen = 0;
    for _ in 0..200 {
        match story::current(d, &gs) {
            Some(Step::Vignette { vignette }) => assert!(json::b(&story::step_done(d, &mut gs), "ok", false), "{vignette}"),
            Some(Step::Fight { encounter, .. }) => {
                let fight = story::fight(d, &gs).expect("a story fight at a fight step");
                seen += 1;
                if seen == n {
                    let entry = d.resolve(WORLD, "encounters", &encounter).clone();
                    println!("story fight {n}: {encounter}");
                    return (entry, battle::begin_world(d, &mut gs, &fight).expect("the fight starts").session);
                }
                let r = story::after_fight(d, &mut gs, &fight, "won", None);
                assert!(json::b(&r, "ok", false), "fight {seen} settles: {:?}", json::sget(&r, "errors"));
            }
            _ => break,
        }
    }
    panic!("a new game reaches story fight {n}");
}

#[test]
fn the_second_fights_hint_matches_its_objective_and_its_exits_are_reachable() {
    let d = data();
    let (e, s) = story_fight(&d, 2);
    let (hint, objective) = (json::s(&e, "objective_text").to_lowercase(), json::s(&e, "objective"));
    if objective != "escape" {
        assert!(ESCAPE_WORDS.iter().all(|w| !hint.contains(w)), "objective {objective}: the hint asks for no escape: {hint}");
    }
    let starts: Vec<V2> = s.w.allies.iter().filter_map(|&a| s.w.grid.positions[a]).collect();
    assert!(!starts.is_empty(), "the party stands on the field");
    for &x in &s.w.grid.exits {
        assert!(s.w.grid.is_walkable(x), "exit {x:?} is walkable");
        let goal: Cells = [x].into_iter().collect();
        assert!(starts.iter().any(|&p| !path::find_path(&s.w.grid, p, &goal, &Cells::new(), true, &Hooks::default()).is_empty()), "exit {x:?} is on a path from the party");
    }
}
