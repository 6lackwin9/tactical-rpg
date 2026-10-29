//! Enemy intent tells (emberwake_battle::intent, INTENT1 section 4) on the shipped data: the cases the recorded Godot
//! traces don't reach (no foresight holder, no enrage at planning, no arriving unit in them).

use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::NO_CELL;
use emberwake_battle::intent::{self, ARRIVING, IDLE};
use emberwake_battle::session::{EncounterDef, Session};
use emberwake_world::battle::WorldRoster;
use emberwake_world::Data;
use std::path::Path;

fn session(d: &Data, world: &str, encounter: &str) -> Session {
    let def = EncounterDef::from_obj(d.resolve(world, "encounters", encounter));
    Session::start(d.cat.clone(), def, &WorldRoster { d, world: world.into() })
}

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// Some world encounter of the shipped data (the first one whose enemies all plan something).
fn planned(d: &Data) -> Session {
    for e in d.own_entries("ashlands", "encounters") {
        let s = session(d, "ashlands", &aegis_core::json::s(e, "id"));
        if !s.enemy_intents().is_empty() {
            return s;
        }
    }
    panic!("an ashlands encounter with enemy tells");
}

#[test]
fn each_tell_is_the_enemys_first_scheduled_action_and_only_while_planning() {
    let d = data();
    let mut s = planned(&d);
    for r in s.enemy_intents() {
        let actions = intent::actions_of(r.unit, &s.resolver.scheduled);
        match actions.first() {
            Some(a) => assert_eq!((r.line.kind, r.line.blocks, r.line.skill.clone()), (intent::kind_of(&s.w, &a.command), (a.start, a.end), a.command.name.clone())),
            None => assert!(r.line.kind == IDLE || r.line.kind == ARRIVING),
        }
        assert!(!r.revealed && r.plan.is_empty(), "nobody reads the plan without foresight or enrage");
    }
    let before: Vec<_> = s.enemy_intents();
    s.plan_apply_preset_all("balanced");
    assert_eq!(s.enemy_intents(), before, "the party's plan does not change the tells");
    assert!(s.confirm_plan());
    assert!(s.enemy_intents().is_empty(), "no tells once the round is confirmed");
}

#[test]
fn a_foresight_holder_targeted_by_an_enemy_reads_its_whole_plan() {
    let d = data();
    let mut s = planned(&d);
    let target = s.enemy_intents().iter().find_map(|r| r.line.target.filter(|t| s.w.allies.contains(t)));
    let Some(holder) = target else { panic!("an enemy aiming at a hero") };
    s.w.units[holder].foresight_source = "wardens_eye".into();
    let tells = s.enemy_intents();
    for r in &tells {
        let aims = intent::actions_of(r.unit, &s.resolver.scheduled).iter().any(|a| intent::targets_unit(a, holder, s.w.pos(holder)));
        assert_eq!(r.revealed, aims, "{} reveals only when it aims at the holder", s.w.units[r.unit].name);
        if aims {
            assert_eq!(r.readers, vec![holder]);
            assert_eq!(r.plan.len(), intent::actions_of(r.unit, &s.resolver.scheduled).len(), "the whole plan");
        }
    }
    assert!(tells.iter().any(|r| r.revealed));
}

#[test]
fn enraged_enemies_show_their_plan_and_empty_ones_hold_or_arrive() {
    let d = data();
    let s = planned(&d);
    let e = s.enemy_intents()[0].unit;
    let raged = intent::intent_of(&s.w, e, &s.resolver.scheduled, &s.on_grid_at_schedule, &s.range_reveals, true);
    assert!(raged.revealed && raged.enraged && raged.readers.is_empty() && !raged.plan.is_empty());
    let idle = intent::intent_of(&s.w, e, &[], &[e], &[], false);
    assert_eq!((idle.line.kind, idle.line.blocks, idle.line.skill.as_str()), (IDLE, (0, 0), ""));
    let arriving = intent::intent_of(&s.w, e, &[], &[], &[], false);
    assert_eq!(arriving.line.kind, ARRIVING, "not on the grid when the round was scheduled");
}

#[test]
fn an_engage_approach_names_its_target_and_a_plain_move_its_goal() {
    let d = data();
    let mut found = (false, false);
    for e in d.own_entries("ashlands", "encounters") {
        let s = session(&d, "ashlands", &aegis_core::json::s(e, "id"));
        for a in &s.resolver.scheduled {
            if let CmdKind::Walk { approach, goal, .. } = &a.command.kind {
                let (target, cell) = (intent::target_of(&a.command), intent::cell_of(&a.command));
                match approach {
                    Some(t) => {
                        assert_eq!((target, cell), (Some(*t), NO_CELL));
                        found.0 = true;
                    }
                    None => {
                        assert_eq!((target, cell), (None, *goal));
                        found.1 = true;
                    }
                }
                assert!(matches!(intent::kind_of(&s.w, &a.command), "move" | "flee"));
            }
        }
    }
    assert!(found.0, "some enemy engages (a move toward its attack target)");
}
