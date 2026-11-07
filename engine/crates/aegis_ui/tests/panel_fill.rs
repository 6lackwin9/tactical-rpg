//! VISCLAR1-C: `Painter::panel`, `Canvas::fill` and `Painter::image_alpha` draw the middle of a shape a row at a time.
//! The pixels must be those of the plain per-pixel rule, which each test writes out through `Canvas::blend`: across
//! sizes, radii, alphas, scales, off-canvas rects, `clip` and `bounds`.

use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::painter::{Fonts, Painter, View};
use std::path::Path;

fn fonts() -> Fonts {
    Fonts::new(&std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap())
}

fn mix(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], ((c[3] as f32) * a.clamp(0.0, 1.0)).round() as u8]
}

/// The rule: every pixel's rounded-box distance gives its fill and border cover.
fn panel_by_pixel(c: &mut Canvas, pr: Rect, fill: Rgba, border: Rgba, radius: f32) {
    let (hw, hh) = (pr.w as f32 / 2.0, pr.h as f32 / 2.0);
    let rad = radius.max(0.0).min(hw).min(hh);
    let (cx, cy) = (pr.x as f32 + hw, pr.y as f32 + hh);
    for y in pr.y..pr.y + pr.h {
        for x in pr.x..pr.x + pr.w {
            let qx = (x as f32 + 0.5 - cx).abs() - (hw - rad);
            let qy = (y as f32 + 0.5 - cy).abs() - (hh - rad);
            let d = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - rad;
            let cover = (0.5 - d).clamp(0.0, 1.0);
            if cover <= 0.0 {
                continue;
            }
            let inner = (-0.5 - d).clamp(0.0, 1.0);
            if inner > 0.0 {
                c.blend(x, y, mix(fill, inner));
            }
            let ring = cover - inner;
            if ring > 0.0 {
                c.blend(x, y, mix(border, ring.max(0.0)));
            }
        }
    }
}

/// A backdrop that changes pixel to pixel, so a wrong blend shows.
fn canvas(w: i32, h: i32) -> Canvas {
    let mut c = Canvas::new(w, h);
    for (i, p) in c.px.chunks_exact_mut(4).enumerate() {
        p.copy_from_slice(&[(i % 251) as u8, (i % 113) as u8, (i % 61) as u8, 255]);
    }
    c
}

const LIMITS: [(Option<Rect>, Option<Rect>); 4] = [
    (None, None),
    (Some(Rect { x: 11, y: 7, w: 40, h: 23 }), None),
    (None, Some(Rect { x: 3, y: 2, w: 30, h: 50 })),
    (Some(Rect { x: 11, y: 7, w: 40, h: 23 }), Some(Rect { x: 3, y: 2, w: 30, h: 50 })),
];

fn limited(mut c: Canvas, limit: (Option<Rect>, Option<Rect>)) -> Canvas {
    c.clip = limit.0;
    c.bounds = limit.1;
    c
}

#[test]
fn a_panel_draws_the_pixels_of_the_per_pixel_rule() {
    let mut f = fonts();
    let rects = [Rect::new(5, 4, 60, 40), Rect::new(0, 0, 3, 3), Rect::new(10, 10, 1, 30), Rect::new(10, 10, 9, 8), Rect::new(-6, -5, 30, 20), Rect::new(50, 40, 40, 40), Rect::new(8, 8, 16, 16), Rect::new(8, 8, 0, 5)];
    let colours: [(Rgba, Rgba); 4] = [([200, 40, 60, 255], [255, 255, 255, 50]), ([20, 90, 200, 117], [0, 0, 0, 0]), ([1, 2, 3, 0], [240, 200, 80, 255]), ([70, 80, 90, 255], [70, 80, 90, 255])];
    let mut drawn = 0;
    for r in rects {
        for radius in [0.0, 0.4, 1.0, 2.0, 3.5, 6.0, 40.0] {
            for (fill, border) in colours {
                for limit in LIMITS {
                    let mut plain = limited(canvas(70, 60), limit);
                    panel_by_pixel(&mut plain, r, fill, border, radius);
                    let mut fast = limited(canvas(70, 60), limit);
                    // a view at scale 1 with no offset: logical px are the canvas's
                    let v = View { s: 1.0, ox: 0.0, oy: 0.0, ..View::fit(640, 360) };
                    Painter { c: &mut fast, v, fonts: &mut f }.panel(r, fill, border, radius);
                    assert!(plain.px == fast.px, "{r:?} radius {radius} {fill:?} {border:?} {limit:?}");
                    drawn += 1;
                }
            }
        }
    }
    assert!(drawn > 800);
}

#[test]
fn a_fill_draws_the_pixels_of_blend() {
    for r in [Rect::new(5, 4, 60, 40), Rect::new(-5, -4, 20, 20), Rect::new(60, 50, 40, 40), Rect::new(200, 200, 5, 5), Rect::new(8, 8, 0, 9), Rect::new(8, 8, -3, 9)] {
        for col in [[9, 8, 7, 255], [200, 100, 50, 90], [1, 1, 1, 0]] {
            for limit in LIMITS {
                let mut plain = limited(canvas(70, 60), limit);
                for y in r.y..r.y + r.h {
                    for x in r.x..r.x + r.w {
                        plain.blend(x, y, col);
                    }
                }
                let mut fast = limited(canvas(70, 60), limit);
                fast.fill(r, col);
                assert!(plain.px == fast.px, "{r:?} {col:?} {limit:?}");
            }
        }
    }
}

#[test]
fn an_image_draws_the_pixels_of_blend() {
    let mut f = fonts();
    let px: Vec<u8> = (0..7 * 5 * 4).map(|i| (i * 37 % 256) as u8).collect();
    let img = Image { w: 7, h: 5, px };
    for r in [Rect::new(5, 4, 21, 15), Rect::new(-4, -3, 14, 10), Rect::new(60, 50, 30, 30), Rect::new(9, 9, 3, 3)] {
        for a in [255u8, 120, 0] {
            for limit in LIMITS {
                let mut plain = limited(canvas(70, 60), limit);
                if a > 0 {
                    for y in 0..r.h {
                        for x in 0..r.w {
                            let i = (((y * img.h / r.h) * img.w + x * img.w / r.w) * 4) as usize;
                            let alpha = ((img.px[i + 3] as u32 * a as u32 + 127) / 255) as u8;
                            plain.blend(r.x + x, r.y + y, [img.px[i], img.px[i + 1], img.px[i + 2], alpha]);
                        }
                    }
                }
                let mut fast = limited(canvas(70, 60), limit);
                let v = View { s: 1.0, ox: 0.0, oy: 0.0, ..View::fit(640, 360) };
                Painter { c: &mut fast, v, fonts: &mut f }.image_alpha(&img, r, a);
                assert!(plain.px == fast.px, "{r:?} alpha {a} {limit:?}");
            }
        }
    }
}
