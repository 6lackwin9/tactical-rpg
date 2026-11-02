//! LAYOUT720-E2: `shot_diff` (docs/specs/LAYOUT720.md section 5): a pixel matches if a pixel of the other image within
//! `ceil(s)` is within the channel tolerance; any other fails, with the bounding box. Rule-built images first, then real
//! drawing (a panel, text and engine art through the painter), and `Rect` as `aegis_ui::canvas` still gives it.

use aegis_ui::art::load_png;
use aegis_ui::canvas::{rgb, Canvas, Image, Rect};
use aegis_ui::painter::{Fonts, Painter, View};
use aegis_ui::shot_diff::{shot_diff, CHANNEL_TOLERANCE, MAX_SCALE};
use std::path::{Path, PathBuf};

const GROUND: [u8; 4] = [40, 50, 60, 255];
const MARK: [u8; 4] = [200, 180, 20, 255];

fn solid(w: i32, h: i32, c: [u8; 4]) -> Image {
    Image { w, h, px: c.iter().copied().cycle().take((w * h * 4) as usize).collect() }
}

fn with(mut i: Image, x: i32, y: i32, c: [u8; 4]) -> Image {
    let o = ((y * i.w + x) * 4) as usize;
    i.px[o..o + 4].copy_from_slice(&c);
    i
}

fn engine_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

#[test]
fn identical_shots_match() {
    let a = with(solid(40, 30, GROUND), 5, 6, MARK);
    let d = shot_diff(&a, &a.clone(), 1.5).unwrap();
    assert!(d.passed());
    assert_eq!((d.unmatched, d.bbox), (0, None));
}

#[test]
fn a_channel_may_differ_by_the_tolerance_and_no_more() {
    let t = CHANNEL_TOLERANCE;
    for channel in 0..4 {
        let mut near = GROUND;
        near[channel] = if channel == 3 { 255 - t } else { near[channel] + t };
        assert!(shot_diff(&solid(9, 9, GROUND), &solid(9, 9, near), 1.0).unwrap().passed(), "channel {channel} by the tolerance");
        let mut far = GROUND;
        far[channel] = if channel == 3 { 255 - t - 1 } else { far[channel] + t + 1 };
        assert!(!shot_diff(&solid(9, 9, GROUND), &solid(9, 9, far), 1.0).unwrap().passed(), "channel {channel} past the tolerance");
    }
}

#[test]
fn a_moved_pixel_matches_within_ceil_s_and_not_past_it() {
    for (s, radius) in [(1.0f32, 1), (1.067, 2), (1.5, 2), (2.0, 2), (3.0, 3)] {
        let at = |x, y| with(solid(41, 41, GROUND), x, y, MARK);
        let d = shot_diff(&at(20, 20), &at(20 + radius, 20), s).unwrap();
        assert!(d.passed(), "s {s}: {d}");
        assert_eq!(d.radius, radius);
        assert!(shot_diff(&at(20, 20), &at(20, 20 - radius), s).unwrap().passed(), "s {s}: up");
        assert!(shot_diff(&at(20, 20), &at(20 + radius, 20 + radius), s).unwrap().passed(), "s {s}: an edge moved on both axes");
        assert!(!shot_diff(&at(20, 20), &at(20 + radius + 1, 20), s).unwrap().passed(), "s {s}: one further across");
        assert!(!shot_diff(&at(20, 20), &at(20, 20 + radius + 1), s).unwrap().passed(), "s {s}: one further down");
    }
}

#[test]
fn a_mark_in_either_shot_alone_fails_with_its_box() {
    let plain = solid(40, 30, GROUND);
    let marked = with(plain.clone(), 12, 7, MARK);
    // The mark's pixel has no match in the plain shot; the plain one's pixels all find ground somewhere.
    for (a, b) in [(&plain, &marked), (&marked, &plain)] {
        let d = shot_diff(a, b, 1.0).unwrap();
        assert!(!d.passed());
        assert_eq!(d.unmatched, 1);
        assert_eq!(d.bbox, Some(Rect::new(12, 7, 1, 1)));
    }
}

#[test]
fn the_box_covers_every_unmatched_pixel_from_both_shots() {
    let a = with(solid(40, 30, GROUND), 3, 25, MARK);
    let b = with(solid(40, 30, GROUND), 30, 2, MARK);
    let d = shot_diff(&a, &b, 1.0).unwrap();
    assert_eq!(d.unmatched, 2);
    assert_eq!(d.bbox, Some(Rect::new(3, 2, 28, 24)));
    let line = d.to_string();
    for part in ["2 pixels", "x 3", "y 2", "w 28", "h 24"] {
        assert!(line.contains(part), "{line}");
    }
}

#[test]
fn a_mismatch_in_a_corner_stays_inside_the_image() {
    let d = shot_diff(&with(solid(5, 5, GROUND), 0, 0, MARK), &with(solid(5, 5, GROUND), 4, 4, MARK), 1.0).unwrap();
    assert_eq!(d.unmatched, 2);
    assert_eq!(d.bbox, Some(Rect::new(0, 0, 5, 5)));
}

#[test]
fn what_cannot_be_compared_is_refused() {
    let good = solid(4, 3, GROUND);
    assert!(shot_diff(&good, &solid(3, 4, GROUND), 1.0).unwrap_err().to_string().contains("size"), "different sizes");
    let short = Image { w: 4, h: 3, px: vec![0; 47] };
    assert!(shot_diff(&short, &good, 1.0).is_err(), "first shot short of its size");
    assert!(shot_diff(&good, &short, 1.0).is_err(), "second shot short of its size");
    assert!(shot_diff(&Image { w: -4, h: -3, px: vec![0; 48] }, &good, 1.0).is_err(), "negative sizes");
    for s in [0.0, -1.0, f32::NAN, f32::INFINITY, MAX_SCALE + 1.0] {
        assert!(shot_diff(&good, &good, s).is_err(), "scale {s}");
    }
    assert!(shot_diff(&good, &good, MAX_SCALE).is_ok(), "the largest scale taken");
    let none = solid(0, 0, GROUND);
    assert!(shot_diff(&none, &none, 1.0).unwrap().passed(), "two empty shots");
}

/// A panel, a line of text and a piece of engine art drawn `by` units right and down of their place.
fn real_shot(window: (u32, u32), by: i32, art: &Image) -> (Image, f32) {
    let font = std::fs::read(engine_root().join("fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut fonts = Fonts::new(&font);
    let mut c = Canvas::new(window.0 as i32, window.1 as i32);
    c.clear(rgb(24, 26, 32));
    let v = View::fit(window.0, window.1);
    {
        let mut p = Painter { c: &mut c, v, fonts: &mut fonts };
        p.panel(Rect::new(40 + by, 30 + by, 200, 100), rgb(70, 74, 90), rgb(190, 190, 200), 4.0);
        p.text(50.0 + by as f32, 40.0 + by as f32, "The quick brown fox", 14.0, rgb(240, 240, 240));
        p.image(art, Rect::new(60 + by, 70 + by, 32, 32));
    }
    (Image { w: c.w, h: c.h, px: c.px }, v.s)
}

#[test]
fn real_drawing_moved_one_unit_matches_and_two_units_does_not() {
    let art = load_png(&engine_root().join("tilesets/common/bridge/v0.png")).expect("engine art");
    for window in [(1280, 720), (1920, 1080)] {
        let (before, s) = real_shot(window, 0, &art);
        assert!(shot_diff(&before, &before.clone(), s).unwrap().passed());
        let (one, _) = real_shot(window, 1, &art);
        let d = shot_diff(&before, &one, s).unwrap();
        assert!(d.passed(), "{window:?}, one unit: {d}");
        let (two, _) = real_shot(window, 2, &art);
        let d = shot_diff(&before, &two, s).unwrap();
        assert!(!d.passed(), "{window:?}, two units");
        // Everything that moved lies around the drawn group, in physical px.
        let group = v_rect(window, Rect::new(40, 30, 202, 102));
        let b = d.bbox.expect("a box");
        assert!(b.x >= group.x && b.y >= group.y && b.x + b.w <= group.x + group.w && b.y + b.h <= group.y + group.h, "{window:?}: {b:?} outside {group:?}");
    }
}

fn v_rect(window: (u32, u32), r: Rect) -> Rect {
    View::fit(window.0, window.1).rect(r)
}

#[test]
fn rect_is_the_layout_crates_rect_with_the_same_api() {
    fn same(r: aegis_ui::canvas::Rect) -> aegis_layout::Rect {
        r
    }
    const CONST_NEW: Rect = Rect::new(1, 2, 3, 4);
    assert_eq!(same(CONST_NEW), aegis_layout::Rect { x: 1, y: 2, w: 3, h: 4 });
    assert_eq!(CONST_NEW.inset(1), Rect::new(2, 3, 1, 2));
    assert!(CONST_NEW.contains(1, 2));
    assert_eq!(CONST_NEW.intersect(Rect::new(0, 0, 2, 3)), Rect::new(1, 2, 1, 1));
}
