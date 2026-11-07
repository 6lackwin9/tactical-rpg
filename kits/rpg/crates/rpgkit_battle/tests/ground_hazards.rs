//! HOOKS1-G on a scratch catalog built here (no shipped data): ground hazards under the rule `ground_hazards` (HOOKS1
//! 6). Expectations come from the rules: a placement covers its shape's walkable tiles, a trap only free ones (GH2); an
//! ATTACK with no power and a hazard aims at a tile, hits nobody and draws nothing (GH3); the newest hazard replaces
//! the old and a terrain hazard acts first (GH4); flat damage through guard, not armour, then the status roll; cover is
//! the larger of terrain and hazard and only for ranged hits; move cost adds (GH5); a trap spares the units it does not
//! affect, stops the move of one it does on its tile, cancels its later Walks and is gone, also in a forced move, a dash
//! and a carried move (GH6, FM6); guard reduces hazard damage (GH5); `rounds` N lasts through round placed + N - 1
//! (GH7); the AI path penalty counts a hazard that would harm the unit and that its side sees (GH8). The session logs a
//! cell per placement, hit and fade and the metrics balance; with the rule off (and in classic) the skills carry no
//! hazard, a zero-power one is a plain attack, and nothing is placed, logged or drawn (4.1).

use indexmap::IndexMap;
use rpgkit_battle::command::{self, Carry, CmdKind, Command};
use rpgkit_battle::ground::{self, Event, Hit, Place, Placed, Shape};
use rpgkit_battle::grid::{GRID_H, NO_CELL, V2};
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::path::{self, Cells, Hooks};
use rpgkit_battle::plan::{Plan, PlanStep, Step, AI_HAZARD_PENALTY};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, ItemDef};
use rpgkit_rules::ruleset::{Ruleset, FORCED_MOVE, GROUND_HAZARDS, ROUND_INTENT, TIMELINE};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HP: i32 = 5000;
/// The terrain index of `wall`, `mud` and `rubble` in `grid::TERRAIN_IDS`.
const WALL: usize = 10;
const MUD: usize = 8;
const RUBBLE: usize = 13;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;

/// Engine skills and the test skills: a tripwire placement (power 0, reach 1-3, 3 rounds), a fire placement, a spit
/// (a hit that leaves fire on the target's tile), a grease pot (BURST, grease on its own cells for 2 rounds), a smoke
/// Defend (BURST on itself for 2 rounds), a shove (push 3), a plain attack, a dash attack (reach 1, dash 5) and a
/// Maneuver lunge (the same with a carried move under `round_intent`). Hazards: smoke (cover 0.3), a hidden
/// tripwire (15 damage, root 90, stops moves), the same tripwire seen, grease (slow 100, move cost +1), fire (round end:
/// 8 damage, burn 80). Mud has an enter hazard (poison 100), rubble cover 0.4. A flask leaves smoke around its user.
fn catalog() -> Catalog {
    let attack = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.2, "reach_min": 1, "reach_max": 4, "target_mode": "ENEMY"});
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
        attack("g_snare", json!({"power_mult": 0.0, "reach_max": 3, "hazard": {"id": "g_trip", "shape": "NONE", "at": "target", "rounds": 3}})),
        attack("g_burn", json!({"power_mult": 0.0, "hazard": {"id": "g_fire", "shape": "NONE", "at": "target"}})),
        attack("g_spit", json!({"hazard": {"id": "g_fire", "shape": "NONE", "at": "target"}})),
        attack("g_pot", json!({"area": "BURST", "hazard": {"id": "g_grease", "shape": "SKILL", "at": "target", "rounds": 2}})),
        attack("g_shove", json!({"forced_move": {"kind": "push", "tiles": 3}})),
        attack("g_plain", json!({})),
        attack("g_dash", json!({"reach_max": 1, "dash_tiles": 5})),
        attack("g_lunge", json!({"reach_max": 1, "dash_tiles": 5, "round_intent": "maneuver"})),
        json!({"id": "g_fade", "name": "g_fade", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.2, "hazard": {"id": "g_smoke", "shape": "BURST", "at": "self", "rounds": 2}}),
    ];
    let unit = |id: &str, side: &str, speed: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "speed": speed, "attack_range": 1, "skill_cap": 5});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let trip = json!({"id": "g_trip", "name": "Tripwire", "on": "enter", "status": "root", "potency": 90, "status_rounds": 1, "damage": 15, "affects": "foes", "trap": true, "hidden": true, "stop_moves": true, "duration": 3});
    let mut seen = trip.clone();
    seen["id"] = json!("g_trip_seen");
    seen["hidden"] = json!(false);
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party", 14, json!({})), unit("p_ally", "party", 12, json!({})), unit("p_foe", "enemy", 10, json!({"armor": 25})), unit("p_foe_b", "enemy", 8, json!({}))],
        "hazards": [
            {"id": "g_smoke", "name": "Soot Smoke", "on": "both", "cover_guard": 0.3, "duration": 2},
            trip,
            seen,
            {"id": "g_grease", "name": "Scalding Grease", "on": "both", "status": "slow", "potency": 100, "move_cost": 1, "duration": 2},
            {"id": "g_fire", "name": "Phosphorus Fire", "on": "round_end", "status": "burn", "potency": 80, "magnitude": 5, "status_rounds": 2, "damage": 8, "duration": 2}
        ],
        "terrain": [{"id": "mud", "name": "Mud", "hazard_status": "poison", "hazard_potency": 100, "hazard_on": "enter"}, {"id": "rubble", "name": "Rubble", "cover_guard": 0.4}]
    }))
}

fn flask() -> ItemDef {
    let row = json!({"id": "g_flask", "name": "Smoke Flask", "item_type": "CONSUMABLE", "use_reach": 0, "hazard": {"id": "g_smoke", "shape": "BURST", "at": "self"}});
    ItemDef::from_obj(row.as_object().unwrap())
}

fn timeline(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

fn rules() -> Ruleset {
    timeline(&[GROUND_HAZARDS, FORCED_MOVE])
}

/// Hero (4, 4) and foe (8, 4) on one row; the ally and foe_b in far corners.
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "g_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe_b"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(0, 8)), (FOE, V2::new(8, 4)), (FOE_B, V2::new(11, 0))] {
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

fn at_tile(w: &World, c: UnitId, skill: &str, cell: V2) -> Command {
    command(w, c, PlanStep::area_at(sid(w, skill), cell), skill)
}

fn hit(w: &World, c: UnitId, t: UnitId, skill: &str) -> Command {
    command(w, c, PlanStep::engage(sid(w, skill), t), skill)
}

/// A Walk along `dirs` (the planned path as given).
fn walk(c: UnitId, dirs: &[V2]) -> Command {
    let mut cmd = Command::new(c, 0, 1, CmdKind::Walk { path: dirs.to_vec(), expected_start: NO_CELL, goal: NO_CELL, approach: None, moved_tiles: 0, cut_short: false, blocked_by: String::new() });
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

/// Lays the row `id` from `by` on `cells` directly (round 1, 3 rounds).
fn lay(w: &mut World, id: &str, by: UnitId, cells: &[V2]) {
    let def = w.cat.hazards[id].clone();
    for c in cells {
        w.grid.ground.insert(*c, Placed { def: def.clone(), by, round: 1, last_round: 3 });
    }
}

fn ids(w: &World) -> Vec<(V2, String)> {
    w.grid.ground.iter().map(|(c, p)| (*c, p.def.id.clone())).collect()
}

const fn v(x: i32, y: i32) -> V2 {
    V2::new(x, y)
}

/// GH2: each shape at the target tile and at the user's: the cells, walkable only (a wall at (7, 4) and (4, 3) is left
/// out); a trap leaves out an occupied tile; SKILL takes the skill's own cells.
#[test]
fn placement_shapes_cover_walkable_tiles() {
    let s = session(rules());
    let mut w = s.w.clone();
    set_terrain(&mut w, v(7, 4), WALL);
    set_terrain(&mut w, v(4, 3), WALL);
    let smoke = w.cat.hazards["g_smoke"].clone();
    let (me, aim) = (v(4, 4), v(6, 4));
    let cells = |shape: Shape, at_self: bool| {
        let p = Place { id: "g_smoke".into(), shape, at_self, rounds: 0 };
        ground::cells(&w, HERO, &smoke, &p, if at_self { me } else { aim }, aim, &[])
    };
    assert_eq!(cells(Shape::One, false), vec![v(6, 4)]);
    assert_eq!(cells(Shape::One, true), vec![v(4, 4)]);
    assert_eq!(cells(Shape::Plus, false), vec![v(6, 4), v(5, 4), v(6, 5), v(6, 3)], "the wall east is left out");
    let burst = cells(Shape::Burst, true);
    assert_eq!((burst.len(), burst.contains(&v(4, 3)), burst.contains(&v(5, 5))), (8, false, true), "3 x 3 around the user but the wall");
    assert_eq!(cells(Shape::Line, false), vec![v(6, 4), v(8, 4)], "from the aim away from the user, the wall left out");
    assert_eq!(cells(Shape::Line, true), vec![v(5, 4), v(6, 4)], "from the tile ahead of the user");
    assert_eq!(cells(Shape::Sweep, false), vec![v(6, 3), v(6, 4), v(6, 5)], "across the heading at the aim");
    assert_eq!(cells(Shape::Sweep, true), vec![v(5, 3), v(5, 4), v(5, 5)], "across the heading ahead of the user");
    // A3.2-S1: ARC covers SWEEP's tiles (the alias it replaces in W2-4); RING the 8 around its centre, never the centre.
    assert_eq!(cells(Shape::Arc, false), cells(Shape::Sweep, false), "ARC at the aim as SWEEP");
    assert_eq!(cells(Shape::Arc, true), cells(Shape::Sweep, true), "ARC ahead of the user as SWEEP");
    let ring = cells(Shape::Ring, true);
    assert_eq!((ring.len(), ring.contains(&me), ring.contains(&v(4, 3)), ring.contains(&v(5, 5))), (7, false, false, true), "around the user but the wall");
    let ring = cells(Shape::Ring, false);
    assert_eq!((ring.len(), ring.contains(&aim), ring.contains(&v(7, 4)), ring.contains(&v(5, 3))), (7, false, false, true), "around the aim but the wall");
    let area = [v(6, 4), v(7, 4), v(6, 5)];
    let p = Place { id: "g_smoke".into(), shape: Shape::Skill, at_self: false, rounds: 0 };
    assert_eq!(ground::cells(&w, HERO, &smoke, &p, aim, aim, &area), vec![v(6, 4), v(6, 5)], "SKILL: the skill's cells");
    let trip = w.cat.hazards["g_trip"].clone();
    let p = Place { id: "g_trip".into(), shape: Shape::Plus, at_self: false, rounds: 0 };
    let on_foe = ground::cells(&w, HERO, &trip, &p, v(8, 4), v(8, 4), &[]);
    assert!(!on_foe.contains(&v(8, 4)) && on_foe.contains(&v(8, 5)), "a trap only on free tiles: {on_foe:?}");
    let pl = Place::of_skill(&w.cat.skills[sid(&w, "g_fade")]).expect("a Defend's hazard");
    assert_eq!((pl.shape, pl.at_self, pl.rounds), (Shape::Burst, true, 2));
    assert!(Place::of_skill(&w.cat.skills[sid(&w, "g_plain")]).is_none());
}

/// A3.2-K2 (FB-106, INTENT2 IR7): a hazard ARC covers the skill's own ARC tiles (`skill::arc_cells`) at the aim, off-axis
/// and diagonal included; at the user's tile, the arc on the snapped tile ahead toward the aim. A hazard LINE takes the
/// snapped direction too (a tie is the diagonal). An open board (no walls), the trap-free smoke.
#[test]
fn hazard_arc_and_line_follow_the_skill_shapes() {
    let s = session(rules());
    let w = s.w.clone();
    let smoke = w.cat.hazards["g_smoke"].clone();
    let me = w.pos(HERO);
    let cells = |shape: Shape, at_self: bool, aim: V2| {
        let p = Place { id: "g_smoke".into(), shape, at_self, rounds: 0 };
        ground::cells(&w, HERO, &smoke, &p, if at_self { me } else { aim }, aim, &[])
    };
    for d in [v(1, 0), v(0, -1), v(1, 1), v(-1, 1), v(2, 0), v(2, 1), v(2, 2), v(-1, -3)] {
        let aim = me + d;
        assert_eq!(cells(Shape::Arc, false, aim), skill::arc_cells(me, aim), "ARC at the aim {d:?} as the skill's ARC");
        let ahead = me + skill::snap(d);
        assert_eq!(cells(Shape::Arc, true, aim), skill::arc_cells(me, ahead), "ARC ahead of the user toward {d:?}");
    }
    assert_eq!(cells(Shape::Line, false, me + v(2, 2)), vec![me + v(2, 2), me + v(3, 3), me + v(4, 4)], "LINE on a tie: the diagonal");
    assert_eq!(cells(Shape::Line, true, me + v(2, 1)), vec![me + v(1, 0), me + v(2, 0), me + v(3, 0)], "LINE (2, 1) snaps east");
    // SWEEP keeps its straight heading (legacy, until W2-4 removes it)
    assert_eq!(cells(Shape::Sweep, true, me + v(1, 0)), vec![me + v(1, -1), me + v(1, 0), me + v(1, 1)]);
}

/// GH3: an ATTACK with power 0 and a hazard is a placement aimed at a tile: no damage, no random draw, its hazard on the
/// tile for its rounds; a trap aimed at an occupied tile places nothing.
#[test]
fn a_placement_attack_hits_nobody_and_draws_nothing() {
    let s = session(rules());
    let skills = &s.w.cat.skills;
    assert!(skill::is_placement(&skills[sid(&s.w, "g_snare")]) && skill::is_area(&skills[sid(&s.w, "g_snare")]));
    assert!(!skill::is_placement(&skills[sid(&s.w, "g_spit")]) && !skill::is_area(&skills[sid(&s.w, "g_spit")]), "a hit that leaves a hazard is no placement");
    let mut w = s.w.clone();
    let hp: Vec<i32> = w.units.iter().map(|u| u.hp).collect();
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, at_tile(&w, HERO, "g_snare", v(6, 4)), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(w.units.iter().map(|u| u.hp).collect::<Vec<_>>(), hp, "no damage");
    assert_eq!(res.rng.randi(), Resolver::new(BLOCKS, 1).rng.randi(), "no random draw");
    assert_eq!(ids(&w), vec![(v(6, 4), "g_trip".to_string())]);
    let p = &w.grid.ground[&v(6, 4)];
    assert_eq!((p.by, p.round, p.last_round), (HERO, 1, 3), "rounds 3 from round 1: through round 3");
    assert_eq!(res.scheduled[a].command.log, "[Place] p_hero uses g_snare");
    assert!(res.log.iter().any(|l| l == "[Hazard] p_hero leaves Tripwire on (6, 4) for 3 rounds"), "{:?}", res.log);

    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    w.grid.positions[FOE] = Some(v(6, 4));
    book(&mut res, HERO, at_tile(&w, HERO, "g_snare", v(6, 4)), 1);
    run(&mut w, &mut res, 1, 1);
    assert!(w.grid.ground.is_empty(), "a trap needs a free tile");
}

/// GH4, GH5: the newest hazard on a tile replaces the old; a terrain hazard stays and acts first; smoke's cover 0.3 only
/// for ranged hits, the larger of terrain and hazard; a hazard's move cost adds to the terrain's.
#[test]
fn newest_replaces_and_terrain_acts_first() {
    let s = session(rules());
    let mut w = s.w.clone();
    w.grid.positions[FOE] = Some(v(9, 4));
    set_terrain(&mut w, v(7, 4), MUD);
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, at_tile(&w, HERO, "g_pot", v(6, 4)), 1);
    book(&mut res, HERO, at_tile(&w, HERO, "g_burn", v(6, 4)), 2);
    run(&mut w, &mut res, 1, 2);
    assert_eq!(w.grid.ground[&v(6, 4)].def.id, "g_fire", "the newest replaces");
    assert_eq!(w.grid.ground[&v(5, 4)].def.id, "g_grease");
    assert_eq!(w.grid.ground.len(), 9, "the burst's 9 tiles, one hazard each");
    assert_eq!((w.grid.move_cost(v(5, 4)), w.grid.move_cost(v(6, 4))), (2, 1), "grease adds 1, fire nothing");

    // the foe walks from (9, 4) to the greased mud at (7, 4): the mud's roll is the first draw, the grease's the second;
    // a seed where both land shows the order in the line too
    let (c_mud, c_grease) = (status::apply_chance(&w.units[FOE], "poison", 100), status::apply_chance(&w.units[FOE], "slow", 100));
    let both = |r: &Resolver| {
        let mut probe = r.rng.clone();
        (probe.randi_range(1, 100) <= c_mud, probe.randi_range(1, 100) <= c_grease)
    };
    let seed = (1..200).find(|&seed| both(&Resolver::new(BLOCKS, seed)) == (true, true)).expect("a seed where both land");
    for seed in [1, seed] {
        let mut w = w.clone();
        let mut res = Resolver::new(BLOCKS, seed);
        let (mud_lands, grease_lands) = both(&res);
        let mut probe = res.rng.clone();
        probe.randi_range(1, 100);
        probe.randi_range(1, 100);
        book(&mut res, FOE, walk(FOE, &[v(-1, 0), v(-1, 0)]), 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.pos(FOE), v(7, 4));
        assert_eq!((w.units[FOE].st.has_any("poison"), w.units[FOE].st.has_any("slow")), (mud_lands, grease_lands), "seed {seed}: terrain, then ground");
        assert_eq!(res.rng.randi(), probe.randi(), "seed {seed}: two draws");
        if mud_lands && grease_lands {
            let line = res.log.iter().find(|l| l.starts_with("[Move]") && l.contains("p_foe")).expect("the walk's line").clone();
            let (mud, grease) = (line.find("by Mud").expect(&line), line.find("enters Scalding Grease").expect(&line));
            assert!(mud < grease, "terrain first: {line}");
        }
    }

    let mut w = s.w.clone();
    lay(&mut w, "g_smoke", FOE, &[v(6, 4)]);
    assert_eq!(command::cover_for(&w.grid, v(9, 4), v(6, 4), true), 0.3, "ranged: smoke's cover");
    assert_eq!(command::cover_for(&w.grid, v(5, 4), v(6, 4), false), 0.0, "melee: none");
    set_terrain(&mut w, v(6, 4), RUBBLE);
    assert_eq!(w.grid.cover_guard(v(6, 4)), 0.4, "the larger of terrain and hazard");
}

/// GH5 (flat damage through guard, not armour) and GH6: a tripwire the placer's ally walks through stays; a foe walking
/// across stops on it, takes 15 (its armour ignored) and the root roll (one draw), the trap is gone and its later Walk is
/// cancelled. A push across a trap stops there with no collision (FM6).
#[test]
fn a_tripwire_spares_friends_and_stops_a_foe() {
    let s = session(rules());
    let mut w = s.w.clone();
    w.grid.positions[ALLY] = Some(v(6, 3));
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, at_tile(&w, HERO, "g_snare", v(6, 4)), 1);
    book(&mut res, ALLY, walk(ALLY, &[v(0, 1), v(0, 1)]), 2);
    let foe_walk = book(&mut res, FOE, walk(FOE, &[v(-1, 0), v(-1, 0), v(-1, 0)]), 3);
    let later = book(&mut res, FOE, walk(FOE, &[v(0, -1)]), 5);
    run(&mut w, &mut res, 1, 2);
    assert_eq!((w.pos(ALLY), w.units[ALLY].hp), (v(6, 5), HP), "the ally passes");
    assert!(w.grid.ground.contains_key(&v(6, 4)), "the trap stays");
    let mut probe = res.rng.clone();
    let roll = probe.randi_range(1, 100);
    let lands = roll <= status::apply_chance(&w.units[FOE], "root", 90);
    run(&mut w, &mut res, 3, 6);
    assert_eq!(w.pos(FOE), v(6, 4), "stopped on the trap");
    assert_eq!(w.units[FOE].hp, HP - 15, "15 flat, the armour ignored");
    assert_eq!(w.units[FOE].st.has_any("root"), lands, "roll {roll}");
    assert_eq!(res.rng.randi(), probe.randi(), "one draw: the root roll");
    assert!(w.grid.ground.is_empty(), "the trap is gone");
    let CmdKind::Walk { moved_tiles, cut_short, blocked_by, .. } = &res.scheduled[foe_walk].command.kind else { panic!() };
    assert_eq!((*moved_tiles, *cut_short, blocked_by.as_str()), (2, true, "the Tripwire"), "the rest of the path is dropped");
    assert!(res.scheduled[later].cancelled, "stop_moves cancels its later Walk");
    let hits: Vec<_> = w.ground.events.iter().filter_map(|e| if let Event::Hit(h) = e { Some(h) } else { None }).collect();
    assert_eq!(hits.len(), 1);
    assert_eq!((hits[0].unit, hits[0].by, hits[0].damage, hits[0].trap, hits[0].stopped), (FOE, HERO, 15, true, 1));

    let mut w = s.w.clone();
    w.grid.positions[FOE] = Some(v(5, 4));
    lay(&mut w, "g_trip", HERO, &[v(7, 4)]);
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&w, HERO, FOE, "g_shove"), 1);
    run(&mut w, &mut res, 1, 1);
    let m = &res.scheduled[a].command.pushes[0];
    assert_eq!((w.pos(FOE), m.tiles, m.blocked_by.as_str(), m.hits.len()), (v(7, 4), 2, "", 0), "the push stops on the trap, no collision");
    assert!(m.line.contains("[Hazard] p_foe springs the Tripwire: 15 damage"), "{}", m.line);
}

/// GH7: `rounds` 2 placed in round 3 lasts through round 4 and fades after its round-end rolls; a Defend places at its
/// start, around itself.
#[test]
fn hazards_last_their_rounds() {
    let s = session(rules());
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    res.current_round = 3;
    book(&mut res, HERO, command(&w, HERO, PlanStep::self_skill(sid(&w, "g_fade")), "g_fade"), 1);
    res.prepare_block(&mut w, 1);
    assert_eq!(w.grid.ground.len(), 9, "placed at the Defend's start, before it lands");
    assert!(w.grid.ground.values().all(|p| p.def.id == "g_smoke" && p.last_round == 4));
    res.apply_round_end_effects(&mut w);
    assert_eq!(w.grid.ground.len(), 9, "there through round 3");
    res.finish_round(&mut w);
    res.apply_round_end_effects(&mut w);
    assert!(w.grid.ground.is_empty(), "gone after round 4");
    assert_eq!(res.log.iter().filter(|l| *l == "[Hazard] Soot Smoke fades").count(), 1, "{:?}", res.log);
    assert!(matches!(w.ground.events.last(), Some(Event::Faded(f)) if f.cells == 9));
}

/// GH2: a consumable leaves its hazard at use (around its user here).
#[test]
fn an_item_places_at_use() {
    let s = session(rules());
    let mut w = s.w.clone();
    let item = flask();
    w.units[HERO].inventory.pouch.push(item.clone());
    let mut cmd = Command::new(HERO, 0, 1, CmdKind::Item { target: HERO, item: Box::new(item.clone()), used: false, last_heal: 0, last_damage: 0, last_boost: 0.0 });
    cmd.name = item.name.clone();
    cmd.hazard = Place::of_item(&item);
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, cmd, 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(w.grid.ground.len(), 9);
    assert!(w.grid.ground.contains_key(&v(4, 4)) && w.grid.ground.contains_key(&v(5, 5)));
}

/// GH8: the AI's path penalty counts a ground hazard that would harm the unit and that its side sees: not a hidden trap
/// (unseen by the foes, harmless to the placer's side), a seen one and grease yes, smoke no; with the rule off none.
/// A foe's route crosses a hidden trap and bends around a seen one.
#[test]
fn the_ai_penalty_counts_visible_harm() {
    let s = session(rules());
    let mut w = s.w.clone();
    lay(&mut w, "g_trip", HERO, &[v(6, 4)]);
    lay(&mut w, "g_trip_seen", HERO, &[v(6, 6)]);
    lay(&mut w, "g_grease", HERO, &[v(6, 2)]);
    lay(&mut w, "g_smoke", HERO, &[v(6, 0)]);
    let pen = |w: &World, c: UnitId, cell: V2| ground::ai_penalty(w, c, cell);
    assert_eq!((pen(&w, FOE, v(6, 4)), pen(&w, ALLY, v(6, 4))), (0, 0), "hidden from the foes, harmless to the party");
    assert_eq!((pen(&w, FOE, v(6, 6)), pen(&w, ALLY, v(6, 6))), (AI_HAZARD_PENALTY, 0), "a seen trap: foes only");
    assert_eq!((pen(&w, FOE, v(6, 2)), pen(&w, ALLY, v(6, 2))), (AI_HAZARD_PENALTY, AI_HAZARD_PENALTY), "grease harms all");
    assert_eq!(pen(&w, FOE, v(6, 0)), 0, "smoke harms nobody");
    assert!(ground::seen_at(&w, FOE, v(6, 4)).is_none() && ground::seen_at(&w, ALLY, v(6, 4)).is_some(), "hidden from the other side");

    let route = |w: &World| {
        let goals: Cells = [v(5, 4)].into_iter().collect();
        let penalty = |cell: V2| ground::ai_penalty(w, FOE, cell);
        let steps = path::find_path(&w.grid, v(8, 4), &goals, &Cells::new(), false, &Hooks { penalty: Some(&penalty), prefer: None });
        steps.iter().scan(v(8, 4), |at, d| {
            *at = *at + *d;
            Some(*at)
        }).collect::<Vec<V2>>()
    };
    assert!(route(&w).contains(&v(6, 4)), "straight across the hidden trap");
    let mut seen = s.w.clone();
    lay(&mut seen, "g_trip_seen", HERO, &[v(6, 4)]);
    assert!(!route(&seen).contains(&v(6, 4)), "around the seen one");

    let mut off = session(timeline(&[])).w.clone();
    lay(&mut off, "g_grease", HERO, &[v(6, 2)]);
    assert_eq!(pen(&off, FOE, v(6, 2)), 0, "rule off: no ground penalty");
}

/// GH5, 4.2: in a session each placement and hazard hit gets a cell: the trap a foe springs mid-round is an ATTACK cell
/// of the placer for the HP it took; the fire at round end an EVENT cell (round-end damage, credited to the placer as
/// round-end damage dealt). The metrics balance and count the placements, damage and traps under `hazards`.
#[test]
fn the_session_logs_and_counts_hazards() {
    let mut s = session(rules());
    s.resolver.scheduled.clear();
    s.w.grid.positions[FOE] = Some(v(5, 4));
    s.w.grid.positions[FOE_B] = Some(v(8, 5));
    s.attach_metrics(BattleMetrics::new(), true);
    let w = &s.w;
    assert_eq!(s.plan.add_step(w, HERO, PlanStep::area_at(sid(w, "g_snare"), v(6, 5)), true), "");
    assert_eq!(s.plan.add_step(w, HERO, PlanStep::engage(sid(w, "g_spit"), FOE), true), "");
    assert!(s.resolver.schedule(FOE_B, walk(FOE_B, &[v(-1, 0), v(-1, 0), v(-1, 0)]), 3));
    while s.current_round == 1 && !s.combat_over && s.phase != Phase::Replay {
        s.step_next();
    }
    assert_eq!(s.w.pos(FOE_B), v(6, 5), "caught by the trap");
    let placed: Vec<&str> = s.log.iter().filter(|e| e.outcome == "PLACED" && e.category == "EVENT").map(|e| e.skill.as_str()).collect();
    assert_eq!(placed, vec!["Tripwire", "Phosphorus Fire"]);
    assert!(s.log.iter().any(|e| e.subtype == "Placement" && e.skill == "g_snare"), "the placement's own cell");
    let trap: Vec<_> = s.log.iter().filter(|e| e.subtype == "Hazard" && e.category == "ATTACK").collect();
    assert_eq!(trap.len(), 1);
    assert_eq!((trap[0].actor.as_str(), trap[0].target.as_str(), trap[0].damage, trap[0].outcome.as_str()), ("p_hero", "p_foe_b", 15, "TRAPPED"));
    let fire: Vec<_> = s.log.iter().filter(|e| e.subtype == "Hazard" && e.category == "EVENT" && e.outcome == "HAZARD").collect();
    assert_eq!(fire.len(), 1);
    assert_eq!((fire[0].target.as_str(), fire[0].value), ("p_foe", 8), "fire at round end: 8 through no guard");
    let m = s.metrics.as_ref().unwrap();
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    let h = &m.hazards["p_hero"];
    assert_eq!((h.placed, h.tiles, h.damage, h.traps), (2, 2, 15 + 8, 1));
    assert_eq!((m.hazards["p_foe_b"].trapped, m.hazards["p_foe_b"].taken, m.hazards["p_foe"].taken), (1, 15, 8));
    let dealt: i32 = s.log.iter().filter(|e| e.category == "ATTACK" && e.actor == "p_hero").map(|e| e.damage).sum();
    assert_eq!((m.units["p_hero"].dmg_dealt, m.units["p_hero"].dot_dealt), (dealt, 8), "the trap as a hit, the fire as round-end damage");
    assert_eq!(m.to_json()["hazards"]["p_hero"]["placed"].as_i64(), Some(2));
}

/// 4.1, 4.2: with the rule off and in classic the fields are ignored. The battle's catalog drops the skills' hazards
/// (HOOKS1-GF), so the snare is no placement: it aims at a unit as any zero-power attack does and places nothing. A
/// command that still carries a hazard (built from the raw catalog) places nothing either: no hazard line, the same
/// random draws as the skills without them.
#[test]
fn off_and_classic_change_nothing() {
    let raw = catalog();
    let on = session(rules());
    assert!(skill::is_placement(&on.w.cat.skills[sid(&on.w, "g_snare")]), "the rule on: a placement");
    for rules in [timeline(&[FORCED_MOVE]), Ruleset::classic()] {
        let s = session(rules.clone());
        assert!(s.w.cat.skills.iter().all(|k| k.hazard.is_none()), "{}: no skill hazard in the battle's catalog", rules.id);
        let snare = &s.w.cat.skills[sid(&s.w, "g_snare")];
        assert!(!skill::is_placement(snare) && !skill::is_area(snare), "{}: the snare is a plain attack", rules.id);
        let mut w = s.w.clone();
        w.grid.positions[FOE] = Some(v(7, 4));
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, hit(&w, HERO, FOE, "g_snare"), 1);
        let mut spit = hit(&w, HERO, FOE, "g_spit");
        spit.hazard = Place::of_skill(raw.skill("g_spit").expect("g_spit"));
        book(&mut res, HERO, spit, 2);
        let mut fade = command(&w, HERO, PlanStep::self_skill(sid(&w, "g_fade")), "g_fade");
        fade.hazard = Place::of_skill(raw.skill("g_fade").expect("g_fade"));
        book(&mut res, HERO, fade, 3);
        run(&mut w, &mut res, 1, 3);
        res.apply_round_end_effects(&mut w);
        let mut plain_w = s.w.clone();
        plain_w.grid.positions[FOE] = Some(v(7, 4));
        let mut plain = Resolver::new(BLOCKS, 1);
        book(&mut plain, HERO, hit(&plain_w, HERO, FOE, "g_snare"), 1);
        book(&mut plain, HERO, hit(&plain_w, HERO, FOE, "g_plain"), 2);
        run(&mut plain_w, &mut plain, 1, 3);
        plain.apply_round_end_effects(&mut plain_w);
        let snared = &res.scheduled[a].command;
        assert!(matches!(snared.kind, CmdKind::Attack { target: Some(FOE), .. }) && snared.hazard.is_none(), "{}: the snare targets the foe, carries nothing", rules.id);
        assert!(w.grid.ground.is_empty() && w.ground.events.is_empty(), "{}: nothing placed", rules.id);
        assert!(res.log.iter().all(|l| !l.starts_with("[Hazard]") && !l.starts_with("[Place]")), "{}: {:?}", rules.id, res.log);
        assert_eq!(res.rng.randi(), plain.rng.randi(), "{}: no new draw", rules.id);
        assert_eq!(w.units[FOE].hp, plain_w.units[FOE].hp, "{}: the snare's and the spit's hits only", rules.id);
    }
}

/// The hazard hits recorded on `u`.
fn hits_on(w: &World, u: UnitId) -> Vec<&Hit> {
    w.ground.events.iter().filter_map(|e| if let Event::Hit(h) = e { Some(h) } else { None }).filter(|h| h.unit == u).collect()
}

/// A hidden tripwire the foe laid on every tile of column x = 6 (any route east from x = 4 crosses it).
fn trip_column(w: &mut World) -> Vec<V2> {
    let column: Vec<V2> = (0..GRID_H).map(|y| v(6, y)).collect();
    lay(w, "g_trip", FOE, &column);
    column
}

/// GH6 on a dash (HOOKS1-GF): the hero's dash attack (reach 1, dash 5) on the foe 4 tiles east stops on the first trap
/// it enters: the trap's 15 flat damage, the trap gone, the rest of the dash dropped (the foe stays out of reach, so
/// the attack lands nothing) and `stop_moves` cancels the hero's later Walk.
#[test]
fn a_trap_stops_a_dash() {
    let s = session(rules());
    let mut w = s.w.clone();
    let column = trip_column(&mut w);
    let foe_hp = w.units[FOE].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "g_dash"), 1);
    let later = book(&mut res, HERO, walk(HERO, &[v(0, 1)]), 3);
    run(&mut w, &mut res, 1, 1);
    let at = w.pos(HERO);
    assert_eq!(at.x, 6, "stopped on the first trap: {at}");
    assert_eq!(w.units[HERO].hp, HP - 15, "the trap's flat damage");
    assert!(!w.grid.ground.contains_key(&at) && w.grid.ground.len() == column.len() - 1, "that trap is gone");
    assert_eq!(w.units[FOE].hp, foe_hp, "the foe out of reach: no hit");
    let h = hits_on(&w, HERO);
    assert_eq!((h.len(), h[0].trap, h[0].stopped), (1, true, 1));
    assert!(res.scheduled[later].cancelled, "stop_moves cancels its later Walk");
    assert!(res.log.iter().any(|l| l.starts_with("[Dash]") && l.contains("springs the Tripwire")), "{:?}", res.log);
}

/// GH6 on a carried move (RULESET1_INTENT MV4, HOOKS1-GF): under `round_intent` the hero's Maneuver lunge on the foe
/// carries its move (an Engage tile in band); the trap across its route stops it there, as a dash: 15 damage, the trap
/// gone, the planned tile not reached and no hit.
#[test]
fn a_trap_stops_a_carried_move() {
    let s = session(timeline(&[GROUND_HAZARDS, ROUND_INTENT]));
    let mut w = s.w.clone();
    round_intent::set_intent(&mut w, HERO, RoundIntent::Maneuver).expect("intent");
    round_intent::set_target(&mut w, HERO, RoundTarget::Unit(FOE)).expect("target");
    let cmd = hit(&w, HERO, FOE, "g_lunge");
    let Carry::Engage(planned) = cmd.carry else { panic!("a carried move: {:?}", cmd.carry) };
    assert!(planned.x > 6, "the planned tile is past the column: {planned}");
    let column = trip_column(&mut w);
    let foe_hp = w.units[FOE].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, cmd, 1);
    run(&mut w, &mut res, 1, 1);
    let at = w.pos(HERO);
    assert_eq!(at.x, 6, "stopped on the first trap, not at {planned}: {at}");
    assert_eq!(w.units[HERO].hp, HP - 15);
    assert!(!w.grid.ground.contains_key(&at) && w.grid.ground.len() == column.len() - 1, "that trap is gone");
    assert_eq!(w.units[FOE].hp, foe_hp, "the foe out of reach: no hit");
    assert_eq!(hits_on(&w, HERO).len(), 1);
}

/// GH5 (HOOKS1-GF): hazard damage goes through guard, not armour (`Combatant.take_damage`: the amount times
/// (1 - guard), rounded, at least 1). The armoured foe on fire at round end takes the fire's full damage; guarding with
/// the `guard` Defend through the last block it takes the reduced amount.
#[test]
fn guard_reduces_hazard_damage() {
    let s = session(rules());
    let mitigation = s.w.cat.skills[sid(&s.w, "guard")].mitigation;
    let fire = s.w.cat.hazards["g_fire"].damage;
    let reduced = ((fire as f64 * (1.0 - mitigation)).round() as i32).max(1);
    assert!(reduced < fire);
    for (guarded, want) in [(false, fire), (true, reduced)] {
        let mut w = s.w.clone();
        let foe_at = w.pos(FOE);
        lay(&mut w, "g_fire", HERO, &[foe_at]);
        let mut res = Resolver::new(BLOCKS, 1);
        res.register(FOE);
        if guarded {
            book(&mut res, FOE, command(&w, FOE, PlanStep::self_skill(sid(&w, "guard")), "guard"), BLOCKS - 1);
        }
        run(&mut w, &mut res, 1, BLOCKS);
        assert_eq!(w.units[FOE].guard > 0.0, guarded, "guarding through the last block");
        res.apply_round_end_effects(&mut w);
        let h = hits_on(&w, FOE);
        assert_eq!(h.len(), 1);
        assert_eq!((h[0].round_end, h[0].damage), (true, want), "guarded {guarded}");
    }
}
