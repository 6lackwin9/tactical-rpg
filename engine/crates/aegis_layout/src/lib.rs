//! Relative layout on whole grid units, game-agnostic (LAYOUT720-E2, docs/specs/LAYOUT720.md section 6; new, no Godot
//! file). A screen states anchors, shares, fills and tokens; this crate resolves them to `Rect` edges for whatever area
//! it is given, so changing a canvas needs no screen edit. The grid's size is never named here: it only ever arrives as
//! the area a caller passes in.
//!
//! `no_std` with `alloc`, no dependencies and whole numbers only: the crate cannot reach a file, thread, socket or clock,
//! and it gives the same answer on every platform.
//!
//! The rounding rule: resolve shared edges, not sizes. A split works out each boundary from its running total, rounded
//! down once, and a cell's size is the difference between two boundaries, so neighbours read one number for their shared
//! edge and the total always equals the room. `resolve` works out sizes in one place; the one exception is `Stack::next`,
//! which reads tokens and shares itself so `overflow` can count what did not fit.
//!
//! Inputs to the relative methods (`pad`, `place`, `at`, `offset`, `cut_*`, `split_*`, `grid`), `resolve` and `Stack` are
//! bounded: a size that cannot fit becomes 0 and nothing panics, whatever numbers come in. Their arithmetic is done in
//! `i64` (`i128` for a product of three) and clamped back. `contains`, `inset` and `intersect` keep their older plain
//! arithmetic, so this does not cover them.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

/// The most cells `Rect::grid` lays out; above it the answer is empty rather than a huge allocation.
pub const MAX_GRID_CELLS: i64 = 65_536;

/// A share of `PER_MILLE` is the whole parent.
const PER_MILLE: i64 = 1000;

/// The largest denominator a running total of shares may reach. It keeps `Shares::of` inside `i128` (an `i32` room times
/// this), and a share that would pass it gets 0, as one past the whole does: it takes several large prime denominators in
/// one list to get here.
const MAX_SHARE_DENOMINATOR: i128 = 1 << 90;

/// A rectangle in whole grid units. Its size may be stored negative by a direct `Rect::new`; the relative methods read
/// that as 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
    pub fn inset(&self, d: i32) -> Rect {
        Rect::new(self.x + d, self.y + d, (self.w - 2 * d).max(0), (self.h - 2 * d).max(0))
    }
    /// The overlap of two rects (zero size when they do not meet).
    pub fn intersect(&self, o: Rect) -> Rect {
        let (x0, y0) = (self.x.max(o.x), self.y.max(o.y));
        let (x1, y1) = ((self.x + self.w).min(o.x + o.w), (self.y + self.h).min(o.y + o.h));
        Rect::new(x0, y0, (x1 - x0).max(0), (y1 - y0).max(0))
    }

    /// The rect less `by` on each side. Padding that cannot fit gives size 0, still inside the rect. A negative inset
    /// grows it (a frame that straddles its panel).
    pub fn pad(self, by: Insets) -> Rect {
        let (x, w) = shrink(self.x, self.w, by.l, by.r);
        let (y, h) = shrink(self.y, self.h, by.t, by.b);
        Rect::new(x, y, w, h)
    }

    /// An element at one of the rect's nine points. Its size is clamped to the rect and it stays inside; a centred odd
    /// remainder rounds toward the top left. The same as `at` on that point.
    pub fn place(self, at: Anchor, w: impl Into<Part>, h: impl Into<Part>) -> Rect {
        let (ax, ay) = at.halves();
        self.at((ax * 500) as u16, (ay * 500) as u16, at, w, h)
    }

    /// An element whose `at` point sits `fx`, `fy` per mille across the rect. It is clamped to the rect like `place`; the
    /// start is rounded down once and the size added, so a lone element has one rounding only.
    pub fn at(self, fx: u16, fy: u16, at: Anchor, w: impl Into<Part>, h: impl Into<Part>) -> Rect {
        let (ax, ay) = at.halves();
        let (x, w) = along(self.x, self.w, fx, ax, w.into());
        let (y, h) = along(self.y, self.h, fy, ay, h.into());
        Rect::new(x, y, w, h)
    }

    /// The rect moved; size unchanged, nothing clamped.
    pub fn offset(self, dx: i32, dy: i32) -> Rect {
        Rect::new(add(self.x, dx.into()), add(self.y, dy.into()), self.w, self.h)
    }

    /// Cuts a band off the top and returns it; `self` keeps the rest, so the two share their edge. A share is of `self`'s
    /// height now, a fill takes it all, and a token that cannot fit gives a 0-high band and leaves `self` as it is.
    pub fn cut_top(&mut self, h: impl Into<Part>) -> Rect {
        let (band, rest) = cut(self.h, h.into());
        let top = Rect::new(self.x, self.y, self.w.max(0), band);
        (self.y, self.h) = (add(self.y, band.into()), rest);
        top
    }
    /// As `cut_top`, off the bottom.
    pub fn cut_bottom(&mut self, h: impl Into<Part>) -> Rect {
        let (band, rest) = cut(self.h, h.into());
        self.h = rest;
        Rect::new(self.x, add(self.y, rest.into()), self.w.max(0), band)
    }
    /// As `cut_top`, off the left.
    pub fn cut_left(&mut self, w: impl Into<Part>) -> Rect {
        let (band, rest) = cut(self.w, w.into());
        let left = Rect::new(self.x, self.y, band, self.h.max(0));
        (self.x, self.w) = (add(self.x, band.into()), rest);
        left
    }
    /// As `cut_top`, off the right.
    pub fn cut_right(&mut self, w: impl Into<Part>) -> Rect {
        let (band, rest) = cut(self.w, w.into());
        self.w = rest;
        Rect::new(add(self.x, rest.into()), self.y, band, self.h.max(0))
    }

    /// Side-by-side cells, `gap` apart (a negative gap is 0), sized by `resolve` over the width less the gaps. The parts
    /// and gaps add up to the width when any part is a fill or the shares total 1,000. Gaps wider than the rect give size
    /// 0 cells, still inside it.
    pub fn split_x(self, parts: &[Part], gap: i32) -> Vec<Rect> {
        let h = self.h.max(0);
        spread(self.x, self.w, parts, gap).into_iter().map(|(x, w)| Rect::new(x, self.y, w, h)).collect()
    }
    /// As `split_x`, stacked downward.
    pub fn split_y(self, parts: &[Part], gap: i32) -> Vec<Rect> {
        let w = self.w.max(0);
        spread(self.y, self.h, parts, gap).into_iter().map(|(y, h)| Rect::new(self.x, y, w, h)).collect()
    }

    /// `cols` x `rows` equal cells, `gap` apart, left to right and then down; cells can differ by 1 unit, and the outer
    /// edges are the rect's. Empty for no cells or above `MAX_GRID_CELLS`.
    pub fn grid(self, cols: i32, rows: i32, gap: i32) -> Vec<Rect> {
        if cols <= 0 || rows <= 0 || i64::from(cols) * i64::from(rows) > MAX_GRID_CELLS {
            return Vec::new();
        }
        let xs = self.split_x(&vec![fill(1); cols as usize], gap);
        let ys = self.split_y(&vec![fill(1); rows as usize], gap);
        ys.iter().flat_map(|r| xs.iter().map(move |c| Rect::new(c.x, r.y, c.w, r.h))).collect()
    }
}

/// Space on each side, in units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Insets {
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
}

impl Insets {
    /// The same on every side.
    pub const fn all(d: i32) -> Self {
        Self { l: d, t: d, r: d, b: d }
    }
    /// `x` on the left and right, `y` on the top and bottom.
    pub const fn xy(x: i32, y: i32) -> Self {
        Self { l: x, t: y, r: x, b: y }
    }
}

/// A point of a rect, and the point of an element placed on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Anchor {
    /// Halves of the way across and down: 0 at the start, 1 in the middle, 2 at the end.
    fn halves(self) -> (i64, i64) {
        match self {
            Anchor::TopLeft => (0, 0),
            Anchor::Top => (1, 0),
            Anchor::TopRight => (2, 0),
            Anchor::Left => (0, 1),
            Anchor::Center => (1, 1),
            Anchor::Right => (2, 1),
            Anchor::BottomLeft => (0, 2),
            Anchor::Bottom => (1, 2),
            Anchor::BottomRight => (2, 2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Units(i32),
    /// `n / d` of the room; `d` is never 0.
    Share { n: u16, d: u16 },
    Fill(u16),
}

/// A size along one side: a share (`frac`, `ratio`), a fill weight (`fill`) or a token's value in units (`From<i32>`),
/// with an optional floor (`min`). Its fields stay private so later work can attach here with no caller changing (a
/// give-way rank; spec section 11.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Part {
    kind: Kind,
    /// The least size in units, never negative; 0 is no floor. A fill does not read it.
    floor: i32,
}

/// A share, per mille of the room it is resolved over. The same as `ratio(per_mille, 1000)`.
pub const fn frac(per_mille: u16) -> Part {
    ratio(per_mille, PER_MILLE as u16)
}

/// A share of `n / d` of the room it is resolved over, for what per mille cannot state ("a third"). Running totals stay
/// exact across any mix of shares, so `ratio(1, 3)` three times ends on the edge. It reads as `frac` does everywhere:
/// past the whole in a list it gets 0. `d` of 0 asks for nothing.
pub const fn ratio(n: u16, d: u16) -> Part {
    let kind = if d == 0 { Kind::Share { n: 0, d: 1 } } else { Kind::Share { n, d } };
    Part { kind, floor: 0 }
}

/// A fill: what is left after tokens and shares, divided by weight.
pub const fn fill(weight: u16) -> Part {
    Part { kind: Kind::Fill(weight), floor: 0 }
}

impl Part {
    /// This size, but never under `units` (a negative floor is none; of two floors the larger holds). For shares and
    /// tokens: a fill is what is left, so it ignores this. In a list the extra comes from the next shares first, then the
    /// fills, so the edges stay shared and the total stays the room. A floor that cannot fit follows the rule of any size
    /// that cannot fit: 0 in `resolve`, the cuts and the splits, 0 and counted in `Stack::overflow`, and clamped to the
    /// parent by `place` and `at`.
    pub const fn min(self, units: i32) -> Part {
        let floor = if units > self.floor { units } else { self.floor };
        Part { kind: self.kind, floor }
    }
}

impl From<i32> for Part {
    /// A token's value, in units.
    fn from(units: i32) -> Self {
        Part { kind: Kind::Units(units), floor: 0 }
    }
}

/// A running total of shares, an exact fraction in lowest terms, at most the whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shares {
    n: i128,
    d: i128,
}

impl Shares {
    const NONE: Shares = Shares { n: 0, d: 1 };

    /// The total with `n / d` added, or `None` when it would pass the whole or `MAX_SHARE_DENOMINATOR`. `d` is above 0.
    fn plus(self, n: u16, d: u16) -> Option<Shares> {
        let (n, d) = (i128::from(n), i128::from(d));
        // `self.d` is at most 2^90 and `d` under 2^16, so nothing below passes 2^107.
        let common = self.d / gcd(self.d, d) * d;
        let sum = self.n * (common / self.d) + n * (common / d);
        let g = gcd(sum, common);
        let (n, d) = (sum / g, common / g);
        (n <= d && d <= MAX_SHARE_DENOMINATOR).then_some(Shares { n, d })
    }

    /// This total of `room`, rounded down once. `room` is non-negative and within `i32`.
    fn of(self, room: i64) -> i64 {
        (i128::from(room) * self.n / self.d) as i64
    }
}

/// Which way a `Stack` runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Left to right.
    X,
    /// Top to bottom.
    Y,
}

/// How roomy a canvas is, per axis. The limits are the caller's: this crate names no breakpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SizeClass {
    Compact,
    Medium,
    Expanded,
}

/// The size an arrangement needs to be chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Need {
    pub min_w: i32,
    pub min_h: i32,
}

/// The size of each part over `room`, in the parts' order: where sizes are worked out, and where later steps attach (spec
/// section 11.1; `Stack::next` has the one exception). Tokens are taken first, in order; then shares of what is left; then fills of what is left after
/// those, by weight. `room` is already less any gaps. Shares and fills come from running totals, so they add up to their
/// room exactly: a fill's last boundary is the room, and shares totalling the whole are all of it. A floor pushes its
/// share's far edge out, and the running total's next boundary past that edge ends the push. A token that does not fit
/// what is left, a share past the whole in total and a floor that does not fit get 0, and the parts after them still get
/// their turn.
pub fn resolve(parts: &[Part], room: i32) -> Vec<i32> {
    let mut sizes = vec![0i64; parts.len()];
    let mut left = i64::from(room.max(0));
    for (size, part) in sizes.iter_mut().zip(parts) {
        if let Kind::Units(n) = part.kind {
            let n = i64::from(n.max(part.floor).max(0));
            if n <= left {
                *size = n;
                left -= n;
            }
        }
    }
    let (share_room, mut total, mut edge) = (left, Shares::NONE, 0i64);
    for (size, part) in sizes.iter_mut().zip(parts) {
        if let Kind::Share { n, d } = part.kind {
            if let Some(sum) = total.plus(n, d) {
                total = sum;
                let next = total.of(share_room).max(edge + i64::from(part.floor));
                if next <= share_room {
                    (*size, edge) = (next - edge, next);
                }
            }
        }
    }
    left -= edge;
    let weights: i64 = parts.iter().map(|p| if let Kind::Fill(w) = p.kind { i64::from(w) } else { 0 }).sum();
    let (mut run, mut edge) = (0i64, 0i64);
    for (size, part) in sizes.iter_mut().zip(parts) {
        if let (Kind::Fill(w), true) = (part.kind, weights > 0) {
            run += i64::from(w);
            let next = mul_div(left, run, weights);
            (*size, edge) = (next - edge, next);
        }
    }
    // Every size is at most `room`, an i32.
    sizes.into_iter().map(|s| s as i32).collect()
}

/// Rects one after another along an axis, cut from an area as they come: `new` runs down, `row` across, `along` either.
/// Tokens as given; a share is of the whole area's length, from a running total, so shares totalling the whole with no gap
/// end on the area's edge (use `Rect::split_x` or `split_y` when shares and gaps must add up); a floor only adds, so past
/// the area it is overflow, and a fill takes all that is left, whatever its weight. A rect that does not fit has 0 length
/// at the area's far end, and its length counts in `overflow`. A rect asking for nothing (a 0 token, or a fill with nothing
/// left) takes no room and no gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stack {
    area: Rect,
    axis: Axis,
    gap: i32,
    used: i64,
    shares: Shares,
    started: bool,
}

impl Stack {
    /// A stack running down.
    pub fn new(area: Rect, gap: i32) -> Self {
        Self::along(area, Axis::Y, gap)
    }

    /// A stack running across, left to right.
    pub fn row(area: Rect, gap: i32) -> Self {
        Self::along(area, Axis::X, gap)
    }

    /// A stack running along `axis`, for a caller that holds the axis as a value.
    pub fn along(area: Rect, axis: Axis, gap: i32) -> Self {
        Self { area, axis, gap: gap.max(0), used: 0, shares: Shares::NONE, started: false }
    }

    /// The area's length along the axis, as no room when stored negative.
    fn full(&self) -> i64 {
        i64::from(match self.axis {
            Axis::X => self.area.w,
            Axis::Y => self.area.h,
        }
        .max(0))
    }

    /// The next rect along the axis; `size` is its length.
    pub fn next(&mut self, size: impl Into<Part>) -> Rect {
        let part = size.into();
        let full = self.full();
        let start = self.used + if self.started { i64::from(self.gap) } else { 0 };
        let left = (full - start).max(0) as i32;
        let floor = i64::from(part.floor);
        // A later rank (spec section 11.1) must apply to the two arms that skip `resolve`: they keep the size that did not
        // fit, because `resolve` would give 0 and `overflow` has to count it.
        let wanted = match part.kind {
            Kind::Units(n) => i64::from(n.max(0)).max(floor),
            Kind::Share { n, d } => match self.shares.plus(n, d) {
                Some(sum) => {
                    let size = sum.of(full) - self.shares.of(full);
                    self.shares = sum;
                    size.max(floor)
                }
                None => mul_div(full, i64::from(n), i64::from(d)).max(floor), // past the whole area: only `overflow` sees it
            },
            Kind::Fill(_) => i64::from(resolve(&[part], left)[0]),
        };
        let size = if start + wanted <= full { wanted as i32 } else { 0 };
        if wanted > 0 {
            self.used = start + wanted;
            self.started = true;
        }
        let at = start.min(full);
        match self.axis {
            Axis::X => Rect::new(add(self.area.x, at), self.area.y, size, self.area.h.max(0)),
            Axis::Y => Rect::new(self.area.x, add(self.area.y, at), self.area.w.max(0), size),
        }
    }

    /// Units past the area, 0 when it fits.
    pub fn overflow(&self) -> i32 {
        (self.used - self.full()).clamp(0, i64::from(i32::MAX)) as i32
    }
}

/// The first choice whose `Need` fits `area`, else `last`. It reads any rect, so a pane can rearrange inside itself.
pub fn pick<T: Copy>(area: Rect, choices: &[(Need, T)], last: T) -> T {
    choices.iter().find(|(need, _)| area.w >= need.min_w && area.h >= need.min_h).map_or(last, |(_, choice)| *choice)
}

/// `a * b / c`, rounded down, for non-negative `a`, `b` and positive `c`.
fn mul_div(a: i64, b: i64, c: i64) -> i64 {
    (i128::from(a) * i128::from(b) / i128::from(c)) as i64
}

/// The greatest common divisor of two non-negative numbers, not both 0.
fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// `a + b`, held inside `i32`.
fn add(a: i32, b: i64) -> i32 {
    (i64::from(a) + b).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// One side of `pad`: the start and size left of `len` after `lo` and `hi`.
fn shrink(start: i32, len: i32, lo: i32, hi: i32) -> (i32, i32) {
    let len = i64::from(len.max(0));
    let size = (len - i64::from(lo) - i64::from(hi)).clamp(0, i64::from(i32::MAX));
    // Never past the far edge, which a negative `hi` moves out.
    (add(start, i64::from(lo).min(len - i64::from(hi.min(0)))), size as i32)
}

/// One side of `at`: the start and size of an element whose point `halves` of the way along it sits `f` per mille along
/// `len`. The whole start is one division, rounded down, then clamped so the element stays inside.
fn along(start: i32, len: i32, f: u16, halves: i64, part: Part) -> (i32, i32) {
    let len = len.max(0);
    // Only what must be clamped changes, so anything a later task adds to `Part` (a rank) still reaches `resolve`.
    let mut capped = part;
    capped.floor = part.floor.min(len);
    capped.kind = match part.kind {
        Kind::Units(n) => Kind::Units(n.clamp(0, len)),
        Kind::Share { n, d } => Kind::Share { n: n.min(d), d },
        Kind::Fill(w) => Kind::Fill(w),
    };
    let size = resolve(&[capped], len)[0];
    let (room, sized) = (i64::from(len), i64::from(size));
    let from = (2 * room * i64::from(f) - PER_MILLE * halves * sized).div_euclid(2 * PER_MILLE);
    (add(start, from.clamp(0, room - sized)), size)
}

/// One side of a cut: the band's size and what stays of `len`.
fn cut(len: i32, part: Part) -> (i32, i32) {
    let len = len.max(0);
    let band = resolve(&[part], len)[0];
    (band, len - band)
}

/// One side of a split: each cell's start and size. The boundaries of `resolve` plus the gaps, so a neighbour's start is
/// always this cell's end plus the gap.
fn spread(start: i32, len: i32, parts: &[Part], gap: i32) -> Vec<(i32, i32)> {
    let (len, gap) = (i64::from(len.max(0)), i64::from(gap.max(0)));
    let gaps = gap.saturating_mul(parts.len().saturating_sub(1) as i64);
    let room = (len - gaps).max(0) as i32;
    let mut run = 0i64;
    resolve(parts, room)
        .into_iter()
        .map(|size| {
            let cell = (add(start, run.min(len)), size);
            run += i64::from(size) + gap;
            cell
        })
        .collect()
}
