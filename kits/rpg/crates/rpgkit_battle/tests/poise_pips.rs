//! S2.2-S1/S2 on a scratch catalog built here (no shipped data): poise pips (`docs/sessions/S2_POISE.md` 1, 3 and 4)
//! behind the ruleset's `poise_mode`. Expectations come from the rules: `T = max(1, roundi(max_poise / poise_pips))`; a
//! hit of stagger taken `D` removes `clamp(floor(D / T), 0, max_pip_loss_per_hit)` pips and the unit breaks at 0 pips;
//! `windup_poise` grants `roundi(windup_poise / T)` temporary pips (at least 1); `unbreakable` and `steadfast` hold at 1
//! pip; `sure_break` removes them all; the break chance is pips lost on pips before; Rest restores a pip per 2 Rest
//! blocks up to `poise_pips`; the arena `stagger` (the collision and trigger path) converts the same way. Under the
//! default `bar` nothing a pip number says changes an outcome or a draw.

use indexmap::IndexMap;
use rpgkit_battle::arena;
use rpgkit_battle::command::Command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::poise::{self, BreakCause, PlannedHit, PoiseState, StaggerHit, Terms, Verdict};
use rpgkit_battle::resolver::{Resolver, ScheduledAction};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{self, Numbers, PoiseMode, Ruleset, ARENA_HAZARDS, POISE, TIMELINE};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
/// The tank's fixed Max Poise (S2 section 5: 5 pips of T 12).
const TANK: i32 = 60;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;

/// Engine skills and the test skills: a Snap jab (its stagger set per hit), a 3-block heavy (a windup, no stagger), a
/// heavy with windup poise 20, an unbreakable heavy, a sure-break snap (stagger 4), a guard.
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
        attack("p_jab", 1, json!({"stagger": 10})),
        attack("p_heavy", 3, json!({})),
        attack("p_smash", 3, json!({"windup_poise": 20})),
        attack("p_judgment", 3, json!({"unbreakable": true})),
        attack("p_sure", 1, json!({"stagger": 4, "sure_break": true})),
        json!({"id": "p_guard", "name": "p_guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.5}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": speed, "attack_range": 1, "skill_cap": 5, "poise": TANK});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("p_hero", "party", 14), unit("p_ally", "party", 12), unit("p_foe", "enemy", 10)]}))
}

/// A timeline ruleset with the rules `poise` and `arena_hazards` (`extra` sets numbers).
fn timeline(extra: Value) -> Ruleset {
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": [POISE, ARENA_HAZARDS]});
    row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    Ruleset::from_row(row.as_object().unwrap())
}

fn pips_rules() -> Ruleset {
    timeline(json!({"poise_mode": "pips"}))
}

/// Hero (6, 4) faces the foe (7, 4); the ally in a far corner. `hazards`: the encounter's arena hazards.
fn session_with(ruleset: Ruleset, hazards: Value) -> Session {
    let row = json!({"id": "p_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337, "arena_hazards": hazards});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(6, 4)), (ALLY, V2::new(0, 8)), (FOE, V2::new(7, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn session(ruleset: Ruleset) -> Session {
    session_with(ruleset, json!([]))
}

fn command(w: &World, c: UnitId, step: Step, skill: &str) -> Command {
    let mut plan = Plan::new();
    let id = w.book().find(skill).unwrap_or_else(|| panic!("skill {skill}"));
    assert_eq!(plan.add_step(w, c, step, true), "", "{skill}");
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

fn hit(w: &World, c: UnitId, t: UnitId, skill: &str) -> Command {
    let id = w.book().find(skill).unwrap_or_else(|| panic!("skill {skill}"));
    command(w, c, PlanStep::engage(id, t), skill)
}

fn own(w: &World, c: UnitId, skill: &str) -> Command {
    let id = w.book().find(skill).unwrap_or_else(|| panic!("skill {skill}"));
    command(w, c, PlanStep::self_skill(id), skill)
}

fn book(res: &mut Resolver, c: UnitId, cmd: Command, start: i32) -> usize {
    assert!(res.schedule(c, cmd, start), "booked at {start}");
    res.scheduled.len() - 1
}

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

fn bar(w: &World, c: UnitId) -> PoiseState {
    *poise::state(w, c).expect("a poise state")
}

/// The hero's `skill` with stagger `stagger` lands on the foe at `block`: its stagger record (the resolver's own rule).
fn strike(w: &mut World, res: &mut Resolver, skill: &str, stagger: i32, block: i32) -> StaggerHit {
    let mut cmd = hit(w, HERO, FOE, skill);
    cmd.impact.stagger = stagger;
    cmd.hits = vec![(FOE, false)];
    let a = book(res, HERO, cmd, block);
    poise::resolve_hits(res, w, a);
    res.scheduled[a].command.staggers[0].clone()
}

/// Pushes a running action of `c` (`start..=end`, impact at `end`), not booked through the planner's AP.
fn running(res: &mut Resolver, c: UnitId, command: Command, start: i32, end: i32) {
    res.scheduled.push(ScheduledAction { unit: c, command, start, impact: end, end, cancelled: false, landed: false, broken: false });
}

fn pips_numbers() -> Numbers {
    Numbers { poise_mode: PoiseMode::Pips, ..Numbers::default() }
}

/// A full bar of Max Poise `max` under `n`.
fn full(n: &Numbers, max: i32) -> PoiseState {
    let mut st = PoiseState::default();
    poise::refill(n, &mut st, max);
    st
}

/// S2 section 3: the three row-only keys, their defaults (`bar` everywhere), how a row sets them and the register's notes
/// on them (ranges, the unknown mode id, the needs).
#[test]
fn the_ruleset_reads_the_pip_numbers_and_defaults_to_bar() {
    let d = Numbers::default();
    assert_eq!((d.poise_mode, d.poise_pips, d.max_pip_loss_per_hit), (PoiseMode::Bar, 5, 3), "S2 section 3 defaults");
    assert_eq!(Ruleset::classic().numbers.poise_mode, PoiseMode::Bar, "the frozen kit's classic");
    assert_eq!(timeline(json!({})).numbers.poise_mode, PoiseMode::Bar, "a row without the key");
    let r = timeline(json!({"poise_mode": "pips", "poise_pips": 6, "max_pip_loss_per_hit": 2}));
    let f = |k: &str| r.numbers.field(k).unwrap();
    assert_eq!((f("poise_mode"), f("poise_pips"), f("max_pip_loss_per_hit")), ("pips".to_string(), "6".to_string(), "2".to_string()));
    assert_eq!(timeline(json!({"poise_mode": "pip"})).numbers.poise_mode, PoiseMode::Bar, "an unknown id reads as bar (the content check refuses it)");
    assert_eq!(ruleset::field_rule_of("poise_mode").unwrap().values, &PoiseMode::IDS);
    for (k, v, lo, hi) in [("poise_pips", 9, 3, 8), ("poise_pips", 2, 3, 8), ("max_pip_loss_per_hit", 0, 1, 5), ("max_pip_loss_per_hit", 6, 1, 5)] {
        let n = timeline(json!({"poise_mode": "pips", k: v})).notes();
        assert!(n.errors.contains(&format!("{k} {v} is outside {lo}..{hi}.")), "{k} {v}: {:?}", n.errors);
    }
    assert!(timeline(json!({"poise_mode": "pips", "poise_pips": 8, "max_pip_loss_per_hit": 5})).notes().errors.is_empty(), "the range ends are in");
    let off = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [], "poise_mode": "pips"}).as_object().unwrap());
    assert!(off.notes().warnings.contains(&"poise_mode is set but read only with poise.".to_string()), "{:?}", off.notes().warnings);
    let unread = timeline(json!({"poise_pips": 6})).notes();
    assert!(unread.warnings.contains(&"poise_pips is set but read only with poise_mode=pips.".to_string()), "{:?}", unread.warnings);
}

/// S2 section 1: `T = max(1, roundi(max_poise / poise_pips))` and `clamp(floor(D / T), 0, max_pip_loss_per_hit)`.
#[test]
fn threshold_and_pip_loss_follow_the_formula() {
    for (pips, cap) in [(5, 3), (3, 1), (8, 5)] {
        let n = Numbers { poise_pips: pips, max_pip_loss_per_hit: cap, ..pips_numbers() };
        for max in 1..=120 {
            let t = (roundi(max as f64 / pips as f64)).max(1);
            assert_eq!(poise::threshold(&n, max), t, "max {max}, {pips} pips");
            for d in 0..=80 {
                assert_eq!(poise::pip_loss(&n, d, t), (d / t).min(cap), "D {d}, T {t}, cap {cap}");
            }
        }
    }
    let n = pips_numbers();
    assert_eq!((poise::threshold(&n, TANK), poise::threshold(&n, 25)), (12, 5), "S2's examples");
    assert_eq!(full(&n, TANK).pips, n.poise_pips, "a refill fills every pip");
    assert_eq!(full(&Numbers::default(), TANK).pips, 0, "bar keeps no pips");
}

/// S2 section 5, test 1 (the resolver's hits): the tank (Max Poise 60, 5 pips, T 12) takes five D = 10 hits and loses
/// no pip; one D = 26 hit removes 2. Under bar the same five hits leave 10 poise and the D = 26 hit breaks it.
#[test]
fn chip_hits_under_the_threshold_remove_no_pip() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let t = poise::threshold(&n, TANK);
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    for b in 1..=5 {
        let st = strike(&mut w, &mut res, "p_jab", 10, b);
        assert_eq!((st.taken, st.pips_before, st.pips_after, st.broke.is_none()), (10, n.poise_pips, n.poise_pips, true), "hit {b}: 10 < T {t}");
    }
    assert_eq!((bar(&w, FOE).pips, bar(&w, FOE).broken), (n.poise_pips, false), "five chip hits never break the tank");
    let heavy = strike(&mut w, &mut res, "p_jab", 26, 6);
    assert_eq!((heavy.taken, heavy.pips_before - heavy.pips_after), (26, 26 / t), "floor(26 / 12) = 2");
    assert_eq!(26 / t, 2);
    assert_eq!(bar(&w, FOE).pips, n.poise_pips - 2);

    let s = session(timeline(json!({})));
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    for b in 1..=5 {
        strike(&mut w, &mut res, "p_jab", 10, b);
    }
    assert_eq!(bar(&w, FOE).value, TANK - 50, "bar: every chip counts");
    assert!(strike(&mut w, &mut res, "p_jab", 26, 6).broke.is_some(), "bar: the heavy hit breaks the chipped bar");
}

/// S2 section 5, test 2: a running Defend halves the stagger (`stagger_guard_mult`) before `floor(D / T)`: a stagger 20
/// hit takes 10, no pip against T 12; unguarded it removes 1.
#[test]
fn guard_halves_the_stagger_before_the_threshold() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let t = poise::threshold(&n, TANK);
    for guarded in [false, true] {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        if guarded {
            let g = own(&w, FOE, "p_guard");
            running(&mut res, FOE, g, 1, 2);
        }
        let st = strike(&mut w, &mut res, "p_jab", 20, 2);
        let want = roundi(20.0 * if guarded { n.stagger_guard_mult } else { 1.0 });
        assert_eq!(st.taken, want, "guarded {guarded}");
        assert_eq!(st.pips_before - st.pips_after, want / t, "guarded {guarded}");
    }
    assert_eq!(roundi(20.0 * n.stagger_guard_mult) / t, 0, "S2: 10 < 12, no pip");
}

/// FB-49: `windup_poise` grants `roundi(windup_poise / T)` temporary pips (at least 1 above 0), which come off when the
/// action lands (the value mirrors them, S2.2-S3); under bar the value rises and no pip is kept.
#[test]
fn windup_poise_grants_temporary_pips() {
    assert_eq!((poise::bonus_pips(0, 12), poise::bonus_pips(1, 12), poise::bonus_pips(5, 12), poise::bonus_pips(20, 12), poise::bonus_pips(30, 12)), (0, 1, 1, 2, 3));
    for t in 1..=20 {
        for wp in 1..=60 {
            assert_eq!(poise::bonus_pips(wp, t), roundi(wp as f64 / t as f64).max(1), "windup poise {wp}, T {t}");
        }
    }
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let bonus = poise::bonus_pips(20, poise::threshold(&n, TANK));
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "p_smash"), 1);
    run(&mut w, &mut res, 1, 1);
    // S2.2-S3 (FB-71): the value mirrors the pips (pips x T)
    let t = poise::threshold(&n, TANK);
    assert_eq!((bar(&w, FOE).pips, bar(&w, FOE).value), (n.poise_pips + bonus, (n.poise_pips + bonus) * t), "the windup adds its pips");
    run(&mut w, &mut res, 2, 3);
    assert_eq!((bar(&w, FOE).pips, bar(&w, FOE).value), (n.poise_pips, n.poise_pips * t), "landed: back to full");

    let s = session(timeline(json!({})));
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "p_smash"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!((bar(&w, FOE).pips, bar(&w, FOE).value), (0, TANK + 20), "bar: no pips");
}

/// FB-49: `steadfast` and an `unbreakable` windup hold the pips at 1; nothing breaks.
#[test]
fn steadfast_and_unbreakable_hold_one_pip() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let mut w = s.w.clone();
    status::grant_now(&mut w.units[FOE], poise::STEADFAST, 1, 0, "test");
    w.poise.get_mut(&FOE).unwrap().pips = 2;
    let mut res = Resolver::new(BLOCKS, 1);
    let big = poise::threshold(&n, TANK) * n.max_pip_loss_per_hit;
    for b in 1..=2 {
        let st = strike(&mut w, &mut res, "p_jab", big, b);
        assert_eq!((st.pips_after, st.broke.is_none()), (1, true), "steadfast hit {b}");
    }

    let mut w = s.w.clone();
    w.poise.get_mut(&FOE).unwrap().pips = 1;
    let mut res = Resolver::new(BLOCKS, 1);
    let j = hit(&w, FOE, HERO, "p_judgment");
    running(&mut res, FOE, j, 1, 3);
    let st = strike(&mut w, &mut res, "p_jab", big, 2);
    assert_eq!((st.windup, st.pips_after, st.broke.is_none()), (true, 1, true), "unbreakable windup");
    res.scheduled[0].landed = true;
    let st = strike(&mut w, &mut res, "p_jab", big, 4);
    assert!(st.broke.is_some(), "landed: breakable again");
}

/// FB-49: a `sure_break` hit into a windup removes every pip (a break) whatever its stagger; out of a windup it only
/// staggers.
#[test]
fn sure_break_removes_every_pip() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    for windup in [false, true] {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        if windup {
            let h = hit(&w, FOE, HERO, "p_heavy");
            running(&mut res, FOE, h, 1, 3);
        }
        let st = strike(&mut w, &mut res, "p_sure", 4, 2);
        assert!(poise::pip_loss(&n, st.taken, poise::threshold(&n, TANK)) == 0, "the stagger alone removes no pip");
        if windup {
            assert_eq!(st.broke.as_ref().map(|b| b.cause), Some(BreakCause::Sure));
            assert_eq!((st.pips_after, bar(&w, FOE).pips, bar(&w, FOE).broken), (0, 0, true));
        } else {
            assert_eq!((st.broke.is_none(), st.pips_after), (true, n.poise_pips));
        }
    }
}

/// FB-49: landed Rest restores a pip per 2 Rest blocks of the round, up to `poise_pips`; a broken unit restores none;
/// under bar no pip is kept.
#[test]
fn rest_restores_a_pip_per_two_blocks() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let mut w = s.w.clone();
    w.poise.get_mut(&FOE).unwrap().pips = 1;
    let mut total = 0;
    for blocks in [1, 1, 3, 1, 4, 2] {
        poise::restore(&mut w, FOE, blocks);
        total += blocks;
        assert_eq!(bar(&w, FOE).pips, (1 + total / 2).min(n.poise_pips), "after {total} Rest blocks");
    }
    let mut w = s.w.clone();
    w.poise.get_mut(&FOE).unwrap().pips = 1;
    let mut res = Resolver::new(BLOCKS, 1);
    for b in 1..=2 {
        book(&mut res, FOE, own(&w, FOE, "rest"), b);
    }
    run(&mut w, &mut res, 1, 2);
    assert_eq!(bar(&w, FOE).pips, 2, "two landed Rest blocks restore a pip");
    let mut w = s.w.clone();
    let st = w.poise.get_mut(&FOE).unwrap();
    (st.pips, st.broken) = (0, true);
    poise::restore(&mut w, FOE, 4);
    assert_eq!(bar(&w, FOE).pips, 0, "broken: nothing until the refill");
    poise::round_start(&mut w);
    assert_eq!((bar(&w, FOE).pips, bar(&w, FOE).rest_blocks), (n.poise_pips, 0), "the refill");

    let s = session(timeline(json!({})));
    let mut w = s.w.clone();
    poise::restore(&mut w, FOE, 4);
    assert_eq!(bar(&w, FOE).pips, 0, "bar: no pips");
}

/// FB-49: the break chance of a hit into a windup is `break_chance(pips lost, pips before)`; a chip hit removes no pip and
/// rolls nothing (bar rolls on the stagger taken over the poise).
#[test]
fn the_break_chance_is_pips_lost_over_pips_before() {
    let n = Numbers { break_chance_scale: 1.0, break_chance_cap: 100, ..pips_numbers() };
    let st = full(&n, TANK);
    let t = poise::threshold(&n, TANK);
    let terms = Terms { windup: true, held: false, sure: None, chance: true };
    for d in [10, 24, 36, 100] {
        let lost = poise::pip_loss(&n, d, t);
        let want = if lost == 0 { Verdict::Holds } else { Verdict::Chance(poise::break_chance(&n, lost, st.pips)) };
        assert_eq!(poise::settle(&n, &st, d, terms).verdict, want, "D {d}");
    }
    assert_eq!(poise::break_chance(&n, 2, 5), 40, "2 of 5 pips at scale 1");
    let b = Numbers { poise_mode: PoiseMode::Bar, ..n.clone() };
    let bst = full(&b, TANK);
    assert_eq!(poise::settle(&b, &bst, 24, terms).verdict, Verdict::Chance(poise::break_chance(&b, 24, TANK)), "bar: taken over poise");

    // the resolver: a chip hit into a windup draws nothing under pips and rolls under bar
    for mode in ["pips", "bar"] {
        let s = session(timeline(json!({"poise_mode": mode, "break_chance_scale": 1.0})));
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        let h = hit(&w, FOE, HERO, "p_heavy");
        running(&mut res, FOE, h, 1, 3);
        let st = strike(&mut w, &mut res, "p_jab", 5, 2);
        assert_eq!(st.taken, roundi(5.0 * s.w.ruleset.numbers.stagger_windup_mult));
        assert_eq!(st.chance.is_some(), mode == "bar", "{mode}");
    }
}

/// S2.2-S2: the arena `stagger` (the collision path the trigger `stagger` shares) converts through `floor(D / T)`.
#[test]
fn arena_stagger_converts_through_the_threshold() {
    let n = pips_rules().numbers;
    let t = poise::threshold(&n, TANK);
    for stagger in [10, 30] {
        let vents = json!([{"id": "vents", "name": "Vents", "columns": [7], "blocks": [1], "line": "The vents roar.", "effect": {"stagger": stagger}}]);
        let s = session_with(pips_rules(), vents);
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        for c in 0..w.units.len() {
            res.register(c);
        }
        let strikes = arena::block_end(&mut res, &mut w, 1, 1, &[]);
        let st = strikes[0].hits.iter().find(|h| h.unit == FOE).and_then(|h| h.stagger.clone()).expect("the foe is struck");
        assert_eq!((st.taken, st.pips_before - st.pips_after), (stagger, stagger / t), "stagger {stagger}");
        assert_eq!(bar(&w, FOE).pips, n.poise_pips - stagger / t);
    }
}

/// S2.2-S2: `forecast_hits` carries the pips along a sequence and reports T; under bar its pip fields are the bar's
/// derived pips and every other field is the one a pip-less forecast gives.
#[test]
fn forecast_hits_carry_the_pips() {
    let planned = |w: &World, stagger: i32, block: i32| {
        let mut imp = hit(w, HERO, FOE, "p_jab").impact;
        imp.stagger = stagger;
        PlannedHit { target: FOE, block, impact: imp, blocks: 1, flank: false, ignore_guard: false }
    };
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let res = Resolver::new(BLOCKS, 1);
    let seq = poise::forecast_hits(&s.w, &res.scheduled, &[planned(&s.w, 26, 1), planned(&s.w, 26, 2), planned(&s.w, 26, 3)]);
    let pips: Vec<(i32, i32, i32)> = seq.iter().map(|f| f.map(|f| (f.threshold, f.pips_before, f.pips_after)).unwrap()).collect();
    let t = poise::threshold(&n, TANK);
    let p = n.poise_pips;
    assert_eq!(pips, vec![(t, p, p - 2), (t, p - 2, p - 4), (t, p - 4, 0)]);
    assert_eq!(seq[2].unwrap().verdict, Verdict::Breaks(BreakCause::Poise), "0 pips: broken");

    let s = session(timeline(json!({})));
    let b = s.w.ruleset.numbers.clone();
    let seq = poise::forecast_hits(&s.w, &res.scheduled, &[planned(&s.w, 26, 1)]);
    let f = seq[0].unwrap();
    assert_eq!((f.before, f.after, f.verdict), (TANK, TANK - 26, Verdict::Holds));
    let after = PoiseState { value: f.after, ..bar(&s.w, FOE) };
    assert_eq!((f.threshold, f.pips_before, f.pips_after), (t, poise::bar_pips(&b, &bar(&s.w, FOE)), poise::bar_pips(&b, &after)));
}

/// The bar rule as RULESET1 P7, P12-P14 state it (the pre-S2 code): the reference `bar` must keep.
fn bar_reference(n: &Numbers, st: &PoiseState, taken: i32, h: Terms) -> (i32, Verdict) {
    if st.broken {
        return (st.value, Verdict::AlreadyBroken);
    }
    let mut after = st.value - taken;
    if h.held {
        after = after.max(st.value.min(1));
    }
    let verdict = if taken > 0 && after <= 0 && !h.held {
        Verdict::Breaks(BreakCause::Poise)
    } else if h.windup && h.sure.is_some() && !h.held {
        Verdict::Breaks(h.sure.unwrap())
    } else if h.chance && h.windup && !h.held && taken > 0 && after > 0 && n.break_chance_scale > 0.0 {
        Verdict::Chance(poise::break_chance(n, taken, st.value))
    } else {
        Verdict::Holds
    };
    (after.max(0), verdict)
}

/// S2.2-S2: under `bar` (the default) a sample of hits settles exactly as the bar rule says whatever the pip numbers and
/// the stored pips; the resolver's records and bars match with other pip numbers set.
#[test]
fn bar_mode_ignores_the_pip_numbers() {
    let mut rng = aegis_core::rng::GodotRng::new(2207);
    for _ in 0..2000 {
        let n = Numbers {
            poise_pips: rng.randi_range(3, 8),
            max_pip_loss_per_hit: rng.randi_range(1, 5),
            break_chance_scale: if rng.randi_range(0, 1) == 1 { 1.0 } else { 0.0 },
            ..Numbers::default()
        };
        let max = rng.randi_range(1, 100);
        let st = PoiseState { max, value: rng.randi_range(0, max + 30), broken: rng.randi_range(0, 5) == 0, broken_last: false, pips: rng.randi_range(0, 9), rest_blocks: 0 };
        let sure = match rng.randi_range(0, 2) {
            0 => None,
            1 => Some(BreakCause::Sure),
            _ => Some(BreakCause::Collision),
        };
        let h = Terms { windup: rng.randi_range(0, 1) == 1, held: rng.randi_range(0, 3) == 0, sure, chance: rng.randi_range(0, 1) == 1 };
        let taken = rng.randi_range(0, 120);
        let got = poise::settle(&n, &st, taken, h);
        assert_eq!((got.after, got.verdict, got.pips), (bar_reference(&n, &st, taken, h).0, bar_reference(&n, &st, taken, h).1, st.pips), "{st:?} taken {taken} {h:?}");
    }

    let plain = session(timeline(json!({"break_chance_scale": 1.0})));
    let other = session(timeline(json!({"break_chance_scale": 1.0, "poise_pips": 3, "max_pip_loss_per_hit": 1})));
    let mut out = Vec::new();
    for s in [&plain, &other] {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        let h = hit(&w, FOE, HERO, "p_heavy");
        running(&mut res, FOE, h, 1, 3);
        let hits: Vec<(i32, i32, i32, Option<(i32, i32)>, bool)> =
            [12, 7, 30].iter().zip(1..).map(|(&sg, b)| strike(&mut w, &mut res, "p_jab", sg, b)).map(|st| (st.taken, st.before, st.after, st.chance, st.broke.is_some())).collect();
        out.push((hits, bar(&w, FOE).value, bar(&w, FOE).pips, res.rng.randi_range(1, 1000)));
    }
    assert_eq!(out[0], out[1], "same records, bars and RNG state");
    assert_eq!(out[0].2, 0, "bar stores no pips");
}

// ---- S2.2-S3 (FB-71): the value mirrors the pips, the stored bonus count, the Rest preview -----------------------

/// FB-71 (1): under pips the value is `pips x T` after a refill, a hit, a windup bonus, a Rest and a break; under bar the
/// value keeps its own numbers.
#[test]
fn the_value_mirrors_the_pips() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let t = poise::threshold(&n, TANK);
    let mirrored = |w: &World| {
        let st = bar(w, FOE);
        assert_eq!(st.value, st.pips * t, "{st:?}");
    };
    let mut w = s.w.clone();
    mirrored(&w);
    assert_eq!(bar(&w, FOE).pips, n.poise_pips, "a full bar");
    let mut res = Resolver::new(BLOCKS, 1);
    for (b, stagger) in [(1, t - 1), (2, t), (3, 2 * t + 1)] {
        let st = strike(&mut w, &mut res, "p_jab", stagger, b);
        assert_eq!((st.after, st.pips_after), (bar(&w, FOE).value, bar(&w, FOE).pips), "the record is the bar");
        mirrored(&w);
    }
    assert_eq!(bar(&w, FOE).pips, n.poise_pips - 3, "a chip, one pip, two pips");
    poise::restore(&mut w, FOE, 2);
    assert_eq!(bar(&w, FOE).pips, n.poise_pips - 2, "two Rest blocks: a pip");
    mirrored(&w);
    let st = strike(&mut w, &mut res, "p_jab", t * n.max_pip_loss_per_hit, 4);
    assert!(st.broke.is_some(), "the last pips go");
    assert_eq!((bar(&w, FOE).value, bar(&w, FOE).pips), (0, 0));

    // a bar with a max that does not divide: still pips x T at the refill
    let mut odd = PoiseState::default();
    poise::refill(&n, &mut odd, 23);
    assert_eq!((odd.pips, odd.value), (n.poise_pips, n.poise_pips * poise::threshold(&n, 23)));

    let s = session(timeline(json!({})));
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    strike(&mut w, &mut res, "p_jab", 7, 1);
    assert_eq!(bar(&w, FOE).value, TANK - 7, "bar: the value drops by D");
}

/// FB-71 (1), (2): the windup bonus pips are stored on the command (`poise_bonus_pips`) and come off as stored even when
/// the max changed meanwhile (a recomputed count would differ); they may sit above `poise_pips`.
#[test]
fn drop_bonus_takes_off_the_stored_pips() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let t = poise::threshold(&n, TANK);
    let bonus = poise::bonus_pips(20, t);
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, FOE, hit(&w, FOE, HERO, "p_smash"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(res.scheduled[a].command.poise_bonus_pips, bonus, "the count rides on the command");
    assert!(bar(&w, FOE).pips > n.poise_pips, "above poise_pips while it winds up");
    // the max shrinks mid-round: a recomputed count would be another number
    let max = TANK / 3;
    assert_ne!(poise::bonus_pips(20, poise::threshold(&n, max)), bonus, "the test needs a count that differs");
    w.poise.get_mut(&FOE).unwrap().max = max;
    run(&mut w, &mut res, 2, 3);
    assert_eq!(bar(&w, FOE).pips, n.poise_pips, "the stored count came off");
    assert_eq!(bar(&w, FOE).value, n.poise_pips * poise::threshold(&n, max), "mirrored on the max now");
    assert_eq!((res.scheduled[a].command.poise_bonus, res.scheduled[a].command.poise_bonus_pips), (0, 0));

    // a break cancels the windup: its bonus is gone with every pip
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, FOE, hit(&w, FOE, HERO, "p_smash"), 1);
    run(&mut w, &mut res, 1, 1);
    let st = strike(&mut w, &mut res, "p_jab", t * n.max_pip_loss_per_hit, 2);
    assert!(st.broke.is_none(), "{} pips before, {} lost", st.pips_before, st.pips_before - st.pips_after);
    poise::break_unit(&mut res, &mut w, FOE, 2, BreakCause::Poise);
    assert_eq!((bar(&w, FOE).pips, res.scheduled[a].command.poise_bonus_pips), (0, 0));
}

/// FB-71 (3): the planner's Rest preview shows the pip gain under pips (a pip per 2 Rest blocks, at most `poise_pips`)
/// and no poise; under bar the poise and no pips; nothing without the rule poise.
#[test]
fn the_rest_preview_shows_the_pips() {
    use rpgkit_battle::round_intent_plan as rip;
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    for blocks in 0..=BLOCKS * 3 {
        let g = rip::rest_of(&s.w, blocks);
        assert_eq!((g.blocks, g.poise, g.pips), (blocks, 0, (blocks / 2).min(n.poise_pips)), "{blocks} Rest blocks");
    }
    let s = session(timeline(json!({})));
    let b = s.w.ruleset.numbers.clone();
    assert_eq!((rip::rest_of(&s.w, 3).poise, rip::rest_of(&s.w, 3).pips), (b.rest_poise_per_block * 3, 0), "bar");
    let s = session(Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [], "poise_mode": "pips"}).as_object().unwrap()));
    assert_eq!((rip::rest_of(&s.w, 4).poise, rip::rest_of(&s.w, 4).pips), (0, 0), "no rule poise");
    // the pips the Rest restores in the battle are the preview's from a fresh round
    let s = session(pips_rules());
    let mut w = s.w.clone();
    w.poise.get_mut(&FOE).unwrap().pips = 0;
    poise::restore(&mut w, FOE, 4);
    assert_eq!(bar(&w, FOE).pips, rip::rest_of(&s.w, 4).pips);
}

// ---- S2.2-S4 (FB-49d, C10): a break under pips queues `stagger` ------------------------------------------------------

/// The blocks a status locks next round on `u` once its queued statuses take effect (the planner's locks).
fn locks_after_roll(w: &mut World, c: UnitId) -> Vec<i32> {
    let n = w.ruleset.numbers.clone();
    status::roll_over(&mut w.units[c], 0, &n);
    poise::round_start(w);
    let mut out: Vec<i32> = w.units[c].st.active.iter().flat_map(|s| status::blocked_blocks(s).iter().copied()).collect();
    out.sort();
    out.dedup();
    out
}

/// S2 section 5 test 3 (the resolver's hits): dropping to 0 pips cancels the wind-up, queues `stagger` (the status
/// row's rounds), and the next round refills the pips with the row's blocks locked.
#[test]
fn a_break_under_pips_queues_stagger_and_refills_next_round() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let t = poise::threshold(&n, TANK);
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let h = hit(&w, FOE, HERO, "p_heavy");
    running(&mut res, FOE, h, 1, 3);
    let big = t * n.max_pip_loss_per_hit;
    let mut b = 1;
    let broke = loop {
        let st = strike(&mut w, &mut res, "p_jab", big, b);
        if st.broke.is_some() {
            break st;
        }
        assert!(st.pips_after > 0, "a hit that leaves no pip breaks");
        b += 1;
    };
    assert_eq!(broke.pips_after, 0, "0 pips (by the pips or the chance roll of a hit into the wind-up)");
    assert!(res.scheduled[0].cancelled && res.scheduled[0].broken, "1. the wind-up is cancelled");
    assert!(w.units[FOE].st.pending.contains(poise::BREAK_STATUS), "2. stagger queued");
    let row = status::row_of(poise::BREAK_STATUS).expect("the stagger row");
    assert_eq!(w.units[FOE].st.pending_rounds.get(poise::BREAK_STATUS).copied(), Some(row.default_rounds), "the row's rounds");
    assert!(!w.units[FOE].has_status(poise::BREAK_STATUS), "not this round");
    let locks = locks_after_roll(&mut w, FOE);
    assert!(w.units[FOE].has_status(poise::BREAK_STATUS), "active next round");
    assert_eq!(locks, row.lock_blocks.to_vec(), "the row's block_lock blocks are locked next round");
    let st = bar(&w, FOE);
    assert_eq!((st.pips, st.value, st.broken, st.broken_last), (n.poise_pips, n.poise_pips * t, false, true), "3. the refill");
}

/// FB-49d: every break cause under pips queues it (a `sure_break` into a wind-up, a collision through the arena path);
/// under `bar` a break queues nothing (RULESET1's consequences only).
#[test]
fn every_pip_break_queues_stagger_and_a_bar_break_none() {
    let s = session(pips_rules());
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let h = hit(&w, FOE, HERO, "p_heavy");
    running(&mut res, FOE, h, 1, 3);
    let st = strike(&mut w, &mut res, "p_sure", 4, 2);
    assert_eq!(st.broke.as_ref().map(|b| b.cause), Some(BreakCause::Sure));
    assert!(w.units[FOE].st.pending.contains(poise::BREAK_STATUS), "sure break");

    let s = session(timeline(json!({})));
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let st = strike(&mut w, &mut res, "p_jab", TANK, 1);
    assert_eq!(st.broke.as_ref().map(|b| b.cause), Some(BreakCause::Poise), "bar: 0 poise breaks");
    assert!(w.units[FOE].st.pending.is_empty(), "bar: no stagger queued");
    let mut u = w.units[FOE].clone();
    assert!(!poise::break_queues_stagger(&w.ruleset.numbers, &mut u), "bar");
}

/// `steadfast` prevents the queued stagger: it holds the pips at 1 so nothing breaks, and a break forced past it (the
/// rule itself) queues nothing.
#[test]
fn steadfast_prevents_the_queued_stagger() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let mut w = s.w.clone();
    status::grant_now(&mut w.units[FOE], poise::STEADFAST, 1, 0, "test");
    let mut res = Resolver::new(BLOCKS, 1);
    let big = poise::threshold(&n, TANK) * n.max_pip_loss_per_hit;
    for b in 1..=3 {
        assert!(strike(&mut w, &mut res, "p_jab", big, b).broke.is_none(), "held at 1 pip");
    }
    assert!(!w.units[FOE].st.pending.contains(poise::BREAK_STATUS));
    poise::break_unit(&mut res, &mut w, FOE, 4, BreakCause::Poise);
    assert!(bar(&w, FOE).broken && !w.units[FOE].st.pending.contains(poise::BREAK_STATUS), "steadfast: no stagger");
}

/// The break's stagger and an AP-debt stagger at the same round start are one instance (the row's `refresh_max`): the
/// same rounds and the same locks as either alone.
#[test]
fn the_break_stagger_and_an_ap_debt_stagger_are_one_instance() {
    let s = session(pips_rules());
    let n = s.w.ruleset.numbers.clone();
    let row = status::row_of(poise::BREAK_STATUS).expect("the stagger row");
    let roll = |debt: bool, broke: bool| {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        if broke {
            poise::break_unit(&mut res, &mut w, FOE, 1, BreakCause::Poise);
        }
        if debt {
            w.units[FOE].ap = -n.debt_stagger;
        }
        let locks = locks_after_roll(&mut w, FOE);
        let u = &w.units[FOE];
        (u.has_status(poise::BREAK_STATUS), u.st.rounds_left.get(poise::BREAK_STATUS).copied().unwrap_or(0).max(i32::from(u.has_status(poise::BREAK_STATUS))), locks)
    };
    let one = (true, row.default_rounds, row.lock_blocks.to_vec());
    assert_eq!(status::debt_status(n.debt_stagger, &n), poise::BREAK_STATUS, "the debt that staggers");
    assert_eq!(roll(false, true), one, "the break alone");
    assert!(roll(true, false).0, "the debt alone staggers");
    assert_eq!(roll(true, true), one, "both: not doubled");
    assert_eq!(roll(false, false), (false, 0, Vec::new()));
}
