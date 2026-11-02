//! LAYOUT720-E4, T3 (docs/specs/LAYOUT720.md section 5): every pane, the pip gauge, the timeline and the desk drawn on the
//! native grid with their numbers doubled are their old pictures. `Same` is to the bit; `Near` allows one grid unit
//! (`shot_diff`) where a text height, a measured width or a whole-number division rounds on a different unit (spec 6.3). The
//! recorded rects are checked too: twice as much, each edge within one old unit. T1 (`theme.rs`, `theme_panes.rs`) holds the
//! old pictures to their recorded hashes, and the sweeps here hold the old numbers on sizes those pages never draw.
//!
//! `AEGIS_SHOT_DIR=<folder>` writes each entry's old and native shots there. `AEGIS_THEME_GAME=<game folder>` runs the table on
//! that game's theme too (its real frames), so that this file names no game.

use aegis_core::assets::Layers;
use aegis_layout::{Insets, Stack};
use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::painter::{nine_content_rect, Fonts, Grid, View};
use aegis_ui::panes::{self, AskView, Corner, MenuItem, MenuStyle, SlotView};
use aegis_ui::pips::{self, PipBlock, PipGauge};
use aegis_ui::shot_diff::shot_diff;
use aegis_ui::theme::{self, Frame, Metrics, Nine, NineFill, Theme};
use aegis_ui::timeline::{self, Timeline};
use aegis_ui::ui::{tip, Gfx, Glyph, Kit, Ui};
use aegis_ui::window::{Desk, Frac, Grab, Layout, Win};
use std::path::Path;
use std::sync::Arc;

const GROUND: Rgba = [9, 18, 27, 255];
const WINDOWS: [(i32, i32); 3] = [(1280, 720), (1920, 1080), (1366, 768)];
const TEXT_SIZES: [f32; 2] = [1.0, 1.3];
const FILL: Rgba = [40, 80, 40, 255];
const LONG: &str = "One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen";

#[derive(Clone, Copy, PartialEq)]
enum Match {
    Same,
    Near,
    /// Within a unit at a whole scale. At another, a curve moved by that unit falls differently on the pixels, so only its
    /// rects are held.
    Whole,
    /// Only its rects are held, at every scale, for the reason in the table.
    Rects,
}
use Match::{Near, Rects, Same, Whole};

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

/// `n` old units in the units this entry is drawn in.
#[derive(Clone, Copy)]
struct U(i32);

impl U {
    fn r(self, x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect::new(x * self.0, y * self.0, w * self.0, h * self.0)
    }
    fn n(self, n: i32) -> i32 {
        n * self.0
    }
    fn grid(self) -> Grid {
        if self.0 == 1 {
            Grid::Old
        } else {
            Grid::Native
        }
    }
}

type Entry = (&'static str, Match, (i32, i32), fn(&mut Ui<()>, U));

fn art(w: i32, h: i32) -> Image {
    let px = (0..w * h).flat_map(|i| [(i % w * 7 % 251) as u8, (i / w * 11 % 241) as u8, 90, 255]).collect();
    Image { w, h, px }
}

fn menu() -> Vec<MenuItem<()>> {
    vec![MenuItem::new("first", "New game", Some('n'), ()), MenuItem::new("second", "Load", Some('l'), ()).with_tip("Open a save."), MenuItem::new("third", "A label much too long to fit in this column at all", None, ()), MenuItem::new("fourth", "Nothing yet", Some('x'), ()).off("Not here yet.")]
}

#[allow(clippy::too_many_arguments)]
fn card(ui: &mut Ui<()>, id: &str, r: Rect, thumb: Option<&Image>, portraits: &[&Image], stamp: &str, selected: bool, corners: usize) {
    let view = SlotView { thumb, title: "A slot's title".into(), lines: vec!["The first line".into(), "Day 3, evening".into(), "A third that may not fit in this card".into()], portraits: portraits.to_vec(), stamp: stamp.into() };
    let corner: Vec<Corner<()>> = (0..corners).map(|i| Corner { key: format!("c{i}"), label: "Lock".into(), glyph: Glyph::Lock, action: (), enabled: i == 0, toggled: i == 1, tip: String::new() }).collect();
    panes::slot_card(ui, id, r, &view, Some(()), selected, &corner);
}

fn ask(ui: &mut Ui<()>, area: Rect, v: &AskView) {
    panes::ask_dialog(ui, "ask", area, v, (), ());
}

fn sample_desk(u: U) -> Desk {
    let wins = vec![Win::new("a", "Alpha", (60, 40)), Win::new("b", "A title too long for its window to show", (60, 40)), Win::new("c", "Gamma", (60, 40))];
    let layouts = vec![Layout { name: "Two".into(), places: vec![("a".into(), Frac::new(0.0, 0.0, 0.5, 1.0)), ("b".into(), Frac::new(0.5, 0.0, 0.25, 0.5))] }, Layout { name: "One".into(), places: vec![("c".into(), Frac::new(0.25, 0.25, 0.5, 0.5))] }];
    Desk::new(u.r(10, 30, 400, 200), wins, layouts).on_grid(u.grid())
}

fn draw_desk(ui: &mut Ui<()>, d: &Desk) {
    d.draw(ui, |ui, _, b| ui.p.fill(Rect::new(b.x, b.y, b.w / 2, b.h / 2), FILL));
}

fn tl(ui: &mut Ui<()>, t: &Timeline, r: Rect) {
    t.draw(ui, "tl", r, |_| (), |_| (), |_, _| ());
}

/// The sample with one glyph on its marks, whose chip is as wide on either grid. A chip's width is a measured glyph rounded up
/// in units, so one that is not moves a unit and its round corners fall on other pixels (`timeline mark widths`).
fn even_marks() -> Timeline {
    let mut t = timeline::sample();
    t.sections.iter_mut().flat_map(|s| &mut s.tracks).flat_map(|k| &mut k.marks).for_each(|m| m.glyph = "!".into());
    t
}

fn table() -> Vec<Entry> {
    vec![
        ("backdrop", Same, (-1, -1), |ui, u| { let i = art(64, 48); panes::backdrop_pane(ui, "b", Some(&i), "pic", u.r(0, 0, 640, 360), 128, 0.6); }),
        ("backdrop no picture", Same, (-1, -1), |ui, u| { panes::backdrop_pane(ui, "b", None, "none", u.r(40, 30, 300, 200), 0, 1.0); }),
        ("backdrop live", Same, (-1, -1), |ui, u| { let mut i = art(40, 40); i.px[3] = 0; panes::backdrop_pane(ui, "b", Some(&i), "see-through", u.r(40, 30, 301, 201), 40, 0.5); }),
        ("menu framed", Same, (-1, -1), |ui, u| { panes::menu_column(ui, "m", u.n(20), u.n(20), u.n(160), u.n(20), u.n(24), &menu()); }),
        ("menu framed hovered", Same, (40, 50), |ui, u| { let hovered = panes::menu_column(ui, "m", u.n(20), u.n(20), u.n(160), u.n(20), u.n(24), &menu()); assert_eq!(hovered, Some(1)); }),
        ("menu text", Same, (-1, -1), |ui, u| { panes::menu_column_styled(ui, "m", u.n(20), u.n(20), u.n(160), u.n(14), u.n(16), &menu(), MenuStyle::Text); }),
        // The mouse's row is one of a label's measured width, in units.
        ("menu text hovered", Near, (40, 40), |ui, u| { let hovered = panes::menu_column_styled(ui, "m", u.n(20), u.n(20), u.n(160), u.n(14), u.n(16), &menu(), MenuStyle::Text); assert_eq!(hovered, Some(1)); }),
        // The y below the title is a text height rounded up, then up to the grid.
        ("menu title", Same, (-1, -1), |ui, u| { let y = panes::menu_title(ui, u.n(20), u.n(20), u.n(200), "A title", "A subtitle"); ui.p.fill(Rect::new(u.n(20), y, u.n(100), u.n(2)), FILL); let z = panes::menu_title(ui, u.n(20), y + u.n(4), u.n(30), "A title too long for it", ""); ui.p.fill(Rect::new(u.n(20), z, u.n(100), u.n(2)), FILL); }),
        ("tab rail", Same, (-1, -1), |ui, u| { let c = panes::tab_rail(ui, "t", u.r(20, 20, 400, 200), &["One", "Two", "Three"], 1, |_| ()); ui.p.fill(c, FILL); }),
        ("tab rail arrows", Same, (-1, -1), |ui, u| { let tabs: Vec<String> = (0..12).map(|i| format!("Tab {i}")).collect(); let c = panes::tab_rail(ui, "t", u.r(20, 20, 400, 100), &tabs, 7, |_| ()); ui.p.fill(c, FILL); }),
        ("tab rail narrow", Same, (-1, -1), |ui, u| { let c = panes::tab_rail(ui, "t", u.r(20, 20, 90, 37), &["A long tab name", "Two", "Three"], 0, |_| ()); ui.p.fill(c, FILL); }),
        ("tab rail hovered", Same, (40, 45), |ui, u| { panes::tab_rail(ui, "t", u.r(20, 20, 400, 200), &["One", "Two", "Three"], 0, |_| ()); }),
        ("slot empty", Same, (-1, -1), |ui, u| { card(ui, "s", u.r(20, 20, 400, 91), None, &[], "", false, 0); }),
        ("slot full", Same, (-1, -1), |ui, u| { let (i, f) = (art(64, 36), art(20, 20)); card(ui, "s", u.r(20, 20, 400, 91), Some(&i), &[&f, &f, &f], "", true, 2); }),
        // Right aligned on a measured width rounded up in units, so its letters round afresh a unit away.
        ("slot stamp", Whole, (-1, -1), |ui, u| { card(ui, "s", u.r(20, 20, 400, 91), None, &[], "2026-11-03 10:30", false, 0); }),
        // A small card's thumbnail keeps 16:9: a whole-number division.
        ("slot small", Near, (-1, -1), |ui, u| { let f = art(20, 20); card(ui, "s", u.r(20, 20, 260, 51), None, &[&f], "A stamp", false, 1); }),
        ("slot narrow", Same, (-1, -1), |ui, u| { let f = art(20, 20); card(ui, "s", u.r(20, 20, 150, 91), None, &[&f, &f], "A stamp too long for this card", false, 2); }),
        ("slot hovered", Same, (300, 40), |ui, u| { card(ui, "s", u.r(20, 20, 400, 91), None, &[], "", false, 0); }),
        // The title's height rounds up in units, which moves the question under it.
        ("ask panel", Near, (-1, -1), |ui, u| { ask(ui, u.r(0, 0, 640, 360), &AskView::new("A title", "Is this the question?")); }),
        ("ask ornate", Near, (-1, -1), |ui, u| { ask(ui, u.r(0, 0, 640, 360), &AskView { ornate: true, no_default: true, ..AskView::new("A title", "Is this the question?") }); }),
        ("ask long", Same, (-1, -1), |ui, u| { ask(ui, u.r(0, 0, 640, 360), &AskView::new("", LONG)); }),
        ("ask cut", Near, (-1, -1), |ui, u| { let q = LONG.repeat(12); ask(ui, u.r(100, 60, 300, 170), &AskView { ornate: true, ..AskView::new("A title much too long to fit on one line of a dialog this small, however it is drawn", &q) }); }),
        ("ask hovered", Near, (340, 200), |ui, u| { ask(ui, u.r(0, 0, 640, 360), &AskView::new("A title", "Is this the question?")); }),
        ("ask answers too wide", Same, (-1, -1), |ui, u| { ask(ui, u.r(0, 0, 640, 360), &AskView { yes: "A very long way of saying yes that runs on and on for a while", no: "And a very long way of saying no, longer still than that one", ..AskView::new("", "Yes or no?") }); }),
        ("pips samples", Near, (-1, -1), |ui, u| { for (k, (name, gauge)) in pips::samples().into_iter().enumerate() { let r = gauge.draw(ui, name, u.n(20), u.n(20 + 14 * k as i32)); assert_eq!(gauge.width(ui), r.w, "{name}"); } }),
        ("pips animating", Near, (-1, -1), |ui, u| { let t = ui.t.clone(); let g = PipGauge { spent: 3.5, edge: 6.3, ..PipGauge::planned(10, &[PipBlock::new(3, t.foe), PipBlock::ghost(2, t.info)], 12, 12) }; g.draw(ui, "g", u.n(20), u.n(20)); }),
        ("pips hovered", Near, (30, 20), |ui, u| { let t = ui.t.clone(); let g = PipGauge::planned(10, &[PipBlock::new(3, t.foe)], 12, 12).with_tip(tip("A title", "A hover card")); g.draw(ui, "g", u.n(20), u.n(20)); }),
        // 624 wide: the lane is a whole number of units per block on either grid.
        ("timeline", Near, (-1, -1), |ui, u| { tl(ui, &even_marks(), u.r(10, 10, 624, 340)); }),
        ("timeline hovered bar", Near, (250, 45), |ui, u| { tl(ui, &even_marks(), u.r(10, 10, 624, 340)); }),
        ("timeline hovered block", Near, (250, 15), |ui, u| { tl(ui, &even_marks(), u.r(10, 10, 624, 340)); }),
        ("timeline narrow", Near, (-1, -1), |ui, u| { tl(ui, &even_marks(), u.r(10, 10, 300, 120)); }),
        // A block's edge is a division of the lane, on a unit of its grid: up to a unit away, and the ruler's text with it.
        ("timeline lane widths", Rects, (-1, -1), |ui, u| { tl(ui, &even_marks(), u.r(10, 10, 620, 340)); }),
        ("timeline mark widths", Rects, (-1, -1), |ui, u| { tl(ui, &timeline::sample(), u.r(10, 10, 624, 340)); }),
        ("timeline icons", Near, (-1, -1), |ui, u| { let mut t = even_marks(); t.sections.iter_mut().flat_map(|s| &mut s.tracks).for_each(|k| k.icon = true); t.draw_icons(ui, "tl", u.r(10, 10, 624, 340), |_| (), |_| (), |_, _| (), |ui, _, sq| ui.p.fill(sq, FILL)); }),
        ("desk", Same, (-1, -1), |ui, u| { draw_desk(ui, &sample_desk(u)); }),
        ("desk collapsed", Same, (-1, -1), |ui, u| { let mut d = sample_desk(u); d.collapse("a"); draw_desk(ui, &d); }),
        ("desk hovered close", Same, (205, 35), |ui, u| { draw_desk(ui, &sample_desk(u)); }),
        ("desk hovered collapse", Same, (193, 35), |ui, u| { draw_desk(ui, &sample_desk(u)); }),
        ("desk hovered grip", Same, (207, 227), |ui, u| { draw_desk(ui, &sample_desk(u)); }),
        ("desk toolbar button", Near, (-1, -1), |ui, u| { let d = Desk::new(u.r(10, 30, 400, 200), vec![Win::new("a", "Alpha", (60, 40))], Vec::new()).on_grid(u.grid()); d.toolbar(ui, u.n(10), u.n(10)); }),
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

fn mismatch(want: Match, old: &Image, native: &Image, s: f32) -> Option<String> {
    if old.px == native.px || want == Rects || (want == Whole && s.fract() != 0.0) {
        return None;
    }
    let diff = shot_diff(old, native, s).unwrap();
    if !diff.passed() {
        return Some(diff.to_string());
    }
    (want == Same).then(|| format!("within {} px, not to the bit as the table says", diff.radius))
}

fn save(path: std::path::PathBuf, img: &Image) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img.px).unwrap();
}

fn run_table(label: &str) {
    let fonts = font();
    let shots_to = std::env::var("AEGIS_SHOT_DIR").ok();
    let mut wrong = Vec::new();
    for (pw, ph) in WINDOWS {
        let s = View::fit(pw as u32, ph as u32).for_grid(Grid::Native).s;
        for scale in TEXT_SIZES {
            for (name, want, mouse, draw) in table() {
                let mut d = |ui: &mut Ui<()>, u: U| draw(ui, u);
                let (old, old_rects) = shot(pw, ph, Grid::Old, scale, mouse, &fonts, &|_, _| {}, &mut d);
                let (native, native_rects) = shot(pw, ph, Grid::Native, scale, mouse, &fonts, &|_, _| {}, &mut d);
                assert!(old.px.chunks(4).any(|p| p != GROUND), "{name}: it drew");
                if let Some(why) = mismatch(want, &old, &native, s) {
                    wrong.push(format!("{label} {name} at {pw}x{ph}, text {scale}: {why}"));
                }
                assert_rects(name, &old_rects, &native_rects);
                if let Some(dir) = &shots_to {
                    let file = format!("{label}_{}_{pw}_text{scale}", name.replace(' ', "_"));
                    save(Path::new(dir).join(format!("{file}_old.png")), &old);
                    save(Path::new(dir).join(format!("{file}_native.png")), &native);
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{} entries:\n{}", wrong.len(), wrong.join("\n"));
}

#[test]
fn every_pane_is_its_old_picture_on_the_native_grid_with_no_theme() {
    theme::set_theme(Theme::default());
    run_table("vector");
}

#[test]
fn every_pane_is_its_old_picture_on_the_native_grid_with_frames() {
    theme::set_theme(framed_theme());
    run_table("framed");
    theme::set_theme(Theme::default());
}

/// A real theme's frames, when `AEGIS_THEME_GAME` names their game (not a check the gate runs: an engine test has no game to
/// read).
#[test]
fn every_pane_is_its_old_picture_under_a_real_theme() {
    let Ok(dir) = std::env::var("AEGIS_THEME_GAME") else { return };
    let t = theme::load(&Layers::for_game(Path::new(&dir))).expect("the theme loads");
    assert!(!t.frames.is_empty(), "it has frames");
    theme::set_theme(t);
    run_table("real");
    theme::set_theme(Theme::default());
}

/// The rows past the fold, scrolled by the same share on each grid.
#[test]
fn a_scrolled_timeline_is_its_old_picture_on_the_native_grid() {
    theme::set_theme(Theme::default());
    let fonts = font();
    let (pw, ph, s) = (1920, 1080, 1.5);
    let prep = |k: &mut Kit, u: U| {
        k.scroll.insert("tl.rows".into(), u.n(90));
    };
    // The scroll bar is covered: its knob's place is a whole-number division of the offset, on a unit of each grid.
    let mut d = |ui: &mut Ui<()>, u: U| {
        tl(ui, &even_marks(), u.r(10, 10, 624, 200));
        ui.p.fill(u.r(631, 10, 3, 200), GROUND);
    };
    let (old, old_rects) = shot(pw, ph, Grid::Old, 1.0, (-1, -1), &fonts, &prep, &mut d);
    let (native, native_rects) = shot(pw, ph, Grid::Native, 1.0, (-1, -1), &fonts, &prep, &mut d);
    let diff = shot_diff(&old, &native, s).unwrap();
    assert!(diff.passed(), "{diff}");
    assert_rects("scrolled", &old_rects, &native_rects);
}

// ---- What has no `Ui` to read -----------------------------------------------------------------------------------------

/// `_on` a grid is the unmoved answer doubled, whole, and `_on` the old grid is the unmoved answer.
#[test]
fn the_answers_on_a_grid_are_the_old_ones_in_its_units() {
    theme::set_theme(Theme::default());
    let (old, native) = (Metrics::of(&Theme::default(), View::fit(1280, 720)), Metrics::of(&Theme::default(), View::fit(1280, 720).for_grid(Grid::Native)));
    let twice = |r: Rect| Rect::new(2 * r.x, 2 * r.y, 2 * r.w, 2 * r.h);
    for slots in [-1, 0, 1, 12] {
        assert_eq!(pips::pips_w_on(slots, 2), 2 * pips::pips_w(slots));
        assert_eq!(pips::pips_w_on(slots, 1), pips::pips_w(slots));
    }
    assert_eq!(pips::pip_rect_on(40, 80, 3, 2), twice(pips::pip_rect(20, 40, 3)));
    let row = Rect::new(10, 20, 300, timeline::ROW_H);
    assert_eq!(timeline::icon_rect_on(twice(row), 2), twice(timeline::icon_rect(row)));
    let t = timeline::sample();
    assert_eq!(t.content_h_on(2), 2 * t.content_h());
    let seg = &t.sections[0].tracks[0].segments[1];
    assert_eq!(t.bar_on(seg, twice(row), 200, 1200, 2), twice(t.bar(seg, row, 100, 600)));
    let mark = Rect::new(50, 0, 7, 10);
    let (a, b) = timeline::clear_of(10, 100, &[mark], 200);
    assert_eq!(timeline::clear_of_on(20, 200, &[twice(mark)], 400, 2), (2 * a, 2 * b), "a mark keeps its old unit clear past it");
    let (a, b) = timeline::clear_of(10, 100, &[mark], 5);
    assert_eq!(timeline::clear_of_on(20, 200, &[twice(mark)], 10, 2), (2 * a, 2 * b), "and before it");
    for r in [Rect::new(20, 20, 400, 200), Rect::new(20, 20, 90, 37)] {
        let (rows, content) = panes::tab_rail_rects(r, 5);
        let (native_rows, native_content) = panes::tab_rail_rects_on(&native, twice(r), 5);
        assert_eq!(native_rows, rows.iter().map(|x| twice(*x)).collect::<Vec<_>>());
        assert_eq!(native_content, twice(content));
        assert_eq!(panes::tab_rail_rects_on(&old, r, 5), (rows, content));
    }
    let r = Rect::new(20, 20, 400, 91);
    let (thumb, tx, py, icons) = panes::slot_rects(r, 2);
    let (n_thumb, n_tx, n_py, n_icons) = panes::slot_rects_on(&native, twice(r), 2);
    assert_eq!((n_thumb, n_tx, n_py), (twice(thumb), 2 * tx, 2 * py));
    assert_eq!(n_icons, icons.iter().map(|x| twice(*x)).collect::<Vec<_>>());
    assert_eq!(panes::snap_on(13, 8), 2 * panes::snap(7), "the grid token doubled snaps as the old one");
    assert_eq!(panes::snap_up_on(10, 8), 2 * panes::snap_up(5));
    assert_eq!(panes::snap_down_on(14, 8), 2 * panes::snap_down(7));
}

// ---- The old numbers, on sizes no pinned page draws ---------------------------------------------------------------------

fn old_tab_rail(t: &Theme, r: Rect, n: usize) -> (Vec<Rect>, Rect) {
    let rail = panes::RAIL_W.min(panes::snap_down(r.w / 2));
    let rows = (0..n).map(|i| Rect::new(r.x, r.y + i as i32 * panes::TAB_STEP, rail, panes::TAB_H)).take_while(|row| row.y + row.h <= r.y + r.h).collect();
    let x = r.x + rail + t.layout.gutter;
    (rows, Rect::new(x, r.y, (r.x + r.w - x).max(0), r.h))
}

fn old_slot(t: &Theme, r: Rect, corners: usize) -> (Rect, i32, i32, Vec<Rect>) {
    let pad = t.layout.padding;
    let th_h = panes::THUMB_H.min(r.h - 2 * pad).max(0);
    let th_w = if th_h == panes::THUMB_H { panes::THUMB_W } else { th_h * 16 / 9 };
    let thumb = Rect::new(r.x + pad, r.y + (r.h - th_h) / 2, th_w, th_h);
    let icons = (0..corners).map(|i| Rect::new(r.x + r.w - 2 - panes::CORNER * (corners - i) as i32, r.y + 2, panes::CORNER, panes::CORNER)).collect();
    (thumb, thumb.x + thumb.w + pad, r.y + r.h - pad - panes::PORTRAIT, icons)
}

/// The old arithmetic of `ask_layout`, as it stood before it went relative.
fn old_ask(ui: &mut Ui<()>, t: &Theme, area: Rect, v: &AskView) -> (Rect, Vec<String>, Rect, Rect) {
    use aegis_ui::ui::{BODY, HEAD};
    let (m, pad, gutter) = (t.layout.margin, t.layout.padding, t.layout.gutter);
    let p = &mut ui.p;
    let s = p.fonts.scale.max(1.0);
    let w = panes::snap_up((panes::ASK_MIN_W as f32 * s).round() as i32).min(panes::snap_down(area.w - 2 * m)).max(0);
    let role = if v.ornate { "ornate" } else { "panel" };
    let view = p.v.s;
    let inset = |r: Rect| t.frame(role).map_or(r.inset(pad), |f| nine_content_rect(&f.nine, r, view).inset(pad / 2));
    let least_h = (panes::ASK_MIN_H as f32 * s).round() as i32;
    let inner_w = inset(Rect::new(0, 0, w, least_h)).w;
    let lines = p.wrap(v.question, BODY, inner_w as f32);
    let lh = p.line_h(BODY).ceil() as i32;
    let title_h = if v.title.is_empty() { 0 } else { p.line_h_face(aegis_ui::painter::Typeface::Display, HEAD).ceil() as i32 + gutter };
    let bh = panes::snap_up(p.line_h(BODY).ceil() as i32 + 6);
    let bw = panes::snap_up((p.text_w(v.yes, BODY).max(p.text_w(v.no, BODY)) + 24.0).ceil() as i32).max(48);
    let frame_pad = least_h - inset(Rect::new(0, 0, w, least_h)).h;
    let need = frame_pad + title_h + lines.len() as i32 * lh + gutter + bh;
    let h = panes::snap_up(least_h.max(need)).min(panes::snap_down(area.h - 2 * m)).max(0);
    let r = Rect::new(area.x + panes::snap((area.w - w) / 2), area.y + panes::snap((area.h - h) / 2), w, h);
    let inner = inset(r);
    let no = Rect::new(inner.x + inner.w - bw, inner.y + inner.h - bh, bw, bh);
    let yes = Rect::new(no.x - gutter - bw, no.y, bw, bh);
    (r, lines, yes, no)
}

/// A zero-wide rect has no picture and takes no click, so where it sits does not matter.
fn alike(a: Rect, b: Rect) -> bool {
    a == b || (a.w <= 0 && b.w <= 0 && a.y == b.y && a.h == b.h)
}

/// Going relative changed no number of a screen on the old grid, over every size in steps, however small: the rail, the card
/// and the dialog, with and without frames, at each text size.
#[test]
fn the_old_grid_keeps_its_numbers_on_every_size() {
    let fonts = font();
    for t in [Theme::default(), framed_theme()] {
        theme::set_theme(t.clone());
        for w in (0..700).step_by(13) {
            for h in (0..390).step_by(11) {
                let r = Rect::new(17, 9, w, h);
                for n in [0, 1, 3, 40] {
                    let (rows, content) = panes::tab_rail_rects(r, n);
                    let (want_rows, want_content) = old_tab_rail(&t, r, n);
                    assert_eq!(rows, want_rows, "{r:?} {n}");
                    assert!(alike(content, want_content), "{r:?}: {content:?} against {want_content:?}");
                }
                for corners in [0, 2] {
                    assert_eq!(panes::slot_rects(r, corners), old_slot(&t, r, corners), "{r:?}");
                }
            }
        }
        let questions = [AskView::new("", "Short?"), AskView::new("A title", LONG), AskView { ornate: true, ..AskView::new("A title", LONG) }, AskView { yes: "A very long way of saying yes, on and on", no: "No", ..AskView::new("", "Short?") }];
        for scale in TEXT_SIZES {
            let mut g = gfx(1280, 720, Grid::Old, scale, &fonts);
            let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            for w in (0..700).step_by(37) {
                for h in (0..390).step_by(23) {
                    let area = Rect::new(11, 7, w, h);
                    for v in &questions {
                        let now = panes::ask_layout(&mut ui.p, area, v);
                        let was = old_ask(&mut ui, &t, area, v);
                        // A vector panel with no room for its padding has no body: `Ui::frame_body`'s stays on its far edge,
                        // where the old sum ran past it. Its answers move by that, and keep their size.
                        let room = t.frame(if v.ornate { "ornate" } else { "panel" }).is_some() || (now.0.w >= 2 * t.layout.padding && now.0.h >= 2 * t.layout.padding);
                        if room {
                            assert_eq!(now, was, "{area:?} text {scale}: {}", v.question);
                        } else {
                            assert_eq!((&now.0, &now.1, now.2.w, now.3.w), (&was.0, &was.1, was.2.w, was.3.w), "{area:?} text {scale}: {}", v.question);
                        }
                    }
                }
            }
        }
    }
    theme::set_theme(Theme::default());
}

// ---- Tokens, and one corner size --------------------------------------------------------------------------------------

/// `metrics` reaches a native pane and no old one: the read path is one.
#[test]
fn a_metrics_key_changes_a_native_pane_and_no_old_one() {
    let fonts = font();
    let mut t = Theme::default();
    t.metrics.padding = Some(30);
    t.metrics.body = Some(20.0);
    t.metrics.small = Some(18.0);
    t.metrics.gutter = Some(40);
    t.metrics.grid = Some(16);
    let mut d = |ui: &mut Ui<()>, u: U| {
        panes::menu_column(ui, "m", u.n(20), u.n(20), u.n(160), u.n(20), u.n(24), &menu());
        panes::tab_rail(ui, "t", u.r(200, 20, 400, 100), &["One", "Two"], 0, |_| ());
        card(ui, "s", u.r(20, 130, 400, 90), None, &[], "A stamp", false, 1);
        ask(ui, u.r(20, 230, 500, 130), &AskView::new("", "Is this the question?"));
        pips::samples()[0].1.draw(ui, "p", u.n(450), u.n(150));
        tl(ui, &timeline::sample(), u.r(430, 160, 200, 60));
    };
    let pic = |t: &Theme, grid: Grid, d: &mut dyn FnMut(&mut Ui<()>, U)| {
        theme::set_theme(t.clone());
        shot(1280, 720, grid, 1.0, (-1, -1), &fonts, &|_, _| {}, d).0
    };
    let base = Theme::default();
    assert!(pic(&base, Grid::Old, &mut d).px == pic(&t, Grid::Old, &mut d).px, "an old screen does not read `metrics`");
    assert!(pic(&base, Grid::Native, &mut d).px != pic(&t, Grid::Native, &mut d).px, "a native one does");
    theme::set_theme(Theme::default());
}

/// Each pane's frames go through `Ui::skin` or `Ui::frame_body`: under `Ui::corner_size` its picture changes and is the one a
/// frame drawn directly at that size gives (the review's finding: the panes drew past it).
#[test]
fn the_panes_draw_their_frames_at_the_one_corner_size() {
    let fonts = font();
    theme::set_theme(framed_theme());
    for grid in [Grid::Old, Grid::Native] {
        let pic = |corner: Option<f32>, draw: &dyn Fn(&mut Ui<()>, U)| {
            let u = U(grid.g());
            let mut g = gfx(1920, 1080, grid, 1.0, &fonts);
            {
                let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
                if let Some(c) = corner {
                    ui.corner_size(c);
                }
                draw(&mut ui, u);
            }
            g.canvas.px
        };
        let entries: [(&str, &dyn Fn(&mut Ui<()>, U)); 3] = [
            ("menu button", &|ui, u| { panes::menu_column(ui, "m", u.n(20), u.n(20), u.n(160), u.n(30), u.n(34), &menu()); }),
            ("slot card", &|ui, u| card(ui, "s", u.r(20, 20, 400, 90), None, &[], "", false, 0)),
            ("ask", &|ui, u| ask(ui, u.r(0, 0, 640, 360), &AskView { ornate: true, ..AskView::new("A title", "Is this the question?") })),
        ];
        for (name, draw) in entries {
            assert!(pic(None, draw) != pic(Some(0.5), draw), "{name} on {grid:?}: it follows `corner_size`");
        }
        // The frame alone, at the size, under the card's first text: the pane's corners are those.
        let framed = pic(Some(0.5), &|ui, u| card(ui, "s", u.r(20, 20, 400, 90), None, &[], "", false, 0));
        let direct = pic(Some(0.5), &|ui, u| {
            let t = ui.t.clone();
            let m = ui.m();
            ui.skin("card", u.r(20, 20, 400, 90), t.card, t.edge, m.radius);
        });
        let w = 1920usize;
        // 1920 wide: 3 physical px per old unit on either grid.
        let s = 3usize;
        let (x0, y0) = (20 * s, 20 * s);
        for y in 0..(2 * s) {
            let (a, b) = (&framed[(y0 + y) * w * 4 + x0 * 4..(y0 + y) * w * 4 + (x0 + 8 * s) * 4], &direct[(y0 + y) * w * 4 + x0 * 4..(y0 + y) * w * 4 + (x0 + 8 * s) * 4]);
            assert_eq!(a, b, "{grid:?}: the top left corner at row {y}");
        }
    }
    theme::set_theme(Theme::default());
}

/// A dialog's text and answers stay in the frame's body at the size its corners draw: past a corner that reaches further under
/// `corner_size`, and in line with what `frame_body` says.
#[test]
fn an_ask_lays_out_in_the_body_of_the_corners_it_draws() {
    let fonts = font();
    let mut t = framed_theme();
    t.frames.get_mut("ornate").unwrap().nine.pad = [2; 4];
    theme::set_theme(t);
    for grid in [Grid::Old, Grid::Native] {
        let u = U(grid.g());
        let area = u.r(0, 0, 640, 360);
        let v = AskView { ornate: true, ..AskView::new("", "Is this the question?") };
        let answers = |corner: Option<f32>| {
            let mut g = gfx(1920, 1080, grid, 1.0, &fonts);
            let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            if let Some(c) = corner {
                ui.corner_size(c);
            }
            let r = panes::ask_dialog(&mut ui, "ask", area, &v, (), ());
            let by_id = |id: &str| ui.widgets.iter().find(|w| w.id == id).unwrap().rect;
            let (yes, no) = (by_id("ask.yes"), by_id("ask.no"));
            let body = ui.frame_body("ornate", r);
            (yes, no, body, ui.m())
        };
        let (_, _, base_body, _) = answers(None);
        let (yes, no, body, m) = answers(Some(3.0));
        assert!(body.w < base_body.w, "{grid:?}: larger corners take more of the frame ({body:?} against {base_body:?})");
        let inner = body.inset(m.padding / 2);
        assert_eq!((no.x + no.w, no.y + no.h), (inner.x + inner.w, inner.y + inner.h), "{grid:?}: No sits in the inner corner of the body it drew");
        assert_eq!(yes.y, no.y);
        assert_eq!(yes.x + yes.w + m.gutter, no.x, "{grid:?}: Yes left of No, one gutter apart");
    }
    theme::set_theme(Theme::default());
}

// ---- The desk ---------------------------------------------------------------------------------------------------------

/// The settings hold fractions: what one grid saved, another reads back to the same place, in its own units.
#[test]
fn a_saved_desk_means_the_same_on_every_grid() {
    let (old, native) = (sample_desk(U(1)), sample_desk(U(2)));
    assert_eq!(old.to_value(), native.to_value(), "a fresh desk saves the same on either");
    let mut moved_old = sample_desk(U(1));
    moved_old.drag("a", Grab::Move, 13, 21);
    moved_old.drag("a", Grab::Resize, -30, -40);
    let saved = moved_old.to_value();
    assert_eq!(saved["wins"].as_array().unwrap().len(), 3);
    let mut back = sample_desk(U(2));
    back.restore(&saved);
    let was = moved_old.rect("a").unwrap();
    let now = back.rect("a").unwrap();
    for (e, f) in [(was.x, now.x), (was.y, now.y), (was.x + was.w, now.x + now.w), (was.y + was.h, now.y + now.h)] {
        assert!((2 * e - f).abs() <= 1, "{was:?} against {now:?}");
    }
    let frac = |d: &Desk| d.win("a").unwrap().frac;
    assert!((frac(&back).x - frac(&moved_old).x).abs() < 0.01 && (frac(&back).w - frac(&moved_old).w).abs() < 0.01, "the fractions come back");
    // And the way home: a native desk's save reads back on the old grid, to the same fractions.
    let mut home = sample_desk(U(1));
    home.restore(&back.to_value());
    assert!((frac(&home).x - frac(&moved_old).x).abs() < 0.01);
    assert_eq!(saved["layout"], home.to_value()["layout"]);
}

/// A window's least size, its title bar and body follow the grid; a drag is in its units.
#[test]
fn a_desk_on_a_grid_takes_its_sizes_from_it() {
    let (mut old, mut native) = (sample_desk(U(1)), sample_desk(U(2)));
    for d in [&mut old, &mut native] {
        d.drag("a", Grab::Resize, -9000, -9000);
    }
    let (a, b) = (old.rect("a").unwrap(), native.rect("a").unwrap());
    assert_eq!((a.w, a.h), (60, 40), "the least size");
    assert_eq!((b.w, b.h), (120, 80), "the least size, in old units on any grid");
    let (ba, bb) = (old.body("a").unwrap(), native.body("a").unwrap());
    assert_eq!(bb, Rect::new(2 * ba.x, 2 * ba.y, 2 * ba.w, 2 * ba.h));
    old.collapse("a");
    native.collapse("a");
    assert_eq!(native.rect("a").unwrap().h, 2 * old.rect("a").unwrap().h);
    assert_eq!(native.rect("a").unwrap().h, 2 * aegis_ui::window::TITLE_H);
}

/// The toolbar's buttons follow on from a measured width each, so their rounding adds up along the row: its relations hold,
/// in units of the grid.
#[test]
fn a_desk_toolbar_keeps_its_relations_on_the_native_grid() {
    let fonts = font();
    theme::set_theme(Theme::default());
    for grid in [Grid::Old, Grid::Native] {
        let u = U(grid.g());
        let mut g = gfx(1920, 1080, grid, 1.0, &fonts);
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        let end = sample_desk(u).toolbar(&mut ui, u.n(10), u.n(10));
        let rects: Vec<Rect> = ui.widgets.iter().map(|w| w.rect).collect();
        assert_eq!(rects.len(), 5, "three windows and two layouts");
        for (i, pair) in rects.windows(2).enumerate() {
            let gap = if i == 2 { u.n(2 + 6) } else { u.n(2) };
            assert_eq!(pair[0].x + pair[0].w + gap, pair[1].x, "{grid:?}: button {i}");
        }
        assert!(rects.iter().all(|r| r.y == u.n(10) && r.h == u.n(12)));
        let last = rects[4];
        assert_eq!(end, last.x + last.w + u.n(2), "the next x");
    }
}

/// A rail's rows on any canvas: a relation, not a number. Each row is inside the rail, one step below the last, and the content
/// starts one gutter past it.
#[test]
fn a_tab_rail_keeps_its_relations_on_the_native_grid() {
    let m = Metrics::of(&Theme::default(), View::fit(1920, 1080).for_grid(Grid::Native));
    let r = Rect::new(40, 30, 900, 300);
    let (rows, content) = panes::tab_rail_rects_on(&m, r, 40);
    assert_eq!(rows.len() as i32, (r.h + m.u(panes::TAB_STEP - panes::TAB_H)) / m.u(panes::TAB_STEP), "as many as fit, none past the rect");
    let mut stack = Stack::new(r.pad(Insets { l: 0, t: 0, r: r.w - rows[0].w, b: 0 }), m.u(panes::TAB_STEP - panes::TAB_H));
    for row in &rows {
        assert_eq!(*row, stack.next(m.tab_h));
        assert!(row.y + row.h <= r.y + r.h && row.x == r.x);
    }
    assert_eq!(content.x, rows[0].x + rows[0].w + m.gutter);
    assert_eq!(content.x + content.w, r.x + r.w, "the content ends on the rect's edge");
}
