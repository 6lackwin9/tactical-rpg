//! SPRITEFIT1: a body's frame fills the size asked for at any scale (`Image::fit_square`, `Image::reduce_box`,
//! `Art::body_clip_flipped`). At a whole multiple it is the plain nearest scale; between multiples it is scaled up to the
//! next one and reduced by a true box to the exact size; below 1x it is today's `downsample`. Expectations follow from
//! that rule and the fixture's own shape.

use aegis_core::scratch::Scratch;
use aegis_core::assets::Layers;
use aegis_ui::art::Art;
use aegis_ui::canvas::{Image, Rect, Rgba, OUTLINE_ALPHA};
use std::path::Path;

const F: i32 = 48;
const A: [u8; 4] = [200, 40, 40, 255];
const B: [u8; 4] = [40, 80, 220, 255];
/// The fixture's solid box: lopsided in x, off the top, standing on the bottom edge.
const SOLID: (i32, i32, i32, i32) = (10, 8, 26, 40);

fn at(img: &Image, x: i32, y: i32) -> [u8; 4] {
    let i = ((y * img.w + x) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]
}

/// A `w` x `h` image, clear but for the box (`x`, `y`, `bw`, `bh`): colour A left of column `split`, B from it on.
fn figure(w: i32, h: i32, (x, y, bw, bh): (i32, i32, i32, i32), split: i32) -> Image {
    let mut px = vec![0u8; (w * h * 4) as usize];
    for yy in y..y + bh {
        for xx in x..x + bw {
            let i = ((yy * w + xx) * 4) as usize;
            px[i..i + 4].copy_from_slice(if xx < split { &A } else { &B });
        }
    }
    Image { w, h, px }
}

fn frame() -> Image {
    figure(F, F, SOLID, 25)
}

fn write_png(path: &Path, img: &Image) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img.px).unwrap();
}

/// A scratch asset root holding one body `hero` whose idle frame is `img`.
fn body_root(name: &str, img: &Image) -> Scratch {
    let root = Scratch::vacant(&format!("sprite_fit_{name}"));
    write_png(&root.join("units/hero/base/sheet.png"), img);
    let clips = format!(r#"{{"frame": [{}, {}], "clips": [{{"id": "idle", "frames": 1, "ms": [100]}}]}}"#, img.w, img.h);
    std::fs::write(root.join("units/hero/base/clips.json"), clips).unwrap();
    root
}

fn near(got: i32, want: f32, what: &str) {
    assert!((got as f32 - want).abs() <= 1.0, "{what}: {got}, wanted {want} within a pixel");
}

#[test]
fn between_multiples_the_opaque_bounds_scale_with_the_size() {
    let f = frame();
    let base = f.fit_square(F).opaque_bounds(OUTLINE_ALPHA).unwrap();
    assert_eq!((base.x, base.y, base.w, base.h), SOLID, "at 1x the fixture's own box");
    for size in [50, 60, 72, 78, 90, 100, 120, 150, 200] {
        let img = f.fit_square(size);
        assert_eq!((img.w, img.h), (size, size), "the square is the size asked for");
        let b = img.opaque_bounds(OUTLINE_ALPHA).unwrap();
        let ratio = size as f32 / F as f32;
        near(b.w, base.w as f32 * ratio, &format!("width at {size}"));
        near(b.h, base.h as f32 * ratio, &format!("height at {size}"));
        near(b.x, base.x as f32 * ratio, &format!("left at {size}"));
        near(b.y, base.y as f32 * ratio, &format!("top at {size}"));
        assert_eq!(b.y + b.h, size, "standing on the bottom edge at {size}");
    }
}

#[test]
fn an_exact_multiple_is_the_plain_nearest_scale() {
    let f = frame();
    for k in [1, 2, 3, 4] {
        assert_eq!(f.fit_square(F * k), f.scale_nearest(k), "{k}x");
    }
    // Every byte, down to the colour a clear pixel stores (the box would blank it), and every partial alpha.
    let mut stored = f.clone();
    for (n, p) in stored.px.chunks_exact_mut(4).enumerate() {
        if p[3] == 0 {
            p.copy_from_slice(&[9, 9, 9, 0]);
        } else if n % 3 == 0 {
            p[3] = 90;
        }
    }
    for k in [1, 2, 3] {
        assert_eq!(stored.fit_square(F * k), stored.scale_nearest(k), "{k}x with stored colours");
    }
}

#[test]
fn below_1x_it_is_the_old_downsample() {
    let f = frame();
    for size in [1, 12, 28, 40, 47] {
        assert_eq!(f.fit_square(size), f.downsample(size, size), "{size}");
    }
    let tall = figure(32, 48, (0, 0, 32, 48), 16);
    assert_eq!(tall.fit_square(40), tall.downsample(32 * 40 / 48, 40).on_square(40), "the same fit by the longer side");
}

/// The box mixes the two colours where the rule says: 25 x 1.5 = 37.5 is the split, so column 37 is half of each, its
/// neighbours pure. A picked pixel (`downsample` between 1x and 2x) would fail this.
#[test]
fn between_multiples_a_seam_softens_and_the_rest_stays_pure() {
    let img = frame().fit_square(72);
    let y = 60;
    assert_eq!(at(&img, 36, y), A);
    let mid = at(&img, 37, y);
    for c in 0..3 {
        assert_eq!(mid[c] as i32, (A[c] as i32 + B[c] as i32 + 1) / 2, "channel {c} of the seam");
    }
    assert_eq!(mid[3], 255);
    assert_eq!(at(&img, 38, y), B);
    assert_eq!(at(&img, 40, y), B);
}

#[test]
fn the_box_averages_area_and_alpha_without_a_dark_fringe() {
    let solid = Image { w: 7, h: 5, px: A.iter().copied().cycle().take(7 * 5 * 4).collect() };
    let small = solid.reduce_box(5, 3);
    assert!(small.px.chunks_exact(4).all(|p| p == A), "a solid colour stays itself at any ratio");
    // half solid, half clear: half the alpha, the solid colour unchanged (the clear pixel's black does not leak in)
    let half = Image { w: 2, h: 1, px: [A, [0, 0, 0, 0]].concat() }.reduce_box(1, 1);
    assert_eq!((half.px[0], half.px[1], half.px[2], half.px[3]), (A[0], A[1], A[2], 128));
    // 4 to 3: [A A B B] covers 4/3 source pixels each: pure, half and half, pure
    let row = Image { w: 4, h: 1, px: [A, A, B, B].concat() }.reduce_box(3, 1);
    assert_eq!((at(&row, 0, 0), at(&row, 2, 0)), (A, B));
    assert_eq!(at(&row, 1, 0), [120, 60, 130, 255]);
    let same = frame().reduce_box(F, F);
    assert_eq!(same, frame(), "1:1 keeps every pixel");
    assert_eq!(Image { w: 0, h: 0, px: vec![] }.reduce_box(3, 3).px, vec![0u8; 36], "nothing to average: clear");
    assert_eq!(frame().reduce_box(0, 4).px.len(), 0);
}

#[test]
fn a_frame_keeps_its_proportions_and_place_in_the_square() {
    let tall = figure(32, 48, (0, 0, 32, 48), 16);
    let wide = figure(48, 32, (0, 0, 48, 32), 24);
    for size in [48, 60, 72, 90, 96, 144] {
        for (name, img) in [("tall", &tall), ("wide", &wide)] {
            let want = |side: i32| (side as f32 * size as f32 / 48.0).round() as i32;
            let (w, h) = (want(img.w), want(img.h));
            let b = img.fit_square(size).opaque_bounds(OUTLINE_ALPHA).unwrap();
            assert_eq!(b, Rect::new((size - w) / 2, size - h, w, h), "{name} at {size}: centred across, on the bottom edge");
        }
    }
}

#[test]
fn a_flipped_frame_mirrors_the_unflipped_one_and_its_outline_and_silhouette_line_up() {
    let root = body_root("flip", &frame());
    let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
    let col: Rgba = [244, 198, 84, 255];
    for size in [F, 72, 90, 96, 144] {
        let plain = art.body_clip_flipped("hero", "", "idle", 0, size, false).unwrap().clone();
        let flipped = art.body_clip_flipped("hero", "", "idle", 0, size, true).unwrap().clone();
        assert_eq!(flipped, plain.mirrored(), "flip mirrors at {size}");
        assert_ne!(flipped, plain, "the fixture is lopsided");
        assert_eq!(frame().mirrored().fit_square(size), flipped, "fitting the mirrored frame gives the same pixels at {size}");

        let solid = plain.opaque_bounds(OUTLINE_ALPHA).unwrap();
        assert_eq!(art.body_bounds("hero", "", size, false), Some(solid), "the hit box is the drawn solid box at {size}");
        let sil = art.body_silhouette("hero", "", "idle", 0, size, col, false).unwrap().clone();
        assert_eq!(sil.opaque_bounds(1), Some(solid), "the silhouette covers the solid box at {size}");

        let ring = art.body_outline("hero", "", "idle", 0, size, col, false).unwrap().clone();
        assert_eq!((ring.w, ring.h), (size + 2, size + 2));
        // The ring's image starts one pixel up and left of the frame, so the solid box sits at +1 in it and the ring
        // reaches one pixel past it on every side.
        assert_eq!(ring.opaque_bounds(1), Some(Rect::new(solid.x, solid.y, solid.w + 2, solid.h + 2)), "the ring hugs the solid box at {size}");
        for y in 0..size {
            for x in 0..size {
                if at(&plain, x, y)[3] >= OUTLINE_ALPHA {
                    assert_eq!(at(&ring, x + 1, y + 1)[3], 0, "no ring over a solid pixel at ({x}, {y}), size {size}");
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn each_size_is_cached_apart_with_its_own_pixels() {
    let root = body_root("cache", &frame());
    let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
    let a72 = art.body_clip("hero", "", "idle", 0, 72).unwrap().clone();
    let a90 = art.body_clip("hero", "", "idle", 0, 90).unwrap().clone();
    assert_eq!((a72.w, a90.w), (72, 90));
    std::fs::remove_dir_all(&root).unwrap();
    assert_eq!(art.body_clip("hero", "", "idle", 0, 72), Some(&a72), "cached: the files are gone");
    assert_eq!(art.body_clip("hero", "", "idle", 0, 90), Some(&a90));
}
