//! REGIONGROUND1-E1: the ground composer (`aegis_ui::ground`, `docs/specs/REGIONGROUND1_ENGINE.md` section 5), checked from
//! its rules on fixtures of flat-colour and two-tone layers: determinism, the edge bound, lanes, no cell lattice, the
//! tone, paint order, wear, strays, bites, sunken rims, the two cache levels, `soft_ellipse`, the Gallery view and the
//! tools' job file.

use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::ground::{self, GroundJob, GroundKey, GroundLayer, CELL_MAX, CELL_PX, LAYERS_MAX, PHYSICAL_MAX, TEX_CELLS, TILTED_CELL};
use aegis_ui::painter::{Fonts, Painter, View};
use aegis_ui::specimens::{KitAct, Specimens, TILE_VIEWS, TILE_ZOOMS};
use aegis_ui::text_audit;
use aegis_ui::theme::Theme;
use aegis_ui::ui::{Gfx, Kit, Ui, Widget};
use serde_json::{json, Value};
use std::path::Path;

const SQUARE: (i32, i32) = (CELL_PX, CELL_PX);
const CELLS: [(i32, i32); 2] = [SQUARE, TILTED_CELL];
const LOW: Rgba = [40, 60, 200, 255];
const UP: Rgba = [200, 60, 40, 255];
const THIRD: Rgba = [40, 200, 60, 255];

fn compose(seed: u64, cols: i32, rows: i32, cell: (i32, i32), cells: &[u8], layers: &[GroundLayer]) -> Image {
    let blocked = vec![false; cells.len()];
    ground::compose(&GroundJob { seed, cols, rows, cell, cells, blocked: &blocked, layers, placed: &[] })
}

fn at(img: &Image, x: i32, y: i32) -> [u8; 3] {
    let i = ((y * img.w + x) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2]]
}

fn rgb3(c: Rgba) -> [u8; 3] {
    [c[0], c[1], c[2]]
}

/// A colour by `k`, rounded as `compose` rounds it.
fn by(c: Rgba, k: f32) -> [u8; 3] {
    [(c[0] as f32 * k + 0.5) as u8, (c[1] as f32 * k + 0.5) as u8, (c[2] as f32 * k + 0.5) as u8]
}

/// The fixtures' hues: red is `UP`, blue is `LOW`, green is `THIRD`, whatever the darkening.
fn is_up(p: [u8; 3]) -> bool {
    p[0] > p[2] && p[0] > p[1]
}
fn is_low(p: [u8; 3]) -> bool {
    p[2] > p[0] && p[2] > p[1]
}

/// `rows` of cols: 0 above row `boundary`, 1 from it down.
fn split_h(cols: i32, rows: i32, boundary: i32) -> Vec<u8> {
    (0..rows).flat_map(|y| (0..cols).map(move |_| (y >= boundary) as u8)).collect()
}

/// `rows` of cols: 0 left of column `boundary`, 1 from it right.
fn split_v(cols: i32, rows: i32, boundary: i32) -> Vec<u8> {
    (0..rows).flat_map(|_| (0..cols).map(move |x| (x >= boundary) as u8)).collect()
}

fn two_flat() -> [GroundLayer<'static>; 2] {
    [GroundLayer::flat(0, LOW), GroundLayer::flat(1, UP)]
}

/// Check 1: the same job twice gives the same bytes; another seed does not; a 4:3 cell gives a 4:3 image.
#[test]
fn a_job_is_a_pure_function_of_its_inputs() {
    let cells = split_h(20, 12, 6);
    let tex = ground::two_tone(64, 64, [90, 80, 70, 255], [84, 74, 64, 255], 1);
    let layers = [GroundLayer { tex: Some(&tex), tone: 0.13, ..GroundLayer::flat(0, LOW) }, GroundLayer { wear: 0.1, strays: true, tone: 0.13, ..GroundLayer::flat(1, UP) }];
    let a = compose(5, 20, 12, SQUARE, &cells, &layers);
    assert_eq!((a.w, a.h), (20 * 16, 12 * 16));
    assert_eq!(a.px.len(), (a.w * a.h * 4) as usize);
    assert!(a.px.chunks_exact(4).all(|p| p[3] == 255), "opaque");
    assert!(a == compose(5, 20, 12, SQUARE, &cells, &layers), "same job, same bytes");
    assert!(a != compose(6, 20, 12, SQUARE, &cells, &layers), "another seed differs");
    let moved = std::thread::spawn({
        let cells = cells.clone();
        move || {
            let tex = ground::two_tone(64, 64, [90, 80, 70, 255], [84, 74, 64, 255], 1);
            let layers = [GroundLayer { tex: Some(&tex), tone: 0.13, ..GroundLayer::flat(0, LOW) }, GroundLayer { wear: 0.1, strays: true, tone: 0.13, ..GroundLayer::flat(1, UP) }];
            compose(5, 20, 12, SQUARE, &cells, &layers)
        }
    })
    .join()
    .unwrap();
    assert!(a == moved, "the same on another thread: no shared state");
    let tilted = compose(5, 20, 12, TILTED_CELL, &cells, &two_flat());
    assert_eq!((tilted.w, tilted.h), (20 * 16, 12 * 12));
    assert_eq!(tilted.w * 3, tilted.h * 4 * 20 / 12, "a 4:3 cell, a 4:3 image per cell");
}

/// Compares a composed image with its pinned file, `tests/fixtures/ground/<name>.png`, and names the first differing
/// pixel. With `AEGIS_GROUND_WRITE_PINS` set it writes the file instead: only after tuning the method's numbers on purpose.
fn pin(name: &str, img: &Image) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ground").join(format!("{name}.png"));
    if std::env::var_os("AEGIS_GROUND_WRITE_PINS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        write_png(&path, img);
        return;
    }
    let want = aegis_ui::art::load_png(&path).unwrap_or_else(|| panic!("{name}: no pin at {}", path.display()));
    assert_eq!((img.w, img.h), (want.w, want.h), "{name}: the image's size");
    let differs = |i: &usize| img.px[i * 4..i * 4 + 4] != want.px[i * 4..i * 4 + 4];
    let n = (img.w * img.h) as usize;
    if let Some(i) = (0..n).find(differs) {
        let (x, y) = (i as i32 % img.w, i as i32 / img.w);
        panic!("{name}: {} of {n} pixels differ, the first at ({x}, {y}): got {:?}, pinned {:?}", (0..n).filter(differs).count(), &img.px[i * 4..i * 4 + 4], &want.px[i * 4..i * 4 + 4]);
    }
}

/// Two layers' bytes pinned across runs, builds and platforms, by form: a change here means a frame is no longer
/// reproducible, or the method's numbers were tuned on purpose.
#[test]
fn pins_two_layers() {
    let cells = split_h(16, 10, 5);
    let tex = ground::two_tone(64, 64, [90, 80, 70, 255], [84, 74, 64, 255], 1);
    let layers = [GroundLayer { tex: Some(&tex), tone: 0.13, ..GroundLayer::flat(0, LOW) }, GroundLayer { wear: 0.1, strays: true, tone: 0.13, ..GroundLayer::flat(1, UP) }];
    pin("two_layers_square", &compose(7, 16, 10, SQUARE, &cells, &layers));
    pin("two_layers_tilted", &compose(7, 16, 10, TILTED_CELL, &cells, &layers));
}

/// Three layers pinned the same way, for the paths two layers never take: a middle ground's own share (the upper ground
/// also lies far from it), the bite's choice of ground under, a sunken rim, and the texture offsets of later places in
/// the list (the list is not in paint order).
#[test]
fn pins_three_layers() {
    let rows = ["aaaaaaaaaaaacc", "aaaaaaaaaaaacc", "aaaaaaaaaaaaaa", "aaaaaaaaaaaaaa", "aaaaaaaaaaaaaa", "bbbbbbbccccccc", "bbbbbbbccccccc", "bbbbbbbccccccc", "bbbbbbbccccccc", "bbbbbbbccccccc"];
    let cells: Vec<u8> = rows.iter().flat_map(|r| r.bytes()).map(|c| match c { b'c' => 0, b'a' => 1, _ => 2 }).collect();
    for (name, cell) in [("three_layers_square", SQUARE), ("three_layers_tilted", TILTED_CELL)] {
        let (tw, th) = (TEX_CELLS * cell.0, TEX_CELLS * cell.1);
        let lowest = ground::two_tone(tw, th, [120, 100, 80, 255], [100, 84, 66, 255], 1);
        let middle = ground::two_tone(tw, th, [50, 70, 110, 255], [44, 62, 100, 255], 2);
        let layers = [
            GroundLayer { wear: 0.1, strays: true, tone: 0.13, ..GroundLayer::flat(2, THIRD) },
            GroundLayer { tex: Some(&lowest), tone: 0.13, ..GroundLayer::flat(0, LOW) },
            GroundLayer { tex: Some(&middle), wear: 0.25, strays: true, tone: 0.05, sunken: true, ..GroundLayer::flat(1, UP) },
        ];
        pin(name, &compose(7, 14, 10, cell, &cells, &layers));
    }
}

/// Empty cells compose nothing; a grid the cells do not fill too.
#[test]
fn an_empty_job_composes_nothing() {
    let img = compose(1, 4, 4, SQUARE, &[], &two_flat());
    assert_eq!((img.w, img.h, img.px.len()), (0, 0, 0));
    let img = compose(1, 4, 4, SQUARE, &[0, 1], &two_flat());
    assert_eq!((img.w, img.h), (0, 0));
    let img = compose(1, 4, 4, SQUARE, &[0; 16], &[]);
    assert_eq!((img.w, img.h), (0, 0));
}

/// Past the limits a job composes nothing, with no huge allocation and no overflow.
#[test]
fn a_job_past_the_limits_composes_nothing() {
    let flat = two_flat();
    let some = vec![0u8; 16];
    for cell in [(i32::MAX, 16), (16, i32::MAX), (CELL_MAX + 1, 16), (16, CELL_MAX + 1), (0, 16), (16, -1)] {
        let img = compose(1, 4, 4, cell, &some, &flat);
        assert_eq!((img.w, img.h, img.px.len()), (0, 0, 0), "{cell:?}");
    }
    // Enough cells for the grid, too many art px.
    let side = 4000;
    let many = vec![0u8; (side * side) as usize];
    let img = compose(1, side, side, (CELL_MAX, CELL_MAX), &many, &flat);
    assert_eq!((img.w, img.h), (0, 0), "past the art px cap");
    // A grid whose art size overflows, whatever the cells.
    let img = compose(1, i32::MAX, 2, SQUARE, &some, &flat);
    assert_eq!((img.w, img.h), (0, 0), "an overflow");
    // At the cell limit it composes.
    let img = compose(1, 4, 4, (CELL_MAX, CELL_MAX), &some, &flat);
    assert_eq!((img.w, img.h), (4 * CELL_MAX, 4 * CELL_MAX));
    // Too many layers: refused, not wrapped into a lower rank.
    let layers: Vec<GroundLayer> = (0..=LAYERS_MAX as i32).map(|i| GroundLayer::flat(i, LOW)).collect();
    let img = compose(1, 4, 4, SQUARE, &some, &layers);
    assert_eq!((img.w, img.h), (0, 0), "{} layers", layers.len());
    let img = compose(1, 4, 4, SQUARE, &some, &layers[..LAYERS_MAX]);
    assert_eq!((img.w, img.h), (64, 64), "{LAYERS_MAX} layers");
}

/// Off-map cells clamp to the nearest edge cell: a ground on every cell reaches the map's border with nothing of the
/// ground under it, and a straight boundary does not change at the border.
#[test]
fn an_edge_does_not_curve_at_the_map_border() {
    for cell in CELLS {
        let all = vec![1u8; 10 * 8];
        let img = compose(3, 10, 8, cell, &all, &two_flat());
        assert!((0..img.h).all(|y| (0..img.w).all(|x| at(&img, x, y) == rgb3(UP))), "{cell:?}: every pixel the upper ground");
    }
}

/// Check 2: on a straight boundary over 30 cells, no upper pixel is more than 6 art px past the cell edge and no lower
/// pixel more than 6 inside it, both ways and in both forms; at least half its columns are off the cell edge.
#[test]
fn an_edge_stays_within_a_third_of_a_cell_and_is_not_straight() {
    for cell in CELLS {
        for seed in 0..20 {
            // Across the columns: the boundary at row 5.
            let cells = split_h(30, 10, 5);
            let img = compose(seed, 30, 10, cell, &cells, &two_flat());
            let edge = 5 * cell.1;
            let mut off = 0;
            for x in 0..img.w {
                let mut first_up = img.h;
                for y in 0..img.h {
                    let p = at(&img, x, y);
                    if is_up(p) {
                        first_up = first_up.min(y);
                        assert!(y >= edge - 6, "{cell:?} seed {seed}: upper pixel ({x}, {y}) past the edge {edge}");
                    } else {
                        assert!(y < edge + 6, "{cell:?} seed {seed}: lower pixel ({x}, {y}) inside the edge {edge}");
                    }
                }
                off += (first_up != edge) as i32;
            }
            assert!(off * 2 >= img.w, "{cell:?} seed {seed}: {off} of {} columns off the cell edge", img.w);
            // Down the rows: the boundary at column 8.
            let cells = split_v(16, 30, 8);
            let img = compose(seed, 16, 30, cell, &cells, &two_flat());
            let edge = 8 * cell.0;
            let mut off = 0;
            for y in 0..img.h {
                let mut first_up = img.w;
                for x in 0..img.w {
                    let p = at(&img, x, y);
                    if is_up(p) {
                        first_up = first_up.min(x);
                        assert!(x >= edge - 6, "{cell:?} seed {seed}: upper pixel ({x}, {y}) past the edge {edge}");
                    } else {
                        assert!(x < edge + 6, "{cell:?} seed {seed}: lower pixel ({x}, {y}) inside the edge {edge}");
                    }
                }
                off += (first_up != edge) as i32;
            }
            assert!(off * 2 >= img.h, "{cell:?} seed {seed}: {off} of {} rows off the cell edge", img.h);
        }
    }
}

/// Check 3: a 1-cell lane over 30 cells keeps going: upper pixels in every column along its centre (the centre 2 art px
/// and 2 each side: a bite is possible where the noise is at its lowest, section 2.4, and is that small).
#[test]
fn a_one_cell_lane_never_breaks() {
    for cell in CELLS {
        for seed in 0..20 {
            let across: Vec<u8> = (0..9).flat_map(|y| (0..30).map(move |_| (y == 4) as u8)).collect();
            let img = compose(seed, 30, 9, cell, &across, &two_flat());
            let mid = 4 * cell.1 + cell.1 / 2;
            for x in 0..img.w {
                assert!((mid - 3..=mid + 2).any(|y| is_up(at(&img, x, y))), "{cell:?} seed {seed}: column {x}");
            }
            let down: Vec<u8> = (0..30).flat_map(|_| (0..9).map(move |x| (x == 4) as u8)).collect();
            let img = compose(seed, 9, 30, cell, &down, &two_flat());
            let mid = 4 * cell.0 + cell.0 / 2;
            for y in 0..img.h {
                assert!((mid - 3..=mid + 2).any(|x| is_up(at(&img, x, y))), "{cell:?} seed {seed}: row {y}");
            }
        }
    }
}

/// Finds where a texture lies in the image: the offset that matches a stretch of its first rows, if any.
fn texture_offset(img: &Image, tex: &Image, x0: i32, y0: i32) -> Option<(i32, i32)> {
    for oy in 0..tex.h {
        'next: for ox in 0..tex.w {
            for y in y0..y0 + 8 {
                for x in x0..x0 + 16 {
                    if at(img, x, y) != at(tex, (x + ox) % tex.w, (y + oy) % tex.h) {
                        continue 'next;
                    }
                }
            }
            return Some((ox, oy));
        }
    }
    None
}

/// Check 4: with one texture and tone 0 the output is the texture by art pixel, with no break at cell edges; with tone
/// above 0 no row or column equals the one a texture period away.
#[test]
fn a_single_ground_has_no_cell_lattice() {
    for cell in CELLS {
        let (tw, th) = (TEX_CELLS * cell.0, TEX_CELLS * cell.1);
        let tex = ground::two_tone(tw, th, [120, 100, 80, 255], [100, 84, 66, 255], 9);
        let cells = vec![0u8; 24 * 24];
        let plain = compose(4, 24, 24, cell, &cells, &[GroundLayer { tex: Some(&tex), ..GroundLayer::flat(0, LOW) }]);
        let (ox, oy) = texture_offset(&plain, &tex, 0, 0).expect("the texture at an offset");
        for y in 0..plain.h {
            for x in 0..plain.w {
                assert_eq!(at(&plain, x, y), at(&tex, (x + ox) % tw, (y + oy) % th), "{cell:?}: a cell-edge step at ({x}, {y})");
            }
        }
        let toned = compose(4, 24, 24, cell, &cells, &[GroundLayer { tex: Some(&tex), tone: 0.2, ..GroundLayer::flat(0, LOW) }]);
        for y in 0..toned.h - th {
            assert!((0..toned.w).any(|x| at(&toned, x, y) != at(&toned, x, y + th)), "{cell:?}: row {y} repeats");
        }
        for x in 0..toned.w - tw {
            assert!((0..toned.h).any(|y| at(&toned, x, y) != at(&toned, x + tw, y)), "{cell:?}: column {x} repeats");
        }
    }
}

/// A cell past a blend's reach is the texture exactly, on both sides of a boundary.
#[test]
fn a_cell_outside_a_blend_is_the_texture_exactly() {
    for cell in CELLS {
        let (tw, th) = (TEX_CELLS * cell.0, TEX_CELLS * cell.1);
        let low = ground::two_tone(tw, th, [120, 100, 80, 255], [100, 84, 66, 255], 1);
        let up = ground::two_tone(tw, th, [70, 90, 110, 255], [60, 80, 100, 255], 2);
        let layers = [GroundLayer { tex: Some(&low), ..GroundLayer::flat(0, LOW) }, GroundLayer { tex: Some(&up), wear: 0.3, strays: true, sunken: true, ..GroundLayer::flat(1, UP) }];
        let cells = split_h(20, 20, 10);
        let img = compose(8, 20, 20, cell, &cells, &layers);
        let reach = 18; // the blur's reach plus its margin, in art px
        for (tex, y0, y1) in [(&low, 0, 10 * cell.1 - reach), (&up, 10 * cell.1 + reach, 20 * cell.1)] {
            let (ox, oy) = texture_offset(&img, tex, 0, y0).expect("the texture at an offset");
            for y in y0..y1 {
                for x in 0..img.w {
                    assert_eq!(at(&img, x, y), at(tex, (x + ox) % tw, (y + oy) % th), "{cell:?}: ({x}, {y})");
                }
            }
        }
    }
}

/// Check 5: the tone has no period (it differs from itself shifted by 64 art px somewhere on every 32-cell row) and stays
/// within +/-(tone / 2).
#[test]
fn the_tone_has_no_period_and_keeps_its_range() {
    let tone = 0.2;
    let colour = [200, 200, 200, 255];
    let cells = vec![0u8; 32 * 12];
    for cell in CELLS {
        let img = compose(11, 32, 12, cell, &cells, &[GroundLayer { tone, ..GroundLayer::flat(0, colour) }]);
        let (lo, hi) = ((200.0 * (1.0 - tone / 2.0)).floor() as u8, (200.0 * (1.0 + tone / 2.0)).ceil() as u8);
        let (mut seen_lo, mut seen_hi) = (255u8, 0u8);
        for y in 0..img.h {
            assert!((0..img.w - 64).any(|x| at(&img, x, y) != at(&img, x + 64, y)), "{cell:?}: row {y} repeats at 64 art px");
            for x in 0..img.w {
                let v = at(&img, x, y)[0];
                assert!((lo..=hi).contains(&v), "{cell:?}: ({x}, {y}) = {v} out of {lo}..{hi}");
                (seen_lo, seen_hi) = (seen_lo.min(v), seen_hi.max(v));
            }
        }
        assert!(seen_lo < 200 && seen_hi > 200, "{cell:?}: the tone goes both ways ({seen_lo}..{seen_hi})");
    }
}

/// A lattice point's value depends on its place alone: a larger map has the smaller map's tone on the cells they share.
#[test]
fn the_noise_does_not_depend_on_the_map_size() {
    let layer = [GroundLayer { tone: 0.2, ..GroundLayer::flat(0, [200, 200, 200, 255]) }];
    let small = compose(11, 8, 6, SQUARE, &[0; 48], &layer);
    let large = compose(11, 16, 12, SQUARE, &[0; 192], &layer);
    for y in 0..small.h {
        for x in 0..small.w {
            assert_eq!(at(&small, x, y), at(&large, x, y));
        }
    }
}

/// Check 6: a lower ground under two meeting upper grounds shows nothing along their shared boundary's centre line.
#[test]
fn the_base_never_peeks_between_two_upper_grounds() {
    // Rows 4-15 hold the two uppers, split at column 12.
    let cells: Vec<u8> = (0..20).flat_map(|y| (0..24).map(move |x| if !(4..16).contains(&y) { 0 } else if x < 12 { 1 } else { 2 })).collect();
    let layers = [GroundLayer::flat(0, LOW), GroundLayer::flat(1, UP), GroundLayer::flat(2, THIRD)];
    for cell in CELLS {
        for seed in 0..20 {
            let img = compose(seed, 24, 20, cell, &cells, &layers);
            // Well inside the block: past the top and bottom edges' reach.
            for y in 4 * cell.1 + 16..16 * cell.1 - 16 {
                for x in 12 * cell.0 - 8..12 * cell.0 + 8 {
                    assert!(!is_low(at(&img, x, y)), "{cell:?} seed {seed}: the base at ({x}, {y})");
                }
            }
        }
    }
}

/// The layers' order, not their place in the list, decides: the same job with the list reversed and indices to match
/// gives the same shapes.
#[test]
fn paint_order_comes_from_order_not_the_list() {
    let cells = split_h(20, 10, 5);
    let swapped: Vec<u8> = cells.iter().map(|c| 1 - c).collect();
    let img = compose(3, 20, 10, SQUARE, &cells, &[GroundLayer::flat(0, LOW), GroundLayer::flat(1, UP)]);
    let other = compose(3, 20, 10, SQUARE, &swapped, &[GroundLayer::flat(1, UP), GroundLayer::flat(0, LOW)]);
    assert!(img == other);
}

/// Check 7: the worn band is taken once, never stacked: every pixel of the base is its colour or one wear of it.
#[test]
fn the_worn_band_never_stacks() {
    let base = [200, 200, 200, 255];
    let layers = [GroundLayer::flat(0, base), GroundLayer { wear: 0.2, ..GroundLayer::flat(1, UP) }, GroundLayer { wear: 0.3, ..GroundLayer::flat(2, THIRD) }];
    // Rows from 6 down: the two uppers side by side, so they end on one boundary.
    let cells: Vec<u8> = (0..12).flat_map(|y| (0..24).map(move |x| if y < 6 { 0 } else if x < 12 { 1 } else { 2 })).collect();
    let allowed = [rgb3(base), by(base, 0.8), by(base, 0.7)];
    let stacked = by(base, 0.8 * 0.7);
    for cell in CELLS {
        let mut seen = [0; 3];
        for seed in 0..10 {
            let img = compose(seed, 24, 12, cell, &cells, &layers);
            for y in 0..img.h {
                for x in 0..img.w {
                    let p = at(&img, x, y);
                    if p[0] == p[1] && p[1] == p[2] {
                        assert_ne!(p, stacked, "{cell:?} seed {seed}: a stacked wear at ({x}, {y})");
                        let k = allowed.iter().position(|a| *a == p);
                        assert!(k.is_some() || p == by(base, 0.95), "{cell:?} seed {seed}: {p:?} at ({x}, {y})");
                        if let Some(k) = k {
                            seen[k] += 1;
                        }
                    }
                }
            }
        }
        assert!(seen.iter().all(|n| *n > 0), "{cell:?}: both wears and the plain colour show ({seen:?})");
    }
}

/// Where two grounds meet, a third with cells out of reach shows nothing: no bite of it, no stray of it, not its wear,
/// though its mask runs under the upper one.
#[test]
fn a_ground_out_of_reach_shows_nothing_at_another_pair_s_edge() {
    // Lowest on the left half and most of the map; the middle ground only in a far corner; the upper ground on the right.
    let cells: Vec<u8> = (0..16).flat_map(|y| (0..30).map(move |x| if x >= 16 { 2 } else if x < 3 && y < 3 { 1 } else { 0 })).collect();
    let far = 3 * CELL_PX + 18; // past the corner's blur
    for cell in CELLS {
        for seed in 0..20 {
            let layers = [GroundLayer::flat(0, LOW), GroundLayer { wear: 0.3, strays: true, ..GroundLayer::flat(1, THIRD) }, GroundLayer { wear: 0.1, ..GroundLayer::flat(2, UP) }];
            let img = compose(seed, 30, 16, cell, &cells, &layers);
            let mut bites = 0;
            for y in 0..img.h {
                for x in far..img.w {
                    let p = at(&img, x, y);
                    assert!(p[1] < 100, "{cell:?} seed {seed}: the far ground's hue at ({x}, {y}): {p:?}");
                    assert!(p != by(LOW, 0.7), "{cell:?} seed {seed}: the far ground's wear at ({x}, {y})");
                    bites += (p == by(LOW, 0.95)) as i32;
                }
            }
            assert!(bites > 0, "{cell:?} seed {seed}: the upper ground bites down to the lowest");
            assert!((0..img.h).any(|y| (far..img.w).any(|x| at(&img, x, y) == by(LOW, 0.9))), "{cell:?} seed {seed}: the upper ground's own wear");
        }
    }
}

/// Where a middle ground and an upper one end on one boundary, the worn band goes from one's wear to the other's over a
/// stretch of the band, as the share fades: no column, or row in the other direction, switches over all at once (a
/// straight step at the blur's reach). Over 40 seeds, at most half of a line's strongly worn pixels turn weak in one step.
#[test]
fn the_worn_band_does_not_switch_wholesale_where_three_grounds_meet() {
    let layers = [GroundLayer::flat(0, LOW), GroundLayer { wear: 0.3, ..GroundLayer::flat(1, UP) }, GroundLayer { wear: 0.1, ..GroundLayer::flat(2, THIRD) }];
    let (strong, weak) = (by(LOW, 0.7), by(LOW, 0.9));
    let (cols, lines) = (32, 12);
    for cell in CELLS {
        // Along x: the lowest above line 6, under it the middle ground left of column 16 and the upper one right of it.
        let across: Vec<u8> = (0..lines).flat_map(|y| (0..cols).map(move |x| if y < 6 { 0 } else if x < 16 { 1 } else { 2 })).collect();
        // Along y: the same turned, in a grid as tall as the first is wide.
        let down: Vec<u8> = (0..cols).flat_map(|y| (0..lines).map(move |x| if x < 6 { 0 } else if y < 16 { 1 } else { 2 })).collect();
        let (mut along_x, mut band_x) = (vec![0; (cols * cell.0) as usize], vec![0; (cols * cell.0) as usize]);
        let (mut along_y, mut band_y) = (vec![0; (cols * cell.1) as usize], vec![0; (cols * cell.1) as usize]);
        for seed in 0..40 {
            let img = compose(seed, cols, lines, cell, &across, &layers);
            for x in 0..img.w - 1 {
                for y in 0..img.h {
                    let (p, q) = (at(&img, x, y), at(&img, x + 1, y));
                    band_x[x as usize] += (p == strong) as i32;
                    along_x[x as usize] += (p == strong && q == weak) as i32;
                }
            }
            let img = compose(seed, lines, cols, cell, &down, &layers);
            for y in 0..img.h - 1 {
                for x in 0..img.w {
                    let (p, q) = (at(&img, x, y), at(&img, x, y + 1));
                    band_y[y as usize] += (p == strong) as i32;
                    along_y[y as usize] += (p == strong && q == weak) as i32;
                }
            }
        }
        for (name, switched, band) in [("column", &along_x, &band_x), ("row", &along_y, &band_y)] {
            assert!(band.iter().sum::<i32>() > 0, "{cell:?}: the strong wear shows");
            for (i, (s, b)) in switched.iter().zip(band.iter()).enumerate() {
                assert!(s * 2 <= *b, "{cell:?}: {name} {i} switches {s} of its {b} strongly worn pixels at once");
            }
        }
    }
}

/// `wear` 0 and `strays` false give no pixel outside the grounds' own colours, by the tone, except the bites (section
/// 2.4: the ground under, x .95, which also has no wear and no stray).
#[test]
fn without_wear_or_strays_only_the_grounds_own_colours_show() {
    let cells = split_h(24, 12, 6);
    for cell in CELLS {
        for seed in 0..10 {
            let img = compose(seed, 24, 12, cell, &cells, &two_flat());
            let bite = by(LOW, 0.95);
            let mut bites = 0;
            for p in img.px.chunks_exact(4).map(|p| [p[0], p[1], p[2]]) {
                bites += (p == bite) as i32;
                assert!(p == rgb3(LOW) || p == rgb3(UP) || p == bite, "{cell:?} seed {seed}: {p:?}");
            }
            assert!(bites > 0, "{cell:?} seed {seed}: bites show");
            // With the tone, the same two hues within their range.
            let toned = [GroundLayer { tone: 0.2, ..GroundLayer::flat(0, LOW) }, GroundLayer { tone: 0.2, ..GroundLayer::flat(1, UP) }];
            let img = compose(seed, 24, 12, cell, &cells, &toned);
            for p in img.px.chunks_exact(4).map(|p| [p[0], p[1], p[2]]) {
                let up = is_up(p);
                let c = if up { UP } else { LOW };
                let (lo, hi) = ((c[0].max(c[2]) as f32 * 0.9 * 0.95).floor() as u8, (c[0].max(c[2]) as f32 * 1.1).ceil() as u8);
                let main = p[0].max(p[2]);
                assert!((lo..=hi).contains(&main), "{cell:?} seed {seed}: {p:?}");
            }
        }
    }
}

/// Check 7: a stray is the upper ground's colour x .92 on the lower side, at most 6 + 6 art px past the edge, and only
/// from a layer that says so.
#[test]
fn strays_stay_near_the_edge_and_come_only_from_a_layer_that_says_so() {
    let cells = split_h(30, 12, 6);
    let layers = [GroundLayer::flat(0, LOW), GroundLayer { strays: true, ..GroundLayer::flat(1, UP) }];
    let stray = by(UP, 0.92);
    for cell in CELLS {
        let edge = 6 * cell.1;
        let mut seen = 0;
        for seed in 0..20 {
            let img = compose(seed, 30, 12, cell, &cells, &layers);
            for y in 0..img.h {
                for x in 0..img.w {
                    if at(&img, x, y) == stray {
                        seen += 1;
                        assert!(y >= edge - 12, "{cell:?} seed {seed}: a stray at ({x}, {y}), edge {edge}");
                        assert!(y < edge, "{cell:?} seed {seed}: a stray inside the upper cells at ({x}, {y})");
                    }
                }
            }
            let none = compose(seed, 30, 12, cell, &cells, &two_flat());
            assert!(!none.px.chunks_exact(4).any(|p| [p[0], p[1], p[2]] == stray), "no strays without the flag");
        }
        assert!(seen > 0, "{cell:?}: strays show");
    }
}

/// Check 8: a sunken ground's rim: x .55 under a north neighbour, x .7 on the other sides, its plain colour inside; the
/// same when the sunken ground lies under a non-sunken one; in the tilted form also x .7 on the two rows after the first.
#[test]
fn a_sunken_ground_has_a_dark_rim_on_its_neighbours_side() {
    for cell in CELLS {
        for (sunken_up, plain) in [(true, UP), (false, LOW)] {
            let layers = [GroundLayer { sunken: !sunken_up, ..GroundLayer::flat(0, LOW) }, GroundLayer { sunken: sunken_up, ..GroundLayer::flat(1, UP) }];
            let cells: Vec<u8> = (0..16).flat_map(|y| (0..16).map(move |x| ((4..12).contains(&x) && (4..12).contains(&y)) as u8)).collect();
            let img = compose(21, 16, 16, cell, &cells, &layers);
            let (first, side) = (by(plain, 0.55), by(plain, 0.7));
            let same = |p: [u8; 3]| if sunken_up { is_up(p) } else { is_low(p) };
            let bite = by(LOW, 0.95);
            let (mut firsts, mut sides, mut plains) = (0, 0, 0);
            for y in 1..img.h - 1 {
                for x in 1..img.w - 1 {
                    let (p, north) = (at(&img, x, y), at(&img, x, y - 1));
                    if p == first {
                        firsts += 1;
                        assert!(!same(north) || north == bite, "{cell:?}: x .55 at ({x}, {y}) with its own ground north");
                    } else if p == side {
                        sides += 1;
                    } else if p == rgb3(plain) {
                        plains += 1;
                        // Next to it, its own ground, or a bite in it (which shows the ground under: still its top ground).
                        let inside = |q: [u8; 3]| same(q) || q == bite;
                        assert!(inside(north) && inside(at(&img, x - 1, y)) && inside(at(&img, x + 1, y)) && inside(at(&img, x, y + 1)), "{cell:?}: a plain pixel on the rim at ({x}, {y})");
                    }
                    // A pixel of the sunken ground with another ground north is darkened (a bite north shows the other
                    // ground's hue over the sunken ground: not a neighbour).
                    if same(p) && p != bite && !same(north) && north != bite {
                        assert!(p == first, "{cell:?}: ({x}, {y}) {p:?} north {north:?}");
                    }
                }
            }
            assert!(firsts > 0 && sides > 0 && plains > 0, "{cell:?}: {firsts} {sides} {plains}");
            if cell == TILTED_CELL {
                // The two rows after the first take x .7: under a first-row pixel lies a x .7 one.
                let mut below = 0;
                for y in 0..img.h - 2 {
                    for x in 0..img.w {
                        below += (at(&img, x, y) == first && at(&img, x, y + 1) == side && at(&img, x, y + 2) == side) as i32;
                    }
                }
                assert!(below > 0, "tilted: the deeper rim");
            }
        }
    }
}

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

/// A canvas of `w` x `h` physical px at its fit scale.
struct Frame {
    canvas: Canvas,
    fonts: Fonts,
    view: View,
}

impl Frame {
    fn new(w: u32, h: u32) -> Self {
        Self { canvas: Canvas::new(w as i32, h as i32), fonts: Fonts::new(&font()), view: View::fit(w, h) }
    }
    fn painter(&mut self) -> Painter<'_> {
        Painter { c: &mut self.canvas, v: self.view, fonts: &mut self.fonts }
    }
}

/// A job's parts, for the cache tests.
struct Fixture {
    cells: Vec<u8>,
    blocked: Vec<bool>,
    layers: Vec<GroundLayer<'static>>,
    cell: (i32, i32),
    cols: i32,
    rows: i32,
}

impl Fixture {
    fn new(cols: i32, rows: i32, cell: (i32, i32)) -> Self {
        let cells = split_h(cols, rows, rows / 2);
        let blocked = vec![false; cells.len()];
        Self { cells, blocked, layers: two_flat().to_vec(), cell, cols, rows }
    }
    fn job(&self) -> GroundJob<'_> {
        GroundJob { seed: 3, cols: self.cols, rows: self.rows, cell: self.cell, cells: &self.cells, blocked: &self.blocked, layers: &self.layers, placed: &[] }
    }
    fn key(&self, map: &str) -> GroundKey {
        GroundKey::of(map, &self.job())
    }
    fn draw(&self, p: &mut Painter, map: &str, r: Rect) {
        ground::draw(p, self.key(map), r, || ground::compose(&self.job()));
    }
}

/// Check 9: 100 draws of one key compose once and expand once.
#[test]
fn a_cache_hit_composes_and_expands_nothing() {
    let fx = Fixture::new(12, 8, SQUARE);
    let mut f = Frame::new(1280, 720);
    let r = Rect::new(0, 0, 192, 128);
    let (c0, e0) = (ground::ground_composes(), ground::ground_expands());
    for _ in 0..100 {
        fx.draw(&mut f.painter(), "m1", r);
    }
    assert_eq!((ground::ground_composes() - c0, ground::ground_expands() - e0), (1, 1));
    // A pan is a hit on both levels.
    for dx in 0..20 {
        fx.draw(&mut f.painter(), "m1", Rect::new(dx, dx, 192, 128));
    }
    assert_eq!((ground::ground_composes() - c0, ground::ground_expands() - e0), (1, 1));
}

/// A scale change (a window resize) makes one new physical copy from the art image, freeing the old one first; it does
/// not compose. Another job on the same map composes, once.
#[test]
fn a_scale_change_makes_a_new_copy_and_no_compose() {
    let fx = Fixture::new(12, 8, SQUARE);
    let r = Rect::new(0, 0, 192, 128);
    let (c0, e0) = (ground::ground_composes(), ground::ground_expands());
    let mut small = Frame::new(1280, 720);
    fx.draw(&mut small.painter(), "scale", r);
    let at_2x = ground::ground_held_bytes();
    let art_bytes = (12 * 16 * 8 * 16 * 4) as usize;
    assert!(at_2x >= art_bytes + 192 * 2 * 128 * 2 * 4);
    let mut large = Frame::new(1920, 1080);
    fx.draw(&mut large.painter(), "scale", r);
    assert_eq!((ground::ground_composes() - c0, ground::ground_expands() - e0), (1, 2), "one new copy, no compose");
    assert_eq!(ground::ground_held_bytes() - at_2x, 192 * 128 * (9 - 4) * 4, "the old copy is gone: only the size difference stays");
    fx.draw(&mut large.painter(), "scale", r);
    assert_eq!((ground::ground_composes() - c0, ground::ground_expands() - e0), (1, 2));
    fx.draw(&mut small.painter(), "scale", r);
    assert_eq!((ground::ground_composes() - c0, ground::ground_expands() - e0), (1, 3), "back: a copy again, the art image held");
}

/// The key holds a texture's real content: a retouch of one pixel the old sampled hash would never read is a new key, and
/// two copies of one texture at different addresses are the same key.
#[test]
fn the_key_holds_a_textures_content_not_its_address() {
    let key = |t: &Image| {
        let layers = [GroundLayer { tex: Some(t), ..GroundLayer::flat(0, LOW) }];
        GroundKey::of("t", &GroundJob { seed: 1, cols: 2, rows: 2, cell: SQUARE, cells: &[0; 4], blocked: &[false; 4], layers: &layers, placed: &[] })
    };
    let tex = ground::two_tone(64, 64, [90, 80, 70, 255], [84, 74, 64, 255], 1);
    let clone = tex.clone();
    assert_ne!(tex.px.as_ptr(), clone.px.as_ptr());
    assert_eq!(key(&tex), key(&clone), "two copies, one key");
    // A sampled hash reads every 16th pixel of 4096: pixel 17 is not among them.
    let mut retouched = tex.clone();
    retouched.px[17 * 4] ^= 1;
    assert_ne!(key(&tex), key(&retouched), "a retouch is a new key");
    // The size counts too, with the same bytes.
    let flat = Image { w: 32, h: 128, px: tex.px.clone() };
    assert_ne!(key(&tex), key(&flat));
    // Two clones share one composed image through the cache.
    let c0 = ground::ground_composes();
    let mut f = Frame::new(640, 360);
    for t in [&tex, &clone] {
        let layers = [GroundLayer { tex: Some(t), ..GroundLayer::flat(0, LOW) }];
        let job = GroundJob { seed: 1, cols: 2, rows: 2, cell: SQUARE, cells: &[0; 4], blocked: &[false; 4], layers: &layers, placed: &[] };
        ground::draw(&mut f.painter(), GroundKey::of("clones", &job), Rect::new(0, 0, 32, 32), || ground::compose(&job));
    }
    assert_eq!(ground::ground_composes() - c0, 1);
}

/// Two views of one map compose twice; flipping back only copies; a new content hash composes; past 4 art images the
/// oldest goes.
#[test]
fn views_and_maps_keep_their_art_images() {
    let square = Fixture::new(12, 8, SQUARE);
    let tilted = Fixture::new(12, 8, TILTED_CELL);
    assert_ne!(square.key("m"), tilted.key("m"), "the cell size is in the key");
    let mut f = Frame::new(1280, 720);
    let r = Rect::new(0, 0, 192, 128);
    let c0 = ground::ground_composes();
    square.draw(&mut f.painter(), "m", r);
    tilted.draw(&mut f.painter(), "m", r);
    assert_eq!(ground::ground_composes() - c0, 2, "two views");
    square.draw(&mut f.painter(), "m", r);
    tilted.draw(&mut f.painter(), "m", r);
    assert_eq!(ground::ground_composes() - c0, 2, "flipping back");
    // A changed grid or layer is a new hash.
    let mut edited = Fixture::new(12, 8, SQUARE);
    edited.cells[0] = 1;
    assert_ne!(edited.key("m"), square.key("m"));
    edited.draw(&mut f.painter(), "m", r);
    assert_eq!(ground::ground_composes() - c0, 3, "an edit");
    let mut retuned = Fixture::new(12, 8, SQUARE);
    retuned.layers[1].wear = 0.2;
    assert_ne!(retuned.key("m"), square.key("m"));
    let tex = ground::two_tone(64, 64, [1, 2, 3, 255], [4, 5, 6, 255], 1);
    let other = ground::two_tone(64, 64, [9, 8, 7, 255], [6, 5, 4, 255], 1);
    let with = |t: &'_ Image| {
        let layers = [GroundLayer { tex: Some(t), ..GroundLayer::flat(0, LOW) }];
        let blocked = vec![false; 4];
        GroundKey::of("t", &GroundJob { seed: 1, cols: 2, rows: 2, cell: SQUARE, cells: &[0; 4], blocked: &blocked, layers: &layers, placed: &[] })
    };
    assert_ne!(with(&tex), with(&other), "a texture's identity is in the key");
    // Five maps: the first is pushed out, the last ones stay.
    let c1 = ground::ground_composes();
    for m in ["a", "b", "c", "d", "e"] {
        square.draw(&mut f.painter(), m, r);
    }
    assert_eq!(ground::ground_composes() - c1, 5);
    square.draw(&mut f.painter(), "e", r);
    square.draw(&mut f.painter(), "b", r);
    assert_eq!(ground::ground_composes() - c1, 5, "among the last 4");
    square.draw(&mut f.painter(), "a", r);
    assert_eq!(ground::ground_composes() - c1, 6, "pushed out");
}

/// The blit equals the same image through `Painter::image` at a whole scale, wherever the rect lies and whatever the clip
/// and bounds; in both forms.
#[test]
fn the_blit_equals_painter_image_at_a_whole_scale() {
    for (w, h) in [(640u32, 360u32), (1280, 720), (1920, 1080)] {
        for cell in CELLS {
            let fx = Fixture::new(12, 8, cell);
            let art = ground::compose(&fx.job());
            let r = Rect::new(-16, 12, 192, 8 * 16 * cell.1 / cell.0);
            let inner = Rect::new(40, 30, 200, 100);
            for (clip, bounds) in [(None, None), (Some(inner), None), (None, Some(inner)), (Some(Rect::new(0, 0, 90, 60)), Some(inner))] {
                let mut live = Frame::new(w, h);
                let mut blit = Frame::new(w, h);
                for frame in [&mut live, &mut blit] {
                    frame.canvas.clear([9, 9, 9, 255]);
                    frame.canvas.clip = clip;
                    frame.canvas.bounds = bounds;
                }
                live.painter().image(&art, r);
                fx.draw(&mut blit.painter(), "blit", r);
                assert!(live.canvas.px == blit.canvas.px, "{w} x {h} {cell:?} clip {clip:?} bounds {bounds:?}");
            }
        }
    }
}

/// At a fractional window scale the rect's rounded edges differ from place to place by a pixel: a pan keeps one copy and
/// leaves no pixel of the rect undrawn.
#[test]
fn a_pan_at_a_fractional_scale_keeps_its_copy_and_leaves_no_gap() {
    let fx = Fixture::new(12, 8, SQUARE);
    let mut f = Frame::new(1366, 768);
    assert_ne!(f.view.s.fract(), 0.0, "a fractional scale");
    let e0 = ground::ground_expands();
    let mut seen = std::collections::BTreeSet::new();
    for dx in 0..40 {
        let r = Rect::new(dx, dx, 192, 128);
        f.canvas.clear([255, 0, 255, 255]);
        fx.draw(&mut f.painter(), "pan", r);
        let pr = f.view.rect(r);
        seen.insert((pr.w, pr.h));
        for y in pr.y.max(0)..pr.y + pr.h {
            for x in pr.x.max(0)..pr.x + pr.w {
                let i = ((y * f.canvas.w + x) * 4) as usize;
                assert_ne!(&f.canvas.px[i..i + 3], &[255, 0, 255], "a gap at ({x}, {y}), dx {dx}");
            }
        }
    }
    assert!(seen.len() > 1, "the rect's size does change along the pan: {seen:?}");
    assert_eq!(ground::ground_expands() - e0, 1);
}

/// A copy past `PHYSICAL_MAX` is not kept: the art image draws through `Painter::image`, the same pixels.
#[test]
fn a_map_past_the_copy_limit_draws_through_the_painter() {
    let fx = Fixture::new(12, 8, SQUARE);
    // Past the limit at this window's scale of 4.
    let (w, h) = (2560u32, 1440u32);
    let s = View::fit(w, h).s as i32;
    let side = (((PHYSICAL_MAX / 4) as f64).sqrt() as i32 + 1) / s + 1;
    let r = Rect::new(0, 0, side, side);
    let art = ground::compose(&fx.job());
    let e0 = ground::ground_expands();
    let (mut drawn, mut live) = (Frame::new(w, h), Frame::new(w, h));
    fx.draw(&mut drawn.painter(), "large", r);
    live.painter().image(&art, r);
    assert_eq!(ground::ground_expands() - e0, 0, "no copy");
    assert!(drawn.canvas.px == live.canvas.px);
}

/// Past the limit no old copy stays held, and `expand` of nothing allocates nothing.
#[test]
fn the_fallback_frees_the_old_copy() {
    let fx = Fixture::new(12, 8, SQUARE);
    let mut f = Frame::new(1280, 720);
    fx.draw(&mut f.painter(), "fb", Rect::new(0, 0, 192, 128));
    let with_copy = ground::ground_held_bytes();
    let art = (192 * 128 * 4) as usize;
    assert_eq!(with_copy - art, 192 * 2 * 128 * 2 * 4);
    let side = ((PHYSICAL_MAX / 4) as f64).sqrt() as i32 / 2 + 2;
    fx.draw(&mut f.painter(), "fb", Rect::new(0, 0, side, side));
    assert_eq!(ground::ground_held_bytes(), art, "the art image only");
    let art_img = ground::compose(&fx.job());
    assert!(ground::expand(&art_img, 0, 9).is_empty() && ground::expand(&art_img, -3, -4).is_empty());
}

/// The spec's memory table, for the largest shipped map, at 4K: the copy and an art image.
#[test]
fn the_largest_map_holds_what_the_spec_says() {
    let fx = Fixture::new(48, 30, SQUARE);
    let mut f = Frame::new(3840, 2160);
    fx.draw(&mut f.painter(), "big", Rect::new(0, 0, 48 * 16, 30 * 16));
    let held = ground::ground_held_bytes();
    assert!(held >= (4608 * 2880 * 4 + 768 * 480 * 4) as usize, "{held}");
    assert!(held <= 59_100_000, "{held}");
}

/// `soft_ellipse`: alpha highest at the centre, 0 at and past the radii, monotonic in between, the same twice.
#[test]
fn soft_ellipse_falls_from_the_centre_to_nothing() {
    for (rx, ry) in [(7, 3), (3, 7), (16, 16), (1, 1)] {
        let col = [10, 20, 30, 200];
        let img = ground::soft_ellipse(rx, ry, col, 90);
        assert_eq!((img.w, img.h), (2 * rx + 1, 2 * ry + 1));
        assert!(img == ground::soft_ellipse(rx, ry, col, 90));
        let a = |x: i32, y: i32| img.px[(((y + ry) * img.w + x + rx) * 4 + 3) as usize];
        assert_eq!(a(0, 0), 90, "the centre");
        for y in -ry..=ry {
            for x in -rx..=rx {
                let i = (((y + ry) * img.w + x + rx) * 4) as usize;
                assert_eq!(&img.px[i..i + 3], &col[..3], "its colour");
                let d2 = (x * x) as f32 / (rx * rx) as f32 + (y * y) as f32 / (ry * ry) as f32;
                if d2 >= 1.0 {
                    assert_eq!(a(x, y), 0, "at and past the radii ({x}, {y})");
                }
                // Toward the centre alpha never falls.
                if x != 0 {
                    assert!(a(x - x.signum(), y) >= a(x, y));
                }
                if y != 0 {
                    assert!(a(x, y - y.signum()) >= a(x, y));
                }
            }
        }
    }
    assert_eq!(ground::soft_ellipse(0, 4, [0, 0, 0, 255], 90).px.len(), 0);
}

fn gfx(scale: f32) -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.fonts.scale = scale;
    g.fonts.audit = Some(Vec::new());
    g.canvas.clear(Theme::default().bg);
    g
}

fn page(k: &Specimens, g: &mut Gfx) -> (Vec<Widget<KitAct>>, Value) {
    const KIT: Rect = Rect { x: 112, y: 22, w: 522, h: 332 };
    let w = {
        let mut ui: Ui<KitAct> = Ui::new(g, (-1, -1), true);
        k.draw(&mut ui, KIT, |a| a);
        ui.widgets
    };
    let boxes: Vec<_> = w.iter().map(|x| (x.id.clone(), x.rect.x as f32, x.rect.y as f32, x.rect.w as f32, x.rect.h as f32)).collect();
    let recs = g.fonts.audit.clone().unwrap_or_default();
    (w, text_audit::issues(&recs, 640.0, 360.0, &boxes))
}

/// Check 10: the Tiles page's Ground view draws at 1x-3x, square and tilted, with no text cut, and only composes when
/// the view changes.
#[test]
fn the_gallery_ground_view_draws_every_view_and_zoom_clean() {
    for view in 0..TILE_VIEWS.len() {
        for zoom in 0..TILE_ZOOMS.len() {
            let mut k = Specimens::default();
            k.act(KitAct::Page(6));
            k.act(KitAct::TileGround);
            k.act(KitAct::TileView(view));
            k.act(KitAct::TileZoom(zoom));
            assert!(k.tiles.ground);
            for scale in [0.85, 1.0, 1.15, 1.3] {
                let mut g = gfx(scale);
                let (w, audit) = page(&k, &mut g);
                for kind in ["edge", "overlap", "spill"] {
                    assert_eq!(audit["counts"][kind], json!(0), "view {view} zoom {zoom} at {scale}: {kind}: {audit}");
                }
                let tab = w.iter().find(|x| x.id == "kit.tiles.ground").expect("the switch");
                assert!(tab.rect.x + tab.rect.w <= 112 + 522, "the switch on the page at {scale}");
                // The stage shows the composed fixture: its corner is the lowest ground's hue, not the stage colour.
                let m = w.iter().find(|x| x.id == "kit.tiles.map").unwrap().rect;
                let i = ((m.y * g.canvas.w + m.x) * 4) as usize;
                let corner = [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2]];
                assert!(corner[0] > corner[2] && corner[0] > 40, "view {view} zoom {zoom}: {corner:?}");
            }
        }
    }
    // Redrawing composes nothing; the other view composes once, then flipping back nothing.
    let mut k = Specimens::default();
    k.act(KitAct::Page(6));
    k.act(KitAct::TileGround);
    page(&k, &mut gfx(1.0));
    let c0 = ground::ground_composes();
    for _ in 0..5 {
        page(&k, &mut gfx(1.0));
    }
    assert_eq!(ground::ground_composes(), c0, "the same view is a hit");
    // The switch back shows the tile view again.
    k.act(KitAct::TileGround);
    assert!(!k.tiles.ground);
    let (w, _) = page(&k, &mut gfx(1.0));
    assert!(w.iter().any(|x| x.id == "kit.tiles.ground"));
    assert_eq!(ground::ground_composes(), c0);
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("aegis_ground_{}_{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_png(path: &Path, img: &Image) {
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img.px).unwrap();
}

/// The tools' way in draws what `compose` draws, pixel for pixel (a board's picture is the game's), and refuses a job it
/// cannot read rather than guessing.
#[test]
fn the_job_file_composes_the_games_picture() {
    let dir = scratch("job");
    let tex = ground::two_tone(64, 64, [120, 100, 80, 255], [100, 84, 66, 255], 1);
    write_png(&dir.join("low.png"), &tex);
    // A larger file is reduced to the job's density, as `Art::image` hands it to the game.
    let large = ground::two_tone(128, 128, [70, 90, 110, 255], [60, 80, 100, 255], 2);
    write_png(&dir.join("up.png"), &large);
    let job = json!({
        "seed": 12, "cell": [16, 16],
        "cells": ["aaaa", "abba", "abba", "aaaa"],
        "legend": {
            "a": {"tex": "low.png", "tone": 0.13},
            "b": {"tex": "up.png", "tone": 0.13, "wear": 0.1, "strays": true, "order": 3},
        },
    });
    std::fs::write(dir.join("job.json"), job.to_string()).unwrap();
    let out = dir.join("out.png");
    let made = ground::compose_file(&dir.join("job.json"), &out).unwrap();
    assert_eq!((made.w, made.h), (64, 64));

    let reduced = large.downsample(64, 64);
    let layers = [GroundLayer { tex: Some(&tex), tone: 0.13, ..GroundLayer::flat(0, [0, 0, 0, 255]) }, GroundLayer { tex: Some(&reduced), tone: 0.13, wear: 0.1, strays: true, ..GroundLayer::flat(3, [0, 0, 0, 255]) }];
    let cells = [0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
    let want = compose(12, 4, 4, SQUARE, &cells, &layers);
    let got = aegis_ui::art::load_png(&out).unwrap();
    assert!(got == want, "the file is the game's picture");

    for (bad, says) in [
        (json!({"cells": ["a"], "legend": {"a": {"colour": [1, 2, 3]}}, "typo": 1}), "unknown field"),
        (json!({"cells": ["ab"], "legend": {"a": {"colour": [1, 2, 3]}}}), "not in the legend"),
        (json!({"cells": ["a", "aa"], "legend": {"a": {"colour": [1, 2, 3]}}}), "wide"),
        (json!({"cells": ["a"], "legend": {"a": {}}}), "needs tex or colour"),
        (json!({"cells": ["a"], "legend": {"a": {"tex": "missing.png"}}}), "cannot read"),
        (json!({"cells": ["a"], "legend": {"a": {"colour": [1, 2, 3], "wer": 0.1}}}), "unknown field"),
        (json!({"cell": [2147483647, 16], "cells": ["a"], "legend": {"a": {"colour": [1, 2, 3]}}}), "cell must be"),
        (json!({"cell": [100000, 100000], "cells": ["a"], "legend": {"a": {"colour": [1, 2, 3]}}}), "cell must be"),
        (json!({"cell": [16.9, 16], "cells": ["a"], "legend": {"a": {"colour": [1, 2, 3]}}}), "cell must be"),
        (json!({"cell": [0, 16], "cells": ["a"], "legend": {"a": {"colour": [1, 2, 3]}}}), "cell must be"),
        (json!({"cell": [33, 16], "cells": ["a"], "legend": {"a": {"colour": [1, 2, 3]}}}), "cell must be"),
        (json!({"cells": ["a"], "legend": {"a": {"colour": [300, 2, 3]}}}), "colour must be"),
        (json!({"cells": ["a"], "legend": {"a": {"colour": [1, -5, 3]}}}), "colour must be"),
        (json!({"cells": ["a"], "legend": {"a": {"colour": [1, 2, 1.5]}}}), "colour must be"),
        (json!({"cells": ["ab"], "legend": {"a": {"colour": [1, 2, 3], "order": 2}, "b": {"colour": [1, 2, 3], "order": 2}}}), "duplicate order"),
        (json!({"cells": ["a"], "legend": {"a": {"colour": [1, 2, 3], "order": 1.5}}}), "order must be"),
        (json!({"cells": ["a"], "legend": {"a": {"colour": [1, 2, 3], "marks_per_100": 11}}}), "marks_per_100 must be"),
        (json!({"cell": [32, 32], "cells": ["a".repeat(20_000)], "legend": {"a": {"colour": [1, 2, 3]}}}), "too large"),
    ] {
        std::fs::write(dir.join("bad.json"), bad.to_string()).unwrap();
        let e = ground::compose_file(&dir.join("bad.json"), &dir.join("bad.png")).unwrap_err();
        assert!(format!("{e:#}").contains(says), "{e:#}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
