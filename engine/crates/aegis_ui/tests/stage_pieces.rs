//! STAGE1-E stage pieces (STAGE1 4.3, check 12): the foot ring (`Painter::ellipse_ring`) touches only pixels near the
//! ellipse's edge, `Image::opaque_bounds` boxes the pixels at or above an alpha, and `Art::body_bounds` is that box of
//! a body's fitted `idle` frame 0 (k = floor(size / frame side), centred across, on the bottom edge), mirrored with
//! `flip`.

use aegis_core::assets::Layers;
use aegis_ui::art::Art;
use aegis_ui::canvas::{Canvas, Image, Rect, OUTLINE_ALPHA};
use aegis_ui::painter::{Fonts, Painter, View};
use std::path::{Path, PathBuf};

fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(px).unwrap();
}

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

/// Draws one ring on a `pw` x `ph` canvas (a 640 x 360 logical view scaled by pw / 640) and returns the canvas and
/// the view's scale.
fn ring(pw: i32, ph: i32, cx: f32, cy: f32, rx: f32, ry: f32) -> (Canvas, f32) {
    let font = font();
    let mut canvas = Canvas::new(pw, ph);
    let mut fonts = Fonts::new(&font);
    let v = View::fit(pw as u32, ph as u32);
    let mut p = Painter { c: &mut canvas, v, fonts: &mut fonts };
    p.ellipse_ring(cx, cy, rx, ry, [255, 0, 0, 255]);
    (canvas, v.s)
}

#[test]
fn the_ring_sets_only_pixels_near_the_edge() {
    for (pw, ph) in [(640, 360), (1280, 720)] {
        let (cx, cy, rx, ry) = (100.0f32, 50.0f32, 12.0f32, 4.0f32);
        let (canvas, s) = ring(pw, ph, cx, cy, rx, ry);
        let (x, y, prx, pry) = (cx * s, cy * s, rx * s, ry * s);
        // The stroke is one logical pixel (s physical): nothing inside the ellipse shrunk by the half stroke plus a
        // pixel, nothing outside the one grown by as much.
        let m = s / 2.0 + 1.0;
        let inside = |ux: f32, uy: f32, a: f32, b: f32| (ux / a).powi(2) + (uy / b).powi(2) < 1.0;
        let mut set = 0;
        for py in 0..ph {
            for px in 0..pw {
                if canvas.px[((py * pw + px) * 4 + 3) as usize] == 0 {
                    continue;
                }
                set += 1;
                let (ux, uy) = (px as f32 + 0.5 - x, py as f32 + 0.5 - y);
                assert!(!inside(ux, uy, prx - m, pry - m), "({px}, {py}) inside the ring at scale {s}");
                assert!(inside(ux, uy, prx + m, pry + m), "({px}, {py}) outside the ring at scale {s}");
            }
        }
        assert!(set > 0);
        let lit = |px: i32, py: i32| canvas.px[((py * pw + px) * 4 + 3) as usize] > 0;
        assert!(!lit(x as i32, y as i32), "no fill at the centre");
        // The ring is closed: every column across the ellipse has a lit pixel above and below the centre line.
        for px in (x - prx + 1.0) as i32..(x + prx - 1.0) as i32 {
            assert!((0..y as i32).any(|py| lit(px, py)) && (y as i32..ph).any(|py| lit(px, py)), "column {px} at scale {s}");
        }
    }
}

#[test]
fn opaque_bounds_box_the_pixels_at_or_above_the_alpha() {
    let blank = Image { w: 6, h: 5, px: vec![0; 6 * 5 * 4] };
    assert_eq!(blank.opaque_bounds(1), None, "an empty image has no bounds");
    let mut one = blank.clone();
    let at = |x: i32, y: i32| ((y * 6 + x) * 4 + 3) as usize;
    one.px[at(4, 2)] = 255;
    assert_eq!(one.opaque_bounds(OUTLINE_ALPHA), Some(Rect::new(4, 2, 1, 1)), "one pixel: that pixel");
    let mut two = one.clone();
    two.px[at(1, 4)] = OUTLINE_ALPHA;
    two.px[at(0, 0)] = OUTLINE_ALPHA - 1;
    assert_eq!(two.opaque_bounds(OUTLINE_ALPHA), Some(Rect::new(1, 2, 4, 3)), "the alpha itself counts, one below does not");
    assert_eq!(two.opaque_bounds(1), Some(Rect::new(0, 0, 5, 5)));
}

/// A body whose 8 x 8 `idle` frame 0 is solid over x 1..3, y 2..8 (left of centre) with a faint pixel at (7, 0)
/// below `OUTLINE_ALPHA`; `idle` frame 1 is empty.
fn body_tree() -> PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_stage_pieces_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (w, h) = (16u32, 8u32);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 2..8 {
        for x in 1..3 {
            let i = ((y * w + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[0xa8, 0x30, 0x28, 255]);
        }
    }
    let faint = 7 * 4;
    px[faint..faint + 4].copy_from_slice(&[0xa8, 0x30, 0x28, OUTLINE_ALPHA - 1]);
    let dir = root.join("units/scout/base");
    write_png(&dir.join("sheet.png"), w, h, &px);
    std::fs::write(dir.join("clips.json"), r##"{"frame": [8, 8], "clips": [{"id": "idle", "frames": 2, "ms": [100, 100]}]}"##).unwrap();
    root
}

#[test]
fn body_bounds_are_the_fitted_idle_frames_solid_box_and_mirror_with_flip() {
    let root = body_tree();
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    for size in [8, 16, 20] {
        // k = floor(size / 8); the k x frame is centred across ((size - 8k) / 2) and stands on the bottom (size - 8k).
        let k = size / 8;
        let (ox, oy) = ((size - 8 * k) / 2, size - 8 * k);
        let want = Rect::new(ox + k, oy + 2 * k, 2 * k, 6 * k);
        let r = art.body_bounds("scout", "", size, false);
        assert_eq!(r, Some(want), "size {size}");
        let f = art.body_bounds("scout", "", size, true).expect("flipped");
        assert_eq!(f, Rect::new(size - want.x - want.w, want.y, want.w, want.h), "the mirror of the unflipped bounds at size {size}");
        assert_eq!(art.body_bounds("scout", "", size, false), r, "cached the same");
    }
    assert_eq!(art.body_bounds("nobody", "", 16, false), None, "no body, no bounds");
    let _ = std::fs::remove_dir_all(&root);
}
