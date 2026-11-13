//! ACT1-HOOKS2 (FB-248 (b); `CAST1.md` T12, T13, `CAST1_ENCOUNTERS.md` CAST1-B): the Act 1 hook openers as
//! deterministic scripted probes on the shipped `sunk_ward` fights, built as the world builds them
//! (`battle::authored_fight`, stage 0, the encounter's own seed) and driven as the autoplay drives them (`auto_draft`),
//! with one founder's round-1 plan scripted. T12: in the rooftop prologue (`enc_pro_0b`) the founder who knows
//! `ash_cloud_fade` casts it in round 1 (in its intent) and its hazard lies on the tiles around her; T13: on the bridge
//! (`enc_pro_0c`) the founder who knows `tripwire_snare` wires the first tile of a foe's round-1 walk the skill can
//! reach, and that foe springs the trap in round 1. The skills, hazards, casters and tiles are read from the data and
//! the fight; the shipped data is only read.

use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::V2;
use emberwake_battle::plan::{PlanStep, Step};
use emberwake_battle::round_intent::{self, RoundIntent};
use emberwake_battle::session::{Phase, Session};
use emberwake_battle::world::UnitId;
use emberwake_world::{battle, Data};
use std::path::Path;

const WORLD: &str = "sunk_ward";
/// T12: the rooftop prologue and its smoke opener.
const ROOFTOP: &str = "enc_pro_0b";
const SMOKE_SKILL: &str = "ash_cloud_fade";
/// T13: the bridge prologue and its trap opener.
const BRIDGE: &str = "enc_pro_0c";
const TRAP_SKILL: &str = "tripwire_snare";
/// The autoplay's tactic (the battle screen's and the sims' default).
const PRESET: &str = "balanced";

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

fn session(d: &Data, id: &str) -> Session {
    let (def, roster) = battle::authored_fight(d, WORLD, id, 0).unwrap_or_else(|| panic!("{WORLD}/{id}"));
    Session::start(d.cat.clone(), def, &roster)
}

fn sid(s: &Session, skill: &str) -> usize {
    s.w.book().find(skill).unwrap_or_else(|| panic!("skill {skill}"))
}

/// The skill's own intent (its first row): the round intent the caster takes to use it.
fn intent_for(s: &Session, skill: &str) -> RoundIntent {
    let row = round_intent::skill_primary(&s.w, &s.w.cat.skills[sid(s, skill)]);
    RoundIntent::parse(row).unwrap_or_else(|| panic!("{skill}: intent row '{row}'"))
}

/// After the auto draft, `c`'s round-1 plan is only `step` (in the skill's intent); the refusal ("" = planned).
fn script(s: &mut Session, c: UnitId, skill: &str, step: Step) -> String {
    if round_intent::on(&s.w) {
        let intent = intent_for(s, skill);
        if let Err(why) = round_intent::set_intent(&mut s.w, c, intent) {
            return why;
        }
    }
    s.plan.clear(c);
    s.plan.add_step(&s.w, c, step, true)
}

/// The founder who can use `skill` with `step` in round 1 of fight `id` (tried on a fresh session each).
fn caster(d: &Data, id: &str, skill: &str, step: impl Fn(&Session) -> Step) -> Option<UnitId> {
    let allies = session(d, id).w.allies.clone();
    allies.into_iter().find(|&c| {
        let mut s = session(d, id);
        s.auto_draft(PRESET);
        let st = step(&s);
        script(&mut s, c, skill, st).is_empty()
    })
}

/// Round 1 resolved to its end.
fn play_round(s: &mut Session) {
    assert!(s.confirm_plan(), "the scripted plan confirms");
    s.resolve_round();
    assert_eq!(s.phase, Phase::Replay, "round 1 resolved");
}

#[test]
fn t12_the_rooftop_smoke_opener_covers_the_tiles_around_its_caster() {
    let d = data();
    let c = caster(&d, ROOFTOP, SMOKE_SKILL, |s| PlanStep::self_skill(sid(s, SMOKE_SKILL))).expect("a founder knows the smoke opener");
    let mut s = session(&d, ROOFTOP);
    s.auto_draft(PRESET);
    let step = PlanStep::self_skill(sid(&s, SMOKE_SKILL));
    assert_eq!(script(&mut s, c, SMOKE_SKILL, step), "");
    let at = s.w.pos(c);
    let hazard = s.w.cat.skills[sid(&s, SMOKE_SKILL)].hazard.clone().expect("the skill places a hazard");
    play_round(&mut s);
    let smoke: Vec<V2> = s.w.grid.ground.iter().filter(|(_, p)| p.def.id == hazard.id && p.by == c).map(|(cell, _)| *cell).collect();
    assert!(!smoke.is_empty(), "T12: the smoke is placed ({} at {at:?})", s.w.units[c].name);
    assert!(smoke.contains(&at), "T12: on the caster's tile");
    assert!(smoke.iter().all(|cell| (cell.x - at.x).abs() <= 1 && (cell.y - at.y).abs() <= 1), "T12: the 3x3 around the caster: {smoke:?}");
    let placed = s.log.iter().filter(|e| e.outcome == "PLACED" && e.category == "EVENT" && e.actor_is_ally).count();
    assert!(placed >= 1, "T12: the placement is logged");
}

/// The cells the foes' round-1 walks cross, in schedule and path order: (foe, cell).
fn foe_walk_cells(s: &Session) -> Vec<(UnitId, V2)> {
    let mut out = Vec::new();
    for a in &s.resolver.scheduled {
        if !s.w.enemies.contains(&a.unit) {
            continue;
        }
        if let CmdKind::Walk { path, expected_start, .. } = &a.command.kind {
            let mut at = *expected_start;
            for dir in path {
                at = V2::new(at.x + dir.x, at.y + dir.y);
                out.push((a.unit, at));
            }
        }
    }
    out
}

#[test]
fn t13_the_bridge_trap_opener_springs_on_a_foe_crossing_it() {
    let d = data();
    let crossings = foe_walk_cells(&session(&d, BRIDGE));
    assert!(!crossings.is_empty(), "T13: a foe walks in round 1");
    // the first crossed tile some founder can wire
    let (foe, cell, c) = crossings
        .iter()
        .find_map(|&(foe, cell)| caster(&d, BRIDGE, TRAP_SKILL, |s| PlanStep::area_at(sid(s, TRAP_SKILL), cell)).map(|c| (foe, cell, c)))
        .expect("a founder can wire a tile a foe crosses");
    let mut s = session(&d, BRIDGE);
    s.auto_draft(PRESET);
    let step = PlanStep::area_at(sid(&s, TRAP_SKILL), cell);
    assert_eq!(script(&mut s, c, TRAP_SKILL, step), "");
    let hp = s.w.units[foe].hp;
    play_round(&mut s);
    let foe_name = s.w.units[foe].name.clone();
    let trapped: Vec<_> = s.log.iter().filter(|e| e.subtype == "Hazard" && e.outcome == "TRAPPED").collect();
    assert_eq!(trapped.len(), 1, "T13: one trap sprung ({} wires {cell:?})", s.w.units[c].name);
    assert!(trapped[0].actor_is_ally && !trapped[0].target_is_ally, "T13: a foe springs the founder's trap");
    assert_eq!(s.w.pos(foe), cell, "T13: {foe_name} stops on the wired tile");
    assert!(s.w.units[foe].hp < hp, "T13: the trap hurts");
    assert!(!s.w.grid.ground.get(&cell).is_some_and(|p| p.def.trap), "T13: a sprung trap is gone");
}
