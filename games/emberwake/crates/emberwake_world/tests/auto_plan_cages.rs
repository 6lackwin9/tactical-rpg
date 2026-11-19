//! PR7-41 (PT-16, PT-22) on the agent systems playtest's save `cages_start` (`games/emberwake/scenarios/saves/`, the
//! playtest of 2026-11-18): The Crossroads Cages started from the saved party, its first round drafted the way the
//! battle screen drafts it (each member's tactic ghost in party order, `plan_apply_preset` per member). The drafted
//! plan confirms; with the test-only switch `regrant` on (the parked PT-22 re-grant rule, FB-334;
//! `AEGIS_SWITCHES=regrant`), no member's auto round spends its AP re-granting a status it already holds. Which units,
//! skills and tiles are involved is read from the battle, not hard-coded; shipped data is only read.

use aegis_core::{json, switches};
use emberwake_battle::session::Session;
use emberwake_world::{battle, save, Data};
use serde_json::json;
use std::path::{Path, PathBuf};

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

fn cages_session(d: &Data) -> Session {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/saves/cages_start/slot_0.json");
    let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let mut gs = save::decode(d, &text).expect("the save loads").state;
    let fight = json!({"encounter": "enc_evt_02"});
    battle::begin_world(d, &mut gs, json::dict(&fight)).expect("the Cages fight starts").session
}

/// The battle screen's planning start: every party member drafts its tactic's ghost, in party order.
fn ghost_draft(s: &mut Session, tactic: &str) {
    for m in s.w.allies.clone() {
        s.plan_apply_preset(m, tactic);
    }
}

fn plan_lines(s: &Session) -> Vec<String> {
    s.w.allies
        .iter()
        .map(|&a| {
            let st = s.plan.compile(&s.w, a);
            let steps: Vec<String> = st.entries.iter().map(|e| format!("{}@{}", s.w.cat.skills[e.skill].id, e.start_block)).collect();
            format!("{}: valid {} ({}) ap_left {} [{}]", s.w.units[a].name, st.error_index < 0, st.error_text, st.ap_left, steps.join(", "))
        })
        .collect()
}

#[test]
fn the_cages_ghost_draft_confirms() {
    let d = data();
    for tactic in ["balanced", "focus_weakest", "defensive"] {
        let mut s = cages_session(&d);
        ghost_draft(&mut s, tactic);
        let lines = plan_lines(&s);
        assert!(s.plan_is_valid(), "{tactic}: every member's drafted round is valid: {lines:#?}");
        // PT-22: no auto round grants its unit the same status twice (the re-grant rule, switch `regrant`; off, Vesper's
        // Fade loop still plans here)
        for &a in s.w.allies.iter().filter(|_| switches::on("regrant")) {
            let grants: Vec<&str> = s.plan.compile(&s.w, a).entries.iter().map(|e| s.w.cat.skills[e.skill].grant_status.as_str()).filter(|g| !g.is_empty()).collect();
            let mut seen: Vec<&str> = Vec::new();
            for g in grants {
                assert!(!seen.contains(&g), "{tactic}: {} grants {g} twice: {lines:#?}", s.w.units[a].name);
                seen.push(g);
            }
        }
        assert!(s.confirm_plan(), "{tactic}: the drafted plan confirms: {lines:#?}");
    }
}
