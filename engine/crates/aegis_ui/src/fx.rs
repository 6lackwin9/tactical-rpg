//! Effects kit and camera shake (FX1-E, FX1 section 3): engine-only, game-agnostic.
//! Godot had token tweens only (`scripts/ui/combat/fx_presets.gd`); this kit draws its own pieces in code.
//!
//! - `Clock`: the effect clock with hit-stop (a hold eats wall time before the clock moves; speed scales the rest).
//! - Primitives (`Projectile`, `Beam`, `Trail`, `Sparks`, `Flash`, `Ring`) wrapped in an `Effect` (start, layer,
//!   seed): every draw is a pure function of the effect and the clock's `now` (no state, no RNG).
//!   STATUSFX1-E adds `Orbit` (dots circling a point) and lets a `Ring` close (`r0 > r1`).
//! - `Fx`: the caller's live list, at most `MAX_LIVE`, drawn by layer in push order.
//! - `Looped` (STATUSFX1-E): an effect repeated every `period`, reseeded per cycle, drawn by the caller every frame
//!   outside the live list; `draw_still` is its reduce-motion look.
//! - Camera shake: `Shake` (strength, duration, axis, envelope, step), presets by generic name (`SHAKE_PRESETS`,
//!   `ShakeTable`), `ShakeLimits` clamped to the engine ceiling, and `Camera`, whose `offset` gives deterministic whole
//!   logical px for `Painter::shift` (the caller shifts its world layer only; hit rects never move).

use crate::canvas::{Rect, Rgba};
use crate::painter::Painter;

/// The most effects a live list holds (Godot `FxPresets.MAX_LIVE` was 24 tweens); a push past it drops the oldest.
pub const MAX_LIVE: usize = 32;
/// The engine's hard shake ceiling: no game's limits go past it.
pub const SHAKE_CEIL_PX: f32 = 6.0;
pub const SHAKE_CEIL_MS: u32 = 3000;
/// The shake presets by generic name (a light hit, a heavy impact, a slow bell sway, a rising tremor, a quake).
pub const SHAKE_PRESETS: [&str; 5] = ["jolt", "heavy", "toll", "tremor", "quake"];

/// A logical point.
pub type Pt = (f32, f32);

/// `h(seed, k)` in [0, 1): splitmix64 of `seed ^ k x 0x9E3779B97F4A7C15`, its top 24 bits over 2^24.
pub fn hash01(seed: u64, k: u64) -> f32 {
    let mut z = (seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15)).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / 16_777_216.0
}

fn with_alpha(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], a.round().clamp(0.0, 255.0) as u8]
}

fn faded(c: Rgba, k: f32) -> Rgba {
    with_alpha(c, c[3] as f32 * k.clamp(0.0, 1.0))
}

fn lerp(a: f32, b: f32, u: f32) -> f32 {
    a + (b - a) * u
}

// ---- Clock ------------------------------------------------------------------------------------------------------

/// The effect clock (FX1 3.1): `ms` moves by wall time x speed, except while a hit-stop `hold` eats the wall time.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Clock {
    pub ms: f64,
    pub hold: f64,
}

impl Clock {
    /// One frame of `wall_ms` (the caller caps it, as the vignette screen does at 100 ms) at `speed`.
    pub fn advance(&mut self, wall_ms: f64, speed: f64) {
        let wall = wall_ms.max(0.0);
        let h = self.hold.min(wall);
        self.hold -= h;
        self.ms += (wall - h) * speed.max(0.0);
    }

    /// A hit-stop of `ms` wall ms: the clock stands still that long (holds do not add; the longer one wins).
    pub fn hold(&mut self, ms: f64) {
        self.hold = self.hold.max(ms);
    }

    /// The clock in whole ms.
    pub fn now(&self) -> u64 {
        self.ms.max(0.0).floor() as u64
    }
}

// ---- Primitives -------------------------------------------------------------------------------------------------

/// Where an effect draws: on the ground (under the bodies) or in the air (over them); the caller picks the pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layer {
    Ground,
    #[default]
    Air,
}

/// A projectile's look.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Look {
    /// A `len`-px shaft and a 2 px head.
    Arrow { len: f32, shaft: Rgba, head: Rgba },
    /// A `len`-px shaft and a 1 px head.
    Bolt { len: f32, shaft: Rgba, head: Rgba },
    /// A `size` square with a 1 px highlight at its top-left.
    Stone { size: f32, col: Rgba, hi: Rgba },
    /// A glow (radius r + 1.5, alpha 90), a core of radius `r` and `tail` dots (at most 3) 12, 24, 36 ms behind.
    Orb { r: f32, core: Rgba, glow: Rgba, tail: u8 },
}

/// A shot from `from` to `to` over `travel_ms`, raised `arc_px x 4u(1 - u)` on the way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projectile {
    pub from: Pt,
    pub to: Pt,
    pub travel_ms: u32,
    pub arc_px: f32,
    pub look: Look,
}

impl Projectile {
    /// The position at `u` (0..1).
    pub fn pos(&self, u: f32) -> Pt {
        (lerp(self.from.0, self.to.0, u), lerp(self.from.1, self.to.1, u) - self.arc_px * 4.0 * u * (1.0 - u))
    }

    /// The unit tangent at `u` (the direction of travel; +x when the path has no length).
    pub fn dir(&self, u: f32) -> Pt {
        let (dx, dy) = (self.to.0 - self.from.0, self.to.1 - self.from.1 - self.arc_px * 4.0 * (1.0 - 2.0 * u));
        let l = (dx * dx + dy * dy).sqrt();
        if l < 1e-4 {
            (1.0, 0.0)
        } else {
            (dx / l, dy / l)
        }
    }
}

/// A straight beam that fades over its life.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Beam {
    pub from: Pt,
    pub to: Pt,
    pub ms: u32,
    /// 1-3 px of `glow`, with a 1 px `core` line on it.
    pub width: u8,
    pub core: Rgba,
    pub glow: Rgba,
}

/// A swing's swept arc on an ellipse (`radius`, `radius x squash`) around `at`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trail {
    pub at: Pt,
    pub radius: f32,
    /// 0 deg = the facing (+x), negative = up; `mirror` flips x (facing left).
    pub from_deg: f32,
    pub to_deg: f32,
    pub ms: u32,
    pub width: u8,
    pub col: Rgba,
    pub mirror: bool,
    pub squash: f32,
}

/// The tail of a trail lags this far behind its head.
pub const TRAIL_TAIL_DEG: f32 = 70.0;
/// A trail's segments.
pub const TRAIL_SEGMENTS: usize = 8;

impl Trail {
    /// The (tail, head) angles in degrees at `u`: the head sweeps `from_deg` to `to_deg` over the first 60 %, the tail
    /// 70 deg behind it, never before `from_deg`.
    pub fn span(&self, u: f32) -> (f32, f32) {
        let head = lerp(self.from_deg, self.to_deg, (u / 0.6).clamp(0.0, 1.0));
        let sgn = if self.to_deg >= self.from_deg { 1.0 } else { -1.0 };
        let tail = head - sgn * TRAIL_TAIL_DEG;
        let tail = if sgn > 0.0 { tail.max(self.from_deg) } else { tail.min(self.from_deg) };
        (tail, head)
    }

    /// The point at `deg` on the trail's ellipse, `dr` px out from its radius.
    pub fn point(&self, deg: f32, dr: f32) -> Pt {
        let a = deg.to_radians();
        let m = if self.mirror { -1.0 } else { 1.0 };
        let r = self.radius + dr;
        (self.at.0 + m * r * a.cos(), self.at.1 + r * self.squash * a.sin())
    }
}

/// A burst of sparks from `at`, flying and falling (gravity + down) as they fade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sparks {
    pub at: Pt,
    pub ms: u32,
    /// 1-24 sparks.
    pub count: u8,
    pub dir_deg: f32,
    pub spread_deg: f32,
    /// px a second.
    pub speed: f32,
    /// px a second squared, + down.
    pub gravity: f32,
    /// 1 or 2 px squares.
    pub size: u8,
    /// Spark i takes `cols[i % 2]`.
    pub cols: [Rgba; 2],
}

impl Sparks {
    /// Every spark's (position, colour with its alpha) `t_ms` after the start, for `seed`: the alpha falls from the
    /// colour's own to 0 over `ms`.
    pub fn at_time(&self, seed: u64, t_ms: u64) -> Vec<(Pt, Rgba)> {
        let u = t_ms as f32 / self.ms.max(1) as f32;
        let t = t_ms as f32 / 1000.0;
        (0..self.count.min(24) as u64)
            .map(|i| {
                let a = (self.dir_deg + self.spread_deg * (hash01(seed, i) - 0.5)).to_radians();
                let v = self.speed * (0.5 + 0.5 * hash01(seed, i + 50));
                let p = (self.at.0 + a.cos() * v * t, self.at.1 + a.sin() * v * t + self.gravity * t * t / 2.0);
                (p, faded(self.cols[i as usize % 2], 1.0 - u))
            })
            .collect()
    }
}

/// A filled ellipse that fades.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flash {
    pub at: Pt,
    pub rx: f32,
    pub ry: f32,
    pub ms: u32,
    pub col: Rgba,
}

/// An ellipse outline whose radius runs `r0` to `r1` (height x `squash`) as it fades: growing when `r0 < r1`, closing
/// when `r0 > r1` (STATUSFX1-E).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    pub at: Pt,
    pub r0: f32,
    pub r1: f32,
    pub ms: u32,
    pub col: Rgba,
    pub squash: f32,
}

/// The alpha of an orbit's dots in the back half (behind `at`) and the front half, out of 255 (STATUSFX1-E).
pub const ORBIT_BACK_ALPHA: u8 = 110;
pub const ORBIT_FRONT_ALPHA: u8 = 255;

/// Dots circling `at` on the ellipse (`rx`, `ry`) once per `ms` (STATUSFX1-E): dot `i` at angle
/// `360 x (u + i / count) + phase_deg` (0 deg = +x, 90 = down, the front); it does not fade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit {
    pub at: Pt,
    pub rx: f32,
    pub ry: f32,
    pub ms: u32,
    /// 1-8 dots, evenly spaced.
    pub count: u8,
    /// 1 or 2 px squares (a star is always the 3 px plus).
    pub size: u8,
    /// Dot i takes `cols[i % 2]`; its alpha is scaled by `ORBIT_BACK_ALPHA` or `ORBIT_FRONT_ALPHA` / 255.
    pub cols: [Rgba; 2],
    pub phase_deg: f32,
    /// A 3 px plus instead of a square.
    pub star: bool,
}

impl Orbit {
    /// The dots drawn (`count` clamped to 1-8).
    pub fn dots(&self) -> u8 {
        self.count.clamp(1, 8)
    }

    /// Dot `i`'s angle in degrees at `u`.
    pub fn angle(&self, i: u8, u: f32) -> f32 {
        360.0 * (u + i as f32 / self.dots() as f32) + self.phase_deg
    }

    /// Dot `i`'s centre at `u`, its colour with its alpha, and whether it is in the back half (sin < 0: above `at`).
    pub fn dot(&self, i: u8, u: f32) -> (Pt, Rgba, bool) {
        let a = self.angle(i, u).to_radians();
        let (s, c) = a.sin_cos();
        let back = s < -1e-6;
        let k = if back { ORBIT_BACK_ALPHA } else { ORBIT_FRONT_ALPHA } as f32 / 255.0;
        ((self.at.0 + self.rx * c, self.at.1 + self.ry * s), faded(self.cols[i as usize % 2], k), back)
    }
}

/// One of the primitives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Prim {
    Projectile(Projectile),
    Beam(Beam),
    Trail(Trail),
    Sparks(Sparks),
    Flash(Flash),
    Ring(Ring),
    Orbit(Orbit),
}

macro_rules! prim_from {
    ($($t:ident),*) => {$(impl From<$t> for Prim { fn from(p: $t) -> Prim { Prim::$t(p) } })*};
}
prim_from!(Projectile, Beam, Trail, Sparks, Flash, Ring, Orbit);

/// The primitives' names (the agent link's `kinds`).
pub const PRIM_KINDS: [&str; 7] = ["projectile", "beam", "trail", "sparks", "flash", "ring", "orbit"];

/// A primitive placed on the clock: it starts at `start`, draws on `layer` and varies by `seed`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effect {
    pub prim: Prim,
    pub start: u64,
    pub layer: Layer,
    pub seed: u64,
}

impl Effect {
    /// `prim` from `start`, in the air, seed 0.
    pub fn new(prim: impl Into<Prim>, start: u64) -> Self {
        Self { prim: prim.into(), start, layer: Layer::Air, seed: 0 }
    }

    pub fn on(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    pub fn seeded(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// The same effect `dx`, `dy` logical px away (a caller that keeps effects in its own frame, a stage or a board,
    /// places them on the canvas).
    pub fn moved(&self, dx: f32, dy: f32) -> Effect {
        let m = |p: Pt| (p.0 + dx, p.1 + dy);
        let prim = match self.prim {
            Prim::Projectile(p) => Prim::Projectile(Projectile { from: m(p.from), to: m(p.to), ..p }),
            Prim::Beam(b) => Prim::Beam(Beam { from: m(b.from), to: m(b.to), ..b }),
            Prim::Trail(t) => Prim::Trail(Trail { at: m(t.at), ..t }),
            Prim::Sparks(s) => Prim::Sparks(Sparks { at: m(s.at), ..s }),
            Prim::Flash(f) => Prim::Flash(Flash { at: m(f.at), ..f }),
            Prim::Ring(r) => Prim::Ring(Ring { at: m(r.at), ..r }),
            Prim::Orbit(o) => Prim::Orbit(Orbit { at: m(o.at), ..o }),
        };
        Effect { prim, ..*self }
    }

    /// Its life in clock ms (a projectile's is its travel).
    pub fn ms(&self) -> u64 {
        (match self.prim {
            Prim::Projectile(p) => p.travel_ms,
            Prim::Beam(b) => b.ms,
            Prim::Trail(t) => t.ms,
            Prim::Sparks(s) => s.ms,
            Prim::Flash(f) => f.ms,
            Prim::Ring(r) => r.ms,
            Prim::Orbit(o) => o.ms,
        }) as u64
    }

    /// The clock ms at which it is done.
    pub fn end(&self) -> u64 {
        self.start + self.ms()
    }

    pub fn done(&self, now: u64) -> bool {
        now >= self.end()
    }

    /// Its primitive's name (`PRIM_KINDS`).
    pub fn kind(&self) -> &'static str {
        PRIM_KINDS[match self.prim {
            Prim::Projectile(_) => 0,
            Prim::Beam(_) => 1,
            Prim::Trail(_) => 2,
            Prim::Sparks(_) => 3,
            Prim::Flash(_) => 4,
            Prim::Ring(_) => 5,
            Prim::Orbit(_) => 6,
        }]
    }

    /// `u` = (now - start) / ms while it shows; None before its start and once done.
    pub fn u(&self, now: u64) -> Option<f32> {
        (now >= self.start && !self.done(now)).then(|| (now - self.start) as f32 / self.ms().max(1) as f32)
    }

    /// Draws it as it is at `now` (nothing before its start or once done).
    pub fn draw(&self, p: &mut Painter, now: u64) {
        let Some(u) = self.u(now) else { return };
        match self.prim {
            Prim::Projectile(pr) => draw_projectile(p, &pr, u, now - self.start),
            Prim::Beam(b) => {
                let k = 1.0 - u;
                band(p, b.from, b.to, b.width.clamp(1, 3), faded(b.glow, k));
                p.line(b.from.0, b.from.1, b.to.0, b.to.1, faded(b.core, k), 0.0);
            }
            Prim::Trail(t) => {
                let (tail, head) = t.span(u);
                if (head - tail).abs() < 0.01 {
                    return;
                }
                let fade = if u > 0.6 { 1.0 - (u - 0.6) / 0.4 } else { 1.0 };
                let w = t.width.max(1);
                for j in 0..TRAIL_SEGMENTS {
                    let (a0, a1) = (lerp(tail, head, j as f32 / TRAIL_SEGMENTS as f32), lerp(tail, head, (j + 1) as f32 / TRAIL_SEGMENTS as f32));
                    let alpha = (40.0 + 190.0 * j as f32 / (TRAIL_SEGMENTS - 1) as f32) * fade * t.col[3] as f32 / 255.0;
                    for k in 0..w {
                        let dr = k as f32 - (w - 1) as f32 / 2.0;
                        let (q0, q1) = (t.point(a0, dr), t.point(a1, dr));
                        p.line(q0.0, q0.1, q1.0, q1.1, with_alpha(t.col, alpha), 0.0);
                    }
                }
            }
            Prim::Sparks(s) => {
                let n = s.size.clamp(1, 2) as i32;
                for (q, col) in s.at_time(self.seed, now - self.start) {
                    p.fill(Rect::new(q.0.round() as i32, q.1.round() as i32, n, n), col);
                }
            }
            Prim::Flash(f) => p.ellipse(f.at.0, f.at.1, f.rx, f.ry, faded(f.col, 1.0 - u)),
            Prim::Ring(r) => {
                let rad = lerp(r.r0, r.r1, u);
                p.ellipse_ring(r.at.0, r.at.1, rad, rad * r.squash, faded(r.col, 1.0 - u));
            }
            Prim::Orbit(o) => draw_orbit(p, &o, u),
        }
    }
}

/// The back dots first, then the front ones over them.
fn draw_orbit(p: &mut Painter, o: &Orbit, u: f32) {
    let dots: Vec<(Pt, Rgba, bool)> = (0..o.dots()).map(|i| o.dot(i, u)).collect();
    for back in [true, false] {
        for (q, col, _) in dots.iter().filter(|d| d.2 == back) {
            let (x, y) = (q.0.round() as i32, q.1.round() as i32);
            if o.star {
                p.fill(Rect::new(x - 1, y, 3, 1), *col);
                p.fill(Rect::new(x, y - 1, 1, 1), *col);
                p.fill(Rect::new(x, y + 1, 1, 1), *col);
            } else {
                let n = o.size.clamp(1, 2) as i32;
                p.fill(Rect::new(x - (n - 1) / 2, y - (n - 1) / 2, n, n), *col);
            }
        }
    }
}

// ---- Looped effects ---------------------------------------------------------------------------------------------

/// An effect that repeats every `period` clock ms from its start (STATUSFX1-E): the caller draws it every frame and
/// never puts it on a live list (so it never evicts a one-shot). Cycle `k` draws the inner effect at local time
/// `t mod period` (nothing once that passes the effect's `ms`) with seed `seed_at(k)`: sparks vary from cycle to cycle,
/// the same way every run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Looped {
    pub effect: Effect,
    pub period: u64,
}

impl Looped {
    /// `effect` every `period` ms (at least 1) from `effect.start`.
    pub fn new(effect: Effect, period: u64) -> Self {
        Self { effect, period: period.max(1) }
    }

    /// The seed of cycle `k`: the effect's own for cycle 0 (the first cycle draws as the one-shot would), a hash of
    /// it and `k` after.
    pub fn seed_at(&self, k: u64) -> u64 {
        if k == 0 {
            return self.effect.seed;
        }
        let mut z = (self.effect.seed ^ k.wrapping_mul(0xD6E8_FEB8_6659_FD93)).wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// (cycle, local ms) at `now`; None before the start.
    pub fn phase(&self, now: u64) -> Option<(u64, u64)> {
        let t = now.checked_sub(self.effect.start)?;
        let period = self.period.max(1);
        Some((t / period, t % period))
    }

    /// The inner effect as cycle `k` draws it, started at clock 0 (draw it at the local time).
    pub fn cycle(&self, k: u64) -> Effect {
        Effect { start: 0, seed: self.seed_at(k), ..self.effect }
    }

    /// Draws it as it is at `now` (nothing before its start or in a cycle's gap after the effect's `ms`).
    pub fn draw(&self, p: &mut Painter, now: u64) {
        if let Some((k, local)) = self.phase(now) {
            self.cycle(k).draw(p, local);
        }
    }

    /// The reduce-motion look: cycle 0 at local time `ms / 2` (its `u` 0.5 for an even `ms`), whatever the clock.
    pub fn draw_still(&self, p: &mut Painter) {
        let e = self.cycle(0);
        e.draw(p, e.ms() / 2);
    }

    /// Whether it shows anything at `now` (started and inside a cycle's effect).
    pub fn showing(&self, now: u64) -> bool {
        self.phase(now).is_some_and(|(_, local)| local < self.effect.ms())
    }

    /// Its inner primitive's name (`PRIM_KINDS`).
    pub fn kind(&self) -> &'static str {
        self.effect.kind()
    }

    /// Its layer (the caller picks the pass).
    pub fn layer(&self) -> Layer {
        self.effect.layer
    }

    /// The same loop `dx`, `dy` logical px away (`Effect::moved`).
    pub fn moved(&self, dx: f32, dy: f32) -> Looped {
        Looped { effect: self.effect.moved(dx, dy), ..*self }
    }
}

/// A `w`-px band of parallel 1 px lines centred on the segment.
fn band(p: &mut Painter, a: Pt, b: Pt, w: u8, col: Rgba) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = (dx * dx + dy * dy).sqrt().max(1e-4);
    let (nx, ny) = (-dy / l, dx / l);
    for k in 0..w {
        let o = k as f32 - (w - 1) as f32 / 2.0;
        p.line(a.0 + nx * o, a.1 + ny * o, b.0 + nx * o, b.1 + ny * o, col, 0.0);
    }
}

fn draw_projectile(p: &mut Painter, pr: &Projectile, u: f32, t: u64) {
    let (x, y) = pr.pos(u);
    let (dx, dy) = pr.dir(u);
    match pr.look {
        Look::Arrow { len, shaft, head } | Look::Bolt { len, shaft, head } => {
            let hl = if matches!(pr.look, Look::Arrow { .. }) { 2.0 } else { 1.0 };
            p.line(x - dx * len, y - dy * len, x, y, shaft, 0.0);
            // The head: the tip's last px in the head colour and two barbs back from it.
            p.line(x - dx * hl, y - dy * hl, x, y, head, 0.0);
            for side in [-1.0f32, 1.0] {
                let (c, s) = (0.8f32, 0.6 * side);
                let (bx, by) = (-(dx * c - dy * s), -(dy * c + dx * s));
                p.line(x, y, x + bx * hl, y + by * hl, head, 0.0);
            }
        }
        Look::Stone { size, col, hi } => {
            let n = size.max(1.0).round() as i32;
            let (x0, y0) = ((x - size / 2.0).round() as i32, (y - size / 2.0).round() as i32);
            p.fill(Rect::new(x0, y0, n, n), col);
            p.fill(Rect::new(x0, y0, 1, 1), hi);
        }
        Look::Orb { r, core, glow, tail } => {
            for (k, a) in [(12u64, 160.0f32), (24, 100.0), (36, 50.0)].into_iter().take(tail.min(3) as usize) {
                if t < k {
                    break;
                }
                let q = pr.pos((t - k) as f32 / pr.travel_ms.max(1) as f32);
                p.fill(Rect::new(q.0.round() as i32, q.1.round() as i32, 1, 1), with_alpha(glow, a));
            }
            p.ellipse(x, y, r + 1.5, r + 1.5, with_alpha(glow, 90.0));
            p.ellipse(x, y, r, r, core);
        }
    }
}

// ---- The live list ----------------------------------------------------------------------------------------------

/// The caller's live effects (FX1 3.3), at most `MAX_LIVE`. Nothing here reads a setting: a caller that skips
/// animations simply pushes nothing.
#[derive(Debug, Clone, Default)]
pub struct Fx {
    pub live: Vec<Effect>,
}

impl Fx {
    /// Adds `e`; when full, the oldest (smallest start, then the first pushed) goes first.
    pub fn push(&mut self, e: Effect) {
        if self.live.len() >= MAX_LIVE {
            let oldest = self.live.iter().enumerate().min_by_key(|(i, x)| (x.start, *i)).map(|(i, _)| i).unwrap_or(0);
            self.live.remove(oldest);
        }
        self.live.push(e);
    }

    /// Drops the effects done at `now`.
    pub fn prune(&mut self, now: u64) {
        self.live.retain(|e| !e.done(now));
    }

    /// Draws `layer`'s effects at `now`, in push order.
    pub fn draw(&self, p: &mut Painter, now: u64, layer: Layer) {
        for e in self.live.iter().filter(|e| e.layer == layer) {
            e.draw(p, now);
        }
    }

    /// The latest end of the live effects (None: the list is empty).
    pub fn busy_until(&self) -> Option<u64> {
        self.live.iter().map(|e| e.end()).max()
    }

    pub fn clear(&mut self) {
        self.live.clear();
    }

    pub fn len(&self) -> usize {
        self.live.len()
    }

    pub fn is_empty(&self) -> bool {
        self.live.is_empty()
    }

    /// The names of the effects not done at `now` (started or waiting), in push order (the agent link).
    pub fn kinds(&self, now: u64) -> Vec<&'static str> {
        self.live.iter().filter(|e| !e.done(now)).map(|e| e.kind()).collect()
    }
}

// ---- Camera shake -----------------------------------------------------------------------------------------------

/// Which way a shake moves the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Both,
}

impl Axis {
    /// `x`, `y`, `both` (None for anything else).
    pub fn parse(s: &str) -> Option<Axis> {
        match s {
            "x" => Some(Axis::X),
            "y" => Some(Axis::Y),
            "both" => Some(Axis::Both),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Both => "both",
        }
    }
}

/// How a shake's strength runs over its life (`u` 0..1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Envelope {
    /// 1 - u.
    Linear,
    /// (1 - u)^2.
    EaseOut,
    /// sin(pi u): rises, then fades.
    Swell,
}

impl Envelope {
    /// `linear`, `ease_out`, `swell` (None for anything else).
    pub fn parse(s: &str) -> Option<Envelope> {
        match s {
            "linear" => Some(Envelope::Linear),
            "ease_out" => Some(Envelope::EaseOut),
            "swell" => Some(Envelope::Swell),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Envelope::Linear => "linear",
            Envelope::EaseOut => "ease_out",
            Envelope::Swell => "swell",
        }
    }

    /// The strength factor at `u` (0 outside 0..1).
    pub fn at(self, u: f32) -> f32 {
        if !(0.0..1.0).contains(&u) {
            return 0.0;
        }
        match self {
            Envelope::Linear => 1.0 - u,
            Envelope::EaseOut => (1.0 - u) * (1.0 - u),
            Envelope::Swell => (std::f32::consts::PI * u).sin().max(0.0),
        }
    }
}

/// A camera shake: `px` at its strongest, for `ms`, along `axis`, shaped by `envelope`, its direction flipping every
/// `step_ms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shake {
    pub px: f32,
    pub ms: u32,
    pub axis: Axis,
    pub envelope: Envelope,
    pub step_ms: u32,
}

impl Shake {
    /// `px` and `ms` clamped to `limits` (themselves clamped to the engine ceiling).
    pub fn clamped(self, limits: ShakeLimits) -> Shake {
        let l = limits.clamped();
        Shake { px: self.px.clamp(0.0, l.max_px), ms: self.ms.min(l.max_ms), ..self }
    }

    /// The strength `t` ms after its start (0 at and after `ms`).
    pub fn amplitude(&self, t: u64) -> f32 {
        if t >= self.ms as u64 {
            return 0.0;
        }
        self.px * self.envelope.at(t as f32 / self.ms.max(1) as f32)
    }

    /// The view offset `t` ms after its start for `seed` (FX1 3.4): whole logical px, (0, 0) at and after `ms`.
    pub fn offset(&self, t: u64, seed: u64) -> (i32, i32) {
        if t >= self.ms as u64 {
            return (0, 0);
        }
        let k = t / self.step_ms.max(1) as u64;
        let a = self.amplitude(t);
        let sign = if k.is_multiple_of(2) { 1.0 } else { -1.0 };
        let m = sign * (0.6 + 0.4 * hash01(seed, k));
        let cap = self.px.floor();
        let whole = |v: f32| v.round().clamp(-cap, cap) as i32;
        match self.axis {
            Axis::X => (whole(a * m), 0),
            Axis::Y => (0, whole(a * m)),
            Axis::Both => (whole(a * m), whole(a * 0.5 * (2.0 * hash01(seed, k + 101) - 1.0))),
        }
    }
}

/// A game's shake limits (the art bible's); never past the engine ceiling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShakeLimits {
    pub max_px: f32,
    pub max_ms: u32,
}

impl ShakeLimits {
    /// The engine ceiling (`SHAKE_CEIL_PX`, `SHAKE_CEIL_MS`).
    pub const CEILING: ShakeLimits = ShakeLimits { max_px: SHAKE_CEIL_PX, max_ms: SHAKE_CEIL_MS };

    /// Limits clamped to the ceiling.
    pub fn new(max_px: f32, max_ms: u32) -> Self {
        ShakeLimits { max_px, max_ms }.clamped()
    }

    pub fn clamped(self) -> Self {
        ShakeLimits { max_px: self.max_px.clamp(0.0, SHAKE_CEIL_PX), max_ms: self.max_ms.min(SHAKE_CEIL_MS) }
    }
}

impl Default for ShakeLimits {
    fn default() -> Self {
        Self::CEILING
    }
}

/// The shakes by preset name (`SHAKE_PRESETS`); a game tunes its own rows over the engine's.
#[derive(Debug, Clone, PartialEq)]
pub struct ShakeTable {
    rows: Vec<(&'static str, Shake)>,
}

impl Default for ShakeTable {
    /// The engine's neutral values (FX1 4.7).
    fn default() -> Self {
        let s = |px, ms, axis, envelope, step_ms| Shake { px, ms, axis, envelope, step_ms };
        Self {
            rows: vec![
                ("jolt", s(1.0, 90, Axis::Both, Envelope::Linear, 30)),
                ("heavy", s(2.0, 160, Axis::Both, Envelope::EaseOut, 30)),
                ("toll", s(2.0, 1100, Axis::X, Envelope::EaseOut, 70)),
                ("tremor", s(2.0, 1400, Axis::Y, Envelope::Swell, 45)),
                ("quake", s(3.0, 900, Axis::Both, Envelope::EaseOut, 35)),
            ],
        }
    }
}

impl ShakeTable {
    /// The shake named `name` (None: not a preset).
    pub fn get(&self, name: &str) -> Option<Shake> {
        self.rows.iter().find(|(n, _)| *n == name).map(|(_, s)| *s)
    }

    /// Sets preset `name`'s shake; false (and no change) when `name` is not in `SHAKE_PRESETS`.
    pub fn set(&mut self, name: &str, shake: Shake) -> bool {
        match self.rows.iter_mut().find(|(n, _)| *n == name) {
            Some(row) => {
                row.1 = shake;
                true
            }
            None => false,
        }
    }

    /// (name, shake) in `SHAKE_PRESETS` order.
    pub fn rows(&self) -> &[(&'static str, Shake)] {
        &self.rows
    }
}

/// The camera: the running shake, its start and its seed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Camera {
    pub shake: Option<(Shake, u64, u64)>,
}

impl Camera {
    /// Starts `shake` (clamped to `limits`) at `now` unless `reduce_motion`; it replaces the running shake only when
    /// its `px` is at least that one's current strength (shakes never add). True when it started.
    pub fn start(&mut self, shake: Shake, now: u64, seed: u64, limits: ShakeLimits, reduce_motion: bool) -> bool {
        if reduce_motion {
            return false;
        }
        let s = shake.clamped(limits);
        if s.px < self.amplitude(now) {
            return false;
        }
        self.shake = Some((s, now, seed));
        true
    }

    /// The running shake's strength at `now` (0 when none runs).
    pub fn amplitude(&self, now: u64) -> f32 {
        self.shake.map_or(0.0, |(s, start, _)| s.amplitude(now.saturating_sub(start)))
    }

    /// Whether a shake runs at `now`.
    pub fn active(&self, now: u64) -> bool {
        self.shake.is_some_and(|(s, start, _)| now.saturating_sub(start) < s.ms as u64)
    }

    /// The view offset at `now`, without changing anything (for a draw that holds the camera read-only).
    pub fn offset_at(&self, now: u64) -> (i32, i32) {
        self.shake.map_or((0, 0), |(s, start, seed)| s.offset(now.saturating_sub(start), seed))
    }

    /// The view offset at `now` for `Painter::shift`; the shake ends once its time is up.
    pub fn offset(&mut self, now: u64) -> (i32, i32) {
        if !self.active(now) {
            self.shake = None;
            return (0, 0);
        }
        self.offset_at(now)
    }

    pub fn stop(&mut self) {
        self.shake = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelopes_start_end_and_swell() {
        assert_eq!((Envelope::Linear.at(0.0), Envelope::EaseOut.at(0.0), Envelope::Swell.at(0.0)), (1.0, 1.0, 0.0));
        assert!((Envelope::Swell.at(0.5) - 1.0).abs() < 1e-6, "a swell peaks half way");
        assert_eq!((Envelope::Linear.at(1.0), Envelope::EaseOut.at(1.0), Envelope::Swell.at(1.0)), (0.0, 0.0, 0.0));
    }

    #[test]
    fn names_parse_back() {
        for a in [Axis::X, Axis::Y, Axis::Both] {
            assert_eq!(Axis::parse(a.name()), Some(a));
        }
        for e in [Envelope::Linear, Envelope::EaseOut, Envelope::Swell] {
            assert_eq!(Envelope::parse(e.name()), Some(e));
        }
        assert_eq!((Axis::parse("z"), Envelope::parse("bounce")), (None, None));
    }

    #[test]
    fn the_default_table_holds_every_preset_and_only_those() {
        let t = ShakeTable::default();
        assert_eq!(t.rows().iter().map(|r| r.0).collect::<Vec<_>>(), SHAKE_PRESETS.to_vec());
        let mut t2 = t.clone();
        assert!(!t2.set("boom", t.get("jolt").unwrap()) && t2 == t, "an unknown name changes nothing");
        assert!(t.get("boom").is_none());
    }
}
