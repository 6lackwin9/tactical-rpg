//! HOOKS1-K on a scratch catalog built here (no shipped data): riders, HP cost and the guard bypass (HOOKS1 8.5) and
//! their metrics. Expectations come from the rules: a rider runs per unit the skill landed on, its effects default to
//! that unit, its conditions are 8.2's (`end_guard` ends a running Defend at once; `other_broken` holds only for a unit
//! broken this round) (RD1). An HP cost is paid at the command's start: HP becomes max(1, hp - cost), no hit event, an
//! `hp_below` crossing; the planner and the round-intent planner refuse the skill at or below the cost (HC1). An
//! `ignore_guard` hit leaves out the guard part of the guard-and-cover term (cover stays) and P6's guard factor (IG1).
//! In a session the trigger, rider and HP-cost cells balance the metrics (the holder is credited, the payer's loss is
//! `hp_paid`), and a redirected part is a hit on the unit that took it. With the rules off (and in classic) nothing
//! changes.

use indexmap::IndexMap;
use rpgkit_battle::command::{self, CmdKind, Command};
use rpgkit_battle::grid::{V2, GRID_W};
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::ruleset::{Ruleset, AP_COST, HP_COST, IGNORE_GUARD, POISE, ROUND_INTENT, TIMELINE, TRIGGERS};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const BAR: i32 = 40;
const COST: i32 = 10;
/// The cover of the rubble tile the foe stands on against ranged hits (the scratch terrain row).
const RUBBLE_COVER: f64 = 0.25;
const RUBBLE: usize = 13;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const BOW: UnitId = 2;
const FOE: UnitId = 3;

/// Engine skills and the test skills: a jab (x0.5), a smash (stagger 60: breaks a bar of 40), Martyr's Rush (HP cost),
/// a beheading (ignores guard, stagger 20) and a staggering jab to compare it with, a brand (rider `end_guard`), a
/// spite cut (rider: 7 damage to a target broken this round), and a mend whose rider gives its target a held guard.
fn catalog() -> Catalog {
    let attack = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY"});
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
        attack("t_jab", json!({})),
        attack("t_smash", json!({"stagger": 60})),
        attack("t_rush", json!({"hp_cost": COST, "ap_cost": 1})),
        attack("t_behead", json!({"ignore_guard": true, "stagger": 20})),
        attack("t_jab_st", json!({"stagger": 20})),
        attack("t_brand", json!({"riders": [{"when": {}, "effects": [{"type": "end_guard"}]}]})),
        attack("t_hex", json!({"power_mult": 0.0, "status_effect": "weaken", "status_potency": 1000, "riders": [{"when": {}, "effects": [{"type": "damage", "value": 7}]}]})),
        attack("t_spite_cut", json!({"riders": [{"when": {"other_broken": true}, "effects": [{"type": "damage", "value": 7}]}]})),
        json!({"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "heal_base": 20, "target_mode": "ALLY",
            "riders": [{"when": {}, "effects": [{"type": "guard", "mitigation": 0.5, "blocks": 0}]}]}),
    ];
    let unit = |id: &str, side: &str, speed: i32, range: i32, order: Value| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 100, "armor": 0, "speed": speed, "attack_range": range,
            "skill_cap": 5, "poise": BAR, "deck": {"order": order}})
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "terrain": [{"id": "rubble", "cover_guard": RUBBLE_COVER}],
        "units": [unit("p_hero", "party", 14, 1, json!({"assault": ["t_rush", "strike"]})), unit("p_ally", "party", 12, 1, json!({"assault": ["t_rush"]})),
            unit("p_bow", "party", 11, 3, json!({})), unit("p_foe", "enemy", 10, 1, json!({}))],
    }))
}

fn timeline(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

fn all() -> Ruleset {
    timeline(&[TRIGGERS, POISE, HP_COST, IGNORE_GUARD])
}

/// Hero (4,4) faces the foe (5,4); the ally beside the hero at (4,5); the bow 2 tiles west of the foe's row at (3,3).
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally", "p_bow"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (BOW, V2::new(3, 3)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

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

/// Gives `c` a passive trigger row (source `passive:<id>`).
fn give(w: &mut World, c: UnitId, id: &str, on: &str, when: Value, effects: Value, limit: &str) {
    let cat = w.cat.clone();
    let e = json!({"type": "trigger", "on": on, "when": when, "effects": effects, "limit": limit});
    w.units[c].apply_effects(&cat, &[e], &format!("passive:{id}"), false);
}

/// How often `holder` fired a trigger on `on` (`rider` for its riders; the records the session has not logged yet).
fn fired(res: &Resolver, holder: UnitId, on: &str) -> usize {
    res.triggers.fired.iter().filter(|f| f.holder == holder && f.on == on).count()
}

fn set_terrain(w: &mut World, cell: V2, t: usize) {
    w.grid.terrain[(cell.y * GRID_W + cell.x) as usize] = t;
}

/// The skill multiplier and crit of the action `a` (an attack), to recompute its damage.
fn mult_and_crit(res: &Resolver, a: usize) -> (f64, bool) {
    match &res.scheduled[a].command.kind {
        CmdKind::Attack { skill_mult, last_crit, .. } => (*skill_mult, *last_crit),
        _ => panic!("an attack"),
    }
}

fn last_damage(res: &Resolver, a: usize) -> i32 {
    match &res.scheduled[a].command.kind {
        CmdKind::Attack { last_damage, .. } => *last_damage,
        _ => panic!("an attack"),
    }
}

/// HC1: `hp_cost` 10 at HP 25 is paid as the command starts (HP 15 before the hit lands; no guard, no hit event) and
/// its `hp_below` crossing fires at once; the planner refuses it at HP 10 and 8 (`HP 10 <= cost 10`), and a second
/// Rush planned after the first sees the HP the first leaves. Rule off: no cost, no refusal.
#[test]
fn hp_cost_is_paid_at_the_start_and_refused_at_or_below() {
    let s = session(all());
    let mut w = s.w.clone();
    w.units[HERO].max_hp = 50;
    w.units[HERO].hp = 25;
    give(&mut w, HERO, "dread", "hp_below", json!({"pct": 40}), json!([{"type": "crit", "value": 1}]), "battle");
    give(&mut w, HERO, "sting", "hit_taken", json!({}), json!([{"type": "crit", "value": 1}]), "none");
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&w, HERO, FOE, "t_rush"), 1);
    assert_eq!(res.scheduled[a].command.hp_cost, COST);
    let order = res.prepare_block(&mut w, 1);
    assert_eq!(w.units[HERO].hp, 25 - COST, "paid at the start, before the hit");
    assert_eq!((fired(&res, HERO, "hp_below"), fired(&res, HERO, "hit_taken")), (1, 0), "a crossing, no hit event");
    assert!(res.log.iter().any(|l| l == &format!("[HP Cost] p_hero bleeds {COST} HP")), "{:?}", res.log);
    for i in order {
        res.execute_single(&mut w, i);
    }
    assert_eq!(w.units[HERO].hp, 25 - COST, "the cost only");

    for hp in [COST, COST - 2] {
        let mut w = s.w.clone();
        w.units[HERO].hp = hp;
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(&w, HERO, PlanStep::engage(sid(&w, "t_rush"), FOE), true), format!("HP {hp} <= cost {COST}"));
    }
    let mut w = s.w.clone();
    w.units[HERO].hp = 25;
    let mut plan = Plan::new();
    for _ in 0..2 {
        assert_eq!(plan.add_step(&w, HERO, PlanStep::engage(sid(&w, "t_rush"), FOE), true), "");
    }
    assert_eq!(plan.compile(&w, HERO).hp_left, 5, "25 - 10 - 10");
    assert_eq!(plan.add_step(&w, HERO, PlanStep::engage(sid(&w, "t_rush"), FOE), true), format!("HP 5 <= cost {COST}"));

    // HP 1 pays nothing more: HP never drops below 1 (a forced start at the floor)
    let mut w = s.w.clone();
    w.units[HERO].hp = 25;
    let mut cmd = hit(&w, HERO, FOE, "t_rush");
    w.units[HERO].hp = 1;
    cmd.hp_cost = COST;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, cmd, 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(w.units[HERO].hp, 1);
    assert!(res.log.iter().all(|l| !l.starts_with("[HP Cost]")));

    for rules in [timeline(&[TRIGGERS, POISE]), Ruleset::classic()] {
        let s = session(rules.clone());
        let mut w = s.w.clone();
        w.units[HERO].hp = COST - 2;
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(&w, HERO, PlanStep::engage(sid(&w, "t_rush"), FOE), true), "", "{}: no refusal", rules.id);
        let cmd = hit(&w, HERO, FOE, "t_rush");
        assert_eq!(cmd.hp_cost, 0, "{}", rules.id);
        let mut res = Resolver::new(BLOCKS, 1);
        book(&mut res, HERO, cmd, 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.units[HERO].hp, COST - 2, "{}: no cost", rules.id);
        assert!(res.log.iter().all(|l| !l.starts_with("[HP Cost]")), "{}", rules.id);
    }
}

/// HC1 (the AI): the round-intent planner places Rush while the HP left exceeds the cost (HP 25: two Rushes leave 5);
/// the hero then fills with Strike, the ally (Rush only) rests with the refusal `HP 5 <= cost 10` as the cause.
#[test]
fn the_round_intent_planner_refuses_an_unpayable_cost() {
    let mut s = session(timeline(&[ROUND_INTENT, AP_COST, HP_COST]));
    s.w.grid.positions[ALLY] = Some(V2::new(6, 4));
    let rush = sid(&s.w, "t_rush");
    for c in [HERO, ALLY] {
        s.w.units[c].max_hp = 50;
        s.w.units[c].hp = 25;
        round_intent::set_intent(&mut s.w, c, RoundIntent::Assault).expect("intent");
        round_intent::set_target(&mut s.w, c, RoundTarget::Unit(FOE)).expect("target");
        let (w, plan) = (&mut s.w, &mut s.plan);
        rip::plan_unit(w, plan, c).unwrap_or_else(|e| panic!("planned: {e}"));
        let entries = s.plan.entries_for(&s.w, c);
        let reasons = rip::reasons(&s.w, &s.plan, c);
        assert_eq!(entries.iter().filter(|e| e.skill == rush).count(), 2, "25 -> 15 -> 5: a third is refused ({reasons:?})");
        assert_eq!(s.plan.compile(&s.w, c).hp_left, 5);
        let refused = reasons.iter().any(|r| r.contains(&format!("HP 5 <= cost {COST}")));
        assert_eq!(refused, c == ALLY, "the ally's Rest names the refusal: {reasons:?}");
    }
}

/// IG1: a guarding foe on rubble (cover 0.25 against ranged hits) takes the bow's beheading with the guard part of the
/// guard-and-cover term left out (cover kept) and P6's guard factor 1.0; the log cell says so. A staggering jab of the
/// same power keeps the guard factor; rule off: the guard applies.
#[test]
fn ignore_guard_skips_the_guard_and_keeps_cover() {
    // the formula: the guard part is 0, the cover stays
    let s = session(all());
    let mut foe = s.w.units[FOE].clone();
    foe.guard = 0.3;
    let bow = &s.w.units[BOW];
    let input = HitInput { ranged: true, cover: RUBBLE_COVER, ..HitInput::default() };
    let with = damage::compute(bow, &foe, input);
    let without = damage::compute(bow, &foe, HitInput { include_guard: false, ..input });
    assert!((with.mit_mult - (1.0 - (0.3 + RUBBLE_COVER))).abs() < 1e-9);
    assert!((without.mit_mult - (1.0 - RUBBLE_COVER)).abs() < 1e-9, "cover kept");

    for (rules, skill, bypass) in [(all(), "t_behead", true), (all(), "t_jab_st", false), (timeline(&[TRIGGERS, POISE]), "t_behead", false)] {
        let s = session(rules.clone());
        let mut w = s.w.clone();
        let at = w.pos(FOE);
        set_terrain(&mut w, at, RUBBLE);
        assert!((command::cover_between(&w, BOW, FOE, true) - RUBBLE_COVER).abs() < 1e-9, "the foe is in cover");
        let mut res = Resolver::new(BLOCKS, 1);
        book(&mut res, FOE, own(&w, FOE, "guard"), 1);
        let a = book(&mut res, BOW, hit(&w, BOW, FOE, skill), 2);
        assert_eq!(res.scheduled[a].command.ignore_guard, bypass, "{} {skill}", rules.id);
        res.prepare_block(&mut w, 1);
        let order = res.prepare_block(&mut w, 2);
        // the units as the hit meets them (the bow in its attack stance, the foe guarding)
        let (bow, target) = (w.units[BOW].clone(), w.units[FOE].clone());
        assert!((target.guard - 0.3).abs() < 1e-9, "the foe guards");
        for i in order {
            res.execute_single(&mut w, i);
        }
        let (mult, crit) = mult_and_crit(&res, a);
        let hit_with = |include_guard: bool| damage::compute(&bow, &target, HitInput { ranged: true, skill_mult: mult, force_crit: crit, cover: RUBBLE_COVER, include_guard, ..HitInput::default() }).final_dmg;
        assert!(hit_with(false) > hit_with(true), "the guard matters here");
        assert_eq!(last_damage(&res, a), hit_with(!bypass), "{} {skill}", rules.id);
        let st = &res.scheduled[a].command.staggers[0];
        let factor = if bypass { 1.0 } else { w.ruleset.numbers.stagger_guard_mult };
        assert!((st.factors.guard - factor).abs() < 1e-9, "{} {skill}: guard factor {}", rules.id, st.factors.guard);
        let cell = rpgkit_battle::log::from_action(&w, &res.scheduled[a], 1, 2);
        assert_eq!(cell.detail.contains("Ignores guard"), bypass, "{}", cell.detail);
    }

    // the breakdown names the bypass only when it left out a guard: an unguarded foe's cell does not
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, BOW, hit(&w, BOW, FOE, "t_behead"), 1);
    run(&mut w, &mut res, 1, 1);
    assert!(last_damage(&res, a) > 0 && !res.scheduled[a].command.guard_bypassed);
    assert!(!rpgkit_battle::log::from_action(&w, &res.scheduled[a], 1, 1).detail.contains("Ignores guard"));
}

/// RD1: the brand's rider `end_guard` ends the foe's running Guard at once (guard 0 now; the Defend stays broken in
/// its next block); a jab without the rider leaves the guard. Rule off: no rider.
#[test]
fn a_rider_end_guard_ends_a_guard_at_once() {
    for (rules, skill, ends) in [(all(), "t_brand", true), (all(), "t_jab", false), (timeline(&[POISE]), "t_brand", false)] {
        let s = session(rules.clone());
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        let g = book(&mut res, FOE, own(&w, FOE, "guard"), 1);
        book(&mut res, HERO, hit(&w, HERO, FOE, skill), 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.units[FOE].guard == 0.0, ends, "{} {skill}: guard {}", rules.id, w.units[FOE].guard);
        assert_eq!(matches!(res.scheduled[g].command.kind, CmdKind::Defend { guard_broken: true, .. }), ends);
        assert_eq!(fired(&res, HERO, "rider"), ends as usize);
        assert_eq!(res.log.iter().any(|l| l == "[Rider] p_hero's t_brand: p_foe's guard ends"), ends, "{:?}", res.log);
        res.prepare_block(&mut w, 2);
        assert_eq!(w.units[FOE].guard == 0.0, ends, "{} {skill}: block 2", rules.id);
    }
}

/// RD1: the spite cut's rider (`when` `other_broken`) deals 7 only to a foe broken this round: not on a whole foe; after
/// the smash broke it, the cut's hit plus 7.
#[test]
fn a_rider_other_broken_needs_a_broken_target() {
    let s = session(all());
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_spite_cut"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(fired(&res, HERO, "rider"), 0, "a whole foe");

    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_smash"), 1);
    let a = book(&mut res, HERO, hit(&w, HERO, FOE, "t_spite_cut"), 2);
    run(&mut w, &mut res, 1, 1);
    assert!(w.poise[&FOE].broken, "the smash broke the foe");
    let before = w.units[FOE].hp;
    run(&mut w, &mut res, 2, 2);
    assert_eq!(fired(&res, HERO, "rider"), 1);
    assert_eq!(before - w.units[FOE].hp, last_damage(&res, a) + 7, "the hit plus the rider's 7");
    assert!(res.log.iter().any(|l| l == "[Rider] p_hero's t_spite_cut: 7 damage to p_foe"), "{:?}", res.log);
}

/// RD1 "damaged by a hit": a status-only attack (power 0) that lands its status lands no rider.
#[test]
fn a_status_only_attack_lands_no_rider() {
    let s = session(all());
    let mut w = s.w.clone();
    let hp = w.units[FOE].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_hex"), 1);
    run(&mut w, &mut res, 1, 1);
    assert!(w.units[FOE].has_status("weaken") || w.units[FOE].st.pending.iter().any(|st| st == "weaken"), "the status landed: {:?}", res.log);
    assert_eq!((fired(&res, HERO, "rider"), w.units[FOE].hp), (0, hp), "no rider: {:?}", res.log);
}

/// HC1, 8.1: the block-start pass raises only the HP cost's crossing; another drop since the last pass (here the ally's,
/// as a hazard met outside an action would) crosses at the next action's pass, as it does without a cost.
#[test]
fn an_hp_cost_does_not_move_other_crossings() {
    for (rules, cost) in [(all(), true), (timeline(&[TRIGGERS, POISE]), false)] {
        let s = session(rules.clone());
        let mut w = s.w.clone();
        for c in [HERO, ALLY] {
            w.units[c].max_hp = 50;
            w.units[c].hp = 25;
            give(&mut w, c, "dread", "hp_below", json!({"pct": 40}), json!([{"type": "crit", "value": 1}]), "battle");
        }
        let mut res = Resolver::new(BLOCKS, 1);
        book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 1);
        book(&mut res, HERO, hit(&w, HERO, FOE, "t_rush"), 2);
        run(&mut w, &mut res, 1, 1);
        w.damage(ALLY, 10, true);
        let order = res.prepare_block(&mut w, 2);
        assert_eq!((fired(&res, HERO, "hp_below"), fired(&res, ALLY, "hp_below")), (cost as usize, 0), "{}: at the block's start", rules.id);
        for i in order {
            res.execute_single(&mut w, i);
        }
        assert_eq!((fired(&res, HERO, "hp_below"), fired(&res, ALLY, "hp_below")), (cost as usize, 1), "{}: after the action", rules.id);
    }
}

/// RD1: a support skill's rider lands on its target (the mend's held guard on the ally).
#[test]
fn a_support_rider_lands_on_its_target() {
    let s = session(all());
    let mut w = s.w.clone();
    w.units[ALLY].hp -= 100;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, command(&w, HERO, PlanStep::ally(sid(&w, "t_mend"), ALLY), "t_mend"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(w.units[ALLY].held_guard.map(|g| g.mitigation), Some(0.5), "{:?}", res.log);
    assert_eq!(w.units[HERO].held_guard, None);
}

/// HOOKS1-K metrics: in a session the HP cost, a trigger's damage and a redirected part balance the metrics through the
/// fight's end (no METRIC FAIL): the cost is the hero's `hp_paid`, the foe's spite damage is its hit on the hero
/// (`damage` / `taken`), the trigger and rider counts sit under `triggers`; classic keeps the metrics shape.
#[test]
fn the_session_metrics_balance_trigger_hp() {
    let mut s = session(all());
    give(&mut s.w, FOE, "spite", "hit_taken", json!({}), json!([{"type": "damage", "value": 7, "to": "other"}]), "round");
    for c in [HERO, ALLY] {
        give(&mut s.w, c, "penance", "ally_hit", json!({"range": 1}), json!([{"type": "redirect", "pct": 35}]), "none");
    }
    s.attach_metrics(BattleMetrics::new(), true);
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_rush"), FOE), true), "");
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_spite_cut"), FOE), true), "");
    while s.current_round == 1 && !s.combat_over && s.phase != Phase::Replay {
        s.step_next();
    }
    let cost: Vec<_> = s.log.iter().filter(|e| e.subtype == "HP Cost").collect();
    assert_eq!(cost.len(), 1);
    assert_eq!((cost[0].category, cost[0].actor.as_str(), cost[0].damage), ("EVENT", "p_hero", COST));
    let spite: i32 = s.log.iter().filter(|e| e.subtype == "Trigger" && e.actor == "p_foe").map(|e| e.damage).sum();
    assert!(spite >= 7, "the foe's spite hit the hero");
    let mut m = s.metrics.take().unwrap();
    m.finish(&s);
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    let (hero, foe) = (&m.triggers["p_hero"], &m.triggers["p_foe"]);
    assert_eq!(hero.hp_paid, COST);
    assert_eq!((foe.fired >= 1, foe.damage, hero.taken), (true, spite, spite));
    assert_eq!(m.to_json()["triggers"]["p_hero"]["hp_paid"].as_i64(), Some(COST as i64));
    // 8.4: the ally took the redirected parts of the foe's hits on the hero: they are the foe's hits on the ally
    assert!(s.log.iter().any(|e| e.detail.contains("p_ally takes 35% of the hit meant for p_hero")), "a redirect happened");
    let ally = &m.units["p_ally"];
    assert!(ally.dmg_taken > 0 && ally.dmg_taken == ally.hp_start - s.w.units[ALLY].hp, "the redirected parts");
    let attacks: i32 = s.log.iter().filter(|e| e.actor == "p_foe" && e.category == "ATTACK").map(|e| e.damage).sum();
    assert_eq!(m.units["p_foe"].dmg_dealt, attacks + spite + ally.dmg_taken, "the spite and the redirected parts count as the foe's damage");

    let mut c = session(Ruleset::classic());
    give(&mut c.w, FOE, "spite", "hit_taken", json!({}), json!([{"type": "damage", "value": 7, "to": "other"}]), "round");
    c.attach_metrics(BattleMetrics::new(), true);
    assert_eq!(c.plan.add_step(&c.w, HERO, PlanStep::engage(sid(&c.w, "t_rush"), FOE), true), "");
    while c.current_round == 1 && !c.combat_over && c.phase != Phase::Replay {
        c.step_next();
    }
    assert!(c.log.iter().all(|e| e.subtype != "HP Cost" && e.subtype != "Trigger"));
    let mut m = c.metrics.take().unwrap();
    m.finish(&c);
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    assert!(m.triggers.is_empty() && m.to_json().get("triggers").is_none(), "classic keeps the metrics shape");
}
