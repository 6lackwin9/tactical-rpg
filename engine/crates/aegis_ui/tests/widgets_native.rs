//! LAYOUT720-E3, T3 (docs/specs/LAYOUT720.md section 5): every kit widget drawn on the native grid with its numbers doubled
//! is its old picture. Most are the same to the bit (`Same`), since a widget's inside is its own rect's numbers times the
//! unit, and text and frames scale exactly. Where a whole-number division or a rounding up falls on a different unit
//! (`Near`, spec 6.3) `shot_diff` allows that one unit. The recorded rects of the old form are checked too: twice as much,
//! each edge within one unit. Done with no theme and with frames, at 1280 x 720 and 1920 x 1080 and at 1366 x 768, where
//! the scale is not whole. T1 (`theme.rs`, `theme_panes.rs`) holds the old pictures to their recorded hashes.
//!
//! `AEGIS_SHOT_DIR=<folder>`: writes each size's old and native boards there as PNGs, to look at.

use aegis_ui::canvas::{rgb, Canvas, Image, Rect, Rgba};
use aegis_ui::painter::{Fonts, Grid, View};
use aegis_ui::shot_diff::shot_diff;
use aegis_ui::theme::{self, Frame, Nine, NineFill, Theme};
use aegis_ui::ui::{draw_tip, tip, Edit, Gfx, Glyph, Kit, Ui, TEXT};
use std::path::Path;
use std::sync::Arc;

const GROUND: Rgba = [9, 18, 27, 255];
const WINDOWS: [(i32, i32); 3] = [(1280, 720), (1920, 1080), (1366, 768)];
const TEXT_SIZES: [f32; 2] = [1.0, 1.3];

#[derive(Clone, Copy, PartialEq)]
enum Match {
    /// To the bit: nothing in the widget rounds on the grid.
    Same,
    /// Within a unit (`shot_diff`), with the reason in the table.
    Near,
    /// Within a unit at a whole scale. At another, a curve moved by that unit falls differently on the pixels, so only its
    /// rects are held.
    Whole,
}
use Match::{Near, Same, Whole};

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
    fn p(self, x: i32, y: i32) -> (i32, i32) {
        (x * self.0, y * self.0)
    }
}

type Entry = (&'static str, Match, (i32, i32), fn(&mut Ui<()>, U));

fn lines() -> Vec<(String, Rgba)> {
    vec![("A first line of a card".into(), TEXT), ("Then a second one, too long to fit in this card".into(), rgb(240, 100, 92)), ("A third".into(), TEXT), ("Cut off by the card's foot".into(), TEXT)]
}

fn icon() -> Image {
    let px = (0..16 * 16).flat_map(|i| [(i * 7 % 251) as u8, (i * 3 % 241) as u8, 90, 255]).collect();
    Image { w: 16, h: 16, px }
}

const LONG: &str = "One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen";

/// One entry per widget and state: its name, how it matches, the mouse in old units, and the drawing. Rects are all even
/// or odd on purpose, so a centring division shows.
fn table() -> Vec<Entry> {
    vec![
        ("header", Same, (-1, -1), |ui, u| { ui.header(u.n(20), u.n(20), "A header"); }),
        // The y below a header is a text height rounded up, in units.
        ("header below", Near, (-1, -1), |ui, u| { let y = ui.header(u.n(20), u.n(20), "A header"); ui.header(u.n(20), y, "Below it"); }),
        ("badge", Same, (-1, -1), |ui, u| { ui.badge(u.r(20, 20, 24, 12), "42", "card"); ui.badge(u.r(60, 20, 25, 13), "7", "gold"); }),
        ("inset", Same, (-1, -1), |ui, u| { let c = ui.inset(u.r(20, 20, 120, 60), false); ui.p.fill(c, [40, 80, 40, 255]); let d = ui.inset(u.r(160, 20, 120, 61), true); ui.p.fill(d, [80, 40, 40, 255]); }),
        ("frame_body", Same, (-1, -1), |ui, u| { for (i, role) in ["panel", "card", "button", "tip", "tab_on", "ornate"].iter().enumerate() { let c = ui.frame_body(role, u.r(20 + 100 * i as i32, 20, 90, 70)); ui.p.fill(c, [40, 80, 40, 120]); } }),
        ("frame", Same, (-1, -1), |ui, u| { let a = ui.frame(u.r(20, 20, 180, 100), "A title", false); ui.p.fill(Rect::new(u.n(30), a, u.n(10), u.n(4)), TEXT); let b = ui.frame(u.r(220, 20, 181, 101), "Centred", true); ui.p.fill(Rect::new(u.n(230), b, u.n(10), u.n(4)), TEXT); }),
        ("button", Same, (-1, -1), |ui, u| { ui.button("a", u.r(20, 20, 80, 16), "Press me", (), true); ui.button("b", u.r(110, 20, 80, 16), "Disabled", (), false); ui.button("c", u.r(200, 20, 19, 16), "Narrow", (), true); ui.button("d", u.r(230, 20, 50, 17), "A label much too long", (), true); ui.button_full("e", u.r(300, 20, 80, 16), "Active", (), true, true, ui.m().small, Vec::new()); }),
        ("button hovered", Same, (30, 25), |ui, u| { ui.button("a", u.r(20, 20, 80, 16), "Press me", (), true); }),
        ("tab", Same, (-1, -1), |ui, u| { ui.tab("a", u.r(20, 20, 60, 18), "Open", (), true); ui.tab("b", u.r(84, 20, 60, 18), "Shut", (), false); }),
        ("card", Same, (-1, -1), |ui, u| { ui.card("a", u.r(20, 20, 120, 34), &lines(), Some(()), false, true, Vec::new()); ui.card("b", u.r(150, 20, 120, 34), &lines(), Some(()), true, true, Vec::new()); ui.card("c", u.r(280, 20, 120, 34), &lines(), None, false, false, Vec::new()); }),
        ("card hovered", Same, (30, 30), |ui, u| { ui.card("a", u.r(20, 20, 120, 34), &lines(), Some(()), false, true, Vec::new()); }),
        ("checkbox", Same, (-1, -1), |ui, u| { ui.checkbox("a", u.n(20), u.n(20), "Off", false, (), true); ui.checkbox("b", u.n(80), u.n(20), "On", true, (), true); ui.checkbox("c", u.n(140), u.n(21), "Disabled", true, (), false); }),
        ("checkbox hovered", Same, (24, 24), |ui, u| { ui.checkbox("a", u.n(20), u.n(20), "A box", false, (), true); }),
        ("stepper", Same, (-1, -1), |ui, u| { ui.stepper("a", u.r(20, 20, 200, 14), 0.5, "50 %", (), (), true); ui.stepper("b", u.r(20, 40, 200, 14), 1.0, "All", (), (), false); }),
        // A value cuts the fill to whole units, and the knob sits on its end; an odd height centres the bar a unit away.
        ("stepper by value", Whole, (-1, -1), |ui, u| { ui.stepper("a", u.r(20, 20, 200, 14), 0.37, "37 %", (), (), true); ui.stepper("b", u.r(20, 40, 200, 15), 1.0, "All", (), (), false); }),
        ("slider", Same, (-1, -1), |ui, u| { ui.slider("a", u.r(20, 20, 200, 14), 3, 8, "Three", |_| (), true); ui.slider("b", u.r(20, 40, 200, 14), 0, 5, "None", |_| (), false); }),
        ("slider odd", Whole, (-1, -1), |ui, u| { ui.slider("b", u.r(20, 40, 200, 15), 2, 5, "Two", |_| (), true); }),
        ("slider hovered", Same, (60, 27), |ui, u| { ui.slider("a", u.r(20, 20, 200, 14), 3, 8, "Three", |_| (), true); }),
        ("dropdown", Same, (-1, -1), |ui, u| { ui.dropdown("a", u.r(20, 20, 120, 14), "A value", (), true, Vec::new()); ui.dropdown("b", u.r(150, 20, 60, 15), "A value too long for it", (), false, Vec::new()); }),
        ("select", Same, (-1, -1), |ui, u| { ui.select("a", u.r(20, 20, 120, 14), &["One".into(), "Two".into()], 1, |_| (), true, Vec::new()); }),
        ("close_x", Same, (-1, -1), |ui, u| { ui.close_x("a", u.r(20, 20, 13, 13), ()); ui.close_x("b", u.r(40, 20, 14, 14), ()); }),
        ("close_x hovered", Same, (24, 24), |ui, u| { ui.close_x("a", u.r(20, 20, 13, 13), ()); }),
        ("para", Same, (-1, -1), |ui, u| { let size = ui.m().body; let col = ui.t.text; ui.para(u.n(20), u.n(20), u.n(200), LONG, size, col); }),
        // The y below a paragraph is its height rounded up, in units.
        ("para below", Near, (-1, -1), |ui, u| { let size = ui.m().body; let col = ui.t.text; let y = ui.para(u.n(20), u.n(20), u.n(200), LONG, size, col); let small = ui.m().small; ui.para(u.n(20), y, u.n(200), LONG, small, col); }),
        ("row", Same, (-1, -1), |ui, u| { ui.row("a", u.r(20, 20, 120, 12), "Plain", TEXT, None, false, Vec::new()); ui.row("b", u.r(20, 34, 120, 13), "Selected", TEXT, Some(()), true, Vec::new()); ui.row("c", u.r(20, 50, 60, 12), "A row too long to show", TEXT, Some(()), false, Vec::new()); }),
        ("row hovered", Same, (30, 25), |ui, u| { ui.row("a", u.r(20, 20, 120, 12), "Clickable", TEXT, Some(()), false, Vec::new()); ui.row("b", u.r(20, 40, 120, 12), "Info", TEXT, None, false, tip("T", "B")); }),
        ("row outlined", Same, (30, 45), |ui, u| { ui.row("b", u.r(20, 40, 120, 12), "Info", TEXT, None, false, tip("T", "B")); }),
        ("icon_button image", Same, (-1, -1), |ui, u| { let i = icon(); ui.icon_button("a", u.r(20, 20, 16, 16), "An icon", Some(&i), Glyph::Lock, (), true, false, Vec::new()); ui.icon_button("b", u.r(40, 20, 16, 16), "Off", Some(&i), Glyph::Lock, (), false, true, Vec::new()); }),
        // Centring in an odd box: the division falls a unit nearer the true middle (spec 6.3).
        ("icon_button odd", Near, (-1, -1), |ui, u| { let i = icon(); ui.icon_button("b", u.r(40, 20, 17, 16), "Off", Some(&i), Glyph::Lock, (), false, true, Vec::new()); }),
        ("icon_button hovered", Same, (25, 25), |ui, u| { let i = icon(); ui.icon_button("a", u.r(20, 20, 16, 16), "An icon", Some(&i), Glyph::Lock, (), true, false, Vec::new()); }),
        ("glyphs", Same, (-1, -1), |ui, u| { for (i, g) in [Glyph::Lock, Glyph::Unlock, Glyph::Delete, Glyph::Help, Glyph::ChevronDown, Glyph::ChevronRight, Glyph::Menu, Glyph::Plus, Glyph::Close, Glyph::Letter('P')].into_iter().enumerate() { ui.icon_button(&format!("g{i}"), u.r(20 + 20 * i as i32, 20, 16, 16), "G", None, g, (), true, i % 2 == 0, Vec::new()); } }),
        ("section", Same, (-1, -1), |ui, u| { ui.section("a", u.r(20, 20, 160, 14), "Open", "", true, (), Vec::new()); ui.section("b", u.r(20, 40, 160, 14), "Shut with a note", "12 of 20", false, (), Vec::new()); ui.section("c", u.r(20, 60, 80, 14), "A title too long to show here", "note", false, (), Vec::new()); }),
        ("section odd", Near, (-1, -1), |ui, u| { ui.section("c", u.r(20, 60, 80, 15), "An odd height", "note", false, (), Vec::new()); }),
        ("section hovered", Same, (30, 25), |ui, u| { ui.section("a", u.r(20, 20, 160, 14), "Open", "", true, (), Vec::new()); }),
        ("text_field", Same, (-1, -1), |ui, u| { ui.text_field("a", u.r(20, 20, 120, 14), "One line of text", false, true); ui.text_field("b", u.r(150, 20, 60, 14), "A line longer than this field shows", false, false); }),
        // The scroll bar's knob is a division of the content's height, rounded up from measured lines.
        ("text_area", Near, (-1, -1), |ui, u| { ui.text_field("a", u.r(20, 20, 120, 30), LONG, true, true); }),
        ("text_field hovered", Same, (30, 25), |ui, u| { ui.text_field("a", u.r(20, 20, 120, 14), "One line of text", false, true); }),
        ("scroll", Near, (-1, -1), |ui, u| { let off = ui.scroll_begin("a", u.r(20, 20, 100, 60), u.n(200)); assert_eq!(off, 0); ui.p.fill(u.r(25, 25, 40, 40), TEXT); ui.scroll_end(); ui.scroll_begin("b", u.r(140, 20, 100, 61), u.n(1000)); ui.scroll_end(); ui.scroll_begin("c", u.r(260, 20, 100, 60), u.n(60)); ui.scroll_end(); }),
        ("clip", Same, (-1, -1), |ui, u| { ui.push_clip(u.r(20, 20, 40, 15)); ui.button("a", u.r(30, 25, 80, 16), "Clipped", (), true); ui.pop_clip(); }),
    ]
}

/// A hover card near each edge: it flips at the canvas's own edges, in its units.
fn tip_table() -> Vec<(&'static str, (i32, i32))> {
    vec![("middle", (300, 150)), ("past the old half, on the native grid", (500, 150)), ("right", (620, 150)), ("bottom", (300, 350)), ("corner", (630, 352)), ("top left", (0, 0))]
}

/// Frames as sidecars write them, in art px and old units: sampled at `scale` 2, tiled, averaged at `scale` 8, straddling.
fn framed_theme() -> Theme {
    let art = |w: i32, h: i32| {
        let px = (0..w * h).flat_map(|i| {
            let (x, y) = (i % w, i / w);
            [(x * 7 % 251) as u8, (y * 11 % 241) as u8, ((x ^ y) * 13 % 239) as u8, 255]
        }).collect();
        Arc::new(Image { w, h, px })
    };
    let frame = |nine: Nine, w: i32, h: i32| Frame { img: art(w, h), nine, source: String::new() };
    let mut t = Theme::default();
    t.layout.focus_glow = 1;
    t.layout.radius = 0.0;
    for role in ["button", "tab", "card", "inset"] {
        t.frames.insert(role.into(), frame(Nine::new([4; 4]), 24, 24));
    }
    // A card's width follows its text: a stretched centre of this noise under its letters would differ at every pixel.
    t.frames.insert("tip".into(), frame(Nine { center: NineFill::None, ..Nine::new([4; 4]) }, 24, 24));
    t.frames.insert("card_on".into(), frame(Nine { center: NineFill::None, ..Nine::new([5; 4]) }, 40, 40));
    t.frames.insert("button_hot".into(), frame(Nine { edges: NineFill::Tile, center: NineFill::Tile, ..Nine::new([8; 4]) }, 48, 48));
    t.frames.insert("panel".into(), frame(Nine { scale: 8, reduce: true, ..Nine::new([40; 4]) }, 200, 200));
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

/// `draw` in the numbers of `grid`: the picture and the recorded rects (id, rect).
fn shot(pw: i32, ph: i32, grid: Grid, scale: f32, mouse: (i32, i32), fonts: &[u8], draw: &mut dyn FnMut(&mut Ui<()>, U)) -> (Image, Vec<(String, Rect)>) {
    let u = U(grid.g());
    let mut g = gfx(pw, ph, grid, scale, fonts);
    let rects = {
        let mouse = if mouse.0 < 0 { mouse } else { u.p(mouse.0, mouse.1) };
        let mut ui: Ui<()> = Ui::new(&mut g, mouse, true);
        draw(&mut ui, u);
        ui.widgets.iter().map(|w| (w.id.clone(), w.rect)).collect()
    };
    (Image { w: g.canvas.w, h: g.canvas.h, px: g.canvas.px }, rects)
}

/// Each edge of a native rect is twice the old one's, within a unit.
fn assert_rects(name: &str, old: &[(String, Rect)], native: &[(String, Rect)]) {
    assert_eq!(old.len(), native.len(), "{name}: the same widgets");
    for ((id, a), (nid, b)) in old.iter().zip(native) {
        assert_eq!(id, nid, "{name}");
        let edges = |r: &Rect| [r.x, r.y, r.x + r.w, r.y + r.h];
        for (e, f) in edges(a).iter().zip(edges(b)) {
            assert!((2 * e - f).abs() <= 2, "{name} {id}: {a:?} against {b:?}");
        }
    }
}

/// What is wrong with the native picture of an entry, if anything.
fn mismatch(want: Match, old: &Image, native: &Image, s: f32) -> Option<String> {
    if old.px == native.px || (want == Whole && s.fract() != 0.0) {
        return None;
    }
    let diff = shot_diff(old, native, s).unwrap();
    if !diff.passed() {
        return Some(diff.to_string());
    }
    (want == Same).then(|| format!("within {} px, not to the bit as the table says", diff.radius))
}

fn run_table(label: &str) {
    let fonts = font();
    let shots_to = std::env::var("AEGIS_SHOT_DIR").ok();
    let mut wrong = Vec::new();
    for (pw, ph) in WINDOWS {
        let s = View::fit(pw as u32, ph as u32).for_grid(Grid::Native).s;
        for scale in TEXT_SIZES {
            let (mut old_board, mut native_board) = (Board::new(pw, ph), Board::new(pw, ph));
            for (name, want, mouse, draw) in table() {
                let mut d = |ui: &mut Ui<()>, u: U| draw(ui, u);
                let (old, old_rects) = shot(pw, ph, Grid::Old, scale, mouse, &fonts, &mut d);
                let (native, native_rects) = shot(pw, ph, Grid::Native, scale, mouse, &fonts, &mut d);
                assert!(old.px.chunks(4).any(|p| p != GROUND), "{name}: it drew");
                if let Some(why) = mismatch(want, &old, &native, s) {
                    wrong.push(format!("{label} {name} at {pw}x{ph}, text {scale}: {why}"));
                }
                assert_rects(name, &old_rects, &native_rects);
                if shots_to.is_some() {
                    old_board.add(&old);
                    native_board.add(&native);
                }
            }
            if let Some(dir) = &shots_to {
                save(Path::new(dir).join(format!("{label}_{pw}_text{scale}_old.png")), &old_board.img);
                save(Path::new(dir).join(format!("{label}_{pw}_text{scale}_native.png")), &native_board.img);
            }
        }
    }
    assert!(wrong.is_empty(), "{} entries:\n{}", wrong.len(), wrong.join("\n"));
}

/// The shots of a table packed in rows, each cut to what it drew, to look at.
struct Board {
    img: Image,
    x: i32,
    y: i32,
    row_h: i32,
}

impl Board {
    fn new(w: i32, h: i32) -> Self {
        Self { img: Image { w, h, px: GROUND.repeat((w * h) as usize) }, x: 4, y: 4, row_h: 0 }
    }

    fn add(&mut self, shot: &Image) {
        let at = |x: i32, y: i32| ((y * shot.w + x) * 4) as usize;
        let lit: Vec<(i32, i32)> = (0..shot.h).flat_map(|y| (0..shot.w).map(move |x| (x, y))).filter(|(x, y)| shot.px[at(*x, *y)..at(*x, *y) + 4] != GROUND).collect();
        let (x0, x1) = (lit.iter().map(|p| p.0).min().unwrap_or(0), lit.iter().map(|p| p.0).max().unwrap_or(0));
        let (y0, y1) = (lit.iter().map(|p| p.1).min().unwrap_or(0), lit.iter().map(|p| p.1).max().unwrap_or(0));
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        if self.x + w + 4 > self.img.w {
            self.x = 4;
            self.y += self.row_h + 6;
            self.row_h = 0;
        }
        if self.y + h > self.img.h {
            return;
        }
        for y in 0..h {
            let (from, to) = (at(x0, y0 + y), ((self.y + y) * self.img.w + self.x) as usize * 4);
            self.img.px[to..to + w as usize * 4].copy_from_slice(&shot.px[from..from + w as usize * 4]);
        }
        self.x += w + 6;
        self.row_h = self.row_h.max(h);
    }
}

fn save(path: std::path::PathBuf, img: &Image) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img.px).unwrap();
}

#[test]
fn every_widget_is_its_old_picture_on_the_native_grid_with_no_theme() {
    theme::set_theme(Theme::default());
    run_table("vector");
}

#[test]
fn every_widget_is_its_old_picture_on_the_native_grid_with_frames() {
    theme::set_theme(framed_theme());
    run_table("framed");
    theme::set_theme(Theme::default());
}

/// A hover card: half size or a flip past native x 640 would show at once (the review's finding). Its size is a measured
/// width rounded up in units, so its far edges may land a unit away; hence `shot_diff`. A flip to the left starts from a
/// measured width cut to whole units, so the whole card may sit a unit to the left, and the text's letters round afresh from
/// there: such a card is its old picture when the mouse is that unit to the right.
#[test]
fn a_tip_is_its_old_picture_on_the_native_grid_and_stays_on_the_canvas() {
    let fonts = font();
    let mut wrong = Vec::new();
    for th in [Theme::default(), framed_theme()] {
        theme::set_theme(th);
        for (pw, ph) in WINDOWS {
            let s = View::fit(pw as u32, ph as u32).for_grid(Grid::Native).s;
            for scale in TEXT_SIZES {
                for (name, at) in tip_table() {
                    let picture = |grid: Grid, nudge: i32| {
                        let mut draw = |ui: &mut Ui<()>, u: U| {
                            // The widget covers the canvas, so the card shows wherever the mouse is.
                            let screen = ui.screen();
                            ui.push("w", "row", "", screen, true, false, None, tip("A title for the card", LONG));
                            let widgets = std::mem::take(&mut ui.widgets);
                            draw_tip(&mut ui.p, &widgets, (u.n(at.0) + nudge, u.n(at.1)));
                        };
                        shot(pw, ph, grid, scale, (-1, -1), &fonts, &mut draw).0
                    };
                    let old = picture(Grid::Old, 0);
                    assert!(old.px.chunks(4).any(|p| p != GROUND), "{name}: it drew");
                    if ![0, 1].iter().any(|nudge| shot_diff(&old, &picture(Grid::Native, *nudge), s).unwrap().passed()) {
                        wrong.push(format!("{name} at {pw}x{ph}, text {scale}: {}", shot_diff(&old, &picture(Grid::Native, 0), s).unwrap()));
                    }
                }
            }
        }
    }
    theme::set_theme(Theme::default());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The list of an open select: it fits against the canvas of its own grid, so it opens above at the old foot and flows
/// into columns the same on either. On the native grid the numbers are simply twice.
#[test]
fn an_open_select_places_its_options_against_its_own_canvas() {
    let fonts = font();
    let options: Vec<String> = (0..30).map(|i| format!("Choice {i}")).collect();
    for (field, n) in [((20, 20, 120, 14), 4), ((20, 340, 120, 14), 4), ((500, 300, 130, 14), 30), ((20, 170, 120, 14), 30), ((20, 170, 120, 9), 30)] {
        let rects = |grid: Grid| {
            let u = U(grid.g());
            let mut g = gfx(1920, 1080, grid, 1.0, &fonts);
            g.kit.open = Some("s".into());
            let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            let (x, y, w, h) = field;
            ui.select("s", u.r(x, y, w, h), &options[..n], 0, |_| (), true, Vec::new());
            ui.widgets.iter().filter(|w| w.kind == "option").map(|w| w.rect).collect::<Vec<_>>()
        };
        let (old, native) = (rects(Grid::Old), rects(Grid::Native));
        assert_eq!(old.len(), n);
        for (a, b) in old.iter().zip(&native) {
            assert_eq!(Rect::new(2 * a.x, 2 * a.y, 2 * a.w, 2 * a.h), *b, "{field:?}");
            assert!(b.x >= 0 && b.y >= 0 && b.x + b.w <= 1280 && b.y + b.h <= 720, "{b:?} on the canvas");
        }
    }
}

/// A text field while it edits: the caret and the selection are in its units, and a selection past a line's end marks the
/// break. The caret is its old picture within a unit. The selection's rect rounds its start down and its width up, each to a
/// whole unit, so its far edge may land two units away, and the letters on that edge mix with another ground: its cases
/// draw no letters, and allow the two units.
#[test]
fn a_field_being_edited_is_its_old_picture_on_the_native_grid() {
    let fonts = font();
    for (pw, ph) in WINDOWS {
        let view = View::fit(pw as u32, ph as u32);
        for (multiline, caret, anchor) in [(false, 5, None), (false, 8, Some(2)), (false, 60, None), (true, 12, None), (true, 90, Some(10))] {
            let selecting = anchor.is_some();
            theme::set_theme(if selecting { Theme { text: [0, 0, 0, 0], ..Theme::default() } } else { Theme::default() });
            let s = if selecting { view.old_scale() } else { view.for_grid(Grid::Native).s };
            let picture = |grid: Grid| {
                let u = U(grid.g());
                let mut g = gfx(pw, ph, grid, 1.0, &fonts);
                let mut e = Edit::new("f", LONG, multiline);
                e.caret = caret;
                e.anchor = anchor;
                g.kit.edit = Some(e);
                let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
                ui.text_field("f", u.r(20, 20, 120, if multiline { 40 } else { 14 }), LONG, multiline, true);
                drop(ui);
                Image { w: g.canvas.w, h: g.canvas.h, px: g.canvas.px }
            };
            let (old, native) = (picture(Grid::Old), picture(Grid::Native));
            let diff = shot_diff(&old, &native, s).unwrap();
            assert!(diff.passed(), "{pw}x{ph} multiline {multiline} caret {caret}: {diff}");
        }
    }
    theme::set_theme(Theme::default());
}

/// The icon square and the size to ask the art for, in any units.
#[test]
fn an_icon_is_one_source_pixel_per_screen_pixel_on_every_grid() {
    let fonts = font();
    for (pw, ph) in WINDOWS {
        let asked = |grid: Grid| {
            let u = U(grid.g());
            let mut g = gfx(pw, ph, grid, 1.0, &fonts);
            let ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            ui.icon_px(u.r(0, 0, 16, 16))
        };
        assert_eq!(asked(Grid::Old), asked(Grid::Native), "{pw}x{ph}");
        assert_eq!(Ui::<()>::icon_rect_on(Rect::new(40, 40, 32, 32), 2), Rect::new(44, 44, 24, 24), "2 old units inside the smaller side");
        assert_eq!(Ui::<()>::icon_rect_on(Rect::new(20, 20, 16, 16), 1), Ui::<()>::icon_rect(Rect::new(20, 20, 16, 16)));
    }
}

/// A theme's `metrics` reaches the widgets on the native grid, and only there: the read path is one.
#[test]
fn a_metrics_key_changes_a_native_widget_and_no_old_one() {
    let fonts = font();
    let mut t = Theme::default();
    t.metrics.body = Some(20.0);
    t.metrics.radius = Some(9.0);
    let mut draw = |ui: &mut Ui<()>, u: U| {
        ui.button("a", u.r(20, 20, 80, 16), "Press me", (), true);
        ui.dropdown("b", u.r(120, 20, 80, 16), "A value", (), true, Vec::new());
        ui.text_field("c", u.r(220, 20, 80, 16), "Typed", false, true);
    };
    theme::set_theme(Theme::default());
    let (old_before, _) = shot(1280, 720, Grid::Old, 1.0, (-1, -1), &fonts, &mut draw);
    let (native_before, _) = shot(1280, 720, Grid::Native, 1.0, (-1, -1), &fonts, &mut draw);
    theme::set_theme(t);
    let (old_after, _) = shot(1280, 720, Grid::Old, 1.0, (-1, -1), &fonts, &mut draw);
    let (native_after, _) = shot(1280, 720, Grid::Native, 1.0, (-1, -1), &fonts, &mut draw);
    theme::set_theme(Theme::default());
    assert!(old_before.px == old_after.px, "an old screen does not read `metrics`");
    assert!(native_before.px != native_after.px, "a native one does");
}
