//! HOOKS1-F on a scratch catalog built here (no shipped data): forced movement under the rule `forced_move` (HOOKS1 5).
//! Expectations come from the rules: a push steps away from the attacker, a pull toward it, on the dominant axis; a
//! diagonal tie takes the axis of the attacker's facing, else x; a pull stops at distance 1 (FM2). Steps stop at the
//! edge, an unwalkable tile, an exit or an active unit (FM3); a blocked step with tiles left hits the moved unit and the
//! unit that blocked it for `collide_mult` x the attacker's power (`damage::compute`, no crit) and `collide_stagger`
//! through P6, and `collide_break` breaks a windup as P14 does (FM4). `immune` holds firm, `bonus_tiles` adds tiles up
//! to 4 (FM5); entered tiles roll their hazards; the move is not the unit's move; `stop_moves` cancels its Walks
//! (FM6, FM7). The session logs a cell per move and per collision hit and the metrics balance; with the rule off (and in
//! classic) nothing moves, logs or draws.

use indexmap::IndexMap;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::forced::{self, Shove, When};
use rpgkit_battle::grid::{V2, NO_CELL};
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::poise::{self, BreakCause};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::ruleset::{Ruleset, FORCED_MOVE, POISE, ROUND_INTENT, TIMELINE};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const BAR: i32 = 40;
/// The terrain index of `wall` and `mud` in `grid::TERRAIN_IDS`.
const WALL: usize = 10;
const MUD: usize = 8;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;

/// Engine skills and the test skills: a push 2 (reach 1-2, collision x0.5 and stagger 6), the same without a forced move,
/// a pull 3 (reach 1-3), a wall slam (push 1, x0.95, collide_break), a hook (pull 1 of a unit that moved, stop_moves), a
/// windup grab (push 1 when in windup), a push 3, a centre burst (PLUS, push 1 from the aim), a heavy and an unbreakable
/// heavy (3 blocks), a guard; passives immune always / guarding and bonus tiles 1 / 2; mud with an enter hazard (slow).
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
        attack("f_shove", 1, json!({"reach_min": 1, "reach_max": 2, "forced_move": {"kind": "push", "tiles": 2, "collide_mult": 0.5, "collide_stagger": 6}})),
        attack("f_plain", 1, json!({"reach_min": 1, "reach_max": 2})),
        attack("f_pull", 1, json!({"reach_min": 1, "reach_max": 3, "forced_move": {"kind": "pull", "tiles": 3}})),
        attack("f_slam", 1, json!({"forced_move": {"kind": "push", "tiles": 1, "collide_mult": 0.95, "collide_break": true}})),
        attack("f_hook", 1, json!({"reach_min": 1, "reach_max": 3, "forced_move": {"kind": "pull", "tiles": 1, "when": "moved", "stop_moves": true}})),
        attack("f_grab", 1, json!({"forced_move": {"kind": "push", "tiles": 1, "when": "windup"}})),
        attack("f_heave", 1, json!({"forced_move": {"kind": "push", "tiles": 3}})),
        attack("f_burst", 1, json!({"area": "PLUS", "reach_min": 1, "reach_max": 3, "forced_move": {"kind": "push", "tiles": 1, "from": "center"}})),
        attack("p_heavy", 3, json!({})),
        attack("p_judgment", 3, json!({"unbreakable": true})),
        json!({"id": "p_guard", "name": "p_guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.5}),
    ];
    let unit = |id: &str, side: &str, speed: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": speed, "attack_range": 1, "skill_cap": 5, "poise": BAR});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party", 14, json!({})), unit("p_ally", "party", 12, json!({})), unit("p_foe", "enemy", 10, json!({"armor": 25})), unit("p_foe_b", "enemy", 8, json!({}))],
        "passives": [
            {"id": "f_unyielding", "name": "Unyielding", "effects": [{"type": "forced_move", "immune": "always"}]},
            {"id": "f_braced", "name": "Braced", "effects": [{"type": "forced_move", "immune": "guarding"}]},
            {"id": "f_hypertrophy", "name": "Hypertrophy", "effects": [{"type": "forced_move", "bonus_tiles": 1}]},
            {"id": "f_titan", "name": "Titan", "effects": [{"type": "forced_move", "bonus_tiles": 2}]}
        ],
        "terrain": [{"id": "mud", "name": "Mud", "hazard_status": "slow", "hazard_potency": 100, "hazard_on": "enter"}]
    }))
}

fn timeline(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// Forced moves and poise (collision stagger and breaks).
fn rules() -> Ruleset {
    timeline(&[FORCED_MOVE, POISE])
}

/// The four units: hero (4,4) faces foe (5,4); ally and foe_b in far corners.
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "f_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe_b"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(0, 8)), (FOE, V2::new(5, 4)), (FOE_B, V2::new(11, 0))] {
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

/// A one-tile Walk by `dir` (the planned path as given).
fn walk(c: UnitId, dir: V2) -> Command {
    let mut cmd = Command::new(c, 0, 1, CmdKind::Walk { path: vec![dir], expected_start: NO_CELL, goal: NO_CELL, approach: None, moved_tiles: 0, cut_short: false, blocked_by: String::new() });
    cmd.name = "Move".into();
    cmd
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

fn set_terrain(w: &mut World, cell: V2, t: usize) {
    let width = rpgkit_battle::grid::GRID_W;
    w.grid.terrain[(cell.y * width + cell.x) as usize] = t;
}

fn passive(w: &mut World, c: UnitId, id: &str) {
    let cat = w.cat.clone();
    let p = cat.passives[id].clone();
    w.units[c].apply_passive(&cat, &p);
}

/// The collision damage of `attacker`'s power x `mult` on `victim` now (FM4: `damage::compute`, no crit roll).
fn collision(w: &World, attacker: UnitId, victim: UnitId, mult: f64) -> i32 {
    damage::compute(&w.units[attacker], &w.units[victim], HitInput { skill_mult: mult, ..HitInput::default() }).final_dmg
}

/// One attack by the hero on `t` at block 1 in a fresh copy of `s`; the world, the resolver and the action index.
fn strike(s: &Session, prep: impl FnOnce(&mut World), skill: &str, t: UnitId) -> (World, Resolver, usize) {
    let mut w = s.w.clone();
    prep(&mut w);
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&w, HERO, t, skill), 1);
    run(&mut w, &mut res, 1, 1);
    (w, res, a)
}

/// FM2: a push 2 steps east away from the attacker; a diagonal tie takes the attacker's facing axis (north), else x; a
/// pull from 3 tiles stops at distance 1.
#[test]
fn pushes_and_pulls_follow_the_dominant_axis() {
    let s = session(rules());
    let (w, res, a) = strike(&s, |_| {}, "f_shove", FOE);
    assert_eq!(w.pos(FOE), V2::new(7, 4), "2 tiles east");
    let m = &res.scheduled[a].command.pushes[0];
    assert_eq!((m.tiles, m.from, m.pull, m.held, m.blocked_by.as_str(), m.hits.len()), (2, V2::new(5, 4), false, false, "", 0));
    assert!(res.log.iter().any(|l| l == "[Push] p_hero drives p_foe 2 tiles to (7, 4)"), "{:?}", res.log);

    let diagonal = |w: &mut World| w.grid.positions[FOE] = Some(V2::new(5, 3));
    let (w, _, _) = strike(&s, |w| {
        diagonal(w);
        w.round_states.entry(HERO).or_default().facing = V2::new(0, -1);
    }, "f_shove", FOE);
    assert_eq!(w.pos(FOE), V2::new(5, 1), "a tie facing north: the y axis, away (north)");
    let (w, _, _) = strike(&s, diagonal, "f_shove", FOE);
    assert_eq!(w.pos(FOE), V2::new(7, 3), "a tie with no straight facing: the x axis");
    assert_eq!(forced::direction(&w, HERO, V2::new(4, 4), V2::new(5, 3), true), V2::new(-1, 0), "a pull steps back along x");

    let (w, res, a) = strike(&s, |w| w.grid.positions[HERO] = Some(V2::new(2, 4)), "f_pull", FOE);
    assert_eq!((w.pos(FOE), w.dist(HERO, FOE)), (V2::new(3, 4), 1), "a pull stops at distance 1");
    let m = &res.scheduled[a].command.pushes[0];
    assert_eq!((m.tiles, m.pull, m.blocked_by.as_str()), (2, true, ""), "stopping short is no collision");
    assert!(res.log.iter().any(|l| l == "[Pull] p_hero drags p_foe 2 tiles to (3, 4)"));
}

/// FM3, FM4: after 1 tile a wall, a unit or an exit stops the push; the moved unit takes the collision hit (the
/// attacker's power x 0.5 through its armour) and stagger 6; a blocking unit takes its own hit and stagger; the exit is
/// never entered.
#[test]
fn a_blocked_push_collides() {
    let s = session(rules());
    let wall = |w: &mut World| set_terrain(w, V2::new(7, 4), WALL);
    let mut probe = s.w.clone();
    wall(&mut probe);
    let expect = collision(&probe, HERO, FOE, 0.5);
    let (w, res, a) = strike(&s, wall, "f_shove", FOE);
    let cmd = &res.scheduled[a].command;
    let CmdKind::Attack { last_damage, .. } = cmd.kind else { panic!() };
    let m = &cmd.pushes[0];
    assert_eq!((w.pos(FOE), m.tiles, m.blocked_by.as_str(), m.blocker), (V2::new(6, 4), 1, "the wall", None));
    assert_eq!((m.hits.len(), m.hits[0].unit, m.hits[0].damage), (1, FOE, expect), "walls take nothing");
    assert_eq!(w.units[FOE].hp, 5000 - last_damage - expect, "the hit, then the collision");
    let st = m.hits[0].stagger.as_ref().expect("a stagger record");
    assert_eq!((st.taken, poise::state(&w, FOE).unwrap().value), (6, BAR - 6));
    assert!(res.log.iter().any(|l| *l == format!("[Collision] p_foe hits the wall: {expect} damage")), "{:?}", res.log);

    let block = |w: &mut World| w.grid.positions[FOE_B] = Some(V2::new(7, 4));
    let mut probe = s.w.clone();
    block(&mut probe);
    let (moved, blocker) = (collision(&probe, HERO, FOE, 0.5), collision(&probe, HERO, FOE_B, 0.5));
    assert_ne!(moved, blocker, "the armour differs, so the two hits do");
    let (w, res, a) = strike(&s, block, "f_shove", FOE);
    let m = &res.scheduled[a].command.pushes[0];
    assert_eq!((w.pos(FOE), w.pos(FOE_B), m.blocked_by.as_str(), m.blocker), (V2::new(6, 4), V2::new(7, 4), "p_foe_b", Some(FOE_B)));
    let hits: Vec<(UnitId, i32, i32)> = m.hits.iter().map(|h| (h.unit, h.damage, h.stagger.as_ref().map_or(0, |st| st.taken))).collect();
    assert_eq!(hits, vec![(FOE, moved, 6), (FOE_B, blocker, 6)], "both units hit and staggered");
    assert_eq!(w.units[FOE_B].hp, 5000 - blocker);

    let (w, res, a) = strike(&s, |w| w.grid.exits = vec![V2::new(7, 4)], "f_shove", FOE);
    let m = &res.scheduled[a].command.pushes[0];
    assert_eq!((w.pos(FOE), w.units[FOE].is_withdrawn, m.blocked_by.as_str()), (V2::new(6, 4), false, "the exit"), "never withdrawn");
    let (w, res, a) = strike(&s, |w| {
        w.grid.exits.clear();
        w.grid.positions[HERO] = Some(V2::new(9, 4));
        w.grid.positions[FOE] = Some(V2::new(10, 4));
    }, "f_heave", FOE);
    let m = &res.scheduled[a].command.pushes[0];
    assert_eq!((w.pos(FOE), m.tiles, m.blocked_by.as_str()), (V2::new(11, 4), 1, "the arena edge"), "the edge stops it");
}

/// FM4: `collide_break` breaks a unit in windup (as P14: whatever its poise); not an unbreakable windup, not a unit out
/// of windup, never the blocker.
#[test]
fn a_wall_slam_breaks_a_windup() {
    let s = session(rules());
    for (windup, skill, breaks) in [(true, "p_heavy", true), (true, "p_judgment", false), (false, "", false)] {
        let mut w = s.w.clone();
        set_terrain(&mut w, V2::new(6, 4), WALL);
        let mut res = Resolver::new(BLOCKS, 1);
        let mine = windup.then(|| book(&mut res, FOE, hit(&w, FOE, HERO, skill), 1));
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, "f_slam"), 1);
        run(&mut w, &mut res, 1, 1);
        let h = &res.scheduled[a].command.pushes[0].hits[0];
        let st = h.stagger.as_ref().expect("collide_break makes a record");
        assert_eq!(st.taken, 0, "{skill}: no collide_stagger, poise alone breaks nothing");
        assert_eq!(st.broke.as_ref().map(|b| b.cause), breaks.then_some(BreakCause::Collision), "{skill}");
        if let Some(i) = mine {
            assert_eq!(res.scheduled[i].cancelled, breaks, "{skill}: the windup is cancelled by the break");
        }
    }
    let mut w = s.w.clone();
    w.grid.positions[FOE_B] = Some(V2::new(6, 4));
    let mut res = Resolver::new(BLOCKS, 1);
    let theirs = book(&mut res, FOE_B, hit(&w, FOE_B, FOE, "p_heavy"), 1);
    let a = book(&mut res, HERO, hit(&w, HERO, FOE, "f_slam"), 1);
    run(&mut w, &mut res, 1, 1);
    let hits = &res.scheduled[a].command.pushes[0].hits;
    assert_eq!(hits[1].unit, FOE_B);
    assert!(hits[1].stagger.is_none() && !res.scheduled[theirs].cancelled, "the blocker in windup does not break");
}

/// FM5: `immune` always holds firm (no move, no collision); `guarding` holds only while a Defend runs; the attacker's
/// `bonus_tiles` add to its pushes, up to 4 tiles.
#[test]
fn immunity_and_bonus_tiles() {
    let s = session(rules());
    let (w, res, a) = strike(&s, |w| {
        passive(w, FOE, "f_unyielding");
        set_terrain(w, V2::new(6, 4), WALL);
    }, "f_shove", FOE);
    let m = &res.scheduled[a].command.pushes[0];
    assert_eq!((w.pos(FOE), m.held, m.hits.len()), (V2::new(5, 4), true, 0), "holds firm, no collision");
    assert!(res.log.iter().any(|l| l == "[Hold] p_foe holds firm"));

    for guarding in [true, false] {
        let mut w = s.w.clone();
        passive(&mut w, FOE, "f_braced");
        let mut res = Resolver::new(BLOCKS, 1);
        if guarding {
            book(&mut res, FOE, own(&w, FOE, "p_guard"), 1);
        }
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, "f_shove"), 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!((res.scheduled[a].command.pushes[0].held, w.pos(FOE) == V2::new(5, 4)), (guarding, guarding), "guarding {guarding}");
    }

    let (w, _, _) = strike(&s, |w| passive(w, HERO, "f_hypertrophy"), "f_shove", FOE);
    assert_eq!(w.pos(FOE), V2::new(8, 4), "push 2 + bonus 1");
    let (w, _, _) = strike(&s, |w| passive(w, HERO, "f_titan"), "f_heave", FOE);
    assert_eq!(w.pos(FOE), V2::new(5 + forced::MAX_TILES, 4), "push 3 + bonus 2, capped at {}", forced::MAX_TILES);
}

/// FM5: the effect reads from passives and from worn items; the gear part is rebuilt on every gear sync.
#[test]
fn passive_and_gear_effects_add_up() {
    let s = session(rules());
    let mut u = s.w.units[FOE].clone();
    let cat = s.w.cat.clone();
    assert_eq!(u.forced_traits(), Default::default());
    u.apply_effects(&cat, &[json!({"type": "forced_move", "immune": "guarding", "bonus_tiles": 1})], "passive:a", false);
    u.apply_effects(&cat, &[json!({"type": "forced_move", "immune": "always", "bonus_tiles": 2})], "gear:b", true);
    let t = u.forced_traits();
    assert_eq!((t.immune_always, t.immune_guarding, t.bonus_tiles), (true, true, 3));
    u.sync_from_stats_and_gear(&cat);
    let t = u.forced_traits();
    assert_eq!((t.immune_always, t.immune_guarding, t.bonus_tiles), (false, true, 1), "no worn item: the gear part is gone");
}

/// FM6, FM7: a tile entered rolls its hazard (mud: slow at potency 100); `when` moved pulls a unit whose Walk landed
/// this round and not one that stood; `stop_moves` cancels its later Walk; a pushed stealthed unit is still found.
#[test]
fn hazards_moves_and_stealth() {
    let s = session(rules());
    let (w, mut res, _) = strike(&s, |w| set_terrain(w, V2::new(6, 4), MUD), "f_shove", FOE);
    let (_, mut floor, _) = strike(&s, |_| {}, "f_shove", FOE);
    assert_eq!(w.pos(FOE), V2::new(7, 4));
    // the mud's roll is the one extra draw (randi_range(1, 100) <= the slow's apply chance)
    let roll = floor.rng.randi_range(1, 100);
    let lands = roll <= status::apply_chance(&s.w.units[FOE], "slow", 100);
    assert_eq!(res.rng.randi(), floor.rng.randi(), "one roll for the mud tile");
    assert_eq!(w.units[FOE].st.has_any("slow"), lands, "roll {roll}");
    assert_eq!(res.log.iter().any(|l| l.starts_with("[Push]") && l.contains("[Hazard] p_foe is Slowed")), lands, "{:?}", res.log);

    for walker in [true, false] {
        let mut w = s.w.clone();
        w.grid.positions[HERO] = Some(V2::new(3, 4));
        w.grid.positions[FOE] = Some(V2::new(if walker { 6 } else { 5 }, 4));
        let mut res = Resolver::new(BLOCKS, 1);
        if walker {
            book(&mut res, FOE, walk(FOE, V2::new(-1, 0)), 1);
        }
        let later = book(&mut res, FOE, walk(FOE, V2::new(0, -1)), 3);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.pos(FOE), V2::new(5, 4), "walker {walker}: at (5, 4) before the hook");
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, "f_hook"), 2);
        run(&mut w, &mut res, 2, 3);
        let pulled = !res.scheduled[a].command.pushes.is_empty();
        assert_eq!(pulled, walker, "when moved");
        assert_eq!(res.scheduled[later].cancelled, walker, "stop_moves cancels the unlanded Walk");
        assert_eq!(w.pos(FOE), if walker { V2::new(4, 4) } else { V2::new(5, 3) });
        if walker {
            assert!(res.log.iter().any(|l| l.ends_with("its moves this round are cancelled")), "{:?}", res.log);
        }
    }

    let mut s = session(timeline(&[ROUND_INTENT, FORCED_MOVE]));
    let w = &mut s.w;
    w.grid.positions[HERO] = Some(V2::new(2, 4));
    for (c, t) in [(HERO, FOE), (FOE, HERO)] {
        round_intent::set_intent(w, c, RoundIntent::Assault).unwrap();
        round_intent::set_target(w, c, RoundTarget::Unit(t)).unwrap();
    }
    status::grant_now(&mut w.units[FOE], "stealth", 2, 0, "test");
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, hit(w, HERO, FOE, "f_pull"), 1);
    run(w, &mut res, 1, 1);
    assert_eq!(w.pos(FOE), V2::new(3, 4), "pulled");
    assert!(!round_intent::hidden(w, FOE), "a forced move is not the unit's move (GD5)");
    let a = book(&mut res, HERO, hit(w, HERO, FOE, "strike"), 2);
    run(w, &mut res, 2, 2);
    assert!(res.scheduled[a].command.log.starts_with("[Attack]"), "still found: {}", res.scheduled[a].command.log);
    book(&mut res, FOE, walk(FOE, V2::new(0, 1)), 3);
    run(w, &mut res, 3, 3);
    assert!(round_intent::hidden(w, FOE), "control: its own Walk hides it");
}

/// FM4: under `forced_move` alone (no rule poise) a collision still deals its damage but makes no stagger record and
/// breaks nothing, also with `collide_break` into a windup.
#[test]
fn without_poise_a_collision_only_damages() {
    let s = session(timeline(&[FORCED_MOVE]));
    assert!(s.w.poise.is_empty(), "no bars");
    for skill in ["f_shove", "f_slam"] {
        let mut w = s.w.clone();
        set_terrain(&mut w, V2::new(6, 4), WALL);
        let mut res = Resolver::new(BLOCKS, 1);
        let heavy = book(&mut res, FOE, hit(&w, FOE, HERO, "p_heavy"), 1);
        let mult = if skill == "f_shove" { 0.5 } else { 0.95 };
        let expect = collision(&w, HERO, FOE, mult);
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, skill), 1);
        run(&mut w, &mut res, 1, 1);
        let h = &res.scheduled[a].command.pushes[0].hits[0];
        assert_eq!((h.unit, h.damage), (FOE, expect), "{skill}: the damage stays");
        assert!(h.stagger.is_none() && !res.scheduled[heavy].cancelled, "{skill}: no stagger record, no break");
        assert!(res.log.iter().all(|l| !l.starts_with("[Break]")), "{skill}: {:?}", res.log);
    }
}

/// FM8: the moved unit's scheduled actions stay booked; each lands only if still in reach at its impact.
#[test]
fn scheduled_actions_stay_and_land_only_in_reach() {
    let s = session(rules());
    for push in [true, false] {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        let heavy = book(&mut res, FOE, hit(&w, FOE, HERO, "p_heavy"), 1);
        book(&mut res, HERO, hit(&w, HERO, FOE, if push { "f_shove" } else { "f_plain" }), 1);
        run(&mut w, &mut res, 1, 3);
        let a = &res.scheduled[heavy];
        assert!(!a.cancelled && a.landed, "push {push}: still booked, it lands");
        assert_eq!(a.command.log.starts_with("[Attack Out of Range]"), push, "push {push}: {}", a.command.log);
    }
}

/// FM1: `when` windup moves only a unit in windup as the hit lands; `from` center pushes away from the area's aim, a unit
/// on the aim cell away from the attacker; a skill's forced move acts once per damaged unit.
#[test]
fn windup_condition_and_the_area_centre() {
    let s = session(rules());
    for windup in [true, false] {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        if windup {
            book(&mut res, FOE, hit(&w, FOE, HERO, "p_heavy"), 1);
        }
        book(&mut res, HERO, hit(&w, HERO, FOE, "f_grab"), 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.pos(FOE) == V2::new(6, 4), windup, "windup {windup}");
    }

    let mut w = s.w.clone();
    w.grid.positions[FOE] = Some(V2::new(6, 3));
    w.grid.positions[FOE_B] = Some(V2::new(6, 4));
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, HERO, PlanStep::area_at(sid(&w, "f_burst"), V2::new(6, 4)), true), "");
    let cmd = plan.entries_for(&w, HERO).into_iter().find(|e| e.skill == sid(&w, "f_burst")).unwrap().build_command(&w, HERO);
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cmd, 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(res.scheduled[a].command.pushes.len(), 2, "each damaged unit once");
    assert_eq!(w.pos(FOE), V2::new(6, 2), "away from the aim (north)");
    assert_eq!(w.pos(FOE_B), V2::new(7, 4), "on the aim cell: away from the attacker");
}

/// 4.1, 4.2: with the rule off and in classic the field is ignored: no move, no line, the same random draws as the
/// skill without it. The command carries the skill's forced move either way (attacks only).
#[test]
fn off_and_classic_change_nothing() {
    for rules in [timeline(&[POISE]), Ruleset::classic()] {
        let s = session(rules.clone());
        let (w, mut res, a) = strike(&s, |_| {}, "f_shove", FOE);
        let (_, mut plain, _) = strike(&s, |_| {}, "f_plain", FOE);
        assert_eq!(w.pos(FOE), V2::new(5, 4), "{}: no move", rules.id);
        assert!(res.scheduled[a].command.pushes.is_empty() && res.log.iter().all(|l| !l.starts_with("[Push]")));
        assert_eq!(res.rng.randi(), plain.rng.randi(), "{}: no new draw", rules.id);
        let shove = res.scheduled[a].command.forced.expect("carried");
        assert_eq!((shove.pull, shove.tiles, shove.when, shove.collide_mult, shove.collide_stagger), (false, 2, When::Always, 0.5, 6));
    }
    let w = &session(rules()).w;
    assert!(Shove::of(&w.cat.skills[sid(w, "f_plain")]).is_none() && Shove::of(&w.cat.skills[sid(w, "p_guard")]).is_none());
    assert_eq!(Shove::of(&w.cat.skills[sid(w, "f_hook")]).map(|f| (f.pull, f.when, f.stop_moves)), Some((true, When::Moved, true)));
}

/// FM9, 4.2: in a session each forced move gets an EVENT cell and each collision hit an ATTACK cell whose damage is the
/// HP lost; the metrics balance (no METRIC FAIL), count the push under `forced` and the collision damage as the
/// driver's; classic keeps the metrics shape.
#[test]
fn the_session_logs_and_counts_forced_moves() {
    let mut s = session(rules());
    s.w.grid.positions[FOE_B] = Some(V2::new(7, 4));
    s.attach_metrics(BattleMetrics::new(), true);
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "f_shove"), FOE), true), "");
    while s.current_round == 1 && !s.combat_over && s.phase != Phase::Replay {
        s.step_next();
    }
    let push: Vec<_> = s.log.iter().filter(|e| e.subtype == "Push").collect();
    assert_eq!(push.len(), 1, "one push cell");
    assert_eq!((push[0].category, push[0].actor.as_str(), push[0].target.as_str(), push[0].value), ("EVENT", "p_hero", "p_foe", 1));
    let hits: Vec<(&str, i32)> = s.log.iter().filter(|e| e.subtype == "Collision").map(|e| (e.target.as_str(), e.damage)).collect();
    assert_eq!(hits.iter().map(|h| h.0).collect::<Vec<_>>(), vec!["p_foe", "p_foe_b"], "the moved unit, then the blocker");
    assert!(hits.iter().all(|h| h.1 > 0));
    let m = s.metrics.as_ref().unwrap();
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    let f = &m.forced["p_hero"];
    assert_eq!((f.pushes, f.tiles, f.collisions, f.collision_damage), (1, 1, 1, hits[0].1 + hits[1].1));
    assert_eq!((m.forced["p_foe"].moved, m.to_json()["forced"]["p_hero"]["collisions"].as_i64()), (1, Some(1)));
    let dealt: i32 = s.log.iter().filter(|e| e.category == "ATTACK" && e.actor == "p_hero").map(|e| e.damage).sum();
    assert_eq!(m.units["p_hero"].dmg_dealt, dealt, "collisions count as the driver's damage");

    let mut c = session(Ruleset::classic());
    c.w.grid.positions[FOE_B] = Some(V2::new(7, 4));
    c.attach_metrics(BattleMetrics::new(), true);
    assert_eq!(c.plan.add_step(&c.w, HERO, PlanStep::engage(sid(&c.w, "f_shove"), FOE), true), "");
    while c.current_round == 1 && !c.combat_over && c.phase != Phase::Replay {
        c.step_next();
    }
    assert!(c.log.iter().all(|e| e.subtype != "Push" && e.subtype != "Collision"));
    let m = c.metrics.as_ref().unwrap();
    assert!(m.forced.is_empty() && m.to_json().get("forced").is_none(), "classic keeps the metrics shape");
}

/// HOOKS1-UI slice 4: `forced::forecast`, read before the hit, is the move the hit then makes (the same steps, end
/// tile, hold, obstacle and blocker) in every case above: a free push, a wall, a blocking unit, an exit, the arena edge,
/// a pull, a diagonal tie on the attacker's facing, an immune unit and bonus tiles. Without the rule it forecasts none.
#[test]
fn the_forecast_is_the_move_the_hit_makes() {
    type Prep = Box<dyn Fn(&mut World)>;
    let s = session(rules());
    let cases: Vec<(&str, &str, Prep)> = vec![
        ("free push", "f_shove", Box::new(|_| {})),
        ("wall", "f_shove", Box::new(|w| set_terrain(w, V2::new(7, 4), WALL))),
        ("blocker", "f_shove", Box::new(|w| w.grid.positions[FOE_B] = Some(V2::new(7, 4)))),
        ("exit", "f_shove", Box::new(|w| w.grid.exits = vec![V2::new(7, 4)])),
        ("edge", "f_heave", Box::new(|w| {
            w.grid.exits.clear();
            w.grid.positions[HERO] = Some(V2::new(9, 4));
            w.grid.positions[FOE] = Some(V2::new(10, 4));
        })),
        ("pull", "f_pull", Box::new(|w| w.grid.positions[HERO] = Some(V2::new(2, 4)))),
        ("tie facing north", "f_shove", Box::new(|w| {
            w.grid.positions[FOE] = Some(V2::new(5, 3));
            w.round_states.entry(HERO).or_default().facing = V2::new(0, -1);
        })),
        ("immune", "f_shove", Box::new(|w| passive(w, FOE, "f_unyielding"))),
        ("bonus tiles", "f_shove", Box::new(|w| passive(w, HERO, "f_hypertrophy"))),
    ];
    for (name, skill, prep) in cases {
        let mut w = s.w.clone();
        prep(&mut w);
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, skill), 1);
        let (cells, hp, log) = (w.grid.positions.clone(), w.units[FOE].hp, res.log.len());
        let fc = forced::forecast(&w, &res.scheduled, HERO, w.pos(HERO), &res.scheduled[a].command, res.scheduled[a].impact, &[FOE]);
        assert_eq!((w.grid.positions.clone(), w.units[FOE].hp, res.log.len()), (cells, hp, log), "{name}: the forecast changes nothing");
        run(&mut w, &mut res, 1, 1);
        let m = &res.scheduled[a].command.pushes[0];
        assert_eq!(fc.len(), 1, "{name}");
        let f = &fc[0];
        assert_eq!((f.from, f.to, f.path.len() as i32, f.pull, f.held, f.blocked_by.as_str(), f.blocker), (m.from, m.to, m.tiles, m.pull, m.held, m.blocked_by.as_str(), m.blocker), "{name}");
        assert_eq!(f.to, w.pos(FOE), "{name}");
    }
    let off = session(timeline(&[POISE]));
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&off.w, HERO, FOE, "f_shove"), 1);
    assert!(forced::forecast(&off.w, &res.scheduled, HERO, off.w.pos(HERO), &res.scheduled[a].command, res.scheduled[a].impact, &[FOE]).is_empty());
}
