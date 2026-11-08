//! The round archive (Session::history; CombatSession.round_actions and round_snapshots): every round keeps its
//! actions and a snapshot of the units at its start and after each resolved block.

use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_world::battle::WorldRoster;
use emberwake_world::Data;
use std::path::Path;

fn sessions() -> Vec<Session> {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let d = Data::open(&dir).expect("data").0;
    d.own_entries("ashlands", "encounters")
        .iter()
        .map(|e| {
            let id = aegis_core::json::s(e, "id");
            let def = EncounterDef::from_obj(d.resolve("ashlands", "encounters", &id));
            Session::start(d.cat.clone(), def, &WorldRoster::new(&d, "ashlands"))
        })
        .collect()
}

/// Plays the current round out (bounded, so a stall fails instead of hanging).
fn resolve_round(s: &mut Session) {
    for _ in 0..10_000 {
        if s.phase == Phase::Replay {
            return;
        }
        s.step_next();
    }
    panic!("the round never ended");
}

#[test]
fn every_round_keeps_its_actions_and_a_snapshot_per_resolved_block() {
    // The first ashlands encounter that is still on after round 1 (round 2 must be archived too).
    let mut found = false;
    for mut s in sessions() {
        assert_eq!(s.history.len(), 1);
        assert_eq!(s.history[0].snapshots, vec![s.snapshot()], "the round's start");
        // Before the plan is confirmed the archive holds the enemies' drafts only; a replan refreshes it.
        assert!(s.history[0].actions.iter().all(|a| s.w.enemies.contains(&a.unit)), "enemy drafts only while planning");
        s.replan_enemies();
        assert_eq!(s.history[0].actions.len(), s.resolver.scheduled.len(), "a replan refreshes the archive");
        s.plan_apply_preset_all("balanced");
        assert!(s.confirm_plan());
        assert_eq!(s.history[0].actions.len(), s.resolver.scheduled.len(), "everyone's actions once confirmed");
        resolve_round(&mut s);
        let cancelled = s.resolver.scheduled.iter().filter(|a| a.cancelled).count();
        assert_eq!(s.history[0].actions.iter().filter(|a| a.cancelled).count(), cancelled, "cancellations archived");
        if s.combat_over {
            continue;
        }
        // A round that runs its course: the start plus one snapshot per block.
        assert_eq!(s.history[0].snapshots.len() as i32, s.def.total_blocks + 1, "one snapshot per resolved block");
        assert!(s.can_advance_round());
        s.advance_round();
        assert_eq!(s.history.len(), 2);
        assert_eq!(s.round_record(2).map(|r| r.snapshots.len()), Some(1));
        assert_eq!(s.round_record(2).map(|r| r.snapshots[0].clone()), Some(s.snapshot()), "round 2 starts from now");
        assert_eq!(s.round_record(1).map(|r| r.snapshots.len() as i32), Some(s.def.total_blocks + 1), "the past round stays");
        found = true;
        break;
    }
    assert!(found, "an ashlands encounter that lasts past round 1");
}
