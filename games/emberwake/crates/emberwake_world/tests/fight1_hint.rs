//! PR7-41 (PT-21) on the shipped data (read only): the first story fight of a new game teaches "Strike while they wind
//! up: a quick <skill> breaks a slow blow". Its first round, drafted the way the battle screen drafts it (each member's
//! tactic ghost), makes that true: the hint's quick skill breaks a foe whose slow blow is winding up before that blow
//! lands (it never hits). The skill is the party skill the hint names, the slow blow the foe's longest wind-up; nothing
//! about the units, skills or numbers is hard-coded.

use aegis_core::json;
use emberwake_battle::command::CmdKind;
use emberwake_battle::session::{Phase, Session};
use emberwake_data::Step;
use emberwake_world::{battle, new_game, story, Data};
use std::path::Path;

const WORLD: &str = "sunk_ward";

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

/// The first story fight of a new game: (its hint, the battle at its start).
fn fight1(d: &Data) -> (String, Session) {
    let mut gs = new_game::create_for_world(d, WORLD, 1).expect("a new game");
    for _ in 0..50 {
        match story::current(d, &gs) {
            Some(Step::Vignette { vignette }) => assert!(json::b(&story::step_done(d, &mut gs), "ok", false), "{vignette}"),
            Some(Step::Fight { encounter, .. }) => {
                let hint = json::s(d.resolve(WORLD, "encounters", &encounter), "objective_text");
                let fight = story::fight(d, &gs).expect("a story fight at a fight step");
                return (hint, battle::begin_world(d, &mut gs, &fight).expect("the first fight starts").session);
            }
            _ => break,
        }
    }
    panic!("a new game reaches a fight");
}

#[test]
fn the_first_fights_quick_strike_breaks_a_slow_blow_before_it_lands() {
    let d = data();
    for tactic in ["balanced", "focus_weakest"] {
        let (hint, mut s) = fight1(&d);
        assert!(hint.contains("breaks a slow blow"), "the first fight's hint: {hint}");
        let quick: Vec<String> = s.w.allies.iter().flat_map(|&a| s.w.units[a].known_skills.clone()).filter(|id| s.w.cat.skill(id).is_some_and(|k| hint.contains(&k.name))).collect();
        assert!(!quick.is_empty(), "{tactic}: a party skill the hint names: {hint}");
        let quick_name = s.w.cat.skill(&quick[0]).unwrap().name.clone();
        for m in s.w.allies.clone() {
            s.plan_apply_preset(m, tactic);
        }
        assert!(s.confirm_plan(), "{tactic}: the drafted round confirms");
        // the foes' slow blows: their attacks with the longest wind-up this round
        let longest = s.resolver.scheduled.iter().filter(|a| s.w.enemies.contains(&a.unit) && matches!(a.command.kind, CmdKind::Attack { .. })).map(|a| a.impact - a.start).max().expect("the foes attack");
        let slow: Vec<usize> = (0..s.resolver.scheduled.len()).filter(|&i| {
            let a = &s.resolver.scheduled[i];
            s.w.enemies.contains(&a.unit) && matches!(a.command.kind, CmdKind::Attack { .. }) && a.impact - a.start == longest
        }).collect();
        assert!(longest > 0, "{tactic}: a blow that winds up");
        let mut broke = Vec::new();
        while s.phase != Phase::Replay && !s.combat_over {
            let landed: Vec<bool> = s.resolver.scheduled.iter().map(|a| a.landed).collect();
            s.step_next();
            for (i, a) in s.resolver.scheduled.iter().enumerate() {
                let newly = a.landed && !landed.get(i).copied().unwrap_or(true);
                if newly && a.command.name == quick_name && s.w.allies.contains(&a.unit) {
                    for st in a.command.staggers.iter().filter(|st| st.windup) {
                        // the slow blow of the struck foe has not landed and is cancelled by the break now
                        if let Some(&j) = slow.iter().find(|&&j| s.resolver.scheduled[j].unit == st.target && !s.resolver.scheduled[j].landed && s.resolver.scheduled[j].broken) {
                            broke.push(j);
                        }
                    }
                }
            }
        }
        broke.dedup();
        assert!(!broke.is_empty(), "{tactic}: a {quick_name} breaks a slow blow before it lands; log: {:#?}", s.resolver.log);
        for j in broke {
            assert!(!s.resolver.scheduled[j].landed, "{tactic}: the broken blow never lands");
        }
    }
}
