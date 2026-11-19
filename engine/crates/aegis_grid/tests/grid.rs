//! WORLD-PRIM1 (ENGINE-9): the grid primitives against what they promise: the cheapest route, ties by direction
//! order then the caller's `better`, A* agreeing with uniform cost, reach within the budget, lines end to end.

use aegis_grid::{line, neighbours, reach, search, walk, Cell, Search, DIRS4, DIRS8};

/// A 7 x 5 room: `#` walls, digits cost that much to enter, anything else costs 1.
const ROOM: [&str; 5] = [".......", ".###...", "...#.9.", ".#.#...", "......."];

fn cost(c: Cell) -> Option<i64> {
    let row = ROOM.get(usize::try_from(c.y).ok()?)?.as_bytes();
    match *row.get(usize::try_from(c.x).ok()?)? {
        b'#' => None,
        d @ b'0'..=b'9' => Some(i64::from(d - b'0')),
        _ => Some(1),
    }
}

fn route(from: Cell, to: Cell, heuristic: bool) -> Option<Vec<Cell>> {
    let h = move |c: Cell| i64::from(c.manhattan(to));
    search(from, &Search { dirs: &DIRS4, step: &|_, b| cost(b), goal: &|c| c == to, heuristic: heuristic.then_some(&h as &dyn Fn(Cell) -> i64), better: None })
}

fn total(from: Cell, steps: &[Cell]) -> i64 {
    walk(from, steps).into_iter().map(|c| cost(c).expect("a route walks open cells")).sum()
}

#[test]
fn the_route_is_the_cheapest_and_a_star_agrees() {
    let (a, b) = (Cell::new(0, 2), Cell::new(6, 2));
    let r = route(a, b, false).expect("reachable");
    assert_eq!(walk(a, &r).last(), Some(&b));
    assert_eq!(total(a, &r), 10, "around the wall, 10 steps at 1 (the 9 is never worth it)");
    let star = route(a, b, true).expect("reachable");
    assert_eq!(total(a, &star), total(a, &r), "A* finds a route as cheap");
    assert_eq!(route(a, a, false), Some(Vec::new()), "from the goal: no steps");
    assert_eq!(route(a, Cell::new(1, 1), false), None, "a wall is never reached");
    let fenced = |_: Cell, b: Cell| cost(b).filter(|_| b.y < 2);
    assert_eq!(search(Cell::new(0, 0), &Search { dirs: &DIRS4, step: &fenced, goal: &|c| c == Cell::new(6, 4), heuristic: None, better: None }), None, "a goal past every open cell: None");
}

#[test]
fn ties_go_to_the_direction_order_then_the_better_goal() {
    let open = |_: Cell, b: Cell| (b.x.abs() <= 3 && b.y.abs() <= 3).then_some(1);
    // Right comes before down in DIRS4, so of the two equal routes to (1, 1) the first step is right.
    let r = search(Cell::new(0, 0), &Search { dirs: &DIRS4, step: &open, goal: &|c| c == Cell::new(1, 1), heuristic: None, better: None }).unwrap();
    assert_eq!(r, [Cell::new(1, 0), Cell::new(0, 1)]);
    // Two goals one step away: the first found (right), unless `better` prefers the other.
    let goals = |c: Cell| c == Cell::new(1, 0) || c == Cell::new(-1, 0);
    let first = search(Cell::new(0, 0), &Search { dirs: &DIRS4, step: &open, goal: &goals, heuristic: None, better: None }).unwrap();
    assert_eq!(first, [Cell::new(1, 0)]);
    let left = |a: Cell, b: Cell| a.x < b.x;
    let picked = search(Cell::new(0, 0), &Search { dirs: &DIRS4, step: &open, goal: &goals, heuristic: None, better: Some(&left) }).unwrap();
    assert_eq!(picked, [Cell::new(-1, 0)]);
    // The same search twice gives the same route.
    for _ in 0..3 {
        assert_eq!(route(Cell::new(0, 0), Cell::new(6, 4), false), route(Cell::new(0, 0), Cell::new(6, 4), false));
    }
}

#[test]
fn reach_holds_every_cell_within_the_budget_at_its_least_cost() {
    let from = Cell::new(0, 0);
    let r = reach(from, 3, &DIRS4, &|_, b| cost(b));
    assert_eq!(r[&from], 0);
    for (c, n) in &r {
        assert!(*n <= 3);
        if *c != from {
            assert_eq!(Some(*n), route(from, *c, false).map(|s| total(from, &s)), "{c:?}: the reach cost is the route cost");
        }
    }
    assert!(r.contains_key(&Cell::new(3, 0)) && !r.contains_key(&Cell::new(4, 0)) && !r.contains_key(&Cell::new(1, 1)));
    let open = |_: Cell, _: Cell| Some(1);
    assert_eq!(reach(from, 1, &DIRS8, &open).len(), 9, "the 8 neighbours and the start");
    assert_eq!(neighbours(from, &DIRS8).count(), 8);
}

#[test]
fn a_line_runs_end_to_end_one_cell_per_step() {
    for (a, b) in [(Cell::new(0, 0), Cell::new(5, 2)), (Cell::new(3, 3), Cell::new(-2, 7)), (Cell::new(1, 1), Cell::new(1, 1)), (Cell::new(0, 4), Cell::new(0, -3))] {
        let l = line(a, b);
        assert_eq!((l.first(), l.last()), (Some(&a), Some(&b)));
        assert_eq!(l.len() as i32, a.chebyshev(b) + 1);
        assert!(l.windows(2).all(|w| w[0].chebyshev(w[1]) == 1), "{a:?} to {b:?}: {l:?}");
    }
}
