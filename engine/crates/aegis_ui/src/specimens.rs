//! The kit's widgets on one page (UIGALLERY1): every widget family of `ui.rs` in its states, on sample values, to be
//! judged on its own (a UI Gallery's "Kit widgets" entry). Game-agnostic: the gallery screen keeps a `Specimens`,
//! wraps `KitAct` in its own action and forwards drops and text changes to it. Pages, picked by the tabs at the
//! page's foot: the widgets, the block timeline with its segment parts, links and marks (RULESET1-E), and the effects
//! kit (FX1-E: each primitive and shake preset on a small stage, the clock's speed and hit-stop, a local reduce
//! motion); the Effects page runs on its own clock, fed by `tick` (wall time) or `advance` (tests). PORTRAITFX1-E: a
//! fourth page, Portraits, draws the engine's code-drawn sample face (`portrait_fx::sample`) through a `PortraitFx` at a
//! chosen size, with toggles for each overlay and state, buttons for the reactions, a local reduce motion and the
//! anchors check, on its own clock.

use crate::canvas::{rgb, rgba, Image, Rect, Rgba};
use crate::fx::{Beam, Camera, Clock, Effect, Flash, Fx, Layer, Look, Projectile, Ring, ShakeLimits, ShakeTable, Sparks, Trail, PRIM_KINDS, SHAKE_PRESETS};
use crate::portrait_fx::{self, Budget, Cue, PortraitFx, PortraitOpts, Tier};
use aegis_core::assets::Face;
use std::cell::RefCell;
use std::collections::BTreeSet;
use crate::timeline::{Link, Mark, Part, PartStyle, Section, Segment, Timeline, Track};
use crate::ui::{tip, Glyph, Ui, BODY, DIM, FOE, GOLD, GOOD, INFO, SMALL, TEXT};

#[derive(Debug, Clone, PartialEq)]
pub enum KitAct {
    Press(&'static str),
    Tab(usize),
    Check,
    Step(i32),
    Pick(usize),
    Slide(usize),
    Card(usize),
    Row(usize),
    Close,
    /// The lock icon button (toggles), any other icon button (its name).
    Lock,
    Icon(&'static str),
    /// The collapsible section header.
    Fold,
    /// The page tabs at the foot.
    Page(usize),
    /// The timeline specimen: a block, a track, a segment (track, segment).
    Block(i32),
    Track(String),
    Segment(String, String),
    /// The Effects page: a primitive (`fx::PRIM_KINDS`), a shake preset (`fx::SHAKE_PRESETS`), the clock speed
    /// (`FX_SPEEDS` index), the hit-stop toggle, the page's own reduce-motion check.
    Effect(&'static str),
    Shake(&'static str),
    FxSpeed(usize),
    HitStop,
    Reduce,
    /// The Portraits page: an overlay or state toggle (`PORTRAIT_OVERLAYS`, `PORTRAIT_STATES`), a reaction
    /// (`PORTRAIT_REACTIONS`), the size (`PORTRAIT_SIZES` index), its reduce-motion and anchors checks.
    PortraitToggle(&'static str),
    PortraitFire(&'static str),
    PortraitSize(usize),
    PortraitReduce,
    PortraitAnchors,
}

pub const TABS: [&str; 3] = ["First", "Second", "Third"];
pub const PAGES: [&str; 4] = ["Widgets", "Timeline", "Effects", "Portraits"];
/// The Portraits page: the sizes on offer (logical px), the toggles, the reactions and the stage (the largest size
/// plus the jolt ceiling and a margin on every side).
pub const PORTRAIT_SIZES: [i32; 6] = [96, 64, 48, 32, 24, 16];
pub const PORTRAIT_OVERLAYS: [&str; 5] = ["glow", "veins", "eye_glow", "ash", "smoke"];
pub const PORTRAIT_STATES: [&str; 4] = ["desaturate", "tint", "fade", "still"];
pub const PORTRAIT_REACTIONS: [&str; 3] = ["flash", "jolt.hit", "jolt.crit"];
pub const PORTRAIT_STAGE: i32 = 96 + 2 * 6;
/// The Effects page: its stage (dark, the dummies 160 px apart), the clock speeds and the hit-stop.
pub const FX_STAGE_W: i32 = 240;
pub const FX_STAGE_H: i32 = 120;
pub const FX_SPEEDS: [(&str, f64); 3] = [("0.5x", 0.5), ("1x", 1.0), ("2x", 2.0)];
pub const FX_HIT_STOP_MS: f64 = 40.0;
/// The dummies' chest points and their foot line, in stage px.
pub const FX_FROM: (f32, f32) = (40.0, 82.0);
pub const FX_TO: (f32, f32) = (200.0, 82.0);
pub const FX_FOOT_Y: f32 = 96.0;
/// Palette hues of the samples (steel, linen, ash, amber, gold, orchid) and the stage.
const STEEL: Rgba = rgb(156, 163, 175);
const LINEN: Rgba = rgb(217, 203, 180);
const ASH: Rgba = rgb(107, 114, 128);
const AMBER: Rgba = rgb(217, 119, 6);
const CANDLE: Rgba = rgb(251, 191, 36);
const ORCHID: Rgba = rgb(192, 132, 252);
const STAGE_BG: Rgba = rgb(14, 15, 20);
pub const OPTIONS: [&str; 4] = ["Small", "Normal", "Large", "Extra large"];
pub const SLIDER_STEPS: usize = 5;

/// The page's sample values; what the last widget used did is shown at the foot of the page.
#[derive(Debug, Clone)]
pub struct Specimens {
    pub tab: usize,
    pub checked: bool,
    pub value: i32,
    pub pick: usize,
    pub slide: usize,
    pub card: usize,
    pub row: usize,
    pub text: String,
    /// The text area's sample (the kit's text editor, PLAYUI1-T).
    pub area: String,
    /// The icon buttons' lock (on: the delete icon is disabled) and whether the section is open.
    pub locked: bool,
    pub open: bool,
    /// The drag and drop specimen: the chips in its two boxes.
    pub left: Vec<String>,
    pub right: Vec<String>,
    pub last: String,
    /// The page on view (`PAGES`), and the timeline specimen's selected block and track.
    pub page: usize,
    pub block: Option<i32>,
    pub track: Option<String>,
    /// The Effects page's state.
    pub fx: FxDemo,
    /// The Portraits page's state.
    pub portrait: PortraitDemo,
}

/// The Portraits page's clock, portrait (its cache behind a `RefCell`: the page draws through `&self`), sample face
/// and choices.
#[derive(Debug, Clone)]
pub struct PortraitDemo {
    pub clock: Clock,
    pub fx: RefCell<PortraitFx>,
    budget: RefCell<Budget>,
    /// The overlays and states toggled on.
    pub on: BTreeSet<&'static str>,
    /// `PORTRAIT_SIZES` index.
    pub size: usize,
    pub reduce: bool,
    pub anchors: bool,
    pub image: Image,
    pub face: Face,
    last_wall: Option<std::time::Instant>,
}

impl Default for PortraitDemo {
    fn default() -> Self {
        let (image, face) = portrait_fx::sample();
        Self { clock: Clock::default(), fx: RefCell::new(PortraitFx::new("kit.sample")), budget: RefCell::new(Budget::default()), on: BTreeSet::new(), size: 0, reduce: false, anchors: false, image, face, last_wall: None }
    }
}

impl PortraitDemo {
    /// Toggles overlay or state `e` and holds the new set; what it did.
    fn toggle(&mut self, e: &'static str) -> String {
        let on = !self.on.remove(e);
        if on {
            self.on.insert(e);
        }
        let cues: Vec<Cue> = self.on.iter().filter_map(|x| Cue::parse(x, "").map(|c| c.with_key(&format!("kit.{x}")))).collect();
        self.fx.borrow_mut().hold(&cues, self.clock.now());
        format!("portrait {} {}", e, if on { "on" } else { "off" })
    }

    /// Fires reaction `what` (`flash`, `jolt.hit`, `jolt.crit`); what it did.
    fn fire(&mut self, what: &str) -> String {
        let cue = match what {
            "flash" => Cue::parse("flash", ""),
            _ => Cue::parse("jolt", what.trim_start_matches("jolt.")),
        };
        let Some(cue) = cue else { return format!("no reaction {what}") };
        self.fx.borrow_mut().fire(cue, self.clock.now());
        let still = self.reduce && what.starts_with("jolt");
        format!("portrait {what}{}", if still { ": no motion (reduce motion)" } else { "" })
    }

    fn opts(&self) -> PortraitOpts {
        PortraitOpts { reduce_motion: self.reduce, rim_default: None, show_anchors: self.anchors }
    }

    /// The portrait's agent state at the page's clock (`PortraitFx::state`).
    pub fn state(&self) -> serde_json::Value {
        self.fx.borrow().state(self.clock.now())
    }
}

/// The Effects page's clock, live list, camera and choices; impacts to come fire their sparks, flash and hit-stop
/// when the clock reaches them.
#[derive(Debug, Clone)]
pub struct FxDemo {
    pub clock: Clock,
    pub fx: Fx,
    pub camera: Camera,
    pub shakes: ShakeTable,
    /// `FX_SPEEDS` index.
    pub speed: usize,
    pub hit_stop: bool,
    pub reduce: bool,
    /// (clock ms, seed) of the impacts to come.
    pub impacts: Vec<(u64, u64)>,
    /// Seeds handed out so far (each press gets the next).
    pub seeds: u64,
    last_wall: Option<std::time::Instant>,
}

impl Default for FxDemo {
    fn default() -> Self {
        Self { clock: Clock::default(), fx: Fx::default(), camera: Camera::default(), shakes: ShakeTable::default(), speed: 1, hit_stop: true, reduce: false, impacts: Vec::new(), seeds: 0, last_wall: None }
    }
}

impl FxDemo {
    fn seed(&mut self) -> u64 {
        self.seeds += 1;
        self.seeds
    }

    /// Plays primitive `kind` from the left dummy to the right one; what it did.
    fn play(&mut self, kind: &str) -> String {
        let now = self.clock.now();
        let seed = self.seed();
        let from = (FX_FROM.0 + 6.0, FX_FROM.1);
        let (e, impact): (Effect, Option<u64>) = match kind {
            "projectile" => {
                let travel = (((FX_TO.0 - from.0) / 560.0) * 1000.0).round() as u32;
                (Effect::new(Projectile { from, to: FX_TO, travel_ms: travel, arc_px: 6.0, look: Look::Arrow { len: 7.0, shaft: LINEN, head: STEEL } }, now), Some(now + travel as u64))
            }
            "beam" => (Effect::new(Beam { from, to: FX_TO, ms: 160, width: 2, core: LINEN, glow: ORCHID }, now), Some(now)),
            "trail" => (Effect::new(Trail { at: (FX_TO.0 - 14.0, FX_TO.1), radius: 13.0, from_deg: -60.0, to_deg: 50.0, ms: 150, width: 2, col: STEEL, mirror: false, squash: 0.75 }, now), Some(now + 90)),
            "sparks" => (Effect::new(Sparks { at: FX_TO, ms: 260, count: 10, dir_deg: -90.0, spread_deg: 160.0, speed: 70.0, gravity: 120.0, size: 1, cols: [AMBER, CANDLE] }, now), None),
            "flash" => (Effect::new(Flash { at: FX_TO, rx: 8.0, ry: 6.0, ms: 140, col: rgba(CANDLE[0], CANDLE[1], CANDLE[2], 160) }, now), None),
            _ => (Effect::new(Ring { at: (FX_TO.0, FX_FOOT_Y), r0: 3.0, r1: 12.0, ms: 260, col: rgba(ASH[0], ASH[1], ASH[2], 200), squash: 0.75 }, now).on(Layer::Ground), None),
        };
        let ms = e.ms();
        self.fx.push(e.seeded(seed));
        if let Some(t) = impact {
            self.impacts.push((t, seed));
        }
        format!("{kind}: {ms} ms{}", if impact.is_some() { ", then the impact" } else { "" })
    }

    /// Starts preset `name` at the engine ceiling; what it did.
    fn shake(&mut self, name: &str) -> String {
        let Some(s) = self.shakes.get(name) else { return format!("no preset {name}") };
        let (now, seed) = (self.clock.now(), self.seed());
        if self.camera.start(s, now, seed, ShakeLimits::CEILING, self.reduce) {
            format!("shake {name}: {} px, {} ms, {}, {}", s.px, s.ms, s.axis.name(), s.envelope.name())
        } else if self.reduce {
            format!("shake {name}: none (reduce motion)")
        } else {
            format!("shake {name}: a stronger shake runs")
        }
    }

    /// `wall_ms` of wall time at the chosen speed: due impacts fire (their sparks and flash at the right dummy and,
    /// when on, the hit-stop), done effects and shakes end.
    pub fn advance(&mut self, wall_ms: f64) {
        self.clock.advance(wall_ms, FX_SPEEDS[self.speed.min(FX_SPEEDS.len() - 1)].1);
        let now = self.clock.now();
        let due: Vec<(u64, u64)> = self.impacts.iter().copied().filter(|(t, _)| *t <= now).collect();
        self.impacts.retain(|(t, _)| *t > now);
        for (t, seed) in due {
            self.fx.push(Effect::new(Sparks { at: FX_TO, ms: 260, count: 6, dir_deg: 0.0, spread_deg: 90.0, speed: 70.0, gravity: 120.0, size: 1, cols: [STEEL, LINEN] }, t).seeded(seed));
            self.fx.push(Effect::new(Flash { at: FX_TO, rx: 5.0, ry: 4.0, ms: 100, col: rgba(255, 255, 255, 150) }, t));
            if self.hit_stop {
                self.clock.hold(FX_HIT_STOP_MS);
            }
        }
        self.fx.prune(now);
        self.camera.offset(now);
    }
}

impl Default for Specimens {
    fn default() -> Self {
        Self {
            tab: 0,
            checked: true,
            value: 3,
            pick: 1,
            slide: 2,
            card: 0,
            row: 1,
            text: "Type here".into(),
            area: "A text area wraps its lines and scrolls. Shift with the arrows selects, so does a drag; a double click takes a word.\nCtrl with Left or Right jumps by word; Ctrl+Z undoes, Ctrl+Y redoes.".into(),
            locked: false,
            open: true,
            left: vec!["Sword".into(), "Shield".into()],
            right: vec!["Potion".into()],
            last: "nothing yet".into(),
            page: 0,
            block: None,
            track: None,
            fx: FxDemo::default(),
            portrait: PortraitDemo::default(),
        }
    }
}

impl Specimens {
    pub fn act(&mut self, a: KitAct) {
        self.last = match &a {
            KitAct::Press(what) => format!("pressed {what}"),
            KitAct::Tab(i) => format!("tab {}", TABS[*i % TABS.len()]),
            KitAct::Check => format!("checkbox {}", if self.checked { "off" } else { "on" }),
            KitAct::Step(d) => format!("stepper {}", (self.value + d).clamp(0, 9)),
            KitAct::Pick(i) => format!("select {}", OPTIONS[*i % OPTIONS.len()]),
            KitAct::Slide(i) => format!("slider step {i}"),
            KitAct::Card(i) => format!("card {}", i + 1),
            KitAct::Row(i) => format!("row {}", i + 1),
            KitAct::Close => "close".into(),
            KitAct::Lock => format!("lock {}", if self.locked { "off" } else { "on" }),
            KitAct::Icon(what) => format!("icon {what}"),
            KitAct::Fold => format!("section {}", if self.open { "folded" } else { "opened" }),
            KitAct::Page(i) => format!("page {}", PAGES[*i % PAGES.len()]),
            KitAct::Block(b) => format!("block {b}"),
            KitAct::Track(t) => format!("track {t}"),
            KitAct::Segment(t, s) => format!("segment {s} on {t}"),
            KitAct::Effect(k) => self.fx.play(k),
            KitAct::Shake(n) => self.fx.shake(n),
            KitAct::FxSpeed(i) => format!("speed {}", FX_SPEEDS[*i % FX_SPEEDS.len()].0),
            KitAct::HitStop => format!("hit-stop {}", if self.fx.hit_stop { "off" } else { "on" }),
            KitAct::Reduce => format!("reduce motion {}", if self.fx.reduce { "off" } else { "on" }),
            KitAct::PortraitToggle(e) => self.portrait.toggle(e),
            KitAct::PortraitFire(w) => self.portrait.fire(w),
            KitAct::PortraitSize(i) => format!("portrait size {} px", PORTRAIT_SIZES[*i % PORTRAIT_SIZES.len()]),
            KitAct::PortraitReduce => format!("portrait reduce motion {}", if self.portrait.reduce { "off" } else { "on" }),
            KitAct::PortraitAnchors => format!("portrait anchors {}", if self.portrait.anchors { "off" } else { "on" }),
        };
        match a {
            KitAct::Tab(i) => self.tab = i % TABS.len(),
            KitAct::Check => self.checked = !self.checked,
            KitAct::Step(d) => self.value = (self.value + d).clamp(0, 9),
            KitAct::Pick(i) => self.pick = i % OPTIONS.len(),
            KitAct::Slide(i) => self.slide = i.min(SLIDER_STEPS - 1),
            KitAct::Card(i) => self.card = i,
            KitAct::Row(i) => self.row = i,
            KitAct::Lock => self.locked = !self.locked,
            KitAct::Fold => self.open = !self.open,
            KitAct::Page(i) => self.page = i % PAGES.len(),
            KitAct::Block(b) => self.block = Some(b),
            KitAct::Track(t) => self.track = Some(t),
            KitAct::FxSpeed(i) => self.fx.speed = i % FX_SPEEDS.len(),
            KitAct::HitStop => self.fx.hit_stop = !self.fx.hit_stop,
            KitAct::Reduce => {
                self.fx.reduce = !self.fx.reduce;
                if self.fx.reduce {
                    self.fx.camera.stop();
                }
            }
            KitAct::PortraitSize(i) => self.portrait.size = i % PORTRAIT_SIZES.len(),
            KitAct::PortraitReduce => self.portrait.reduce = !self.portrait.reduce,
            KitAct::PortraitAnchors => self.portrait.anchors = !self.portrait.anchors,
            KitAct::Press(_) | KitAct::Close | KitAct::Icon(_) | KitAct::Segment(..) | KitAct::Effect(_) | KitAct::Shake(_) | KitAct::PortraitToggle(_) | KitAct::PortraitFire(_) => {}
        }
    }

    /// The Effects or Portraits page's clock moves by the wall time since the last call (at most
    /// `vignette::MAX_FRAME_MS`), while that page is on view (the gallery screen calls it every frame).
    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        let since = |last: Option<std::time::Instant>| last.map_or(0.0, |l| (now.duration_since(l).as_secs_f64() * 1000.0).min(crate::vignette::MAX_FRAME_MS as f64));
        if self.page == 2 {
            let ms = since(self.fx.last_wall);
            self.fx.last_wall = Some(now);
            self.fx.advance(ms);
        } else {
            self.fx.last_wall = None;
        }
        if self.page == 3 {
            let ms = since(self.portrait.last_wall);
            self.portrait.last_wall = Some(now);
            self.portrait.clock.advance(ms, 1.0);
        } else {
            self.portrait.last_wall = None;
        }
    }

    /// The Effects page's clock moves by `wall_ms` (`FxDemo::advance`), and the Portraits page's clock with it.
    pub fn advance(&mut self, wall_ms: f64) {
        self.fx.advance(wall_ms);
        self.portrait.clock.advance(wall_ms, 1.0);
    }

    /// A chip (`chip.<name>`) onto the other box (`box.left` / `box.right`).
    pub fn can_drop(&self, from: &str, to: &str) -> bool {
        let Some(chip) = from.strip_prefix("chip.") else { return false };
        match to {
            "box.left" => self.right.iter().any(|c| c == chip),
            "box.right" => self.left.iter().any(|c| c == chip),
            _ => false,
        }
    }

    pub fn accept_drop(&mut self, from: &str, to: &str) {
        if !self.can_drop(from, to) {
            return;
        }
        let chip = from.trim_start_matches("chip.").to_string();
        let (src, dst) = if to == "box.left" { (&mut self.right, &mut self.left) } else { (&mut self.left, &mut self.right) };
        src.retain(|c| *c != chip);
        dst.push(chip.clone());
        self.last = format!("dropped {chip} on the {} box", to.trim_start_matches("box."));
    }

    pub fn text_changed(&mut self, id: &str, text: &str) {
        match id {
            "kit.text" => self.text = text.to_string(),
            "kit.area" => self.area = text.to_string(),
            _ => {}
        }
    }

    /// Draws the page on view in `r`, the page tabs (`kit.page.<i>`) and what happened last at its foot. Widget ids
    /// start with `kit.`.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: impl Fn(KitAct) -> A) {
        let mut tx = r.x + r.w - 6;
        for (i, p) in PAGES.iter().enumerate().rev() {
            let w = (ui.p.text_w(p, BODY) + 12.0).ceil() as i32;
            tx -= w;
            ui.tab(&format!("kit.page.{i}"), Rect::new(tx, r.y + r.h - 15, w, 14), p, wrap(KitAct::Page(i)), self.page == i);
            tx -= 2;
        }
        if self.page == 1 {
            self.draw_timeline(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else if self.page == 2 {
            self.draw_effects(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else if self.page == 3 {
            self.draw_portraits(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else {
            self.draw_widgets(ui, r, &wrap);
        }
        ui.p.text_fit((r.x + 6) as f32, (r.y + r.h - 12) as f32, &format!("Last: {}", self.last), SMALL, (tx - r.x - 10) as f32, GOOD);
    }

    /// The timeline page: a sample round on two sides, every part, a whole-hatched segment, links, marks (one on a
    /// masked track), then a legend.
    fn draw_timeline<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let x = r.x + 6;
        let w = r.w - 12;
        ui.p.text(x as f32, (r.y + 4) as f32, "Timeline: segment parts, hatching, links and marks (hover for the cards; clicks select)", SMALL, INFO);
        let y = r.y + 4 + ui.p.line_h(SMALL).ceil() as i32 + 2;
        let t = sample_timeline(self.block, self.track.clone());
        let h = (crate::timeline::RULER_H + 2 + t.content_h()).min(r.y + r.h - y - 30);
        t.draw(ui, "kit.timeline", Rect::new(x, y, w, h), |b| wrap(KitAct::Block(b)), |k| wrap(KitAct::Track(k.to_string())), |k, s| wrap(KitAct::Segment(k.to_string(), s.to_string())));
        ui.para(
            x,
            y + h + 6,
            w,
            "A windup is shaded, an impact bright with an edge where it lands, a recovery hatched; a segment can be hatched whole. A link joins one segment's end to the next one's start. A mark is a glyph in a colour with a card, inside its block; it takes no click, the text beside it makes room, and it shows on a masked track too.",
            SMALL,
            DIM,
        );
    }

    /// The Effects page: the stage (`kit.fx.stage`) with its two dummies and the live effects, shaken by the camera
    /// (the panel and the controls stay put); beside it a button per primitive (`kit.fx.<kind>`) and per shake preset
    /// (`kit.fx.shake.<name>`), the speed (`kit.fx.speed`), the hit-stop (`kit.fx.hitstop`) and reduce motion
    /// (`kit.fx.reduce`); under it the clock's state.
    fn draw_effects<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let x = r.x + 6;
        let line = ui.p.line_h(SMALL).ceil() as i32;
        ui.p.text(x as f32, (r.y + 4) as f32, "Effects: drawn primitives on the effect clock and camera shakes by preset", SMALL, INFO);
        let y = r.y + 4 + line + 2;
        let stage = Rect::new(x, y, FX_STAGE_W, FX_STAGE_H);
        let d = &self.fx;
        let now = d.clock.now();
        ui.p.panel(Rect::new(stage.x - 1, stage.y - 1, stage.w + 2, stage.h + 2), STAGE_BG, crate::ui::BORDER, 2.0);
        let (dx, dy) = d.camera.offset_at(now);
        ui.push("kit.fx.stage", "stage", format!("effects stage: {} live, shake ({dx}, {dy})", d.fx.kinds(now).len()), stage, true, false, None, Vec::new());
        let saved = ui.p.c.clip;
        ui.p.clip(Some(stage));
        ui.p.shift(dx, dy);
        let (ox, oy) = (stage.x as f32, stage.y as f32);
        ui.p.fill(Rect::new(stage.x - 8, stage.y + FX_FOOT_Y as i32 + 1, FX_STAGE_W + 16, 1), rgb(44, 48, 60));
        let placed = |layer: Layer| Fx { live: d.fx.live.iter().filter(|e| e.layer == layer).map(|e| e.moved(ox, oy)).collect() };
        placed(Layer::Ground).draw(&mut ui.p, now, Layer::Ground);
        for (cx, _) in [FX_FROM, FX_TO] {
            let (fx_, fy) = (ox + cx, oy + FX_FOOT_Y);
            ui.p.ellipse(fx_, fy, 8.0, 2.5, rgba(0, 0, 0, 120));
            ui.p.fill(Rect::new(fx_ as i32 - 4, fy as i32 - 22, 8, 22), rgb(70, 78, 96));
            ui.p.fill(Rect::new(fx_ as i32 - 3, fy as i32 - 29, 6, 6), rgb(96, 104, 122));
        }
        placed(Layer::Air).draw(&mut ui.p, now, Layer::Air);
        ui.p.shift(-dx, -dy);
        ui.p.c.clip = saved;
        let state = format!("Clock {} ms, hold {} ms, {} live, impacts to come: {}", now, d.clock.hold.ceil(), d.fx.len(), d.impacts.len());
        ui.p.text_fit(x as f32, (stage.y + stage.h + 4) as f32, &state, SMALL, FX_STAGE_W as f32, DIM);
        ui.para(
            x,
            stage.y + stage.h + 6 + line,
            FX_STAGE_W,
            "Every piece is a pure function of the clock and a seed. A hit-stop holds the clock; the speed scales it. Shakes move the stage only, in whole pixels, never past the engine ceiling (6 px, 3 s); reduce motion stops them and keeps the effects.",
            SMALL,
            TEXT,
        );

        // -- Controls beside the stage
        let x2 = stage.x + stage.w + 10;
        let right = r.x + r.w - 6;
        let bw = |ui: &mut Ui<A>, t: &str| (ui.p.text_w(t, BODY) + 12.0).ceil() as i32;
        let head = |ui: &mut Ui<A>, y: i32, t: &str| -> i32 {
            ui.p.text(x2 as f32, y as f32, t, SMALL, INFO);
            y + line + 1
        };
        let mut yy = head(ui, y, "Primitives (left dummy to right)");
        let mut bx = x2;
        for kind in PRIM_KINDS {
            let label = cap(kind);
            let w = bw(ui, &label);
            if bx + w > right {
                bx = x2;
                yy += 16;
            }
            ui.button(&format!("kit.fx.{kind}"), Rect::new(bx, yy, w, 14), &label, wrap(KitAct::Effect(kind)), true);
            bx += w + 3;
        }
        yy += 14 + 5;
        yy = head(ui, yy, "Camera shakes (engine presets)");
        let mut bx = x2;
        for name in SHAKE_PRESETS {
            let w = bw(ui, name);
            if bx + w > right {
                bx = x2;
                yy += 16;
            }
            ui.button(&format!("kit.fx.shake.{name}"), Rect::new(bx, yy, w, 14), name, wrap(KitAct::Shake(name)), true);
            bx += w + 3;
        }
        yy += 14 + 5;
        yy = head(ui, yy, "Clock speed, hit-stop, motion");
        let speeds: Vec<String> = FX_SPEEDS.iter().map(|s| s.0.to_string()).collect();
        ui.select("kit.fx.speed", Rect::new(x2, yy, 56, 14), &speeds, d.speed, |i| wrap(KitAct::FxSpeed(i)), true, tip("Speed", "Scales the effect clock (hit-stops stay wall time)."));
        ui.checkbox("kit.fx.hitstop", x2 + 64, yy + 2, &format!("Hit-stop {} ms", FX_HIT_STOP_MS), d.hit_stop, wrap(KitAct::HitStop), true);
        yy += 14 + 5;
        ui.checkbox("kit.fx.reduce", x2, yy + 2, "Reduce motion (this page)", d.reduce, wrap(KitAct::Reduce), true);
    }

    /// The Portraits page: the stage (`kit.portrait.face`: the portrait and its jolt room) with the sample face at the
    /// chosen size and, under it, the tier and what is live; beside it a toggle per overlay and state
    /// (`kit.portrait.<effect>`), a button per reaction (`kit.portrait.flash`, `kit.portrait.jolt.hit`,
    /// `kit.portrait.jolt.crit`), the size (`kit.portrait.size`), reduce motion (`kit.portrait.reduce`) and the
    /// anchors check (`kit.portrait.anchors`).
    fn draw_portraits<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let d = &self.portrait;
        let x = r.x + 6;
        let line = ui.p.line_h(SMALL).ceil() as i32;
        ui.p.text(x as f32, (r.y + 4) as f32, "Portraits: base life, overlays, states and reactions over a code-drawn face", SMALL, INFO);
        let y = r.y + 4 + line + 2;
        let stage = Rect::new(x, y, PORTRAIT_STAGE, PORTRAIT_STAGE);
        ui.p.panel(stage, STAGE_BG, crate::ui::BORDER, 2.0);
        let size = PORTRAIT_SIZES[d.size.min(PORTRAIT_SIZES.len() - 1)];
        let face = Rect::new(stage.x + (stage.w - size) / 2, stage.y + (stage.h - size) / 2, size, size);
        let now = d.clock.now();
        {
            let mut budget = d.budget.borrow_mut();
            budget.begin();
            d.fx.borrow_mut().draw_image(&mut ui.p, &d.image, "kit.sample", Some(&d.face), face, now, &d.opts(), &mut budget);
        }
        let (tier, live) = {
            let fx = d.fx.borrow();
            (fx.tier().map_or("", Tier::name), fx.live(now))
        };
        let device = (size as f32 * ui.p.v.s).round() as i32;
        ui.push("kit.portrait.face", "portrait", format!("sample portrait: {size} px, tier {tier}"), stage, true, false, None, Vec::new());
        let col_w = 200;
        let foot = format!("Tier {tier} ({device} device px). Live: {}", if live.is_empty() { "nothing".to_string() } else { live.join(", ") });
        let fy = ui.para(x, stage.y + stage.h + 4, col_w, &foot, SMALL, DIM);
        ui.para(
            x,
            fy + 4,
            col_w,
            "Pure functions of the clock and a seed from the key, composed at the drawn size and cached. From 96 device px everything draws; 48-95 drops breath, veins and smoke; below 48 only flashes, colour states, fade and the eye glow's core. Reduce motion stops breath, jolts, flicker and drift.",
            SMALL,
            TEXT,
        );

        // -- Controls beside the stage
        let x2 = x + col_w + 10;
        let right = r.x + r.w - 6;
        let head = |ui: &mut Ui<A>, y: i32, t: &str| -> i32 {
            ui.p.text(x2 as f32, y as f32, t, SMALL, INFO);
            y + line + 1
        };
        let mut yy = y;
        for (title, names) in [("Overlays (held at full strength)", &PORTRAIT_OVERLAYS[..]), ("States", &PORTRAIT_STATES[..])] {
            yy = head(ui, yy, title);
            let mut bx = x2;
            for name in names.iter().copied() {
                let label = cap(&name.replace('_', " "));
                let w = 12 + ui.p.text_w(&label, BODY).ceil() as i32 + 4;
                if bx + w > right {
                    bx = x2;
                    yy += 16;
                }
                ui.checkbox(&format!("kit.portrait.{name}"), bx, yy + 2, &label, d.on.contains(name), wrap(KitAct::PortraitToggle(name)), true);
                bx += w + 8;
            }
            yy += 14 + 5;
        }
        yy = head(ui, yy, "Reactions");
        let mut bx = x2;
        for (what, label) in PORTRAIT_REACTIONS.iter().zip(["Flash", "Hit jolt", "Crit jolt"]) {
            let w = (ui.p.text_w(label, BODY) + 12.0).ceil() as i32;
            ui.button(&format!("kit.portrait.{what}"), Rect::new(bx, yy, w, 14), label, wrap(KitAct::PortraitFire(what)), true);
            bx += w + 3;
        }
        yy += 14 + 5;
        yy = head(ui, yy, "Size, motion, anchors");
        let sizes: Vec<String> = PORTRAIT_SIZES.iter().map(|s| format!("{s} px")).collect();
        ui.select("kit.portrait.size", Rect::new(x2, yy, 56, 14), &sizes, d.size, |i| wrap(KitAct::PortraitSize(i)), true, tip("Size", "The portrait's side in logical px; the tier follows the device px."));
        ui.checkbox("kit.portrait.reduce", x2 + 64, yy + 2, "Reduce motion (this page)", d.reduce, wrap(KitAct::PortraitReduce), true);
        yy += 14 + 5;
        ui.checkbox("kit.portrait.anchors", x2, yy + 2, "Anchors (eye boxes, glow, vein roots)", d.anchors, wrap(KitAct::PortraitAnchors), true);
    }

    /// The widgets page: two columns of named groups.
    fn draw_widgets<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let col_w = (r.w - 18) / 2;
        let (x1, x2) = (r.x + 6, r.x + 12 + col_w);
        let gap = 5;
        let head = |ui: &mut Ui<A>, x: i32, y: i32, t: &str| -> i32 {
            ui.p.text(x as f32, y as f32, t, SMALL, INFO);
            y + ui.p.line_h(SMALL).ceil() as i32 + 1
        };
        let bw = |ui: &mut Ui<A>, t: &str| (ui.p.text_w(t, BODY) + 12.0).ceil() as i32;

        // -- Left column
        let mut y = r.y + 4;
        y = head(ui, x1, y, "Buttons: normal, selected, disabled, with a hover card");
        let mut x = x1;
        for (id, label, active, enabled) in [("kit.button", "Button", false, true), ("kit.button.active", "Selected", true, true), ("kit.button.off", "Disabled", false, false)] {
            let w = bw(ui, label);
            ui.button_full(id, Rect::new(x, y, w, 14), label, wrap(KitAct::Press(label_key(id))), enabled, active, BODY, Vec::new());
            x += w + 4;
        }
        let w = bw(ui, "Hover me");
        ui.button_tip("kit.button.tip", Rect::new(x, y, w, 14), "Hover me", wrap(KitAct::Press("hover card")), true, tip("Hover card", "Every widget can carry one: a title line and its text, wrapped."));
        y += 14 + gap;
        y = head(ui, x1, y, "Tabs");
        let mut x = x1;
        for (i, t) in TABS.iter().enumerate() {
            let w = bw(ui, t);
            ui.tab(&format!("kit.tab.{i}"), Rect::new(x, y, w, 14), t, wrap(KitAct::Tab(i)), self.tab == i);
            x += w + 2;
        }
        y += 14 + gap;
        y = head(ui, x1, y, "Checkbox and stepper");
        ui.checkbox("kit.check", x1, y + 2, if self.checked { "Checked" } else { "Not checked" }, self.checked, wrap(KitAct::Check), true);
        ui.stepper("kit.stepper", Rect::new(x1 + 80, y, 130, 14), self.value as f32 / 9.0, &self.value.to_string(), wrap(KitAct::Step(-1)), wrap(KitAct::Step(1)), true);
        y += 14 + gap;
        y = head(ui, x1, y, "Dropdown (a menu) and select (a choice list)");
        let half = (col_w - 4) / 2;
        ui.dropdown("kit.dropdown", Rect::new(x1, y, half, 14), "Dropdown", wrap(KitAct::Press("dropdown")), true, tip("Dropdown", "Opens a menu of its own (the screen draws it)."));
        let options: Vec<String> = OPTIONS.iter().map(|o| o.to_string()).collect();
        ui.select("kit.select", Rect::new(x1 + half + 4, y, half, 14), &options, self.pick, |i| wrap(KitAct::Pick(i)), true, tip("Select", "Opens its choice list; arrows and Enter work too."));
        y += 14 + gap;
        y = head(ui, x1, y, "Slider");
        ui.slider("kit.slider", Rect::new(x1, y, col_w.min(160), 14), self.slide, SLIDER_STEPS, &format!("Step {} of {}", self.slide + 1, SLIDER_STEPS), |i| wrap(KitAct::Slide(i)), true);
        y += 14 + gap;
        y = head(ui, x1, y, "Text field (click to type)");
        ui.text_field("kit.text", Rect::new(x1, y, col_w.min(180), 14), &self.text, false, true);
        y += 14 + gap;
        y = head(ui, x1, y, "Text area: Up / Down, Shift selects, Ctrl+arrows by word, Ctrl+A, C, X, V, Z, Y");
        let ah = (ui.p.line_h(BODY) * 4.0).ceil() as i32 + 6;
        ui.text_field("kit.area", Rect::new(x1, y, col_w, ah), &self.area, true, true);
        y += ah + gap;
        y = head(ui, x1, y, "Frame, paragraph and close button");
        let fr = Rect::new(x1, y, col_w.min(200), (r.y + r.h - 18 - y).clamp(46, 70));
        let body_y = ui.frame(fr, "Frame", false);
        ui.close_x("kit.close", Rect::new(fr.x + fr.w - 16, fr.y + 4, 12, 12), wrap(KitAct::Close));
        ui.para(fr.x + 6, body_y, fr.w - 12, "A paragraph wraps to its width and returns where it ends, so what follows sits under it.", SMALL, TEXT);

        // -- Right column
        let mut y = r.y + 4;
        y = head(ui, x2, y, "Cards (click to select)");
        let cw = (col_w - 4) / 2;
        for i in 0..2 {
            let lines: Vec<(String, Rgba)> = vec![(format!("Card {}", i + 1), GOLD), ("Two lines of text".into(), TEXT)];
            ui.card(&format!("kit.card.{i}"), Rect::new(x2 + i as i32 * (cw + 4), y, cw, 26), &lines, Some(wrap(KitAct::Card(i))), self.card == i, true, Vec::new());
        }
        y += 26 + gap;
        y = head(ui, x2, y, "Rows: clickable, selected, info (hover card only)");
        for i in 0..2 {
            ui.row(&format!("kit.row.{i}"), Rect::new(x2, y, col_w, 13), &format!("Row {}", i + 1), if self.row == i { GOLD } else { TEXT }, Some(wrap(KitAct::Row(i))), self.row == i, Vec::new());
            y += 14;
        }
        ui.row("kit.row.info", Rect::new(x2, y, col_w, 13), "Info row", DIM, None, false, tip("Info row", "No action: hovering shows its card and an outline."));
        y += 13 + gap;
        y = head(ui, x2, y, "Drag and drop (drag a chip to the other box)");
        let bh = 34;
        for (side, chips, bx) in [("left", &self.left, x2), ("right", &self.right, x2 + cw + 4)] {
            let b = Rect::new(bx, y, cw, bh);
            ui.p.panel(b, crate::ui::CARD, crate::ui::BORDER, 2.0);
            ui.push(format!("kit.box.{side}"), "drop", format!("{side} box"), b, true, false, None, Vec::new());
            ui.drop_target(format!("box.{side}"));
            let mut cx = b.x + 3;
            for c in chips {
                let w = bw(ui, c);
                let chip = Rect::new(cx, b.y + 3, w, 12);
                ui.button_full(&format!("kit.chip.{c}"), chip, c, wrap(KitAct::Press("chip")), true, false, SMALL, Vec::new());
                ui.draggable(format!("chip.{c}"));
                cx += w + 3;
            }
        }
        y += bh + gap;
        y = head(ui, x2, y, "Icon buttons: image, drawn shapes (file missing), toggled, disabled");
        let ib = |i: i32| Rect::new(x2 + i * 19, y, 16, 16);
        let px = ui.icon_px(ib(0));
        let sample = sample_icon(px);
        ui.icon_button("kit.icon.image", ib(0), "Image icon", Some(&sample), Glyph::Letter('i'), wrap(KitAct::Icon("image")), true, false, tip("Image icon", "An image from the asset layers; a drawn shape stands in while the file is missing."));
        let (lock, lock_tip) = if self.locked { ("Unlock", "Unlock") } else { ("Lock", "Lock (protect from overwrite and delete)") };
        ui.icon_button("kit.icon.lock", ib(1), lock, None, if self.locked { Glyph::Lock } else { Glyph::Unlock }, wrap(KitAct::Lock), true, self.locked, tip(lock_tip, ""));
        let del_tip = if self.locked { tip("Delete", "Unlock it to delete it.") } else { Vec::new() };
        ui.icon_button("kit.icon.delete", ib(2), "Delete", None, Glyph::Delete, wrap(KitAct::Icon("delete")), !self.locked, false, del_tip);
        ui.icon_button("kit.icon.help", ib(3), "Help", None, Glyph::Help, wrap(KitAct::Icon("help")), true, false, tip("Help (?)", "A hover card can list the keys."));
        ui.icon_button("kit.icon.menu", ib(4), "Menu", None, Glyph::Menu, wrap(KitAct::Icon("menu")), true, false, Vec::new());
        ui.icon_button("kit.icon.letter", ib(5), "Party", None, Glyph::Letter('P'), wrap(KitAct::Icon("party")), true, false, tip("Party (P)", "A letter stands in for an icon with no shape of its own."));
        ui.icon_button("kit.icon.off", ib(6), "Disabled", Some(&sample), Glyph::Letter('i'), wrap(KitAct::Icon("disabled")), false, false, tip("Disabled", "Dimmed; takes no click."));
        y += 16 + gap;
        y = head(ui, x2, y, "Section header (click to fold)");
        let open = ui.section("kit.section", Rect::new(x2, y, col_w, 14), "Section", "2 rows", self.open, wrap(KitAct::Fold), tip("Section", "A click folds or opens it; the screen keeps the state."));
        y += 14 + 1;
        if open {
            for i in 0..2 {
                ui.row(&format!("kit.section.row.{i}"), Rect::new(x2 + 8, y, col_w - 8, 12), &format!("Content row {}", i + 1), TEXT, None, false, Vec::new());
                y += 13;
            }
        }
        y += gap - 1;
        y = head(ui, x2, y, "Scroll area (the wheel over it)");
        let view = Rect::new(x2, y, col_w, (r.y + r.h - 18 - y).max(24));
        ui.p.panel(view, crate::ui::CARD, crate::ui::BORDER, 2.0);
        let line = ui.p.line_h(SMALL).ceil() as i32 + 1;
        let off = ui.scroll_begin("kit.scroll", view, 12 * line + 4);
        for i in 0..12 {
            ui.p.text((view.x + 4) as f32, (view.y + 2 - off + i * line) as f32, &format!("Line {} of a list taller than its area", i + 1), SMALL, TEXT);
        }
        ui.scroll_end();
    }
}

/// The timeline specimen's round: 8 blocks, three tracks on one side and two on the other (one masked).
fn sample_timeline(block: Option<i32>, track: Option<String>) -> Timeline {
    let part = |start, end, style| Part { start, end, style };
    let seg = |key: &str, start, end, label: &str, fill: Rgba, parts: Vec<Part>, card: &str| Segment { key: key.into(), start, end, label: label.into(), fill, parts, tip: tip(label, card), ..Segment::default() };
    let mark = |key: &str, block, glyph: &str, colour, title: &str, body: &str| Mark { key: key.into(), block, glyph: glyph.into(), colour, tip: tip(title, body) };
    let link = |from: &str, to: &str, colour, body: &str| Link { from: from.into(), to: to.into(), colour, tip: tip("Link", body) };
    let (blue, dim) = (rgb(70, 110, 170), rgb(44, 56, 78));
    let red = rgb(160, 70, 64);
    let lead = Track {
        key: "lead".into(),
        label: "Lead".into(),
        colour: TEXT,
        segments: vec![
            seg("strike", 1, 3, "Strike", blue, vec![part(1, 2, PartStyle::Windup), part(3, 3, PartStyle::Impact)], "Windup B1-B2 (shaded), impact B3 (bright, edged)."),
            seg("follow", 4, 5, "Follow-up", blue, vec![part(4, 4, PartStyle::Windup), part(5, 5, PartStyle::Impact)], "Linked to the strike before it."),
            seg("rest", 6, 8, "Rest", dim, Vec::new(), "A plain segment: no parts."),
        ],
        links: vec![link("strike", "follow", GOLD, "Chained: the follow-up starts as the strike lands.")],
        marks: vec![mark("gain", 8, "+", GOOD, "Mark", "A glyph in a colour at one block, with its card.")],
        ..Track::default()
    };
    let scout = Track {
        key: "scout".into(),
        label: "Scout".into(),
        colour: TEXT,
        segments: vec![
            Segment { dashed: true, ..seg("move", 1, 2, "Move", rgb(60, 80, 110), Vec::new(), "Dashed: a planned move.") },
            seg("shot", 3, 6, "Shot", blue, vec![part(3, 5, PartStyle::Windup), part(6, 6, PartStyle::Impact)], "A long windup, impact at B6."),
        ],
        links: vec![link("move", "shot", INFO, "Carried: the shot follows the move.")],
        marks: vec![mark("step", 2, ">", INFO, "Step", "A mark on the move's last block.")],
        ..Track::default()
    };
    let warden = Track {
        key: "warden".into(),
        label: "Warden".into(),
        colour: TEXT,
        segments: vec![
            seg("hold", 1, 4, "Hold", rgb(60, 120, 100), Vec::new(), "A plain segment."),
            Segment { hatched: true, ..seg("dropped", 5, 8, "Dropped", rgb(60, 120, 100), Vec::new(), "Hatched whole: a span that will not happen.") },
        ],
        marks: vec![mark("break", 5, "x", FOE, "Broken", "A mark at a segment's first block: its label starts after it."), mark("note", 5, "i", INFO, "Note", "Two marks in one block sit side by side.")],
        ..Track::default()
    };
    let brute = Track {
        key: "brute".into(),
        label: "Brute".into(),
        colour: FOE,
        segments: vec![seg("smash", 1, 7, "Heavy smash", red, vec![part(1, 3, PartStyle::Windup), part(4, 4, PartStyle::Impact), part(5, 7, PartStyle::Recovery)], "Windup B1-B3, impact B4, recovery B5-B7 (hatched).")],
        marks: vec![mark("impact", 4, "!", FOE, "Impact", "A badge where something lands.")],
        ..Track::default()
    };
    let lurker = Track {
        key: "lurker".into(),
        label: "Lurker".into(),
        colour: FOE,
        segments: vec![seg("hidden", 1, 8, "Hidden", red, Vec::new(), "")],
        masked: Some("Plans in secret".into()),
        marks: vec![mark("impact", 4, "!", FOE, "Impact", "Marks show on a masked track: something lands here, the rest is hidden."), mark("guess", 7, "?", DIM, "Unknown", "")],
        ..Track::default()
    };
    Timeline {
        blocks: 8,
        playhead: 0,
        selected_block: block,
        selected_track: track,
        sections: vec![Section { title: "Allies".into(), colour: INFO, tracks: vec![lead, scout, warden] }, Section { title: "Enemies".into(), colour: FOE, tracks: vec![brute, lurker] }],
    }
}

/// The image icon specimen: a gold diamond with a dark outline, `n` pixels square (an icon as the art would give it).
fn sample_icon(n: i32) -> Image {
    let n = n.max(4);
    let mut px = vec![0u8; (n * n * 4) as usize];
    let c = (n - 1) as f32 / 2.0;
    for y in 0..n {
        for x in 0..n {
            let d = (x as f32 - c).abs() + (y as f32 - c).abs();
            let col = if d <= c * 0.7 { [244, 198, 84, 255] } else if d <= c * 0.95 { [70, 52, 20, 255] } else { [0, 0, 0, 0] };
            px[((y * n + x) * 4) as usize..((y * n + x) * 4 + 4) as usize].copy_from_slice(&col);
        }
    }
    Image { w: n, h: n, px }
}

/// `kind` with its first letter in upper case.
fn cap(kind: &str) -> String {
    let mut c = kind.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

fn label_key(id: &str) -> &'static str {
    match id {
        "kit.button.active" => "the selected button",
        "kit.button.off" => "the disabled button",
        _ => "the button",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chip_moves_to_the_other_box_only() {
        let mut k = Specimens::default();
        assert!(k.can_drop("chip.Sword", "box.right"));
        assert!(!k.can_drop("chip.Sword", "box.left"), "already in the left box");
        assert!(!k.can_drop("Sword", "box.right"), "not a chip");
        k.accept_drop("chip.Sword", "box.right");
        assert_eq!((k.left.clone(), k.right.clone()), (vec!["Shield".to_string()], vec!["Potion".to_string(), "Sword".to_string()]));
        k.accept_drop("chip.Sword", "box.right");
        assert_eq!(k.right.len(), 2, "a refused drop changes nothing");
    }

    #[test]
    fn actions_update_the_sample_values_and_say_what_happened() {
        let mut k = Specimens::default();
        k.act(KitAct::Check);
        assert!(!k.checked);
        k.act(KitAct::Step(10));
        assert_eq!((k.value, k.last.as_str()), (9, "stepper 9"), "the stepper stops at 9");
        k.act(KitAct::Pick(3));
        assert_eq!((k.pick, k.last.as_str()), (3, "select Extra large"));
        k.act(KitAct::Slide(99));
        assert_eq!(k.slide, SLIDER_STEPS - 1);
    }
}
