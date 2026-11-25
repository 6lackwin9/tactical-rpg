//! INPUT1-PB (docs/specs/INPUT1.md PB4) on scratch catalogs built here (no shipped data): the combat log in the parts it
//! was written in (`Session::log_groups`, from `log_marks`). Expectations come from the rules: the parts read in order
//! are the log exactly (`log_lines(0, 0)`); each popped action's line sits under the block it resolved in (its last
//! block); a phase line said at a block's start sits under that block; under the round intent the "[Round]" summary
//! line sits in the round's end part; no part is empty and the parts run in order (round start, the blocks, the end).

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::session::{EncounterDef, LogGroup, Phase, Session, LOG_ROUND_END, LOG_ROUND_START};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;

fn catalog() -> Arc<Catalog> {
    let unit = |id: &str, name: &str, side: &str| json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Arc::new(Catalog::from_value(&json!({
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
        ],
        "units": [unit("k_ash", "Ash", "party"), unit("k_bryn", "Bryn", "party"), unit("k_brute", "Brute", "enemy"), unit("k_guard", "Guard", "enemy")],
    })))
}

/// A fight on an open floor, the party next to the enemies, with a phase line said at round 1 block 3's start.
fn session(ruleset: Ruleset) -> Session {
    let phases = json!([{"id": "mid", "name": "Mid", "trigger": {"kind": "round", "round": 1, "block": 3}, "effects": [{"type": "line", "text": "Hold the line."}]}]);
    let row = json!({"id": "parts", "name": "Parts", "rows": vec![".".repeat(12); 9], "party": ["k_ash", "k_bryn"], "enemies": ["k_brute", "k_guard"],
        "phases": phases, "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 2, "max_rounds": 4, "rng_seed": 7});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(catalog(), def, &IndexMap::new());
    for (c, cell) in [(0, V2::new(4, 4)), (1, V2::new(4, 5)), (2, V2::new(5, 4)), (3, V2::new(5, 5))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.replan_enemies();
    s
}

fn timeline() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT], "basics": ["move", "rest", "strike", "guard"]}).as_object().unwrap())
}

/// Plays `rounds` rounds with the party's aggressive draft; each popped action's (round, block, line) is noted.
fn play(s: &mut Session, rounds: i32) -> Vec<(i32, i32, String)> {
    let mut popped = Vec::new();
    for r in 1..=rounds {
        let presets: Vec<(usize, String)> = s.w.allies.iter().map(|&a| (a, "aggressive".to_string())).collect();
        rpgkit_battle::ai::plan_party_intents(&mut s.w, &mut s.plan, &presets);
        assert!(s.confirm_plan(), "round {r} confirms");
        while s.phase != Phase::Replay {
            let before = s.resolver.log.len();
            s.step_next();
            if let Some((round, block, idx)) = s.last_popped.filter(|p| p.0 == s.current_round) {
                let line = s.resolver.scheduled[idx].command.log.clone();
                if !line.is_empty() && s.resolver.log[before..].contains(&line) {
                    popped.push((round, block, line));
                }
            }
        }
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    popped
}

/// A part's place in its round: the start, the blocks in order, the end.
fn order(g: &LogGroup) -> (i32, i32) {
    (g.round, if g.block == LOG_ROUND_END { i32::MAX } else { g.block })
}

fn check_parts(s: &Session, popped: &[(i32, i32, String)]) -> Vec<LogGroup> {
    let groups = s.log_groups();
    let flat: Vec<String> = groups.iter().flat_map(|g| g.lines.clone()).collect();
    assert_eq!(flat, s.log_lines(0, 0), "the parts read in order are the log");
    assert!(groups.iter().all(|g| !g.lines.is_empty()), "no empty part");
    assert!(groups.windows(2).all(|w| order(&w[0]) < order(&w[1])), "the parts run in order: {:?}", groups.iter().map(|g| (g.round, g.block)).collect::<Vec<_>>());
    assert!(groups.iter().all(|g| g.block == LOG_ROUND_START || g.block == LOG_ROUND_END || (1..=BLOCKS).contains(&g.block)));
    assert!(!popped.is_empty(), "actions resolved");
    for (round, block, line) in popped {
        assert!(groups.iter().any(|g| (g.round, g.block) == (*round, *block) && g.lines.contains(line)), "{line} under round {round} block {block}");
    }
    let phase = groups.iter().find(|g| g.lines.iter().any(|l| l == "[Phase] Mid: Hold the line.")).expect("the phase line");
    assert_eq!((phase.round, phase.block), (1, 3), "said at block 3's start");
    groups
}

#[test]
fn each_line_sits_in_the_part_that_wrote_it() {
    let mut s = session(Ruleset::classic());
    let popped = play(&mut s, 2);
    check_parts(&s, &popped);
}

#[test]
fn the_round_summary_sits_in_the_rounds_end_part() {
    let mut s = session(timeline());
    let popped = play(&mut s, 2);
    let groups = check_parts(&s, &popped);
    let summaries: Vec<&LogGroup> = groups.iter().filter(|g| g.lines.iter().any(|l| l.starts_with("[Round]"))).collect();
    assert!(!summaries.is_empty(), "the round intent closes each round with its summary");
    assert!(summaries.iter().all(|g| g.block == LOG_ROUND_END && g.lines.last().is_some_and(|l| l.starts_with("[Round]"))), "the summary ends its round's end part");
}

#[test]
fn a_new_battle_has_no_part_but_its_start() {
    let s = session(Ruleset::classic());
    assert!(s.log_marks.iter().all(|m| (m.1, m.2) == (1, LOG_ROUND_START)));
    assert!(s.log_groups().iter().all(|g| (g.round, g.block) == (1, LOG_ROUND_START)));
}
