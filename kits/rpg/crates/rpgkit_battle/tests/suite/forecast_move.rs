//! PR7-90 FORECAST-MOVE1 (PT-85, FB-406) on a scratch catalog built here (no shipped data): a hit forecast reads where
//! its target stands in the block the hit lands, after every move booked before it (`intent::position_at` over the
//! round as it will run). The three logged cases each had a walk the old read took over the board as it stood at
//! planning: a friend's tile stopped it (the real walk passes through, ESCAPE-PLAN1 (2)), a foe stepping in front
//! earlier in the block did not, and a tile a friend leaves earlier stayed blocked. Each test plays the round and
//! checks the forecast tile against the tile the round leaves the unit on, and the forecast hit against the real one.

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::intent;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const MATE: UnitId = 1;
const FOE: UnitId = 2;
const RUNNER: UnitId = 3;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_chop", "name": "Chop", "kind": "ATTACK", "level": 1, "blocks": 2, "round_intent": "assault", "power_mult": 1.0, "reach_min": 1, "reach_max": 1}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": speed, "attack_range": 1, "skill_cap": 5});
    let units = [unit("f_hero", "party", 10), unit("f_mate", "party", 12), unit("f_foe", "enemy", 8), unit("f_runner", "enemy", 20)];
    Catalog::from_value(&json!({"skills": skills, "units": units}))
}

fn session(cells: [V2; 4]) -> Session {
    let row = json!({"id": "f_fight", "name": "Fight", "party": ["f_hero", "f_mate"], "enemies": ["f_foe", "f_runner"], "total_blocks": BLOCKS, "unit_ap": 12, "ap_regen": 0, "max_rounds": 8, "rng_seed": 7});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT]}).as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in cells.into_iter().enumerate() {
        s.w.grid.positions[c] = Some(cell);
    }
    s.resolver.scheduled.clear();
    s.plan = Plan::new();
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn add(plan: &mut Plan, w: &World, c: UnitId, step: rpgkit_battle::plan::Step) {
    assert_eq!(plan.add_step(w, c, step, false), "", "step for {}", w.units[c].name);
}

/// The enemies' round, booked before the party plans: each `(unit, steps)` in order.
fn enemy_round(s: &mut Session, steps: &[(UnitId, Vec<rpgkit_battle::plan::Step>)]) {
    let mut plan = Plan::new();
    for (c, list) in steps {
        for st in list {
            add(&mut plan, &s.w, *c, st.clone());
        }
    }
    plan.commit(&s.w, &mut s.resolver);
}

/// Plays the round until `block` starts; the tile each unit stands on then.
fn tiles_at(s: &mut Session, block: i32) -> Vec<V2> {
    assert!(s.confirm_plan(), "the round confirms");
    for _ in 0..500 {
        if s.phase == Phase::Replay || s.combat_over || (s.phase == Phase::Resolving && s.current_block >= block) {
            break;
        }
        s.step_next();
    }
    (0..s.w.units.len()).map(|u| s.w.pos(u)).collect()
}

/// The foe's Chop on the hero (B2-B3, lands B3), and the forecast of it over the round as it will run.
fn chop_forecast(s: &Session) -> (V2, Option<intent::HitOn>) {
    let round = intent::planned_schedule(&s.w, &s.resolver.scheduled, &s.plan);
    let a = round.iter().find(|a| a.unit == FOE && a.command.skill == "t_chop").expect("the foe's chop");
    (intent::position_at(&s.w, &round, HERO, a.impact), intent::hit_on(&s.w, &round, a, HERO))
}

/// The foe chops at the hero from where it stands (B2-B3).
fn foe_chops(s: &mut Session) -> rpgkit_battle::plan::Step {
    round_intent::set_intent(&mut s.w, FOE, RoundIntent::Assault).expect("intent");
    round_intent::set_target(&mut s.w, FOE, RoundTarget::Unit(HERO)).expect("target");
    PlanStep::engage(sid(&s.w, "t_chop"), HERO)
}

/// Case "a hero walks off, the forecast keeps him in reach" (try 2 R2): the hero's route passes the mate's tile
/// (ESCAPE-PLAN1 (2)); the old read stopped him before it, next to the foe. He ends out of the chop's reach, and the
/// forecast says so.
#[test]
fn a_walk_through_a_friend_is_forecast_to_its_end() {
    let mut s = session([V2::new(5, 4), V2::new(4, 4), V2::new(6, 4), V2::new(11, 8)]);
    let rest = sid(&s.w, "rest");
    let chop = foe_chops(&mut s);
    enemy_round(&mut s, &[(FOE, vec![PlanStep::self_skill(rest), chop])]);
    let mv = sid(&s.w, "move");
    let mut party = Plan::new();
    // the hero steps west through the mate's tile to (2, 4) in B1 (3 tiles: through is shorter than around)
    add(&mut party, &s.w, HERO, PlanStep::go_to(mv, V2::new(2, 4)));
    s.plan = party;
    let (at, hit) = chop_forecast(&s);
    let real = tiles_at(&mut s, 3);
    assert_eq!(at, real[HERO], "the forecast tile is the tile the round leaves the hero on");
    assert_eq!(at, V2::new(2, 4));
    assert!(hit.as_ref().is_none_or(|h| h.damage == 0), "out of the chop's reach: no damage forecast ({hit:?})");
}

/// Case "a hero cut short by a faster foe" (try 2 R4): a faster foe steps into the hero's route earlier in the block; the
/// hero stops before it, in the chop's reach, and the forecast says so (the old read let him walk on).
#[test]
fn a_foe_stepping_in_front_is_forecast_to_stop_the_walk() {
    let mut s = session([V2::new(2, 4), V2::new(0, 0), V2::new(9, 8), V2::new(4, 2)]);
    let mv = sid(&s.w, "move");
    // the runner (speed 20) steps to (4, 4) in B1, ahead of the hero (speed 10)
    enemy_round(&mut s, &[(RUNNER, vec![PlanStep::go_to(mv, V2::new(4, 4))])]);
    let mut party = Plan::new();
    add(&mut party, &s.w, HERO, PlanStep::go_to(mv, V2::new(6, 4)));
    s.plan = party;
    let round = intent::planned_schedule(&s.w, &s.resolver.scheduled, &s.plan);
    let at = intent::position_at(&s.w, &round, HERO, 3);
    let real = tiles_at(&mut s, 3);
    assert_eq!(real[RUNNER], V2::new(4, 4), "the fixture: the runner got there first");
    assert_eq!(at, real[HERO], "the forecast tile is the tile the round leaves the hero on");
    assert_ne!(at, V2::new(6, 4), "the runner stops the walk");
}

/// Case "a hero forecast hit, took nothing" (try 3 R1): a friend leaves a tile earlier in the block; the hero walks on
/// through where it stood, away from the foe, and the forecast follows (the old read stopped him at the friend's old
/// tile, in reach).
#[test]
fn a_tile_a_friend_leaves_earlier_is_forecast_free() {
    let mut s = session([V2::new(5, 4), V2::new(4, 4), V2::new(6, 4), V2::new(11, 8)]);
    let mv = sid(&s.w, "move");
    let rest = sid(&s.w, "rest");
    let chop = foe_chops(&mut s);
    enemy_round(&mut s, &[(FOE, vec![PlanStep::self_skill(rest), chop])]);
    let mut party = Plan::new();
    // the mate (faster) walks north out of the corridor first; the hero then walks west through its old tile
    add(&mut party, &s.w, MATE, PlanStep::go_to(mv, V2::new(4, 2)));
    add(&mut party, &s.w, HERO, PlanStep::go_to(mv, V2::new(3, 4)));
    s.plan = party;
    let (at, hit) = chop_forecast(&s);
    let real = tiles_at(&mut s, 3);
    assert_eq!(at, real[HERO], "the forecast tile is the tile the round leaves the hero on");
    assert!(hit.as_ref().is_none_or(|h| h.damage == 0 || h.fails.is_some()), "out of reach: {hit:?}");
}
