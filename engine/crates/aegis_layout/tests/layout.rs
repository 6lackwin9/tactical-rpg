//! LAYOUT720-E2: the crate's rules (docs/specs/LAYOUT720.md 6.1) and the worked example of 6.3. Expectations come from the
//! rules or the spec's table, never from a printed result.

use aegis_layout::{fill, frac, pick, resolve, Anchor, Insets, Need, Part, Rect, Stack, MAX_GRID_CELLS};

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
