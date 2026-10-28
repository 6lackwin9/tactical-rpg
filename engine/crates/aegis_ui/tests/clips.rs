//! SPRITE1-L: body clips played by time (`aegis_core::assets::{Clip, Clips, ClipPlayer}`), sliced from a scratch sheet
//! (`aegis_ui::art::Art::played`), the silhouette outline (`Image::outline`, `Art::body_outline`), the frame-by-frame
//! body check (`Art::body_problems`), mirrored frames (`flip`: a sheet facing right drawn facing left) and the foot
//! ellipse (`Painter::ellipse`). Expectations follow from the clips.json
//! rules (ASSETORG2 4.1) and the fixture's own durations.

use aegis_core::assets::{Clip, ClipPlayer, Clips, Layers, View as AssetView};
use aegis_ui::art::Art;
use aegis_ui::canvas::{Canvas, Image, Rgba};
use aegis_ui::painter::{Fonts, Painter, View};
use serde_json::json;
use std::path::{Path, PathBuf};

const F: u32 = 8;

/// (id, ms per frame, loop) of the fixture body, in row order.
fn fixture_clips() -> Vec<(&'static str, Vec<u32>, bool)> {
    vec![("idle", vec![100, 100], true), ("attack", vec![50, 60, 70], false), ("walk", vec![80, 80], true), ("death", vec![100, 200], false)]
}

/// The colour of frame `n` of row `row` in the fixture sheet.
fn colour(row: usize, n: u32) -> [u8; 4] {
    [10 + row as u8 * 40, 10 + n as u8 * 40, 200, 255]
}

fn clips_json() -> serde_json::Value {
    let clips: Vec<_> = fixture_clips().into_iter().map(|(id, ms, l)| json!({"id": id, "frames": ms.len(), "ms": ms, "loop": l})).collect();
    json!({"frame": [F, F], "palette": {"main": "#0a0ac8"}, "clips": clips})
}

fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(px).unwrap();
}

/// A scratch asset root with body `hero`: every frame a 4x4 figure (x and y 2..6) in its own colour; `blank` leaves
/// that (row, frame) empty.
fn body_root(name: &str, blank: Option<(usize, u32)>) -> PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_ui_clips_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let rows = fixture_clips();
    let cols = rows.iter().map(|r| r.1.len() as u32).max().unwrap();
    let (w, h) = (cols * F, rows.len() as u32 * F);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for (row, (_, ms, _)) in rows.iter().enumerate() {
        for n in 0..ms.len() as u32 {
            if blank == Some((row, n)) {
                continue;
            }
            for y in 2..6 {
                for x in 2..6 {
                    let (sx, sy) = (n * F + x, row as u32 * F + y);
                    let i = ((sy * w + sx) * 4) as usize;
                    px[i..i + 4].copy_from_slice(&colour(row, n));
                }
            }
        }
    }
    let dir = root.join("units/hero/base");
    write_png(&dir.join("sheet.png"), w, h, &px);
    std::fs::write(dir.join("clips.json"), clips_json().to_string()).unwrap();
    root
}

fn at(img: &Image, x: i32, y: i32) -> [u8; 4] {
    let i = ((y * img.w + x) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]
}

#[test]
fn a_clip_walks_its_durations_looping_or_holding_the_last_frame() {
    let looping = Clip { id: "idle".into(), frames: 3, ms: vec![100, 50, 150], looped: true, of: String::new(), view: AssetView::Side, impact: None, loop_from: 0, strike: 0 };
    assert_eq!(looping.total_ms(), 300);
    let frames: Vec<(u32, bool)> = [0, 99, 100, 149, 150, 299, 300, 450].iter().map(|t| looping.frame_at(*t)).collect();
    assert_eq!(frames, [(0, false), (0, false), (1, false), (1, false), (2, false), (2, false), (0, false), (2, false)], "a loop wraps at 300 ms");
    let once = Clip { id: "attack".into(), frames: 3, ms: vec![100, 50, 150], looped: false, of: String::new(), view: AssetView::Side, impact: None, loop_from: 0, strike: 0 };
    assert_eq!([once.frame_at(0), once.frame_at(149), once.frame_at(299), once.frame_at(300), once.frame_at(10_000)], [(0, false), (1, false), (2, false), (2, true), (2, true)], "a one-shot holds its last frame");
}

#[test]
fn a_missing_clip_falls_back_to_idle_then_to_the_first_frame() {
    let c = Clips::parse(&clips_json()).unwrap();
    let f = c.frame_at("attack", 115).unwrap();
    assert_eq!((f.row, f.clip.as_str(), f.frame, f.done), (1, "attack", 2, false), "50 + 60 ms in: the third frame");
    let f = c.frame_at("shield_bash", 150).unwrap();
    assert_eq!((f.row, f.clip.as_str(), f.frame), (0, "idle", 1), "a clip the body lacks plays idle");
    // A body without idle (built by hand: parse refuses it) shows its first clip's first frame, still.
    let no_idle = Clips { frame: (F, F), palette: Default::default(), clips: vec![Clip { id: "walk".into(), frames: 2, ms: vec![80, 80], looped: true, of: String::new(), view: AssetView::Side, impact: None, loop_from: 0, strike: 0 }] };
    let f = no_idle.frame_at("hurt", 90).unwrap();
    assert_eq!((f.row, f.clip.as_str(), f.frame), (0, "walk", 0));
    assert!(Clips::default().frame_at("idle", 0).is_none(), "no clips: nothing to draw (the caller's fallback)");
}

#[test]
fn a_player_shows_its_shot_then_its_base_loop() {
    let c = Clips::parse(&clips_json()).unwrap();
    let attack: u64 = 50 + 60 + 70;
    let mut p = ClipPlayer::default();
    assert_eq!(p.frame(&c, 1_000).unwrap().clip, "idle");
    p.play("attack", 1_000);
    assert_eq!(p.playing(&c, 1_000), "attack");
    assert_eq!(p.frame(&c, 1_000 + attack - 1).unwrap().frame, 2);
    assert_eq!(p.frame(&c, 1_000 + attack).unwrap().clip, "idle", "over: the base loop again");
    assert_eq!(p.playing(&c, 1_000 + attack), "");
    p.play_for("walk", 2_000, 400);
    assert_eq!(p.frame(&c, 2_000 + 170).unwrap().frame, 0, "a timed shot loops: 170 ms is past one 160 ms walk");
    assert_eq!(p.frame(&c, 2_400).unwrap().clip, "idle");
    p.set_base("walk", 3_000);
    assert_eq!(p.frame(&c, 3_090).unwrap().frame, 1, "the base loop counts from when it was set");
    p.set_base("walk", 3_050);
    assert_eq!(p.frame(&c, 3_090).unwrap().frame, 1, "setting the same base again does not restart it");
    p.play_and_hold("death", 4_000);
    assert_eq!(p.held_for(&c, 4_299), None);
    let f = p.frame(&c, 9_000).unwrap();
    assert_eq!((f.clip.as_str(), f.frame, f.done), ("death", 1, true), "a held shot keeps its last frame");
    assert_eq!(p.held_for(&c, 4_300 + 250), Some(250));
    p.stop();
    assert_eq!(p.frame(&c, 9_000).unwrap().clip, "walk");
}

#[test]
fn art_slices_the_frame_a_player_shows_from_the_sheet() {
    let root = body_root("slice", None);
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    assert!(art.has_clip("hero", "attack") && !art.has_clip("hero", "hurt"));
    assert_eq!(art.first_body(&["hero/goblin".to_string(), "hero".to_string()]).as_deref(), Some("hero"));
    let mut p = ClipPlayer::default();
    p.play("attack", 0);
    for (t, n) in [(0u64, 0u32), (50, 1), (110, 2)] {
        let (f, img) = art.played("hero", "", &p, t, F as i32, false).unwrap();
        assert_eq!((f.row, f.frame), (1, n));
        assert_eq!(at(img, 3, 3), colour(1, n), "attack frame {n} at {t} ms");
        assert_eq!(at(img, 0, 0)[3], 0, "the frame's own transparency");
    }
    p.play("hurt", 0);
    let (f, img) = art.played("hero", "", &p, 120, F as i32, false).unwrap();
    assert_eq!((f.clip.as_str(), at(img, 3, 3)), ("idle", colour(0, 1)), "no hurt clip: idle at its own time");
    assert!(art.played("nobody", "", &p, 0, F as i32, false).is_none(), "no body: the caller's fallback");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_flipped_frame_and_its_outline_are_the_sheet_frame_mirrored() {
    // A one-clip body whose frame is lopsided: a 3-pixel bar on the left (x 1..3) and one pixel on the right (x 6).
    let root = std::env::temp_dir().join(format!("aegis_ui_clips_flip_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut px = vec![0u8; (F * F * 4) as usize];
    for x in [1u32, 2, 3, 6] {
        let i = ((3 * F + x) * 4) as usize;
        px[i..i + 4].copy_from_slice(&[200, 40, 40, 255]);
    }
    write_png(&root.join("units/arrow/base/sheet.png"), F, F, &px);
    let clips = json!({"frame": [F, F], "clips": [{"id": "idle", "frames": 1, "ms": [100], "loop": true}]});
    std::fs::write(root.join("units/arrow/base/clips.json"), clips.to_string()).unwrap();
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    let p = ClipPlayer::default();
    let size = F as i32 * 3;
    let plain = art.played("arrow", "", &p, 0, size, false).unwrap().1.clone();
    let (f, flipped) = art.played("arrow", "", &p, 0, size, true).map(|(f, i)| (f, i.clone())).unwrap();
    let flipped = &flipped;
    assert_eq!(f.clip, "idle");
    assert_eq!(*flipped, plain.mirrored(), "the drawn frame is the sheet frame mirrored");
    assert_ne!(*flipped, plain, "the fixture is lopsided, so mirroring shows");
    let sheet = art.body_frame("arrow", "", "idle", 0).unwrap();
    assert_eq!(at(&plain, 3 * 3 + 1, 3 * 3 + 1), at(&sheet, 3, 3), "unflipped: the sheet as it is (scaled by 3)");
    assert_eq!(at(flipped, size - 1 - (3 * 3 + 1), 3 * 3 + 1), at(&sheet, 3, 3), "flipped: the same pixel from the other side");
    let gold: Rgba = [244, 198, 84, 255];
    let o_plain = art.body_outline("arrow", "", "idle", 0, size, gold, false).unwrap().clone();
    let o_flip = art.body_outline("arrow", "", "idle", 0, size, gold, true).unwrap().clone();
    assert_eq!(o_flip, plain.mirrored().outline(gold), "the outline rings the mirrored frame");
    assert_eq!(o_flip, o_plain.mirrored(), "and is the plain outline mirrored");
    std::fs::remove_dir_all(&root).unwrap();
    assert!(art.played("arrow", "", &p, 0, size, true).is_some(), "cached apart: the files are gone");
}

#[test]
fn an_outline_rings_the_solid_pixels_and_is_cached() {
    // A 3x3 solid block in a 5x5 image: its ring touching sides or corners is 5x5 - 3x3 = 16 pixels.
    let mut img = Image { w: 5, h: 5, px: vec![0; 100] };
    for y in 1..4 {
        for x in 1..4 {
            let i = ((y * 5 + x) * 4) as usize;
            img.px[i..i + 4].copy_from_slice(&[9, 9, 9, 255]);
        }
    }
    // A faint pixel (alpha below the silhouette's) is no part of it.
    img.px[3] = 40;
    let gold: Rgba = [244, 198, 84, 255];
    let o = img.outline(gold);
    assert_eq!((o.w, o.h), (7, 7), "one pixel more on every side");
    let lit: Vec<(i32, i32)> = (0..7).flat_map(|y| (0..7).map(move |x| (x, y))).filter(|(x, y)| at(&o, *x, *y) == gold).collect();
    assert_eq!(lit.len(), 16);
    assert!(lit.iter().all(|(x, y)| (1..=5).contains(x) && (1..=5).contains(y) && !((2..=4).contains(x) && (2..=4).contains(y))));
    let root = body_root("outline", None);
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    let o = art.body_outline("hero", "", "idle", 0, F as i32, gold, false).unwrap().clone();
    // The 4x4 figure at 2..6 of the 8x8 frame: its ring at 1..=6 of the frame, 2..=7 of the outline.
    let lit = (0..10).flat_map(|y| (0..10).map(move |x| (x, y))).filter(|(x, y)| at(&o, *x, *y) == gold).count();
    assert_eq!((o.w, lit), (10, 6 * 6 - 4 * 4));
    std::fs::remove_dir_all(&root).unwrap();
    assert!(art.body_outline("hero", "", "idle", 0, F as i32, gold, false).is_some(), "cached: the files are gone");
}

#[test]
fn a_body_is_checked_against_its_clips_frame_by_frame() {
    let good = body_root("good", None);
    assert!(Art::layered(Layers::of(vec![good.clone()])).body_problems("hero").is_empty());
    let gap = body_root("gap", Some((1, 2)));
    assert_eq!(Art::layered(Layers::of(vec![gap.clone()])).body_problems("hero"), ["clip 'attack' frame 2 is empty."]);
    // One clip more than the sheet has rows.
    let mut v = clips_json();
    v["clips"].as_array_mut().unwrap().push(json!({"id": "hurt", "frames": 1, "ms": [100]}));
    std::fs::write(gap.join("units/hero/base/clips.json"), v.to_string()).unwrap();
    let p = Art::layered(Layers::of(vec![gap.clone()])).body_problems("hero");
    assert_eq!(p, [format!("the sheet is {}x{}; its clips need at least {}x{}.", 3 * F, 4 * F, 3 * F, 5 * F)]);
    assert!(Art::layered(Layers::of(vec![gap.clone()])).body_problems("nobody")[0].contains("units/nobody/base/clips.json"));
    for r in [good, gap] {
        let _ = std::fs::remove_dir_all(&r);
    }
}

#[test]
fn a_foot_ellipse_fills_its_inside_only() {
    let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut canvas = Canvas::new(640, 360);
    let mut fonts = Fonts::new(&font);
    let mut p = Painter { c: &mut canvas, v: View::fit(640, 360), fonts: &mut fonts };
    p.ellipse(100.0, 50.0, 12.0, 4.0, [255, 0, 0, 255]);
    let px = |x: i32, y: i32| {
        let i = ((y * 640 + x) * 4) as usize;
        [canvas.px[i], canvas.px[i + 1], canvas.px[i + 2]]
    };
    assert_eq!(px(100, 50), [255, 0, 0], "the centre");
    assert_eq!(px(110, 50), [255, 0, 0], "inside along the long radius");
    assert_ne!(px(100, 56), [255, 0, 0], "outside the short radius");
    assert_ne!(px(114, 50), [255, 0, 0], "outside the long radius");
}
