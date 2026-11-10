//! PK1-S1 (docs/sessions/PK1_CORRUPTION.md section 3, hooks 1-7, as amended by FB-167) on a scratch catalog built here
//! (no shipped data). Expectations come from the rules: a band's `heal_taken` scales every heal after its formula
//! (`amount * (100 + pct) / 100`, rounded down, at least 0) while the band is held; a `damage` effect's `pct_of_hit` is
//! `trigger::part` of the event's hit, unmitigated, and never takes the unit below 1 HP; `strike_nearest` strikes the
//! nearest other unit of any side (ties: the lowest unit id) with one crit draw and grants its `then_status` next round;
//! an `ap` `set` holds the unit's AP at most there at the next round start (`rounds` of them) and stuns a unit that
//! pays no AP instead; `mark` records into the battle's marks (and `HeroEnd`), an `unmark` item removes the latest one
//! and calms a turned ally at the next round start, at its battle-start place; a hazard `meter` (arena effect, ground
//! row) changes only its holders' meter.

use rpgkit_battle::command::{self, CmdKind, Command};
use rpgkit_battle::encounter::HOSTILE_ALL;
use rpgkit_battle::grid::V2;
use rpgkit_battle::ground::{self, Placed};
use rpgkit_battle::log::PHASE_SIDE;
use rpgkit_battle::meter;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Phase, Roster, Session};
use rpgkit_battle::trigger::{self, SPASM_TAG};
use rpgkit_battle::world::{heal_taken, UnitId, World};
use rpgkit_battle::{arena, mark};
use rpgkit_data::{Catalog, HazardDef, ItemDef, PassiveDef, UnitDef};
use rpgkit_rules::ruleset::{Ruleset, ARENA_HAZARDS, ENEMY_MOVES, GROUND_HAZARDS, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const HP: i32 = 500;
/// The tension meter: its second band (from TENSE) holds `heal_taken` HEAL_TAKEN; the hero starts at START.
const TENSE: i32 = 25;
const START: i32 = 30;
const HEAL_TAKEN: i32 = -25;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY"}),
    ];
    let meters = vec![json!({"id": "tension", "name": "Tension", "short": "Te", "min": 0, "max": 100, "start": START, "show": "bar",
        "bands": [{"id": "calm", "from": 0}, {"id": "tense", "from": TENSE, "effects": [{"type": "heal_taken", "value": HEAL_TAKEN}]}]})];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "unit_meters": meters,
        "units": [unit("p_hero", "party", 14), unit("p_ally", "party", 12), unit("p_foe", "enemy", 10), unit("p_guest", "party", 9)],
    }))
}

/// The hero holds the tension meter (passive `trait0`).
struct Holders;

impl Roster for Holders {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if r == "p_hero" { vec!["trait0".to_string()] } else { Vec::new() };
        Overrides { passives, ..Overrides::default() }
    }
}

fn ruleset(extra: &[&str]) -> Ruleset {
    let rules: Vec<&str> = [TRIGGERS, UNIT_METERS].into_iter().chain(extra.iter().copied()).collect();
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// Hero (4,4), the ally below it (4,5), the foe at `foe`; `extra` rules and encounter keys.
fn session_with(rules: &[&str], foe: V2, row_extra: Value) -> Session {
    let mut row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    for (k, v) in row_extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset(rules);
    let mut cat = catalog();
    cat.passives.insert("trait0".into(), PassiveDef { id: "trait0".into(), name: "Trait".into(), effects: vec![json!({"type": "meter", "meter": "tension"})], slot: String::new() });
    let mut s = Session::start(Arc::new(cat), def, &Holders);
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, foe)] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn session() -> Session {
    session_with(&[], V2::new(5, 4), json!({}))
}

/// Gives `c` a passive trigger row (source `passive:<id>`, limit none).
fn give(w: &mut World, c: UnitId, id: &str, on: &str, effects: Value) {
    let cat = w.cat.clone();
    let e = json!({"type": "trigger", "on": on, "when": {}, "effects": effects, "limit": "none"});
    w.units[c].apply_effects(&cat, &[e], &format!("passive:{id}"), false);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn jab(w: &World, c: UnitId, t: UnitId) -> Command {
    let mut plan = Plan::new();
    let step: Step = PlanStep::engage(sid(w, "t_jab"), t);
    assert_eq!(plan.add_step(w, c, step, true), "");
    let id = sid(w, "t_jab");
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the jab").build_command(w, c)
}

fn run(w: &mut World, res: &mut Resolver) {
    for b in 1..=BLOCKS {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

fn item(extra: Value) -> ItemDef {
    let mut row = json!({"id": "t_salt", "name": "Salt", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "use_blocks": 1, "use_reach": 1});
    row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    ItemDef::from_obj(row.as_object().unwrap())
}

/// `c` uses `it` on `t` (one copy given first), alone in a fresh resolver; the item command after the round.
fn use_item(w: &mut World, c: UnitId, t: UnitId, it: &ItemDef) -> Command {
    w.units[c].inventory.pouch.push(it.clone());
    w.add_item_skills();
    let mut cmd = Command::new(c, 0, 1, CmdKind::Item { target: t, item: Box::new(it.clone()), used: false, last_heal: 0, last_damage: 0, last_boost: 0.0, cell: None, struck_units: Vec::new() });
    cmd.name = it.name.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(c, cmd, 1), "booked");
    run(w, &mut res);
    res.scheduled.pop().unwrap().command
}

/// A fresh resolver with the three units registered (as a battle's is).
fn registered() -> Resolver {
    let mut res = Resolver::new(BLOCKS, 1);
    for c in [HERO, ALLY, FOE] {
        res.register(c);
    }
    res
}

/// Resolves the round with an empty party plan, then starts the next one.
fn next_round(s: &mut Session) {
    assert!(s.confirm_plan(), "the plan is confirmed");
    while s.phase != Phase::Replay && !s.combat_over {
        s.step_next();
    }
    assert!(s.can_advance_round(), "the fight goes on");
    s.advance_round();
}

// ---- 1. heal_taken ---------------------------------------------------------------------------------------------------

/// The rule itself: a percent added, rounded down, at least 0; 0 changes nothing.
#[test]
fn heal_taken_rounds_down_and_never_goes_below_zero() {
    assert_eq!(heal_taken(40, -25), 30);
    assert_eq!(heal_taken(10, -25), 7, "7.5 rounds down");
    assert_eq!(heal_taken(10, 50), 15);
    assert_eq!(heal_taken(10, 0), 10);
    assert_eq!(heal_taken(10, -100), 0);
    assert_eq!(heal_taken(10, -150), 0, "at least 0");
    assert_eq!(heal_taken(0, -25), 0);
}

/// While the hero holds the band its heals (a direct heal and a potion, both after their formula) are scaled; once a
/// meter move leaves the band they are not.
#[test]
fn heal_taken_scales_every_heal_while_the_band_is_held() {
    let mut w = session().w;
    assert_eq!(meter::value(&w, HERO, "tension"), Some(START));
    assert_eq!(w.units[HERO].band_heal_taken, HEAL_TAKEN, "the band holds at the start");
    assert_eq!(w.units[ALLY].band_heal_taken, 0, "no meter, no band");
    w.units[HERO].hp = HP - 100;
    w.heal(HERO, 40);
    assert_eq!(w.units[HERO].hp, HP - 100 + heal_taken(40, HEAL_TAKEN));
    let potion = item(json!({"heal": 40}));
    let before = w.units[HERO].hp;
    let cmd = use_item(&mut w, ALLY, HERO, &potion);
    assert_eq!(w.units[HERO].hp - before, heal_taken(40, HEAL_TAKEN), "{}", cmd.log);
    // the ally, holding no band, takes the potion whole
    w.units[ALLY].hp = HP - 100;
    use_item(&mut w, HERO, ALLY, &potion);
    assert_eq!(w.units[ALLY].hp, HP - 100 + 40);
    // a meter move below TENSE leaves the band: the heal is whole again
    give(&mut w, HERO, "ease", "round_start", json!([{"type": "meter", "meter": "tension", "delta": -(START - TENSE + 1)}]));
    let mut res = Resolver::new(BLOCKS, 1);
    trigger::round_start(&mut res, &mut w);
    assert!(meter::value(&w, HERO, "tension").unwrap() < TENSE);
    assert_eq!(w.units[HERO].band_heal_taken, 0);
    let before = w.units[HERO].hp;
    w.heal(HERO, 40);
    assert_eq!(w.units[HERO].hp - before, 40);
}

// ---- 2. damage pct_of_hit (recoil) -----------------------------------------------------------------------------------

/// The hero's `hit_dealt` recoil takes `part(hit, pct)` of the hit it dealt (unmitigated), logged as its trigger; at 1
/// HP it takes nothing and logs nothing; on an event without a hit it does nothing.
#[test]
fn recoil_is_a_share_of_the_hit_and_never_downs_the_unit() {
    let pct = 10;
    let recoil = json!([{"type": "damage", "pct_of_hit": pct}]);
    let mut w = session().w;
    give(&mut w, HERO, "recoil", "hit_dealt", recoil.clone());
    let hp = w.units[HERO].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    let cmd = jab(&w, HERO, FOE);
    assert!(res.schedule(HERO, cmd, 1));
    run(&mut w, &mut res);
    let hit = res.scheduled[0].command.struck.iter().find(|s| s.target == FOE).map(|s| s.damage).expect("the jab hit");
    assert!(hit > 0);
    let want = trigger::part(hit, pct);
    assert_eq!(hp - w.units[HERO].hp, want);
    assert!(res.log.iter().any(|l| l == &format!("[Trigger] p_hero's recoil: {want} damage to p_hero")), "{:?}", res.log);

    w.units[HERO].hp = 1;
    let mut res = Resolver::new(BLOCKS, 1);
    let cmd = jab(&w, HERO, FOE);
    assert!(res.schedule(HERO, cmd, 1));
    run(&mut w, &mut res);
    assert_eq!(w.units[HERO].hp, 1, "recoil never downs the unit");
    assert!(!res.log.iter().any(|l| l.starts_with("[Trigger] p_hero's recoil")), "no line when nothing is taken: {:?}", res.log);

    let mut w = session().w;
    give(&mut w, HERO, "recoil", "round_end", recoil);
    let hp = w.units[HERO].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    trigger::round_end(&mut res, &mut w);
    assert_eq!(w.units[HERO].hp, hp, "no hit, no recoil");
}

// ---- 3. strike_nearest -----------------------------------------------------------------------------------------------

/// The nearest other unit on the field, of any side; a tie goes to the lowest unit id; nobody for a unit off the field.
#[test]
fn the_nearest_other_unit_is_of_any_side_and_ties_go_to_the_lowest_id() {
    let mut w = session_with(&[], V2::new(6, 4), json!({})).w;
    assert_eq!(trigger::nearest_other(&w, HERO), Some(ALLY), "the ally at 1 before the foe at 2");
    w.grid.positions[FOE] = Some(V2::new(5, 4));
    assert_eq!(trigger::nearest_other(&w, HERO), Some(ALLY), "a tie at 1: the lower id");
    w.grid.positions[ALLY] = Some(V2::new(4, 7));
    assert_eq!(trigger::nearest_other(&w, HERO), Some(FOE));
    w.grid.positions[HERO] = None;
    assert_eq!(trigger::nearest_other(&w, HERO), None);
}

/// A round-end `strike_nearest` hits the nearest unit (here the ally) with one crit draw and no AP, then the striker
/// takes its `then_status` next round; the same seed deals the same; the line is tagged `[Spasm]`.
#[test]
fn strike_nearest_strikes_at_once_and_grants_its_status_next_round() {
    let strike = |seed: i64| {
        let mut w = session_with(&[], V2::new(6, 4), json!({})).w;
        give(&mut w, HERO, "spasm", "round_end", json!([{"type": "strike_nearest", "power": 1.0, "then_status": "stun", "then_rounds": 1}]));
        let (ap, ally, foe) = (w.units[HERO].ap, w.units[ALLY].hp, w.units[FOE].hp);
        let mut res = Resolver::new(BLOCKS, seed);
        let draws = res.rng.draws;
        trigger::round_end(&mut res, &mut w);
        assert_eq!(w.units[FOE].hp, foe, "the farther foe is spared");
        assert_eq!(w.units[HERO].ap, ap, "no AP");
        assert!(w.units[HERO].st.pending.contains("stun"), "stunned next round");
        (ally - w.units[ALLY].hp, res.rng.draws - draws, res.log.clone())
    };
    let (dealt, draws, log) = strike(7);
    assert!(dealt > 0);
    let mut one = aegis_core::rng::GodotRng::from_i64(7);
    one.randf();
    assert_eq!(draws, one.draws, "one crit roll (a randf)");
    assert!(log.iter().any(|l| l.starts_with(&format!("{SPASM_TAG} p_hero's spasm: p_hero strikes p_ally for {dealt}"))), "{log:?}");
    assert_eq!(strike(7).0, dealt, "the same seed, the same strike");
}

// ---- 4. ap set -------------------------------------------------------------------------------------------------------

/// An `ap` `set` 0 fired at round end holds the hero's AP at 0 at the next round start (logged, `ap_held`), for
/// `rounds` round starts; a unit that pays no AP (an enemy under `enemy_moves`) is stunned next round instead.
#[test]
fn ap_set_holds_the_next_rounds_ap_or_stuns_a_unit_that_pays_none() {
    let mut w = session_with(&[ENEMY_MOVES], V2::new(5, 4), json!({})).w;
    let set = json!([{"type": "ap", "set": 0, "rounds": 2}]);
    give(&mut w, HERO, "collapse", "round_end", set.clone());
    give(&mut w, FOE, "collapse", "round_end", set);
    assert!(w.pays_ap(HERO) && !w.pays_ap(FOE));
    let mut res = Resolver::new(BLOCKS, 1);
    trigger::round_end(&mut res, &mut w);
    assert_eq!(res.triggers.ap_sets(), vec![(HERO, 0, 2)], "the foe holds no set");
    assert!(w.units[FOE].st.pending.contains(&w.ruleset.numbers.debt_stun_status), "the foe is stunned next round");
    assert!(w.units[HERO].ap > 0);
    trigger::apply_ap_sets(&mut res, &mut w);
    assert_eq!(w.units[HERO].ap, 0);
    assert_eq!(res.triggers.ap_held(HERO), Some((0, "collapse")));
    assert!(res.log.iter().any(|l| l.starts_with("[Trigger] p_hero's collapse: p_hero's AP is held at 0 this round")), "{:?}", res.log);
    w.units[HERO].ap = 4;
    trigger::apply_ap_sets(&mut res, &mut w);
    assert_eq!(w.units[HERO].ap, 0, "the second of its rounds");
    w.units[HERO].ap = 4;
    trigger::apply_ap_sets(&mut res, &mut w);
    assert_eq!((w.units[HERO].ap, res.triggers.ap_held(HERO)), (4, None), "spent");
}

/// In a battle the set takes hold at the next round start, before planning.
#[test]
fn a_session_holds_the_ap_at_the_next_round_start() {
    let mut s = session();
    give(&mut s.w, HERO, "collapse", "round_end", json!([{"type": "ap", "set": 0}]));
    next_round(&mut s);
    assert!(s.w.units[HERO].ap <= 0, "AP {}", s.w.units[HERO].ap);
    assert!(s.resolver.triggers.ap_held(HERO).is_some());
    assert!(s.w.units[ALLY].ap > 0, "only the holder");
}

// ---- 5 and 7. mark and unmark ----------------------------------------------------------------------------------------

/// `mark` records in order into the battle's marks; an `unmark` item removes the target's latest entry of its id
/// (another id stays), and with none left says so.
#[test]
fn mark_records_and_unmark_takes_the_latest_back() {
    let mut w = session().w;
    give(&mut w, HERO, "breach", "round_end", json!([{"type": "mark", "mark": "calcify", "value": 1}, {"type": "mark", "mark": "scar", "value": 2}]));
    let mut res = Resolver::new(BLOCKS, 1);
    trigger::round_end(&mut res, &mut w);
    trigger::round_end(&mut res, &mut w);
    let pairs = |v: &[(&str, i32)]| v.iter().map(|(m, n)| (m.to_string(), *n)).collect::<Vec<_>>();
    assert_eq!(w.marks.of(HERO), pairs(&[("calcify", 1), ("scar", 2), ("calcify", 1), ("scar", 2)]));
    assert!(res.log.iter().any(|l| l == "[Trigger] p_hero's breach: p_hero is marked calcify +1"), "{:?}", res.log);
    let salt = item(json!({"unmark": {"mark": "calcify"}}));
    assert_eq!(salt.unmark, "calcify");
    assert_eq!(command::item_side_problem(&w, ALLY, &salt, Some(HERO)), "");
    let cmd = use_item(&mut w, ALLY, HERO, &salt);
    assert!(cmd.log.contains("p_hero's calcify undone"), "{}", cmd.log);
    assert_eq!(w.marks.of(HERO), pairs(&[("calcify", 1), ("scar", 2), ("scar", 2)]), "the latest calcify went");
    use_item(&mut w, ALLY, HERO, &salt);
    let cmd = use_item(&mut w, ALLY, HERO, &salt);
    assert!(cmd.log.contains("no calcify to undo"), "{}", cmd.log);
    assert_eq!(w.marks.of(HERO), pairs(&[("scar", 2), ("scar", 2)]));
}

/// A hero turned hostile to all (a breach's `side`) with a mark: a potion cannot reach it, the unmark item can; it
/// removes the mark and the hero rejoins the party side at the next round start, first among the allies again, with a
/// `[Side]` cell. Without a mark to undo, no calm.
#[test]
fn unmark_calms_a_turned_ally_at_the_next_round_start() {
    let mut s = session();
    let salt = item(json!({"unmark": {"mark": "calcify"}}));
    assert!(s.confirm_plan());
    while s.phase != Phase::Replay && !s.combat_over {
        s.step_next();
    }
    s.w.marks.record(HERO, "calcify", 1);
    assert!(s.w.set_side(HERO, HOSTILE_ALL));
    assert!(mark::turned_ally(&s.w, HERO));
    assert_ne!(command::item_side_problem(&s.w, ALLY, &item(json!({"heal": 10})), Some(HERO)), "", "a potion is for friends");
    assert_eq!(command::item_side_problem(&s.w, ALLY, &salt, Some(HERO)), "", "the unmark reaches a turned ally");
    let cmd = use_item(&mut s.w, ALLY, HERO, &salt);
    assert!(cmd.log.contains("calms at the next round start"), "{}", cmd.log);
    assert!(s.w.marks.calming(HERO) && s.w.sides.is_rogue(HERO), "still hostile this round");
    assert!(s.can_advance_round());
    s.advance_round();
    assert!(!s.w.sides.is_rogue(HERO));
    assert_eq!(s.w.allies, vec![HERO, ALLY], "back at its start place");
    assert!(s.w.marks.of(HERO).is_empty());
    assert!(s.log.iter().any(|e| e.category == "EVENT" && e.subtype == PHASE_SIDE && e.detail == "p_hero joins the party side."), "the [Side] cell");

    // no mark to undo: the turned ally stays hostile
    let mut s = session();
    assert!(s.w.set_side(HERO, HOSTILE_ALL));
    let cmd = use_item(&mut s.w, ALLY, HERO, &salt);
    assert!(cmd.log.contains("no calcify to undo"), "{}", cmd.log);
    assert!(!s.w.marks.calming(HERO));
}

/// A protected guest a phase `side` turned hostile to all leaves the guests (PHASEFX1); calmed by an unmark it rejoins
/// the party side as a protected guest again (not a hero: it plans as a guest), at its start place, not unkillable.
#[test]
fn a_calmed_guest_is_a_protected_guest_again() {
    let phases = json!([{"id": "turn", "name": "Turn", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "side", "units": ["p_guest"], "to": HOSTILE_ALL}]}]);
    let mut s = session_with(&[], V2::new(8, 4), json!({"guests": ["p_guest"], "protect": ["p_guest"], "phases": phases}));
    let g = *s.guests.first().expect("the guest");
    assert!(s.protected.contains(&g));
    s.w.grid.positions[g] = Some(V2::new(3, 4));
    let start = s.w.allies.clone();
    next_round(&mut s);
    assert!(s.w.sides.is_rogue(g), "the phase turned it");
    assert!(!s.guests.contains(&g) && !s.protected.contains(&g), "PHASEFX1: a unit leaving the party side stops being a guest");
    s.w.marks.record(g, "calcify", 1);
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(8, 4)), (g, V2::new(3, 5))] {
        s.w.grid.positions[c] = Some(cell);
    }
    let cmd = use_item(&mut s.w, ALLY, g, &item(json!({"unmark": {"mark": "calcify"}})));
    assert!(cmd.log.contains("calms at the next round start"), "{}", cmd.log);
    next_round(&mut s);
    assert!(!s.w.sides.is_rogue(g));
    assert_eq!(s.w.allies, start, "back at its start place");
    assert!(s.guests.contains(&g) && s.protected.contains(&g), "a protected guest again");
    assert!(!s.w.unkillable.contains(&g));
}

// ---- 6. hazard meter -------------------------------------------------------------------------------------------------

/// An arena hazard's `meter` raises the meter of each struck holder (its own `[Meter]` line), never a unit without it.
#[test]
fn an_arena_hazard_meter_changes_only_its_holders() {
    let rows = json!({"arena_hazards": [{"id": "vent", "name": "Vent", "cells": [[4, 4], [4, 5]], "blocks": [1], "effect": {"meter": {"meter": "tension", "delta": 5}}}]});
    let mut w = session_with(&[ARENA_HAZARDS], V2::new(5, 4), rows).w;
    assert_eq!(w.arena.hazards[0].effect.meter.as_ref().map(|m| m.delta), Some(5));
    let mut res = registered();
    let strikes = arena::block_end(&mut res, &mut w, 1, 1, &[]);
    assert_eq!(strikes[0].hits.len(), 2, "both units on its cells");
    assert_eq!(meter::value(&w, HERO, "tension"), Some(START + 5));
    assert_eq!(meter::value(&w, ALLY, "tension"), None);
    assert!(res.log.iter().any(|l| l == &format!("[Meter] p_hero's Vent: p_hero's Tension +5 ({START} -> {})", START + 5)), "{:?}", res.log);
}

/// A ground hazard row's `meter` acts on entering (after `settle`) and at round end, on holders only.
#[test]
fn a_ground_hazard_meter_changes_only_its_holders() {
    let mut w = session_with(&[GROUND_HAZARDS], V2::new(5, 4), json!({})).w;
    let def = HazardDef::from_obj(json!({"id": "miasma", "name": "Miasma", "on": "both", "meter": {"meter": "tension", "delta": -4}}).as_object().unwrap());
    assert_eq!(def.meter.as_ref().map(|m| (m.meter.as_str(), m.delta)), Some(("tension", -4)));
    assert_eq!(HazardDef::from_obj(json!({"id": "x", "meter": {}}).as_object().unwrap()).meter, None);
    for c in [V2::new(4, 4), V2::new(4, 5)] {
        w.grid.ground.insert(c, Placed { def: def.clone(), by: FOE, round: 1, last_round: 3 });
    }
    let mut res = registered();
    let mut rng = aegis_core::rng::GodotRng::from_i64(1);
    ground::enter(&mut w, &mut rng, HERO, V2::new(4, 4), &mut Vec::new());
    ground::enter(&mut w, &mut rng, ALLY, V2::new(4, 5), &mut Vec::new());
    assert_eq!(meter::value(&w, HERO, "tension"), Some(START), "waits for the settle");
    ground::settle(&mut res, &mut w);
    assert_eq!(meter::value(&w, HERO, "tension"), Some(START - 4));
    ground::round_end(&mut res, &mut w);
    assert_eq!(meter::value(&w, HERO, "tension"), Some(START - 8));
    assert_eq!(meter::value(&w, ALLY, "tension"), None);
    assert!(res.log.iter().any(|l| l.starts_with("[Meter] p_hero's Miasma: p_hero's Tension -4")), "{:?}", res.log);
}

// ---- 0 diffs ---------------------------------------------------------------------------------------------------------

/// A battle that uses none of the hooks records no mark, no AP set and no band heal change.
#[test]
fn a_battle_without_the_hooks_keeps_them_empty() {
    let mut s = session_with(&[], V2::new(5, 4), json!({}));
    next_round(&mut s);
    assert!(s.w.marks.list.is_empty());
    assert!(s.resolver.triggers.ap_sets().is_empty());
    assert_eq!(s.resolver.triggers.ap_held(HERO), None);
    assert_eq!(s.w.allies, vec![HERO, ALLY]);
}
