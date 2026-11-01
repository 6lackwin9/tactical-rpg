//! A pip gauge (INPUT1-AP, engine-only; no Godot original): a row of small pips for a budget planned against, one pip a
//! point (or `unit` points when the maximum is above the gauge's cap). Each pip has an outline, solid for a point held
//! now and a dim slot above it, and a fill: full for a point kept, ghost (at the timeline's `GHOST_ALPHA`) for a point
//! the next refill brings back, none for a point spent. The gauge can turn every outline to its alert colour (a pick the
//! budget cannot pay) and carries a short label before its first pip (a debt) or after its last (a shortfall). Levels
//! are counted in pips and may be fractional while a screen animates them (a pip fills or empties by its fraction).
//! A screen builds a `PipGauge` from its own numbers each frame and draws it; it pushes one info widget (its reading in
//! pip codes, the hover card the screen's). `samples` are the UI Gallery's states.

use crate::canvas::{Rect, Rgba};
use crate::timeline::GHOST_ALPHA;
use crate::ui::{Tip, Ui, DIM, FOE, GOLD, SMALL, TEXT};

/// One pip's size and the gap between pips (canvas pixels): 12 pips take 59.
pub const PIP_W: i32 = 4;
pub const PIP_H: i32 = 6;
pub const PIP_GAP: i32 = 1;
/// The gap between a label and the pips.
pub const LABEL_GAP: i32 = 2;

/// The points a pip stands for: one, or `ceil(max / cap)` when `max` is above `cap` (the gauge's most pips).
pub fn unit(max: i32, cap: i32) -> i32 {
    if cap <= 0 || max <= cap {
        1
    } else {
        (max + cap - 1) / cap
    }
}

/// `n` points in pips of `unit` points, rounded up (none for `n <= 0`).
pub fn count(n: i32, unit: i32) -> i32 {
    let u = unit.max(1);
    if n <= 0 {
        0
    } else {
        (n + u - 1) / u
    }
}

/// The width of `slots` pips side by side.
pub fn pips_w(slots: i32) -> i32 {
    if slots <= 0 {
        0
    } else {
        slots * (PIP_W + PIP_GAP) - PIP_GAP
    }
}

/// A pip as read (settled levels): full; hollow (outlined, empty); hollow with a ghost fill; a slot with a ghost fill;
/// an empty slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pip {
    Full,
    Hollow,
    HollowGhost,
    SlotGhost,
    Slot,
}

impl Pip {
    /// The pip's short code: "F", "H", "Hg", "Sg", "S".
    pub fn code(self) -> &'static str {
        match self {
            Pip::Full => "F",
            Pip::Hollow => "H",
            Pip::HollowGhost => "Hg",
            Pip::SlotGhost => "Sg",
            Pip::Slot => "S",
        }
    }
}

/// The gauge's colours: the fill (full and, at `GHOST_ALPHA`, ghost), a solid outline, a slot's outline, the alert
/// (outlines and labels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colours {
    pub fill: Rgba,
    pub edge: Rgba,
    pub slot: Rgba,
    pub alert: Rgba,
}

impl Default for Colours {
    fn default() -> Self {
        Colours { fill: GOLD, edge: TEXT, slot: DIM, alert: FOE }
    }
}

/// A pip gauge. Levels count pips from the left: pips `1..=edge` have a solid outline, `1..=fill` are full, pips above
/// `fill` up to `ghost` have a ghost fill. `Default` is an empty gauge.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PipGauge {
    /// Pips drawn (the maximum in pips).
    pub slots: i32,
    /// The points a pip stands for (1 or more).
    pub unit: i32,
    pub edge: f32,
    pub fill: f32,
    pub ghost: f32,
    /// Every outline in the alert colour.
    pub alert: bool,
    /// A label before the first pip and one after the last, in the alert colour.
    pub before: Option<String>,
    pub after: Option<String>,
    pub colours: Colours,
    /// The hover card (the screen's numbers).
    pub tip: Tip,
}

impl PipGauge {
    /// A budget being planned, in points: `have` now, `keep` left after the plan, `next` after the next refill, `max`, at
    /// most `cap` pips. Pip `i`: full when `i <= keep`, ghost when `keep < i <= next`, outline solid when `i <= have`
    /// (every count in pips, rounded up). A debt (`have < 0`) draws every pip as an empty slot and "-<debt>" before them.
    pub fn planned(have: i32, keep: i32, next: i32, max: i32, cap: i32) -> PipGauge {
        let u = unit(max, cap);
        let slots = count(max, u);
        if have < 0 {
            return PipGauge { slots, unit: u, before: Some(format!("-{}", -have)), ..PipGauge::default() };
        }
        let edge = count(have, u).min(slots);
        let fill = count(keep, u).min(edge);
        let ghost = count(next, u).clamp(fill, slots);
        PipGauge { slots, unit: u, edge: edge as f32, fill: fill as f32, ghost: ghost as f32, ..PipGauge::default() }
    }

    /// A budget being spent, in points: `start` held when the spending began, `have` now, `max`, at most `cap` pips:
    /// full pips up to `have`, hollow ones up to `start` (what was paid), slots to `max`. A debt as in `planned`.
    pub fn live(start: i32, have: i32, max: i32, cap: i32) -> PipGauge {
        let u = unit(max, cap);
        let slots = count(max, u);
        if have < 0 {
            return PipGauge { slots, unit: u, before: Some(format!("-{}", -have)), ..PipGauge::default() };
        }
        let fill = count(have, u).min(slots);
        let edge = count(start.max(have), u).min(slots);
        PipGauge { slots, unit: u, edge: edge as f32, fill: fill as f32, ghost: fill as f32, ..PipGauge::default() }
    }

    /// The gauge with a pick `short` points beyond the budget: every outline in the alert colour and "-<short>" after
    /// the last pip (a debt's label before the pips stays; then no label after).
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

    /// Each pip as read (a fractional level counts from its half).
    pub fn pips(&self) -> Vec<Pip> {
        (1..=self.slots)
            .map(|i| {
                let lit = |level: f32| level >= i as f32 - 0.5;
                match (lit(self.fill), lit(self.edge), lit(self.ghost)) {
                    (true, _, _) => Pip::Full,
                    (false, true, true) => Pip::HollowGhost,
                    (false, true, false) => Pip::Hollow,
                    (false, false, true) => Pip::SlotGhost,
                    (false, false, false) => Pip::Slot,
                }
            })
            .collect()
    }

    /// The pips' codes, space separated ("F F Hg S").
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

    /// Draws the gauge from `x`, its pips centred on `cy`, and pushes the info widget `id` (kind "pips", its reading,
    /// the hover card); returns the widget's rect.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, id: &str, x: i32, cy: i32) -> Rect {
        let lh = ui.p.line_h(SMALL).ceil() as i32;
        let h = PIP_H.max(lh);
        let mut at = x;
        if let Some(l) = &self.before {
            ui.p.text(at as f32, (cy - lh / 2) as f32, l, SMALL, self.colours.alert);
            at += label_w(ui, l) + LABEL_GAP;
        }
        let y = cy - PIP_H / 2;
        for k in 0..self.slots {
            let r = Rect::new(at + k * (PIP_W + PIP_GAP), y, PIP_W, PIP_H);
            let part = |level: f32| (level - k as f32).clamp(0.0, 1.0);
            let inner = Rect::new(r.x + 1, r.y + 1, r.w - 2, r.h - 2);
            let (full, ghost) = (part(self.fill), part(self.ghost));
            if ghost > full {
                ui.p.fill(inner, faded(self.colours.fill, GHOST_ALPHA * ghost));
            }
            if full > 0.0 {
                ui.p.fill(inner, faded(self.colours.fill, full));
            }
            let edge = if self.alert { self.colours.alert } else { mix(self.colours.slot, self.colours.edge, part(self.edge)) };
            ui.p.fill(Rect::new(r.x, r.y, r.w, 1), edge);
            ui.p.fill(Rect::new(r.x, r.y + r.h - 1, r.w, 1), edge);
            ui.p.fill(Rect::new(r.x, r.y + 1, 1, r.h - 2), edge);
            ui.p.fill(Rect::new(r.x + r.w - 1, r.y + 1, 1, r.h - 2), edge);
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

/// A label's width.
pub fn label_w<A: Clone>(ui: &mut Ui<A>, s: &str) -> i32 {
    ui.p.text_w(s, SMALL).ceil() as i32
}

/// `c` at `k` of its alpha.
fn faded(c: Rgba, k: f32) -> Rgba {
    [c[0], c[1], c[2], (c[3] as f32 * k.clamp(0.0, 1.0)).round() as u8]
}

/// From `a` (k = 0) to `b` (k = 1).
fn mix(a: Rgba, b: Rgba, k: f32) -> Rgba {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k.clamp(0.0, 1.0)).round() as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), m(a[3], b[3])]
}

/// The UI Gallery's states (name, gauge; at most 12 pips): planned budgets of 10 (kept 3, 4 and 2; nothing kept with a
/// ghost refill), a pick one short, a debt of 2, and a budget of 14 at 2 points a pip.
pub fn samples() -> Vec<(&'static str, PipGauge)> {
    let numbers = |have: i32, keep: i32, next: i32, max: i32| vec![(format!("Now {have} of {max}"), GOLD), (format!("Left {keep}, next {next}"), TEXT)];
    let planned = |have, keep, next, max| PipGauge::planned(have, keep, next, max, 12).with_tip(numbers(have, keep, next, max));
    vec![
        ("Now 10, left 3, next 7", planned(10, 3, 7, 10)),
        ("Now 10, left 4, next 8", planned(10, 4, 8, 10)),
        ("Now 10, left 2, next 6", planned(10, 2, 6, 10)),
        ("Now 3, left 0, next 5", planned(3, 0, 5, 10)),
        ("A pick 1 short (red)", planned(3, 3, 5, 10).short(1)),
        ("A debt of 2", planned(-2, -2, 1, 10)),
        ("Max 14, 2 a pip: now 14, left 9, next 12", planned(14, 9, 12, 14)),
        ("Spending: 10 held, 6 left", PipGauge::live(10, 6, 10, 12).with_tip(vec![("Spent 4 of 10".into(), GOLD)])),
    ]
}
