//! GridPath (scripts/combat/arena/grid_path.gd): cost-weighted routes. Uniform-cost search with Godot's packed heap
//! keys (cost, steps, insertion order), so equal routes resolve exactly as in the reference; the search itself is the
//! engine's grid primitive (`aegis_grid::search`, WORLD-PRIM1), this module the arena's step costs and goal ties.

use crate::grid::{square_cells, square_covers, Grid, V2, DIRS};
use aegis_grid::{Cell, Search};
use std::collections::HashSet;

pub type Cells = HashSet<V2>;

/// Optional AI hooks: extra cost of entering a cell, and a preference among equally ranked destinations.
#[derive(Default)]
pub struct Hooks<'a> {
    pub penalty: Option<&'a dyn Fn(V2) -> i32>,
    pub prefer: Option<&'a dyn Fn(V2) -> f64>,
}

/// Godot's is_equal_approx for doubles.
pub fn approx_eq(a: f64, b: f64) -> bool {
    if a == b {
        return true;
    }
    let tol = (0.00001 * a.abs()).max(0.00001);
    (a - b).abs() < tol
}

/// Cheapest unit steps from `start` to any goal (empty when on a goal or unreachable). Ties: fewer steps, then the
/// preferred destination, then the one nearest the arena centre, then DIRS order. Exits are entered only as goals.
/// WORLD-PRIM1 (ENGINE-9): the search is the engine's (`aegis_grid::search`, the same ranks and order); this says what a
/// step costs, what a goal is and which goal is better.
pub fn find_path(grid: &Grid, start: V2, goals: &Cells, blocked: &Cells, goals_are_exits: bool, hooks: &Hooks) -> Vec<V2> {
    find_path_sized(grid, start, goals, blocked, goals_are_exits, hooks, 1)
}

/// BIGUNIT1 BU6: `find_path` for a walker of footprint `size` (1: `find_path`; 2: a 2 x 2 square whose anchor the
/// route moves, `find_square_path`).
pub fn find_path_sized(grid: &Grid, start: V2, goals: &Cells, blocked: &Cells, goals_are_exits: bool, hooks: &Hooks, size: i32) -> Vec<V2> {
    if size > 1 {
        return find_square_path(grid, start, goals, blocked, hooks, size);
    }
    let step = |_: Cell, to: Cell| -> Option<i64> {
        let nxt = v2(to);
        if !grid.is_valid(nxt) {
            return None;
        }
        if goals.contains(&nxt) {
            if !goals_are_exits && (blocked.contains(&nxt) || !grid.is_walkable(nxt)) {
                return None;
            }
        } else if blocked.contains(&nxt) || !grid.is_walkable(nxt) || grid.is_exit(nxt) {
            return None;
        }
        Some(enter_cost(grid, nxt) as i64 + hooks.penalty.map_or(0, |p| p(nxt).max(0) as i64))
    };
    route(start, goals, hooks, &step)
}

/// The engine's search from `start` over `step` to the nearest of `goals`, ties to the preferred goal, then the one
/// nearest the arena centre (empty when on a goal or unreachable).
fn route(start: V2, goals: &Cells, hooks: &Hooks, step: &dyn Fn(Cell, Cell) -> Option<i64>) -> Vec<V2> {
    let better = |a: Cell, b: Cell| {
        let (a, b) = (v2(a), v2(b));
        let (pa, pb) = (hooks.prefer.map(|f| f(a)).unwrap_or(0.0), hooks.prefer.map(|f| f(b)).unwrap_or(0.0));
        pa > pb || (approx_eq(pa, pb) && Grid::centre_distance(a) < Grid::centre_distance(b))
    };
    let goal = |c: Cell| goals.contains(&v2(c));
    let dirs = DIRS.map(cell);
    let found = aegis_grid::search(cell(start), &Search { dirs: &dirs, step, goal: &goal, heuristic: None, better: Some(&better) });
    found.unwrap_or_default().into_iter().map(v2).collect()
}

fn cell(v: V2) -> Cell {
    Cell::new(v.x, v.y)
}

fn v2(c: Cell) -> V2 {
    V2 { x: c.x, y: c.y }
}

/// BIGUNIT1 BU6: `find_path` for a square walker of side `n`: the route moves its anchor; a step is open only
/// when every cell of the destination square is on the grid, walkable, not an exit and not blocked (a big unit never
/// leaves by an exit); a step costs the highest enter cost among the cells it newly covers (the penalty hook likewise).
/// Same ranks and tie-breaks as `find_path`.
fn find_square_path(grid: &Grid, start: V2, goals: &Cells, blocked: &Cells, hooks: &Hooks, n: i32) -> Vec<V2> {
    let open = |a: V2| square_cells(a, n).all(|c| grid.is_walkable(c) && !grid.is_exit(c) && !blocked.contains(&c));
    let step = |from: Cell, to: Cell| -> Option<i64> {
        let (cur, nxt) = (v2(from), v2(to));
        if !open(nxt) {
            return None;
        }
        let mut enter = square_step_cost(grid, cur, nxt, n) as i64;
        if let Some(p) = hooks.penalty {
            enter += square_cells(nxt, n).filter(|c| !square_covers(cur, n, *c)).map(|c| p(c).max(0)).max().unwrap_or(0) as i64;
        }
        Some(enter)
    };
    route(start, goals, hooks, &step)
}

/// BIGUNIT1 BU6: the cost of a square of side `size` stepping its anchor from `from` to `to`: the highest enter cost
/// among the cells it newly covers (size 1: `enter_cost(to)`).
pub fn square_step_cost(grid: &Grid, from: V2, to: V2, size: i32) -> i32 {
    if size <= 1 {
        return enter_cost(grid, to);
    }
    square_cells(to, size).filter(|c| !square_covers(from, size, *c)).map(|c| enter_cost(grid, c)).max().unwrap_or(enter_cost(grid, to))
}

/// An exit tile costs 1 to enter, any other tile its terrain move cost.
pub fn enter_cost(grid: &Grid, cell: V2) -> i32 {
    if grid.is_exit(cell) {
        1
    } else {
        grid.move_cost(cell)
    }
}

pub fn path_cost(grid: &Grid, start: V2, dirs: &[V2]) -> i32 {
    path_cost_sized(grid, start, dirs, 1)
}

/// BIGUNIT1 BU6: `path_cost` for a walker of footprint `size` (`square_step_cost` per step).
pub fn path_cost_sized(grid: &Grid, start: V2, dirs: &[V2], size: i32) -> i32 {
    let mut total = 0;
    let mut cell = start;
    for d in dirs {
        total += square_step_cost(grid, cell, cell + *d, size);
        cell = cell + *d;
    }
    total
}

/// `dirs` split into move blocks of at most `budget` cost each; a block always takes its first tile.
pub fn chunk(grid: &Grid, start: V2, dirs: &[V2], budget: i32) -> Vec<Vec<V2>> {
    chunk_sized(grid, start, dirs, budget, 1)
}

/// BIGUNIT1 BU6: `chunk` for a walker of footprint `size`.
pub fn chunk_sized(grid: &Grid, start: V2, dirs: &[V2], budget: i32, size: i32) -> Vec<Vec<V2>> {
    let cap = budget.max(1);
    let mut out = Vec::new();
    let mut cell = start;
    let mut current: Vec<V2> = Vec::new();
    let mut spent = 0;
    for d in dirs {
        let cost = square_step_cost(grid, cell, cell + *d, size);
        if !current.is_empty() && spent + cost > cap {
            out.push(std::mem::take(&mut current));
            spent = 0;
        }
        current.push(*d);
        spent += cost;
        cell = cell + *d;
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

pub fn blocks_needed(grid: &Grid, start: V2, dirs: &[V2], budget: i32) -> usize {
    chunk(grid, start, dirs, budget).len()
}

/// BIGUNIT1 BU6: `blocks_needed` for a walker of footprint `size`.
pub fn blocks_needed_sized(grid: &Grid, start: V2, dirs: &[V2], budget: i32, size: i32) -> usize {
    chunk_sized(grid, start, dirs, budget, size).len()
}

/// The leading part of `dirs` that fits in `blocks` move blocks.
pub fn fit_blocks(grid: &Grid, start: V2, dirs: &[V2], budget: i32, blocks: usize) -> Vec<V2> {
    chunk(grid, start, dirs, budget).into_iter().take(blocks).flatten().collect()
}

/// BIGUNIT1 BU6: `fit_blocks` for a walker of footprint `size`.
pub fn fit_blocks_sized(grid: &Grid, start: V2, dirs: &[V2], budget: i32, blocks: usize, size: i32) -> Vec<V2> {
    chunk_sized(grid, start, dirs, budget, size).into_iter().take(blocks).flatten().collect()
}
