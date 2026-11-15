//! Portrait overlays (ASSETORG4 section 2.2, FB-227): the procedural overlay FX a portrait may draw over its picture
//! (`glow` on the glow anchors, growing `lines` from the line roots, a `point_glow` on the eye boxes, falling
//! `particles_rise` specks and rising `drift` puffs) as an opt-in layer of `portrait_fx`. Engine-only, game-agnostic.
//!
//! - Base portrait life (breath, rim flicker, blink), states, reactions and expression frames and effects draw without
//!   this module; a portrait draws overlays only when its `PortraitOpts::overlays` is `Mount::Mounted`. Unmounted (the
//!   default), an overlay cue or a sidecar `idle` overlay is held but has no level: nothing is composed for it, `live`
//!   and `state` do not list it, and it never makes a portrait time-varying.
//! - Who mounts it: the game (its own switch, e.g. a manifest key) and the player (the settings key `SETTING`, on when
//!   missing): `Mount::resolve`. The engine's Kit widgets Portraits page mounts it to show every toggle.
//! - The painting (`paint_anchored` before the expression mark, `paint_rising` after it) is called by
//!   `portrait_fx::compose` only for a mounted draw; its numbers are PORTRAITFX1 4.3 steps 5 and 6.

use crate::canvas::Image;
use crate::fx::hash01;
use crate::portrait_fx::{alpha, cycle, drift, fill_ellipse, flare_on, over, particle_specks, Level, Tier, LINE_GROW_MS};
use crate::screen::Settings;
use aegis_core::assets::{Face, IDLE_OVERLAYS};
use std::collections::{BTreeMap, BTreeSet};

/// The overlay effects this module draws (the names a sidecar's `idle` may hold, `aegis_core::assets::IDLE_OVERLAYS`).
pub const OVERLAYS: [&str; 5] = IDLE_OVERLAYS;
/// The player's settings key (an accessibility switch beside `reduce_motion`): false hides every portrait overlay of a
/// game that mounts them; missing reads as true (`player_on`).
pub const SETTING: &str = "portrait_overlays";

/// Whether `effect` is one of `OVERLAYS`.
pub fn is_overlay(effect: &str) -> bool {
    OVERLAYS.contains(&effect)
}

/// The player's choice (`SETTING`; true when missing or not a boolean).
pub fn player_on(settings: &Settings) -> bool {
    settings.v[SETTING].as_bool().unwrap_or(true)
}

/// Whether a portrait draw mounts the overlay layer (`PortraitOpts::overlays`); `Unmounted`, the default, draws base
/// portraits only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mount {
    #[default]
    Unmounted,
    Mounted,
}

impl Mount {
    /// `Mounted` for true.
    pub fn of(on: bool) -> Mount {
        if on {
            Mount::Mounted
        } else {
            Mount::Unmounted
        }
    }

    pub fn is_mounted(self) -> bool {
        self == Mount::Mounted
    }

    /// The game's switch and the player's setting together: mounted only when the game mounts the layer and the
    /// player has not turned it off (`player_on`).
    pub fn resolve(game: bool, settings: &Settings) -> Mount {
        Mount::of(game && player_on(settings))
    }
}

/// Where a composition paints: the face (its anchors in image px, scaled by `k` to the composed side `s`), the inner
/// rect [lo, hi) of height `h_in`, the motion (stretch `a`, offset `mx`, `my`) the anchored overlays follow, the eye
/// boxes in composed px, the clock `t`, reduce motion, the tier and the portrait's seed.
pub(crate) struct Paint<'a> {
    pub face: &'a Face,
    pub k: f32,
    pub s: i32,
    pub lo: i32,
    pub hi: i32,
    pub h_in: i32,
    pub a: i32,
    pub mx: i32,
    pub my: i32,
    pub eyes: &'a [(i32, i32, i32, i32)],
    pub t: u64,
    pub reduce: bool,
    pub tier: Tier,
    pub seed: u64,
}

impl Paint<'_> {
    /// An anchor's y after the motion: bottom - (bottom - y) (H + a) / H + my.
    fn by(&self, y: f32) -> f32 {
        self.hi as f32 - (self.hi as f32 - y) * (self.h_in + self.a) as f32 / self.h_in as f32 + self.my as f32
    }

    /// An anchor's x after the motion: x + mx.
    fn bx(&self, x: f32) -> f32 {
        x + self.mx as f32
    }
}

/// The pixels of a line from `a` to `b` (Bresenham).
fn line_px(a: (i32, i32), b: (i32, i32), out: &mut BTreeSet<(i32, i32)>) {
    let (dx, dy) = ((b.0 - a.0).abs(), -(b.1 - a.1).abs());
    let (sx, sy) = (if a.0 < b.0 { 1 } else { -1 }, if a.1 < b.1 { 1 } else { -1 });
    let (mut x, mut y, mut err) = (a.0, a.1, dx + dy);
    loop {
        out.insert((x, y));
        if x == b.0 && y == b.1 {
            return;
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

/// A `lines` polyline: `segs` segments of `step` px from `from`, segment j turned 25 (2 h(k0 + j) - 1) deg from `deg`.
fn line_points(seed: u64, from: (f32, f32), deg: f32, step: f32, segs: usize, k0: u64) -> Vec<(f32, f32)> {
    let mut pts = vec![from];
    for j in 0..segs {
        let a = (deg + 25.0 * (2.0 * hash01(seed, k0 + j as u64) - 1.0)).to_radians();
        let p = pts[j];
        pts.push((p.0 + step * a.cos(), p.1 + step * a.sin()));
    }
    pts
}

/// Step 5: the anchored overlays at their `levels`: glow, lines, point glow (they follow the motion).
pub(crate) fn paint_anchored(img: &mut Image, c: &Paint, levels: &BTreeMap<&'static str, Level>) {
    let lv = |e: &str| levels.get(e).copied();
    let (face, k, s, lo, hi, h_in, t, reduce, tier, seed) = (c.face, c.k, c.s, c.lo, c.hi, c.h_in, c.t, c.reduce, c.tier, c.seed);
    if let Some(l) = lv("glow") {
        let pulse = if reduce { 0.8 } else { 0.6 + 0.4 * cycle(t, 1800.0).sin() };
        let anchors: Vec<(f32, f32, f32)> = if face.glow.is_empty() {
            vec![(lo as f32 + 0.5 * h_in as f32, lo as f32 + 0.7 * h_in as f32, 0.15 * s as f32)]
        } else {
            face.glow.iter().map(|g| (c.bx(g[0] as f32 * k), c.by(g[1] as f32 * k), g[2] as f32 * k)).collect()
        };
        for (cx, cy, r) in anchors {
            for (f, a) in [(1.0, 40.0), (0.66, 60.0), (0.33, 90.0)] {
                fill_ellipse(img, cx, cy, r * f, r * f, alpha(l.color, a * l.i * pulse), lo, hi);
            }
        }
    }
    if let Some(l) = lv("lines") {
        let pulse = if reduce { 0.8 } else { 0.6 + 0.4 * cycle(t, 2400.0).sin() };
        let grown = if reduce { 1.0 } else { (t.saturating_sub(l.start) as f32 / LINE_GROW_MS as f32).clamp(0.0, 1.0) };
        let wide = tier == Tier::Full && s > 192;
        let step = (3.0 * k).max(1.0);
        for (ri, v) in face.lines.iter().enumerate() {
            let root = (c.bx(v[0] as f32 * k), c.by(v[1] as f32 * k));
            let len = v[3] as f32 * k;
            let segs = (v[3] as f32 / 3.0).ceil().max(1.0) as usize;
            let k0 = 400 + ri as u64 * 1000;
            let trunk = line_points(seed, root, v[2] as f32, step, segs, k0);
            // Trunk first (the first 60 % of the growth), then the branches.
            let mut strokes: Vec<(Vec<(f32, f32)>, usize)> = vec![(trunk.clone(), (segs as f32 * (grown / 0.6).min(1.0)).round() as usize)];
            let bsegs = ((0.4 * v[3] as f32) / 3.0).ceil().max(1.0) as usize;
            let bshow = (bsegs as f32 * ((grown - 0.6) / 0.4).clamp(0.0, 1.0)).round() as usize;
            for (bi, (at, turn)) in [(0.4f32, 35.0f32), (0.7, -35.0)].into_iter().enumerate() {
                let from = trunk[((segs as f32 * at).round() as usize).min(segs)];
                strokes.push((line_points(seed, from, v[2] as f32 + turn, step, bsegs, k0 + 200 * (bi as u64 + 1)), bshow));
            }
            let mut px: BTreeSet<(i32, i32)> = BTreeSet::new();
            for (pts, shown) in strokes {
                for w in pts.windows(2).take(shown) {
                    line_px((w[0].0.round() as i32, w[0].1.round() as i32), (w[1].0.round() as i32, w[1].1.round() as i32), &mut px);
                }
            }
            if wide {
                let base: Vec<(i32, i32)> = px.iter().copied().collect();
                for (x, y) in base {
                    px.extend([(x + 1, y), (x, y + 1), (x + 1, y + 1)]);
                }
            }
            let col = alpha(l.color, 255.0 * l.i * pulse);
            for (x, y) in px {
                let (dx, dy) = (x as f32 + 0.5 - root.0, y as f32 + 0.5 - root.1);
                if dx * dx + dy * dy <= len * len {
                    over(img, x, y, col, lo, hi);
                }
            }
        }
    }
    if let Some(l) = lv("point_glow") {
        let flare = if !reduce && tier != Tier::Minimal && flare_on(seed, t) { 1.4 } else { 1.0 };
        for &(x0, y0, x1, y1) in c.eyes {
            let (w, h) = ((x1 - x0) as f32, (y1 - y0) as f32);
            let (cx, cy) = ((x0 + x1) as f32 / 2.0, (y0 + y1) as f32 / 2.0);
            if tier != Tier::Minimal {
                fill_ellipse(img, cx, cy, 0.9 * w, 1.1 * h, alpha(l.color, 70.0 * l.i * flare), lo, hi);
            }
            fill_ellipse(img, cx, cy, w / 2.0, h / 2.0, alpha(l.color, 200.0 * l.i * flare), lo, hi);
        }
    }
}

/// Step 6: the free overlays at their `levels`: particles, then drift (they do not follow the motion).
pub(crate) fn paint_rising(img: &mut Image, c: &Paint, levels: &BTreeMap<&'static str, Level>) {
    let lv = |e: &str| levels.get(e).copied();
    let (s, lo, hi, h_in, t, tier, seed) = (c.s, c.lo, c.hi, c.h_in, c.t, c.tier, c.seed);
    if let Some(l) = lv("particles_rise") {
        let sz = (s / 96).max(1);
        for (x, y) in particle_specks(seed, t, l.i, tier, s, lo, hi) {
            for yy in y..y + sz {
                for xx in x..x + sz {
                    over(img, xx, yy, alpha(l.color, 160.0 * l.i), lo, hi);
                }
            }
        }
    }
    if let Some(l) = lv("drift") {
        let w = h_in as f32;
        for kk in 0..4u64 {
            let r = (0.05 + 0.04 * hash01(seed, 800 + kk)) * s as f32;
            let x = lo as f32 + hash01(seed, 820 + kk) * w;
            let risen = (hash01(seed, 840 + kk) * w + drift(t, 0.04 * s as f32, w)).rem_euclid(w);
            let a = 50.0 * l.i * (1.0 - risen / w);
            fill_ellipse(img, x, hi as f32 - risen, r, r, alpha(l.color, a), lo, hi);
        }
    }
}

