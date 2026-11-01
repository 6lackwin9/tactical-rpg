//! Portrait effects (PORTRAITFX1-E, docs/specs/PORTRAITFX1.md section 4): engine-only (Godot had no portrait
//! effects), game-agnostic. A `PortraitFx` per drawn portrait composes the picture with its base life (breath, rim
//! flicker, blink), overlays (glow, veins, eye glow, ash, smoke), states (desaturate, tint, fade, still) and reactions
//! (flash, jolt) on the CPU at the drawn size, caches the result and draws it into the caller's rect.
//!
//! - Anchors come from the face sidecar (`aegis_core::assets::Face`, read by `Art::face`), scaled from its `size` to
//!   the composed side; effects never draw on or move the sidecar's border (only `jolt` moves the whole portrait).
//! - Cues (`Cue`): `hold` sets the state cues (each eases in and out over `EASE_MS`), `fire` starts a reaction; per
//!   effect the strongest wins. `expression` reports the winning `face` cue (the caller draws that frame).
//! - Time: the caller's clock (`now` in ms); every random choice is `fx::hash01(seed, k)` with the seed from the key
//!   (FNV-1a 64); nothing reads a game RNG. Time-varying effects and every cue level read `tq`, the clock in 30
//!   steps a second; periodic motion takes `t mod period` before any f32 (`cycle`), so it stays smooth for hours.
//! - Size tiers (`Tier`), reduce motion (`PortraitOpts`), the per-screen `Budget` (at most `MAX_ANIMATED`
//!   time-varying portraits a frame), the agent state (`state`) and the code-drawn sample face (`sample`).
//! - Expression effects (PORTRAITFX1-EX, `ExpressionFx`): the caller's table of what an expression adds while a `face`
//!   cue holds it (a tint over the sidecar's face box, a code-drawn mark anchored to the face or eye box, a short
//!   shake); rows are data the game passes (`set_expressions`), the engine names no expression.

use crate::art::Art;
use crate::canvas::{Image, Rect, Rgba};
use crate::fx::{hash01, Axis, Camera, Envelope, Shake, ShakeLimits};
use crate::painter::Painter;
use aegis_core::assets::{Face, IdleOverlay, IDLE_OVERLAYS};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::f32::consts::PI;

/// The cue vocabulary (spec 4.2), in this order.
pub const EFFECTS: [&str; 12] = ["flash", "jolt", "desaturate", "tint", "fade", "still", "glow", "veins", "eye_glow", "ash", "smoke", "face"];
/// The base life every portrait runs where its tier allows (not cues).
pub const BASE_LIFE: [&str; 3] = ["breath", "rim", "blink"];
/// The jolt presets.
pub const PORTRAIT_JOLTS: [&str; 2] = ["hit", "crit"];
/// A held cue eases in (and out) over this long.
pub const EASE_MS: u64 = 250;
/// The most time-varying portraits a screen recomposes per frame (`Budget`).
pub const MAX_ANIMATED: usize = 8;
/// The largest side a portrait composes at (device px); a bigger rect scales the composed image up.
pub const COMPOSE_MAX: i32 = 256;
/// Time-varying effects step this many times a second (`tq`).
pub const STEPS_PER_SECOND: u64 = 30;
/// The tiers' lower sides in device px: full from 96, reduced from 48, minimal below.
pub const FULL_SIDE: i32 = 96;
pub const REDUCED_SIDE: i32 = 48;
/// The jolt ceiling (4 px, 600 ms).
pub const JOLT_LIMITS: ShakeLimits = ShakeLimits { max_px: 4.0, max_ms: 600 };
/// A blink lasts this long; a double blink's second starts this long after the first.
pub const BLINK_MS: u64 = 130;
pub const DOUBLE_BLINK_MS: u64 = 220;
/// A flash fired with no `ms`.
pub const FLASH_MS: u32 = 160;
/// An eye-glow flare.
pub const FLARE_MS: u64 = 140;
/// Veins grow to full length over this long.
pub const VEIN_GROW_MS: u64 = 1800;
/// The anchors check's outline colours (eye boxes, glow circles, vein roots).
pub const ANCHOR_EYE: Rgba = [34, 211, 238, 255];
pub const ANCHOR_GLOW: Rgba = [244, 114, 182, 255];
pub const ANCHOR_VEIN: Rgba = [250, 204, 21, 255];

/// The engine's default colour of an effect (plain numbers; a game passes its own through cues and sidecars).
pub fn default_color(effect: &str) -> Rgba {
    match effect {
        "tint" => [0x06, 0xb6, 0xd4, 255],
        "glow" | "veins" => [0x93, 0x33, 0xea, 255],
        "eye_glow" => [0xe1, 0x1d, 0x48, 255],
        "ash" | "smoke" => [0x6b, 0x72, 0x80, 255],
        _ => [255, 255, 255, 255],
    }
}

/// A jolt preset's shake (`hit`: 2 px, 180 ms, x; `crit`: 3 px, 320 ms, both; ease-out, a flip every 30 ms).
pub fn jolt_preset(name: &str) -> Option<Shake> {
    match name {
        "hit" => Some(Shake { px: 2.0, ms: 180, axis: Axis::X, envelope: Envelope::EaseOut, step_ms: 30 }),
        "crit" => Some(Shake { px: 3.0, ms: 320, axis: Axis::Both, envelope: Envelope::EaseOut, step_ms: 30 }),
        _ => None,
    }
}

/// FNV-1a 64 of `key`: a portrait's seed.
pub fn seed_of(key: &str) -> u64 {
    key.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3))
}

/// The clock in 30 steps a second: `floor(floor(3 now / 100) x 100 / 3)` ms.
pub fn tq(now: u64) -> u64 {
    (now * 3 / 100) * 100 / 3
}

fn rgb3(c: [u8; 3]) -> Rgba {
    [c[0], c[1], c[2], 255]
}

// ---- Size tiers -----------------------------------------------------------------------------------------------------

/// How much a portrait draws, by its drawn side in device px (spec 4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    Full,
    Reduced,
    Minimal,
}

impl Tier {
    /// Full from `FULL_SIDE`, reduced from `REDUCED_SIDE`, minimal below.
    pub fn of(side: i32) -> Tier {
        if side >= FULL_SIDE {
            Tier::Full
        } else if side >= REDUCED_SIDE {
            Tier::Reduced
        } else {
            Tier::Minimal
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Tier::Full => "full",
            Tier::Reduced => "reduced",
            Tier::Minimal => "minimal",
        }
    }

    /// Whether the tier draws `effect` (base life or cue): reduced drops `breath`, `veins` and `smoke` (and halves the
    /// ash and the jolt); minimal keeps `flash`, `desaturate`, `tint`, `fade`, `face` and the `eye_glow` core only.
    pub fn allows(self, effect: &str) -> bool {
        let known = BASE_LIFE.contains(&effect) || EFFECTS.contains(&effect);
        match self {
            Tier::Full => known,
            Tier::Reduced => known && !matches!(effect, "breath" | "veins" | "smoke"),
            Tier::Minimal => matches!(effect, "flash" | "desaturate" | "tint" | "fade" | "face" | "eye_glow"),
        }
    }

    /// The effects the tier draws, base life first, then `EFFECTS` order.
    pub fn effects(self) -> Vec<&'static str> {
        BASE_LIFE.iter().chain(EFFECTS.iter()).copied().filter(|e| self.allows(e)).collect()
    }
}

// ---- Cues -----------------------------------------------------------------------------------------------------------

/// One effect at an intensity: a state while held, a reaction when fired (spec 4.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    /// A name of `EFFECTS`.
    pub effect: &'static str,
    /// 0-1.
    pub intensity: f32,
    /// A fired cue's length (0: a flash's `FLASH_MS`, a jolt's preset length; other effects need it to fire).
    pub ms: u32,
    /// None: the effect's default colour.
    pub color: Option<Rgba>,
    /// A jolt's preset (`PORTRAIT_JOLTS`); "" for every other effect.
    pub preset: &'static str,
    /// A `face` cue's expression ("" = neutral).
    pub expression: String,
    /// A `face` cue's priority (the highest wins).
    pub priority: i32,
    /// Who holds it (the hold set's identity; default the effect's name).
    pub key: String,
}

impl Cue {
    /// A cue of `effect` at intensity 1, ms 0, the default colour, priority 50, keyed by its effect; None for an
    /// unknown effect, or a `jolt` whose `preset` is not in `PORTRAIT_JOLTS` (the caller drops it).
    pub fn parse(effect: &str, preset: &str) -> Option<Cue> {
        let e = *EFFECTS.iter().find(|x| **x == effect)?;
        let preset = if e == "jolt" { *PORTRAIT_JOLTS.iter().find(|x| **x == preset)? } else { "" };
        Some(Cue { effect: e, intensity: 1.0, ms: 0, color: None, preset, expression: String::new(), priority: 50, key: e.to_string() })
    }

    /// A sidecar's idle overlay as a held cue (`idle.<effect>`); None when its effect is not one of
    /// `assets::IDLE_OVERLAYS` (a sidecar never holds a state such as `still` or `fade`, nor a reaction).
    pub fn idle(o: &IdleOverlay) -> Option<Cue> {
        if !IDLE_OVERLAYS.contains(&o.effect.as_str()) {
            return None;
        }
        Some(Cue::parse(&o.effect, "")?.with_intensity(o.intensity).with_color(o.color.map(rgb3)).with_key(&format!("idle.{}", o.effect)))
    }

    pub fn with_intensity(mut self, i: f32) -> Self {
        self.intensity = i.clamp(0.0, 1.0);
        self
    }

    pub fn with_ms(mut self, ms: u32) -> Self {
        self.ms = ms;
        self
    }

    pub fn with_color(mut self, c: Option<Rgba>) -> Self {
        self.color = c;
        self
    }

    pub fn with_expression(mut self, e: &str) -> Self {
        self.expression = e.to_string();
        self
    }

    pub fn with_priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }

    pub fn with_key(mut self, k: &str) -> Self {
        self.key = k.to_string();
        self
    }

    /// A jolt cue's shake: its preset with `px` x intensity and its own `ms` when above 0.
    pub fn shake(&self) -> Option<Shake> {
        let s = jolt_preset(self.preset)?;
        Some(Shake { px: s.px * self.intensity, ms: if self.ms > 0 { self.ms } else { s.ms }, ..s })
    }
}

/// The held cues of a sidecar's `idle` list.
pub fn idle_cues(face: &Face) -> Vec<Cue> {
    face.idle.iter().filter_map(Cue::idle).collect()
}

/// An effect's strength at a moment: the winning cue's intensity (eases and fades applied), its colour and when it
/// started.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    pub i: f32,
    pub color: Rgba,
    pub start: u64,
}

#[derive(Debug, Clone, PartialEq)]
struct Held {
    cue: Cue,
    since: u64,
    gone: Option<u64>,
}

impl Held {
    /// The ease factor at `now`: in over `EASE_MS` from `since`, out over `EASE_MS` from `gone`.
    fn ease(&self, now: u64) -> f32 {
        let inn = |t: u64| if t < self.since { 0.0 } else { ((t - self.since) as f32 / EASE_MS as f32).min(1.0) };
        match self.gone {
            None => inn(now),
            Some(g) => inn(g) * (1.0 - (now.saturating_sub(g) as f32 / EASE_MS as f32)).clamp(0.0, 1.0),
        }
    }
}

// ---- Expression effects (PORTRAITFX1-EX) ----------------------------------------------------------------------------

/// The marks an expression effect may draw: code-drawn pixel shapes (`vein` the anger mark, `sweat` a drop).
pub const MARKS: [&str; 2] = ["vein", "sweat"];
/// The boxes a mark's anchor is relative to: the sidecar's face box (`Face::face_box`) or the union of its eye boxes.
pub const MARK_BOXES: [&str; 2] = ["face", "eyes"];
/// A pulsing mark grows by up to this share of its width and its alpha falls to this share at the pulse's low.
pub const MARK_GROW: f32 = 0.2;
pub const MARK_ALPHA_LOW: f32 = 0.7;
/// An expression shake flips every this many ms (as a jolt).
pub const SHAKE_STEP_MS: u32 = 30;

/// A mark's shape. Each is a small bitmap (`X` the colour, `H` its highlight, `O` its outline) scaled by nearest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Vein,
    Sweat,
}

/// The anger mark: four corners whose bends point at the centre.
const VEIN_BITMAP: [&str; 9] = ["..X...X..", "..X...X..", "XXX...XXX", ".........", ".........", ".........", "XXX...XXX", "..X...X..", "..X...X.."];
/// A sweat drop with a highlight and an outline.
const SWEAT_BITMAP: [&str; 7] = ["..O..", ".OXO.", ".OXO.", "OHXXO", "OXXXO", "OXXXO", ".OOO."];

impl Mark {
    /// The mark named `name` (`MARKS`); None for "" or an unknown name.
    pub fn parse(name: &str) -> Option<Mark> {
        match name {
            "vein" => Some(Mark::Vein),
            "sweat" => Some(Mark::Sweat),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Mark::Vein => "vein",
            Mark::Sweat => "sweat",
        }
    }

    /// The bitmap rows (all of one width).
    pub fn bitmap(self) -> &'static [&'static str] {
        match self {
            Mark::Vein => &VEIN_BITMAP,
            Mark::Sweat => &SWEAT_BITMAP,
        }
    }

    /// The engine's default colour (plain numbers; the game's rows name their own).
    pub fn default_color(self) -> Rgba {
        match self {
            Mark::Vein => [0xe1, 0x1d, 0x48, 255],
            Mark::Sweat => [0x7d, 0xd3, 0xfc, 255],
        }
    }

    /// The colour of bitmap cell `ch` for colour `c` (None: clear): `X` c, `H` c toward white by 0.7, `O` c x 0.45.
    pub fn cell(ch: char, c: Rgba) -> Option<Rgba> {
        let f = |k: usize, g: &dyn Fn(f32) -> f32| g(c[k] as f32).round().clamp(0.0, 255.0) as u8;
        match ch {
            'X' => Some(c),
            'H' => Some([f(0, &|v| v + (255.0 - v) * 0.7), f(1, &|v| v + (255.0 - v) * 0.7), f(2, &|v| v + (255.0 - v) * 0.7), 255]),
            'O' => Some([f(0, &|v| v * 0.45), f(1, &|v| v * 0.45), f(2, &|v| v * 0.45), 255]),
            _ => None,
        }
    }
}

/// The box a mark's anchor is relative to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkBox {
    Face,
    Eyes,
}

impl MarkBox {
    /// `face` or `eyes` (`MARK_BOXES`); anything else is the face box.
    pub fn parse(name: &str) -> MarkBox {
        if name == "eyes" {
            MarkBox::Eyes
        } else {
            MarkBox::Face
        }
    }
}

/// One expression's runtime effect (a row of the caller's table): drawn while a `face` cue holds `expression`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionFx {
    pub expression: String,
    /// The tint over the face box (None: no tint), its strength 0-1 (`tinted` at strength x the soft-ellipse weight)
    /// and its length: 0 holds while the expression holds (easing in over `EASE_MS`), above 0 fades out over that
    /// many ms from the expression's start (a flash).
    pub tint: Option<Rgba>,
    pub tint_strength: f32,
    pub tint_ms: u32,
    /// The mark (None: none), its colour (None: the mark's default), the box its anchor is in, the anchor in units of
    /// that box (0-1 inside it; outside allowed, as a temple beside the face), its width as a share of the composed
    /// side (at least the bitmap's own width) and its pulse period (0: steady).
    pub mark: Option<Mark>,
    pub mark_color: Option<Rgba>,
    pub mark_box: MarkBox,
    pub mark_x: f32,
    pub mark_y: f32,
    pub mark_size: f32,
    pub mark_pulse_ms: u32,
    /// A shake of the whole portrait from the expression's start (0: none): `shake_px` logical px over `shake_ms`.
    pub shake_px: f32,
    pub shake_ms: u32,
}

impl ExpressionFx {
    /// A row for `expression` that draws nothing (set the parts it draws).
    pub fn new(expression: &str) -> Self {
        Self { expression: expression.to_string(), tint: None, tint_strength: 0.0, tint_ms: 0, mark: None, mark_color: None, mark_box: MarkBox::Face, mark_x: 0.5, mark_y: 0.5, mark_size: 0.1, mark_pulse_ms: 0, shake_px: 0.0, shake_ms: 0 }
    }

    /// The tint's strength `since` ms after the expression started: `tint_ms` 0 eases in over `EASE_MS` and holds;
    /// above 0 it falls from full to 0 over `tint_ms`.
    pub fn tint_at(&self, since: u64) -> f32 {
        let s = self.tint_strength.clamp(0.0, 1.0);
        if self.tint_ms == 0 {
            s * (since as f32 / EASE_MS as f32).min(1.0)
        } else if since < self.tint_ms as u64 {
            s * (1.0 - since as f32 / self.tint_ms as f32)
        } else {
            0.0
        }
    }

    /// The pulse at `t`: 0.5 + 0.5 sin(2 pi t / `mark_pulse_ms`), in 0-1 (1 with no pulse).
    pub fn pulse(&self, t: u64) -> f32 {
        if self.mark_pulse_ms == 0 {
            return 1.0;
        }
        0.5 + 0.5 * cycle(t, self.mark_pulse_ms as f64).sin()
    }

    /// The shake (axis x, ease-out, a flip every `SHAKE_STEP_MS`, within `JOLT_LIMITS`); None without one.
    pub fn shake(&self) -> Option<Shake> {
        (self.shake_px > 0.0 && self.shake_ms > 0).then(|| Shake { px: self.shake_px, ms: self.shake_ms, axis: Axis::X, envelope: Envelope::EaseOut, step_ms: SHAKE_STEP_MS }.clamped(JOLT_LIMITS))
    }

    /// The box the mark's anchor is in, in image px of `face` (None: the sidecar has no such box).
    pub fn anchor_box(&self, face: &Face) -> Option<[i32; 4]> {
        match self.mark_box {
            MarkBox::Face => face.face_box(),
            MarkBox::Eyes => face.eyes_box(),
        }
    }

    /// The mark's centre in composed px (`k` = composed side / sidecar size), before the breath moves it.
    pub fn anchor(&self, face: &Face, k: f32) -> Option<(f32, f32)> {
        let b = self.anchor_box(face)?;
        Some(((b[0] as f32 + self.mark_x * b[2] as f32) * k, (b[1] as f32 + self.mark_y * b[3] as f32) * k))
    }
}

/// A mark's width in composed px at side `s`: round(`size` x `s` x `grow`), at least its bitmap's width.
pub fn mark_width(m: Mark, size: f32, s: i32, grow: f32) -> i32 {
    let w = m.bitmap()[0].len() as i32;
    ((size * s as f32 * grow).round() as i32).max(w)
}

/// A pulse `p` (0-1) as the mark's growth and alpha factor: 1 + `MARK_GROW` p, `MARK_ALPHA_LOW` + (1 - low) p.
pub fn mark_pulse(p: f32) -> (f32, f32) {
    (1.0 + MARK_GROW * p, MARK_ALPHA_LOW + (1.0 - MARK_ALPHA_LOW) * p)
}

/// The tint's weight at (dx, dy) in units of the face box's half sides from its centre: a soft ellipse, 1 inside half
/// its area (d^2 <= 0.5), falling to 0 at its edge: clamp(2 (1 - d^2), 0, 1).
pub fn face_weight(dx: f32, dy: f32) -> f32 {
    (2.0 * (1.0 - (dx * dx + dy * dy))).clamp(0.0, 1.0)
}

/// The rect a mark of width `w` covers, centred on (cx, cy) in composed px: (x0, y0, w, h).
pub fn mark_rect(m: Mark, w: i32, cx: f32, cy: f32) -> (i32, i32, i32, i32) {
    let bm = m.bitmap();
    let h = ((w as f32 * bm.len() as f32 / bm[0].len() as f32).round() as i32).max(1);
    ((cx - w as f32 / 2.0).round() as i32, (cy - h as f32 / 2.0).round() as i32, w, h)
}

/// Draws mark `m` in colour `c` at alpha factor `a`, `w` px wide, centred on (cx, cy), clipped to [lo, hi).
#[allow(clippy::too_many_arguments)]
fn draw_mark(img: &mut Image, m: Mark, c: Rgba, w: i32, a: f32, cx: f32, cy: f32, lo: i32, hi: i32) {
    let bm = m.bitmap();
    let (bw, bh) = (bm[0].len() as i32, bm.len() as i32);
    let (x0, y0, w, h) = mark_rect(m, w, cx, cy);
    for dy in 0..h {
        let row = bm[(dy * bh / h) as usize].as_bytes();
        for dx in 0..w {
            if let Some(col) = Mark::cell(row[(dx * bw / w) as usize] as char, c) {
                over(img, x0 + dx, y0 + dy, alpha(col, 255.0 * a), lo, hi);
            }
        }
    }
}

// ---- Drawing options, budget --------------------------------------------------------------------------------------

/// How a portrait draws: reduce motion (the setting), the rim key when the sidecar names none, the anchors check.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PortraitOpts {
    pub reduce_motion: bool,
    pub rim_default: Option<Rgba>,
    pub show_anchors: bool,
}

/// One per screen: `begin` at the start of its draw; at most `MAX_ANIMATED` time-varying portraits move per frame,
/// the rest draw their last composed image (frozen time, current states).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Budget {
    pub used: usize,
}

impl Budget {
    pub fn begin(&mut self) {
        self.used = 0;
    }

    fn take(&mut self) -> bool {
        if self.used < MAX_ANIMATED {
            self.used += 1;
            true
        } else {
            false
        }
    }
}

// ---- The portrait ---------------------------------------------------------------------------------------------------

/// Everything a composed image depends on: it recomposes only when this changes.
#[derive(Debug, Clone, PartialEq)]
struct Key {
    src: String,
    s: i32,
    tier: Tier,
    reduce: bool,
    anchors: bool,
    rim: Option<Rgba>,
    face: Option<Face>,
    /// The time read when something moves continuously (breath, rim, pulses, ash, smoke).
    t: Option<u64>,
    /// The blink phase (0 open, 1 half, 2 closed).
    blink: u8,
    still: bool,
    /// Every live effect's intensity in 1/32 and its colour.
    levels: Vec<(&'static str, u32, Rgba)>,
    /// PORTRAITFX1-EX: the expression effect drawn (its tint in 1/32, its mark's width and alpha in 1/32).
    expr: Option<ExprKey>,
}

/// The cache key part of an expression effect.
#[derive(Debug, Clone, PartialEq)]
struct ExprKey {
    expression: String,
    tint: Option<(Rgba, u32)>,
    mark: Option<(Mark, Rgba, i32, u32)>,
}

/// What a composition draws of an expression effect: the row, its tint (colour, strength now) and its mark (shape,
/// colour, width in composed px, alpha factor).
struct ExprDraw {
    row: ExpressionFx,
    tint: Option<(Rgba, f32)>,
    mark: Option<(Mark, Rgba, i32, f32)>,
}

/// What one draw composes: the key, the time it reads and the exact levels.
struct Plan {
    key: Key,
    t: u64,
    breath: bool,
    rim_f: f32,
    blinking: bool,
    levels: BTreeMap<&'static str, Level>,
    expr: Option<ExprDraw>,
}

/// The context of the last draw (for `state`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Last {
    tier: Tier,
    reduce: bool,
    rim: bool,
    blink: bool,
}

/// One portrait's effects, timing and cache (spec 4.9).
#[derive(Debug, Clone)]
pub struct PortraitFx {
    pub key: String,
    pub seed: u64,
    holds: BTreeMap<String, Held>,
    timed: Vec<(Cue, u64)>,
    jolt: Camera,
    cache: Option<(Key, Image)>,
    /// The time the cached image read (a frozen portrait keeps it).
    frozen: Option<u64>,
    last: Option<Last>,
    /// How many times it composed (the budget and cache checks).
    pub composes: u64,
    /// PORTRAITFX1-EX: the caller's expression-effect rows (`set_expressions`).
    exprs: Vec<ExpressionFx>,
    /// The expression-effect parts the last draw drew (`tint`, `mark`; for `state`).
    ex_live: Vec<&'static str>,
}

/// The effects a composition draws from cues, in order.
const DRAWN: [&str; 10] = ["still", "desaturate", "tint", "glow", "veins", "eye_glow", "ash", "smoke", "flash", "fade"];

impl PortraitFx {
    /// A portrait keyed by `key` (the character id, else the unit id, else the path): its seed is FNV-1a 64 of it.
    pub fn new(key: &str) -> Self {
        Self { key: key.to_string(), seed: seed_of(key), holds: BTreeMap::new(), timed: Vec::new(), jolt: Camera::default(), cache: None, frozen: None, last: None, composes: 0, exprs: Vec::new(), ex_live: Vec::new() }
    }

    /// PORTRAITFX1-EX: the expression-effect rows this portrait draws (the caller's table; replaces the previous one).
    pub fn set_expressions(&mut self, rows: &[ExpressionFx]) {
        if self.exprs != rows {
            self.exprs = rows.to_vec();
        }
    }

    /// The expression-effect rows set.
    pub fn expressions(&self) -> &[ExpressionFx] {
        &self.exprs
    }

    /// Sets the state cues that hold from `now` (the sidecar's `idle`, the game's state rows), replacing the previous
    /// set: a key that appears eases in from 0 over `EASE_MS`, one that leaves eases out. Jolts are never held.
    pub fn hold(&mut self, cues: &[Cue], now: u64) {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for c in cues.iter().filter(|c| c.effect != "jolt") {
            let mut key = c.key.clone();
            let mut n = 1;
            while seen.contains(&key) {
                n += 1;
                key = format!("{}#{n}", c.key);
            }
            seen.insert(key.clone());
            match self.holds.get_mut(&key) {
                Some(h) => {
                    // PORTRAITFX1-EX: a held face that changes its expression starts again (its effects run from now)
                    if h.cue.effect == "face" && h.gone.is_none() && h.cue.expression != c.expression {
                        h.since = now;
                    }
                    if h.gone.is_some() {
                        let lv = h.ease(now);
                        h.since = now.saturating_sub((lv * EASE_MS as f32).round() as u64);
                        h.gone = None;
                    }
                    h.cue = c.clone();
                }
                None => {
                    self.holds.insert(key, Held { cue: c.clone(), since: now, gone: None });
                }
            }
        }
        for (k, h) in self.holds.iter_mut() {
            if !seen.contains(k) && h.gone.is_none() {
                h.gone = Some(now);
            }
        }
        self.prune(now);
    }

    /// Starts a reaction at `now`: a `flash` (`ms` 0: `FLASH_MS`) and a `jolt` run once; any other effect with `ms`
    /// above 0 runs for `ms` at intensity x (1 - u) (a `face` cue holds its expression for `ms`, no fade).
    pub fn fire(&mut self, cue: Cue, now: u64) {
        if cue.effect == "jolt" {
            if let Some(s) = cue.shake() {
                self.fire_jolt(s, now);
            }
            return;
        }
        let ms = if cue.effect == "flash" && cue.ms == 0 { FLASH_MS } else { cue.ms };
        if ms > 0 {
            self.timed.push((Cue { ms, ..cue }, now));
        }
    }

    /// Starts a jolt of `shake` (clamped to `JOLT_LIMITS`) at `now`; it replaces the running one only when its
    /// strength is at least that one's current strength (`fx::Camera::start`). True when it started.
    pub fn fire_jolt(&mut self, shake: Shake, now: u64) -> bool {
        self.jolt.start(shake, now, self.seed ^ now, JOLT_LIMITS, false)
    }

    /// Stops the running jolt.
    pub fn stop_jolt(&mut self) {
        self.jolt.stop();
    }

    /// The running jolt's strength at `now` (0: none).
    pub fn jolt_amplitude(&self, now: u64) -> f32 {
        self.jolt.amplitude(now)
    }

    fn prune(&mut self, now: u64) {
        self.timed.retain(|(c, s)| now < s + c.ms as u64);
        self.holds.retain(|_, h| h.gone.is_none_or(|g| now < g + EASE_MS));
        if !self.jolt.active(now) {
            self.jolt.stop();
        }
    }

    /// The strength of `effect` at `now`: the strongest of its held and fired cues (never summed), with that cue's
    /// colour (ties: the first key in sort order); None when nothing gives it a strength above 0.
    pub fn level(&self, effect: &str, now: u64) -> Option<Level> {
        let held = self.holds.iter().filter(|(_, h)| h.cue.effect == effect).map(|(k, h)| (h.cue.intensity * h.ease(now), k.as_str(), &h.cue, h.since));
        let fired = self.timed.iter().filter(|(c, s)| c.effect == effect && now >= *s && now < s + c.ms as u64).map(|(c, s)| {
            let u = (now - s) as f32 / c.ms.max(1) as f32;
            (if effect == "face" { c.intensity } else { c.intensity * (1.0 - u) }, c.key.as_str(), c, *s)
        });
        let mut best: Option<(f32, &str, &Cue, u64)> = None;
        for (i, k, c, s) in held.chain(fired).filter(|x| x.0 > 0.0) {
            if best.is_none_or(|(bi, bk, _, _)| i > bi || (i == bi && k < bk)) {
                best = Some((i, k, c, s));
            }
        }
        best.map(|(i, _, c, start)| Level { i, color: c.color.unwrap_or_else(|| default_color(c.effect)), start })
    }

    /// The winning `face` cue's expression at `now` (held, or fired and within its `ms`): the highest priority, then
    /// the latest started; "" with none.
    pub fn expression(&self, now: u64) -> &str {
        self.expression_since(now).map_or("", |(e, _)| e)
    }

    /// The winning `face` cue's expression and when that cue started (None: no face cue).
    pub fn expression_since(&self, now: u64) -> Option<(&str, u64)> {
        let held = self.holds.values().filter(|h| h.cue.effect == "face" && h.gone.is_none() && h.since <= now).map(|h| (&h.cue, h.since));
        let fired = self.timed.iter().filter(|(c, s)| c.effect == "face" && now >= *s && now < s + c.ms as u64).map(|(c, s)| (c, *s));
        let mut best: Option<(&Cue, u64)> = None;
        for (c, s) in held.chain(fired) {
            if best.is_none_or(|(b, bs)| (c.priority, s) >= (b.priority, bs)) {
                best = Some((c, s));
            }
        }
        best.map(|(c, s)| (c.expression.as_str(), s))
    }

    /// PORTRAITFX1-EX: the expression-effect row of the expression at `now` and when it started (None: no face cue, or
    /// no row for its expression).
    pub fn expression_fx(&self, now: u64) -> Option<(&ExpressionFx, u64)> {
        let (e, since) = self.expression_since(now)?;
        self.exprs.iter().find(|r| !e.is_empty() && r.expression == e).map(|r| (r, since))
    }

    /// The expression shake's offset at `now` in whole logical px for the last draw's tier (halved at reduced, at
    /// least 1 px; none at minimal or under reduce motion), from the expression's start.
    pub fn expression_shake(&self, now: u64) -> (i32, i32) {
        let Some(l) = self.last else { return (0, 0) };
        if l.reduce || l.tier == Tier::Minimal {
            return (0, 0);
        }
        match self.expression_fx(now).and_then(|(r, since)| Some((r.shake()?, since))) {
            Some((s, since)) if now >= since => {
                let s = if l.tier == Tier::Reduced { Shake { px: (s.px / 2.0).max(1.0), ..s } } else { s };
                s.offset(now - since, self.seed ^ since)
            }
            _ => (0, 0),
        }
    }

    /// Draws the portrait at `rel` (the path the caller's chain resolved to, `Art::first_rel`) into the logical rect
    /// `r` at clock `now`, with its sidecar `face` (None: no anchors); false when `rel` loads nothing (the caller draws
    /// its fallback).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, p: &mut Painter, art: &mut Art, rel: &str, face: Option<&Face>, r: Rect, now: u64, opts: &PortraitOpts, budget: &mut Budget) -> bool {
        if rel.is_empty() || !art.has(rel) {
            return false;
        }
        let side = side_of(p, r);
        if side <= 0 {
            return true;
        }
        let plan = self.plan(rel, face, side, now, opts, budget);
        if self.cache.as_ref().map(|c| &c.0) != Some(&plan.key) {
            let s = plan.key.s;
            let Some(src) = art.image(rel, s, s) else { return false };
            let img = compose(src, face, &plan, self.seed);
            self.store(plan, img);
        }
        self.blit(p, r, now);
        true
    }

    /// `draw` with an image the caller holds (a code-drawn face, the Kit widgets sample); `id` names it in the cache.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_image(&mut self, p: &mut Painter, src: &Image, id: &str, face: Option<&Face>, r: Rect, now: u64, opts: &PortraitOpts, budget: &mut Budget) {
        let side = side_of(p, r);
        if side <= 0 {
            return;
        }
        let plan = self.plan(id, face, side, now, opts, budget);
        if self.cache.as_ref().map(|c| &c.0) != Some(&plan.key) {
            let img = compose(src, face, &plan, self.seed);
            self.store(plan, img);
        }
        self.blit(p, r, now);
    }

    /// The last composed image (None before the first draw).
    pub fn composed(&self) -> Option<&Image> {
        self.cache.as_ref().map(|c| &c.1)
    }

    fn store(&mut self, plan: Plan, img: Image) {
        self.frozen = Some(plan.t);
        self.cache = Some((plan.key, img));
        self.composes += 1;
    }

    /// What a draw at `side` device px and `now` composes, and the context `state` reads.
    fn plan(&mut self, src: &str, face: Option<&Face>, side: i32, now: u64, opts: &PortraitOpts, budget: &mut Budget) -> Plan {
        self.prune(now);
        let tier = Tier::of(side);
        let s = side.min(COMPOSE_MAX);
        let reduce = opts.reduce_motion;
        // Every level (eases, fired cues) is read at the 30-a-second step too, so nothing recomposes more often.
        let now_q = tq(now);
        let mut levels: BTreeMap<&'static str, Level> = BTreeMap::new();
        for e in DRAWN {
            if !tier.allows(e) || (reduce && matches!(e, "ash" | "smoke")) {
                continue;
            }
            if let Some(l) = self.level(e, now_q) {
                levels.insert(e, l);
            }
        }
        let still = levels.contains_key("still");
        let rim = face.and_then(|f| f.rim).map(rgb3).or(opts.rim_default);
        let breath = tier == Tier::Full && !still && !reduce;
        let rim_live = rim.is_some() && tier.allows("rim") && !still && !reduce;
        let k = face.map_or(1.0, |f| s as f32 / f.size.max(1) as f32);
        let can_blink = face.is_some_and(|f| f.blink && !f.eyes.is_empty() && tier.allows("blink") && (tier == Tier::Full || f.eyes.iter().any(|e| (e[3] as f32 * k).round() >= 2.0)));
        let blinking = can_blink && !still;
        let moving = |e: &str| levels.contains_key(e) && !reduce;
        // PORTRAITFX1-EX: the expression's row (its tint at every tier, its mark from reduced up; a pulse moves it)
        let ex = self.expression_fx(now_q).map(|(r, since)| (r.clone(), since));
        let pulsing = ex.as_ref().is_some_and(|(r, _)| r.mark.is_some() && r.mark_pulse_ms > 0 && tier != Tier::Minimal && !reduce);
        let animated = breath || rim_live || moving("glow") || moving("veins") || (moving("eye_glow") && tier != Tier::Minimal) || moving("ash") || moving("smoke") || pulsing;
        let t = if (animated || blinking) && !budget.take() { self.frozen.unwrap_or(now_q) } else { now_q };
        let expr = ex.map(|(row, since)| {
            let tint = row.tint.map(|c| (c, row.tint_at(now_q.saturating_sub(since)))).filter(|(_, i)| *i > 0.0);
            let mark = row.mark.filter(|_| tier != Tier::Minimal).map(|m| {
                let (grow, a) = if pulsing { mark_pulse(row.pulse(t)) } else { (1.0, 1.0) };
                (m, row.mark_color.unwrap_or_else(|| m.default_color()), mark_width(m, row.mark_size, s, grow), a)
            });
            ExprDraw { row, tint, mark }
        });
        let face_box = face.and_then(Face::face_box).is_some();
        self.ex_live = expr.as_ref().map_or(Vec::new(), |x| {
            let mut v = Vec::new();
            if x.tint.is_some() && face_box {
                v.push("tint");
            }
            if x.mark.is_some() && face.and_then(|f| x.row.anchor_box(f)).is_some() {
                v.push("mark");
            }
            v
        });
        let rim_f = if rim_live { rim_factor(self.seed, t) } else { 1.0 };
        let blink = if blinking { blink_phase(self.seed, t) } else { 0 };
        self.last = Some(Last { tier, reduce, rim: rim.is_some(), blink: can_blink });
        let q = levels.iter().filter(|(e, _)| **e != "still").map(|(e, l)| (*e, (l.i * 32.0).round() as u32, l.color)).collect();
        let ex_key = expr.as_ref().map(|x| ExprKey {
            expression: x.row.expression.clone(),
            tint: x.tint.map(|(c, i)| (c, (i * 32.0).round() as u32)),
            mark: x.mark.map(|(m, c, w, a)| (m, c, w, (a * 32.0).round() as u32)),
        });
        let key = Key { src: src.to_string(), s, tier, reduce, anchors: opts.show_anchors, rim: if rim_live { rim } else { None }, face: face.cloned(), t: animated.then_some(t), blink, still, levels: q, expr: ex_key };
        Plan { key, t, breath, rim_f, blinking, levels, expr }
    }

    /// The jolt offset at `now` in whole logical px for the last draw's tier (halved at reduced, at least 1 px; none at
    /// minimal or under reduce motion).
    pub fn jolt_offset(&self, now: u64) -> (i32, i32) {
        let Some(l) = self.last else { return self.jolt.offset_at(now) };
        if l.reduce || l.tier == Tier::Minimal {
            return (0, 0);
        }
        match self.jolt.shake {
            Some((s, start, seed)) if now >= start => {
                let s = if l.tier == Tier::Reduced { Shake { px: (s.px / 2.0).max(1.0), ..s } } else { s };
                s.offset(now - start, seed)
            }
            _ => (0, 0),
        }
    }

    fn blit(&self, p: &mut Painter, r: Rect, now: u64) {
        let Some((_, img)) = &self.cache else { return };
        let (dx, dy) = self.offset(now);
        p.image(img, Rect::new(r.x + dx, r.y + dy, r.w, r.h));
    }

    /// The portrait's offset at `now`: the jolt plus the expression shake, each axis within the jolt ceiling.
    pub fn offset(&self, now: u64) -> (i32, i32) {
        let ((jx, jy), (sx, sy)) = (self.jolt_offset(now), self.expression_shake(now));
        let m = JOLT_LIMITS.max_px as i32;
        ((jx + sx).clamp(-m, m), (jy + sy).clamp(-m, m))
    }

    /// The effects live at `now` for the last draw's tier and options: base life first, then `EFFECTS` order.
    pub fn live(&self, now: u64) -> Vec<&'static str> {
        let Some(l) = self.last else { return Vec::new() };
        let still = l.tier.allows("still") && self.level("still", now).is_some();
        let mut out = Vec::new();
        if l.tier == Tier::Full && !still && !l.reduce {
            out.push("breath");
        }
        if l.rim && l.tier.allows("rim") && !still && !l.reduce {
            out.push("rim");
        }
        if l.blink && !still {
            out.push("blink");
        }
        for e in EFFECTS {
            let on = match e {
                "jolt" => !l.reduce && l.tier != Tier::Minimal && self.jolt.active(now),
                "face" => !self.expression(now).is_empty(),
                _ => l.tier.allows(e) && !(l.reduce && matches!(e, "ash" | "smoke")) && self.level(e, now).is_some(),
            };
            if on {
                out.push(e);
            }
        }
        out
    }

    /// The tier of the last draw (None before it).
    pub fn tier(&self) -> Option<Tier> {
        self.last.map(|l| l.tier)
    }

    /// The agent link's view (spec 4.11): `{key, expression, tier, effects, jolt: [dx, dy]}`; PORTRAITFX1-EX adds
    /// `expression_fx` (the parts the last draw drew: `tint`, `mark`) and `shake` ([dx, dy] of the expression shake).
    pub fn state(&self, now: u64) -> Value {
        let (dx, dy) = self.jolt_offset(now);
        let (sx, sy) = self.expression_shake(now);
        json!({"key": self.key, "expression": self.expression(now), "tier": self.tier().map_or("", Tier::name), "effects": self.live(now), "jolt": [dx, dy], "expression_fx": self.ex_live, "shake": [sx, sy]})
    }
}

/// The side of the logical rect `r` in device px.
fn side_of(p: &Painter, r: Rect) -> i32 {
    let pr = p.v.rect(r);
    pr.w.min(pr.h)
}

// ---- Timing rules ---------------------------------------------------------------------------------------------------

/// The breath period for `seed`: 3600 x (0.9 + 0.2 h(1)) ms.
pub fn breath_period(seed: u64) -> f32 {
    3600.0 * (0.9 + 0.2 * hash01(seed, 1))
}

/// The breath amplitude at composed side `s`: max(1, round(0.012 s)) px.
pub fn breath_amp(s: i32) -> i32 {
    ((0.012 * s as f32).round() as i32).max(1)
}

/// The angle 2 pi t / `period` (ms), taken from `t mod period` in f64 first, so motion stays smooth after hours of
/// clock time (an f32 of a large `t` loses its milliseconds).
pub fn cycle(t: u64, period: f64) -> f32 {
    (std::f64::consts::TAU * ((t as f64) % period) / period) as f32
}

/// `x` mod `w` for a drift of `px_per_s` px a second over `t` ms, in f64 first (as `cycle`).
fn drift(t: u64, px_per_s: f32, w: f32) -> f32 {
    ((px_per_s as f64 * t as f64 / 1000.0) % w as f64) as f32
}

/// The breath stretch at `t` in whole px: amp (1 - cos(2 pi t / period)) / 2, rounded.
pub fn breath_px(seed: u64, t: u64, s: i32) -> i32 {
    let a = breath_amp(s) as f32 * (1.0 - cycle(t, breath_period(seed) as f64).cos()) / 2.0;
    a.round() as i32
}

/// The rim value factor at `t`: 1 + 0.12 (0.6 sin(2 pi t / 1700) + 0.4 (2 h(100 + floor(t / 90)) - 1)).
pub fn rim_factor(seed: u64, t: u64) -> f32 {
    1.0 + 0.12 * (0.6 * cycle(t, 1700.0).sin() + 0.4 * (2.0 * hash01(seed, 100 + t / 90) - 1.0))
}

/// The start of every blink before `until` (a double blink's second included): blink n starts
/// round(2600 + 3400 h(200 + n)) ms after blink n - 1 ends; when h(300 + n) < 0.12 a second follows 220 ms after its
/// start.
pub fn blink_starts(seed: u64, until: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let (mut end, mut n) = (0u64, 0u64);
    loop {
        let start = end + (2600.0 + 3400.0 * hash01(seed, 200 + n) as f64).round() as u64;
        if start >= until {
            return out;
        }
        out.push(start);
        end = start + BLINK_MS;
        if hash01(seed, 300 + n) < 0.12 {
            let second = start + DOUBLE_BLINK_MS;
            if second < until {
                out.push(second);
            }
            end = second + BLINK_MS;
        }
        n += 1;
    }
}

/// The blink phase at `t`: 0 open, 1 half closed (the first 40 ms and the last 40 ms of a blink), 2 closed (40-90).
pub fn blink_phase(seed: u64, t: u64) -> u8 {
    let (mut end, mut n) = (0u64, 0u64);
    loop {
        let start = end + (2600.0 + 3400.0 * hash01(seed, 200 + n) as f64).round() as u64;
        if t < start {
            return 0;
        }
        let mut starts = vec![start];
        end = start + BLINK_MS;
        if hash01(seed, 300 + n) < 0.12 {
            starts.push(start + DOUBLE_BLINK_MS);
            end = start + DOUBLE_BLINK_MS + BLINK_MS;
        }
        if t < end {
            return starts.iter().filter(|s| t >= **s && t < **s + BLINK_MS).map(|s| if (40..90).contains(&(t - s)) { 2 } else { 1 }).next().unwrap_or(0);
        }
        n += 1;
    }
}

/// Whether an eye-glow flare shows at `t`: flare n starts round(3000 + 4000 h(500 + n)) ms after flare n - 1 starts
/// (the first after 0) and lasts `FLARE_MS`.
pub fn flare_on(seed: u64, t: u64) -> bool {
    let (mut at, mut n) = (0u64, 0u64);
    loop {
        at += (3000.0 + 4000.0 * hash01(seed, 500 + n) as f64).round() as u64;
        if t < at {
            return false;
        }
        if t < at + FLARE_MS {
            return true;
        }
        n += 1;
    }
}

/// The ash specks' top-left corners at `t` for intensity `i`: min(32, round(18 i)) specks (halved, rounded up, at
/// reduced), speck k at h(600 + k) of the inner width plus a 2 % sway (sin(t / 900 + k)), starting h(650 + k) down
/// and falling (0.03 + 0.03 h(700 + k)) x `s` px a second, wrapping inside the inner rect [lo, hi).
pub fn ash_specks(seed: u64, t: u64, i: f32, tier: Tier, s: i32, lo: i32, hi: i32) -> Vec<(i32, i32)> {
    let n = ((18.0 * i).round() as usize).min(32);
    let n = if tier == Tier::Reduced { n.div_ceil(2) } else { n };
    let w = (hi - lo).max(1) as f32;
    // sin(t / 900 + k): t / 900 taken mod 2 pi first.
    let sway = cycle(t, 900.0 * std::f64::consts::TAU);
    (0..n as u64)
        .map(|k| {
            let x = (hash01(seed, 600 + k) * w + 0.02 * w * (sway + k as f32).sin()).rem_euclid(w);
            let v = (0.03 + 0.03 * hash01(seed, 700 + k)) * s as f32;
            let y = (hash01(seed, 650 + k) * w + drift(t, v, w)).rem_euclid(w);
            (lo + x.floor() as i32, lo + y.floor() as i32)
        })
        .collect()
}

// ---- Composition ----------------------------------------------------------------------------------------------------

fn get(img: &Image, x: i32, y: i32) -> Rgba {
    let i = ((y * img.w + x) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]
}

fn put(img: &mut Image, x: i32, y: i32, c: Rgba) {
    let i = ((y * img.w + x) * 4) as usize;
    img.px[i..i + 4].copy_from_slice(&c);
}

/// `c` over the pixel (x, y) when it lies in [lo, hi) on both axes.
fn over(img: &mut Image, x: i32, y: i32, c: Rgba, lo: i32, hi: i32) {
    if x < lo || y < lo || x >= hi || y >= hi || c[3] == 0 {
        return;
    }
    let d = get(img, x, y);
    let (a, da) = (c[3] as f32 / 255.0, d[3] as f32 / 255.0);
    let oa = a + da * (1.0 - a);
    if oa <= 0.0 {
        return;
    }
    let ch = |k: usize| ((c[k] as f32 * a + d[k] as f32 * da * (1.0 - a)) / oa).round().clamp(0.0, 255.0) as u8;
    put(img, x, y, [ch(0), ch(1), ch(2), (oa * 255.0).round() as u8]);
}

fn alpha(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], a.round().clamp(0.0, 255.0) as u8]
}

#[allow(clippy::too_many_arguments)]
fn fill_ellipse(img: &mut Image, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgba, lo: i32, hi: i32) {
    let (rx, ry) = (rx.max(0.5), ry.max(0.5));
    for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
        for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            if dx * dx + dy * dy <= 1.0 {
                over(img, x, y, c, lo, hi);
            }
        }
    }
}

fn hsv(c: Rgba) -> (f32, f32, f32) {
    let (r, g, b) = (c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let d = max - min;
    let h = if d <= 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    (h, if max <= 0.0 { 0.0 } else { d / max }, max)
}

/// Whether `c` is lit by the rim key: hue within 18 deg, saturation and value at least 0.45.
pub fn rim_lit(c: Rgba, key: Rgba) -> bool {
    let ((h, s, v), (kh, _, _)) = (hsv(c), hsv(key));
    let dh = (h - kh).abs();
    dh.min(360.0 - dh) <= 18.0 && s >= 0.45 && v >= 0.45
}

/// The luminance 0.299 r + 0.587 g + 0.114 b.
pub fn luminance(c: Rgba) -> f32 {
    0.299 * c[0] as f32 + 0.587 * c[1] as f32 + 0.114 * c[2] as f32
}

/// `desaturate` at `i`: lerp(c, lum, i) per channel.
pub fn desaturated(c: Rgba, i: f32) -> Rgba {
    let l = luminance(c);
    let ch = |k: usize| (c[k] as f32 + (l - c[k] as f32) * i).round().clamp(0.0, 255.0) as u8;
    [ch(0), ch(1), ch(2), c[3]]
}

/// `tint` at `i` toward `col`: target = min(255, lum / 255 x col x 1.25), lerp(c, target, 0.6 i).
pub fn tinted(c: Rgba, col: Rgba, i: f32) -> Rgba {
    let l = luminance(c);
    let ch = |k: usize| {
        let target = (l / 255.0 * col[k] as f32 * 1.25).min(255.0);
        (c[k] as f32 + (target - c[k] as f32) * 0.6 * i).round().clamp(0.0, 255.0) as u8
    };
    [ch(0), ch(1), ch(2), c[3]]
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

/// A vein polyline: `segs` segments of `step` px from `from`, segment j turned 25 (2 h(k0 + j) - 1) deg from `deg`.
fn vein_points(seed: u64, from: (f32, f32), deg: f32, step: f32, segs: usize, k0: u64) -> Vec<(f32, f32)> {
    let mut pts = vec![from];
    for j in 0..segs {
        let a = (deg + 25.0 * (2.0 * hash01(seed, k0 + j as u64) - 1.0)).to_radians();
        let p = pts[j];
        pts.push((p.0 + step * a.cos(), p.1 + step * a.sin()));
    }
    pts
}

/// Composes the portrait (spec 4.3 steps 1-8, then the anchors check) at the plan's side.
fn compose(src: &Image, face: Option<&Face>, plan: &Plan, seed: u64) -> Image {
    let key = &plan.key;
    let s = key.s;
    let mut img = if src.w == s && src.h == s { src.clone() } else { src.downsample(s, s) };
    let plain;
    let face = match face {
        Some(f) => f,
        None => {
            plain = Face::plain(s);
            &plain
        }
    };
    let k = s as f32 / face.size.max(1) as f32;
    let b = ((face.border as f32 * k).round() as i32).clamp(0, (s - 1) / 2);
    let (lo, hi) = (b, s - b);
    let h_in = (hi - lo).max(1);
    let t = plan.t;
    let lv = |e: &str| plan.levels.get(e).copied();
    let reduce = key.reduce;
    let tier = key.tier;

    // 2. breath: the inner rect stretched up from its bottom edge (nearest rows).
    let a = if plan.breath { breath_px(seed, t, s) } else { 0 };
    if a > 0 {
        let orig = img.clone();
        for yd in lo..hi {
            let d = (hi - 1 - yd) as i64;
            let ds = (d * h_in as i64 / (h_in + a) as i64) as i32;
            let ys = hi - 1 - ds;
            for x in lo..hi {
                put(&mut img, x, yd, get(&orig, x, ys));
            }
        }
    }
    // Anchors follow the breath: y' = bottom - (bottom - y) (H + a) / H.
    let by = |y: f32| hi as f32 - (hi as f32 - y) * (h_in + a) as f32 / h_in as f32;

    // 3. colour: rim, desaturate, tint.
    let rim = key.rim.filter(|_| plan.rim_f != 1.0);
    let (des, tint) = (lv("desaturate"), lv("tint"));
    if rim.is_some() || des.is_some() || tint.is_some() {
        for y in lo..hi {
            for x in lo..hi {
                let mut c = get(&img, x, y);
                if let Some(rk) = rim {
                    if rim_lit(c, rk) {
                        for ch in c.iter_mut().take(3) {
                            *ch = (*ch as f32 * plan.rim_f).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                }
                if let Some(l) = des {
                    c = desaturated(c, l.i);
                }
                if let Some(l) = tint {
                    c = tinted(c, l.color, l.i);
                }
                put(&mut img, x, y, c);
            }
        }
    }

    // 3b. PORTRAITFX1-EX: the expression's tint over the face box (a soft ellipse inside it; no box, no tint).
    if let (Some((col, i)), Some(fb)) = (plan.expr.as_ref().and_then(|x| x.tint), face.face_box()) {
        let (x0, x1) = (fb[0] as f32 * k, (fb[0] + fb[2]) as f32 * k);
        let (y0, y1) = (by(fb[1] as f32 * k), by((fb[1] + fb[3]) as f32 * k));
        let (cx, cy, rx, ry) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0, ((x1 - x0) / 2.0).max(0.5), ((y1 - y0) / 2.0).max(0.5));
        for y in (y0.floor() as i32).max(lo)..(y1.ceil() as i32).min(hi) {
            for x in (x0.floor() as i32).max(lo)..(x1.ceil() as i32).min(hi) {
                let w = face_weight((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
                let c = get(&img, x, y);
                if w > 0.0 && c[3] > 0 {
                    put(&mut img, x, y, tinted(c, col, i * w));
                }
            }
        }
    }

    // The eye boxes in composed px: [x0, x1) x [y0, y1).
    let eyes: Vec<(i32, i32, i32, i32)> = face
        .eyes
        .iter()
        .map(|e| {
            let x0 = (e[0] as f32 * k).round() as i32;
            let x1 = ((e[0] + e[2]) as f32 * k).round() as i32;
            let y0 = by(e[1] as f32 * k).round() as i32;
            let y1 = by((e[1] + e[3]) as f32 * k).round() as i32;
            (x0, y0, x1.max(x0 + 1), y1.max(y0 + 1))
        })
        .collect();

    // 4. blink lids.
    if plan.blinking && key.blink > 0 {
        let lids: Vec<Rgba> = eyes.iter().map(|&(x0, y0, x1, _)| face.lid.map(rgb3).unwrap_or_else(|| get(&img, ((x0 + x1) / 2).clamp(lo, hi - 1), (y0 - 1).clamp(lo, hi - 1)))).collect();
        for (&(x0, y0, x1, y1), lid) in eyes.iter().zip(lids) {
            let h = y1 - y0;
            if tier != Tier::Full && h < 2 {
                continue;
            }
            let lid = [lid[0], lid[1], lid[2], 255];
            let rows = if key.blink == 2 { h } else { (h + 1) / 2 };
            for y in y0..y0 + rows {
                for x in x0..x1 {
                    over(&mut img, x, y, lid, lo, hi);
                }
            }
            if key.blink == 2 {
                let dark = [(lid[0] as f32 * 0.6) as u8, (lid[1] as f32 * 0.6) as u8, (lid[2] as f32 * 0.6) as u8, 255];
                for x in x0..x1 {
                    over(&mut img, x, y0 + 2 * h / 3, dark, lo, hi);
                }
            }
        }
    }

    // 5. overlays: glow, veins, eye glow.
    if let Some(l) = lv("glow") {
        let pulse = if reduce { 0.8 } else { 0.6 + 0.4 * cycle(t, 1800.0).sin() };
        let anchors: Vec<(f32, f32, f32)> = if face.glow.is_empty() {
            vec![(lo as f32 + 0.5 * h_in as f32, lo as f32 + 0.7 * h_in as f32, 0.15 * s as f32)]
        } else {
            face.glow.iter().map(|g| (g[0] as f32 * k, by(g[1] as f32 * k), g[2] as f32 * k)).collect()
        };
        for (cx, cy, r) in anchors {
            for (f, a) in [(1.0, 40.0), (0.66, 60.0), (0.33, 90.0)] {
                fill_ellipse(&mut img, cx, cy, r * f, r * f, alpha(l.color, a * l.i * pulse), lo, hi);
            }
        }
    }
    if let Some(l) = lv("veins") {
        let pulse = if reduce { 0.8 } else { 0.6 + 0.4 * cycle(t, 2400.0).sin() };
        let grown = if reduce { 1.0 } else { (t.saturating_sub(l.start) as f32 / VEIN_GROW_MS as f32).clamp(0.0, 1.0) };
        let wide = tier == Tier::Full && s > 192;
        let step = (3.0 * k).max(1.0);
        for (ri, v) in face.veins.iter().enumerate() {
            let root = (v[0] as f32 * k, by(v[1] as f32 * k));
            let len = v[3] as f32 * k;
            let segs = (v[3] as f32 / 3.0).ceil().max(1.0) as usize;
            let k0 = 400 + ri as u64 * 1000;
            let trunk = vein_points(seed, root, v[2] as f32, step, segs, k0);
            // Trunk first (the first 60 % of the growth), then the branches.
            let mut lines: Vec<(Vec<(f32, f32)>, usize)> = vec![(trunk.clone(), (segs as f32 * (grown / 0.6).min(1.0)).round() as usize)];
            let bsegs = ((0.4 * v[3] as f32) / 3.0).ceil().max(1.0) as usize;
            let bshow = (bsegs as f32 * ((grown - 0.6) / 0.4).clamp(0.0, 1.0)).round() as usize;
            for (bi, (at, turn)) in [(0.4f32, 35.0f32), (0.7, -35.0)].into_iter().enumerate() {
                let from = trunk[((segs as f32 * at).round() as usize).min(segs)];
                lines.push((vein_points(seed, from, v[2] as f32 + turn, step, bsegs, k0 + 200 * (bi as u64 + 1)), bshow));
            }
            let mut px: BTreeSet<(i32, i32)> = BTreeSet::new();
            for (pts, shown) in lines {
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
                    over(&mut img, x, y, col, lo, hi);
                }
            }
        }
    }
    if let Some(l) = lv("eye_glow") {
        let flare = if !reduce && tier != Tier::Minimal && flare_on(seed, t) { 1.4 } else { 1.0 };
        for &(x0, y0, x1, y1) in &eyes {
            let (w, h) = ((x1 - x0) as f32, (y1 - y0) as f32);
            let (cx, cy) = ((x0 + x1) as f32 / 2.0, (y0 + y1) as f32 / 2.0);
            if tier != Tier::Minimal {
                fill_ellipse(&mut img, cx, cy, 0.9 * w, 1.1 * h, alpha(l.color, 70.0 * l.i * flare), lo, hi);
            }
            fill_ellipse(&mut img, cx, cy, w / 2.0, h / 2.0, alpha(l.color, 200.0 * l.i * flare), lo, hi);
        }
    }

    // 5b. PORTRAITFX1-EX: the expression's mark, centred on its anchor in the face or eye box (no box, no mark).
    if let Some(ExprDraw { row, mark: Some((m, col, w, a)), .. }) = plan.expr.as_ref() {
        if let Some((ax, ay)) = row.anchor(face, k) {
            draw_mark(&mut img, *m, *col, *w, *a, ax, by(ay), lo, hi);
        }
    }

    // 6. ash, then smoke.
    if let Some(l) = lv("ash") {
        let sz = (s / 96).max(1);
        for (x, y) in ash_specks(seed, t, l.i, tier, s, lo, hi) {
            for yy in y..y + sz {
                for xx in x..x + sz {
                    over(&mut img, xx, yy, alpha(l.color, 160.0 * l.i), lo, hi);
                }
            }
        }
    }
    if let Some(l) = lv("smoke") {
        let w = h_in as f32;
        for kk in 0..4u64 {
            let r = (0.05 + 0.04 * hash01(seed, 800 + kk)) * s as f32;
            let x = lo as f32 + hash01(seed, 820 + kk) * w;
            let risen = (hash01(seed, 840 + kk) * w + drift(t, 0.04 * s as f32, w)).rem_euclid(w);
            let a = 50.0 * l.i * (1.0 - risen / w);
            fill_ellipse(&mut img, x, hi as f32 - risen, r, r, alpha(l.color, a), lo, hi);
        }
    }

    // 7. flash.
    if let Some(l) = lv("flash") {
        let c = alpha(l.color, 200.0 * l.i);
        for y in lo..hi {
            for x in lo..hi {
                over(&mut img, x, y, c, lo, hi);
            }
        }
    }

    // 8. fade: the whole image's alpha (border included), never below 0.3.
    if let Some(l) = lv("fade") {
        let f = (1.0 - 0.7 * l.i).max(0.3);
        for a in img.px.iter_mut().skip(3).step_by(4) {
            *a = (*a as f32 * f).round() as u8;
        }
    }

    // The anchors check: 1 px outlines of the eye boxes, glow circles and vein roots (inside the border too).
    if key.anchors {
        let ring = |img: &mut Image, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgba| {
            for x in x0..x1 {
                over(img, x, y0, c, lo, hi);
                over(img, x, y1 - 1, c, lo, hi);
            }
            for y in y0..y1 {
                over(img, x0, y, c, lo, hi);
                over(img, x1 - 1, y, c, lo, hi);
            }
        };
        for &(x0, y0, x1, y1) in &eyes {
            ring(&mut img, x0, y0, x1, y1, ANCHOR_EYE);
        }
        for v in &face.veins {
            let (x, y) = ((v[0] as f32 * k).round() as i32, by(v[1] as f32 * k).round() as i32);
            ring(&mut img, x - 1, y - 1, x + 2, y + 2, ANCHOR_VEIN);
        }
        for g in &face.glow {
            let (cx, cy, r) = (g[0] as f32 * k, by(g[1] as f32 * k), g[2] as f32 * k);
            let n = (r * 8.0).ceil().max(8.0) as i32;
            for j in 0..n {
                let a = 2.0 * PI * j as f32 / n as f32;
                over(&mut img, (cx + r * a.cos()).floor() as i32, (cy + r * a.sin()).floor() as i32, ANCHOR_GLOW, lo, hi);
            }
        }
    }
    img
}

// ---- The sample face ------------------------------------------------------------------------------------------------

/// The engine's code-drawn sample face (spec 4.10): a 96 x 96 bust on a dark glow inside a 3 px border, two 5 x 3 eye
/// boxes, an amber rim column on the head's right edge, one glow anchor on the chest and one vein root on the cheek;
/// and its sidecar.
pub fn sample() -> (Image, Face) {
    const N: i32 = 96;
    let mut img = Image { w: N, h: N, px: vec![0; (N * N * 4) as usize] };
    let skin: Rgba = [164, 122, 95, 255];
    let amber: Rgba = [217, 119, 6, 255];
    for y in 0..N {
        for x in 0..N {
            // The dark glow: brighter toward the head.
            let d = (((x - 48) * (x - 48) + (y - 40) * (y - 40)) as f32).sqrt() / 60.0;
            let g = (1.0 - d).clamp(0.0, 1.0);
            let mut c: Rgba = [(18.0 + 26.0 * g) as u8, (14.0 + 18.0 * g) as u8, (20.0 + 26.0 * g) as u8, 255];
            // Shoulders and cloak.
            let sh = ((x - 48) as f32 / 40.0).powi(2) + ((y - 100) as f32 / 34.0).powi(2);
            if sh <= 1.0 && y >= 68 {
                c = if (x + y) % 7 == 0 { [44, 50, 68, 255] } else { [56, 62, 84, 255] };
            }
            // Neck.
            if (42..54).contains(&x) && (52..70).contains(&y) {
                c = [140, 102, 80, 255];
            }
            // Head, hair on top.
            let hd = ((x - 48) as f32 / 15.0).powi(2) + ((y - 40) as f32 / 19.0).powi(2);
            if hd <= 1.0 {
                c = if y < 30 { [62, 42, 32, 255] } else { skin };
                // The amber rim on the head's right edge.
                let edge = ((x + 2 - 48) as f32 / 15.0).powi(2) + ((y - 40) as f32 / 19.0).powi(2);
                if x > 48 && edge > 1.0 && y >= 26 {
                    c = amber;
                }
            }
            img.px[((y * N + x) * 4) as usize..((y * N + x) * 4 + 4) as usize].copy_from_slice(&c);
        }
    }
    // The eyes: white with a dark iris.
    for (ex, ey) in [(38, 38), (53, 38)] {
        for y in ey..ey + 3 {
            for x in ex..ex + 5 {
                put(&mut img, x, y, [214, 204, 192, 255]);
            }
        }
        put(&mut img, ex + 2, ey + 1, [36, 26, 26, 255]);
        put(&mut img, ex + 1, ey + 1, [70, 50, 44, 255]);
    }
    // The 3 px border: a bronze frame with a dark inner line.
    for y in 0..N {
        for x in 0..N {
            let e = x.min(y).min(N - 1 - x).min(N - 1 - y);
            if e < 3 {
                put(&mut img, x, y, if e == 2 { [40, 30, 22, 255] } else { [120, 92, 58, 255] });
            }
        }
    }
    let face = Face { size: N, border: 3, eyes: vec![[38, 38, 5, 3], [53, 38, 5, 3]], blink: true, lid: None, glow: vec![[48, 80, 10]], veins: vec![[57, 47, 100, 14]], mouth: None, face: None, rim: Some([217, 119, 6]), idle: Vec::new() };
    (img, face)
}
