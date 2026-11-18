//! BIGUNIT1 (docs/specs/BIGUNIT1.md section 5) on a scratch catalog built here (no shipped data): units with
//! `footprint: 2` fill the 2 x 2 square anchored at their position. Expectations come from the rules: every cell of the
//! square is occupied (BU4); distance is the least distance to any cell, under either metric (BU5); a step needs the
//! whole destination square free and costs the dearest cell it enters, and a big unit never leaves by an exit (BU6);
//! the cells newly covered roll their hazards (BU7); a forced move needs the whole square to fit and a 2-wide edge can
//! meet two blockers, each hit (BU8); an area hits a big unit once (BU9); cover is the least over its cells (BU10); its
//! front is the side of the square it faces (BU11); a swap into a bigger unit takes a square holding the old cell, in
//! the fixed order, else the nearest, else makes room by pushing units back (BU12); the AI never overlaps it (BU13).
//! One-cell units keep their old answers (the goldens check the rest).

use indexmap::IndexMap;
use rpgkit_battle::command::{self, cover_for, CmdKind, Command};
use rpgkit_battle::forced::{self, Collide};
use rpgkit_battle::grid::{square_cells, V2, NO_CELL};
use rpgkit_battle::path::{self, Cells, Hooks};
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{StepTo, UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, FORCED_MOVE, ROUND_INTENT, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const BIG: UnitId = 2;
const FOE: UnitId = 3;
const FOE_B: UnitId = 4;

/// The terrain index of `wall` and `mud` in `grid::TERRAIN_IDS`.
const WALL: usize = 10;
const MUD: usize = 8;

/// The big unit's anchor in the fixtures: it fills (5, 4), (6, 4), (5, 5) and (6, 5).
const AT: V2 = V2 { x: 5, y: 4 };

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "b_burst", "name": "Burst", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.2, "target_mode": "ENEMY", "area": "PLUS", "reach_min": 1, "reach_max": 4}),
    ];
    let unit = |id: &str, name: &str, side: &str, extra: Value| {
        let mut row = json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": 12, "attack_range": 1, "skill_cap": 5, "ai_role": "AGGRESSOR"});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [
            unit("b_hero", "Hero", "party", json!({})),
            unit("b_ally", "Ally", "party", json!({})),
            unit("b_giant", "Giant", "enemy", json!({"footprint": 2})),
            unit("b_foe", "Foe", "enemy", json!({})),
            unit("b_foe_b", "FoeB", "enemy", json!({})),
            unit("b_boss", "Boss", "enemy", json!({})),
        ],
        "terrain": [{"id": "mud", "name": "Mud", "hazard_status": "slow", "hazard_potency": 100, "hazard_on": "enter"}]
    }))
}

fn floor() -> Vec<String> {
    vec![".".repeat(12); 9]
}

/// A battle on open floor (one exit at (11, 4)): hero (2, 4), ally (0, 8), the giant at AT, foe (11, 0), foe_b (11, 8).
fn session(ruleset: Option<Ruleset>) -> Session {
    let row = json!({"id": "b_fight", "name": "Fight", "rows": floor(), "party": ["b_hero", "b_ally"], "enemies": ["b_giant", "b_foe", "b_foe_b"],
        "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    if let Some(r) = ruleset {
        def.ruleset = r;
    }
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(2, 4)), (ALLY, V2::new(0, 8)), (BIG, AT), (FOE, V2::new(11, 0)), (FOE_B, V2::new(11, 8))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn set_terrain(w: &mut World, cell: V2, t: usize) {
    w.grid.terrain[(cell.y * rpgkit_battle::grid::GRID_W + cell.x) as usize] = t;
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

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

/// No two active units share a cell and every big unit's square fits the board (inside, walkable, not an exit).
fn assert_no_overlap(w: &World, when: &str) {
    let mut seen: Vec<(V2, UnitId)> = Vec::new();
    for u in (0..w.units.len()).filter(|&u| w.has(u)) {
        for cell in w.cells_of(u) {
            assert!(w.grid.is_walkable(cell), "{when}: unit {u} on {cell}, not walkable");
            if let Some((_, o)) = seen.iter().find(|(c, _)| *c == cell) {
                panic!("{when}: units {o} and {u} share {cell}");
            }
            seen.push((cell, u));
        }
        if w.size_of(u) > 1 {
            assert!(w.grid.square_fits(w.pos(u), w.size_of(u)), "{when}: unit {u}'s square at {} does not fit", w.pos(u));
        }
    }
}

/// BU1, BU2, BU4: the row's footprint reaches the unit; the anchor is the position; all four cells answer the occupant.
#[test]
fn a_big_unit_occupies_its_four_cells() {
    let s = session(None);
    let w = &s.w;
    assert_eq!((w.units[BIG].footprint, w.size_of(BIG), w.size_of(HERO)), (2, 2, 1), "footprint from the row; others 1");
    assert_eq!(w.pos(BIG), AT, "the position is the anchor (top-left)");
    let cells = vec![AT, AT + V2::new(1, 0), AT + V2::new(0, 1), AT + V2::new(1, 1)];
    assert_eq!(w.cells_of(BIG), cells, "reading order");
    for c in &cells {
        assert_eq!(w.occupant(*c), Some(BIG), "{c}");
        assert!(w.is_occupied(*c));
    }
    for c in [AT + V2::new(2, 0), AT + V2::new(0, 2), AT - V2::new(1, 0)] {
        assert_eq!(w.occupant(c), None, "{c} is outside the square");
    }
    assert_eq!(w.cells_of(HERO), vec![V2::new(2, 4)], "a one-cell unit fills its tile only");
}

/// BU5: the least distance to any of the four cells, Manhattan and under diagonal melee; reach counts both ways.
#[test]
fn distance_is_to_the_nearest_cell() {
    let mut s = session(None);
    let w = &mut s.w;
    // hero (2, 4): nearest cell (5, 4), 3 away; from (8, 6) the nearest is (6, 5): 2 + 1
    assert_eq!((w.dist(HERO, BIG), w.dist(BIG, HERO)), (3, 3));
    assert_eq!(w.dist_to(BIG, V2::new(8, 6)), 3);
    assert_eq!(w.dist_to(BIG, V2::new(6, 6)), 1, "below (6, 5)");
    assert_eq!(w.dist_to(BIG, V2::new(6, 5)), 0, "a cell of the square");
    // a big unit standing elsewhere (a planned anchor) measures from its square there
    assert_eq!(w.dist_at(BIG, V2::new(0, 4), HERO), 1, "anchored at (0, 4) its cell (1, 4) touches the hero");
    // diagonal: (4, 3) is diagonal to (5, 4): Manhattan 2, the band 1 under diagonal melee
    w.grid.positions[HERO] = Some(V2::new(4, 3));
    assert_eq!((w.dist(HERO, BIG), w.band_at(false, HERO, w.pos(HERO), BIG), w.band_at(true, HERO, w.pos(HERO), BIG)), (2, 2, 1));
    // the one-cell answers stay V2::dist and band_dist
    assert_eq!(w.dist(HERO, ALLY), w.pos(HERO).dist(w.pos(ALLY)));
    assert_eq!(w.band_at(true, HERO, w.pos(HERO), ALLY), round_intent::band_dist(true, w.pos(HERO), w.pos(ALLY)));
}

/// BU6: a step needs the whole destination square on the board, walkable, not an exit and free; a big unit never leaves
/// by an exit; the route moves the anchor around blockers; a step costs the dearest cell it newly covers.
#[test]
fn a_big_unit_moves_only_where_its_square_fits() {
    let mut s = session(None);
    let w = &mut s.w;
    let east = V2::new(1, 0);
    assert_eq!(w.step_from(BIG, AT, east), StepTo::Enters(AT + east), "free floor");
    set_terrain(w, V2::new(7, 5), WALL);
    assert_eq!(w.step_from(BIG, AT, east), StepTo::Blocked, "one entered cell is a wall");
    set_terrain(w, V2::new(7, 5), 0);
    w.grid.positions[FOE] = Some(V2::new(7, 4));
    assert_eq!(w.step_from(BIG, AT, east), StepTo::Blocked, "a unit on one entered cell");
    assert!(!w.move_unit(BIG, east), "move_unit refuses it too");
    assert_eq!(w.pos(BIG), AT);
    w.grid.positions[FOE] = Some(V2::new(11, 0));
    // the bottom edge: anchored on row 7 a step south leaves the board
    w.grid.positions[BIG] = Some(V2::new(5, 7));
    assert_eq!(w.step_from(BIG, V2::new(5, 7), V2::new(0, 1)), StepTo::Blocked, "off the board");
    // an exit never takes a big unit: anchored at (9, 4), its east step covers the exit (11, 4)
    w.grid.positions[BIG] = Some(V2::new(9, 4));
    assert!(w.grid.is_exit(V2::new(11, 4)), "fixture exit");
    assert_eq!(w.step_from(BIG, V2::new(9, 4), east), StepTo::Blocked, "a big unit cannot leave by an exit");
    assert!(!w.units[BIG].is_withdrawn);
    w.grid.positions[BIG] = Some(AT);

    // a route around a unit: the hero at (8, 4) blocks the straight line east along rows 4-5
    w.grid.positions[HERO] = Some(V2::new(8, 4));
    let blocked: Cells = [V2::new(8, 4)].into_iter().collect();
    let goals: Cells = [V2::new(9, 4)].into_iter().collect();
    let route = path::find_path_sized(&w.grid, AT, &goals, &blocked, false, &Hooks::default(), 2);
    assert!(!route.is_empty(), "a route exists");
    let mut at = AT;
    for d in &route {
        at = at + *d;
        assert!(square_cells(at, 2).all(|c| !blocked.contains(&c) && w.grid.is_walkable(c)), "every square on the route is clear ({at})");
    }
    assert_eq!(at, V2::new(9, 4));
    // the one-cell route is the old find_path
    let one = path::find_path(&w.grid, V2::new(2, 2), &goals, &blocked, false, &Hooks::default());
    assert_eq!(one, path::find_path_sized(&w.grid, V2::new(2, 2), &goals, &blocked, false, &Hooks::default(), 1));

    // cost: mud under one of the two entered cells makes the step cost mud's move cost
    set_terrain(w, V2::new(7, 5), MUD);
    let mud = w.grid.move_cost(V2::new(7, 5));
    assert!(mud > 1, "fixture: mud costs more than floor");
    assert_eq!(path::square_step_cost(&w.grid, AT, AT + east, 2), mud, "the dearest cell entered");
    assert_eq!(path::square_step_cost(&w.grid, AT, AT + V2::new(0, -1), 2), 1, "north: floor only");
    assert_eq!(path::path_cost_sized(&w.grid, AT, &[east], 2), mud);
}

/// BU7: a step rolls the cells it newly covers (two for an orthogonal step), each hazard status once.
#[test]
fn a_big_unit_rolls_the_cells_it_newly_covers() {
    let s = session(None);
    let w = &s.w;
    assert_eq!(command::newly_covered(w, BIG, AT, AT + V2::new(1, 0)), vec![V2::new(7, 4), V2::new(7, 5)]);
    assert_eq!(command::newly_covered(w, BIG, AT, AT + V2::new(0, -1)), vec![V2::new(5, 3), V2::new(6, 3)]);
    assert_eq!(command::newly_covered(w, HERO, V2::new(2, 4), V2::new(3, 4)), vec![V2::new(3, 4)], "one-cell: the tile entered");
    // a walk east: mud on the newly covered (7, 5) is one extra draw; mud on (5, 5) (covered before and after) none
    let walk = |mud: Option<V2>| {
        let mut w = s.w.clone();
        if let Some(c) = mud {
            set_terrain(&mut w, c, MUD);
        }
        let mut res = Resolver::new(6, 1);
        let mut cmd = Command::new(BIG, 0, 1, CmdKind::Walk { path: vec![V2::new(1, 0)], expected_start: NO_CELL, goal: NO_CELL, approach: None, moved_tiles: 0, cut_short: false, blocked_by: String::new() });
        cmd.name = "Move".into();
        assert!(res.schedule(BIG, cmd, 1));
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.pos(BIG), AT + V2::new(1, 0), "it walked");
        (w, res)
    };
    let (_, mut floor) = walk(None);
    let (_, mut kept) = walk(Some(V2::new(5, 5)));
    let (w, mut entered) = walk(Some(V2::new(7, 5)));
    assert_eq!(kept.rng.randi(), floor.rng.randi(), "no roll for a cell it already covered");
    let mut floor2 = walk(None).1;
    let roll = floor2.rng.randi_range(1, 100);
    let lands = roll <= rpgkit_rules::status::apply_chance(&s.w.units[BIG], "slow", 100);
    assert_eq!(entered.rng.randi(), floor2.rng.randi(), "one roll for the newly covered mud");
    assert_eq!(w.units[BIG].st.has_any("slow"), lands, "roll {roll}");
}

/// BU8: a forced move moves the square only where it fits; the 2-wide edge meeting two units hits both (reading order).
#[test]
fn a_pushed_big_unit_collides_with_both_blockers() {
    let mut s = session(Some(Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [FORCED_MOVE]}).as_object().unwrap())));
    let w = &mut s.w;
    let collide = Collide { attacker: None, mult: 0.0, ranged: false, offhand: false, flat: 10, stagger: 0, sure_break: false, blocks: 1 };
    // free: a push 2 east moves the anchor 2 tiles
    let (steps, refused) = forced::steps_all(w, BIG, V2::new(1, 0), 2, None);
    assert_eq!((steps, refused), (vec![V2::new(6, 4), V2::new(7, 4)], Vec::new()));
    // two blockers on the entered column (8, 4) and (8, 5): the first step fits, the second meets both
    w.grid.positions[FOE] = Some(V2::new(8, 5));
    w.grid.positions[FOE_B] = Some(V2::new(8, 4));
    let mut res = Resolver::new(6, 1);
    let m = forced::shove_unit(&mut res, w, BIG, Some(HERO), V2::new(1, 0), 3, None, false, &collide, 1);
    assert_eq!((m.from, m.to, m.tiles), (AT, V2::new(6, 4), 1), "one step, then the collision");
    assert_eq!(m.blockers, vec![FOE_B, FOE], "both blockers, reading order ((8, 4) before (8, 5))");
    assert_eq!(m.blocker, Some(FOE_B), "the first stays the old field");
    let hit: Vec<UnitId> = m.hits.iter().map(|h| h.unit).collect();
    assert_eq!(hit, vec![BIG, FOE_B, FOE], "the moved unit, then each blocker");
    assert!(m.hits.iter().all(|h| h.damage == 10), "the flat hit on each");
    // a wall under one cell of the next column stops it the same way, with no blocker
    w.grid.positions[FOE] = Some(V2::new(11, 0));
    w.grid.positions[FOE_B] = Some(V2::new(11, 8));
    set_terrain(w, V2::new(8, 5), WALL);
    let (steps, refused) = forced::steps_all(w, BIG, V2::new(1, 0), 3, None);
    assert_eq!((steps, refused), (Vec::new(), vec![V2::new(8, 5)]), "anchored at (6, 4): the next square holds the wall (8, 5)");
}

/// BU9: an area covering three of the big unit's cells hits it once.
#[test]
fn an_area_hits_a_big_unit_once() {
    let s = session(None);
    let w = &s.w;
    // PLUS around (6, 4) covers (6, 4), (5, 4), (7, 4), (6, 3), (6, 5): three of the giant's cells
    let cells = vec![V2::new(6, 4), V2::new(5, 4), V2::new(7, 4), V2::new(6, 3), V2::new(6, 5)];
    assert_eq!(w.occupants_once(&cells), vec![(V2::new(6, 4), BIG)], "once, at its first cell in the list");
    let mut w = s.w.clone();
    w.grid.positions[HERO] = Some(V2::new(6, 1));
    let mut res = Resolver::new(6, 1);
    let cmd = command(&w, HERO, PlanStep::area_at(sid(&w, "b_burst"), V2::new(6, 4)), "b_burst");
    assert!(res.schedule(HERO, cmd, 1));
    run(&mut w, &mut res, 1, 1);
    let struck: Vec<UnitId> = res.scheduled[0].command.struck.iter().map(|s| s.target).collect();
    assert_eq!(struck, vec![BIG], "one hit on the giant: {:?}", res.log);
}

/// BU10: cover is `cover_for` on each of the four cells and the least applies.
#[test]
fn cover_is_the_least_over_the_cells() {
    let mut s = session(None);
    let w = &mut s.w;
    w.grid.positions[HERO] = Some(V2::new(5, 0));
    set_terrain(w, V2::new(5, 3), WALL);
    let per_cell: Vec<f64> = w.cells_of(BIG).into_iter().map(|c| cover_for(&w.grid, w.nearest_cell(HERO, c), c, true)).collect();
    let least = per_cell.iter().copied().fold(f64::INFINITY, f64::min);
    assert_eq!(command::cover_between(w, HERO, BIG, true), least);
    assert_eq!(command::cover_between(w, HERO, BIG, false), 0.0, "melee: no cover");
    // the anchor alone (behind the wall) would read the wall's cover
    assert_eq!(per_cell[0], cover_for(&w.grid, V2::new(5, 0), AT, true));
    assert!(per_cell[0] >= least);
    // one-cell units: cover_for between their tiles
    assert_eq!(command::cover_between(w, HERO, ALLY, true), cover_for(&w.grid, w.pos(HERO), w.pos(ALLY), true));
}

/// BU11: the big unit's front is the side of its square it faces, measured from its cell nearest the attacker; a unit
/// beside or behind the square flanks it while it is engaged with another.
#[test]
fn a_big_unit_is_flanked_beside_its_square() {
    let ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT]}).as_object().unwrap());
    let mut s = session(Some(ruleset));
    let w = &mut s.w;
    w.grid.positions[HERO] = Some(V2::new(4, 4));
    let st = w.round_states.entry(BIG).or_default();
    st.target = Some(RoundTarget::Unit(HERO));
    st.facing = V2::new(-1, 0);
    assert_eq!(round_intent::engaged_with(w, BIG), Some(HERO), "the hero touches its west side");
    assert!(round_intent::in_front(w, BIG, V2::new(4, 5)), "west of (5, 5): the face");
    assert!(!round_intent::in_front(w, BIG, V2::new(7, 4)), "east of (6, 4): behind");
    assert!(!round_intent::in_front(w, BIG, V2::new(6, 3)), "north of (6, 4): beside");
    w.grid.positions[ALLY] = Some(V2::new(7, 5));
    assert!(round_intent::is_flank(w, ALLY, BIG), "behind the square");
    w.grid.positions[ALLY] = Some(V2::new(4, 5));
    assert!(!round_intent::is_flank(w, ALLY, BIG), "on its face");
    // facing re-read from the nearest cell: the hero west of the square gives (-1, 0)
    w.round_states.entry(BIG).or_default().facing = V2::new(0, 0);
    round_intent::update_facing(w, BIG);
    assert_eq!(round_intent::facing(w, BIG), V2::new(-1, 0));
}

/// A round-2 swap of `b_boss` into `b_giant` with the boss at `boss` and the other units at `cells` (`rows` the board).
fn swap_session(rows: Vec<String>, boss: V2, cells: &[(UnitId, V2)]) -> Session {
    swap_session_with(rows, json!(["b_foe", "b_foe_b", "b_boss"]), boss, cells)
}

/// `swap_session` with the enemies `enemies` (three refs, the boss last).
fn swap_session_with(rows: Vec<String>, enemies: Value, boss: V2, cells: &[(UnitId, V2)]) -> Session {
    let row = json!({"id": "b_swap", "name": "Swap", "rows": rows, "party": ["b_hero", "b_ally"], "enemies": enemies,
        "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 1337,
        "phases": [{"id": "grow", "name": "Grow", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "swap", "unit": "b_boss", "to": "b_giant"}]}]});
    let def = EncounterDef::from_obj(row.as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    place_all(&mut s, boss, cells);
    s
}

const SWAP_BOSS: UnitId = 4;

fn place_all(s: &mut Session, boss: V2, cells: &[(UnitId, V2)]) {
    for &(c, cell) in cells {
        s.w.grid.positions[c] = Some(cell);
    }
    s.w.grid.positions[SWAP_BOSS] = Some(boss);
}

/// Round 1 resolves with every unit held in place (guards), then round 2 starts and the swap fires.
fn grow(s: &mut Session, boss: V2, cells: &[(UnitId, V2)]) {
    assert!(s.confirm_plan());
    while s.phase != Phase::Replay && !s.combat_over {
        s.step_next();
    }
    place_all(s, boss, cells);
    s.advance_round();
}

/// BU12: the old cell as top-left first; when that square is taken, top-right, then bottom-left, then bottom-right.
#[test]
fn a_swap_into_a_big_unit_tries_the_four_squares_in_order() {
    // the boss at (8, 4); the other units far away unless a case puts them on a square
    let far = [(0, V2::new(0, 0)), (1, V2::new(0, 8)), (2, V2::new(11, 0))];
    let mut s = swap_session(floor(), V2::new(8, 4), &[far[0], far[1], far[2], (3, V2::new(11, 8))]);
    grow(&mut s, V2::new(8, 4), &[far[0], far[1], far[2], (3, V2::new(11, 8))]);
    assert_eq!((s.w.size_of(SWAP_BOSS), s.w.pos(SWAP_BOSS)), (2, V2::new(8, 4)), "top-left: free");
    // a unit on (9, 5) takes the top-left square; top-right (anchor (7, 4)) is free
    let cells = [far[0], far[1], far[2], (3, V2::new(9, 5))];
    let mut s = swap_session(floor(), V2::new(8, 4), &cells);
    grow(&mut s, V2::new(8, 4), &cells);
    assert_eq!(s.w.pos(SWAP_BOSS), V2::new(7, 4), "top-right: the old cell is the square's top-right");
    // units on (9, 5) and (7, 4): bottom-left (anchor (8, 3)) is next
    let cells = [far[0], far[1], (2, V2::new(7, 4)), (3, V2::new(9, 5))];
    let mut s = swap_session(floor(), V2::new(8, 4), &cells);
    grow(&mut s, V2::new(8, 4), &cells);
    assert_eq!(s.w.pos(SWAP_BOSS), V2::new(8, 3), "bottom-left");
    // units on (8, 5) (top-left and top-right) and (9, 4) (bottom-left): bottom-right (anchor (7, 3))
    let cells = [far[0], far[1], (2, V2::new(8, 5)), (3, V2::new(9, 4))];
    let mut s = swap_session(floor(), V2::new(8, 4), &cells);
    grow(&mut s, V2::new(8, 4), &cells);
    assert_eq!(s.w.pos(SWAP_BOSS), V2::new(7, 3), "bottom-right");
    assert_no_overlap(&s.w, "after the swap");
}

/// BU12: with no free 2 x 2 square anywhere the new unit takes the first walkable square around its old cell and pushes
/// the unit there to the nearest free cell outside it (no collision damage), naming it in the log.
#[test]
fn a_swap_with_no_room_pushes_units_back() {
    // floor only on x 9-10, y 3-5 (plus nothing else): the squares (9, 3) and (9, 4) both hold the foe at (9, 4)
    let mut rows = vec!["#".repeat(12); 9];
    for y in 3..=5 {
        rows[y] = format!("{}..#", "#".repeat(9));
    }
    let cells = [(0, V2::new(0, 0)), (1, V2::new(0, 8)), (2, V2::new(9, 4)), (3, V2::new(11, 8))];
    let mut s = swap_session(rows, V2::new(10, 4), &cells);
    let hp = s.w.units[2].hp;
    grow(&mut s, V2::new(10, 4), &cells);
    // the four squares around (10, 4): top-left (10, 4) covers the wall (11, 4); top-right (9, 4) is all floor
    assert_eq!(s.w.pos(SWAP_BOSS), V2::new(9, 4), "the first walkable square");
    assert_eq!(s.w.pos(2), V2::new(9, 3), "the foe pushed to the nearest free cell outside it (distance 1, reading order)");
    assert_eq!(s.w.units[2].hp, hp, "a push is not an attack");
    assert!(s.encounter_lines.iter().any(|(_, l)| l.starts_with("[Push] Foe is pushed back from (9, 4) to (9, 3)")), "{:?}", s.encounter_lines);
}

/// Walls everywhere but the floor cells `floor`.
fn walled(floor: &[(i32, i32)]) -> Vec<String> {
    let mut rows: Vec<Vec<char>> = vec![vec!['#'; 12]; 9];
    for &(x, y) in floor {
        rows[y as usize][x as usize] = '.';
    }
    rows.into_iter().map(|r| r.into_iter().collect()).collect()
}

/// BU12 chain: the unit pushed out has no free spot of its own (a 2 x 2 unit with no free square), so it takes the
/// nearest spot whose unit has not moved yet and that unit is pushed in turn to the nearest free cell.
#[test]
fn a_swap_push_chains_when_the_pushed_unit_has_no_room() {
    // floor x 0-5, y 3-4; the giant (2) on (3, 3)-(4, 4), the foe (3) on (5, 3), the boss (4) on (2, 3), the hero (1, 4),
    // the ally (0, 4): free are (0, 3), (1, 3), (2, 4) and (5, 4), so no 2 x 2 square is free
    let floor: Vec<(i32, i32)> = (0..6).flat_map(|x| [(x, 3), (x, 4)]).collect();
    let cells = [(0, V2::new(1, 4)), (1, V2::new(0, 4)), (2, V2::new(3, 3)), (3, V2::new(5, 3))];
    let mut s = swap_session_with(walled(&floor), json!(["b_giant", "b_foe", "b_boss"]), V2::new(2, 3), &cells);
    grow(&mut s, V2::new(2, 3), &cells);
    // the boss takes (2, 3) as top-left (terrain fits); the giant there has no free square ((0, 3) holds the hero and
    // the ally, (4, 3) the foe): it takes the nearer (4, 3) and the foe goes on to the nearest free cell, (1, 3)
    assert_eq!((s.w.size_of(SWAP_BOSS), s.w.pos(SWAP_BOSS)), (2, V2::new(2, 3)));
    assert_eq!((s.w.pos(2), s.w.pos(3)), (V2::new(4, 3), V2::new(1, 3)), "the giant, then the foe it bumped");
    assert_eq!((s.w.pos(0), s.w.pos(1)), (V2::new(1, 4), V2::new(0, 4)), "the rest stay");
    let pushes: Vec<&String> = s.encounter_lines.iter().map(|(_, l)| l).filter(|l| l.starts_with("[Push]")).collect();
    assert_eq!(pushes.len(), 2, "{pushes:?}");
    assert!(pushes[0].starts_with("[Push] Giant is pushed back from (3, 3) to (4, 3)") && pushes[1].starts_with("[Push] Foe is pushed back from (5, 3) to (1, 3)"), "{pushes:?}");
    assert_no_overlap(&s.w, "after the chain");
}

/// BU12 no room: with fewer free walkable cells than the units need, no square can be cleared: the new unit stays on
/// its old cell at one cell (logged), nobody moves and nothing overlaps.
#[test]
fn a_swap_on_a_crowded_board_stays_on_one_cell() {
    // 7 floor cells ((0..=3, 3) and (0..=2, 4)) for four one-cell units and a 2 x 2 one (8 cells)
    let floor = [(0, 3), (1, 3), (2, 3), (3, 3), (0, 4), (1, 4), (2, 4)];
    let cells = [(0, V2::new(0, 3)), (1, V2::new(0, 4)), (2, V2::new(1, 3)), (3, V2::new(1, 4))];
    let mut s = swap_session(walled(&floor), V2::new(2, 3), &cells);
    grow(&mut s, V2::new(2, 3), &cells);
    assert_eq!((s.w.size_of(SWAP_BOSS), s.w.pos(SWAP_BOSS)), (1, V2::new(2, 3)), "one cell, where it stood");
    assert_eq!(s.w.units[SWAP_BOSS].name, "Giant", "the swap itself happened");
    for (u, cell) in cells {
        assert_eq!(s.w.pos(u), cell, "unit {u} not pushed");
    }
    assert!(s.encounter_lines.iter().any(|(_, l)| l == "[Push] Giant finds no room to fill 2 x 2 cells and stays on one cell"), "{:?}", s.encounter_lines);
    assert!(!s.encounter_lines.iter().any(|(_, l)| l.contains("is pushed back")));
    assert_no_overlap(&s.w, "crowded");
}

/// BU13: a big enemy's AI walks and attacks over a few rounds without ever overlapping a unit or leaving its square's
/// fit, and closes on the party.
#[test]
fn the_ai_moves_a_big_unit_without_overlap() {
    let mut s = session(None);
    s.w.grid.positions[BIG] = Some(V2::new(8, 4));
    s.w.grid.positions[HERO] = Some(V2::new(2, 4));
    s.replan_enemies();
    let start = s.w.dist(BIG, HERO);
    for round in 1..=3 {
        assert!(s.confirm_plan(), "round {round}");
        while s.phase != Phase::Replay && !s.combat_over {
            s.step_next();
            assert_no_overlap(&s.w, &format!("round {round}"));
        }
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    assert!(s.w.dist(BIG, HERO) < start, "it closed in ({start} before)");
}
