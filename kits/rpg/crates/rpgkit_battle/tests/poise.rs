//! RULESET1-P on a scratch catalog built here (no shipped data): poise, stagger and breaks (RULESET1 5), Active Guard's
//! halving (6, G2) and armour pierce (7, A2) under the rules `poise` and `pierce`. Expectations come from the rules: Max
//! Poise is the unit row's fixed `poise` or the ruleset's derived row and refills each round (P1-P3); stagger taken is
//! `roundi(stagger x windup x guard x repeat x flank x passive x hook)` (P6); a break cancels every action that has not
//! landed and drops the guard (P7), once a round (P8), and costs double stagger the next round (P10); windup poise rises
//! and falls back, unbreakable and `steadfast` hold at 1 (P11, P12); the chance roll draws once per qualifying hit (P13);
//! `sure_break` breaks a windup (P14); Rest restores poise (RS3); a piercing hit skips armour, guard and cover still cut
//! (A2). Classic draws, logs and records nothing new.

use indexmap::IndexMap;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::grid::V2;
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::poise::{self, BreakCause, PoiseState};
use rpgkit_battle::resolver::{Resolver, ScheduledAction};
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::derived;
use rpgkit_rules::ruleset::{Ruleset, ENEMY_MOVES, FLANK, PIERCE, POISE, ROUND_INTENT, TIMELINE};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::status;
use rpgkit_rules::unit::StaggerTaken;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;
/// The fixed Max Poise of the test units that set one.
const BAR: i32 = 40;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;
const DERIVED: UnitId = 4;
const NO_BAR: UnitId = 5;

/// Engine skills and the test skills: a Snap jab (stagger 12), a 2-block thrust (8), a 3-block heavy (no stagger), a
/// heavy with windup poise 20, an unbreakable heavy, a sure-break snap (4), an armour-piercing spike, an area burst (10),
/// a guard (mitigation 0.5), a 3-block focus, an enemy jab with 2 recovery blocks.
fn catalog() -> Catalog {
    let attack = |id: &str, blocks: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": blocks, "power_mult": 0.2, "target_mode": "ENEMY"});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        attack("p_jab", 1, json!({"stagger": 12})),
        attack("p_thrust", 2, json!({"stagger": 8})),
        attack("p_heavy", 3, json!({})),
        attack("p_smash", 3, json!({"windup_poise": 20})),
        attack("p_judgment", 3, json!({"unbreakable": true})),
        attack("p_sure", 1, json!({"stagger": 4, "sure_break": true})),
        attack("p_spike", 1, json!({"armor_piercing": true, "power_mult": 1.0})),
        attack("p_burst", 1, json!({"stagger": 10, "area": "PLUS"})),
        attack("p_jab_rec", 1, json!({"stagger": 12, "recovery_blocks": 2})),
        json!({"id": "p_guard", "name": "p_guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.5}),
        json!({"id": "p_focus", "name": "p_focus", "kind": "FOCUS", "level": 1, "blocks": 3, "boost": 0.1, "boost_rounds": 1}),
    ];
    let unit = |id: &str, side: &str, speed: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": speed, "attack_range": 1, "skill_cap": 5, "poise": BAR});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [
            unit("p_hero", "party", 14, json!({})), unit("p_ally", "party", 12, json!({})),
            unit("p_foe", "enemy", 10, json!({"armor": 25})), unit("p_foe_b", "enemy", 8, json!({})),
            unit("p_derived", "enemy", 6, json!({"poise": -1})), unit("p_nobar", "enemy", 4, json!({"poise": 0}))
        ],
        "derived_stats": [{"id": "max_poise", "name": "Max Poise", "formula": "CONS * per_cons + gear_mitigation", "constants": {"per_cons": 2}, "min": 1, "max": 200, "rulesets": ["timeline"]}]
    }))
}

/// A timeline ruleset with `rules` (the Max Poise row named; `extra` overrides numbers).
fn timeline(rules: &[&str], extra: Value) -> Ruleset {
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "poise_stat": "max_poise"});
    row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    Ruleset::from_row(row.as_object().unwrap())
}

fn poise_rules() -> Ruleset {
    timeline(&[POISE, PIERCE], json!({}))
}

/// The six units (ids 0..6), deployed, then placed: hero (4,4) faces foe (5,4); foe_b (5,5) below the foe; the others
/// in far corners.
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "p_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe_b", "p_derived", "p_nobar"],
        "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(0, 8)), (FOE, V2::new(5, 4)), (FOE_B, V2::new(5, 5)), (DERIVED, V2::new(11, 0)), (NO_BAR, V2::new(11, 8))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// The command unit `c` would run for `step` of skill `skill` (the planner's own build).
fn command(w: &World, c: UnitId, step: Step, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

fn hit(w: &World, c: UnitId, t: UnitId, skill: &str) -> Command {
    command(w, c, PlanStep::engage(sid(w, skill), t), skill)
}

fn own(w: &World, c: UnitId, skill: &str) -> Command {
    command(w, c, PlanStep::self_skill(sid(w, skill)), skill)
}

/// Books `cmd` for `c` from `start` (refused = a test bug).
fn book(res: &mut Resolver, c: UnitId, cmd: Command, start: i32) -> usize {
    assert!(res.schedule(c, cmd, start), "booked at {start}");
    res.scheduled.len() - 1
}

/// Runs blocks `from..=to` (starts, then every landing action).
fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

fn set_poise(w: &mut World, c: UnitId, value: i32) {
    w.poise.get_mut(&c).unwrap().value = value;
}

fn bar(w: &World, c: UnitId) -> PoiseState {
    *poise::state(w, c).expect("a poise state")
}

/// P1-P3: Max Poise is the row's fixed `poise`, none for 0, else the ruleset's derived row; every bar refills at the
/// round start and remembers last round's break; classic keeps no bars.
#[test]
fn max_poise_follows_the_unit_row_and_refills_each_round() {
    let s = session(poise_rules());
    let w = &s.w;
    for c in [HERO, ALLY, FOE, FOE_B] {
        assert_eq!(bar(w, c), PoiseState { max: BAR, value: BAR, broken: false, broken_last: false }, "unit {c}: the fixed poise");
    }
    let derived_max = derived::value_for(&w.cat, &w.ruleset, &w.units[DERIVED], "max_poise").expect("the row is in timeline");
    assert!(derived_max >= 1);
    assert_eq!((bar(w, DERIVED).max, bar(w, DERIVED).value), (derived_max, derived_max), "poise -1: the derived row");
    assert_eq!(bar(w, NO_BAR).max, 0, "poise 0: no bar");
    let mut w = s.w.clone();
    set_poise(&mut w, HERO, 3);
    w.poise.get_mut(&HERO).unwrap().broken = true;
    poise::round_start(&mut w);
    assert_eq!(bar(&w, HERO), PoiseState { max: BAR, value: BAR, broken: false, broken_last: true }, "refilled; the break is remembered");
    poise::round_start(&mut w);
    assert!(!bar(&w, HERO).broken_last, "two rounds on, forgotten");
    let classic = session(Ruleset::classic());
    assert!(classic.w.poise.is_empty() && poise::state(&classic.w, HERO).is_none(), "classic keeps no bars");
    let mut off = poise_rules();
    off.adjust("-poise").unwrap();
    assert!(session(off).w.poise.is_empty(), "the extension switched off");
}

/// P6: every combination of windup, guard, repeat, flank and passive over random stagger: `roundi(s x product)`, the
/// factors read from the battle (a running action, a running Defend, last round's break, the hit, the passives).
#[test]
fn stagger_is_the_product_of_its_factors() {
    let s = session(poise_rules());
    let n = s.w.ruleset.numbers.clone();
    let mut rng = aegis_core::rng::GodotRng::new(99);
    for mask in 0..32u32 {
        let (windup, guard, repeat, flank, passive) = (mask & 1 != 0, mask & 2 != 0, mask & 4 != 0, mask & 8 != 0, mask & 16 != 0);
        for _ in 0..12 {
            let stagger = rng.randi_range(0, 100);
            let mut w = s.w.clone();
            let mut res = Resolver::new(BLOCKS, 1);
            w.poise.insert(FOE, PoiseState { max: 1000, value: 1000, broken: false, broken_last: repeat });
            if passive {
                w.units[FOE].stagger_taken = vec![StaggerTaken { mult: 0.5, max_blocks: 0 }];
            }
            if windup {
                let cmd = hit(&w, FOE, HERO, "p_heavy");
                res.scheduled.push(ScheduledAction { unit: FOE, command: cmd, start: 1, impact: 3, end: 3, cancelled: false, landed: false, broken: false });
            }
            if guard {
                let cmd = own(&w, FOE, "p_guard");
                res.scheduled.push(ScheduledAction { unit: FOE, command: cmd, start: 1, impact: 2, end: 2, cancelled: false, landed: false, broken: false });
            }
            let mut cmd = hit(&w, HERO, FOE, "p_jab");
            cmd.impact.stagger = stagger;
            cmd.hits = vec![(FOE, flank)];
            let a = book(&mut res, HERO, cmd, 2);
            poise::resolve_hits(&mut res, &mut w, a);
            let pick = |on: bool, m: f64| if on { m } else { 1.0 };
            let want = roundi(stagger as f64 * pick(windup, n.stagger_windup_mult) * pick(guard, n.stagger_guard_mult) * pick(repeat, n.stagger_repeat_mult) * pick(flank, n.flank_stagger_mult) * pick(passive, 0.5)).max(0);
            let case = format!("s {stagger}, windup {windup}, guard {guard}, repeat {repeat}, flank {flank}, passive {passive}");
            if stagger == 0 {
                assert!(res.scheduled[a].command.staggers.is_empty(), "{case}: a skill without stagger staggers nobody");
                continue;
            }
            let st = &res.scheduled[a].command.staggers[0];
            assert_eq!(st.taken, want, "{case}");
            assert_eq!((st.windup, st.before, st.after, st.broke.is_none()), (windup, 1000, 1000 - want, true), "{case}");
            assert_eq!(bar(&w, FOE).value, 1000 - want, "{case}");
        }
    }
}

/// P6: a `stagger_taken` passive applies to actions of at most `max_blocks` blocks (0 = any): Snap-proof (mult 0,
/// max_blocks 1) takes nothing from a Snap and full stagger from a 2-block thrust.
#[test]
fn a_passive_matches_the_action_blocks() {
    let s = session(poise_rules());
    for (skill, blocks, base) in [("p_jab", 1, 12), ("p_thrust", 2, 8)] {
        let mut w = s.w.clone();
        w.units[FOE].stagger_taken = vec![StaggerTaken { mult: 0.0, max_blocks: 1 }];
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, skill), 1);
        run(&mut w, &mut res, 1, blocks);
        let st = &res.scheduled[a].command.staggers[0];
        assert_eq!(st.taken, if blocks <= 1 { 0 } else { base }, "{skill}");
    }
}

/// P7, P9: a break cancels every action of the unit that has not landed: a later action in the same block, its Rest
/// blocks and an action not started (unpaid); paid AP stays spent; the log names the action in windup. A running guard
/// drops at once; the recovery of a landed action ends.
#[test]
fn a_break_cancels_what_has_not_landed() {
    let s = session(poise_rules());
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    // the foe: a jab landing in block 2 after the hero's (lower initiative), a Rest in 3, a heavy in 4-6
    let foe_jab = book(&mut res, FOE, hit(&w, FOE, HERO, "p_jab"), 2);
    let foe_rest = book(&mut res, FOE, own(&w, FOE, "rest"), 3);
    let foe_heavy = book(&mut res, FOE, hit(&w, FOE, HERO, "p_heavy"), 4);
    let hero_jab = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 2);
    set_poise(&mut w, FOE, 1);
    let (ap0, hero_hp0) = (w.units[FOE].ap, w.units[HERO].hp);
    run(&mut w, &mut res, 1, BLOCKS);
    let st = res.scheduled[hero_jab].command.staggers[0].clone();
    assert_eq!(st.broke.as_ref().map(|b| b.cause), Some(BreakCause::Poise));
    assert_eq!(st.broke.as_ref().unwrap().skill, "p_jab", "the action in windup is named");
    for i in [foe_jab, foe_rest, foe_heavy] {
        assert!(res.scheduled[i].cancelled && res.scheduled[i].broken && !res.scheduled[i].landed, "action {i} cancelled");
    }
    assert_eq!(w.units[HERO].hp, hero_hp0, "the foe's jab never landed (no mutual trade)");
    let jab = &res.scheduled[foe_jab].command;
    assert_eq!(w.units[FOE].ap, ap0 - jab.ap_cost - jab.stance_switch_cost, "the started jab stays paid; the heavy pays nothing");
    assert_eq!((bar(&w, FOE).value, bar(&w, FOE).broken), (0, true));
    assert!(res.log.iter().any(|l| l == "[Break] p_foe's p_jab is broken"), "{:?}", res.log);

    // a running guard drops at once: one still winding to its landing (cancelled), one that landed earlier in the block
    // (a Defend lands first: priority 1) and still runs (its guard broken)
    for (start, landed) in [(2, false), (1, true)] {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        let guard = book(&mut res, FOE, own(&w, FOE, "p_guard"), start);
        book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 2);
        set_poise(&mut w, FOE, 1);
        run(&mut w, &mut res, 1, 1);
        let prep = res.prepare_block(&mut w, 2);
        assert!(w.units[FOE].guard > 0.0, "guard from {start}: guarding before the hit");
        for i in prep {
            res.execute_single(&mut w, i);
        }
        let g = &res.scheduled[guard];
        assert_eq!((g.landed, g.cancelled), (landed, !landed), "guard from {start}");
        assert!(!poise::guarded(&res.scheduled, FOE, 2), "guard from {start}: no Defend runs any more");
        assert_eq!(w.units[FOE].guard, 0.0, "guard from {start}: the guard drops at once");
    }

    // a landed action's recovery ends (an enemy under enemy_moves books recovery)
    let s = session(timeline(&[POISE, ENEMY_MOVES], json!({})));
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let rec = book(&mut res, FOE, hit(&w, FOE, HERO, "p_jab_rec"), 1);
    assert_eq!(res.scheduled[rec].end, 3, "lands in 1, recovers in 2-3");
    book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 2);
    set_poise(&mut w, FOE, 1);
    run(&mut w, &mut res, 1, 2);
    assert!(res.scheduled[rec].landed && !res.scheduled[rec].cancelled && res.scheduled[rec].end == 2, "the recovery ends at the break");
}

/// P8, P10: two breaking hits in a round record one break; the next round the unit takes stagger x repeat.
#[test]
fn one_break_a_round_and_double_stagger_after() {
    let s = session(poise_rules());
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let first = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 1);
    let second = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 2);
    set_poise(&mut w, FOE, 1);
    run(&mut w, &mut res, 1, 2);
    assert!(res.scheduled[first].command.staggers[0].broke.is_some());
    let again = &res.scheduled[second].command.staggers[0];
    assert_eq!((again.broke.clone(), again.before, again.after), (None, 0, 0), "no second break this round");
    assert_eq!(res.log.iter().filter(|l| l.starts_with("[Break]")).count(), 1);
    poise::round_start(&mut w);
    let mut res = Resolver::new(BLOCKS, 2);
    let next = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 1);
    run(&mut w, &mut res, 1, 1);
    let st = &res.scheduled[next].command.staggers[0];
    let n = &w.ruleset.numbers;
    assert_eq!((st.factors.repeat, st.taken), (n.stagger_repeat_mult, roundi(12.0 * n.stagger_repeat_mult)), "P10");
    poise::round_start(&mut w);
    let mut res = Resolver::new(BLOCKS, 3);
    let later = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(res.scheduled[later].command.staggers[0].taken, 12, "no break last round: no repeat factor");
}

/// P11: windup poise rises at the start and comes off at the landing, never below 0; P12: an unbreakable windup and
/// `steadfast` hold at 1 and draw no chance roll.
#[test]
fn windup_poise_and_unbreakable_hold_a_unit_up() {
    let s = session(timeline(&[POISE], json!({"break_chance_scale": 1.0})));
    let n = s.w.ruleset.numbers.clone();
    // no stagger: up by 20 for the windup, back to the bar when it lands
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "p_smash"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(bar(&w, FOE).value, BAR + 20);
    run(&mut w, &mut res, 2, 3);
    assert_eq!(bar(&w, FOE).value, BAR, "back to the bar");
    // a jab in its windup: (bar + 20 - 24), then the bonus comes off, never below 0
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "p_smash"), 1);
    let jab = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 2);
    run(&mut w, &mut res, 1, 3);
    let taken = roundi(12.0 * n.stagger_windup_mult);
    assert_eq!(res.scheduled[jab].command.staggers[0].after, BAR + 20 - taken);
    assert_eq!(bar(&w, FOE).value, (BAR + 20 - taken - 20).max(0));

    // unbreakable: a breaking hit leaves 1 and rolls no chance; after the landing the unit breaks
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "p_judgment"), 1);
    let held = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 2);
    let after = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 4);
    set_poise(&mut w, FOE, 5);
    run(&mut w, &mut res, 1, 4);
    let st = &res.scheduled[held].command.staggers[0];
    assert_eq!((st.after, st.broke.clone(), st.chance), (1, None, None), "held at 1, no roll");
    assert!(res.scheduled[after].command.staggers[0].broke.is_some(), "landed: breakable again");

    // steadfast: held at 1 out of windup; a sure break does not break it either
    for skill in ["p_jab", "p_sure"] {
        let mut w = s.w.clone();
        status::grant_now(&mut w.units[FOE], poise::STEADFAST, 1, 0, "test");
        let mut res = Resolver::new(BLOCKS, 1);
        book(&mut res, FOE, hit(&w, FOE, HERO, "p_heavy"), 1);
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, skill), 2);
        set_poise(&mut w, FOE, 3);
        run(&mut w, &mut res, 1, 2);
        let st = &res.scheduled[a].command.staggers[0];
        assert_eq!((st.after, st.broke.clone(), st.chance), (1, None, None), "{skill}: steadfast");
    }
}

/// P13: with `break_chance_scale` 0 no hit draws; with 0.5 each hit into a windup that leaves poise above 0 draws once
/// (`min(cap, floor(100 x taken / before x scale))`), a success breaks; a hit out of windup never draws; the same seed
/// gives the same result.
#[test]
fn the_break_chance_draws_once_per_qualifying_hit() {
    let one = |scale: f64, seed: i64, windup: bool| {
        let s = session(timeline(&[POISE], json!({"break_chance_scale": scale})));
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, seed);
        if windup {
            book(&mut res, FOE, hit(&w, FOE, HERO, "p_heavy"), 1);
        }
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, "p_jab"), 2);
        set_poise(&mut w, FOE, 30);
        run(&mut w, &mut res, 1, 2);
        (res.scheduled[a].command.staggers[0].clone(), res.rng.draws, w.ruleset.numbers.clone())
    };
    let mut successes = 0;
    for seed in 1..=60 {
        let (off, off_draws, _) = one(0.0, seed, true);
        let (on, on_draws, n) = one(0.5, seed, true);
        assert_eq!(off.chance, None, "seed {seed}: scale 0 rolls nothing");
        assert_eq!(on_draws, off_draws + 1, "seed {seed}: one draw for the hit");
        let want = (100.0 * on.taken as f64 / 30.0 * n.break_chance_scale).floor() as i32;
        let (chance, roll) = on.chance.expect("rolled");
        assert_eq!(chance, want.min(n.break_chance_cap), "seed {seed}");
        assert_eq!(on.broke.as_ref().map(|b| b.cause), (roll <= chance).then_some(BreakCause::Chance), "seed {seed}: a success breaks");
        successes += (roll <= chance) as i32;
        assert_eq!(one(0.5, seed, true).0, on, "seed {seed}: the same seed, the same result");
        let (out, out_draws, _) = one(0.5, seed, false);
        assert_eq!((out.chance, out_draws), (None, off_draws), "seed {seed}: out of windup no roll");
    }
    assert!(successes > 0 && successes < 60, "the seeds cover both outcomes ({successes})");
}

/// P14: `sure_break` breaks a target in windup whatever its poise, not one out of windup, not an unbreakable windup.
#[test]
fn a_sure_break_breaks_a_windup() {
    let s = session(poise_rules());
    for (victim, breaks) in [(Some("p_heavy"), true), (None, false), (Some("p_judgment"), false), (Some("p_focus"), true)] {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        if let Some(skill) = victim {
            let cmd = if skill == "p_focus" { own(&w, FOE, skill) } else { hit(&w, FOE, HERO, skill) };
            book(&mut res, FOE, cmd, 1);
        }
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, "p_sure"), 2);
        run(&mut w, &mut res, 1, 2);
        let st = &res.scheduled[a].command.staggers[0];
        assert!(st.before > st.taken, "{victim:?}: poise alone would not break");
        assert_eq!(st.broke.as_ref().map(|b| b.cause), breaks.then_some(BreakCause::Sure), "{victim:?}");
    }
}

/// P4, RS3: an area staggers every unit it damages, in hit order; landed Rest restores poise up to Max Poise; a unit
/// without a bar takes nothing.
#[test]
fn areas_rest_and_units_without_a_bar() {
    let s = session(poise_rules());
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, HERO, PlanStep::area_at(sid(&w, "p_burst"), V2::new(5, 4)), true), "");
    let cmd = plan.entries_for(&w, HERO).into_iter().find(|e| e.skill == sid(&w, "p_burst")).unwrap().build_command(&w, HERO);
    let a = book(&mut res, HERO, cmd, 1);
    run(&mut w, &mut res, 1, 1);
    let hits: Vec<UnitId> = res.scheduled[a].command.hits.iter().map(|h| h.0).collect();
    let staggered: Vec<(UnitId, i32)> = res.scheduled[a].command.staggers.iter().map(|st| (st.target, st.taken)).collect();
    assert!(hits.contains(&FOE) && hits.contains(&FOE_B), "{hits:?}");
    assert_eq!(staggered, hits.iter().map(|&h| (h, 10)).collect::<Vec<_>>(), "each damaged unit, in hit order");

    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    for b in 1..=3 {
        book(&mut res, FOE, own(&w, FOE, "rest"), b);
    }
    set_poise(&mut w, FOE, 10);
    run(&mut w, &mut res, 1, 3);
    let per = w.ruleset.numbers.rest_poise_per_block;
    assert_eq!(bar(&w, FOE).value, (10 + 3 * per).min(BAR), "RS3");
    set_poise(&mut w, FOE, BAR - 1);
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, own(&w, FOE, "rest"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(bar(&w, FOE).value, BAR, "up to Max Poise");

    let mut w = s.w.clone();
    w.grid.positions[NO_BAR] = Some(V2::new(3, 4));
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&w, HERO, NO_BAR, "p_sure"), 1);
    book(&mut res, NO_BAR, hit(&w, NO_BAR, HERO, "p_heavy"), 1);
    run(&mut w, &mut res, 1, 1);
    assert!(res.scheduled[a].command.staggers.is_empty(), "no bar: never staggered, never broken");
}

/// MV3, P6: a flank hit (rules round_intent and flank) multiplies stagger by `flank_stagger_mult`; without the rule
/// `flank` the factor is 1.0.
#[test]
fn a_flank_hit_staggers_harder() {
    for flank_on in [true, false] {
        let rules: Vec<&str> = if flank_on { vec![ROUND_INTENT, POISE, FLANK] } else { vec![ROUND_INTENT, POISE] };
        let mut s = session(timeline(&rules, json!({})));
        let w = &mut s.w;
        let t_at = V2::new(5, 4);
        w.grid.positions[FOE] = Some(t_at);
        w.grid.positions[ALLY] = Some(t_at + V2::new(1, 0));
        w.grid.positions[FOE_B] = Some(V2::new(0, 0));
        w.grid.positions[HERO] = Some(t_at + V2::new(-1, 0));
        round_intent::set_intent(w, FOE, RoundIntent::Assault).unwrap();
        round_intent::set_target(w, FOE, RoundTarget::Unit(ALLY)).unwrap();
        round_intent::set_intent(w, HERO, RoundIntent::Assault).unwrap();
        round_intent::set_target(w, HERO, RoundTarget::Unit(FOE)).unwrap();
        assert!(round_intent::is_flank(w, HERO, FOE));
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, hit(w, HERO, FOE, "p_jab"), 1);
        run(w, &mut res, 1, 1);
        let st = &res.scheduled[a].command.staggers[0];
        let mult = if flank_on { w.ruleset.numbers.flank_stagger_mult } else { 1.0 };
        assert_eq!((st.factors.flank, st.taken), (mult, roundi(12.0 * mult)), "flank rule {flank_on}");
    }
}

/// A2: `compute` with `ignore_armor` equals a hit on the same target with armour 0; guard and cover still cut. In a
/// battle an `armor_piercing` hit skips armour only under the rule pierce and says so in its log cell.
#[test]
fn a_piercing_hit_skips_the_armour() {
    let s = session(poise_rules());
    let w = &s.w;
    let (a, t) = (&w.units[HERO], &w.units[FOE]);
    assert!(damage::armor_rating(t) > 0, "the fixture target wears armour");
    let mut bare = t.clone();
    bare.armor_override = 0;
    for (guard, cover) in [(0.0, 0.0), (0.5, 0.0), (0.3, 0.25)] {
        let (mut t1, mut t0) = (t.clone(), bare.clone());
        t1.guard = guard;
        t0.guard = guard;
        let input = HitInput { skill_mult: 1.0, cover, ..HitInput::default() };
        let pierced = damage::compute(a, &t1, HitInput { ignore_armor: true, ..input });
        let zero = damage::compute(a, &t0, input);
        assert_eq!((pierced.final_dmg, pierced.armor_ratio), (zero.final_dmg, 0.0), "guard {guard}, cover {cover}");
        assert!(pierced.armor_pierced && !zero.armor_pierced);
        assert_eq!(pierced.armor_rating, damage::armor_rating(t), "the rating is still named");
        assert!(pierced.final_dmg > damage::compute(a, &t1, input).final_dmg, "armour 25 cuts a plain hit");
    }
    let mut dealt = Vec::new();
    for rules in [poise_rules(), timeline(&[POISE], json!({})), Ruleset::classic()] {
        let pierce = rules.has(PIERCE);
        let mut s = session(rules);
        let mut res = Resolver::new(BLOCKS, 5);
        let i = book(&mut res, HERO, hit(&s.w, HERO, FOE, "p_spike"), 1);
        let hp = s.w.units[FOE].hp;
        run(&mut s.w, &mut res, 1, 1);
        dealt.push((pierce, hp - s.w.units[FOE].hp));
        let entry = rpgkit_battle::log::from_action(&s.w, &res.scheduled[i], 1, 1);
        assert_eq!(entry.detail.contains("Armour pierced"), pierce, "{}", entry.detail);
    }
    assert!(dealt[0].1 > dealt[1].1 && dealt[1].1 == dealt[2].1, "pierce on beats pierce off and classic: {dealt:?}");
}

/// The session: a break gets its EVENT cell (outcome BREAK), the broken unit's cancelled actions get none, the metrics
/// count it under `poise`; classic logs and records nothing of it (no `poise` key, no stagger detail, no new draw).
#[test]
fn the_session_logs_and_counts_breaks_and_classic_stays_as_it_was() {
    let mut s = session(poise_rules());
    s.attach_metrics(BattleMetrics::new(), true);
    set_poise(&mut s.w, FOE, 1);
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "p_jab"), FOE), true), "");
    while s.current_round == 1 && !s.combat_over && s.phase != rpgkit_battle::session::Phase::Replay {
        s.step_next();
    }
    let breaks: Vec<_> = s.log.iter().filter(|e| e.outcome == "BREAK").collect();
    assert_eq!(breaks.len(), 1, "one break cell");
    assert_eq!((breaks[0].category, breaks[0].actor.as_str(), breaks[0].status.as_str()), ("EVENT", "p_foe", "poise"));
    let foe_cells_after = s.log.iter().skip_while(|e| e.outcome != "BREAK").filter(|e| e.actor == "p_foe" && e.category != "EVENT").count();
    assert_eq!(foe_cells_after, 0, "the broken unit's cancelled actions get no cell");
    assert!(s.resolver.scheduled.iter().filter(|a| a.unit == FOE).all(|a| a.landed || a.cancelled));
    let m = s.metrics.as_ref().unwrap();
    assert_eq!((m.poise["p_hero"].breaks, m.poise["p_foe"].broken), (1, 1));
    assert!(m.to_json().get("poise").is_some());
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    let snap = s.snapshot();
    assert_eq!((snap[FOE].poise, snap[FOE].max_poise), (0, BAR));

    let mut c = session(Ruleset::classic());
    c.attach_metrics(BattleMetrics::new(), true);
    assert_eq!(c.plan.add_step(&c.w, HERO, PlanStep::engage(sid(&c.w, "p_jab"), FOE), true), "");
    while c.current_round == 1 && !c.combat_over && c.phase != rpgkit_battle::session::Phase::Replay {
        c.step_next();
    }
    assert!(c.w.poise.is_empty() && c.log.iter().all(|e| e.outcome != "BREAK" && !e.detail.contains("Stagger")));
    assert!(c.resolver.scheduled.iter().all(|a| a.command.staggers.is_empty() && a.command.poise_bonus == 0));
    let m = c.metrics.as_ref().unwrap();
    assert!(m.poise.is_empty() && m.to_json().get("poise").is_none(), "classic keeps the metrics shape");
    assert_eq!((c.snapshot()[FOE].poise, c.snapshot()[FOE].max_poise), (0, 0));
}

/// The planner's command carries the skill's poise fields (items carry none); classic carries them too but no rule reads
/// them.
#[test]
fn commands_carry_the_skill_impact() {
    for rules in [poise_rules(), Ruleset::classic()] {
        let s = session(rules);
        let w = &s.w;
        let jab = hit(w, HERO, FOE, "p_sure");
        assert_eq!((jab.impact.stagger, jab.impact.sure_break, jab.impact.armor_piercing), (4, true, false));
        let smash = hit(w, HERO, FOE, "p_smash");
        assert_eq!((smash.impact.windup_poise, smash.impact.unbreakable), (20, false));
        assert!(hit(w, HERO, FOE, "p_judgment").impact.unbreakable && hit(w, HERO, FOE, "p_spike").impact.armor_piercing);
        assert!(matches!(own(w, HERO, "p_guard").kind, CmdKind::Defend { .. }) && own(w, HERO, "p_guard").impact == poise::Impact::default());
    }
}
