//! WORLD-PRIM1 (ENGINE-9, WP1): grid primitives any game routes with. Cells (`Cell`, `+` and `-`), the 4 and 8
//! neighbours (`DIRS4`, `DIRS8`, `neighbours`), a cost search (`search`: uniform cost, or A* with a heuristic) whose
//! ties resolve the same way every run, a reach flood fill (`reach`) and the line of cells between two cells (`line`).
//!
//! The search ranks a cell by (cost, steps, the order it was found), packed into one key (`cost << 10 | steps`, then
//! `<< 20 | order`) in a binary heap, so equal routes resolve as found: by the caller's direction order, then first
//! found. The caller says what a step costs (`step(from, to)`: None when it may not be taken), what a goal is and, among
//! goals of equal rank, which is better (`better(a, b)`). Limits: 2^10 steps and 2^20 cells searched per run, costs
//! below 2^33 (far past any grid a game draws).

#![no_std]
extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::ops::{Add, Sub};

/// A cell of a grid (or a step between two cells).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

impl Cell {
    pub const fn new(x: i32, y: i32) -> Cell {
        Cell { x, y }
    }
    /// Steps between two cells moving in the 4 directions.
    pub fn manhattan(self, o: Cell) -> i32 {
        (self.x - o.x).abs() + (self.y - o.y).abs()
    }
    /// Steps between two cells moving in the 8 directions.
    pub fn chebyshev(self, o: Cell) -> i32 {
        (self.x - o.x).abs().max((self.y - o.y).abs())
    }
}

impl Add for Cell {
    type Output = Cell;
    fn add(self, o: Cell) -> Cell {
        Cell::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Cell {
    type Output = Cell;
    fn sub(self, o: Cell) -> Cell {
        Cell::new(self.x - o.x, self.y - o.y)
    }
}

/// The 4 directions: right, up, left, down (y grows down).
pub const DIRS4: [Cell; 4] = [Cell::new(1, 0), Cell::new(0, -1), Cell::new(-1, 0), Cell::new(0, 1)];
/// The 8 directions: the 4, then the diagonals clockwise from up-right.
pub const DIRS8: [Cell; 8] = [Cell::new(1, 0), Cell::new(0, -1), Cell::new(-1, 0), Cell::new(0, 1), Cell::new(1, -1), Cell::new(-1, -1), Cell::new(-1, 1), Cell::new(1, 1)];

/// The cells next to `c` in `dirs` order (`DIRS4` or `DIRS8`).
pub fn neighbours(c: Cell, dirs: &[Cell]) -> impl Iterator<Item = Cell> + '_ {
    dirs.iter().map(move |d| c + *d)
}

const SEQ_BITS: i64 = 20;
const STEP_BITS: i64 = 10;
const SEQ_MASK: i64 = (1 << SEQ_BITS) - 1;
const STEP_MASK: i64 = (1 << STEP_BITS) - 1;

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
    let last = heap.pop().unwrap_or(top);
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
        heap.swap(i, m);
        i = m;
    }
    top
}

/// What a search asks of its caller (see the module).
pub struct Search<'a> {
    /// The directions tried from each cell, in order (ties go to the earlier one).
    pub dirs: &'a [Cell],
    /// The cost of stepping `from` to `to` (at least 0); None: the step may not be taken.
    pub step: &'a dyn Fn(Cell, Cell) -> Option<i64>,
    /// Whether a cell is a goal. The start is never one (a search from a goal finds no steps).
    pub goal: &'a dyn Fn(Cell) -> bool,
    /// A* (admissible, never more than the real cost): the least cost left from a cell; None: uniform cost.
    pub heuristic: Option<&'a dyn Fn(Cell) -> i64>,
    /// Among goals of the same rank, whether `a` is better than `b` (None: the first found).
    pub better: Option<&'a dyn Fn(Cell, Cell) -> bool>,
}

/// The cheapest route from `start` to a goal, as steps (empty when `start` is a goal); None when no goal is reachable.
/// Goals are not walked through.
pub fn search(start: Cell, s: &Search) -> Option<Vec<Cell>> {
    if (s.goal)(start) {
        return Some(Vec::new());
    }
    let h = |c: Cell| s.heuristic.map_or(0, |f| f(c).max(0));
    let mut came_from: BTreeMap<Cell, Cell> = BTreeMap::new();
    came_from.insert(start, start);
    let mut best: BTreeMap<Cell, i64> = BTreeMap::new();
    let mut cost: BTreeMap<Cell, i64> = BTreeMap::new();
    let start_rank = h(start) << STEP_BITS;
    best.insert(start, start_rank);
    cost.insert(start, 0);
    let mut closed: BTreeSet<Cell> = BTreeSet::new();
    let (mut heap, mut cells) = (Vec::new(), Vec::new());
    push(&mut heap, start_rank, 0);
    cells.push(start);
    let mut found: Option<(Cell, i64)> = None;
    while !heap.is_empty() {
        let key = pop(&mut heap);
        let cur = cells[(key & SEQ_MASK) as usize];
        let rank = key >> SEQ_BITS;
        if closed.contains(&cur) || best.get(&cur).copied().unwrap_or(0) != rank {
            continue;
        }
        if found.is_some_and(|(_, k)| rank > k) {
            break;
        }
        closed.insert(cur);
        if cur != start && (s.goal)(cur) {
            let take = match found {
                None => true,
                Some((f, _)) => s.better.is_some_and(|b| b(cur, f)),
            };
            if take {
                found = Some((cur, rank));
            }
            continue;
        }
        let cur_cost = cost[&cur];
        let cur_steps = rank & STEP_MASK;
        for d in s.dirs {
            let nxt = cur + *d;
            if closed.contains(&nxt) {
                continue;
            }
            let Some(enter) = (s.step)(cur, nxt) else { continue };
            let nxt_cost = cur_cost + enter.max(0);
            let nxt_rank = ((nxt_cost + h(nxt)) << STEP_BITS) + cur_steps + 1;
            if best.get(&nxt).is_some_and(|b| *b <= nxt_rank) {
                continue;
            }
            best.insert(nxt, nxt_rank);
            cost.insert(nxt, nxt_cost);
            came_from.insert(nxt, cur);
            push(&mut heap, nxt_rank, cells.len() as i64);
            cells.push(nxt);
        }
    }
    let (mut cell, _) = found?;
    let mut steps = Vec::new();
    while cell != start {
        let prev = came_from[&cell];
        steps.push(cell - prev);
        cell = prev;
    }
    steps.reverse();
    Some(steps)
}

/// The cells of `steps` walked from `start` (the start not included).
pub fn walk(start: Cell, steps: &[Cell]) -> Vec<Cell> {
    let mut at = start;
    steps.iter().map(|d| {
        at = at + *d;
        at
    }).collect()
}

/// Every cell reachable from `start` within `budget`, with its least cost (the start at 0), stepping in `dirs` with
/// `step` (as in `Search`).
pub fn reach(start: Cell, budget: i64, dirs: &[Cell], step: &dyn Fn(Cell, Cell) -> Option<i64>) -> BTreeMap<Cell, i64> {
    let mut out: BTreeMap<Cell, i64> = BTreeMap::new();
    out.insert(start, 0);
    let (mut heap, mut cells) = (Vec::new(), Vec::new());
    push(&mut heap, 0, 0);
    cells.push(start);
    let mut done: BTreeSet<Cell> = BTreeSet::new();
    while !heap.is_empty() {
        let key = pop(&mut heap);
        let cur = cells[(key & SEQ_MASK) as usize];
        let c = key >> SEQ_BITS;
        if !done.insert(cur) || out.get(&cur).is_some_and(|b| *b != c) {
            continue;
        }
        for d in dirs {
            let nxt = cur + *d;
            let Some(enter) = step(cur, nxt) else { continue };
            let nc = c + enter.max(0);
            if nc > budget || out.get(&nxt).is_some_and(|b| *b <= nc) {
                continue;
            }
            out.insert(nxt, nc);
            push(&mut heap, nc, cells.len() as i64);
            cells.push(nxt);
        }
    }
    out
}

/// The cells on the line from `a` to `b`, both ends included (Bresenham: every cell the line passes, one per step of
/// the longer axis).
pub fn line(a: Cell, b: Cell) -> Vec<Cell> {
    let (dx, dy) = ((b.x - a.x).abs(), -(b.y - a.y).abs());
    let (sx, sy) = (if a.x < b.x { 1 } else { -1 }, if a.y < b.y { 1 } else { -1 });
    let (mut x, mut y, mut err) = (a.x, a.y, dx + dy);
    let mut out = Vec::new();
    loop {
        out.push(Cell::new(x, y));
        if x == b.x && y == b.y {
            return out;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
