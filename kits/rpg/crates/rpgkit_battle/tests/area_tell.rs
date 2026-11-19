//! PR7-41 (PT-17) on a scratch catalog built here (no shipped data): an area tell marks the tiles the area strikes
//! when it lands (`intent::area_cells`). A shaped Assault row (INTENT2 IR5, IR7, IR17: locked on its target, its shape
//! centred where the target stands at the impact) follows its target's planned move, so the marks read over the round
//! as it will run (`intent::planned_schedule`) are the tiles it strikes: the round hits every unit in them and none
//! outside. A tile area (no target) keeps the tiles fixed when it was planned.

use indexmap::IndexMap;
use rpgkit_battle::command::CmdKind;
use rpgkit_battle::grid::V2;
use rpgkit_battle::intent;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, FIELD_INTENT, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_sweep", "name": "Sweep", "kind": "ATTACK", "level": 1, "blocks": 3, "round_intent": "assault", "area": "ARC", "power_mult": 1.0}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("a_hero", "party", 14), unit("a_ally", "party", 12), unit("a_foe", "enemy", 10)]}))
}

/// The foe at (6, 4), the hero north of it at (6, 3), the ally at (5, 5) (off the arc aimed north).
fn session() -> Session {
    let row = json!({"id": "a_fight", "name": "Fight", "party": ["a_hero", "a_ally"], "enemies": ["a_foe"], "total_blocks": BLOCKS, "unit_ap": 12, "ap_regen": 0, "max_rounds": 8, "rng_seed": 7});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, FIELD_INTENT]}).as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(6, 3)), (ALLY, V2::new(5, 5)), (FOE, V2::new(6, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn add(plan: &mut Plan, w: &World, c: UnitId, step: rpgkit_battle::plan::Step) {
    assert_eq!(plan.add_step(w, c, step, false), "", "step for {}", w.units[c].name);
}

/// The foe's round: the shaped Assault Sweep locked on the hero (aimed at its tile now), scheduled before the party
/// plans.
fn schedule_sweep(s: &mut Session) {
    round_intent::set_intent(&mut s.w, FOE, RoundIntent::Assault).expect("intent");
    round_intent::set_target(&mut s.w, FOE, RoundTarget::Unit(HERO)).expect("target");
    let mut plan = Plan::new();
    let at = s.w.pos(HERO);
    add(&mut plan, &s.w, FOE, PlanStep::area_on(sid(&s.w, "t_sweep"), HERO, at));
    s.resolver.scheduled.clear();
    plan.commit(&s.w, &mut s.resolver);
}

/// The scheduled sweep (its index and command's planned tiles).
fn sweep(s: &Session) -> (usize, Vec<V2>) {
    s.resolver.scheduled.iter().enumerate().find_map(|(i, a)| match &a.command.kind {
        CmdKind::Area { cells, target: Some(_), .. } if a.unit == FOE => Some((i, cells.clone())),
        _ => None,
    }).expect("the foe's shaped sweep")
}

/// Plays the round out; the units the sweep struck.
fn play(s: &mut Session, i: usize) -> Vec<UnitId> {
    assert!(s.confirm_plan(), "the round confirms");
    while s.phase != Phase::Replay && !s.combat_over {
        s.step_next();
    }
    let mut struck: Vec<UnitId> = s.resolver.scheduled[i].command.hits.iter().map(|h| h.0).collect();
    struck.sort_unstable();
    struck
}

/// The units standing in `tiles` at block `block` of `round`.
fn inside(w: &World, round: &[rpgkit_battle::resolver::ScheduledAction], tiles: &[V2], block: i32) -> Vec<UnitId> {
    (0..w.units.len()).filter(|&u| u != FOE && w.has(u) && tiles.contains(&intent::position_at(w, round, u, block))).collect()
}

/// PT-17: the hero walks to the foe's west side before the sweep lands; the marks follow it (the arc turns to face west) and
/// the round strikes exactly the units in them: the ally, outside the arc first aimed north, is struck.
#[test]
fn a_shaped_area_marks_follow_its_targets_planned_move() {
    let mut s = session();
    schedule_sweep(&mut s);
    let (i, planned) = sweep(&s);
    let mv = sid(&s.w, "move");
    round_intent::set_intent(&mut s.w, HERO, RoundIntent::Maneuver).expect("hero intent");
    let mut party = Plan::new();
    add(&mut party, &s.w, HERO, PlanStep::go_to(mv, V2::new(5, 4)));
    s.plan = party;
    let round = intent::planned_schedule(&s.w, &s.resolver.scheduled, &s.plan);
    let impact = round[i].impact;
    assert_eq!(intent::position_at(&s.w, &round, HERO, impact), V2::new(5, 4), "the hero's planned tile at the impact");
    let marked = intent::area_cells(&s.w, &round, &round[i]);
    assert_ne!(marked, planned, "the marks follow the hero off the tiles first aimed at");
    let expected = inside(&s.w, &round, &marked, impact);
    assert!(expected.contains(&HERO) && expected.contains(&ALLY), "{marked:?} {expected:?}");
    assert!(!planned.contains(&s.w.pos(ALLY)), "the ally stands off the arc first aimed: {planned:?}");
    assert_eq!(play(&mut s, i), expected, "the round strikes the units in the marks, none outside");
}

/// PT-17: with no move the marks are the planned tiles and the round strikes the hero alone.
#[test]
fn a_shaped_area_on_a_still_target_marks_its_planned_tiles() {
    let mut s = session();
    schedule_sweep(&mut s);
    let (i, planned) = sweep(&s);
    s.plan = Plan::new();
    let round = intent::planned_schedule(&s.w, &s.resolver.scheduled, &s.plan);
    let marked = intent::area_cells(&s.w, &round, &round[i]);
    assert_eq!(marked, planned);
    assert_eq!(play(&mut s, i), inside(&s.w, &round, &marked, round[i].impact));
}
