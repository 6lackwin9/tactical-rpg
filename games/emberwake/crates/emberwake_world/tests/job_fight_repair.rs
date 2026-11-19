//! PR7-41 (PT-15): the playtest save `m3_stuck` (`games/emberwake/scenarios/saves/m3_stuck`, the agent systems playtest
//! 2026-11-18) holds a board job left active at its fight although that fight was won before the job was taken. Loaded
//! through the game's save façade it is repaired: every such job turns ready (with a load warning) and turns in. Which
//! job is stuck is read from the save and the rules, not hard-coded; shipped data is only read.

use aegis_core::json;
use emberwake_world::{board, quest, save, Data, GameState};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

fn save_text() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/saves/m3_stuck/slot_0.json");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// The jobs the raw save holds as active on a `win_encounter` objective whose fight the save shows as won.
fn stuck_jobs(d: &Data, gs: &GameState) -> Vec<String> {
    gs.quests
        .keys()
        .filter(|qid| {
            let cur = quest::current_objective(d, gs, qid);
            json::s(&cur, "kind") == "win_encounter" && quest::encounter_won(d, gs, &json::s(&cur, "target"))
        })
        .cloned()
        .collect()
}

#[test]
fn the_m3_stuck_save_loads_with_its_won_job_ready_to_turn_in() {
    let d = data();
    let text = save_text();
    // the raw state, read without the repair, is stuck on a job of a board
    let envelope: Value = serde_json::from_str(&text).expect("json");
    let raw = GameState::from_dict(&d, json::dget(json::dict(&envelope), "game_state"));
    let stuck = stuck_jobs(&d, &raw);
    assert!(!stuck.is_empty(), "the playtest save holds a job stuck on a won fight");
    for qid in &stuck {
        assert_eq!(quest::status(&d, &raw, qid), "active");
        assert!(board::on_a_board(&quest::resolved(&d, &raw, qid)), "{qid} is a job");
    }

    let loaded = save::decode(&d, &text).expect("the save loads");
    let gs = loaded.state;
    assert!(stuck_jobs(&d, &gs).is_empty(), "no job is left on a won fight");
    for qid in &stuck {
        assert_eq!(quest::status(&d, &gs, qid), "ready", "{qid} is ready to turn in");
        assert!(gs.flag_true(&quest::ready_flag(qid)));
        let name = json::s(&quest::resolved(&d, &gs, qid), "name");
        assert!(loaded.warnings.iter().any(|w| w.starts_with(&format!("{name}: ")) && !w.contains(qid.as_str())), "a load warning names {qid} in player words: {:?}", loaded.warnings);
    }
    // nothing else moved: every other quest keeps its saved state
    for (qid, v) in &raw.quests {
        if !stuck.contains(qid) {
            assert_eq!(gs.quests.get(qid), Some(v), "{qid} unchanged");
        }
    }

    // and it plays on: each repaired job turns in and its complete flags are set
    let mut gs = gs;
    let recipient = gs.party.active_ids.first().cloned().expect("an active member");
    for qid in &stuck {
        let done = quest::complete(&d, &mut gs, qid, &recipient, &[]);
        assert!(json::b(&done, "ok", false), "{qid} turns in: {done:?}");
        assert_eq!(quest::status(&d, &gs, qid), "completed");
        for c in json::aget(&quest::resolved(&d, &gs, qid), "complete_flags") {
            assert!(quest::condition_met(&gs, json::dict(c)), "{qid}: complete flag {c} is set");
        }
    }

    // a repaired save writes and reads back unchanged, with no further warning
    let dir = aegis_core::scratch::Scratch::vacant("emberwake_m3_stuck");
    assert!(save::write(&d, &dir, &gs, 1, 1).is_empty());
    let back = save::read(&d, &dir, 1).expect("reads back");
    assert!(back.warnings.iter().all(|w| !w.contains("already won")), "{:?}", back.warnings);
    assert_eq!(back.state, gs);
}
