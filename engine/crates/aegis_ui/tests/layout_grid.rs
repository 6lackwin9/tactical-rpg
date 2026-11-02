//! The grid under each screen (LAYOUT720-E1, docs/specs/LAYOUT720.md 5 and 7.1): `Grid::Old` is exactly today's view; a
//! native screen with every number doubled draws today's picture to the pixel; a quadrant screen draws today's numbers at
//! half size in the top-left. The shell's half (per-screen view, mouse, hit tests, agent link) is tested in `shell.rs`,
//! as `Shell` is private to the crate. T1 is the pinned hashes of `theme.rs` and `theme_panes.rs`.

use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::painter::{Fonts, Grid, Painter, View, GRID};
use aegis_ui::preview;
use aegis_ui::screen::WINDOW_SIZES;
use aegis_ui::theme::{Frame, Nine, NineFill};
use aegis_ui::ui::{Gfx, Kit};
use std::path::Path;
use std::sync::Arc;

const BG: Rgba = [9, 18, 27, 255];
const COL: Rgba = [230, 180, 60, 255];
const EDGE: Rgba = [40, 200, 220, 255];

fn fonts() -> Fonts {
    let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    Fonts::new(&font)
}

/// Every offered window size, and two that letterbox (a view with offsets).
fn sizes() -> Vec<(u32, u32)> {
    let mut all: Vec<(u32, u32)> = WINDOW_SIZES.iter().map(|s| {
        let (w, h) = s.split_once('x').unwrap();
        (w.parse().unwrap(), h.parse().unwrap())
    }).collect();
    all.extend([(3440, 1440), (1280, 800)]);
    all
}

/// Art where every pixel differs from its neighbours, so a sampling or slice that moves shows.
fn art(w: i32, h: i32) -> Image {
    let px = (0..w * h).flat_map(|i| {
        let (x, y) = (i % w, i / w);
        [(x * 7 % 251) as u8, (y * 11 % 241) as u8, ((x ^ y) * 13 % 239) as u8, if (x + y) % 5 == 0 { 120 } else { 255 }]
    }).collect();
    Image { w, h, px }
}

fn frame(nine: Nine, w: i32, h: i32) -> Frame {
    Frame { img: Arc::new(art(w, h)), nine, source: String::new() }
}

/// The frames as sidecars write them: `scale` 2 stretched and tiled, averaged at `scale` 8, a straddling one with its
/// content past the corners. All in art px and old units, whatever the grid.
fn frames() -> Vec<Frame> {
    vec![
        frame(Nine { center: NineFill::None, ..Nine::new([5; 4]) }, 40, 40),
        frame(Nine { edges: NineFill::Tile, center: NineFill::Tile, ..Nine::new([8; 4]) }, 48, 48),
        frame(Nine { scale: 8, reduce: true, ..Nine::new([40; 4]) }, 200, 200),
        frame(Nine { scale: 4, straddle: true, corner_anchor: Some([8, 8]), corner_inner: Some([20, 20]), center: NineFill::None, ..Nine::new([32; 4]) }, 96, 96),
        frame(Nine { scale: 4, straddle: true, edges: NineFill::Tile, corner_anchor: Some([8, 8]), line_width: None, ..Nine::new([32; 4]) }, 96, 96),
    ]
}

/// One of everything the painter draws, every position and size in old units times `u`.
fn scene(p: &mut Painter, u: i32, frames: &[Frame]) {
    let f = u as f32;
    let r = |x: i32, y: i32, w: i32, h: i32| Rect::new(x * u, y * u, w * u, h * u);
    p.fill(r(5, 5, 60, 30), COL);
    p.panel(r(70, 5, 61, 31), [20, 40, 60, 255], EDGE, 3.0 * f);
    p.image(&art(33, 21), r(140, 5, 33, 21));
    p.image_alpha(&art(33, 21), r(180, 5, 17, 11), 128);
    p.line(210.0 * f, 5.0 * f, 280.0 * f, 40.0 * f, COL, 0.0);
    p.line(290.0 * f, 5.0 * f, 290.0 * f, 40.0 * f, EDGE, 3.0 * f);
    p.arrow(300.0 * f, 5.0 * f, 360.0 * f, 35.0 * f, COL, 2.0 * f, 5.0 * f);
    p.ellipse(380.0 * f, 20.0 * f, 13.0 * f, 7.0 * f, EDGE);
    p.ellipse_ring(420.0 * f, 20.0 * f, 13.0 * f, 7.0 * f, COL);
    p.circle(460.0 * f, 20.0 * f, 11.0 * f, EDGE);
    p.text(5.0 * f, 45.0 * f, "Plain text at the body size", 7.0 * f, COL);
    p.text_center(200.0 * f, 45.0 * f, "Centred", 12.0 * f, EDGE);
    p.text_right(400.0 * f, 45.0 * f, "To the right", 7.0 * f, COL);
    p.text_fit(420.0 * f, 45.0 * f, "A line too long for its box is shortened", 7.0 * f, 60.0 * f, COL);
    p.rich(5.0 * f, 62.0 * f, 160.0 * f, "[b]Bold[/b] and [color=#ff8040]coloured[/color] words that wrap onto a second line", 7.0 * f, COL);
    let at = [r(5, 90, 90, 61), r(105, 90, 121, 80), r(235, 90, 150, 110), r(400, 110, 101, 70), r(520, 110, 90, 70)];
    for (fr, at) in frames.iter().zip(at) {
        p.skin(Some(fr), at, [60, 30, 90, 255], EDGE, 3.0 * f);
    }
    p.nine(&frames[0].img, &frames[0].nine, r(5, 210, 9, 7));
    p.shift(2 * u, 3 * u);
    p.fill(r(30, 210, 11, 9), EDGE);
    p.shift(-2 * u, -3 * u);
}

/// `scene` in numbers of `u` on a window of (`pw`, `ph`), under `view`.
fn drawn(pw: u32, ph: u32, view: View, u: i32, frames: &[Frame]) -> Canvas {
    let mut c = Canvas::new(pw as i32, ph as i32);
    c.clear(BG);
    let mut f = fonts();
    let mut p = Painter { c: &mut c, v: view, fonts: &mut f };
    scene(&mut p, u, frames);
    c
}

#[test]
fn the_old_grid_is_todays_view_to_the_bit() {
    for (w, h) in sizes() {
        let (fit, old) = (View::fit(w, h), View::fit(w, h).for_grid(Grid::Old));
        assert_eq!((old.s.to_bits(), old.ox.to_bits(), old.oy.to_bits(), old.g), (fit.s.to_bits(), fit.ox.to_bits(), fit.oy.to_bits(), 1), "{w}x{h}");
        assert_eq!(old.canvas(), (640, 360));
    }
    assert_eq!(Grid::default(), Grid::Old, "unmoved screens stay as they are");
}

/// T2: a doubled rect lands on the same pixels, exactly, at every size.
#[test]
fn a_doubled_rect_on_the_native_grid_is_the_old_rect_exactly() {
    for (w, h) in sizes() {
        let (old, native) = (View::fit(w, h), View::fit(w, h).for_grid(Grid::Native));
        assert_eq!((native.ox, native.oy), (old.ox, old.oy), "the letterbox stays");
        for x in (-20..700).step_by(7) {
            for y in (-20..400).step_by(11) {
                let r = Rect::new(x, y, x.abs() % 97 + 1, y.abs() % 53 + 1);
                let doubled = Rect::new(2 * r.x, 2 * r.y, 2 * r.w, 2 * r.h);
                assert_eq!(native.rect(doubled), old.rect(r), "{w}x{h} {r:?}");
            }
        }
    }
}

#[test]
fn the_canvas_and_the_mouse_come_in_the_screens_units() {
    assert_eq!(View::fit(1280, 720).for_grid(Grid::Native).canvas(), GRID);
    assert_eq!(View::fit(1280, 720).for_grid(Grid::Quadrant).canvas(), (640, 360), "old numbers");
    assert_eq!((Grid::Old.extent(), Grid::Quadrant.extent(), Grid::Native.extent()), ((640, 360), GRID, GRID));
    for (w, h) in sizes() {
        let (old, native, quadrant) = (View::fit(w, h), View::fit(w, h).for_grid(Grid::Native), View::fit(w, h).for_grid(Grid::Quadrant));
        for (x, y) in [(0, 0), (1, 1), (333, 77), (639, 359)] {
            let at = old.to_physical(x, y);
            assert_eq!(old.to_logical(at.0, at.1), (x, y), "{w}x{h}: the middle of a point reads back as it");
            let (nx, ny) = native.to_logical(at.0, at.1);
            assert_eq!((nx / 2, ny / 2), (x, y), "{w}x{h}: one old unit is two native ones");
            assert_eq!(quadrant.to_logical(at.0, at.1), (nx, ny), "{w}x{h}: both halve the scale");
        }
    }
}

/// Mutation check for each painter fix: a stroke or a frame sized by `s` alone shows here at the sizes where `s` matters.
#[test]
fn every_painter_call_draws_the_same_pixels_native_and_old() {
    let frames = frames();
    for (w, h) in sizes() {
        let old = drawn(w, h, View::fit(w, h), 1, &frames);
        let native = drawn(w, h, View::fit(w, h).for_grid(Grid::Native), 2, &frames);
        let at: Vec<(usize, usize)> = old.px.chunks(4).zip(native.px.chunks(4)).enumerate().filter(|(_, (a, b))| a != b).map(|(i, _)| (i % w as usize, i / w as usize)).collect();
        assert!(at.is_empty(), "{w}x{h}: {} pixels differ, x {:?} y {:?}", at.len(), at.iter().map(|p| p.0).min().zip(at.iter().map(|p| p.0).max()), at.iter().map(|p| p.1).min().zip(at.iter().map(|p| p.1).max()));
    }
}

/// The thin strokes themselves, in numbers, at 1080p where the spec says they would thin from 2 px to 1.
#[test]
fn a_native_line_keeps_its_thickness_at_1080p() {
    let view = View::fit(1920, 1080).for_grid(Grid::Native);
    assert_eq!(view.old_scale(), 3.0);
    let mut c = Canvas::new(1920, 1080);
    let mut f = fonts();
    Painter { c: &mut c, v: view, fonts: &mut f }.line(100.0, 100.0, 100.0, 200.0, COL, 0.0);
    let row = 225 * 1920 * 4;
    let lit = (0..1920).filter(|x| c.px[row + x * 4 + 3] != 0).count();
    assert_eq!(lit, (3.0f32 * 0.6).round() as usize, "as thick as the same line on the old grid");
}

/// Quadrant on a 1440p window is, in its quarter, today's 720p picture; the other three quarters stay blank. The frames'
/// sidecars do not change: their corners halve with the scale.
#[test]
fn a_quadrant_screen_is_the_old_picture_in_the_top_left_quarter() {
    let frames = frames();
    let old = drawn(1280, 720, View::fit(1280, 720), 1, &frames);
    let quadrant = drawn(2560, 1440, View::fit(2560, 1440).for_grid(Grid::Quadrant), 1, &frames);
    for y in 0..720usize {
        let (a, b) = (y * 1280 * 4, y * 2560 * 4);
        assert!(old.px[a..a + 1280 * 4] == quadrant.px[b..b + 1280 * 4], "row {y} of the quarter");
    }
    for (i, px) in quadrant.px.chunks(4).enumerate() {
        let (x, y) = (i % 2560, i / 2560);
        if x >= 1280 || y >= 720 {
            assert_eq!(px, BG, "({x}, {y}) past the quarter");
        }
    }
}

#[test]
fn a_frames_content_rect_scales_with_the_grid_and_never_with_the_sidecar() {
    for fr in frames() {
        for (w, h) in sizes() {
            let (old, native) = (View::fit(w, h), View::fit(w, h).for_grid(Grid::Native));
            for r in [Rect::new(10, 20, 100, 60), Rect::new(0, 0, 8, 8), Rect::new(33, 7, 201, 133)] {
                let c = old.nine_content(&fr.nine, r);
                assert_eq!(c, aegis_ui::painter::nine_content_rect(&fr.nine, r, old.s), "the old view: as before");
                let d = native.nine_content(&fr.nine, Rect::new(2 * r.x, 2 * r.y, 2 * r.w, 2 * r.h));
                assert_eq!(d, Rect::new(2 * c.x, 2 * c.y, 2 * c.w, 2 * c.h), "{w}x{h} {r:?} {:?}", fr.nine.slice);
            }
        }
    }
}

fn gfx(pw: u32, ph: u32, view: View) -> Gfx {
    let mut canvas = Canvas::new(pw as i32, ph as i32);
    canvas.clear(BG);
    Gfx { canvas, view, fonts: fonts(), kit: Kit::default() }
}

fn preview_scene(g: &mut Gfx, u: i32, frames: &[Frame]) {
    let mut p = Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
    scene(&mut p, u, frames);
}

/// A preview shows a screen as its grid draws it: a native one as the same picture as its old form; a quadrant one in
/// the top-left of the box; and any of them inside an outer screen on any grid, the box being in the outer's units.
#[test]
fn a_preview_takes_the_shown_screens_grid() {
    let frames = frames();
    let box_old = Rect::new(40, 30, 320, 180);
    let whole = View::fit(1280, 720);
    let mut old = gfx(1280, 720, whole);
    preview::screen_in(&mut old, box_old, Grid::Old, |g| preview_scene(g, 1, &frames));
    assert_eq!(old.view, whole, "the view comes back");
    let mut native = gfx(1280, 720, whole);
    preview::screen_in(&mut native, box_old, Grid::Native, |g| {
        assert_eq!(g.view.g, 2, "the shown screen's grid, not the outer's");
        preview_scene(g, 2, &frames)
    });
    assert!(old.canvas.px == native.canvas.px, "a native screen's preview is its old form's picture");

    // From an outer screen on the native grid: the same box, in its units.
    let mut outer_native = gfx(1280, 720, whole.for_grid(Grid::Native));
    let box_native = Rect::new(80, 60, 640, 360);
    preview::screen_in(&mut outer_native, box_native, Grid::Old, |g| {
        assert_eq!(g.view.g, 1);
        preview_scene(g, 1, &frames)
    });
    assert!(old.canvas.px == outer_native.canvas.px, "the box is in the outer's units");

    // A quadrant screen: the same numbers in the top-left of the box, half the size, within the box.
    let mut quadrant = gfx(1280, 720, whole);
    preview::screen_in(&mut quadrant, box_old, Grid::Quadrant, |g| {
        let v = g.view;
        assert_eq!((v.g, v.x(0.0), v.y(0.0)), (1, whole.x(40.0), whole.y(30.0)));
        assert_eq!((v.x(640.0), v.y(360.0)), (whole.x(40.0 + 160.0), whole.y(30.0 + 90.0)), "a quarter of the box");
        preview_scene(g, 1, &frames)
    });
    let lit = |c: &Canvas, x: usize, y: usize| c.px[(y * 1280 + x) * 4..(y * 1280 + x) * 4 + 4] != BG;
    let inside = (0..720).any(|y| (0..1280).any(|x| x < 400 && y < 240 && lit(&quadrant.canvas, x, y)));
    let beyond = (0..720).any(|y| (0..1280).any(|x| (x >= 400 || y >= 240) && lit(&quadrant.canvas, x, y)));
    assert!(inside && !beyond, "only inside the quarter of the box: {inside} {beyond}");
}
