//! TITLEFX1 (PR7-28, LOOK1-M1F): the title backdrop's fire layer. Lantern glows that flicker and embers that rise, at
//! spots placed in data (the picture's sidecar `<picture>.fx.json`, e.g. `backgrounds/menu/title.fx.json`, fractions
//! of the picture so the art studio can move them without code), drawn over the backdrop with `aegis_ui::fx` on the
//! menu's own clock; nothing under Reduce motion, and nothing over a picture without a sidecar (the picture itself
//! never changes).
//!
//! The sidecar: `{"spots": [{"kind": "lantern", "at": [u, v]}, {"kind": "embers", "at": [u, v]}]}`; `u` and `v` are
//! 0..1 across and down the picture. A lantern draws three looped glows
//! (nested, each on its own beat); an ember spot sends a few sparks rising and fading every few seconds.

use aegis_core::assets::{read_json, Layers};
use aegis_core::json::{self, s};
use aegis_ui::canvas::{rgba, Rect, Rgba};
use aegis_ui::fx::{hash01, Clock, Effect, Flash, Looped, Sparks};
use aegis_ui::painter::Painter;

/// The spots sidecar of picture `rel` (`backgrounds/menu/title.png` -> `backgrounds/menu/title.fx.json`).
pub fn sidecar(rel: &str) -> String {
    format!("{}.fx.json", rel.trim_end_matches(".png"))
}
/// The kinds of spot.
pub const LANTERN: &str = "lantern";
pub const EMBERS: &str = "embers";

/// A lantern's glows: (ms of a pulse, radius at the 360 px canvas, colour). Three nested discs, each fading on its own beat, make a soft falloff that flickers.
const GLOWS: [(u32, f32, Rgba); 3] = [(1400, 12.0, rgba(254, 215, 120, 24)), (900, 8.0, rgba(254, 200, 90, 30)), (530, 4.5, rgba(251, 191, 36, 44))];
/// An ember spot's cycle (ms), sparks a cycle, rise speed and lift (px a second, px a second squared, at 360 px).
const EMBER_MS: u32 = 2800;
const EMBER_COUNT: u8 = 3;
const EMBER_SPEED: f32 = 9.0;
const EMBER_LIFT: f32 = 3.0;
const EMBER_COLS: [Rgba; 2] = [rgba(251, 191, 36, 230), rgba(249, 115, 22, 220)];
/// The longest frame the clock moves by (a stall does not jump the embers).
const MAX_FRAME_MS: f64 = 100.0;

/// A placed spot.
#[derive(Debug, Clone, PartialEq)]
pub struct Spot {
    pub kind: String,
    /// Across and down the picture, 0..1.
    pub at: (f32, f32),
}

/// The fire layer: its spots, the picture they were placed on, and its clock.
#[derive(Debug, Clone, Default)]
pub struct TitleFx {
    pub picture: String,
    pub spots: Vec<Spot>,
    clock: Clock,
    last_wall: Option<aegis_core::clock::Instant>,
}

impl TitleFx {
    /// The spots of `picture` from its sidecar in the asset layers (none when the sidecar is missing; a malformed spot
    /// is left out: a spot needs a known kind and both fractions in 0..1).
    pub fn load(layers: &Layers, picture: &str) -> TitleFx {
        let Some(v) = read_json(layers, &sidecar(picture)) else { return TitleFx { picture: picture.into(), ..TitleFx::default() } };
        let d = json::dict(&v);
        let spots = json::aget(d, "spots")
            .iter()
            .map(json::dict)
            .filter_map(|sp| {
                let kind = s(sp, "kind");
                let at = json::aget(sp, "at");
                let (u, v) = (at.first()?.as_f64()? as f32, at.get(1)?.as_f64()? as f32);
                ([LANTERN, EMBERS].contains(&kind.as_str()) && (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v)).then_some(Spot { kind, at: (u, v) })
            })
            .collect();
        TitleFx { picture: picture.into(), spots, ..TitleFx::default() }
    }

    /// The layer shows over `picture` (the one drawn) unless Reduce motion is on.
    pub fn on(&self, picture: &str, reduce_motion: bool) -> bool {
        !reduce_motion && !self.spots.is_empty() && self.picture == picture
    }

    /// The clock moves by the wall time since the last frame (at most `MAX_FRAME_MS`).
    pub fn tick(&mut self) {
        let now = aegis_core::clock::Instant::now();
        let ms = self.last_wall.map_or(0.0, |l| (now.duration_since(l).as_secs_f64() * 1000.0).min(MAX_FRAME_MS));
        self.last_wall = Some(now);
        self.clock.advance(ms, 1.0);
    }

    /// The clock in ms.
    pub fn now(&self) -> u64 {
        self.clock.now()
    }

    /// Where spot `at` lands on `area` when a `pic` sized picture covers it (`panes::backdrop`'s cover fit: scaled to
    /// fill, centred, the overflow cut).
    pub fn place(at: (f32, f32), area: Rect, pic: (i32, i32)) -> (f32, f32) {
        let (pw, ph) = (pic.0.max(1) as f32, pic.1.max(1) as f32);
        let k = (area.w as f32 / pw).max(area.h as f32 / ph);
        let (w, h) = (pw * k, ph * k);
        (area.x as f32 + (area.w as f32 - w) / 2.0 + at.0 * w, area.y as f32 + (area.h as f32 - h) / 2.0 + at.1 * h)
    }

    /// The looped effects of every spot on `area` (sizes scaled from the 360 px canvas), each spot out of step with
    /// the others by its index.
    pub fn effects(&self, area: Rect, pic: (i32, i32)) -> Vec<Looped> {
        let sc = area.h.max(1) as f32 / 360.0;
        let mut out = Vec::new();
        for (i, sp) in self.spots.iter().enumerate() {
            let at = Self::place(sp.at, area, pic);
            let i = i as u64;
            if sp.kind == LANTERN {
                for (g, (ms, r, col)) in GLOWS.iter().enumerate() {
                    let start = (hash01(i, g as u64) * *ms as f32) as u64;
                    let glow = Flash { at, rx: r * sc, ry: r * sc, ms: *ms, col: *col };
                    out.push(Looped::new(Effect::new(glow, start), u64::from(*ms)));
                }
            } else {
                let start = (hash01(i, 7) * EMBER_MS as f32) as u64;
                let sparks = Sparks { at, ms: EMBER_MS, count: EMBER_COUNT, dir_deg: -90.0, spread_deg: 50.0, speed: EMBER_SPEED * sc, gravity: -EMBER_LIFT * sc, size: if sc >= 2.0 { 2 } else { 1 }, cols: EMBER_COLS };
                out.push(Looped::new(Effect::new(sparks, start).seeded(i + 1), u64::from(EMBER_MS)));
            }
        }
        out
    }

    /// Draws the layer on `area` at the clock's time.
    pub fn draw(&self, p: &mut Painter, area: Rect, pic: (i32, i32)) {
        let now = self.now();
        for e in self.effects(area, pic) {
            e.draw(p, now);
        }
    }

    /// The agent link's reading: on or off, and the spots by kind.
    pub fn state(&self, on: bool) -> serde_json::Value {
        let count = |k: &str| self.spots.iter().filter(|sp| sp.kind == k).count();
        serde_json::json!({"on": on, "lanterns": count(LANTERN), "embers": count(EMBERS)})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped() -> TitleFx {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        TitleFx::load(&Layers::of(vec![root]), crate::menus::TITLE_ART)
    }

    /// The shipped file places the mock's five lanterns and six ember spots on the title picture, all on the picture.
    #[test]
    fn the_shipped_spots_are_the_lanterns_and_embers_of_the_title_picture() {
        let fx = shipped();
        assert_eq!(fx.picture, crate::menus::TITLE_ART);
        let n = |k: &str| fx.spots.iter().filter(|sp| sp.kind == k).count();
        assert_eq!((n(LANTERN), n(EMBERS)), (5, 6));
        assert_eq!(fx.effects(Rect::new(0, 0, 640, 360), (1920, 1080)).len(), 5 * GLOWS.len() + 6);
    }

    /// The layer is off under Reduce motion, over another picture, or with no spots.
    #[test]
    fn it_is_off_under_reduce_motion_and_over_another_picture() {
        let fx = shipped();
        assert!(fx.on(crate::menus::TITLE_ART, false));
        assert!(!fx.on(crate::menus::TITLE_ART, true), "Reduce motion");
        assert!(!fx.on(crate::menus::MENU_ART, false), "the spots belong to the title picture");
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        assert!(TitleFx::load(&Layers::of(vec![root]), crate::menus::MENU_ART).spots.is_empty(), "the menu picture has no sidecar");
        assert!(!TitleFx::default().on(crate::menus::TITLE_ART, false));
    }

    /// A spot lands where the cover fit puts that point of the picture: the same aspect maps straight across; a wider
    /// area cuts the picture's top and bottom.
    #[test]
    fn a_spot_follows_the_cover_fit() {
        assert_eq!(TitleFx::place((0.5, 0.5), Rect::new(0, 0, 640, 360), (1920, 1080)), (320.0, 180.0));
        assert_eq!(TitleFx::place((0.25, 0.75), Rect::new(0, 0, 1280, 720), (1920, 1080)), (320.0, 540.0));
        // 640 x 240 from a 16:9 picture: scaled to 640 x 360, 60 px cut above and below
        assert_eq!(TitleFx::place((0.5, 0.0), Rect::new(0, 0, 640, 240), (1920, 1080)), (320.0, -60.0));
    }

    /// A malformed spot (an unknown kind, a fraction off the picture) is left out.
    #[test]
    fn a_malformed_spot_is_left_out() {
        let dir = aegis_core::scratch::Scratch::vacant("emberwake_title_fx");
        std::fs::create_dir_all(&dir).unwrap();
        let body = r#"{"spots": [{"kind": "lantern", "at": [0.5, 0.5]}, {"kind": "torch", "at": [0.1, 0.1]}, {"kind": "embers", "at": [1.5, 0.2]}, {"kind": "embers", "at": [0.2]}]}"#;
        std::fs::write(dir.join(sidecar("p.png")), body).unwrap();
        let fx = TitleFx::load(&Layers::of(vec![dir.to_path_buf()]), "p.png");
        assert_eq!(fx.spots, vec![Spot { kind: LANTERN.into(), at: (0.5, 0.5) }]);
    }
}
