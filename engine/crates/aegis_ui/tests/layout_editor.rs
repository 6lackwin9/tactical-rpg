//! PR5-48 (MAP-LAYOUT-ED): the layout editor (`aegis_ui::layout_editor`) on generic fixtures: positions map through the
//! padded viewport both ways (outside 0..1 too), markers are boxed by the ground-point rule, a drag moves a marker or a
//! point and is one undo step, the arrows nudge fine and coarse, the scale slider and + / - stay in `SCALE_RANGE`, points
//! are inserted and removed, the switches change what the stage draws and nothing else, undo walks back in order and is
//! capped, Save is only a request and only in a build that may write, the stage draws inside its viewport, the agent
//! link's `layout_markers` state and `layout` ops work on the pane, and the same over a real link through a screen that
//! hosts the pane (the shell's handle drag, `Screen::pointer`). PR5-49 (MAP-POLISH1): a polyline's `ends` are fixed
//! anchors (no handle) with a live smooth preview through end, points, end that follows its markers, and a marker
//! without an image is drawn and grabbed as its `PIN_PX` pin (the caller's picture, else a dot).

use aegis_core::scratch::Scratch;
use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::{Canvas, Image, Rect};
use aegis_ui::layout_editor::{self, clamp_scale, pin_rect, scale_at, scale_steps, spline, step_of, writes_allowed, LayoutAct, LayoutEdit, LayoutEditor, Marker, MarkerBox, Polyline, Sel, Show, Viewport, NUDGE, PIN_PX, SCALE_RANGE, SCALE_STEP, STATE_KEY, UNDO_MAX};
use aegis_ui::painter::{Fonts, View};
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Key, PointerPhase, Screen, Settings};
use aegis_ui::shell::headless::{Headless, DEFAULT_FPS, IDLE_EXIT_S};
use aegis_ui::shell::{run, Options};
use aegis_ui::specimens::{KitAct, Specimens, PAGES};
use aegis_ui::ui::{Gfx, Kit, Ui, Widget, BG};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use aegis_core::clock::{Duration, Instant};

/// The fixture viewport: positions 0..1 span (40, 40) to (380, 300).
const VIEW: Viewport = Viewport { rect: Rect::new(10, 20, 400, 300), pad: (30.0, 20.0) };

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

fn gfx(w: i32, h: i32) -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(w, h), view: View::fit(w as u32, h as u32), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.canvas.clear(BG);
    g
}

fn square(side: i32) -> Rc<Image> {
    Rc::new(Image { w: side, h: side, px: vec![200; (side * side * 4) as usize] })
}

/// Two markers (one past the right edge) and a polyline of three points, on `VIEW`.
fn editor() -> LayoutEditor {
    let mut a = Marker::new("a", "Alpha", [0.25, 0.5], 1.0);
    a.image = Some(square(16));
    let b = Marker::new("b", "Beta", [1.042, 0.9], 1.5);
    let mut e = LayoutEditor::new(vec![a, b], vec![Polyline::new("r1", vec![[0.1, 0.1], [0.3, 0.2], [0.6, 0.2]]), Polyline::new("r2", Vec::new())]).with_view(VIEW);
    e.marker_box = MarkerBox { side: 40.0, foot: 6.0 };
    e
}

fn close(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
}

fn pos(e: &LayoutEditor, id: &str) -> [f32; 2] {
    e.marker(id).unwrap().pos
}

fn points(e: &LayoutEditor, id: &str) -> Vec<[f32; 2]> {
    e.polyline(id).unwrap().points.clone()
}

fn round4(v: f32) -> f32 {
    (v * 10_000.0).round() / 10_000.0
}

/// The position a drag by (`dx`, `dy`) screen px takes `p` to, by the viewport's own mapping.
fn dragged(p: [f32; 2], dx: i32, dy: i32) -> [f32; 2] {
    let s = VIEW.to_screen(p);
    let n = VIEW.to_norm((s.0 + dx as f32, s.1 + dy as f32));
    [round4(n[0]), round4(n[1])]
}

fn at(p: (f32, f32)) -> (i32, i32) {
    (p.0.round() as i32, p.1.round() as i32)
}

#[test]
fn positions_map_through_the_padded_viewport_both_ways_outside_0_to_1_too() {
    let (ax, ay, aw, ah) = VIEW.area();
    let r = VIEW.rect;
    assert_eq!((ax, ay, aw, ah), (r.x as f32 + VIEW.pad.0, r.y as f32 + VIEW.pad.1, r.w as f32 - 2.0 * VIEW.pad.0, r.h as f32 - 2.0 * VIEW.pad.1));
    assert_eq!(VIEW.to_screen([0.0, 0.0]), (ax, ay), "0 is the padded corner, not the rect's");
    assert_eq!(VIEW.to_screen([1.0, 1.0]), (ax + aw, ay + ah));
    for p in [[0.5, 0.5], [1.042, 0.99], [-0.25, 1.25], [-0.01, 0.2], [0.333, 0.667]] {
        let s = VIEW.to_screen(p);
        assert!((s.0 - (ax + p[0] * aw)).abs() < 1e-3 && (s.1 - (ay + p[1] * ah)).abs() < 1e-3, "{p:?}: x = rect.x + pad.x + fx * (rect.w - 2 pad.x)");
        assert!(close(VIEW.to_norm(s), p), "{p:?} comes back");
    }
    assert!(VIEW.to_screen([1.042, 0.5]).0 > ax + aw, "an edge anchor lies in the padding");
    // A viewport with no area maps without a division by zero.
    let flat = Viewport::new(Rect::new(0, 0, 20, 20), (10.0, 10.0));
    assert!(flat.to_norm((15.0, 15.0)).iter().all(|v| v.is_finite()));
    // A fitted viewport keeps the aspect and is centred in its stage.
    let v = Viewport::fit(Rect::new(0, 0, 300, 100), (3, 2), (0.1, 0.1));
    assert_eq!(v.rect.w * 2, v.rect.h * 3, "the aspect kept");
    assert_eq!(v.rect.h, 100, "as tall as the stage allows");
    assert_eq!(v.rect.x, (300 - v.rect.w) / 2);
    assert_eq!(v.pad, ((v.rect.w as f32 * 0.1).round(), (v.rect.h as f32 * 0.1).round()));
}

#[test]
fn a_marker_is_boxed_by_the_ground_point_rule() {
    let rule = MarkerBox { side: 48.0, foot: 8.0 };
    let g = (100.0, 200.0);
    for scale in [0.25, 1.0, 1.5, 4.0] {
        let b = rule.rect(g, scale);
        let side = (48.0f32 * scale).round() as i32;
        assert_eq!((b.w, b.h), (side, side), "scale {scale}");
        assert!((b.x + b.w / 2 - 100).abs() <= 1, "centred on the point at scale {scale}");
        assert_eq!(b.y + b.h, 208, "the bottom stays `foot` below the point at scale {scale}: it grows upward");
    }
}

#[test]
fn dragging_a_marker_moves_it_and_one_undo_puts_it_back() {
    let mut e = editor();
    let start = pos(&e, "a");
    let b = e.marker_box.rect(VIEW.to_screen(start), 1.0);
    let press = (b.x + b.w / 2, b.y + b.h / 2);
    assert!(e.pointer("layout.marker.a", PointerPhase::Down, press));
    assert_eq!(e.selected, Some(Sel::Marker(0)), "the press selects");
    for k in 1..=4 {
        assert!(e.pointer("layout.marker.a", PointerPhase::Move, (press.0 + 10 * k, press.1 - 5 * k)));
    }
    assert!(close(pos(&e, "a"), dragged(start, 40, -20)), "the ground point follows the pointer with the grab's offset");
    assert_eq!(e.undo_depth(), 0, "no step while dragging");
    assert!(e.pointer("layout.marker.a", PointerPhase::Up, (press.0 + 50, press.1 - 25)));
    assert!(close(pos(&e, "a"), dragged(start, 50, -25)));
    assert_eq!(e.undo_depth(), 1, "the drag is one step");
    assert_eq!(e.changes(), vec![LayoutEdit::Marker { id: "a".into(), pos: pos(&e, "a"), scale: 1.0 }]);
    assert!(e.dirty());
    assert_eq!(e.act(LayoutAct::Undo), vec![LayoutEdit::Marker { id: "a".into(), pos: start, scale: 1.0 }]);
    assert_eq!(pos(&e, "a"), start);
    assert!(!e.dirty(), "back to the values loaded");
    // A drag that ends where it began is no step; a cancelled one goes back and is none.
    e.pointer("layout.marker.a", PointerPhase::Down, press);
    e.pointer("layout.marker.a", PointerPhase::Move, (press.0 + 30, press.1));
    e.pointer("layout.marker.a", PointerPhase::Up, press);
    assert_eq!((pos(&e, "a"), e.undo_depth()), (start, 0));
    e.pointer("layout.marker.a", PointerPhase::Down, press);
    e.pointer("layout.marker.a", PointerPhase::Move, (press.0 + 30, press.1 + 30));
    assert!(e.dragging());
    assert!(e.pointer("layout.marker.a", PointerPhase::Cancel, (0, 0)));
    assert_eq!((pos(&e, "a"), e.undo_depth(), e.dragging()), (start, 0, false));
    // Ids that are not the pane's, or name nothing, are not taken.
    assert!(!e.pointer("other.marker.a", PointerPhase::Down, press));
    assert!(!e.pointer("layout.marker.nobody", PointerPhase::Down, press));
    assert!(!e.pointer("layout.point.9.r1", PointerPhase::Down, press));
    assert!(!e.pointer("layout.marker.a", PointerPhase::Move, press), "no move without a press");
}

#[test]
fn dragging_a_point_or_a_segment_middle() {
    let mut e = editor();
    let before = points(&e, "r1");
    let p = at(VIEW.to_screen(before[1]));
    assert!(e.pointer("layout.point.1.r1", PointerPhase::Down, p));
    assert_eq!(e.selected, Some(Sel::Point(0, 1)));
    e.pointer("layout.point.1.r1", PointerPhase::Up, (p.0 + 17, p.1 + 9));
    let now = points(&e, "r1");
    assert!(close(now[1], dragged(before[1], 17, 9)), "the point follows");
    assert_eq!((now[0], now[2]), (before[0], before[2]), "the others stay");
    assert_eq!(e.undo_depth(), 1);
    // A press on a segment's middle inserts a point there (a step) and drags it (another).
    let mid = [(now[1][0] + now[2][0]) / 2.0, (now[1][1] + now[2][1]) / 2.0];
    let m = at(VIEW.to_screen(mid));
    assert!(e.pointer("layout.mid.1.r1", PointerPhase::Down, m));
    assert_eq!(points(&e, "r1").len(), 4);
    assert_eq!(e.selected, Some(Sel::Point(0, 2)));
    e.pointer("layout.mid.1.r1", PointerPhase::Up, (m.0, m.1 + 12));
    let after = points(&e, "r1");
    assert!(close(after[2], [round4(mid[0]), VIEW.to_norm((VIEW.to_screen(mid).0, VIEW.to_screen(mid).1 + 12.0))[1]]), "{after:?}");
    assert_eq!(e.undo_depth(), 3);
    e.act(LayoutAct::Undo);
    assert!(close(points(&e, "r1")[2], [round4(mid[0]), round4(mid[1])]), "the drag goes first");
    e.act(LayoutAct::Undo);
    assert_eq!(points(&e, "r1"), now, "then the insert");
    e.act(LayoutAct::Undo);
    assert_eq!(points(&e, "r1"), before);
    assert!(!e.pointer("layout.mid.2.r1", PointerPhase::Down, m), "no segment past the last point");
}

#[test]
fn the_arrows_nudge_the_selection_fine_and_coarse() {
    let mut e = editor();
    assert!(!e.key(Key::Right, false, false), "nothing selected: the key is not the pane's");
    e.act(LayoutAct::Select(Some(Sel::Marker(0))));
    let p = pos(&e, "a");
    assert!(e.key(Key::Right, false, false));
    assert_eq!(pos(&e, "a"), [round4(p[0] + NUDGE.0), p[1]]);
    assert!(e.key(Key::Up, true, false));
    assert_eq!(pos(&e, "a"), [round4(p[0] + NUDGE.0), round4(p[1] - NUDGE.1)], "Shift nudges coarse");
    assert!(e.key(Key::Left, false, false) && e.key(Key::Down, true, false));
    assert_eq!(pos(&e, "a"), p, "back where it was");
    assert_eq!(e.undo_depth(), 4, "each nudge is a step");
    assert!(NUDGE.1 > NUDGE.0);
    // A point nudges too, and past 0..1.
    e.act(LayoutAct::Select(Some(Sel::Point(0, 0))));
    let q = points(&e, "r1")[0];
    for _ in 0..5 {
        e.key(Key::Left, true, false);
    }
    assert_eq!(points(&e, "r1")[0], [round4(q[0] - 5.0 * NUDGE.1), q[1]]);
    assert!(points(&e, "r1")[0][0] < 0.0, "not clamped");
    assert_eq!(pos(&e, "a"), p);
}

#[test]
fn the_scale_slider_and_plus_minus_stay_in_range() {
    assert_eq!((scale_at(0), scale_at(usize::MAX)), SCALE_RANGE);
    assert_eq!(scale_at(scale_steps() - 1), SCALE_RANGE.1);
    assert_eq!(scale_at(step_of(1.0)), 1.0);
    assert_eq!((clamp_scale(0.1), clamp_scale(9.0)), SCALE_RANGE);
    let mut e = editor();
    e.act(LayoutAct::Slide(10));
    assert_eq!((e.marker("a").unwrap().scale, e.undo_depth()), (1.0, 0), "nothing selected: no change");
    e.act(LayoutAct::Select(Some(Sel::Marker(0))));
    e.act(LayoutAct::Slide(0));
    assert_eq!(e.marker("a").unwrap().scale, SCALE_RANGE.0);
    e.act(LayoutAct::Slide(usize::MAX));
    assert_eq!(e.marker("a").unwrap().scale, SCALE_RANGE.1);
    assert_eq!(e.undo_depth(), 1, "slider moves on one marker are one step");
    assert!(e.key(Key::Char('+'), false, false));
    assert_eq!((e.marker("a").unwrap().scale, e.undo_depth()), (SCALE_RANGE.1, 1), "+ at the top changes nothing");
    assert!(e.key(Key::Char('-'), false, false));
    assert_eq!((e.marker("a").unwrap().scale, e.undo_depth()), (clamp_scale(SCALE_RANGE.1 - SCALE_STEP), 2), "- is a step of its own");
    e.act(LayoutAct::Undo);
    e.act(LayoutAct::Undo);
    assert_eq!(e.marker("a").unwrap().scale, 1.0, "two undos: back past the whole slider drag");
    e.act(LayoutAct::Slide(0));
    for _ in 0..3 {
        e.act(LayoutAct::ScaleStep(-1));
    }
    assert_eq!(e.marker("a").unwrap().scale, SCALE_RANGE.0, "- at the bottom stays");
    e.act(LayoutAct::ScaleStep(1));
    assert_eq!(e.marker("a").unwrap().scale, clamp_scale(SCALE_RANGE.0 + SCALE_STEP));
    assert_eq!(e.marker("b").unwrap().scale, 1.5, "only the selected marker");
}

#[test]
fn points_are_inserted_and_removed() {
    let mut e = editor();
    let p = points(&e, "r1");
    e.act(LayoutAct::Select(Some(Sel::Point(0, 0))));
    let ed = e.act(LayoutAct::InsertPoint);
    let mid = [round4((p[0][0] + p[1][0]) / 2.0), round4((p[0][1] + p[1][1]) / 2.0)];
    assert_eq!(points(&e, "r1"), vec![p[0], mid, p[1], p[2]], "halfway to the next");
    assert_eq!(ed, vec![LayoutEdit::Points { id: "r1".into(), points: points(&e, "r1") }]);
    assert_eq!(e.selected, Some(Sel::Point(0, 1)));
    e.act(LayoutAct::Select(Some(Sel::Point(0, 3))));
    e.act(LayoutAct::InsertPoint);
    let last = points(&e, "r1")[4];
    assert!(close(last, [p[2][0] + (p[2][0] - p[1][0]) / 2.0, p[2][1] + (p[2][1] - p[1][1]) / 2.0]), "past the last: half a segment on");
    e.act(LayoutAct::RemovePoint);
    assert_eq!(points(&e, "r1"), vec![p[0], mid, p[1], p[2]]);
    assert_eq!(e.selected, Some(Sel::Point(0, 3)), "the selection goes to the point before");
    // The empty polyline: Insert point on it starts in the middle; Remove point down to none.
    e.act(LayoutAct::Select(None));
    e.act(LayoutAct::Line(1));
    e.act(LayoutAct::InsertPoint);
    assert_eq!(points(&e, "r2"), vec![[0.5, 0.5]]);
    e.act(LayoutAct::RemovePoint);
    assert!(points(&e, "r2").is_empty());
    assert_eq!(e.selected, None);
    assert!(e.act(LayoutAct::RemovePoint).is_empty(), "nothing selected, nothing removed");
    assert_eq!(e.undo_depth(), 5);
    for _ in 0..5 {
        e.act(LayoutAct::Undo);
    }
    assert_eq!((points(&e, "r1"), points(&e, "r2")), (p, Vec::new()));
}

#[test]
fn the_switches_change_the_stage_and_nothing_else() {
    let mut e = editor();
    let draw = |e: &LayoutEditor| {
        let mut g = gfx(640, 360);
        let mut ui: Ui<LayoutAct> = Ui::new(&mut g, (-1, -1), true);
        e.draw_stage(&mut ui, &|a| a);
        drop(ui);
        g.canvas.px
    };
    let base = draw(&e);
    for (s, flag) in [(Show::Grid, 0usize), (Show::Labels, 1), (Show::Rings, 2)] {
        let was = [e.grid, e.labels, e.rings][flag];
        assert!(e.act(LayoutAct::Toggle(s)).is_empty(), "{s:?} is no edit");
        assert_eq!([e.grid, e.labels, e.rings][flag], !was);
        assert_ne!(draw(&e), base, "{s:?} changes the picture");
        e.act(LayoutAct::Toggle(s));
        assert_eq!(draw(&e), base, "{s:?} back: the same picture");
    }
    assert_eq!((e.undo_depth(), e.dirty()), (0, false));
}

#[test]
fn undo_walks_back_in_order_and_keeps_at_most_its_cap() {
    let mut e = editor();
    let (a, r) = (pos(&e, "a"), points(&e, "r1"));
    e.set_marker("a", Some([0.4, 0.4]), None).unwrap();
    e.set_points("r1", vec![[0.0, 0.0]]).unwrap();
    e.set_marker("a", None, Some(2.0)).unwrap();
    assert_eq!(e.undo_depth(), 3);
    assert_eq!(e.undo(), Some(LayoutEdit::Marker { id: "a".into(), pos: [0.4, 0.4], scale: 1.0 }), "the last first");
    assert_eq!(e.undo(), Some(LayoutEdit::Points { id: "r1".into(), points: r.clone() }));
    assert_eq!(e.undo(), Some(LayoutEdit::Marker { id: "a".into(), pos: a, scale: 1.0 }));
    assert_eq!(e.undo(), None);
    assert!(e.act(LayoutAct::Undo).is_empty() && e.note == "nothing to undo");
    // Ctrl+Z is Undo; z alone is not.
    e.act(LayoutAct::Select(Some(Sel::Marker(0))));
    e.key(Key::Right, false, false);
    assert!(!e.key(Key::Char('z'), false, false));
    assert!(e.key(Key::Char('z'), false, true));
    assert_eq!((pos(&e, "a"), e.undo_depth()), (a, 0));
    for _ in 0..UNDO_MAX + 10 {
        e.key(Key::Right, false, false);
    }
    assert_eq!(e.undo_depth(), UNDO_MAX, "the oldest steps go");
    while e.undo().is_some() {}
    assert_eq!(pos(&e, "a"), [round4(a[0] + 10.0 * NUDGE.0), a[1]], "back to the oldest step kept");
}

#[test]
fn save_is_only_a_request_and_only_where_writes_are_allowed() {
    assert_eq!(writes_allowed(), cfg!(debug_assertions) && !cfg!(target_arch = "wasm32"));
    let mut e = editor();
    assert!(e.request_save().unwrap_or_default().contains("nothing changed") || !writes_allowed());
    assert!(!e.save_pending());
    e.set_marker("b", Some([1.1, 0.5]), Some(0.5)).unwrap();
    assert!(e.request_save_as(false).is_err(), "a build that may not write refuses");
    assert!(!e.save_pending());
    let wanted = e.changes();
    match e.request_save() {
        Ok(_) => {
            assert!(writes_allowed());
            assert_eq!(e.take_save(), Some(wanted), "the changed values, for the caller to write");
            assert!(e.dirty(), "still unsaved until the caller says so");
            e.mark_saved();
            assert!(!e.dirty() && e.changes().is_empty() && e.take_save().is_none());
            assert_eq!(e.undo_depth(), 1, "the undo steps stay");
            e.act(LayoutAct::Undo);
            assert!(e.dirty(), "undo past a save is a change again");
        }
        Err(_) => assert!(!writes_allowed()),
    }
    // The Save button is drawn only where writes are allowed.
    let mut g = gfx(640, 360);
    let mut ui: Ui<LayoutAct> = Ui::new(&mut g, (-1, -1), true);
    e.draw(&mut ui, Rect::new(0, 0, 640, 360), &|a| a);
    assert_eq!(ui.widgets.iter().any(|w| w.id == "layout.save"), writes_allowed());
}

/// Pass 2: without `backdrop` the stage draws over what the caller drew there (no fill, no background), so a live screen
/// is edited in place; with it, the stage paints its ground.
#[test]
fn without_a_backdrop_the_stage_draws_over_the_callers_picture() {
    let mut e = editor();
    e.background = Some(square(8));
    e.grid = false;
    let draw = |e: &LayoutEditor| {
        let mut g = gfx(640, 360);
        let r = VIEW.rect;
        g.canvas.fill(r, [9, 99, 9, 255]);
        let mut ui: Ui<LayoutAct> = Ui::new(&mut g, (-1, -1), true);
        e.draw_stage(&mut ui, &|a| a);
        let handles = ui.widgets.iter().filter(|w| w.kind == "handle").count();
        drop(ui);
        (g.canvas.px, handles)
    };
    // a spot of the stage away from the markers, the polylines and the padded area's edge
    let spot = |px: &[u8]| {
        let (x, y) = (VIEW.rect.x + 8, VIEW.rect.y + VIEW.rect.h / 2);
        let i = ((y * 640 + x) * 4) as usize;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    };
    let (painted, n) = draw(&e);
    assert_ne!(spot(&painted), [9, 99, 9, 255], "the backdrop covers the caller's picture");
    e.backdrop = false;
    let (over, m) = draw(&e);
    assert_eq!(spot(&over), [9, 99, 9, 255], "no backdrop: the caller's picture shows through");
    assert_eq!(n, m, "the same handles either way");
}

#[test]
fn the_stage_draws_inside_its_viewport_and_the_pane_at_any_size() {
    for (w, h) in [(640, 360), (1280, 720), (1920, 1080)] {
        for scale in [SCALE_RANGE.0, 1.0, SCALE_RANGE.1] {
            let mut e = editor();
            e.set_view(Viewport::new(Rect::new(200, 60, 300, 200), (24.0, 16.0)));
            e.set_marker("a", Some([-0.2, 1.25]), Some(scale)).unwrap();
            e.set_marker("b", None, Some(scale)).unwrap();
            e.background = Some(square(8));
            let mut g = gfx(w, h);
            let blank = g.canvas.px.clone();
            {
                let mut ui: Ui<LayoutAct> = Ui::new(&mut g, (-1, -1), true);
                e.draw_stage(&mut ui, &|a| a);
                let handles: Vec<&Widget<LayoutAct>> = ui.widgets.iter().filter(|w| w.kind == "handle").collect();
                assert!(handles.iter().any(|w| w.id == "layout.marker.a") && handles.iter().any(|w| w.id == "layout.point.2.r1") && handles.iter().any(|w| w.id == "layout.mid.0.r1"));
                assert!(handles.iter().all(|w| e.view().rect.intersect(w.rect) == w.rect || w.rect.w <= 0 || w.rect.h <= 0), "handles are cut to the viewport");
            }
            let phys = g.view.rect(e.view().rect);
            for y in 0..g.canvas.h {
                for x in 0..g.canvas.w {
                    if !phys.contains(x, y) {
                        let i = ((y * g.canvas.w + x) * 4) as usize;
                        assert_eq!(g.canvas.px[i..i + 4], blank[i..i + 4], "{w}x{h} scale {scale}: drew at ({x}, {y}) outside {phys:?}");
                    }
                }
            }
            let mut ui: Ui<LayoutAct> = Ui::new(&mut g, (-1, -1), true);
            e.draw(&mut ui, Rect::new(0, 0, 640, 360), &|a| a);
        }
    }
}

#[test]
fn the_agent_link_reads_the_layout_and_sets_markers_and_routes() {
    let mut e = editor();
    let s = e.state();
    assert_eq!(s["markers"][0], json!({"id": "a", "label": "Alpha", "pos": [0.25, 0.5], "scale": 1.0, "image": true}));
    assert_eq!(s["markers"][1]["pos"], json!([1.042, 0.9]), "kept to 4 decimals, no float noise");
    assert_eq!(s["routes"][0], json!({"id": "r1", "points": [[0.1, 0.1], [0.3, 0.2], [0.6, 0.2]]}));
    assert_eq!((s["dirty"].clone(), s["undo"].clone(), s["writes"].clone()), (json!(false), json!(0), json!(writes_allowed())));
    assert_eq!(e.command(&json!({"cmd": STATE_KEY})).unwrap()[STATE_KEY], s);
    let r = e.command(&json!({"cmd": "layout", "id": "a", "pos": [1.042, -0.1], "scale": 1.25})).unwrap();
    assert_eq!(r, json!({"ok": true, "edit": {"id": "a", "pos": [1.042, -0.1], "scale": 1.25}, "clamped": false, "dirty": true, "undo": 1}));
    assert_eq!((pos(&e, "a"), e.marker("a").unwrap().scale), ([1.042, -0.1], 1.25));
    let r = e.command(&json!({"cmd": "layout", "id": "a", "scale": 9})).unwrap();
    assert_eq!((r["edit"]["scale"].clone(), r["clamped"].clone()), (json!(SCALE_RANGE.1), json!(true)), "scale is clamped and says so");
    assert_eq!(r["edit"]["pos"], json!([1.042, -0.1]), "pos left as it was");
    let r = e.command(&json!({"cmd": "layout", "route": "r1", "points": [[0.2, 0.2], [0.8, 1.1]]})).unwrap();
    assert_eq!(r["edit"], json!({"route": "r1", "points": [[0.2, 0.2], [0.8, 1.1]]}));
    assert_eq!(points(&e, "r1"), vec![[0.2, 0.2], [0.8, 1.1]]);
    assert_eq!(e.undo_depth(), 3, "each op one step");
    for bad in [
        json!({"cmd": "layout", "id": "nobody", "pos": [0.1, 0.1]}),
        json!({"cmd": "layout", "id": "a"}),
        json!({"cmd": "layout", "id": "a", "pos": [0.1]}),
        json!({"cmd": "layout", "id": "a", "pos": ["x", 0.1]}),
        json!({"cmd": "layout", "id": "a", "scale": "big"}),
        json!({"cmd": "layout", "route": "r1"}),
        json!({"cmd": "layout", "route": "r1", "points": [[0.1, 0.1], [0.2]]}),
        json!({"cmd": "layout", "route": "nowhere", "points": []}),
        json!({"cmd": "layout", "id": "a", "route": "r1", "pos": [0.1, 0.1]}),
        json!({"cmd": "layout"}),
    ] {
        let r = e.command(&bad).unwrap();
        assert_eq!(r["ok"], json!(false), "{bad}");
        assert!(r["error"].as_str().is_some_and(|m| !m.is_empty()));
    }
    assert_eq!(e.undo_depth(), 3, "a refused op changes nothing");
    assert!(e.command(&json!({"cmd": "ui_tree"})).is_none(), "other commands are not the pane's");
    let s = e.state();
    assert_eq!((s["dirty"].clone(), s["markers"][0]["pos"].clone()), (json!(true), json!([1.042, -0.1])));
}

#[test]
fn the_gallery_has_a_layout_page_that_hosts_the_editor() {
    let page = PAGES.iter().position(|p| *p == "Layout").expect("a Layout page");
    let mut k = Specimens::default();
    k.act(KitAct::Page(page));
    let mut g = gfx(640, 360);
    let w = {
        let mut ui: Ui<KitAct> = Ui::new(&mut g, (-1, -1), true);
        k.draw(&mut ui, Rect::new(0, 0, 640, 360), |a| a);
        ui.widgets
    };
    let marker = w.iter().find(|w| w.kind == "handle" && w.id.starts_with("layout.marker.")).expect("a marker handle");
    let id = marker.id.clone();
    let mid = (marker.rect.x + marker.rect.w / 2, marker.rect.y + marker.rect.h / 2);
    let mark = id.trim_start_matches("layout.marker.").to_string();
    let was = k.layout.marker(&mark).unwrap().pos;
    assert!(k.pointer(&id, PointerPhase::Down, mid));
    assert!(k.pointer(&id, PointerPhase::Up, (mid.0 + 8, mid.1)));
    assert!(k.layout.marker(&mark).unwrap().pos[0] > was[0]);
    assert!(k.key(Key::Char('-')) && k.layout.undo_depth() == 2);
    k.act(KitAct::Layout(LayoutAct::Undo));
    assert_eq!(k.layout.undo_depth(), 1);
    assert!(k.last.starts_with("undone"));
    k.act(KitAct::Page(0));
    assert!(!k.pointer(&id, PointerPhase::Down, mid), "only on its page");
    let sample = layout_editor::sample();
    assert!(sample.markers().iter().any(|m| !(0.0..=1.0).contains(&m.pos[0])), "the sample shows an edge anchor");
}

// ---- over a real agent link: a screen that hosts the pane ----

struct App;

impl Context for App {
    const MORE_COMMANDS: &'static str = "layout, layout_markers";
    fn note(&mut self, _line: String) {}
    fn agent_summary(&self) -> Value {
        Value::Null
    }
    fn agent_game(&self) -> Value {
        Value::Null
    }
}

type AppCx = Cx<App>;

struct Host {
    ed: LayoutEditor,
    w: Vec<Widget<LayoutAct>>,
}

impl Host {
    fn act(&mut self, _cx: &mut AppCx, a: LayoutAct) {
        self.ed.act(a);
    }
}

impl Screen<App> for Host {
    fn id(&self) -> &'static str {
        "layout_host"
    }
    fn draw(&mut self, _cx: &mut AppCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<LayoutAct> = Ui::new(g, mouse, live);
        self.ed.draw(&mut ui, Rect::new(0, 0, 640, 360), &|a| a);
        self.w = ui.widgets;
    }
    fn pointer(&mut self, _cx: &mut AppCx, id: &str, phase: PointerPhase, pos: (i32, i32)) {
        self.ed.pointer(id, phase, pos);
    }
    fn key(&mut self, cx: &mut AppCx, k: Key) {
        self.ed.key(k, cx.held.shift, cx.held.ctrl);
    }
    fn arrow_focus(&self) -> bool {
        false
    }
    fn command(&mut self, _cx: &AppCx, body: &Value) -> Option<Value> {
        self.ed.command(body)
    }
    fn state(&self, _cx: &AppCx) -> Value {
        json!({STATE_KEY: self.ed.state()})
    }
    aegis_ui::widget_plumbing!(AppCx);
}

fn context() -> AppCx {
    static N: AtomicU32 = AtomicU32::new(0);
    let settings = Scratch::new(&format!("layout_editor_settings_{}", N.fetch_add(1, Ordering::Relaxed))).until_thread_end().join("settings.json");
    Cx {
        settings: Settings::load(settings, json!({})),
        art: Art::new(None),
        audio: Audio::offline(None),
        nav: Vec::new(),
        notice: Vec::new(),
        shot_dir: std::env::temp_dir(),
        quit: false,
        answers: Vec::new(),
        playtest: Playtests::off(),
        held: Default::default(),
        app: App,
    }
}

struct Link(BufReader<TcpStream>);

impl Link {
    fn ask(&mut self, body: Value) -> Value {
        writeln!(self.0.get_mut(), "{body}").unwrap();
        let mut line = String::new();
        self.0.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }
}

#[test]
fn over_the_agent_link_a_hosting_screen_drags_reads_and_sets_the_layout() {
    let port = TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port();
    let options = Options {
        title: "test".into(),
        font: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf"),
        port,
        icon: String::new(),
        canvas: Value::Null,
        window_sizes: Value::Null,
        headless: Some(Headless { size: (800, 450), fps: DEFAULT_FPS, dpr: 1.0, idle: Duration::from_secs(IDLE_EXIT_S), stepped: false }),
    };
    let thread = std::thread::spawn(move || {
        let host = Host { ed: editor().with_view(Viewport::new(Rect::new(190, 30, 420, 300), (30.0, 20.0))), w: Vec::new() };
        run(context(), vec![Box::new(host) as Box<dyn Screen<App>>], options)
    });
    let by = Instant::now() + Duration::from_secs(30);
    let stream = loop {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => break s,
            Err(e) => {
                assert!(Instant::now() < by, "no link: {e}");
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    };
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
    let mut link = Link(BufReader::new(stream));
    // The first frame draws the pane's widgets.
    let tree = loop {
        let t = link.ask(json!({"cmd": "ui_tree"}));
        if t["widgets"].as_array().is_some_and(|w| w.iter().any(|w| w["id"] == "layout.marker.a")) {
            break t;
        }
        assert!(Instant::now() < by, "the pane never drew");
        std::thread::sleep(Duration::from_millis(20));
    };
    let state = &tree["state"][STATE_KEY];
    assert_eq!(state["markers"][0]["pos"], json!([0.25, 0.5]));
    let view = Viewport::new(Rect::new(190, 30, 420, 300), (30.0, 20.0));
    let handle = tree["widgets"].as_array().unwrap().iter().find(|w| w["id"] == "layout.marker.a").unwrap().clone();
    assert_eq!(handle["kind"], json!("handle"));
    let r: Vec<i64> = handle["rect"].as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect();
    let from = (r[0] + r[2] / 2, r[1] + r[3] / 2);
    let to = (from.0 + 42, from.1 - 21);
    let reply = link.ask(json!({"cmd": "drag", "id": "layout.marker.a", "to": [to.0, to.1]}));
    assert_eq!(reply["ok"], json!(true), "{reply}");
    let s = link.ask(json!({"cmd": "layout_markers"}));
    let got = s[STATE_KEY]["markers"][0]["pos"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect::<Vec<_>>();
    let g0 = view.to_screen([0.25, 0.5]);
    let want = view.to_norm((g0.0 + 42.0, g0.1 - 21.0));
    assert!(close([got[0], got[1]], want), "the shell's handle drag moved the marker by the pointer: {got:?} vs {want:?}");
    assert_eq!(s[STATE_KEY]["undo"], json!(1), "one step for the drag");
    assert_eq!(s[STATE_KEY]["dragging"], json!(false), "the release ended it");
    // The ops over the link.
    let r = link.ask(json!({"cmd": "layout", "id": "b", "pos": [0.5, 0.75], "scale": 2.0}));
    assert_eq!(r["edit"], json!({"id": "b", "pos": [0.5, 0.75], "scale": 2.0}), "{r}");
    let r = link.ask(json!({"cmd": "layout", "route": "r2", "points": [[0.1, 0.9], [0.9, 0.1]]}));
    assert_eq!(r["edit"], json!({"route": "r2", "points": [[0.1, 0.9], [0.9, 0.1]]}), "{r}");
    let tree = link.ask(json!({"cmd": "ui_tree"}));
    assert_eq!(tree["state"][STATE_KEY]["markers"][1]["scale"], json!(2.0));
    assert_eq!(tree["state"][STATE_KEY]["undo"], json!(3));
    // The Undo button by id, as an agent clicks it, once a frame has drawn it enabled.
    let by = Instant::now() + Duration::from_secs(30);
    while !link.ask(json!({"cmd": "ui_tree"}))["widgets"].as_array().unwrap().iter().any(|w| w["id"] == "layout.undo" && w["enabled"] == json!(true) && w["text"] == json!("Undo (3)")) {
        assert!(Instant::now() < by, "the Undo button never showed the three steps");
        std::thread::sleep(Duration::from_millis(20));
    }
    let r = link.ask(json!({"cmd": "click", "id": "layout.undo"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    let s = link.ask(json!({"cmd": "layout_markers"}));
    assert_eq!(s[STATE_KEY]["routes"][1]["points"], json!([]), "the last op undone");
    assert_eq!(link.ask(json!({"cmd": "quit"}))["ok"], json!(true));
    thread.join().unwrap().unwrap();
}

// ---- PR5-49 (MAP-POLISH1): polyline ends, the live spline preview and the pin fallback ---------------------------

/// `editor()` with polyline `r1` running from marker `a` to marker `b` and `r2` (no points) between them too.
fn ended() -> LayoutEditor {
    let mut a = Marker::new("a", "Alpha", [0.25, 0.5], 1.0);
    a.image = Some(square(16));
    let b = Marker::new("b", "Beta", [1.042, 0.9], 1.5);
    let lines = vec![Polyline::new("r1", vec![[0.1, 0.1], [0.3, 0.2], [0.6, 0.2]]).with_ends("a", "b"), Polyline::new("r2", Vec::new()).with_ends("b", "a"), Polyline::new("free", vec![[0.2, 0.2], [0.4, 0.4]])];
    let mut e = LayoutEditor::new(vec![a, b], lines).with_view(VIEW);
    e.marker_box = MarkerBox { side: 40.0, foot: 6.0 };
    e
}

/// The handle widgets the stage pushes (id, rect).
fn handles(e: &LayoutEditor) -> Vec<(String, Rect)> {
    let mut g = gfx(640, 360);
    let mut ui: Ui<LayoutAct> = Ui::new(&mut g, (-1, -1), true);
    e.draw_stage(&mut ui, &|a| a);
    ui.widgets.iter().filter(|w| w.kind == "handle").map(|w| (w.id.clone(), w.rect)).collect()
}

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
}

#[test]
fn the_spline_passes_through_every_point_and_two_points_are_straight() {
    assert_eq!(spline(&[(0.0, 0.0), (10.0, 0.0)]), [(0.0, 0.0), (10.0, 0.0)]);
    assert!(spline(&[(1.0, 1.0)]).is_empty() && spline(&[]).is_empty());
    let pts = [(0.0, 0.0), (20.0, 10.0), (40.0, 0.0), (60.0, 30.0)];
    let line = spline(&pts);
    for p in pts {
        assert!(line.iter().any(|q| near(*q, p)), "{p:?} lies on the line");
    }
    assert!(near(line[0], pts[0]) && near(*line.last().unwrap(), pts[3]), "it starts and ends on the end points");
    assert!(line.windows(2).all(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt() <= 2.5), "sampled about every 2 px");
}

#[test]
fn a_polyline_between_two_markers_runs_end_points_end_and_previews_smoothly() {
    let mut e = ended();
    let (a, b) = (pos(&e, "a"), pos(&e, "b"));
    assert_eq!(e.end_points(0), Some((a, b)));
    assert_eq!(e.chain(0), vec![a, [0.1, 0.1], [0.3, 0.2], [0.6, 0.2], b], "end, points, end");
    assert_eq!(e.chain(1), vec![b, a], "no points: end to end");
    assert_eq!(e.chain(2), points(&e, "free"), "a free line is its points");
    assert!(e.preview(2).is_empty(), "no preview for a free line");
    let preview = e.preview(0);
    let screen: Vec<(f32, f32)> = e.chain(0).into_iter().map(|p| VIEW.to_screen(p)).collect();
    assert_eq!(preview, spline(&screen), "the preview is the spline through the run on screen");
    for p in &screen {
        assert!(preview.iter().any(|q| near(*q, *p)), "{p:?}: the ends and every point lie on the preview");
    }
    assert_eq!(e.preview(1), screen_of(&[b, a]), "end to end: straight");
    // the preview is live: it follows a moved end marker
    e.command(&json!({"cmd": "layout", "id": "b", "pos": [0.8, 0.95]})).unwrap();
    assert_eq!(*e.preview(0).last().unwrap(), VIEW.to_screen([0.8, 0.95]), "it ends on the end marker where it is now");
    assert_eq!(e.chain(0).last(), Some(&[0.8, 0.95]));
    // an end naming no marker leaves a free line
    let lone = LayoutEditor::new(vec![Marker::new("a", "A", [0.5, 0.5], 1.0)], vec![Polyline::new("x", vec![[0.1, 0.1]]).with_ends("a", "nobody")]);
    assert_eq!((lone.end_points(0), lone.chain(0), lone.preview(0).len()), (None, vec![[0.1, 0.1]], 0));
}

fn screen_of(pts: &[[f32; 2]]) -> Vec<(f32, f32)> {
    pts.iter().map(|p| VIEW.to_screen(*p)).collect()
}

#[test]
fn a_polylines_ends_are_fixed_anchors_not_handles() {
    let mut e = ended();
    let hs = handles(&e);
    let ids: Vec<&str> = hs.iter().map(|(i, _)| i.as_str()).collect();
    // each marker one handle; each point of r1 one handle; no handle at all for the ends
    for k in 0..3 {
        assert!(ids.contains(&format!("layout.point.{k}.r1").as_str()), "point {k}");
    }
    assert!(!ids.contains(&"layout.point.3.r1") && !ids.iter().any(|i| i.contains("anchor") || i.contains("end.r2.") || i.starts_with("layout.point.") && i.ends_with(".r2")), "{ids:?}");
    // the segments from the ends take a middle handle (start, end), the inner ones theirs; r2 (no points) one, its start
    for id in ["layout.mid.start.r1", "layout.mid.0.r1", "layout.mid.1.r1", "layout.mid.end.r1", "layout.mid.start.r2"] {
        assert!(ids.contains(&id), "{id}: {ids:?}");
    }
    assert!(!ids.contains(&"layout.mid.2.r1") && !ids.contains(&"layout.mid.end.r2"));
    // no press can move an end: the only handle over an end's point is its marker's, which moves the marker (and the end
    // with it), never the polyline's points
    let before = points(&e, "r1");
    let at_b = at(VIEW.to_screen(pos(&e, "b")));
    let over_b: Vec<&str> = hs.iter().filter(|(_, r)| r.contains(at_b.0, at_b.1)).map(|(i, _)| i.as_str()).collect();
    assert!(over_b.iter().all(|i| *i == "layout.marker.b" || i.starts_with("layout.mid.")), "{over_b:?}");
    assert!(!e.pointer("layout.point.3.r1", PointerPhase::Down, at_b), "past the last point: no point to grab");
    assert!(!e.pointer("layout.anchor.1.r1", PointerPhase::Down, at_b));
    assert!(e.pointer("layout.marker.b", PointerPhase::Down, at_b));
    e.pointer("layout.marker.b", PointerPhase::Up, (at_b.0 - 20, at_b.1 - 10));
    assert_eq!(points(&e, "r1"), before, "the points stay");
    assert_eq!(e.chain(0).last(), Some(&pos(&e, "b")), "the end moved with its marker");
    assert_eq!(e.changes(), vec![LayoutEdit::Marker { id: "b".into(), pos: pos(&e, "b"), scale: 1.5 }], "a marker edit, no polyline edit");
}

#[test]
fn a_segment_from_an_end_inserts_before_the_first_point_or_after_the_last() {
    let mut e = ended();
    let (a, b) = (pos(&e, "a"), pos(&e, "b"));
    let p = points(&e, "r1");
    let half = |x: [f32; 2], y: [f32; 2]| [round4((x[0] + y[0]) / 2.0), round4((x[1] + y[1]) / 2.0)];
    assert!(e.pointer("layout.mid.start.r1", PointerPhase::Down, (0, 0)));
    e.pointer("layout.mid.start.r1", PointerPhase::Cancel, (0, 0));
    assert_eq!(points(&e, "r1")[0], half(a, p[0]), "halfway from the first end to the first point");
    assert!(e.pointer("layout.mid.end.r1", PointerPhase::Down, (0, 0)));
    e.pointer("layout.mid.end.r1", PointerPhase::Cancel, (0, 0));
    assert_eq!(*points(&e, "r1").last().unwrap(), half(p[2], b), "halfway from the last point to the second end");
    assert!(e.pointer("layout.mid.start.r2", PointerPhase::Down, (0, 0)));
    e.pointer("layout.mid.start.r2", PointerPhase::Cancel, (0, 0));
    assert_eq!(points(&e, "r2"), vec![half(b, a)], "no points: halfway between the ends");
    assert!(!e.pointer("layout.mid.start.free", PointerPhase::Down, (0, 0)), "a free line has no end segments");
    // Insert point: past the last point halfway to the second end; on an empty one halfway between the ends
    let mut e = ended();
    e.act(LayoutAct::Select(Some(Sel::Point(0, 2))));
    e.act(LayoutAct::InsertPoint);
    assert_eq!(points(&e, "r1")[3], half(p[2], b));
    e.act(LayoutAct::Select(None));
    e.act(LayoutAct::Line(1));
    e.act(LayoutAct::InsertPoint);
    assert_eq!(points(&e, "r2"), vec![half(b, a)]);
}

#[test]
fn a_marker_without_an_image_is_its_small_pin() {
    let mut e = ended();
    let g = VIEW.to_screen(pos(&e, "b"));
    let pin = pin_rect(g);
    assert_eq!((pin.w, pin.h), (PIN_PX as i32, PIN_PX as i32));
    assert!((pin.x + pin.w / 2 - g.0 as i32).abs() <= 1 && (pin.y + pin.h / 2 - g.1 as i32).abs() <= 1, "centred on the point");
    let b = e.marker("b").unwrap().clone();
    assert_eq!(e.marker_rect(&b), pin, "no image: the pin, not the {} px box", e.marker_box.side * b.scale);
    let a = e.marker("a").unwrap().clone();
    assert_eq!(e.marker_rect(&a), e.marker_box.rect(VIEW.to_screen(a.pos), a.scale), "an image: the ground-point box");
    let hs = handles(&e);
    assert_eq!(hs.iter().find(|(i, _)| i == "layout.marker.b").map(|h| h.1), Some(pin), "grabbed at the pin");
    // the pin draws inside its box: the caller's picture there, else a dot
    let draw = |e: &LayoutEditor| {
        let mut g = gfx(640, 360);
        let mut ui: Ui<LayoutAct> = Ui::new(&mut g, (-1, -1), true);
        e.draw_stage(&mut ui, &|a| a);
        drop(ui);
        g.canvas.px
    };
    e.grid = false;
    e.labels = false;
    e.rings = false;
    let dot = draw(&e);
    let red = Rc::new(Image { w: 14, h: 14, px: [230u8, 20, 20, 255].repeat(14 * 14) });
    let i = e.markers().iter().position(|m| m.id == "b").unwrap();
    let mut markers = e.markers().to_vec();
    markers[i].pin = Some(red);
    let mut with_pin = LayoutEditor::new(markers, e.polylines().to_vec()).with_view(VIEW);
    with_pin.marker_box = e.marker_box;
    (with_pin.grid, with_pin.labels, with_pin.rings) = (false, false, false);
    let pinned = draw(&with_pin);
    let px = |buf: &[u8], x: i32, y: i32| {
        let k = ((y * 640 + x) * 4) as usize;
        [buf[k], buf[k + 1], buf[k + 2]]
    };
    // a corner of the pin box (inside it, away from the dot and the anchors)
    let (cx, cy) = (pin.x + 1, pin.y + 1);
    assert_eq!(px(&pinned, cx, cy), [230, 20, 20], "the caller's pin fills its 14 px box");
    assert_ne!(px(&dot, cx, cy), [230, 20, 20]);
    assert_eq!(with_pin.state()["markers"][i]["pin"], json!(true), "the agent link says it has a pin");
    assert!(e.state()["markers"][i].get("pin").is_none(), "no pin: no key");
    // marker_art off: the caller drew the pictures, the stage leaves the box as it was
    let mut bare = with_pin.clone();
    bare.marker_art = false;
    assert_ne!(px(&draw(&bare), cx, cy), [230, 20, 20], "no picture drawn");
    assert_eq!(handles(&bare), handles(&with_pin), "the same handles");
}

#[test]
fn the_agent_link_names_a_polylines_ends() {
    let e = ended();
    let s = e.state();
    assert_eq!(s["routes"][0], json!({"id": "r1", "points": [[0.1, 0.1], [0.3, 0.2], [0.6, 0.2]], "ends": ["a", "b"]}));
    assert!(s["routes"][2].get("ends").is_none(), "a free line has none");
    assert_eq!(editor().state()["routes"][0], json!({"id": "r1", "points": [[0.1, 0.1], [0.3, 0.2], [0.6, 0.2]]}), "unchanged for a line without ends");
}
