//! LAYOUT720-E4, T3 for the pip gauge (moved from the engine's `panes_native.rs` with the gauge, ENGINE-5): drawn on
//! the native grid with its numbers doubled it is its old picture (`Near`: within one grid unit, `shot_diff`), with no
//! theme and with frames; its rects are twice the old ones; its `_on` answers are the old ones in the grid's units.

use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::painter::{Fonts, Grid, View};
use aegis_ui::shot_diff::shot_diff;
use aegis_ui::theme::{self, Frame, Nine, NineFill, Theme};
use aegis_ui::ui::{tip, Gfx, Kit, Ui};
use rpgkit_ui::pips::{self, PipBlock, PipGauge};
use std::path::Path;
use std::sync::Arc;

const GROUND: Rgba = [9, 18, 27, 255];
const WINDOWS: [(i32, i32); 3] = [(1280, 720), (1920, 1080), (1366, 768)];
const TEXT_SIZES: [f32; 2] = [1.0, 1.3];

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

/// `n` old units in the units this entry is drawn in.
#[derive(Clone, Copy)]
struct U(i32);

impl U {
    fn n(self, n: i32) -> i32 {
        n * self.0
    }
}

type Entry = (&'static str, (i32, i32), fn(&mut Ui<()>, U));

fn table() -> Vec<Entry> {
    vec![
        ("pips samples", (-1, -1), |ui, u| { for (k, (name, gauge)) in pips::samples().into_iter().enumerate() { let r = gauge.draw(ui, name, u.n(20), u.n(20 + 14 * k as i32)); assert_eq!(gauge.width(ui), r.w, "{name}"); } }),
        ("pips animating", (-1, -1), |ui, u| { let t = ui.t.clone(); let g = PipGauge { spent: 3.5, edge: 6.3, ..PipGauge::planned(10, &[PipBlock::new(3, t.foe), PipBlock::ghost(2, t.info)], 12, 12) }; g.draw(ui, "g", u.n(20), u.n(20)); }),
        ("pips hovered", (30, 20), |ui, u| { let t = ui.t.clone(); let g = PipGauge::planned(10, &[PipBlock::new(3, t.foe)], 12, 12).with_tip(tip("A title", "A hover card")); g.draw(ui, "g", u.n(20), u.n(20)); }),
    ]
}

/// Frames as sidecars write them: sampled at `scale` 2, tiled, averaged at `scale` 8, straddling with a corner's reach.
fn framed_theme() -> Theme {
    let img = |w: i32, h: i32| {
        let px = (0..w * h).flat_map(|i| {
            let (x, y) = (i % w, i / w);
            [(x * 7 % 251) as u8, (y * 11 % 241) as u8, ((x ^ y) * 13 % 239) as u8, 255]
        }).collect();
        Arc::new(Image { w, h, px })
    };
    let frame = |nine: Nine, w: i32, h: i32| Frame { img: img(w, h), nine, source: String::new() };
    let mut t = Theme::default();
    t.layout.focus_glow = 1;
    t.layout.radius = 0.0;
    for role in ["button", "tab", "card", "inset", "button_off", "tab_on", "card_hot"] {
        t.frames.insert(role.into(), frame(Nine { center: NineFill::None, ..Nine::new([4; 4]) }, 24, 24));
    }
    t.frames.insert("card_on".into(), frame(Nine { center: NineFill::None, ..Nine::new([5; 4]) }, 40, 40));
    t.frames.insert("button_hot".into(), frame(Nine { edges: NineFill::Tile, center: NineFill::None, ..Nine::new([8; 4]) }, 48, 48));
    t.frames.insert("panel".into(), frame(Nine { scale: 8, reduce: true, center: NineFill::None, pad: [6; 4], ..Nine::new([40; 4]) }, 200, 200));
    t.frames.insert("ornate".into(), frame(Nine { scale: 4, straddle: true, corner_anchor: Some([8, 8]), corner_inner: Some([20, 20]), center: NineFill::None, ..Nine::new([32; 4]) }, 96, 96));
    t
}

fn gfx(pw: i32, ph: i32, grid: Grid, scale: f32, fonts: &[u8]) -> Gfx {
    let mut canvas = Canvas::new(pw, ph);
    canvas.clear(GROUND);
    let mut f = Fonts::new(fonts);
    f.scale = scale;
    Gfx { canvas, view: View::fit(pw as u32, ph as u32).for_grid(grid), fonts: f, kit: Kit::default() }
}

/// `draw` in the numbers of `grid`, after `prep` set up the kit: the picture and the recorded rects (id, rect).
#[allow(clippy::too_many_arguments)]
fn shot(pw: i32, ph: i32, grid: Grid, scale: f32, mouse: (i32, i32), fonts: &[u8], prep: &dyn Fn(&mut Kit, U), draw: &mut dyn FnMut(&mut Ui<()>, U)) -> (Image, Vec<(String, Rect)>) {
    let u = U(grid.g());
    let mut g = gfx(pw, ph, grid, scale, fonts);
    prep(&mut g.kit, u);
    let rects = {
        let mouse = if mouse.0 < 0 { mouse } else { (u.n(mouse.0), u.n(mouse.1)) };
        let mut ui: Ui<()> = Ui::new(&mut g, mouse, true);
        draw(&mut ui, u);
        ui.widgets.iter().map(|w| (w.id.clone(), w.rect)).collect()
    };
    (Image { w: g.canvas.w, h: g.canvas.h, px: g.canvas.px }, rects)
}

/// Each edge of a native rect is twice the old one's, within an old unit.
fn assert_rects(name: &str, old: &[(String, Rect)], native: &[(String, Rect)]) {
    assert_eq!(old.iter().map(|w| &w.0).collect::<Vec<_>>(), native.iter().map(|w| &w.0).collect::<Vec<_>>(), "{name}: the same widgets, in order");
    for ((id, a), (_, b)) in old.iter().zip(native) {
        let edges = |r: &Rect| [r.x, r.y, r.x + r.w, r.y + r.h];
        for (e, f) in edges(a).iter().zip(edges(b)) {
            assert!((2 * e - f).abs() <= 2, "{name} {id}: {a:?} against {b:?}");
        }
    }
}

fn run_table(label: &str) {
    let fonts = font();
    let mut wrong = Vec::new();
    for (pw, ph) in WINDOWS {
        let s = View::fit(pw as u32, ph as u32).for_grid(Grid::Native).s;
        for scale in TEXT_SIZES {
            for (name, mouse, draw) in table() {
                let mut d = |ui: &mut Ui<()>, u: U| draw(ui, u);
                let (old, old_rects) = shot(pw, ph, Grid::Old, scale, mouse, &fonts, &|_, _| {}, &mut d);
                let (native, native_rects) = shot(pw, ph, Grid::Native, scale, mouse, &fonts, &|_, _| {}, &mut d);
                assert!(old.px.chunks(4).any(|p| p != GROUND), "{name}: it drew");
                if old.px != native.px {
                    let diff = shot_diff(&old, &native, s).unwrap();
                    if !diff.passed() {
                        wrong.push(format!("{label} {name} at {pw}x{ph}, text {scale}: {diff}"));
                    }
                }
                assert_rects(name, &old_rects, &native_rects);
            }
        }
    }
    assert!(wrong.is_empty(), "{} entries:\n{}", wrong.len(), wrong.join("\n"));
}

#[test]
fn the_pip_gauge_is_its_old_picture_on_the_native_grid() {
    theme::set_theme(Theme::default());
    run_table("vector");
    theme::set_theme(framed_theme());
    run_table("framed");
    theme::set_theme(Theme::default());
}

/// `_on` a grid is the unmoved answer doubled, and `_on` the old grid is the unmoved answer.
#[test]
fn the_pip_answers_on_a_grid_are_the_old_ones_in_its_units() {
    let twice = |r: Rect| Rect::new(2 * r.x, 2 * r.y, 2 * r.w, 2 * r.h);
    for slots in [-1, 0, 1, 12] {
        assert_eq!(pips::pips_w_on(slots, 2), 2 * pips::pips_w(slots));
        assert_eq!(pips::pips_w_on(slots, 1), pips::pips_w(slots));
    }
    assert_eq!(pips::pip_rect_on(40, 80, 3, 2), twice(pips::pip_rect(20, 40, 3)));
}
