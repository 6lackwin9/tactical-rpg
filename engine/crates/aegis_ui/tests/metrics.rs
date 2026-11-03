//! LAYOUT720-E3: what a screen's layout reads (docs/specs/LAYOUT720.md 6.4, 9, 14): `Metrics` and the theme's `metrics` block,
//! `Env`, `Ui::screen`, `Ui::image_in`, `Ui::frame_body` and `Ui::corner_size`. Every expected value comes from the rules:
//! today's constants and `layout` on the old grids, those doubled on the native one unless a theme file names the key.

use aegis_core::assets::Layers;
use aegis_layout::{Anchor, SizeClass};
use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::focus;
use aegis_ui::painter::{Fonts, Grid, View, GRID};
use aegis_ui::theme::{self, Frame, Metrics, Nine, NineFill, Theme, BREAKPOINTS, CORNER_SIZE};
use aegis_ui::ui::{self, Gfx, Kit, Pointer, Ui};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fonts() -> Fonts {
    Fonts::new(&std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap())
}

fn gfx(pw: i32, ph: i32, grid: Grid) -> Gfx {
    Gfx { canvas: Canvas::new(pw, ph), view: View::fit(pw as u32, ph as u32).for_grid(grid), fonts: fonts(), kit: Kit::default() }
}

fn view(grid: Grid) -> View {
    View::fit(1280, 720).for_grid(grid)
}

/// A game layer with this theme file, loaded.
fn load(tag: &str, v: Value) -> Result<Theme, Vec<String>> {
    let dir = scratch(tag);
    std::fs::create_dir_all(dir.join("ui")).unwrap();
    std::fs::write(dir.join("ui/theme.json"), v.to_string()).unwrap();
    theme::load(&Layers::of(vec![dir]))
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aegis_metrics_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn error_of(tag: &str, metrics: Value) -> String {
    load(tag, json!({"version": 1, "metrics": metrics})).expect_err("refused").join("\n")
}

#[test]
fn the_old_grids_read_todays_constants_and_layout_whatever_the_theme_says() {
    let mut t = Theme::default();
    t.layout.padding = 9;
    t.layout.radius = 1.5;
    t.layout.focus_glow = 3;
    t.metrics = load("old", json!({"version": 1, "metrics": {"body": 20, "padding": 30, "radius": 9, "tab_h": 40, "focus_glow": 5}})).unwrap().metrics;
    for grid in [Grid::Old, Grid::Quadrant] {
        let m = Metrics::of(&t, view(grid));
        assert_eq!((m.body, m.small, m.title, m.head, m.line), (ui::BODY, ui::SMALL, ui::TITLE, ui::HEAD, ui::LINE), "{grid:?}");
        assert_eq!((m.radius, m.padding, m.focus_glow, m.frame_title_h), (1.5, 9, 3, t.layout.frame_title_h), "{grid:?}");
        assert_eq!((m.tab_h, m.g, m.u(7), m.uf(0.5)), (aegis_ui::panes::TAB_H, 1, 7, 0.5), "{grid:?}");
    }
}

#[test]
fn with_no_metrics_key_the_native_grid_is_todays_values_doubled() {
    let t = Theme::default();
    let (old, m) = (Metrics::of(&t, view(Grid::Old)), Metrics::of(&t, view(Grid::Native)));
    assert_eq!((m.body, m.small, m.title, m.head, m.line), (2.0 * old.body, 2.0 * old.small, 2.0 * old.title, 2.0 * old.head, 2.0 * old.line));
    assert_eq!((m.radius, m.frame_radius), (2.0 * old.radius, 2.0 * old.frame_radius));
    assert_eq!((m.margin, m.gutter, m.padding, m.grid), (2 * old.margin, 2 * old.gutter, 2 * old.padding, 2 * old.grid));
    assert_eq!((m.frame_title_h, m.tab_h, m.row_h, m.focus_glow), (2 * old.frame_title_h, 2 * old.tab_h, 2 * old.row_h, 2 * old.focus_glow));
    assert_eq!((m.g, m.u(7), m.uf(0.5)), (2, 14, 1.0));
    // A theme's `layout` carries over, so a moved screen keeps its picture under that theme.
    let shaped = load("shaped", json!({"version": 1, "layout": {"radius": 0, "focus_glow": 1, "padding": 7}})).unwrap();
    let s = Metrics::of(&shaped, view(Grid::Native));
    assert_eq!((s.radius, s.focus_glow, s.padding), (0.0, 2, 14));
}

#[test]
fn a_metrics_key_sets_the_native_grid_and_only_it() {
    let t = load("set", json!({"version": 1, "metrics": {"body": 16.5, "small": 13, "title": 17, "head": 26, "line": 21, "radius": 7, "margin": 20, "gutter": 14, "padding": 10, "grid": 4, "frame_title_h": 60, "tab_h": 40, "row_h": 30, "focus_glow": 3, "breakpoints": {"width": [600, 1000], "height": [300, 500]}}})).unwrap();
    let m = Metrics::of(&t, view(Grid::Native));
    assert_eq!((m.body, m.small, m.title, m.head, m.line, m.radius), (16.5, 13.0, 17.0, 26.0, 21.0, 7.0));
    assert_eq!((m.margin, m.gutter, m.padding, m.grid, m.frame_title_h, m.tab_h, m.row_h, m.focus_glow), (20, 14, 10, 4, 60, 40, 30, 3));
    assert_eq!((m.breakpoints.width, m.breakpoints.height), ([600, 1000], [300, 500]));
    assert_eq!(Metrics::of(&Theme::default(), view(Grid::Old)).padding, Metrics::of(&t, view(Grid::Old)).padding, "nothing on the old grid");
}

#[test]
fn layers_merge_metrics_key_by_key() {
    let dir = scratch("layers");
    let (game, kit) = (dir.join("game"), dir.join("kit"));
    for (root, v) in [(&kit, json!({"version": 1, "metrics": {"body": 15, "padding": 10, "breakpoints": {"width": [700, 1100]}}})), (&game, json!({"version": 1, "metrics": {"body": 16, "breakpoints": {"height": [400, 600]}}}))] {
        std::fs::create_dir_all(root.join("ui")).unwrap();
        std::fs::write(root.join("ui/theme.json"), v.to_string()).unwrap();
    }
    let t = theme::load(&Layers::of(vec![game, kit])).unwrap();
    let m = Metrics::of(&t, view(Grid::Native));
    assert_eq!((m.body, m.padding, m.gutter), (16.0, 10, 2 * Theme::default().layout.gutter), "the game's over the kit's, the rest kept");
    assert_eq!((m.breakpoints.width, m.breakpoints.height), ([700, 1100], [400, 600]));
}

#[test]
fn bad_metrics_are_refused_with_every_problem_named() {
    assert!(error_of("unknown", json!({"bodyy": 14})).contains("unknown metrics key 'bodyy'"));
    assert!(error_of("zero", json!({"body": 0})).contains("metrics.body must be a number 1-64"));
    assert!(error_of("big", json!({"head": 500})).contains("metrics.head must be a number 1-64"));
    assert!(error_of("text", json!({"line": "18"})).contains("metrics.line must be"));
    assert!(error_of("whole", json!({"margin": 1.5})).contains("metrics.margin must be a whole number 1-64"));
    assert!(error_of("nopad", json!({"padding": 0})).contains("metrics.padding must be a whole number"));
    assert!(error_of("glow", json!({"focus_glow": -1})).contains("metrics.focus_glow must be a whole number 0-6"));
    assert!(error_of("radius", json!({"radius": -1})).contains("metrics.radius must be a number 0-16"));
    assert!(load("zeroes", json!({"version": 1, "metrics": {"radius": 0, "focus_glow": 0}})).is_ok(), "0 is allowed where the spec says");
    assert!(error_of("bp", json!({"breakpoints": 5})).contains("metrics.breakpoints must be an object"));
    assert!(error_of("bpkey", json!({"breakpoints": {"depth": [1, 2]}})).contains("unknown metrics.breakpoints key 'depth'"));
    for bad in [json!([800]), json!([1200, 800]), json!([800, 800]), json!([0, 800]), json!([800, 1.5]), json!([800, 100001]), json!("x")] {
        assert!(error_of("axis", json!({"breakpoints": {"width": bad}})).contains("metrics.breakpoints.width must be [compact, medium]"), "{bad}");
    }
    let many = error_of("many", json!({"body": 0, "nope": 1, "breakpoints": {"height": []}}));
    assert_eq!(many.lines().count(), 3, "every problem: {many}");
}

#[test]
fn hair_is_one_physical_pixel_at_any_scale() {
    let t = Theme::default();
    let hair = |s: f32| Metrics::of(&t, View { s, ox: 0.0, oy: 0.0, g: 2, canvas: GRID }).hair;
    assert_eq!((hair(1.0), hair(1.5), hair(3.0)), (1, 1, 1));
    assert_eq!((hair(0.5), hair(0.4), hair(0.25)), (2, 3, 4));
    for s in [0.2f32, 0.3, 0.7, 0.99] {
        assert!(hair(s) as f32 * s >= 1.0, "{s}");
    }
}

#[test]
fn screen_and_env_come_from_the_view() {
    for (grid, canvas) in [(Grid::Old, (640, 360)), (Grid::Quadrant, (640, 360)), (Grid::Native, GRID)] {
        for (pw, ph) in [(1280, 720), (1920, 1080), (2000, 900)] {
            let mut g = gfx(pw, ph, grid);
            g.kit.device.dpr = 1.25;
            let ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            assert_eq!(ui.screen(), Rect::new(0, 0, canvas.0, canvas.1), "{grid:?}");
            let e = ui.env();
            assert_eq!((e.canvas, e.dpr, e.ui_scale, e.pointer), (canvas, 1.25, 1.0, Pointer::Mouse), "{grid:?}");
            assert_eq!((e.width_class, e.height_class), (SizeClass::Expanded, SizeClass::Expanded), "this canvas is Expanded on every grid");
        }
    }
}

/// The classes read the canvas against the limits, in grid units. A canvas other than the fixed one arrives with E5, so the
/// test moves the limits instead.
#[test]
fn size_classes_follow_the_breakpoints() {
    let (w, h) = GRID;
    let cases = [
        ([w + 1, w + 2], SizeClass::Compact),
        ([w, w + 1], SizeClass::Medium),
        ([w - 1, w], SizeClass::Expanded),
    ];
    for (limits, class) in cases {
        let mut t = Theme::default();
        t.metrics.breakpoints_width = Some(limits);
        t.metrics.breakpoints_height = Some([h - limits[0] + w, h - limits[1] + w]);
        theme::set_theme(t);
        for grid in [Grid::Old, Grid::Native] {
            let mut g = gfx(1280, 720, grid);
            let ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            assert_eq!(ui.env().width_class, class, "{limits:?} {grid:?}");
        }
    }
    theme::set_theme(Theme::default());
    assert_eq!((BREAKPOINTS.width, BREAKPOINTS.height), ([800, 1200], [480, 700]), "the spec's defaults");
}

/// `Rect::place` from the screen's rect: a dialog keeps its place in the canvas on each grid.
#[test]
fn a_screen_cut_from_its_rect_reads_the_same_share_on_the_native_grid() {
    let mut g = gfx(1920, 1080, Grid::Native);
    let ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
    let r = ui.screen().place(Anchor::Center, aegis_layout::frac(500), aegis_layout::frac(500));
    assert_eq!(r, Rect::new(GRID.0 / 4, GRID.1 / 4, GRID.0 / 2, GRID.1 / 2));
}

#[test]
fn image_in_draws_as_the_painter_does() {
    let img = Image { w: 2, h: 2, px: [[200, 10, 10, 255], [10, 200, 10, 255], [10, 10, 200, 255], [200, 200, 10, 255]].concat() };
    let at = Rect::new(40, 30, 64, 32);
    let mut a = gfx(1280, 720, Grid::Native);
    let mut b = gfx(1280, 720, Grid::Native);
    Ui::<()>::new(&mut a, (-1, -1), true).image_in(&img, at);
    Ui::<()>::new(&mut b, (-1, -1), true).p.image(&img, at);
    assert!(a.canvas.px == b.canvas.px);
    assert!(a.canvas.px.iter().any(|v| *v != 0), "it drew");
}

const RED: Rgba = [230, 30, 30, 255];
const CLEAR: Rgba = [0, 0, 0, 0];

/// A 96 px frame whose corners are one colour and the rest see-through: the drawn corner's size can be read off the canvas.
fn red_corners(straddle: bool) -> Frame {
    let (n, slice) = (96, 32);
    let px = (0..n * n).flat_map(|i| {
        let (x, y) = (i % n, i / n);
        let corner = (x < slice || x >= n - slice) && (y < slice || y >= n - slice);
        if corner { RED } else { CLEAR }
    }).collect();
    let nine = Nine { scale: 4, straddle, center: NineFill::None, reduce: !straddle, ..Nine::new([slice; 4]) };
    Frame { img: Arc::new(Image { w: n, h: n, px }), nine, source: String::new() }
}

fn with_frame(role: &str, f: Frame) {
    let mut t = Theme::default();
    t.frames.insert(role.into(), f);
    theme::set_theme(t);
}

/// The bounding box of the red pixels in physical `r`.
fn red_box(c: &Canvas, r: Rect) -> Option<(i32, i32)> {
    let mut seen: Option<(i32, i32, i32, i32)> = None;
    for y in r.y.max(0)..(r.y + r.h).min(c.h) {
        for x in r.x.max(0)..(r.x + r.w).min(c.w) {
            let i = ((y * c.w + x) * 4) as usize;
            if c.px[i] > 150 && c.px[i + 1] < 90 && c.px[i + 3] > 0 {
                let (a, b, d, e) = seen.unwrap_or((x, y, x, y));
                seen = Some((a.min(x), b.min(y), d.max(x), e.max(y)));
            }
        }
    }
    seen.map(|(a, b, d, e)| (d - a + 1, e - b + 1))
}

/// Panes of three sizes under one `corner_size`: the top-left corner has one drawn size on each, set by the call, and it
/// differs from the sidecar's. No pane falls below the chosen size.
#[test]
fn one_corner_size_holds_on_every_pane_of_a_screen() {
    for straddle in [true, false] {
        with_frame("panel", red_corners(straddle));
        // Far apart, so each pane's corner can be read alone.
        let panes = [Rect::new(100, 100, 700, 400), Rect::new(1000, 100, 220, 180), Rect::new(1000, 400, 150, 150)];
        let measure = |size: Option<f32>| -> Vec<(i32, i32)> {
            let mut g = gfx(1920, 1080, Grid::Native);
            let v = g.view;
            let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            if let Some(s) = size {
                ui.corner_size(s);
            }
            for r in panes {
                ui.frame_body("panel", r);
            }
            drop(ui);
            panes.iter().map(|r| {
                let p = v.rect(*r);
                // Around the top-left corner, up to the panel's middle.
                red_box(&g.canvas, Rect::new(p.x - 60, p.y - 60, p.w / 2 + 60, p.h / 2 + 60)).unwrap()
            }).collect()
        };
        let (plain, half) = (measure(None), measure(Some(0.5)));
        assert!(plain.iter().all(|c| *c == plain[0]), "straddle {straddle}: one size from the sidecar: {plain:?}");
        assert!(half.iter().all(|c| *c == half[0]), "straddle {straddle}: one size from the call: {half:?}");
        let (art_px, k) = (32.0, View::fit(1920, 1080).s / 4.0);
        assert!(half[0].0 as f32 >= art_px * k * 0.5 - 1.0, "straddle {straddle}: not below the chosen size: {half:?}");
        assert!(half[0].0 < plain[0].0, "straddle {straddle}: smaller than the sidecar's: {half:?} {plain:?}");
    }
    theme::set_theme(Theme::default());
}

#[test]
fn a_corner_size_outside_the_limits_is_refused() {
    with_frame("panel", red_corners(true));
    let r = Rect::new(100, 100, 700, 400);
    let corner = |size: f32| {
        let mut g = gfx(1280, 720, Grid::Native);
        let v = g.view;
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        ui.corner_size(size);
        ui.frame_body("panel", r);
        drop(ui);
        let p = v.rect(r);
        red_box(&g.canvas, Rect::new(p.x - 60, p.y - 60, p.w / 2, p.h / 2)).unwrap()
    };
    let at_one = corner(1.0);
    for bad in [f32::NAN, f32::INFINITY, -1.0, 0.0, CORNER_SIZE.0 - 0.01, CORNER_SIZE.1 + 0.01] {
        assert_eq!(corner(bad), at_one, "{bad}");
    }
    assert_ne!(corner(CORNER_SIZE.0), at_one, "the lowest limit is taken");
    theme::set_theme(Theme::default());
}

/// `frame_body` draws the role and gives the content rect: the frame's pad and corners on any grid, the padding token with
/// no frame. Content past a corner's reach moves with `corner_size`.
#[test]
fn frame_body_returns_the_content_rect_on_every_grid() {
    theme::set_theme(Theme::default());
    for grid in [Grid::Old, Grid::Native] {
        let u = grid.g();
        let mut g = gfx(1920, 1080, grid);
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        let r = Rect::new(20 * u, 20 * u, 200 * u, 100 * u);
        let pad = ui.m().padding;
        assert_eq!(ui.frame_body("inset", r), Rect::new(r.x + pad, r.y + pad, r.w - 2 * pad, r.h - 2 * pad), "{grid:?}: no frame");
        drop(ui);
        assert!(g.canvas.px.iter().any(|v| *v != 0), "it drew");
    }
    let mut framed = red_corners(false);
    framed.nine.pad = [3, 4, 5, 6];
    with_frame("inset", framed.clone());
    for grid in [Grid::Old, Grid::Native] {
        let u = grid.g();
        let mut g = gfx(1920, 1080, grid);
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        let r = Rect::new(20 * u, 20 * u, 200 * u, 100 * u);
        let measured = ui.frame_content("inset", r);
        assert_eq!(ui.frame_body("inset", r), Rect::new(r.x + 3 * u, r.y + 4 * u, r.w - 8 * u, r.h - 10 * u), "{grid:?}: the sidecar's pad, in old units");
        assert_eq!(ui.frame_content("inset", r), measured, "{grid:?}: measuring first gives what drawing gives");
        assert_eq!(ui.inset(r, false), Rect::new(r.x + 3 * u, r.y + 4 * u, r.w - 8 * u, r.h - 10 * u), "{grid:?}: `inset` is the same");
    }
    // A corner that reaches into the panel: past it at the size it draws, under the call.
    let mut reaching = red_corners(true);
    reaching.nine.corner_anchor = Some([0, 0]);
    reaching.nine.corner_inner = Some([28, 28]);
    reaching.nine.pad = [0; 4];
    with_frame("panel", reaching.clone());
    let mut g = gfx(1920, 1080, Grid::Native);
    let v = g.view;
    let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
    let r = Rect::new(20, 20, 800, 500);
    let full = ui.frame_body("panel", r);
    assert_eq!(full, v.nine_content(&reaching.nine, r));
    ui.corner_size(0.5);
    let small = ui.frame_body("panel", r);
    let mut half = reaching.nine.clone();
    half.corner_size = 0.5;
    assert_eq!(small, v.nine_content(&half, r), "the content rect is the one the corners draw at");
    assert!(small.x < full.x, "smaller corners reach less far: {small:?} {full:?}");
    drop(ui);
    theme::set_theme(Theme::default());
}

/// `Ui::new` starts with no corner size: the call lasts one frame of drawing.
#[test]
fn a_corner_size_lasts_for_one_ui() {
    with_frame("panel", red_corners(true));
    let r = Rect::new(100, 100, 700, 400);
    let shot = |call: bool| {
        let mut g = gfx(1280, 720, Grid::Native);
        if call {
            Ui::<()>::new(&mut g, (-1, -1), true).corner_size(0.5);
        }
        Ui::<()>::new(&mut g, (-1, -1), true).frame_body("panel", r);
        g.canvas.px
    };
    assert!(shot(true) == shot(false));
    theme::set_theme(Theme::default());
}

/// The arrow keys read widgets in any unit: the same moves with every rect doubled.
#[test]
fn focus_moves_the_same_on_doubled_rects() {
    let w = |id: &str, x: i32, y: i32, u: i32| json!({"id": id, "kind": "button", "rect": [x * u, y * u, 41 * u, 15 * u], "enabled": true, "clickable": true});
    let at = |u: i32| vec![w("a", 10, 11, u), w("b", 63, 9, u), w("c", 11, 43, u), w("d", 61, 41, u)];
    for from in ["a", "b", "c", "d"] {
        for dir in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (old, native) = (at(1), at(2));
            assert_eq!(focus::step(&old, Some(from), focus::Move::Dir(dir.0, dir.1), false), focus::step(&native, Some(from), focus::Move::Dir(dir.0, dir.1), false), "{from} {dir:?}");
        }
    }
}

#[test]
fn a_tip_is_read_from_the_mouse_first_then_the_focus() {
    let w = |id: &str, x: i32, tip: Value| json!({"id": id, "kind": "button", "rect": [x, 10, 40, 20], "enabled": true, "clickable": true, "tip": tip});
    let infos = vec![w("tipped", 10, json!(["A tip"])), w("bare", 60, json!([])), w("other", 110, json!(["Another"]))];
    let (over, off) = ((115, 15), (300, 300));
    assert_eq!(focus::tip_point(&infos, over, Some("tipped")), over, "the mouse over a card wins");
    assert_eq!(focus::tip_point(&infos, off, Some("tipped")), (30, 20), "else the focused widget's middle");
    assert_eq!(focus::tip_point(&infos, (65, 15), Some("tipped")), (30, 20), "over a widget with no card");
    assert_eq!(focus::tip_point(&infos, off, Some("bare")), off, "a focus with no card shows none");
    assert_eq!(focus::tip_point(&infos, off, Some("gone")), off, "a focus no longer there");
    assert_eq!(focus::tip_point(&infos, off, None), off);
}
