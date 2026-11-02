//! LOOK1-FT: the frame tuner (`aegis_ui::frame_tuner`) and the sidecar keys it adds, on scratch asset layers written by the
//! test. Sliders stay in bounds; an override touches neither the theme nor a file; Save merges into the sidecar, which loads
//! and stays byte-stable when nothing changed; Revert restores; a release build refuses writes; extreme values draw inside
//! the preview; the new keys draw what they say and are refused out of bounds.

use aegis_core::assets::Layers;
use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::frame_tuner::{find_tip, merged_sidecar, sidecar_text, writes_allowed, FrameTuner, Param, TunerAct, HEX_FIELD, PARAMS};
use aegis_ui::painter::{nine_art_line, nine_content_rect, Fonts, Painter, View};
use aegis_ui::screen::Key;
use aegis_ui::specimens::{KitAct, Specimens, PAGES};
use aegis_ui::theme::{self, line_colour, Nine, NineFill, CORNER_SIZE, FRAME_ROLES, LINE_WIDTH};
use aegis_ui::ui::{Gfx, Kit, Ui, BG};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const LINE: Rgba = [153, 27, 27, 255];
const ARM: Rgba = [200, 160, 60, 255];
const SIDE: &str = "{\"slice\": [32, 32, 32, 32], \"corner_anchor\": [8, 8], \"scale\": 4, \"pad\": [2, 2, 2, 2], \"straddle\": true}\n";

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

fn put(img: &mut Image, x: i32, y: i32, c: Rgba) {
    let at = ((y * img.w + x) * 4) as usize;
    img.px[at..at + 4].copy_from_slice(&c);
}

/// An 80 x 80 straddling frame: a corner with the anchor at (8, 8), an arm 3 px thick along each side and, at `tip`, a head
/// that widens to 11 px at x 24 and comes to a point at x 31 (else the arm runs on to the slice's end); a 4 px line in the
/// middle of each edge band, if `line`; a dark centre.
fn straddle_art(tip: bool, line: bool) -> Image {
    let mut img = Image { w: 80, h: 80, px: vec![0; 80 * 80 * 4] };
    let mut corner = |a: i32, b: i32| {
        for (fx, fy) in [(false, false), (true, false), (false, true), (true, true)] {
            for (x, y) in [(a, b), (b, a)] {
                put(&mut img, if fx { 79 - x } else { x }, if fy { 79 - y } else { y }, ARM);
            }
        }
    };
    for along in 8..32 {
        let half = if !tip || along < 24 { 1 } else { 5 - (along - 24) * 5 / 7 };
        for off in -half..=half {
            corner(along, 8 + off);
        }
    }
    for y in 32..48 {
        for x in 32..48 {
            put(&mut img, x, y, [20, 20, 28, 255]);
        }
    }
    if line {
        for along in 32..48 {
            for off in 14..18 {
                for (x, y) in [(along, off), (along, 79 - off), (off, along), (79 - off, along)] {
                    put(&mut img, x, y, LINE);
                }
            }
        }
    }
    img
}

/// A plain 16 x 16 frame cut 4 px in, drawn inside its rect.
fn plain_art() -> Image {
    let mut img = Image { w: 16, h: 16, px: vec![0; 16 * 16 * 4] };
    for y in 0..16 {
        for x in 0..16 {
            let edge = !(4..12).contains(&x) || !(4..12).contains(&y);
            put(&mut img, x, y, if edge { ARM } else { [20, 20, 28, 255] });
        }
    }
    img
}

fn write_png(path: &Path, img: &Image) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img.px).unwrap();
}

/// One asset layer with `ornate` (a tip and a line), `ornate_full` (tiled, an arm to the end) and `panel` (inside its rect),
/// the theme installed from it. Its folder is the test's.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aegis_frame_tuner_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let frames = dir.join("ui/frames");
    write_png(&frames.join("ornate.png"), &straddle_art(true, true));
    std::fs::write(frames.join("ornate.json"), SIDE).unwrap();
    write_png(&frames.join("ornate_full.png"), &straddle_art(false, true));
    std::fs::write(frames.join("ornate_full.json"), SIDE.replace("\"straddle\": true", "\"straddle\": true, \"edges\": \"tile\"")).unwrap();
    write_png(&frames.join("panel.png"), &plain_art());
    std::fs::write(frames.join("panel.json"), "{\"slice\": [4, 4, 4, 4], \"scale\": 2}\n").unwrap();
    let layers = Layers::of(vec![dir.clone()]);
    theme::install(theme::load(&layers).expect("the scratch theme loads"));
    theme::set_layers(layers);
    dir
}

fn side_bytes(dir: &Path, role: &str) -> Vec<u8> {
    std::fs::read(dir.join("ui/frames").join(format!("{role}.json"))).unwrap()
}

fn tuner_on(role: &str) -> FrameTuner {
    let mut t = FrameTuner::default();
    t.role = FRAME_ROLES.iter().position(|r| r.0 == role).unwrap();
    t
}

fn straddling() -> Nine {
    Nine { straddle: true, corner_anchor: Some([8, 8]), scale: 4, ..Nine::new([32; 4]) }
}

fn gfx(w: i32, h: i32, text: f32) -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(w, h), view: View::fit(w as u32, h as u32), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.fonts.scale = text;
    g.canvas.clear(BG);
    g
}

/// `nine` on the frame `img` over the logical `panel`, on a clear canvas at view 2.
fn drawn(img: &Image, nine: &Nine, panel: Rect) -> Canvas {
    let mut c = Canvas::new(400, 400);
    let mut fonts = Fonts::new(&font());
    let mut p = Painter { c: &mut c, v: View { s: 2.0, ox: 0.0, oy: 0.0, g: 1 }, fonts: &mut fonts };
    p.nine(img, nine, panel);
    c
}

fn alpha(c: &Canvas, x: i32, y: i32) -> u8 {
    c.px[((y * c.w + x) * 4 + 3) as usize]
}

/// How many physical px of `c` have anything on them.
fn drawn_count(c: &Canvas) -> usize {
    c.px.chunks(4).filter(|p| p[3] > 0).count()
}

/// The largest x below `limit` with anything on row `y`, or -1.
fn rightmost_on_row(c: &Canvas, y: i32, limit: i32) -> i32 {
    (0..limit.min(c.w)).rev().find(|x| alpha(c, *x, y) > 0).unwrap_or(-1)
}

#[test]
fn sliders_stay_within_their_bounds() {
    let _dir = scratch("bounds");
    for p in PARAMS {
        let (lo, hi, _) = p.range();
        assert_eq!(p.value(0), lo, "{p:?}");
        assert_eq!(p.value(usize::MAX), hi, "{p:?}: past the end is the end");
        assert_eq!(p.value(p.steps() - 1), hi, "{p:?}");
        for i in 0..p.steps() {
            assert_eq!(p.step_of(p.value(i)), i, "{p:?} step {i}");
            assert!((lo..=hi).contains(&p.value(i)), "{p:?} step {i}");
        }
        assert_eq!(p.step_of(hi + 1000.0), p.steps() - 1, "{p:?}");
        assert_eq!(p.step_of(lo - 1000.0), 0, "{p:?}");
        let mut t = tuner_on("ornate");
        t.act(TunerAct::Slide(p, usize::MAX));
        assert_eq!(t.value(p), Some(hi), "{p:?} on the frame");
        t.act(TunerAct::Slide(p, 0));
        assert_eq!(t.value(p), Some(lo), "{p:?} on the frame");
    }
    assert_eq!(Param::Corner.range().0, CORNER_SIZE.0);
    assert_eq!(Param::Line.range().1, LINE_WIDTH.1);
}

#[test]
fn every_value_a_slider_reaches_loads_when_saved() {
    let dir = scratch("reach");
    for (p, role) in [(Param::Corner, "ornate"), (Param::Tip, "ornate"), (Param::Line, "ornate"), (Param::Pad, "ornate"), (Param::Corner, "panel"), (Param::Pad, "panel")] {
        for i in [0, p.steps() / 3, p.steps() - 1] {
            let mut t = tuner_on(role);
            t.act(TunerAct::Slide(p, i));
            if writes_allowed() && t.changed() {
                t.save().unwrap_or_else(|e| panic!("{p:?} step {i}: {e}"));
                assert!(!t.changed());
                assert_eq!(t.value(p), Some(p.value(i)), "{p:?} step {i} after the reload");
                t.revert().unwrap();
            }
        }
    }
    assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
}

#[test]
fn an_override_touches_neither_the_theme_nor_the_file() {
    let dir = scratch("override");
    let before = theme::loaded().frames["ornate"].nine.clone();
    let mut t = tuner_on("ornate");
    t.act(TunerAct::Slide(Param::Corner, Param::Corner.steps() - 1));
    t.act(TunerAct::Slide(Param::Pad, 9));
    t.act(TunerAct::Straddle);
    assert!(t.changed());
    assert_ne!(t.frame().unwrap().nine, before);
    assert_eq!(theme::loaded().frames["ornate"].nine, before);
    assert_eq!(theme::th().frames["ornate"].nine, before);
    assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    // Back at the file's values there is no override at all.
    t.act(TunerAct::Straddle);
    t.act(TunerAct::Slide(Param::Corner, Param::Corner.step_of(1.0)));
    t.act(TunerAct::Slide(Param::Pad, Param::Pad.step_of(2.0)));
    assert!(!t.changed(), "{:?}", t.frame().unwrap().nine);
}

#[test]
fn the_writer_keeps_the_art_tools_form_and_key_order() {
    let value: Value = serde_json::from_str(SIDE).unwrap();
    assert_eq!(sidecar_text(&value) + "\n", SIDE, "the writer's text of an art tool file is that file");
    let was = straddling();
    let was = Nine { pad: [2; 4], ..was };
    assert_eq!(merged_sidecar(&value, &was, &was), None, "nothing changed");
    let tuned = Nine { corner_size: 1.35, pad: [5; 4], ..was.clone() };
    let merged = merged_sidecar(&value, &was, &tuned).unwrap();
    let text = sidecar_text(&merged);
    assert_eq!(text, "{\"slice\": [32, 32, 32, 32], \"corner_anchor\": [8, 8], \"scale\": 4, \"pad\": [5, 5, 5, 5], \"straddle\": true, \"corner_size\": 1.35}");
    assert_eq!(Nine::parse(&merged, 80, 80, "t").unwrap(), tuned);
    // A value back at the default leaves the file.
    let back = merged_sidecar(&merged, &tuned, &Nine { corner_size: 1.0, ..tuned.clone() }).unwrap();
    assert!(back.get("corner_size").is_none());
}

#[test]
fn save_writes_a_sidecar_that_loads_and_is_byte_stable_when_unchanged() {
    if !writes_allowed() {
        return;
    }
    let dir = scratch("save");
    let mut t = tuner_on("ornate");
    // Nothing tuned: nothing written.
    assert!(t.save().unwrap().contains("nothing"), "{}", t.note);
    assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    // Moved and moved back: still nothing.
    t.act(TunerAct::Slide(Param::Corner, 0));
    t.act(TunerAct::Slide(Param::Corner, Param::Corner.step_of(1.0)));
    t.act(TunerAct::Save);
    assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    // A real change lands, loads through the theme, and is in force.
    t.act(TunerAct::Slide(Param::Corner, Param::Corner.step_of(1.5)));
    t.act(TunerAct::Save);
    assert!(t.note.starts_with("saved"), "{}", t.note);
    assert!(!t.changed());
    let saved = side_bytes(&dir, "ornate");
    assert_ne!(saved, SIDE.as_bytes());
    assert_eq!(theme::loaded().frames["ornate"].nine.corner_size, 1.5);
    assert_eq!(theme::th().frames["ornate"].nine.corner_size, 1.5);
    let again = theme::load(&Layers::of(vec![dir.clone()])).expect("the saved sidecar loads");
    assert_eq!(again.frames["ornate"].nine.corner_size, 1.5);
    // Every other key is as it was.
    let (old, new): (Value, Value) = (serde_json::from_str(SIDE).unwrap(), serde_json::from_slice(&saved).unwrap());
    for (k, v) in old.as_object().unwrap() {
        assert_eq!(new.get(k), Some(v), "{k}");
    }
    // Saved again unchanged, by this tuner or a fresh one: the same bytes.
    t.act(TunerAct::Save);
    assert_eq!(side_bytes(&dir, "ornate"), saved);
    let mut fresh = tuner_on("ornate");
    fresh.act(TunerAct::Slide(Param::Pad, 3));
    fresh.act(TunerAct::Slide(Param::Pad, 2));
    fresh.act(TunerAct::Save);
    assert_eq!(side_bytes(&dir, "ornate"), saved);
    // Other frames' files stay.
    assert_eq!(side_bytes(&dir, "panel"), b"{\"slice\": [4, 4, 4, 4], \"scale\": 2}\n");
}

#[test]
fn a_saved_tip_and_line_load_and_draw_the_same() {
    if !writes_allowed() {
        return;
    }
    let dir = scratch("tipline");
    let mut t = tuner_on("ornate");
    t.act(TunerAct::Slide(Param::Tip, Param::Tip.step_of(1.5)));
    t.act(TunerAct::Slide(Param::Line, Param::Line.step_of(3.0)));
    t.act(TunerAct::Colour(1 + theme::ROLES.iter().position(|r| *r == "gold").unwrap()));
    let tuned = t.frame().unwrap();
    let want = drawn(&tuned.img, &tuned.nine, Rect::new(30, 30, 60, 40));
    t.act(TunerAct::Save);
    assert!(t.note.starts_with("saved"), "{}", t.note);
    let f = theme::load(&Layers::of(vec![dir])).unwrap().frames["ornate"].clone();
    assert_eq!(f.nine.line_color.as_ref().map(|c| c.rgba), Some(theme::loaded().gold), "the role takes the theme's colour");
    assert_eq!(f.nine, tuned.nine);
    assert_eq!(drawn(&f.img, &f.nine, Rect::new(30, 30, 60, 40)).px, want.px);
}

#[test]
fn revert_drops_the_override_then_restores_the_file() {
    let dir = scratch("revert");
    let mut t = tuner_on("ornate");
    t.act(TunerAct::Slide(Param::Corner, 0));
    assert!(t.changed());
    t.act(TunerAct::Revert);
    assert!(!t.changed());
    assert_eq!(t.frame().unwrap().nine, theme::loaded().frames["ornate"].nine);
    assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    if !writes_allowed() {
        return;
    }
    t.act(TunerAct::Slide(Param::Corner, 0));
    t.act(TunerAct::Save);
    assert_ne!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    // A second save, then Revert with a change unsaved: the change goes first, the file stays.
    t.act(TunerAct::Slide(Param::Pad, 7));
    t.act(TunerAct::Save);
    let second = side_bytes(&dir, "ornate");
    t.act(TunerAct::Slide(Param::Pad, 8));
    t.act(TunerAct::Revert);
    assert_eq!(side_bytes(&dir, "ornate"), second);
    assert_eq!(theme::loaded().frames["ornate"].nine.pad, [7; 4]);
    // The next Revert goes back to before the first save, and the theme with it.
    t.act(TunerAct::Revert);
    assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    assert_eq!(theme::loaded().frames["ornate"].nine.corner_size, 1.0);
    assert_eq!(theme::loaded().frames["ornate"].nine.pad, [2; 4]);
    assert_eq!(t.frame().unwrap().nine, theme::loaded().frames["ornate"].nine);
}

#[test]
fn a_release_build_refuses_writes() {
    assert_eq!(writes_allowed(), cfg!(debug_assertions));
    let dir = scratch("release");
    let mut t = tuner_on("ornate");
    t.act(TunerAct::Slide(Param::Corner, 0));
    let refused = t.save_as(false);
    assert!(refused.as_ref().is_err_and(|e| e.contains("release")), "{refused:?}");
    assert!(t.changed(), "a refused save keeps the override");
    assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    if !writes_allowed() {
        // Run under `cargo test --release`: the act itself writes nothing, with the override in place.
        t.act(TunerAct::Save);
        assert!(t.note.contains("release"), "{}", t.note);
        assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
        assert!(t.save_as(true).is_err(), "no caller gets past the build");
        assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    }
}

#[test]
fn a_save_the_theme_refuses_puts_the_file_back() {
    if !writes_allowed() {
        return;
    }
    let dir = scratch("refused");
    // Another file broken behind the tuner's back: the reload fails, so the sidecar goes back and the theme stays.
    std::fs::write(dir.join("ui/frames/panel.json"), "{\"slice\": [40, 40, 40, 40]}").unwrap();
    let mut t = tuner_on("ornate");
    t.act(TunerAct::Slide(Param::Corner, 0));
    let refused = t.save();
    assert!(refused.as_ref().is_err_and(|e| e.contains("put back")), "{refused:?}");
    assert_eq!(side_bytes(&dir, "ornate"), SIDE.as_bytes());
    assert_eq!(theme::loaded().frames["ornate"].nine.corner_size, 1.0);
    assert!(t.changed());
}

#[test]
fn a_role_without_a_file_tunes_the_file_that_draws_it_and_a_vector_panel_refuses() {
    let dir = scratch("fallback");
    // `tip` falls back to `panel`.
    let mut t = tuner_on("tip");
    assert!(t.frame().unwrap().source.contains("panel"));
    t.act(TunerAct::Slide(Param::Pad, 5));
    assert!(t.changed());
    if writes_allowed() {
        t.act(TunerAct::Save);
        assert!(t.note.contains("panel.json"), "{}", t.note);
        assert_ne!(side_bytes(&dir, "panel"), b"{\"slice\": [4, 4, 4, 4], \"scale\": 2}\n");
        assert!(!dir.join("ui/frames/tip.json").exists());
    }
    // A theme with no frame files: every frame control does nothing and nothing panics.
    let empty = std::env::temp_dir().join(format!("aegis_frame_tuner_empty_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).unwrap();
    let layers = Layers::of(vec![empty]);
    theme::install(theme::load(&layers).unwrap());
    theme::set_layers(layers);
    let mut t = tuner_on("ornate");
    assert!(t.frame().is_none());
    t.act(TunerAct::Slide(Param::Corner, 0));
    t.act(TunerAct::Straddle);
    t.act(TunerAct::Colour(1));
    t.hex_changed("#112233");
    assert!(t.save_as(true).is_err());
    assert!(t.revert().is_err());
    assert!(!t.changed());
    assert_eq!(t.value(Param::Corner), None);
    let mut g = gfx(1280, 720, 1.0);
    let mut ui: Ui<TunerAct> = Ui::new(&mut g, (-1, -1), true);
    t.draw(&mut ui, Rect::new(112, 22, 522, 314), &|a| a);
}

#[test]
fn the_hex_field_takes_a_colour_and_refuses_the_rest() {
    let _dir = scratch("hex");
    let mut t = tuner_on("ornate");
    for bad in ["", "#", "#12", "red", "#gggggg", "#1234567"] {
        t.hex_changed(bad);
        assert_eq!(t.hex, bad);
        assert!(!t.changed(), "{bad}");
    }
    t.hex_changed("#1234567890");
    assert_eq!(t.hex, "#1234567", "too long is refused, the text stays");
    t.hex_changed("#00FF00");
    let c = t.frame().unwrap().nine.line_color.unwrap();
    assert_eq!((c.name.as_str(), c.rgba), ("#00ff00", [0, 255, 0, 255]));
    t.act(TunerAct::Colour(0));
    assert!(t.frame().unwrap().nine.line_color.is_none());
    assert!(!t.changed(), "the art's own colour again is no change");
    // A frame with no plain line takes none.
    let mut t = tuner_on("ornate_full");
    t.hex_changed("#00ff00");
    assert!(t.frame().unwrap().nine.line_color.is_none());
    assert_eq!(t.value(Param::Line), None);
    // And its index past the roles is harmless.
    let mut t = tuner_on("ornate");
    t.act(TunerAct::Colour(usize::MAX));
    assert!(t.frame().unwrap().nine.line_color.is_none());
}

#[test]
fn extreme_values_draw_inside_the_preview_without_a_panic() {
    let _dir = scratch("extreme");
    let lows = |t: &mut FrameTuner| PARAMS.iter().for_each(|p| t.act(TunerAct::Slide(*p, 0)));
    let highs = |t: &mut FrameTuner| PARAMS.iter().for_each(|p| t.act(TunerAct::Slide(*p, usize::MAX)));
    let stage = Rect::new(100, 60, 120, 80);
    for (w, h) in [(640, 360), (1280, 720), (1920, 1080)] {
        for text in [0.85, 1.3] {
            for role in ["ornate", "ornate_full", "panel", "inset"] {
                for set in [&lows as &dyn Fn(&mut FrameTuner), &highs] {
                    for straddle in [false, true] {
                        let mut t = tuner_on(role);
                        set(&mut t);
                        if straddle {
                            t.act(TunerAct::Straddle);
                        }
                        let mut g = gfx(w, h, text);
                        let blank = g.canvas.px.clone();
                        {
                            let mut ui: Ui<TunerAct> = Ui::new(&mut g, (-1, -1), true);
                            t.draw_preview(&mut ui, stage);
                        }
                        let v = g.view;
                        let phys = v.rect(stage);
                        for y in 0..g.canvas.h {
                            for x in 0..g.canvas.w {
                                if !phys.contains(x, y) {
                                    let at = ((y * g.canvas.w + x) * 4) as usize;
                                    assert_eq!(g.canvas.px[at..at + 4], blank[at..at + 4], "{role} {w}x{h} text {text} straddle {straddle}: drew at ({x}, {y}) outside {phys:?}");
                                }
                            }
                        }
                        // The whole pane too, the sliders at their ends.
                        let mut ui: Ui<TunerAct> = Ui::new(&mut g, (-1, -1), true);
                        t.draw(&mut ui, Rect::new(112, 22, 522, 314), &|a| a);
                    }
                }
            }
        }
    }
}

#[test]
fn a_panel_too_small_for_its_corners_keeps_them_apart_at_any_size() {
    let img = straddle_art(true, true);
    for corner_size in [CORNER_SIZE.0, 1.0, CORNER_SIZE.1] {
        for tip_size in [CORNER_SIZE.0, 1.0, CORNER_SIZE.1] {
            let nine = Nine { corner_size, tip: Some([24, 7]), tip_size, ..straddling() };
            let tiny = drawn(&img, &nine, Rect::new(40, 40, 16, 16));
            assert!(drawn_count(&tiny) > 0);
            let content = nine_content_rect(&nine, Rect::new(0, 0, 16, 16), 2.0);
            assert!(content.w >= 0 && content.h >= 0);
        }
    }
}

#[test]
fn corner_size_draws_the_corners_larger_from_the_panel_corner() {
    let img = straddle_art(true, true);
    let base = straddling();
    let panel = Rect::new(40, 40, 60, 60);
    // On the row just above the panel's top (the 1 px line is on the top row itself), the end of the top left corner's arm,
    // left of the panel's middle at x 140.
    let at = |size: f32| rightmost_on_row(&drawn(&img, &Nine { corner_size: size, line_width: Some(0.5), ..base.clone() }, panel), 80 - 1, 140);
    let (small, one, big) = (at(0.5), at(1.0), at(2.0));
    println!("corner reach on a row above the panel: {small} {one} {big}");
    assert!(small < one && one < big, "{small} {one} {big}");
    // The default is the art's size to the pixel.
    assert_eq!(drawn(&img, &Nine { corner_size: 1.0, ..base.clone() }, panel).px, drawn(&img, &base, panel).px);
    // Inside its rect, the corners grow with the bands.
    let plain = plain_art();
    let inside = Nine { scale: 2, ..Nine::new([4; 4]) };
    let count = |n: &Nine| drawn(&plain, n, panel).px.chunks(4).filter(|p| p[0] == ARM[0]).count();
    assert!(count(&Nine { corner_size: 2.0, ..inside.clone() }) > count(&inside));
    assert!(count(&Nine { corner_size: 0.5, ..inside.clone() }) < count(&inside));
    assert_eq!(drawn(&plain, &Nine { corner_size: 1.0, ..inside.clone() }, panel).px, drawn(&plain, &inside, panel).px);
}

#[test]
fn the_content_rect_clears_the_corners_at_their_drawn_size() {
    let base = Nine { corner_inner: Some([20, 20]), pad: [0; 4], ..straddling() };
    let r = Rect::new(0, 0, 200, 200);
    let inset = |size: f32| nine_content_rect(&Nine { corner_size: size, ..base.clone() }, r, 2.0).x;
    assert!(inset(2.0) > inset(1.0) && inset(1.0) >= inset(0.5), "{} {} {}", inset(0.5), inset(1.0), inset(2.0));
}

#[test]
fn a_tip_is_found_on_art_with_one_and_not_on_a_plain_arm() {
    let tipped = straddle_art(true, true);
    assert_eq!(find_tip(&tipped, &straddling()).map(|t| t[0]), Some(24), "the head's base");
    assert!(find_tip(&tipped, &straddling()).unwrap()[1] >= 5, "the head's half width is inside");
    assert_eq!(find_tip(&straddle_art(false, true), &straddling()), None, "an arm to the slice's end");
    assert_eq!(find_tip(&tipped, &Nine { straddle: false, ..straddling() }), None, "a frame inside its rect");
}

#[test]
fn tip_size_moves_the_head_and_leaves_the_arm_and_the_rest() {
    let img = straddle_art(true, true);
    let tip = find_tip(&img, &straddling());
    let base = Nine { tip, ..straddling() };
    let panel = Rect::new(60, 60, 100, 100);
    // At view 2, scale 4: half a physical px per art px; the anchor's row is the panel's top. Two rows above it the line
    // and the arm are gone and only the head is left, of the top left corner to the left of the panel's middle.
    let row = 60 * 2;
    let reach = |size: f32| rightmost_on_row(&drawn(&img, &Nine { tip_size: size, ..base.clone() }, panel), row - 2, 220);
    let (small, one, big) = (reach(0.5), reach(1.0), reach(2.0));
    println!("the head's end on the anchor's row: {small} {one} {big}");
    assert!(small < one && one < big, "{small} {one} {big}");
    // At 1 the head draws as the art, with or without the key.
    assert_eq!(drawn(&img, &base, panel).px, drawn(&img, &straddling(), panel).px);
    // The arm's root and the far corner's art do not move: the anchor's neighbourhood is the same.
    let (a, b) = (drawn(&img, &base, panel), drawn(&img, &Nine { tip_size: 2.0, ..base.clone() }, panel));
    for y in (row - 2)..(row + 2) {
        for x in 120..128 {
            assert_eq!(alpha(&a, x, y), alpha(&b, x, y), "near the anchor at ({x}, {y})");
        }
    }
    // The head's base stays where the art has it (the corner is not stretched to the larger size).
    let base_x = |c: &Canvas| (120..220).find(|x| alpha(c, *x, row - 2) > 0).unwrap_or(-1);
    assert!((base_x(&a) - base_x(&b)).abs() <= 1, "the base moved: {} {}", base_x(&a), base_x(&b));
    // Four corners, mirrored: the head grows inward on the right too.
    let left_of = |c: &Canvas| (200..c.w).find(|x| alpha(c, *x, row - 2) > 0).unwrap_or(c.w);
    let wide = Rect::new(20, 60, 160, 40);
    assert!(left_of(&drawn(&img, &Nine { tip_size: 2.0, ..base.clone() }, wide)) < left_of(&drawn(&img, &base, wide)));
}

#[test]
fn the_line_keys_draw_a_line_of_that_width_and_colour() {
    let img = straddle_art(true, true);
    assert_eq!(nine_art_line(&img, &straddling()), Some((1.0, LINE)), "4 art px at scale 4");
    assert_eq!(nine_art_line(&img, &Nine { straddle: false, ..straddling() }), None);
    assert_eq!(nine_art_line(&img, &Nine { edges: NineFill::Tile, ..straddling() }), None);
    assert_eq!(nine_art_line(&straddle_art(true, false), &straddling()), None);
    let base = straddling();
    let panel = Rect::new(40, 40, 60, 60);
    // Down through the middle of the top edge, at view 2.
    let column = |c: &Canvas| -> Vec<Rgba> {
        (60..100).map(|y| { let i = ((y * c.w + 140) * 4) as usize; [c.px[i], c.px[i + 1], c.px[i + 2], c.px[i + 3]] }).filter(|p| p[3] > 0 && p[..3] != [20, 20, 28]).collect()
    };
    let art = column(&drawn(&img, &base, panel));
    println!("the art's line, physical px: {art:?}");
    assert_eq!(art.len(), 2, "1 logical px at view 2: {art:?}");
    let wide = column(&drawn(&img, &Nine { line_width: Some(3.0), ..base.clone() }, panel));
    assert_eq!(wide.len(), 6, "3 logical px at view 2: {wide:?}");
    assert!(wide.iter().all(|p| *p == LINE), "the art's own colour: {wide:?}");
    let green = [0, 255, 0, 255];
    let colour = line_colour("#00ff00");
    assert_eq!(colour.as_ref().map(|c| c.rgba), Some(green));
    let tinted = column(&drawn(&img, &Nine { line_color: colour, ..base.clone() }, panel));
    assert_eq!(tinted, vec![green; 2], "the art's own width in the colour");
    assert_eq!(line_colour("no_role"), None);
    // The line sits on the boundary whatever its width: half each side at an even width.
    let c = drawn(&img, &Nine { line_width: Some(4.0), ..base.clone() }, panel);
    let (top, mid) = (80, 140);
    assert!(alpha(&c, mid, top - 4) > 0 && alpha(&c, mid, top - 5) == 0, "above");
    assert_eq!(c.px[((top + 3) * c.w + mid) as usize * 4], LINE[0], "below");
}

#[test]
fn a_changed_colour_is_a_new_frame_to_the_cache() {
    let img = straddle_art(true, true);
    let panel = Rect::new(40, 40, 60, 60);
    let with = |hex: &str| drawn(&img, &Nine { line_color: line_colour(hex), ..straddling() }, panel);
    assert_ne!(with("#00ff00").px, with("#0000ff").px);
    // Same name, other colour (a role's colour after a theme change).
    let named = |rgba: Rgba| drawn(&img, &Nine { line_color: Some(theme::LineColour { name: "gold".into(), rgba }), ..straddling() }, panel);
    assert_ne!(named([1, 2, 3, 255]).px, named([200, 2, 3, 255]).px);
}

#[test]
fn the_new_sidecar_keys_are_bounded_and_checked() {
    let base = json!({ "slice": [32, 32, 32, 32], "straddle": true });
    let with = |extra: Value| {
        let mut v = base.clone();
        v.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        Nine::parse(&v, 80, 80, "t")
    };
    assert_eq!(with(json!({ "corner_size": 2.0 })).unwrap().corner_size, 2.0);
    let n = with(json!({ "tip": [24, 6], "tip_size": 1.5, "line_width": 2.5, "line_color": "#112233" })).unwrap();
    assert_eq!((n.tip, n.tip_size, n.line_width), (Some([24, 6]), 1.5, Some(2.5)));
    assert_eq!(n.line_color.unwrap().rgba, [0x11, 0x22, 0x33, 255]);
    assert_eq!(with(json!({ "line_color": "gold" })).unwrap().line_color.unwrap().name, "gold");
    for (bad, says) in [
        (json!({ "corner_size": 0.1 }), "corner_size"),
        (json!({ "corner_size": 3.5 }), "corner_size"),
        (json!({ "corner_size": "big" }), "corner_size"),
        (json!({ "tip_size": 1.5 }), "tip_size needs tip"),
        (json!({ "tip": [24, 6], "tip_size": 9 }), "tip_size"),
        (json!({ "tip": [0, 6] }), "tip must"),
        (json!({ "tip": [32, 6] }), "tip must"),
        (json!({ "tip": [24, 0] }), "tip must"),
        (json!({ "tip": [24, 33] }), "tip must"),
        (json!({ "tip": [24] }), "tip must"),
        (json!({ "line_width": 0.25 }), "line_width"),
        (json!({ "line_width": 8.5 }), "line_width"),
        (json!({ "line_color": "no_role" }), "line_color"),
        (json!({ "line_color": 3 }), "line_color"),
        (json!({ "line_width": 2, "edges": "tile" }), "stretched edges"),
    ] {
        let errs = with(bad.clone()).expect_err(&bad.to_string());
        assert!(errs.iter().any(|e| e.contains(says)), "{bad}: {errs:?}");
    }
    let inside = json!({ "slice": [32, 32, 32, 32], "tip": [24, 6], "line_width": 2 });
    let errs = Nine::parse(&inside, 80, 80, "t").unwrap_err();
    assert!(errs.iter().any(|e| e.contains("tip needs straddle")), "{errs:?}");
    assert!(errs.iter().any(|e| e.contains("need straddle")), "{errs:?}");
}

#[test]
fn turning_straddle_off_keeps_a_sidecar_that_loads() {
    let _dir = scratch("off");
    let mut t = tuner_on("ornate");
    t.act(TunerAct::Slide(Param::Tip, Param::Tip.step_of(2.0)));
    t.act(TunerAct::Slide(Param::Line, Param::Line.step_of(3.0)));
    t.hex_changed("#00ff00");
    t.act(TunerAct::Straddle);
    let off = t.frame().unwrap();
    assert!(!off.nine.straddle);
    let text = sidecar_text(&Value::Object(aegis_ui::frame_tuner::sidecar_of(&off.nine)));
    let v: Value = serde_json::from_str(&text).unwrap();
    assert!(Nine::parse(&v, 80, 80, "t").is_ok(), "{text}");
    assert_eq!(t.value(Param::Tip), None);
    assert_eq!(t.value(Param::Line), None);
    // A frame inside its rect takes no tip or line from the sliders.
    t.act(TunerAct::Slide(Param::Tip, 0));
    t.act(TunerAct::Slide(Param::Line, 0));
    assert_eq!(t.frame().unwrap().nine.tip, None);
    assert_eq!(t.frame().unwrap().nine.line_width, None);
}

#[test]
fn the_frames_page_is_reached_and_takes_its_keys() {
    let _dir = scratch("page");
    let page = PAGES.iter().position(|p| *p == "Frames").expect("a Frames page");
    let mut k = Specimens::default();
    k.act(KitAct::Page(page));
    let mut g = gfx(1280, 720, 1.0);
    let ids: Vec<String> = {
        let mut ui: Ui<KitAct> = Ui::new(&mut g, (-1, -1), true);
        k.draw(&mut ui, Rect::new(112, 22, 522, 332), |a| a);
        ui.widgets.iter().map(|w| w.id.clone()).collect()
    };
    for want in ["kit.frame.role", "kit.frame.width", "kit.frame.height", "kit.frame.corner", "kit.frame.tip", "kit.frame.line", "kit.frame.padding", "kit.frame.colour", HEX_FIELD, "kit.frame.straddle", "kit.frame.big"] {
        assert!(ids.iter().any(|i| i == want), "{want} in {ids:?}");
    }
    assert_eq!(ids.iter().any(|i| i == "kit.frame.save"), writes_allowed());
    assert_eq!(ids.iter().any(|i| i == "kit.frame.revert"), writes_allowed());
    // Mouse: a slider's step is an action, as the shell's drag and click use it.
    let before = k.tuner.frame().unwrap().nine.corner_size;
    k.act(KitAct::Tuner(TunerAct::Slide(Param::Corner, 0)));
    assert_ne!(k.tuner.frame().unwrap().nine.corner_size, before);
    assert!(k.last.contains("Corner"), "{}", k.last);
    // Keyboard.
    let role = k.tuner.role;
    assert!(k.key(Key::PageDown));
    assert_eq!(k.tuner.role, (role + 1) % FRAME_ROLES.len());
    assert!(k.key(Key::PageUp));
    assert_eq!(k.tuner.role, role);
    assert!(!k.key(Key::Esc), "Esc leaves the gallery from the page");
    k.act(KitAct::Tuner(TunerAct::Big));
    assert!(k.key(Key::Esc), "the big preview takes it first");
    assert!(!k.tuner.big);
    k.text_changed(HEX_FIELD, "#00ff00");
    assert_eq!(k.tuner.hex, "#00ff00");
}
