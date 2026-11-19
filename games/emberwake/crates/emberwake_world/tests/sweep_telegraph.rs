//! PR7-41 (PT-17) on the agent systems playtest's save `sweep_start` (`games/emberwake/scenarios/saves/`, the playtest
//! of 2026-11-18): The Sweep started from the saved party and played a few rounds on each tactic. Every area attack
//! (Captain Vane's Gilded Sunder among them) hits no unit outside the tiles its tell marks while the party plans (the
//! kit's `intent::area_cells` over the round as it will run, `intent::planned_schedule`). Which units, skills and tiles
//! are involved is read from the battle, not hard-coded; shipped data is only read.

use aegis_core::json;
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::V2;
use emberwake_battle::intent;
use emberwake_battle::session::{Phase, Session};
use emberwake_world::{battle, save, Data};
use serde_json::json;
use std::path::{Path, PathBuf};

const ROUNDS: usize = 6;

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

fn sweep_session(d: &Data) -> Session {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/saves/sweep_start/slot_0.json");
    let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let mut gs = save::decode(d, &text).expect("the save loads").state;
    let fight = json!({"encounter": "enc_evt_01"});
    battle::begin_world(d, &mut gs, json::dict(&fight)).expect("the Sweep starts").session
}

/// The marked tiles of every enemy area action of the round (index into the schedule after confirm, tiles), read while
/// the party's plan is still a plan.
fn marks(s: &Session) -> Vec<(usize, String, Vec<V2>)> {
    let round = intent::planned_schedule(&s.w, &s.resolver.scheduled, &s.plan);
    let n = s.resolver.scheduled.len();
    round[..n].iter().enumerate().filter(|(_, a)| s.w.enemies.contains(&a.unit) && matches!(a.command.kind, CmdKind::Area { .. })).map(|(i, a)| (i, a.command.name.clone(), intent::area_cells(&s.w, &round, a))).collect()
}

#[test]
fn every_area_hit_lands_inside_its_marked_tiles() {
    let d = data();
    let mut areas = 0;
    for tactic in ["balanced", "focus_weakest", "defensive"] {
        let mut s = sweep_session(&d);
        for round in 1..=ROUNDS {
            for m in s.w.allies.clone() {
                s.plan_apply_preset(m, tactic);
            }
            let marked = marks(&s);
            assert!(s.confirm_plan(), "{tactic} round {round}: the drafted plan confirms");
            while s.phase != Phase::Replay && !s.combat_over {
                let before: Vec<Option<V2>> = (0..s.w.units.len()).map(|u| s.w.grid.pos(u)).collect();
                let landed: Vec<bool> = s.resolver.scheduled.iter().map(|a| a.landed).collect();
                s.step_next();
                for (i, name, tiles) in &marked {
                    let a = &s.resolver.scheduled[*i];
                    if landed.get(*i).copied().unwrap_or(true) || !a.landed {
                        continue;
                    }
                    areas += 1;
                    for (u, _) in &a.command.hits {
                        let at = before[*u].expect("a struck unit stood on the grid");
                        assert!(s.w.cells_at(*u, at).iter().any(|c| tiles.contains(c)), "{tactic} round {round}: {name} struck {} at {at:?} outside its marks {tiles:?}", s.w.units[*u].name);
                    }
                }
            }
            if s.combat_over {
                break;
            }
            s.advance_round();
        }
    }
    assert!(areas > 0, "the Sweep's areas landed in the rounds played");
}
