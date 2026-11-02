//! A pip gauge (INPUT1-AP, reworked by INPUT2 to the user's style D; engine-only, no Godot original): a fixed row of
//! slots for a budget planned against, one slot a point (or `unit` points when the maximum is above the gauge's cap).
//! The row never moves or resizes with the plan: a slot per point up to the maximum, a uniform gap. Each planned block
//! is a capsule over its slots, filled with the colour its caller gives (a ghost block at the timeline's
//! `GHOST_ALPHA`), with its own bright rim (so two neighbouring blocks of one colour read as two) and thin darker ticks
//! between its slots. The points held and not planned are dark recessed sockets with a light outline; the slots above
//! the points held (up to the maximum) are dim crossed slots. The gauge can turn the outlines to its alert colour (a pick
//! the budget cannot pay) and carries a short label before its first slot (a debt) or after its last (a shortfall).
//! Levels are counted in slots and may be fractional while a screen animates them.
//! A screen builds a `PipGauge` from its own numbers each frame and draws it; it pushes one info widget (its reading in
//! pip codes, the hover card the screen's). The widget names no game: the block colours are the caller's. `samples` are
//! the UI Gallery's states.

use crate::canvas::{rgba, Rect, Rgba};
use crate::theme::th;
use crate::timeline::GHOST_ALPHA;
use crate::ui::{Tip, Ui, SMALL};

/// One slot's size and the gap between slots (canvas pixels): 12 slots take 71.
pub const PIP_W: i32 = 5;
pub const PIP_H: i32 = 8;
pub const PIP_GAP: i32 = 1;
/// The gap between a label and the slots.
pub const LABEL_GAP: i32 = 2;
/// How far a block's rim goes toward white and its ticks toward black.
pub const RIM_LIGHT: f32 = 0.55;
pub const TICK_DARK: f32 = 0.45;

/// The points a slot stands for: one, or `ceil(max / cap)` when `max` is above `cap` (the gauge's most slots).
pub fn unit(max: i32, cap: i32) -> i32 {
    if cap <= 0 || max <= cap {
        1
    } else {
        (max + cap - 1) / cap
    }
}

/// `n` points in slots of `unit` points, rounded up (none for `n <= 0`).
pub fn count(n: i32, unit: i32) -> i32 {
    let u = unit.max(1);
    if n <= 0 {
        0
    } else {
        (n + u - 1) / u
    }
}

/// The width of `slots` slots side by side.
pub fn pips_w(slots: i32) -> i32 {
    if slots <= 0 {
        0
    } else {
        slots * (PIP_W + PIP_GAP) - PIP_GAP
    }
}

/// Slot `k`'s rect (0 = the first) for a row starting at `x`, centred on `cy`: fixed whatever the plan.
pub fn pip_rect(x: i32, cy: i32, k: i32) -> Rect {
    Rect::new(x + k * (PIP_W + PIP_GAP), cy - PIP_H / 2, PIP_W, PIP_H)
}

/// A planned block: its points, its fill colour (the caller's) and whether it is a ghost (drawn at `GHOST_ALPHA`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PipBlock {
    pub ap: i32,
    pub colour: Rgba,
    pub ghost: bool,
}

impl PipBlock {
    pub fn new(ap: i32, colour: Rgba) -> PipBlock {
        PipBlock { ap, colour, ghost: false }
    }

    pub fn ghost(ap: i32, colour: Rgba) -> PipBlock {
        PipBlock { ap, colour, ghost: true }
    }
}

/// A slot as read (settled levels): in planned block `n` (1 = the first), a free socket (held, not planned), a crossed
/// slot (above the points held).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pip {
    Block(usize),
    Free,
    Crossed,
}

impl Pip {
    /// The slot's short code: "B1", "B2", ... for a block's slot, "O" a free socket, "X" a crossed slot.
    pub fn code(self) -> String {
        match self {
            Pip::Block(n) => format!("B{n}"),
            Pip::Free => "O".into(),
            Pip::Crossed => "X".into(),
        }
    }
}

/// The gauge's chrome colours: a socket's fill and outline, a crossed slot's outline and cross, the alert (outlines
/// and labels). `default` reads the theme in force (LOOK1-E1B); a block's own colour is the caller's (a meaning, such as
/// the skill kind, not chrome), and its ticks and rim are that colour mixed toward black and white, so they read on any
/// colour under any theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colours {
    pub socket: Rgba,
    pub socket_edge: Rgba,
    pub crossed: Rgba,
    pub alert: Rgba,
}

impl Default for Colours {
    fn default() -> Self {
        let t = th();
        Colours { socket: t.socket, socket_edge: t.socket_edge, crossed: rgba(t.dim[0], t.dim[1], t.dim[2], 110), alert: t.foe }
    }
}

/// One block as laid out: its first slot (0-based), its slots, its colour and ghost flag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Laid {
    pub from: i32,
    pub slots: i32,
    pub colour: Rgba,
    pub ghost: bool,
}

/// A pip gauge. Slots count from the left: the blocks fill slots `1..=spent` in order, free sockets follow up to
/// `edge` (the points held), crossed slots above it. `Default` is an empty gauge.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PipGauge {
    /// Slots drawn (the maximum in slots).
    pub slots: i32,
    /// The points a slot stands for (1 or more).
    pub unit: i32,
    /// The blocks as laid out (slots counted from the cumulative points, so rounding never adds slots).
    pub blocks: Vec<Laid>,
    /// The slots the blocks show (their total; lower while a screen animates them in).
    pub spent: f32,
    /// The slots held (sockets end here, crossed slots follow).
    pub edge: f32,
    /// Every outline in the alert colour.
    pub alert: bool,
    /// A label before the first slot and one after the last, in the alert colour.
    pub before: Option<String>,
    pub after: Option<String>,
    pub colours: Colours,
    /// The hover card (the screen's numbers).
    pub tip: Tip,
}

/// Lays `blocks` out from slot 0 (points in slots of `u`, from the running total, at most `slots`); blocks rounding
/// to no slot are dropped.
fn lay(blocks: &[PipBlock], u: i32, slots: i32) -> Vec<Laid> {
    let (mut sum, mut at) = (0, 0);
    let mut out = Vec::new();
    for b in blocks.iter().filter(|b| b.ap > 0) {
        sum += b.ap;
        let end = count(sum, u).min(slots);
        if end > at {
            out.push(Laid { from: at, slots: end - at, colour: b.colour, ghost: b.ghost });
            at = end;
        }
    }
    out
}

impl PipGauge {
    /// A budget being planned, in points: `now` held, `blocks` the plan in order, `max`, at most `cap` slots. The blocks
    /// fill the first slots, free sockets follow up to `now`, crossed slots to `max`. A debt (`now < 0`) draws every slot
    /// crossed and "-<debt>" before them.
    pub fn planned(now: i32, blocks: &[PipBlock], max: i32, cap: i32) -> PipGauge {
        let u = unit(max, cap);
        let slots = count(max, u);
        if now < 0 {
            return PipGauge { slots, unit: u, before: Some(format!("-{}", -now)), ..PipGauge::default() };
        }
        let laid = lay(blocks, u, slots);
        let spent = laid.last().map_or(0, |l| l.from + l.slots);
        PipGauge { slots, unit: u, blocks: laid, spent: spent as f32, edge: count(now, u).min(slots) as f32, ..PipGauge::default() }
    }

    /// A budget being spent, in points: `start` held when the spending began, `have` now, `max`, at most `cap` slots:
    /// what was paid (`start - have`) is one block in `paid`, free sockets hold `have`, crossed slots go to `max`. A debt
    /// as in `planned`.
    pub fn live(start: i32, have: i32, paid: Rgba, max: i32, cap: i32) -> PipGauge {
        let u = unit(max, cap);
        let slots = count(max, u);
        if have < 0 {
            return PipGauge { slots, unit: u, before: Some(format!("-{}", -have)), ..PipGauge::default() };
        }
        let held = start.max(have);
        let edge = count(held, u).min(slots);
        let paid_slots = (edge - count(have, u)).max(0);
        let blocks = if paid_slots > 0 { vec![Laid { from: 0, slots: paid_slots, colour: paid, ghost: false }] } else { Vec::new() };
        PipGauge { slots, unit: u, blocks, spent: paid_slots as f32, edge: edge as f32, ..PipGauge::default() }
    }

    /// The gauge with a pick `short` points beyond the budget: every outline in the alert colour and "-<short>" after
    /// the last slot (a debt's label before the slots stays; then no label after).
    pub fn short(mut self, short: i32) -> PipGauge {
        self.alert = true;
        if self.before.is_none() && short > 0 {
            self.after = Some(format!("-{short}"));
        }
        self
    }

    /// The gauge with its hover card.
    pub fn with_tip(mut self, tip: Tip) -> PipGauge {
        self.tip = tip;
        self
    }

    /// The block slot `k` (0-based) lies in (its index), if any.
    fn block_at(&self, k: i32) -> Option<usize> {
        self.blocks.iter().position(|b| k >= b.from && k < b.from + b.slots)
    }

    /// Each slot as read (a fractional level counts from its half).
    pub fn pips(&self) -> Vec<Pip> {
        (0..self.slots)
            .map(|k| {
                let lit = |level: f32| level >= k as f32 + 0.5;
                match self.block_at(k) {
                    Some(n) if lit(self.spent) => Pip::Block(n + 1),
                    _ if lit(self.edge) => Pip::Free,
                    _ => Pip::Crossed,
                }
            })
            .collect()
    }

    /// The slots' codes, space separated ("B1 B1 B2 O X").
    pub fn reading(&self) -> String {
        self.pips().iter().map(|p| p.code()).collect::<Vec<_>>().join(" ")
    }

    /// The width the gauge takes with its labels.
    pub fn width<A: Clone>(&self, ui: &mut Ui<A>) -> i32 {
        let mut w = pips_w(self.slots);
        for l in [&self.before, &self.after].into_iter().flatten() {
            w += label_w(ui, l) + LABEL_GAP;
        }
        w
    }

    /// Draws the gauge from `x`, its slots centred on `cy`, and pushes the info widget `id` (kind "pips", its reading,
    /// the hover card); returns the widget's rect.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, id: &str, x: i32, cy: i32) -> Rect {
        let lh = ui.p.line_h(SMALL).ceil() as i32;
        let h = PIP_H.max(lh);
        let mut at = x;
        if let Some(l) = &self.before {
            ui.p.text(at as f32, (cy - lh / 2) as f32, l, SMALL, self.colours.alert);
            at += label_w(ui, l) + LABEL_GAP;
        }
        let c = self.colours;
        // the fixed row: sockets up to the points held, crossed slots above
        for k in 0..self.slots {
            let r = pip_rect(at, cy, k);
            let held = (self.edge - k as f32).clamp(0.0, 1.0);
            let edge = if self.alert { c.alert } else { mix(c.crossed, c.socket_edge, held) };
            if held > 0.0 {
                ui.p.fill(inner(r), faded(c.socket, held));
            }
            if held < 1.0 {
                // the cross: a diagonal from the bottom left to the top right
                let cross = faded(if self.alert { c.alert } else { c.crossed }, 1.0 - held);
                for i in 0..(r.h - 2) {
                    let px = r.x + 1 + i * (r.w - 2) / (r.h - 2);
                    ui.p.fill(Rect::new(px, r.y + r.h - 2 - i, 1, 1), cross);
                }
            }
            outline(ui, r, edge, false);
        }
        // the planned blocks: one capsule each, shown up to `spent`
        for b in &self.blocks {
            let shown = (self.spent - b.from as f32).clamp(0.0, b.slots as f32);
            if shown <= 0.0 {
                continue;
            }
            let full = pip_rect(at, cy, b.from);
            let w = ((shown * (PIP_W + PIP_GAP) as f32).round() as i32 - PIP_GAP).max(2);
            let r = Rect::new(full.x, full.y, w, PIP_H);
            let fill = if b.ghost { faded(b.colour, GHOST_ALPHA) } else { b.colour };
            ui.p.fill(inner(r), fill);
            let tick = mix(b.colour, [0, 0, 0, 255], TICK_DARK);
            for j in 1..b.slots {
                let tx = full.x + j * (PIP_W + PIP_GAP) - 1;
                if tx < r.x + r.w - 1 {
                    ui.p.fill(Rect::new(tx, r.y + 2, 1, r.h - 4), tick);
                }
            }
            outline(ui, r, mix(b.colour, [255, 255, 255, 255], RIM_LIGHT), true);
        }
        at += pips_w(self.slots);
        if let Some(l) = &self.after {
            at += LABEL_GAP;
            ui.p.text(at as f32, (cy - lh / 2) as f32, l, SMALL, self.colours.alert);
            at += label_w(ui, l);
        }
        let rect = Rect::new(x, cy - h / 2, (at - x).max(1), h);
        ui.push(id, "pips", self.reading(), rect, true, false, None, self.tip.clone());
        rect
    }
}

/// `r` less its 1 px outline.
fn inner(r: Rect) -> Rect {
    Rect::new(r.x + 1, r.y + 1, (r.w - 2).max(0), (r.h - 2).max(0))
}

/// A 1 px outline of `r`; `round` leaves the four corner pixels out (a capsule's rounded look).
fn outline<A: Clone>(ui: &mut Ui<A>, r: Rect, c: Rgba, round: bool) {
    let k = i32::from(round);
    ui.p.fill(Rect::new(r.x + k, r.y, r.w - 2 * k, 1), c);
    ui.p.fill(Rect::new(r.x + k, r.y + r.h - 1, r.w - 2 * k, 1), c);
    ui.p.fill(Rect::new(r.x, r.y + 1, 1, r.h - 2), c);
    ui.p.fill(Rect::new(r.x + r.w - 1, r.y + 1, 1, r.h - 2), c);
}

/// A label's width.
pub fn label_w<A: Clone>(ui: &mut Ui<A>, s: &str) -> i32 {
    ui.p.text_w(s, SMALL).ceil() as i32
}

/// `c` at `k` of its alpha.
fn faded(c: Rgba, k: f32) -> Rgba {
    [c[0], c[1], c[2], (c[3] as f32 * k.clamp(0.0, 1.0)).round() as u8]
}

/// From `a` (k = 0) to `b` (k = 1).
pub fn mix(a: Rgba, b: Rgba, k: f32) -> Rgba {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k.clamp(0.0, 1.0)).round() as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), m(a[3], b[3])]
}

/// The UI Gallery's states (name, gauge; 12 slots, 10 points held unless named): style D's four rows (two attacks and a
/// move, three attacks and a defend, one long attack, nothing planned), a ghost plan, a pick 1 short, a debt of 2, a
/// budget of 14 at 2 points a slot and a spending one. Sample colours only (a game passes its own).
pub fn samples() -> Vec<(&'static str, PipGauge)> {
    let t = th();
    let (attack, moving, defend) = (t.foe, t.info, t.gold);
    let tip = |now: i32, plan: &[PipBlock], max: i32| {
        let spent: i32 = plan.iter().map(|b| b.ap).sum();
        vec![(format!("Now {now} of {max}"), t.gold), (format!("Plan {spent}, left {}", now - spent), t.text)]
    };
    let planned = |now: i32, plan: Vec<PipBlock>, max: i32| PipGauge::planned(now, &plan, max, 12).with_tip(tip(now, &plan, max));
    let a = |ap| PipBlock::new(ap, attack);
    vec![
        ("Two attacks and a move", planned(10, vec![a(3), a(2), PipBlock::new(2, moving)], 12)),
        ("Three attacks and a defend", planned(10, vec![a(2), a(2), a(3), PipBlock::new(1, defend)], 12)),
        ("One long attack", planned(10, vec![a(5)], 12)),
        ("Nothing planned", planned(10, Vec::new(), 12)),
        ("A ghost plan", planned(10, vec![PipBlock::ghost(4, attack), PipBlock::ghost(2, attack)], 12)),
        ("A pick 1 short (red)", planned(3, vec![a(3)], 12).short(1)),
        ("A debt of 2", planned(-2, Vec::new(), 12)),
        ("Max 14, 2 a slot: now 14, plan 5", planned(14, vec![a(4), PipBlock::new(1, moving)], 14)),
        ("Spending: 10 held, 6 left", PipGauge::live(10, 6, t.dim, 12, 12).with_tip(vec![("Spent 4 of 10".into(), t.gold)])),
    ]
}
