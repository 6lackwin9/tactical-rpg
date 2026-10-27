//! PORTRAITFX1-E (docs/specs/PORTRAITFX1.md section 4, checks E2-E15 of PORTRAITFX1_TASKS.md 1.1; E1 is in
//! `aegis_core::assets`' unit tests): `Art::first_rel` and `Art::face`, the portrait effects (base life, overlays,
//! states, reactions, composition order, cues and holds, seeds, size tiers, reduce motion, cache and budget, agent
//! state) and the Kit widgets Portraits page. Expectations follow from the spec's rules; images are built here and
//! files go to scratch folders under the temp dir.

use aegis_core::assets::{Face, Layers};
use aegis_ui::art::Art;
use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::fx::{hash01, Axis, Envelope, Shake};
use aegis_ui::painter::{Fonts, Painter, TextRec, View};
use aegis_ui::portrait_fx::*;
use aegis_ui::specimens::{KitAct, Specimens, PORTRAIT_OVERLAYS, PORTRAIT_REACTIONS, PORTRAIT_SIZES, PORTRAIT_STATES};
use aegis_ui::text_audit;
use aegis_ui::ui::{by_id, Gfx, Kit, Ui, Widget, BG};
use serde_json::json;
use std::path::{Path, PathBuf};

const EDGE: Rgba = [250, 0, 250, 255];
const GREY: Rgba = [100, 100, 100, 255];
const SKIN: Rgba = [164, 122, 95, 255];
const AMBER: Rgba = [217, 119, 6, 255];

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

/// An `n` x `n` image of `f(x, y)`.
fn image(n: i32, f: impl Fn(i32, i32) -> Rgba) -> Image {
    let mut px = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            px.extend_from_slice(&f(x, y));
        }
    }
    Image { w: n, h: n, px }
}

fn at(img: &Image, x: i32, y: i32) -> Rgba {
    let i = ((y * img.w + x) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]
}

fn in_border(n: i32, b: i32, x: i32, y: i32) -> bool {
    x < b || y < b || x >= n - b || y >= n - b
}

/// The default border of a plain `n` face (`round(n / 30)`), `EDGE` coloured; inside `inner(x, y)`.
fn framed(n: i32, inner: impl Fn(i32, i32) -> Rgba) -> Image {
    let b = Face::plain(n).border;
    image(n, |x, y| if in_border(n, b, x, y) { EDGE } else { inner(x, y) })
}

/// Rows told apart by their red channel (y), so a moved row shows.
fn rows(n: i32) -> Image {
    framed(n, |x, y| [y as u8, 40 + (x % 100) as u8, 60, 255])
}

fn canvas_px(c: &Canvas, x: i32, y: i32) -> Rgba {
    let i = ((y * c.w + x) * 4) as usize;
    [c.px[i], c.px[i + 1], c.px[i + 2], c.px[i + 3]]
}

/// A 640 x 360 canvas at scale 1 (logical px = device px) and its fonts.
struct Rig {
    c: Canvas,
    fonts: Fonts,
}

impl Rig {
    fn new() -> Rig {
        Rig { c: Canvas::new(640, 360), fonts: Fonts::new(&font()) }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw(&mut self, fx: &mut PortraitFx, src: &Image, face: Option<&Face>, r: Rect, now: u64, opts: &PortraitOpts, budget: &mut Budget) {
        let mut p = Painter { c: &mut self.c, v: View::fit(640, 360), fonts: &mut self.fonts };
        fx.draw_image(&mut p, src, "src", face, r, now, opts, budget);
    }

    /// The composed image of `fx` drawn at `side` px at `now`.
    fn compose(&mut self, fx: &mut PortraitFx, src: &Image, face: Option<&Face>, side: i32, now: u64, opts: PortraitOpts) -> Image {
        self.draw(fx, src, face, Rect::new(8, 8, side, side), now, &opts, &mut Budget::default());
        fx.composed().unwrap().clone()
    }
}

fn reduce() -> PortraitOpts {
    PortraitOpts { reduce_motion: true, ..PortraitOpts::default() }
}

fn cue(effect: &str) -> Cue {
    Cue::parse(effect, "").unwrap()
}

/// FNV-1a 64 written out from its definition.
fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_portrait_fx_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
}

fn write_png(path: &Path, img: &Image) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img.px).unwrap();
}

#[test]
fn e2_first_rel_and_the_sidecar_of_the_pngs_own_layer() {
    let root = scratch("layers");
    let (game, kit) = (root.join("game"), root.join("kit"));
    let src = rows(96);
    write_png(&kit.join("portraits/common/base.png"), &src);
    std::fs::write(kit.join("portraits/common/base.face.json"), r#"{"size": 96, "eyes": [[40, 40, 6, 4]]}"#).unwrap();
    write_png(&game.join("portraits/npcs/own.png"), &src);
    // A kit sidecar for a game PNG: not the PNG's layer, so never read.
    std::fs::create_dir_all(kit.join("portraits/npcs")).unwrap();
    std::fs::write(kit.join("portraits/npcs/own.face.json"), r#"{"size": 96}"#).unwrap();
    let mut art = Art::layered(Layers::of(vec![game.clone(), kit.clone()]));
    let chain = vec!["portraits/npcs/nobody.png".to_string(), "portraits/common/base.png".to_string()];
    assert_eq!(art.first_rel(&chain).as_deref(), Some("portraits/common/base.png"), "the first path a layer holds");
    assert_eq!(art.first_rel(&chain[..1]), None);
    assert_eq!(art.face("portraits/common/base.png").map(|f| f.eyes.clone()), Some(vec![[40, 40, 6, 4]]));
    assert!(art.face("portraits/npcs/own.png").is_none(), "the game PNG's layer holds no sidecar");
    std::fs::remove_file(kit.join("portraits/common/base.face.json")).unwrap();
    assert!(art.face("portraits/common/base.png").is_some(), "cached");
    // `draw` resolves the PNG through the layers; a missing file returns false for the caller's fallback.
    let mut rig = Rig::new();
    let mut fx = PortraitFx::new("e2");
    let ok = {
        let mut p = Painter { c: &mut rig.c, v: View::fit(640, 360), fonts: &mut rig.fonts };
        let missing = fx.draw(&mut p, &mut art, "portraits/npcs/nobody.png", None, Rect::new(0, 0, 40, 40), 0, &PortraitOpts::default(), &mut Budget::default());
        let found = fx.draw(&mut p, &mut art, "portraits/common/base.png", None, Rect::new(0, 0, 40, 40), 0, &PortraitOpts::default(), &mut Budget::default());
        (missing, found)
    };
    assert_eq!(ok, (false, true));
    assert_eq!(fx.composed().map(|i| (i.w, i.h)), Some((40, 40)), "composed at the drawn side");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn e3_the_same_key_and_time_give_the_same_pixels_and_the_seed_is_fnv1a() {
    assert_eq!((fnv1a64(""), fnv1a64("a")), (0xcbf29ce484222325, 0xaf63dc4c8601ec8c), "the published FNV-1a 64 vectors");
    assert_eq!(PortraitFx::new("tess").seed, fnv1a64("tess"));
    assert_eq!(seed_of("unit_7"), fnv1a64("unit_7"));
    let (img, face) = sample();
    let mut rig = Rig::new();
    let draws: Vec<Image> = (0..2)
        .map(|_| {
            let mut fx = PortraitFx::new("tess");
            fx.hold(&[cue("glow"), cue("ash")], 0);
            rig.compose(&mut fx, &img, Some(&face), 96, 4321, PortraitOpts::default())
        })
        .collect();
    assert_eq!(draws[0], draws[1], "two portraits of one key at one time draw the same pixels");
    let first = |key: &str| blink_starts(seed_of(key), 20_000)[0];
    assert_ne!(first("tess"), first("bram"), "two keys blink at different times");
    for key in ["tess", "bram"] {
        assert_eq!(first(key), (2600.0 + 3400.0 * hash01(seed_of(key), 200) as f64).round() as u64);
    }
}

#[test]
fn e4_breath_lifts_the_inner_rect_from_its_bottom_edge_only_at_full_tier() {
    let mut rig = Rig::new();
    let n = 240;
    let src = rows(n);
    let mut fx = PortraitFx::new("breath");
    let half = (breath_period(fx.seed) / 2.0).round() as u64;
    let out = rig.compose(&mut fx, &src, None, n, half, PortraitOpts::default());
    let (b, amp) = (Face::plain(n).border, breath_amp(n));
    assert_eq!((b, amp), (8, 3), "round(240 / 30), max(1, round(0.012 x 240))");
    for x in [b, n / 2, n - b - 1] {
        assert_eq!(at(&out, x, b), at(&src, x, b + amp), "the inner top row shows the row {amp} px below it");
        assert_eq!(at(&out, x, n - b - 1), at(&src, x, n - b - 1), "the inner bottom row stays");
    }
    for y in 0..n {
        for x in 0..n {
            if in_border(n, b, x, y) {
                assert_eq!(at(&out, x, y), at(&src, x, y), "the border never moves ({x}, {y})");
            }
        }
    }
    let mut still = PortraitFx::new("breath");
    still.hold(&[cue("still")], 0);
    assert_eq!(rig.compose(&mut still, &src, None, n, half, PortraitOpts::default()), src, "none under still");
    assert_eq!(rig.compose(&mut PortraitFx::new("breath"), &src, None, n, half, reduce()), src, "none under reduce motion");
    let small = rows(64);
    let mut fx = PortraitFx::new("breath");
    let half = (breath_period(fx.seed) / 2.0).round() as u64;
    assert_eq!(rig.compose(&mut fx, &small, None, 64, half, PortraitOpts::default()), small, "none at the reduced tier");
}

#[test]
fn e5_rim_flickers_key_coloured_pixels_only() {
    let mut rig = Rig::new();
    let n = 96;
    let src = image(n, |x, y| if in_border(n, 3, x, y) || x < 48 { AMBER } else { GREY });
    let face = Face { rim: Some([217, 119, 6]), ..Face::plain(n) };
    assert!(rim_lit(AMBER, AMBER) && !rim_lit(GREY, AMBER));
    let mut seen = std::collections::BTreeSet::new();
    let mut fx = PortraitFx::new("rim");
    for now in (0..6000).step_by(97) {
        let out = rig.compose(&mut fx, &src, Some(&face), n, now, PortraitOpts::default());
        let f = rim_factor(fx.seed, tq(now));
        assert!((0.88 - 1e-6..=1.12 + 1e-6).contains(&f), "f stays in 1 +- 0.12: {f}");
        let c = at(&out, 20, 40);
        assert_eq!(c[0], (217.0 * f).round().min(255.0) as u8, "value x f at {now}");
        seen.insert(c[0]);
        assert_eq!(at(&out, 70, 40), GREY, "a grey pixel never changes");
        assert_eq!(at(&out, 1, 40), AMBER, "nor the border");
    }
    assert!(seen.len() > 3, "it flickers over time: {seen:?}");
    let mut steady = PortraitFx::new("rim");
    for now in [0, 700, 2900] {
        assert_eq!(rig.compose(&mut steady, &src, Some(&face), n, now, reduce()), src, "steady under reduce motion");
    }
    let mut keyless = PortraitFx::new("rim");
    assert_eq!(rig.compose(&mut keyless, &src, None, n, 1234, PortraitOpts::default()), src, "no key, no flicker");
}

/// A face with one 6 x 4 eye box at (40, 40) on skin.
fn eye_face(blink: bool) -> (Image, Face) {
    let n = 96;
    let img = framed(n, |x, y| if (40..46).contains(&x) && (40..44).contains(&y) { [230, 230, 230, 255] } else { SKIN });
    (img, Face { eyes: vec![[40, 40, 6, 4]], blink, ..Face::plain(n) })
}

/// A key whose first blink is double (h(300) < 0.12).
fn double_key() -> String {
    (0..).map(|i| format!("double{i}")).find(|k| hash01(seed_of(k), 300) < 0.12).unwrap()
}

#[test]
fn e6_blinks_close_the_eye_boxes_on_their_schedule_even_under_reduce_motion() {
    let mut rig = Rig::new();
    let (img, face) = eye_face(true);
    let mut fx = PortraitFx::new("blinker");
    let first = blink_starts(fx.seed, 20_000)[0];
    assert_eq!(first, (2600.0 + 3400.0 * hash01(fx.seed, 200) as f64).round() as u64);
    assert_eq!(rig.compose(&mut fx, &img, Some(&face), 96, first - 40, reduce()), img, "open before it");
    let closed = rig.compose(&mut fx, &img, Some(&face), 96, first + 75, reduce());
    let lash = 40 + 2 * 4 / 3;
    let dark = [(164.0 * 0.6) as u8, (122.0 * 0.6) as u8, (95.0 * 0.6) as u8, 255];
    for y in 40..44 {
        for x in 40..46 {
            assert_eq!(at(&closed, x, y), if y == lash { dark } else { SKIN }, "closed: the lid sampled 1 px above ({x}, {y})");
        }
    }
    let half = rig.compose(&mut fx, &img, Some(&face), 96, first + 38, reduce());
    assert_eq!((at(&half, 42, 40), at(&half, 42, 41), at(&half, 42, 42)), (SKIN, SKIN, [230, 230, 230, 255]), "half closed: the top half");
    assert_eq!(rig.compose(&mut fx, &img, Some(&face), 96, first + BLINK_MS + 40, reduce()), img, "open again after 130 ms");

    let key = double_key();
    let starts = blink_starts(seed_of(&key), 20_000);
    assert_eq!(starts[1] - starts[0], DOUBLE_BLINK_MS, "a double blink 220 ms apart");
    let mut twice = PortraitFx::new(&key);
    let second = rig.compose(&mut twice, &img, Some(&face), 96, starts[1] + 75, reduce());
    assert_eq!(at(&second, 42, 41), SKIN, "the second closes too");

    let (_, lidless) = eye_face(false);
    assert_eq!(rig.compose(&mut PortraitFx::new("blinker"), &img, Some(&lidless), 96, first + 75, reduce()), img, "blink false: none");
    assert_eq!(rig.compose(&mut PortraitFx::new("blinker"), &img, Some(&Face::plain(96)), 96, first + 75, reduce()), img, "no eyes: none");
}

#[test]
fn e7_overlays_glow_veins_eye_glow_and_ash() {
    let mut rig = Rig::new();
    let n = 96;
    let flat = framed(n, |_, _| GREY);
    // Glow with no anchor: centred at (0.5, 0.7) of the inner rect.
    let mut fx = PortraitFx::new("glow");
    fx.hold(&[cue("glow")], 0);
    let out = rig.compose(&mut fx, &flat, None, n, 300, reduce());
    let changed: Vec<(i32, i32)> = (0..n).flat_map(|y| (0..n).map(move |x| (x, y))).filter(|&(x, y)| at(&out, x, y) != at(&flat, x, y)).collect();
    let (x0, x1) = (changed.iter().map(|p| p.0).min().unwrap(), changed.iter().map(|p| p.0).max().unwrap());
    let (y0, y1) = (changed.iter().map(|p| p.1).min().unwrap(), changed.iter().map(|p| p.1).max().unwrap());
    let (lo, h) = (3.0, 90.0);
    assert!(((x0 + x1 + 1) as f32 / 2.0 - (lo + 0.5 * h)).abs() <= 1.0 && ((y0 + y1 + 1) as f32 / 2.0 - (lo + 0.7 * h)).abs() <= 1.0, "centre ({x0}-{x1}, {y0}-{y1})");
    assert!(((x1 - x0 + 1) as f32 / 2.0 - 0.15 * n as f32).abs() <= 1.0, "radius 0.15 size");

    // Veins: nothing at their start, full length at 1800 ms, nothing beyond len from the root.
    let root = (48.0f32, 48.0f32);
    let face = Face { veins: vec![[48, 48, 0, 30]], ..Face::plain(n) };
    let reach = |img: &Image| -> f32 {
        let mut far = 0.0f32;
        for y in 0..n {
            for x in 0..n {
                if at(img, x, y) != GREY && !in_border(n, 3, x, y) {
                    far = far.max(((x as f32 + 0.5 - root.0).powi(2) + (y as f32 + 0.5 - root.1).powi(2)).sqrt());
                }
            }
        }
        far
    };
    let mut veins = PortraitFx::new("veins");
    veins.hold(&[cue("veins")], 0);
    assert_eq!(rig.compose(&mut veins, &flat, Some(&face), n, 0, PortraitOpts::default()), flat, "nothing at the start");
    let half = reach(&rig.compose(&mut veins, &flat, Some(&face), n, 900, PortraitOpts::default()));
    let full = reach(&rig.compose(&mut veins, &flat, Some(&face), n, VEIN_GROW_MS, PortraitOpts::default()));
    assert!((30.0 * 0.8..=30.0 + 2.0).contains(&full), "full length at 1800 ms, within len (+ the breath): {full}");
    assert!(half < full, "still growing half way: {half} < {full}");
    let mut steady = PortraitFx::new("veins");
    steady.hold(&[cue("veins")], 0);
    let r = reach(&rig.compose(&mut steady, &flat, Some(&face), n, 300, reduce()));
    assert!((24.0..=30.0).contains(&r), "reduce motion: full length at once, never past len: {r}");

    // Eye glow burns through a closed blink.
    let (img, eyes) = eye_face(true);
    let first = blink_starts(seed_of("glowing"), 20_000)[0];
    let mut lit = PortraitFx::new("glowing");
    lit.hold(&[cue("eye_glow")], 0);
    let shut = rig.compose(&mut PortraitFx::new("glowing"), &img, Some(&eyes), n, first + 75, reduce());
    let burning = rig.compose(&mut lit, &img, Some(&eyes), n, first + 75, reduce());
    assert_eq!(at(&shut, 42, 41), SKIN);
    assert_ne!(at(&burning, 42, 41), SKIN, "the eye glow is drawn after the lids");

    // Ash: min(32, round(18 i)) specks (halved at the reduced tier); none of ash or smoke under reduce motion.
    for (i, want) in [(1.0f32, 18usize), (0.3, 5), (2.0, 32)] {
        assert_eq!(ash_specks(1, 500, i, Tier::Full, n, 3, 93).len(), want, "i {i}");
    }
    assert_eq!(ash_specks(1, 500, 1.0, Tier::Reduced, 64, 2, 62).len(), 9);
    let mut ash = PortraitFx::new("ash");
    ash.hold(&[cue("ash")], 0);
    let out = rig.compose(&mut ash, &flat, None, n, 600, PortraitOpts::default());
    let specks = ash_specks(ash.seed, tq(600), 1.0, Tier::Full, n, 3, 93);
    assert_eq!(specks.len(), 18);
    assert!(specks.iter().all(|&(x, y)| at(&out, x, y) != GREY), "every speck shows");
    let mut drift = PortraitFx::new("ash");
    drift.hold(&[cue("ash"), cue("smoke")], 0);
    assert_eq!(rig.compose(&mut drift, &flat, None, n, 600, reduce()), flat, "ash and smoke: none under reduce motion");
}

#[test]
fn e8_states_follow_their_formulas_and_held_cues_ease() {
    let mut rig = Rig::new();
    let n = 40; // minimal tier: no base life
    let src = rows(n);
    let c = at(&src, 10, 12);
    let lum = 0.299 * c[0] as f32 + 0.587 * c[1] as f32 + 0.114 * c[2] as f32;
    let mut fx = PortraitFx::new("states");
    fx.hold(&[cue("desaturate")], 0);
    let out = rig.compose(&mut fx, &src, None, n, 300, PortraitOpts::default());
    let l = lum.round() as u8;
    assert_eq!(at(&out, 10, 12), [l, l, l, 255], "desaturate 1.0: the luminance");
    let mut tint = PortraitFx::new("states");
    tint.hold(&[cue("tint")], 0);
    let out = rig.compose(&mut tint, &src, None, n, 300, PortraitOpts::default());
    let col = [0x06, 0xb6, 0xd4];
    let want: Vec<u8> = (0..3).map(|k| (c[k] as f32 + ((lum / 255.0 * col[k] as f32 * 1.25).min(255.0) - c[k] as f32) * 0.6).round() as u8).collect();
    assert_eq!(&at(&out, 10, 12)[..3], &want[..], "tint toward #06b6d4");
    let mut fade = PortraitFx::new("states");
    fade.hold(&[cue("fade")], 0);
    let out = rig.compose(&mut fade, &src, None, n, 300, PortraitOpts::default());
    assert!(out.px.iter().skip(3).step_by(4).all(|a| *a == (255.0f32 * 0.3).round() as u8), "fade 1.0 leaves alpha 0.3, the border too");

    let mut eased = PortraitFx::new("ease");
    eased.hold(&[cue("desaturate").with_intensity(0.8)], 1000);
    assert!(eased.level("desaturate", 1000).is_none(), "0 at its hold");
    assert!((eased.level("desaturate", 1000 + EASE_MS / 2).unwrap().i - 0.4).abs() < 1e-4, "half way in");
    assert!((eased.level("desaturate", 1000 + EASE_MS).unwrap().i - 0.8).abs() < 1e-6, "full at 250 ms");
    eased.hold(&[], 2000);
    assert!((eased.level("desaturate", 2000).unwrap().i - 0.8).abs() < 1e-6, "it leaves from full");
    assert!(eased.level("desaturate", 2000 + EASE_MS).is_none(), "0 again 250 ms after it leaves the set");
}

#[test]
fn e9_flash_and_jolt_run_once_and_jolts_keep_their_limits() {
    let mut rig = Rig::new();
    let n = 40;
    let src = rows(n);
    let mut fx = PortraitFx::new("flash");
    fx.fire(cue("flash"), 1000);
    let out = rig.compose(&mut fx, &src, None, n, 1000, PortraitOpts::default());
    let c = at(&src, 10, 12);
    let a = 200.0f32 / 255.0;
    let want: Vec<u8> = (0..3).map(|k| (255.0 * a + c[k] as f32 * (1.0 - a)).round() as u8).collect();
    assert_eq!(&at(&out, 10, 12)[..3], &want[..], "white at alpha 200 i at its start");
    assert_eq!(at(&out, 0, 0), EDGE, "the border stays");
    assert_eq!(rig.compose(&mut fx, &src, None, n, 1000 + FLASH_MS as u64, PortraitOpts::default()), src, "gone at its ms");

    let mut hit = PortraitFx::new("jolt");
    rig.compose(&mut hit, &src, None, 96, 0, PortraitOpts::default());
    hit.fire(Cue::parse("jolt", "hit").unwrap(), 0);
    let offs: Vec<(i32, i32)> = (0..400).map(|t| hit.jolt_offset(t)).collect();
    assert!(offs.iter().all(|o| o.0.abs() <= 2 && o.1 == 0), "hit: at most 2 px, x only");
    assert!(offs.iter().any(|o| o.0 != 0) && offs[180..].iter().all(|o| *o == (0, 0)), "it moves, then rests after 180 ms");

    let mut crit = PortraitFx::new("jolt");
    crit.fire(Cue::parse("jolt", "crit").unwrap(), 0);
    let strong = crit.jolt_amplitude(10);
    assert!(!crit.fire_jolt(jolt_preset("hit").unwrap(), 10), "a weaker jolt does not replace a stronger running one");
    assert_eq!(crit.jolt_amplitude(10), strong);

    let mut big = PortraitFx::new("big");
    rig.compose(&mut big, &src, None, 96, 0, PortraitOpts::default());
    big.fire_jolt(Shake { px: 9.0, ms: 300, axis: Axis::Both, envelope: Envelope::Linear, step_ms: 30 }, 0);
    assert_eq!(big.jolt_amplitude(0), 4.0, "clamped to the 4 px ceiling");
    assert!((0..300).all(|t| big.jolt_offset(t).0.abs() <= 4 && big.jolt_offset(t).1.abs() <= 4));

    let mut calm = PortraitFx::new("calm");
    rig.compose(&mut calm, &src, None, 96, 0, reduce());
    calm.fire(Cue::parse("jolt", "crit").unwrap(), 0);
    assert!((0..320).all(|t| calm.jolt_offset(t) == (0, 0)), "no jolt under reduce motion");
}

#[test]
fn e10_the_strongest_cue_wins_and_faces_go_by_priority_then_start() {
    let mut fx = PortraitFx::new("mix");
    fx.hold(&[cue("desaturate").with_intensity(0.3).with_key("wounded"), cue("desaturate").with_intensity(0.5).with_key("low")], 0);
    assert!((fx.level("desaturate", 300).unwrap().i - 0.5).abs() < 1e-6, "0.3 and 0.5 give 0.5, never the sum");
    let mut same = PortraitFx::new("mix");
    same.hold(&[cue("desaturate").with_intensity(0.5), cue("desaturate").with_intensity(0.3)], 0);
    assert!((same.level("desaturate", 300).unwrap().i - 0.5).abs() < 1e-6, "two cues under one key both count");

    let mut faces = PortraitFx::new("faces");
    assert_eq!(faces.expression(0), "", "no face cue");
    faces.hold(&[cue("face").with_expression("hurt").with_priority(60).with_key("low_hp"), cue("face").with_expression("angry").with_priority(40).with_key("empowered")], 0);
    assert_eq!(faces.expression(10), "hurt", "the higher priority");
    faces.fire(cue("face").with_expression("afraid").with_priority(60).with_ms(900), 500);
    assert_eq!(faces.expression(600), "afraid", "equal priority: the later one");
    assert_eq!(faces.expression(1500), "hurt", "the fired face ends after its ms");
}

#[test]
fn e11_tiers_draw_exactly_their_effects() {
    assert_eq!((Tier::of(96), Tier::of(64), Tier::of(40)), (Tier::Full, Tier::Reduced, Tier::Minimal));
    assert_eq!((Tier::of(95), Tier::of(48), Tier::of(47)), (Tier::Reduced, Tier::Reduced, Tier::Minimal));
    let all: Vec<&str> = BASE_LIFE.iter().chain(EFFECTS.iter()).copied().collect();
    assert_eq!(Tier::Full.effects(), all);
    assert_eq!(Tier::Reduced.effects(), all.iter().copied().filter(|e| !["breath", "veins", "smoke"].contains(e)).collect::<Vec<_>>());
    assert_eq!(Tier::Minimal.effects(), ["flash", "desaturate", "tint", "fade", "eye_glow", "face"]);

    let mut rig = Rig::new();
    let face = Face { eyes: vec![[40, 40, 6, 4]], glow: vec![[48, 70, 10]], veins: vec![[50, 50, 90, 20]], rim: Some([217, 119, 6]), ..Face::plain(96) };
    for side in [96, 64, 40] {
        let src = rows(side);
        let mut fx = PortraitFx::new("tiers");
        fx.hold(&["glow", "veins", "eye_glow", "ash", "smoke", "desaturate", "tint", "fade"].map(cue), 0);
        fx.fire(cue("flash").with_ms(1000), 300);
        fx.fire(Cue::parse("jolt", "crit").unwrap(), 300);
        rig.compose(&mut fx, &src, Some(&face), side, 310, PortraitOpts::default());
        let tier = Tier::of(side);
        let want: Vec<&str> = tier.effects().into_iter().filter(|e| !["still", "face"].contains(e)).collect();
        assert_eq!(fx.live(310), want, "side {side}");
        assert_eq!(fx.state(310)["tier"], tier.name());
    }
}

#[test]
fn e12_composition_is_cached_stepped_capped_and_budgeted() {
    let mut rig = Rig::new();
    let src = rows(96);
    let mut quiet = PortraitFx::new("quiet");
    for f in 0..100 {
        rig.compose(&mut quiet, &src, None, 96, f * 16, reduce());
    }
    assert_eq!(quiet.composes, 1, "nothing time-varying (reduce motion, no eyes): composed once");
    let mut tiny = PortraitFx::new("tiny");
    let small = rows(40);
    for f in 0..100 {
        rig.compose(&mut tiny, &small, None, 40, f * 16, PortraitOpts::default());
    }
    assert_eq!(tiny.composes, 1, "the minimal tier with no reaction: composed once");

    let (img, face) = sample();
    let mut idle = PortraitFx::new("idle");
    for f in 0..144u64 {
        rig.compose(&mut idle, &img, Some(&face), 96, f * 1000 / 144, PortraitOpts::default());
    }
    assert!(idle.composes > 1 && idle.composes <= STEPS_PER_SECOND, "an idle portrait over 1000 ms at 144 frames: {}", idle.composes);

    // A running flash and an ease read the same steps: at most 30 compositions a second at 144 frames.
    let mut flashing = PortraitFx::new("flashing");
    flashing.fire(cue("flash").with_ms(1000), 0);
    let mut easing = PortraitFx::new("easing");
    easing.hold(&[cue("desaturate")], 0);
    for f in 0..144u64 {
        rig.compose(&mut flashing, &small, None, 40, f * 1000 / 144, PortraitOpts::default());
        rig.compose(&mut easing, &small, None, 40, f * 1000 / 144, PortraitOpts::default());
    }
    assert!(flashing.composes > 1 && flashing.composes <= STEPS_PER_SECOND, "a 1000 ms flash at 144 frames: {}", flashing.composes);
    assert!(easing.composes > 1 && easing.composes <= STEPS_PER_SECOND, "a 250 ms ease at 144 frames: {}", easing.composes);

    let mut big = PortraitFx::new("big");
    rig.draw(&mut big, &rows(96), None, Rect::new(0, 0, 400, 400), 0, &PortraitOpts::default(), &mut Budget::default());
    assert_eq!(big.composed().map(|i| i.w), Some(COMPOSE_MAX), "a 400 px rect composes at 256");

    let mut fxs: Vec<PortraitFx> = (0..9).map(|i| PortraitFx::new(&format!("p{i}"))).collect();
    let mut budget = Budget::default();
    for now in [0u64, 500] {
        budget.begin();
        for (i, fx) in fxs.iter_mut().enumerate() {
            rig.draw(fx, &img, Some(&face), Rect::new(i as i32 * 10, 0, 96, 96), now, &PortraitOpts::default(), &mut budget);
        }
    }
    assert_eq!(fxs.iter().map(|f| f.composes).collect::<Vec<_>>(), [2, 2, 2, 2, 2, 2, 2, 2, 1], "the 9th animated portrait of a frame keeps its last image");
    assert_eq!(MAX_ANIMATED, 8);
}

#[test]
fn e13_the_border_moves_only_with_the_jolt_flash_covers_and_fade_multiplies() {
    let mut rig = Rig::new();
    let (img, face) = sample();
    let n = 96;
    let all = ["glow", "veins", "eye_glow", "ash", "smoke", "desaturate", "tint"].map(cue);
    let first = blink_starts(seed_of("order"), 20_000)[0];
    let mut fx = PortraitFx::new("order");
    fx.hold(&all, 0);
    fx.fire(cue("flash").with_ms(5000), first);
    let out = rig.compose(&mut fx, &img, Some(&face), n, first + 75, PortraitOpts::default());
    for y in 0..n {
        for x in 0..n {
            if in_border(n, face.border, x, y) {
                assert_eq!(at(&out, x, y), at(&img, x, y), "no effect but the jolt touches the border ({x}, {y})");
            }
        }
    }
    // The glow anchor's centre with and without a flash over it.
    let (gx, gy) = (face.glow[0][0], face.glow[0][1]);
    let mut glow = PortraitFx::new("order2");
    glow.hold(&[cue("glow")], 0);
    let plain = rig.compose(&mut glow, &img, Some(&face), n, 300, reduce());
    let mut flashed = PortraitFx::new("order2");
    flashed.hold(&[cue("glow")], 0);
    flashed.fire(cue("flash").with_ms(1000), 300);
    let lit = rig.compose(&mut flashed, &img, Some(&face), n, 300, reduce());
    assert_ne!(at(&plain, gx, gy), at(&img, gx, gy), "the glow shows");
    let a = 200.0f32 / 255.0;
    let p = at(&plain, gx, gy);
    let want: Vec<u8> = (0..3).map(|k| (255.0 * a + p[k] as f32 * (1.0 - a)).round() as u8).collect();
    assert_eq!(&at(&lit, gx, gy)[..3], &want[..], "the flash covers the overlay pixel");
    let mut faded = PortraitFx::new("order2");
    faded.hold(&[cue("glow"), cue("fade")], 0);
    let dim = rig.compose(&mut faded, &img, Some(&face), n, 300, reduce());
    assert_eq!(at(&dim, gx, gy), [p[0], p[1], p[2], (p[3] as f32 * 0.3).round() as u8], "fade multiplies the overlay pixel's alpha");
}

#[test]
fn e14_cue_names_parse_and_the_lists_hold_the_spec_names() {
    assert_eq!(EFFECTS, ["flash", "jolt", "desaturate", "tint", "fade", "still", "glow", "veins", "eye_glow", "ash", "smoke", "face"]);
    assert_eq!(PORTRAIT_JOLTS, ["hit", "crit"]);
    assert!(Cue::parse("sparkle", "").is_none(), "an unknown effect");
    assert!(Cue::parse("jolt", "boom").is_none() && Cue::parse("jolt", "").is_none(), "an unknown or missing jolt preset");
    let c = Cue::parse("jolt", "crit").unwrap();
    assert_eq!((c.effect, c.preset, c.intensity, c.priority), ("jolt", "crit", 1.0, 50));
    assert_eq!(Cue::parse("tint", "crit").unwrap().preset, "", "only a jolt keeps a preset");
    let guard = c.with_intensity(0.5).shake().unwrap();
    assert_eq!((guard.px, guard.ms), (1.5, 320), "a jolt cue's intensity scales its preset");
}

#[test]
fn a_sidecar_idle_list_holds_overlays_only() {
    let idle = |e: &str| aegis_core::assets::IdleOverlay { effect: e.into(), color: None, intensity: 1.0 };
    for e in ["still", "fade", "desaturate", "flash", "jolt", "face", "sparkle"] {
        assert!(Cue::idle(&idle(e)).is_none(), "{e} is not an idle overlay");
    }
    for e in aegis_core::assets::IDLE_OVERLAYS {
        assert_eq!(Cue::idle(&idle(e)).map(|c| c.key), Some(format!("idle.{e}")));
    }
    // A sidecar that names `still` (not refused by drawing, only by the asset check) never freezes the portrait.
    let face = Face { idle: vec![idle("still"), idle("fade"), idle("glow")], ..Face::plain(96) };
    assert_eq!(idle_cues(&face).iter().map(|c| c.effect).collect::<Vec<_>>(), ["glow"]);
    let mut fx = PortraitFx::new("idle_still");
    fx.hold(&idle_cues(&face), 0);
    let mut rig = Rig::new();
    rig.compose(&mut fx, &rows(96), Some(&face), 96, 300, PortraitOpts::default());
    assert_eq!(fx.live(300), ["breath", "glow"], "breath runs, no still, no fade");
}

#[test]
fn periodic_motion_stays_smooth_after_hours_of_clock() {
    let hours = 10 * 3600 * 1000u64;
    let t = hours / 1700 * 1700 + 425; // a quarter period of the rim's sine
    assert!((cycle(t, 1700.0) - std::f32::consts::FRAC_PI_2).abs() < 1e-5, "{}", cycle(t, 1700.0));
    assert_eq!(cycle(hours / 1800 * 1800, 1800.0), 0.0);
    let seed = seed_of("long");
    let p = breath_period(seed) as f64;
    let start = ((hours as f64 / p).ceil() * p).ceil() as u64;
    let early: Vec<i32> = (0..120).map(|k| breath_px(seed, k * 33, 240)).collect();
    let late: Vec<i32> = (0..120).map(|k| breath_px(seed, start + k * 33, 240)).collect();
    let steps = |v: &[i32]| v.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(late.iter().all(|a| (0..=breath_amp(240)).contains(a)) && steps(&late) > 0 && steps(&late) <= steps(&early) + 2, "{early:?} vs {late:?}");
    let rims: Vec<f32> = (0..60).map(|k| rim_factor(seed, hours + k * 33)).collect();
    assert!(rims.iter().all(|f| (0.88 - 1e-4..=1.12 + 1e-4).contains(f)));
}

/// The Kit widgets page drawn as the gallery does: widgets, pixels, the texts drawn and the text audit.
fn draw_kit(k: &Specimens) -> (Vec<Widget<KitAct>>, Canvas, Vec<TextRec>, serde_json::Value) {
    let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.canvas.clear(BG);
    g.fonts.audit = Some(Vec::new());
    let w = {
        let mut ui: Ui<KitAct> = Ui::new(&mut g, (-1, -1), true);
        k.draw(&mut ui, Rect::new(112, 22, 522, 332), |a| a);
        ui.widgets
    };
    let recs = g.fonts.audit.take().unwrap_or_default();
    let boxes: Vec<_> = w.iter().map(|x| (x.id.clone(), x.rect.x as f32, x.rect.y as f32, x.rect.w as f32, x.rect.h as f32)).collect();
    let issues = text_audit::issues(&recs, 640.0, 360.0, &boxes);
    (w, g.canvas, recs, issues)
}

#[test]
fn e15_the_kit_portraits_page_has_its_controls_jolts_inside_its_stage_and_names_the_tier() {
    let mut k = Specimens::default();
    let (w, _, _, _) = draw_kit(&k);
    k.act(by_id(&w, "kit.page.3").unwrap());
    assert_eq!(k.last, "page Portraits");
    let (w, _, recs, audit) = draw_kit(&k);
    assert_eq!(audit["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "the page reads clean: {audit}");
    for name in PORTRAIT_OVERLAYS.iter().chain(PORTRAIT_STATES.iter()).chain(PORTRAIT_REACTIONS.iter()) {
        assert!(by_id(&w, &format!("kit.portrait.{name}")).is_ok(), "control for {name}");
    }
    assert!(w.iter().any(|x| x.id == "kit.portrait.size") && by_id(&w, "kit.portrait.reduce").is_ok() && by_id(&w, "kit.portrait.anchors").is_ok());
    let stage = w.iter().find(|x| x.id == "kit.portrait.face").unwrap().rect;
    assert!(recs.iter().any(|t| t.text.starts_with("Tier full")), "96 px at scale 1: full");
    let live = k.portrait.state();
    assert_eq!((live["tier"].as_str(), live["effects"][0].as_str()), (Some("full"), Some("breath")), "{live}");

    // A crit jolt moves portrait pixels, none outside the stage (the status text under it names the jolt).
    k.act(by_id(&w, "kit.portrait.jolt.crit").unwrap());
    k.advance(20.0);
    let (_, shaken, _, _) = draw_kit(&k);
    let still = k.clone();
    still.portrait.fx.borrow_mut().stop_jolt();
    let (_, calm, _, _) = draw_kit(&still);
    let status = Rect::new(stage.x, stage.y + stage.h, 200, 360);
    let (mut inside, mut outside) = (0, 0);
    for y in 0..360 {
        for x in 0..640 {
            if canvas_px(&shaken, x, y) != canvas_px(&calm, x, y) {
                if stage.contains(x, y) {
                    inside += 1;
                } else if !status.contains(x, y) {
                    outside += 1;
                }
            }
        }
    }
    assert!(inside > 0, "the portrait moves");
    assert_eq!(outside, 0, "nothing outside its stage");
    assert!(k.portrait.state()["effects"].as_array().unwrap().iter().any(|e| e == "jolt"), "state lists the running jolt");

    // Toggles hold their effect; size 16 is the minimal tier.
    k.act(KitAct::PortraitToggle("glow"));
    k.advance(300.0);
    draw_kit(&k);
    assert!(k.portrait.state()["effects"].as_array().unwrap().iter().any(|e| e == "glow"), "state lists the held glow");
    k.act(KitAct::PortraitSize(PORTRAIT_SIZES.iter().position(|s| *s == 16).unwrap()));
    let (_, _, recs, audit) = draw_kit(&k);
    assert!(recs.iter().any(|t| t.text.starts_with("Tier minimal")), "size 16 names tier minimal");
    assert_eq!(audit["counts"]["cut"], 0);
    assert_eq!(k.portrait.state()["tier"], "minimal");
}
