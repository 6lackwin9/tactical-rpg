//! GridPath (scripts/combat/arena/grid_path.gd): cost-weighted routes. Uniform-cost search with Godot's packed heap
//! keys (cost, steps, insertion order), so equal routes resolve exactly as in the reference.

use crate::grid::{Grid, V2, DIRS, NO_CELL};
use std::collections::{HashMap, HashSet};

const SEQ_BITS: i64 = 20;
const STEP_BITS: i64 = 10;
const SEQ_MASK: i64 = (1 << SEQ_BITS) - 1;
const STEP_MASK: i64 = (1 << STEP_BITS) - 1;

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
pub fn find_path(grid: &Grid, start: V2, goals: &Cells, blocked: &Cells, goals_are_exits: bool, hooks: &Hooks) -> Vec<V2> {
    let mut steps = Vec::new();
    if goals.contains(&start) {
        return steps;
    }
    let mut came_from: HashMap<V2, V2> = HashMap::new();
    came_from.insert(start, start);
    let mut best: HashMap<V2, i64> = HashMap::new();
    best.insert(start, 0);
    let mut closed: HashSet<V2> = HashSet::new();
    let mut heap: Vec<i64> = Vec::new();
    let mut cells: Vec<V2> = Vec::new();
    push(&mut heap, 0, 0);
    cells.push(start);
    let mut found = NO_CELL;
    let mut found_key: i64 = -1;
    let mut found_pref = 0.0;
    while !heap.is_empty() {
        let key = pop(&mut heap);
        let cur = cells[(key & SEQ_MASK) as usize];
        let rank = key >> SEQ_BITS;
        if closed.contains(&cur) || best.get(&cur).copied().unwrap_or(0) != rank {
            continue;
        }
        if found_key >= 0 && rank > found_key {
            break;
        }
        closed.insert(cur);
        if goals.contains(&cur) && cur != start {
            let pref = hooks.prefer.map(|f| f(cur)).unwrap_or(0.0);
            if found == NO_CELL || pref > found_pref || (approx_eq(pref, found_pref) && Grid::centre_distance(cur) < Grid::centre_distance(found)) {
                found = cur;
                found_key = rank;
                found_pref = pref;
            }
            continue;
        }
        let cur_cost = rank >> STEP_BITS;
        let cur_steps = rank & STEP_MASK;
        for d in DIRS {
            let nxt = cur + d;
            if closed.contains(&nxt) || !grid.is_valid(nxt) {
                continue;
            }
            if goals.contains(&nxt) {
                if !goals_are_exits && (blocked.contains(&nxt) || !grid.is_walkable(nxt)) {
                    continue;
                }
            } else if blocked.contains(&nxt) || !grid.is_walkable(nxt) || grid.is_exit(nxt) {
                continue;
            }
            let mut enter = enter_cost(grid, nxt) as i64;
            if let Some(p) = hooks.penalty {
                enter += p(nxt).max(0) as i64;
            }
            let nxt_rank = ((cur_cost + enter) << STEP_BITS) + cur_steps + 1;
            if best.get(&nxt).is_some_and(|b| *b <= nxt_rank) {
                continue;
            }
            best.insert(nxt, nxt_rank);
            came_from.insert(nxt, cur);
            push(&mut heap, nxt_rank, cells.len() as i64);
            cells.push(nxt);
        }
    }
    if found == NO_CELL {
        return steps;
    }
    let mut cell = found;
    while cell != start {
        let prev = came_from[&cell];
        steps.push(cell - prev);
        cell = prev;
    }
    steps.reverse();
    steps
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
    let mut total = 0;
    let mut cell = start;
    for d in dirs {
        cell = cell + *d;
        total += enter_cost(grid, cell);
    }
    total
}

/// `dirs` split into move blocks of at most `budget` cost each; a block always takes its first tile.
pub fn chunk(grid: &Grid, start: V2, dirs: &[V2], budget: i32) -> Vec<Vec<V2>> {
    let cap = budget.max(1);
    let mut out = Vec::new();
    let mut cell = start;
    let mut current: Vec<V2> = Vec::new();
    let mut spent = 0;
    for d in dirs {
        let cost = enter_cost(grid, cell + *d);
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

/// The leading part of `dirs` that fits in `blocks` move blocks.
pub fn fit_blocks(grid: &Grid, start: V2, dirs: &[V2], budget: i32, blocks: usize) -> Vec<V2> {
    chunk(grid, start, dirs, budget).into_iter().take(blocks).flatten().collect()
}

fn push(heap: &mut Vec<i64>, rank: i64, seq: i64) {
    heap.push((rank << SEQ_BITS) | seq);
    let mut i = heap.len() - 1;
    while i > 0 {
        let parent = (i - 1) >> 1;
        if heap[parent] <= heap[i] {
            break;
        }
        heap.swap(parent, i);
        i = parent;
    }
}

fn pop(heap: &mut Vec<i64>) -> i64 {
    let top = heap[0];
    let last = heap.pop().unwrap();
    if heap.is_empty() {
        return top;
    }
    heap[0] = last;
    let n = heap.len();
    let mut i = 0;
    loop {
        let l = 2 * i + 1;
        let r = l + 1;
        let mut m = i;
        if l < n && heap[l] < heap[m] {
            m = l;
        }
        if r < n && heap[r] < heap[m] {
            m = r;
        }
        if m == i {
            break;
        }
        heap.swap(m, i);
        i = m;
    }
    top
}
