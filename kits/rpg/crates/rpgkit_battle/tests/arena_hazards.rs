//! HOOKS1-H on a scratch catalog built here (no shipped data): scheduled arena hazards under the rule `arena_hazards`
//! (HOOKS1 7). Expectations come from the rules: a hazard strikes at the end of each listed block of each listed round,
//! every active, placed, unreserved unit on its cells, either side, in registration order: damage (flat or a share of
//! max HP) through guard, not armour, the status roll, stagger through P6 (x the windup factor; it can break a windup),
//! then a push that collides with a wall or a unit as FM4 does (AH2); the strike comes after the block's actions and
//! before the verdict; the round's struck cells and blocks are known at its start and PL4's threat counts a cell due
//! this round (AH3, AT3). The session logs a cell per strike and per struck unit and the metrics balance; with the rule
//! off (and in classic) nothing strikes, logs or draws (4.1, 4.2).

use indexmap::IndexMap;
use rpgkit_battle::arena::{self, Strike};
use rpgkit_battle::command::Command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::intent;
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, Aim};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, ARENA_HAZARDS, FORCED_MOVE, POISE, TIMELINE};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HP: i32 = 5000;
const BAR: i32 = 50;
/// The terrain index of `wall` in `grid::TERRAIN_IDS`.
const WALL: usize = 10;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;

/// Engine skills and a 3-block heavy (a windup); four units with fixed HP and poise, the foe armoured.
fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "a_heavy", "name": "a_heavy", "kind": "ATTACK", "level": 1, "blocks": 3, "power_mult": 0.2, "target_mode": "ENEMY"}),
    ];
    let unit = |id: &str, side: &str, speed: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "speed": speed, "attack_range": 1, "skill_cap": 5, "poise": BAR});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party", 14, json!({})), unit("p_ally", "party", 12, json!({})), unit("p_foe", "enemy", 10, json!({"armor": 25})), unit("p_foe_b", "enemy", 8, json!({}))]
    }))
}

/// The slice's vents (HOOKS1_TASKS 2: columns 7-8 at blocks 3 and 6, 12% of max HP, stagger 10, burn).
fn vents() -> Value {
    json!({"id": "vents", "name": "Slag Vents", "columns": [7, 8], "blocks": [3, 6], "line": "The slag vents roar.",
        "effect": {"damage_pct": 12, "stagger": 10, "status": "burn", "potency": 100, "magnitude": 3, "status_rounds": 1}})
}

/// A belt tile: pushes east 2 at the end of block 1, 7 damage on a collision.
fn belt() -> Value {
    json!({"id": "belt", "name": "Belt", "cells": [[8, 2]], "blocks": [1], "effect": {"push": {"dir": "east", "tiles": 2, "collide_damage": 7}}})
}

/// Row 4 strikes in round 2 only (5 flat damage at block 1's end).
fn late() -> Value {
    json!({"id": "late", "name": "Late Vent", "rows": [4], "rounds": [2], "blocks": [1], "effect": {"damage": 5}})
}

fn timeline(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

fn rules() -> Ruleset {
    timeline(&[ARENA_HAZARDS, POISE, FORCED_MOVE])
}

/// Hero (6, 4) next to the foe on the vents (7, 4); foe_b on the vents at (8, 0); the ally in a far corner.
fn session(ruleset: Ruleset, hazards: Value) -> Session {
    let row = json!({"id": "a_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe_b"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337, "arena_hazards": hazards});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, v(6, 4)), (ALLY, v(0, 8)), (FOE, v(7, 4)), (FOE_B, v(8, 0))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

const fn v(x: i32, y: i32) -> V2 {
    V2::new(x, y)
}

fn heavy(w: &World, c: UnitId, t: UnitId) -> Command {
    let mut plan = Plan::new();
    let id = w.book().find("a_heavy").expect("a_heavy");
    let step: Step = PlanStep::engage(id, t);
    assert_eq!(plan.add_step(w, c, step, true), "", "a_heavy");
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the heavy's entry").build_command(w, c)
}

/// A battle world and a resolver with every unit registered (allies, then enemies).
fn world(s: &Session) -> (World, Resolver) {
    let mut res = Resolver::new(BLOCKS, 1);
    for c in 0..s.w.units.len() {
        res.register(c);
    }
    (s.w.clone(), res)
}

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

fn set_terrain(w: &mut World, cell: V2, t: usize) {
    let width = rpgkit_battle::grid::GRID_W;
    w.grid.terrain[(cell.y * width + cell.x) as usize] = t;
}

/// AH2: 12% of max HP, rounded.
fn pct_damage(w: &World, c: UnitId) -> i32 {
    (w.units[c].max_hp as f64 * 0.12).round() as i32
}

fn struck(strikes: &[Strike]) -> Vec<UnitId> {
    strikes.iter().flat_map(|s| s.hits.iter().map(|h| h.unit)).collect()
}

/// AH1, AH2: nothing at a block that is not listed; at block 3's end both foes on the columns are struck, in
/// registration order, the hero beside them is not; 12% of max HP through no armour (the armoured foe loses the same),
/// burn lands (potency 100, no resistance) as the hazard's; one random draw per status roll.
#[test]
fn a_strike_hits_the_units_on_its_cells_through_no_armour() {
    let s = session(rules(), json!([vents()]));
    let (mut w, mut res) = world(&s);
    assert!(arena::block_end(&mut res, &mut w, 1, 2, &[]).is_empty(), "block 2 is not listed");
    let hp: Vec<i32> = w.units.iter().map(|u| u.hp).collect();
    // The status rolls: one randi_range(1, 100) per struck unit in order, landing at or under the chance
    let mut rolls = Resolver::new(BLOCKS, 1).rng;
    let lands: Vec<bool> = [FOE, FOE_B].iter().map(|&c| rolls.randi_range(1, 100) <= status::apply_chance(&w.units[c], "burn", 100)).collect();
    let strikes = arena::block_end(&mut res, &mut w, 1, 3, &[]);
    assert_eq!(strikes.len(), 1);
    let st = &strikes[0];
    assert_eq!((st.id.as_str(), st.name.as_str(), st.line.as_str(), st.cells.len()), ("vents", "Slag Vents", "The slag vents roar.", 18));
    assert_eq!(struck(&strikes), vec![FOE, FOE_B], "registration order; the hero on (6, 4) is not struck");
    for (h, lands) in st.hits.iter().zip(lands) {
        let want = pct_damage(&w, h.unit);
        assert_eq!((h.damage, hp[h.unit] - w.units[h.unit].hp), (want, want), "{}: 12% of max HP, no armour", w.units[h.unit].name);
        assert_eq!(h.status == "burn", lands, "the roll decides");
        assert_eq!(w.units[h.unit].st.pending.iter().any(|k| k == "burn"), lands, "burn queued when it landed");
    }
    assert_eq!(w.units[HERO].hp, hp[HERO]);
    assert!(res.log.iter().any(|l| l == "[Arena] The slag vents roar."), "{:?}", res.log);
    assert_eq!(res.rng.randi(), rolls.randi(), "two status rolls, nothing else drawn");
}

/// AH2 (HQ10): the damage goes through guard (a unit at guard 0.5 takes half), flat `damage` as given.
#[test]
fn guard_reduces_a_strike() {
    let s = session(rules(), json!([vents()]));
    let (mut w, mut res) = world(&s);
    w.units[FOE_B].guard = 0.5;
    let before = w.units[FOE_B].hp;
    arena::block_end(&mut res, &mut w, 1, 3, &[]);
    let want = (pct_damage(&w, FOE_B) as f64 * 0.5).round() as i32;
    assert_eq!(before - w.units[FOE_B].hp, want, "half through guard 0.5");
}

/// AH2, P6: the stagger runs through P6, so a unit in windup takes it x the ruleset's windup factor; a strike that
/// takes a windup's poise to 0 breaks it and its action is cancelled.
#[test]
fn stagger_doubles_into_a_windup_and_can_break_it() {
    let s = session(rules(), json!([vents()]));
    let mult = s.w.ruleset.numbers.stagger_windup_mult;
    for low in [false, true] {
        let (mut w, mut res) = world(&s);
        assert!(res.schedule(FOE, heavy(&w, FOE, HERO), 2), "booked at 2");
        let idx = res.scheduled.len() - 1;
        run(&mut w, &mut res, 1, 3);
        assert!(rpgkit_battle::poise::in_windup(&res.scheduled, FOE, 3), "the heavy winds up at block 3");
        if low {
            w.poise.get_mut(&FOE).unwrap().value = (10.0 * mult) as i32;
        }
        let strikes = arena::block_end(&mut res, &mut w, 1, 3, &[]);
        let hits = &strikes[0].hits;
        let (foe, foe_b) = (hits[0].stagger.as_ref().unwrap(), hits[1].stagger.as_ref().unwrap());
        assert_eq!((foe.windup, foe.taken), (true, (10.0 * mult).round() as i32), "x the windup factor");
        assert_eq!((foe_b.windup, foe_b.taken, foe_b.broke.is_some()), (false, 10, false));
        assert_eq!(foe.broke.is_some(), low, "low {low}: poise to 0 breaks the windup");
        assert_eq!(res.scheduled[idx].cancelled, low, "low {low}: the broken windup is cancelled");
        if !low {
            assert_eq!(rpgkit_battle::poise::state(&w, FOE).unwrap().value, BAR - foe.taken);
        }
    }
}

/// AH2, FM4, FM9 (HQ11): a push east 2 into a wall moves the unit one tile and the collision costs it the flat damage;
/// into a unit both take it; under the rule forced_move off the push is ignored.
#[test]
fn a_push_collides_with_a_wall_or_a_unit() {
    let s = session(rules(), json!([belt()]));
    let (mut w, mut res) = world(&s);
    w.grid.positions[FOE_B] = Some(v(8, 2));
    set_terrain(&mut w, v(10, 2), WALL);
    let before = w.units[FOE_B].hp;
    let strikes = arena::block_end(&mut res, &mut w, 1, 1, &[]);
    let m = strikes[0].hits[0].push.as_ref().expect("a push");
    assert_eq!((m.from, m.to, m.tiles, m.by), (v(8, 2), v(9, 2), 1, None));
    assert_eq!(m.blocked_by, "the wall");
    assert_eq!((m.hits.len(), m.hits[0].damage, before - w.units[FOE_B].hp), (1, 7, 7), "flat collision damage");
    assert_eq!(strikes[0].hits[0].damage, 0, "the belt itself deals nothing");

    let (mut w, mut res) = world(&s);
    w.grid.positions[FOE_B] = Some(v(8, 2));
    w.grid.positions[HERO] = Some(v(10, 2));
    let hp: Vec<i32> = w.units.iter().map(|u| u.hp).collect();
    let strikes = arena::block_end(&mut res, &mut w, 1, 1, &[]);
    let m = strikes[0].hits[0].push.as_ref().unwrap();
    assert_eq!((m.to, m.blocker), (v(9, 2), Some(HERO)));
    assert_eq!((hp[FOE_B] - w.units[FOE_B].hp, hp[HERO] - w.units[HERO].hp), (7, 7), "both take the hit");

    let off = session(timeline(&[ARENA_HAZARDS, POISE]), json!([belt()]));
    let (mut w, mut res) = world(&off);
    w.grid.positions[FOE_B] = Some(v(8, 2));
    let strikes = arena::block_end(&mut res, &mut w, 1, 1, &[]);
    assert_eq!((strikes[0].hits.len(), strikes[0].hits[0].push.is_none(), w.pos(FOE_B)), (1, true, v(8, 2)), "no forced_move: no push");
}

/// AH2: a reserved unit, a unit off the board and a downed unit on a struck cell are untouched.
#[test]
fn reserved_downed_and_unplaced_units_are_untouched() {
    let s = session(rules(), json!([vents()]));
    let (mut w, mut res) = world(&s);
    w.grid.positions[HERO] = Some(v(7, 6));
    w.units[HERO].hp = 0;
    w.grid.positions[FOE_B] = None;
    let hp: Vec<i32> = w.units.iter().map(|u| u.hp).collect();
    let strikes = arena::block_end(&mut res, &mut w, 1, 3, &[FOE]);
    assert_eq!(strikes.len(), 1, "the vents still roar");
    assert!(strikes[0].hits.is_empty(), "{:?}", struck(&strikes));
    assert_eq!(w.units.iter().map(|u| u.hp).collect::<Vec<_>>(), hp);
}

/// AH3: the round's struck cells and blocks are known at its start (round-limited rows only in their rounds), in the
/// host's JSON shape; none without the rule.
#[test]
fn tells_list_the_rounds_cells_and_blocks() {
    let mut s = session(rules(), json!([vents(), late()]));
    let tells = s.arena_tells();
    assert_eq!(tells.len(), 1, "the late vent strikes in round 2 only");
    assert_eq!((tells[0].id.as_str(), tells[0].blocks.clone(), tells[0].cells.len()), ("vents", vec![3, 6], 18));
    assert_eq!((tells[0].cells[0], tells[0].cells[9]), (v(7, 0), v(8, 0)), "column 7 top to bottom, then column 8");
    let j = intent::arena_json(&tells);
    assert_eq!(j[0]["cells"][0], json!([7, 0]));
    assert_eq!(j[0]["blocks"], json!([3, 6]));
    while s.current_round == 1 && s.phase != Phase::Replay {
        s.step_next();
    }
    assert!(s.can_advance_round(), "round 1 ends");
    s.advance_round();
    let ids: Vec<String> = s.arena_tells().into_iter().map(|t| t.id).collect();
    assert_eq!(ids, vec!["vents".to_string(), "late".to_string()]);
    assert_eq!(s.arena_tells()[1].cells.len(), 12, "row 4");
    let off = session(timeline(&[POISE, FORCED_MOVE]), json!([vents()]));
    assert!(off.arena_tells().is_empty() && off.w.arena.hazards.is_empty());
}

/// AT3: PL4's threat count adds 1 for a cell an arena hazard strikes this round, so of two tiles equal otherwise the
/// one off the vents ranks first (without the hazard, the lower cell index wins).
#[test]
fn pl4_avoids_cells_due_this_round() {
    let s = session(rules(), json!([vents()]));
    let mut w = s.w.clone();
    w.grid.positions[FOE] = Some(v(0, 0));
    w.grid.positions[FOE_B] = Some(v(0, 1));
    let mut plain = w.clone();
    plain.arena.hazards.clear();
    let (vent, dry) = (v(7, 3), v(9, 3));
    assert!(arena::due_on(&w, vent) && !arena::due_on(&w, dry));
    assert_eq!(round_intent::threat(&plain, HERO, vent), round_intent::threat(&plain, HERO, dry), "the foes threaten both alike");
    assert_eq!(round_intent::threat(&w, HERO, vent), round_intent::threat(&plain, HERO, vent) + 1);
    let cands = || vec![(vent, vec![vent]), (dry, vec![dry])];
    let mut a = cands();
    round_intent::rank_tiles(&plain, HERO, v(8, 3), false, Aim::Toward, &mut a);
    assert_eq!(a[0].0, vent, "without the hazard: the lower index");
    let mut b = cands();
    round_intent::rank_tiles(&w, HERO, v(8, 3), false, Aim::Toward, &mut b);
    assert_eq!(b[0].0, dry, "the vent cell is a threat");
}

/// Runs round 1 of `s` with no planned action (the enemies' drafts dropped).
fn idle_round(s: &mut Session) {
    s.resolver.scheduled.clear();
    while s.current_round == 1 && !s.combat_over && s.phase != Phase::Replay {
        s.step_next();
    }
}

/// AH2, 4.2: in a session each strike gets an EVENT cell (the hazard as the actor, its line, the units caught) and each
/// struck unit a cell with the HP it lost, at blocks 3 and 6; the metrics balance (the strikes' HP is the struck units'
/// arena loss, the burn's round-end damage theirs) and count the strikes.
#[test]
fn the_session_logs_strikes_and_metrics_balance() {
    let mut s = session(rules(), json!([vents()]));
    s.attach_metrics(BattleMetrics::new(), true);
    idle_round(&mut s);
    let strikes: Vec<_> = s.log.iter().filter(|e| e.subtype == "Arena" && e.outcome == "STRIKE").collect();
    assert_eq!(strikes.iter().map(|e| e.block).collect::<Vec<_>>(), vec![3, 6]);
    assert_eq!((strikes[0].actor.as_str(), strikes[0].detail.as_str(), strikes[0].value), ("Slag Vents", "The slag vents roar.", 2));
    let cells: Vec<_> = s.log.iter().filter(|e| e.subtype == "Arena" && e.outcome == "ARENA").collect();
    assert_eq!(cells.iter().map(|e| (e.block, e.target.as_str())).collect::<Vec<_>>(), vec![(3, "p_foe"), (3, "p_foe_b"), (6, "p_foe"), (6, "p_foe_b")]);
    assert!(cells.iter().all(|e| e.damage == pct_damage(&s.w, FOE)), "12% each strike, no guard at a block's end");
    let m = s.metrics.as_ref().unwrap();
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    let taken: i32 = cells.iter().filter(|e| e.target == "p_foe").map(|e| e.damage).sum();
    let burns = cells.iter().filter(|e| e.target == "p_foe" && e.detail.contains(&status::display_name("burn"))).count() as i32;
    assert_eq!((m.arena["p_foe"].struck, m.arena["p_foe"].taken, m.arena["p_foe"].statuses), (2, taken, burns));
    assert!(!m.arena.contains_key("p_hero"));
    assert_eq!(m.to_json()["arena"]["strikes"].as_i64(), Some(2));
}

/// AH2: the strike comes before the verdict: foes on the vents with 1 HP fall at block 3's end and the battle is won
/// there.
#[test]
fn a_strike_comes_before_the_verdict() {
    let mut s = session(rules(), json!([vents()]));
    s.w.units[FOE].hp = 1;
    s.w.units[FOE_B].hp = 1;
    idle_round(&mut s);
    assert_eq!((s.outcome.as_str(), s.combat_over, s.current_block), ("won", true, 3));
    assert!(s.log.iter().filter(|e| e.subtype == "Arena" && e.outcome == "DEFEATED").count() == 2, "both fall to the vents");
}

/// 4.1, 4.2: with the rule off and in classic, an encounter with arena hazards plays exactly as one without (log, HP,
/// random draws, the AI's plans); with the rule on and no hazard row, as with the rule off.
#[test]
fn off_and_classic_change_nothing() {
    let play = |rules: Ruleset, hazards: Value| {
        let mut s = session(rules, hazards);
        for _ in 0..2 {
            while !s.combat_over && s.phase != Phase::Replay {
                s.step_next();
            }
            if s.can_advance_round() {
                s.advance_round();
            }
        }
        let log: Vec<Value> = s.log.iter().map(|e| e.trace()).collect();
        let hp: Vec<i32> = s.w.units.iter().map(|u| u.hp).collect();
        (log, hp, s.resolver.rng.randi(), s.w.arena.hazards.len())
    };
    for rules in [timeline(&[POISE, FORCED_MOVE]), Ruleset::classic()] {
        let with = play(rules.clone(), json!([vents(), belt()]));
        let without = play(rules.clone(), json!([]));
        assert_eq!(with.3, 0, "{}: no schedule", rules.id);
        assert!(with.0.len() > 4, "{}: the battle ran", rules.id);
        assert_eq!(with, without, "{}: the rows change nothing", rules.id);
    }
    assert_eq!(play(rules(), json!([])), play(timeline(&[POISE, FORCED_MOVE]), json!([])), "the rule on without a row");
}
