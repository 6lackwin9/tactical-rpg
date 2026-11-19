//! The Kit widgets' Portraits page (PORTRAITFX1-E; ENGINE-5 part 2 moved it from the engine's `specimens`): the
//! kit's code-drawn sample face (`portrait_fx::sample`) through a `PortraitFx` at a chosen size, with toggles for each
//! overlay and state, buttons for the reactions, a local reduce motion, the anchors check and (PORTRAITFX2-M) the
//! motion profile, on its own clock. It is a `KitPage`: a game's gallery keeps a `KitWidgets` (the engine's pages with
//! this one at the fourth tab, where it always was), so every control id (`kit.portrait.*`) and pixel is as before.

use crate::portrait_fx::{Budget, Cue, ExpressionFx, Motion, PortraitFx, PortraitOpts, Tier, MOTION_SAMPLES};
use crate::portrait_overlays::Mount;
use aegis_core::assets::Face;
use aegis_ui::canvas::{rgb, Image, Rect, Rgba};
use aegis_ui::fx::Clock;
use aegis_ui::specimens::{KitAct, KitPage, PageAct, SpecimensWith};
use aegis_ui::ui::{tip, Ui, BODY, BORDER, DIM, INFO, SMALL, TEXT};
use std::cell::RefCell;
use std::collections::BTreeSet;

/// The engine's Kit widgets with the Portraits page.
pub type KitWidgets = SpecimensWith<PortraitDemo>;

/// The page: the sizes on offer (logical px), the toggles, the reactions and the stage (the largest size
/// plus the jolt ceiling and a margin on every side).
pub const PORTRAIT_SIZES: [i32; 6] = [96, 64, 48, 32, 24, 16];
pub const PORTRAIT_OVERLAYS: [&str; 5] = ["glow", "lines", "point_glow", "particles_rise", "drift"];
/// The overlay toggles' labels (`PORTRAIT_OVERLAYS` order): FB-206 (2b) renamed the cue names only; the page keeps the
/// words it always showed, so it draws byte for byte as before (the theme hashes, `tests/fixtures/theme`).
pub const PORTRAIT_OVERLAY_LABELS: [&str; 5] = ["Glow", "Veins", "Eye glow", "Ash", "Smoke"];
pub const PORTRAIT_STATES: [&str; 4] = ["desaturate", "tint", "fade", "still"];
pub const PORTRAIT_REACTIONS: [&str; 3] = ["flash", "jolt.hit", "jolt.crit"];
pub const PORTRAIT_STAGE: i32 = 96 + 2 * 6;
/// The stage's ground (the Effects page's).
const STAGE_BG: Rgba = rgb(14, 15, 20);

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
    /// PORTRAITFX2-M: the motion profile shown (`MOTION_SAMPLES` index; 0, the breath, holds no face).
    pub motion: usize,
    pub image: Image,
    pub face: Face,
}

impl Default for PortraitDemo {
    fn default() -> Self {
        let (image, face) = crate::portrait_fx::sample();
        let mut fx = PortraitFx::new("kit.sample");
        // one face per sample profile (named after it), so choosing one starts its entry and blends from the last
        let rows: Vec<ExpressionFx> = MOTION_SAMPLES.iter().filter_map(|n| Motion::sample(n).map(|m| ExpressionFx { motion: m, ..ExpressionFx::new(n) })).collect();
        fx.set_expressions(&rows);
        Self { clock: Clock::default(), fx: RefCell::new(fx), budget: RefCell::new(Budget::default()), on: BTreeSet::new(), size: 0, reduce: false, anchors: false, motion: 0, image, face }
    }
}

impl PortraitDemo {
    /// Toggles overlay or state `e` and holds the new set; what it did.
    fn toggle(&mut self, e: &'static str) -> String {
        let on = !self.on.remove(e);
        if on {
            self.on.insert(e);
        }
        self.hold();
        format!("portrait {} {}", e, if on { "on" } else { "off" })
    }

    /// Holds the toggled set and the chosen motion's face.
    fn hold(&mut self) {
        let mut cues: Vec<Cue> = self.on.iter().filter_map(|x| Cue::parse(x, "").map(|c| c.with_key(&format!("kit.{x}")))).collect();
        let name = MOTION_SAMPLES[self.motion % MOTION_SAMPLES.len()];
        if self.motion > 0 {
            cues.extend(Cue::parse("face", "").map(|c| c.with_expression(name).with_key("kit.motion")));
        }
        self.fx.borrow_mut().hold(&cues, self.clock.now());
    }

    /// PORTRAITFX2-M: shows motion profile `i`; what it did.
    fn set_motion(&mut self, i: usize) -> String {
        self.motion = i % MOTION_SAMPLES.len();
        self.hold();
        format!("portrait motion {}", MOTION_SAMPLES[self.motion])
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
        // the page shows every overlay toggle, so it mounts the opt-in overlay layer (ASSETORG4)
        PortraitOpts { reduce_motion: self.reduce, rim_default: None, show_anchors: self.anchors, tier_cap: None, overlays: Mount::Mounted }
    }

    /// The portrait's agent state at the page's clock (`PortraitFx::state`).
    pub fn state(&self) -> serde_json::Value {
        self.fx.borrow().state(self.clock.now())
    }
}

impl KitPage for PortraitDemo {
    const NAME: Option<&'static str> = Some("Portraits");
    const AT: usize = 3;

    fn act(&mut self, a: &PageAct) -> String {
        match *a {
            PageAct::Toggle(e) => self.toggle(e),
            PageAct::Fire(w) => self.fire(w),
            PageAct::Pick("motion", i) => self.set_motion(i),
            PageAct::Pick(_, i) => {
                self.size = i % PORTRAIT_SIZES.len();
                format!("portrait size {} px", PORTRAIT_SIZES[self.size])
            }
            PageAct::Check("anchors") => {
                self.anchors = !self.anchors;
                format!("portrait anchors {}", if self.anchors { "on" } else { "off" })
            }
            PageAct::Check(_) => {
                self.reduce = !self.reduce;
                format!("portrait reduce motion {}", if self.reduce { "on" } else { "off" })
            }
        }
    }

    fn advance(&mut self, ms: f64) {
        self.clock.advance(ms, 1.0);
    }

    /// The Portraits page: the stage (`kit.portrait.face`: the portrait and its jolt room) with the sample face at the
    /// chosen size and, under it, the tier and what is live; beside it a toggle per overlay and state
    /// (`kit.portrait.<effect>`), a button per reaction (`kit.portrait.flash`, `kit.portrait.jolt.hit`,
    /// `kit.portrait.jolt.crit`), the size (`kit.portrait.size`), reduce motion (`kit.portrait.reduce`), the
    /// anchors check (`kit.portrait.anchors`) and (PORTRAITFX2-M) the motion profile (`kit.portrait.motion`, the foot
    /// line naming its live parts).
    fn draw<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let d = self;
        let x = r.x + 6;
        let line = ui.p.line_h(SMALL).ceil() as i32;
        ui.p.text(x as f32, (r.y + 4) as f32, "Portraits: base life, overlays, states and reactions over a code-drawn face", SMALL, INFO);
        let y = r.y + 4 + line + 2;
        let stage = Rect::new(x, y, PORTRAIT_STAGE, PORTRAIT_STAGE);
        ui.p.panel(stage, STAGE_BG, BORDER, 2.0);
        let size = PORTRAIT_SIZES[d.size.min(PORTRAIT_SIZES.len() - 1)];
        let face = Rect::new(stage.x + (stage.w - size) / 2, stage.y + (stage.h - size) / 2, size, size);
        let now = d.clock.now();
        {
            let mut budget = d.budget.borrow_mut();
            budget.begin();
            d.fx.borrow_mut().draw_image(&mut ui.p, &d.image, "kit.sample", Some(&d.face), face, now, &d.opts(), &mut budget);
        }
        let (tier, live, parts) = {
            let fx = d.fx.borrow();
            (fx.tier().map_or("", Tier::name), fx.live(now), fx.motion(crate::portrait_fx::tq(now)).parts)
        };
        let device = (size as f32 * ui.p.v.s).round() as i32;
        ui.push("kit.portrait.face", "portrait", format!("sample portrait: {size} px, tier {tier}"), stage, true, false, None, Vec::new());
        let col_w = 200;
        let named = |v: &[&str]| if v.is_empty() { "nothing".to_string() } else { v.join(", ") };
        let foot = format!("Tier {tier} ({device} device px). Live: {}", named(&live));
        let fy = ui.para(x, stage.y + stage.h + 4, col_w, &foot, SMALL, DIM);
        let fy = ui.para(x, fy, col_w, &format!("Motion {}: {}", MOTION_SAMPLES[d.motion % MOTION_SAMPLES.len()], named(&parts)), SMALL, DIM);
        ui.para(
            x,
            fy + 4,
            col_w,
            // the labels' words (`PORTRAIT_OVERLAY_LABELS`), kept so the page draws as before FB-206 (2b)
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
                let label = portrait_label(name);
                let w = 12 + ui.p.text_w(&label, BODY).ceil() as i32 + 4;
                if bx + w > right {
                    bx = x2;
                    yy += 16;
                }
                ui.checkbox(&format!("kit.portrait.{name}"), bx, yy + 2, &label, d.on.contains(name), wrap(KitAct::Extra(PageAct::Toggle(name))), true);
                bx += w + 8;
            }
            yy += 14 + 5;
        }
        yy = head(ui, yy, "Reactions");
        let mut bx = x2;
        for (what, label) in PORTRAIT_REACTIONS.iter().zip(["Flash", "Hit jolt", "Crit jolt"]) {
            let w = (ui.p.text_w(label, BODY) + 12.0).ceil() as i32;
            ui.button(&format!("kit.portrait.{what}"), Rect::new(bx, yy, w, 14), label, wrap(KitAct::Extra(PageAct::Fire(what))), true);
            bx += w + 3;
        }
        yy += 14 + 5;
        yy = head(ui, yy, "Size, motion, anchors");
        let sizes: Vec<String> = PORTRAIT_SIZES.iter().map(|s| format!("{s} px")).collect();
        ui.select("kit.portrait.size", Rect::new(x2, yy, 56, 14), &sizes, d.size, |i| wrap(KitAct::Extra(PageAct::Pick("size", i))), true, tip("Size", "The portrait's side in logical px; the tier follows the device px."));
        ui.checkbox("kit.portrait.reduce", x2 + 64, yy + 2, "Reduce motion (this page)", d.reduce, wrap(KitAct::Extra(PageAct::Check("reduce"))), true);
        yy += 14 + 5;
        ui.checkbox("kit.portrait.anchors", x2, yy + 2, "Anchors (eye boxes, glow, vein roots)", d.anchors, wrap(KitAct::Extra(PageAct::Check("anchors"))), true);
        // PORTRAITFX2-M: the motion profile
        yy += 14 + 5;
        let names: Vec<String> = MOTION_SAMPLES.iter().map(|n| cap(n)).collect();
        ui.select("kit.portrait.motion", Rect::new(x2, yy, 72, 14), &names, d.motion, |i| wrap(KitAct::Extra(PageAct::Pick("motion", i))), true, tip("Motion", "The face's motion profile: parts combined by one function (cycle, lean, bob, entry, settle, rest, shiver, flush)."));
        ui.p.text((x2 + 80) as f32, (yy + 2) as f32, "Motion profile", SMALL, DIM);
    }
}

/// A Portraits page toggle's label: an overlay's `PORTRAIT_OVERLAY_LABELS` entry, else `name` with `_` as spaces and
/// its first letter in upper case.
pub fn portrait_label(name: &str) -> String {
    match PORTRAIT_OVERLAYS.iter().position(|o| *o == name) {
        Some(i) => PORTRAIT_OVERLAY_LABELS[i].to_string(),
        None => cap(&name.replace('_', " ")),
    }
}

/// `kind` with its first letter in upper case.
fn cap(kind: &str) -> String {
    let mut c = kind.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}
