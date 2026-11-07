//! P1.2-S1 (docs/sessions/P_SPACE.md 1, FB-51) on scratch catalogs built here (no shipped data): a terrain row's
//! `walkable` and `stops_forced_move` are read from the row. A row that leaves them out, and a type with no row, read
//! the fallback the code fixed before (`grid::terrain_walkable`: deep water, lava, wall and tree block), and
//! `stops_forced_move` reads not walkable; an explicit value is read as given: pathfinding routes over a row made
//! walkable and around one made unwalkable, and a forced move collides before a walkable tile whose row stops forced
//! moves.

use indexmap::IndexMap;
use rpgkit_battle::command::Command;
use rpgkit_battle::grid::{terrain_walkable, Grid, TerrainTable, TERRAIN_IDS, V2};
use rpgkit_battle::path::{self, Cells, Hooks};
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, FORCED_MOVE, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const FOE: UnitId = 1;

fn index(id: &str) -> usize {
    TERRAIN_IDS.iter().position(|t| *t == id).unwrap_or_else(|| panic!("terrain {id}"))
}

/// A row of every terrain type with only its id (and a cover), as a row written before the fields existed.
fn bare_rows() -> Vec<Value> {
    TERRAIN_IDS.iter().map(|id| json!({"id": id, "name": id, "cover_guard": 0.1})).collect()
}

fn table(rows: Vec<Value>) -> TerrainTable {
    TerrainTable::from_catalog(&Catalog::from_value(&json!({"terrain": rows})))
}

/// The fallback is the code's old rule, for a type with no row and for a row without the keys.
#[test]
fn the_fallback_is_the_old_rule() {
    for (what, t) in [("no rows", table(Vec::new())), ("rows without the keys", table(bare_rows()))] {
        for (i, id) in TERRAIN_IDS.iter().enumerate() {
            let row = &t.rows[i];
            assert_eq!(row.walkable, terrain_walkable(id), "{what}: {id}");
            assert_eq!(row.stops_forced_move, !row.walkable, "{what}: {id}");
        }
    }
    assert!(["deep_water", "lava", "wall", "tree"].iter().all(|id| !terrain_walkable(id)), "the blocked types");
    assert_eq!(TERRAIN_IDS.iter().filter(|id| terrain_walkable(id)).count(), TERRAIN_IDS.len() - 4);
}

/// Explicit values are read as given; `stops_forced_move` alone follows `walkable` when left out.
#[test]
fn explicit_values_are_read() {
    let mut rows = bare_rows();
    rows[index("grass")]["walkable"] = json!(false);
    rows[index("wall")]["walkable"] = json!(true);
    rows[index("tree")]["stops_forced_move"] = json!(false);
    rows[index("sand")]["stops_forced_move"] = json!(true);
    let t = table(rows);
    let at = |id: &str| (t.rows[index(id)].walkable, t.rows[index(id)].stops_forced_move);
    assert_eq!(at("grass"), (false, true), "made unwalkable: stops forced moves by default");
    assert_eq!(at("wall"), (true, false), "made walkable: lets forced moves in by default");
    assert_eq!(at("tree"), (false, false), "set alone");
    assert_eq!(at("sand"), (true, true), "set alone");
    assert_eq!(at("floor"), (true, false), "untouched");
}

/// Pathfinding reads the row: a wall line is crossed once wall rows are walkable, a grass line is routed around once
/// grass rows are not.
#[test]
fn pathfinding_reads_walkable() {
    let path_len = |rows: Vec<Value>, line: usize| {
        let mut g = Grid::new(false, Vec::new(), table(rows), 0);
        // a vertical line at x = 5 for y 0..=7 (row 8 open)
        for y in 0..8 {
            g.terrain[(y * rpgkit_battle::grid::GRID_W + 5) as usize] = line;
        }
        let goals: Cells = [V2::new(8, 0)].into_iter().collect();
        path::find_path(&g, V2::new(2, 0), &goals, &Cells::new(), false, &Hooks { penalty: None, prefer: None }).len()
    };
    let straight = 6;
    assert!(path_len(bare_rows(), index("wall")) > straight, "a wall line is routed around");
    let mut open = bare_rows();
    open[index("wall")]["walkable"] = json!(true);
    assert_eq!(path_len(open, index("wall")), straight, "a walkable wall is crossed");
    assert_eq!(path_len(bare_rows(), index("grass")), straight, "grass is crossed");
    let mut shut = bare_rows();
    shut[index("grass")]["walkable"] = json!(false);
    assert!(path_len(shut, index("grass")) > straight, "an unwalkable grass line is routed around");
}

fn session(rows: Vec<Value>) -> Session {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_shove", "name": "t_shove", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.2, "target_mode": "ENEMY", "reach_min": 1, "reach_max": 1, "forced_move": {"kind": "push", "tiles": 2}}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": speed, "attack_range": 1, "skill_cap": 5});
    let cat = Catalog::from_value(&json!({"skills": skills, "units": [unit("t_hero", "party", 14), unit("t_foe", "enemy", 10)], "terrain": rows}));
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["t_hero"], "enemies": ["t_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [FORCED_MOVE]}).as_object().unwrap());
    let mut s = Session::start(Arc::new(cat), def, &IndexMap::new());
    s.w.grid.positions[HERO] = Some(V2::new(4, 4));
    s.w.grid.positions[FOE] = Some(V2::new(5, 4));
    s
}

fn shove(w: &World) -> Command {
    let id = w.book().find("t_shove").expect("t_shove");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(id, FOE), true), "");
    plan.entries_for(w, HERO).into_iter().find(|e| e.skill == id).expect("the entry").build_command(w, HERO)
}

/// The hero pushes the foe 2 tiles east with `terrain` on (7, 4): where the foe ends and what blocked it.
fn push_into(rows: Vec<Value>, terrain: &str) -> (V2, String) {
    let mut s = session(rows);
    s.w.grid.terrain[(4 * rpgkit_battle::grid::GRID_W + 7) as usize] = index(terrain);
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(HERO, shove(&s.w), 1));
    for i in res.prepare_block(&mut s.w, 1) {
        res.execute_single(&mut s.w, i);
    }
    let m = &res.scheduled[0].command.pushes[0];
    (s.w.pos(FOE), m.blocked_by.clone())
}

/// A forced move reads `stops_forced_move`: by default a sand tile lets the push in and a wall stops it; a sand row that
/// stops forced moves stops it before the tile, and a wall row made walkable that does not lets it in.
#[test]
fn forced_moves_read_stops_forced_move() {
    assert_eq!(push_into(bare_rows(), "sand"), (V2::new(7, 4), String::new()), "sand by default");
    assert_eq!(push_into(bare_rows(), "wall").0, V2::new(6, 4), "a wall by default");
    let mut stop = bare_rows();
    stop[index("sand")]["stops_forced_move"] = json!(true);
    let (cell, by) = push_into(stop, "sand");
    assert_eq!(cell, V2::new(6, 4), "a sand row that stops forced moves");
    assert!(by.contains("sand"), "{by}");
    let mut open = bare_rows();
    open[index("wall")]["walkable"] = json!(true);
    assert_eq!(push_into(open, "wall"), (V2::new(7, 4), String::new()), "a walkable wall that does not stop them");
}
