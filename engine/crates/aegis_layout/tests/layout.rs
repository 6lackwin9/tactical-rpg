//! LAYOUT720-E2: the crate's rules (docs/specs/LAYOUT720.md 6.1) and the worked example of 6.3. Expectations come from the
//! rules or the spec's table, never from a printed result. LAYOUTDATA1-E1 (docs/specs/LAYOUTDATA1.md 10.1, gaps 1, 2, 5):
//! `ratio`, `Part::min` and `Stack` on an axis, at the end.

use aegis_layout::{fill, frac, pick, ratio, resolve, Anchor, Axis, Insets, Need, Part, Rect, Stack, MAX_GRID_CELLS};

/// One past the last column.
fn right(r: Rect) -> i32 {
    r.x + r.w
}

/// One past the last row.
fn bottom(r: Rect) -> i32 {
    r.y + r.h
}

/// The worked example's screen on a canvas: margin 16, a rail of frac(186), gap 12, then fill(2), fill(1) with gap 12.
fn worked_example(canvas: Rect) -> (Rect, Rect, Rect, Rect) {
    let area = canvas.pad(Insets::all(16));
    let mut body = area;
    let rail = body.cut_left(frac(186));
    body.cut_left(12);
    let panes = body.split_x(&[fill(2), fill(1)], 12);
    (area, rail, panes[0], panes[1])
}

#[test]
fn worked_example_at_the_fixed_canvas() {
    let (area, rail, list, detail) = worked_example(Rect::new(0, 0, 1280, 720));
    assert_eq!((area.x, right(area), area.w), (16, 1264, 1248));
    assert_eq!((rail.x, right(rail)), (16, 248));
    assert_eq!((list.x, right(list), list.w), (260, 921, 661));
    assert_eq!((detail.x, right(detail), detail.w), (933, 1264, 331));
    assert_eq!(right(detail), right(area), "ends on the area's edge");
    assert_eq!((rail.y, rail.h, list.h, detail.h), (16, 688, 688, 688), "every column is the area's height");
}

#[test]
fn worked_example_at_the_ultrawide_canvas() {
    let (area, rail, list, detail) = worked_example(Rect::new(0, 0, 1720, 720));
    assert_eq!((area.x, right(area), area.w), (16, 1704, 1688));
    assert_eq!((rail.x, right(rail)), (16, 329));
    assert_eq!((list.x, right(list), list.w), (341, 1241, 900));
    assert_eq!((detail.x, right(detail), detail.w), (1253, 1704, 451));
    assert_eq!(right(detail), right(area), "ends on the area's edge");
}

#[test]
fn the_worked_example_is_where_rounding_sizes_would_lose_a_unit() {
    for (w, room) in [(1280, 992), (1720, 1351)] {
        let (area, _, list, detail) = worked_example(Rect::new(0, 0, w, 720));
        // Rounding each size on its own loses the remainder at the far end; the shared edge does not.
        let by_sizes = list.x + room * 2 / 3 + 12 + room / 3;
        assert_eq!(by_sizes, right(area) - 1, "the example must be one that shows the rule");
        assert_eq!(right(detail), right(area));
    }
}

#[test]
fn a_split_adds_up_at_every_part_count_and_width() {
    for parts in 1..=12i32 {
        let equal = vec![fill(1); parts as usize];
        let uneven: Vec<Part> = (1..=parts as u16).map(fill).collect();
        let total: i32 = (1..=parts).sum();
        for width in 1..=2000 {
            for gap in [0, 1, 12] {
                let area = Rect::new(7, 3, width, 50);
                let gaps = gap * (parts - 1);
                let room = width - gaps;
                for (weights, sum) in [(&equal, parts), (&uneven, total)] {
                    let cells = area.split_x(weights, gap);
                    assert_eq!(cells.len(), parts as usize);
                    assert!(cells.iter().all(|c| c.w >= 0 && c.x >= area.x && right(*c) <= right(area)), "{parts} parts, width {width}, gap {gap}");
                    if room < 0 {
                        assert!(cells.iter().all(|c| c.w == 0), "gaps wider than the rect: {parts} parts, width {width}, gap {gap}");
                        continue;
                    }
                    assert_eq!(cells[0].x, area.x);
                    assert_eq!(right(cells[parts as usize - 1]), right(area), "ends on the edge: {parts} parts, width {width}, gap {gap}");
                    let mut weight = 0;
                    for (i, c) in cells.iter().enumerate() {
                        weight += if sum == parts { 1 } else { i as i32 + 1 };
                        // Each boundary is the running total rounded down once.
                        if i + 1 < parts as usize {
                            assert_eq!(right(*c), area.x + room * weight / sum + gap * i as i32, "boundary {i}: {parts} parts, width {width}, gap {gap}");
                            assert_eq!(cells[i + 1].x, right(*c) + gap, "neighbours one gap apart: {parts} parts, width {width}, gap {gap}");
                        }
                    }
                }
                // Equal fills differ by at most 1 unit, the whole cost of the rule, however many there are.
                let cells = area.split_x(&equal, gap);
                let (min, max) = (cells.iter().map(|c| c.w).min().unwrap(), cells.iter().map(|c| c.w).max().unwrap());
                assert!(max - min <= 1, "{parts} parts, width {width}, gap {gap}");
            }
        }
    }
}

#[test]
fn split_y_follows_the_same_rule_as_split_x() {
    for parts in 1..=12usize {
        let weights: Vec<Part> = (1..=parts as u16).map(fill).collect();
        for len in 1..=500 {
            let across = Rect::new(4, 9, len, 33).split_x(&weights, 2);
            let down = Rect::new(9, 4, 33, len).split_y(&weights, 2);
            for (a, d) in across.iter().zip(&down) {
                assert_eq!((a.x, a.w, a.y, a.h), (d.y, d.h, d.x, d.w));
            }
        }
    }
}

#[test]
fn resolve_takes_tokens_then_shares_of_the_rest_then_fills() {
    // In the parts' order whatever the kind: 100 first, half of the 1,000 left, then the 500 left for the fill.
    assert_eq!(resolve(&[fill(1), 100.into(), frac(500)], 1100), [500, 100, 500]);
    // A share is of what the tokens leave, not of the whole room.
    assert_eq!(resolve(&[200.into(), frac(500), frac(500)], 1000), [200, 400, 400]);
    // Fills divide by weight.
    assert_eq!(resolve(&[fill(3), fill(1)], 100), [75, 25]);
    // Less than the whole in shares and no fill: the rest stays unused.
    assert_eq!(resolve(&[frac(250)], 100), [25]);
    assert_eq!(resolve(&[], 100), Vec::<i32>::new());
}

#[test]
fn shares_totalling_a_thousand_end_on_the_edge_at_every_width() {
    for width in 1..=2000 {
        let sizes = resolve(&[frac(333), frac(333), frac(334)], width);
        assert_eq!(sizes.iter().sum::<i32>(), width, "width {width}");
        // After a token too: the shares are of what is left.
        let sizes = resolve(&[1.into(), frac(500), frac(500)], width);
        assert_eq!(sizes.iter().sum::<i32>(), width, "width {width}, with a token");
        let cells = Rect::new(5, 0, width, 9).split_x(&[frac(186), frac(814)], 0);
        assert_eq!(right(cells[1]), 5 + width, "width {width}, split");
        assert_eq!(cells[1].x, right(cells[0]), "width {width}, split");
    }
}

#[test]
fn a_size_that_cannot_fit_becomes_zero() {
    assert_eq!(resolve(&[100.into()], 50), [0]);
    assert_eq!(resolve(&[100.into(), 20.into()], 50), [0, 20], "a later token that fits still gets its turn");
    assert_eq!(resolve(&[600.into(), fill(1)], 500), [0, 500], "what the token did not take stays for the fill");
    assert_eq!(resolve(&[frac(600), frac(600), frac(300)], 1000), [600, 0, 300], "a share past 1,000 in total");
    assert_eq!(resolve(&[frac(1200)], 1000), [0]);
    assert_eq!(resolve(&[(-5).into(), fill(1)], 10), [0, 10], "a negative token is nothing");
    assert_eq!(resolve(&[fill(0), fill(0)], 10), [0, 0], "no weight, no size");
    assert_eq!(resolve(&[fill(1), 5.into(), frac(500)], -10), [0, 0, 0], "negative room is no room");
}

#[test]
fn extreme_numbers_neither_panic_nor_go_negative() {
    let big = [i32::MIN, -1_000_000, -1, 0, 1, 1_000_000, i32::MAX];
    let parts = |n: i32| -> [Part; 4] { [n.into(), frac(u16::MAX), fill(u16::MAX), fill(0)] };
    let rects = big.iter().flat_map(|&x| big.iter().map(move |&w| Rect::new(x, x, w, w)));
    for r in rects {
        for &n in &big {
            let mut sizes = vec![];
            for part in parts(n) {
                sizes.push(r.place(Anchor::Center, part, part));
                sizes.push(r.at(u16::MAX, u16::MAX, Anchor::BottomRight, part, part));
                let mut c = r;
                sizes.push(c.cut_top(part));
                sizes.push(c.cut_bottom(part));
                sizes.push(c.cut_left(part));
                sizes.push(c.cut_right(part));
                sizes.push(c);
                let mut s = Stack::new(r, n);
                sizes.push(s.next(part));
                sizes.push(s.next(part));
                assert!(s.overflow() >= 0);
            }
            sizes.push(r.pad(Insets::all(n)));
            sizes.extend(r.split_x(&parts(n), n));
            sizes.extend(r.split_y(&parts(n), n));
            sizes.extend(r.grid(2, 2, n));
            let _ = r.offset(n, n);
            for s in sizes {
                assert!(s.w >= 0 && s.h >= 0, "{r:?} with {n}: {s:?}");
            }
            assert!(resolve(&parts(n), n).iter().all(|&s| s >= 0));
        }
    }
}

#[test]
fn place_puts_an_element_on_each_of_the_nine_points() {
    let p = Rect::new(10, 20, 100, 60);
    let at = |a| p.place(a, 30, 10);
    assert_eq!(at(Anchor::TopLeft), Rect::new(10, 20, 30, 10));
    assert_eq!(at(Anchor::Top), Rect::new(45, 20, 30, 10));
    assert_eq!(at(Anchor::TopRight), Rect::new(80, 20, 30, 10));
    assert_eq!(at(Anchor::Left), Rect::new(10, 45, 30, 10));
    assert_eq!(at(Anchor::Center), Rect::new(45, 45, 30, 10));
    assert_eq!(at(Anchor::Right), Rect::new(80, 45, 30, 10));
    assert_eq!(at(Anchor::BottomLeft), Rect::new(10, 70, 30, 10));
    assert_eq!(at(Anchor::Bottom), Rect::new(45, 70, 30, 10));
    assert_eq!(at(Anchor::BottomRight), Rect::new(80, 70, 30, 10));
}

#[test]
fn place_reads_shares_of_the_parent_and_a_fill_as_all_of_it() {
    let canvas = Rect::new(0, 0, 1280, 720);
    let r = canvas.place(Anchor::Center, frac(703), frac(861));
    assert_eq!((r.w, r.h), (1280 * 703 / 1000, 720 * 861 / 1000), "a share rounds down");
    assert_eq!((r.x, r.y), ((1280 - r.w) / 2, (720 - r.h) / 2));
    assert_eq!(canvas.place(Anchor::Top, fill(1), 40), Rect::new(0, 0, 1280, 40));
}

#[test]
fn a_centred_odd_remainder_rounds_toward_the_top_left() {
    for len in 1..=300 {
        for size in 0..=len {
            let r = Rect::new(0, 0, len, len).place(Anchor::Center, size, size);
            let before = r.x;
            let after = len - right(r);
            assert!(after == before || after == before + 1, "len {len}, size {size}: {before} before, {after} after");
            assert_eq!(r.x, r.y);
        }
    }
}

#[test]
fn place_and_at_clamp_to_the_parent() {
    let p = Rect::new(10, 20, 100, 60);
    assert_eq!(p.place(Anchor::Center, 500, 500), p, "too big: the parent's size");
    assert_eq!(p.place(Anchor::BottomRight, frac(2000), frac(2000)), p, "a share over the whole: the parent's size");
    assert_eq!(p.at(1000, 1000, Anchor::TopLeft, 30, 10), Rect::new(80, 70, 30, 10), "the far corner stays inside");
    assert_eq!(p.at(0, 0, Anchor::BottomRight, 30, 10), Rect::new(10, 20, 30, 10), "the near corner stays inside");
    assert_eq!(p.at(u16::MAX, u16::MAX, Anchor::Center, 30, 10), Rect::new(80, 70, 30, 10));
    for fx in (0..=1000).step_by(7) {
        for w in [0, 1, 33, 99, 100, 101] {
            let r = p.at(fx, fx, Anchor::Center, w, w.min(60));
            assert!(r.x >= p.x && right(r) <= right(p) && r.y >= p.y && bottom(r) <= bottom(p), "fx {fx}, w {w}: {r:?}");
        }
    }
}

#[test]
fn at_rounds_once_and_agrees_with_place_on_the_anchor_points() {
    // The anchor's own point of the element sits on the per mille point of the parent.
    assert_eq!(Rect::new(0, 0, 1000, 1000).at(250, 750, Anchor::TopLeft, 10, 10), Rect::new(250, 750, 10, 10));
    assert_eq!(Rect::new(0, 0, 1000, 1000).at(250, 750, Anchor::BottomRight, 10, 10), Rect::new(240, 740, 10, 10));
    assert_eq!(Rect::new(0, 0, 1000, 1000).at(250, 750, Anchor::Center, 10, 10), Rect::new(245, 745, 10, 10));
    let points = [
        (Anchor::TopLeft, 0, 0),
        (Anchor::Top, 500, 0),
        (Anchor::TopRight, 1000, 0),
        (Anchor::Left, 0, 500),
        (Anchor::Center, 500, 500),
        (Anchor::Right, 1000, 500),
        (Anchor::BottomLeft, 0, 1000),
        (Anchor::Bottom, 500, 1000),
        (Anchor::BottomRight, 1000, 1000),
    ];
    for len in 1..=120 {
        let p = Rect::new(3, 5, len, len + 1);
        for size in 0..=len {
            for (a, fx, fy) in points {
                assert_eq!(p.at(fx, fy, a, size, size), p.place(a, size, size), "{a:?}, len {len}, size {size}");
            }
        }
    }
}

#[test]
fn cuts_share_their_edge_with_what_stays() {
    let whole = Rect::new(10, 20, 301, 201);
    for part in [Part::from(50), frac(333), frac(1000), fill(1)] {
        let (mut t, mut b, mut l, mut r) = (whole, whole, whole, whole);
        let top = t.cut_top(part);
        assert_eq!((top.x, top.y, top.w, bottom(top)), (whole.x, whole.y, whole.w, t.y));
        assert_eq!(bottom(t), bottom(whole));
        let foot = b.cut_bottom(part);
        assert_eq!((foot.x, foot.y, bottom(foot), foot.w), (whole.x, bottom(b), bottom(whole), whole.w));
        assert_eq!(b.y, whole.y);
        let left = l.cut_left(part);
        assert_eq!((left.x, left.y, right(left), left.h), (whole.x, whole.y, l.x, whole.h));
        assert_eq!(right(l), right(whole));
        let side = r.cut_right(part);
        assert_eq!((side.x, right(side), side.h), (right(r), right(whole), whole.h));
        assert_eq!(r.x, whole.x);
    }
    // A share is of the rect as it stands now, rounded down.
    let mut c = Rect::new(0, 0, 1000, 10);
    assert_eq!(c.cut_left(frac(500)).w, 500);
    assert_eq!(c.cut_left(frac(500)).w, 250);
    assert_eq!(c.cut_left(frac(1000)).w, 250);
    assert_eq!(c.w, 0);
}

#[test]
fn a_cut_that_cannot_fit_is_zero_and_leaves_the_rest() {
    let whole = Rect::new(10, 20, 100, 50);
    let mut c = whole;
    assert_eq!(c.cut_top(51).h, 0);
    assert_eq!(c.cut_bottom(51).h, 0);
    assert_eq!(c.cut_left(101).w, 0);
    assert_eq!(c.cut_right(101).w, 0);
    assert_eq!(c, whole);
    assert_eq!(c.cut_top(frac(1001)).h, 0);
    assert_eq!(c, whole);
}

#[test]
fn pad_takes_from_each_side_and_stays_inside_when_it_cannot_fit() {
    let p = Rect::new(10, 20, 100, 50);
    assert_eq!(p.pad(Insets { l: 1, t: 2, r: 3, b: 4 }), Rect::new(11, 22, 96, 44));
    assert_eq!(p.pad(Insets::all(5)), Rect::new(15, 25, 90, 40));
    assert_eq!(p.pad(Insets::xy(5, 2)), Rect::new(15, 22, 90, 46));
    let too_much = p.pad(Insets::all(200));
    assert_eq!((too_much.w, too_much.h), (0, 0));
    assert_eq!((too_much.x, too_much.y), (right(p), bottom(p)), "still inside the parent");
    assert_eq!(p.pad(Insets::all(-2)), Rect::new(8, 18, 104, 54), "a negative inset grows it");
    assert_eq!(p.offset(3, -4), Rect::new(13, 16, 100, 50));
}

#[test]
fn grid_cells_run_across_then_down_and_add_up() {
    for w in 1..=200 {
        for (cols, rows, gap) in [(1, 1, 0), (3, 2, 0), (3, 2, 4), (7, 5, 1)] {
            let area = Rect::new(2, 6, w, w + 9);
            let cells = area.grid(cols, rows, gap);
            assert_eq!(cells.len(), (cols * rows) as usize);
            let cell = |c: i32, r: i32| cells[(r * cols + c) as usize];
            let (wide, tall) = (w >= gap * (cols - 1), w + 9 >= gap * (rows - 1));
            for r in 0..rows {
                for c in 0..cols {
                    if c + 1 < cols && wide {
                        assert_eq!(cell(c + 1, r).x, right(cell(c, r)) + gap, "w {w}: column {c}");
                    }
                    if r + 1 < rows && tall {
                        assert_eq!(cell(c, r + 1).y, bottom(cell(c, r)) + gap, "w {w}: row {r}");
                    }
                }
            }
            if wide {
                assert_eq!(right(cell(cols - 1, 0)), right(area), "w {w}");
            }
            if tall {
                assert_eq!(bottom(cell(0, rows - 1)), bottom(area), "w {w}");
            }
        }
    }
}

#[test]
fn grid_of_nothing_or_too_much_is_empty() {
    let r = Rect::new(0, 0, 100, 100);
    assert!(r.grid(0, 3, 0).is_empty());
    assert!(r.grid(3, -1, 0).is_empty());
    assert!(r.grid(i32::MAX, i32::MAX, 0).is_empty());
    let side = (MAX_GRID_CELLS as f64).sqrt() as i32;
    assert_eq!(r.grid(side, side, 0).len(), (side * side) as usize, "at the limit");
    assert!(r.grid(side + 1, side + 1, 0).is_empty(), "past the limit");
}

#[test]
fn a_stack_runs_down_one_gap_apart() {
    let area = Rect::new(5, 10, 80, 100);
    let mut s = Stack::new(area, 4);
    let (a, b) = (s.next(20), s.next(30));
    assert_eq!(a, Rect::new(5, 10, 80, 20), "no gap before the first");
    assert_eq!(b.y, bottom(a) + 4);
    assert_eq!(s.overflow(), 0);
    let rest = s.next(fill(1));
    assert_eq!(bottom(rest), bottom(area), "a fill takes what is left");
    assert_eq!(s.overflow(), 0);
}

#[test]
fn stack_overflow_counts_the_units_past_the_area() {
    let area = Rect::new(5, 10, 80, 100);
    let mut s = Stack::new(area, 4);
    s.next(90);
    assert_eq!(s.overflow(), 0);
    let late = s.next(20);
    assert_eq!(late.h, 0, "a size that cannot fit becomes 0");
    assert!(late.y >= area.y && bottom(late) <= bottom(area), "and stays inside");
    assert_eq!(s.overflow(), 90 + 4 + 20 - 100);
    let later = s.next(fill(1));
    assert_eq!(later.h, 0, "a fill has nothing left");
    let mut exact = Stack::new(area, 0);
    exact.next(100);
    assert_eq!(exact.overflow(), 0, "exactly full is not overflow");
    assert_eq!(Stack::new(area, 0).overflow(), 0);
}

#[test]
fn stack_shares_use_a_running_total_at_every_height() {
    for h in 1..=2000 {
        let area = Rect::new(0, 3, 10, h);
        let mut s = Stack::new(area, 0);
        let cells = [s.next(frac(333)), s.next(frac(333)), s.next(frac(334))];
        assert_eq!(cells[1].y, bottom(cells[0]), "h {h}");
        assert_eq!(cells[2].y, bottom(cells[1]), "h {h}");
        assert_eq!(bottom(cells[2]), bottom(area), "h {h}: shares totalling 1,000 end on the edge");
        assert_eq!(s.overflow(), 0, "h {h}");
    }
}

#[test]
fn pick_takes_the_first_choice_that_fits_else_the_last() {
    let choices = [(Need { min_w: 800, min_h: 400 }, "wide"), (Need { min_w: 400, min_h: 400 }, "medium")];
    assert_eq!(pick(Rect::new(0, 0, 1280, 720), &choices, "stack"), "wide");
    assert_eq!(pick(Rect::new(0, 0, 800, 400), &choices, "stack"), "wide", "a Need met exactly fits");
    assert_eq!(pick(Rect::new(0, 0, 799, 400), &choices, "stack"), "medium");
    assert_eq!(pick(Rect::new(0, 0, 1280, 399), &choices, "stack"), "stack", "both sides count");
    assert_eq!(pick(Rect::new(500, 500, 400, 400), &choices, "stack"), "medium", "any rect, wherever it is");
    assert_eq!(pick(Rect::new(0, 0, 1280, 720), &[], "stack"), "stack");
}

#[test]
fn the_grid_size_is_only_ever_the_area_given() {
    // The same screen on any canvas holds the same relations; nothing here reads a fixed size.
    for (w, h) in [(640, 360), (1280, 720), (1280, 800), (1720, 720), (800, 600), (390, 844), (2560, 1440)] {
        let (area, rail, list, detail) = worked_example(Rect::new(0, 0, w, h));
        assert_eq!((rail.x, rail.y), (area.x, area.y));
        assert_eq!(list.x, right(rail) + 12);
        assert_eq!(detail.x, right(list) + 12);
        assert_eq!(right(detail), right(area));
        assert_eq!(rail.w, area.w * 186 / 1000);
        assert!(list.w.abs_diff(2 * detail.w) <= 2, "{w} x {h}: 2 to 1, within the rounding");
    }
}

#[test]
fn rect_keeps_its_older_methods() {
    let r = Rect::new(10, 20, 30, 40);
    assert!(r.contains(10, 20) && r.contains(39, 59));
    assert!(!r.contains(40, 20) && !r.contains(10, 60) && !r.contains(9, 20));
    assert_eq!(r.inset(5), Rect::new(15, 25, 20, 30));
    assert_eq!(r.inset(100), Rect::new(110, 120, 0, 0), "inset keeps today's position when it collapses");
    assert_eq!(r.intersect(Rect::new(30, 40, 100, 100)), Rect::new(30, 40, 10, 20));
    assert_eq!(r.intersect(Rect::new(500, 500, 1, 1)).w, 0);
    assert_eq!(Rect::default(), Rect::new(0, 0, 0, 0));
}

#[test]
fn a_stack_rect_asking_for_nothing_takes_no_room_and_no_gap() {
    let area = Rect::new(0, 0, 10, 100);
    for empty in [Part::from(0), Part::from(-3), frac(0), fill(1), fill(0)] {
        let mut full = Stack::new(area, 4);
        full.next(100);
        let none = full.next(empty);
        assert_eq!(none.h, 0, "{empty:?}");
        assert!(none.y >= area.y && bottom(none) <= bottom(area), "{empty:?}: inside the area");
        assert_eq!(full.overflow(), 0, "{empty:?}: exactly full is not overflow");
        full.next(5);
        assert_eq!(full.overflow(), 4 + 5, "{empty:?}: a real rect after it counts one gap");
    }
    // On a fresh stack a fill with weight has the whole area to take, so it is not on this list.
    for empty in [Part::from(0), Part::from(-3), frac(0), fill(0)] {
        let mut first = Stack::new(area, 4);
        first.next(empty);
        assert_eq!(first.next(10), Rect::new(0, 0, 10, 10), "{empty:?}: still no gap before the first real rect");
    }
    let mut no_room_for_the_gap = Stack::new(area, 4);
    no_room_for_the_gap.next(98);
    assert_eq!(no_room_for_the_gap.next(fill(1)).h, 0);
    assert_eq!(no_room_for_the_gap.overflow(), 0, "a gap that does not fit, before nothing, is not overflow");
}

#[test]
fn a_stack_share_is_of_the_whole_area_and_a_split_share_of_what_tokens_leave() {
    let area = Rect::new(0, 0, 10, 1000);
    let mut s = Stack::new(area, 0);
    let (a, b, c) = (s.next(40), s.next(frac(500)), s.next(frac(500)));
    assert_eq!((a.h, b.h, c.h), (40, 500, 0));
    assert_eq!(s.overflow(), 40);
    let cells = area.split_y(&[40.into(), frac(500), frac(500)], 0);
    assert_eq!((cells[0].h, cells[1].h, cells[2].h), (40, 480, 480));
    assert_eq!(bottom(cells[2]), bottom(area));
}

#[test]
fn pad_past_the_width_with_a_negative_far_side_ends_on_the_grown_edge() {
    let p = Rect::new(10, 20, 10, 10);
    let r = p.pad(Insets { l: 20, t: 20, r: -15, b: -15 });
    assert_eq!(r, Rect::new(30, 40, 5, 5), "the start is the whole inset, the far edge the rect's plus 15");
    let r = p.pad(Insets { l: 30, t: 30, r: -5, b: -5 });
    assert_eq!((r.w, r.h), (0, 0));
    assert_eq!((r.x, r.y), (right(p) + 5, bottom(p) + 5), "no size left: on the grown far edge");
}

#[test]
fn frac_is_a_ratio_over_a_thousand() {
    assert_eq!(frac(186), ratio(186, 1000));
    assert_eq!(frac(u16::MAX), ratio(u16::MAX, 1000));
}

#[test]
fn ratios_totalling_the_whole_end_on_the_edge_at_every_width() {
    for room in 0..=2000 {
        let width = room;
        // Thirds, sevenths, and a mix of denominators that per mille cannot state: boundaries are the exact total, rounded
        // down once.
        let thirds = resolve(&[ratio(1, 3), ratio(1, 3), ratio(1, 3)], room);
        assert_eq!(thirds, [room / 3, room * 2 / 3 - room / 3, room - room * 2 / 3], "width {width}");
        let mixed = resolve(&[ratio(1, 3), frac(250), ratio(5, 12)], room);
        assert_eq!(mixed, [room / 3, room * 7 / 12 - room / 3, room - room * 7 / 12], "width {width}");
        let sevenths = resolve(&[ratio(3, 7), fill(1)], room);
        assert_eq!(sevenths, [room * 3 / 7, room - room * 3 / 7], "width {width}");
        let cells = Rect::new(5, 0, width, 9).split_x(&[ratio(1, 3), ratio(2, 3)], 0);
        assert_eq!(cells[1].x, right(cells[0]), "width {width}: a shared edge");
        assert_eq!(right(cells[1]), 5 + width, "width {width}");
    }
}

#[test]
fn a_ratio_past_the_whole_or_over_nothing_gets_zero() {
    assert_eq!(resolve(&[ratio(2, 3), ratio(2, 3), ratio(1, 3)], 300), [200, 0, 100], "the later share still gets its turn");
    assert_eq!(resolve(&[ratio(3, 2)], 300), [0]);
    assert_eq!(resolve(&[ratio(1, 0), fill(1)], 300), [0, 300], "a zero denominator asks for nothing");
    assert_eq!(ratio(1, 0), ratio(0, 1));
    assert_eq!(resolve(&[ratio(0, 7), fill(1)], 300), [0, 300]);
    let p = Rect::new(10, 20, 90, 60);
    assert_eq!(p.place(Anchor::Center, ratio(3, 2), ratio(5, 1)), p, "`place` clamps it to the parent, as it does a frac");
}

#[test]
fn a_running_total_too_fine_to_hold_is_refused_not_wrapped() {
    // Primes under 65,536: nothing cancels, so the denominator is their product and passes 2^90 at the sixth.
    let primes = [65521u16, 65519, 65497, 65479, 65449, 65447];
    let product: u128 = primes[..5].iter().map(|&p| u128::from(p)).product();
    assert!(product <= 1 << 90 && product * u128::from(primes[5]) > 1 << 90, "the list must straddle the limit");
    let parts: Vec<Part> = primes.iter().map(|&p| ratio(1, p)).chain([fill(1)]).collect();
    let room = i32::MAX;
    let sizes = resolve(&parts, room);
    for (i, p) in primes[..5].iter().enumerate() {
        let near = i32::MAX / i32::from(*p);
        assert!(sizes[i].abs_diff(near) <= 1, "share {i}: {} against {near}", sizes[i]);
    }
    assert_eq!(sizes[5], 0, "refused, as a share past the whole is");
    assert_eq!(sizes.iter().map(|&s| i64::from(s)).sum::<i64>(), i64::from(room), "the fill still ends on the edge");
}

#[test]
fn a_floor_holds_a_share_at_its_least() {
    assert_eq!(resolve(&[frac(100).min(30)], 200), [30], "a share under its floor");
    assert_eq!(resolve(&[frac(500).min(30)], 200), [100], "a share over its floor is untouched");
    assert_eq!(resolve(&[ratio(1, 3).min(150)], 300), [150], "on a ratio");
    assert_eq!(resolve(&[Part::from(10).min(25), fill(1)], 100), [25, 75], "on a token");
    assert_eq!(resolve(&[Part::from(40).min(25), fill(1)], 100), [40, 60]);
    assert_eq!(resolve(&[Part::from(-5).min(25), fill(1)], 100), [25, 75], "a negative token still has its floor");
    assert_eq!(frac(100).min(-7), frac(100), "a negative floor is none");
    assert_eq!(frac(100).min(0), frac(100));
    assert_eq!(frac(100).min(9).min(4), frac(100).min(9), "of two floors the larger holds");
    assert_eq!(frac(100).min(4).min(9), frac(100).min(9));
    assert_eq!(resolve(&[fill(1).min(500), 20.into()], 100), [80, 20], "a fill ignores it");
}

#[test]
fn the_extra_for_a_floor_comes_from_the_next_shares_then_the_fills() {
    // The running total's boundaries stay where they are, so a floor only moves edges until the next boundary passes it.
    assert_eq!(resolve(&[frac(300).min(400), frac(200), frac(500)], 1000), [400, 100, 500]);
    assert_eq!(resolve(&[frac(100).min(400), frac(200), frac(700)], 1000), [400, 0, 600], "past the next boundary too");
    assert_eq!(resolve(&[frac(100).min(400), frac(100).min(50), frac(800)], 1000), [400, 50, 550]);
    assert_eq!(resolve(&[frac(100).min(400), fill(1), fill(1)], 1000), [400, 300, 300]);
    assert_eq!(resolve(&[100.into(), frac(100).min(400), fill(1)], 1000), [100, 400, 500], "the token comes first");
}

#[test]
fn floors_keep_shared_edges_and_the_total_at_every_width() {
    for width in 0..=1500 {
        let floor = 120;
        let parts = [ratio(1, 7).min(floor), frac(100), ratio(1, 3).min(floor), frac(100).min(floor), fill(1)];
        for gap in [0, 5] {
            let area = Rect::new(3, 1, width, 7);
            let cells = area.split_x(&parts, gap);
            let room = width - gap * 4;
            if room < 0 {
                continue;
            }
            for pair in cells.windows(2) {
                assert_eq!(pair[1].x, right(pair[0]) + gap, "width {width}, gap {gap}");
            }
            assert_eq!(right(cells[4]), right(area), "width {width}, gap {gap}: the fill ends on the edge");
            assert!(cells.iter().all(|c| c.w >= 0), "width {width}, gap {gap}");
            for i in [0, 2, 3] {
                assert!(cells[i].w == 0 || cells[i].w >= floor, "width {width}, gap {gap}: part {i} is {}", cells[i].w);
            }
        }
        // With no fill, shares totalling the whole still end on the edge once the room holds the floor.
        let sizes = resolve(&[frac(100).min(floor), frac(400), frac(500)], width);
        if width >= floor {
            assert_eq!(sizes[0], floor.max(width / 10), "width {width}");
            assert_eq!(sizes.iter().sum::<i32>(), width, "width {width}");
        }
    }
}

#[test]
fn no_floor_that_does_not_bind_changes_a_list() {
    for width in 0..=1500 {
        let plain = resolve(&[frac(333), frac(333), 12.into(), fill(1), frac(334)], width);
        let floored = resolve(&[frac(333).min(0), frac(333).min(-4), Part::from(12).min(12), fill(1).min(9000), frac(334)], width);
        assert_eq!(plain, floored, "width {width}");
    }
}

#[test]
fn a_floor_that_cannot_fit_is_zero_and_the_rest_still_get_their_turn() {
    assert_eq!(resolve(&[frac(500).min(600)], 500), [0]);
    assert_eq!(resolve(&[Part::from(10).min(600), 20.into()], 500), [0, 20], "a token's floor");
    assert_eq!(resolve(&[frac(500).min(600), frac(500)], 500), [0, 500], "the total runs on, so the next ends on the edge");
    assert_eq!(resolve(&[frac(500).min(600), fill(1)], 500), [0, 500]);
    assert_eq!(resolve(&[frac(500), frac(500).min(300)], 600), [300, 300], "exactly fits");
    assert_eq!(resolve(&[frac(500), frac(500).min(301)], 600), [300, 0]);
    let whole = Rect::new(10, 20, 100, 50);
    let mut c = whole;
    assert_eq!(c.cut_left(frac(100).min(101)).w, 0, "a cut");
    assert_eq!(c, whole, "and the rest stays");
    assert_eq!(c.cut_top(frac(100).min(30)).h, 30, "one that fits cuts its floor");
    assert_eq!(c.y, 50);
}

#[test]
fn place_and_at_clamp_a_floor_to_the_parent() {
    let p = Rect::new(10, 20, 100, 60);
    assert_eq!(p.place(Anchor::Center, frac(100).min(500), frac(100).min(500)), p, "a floor past the parent");
    assert_eq!(p.place(Anchor::Center, Part::from(10).min(500), Part::from(10).min(500)), p, "on a token");
    assert_eq!(p.place(Anchor::TopLeft, frac(100).min(40), ratio(1, 3).min(30)), Rect::new(10, 20, 40, 30));
    assert_eq!(p.place(Anchor::TopLeft, frac(500).min(40), ratio(1, 3)), Rect::new(10, 20, 50, 20), "not binding");
    assert_eq!(p.at(1000, 1000, Anchor::TopLeft, frac(100).min(40), 10), Rect::new(70, 70, 40, 10), "stays inside");
}

#[test]
fn a_floor_in_a_stack_only_adds_and_past_the_area_is_overflow() {
    let area = Rect::new(0, 0, 10, 1000);
    let mut s = Stack::new(area, 0);
    let (a, b) = (s.next(frac(100).min(300)), s.next(frac(700)));
    assert_eq!((a.h, b.y, b.h), (300, 300, 700));
    assert_eq!(s.overflow(), 0);
    let mut s = Stack::new(area, 0);
    let (a, b) = (s.next(frac(500).min(600)), s.next(frac(500)));
    assert_eq!((a.h, b.h), (600, 0));
    assert_eq!(s.overflow(), 100, "the floor's extra is reported, not taken from a neighbour");
    let mut s = Stack::new(area, 0);
    assert_eq!(s.next(Part::from(10).min(25)).h, 25, "on a token");
    assert_eq!(s.next(frac(100).min(1200)).h, 0, "a floor past the area is 0");
    assert_eq!(s.overflow(), 25 + 1200 - 1000);
}

#[test]
fn a_stack_of_ratios_ends_on_the_edge_at_every_length() {
    for h in 0..=2000 {
        let area = Rect::new(0, 3, 10, h);
        let mut s = Stack::new(area, 0);
        let cells = [s.next(ratio(1, 3)), s.next(frac(250)), s.next(ratio(5, 12))];
        assert_eq!((cells[0].h, cells[1].y, cells[2].y), (h / 3, 3 + h / 3, 3 + h * 7 / 12), "h {h}");
        assert_eq!(bottom(cells[2]), bottom(area), "h {h}");
        assert_eq!(s.overflow(), 0, "h {h}");
    }
}

/// A rect with its axes swapped.
fn swapped(r: Rect) -> Rect {
    Rect::new(r.y, r.x, r.h, r.w)
}

#[test]
fn a_row_is_a_stack_turned_on_its_side() {
    let lists: [&[Part]; 6] = [
        &[Part::from(20), Part::from(30), fill(1)],
        &[frac(333), frac(333), frac(334)],
        &[ratio(1, 3), Part::from(0), ratio(2, 3).min(40)],
        &[Part::from(90), Part::from(20), fill(1), Part::from(1)],
        &[frac(600), frac(600), frac(300)],
        &[Part::from(-3), fill(0), frac(0), Part::from(5).min(70)],
    ];
    for parts in lists {
        for len in [0, 1, 7, 99, 100, 101, 1000] {
            for gap in [0, 4] {
                let down_area = Rect::new(5, 10, 80, len);
                let mut down = Stack::new(down_area, gap);
                let mut across = Stack::row(swapped(down_area), gap);
                for &part in parts {
                    assert_eq!(across.next(part), swapped(down.next(part)), "{parts:?}, length {len}, gap {gap}");
                    assert_eq!(across.overflow(), down.overflow(), "{parts:?}, length {len}, gap {gap}");
                }
            }
        }
    }
}

#[test]
fn a_row_runs_across_one_gap_apart_and_counts_its_overflow_in_width() {
    let area = Rect::new(5, 10, 100, 40);
    let mut s = Stack::row(area, 4);
    let (a, b) = (s.next(20), s.next(30));
    assert_eq!(a, Rect::new(5, 10, 20, 40), "no gap before the first, and the area's height");
    assert_eq!((b.x, b.y, b.h), (right(a) + 4, 10, 40));
    let rest = s.next(fill(1));
    assert_eq!(right(rest), right(area), "a fill takes what is left");
    assert_eq!(s.overflow(), 0);
    let late = s.next(50);
    assert_eq!(late.w, 0);
    assert_eq!(s.overflow(), 4 + 50, "past the width, whatever the height");
    let mut tall = Stack::row(Rect::new(0, 0, 10, 500), 0);
    tall.next(11);
    assert_eq!(tall.overflow(), 1, "the height gives it no room");
    let mut wide = Stack::new(Rect::new(0, 0, 500, 10), 0);
    wide.next(11);
    assert_eq!(wide.overflow(), 1, "nor the width to a stack running down");
}

#[test]
fn along_names_the_same_two_stacks() {
    let area = Rect::new(1, 2, 30, 40);
    assert_eq!(Stack::along(area, Axis::Y, 3), Stack::new(area, 3));
    assert_eq!(Stack::along(area, Axis::X, 3), Stack::row(area, 3));
}

#[test]
fn new_parts_at_extreme_numbers_neither_panic_nor_go_negative() {
    let big = [i32::MIN, -1, 0, 1, 1_000_000, i32::MAX];
    for &n in &big {
        for &room in &big {
            let r = Rect::new(room, room, room, room);
            let parts = [
                ratio(u16::MAX, 1),
                ratio(1, u16::MAX),
                ratio(u16::MAX, u16::MAX),
                ratio(u16::MAX, 0),
                frac(500).min(n),
                ratio(1, 3).min(n),
                Part::from(n).min(n),
                fill(u16::MAX).min(n),
            ];
            let sizes = resolve(&parts, room);
            assert!(sizes.iter().all(|&s| s >= 0), "{n}, {room}: {sizes:?}");
            assert!(sizes.iter().map(|&s| i64::from(s)).sum::<i64>() <= i64::from(room.max(0)), "{n}, {room}: {sizes:?}");
            for part in parts {
                let (mut c, mut down, mut across) = (r, Stack::new(r, n), Stack::row(r, n));
                let mut got = vec![r.place(Anchor::Center, part, part), c.cut_top(part), c.cut_right(part), c];
                got.extend([down.next(part), down.next(part), across.next(part), across.next(part)]);
                assert!(got.iter().all(|s| s.w >= 0 && s.h >= 0), "{n}, {room}: {got:?}");
                assert!(down.overflow() >= 0 && across.overflow() >= 0);
            }
        }
    }
}
