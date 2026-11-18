//! ROBUST1 (ENGINE-10): property tests. Thousands of random rects and part lists (a fixed-seed generator, so a failure
//! repeats) against the rules the crate states: sizes never negative and never past the room, a fill ends on the room,
//! cuts share their edge, placed and split rects stay inside their parent.

use aegis_layout::{fill, frac, ratio, resolve, Anchor, Part, Rect, Stack};

/// A small xorshift generator: the crate has no dependencies, its tests take none either.
struct Gen(u64);

impl Gen {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn int(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo + 1) as u64) as i32
    }
    fn part(&mut self) -> Part {
        let p = match self.int(0, 3) {
            0 => Part::from(self.int(-5, 400)),
            1 => frac(self.int(0, 1200) as u16),
            2 => ratio(self.int(0, 5) as u16, self.int(0, 7) as u16),
            _ => fill(self.int(0, 4) as u16),
        };
        if self.int(0, 4) == 0 { p.min(self.int(-3, 120)) } else { p }
    }
    fn parts(&mut self) -> Vec<Part> {
        (0..self.int(0, 7)).map(|_| self.part()).collect()
    }
    fn rect(&mut self) -> Rect {
        Rect::new(self.int(-200, 1500), self.int(-200, 900), self.int(0, 1400), self.int(0, 800))
    }
}

const RUNS: usize = 4000;

fn inside(r: Rect, parent: Rect) -> bool {
    r.w >= 0 && r.h >= 0 && r.x >= parent.x && r.y >= parent.y && r.x + r.w <= parent.x + parent.w && r.y + r.h <= parent.y + parent.h
}

#[test]
fn resolved_sizes_are_never_negative_never_past_the_room_and_a_fill_ends_on_it() {
    let mut g = Gen(0x9e37_79b9_7f4a_7c15);
    for run in 0..RUNS {
        let parts = g.parts();
        let room = g.int(-10, 2000);
        let sizes = resolve(&parts, room);
        assert_eq!(sizes.len(), parts.len());
        assert!(sizes.iter().all(|s| *s >= 0), "run {run}: {parts:?} over {room}: {sizes:?}");
        let total: i64 = sizes.iter().map(|s| i64::from(*s)).sum();
        assert!(total <= i64::from(room.max(0)), "run {run}: {parts:?} over {room}: {sizes:?} totals {total}");
        let has_fill = parts.iter().any(|p| *p == fill(1) || *p == fill(2) || *p == fill(3) || *p == fill(4));
        if has_fill {
            assert_eq!(total, i64::from(room.max(0)), "run {run}: a fill ends on the room: {parts:?} over {room}: {sizes:?}");
        }
    }
}

#[test]
fn a_cut_and_what_is_left_share_their_edge_and_stay_inside() {
    let mut g = Gen(0x2545_f491_4f6c_dd1d);
    for run in 0..RUNS {
        let parent = g.rect();
        let part = g.part();
        for side in 0..4 {
            let mut rest = parent;
            let band = match side {
                0 => rest.cut_top(part),
                1 => rest.cut_bottom(part),
                2 => rest.cut_left(part),
                _ => rest.cut_right(part),
            };
            assert!(inside(band, parent) && inside(rest, parent), "run {run} side {side}: {parent:?} {part:?} -> {band:?} + {rest:?}");
            if side < 2 {
                assert_eq!(band.h + rest.h, parent.h, "run {run}: heights add up");
            } else {
                assert_eq!(band.w + rest.w, parent.w, "run {run}: widths add up");
            }
            let touching = match side {
                0 => band.y + band.h == rest.y,
                1 => rest.y + rest.h == band.y,
                2 => band.x + band.w == rest.x,
                _ => rest.x + rest.w == band.x,
            };
            assert!(touching, "run {run} side {side}: the two share their edge: {band:?} {rest:?}");
        }
    }
}

#[test]
fn placed_split_and_stacked_rects_stay_inside_their_parent_in_order() {
    let anchors = [Anchor::TopLeft, Anchor::Top, Anchor::TopRight, Anchor::Left, Anchor::Center, Anchor::Right, Anchor::BottomLeft, Anchor::Bottom, Anchor::BottomRight];
    let mut g = Gen(0x5851_f42d_4c95_7f2d);
    for run in 0..RUNS {
        let parent = g.rect();
        let at = anchors[g.int(0, 8) as usize];
        let (w, h) = (g.part(), g.part());
        let placed = parent.place(at, w, h);
        assert!(inside(placed, parent), "run {run}: place {at:?} {w:?} {h:?} in {parent:?} -> {placed:?}");
        let parts = g.parts();
        let gap = g.int(-4, 40);
        for (cells, across) in [(parent.split_x(&parts, gap), true), (parent.split_y(&parts, gap), false)] {
            assert_eq!(cells.len(), parts.len());
            for pair in cells.windows(2) {
                let ordered = if across { pair[0].x + pair[0].w <= pair[1].x } else { pair[0].y + pair[0].h <= pair[1].y };
                assert!(ordered, "run {run}: cells in order and apart: {pair:?}");
            }
            assert!(cells.iter().all(|c| inside(*c, parent)), "run {run}: {parts:?} gap {gap} in {parent:?} -> {cells:?}");
        }
        let mut stack = Stack::new(parent, gap.max(0));
        for _ in 0..g.int(0, 8) {
            let r = stack.next(g.part());
            assert!(r.w >= 0 && r.h >= 0, "run {run}: {r:?}");
            assert!(inside(r, parent) || r.h == 0, "run {run}: a stacked row is inside or empty: {r:?} in {parent:?}");
        }
        assert!(stack.overflow() >= 0);
    }
}
