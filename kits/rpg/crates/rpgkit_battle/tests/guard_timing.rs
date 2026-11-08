//! S1.2-S4 (docs/sessions/S1_STATUS.md section 1 "Guard timing (Option 1C)", section 5 test 2; FB-48): a Guard skill's
//! guard is held (`HeldGuard`, `Unit.guard` the accumulator) from the block its wind-up starts through the round's last
//! block in every ruleset; a multi-round Guard skill queues the full-round `guard` status for its later rounds at its
//! fraction (the ruleset's `guard_multi_round_mitigation` when it authors none); the sources do not stack (the larger
//! counts); a Poise break and a stun end the guard and its status.

use indexmap::IndexMap;
use rpgkit_battle::command::Command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Numbers, Ruleset, POISE, TIMELINE};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::status;
use rpgkit_rules::unit::HeldGuard;
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const FOE: UnitId = 1;

/// A 2-block Guard (0.3), a 2-block parry (0.15), a counter stance with no guard (0, as riposte's third rank), a Guard
/// and a 3-round brace without a `mitigation` key, and a crushing jab (stagger far above any bar).
fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_parry", "name": "Parry", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.15}),
        json!({"id": "t_counter", "name": "Counter", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.0}),
        json!({"id": "t_stand", "name": "Stand", "kind": "DEFEND", "level": 1, "blocks": 1}),
        json!({"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 2, "guard_rounds": 3}),
        json!({"id": "t_thin", "name": "Thin Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.05, "guard_rounds": 2}),
        json!({"id": "t_wall", "name": "Wall", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.95, "guard_rounds": 2}),
        json!({"id": "t_tower", "name": "Tower", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.4, "guard_rounds": 2}),
        json!({"id": "t_crush", "name": "Crush", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.1, "stagger": 400, "target_mode": "ENEMY"}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5, "poise": 40});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("g_hero", "party", 14), unit("g_foe", "enemy", 10)]}))
}

/// The timeline without the rule `triggers` (1C holds in every ruleset), with poise for the break.
fn session() -> Session {
    let row = json!({"id": "g_fight", "name": "Fight", "party": ["g_hero"], "enemies": ["g_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [POISE]}).as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn resolver() -> Resolver {
    let mut res = Resolver::new(BLOCKS, 1);
    res.register(HERO);
    res.register(FOE);
    res
}

fn command(w: &World, c: UnitId, step: Step, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "{skill}");
    let id = w.book().find(skill).unwrap_or_else(|| panic!("skill {skill}"));
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

fn own(w: &World, c: UnitId, skill: &str) -> Command {
    command(w, c, PlanStep::self_skill(w.book().find(skill).unwrap()), skill)
}

fn book(res: &mut Resolver, c: UnitId, cmd: Command, start: i32) {
    assert!(res.schedule(c, cmd, start), "booked at {start}");
}

/// Runs blocks `from..=to`, returning the hero's guard as each block is prepared.
fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) -> Vec<f64> {
    let mut guards = Vec::new();
    for b in from..=to {
        let resolving = res.prepare_block(w, b);
        guards.push(w.units[HERO].guard);
        for i in resolving {
            res.execute_single(w, i);
        }
    }
    guards
}

/// The round's end and the next round's start (statuses roll over).
fn next_round(w: &mut World, res: &mut Resolver) {
    res.apply_round_end_effects(w);
    res.finish_round(w);
    let n = w.ruleset.numbers.clone();
    for c in [HERO, FOE] {
        status::roll_over(&mut w.units[c], 0, &n);
    }
}

/// Section 5 test 2, first half: a 2-block Guard started at block 2 raises `Unit.guard` via `HeldGuard` from block 2
/// through block 6 (not only over its own blocks 2-3), and the guard ends with the round.
#[test]
fn a_guard_started_at_block_2_holds_from_block_2_through_block_6() {
    let mut s = session();
    let w = &mut s.w;
    let mut res = resolver();
    book(&mut res, HERO, own(w, HERO, "guard"), 2);
    let guards = run(w, &mut res, 1, 1);
    assert_eq!(guards, [0.0], "no guard before the wind-up starts");
    assert_eq!(w.units[HERO].held_guard, None);
    let guards = run(w, &mut res, 2, BLOCKS);
    assert_eq!(guards, [0.3; 5], "blocks 2-6");
    assert_eq!(w.units[HERO].held_guard, Some(HeldGuard { mitigation: 0.3, last_block: BLOCKS }), "held from the wind-up start to the round's end");
    assert_eq!(w.units[HERO].take_damage(100, false), 70, "take_damage applies (1 - guard)");
    next_round(w, &mut res);
    assert_eq!(w.units[HERO].held_guard, None, "the round's end ends the held guard (no trigger rule needed)");
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.0; 6], "nothing next round");
}

/// Section 3: a Guard skill row without a fraction takes the ruleset's: `guard_common_mitigation` for a 1-round guard,
/// `guard_multi_round_mitigation` for a multi-round one (lower, FB-48); an authored fraction is kept, 0 meaning none.
#[test]
fn an_unauthored_fraction_takes_the_ruleset_default() {
    let d = Numbers::default();
    assert_eq!((d.guard_common_mitigation, d.guard_multi_round_mitigation), (status::GUARD_COMMON_MITIGATION, status::GUARD_MULTI_ROUND_MITIGATION));
    assert!(d.guard_multi_round_mitigation < d.guard_common_mitigation, "the multi-round guard is the lower");
    let mut s = session();
    let w = &mut s.w;
    let fraction = |w: &World, id: &str| rpgkit_battle::skill::guard_mitigation(&w.cat.skills[w.book().find(id).unwrap()], &w.ruleset.numbers);
    assert_eq!(
        [fraction(w, "guard"), fraction(w, "t_parry"), fraction(w, "t_counter"), fraction(w, "t_stand"), fraction(w, "t_brace")],
        [0.3, 0.15, 0.0, d.guard_common_mitigation, d.guard_multi_round_mitigation]
    );
    let mut res = resolver();
    book(&mut res, HERO, own(w, HERO, "t_counter"), 1);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.0; 6], "an authored 0 holds no guard");
    next_round(w, &mut res);
    book(&mut res, HERO, own(w, HERO, "t_stand"), 1);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [d.guard_common_mitigation; 6]);
    assert!(!w.units[HERO].st.has_any(status::GUARD), "a 1-round guard queues no status");
    // a ruleset row moves the default
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": [POISE], "guard_multi_round_mitigation": 0.25});
    w.ruleset = Ruleset::from_row(row.as_object_mut().unwrap());
    assert_eq!(fraction(w, "t_brace"), 0.25);
}

/// Section 5 test 2, second half: a multi-round Guard skill (a test-only `brace`, `guard_rounds` 3) holds its guard from
/// the wind-up start in round 1 and queues the full-round `guard` status for rounds 2-3 at the lower fraction (0.20).
#[test]
fn a_multi_round_guard_queues_the_full_round_guard_status() {
    let mut s = session();
    let w = &mut s.w;
    let low = w.ruleset.numbers.guard_multi_round_mitigation;
    let mut res = resolver();
    book(&mut res, HERO, own(w, HERO, "t_brace"), 1);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [low; 6], "round 1: from the wind-up start");
    let st = &w.units[HERO].st;
    assert!(st.pending.contains(status::GUARD) && !st.active.contains(status::GUARD), "queued at the impact for next round");
    assert_eq!((st.pending_rounds.get(status::GUARD).copied(), st.pending_magnitudes.get(status::GUARD).copied()), (Some(2), Some(20)), "rounds 2-3 at 20%");
    for round in 2..=3 {
        next_round(w, &mut res);
        assert!(w.units[HERO].has_status(status::GUARD), "round {round}: active");
        assert_eq!(run(w, &mut res, 1, BLOCKS), [low; 6], "round {round}: every block, no Defend");
        assert_eq!(w.units[HERO].held_guard, None, "round {round}: the status, not a held guard");
    }
    next_round(w, &mut res);
    assert!(!w.units[HERO].st.has_any(status::GUARD), "round 4: over");
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.0; 6]);
}

/// S1 section 1: `guard_mitigation = max(running_defend, guard_status)`: a Defend over an active guard status gives the
/// larger fraction, never both.
#[test]
fn guard_sources_do_not_stack() {
    let mut s = session();
    let w = &mut s.w;
    status::grant_now(&mut w.units[HERO], status::GUARD, 1, 20, "brace");
    let mut res = resolver();
    book(&mut res, HERO, own(w, HERO, "guard"), 3);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.2, 0.2, 0.3, 0.3, 0.3, 0.3], "the status, then the larger Defend; not 0.5 nor 1 - 0.8 x 0.7");
    assert_eq!(w.units[HERO].take_damage(100, false), 70);
    next_round(w, &mut res);
    status::grant_now(&mut w.units[HERO], status::GUARD, 1, 20, "brace");
    book(&mut res, HERO, own(w, HERO, "t_parry"), 3);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.2; 6], "a weaker Defend leaves the status's fraction");
}

/// FB-48: a Poise break ends the guard: the held guard goes, the `guard` status instance is removed (not only the
/// Defend marked broken), and the guard is 0 from then on.
#[test]
fn a_poise_break_ends_the_guard_and_removes_the_status() {
    let mut s = session();
    let w = &mut s.w;
    status::grant_now(&mut w.units[HERO], status::GUARD, 2, 20, "brace");
    let mut res = resolver();
    book(&mut res, HERO, own(w, HERO, "guard"), 1);
    book(&mut res, FOE, command(w, FOE, PlanStep::engage(w.book().find("t_crush").unwrap(), HERO), "t_crush"), 3);
    assert_eq!(run(w, &mut res, 1, 2), [0.3, 0.3]);
    run(w, &mut res, 3, 3);
    assert!(w.poise.get(&HERO).is_some_and(|p| p.broken), "the crush breaks the hero");
    assert_eq!(w.units[HERO].held_guard, None, "the held guard is gone");
    assert!(!w.units[HERO].st.has_any(status::GUARD), "the guard status instance is removed");
    assert_eq!(run(w, &mut res, 4, BLOCKS), [0.0; 3], "no guard for the rest of the round");
}

/// S1 section 1 ("unless Poise-broken or Stunned"): a stun ends a held `guard` status in the blocks it locks; a Guard
/// started once the stun's blocks are over holds as usual.
#[test]
fn a_stun_ends_the_guard() {
    let mut s = session();
    let w = &mut s.w;
    status::grant_now(&mut w.units[HERO], status::GUARD, 2, 20, "brace");
    status::grant_now(&mut w.units[HERO], "stun", 1, 0, "foe");
    let mut res = resolver();
    book(&mut res, HERO, own(w, HERO, "guard"), 3);
    let guards = run(w, &mut res, 1, BLOCKS);
    assert!(!w.units[HERO].st.has_any(status::GUARD), "the stun removed the guard status");
    assert_eq!(guards, [0.0, 0.0, 0.3, 0.3, 0.3, 0.3], "nothing while stunned; the later Guard from its wind-up");
}

/// Review fix: the held guard is the skill's authored fraction, unclamped (a trigger's `guard` effect keeps its 0.1-0.9
/// clamp): a 0.05 Defend (Riposte rank 2) holds exactly 0.05 through block 6, a 0.95 one 0.95; the later rounds' `guard`
/// status is capped at the row's maximum (90) and not queued below its minimum (10).
#[test]
fn the_held_guard_is_the_authored_fraction() {
    let mut s = session();
    let w = &mut s.w;
    let mut res = resolver();
    book(&mut res, HERO, own(w, HERO, "t_thin"), 1);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.05; 6], "0.05 from the wind-up start through block 6, not raised to 0.1");
    assert_eq!(w.units[HERO].held_guard, Some(HeldGuard { mitigation: 0.05, last_block: BLOCKS }));
    assert!(!w.units[HERO].st.has_any(status::GUARD), "5% is below the row's minimum: no later-round status");
    next_round(w, &mut res);
    book(&mut res, HERO, own(w, HERO, "t_wall"), 1);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.95; 6], "0.95 after its own blocks too, as the running Defend gives");
    assert_eq!(w.units[HERO].st.pending_magnitudes.get(status::GUARD).copied(), Some(90), "capped at the row's maximum");
}

/// AUDIT-11 (FB-85): the queued Round 2+ `guard` status takes the ruleset's `guard_multi_round_mitigation` unless the
/// skill authors its own `mitigation`: a ruleset row that moves the multi-round fraction moves an unauthored brace's
/// later rounds and leaves an authored multi-round guard (0.4) at its own fraction.
#[test]
fn later_rounds_take_the_multi_round_fraction_unless_authored() {
    let mut s = session();
    let w = &mut s.w;
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": [POISE], "guard_multi_round_mitigation": 0.25});
    w.ruleset = Ruleset::from_row(row.as_object_mut().unwrap());
    let skill = |w: &World, id: &str| w.cat.skills[w.book().find(id).unwrap()].clone();
    let n = w.ruleset.numbers.clone();
    let (brace, tower) = (skill(w, "t_brace"), skill(w, "t_tower"));
    assert!(!rpgkit_battle::skill::mitigation_authored(&brace) && rpgkit_battle::skill::mitigation_authored(&tower));
    assert_eq!([rpgkit_battle::skill::guard_later_mitigation(&brace, &n), rpgkit_battle::skill::guard_later_mitigation(&tower, &n)], [0.25, 0.4]);
    let mut res = resolver();
    book(&mut res, HERO, own(w, HERO, "t_brace"), 1);
    run(w, &mut res, 1, BLOCKS);
    let queued = |w: &World| (w.units[HERO].st.pending_rounds.get(status::GUARD).copied(), w.units[HERO].st.pending_magnitudes.get(status::GUARD).copied());
    assert_eq!(queued(w), (Some(2), Some(roundi(n.guard_multi_round_mitigation * 100.0))), "unauthored: the ruleset's multi-round fraction");
    for _ in 0..3 {
        next_round(w, &mut res);
    }
    assert!(!w.units[HERO].st.has_any(status::GUARD));
    book(&mut res, HERO, own(w, HERO, "t_tower"), 1);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.4; 6], "round 1: the authored fraction");
    assert_eq!(queued(w), (Some(1), Some(40)), "authored: its own fraction for round 2, not the ruleset's");
    next_round(w, &mut res);
    assert_eq!(run(w, &mut res, 1, BLOCKS), [0.4; 6], "round 2: the status at the authored fraction");
}
