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

// ---- PORTRAITFX1-EX: expression effects ---------------------------------------------------------------------------

const RED: Rgba = [225, 29, 72, 255];
const GREEN: Rgba = [0, 255, 0, 255];

/// A 96 px sample with a face box (no eyes, so no blink) over a plain skin image.
fn ex_face() -> (Image, Face) {
    (framed(96, |_, _| SKIN), Face { face: Some([24, 20, 48, 56]), ..Face::plain(96) })
}

fn face_cue(e: &str) -> Cue {
    cue("face").with_expression(e).with_key("line.face")
}

fn angry() -> ExpressionFx {
    ExpressionFx { tint: Some(RED), tint_strength: 0.5, ..ExpressionFx::new("angry") }
}

/// EX1: a held expression tints its face box: the box centre by `tinted` at the row's strength (eased in over
/// EASE_MS), pixels outside the box (and its soft ellipse's corners) untouched; a timed tint (`tint_ms`) falls from full
/// to 0; no face box, no sidecar or no row for the expression: no tint and no crash.
#[test]
fn ex1_the_expression_tint_covers_the_face_box() {
    let mut rig = Rig::new();
    let (src, face) = ex_face();
    let mut fx = PortraitFx::new("ex1");
    fx.set_expressions(&[angry()]);
    fx.hold(&[face_cue("angry")], 0);
    let out = rig.compose(&mut fx, &src, Some(&face), 96, 1000, reduce());
    assert_eq!(at(&out, 48, 48), tinted(SKIN, RED, 0.5), "the centre at full strength");
    assert_eq!(at(&out, 10, 10), SKIN, "outside the box");
    assert_eq!(at(&out, 24, 20), SKIN, "the box corner is outside the soft ellipse");
    assert_eq!(face_weight(0.0, 0.0), 1.0);
    assert_eq!(fx.state(1000)["expression_fx"], json!(["tint"]));
    let early = rig.compose(&mut fx, &src, Some(&face), 96, 100, reduce());
    assert_eq!(at(&early, 48, 48), tinted(SKIN, RED, angry().tint_at(tq(100))), "easing in over EASE_MS");

    // a timed tint: the hurt flash over the face
    let hurt = ExpressionFx { tint: Some(RED), tint_strength: 0.8, tint_ms: 300, ..ExpressionFx::new("hurt") };
    let mut hit = PortraitFx::new("ex1_hurt");
    hit.set_expressions(&[hurt.clone()]);
    hit.fire(face_cue("hurt").with_ms(900), 0);
    let out = rig.compose(&mut hit, &src, Some(&face), 96, 100, reduce());
    assert_eq!(at(&out, 48, 48), tinted(SKIN, RED, hurt.tint_at(tq(100))), "fading from its start");
    assert!(hurt.tint_at(0) == 0.8 && hurt.tint_at(300) == 0.0);
    assert_eq!(rig.compose(&mut hit, &src, Some(&face), 96, 400, reduce()), src, "gone after tint_ms (the face still holds)");
    assert_eq!(hit.expression(400), "hurt");

    // the face box from the eye boxes when the sidecar names none
    let eyes = Face { eyes: vec![[38, 38, 5, 3], [53, 38, 5, 3]], blink: false, ..Face::plain(96) };
    let fb = eyes.face_box().expect("a box from the eyes");
    let mut derived = PortraitFx::new("ex1_eyes");
    derived.set_expressions(&[angry()]);
    derived.hold(&[face_cue("angry")], 0);
    let out = rig.compose(&mut derived, &src, Some(&eyes), 96, 1000, reduce());
    let (cx, cy) = (fb[0] + fb[2] / 2, fb[1] + fb[3] / 2);
    assert_eq!(at(&out, cx, cy), tinted(SKIN, RED, 0.5), "the derived box's centre");

    // no face box, no sidecar, another expression: nothing
    for (name, f, e) in [("plain", Some(Face::plain(96)), "angry"), ("none", None, "angry"), ("other", Some(face.clone()), "afraid")] {
        let mut fx = PortraitFx::new(name);
        fx.set_expressions(&[angry()]);
        fx.hold(&[face_cue(e)], 0);
        assert_eq!(rig.compose(&mut fx, &src, f.as_ref(), 96, 1000, reduce()), src, "{name}: untouched");
        assert_eq!(fx.state(1000)["expression_fx"], json!([]), "{name}");
    }
}

/// The pixels of `out` that differ from `base` and their bounding box (x0, y0, w, h).
fn changed(out: &Image, base: &Image) -> Option<(i32, i32, i32, i32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for y in 0..out.h {
        for x in 0..out.w {
            if at(out, x, y) != at(base, x, y) {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
    }
    (x0 <= x1).then(|| (x0, y0, x1 - x0 + 1, y1 - y0 + 1))
}

/// EX2: a mark is drawn centred on its anchor in the face box (or the eye box), its bitmap scaled by nearest to its
/// width (`mark_width`); the vein's `X` cells take the row's colour and nothing outside its rect changes; the minimal
/// tier draws no mark (the tint stays); a pulse grows the mark, reduce motion holds it steady.
#[test]
fn ex2_the_mark_is_drawn_at_its_anchor() {
    let mut rig = Rig::new();
    let (src, face) = ex_face();
    let vein = ExpressionFx { mark: Some(Mark::Vein), mark_color: Some(GREEN), mark_x: 0.85, mark_y: 0.15, mark_size: 0.12, ..ExpressionFx::new("angry") };
    let mut fx = PortraitFx::new("ex2");
    fx.set_expressions(&[vein.clone()]);
    fx.hold(&[face_cue("angry")], 0);
    let out = rig.compose(&mut fx, &src, Some(&face), 96, 1000, reduce());
    let (ax, ay) = vein.anchor(&face, 1.0).unwrap();
    assert_eq!((ax, ay), (24.0 + 0.85 * 48.0, 20.0 + 0.15 * 56.0), "the anchor in units of the face box");
    let w = mark_width(Mark::Vein, 0.12, 96, 1.0);
    let rect = mark_rect(Mark::Vein, w, ax, ay);
    assert_eq!(changed(&out, &src), Some(rect), "the mark covers its rect and nothing else changes");
    let bm = Mark::Vein.bitmap();
    let (bw, bh) = (bm[0].len() as i32, bm.len() as i32);
    for dy in 0..rect.3 {
        for dx in 0..rect.2 {
            let cell = bm[(dy * bh / rect.3) as usize].as_bytes()[(dx * bw / rect.2) as usize];
            let px = at(&out, rect.0 + dx, rect.1 + dy);
            assert_eq!(px == GREEN, cell == b'X', "cell ({dx}, {dy})");
        }
    }
    assert_eq!(fx.state(1000)["expression_fx"], json!(["mark"]));

    // the eye box anchor
    let eyes = Face { eyes: vec![[38, 38, 5, 3], [53, 38, 5, 3]], blink: false, ..Face::plain(96) };
    let sweat = ExpressionFx { mark: Some(Mark::Sweat), mark_box: MarkBox::Eyes, mark_x: 1.2, mark_y: -1.0, mark_size: 0.08, ..ExpressionFx::new("afraid") };
    let mut fx = PortraitFx::new("ex2_eyes");
    fx.set_expressions(&[sweat.clone()]);
    fx.hold(&[face_cue("afraid")], 0);
    let out = rig.compose(&mut fx, &src, Some(&eyes), 96, 1000, reduce());
    let (ax, ay) = sweat.anchor(&eyes, 1.0).unwrap();
    let eb = eyes.eyes_box().unwrap();
    assert_eq!((ax, ay), (eb[0] as f32 + 1.2 * eb[2] as f32, eb[1] as f32 - eb[3] as f32));
    let drop = mark_rect(Mark::Sweat, mark_width(Mark::Sweat, 0.08, 96, 1.0), ax, ay);
    assert_eq!(changed(&out, &src), Some(drop));
    assert_eq!(at(&out, drop.0 + drop.2 / 2, drop.1), Mark::cell('O', Mark::Sweat.default_color()).unwrap(), "the drop's tip: the outline in the default colour");

    // the minimal tier: the tint, no mark
    let both = ExpressionFx { tint: Some(RED), tint_strength: 0.5, ..vein.clone() };
    let mut small = PortraitFx::new("ex2_small");
    small.set_expressions(&[both]);
    small.hold(&[face_cue("angry")], 0);
    let out = rig.compose(&mut small, &src, Some(&face), 32, 1000, reduce());
    assert_eq!(small.tier(), Some(Tier::Minimal));
    assert_eq!(small.state(1000)["expression_fx"], json!(["tint"]), "no mark at the minimal tier");
    assert!((0..32).all(|y| (0..32).all(|x| at(&out, x, y) != GREEN)));

    // a pulse (reduced tier: no breath) grows the mark; reduce motion keeps it steady
    let pulsing = ExpressionFx { mark_pulse_ms: 900, mark_size: 0.2, ..vein.clone() };
    let base = rig.compose(&mut PortraitFx::new("ex2_base"), &src, Some(&face), 64, 0, PortraitOpts::default());
    let widths = |opts: PortraitOpts, rig: &mut Rig| -> Vec<i32> {
        let mut fx = PortraitFx::new("ex2_pulse");
        fx.set_expressions(&[pulsing.clone()]);
        fx.hold(&[face_cue("angry")], 0);
        [225u64, 675].iter().map(|&t| changed(&rig.compose(&mut fx, &src, Some(&face), 64, t, opts), &base).map_or(0, |r| r.2)).collect()
    };
    let want: Vec<i32> = [225u64, 675].iter().map(|&t| mark_width(Mark::Vein, 0.2, 64, mark_pulse(pulsing.pulse(tq(t))).0)).collect();
    assert_ne!(want[0], want[1], "the pulse's high and low differ");
    assert_eq!(widths(PortraitOpts::default(), &mut rig), want, "the pulse grows the mark");
    let steady = mark_width(Mark::Vein, 0.2, 64, 1.0);
    assert_eq!(widths(reduce(), &mut rig), [steady, steady], "reduce motion: no pulse");
}

/// EX3: an expression's shake moves the whole portrait from the expression's start as its `Shake` (axis x, ease-out)
/// says; halved at the reduced tier (at least 1 px), none at the minimal tier or under reduce motion, none after its
/// ms; the drawn offset is the jolt plus the shake within the jolt ceiling.
#[test]
fn ex3_the_expression_shake_follows_its_row() {
    let mut rig = Rig::new();
    let (src, face) = ex_face();
    let hurt = ExpressionFx { shake_px: 2.0, shake_ms: 200, ..ExpressionFx::new("hurt") };
    let shake = hurt.shake().unwrap();
    assert_eq!(shake, Shake { px: 2.0, ms: 200, axis: Axis::X, envelope: Envelope::EaseOut, step_ms: SHAKE_STEP_MS });
    let run = |side: i32, opts: PortraitOpts, rig: &mut Rig| -> (PortraitFx, Vec<(i32, i32)>) {
        let mut fx = PortraitFx::new("ex3");
        fx.set_expressions(&[hurt.clone()]);
        fx.fire(face_cue("hurt").with_ms(900), 1000);
        rig.compose(&mut fx, &src, Some(&face), side, 1000, opts);
        let offs = (1000..1400).map(|t| fx.expression_shake(t)).collect();
        (fx, offs)
    };
    let (fx, full) = run(96, PortraitOpts::default(), &mut rig);
    let seed = fx.seed ^ 1000;
    assert_eq!(full, (0..400u64).map(|t| shake.offset(t, seed)).collect::<Vec<_>>(), "the row's shake from the start");
    assert!(full.iter().any(|o| o.0 != 0) && full[200..].iter().all(|o| *o == (0, 0)), "it moves, then rests after shake_ms");
    assert_eq!(fx.state(1010)["shake"], json!([full[10].0, full[10].1]));
    let half = Shake { px: 1.0, ..shake };
    assert_eq!(run(64, PortraitOpts::default(), &mut rig).1, (0..400u64).map(|t| half.offset(t, seed)).collect::<Vec<_>>(), "halved at the reduced tier");
    assert!(run(32, PortraitOpts::default(), &mut rig).1.iter().all(|o| *o == (0, 0)), "none at the minimal tier");
    assert!(run(96, reduce(), &mut rig).1.iter().all(|o| *o == (0, 0)), "none under reduce motion");

    // the jolt and the shake add up within the ceiling
    let (mut fx, _) = run(96, PortraitOpts::default(), &mut rig);
    fx.fire(Cue::parse("jolt", "crit").unwrap(), 1000);
    let m = JOLT_LIMITS.max_px as i32;
    for t in 1000..1400 {
        let ((jx, jy), (sx, sy)) = (fx.jolt_offset(t), fx.expression_shake(t));
        assert_eq!(fx.offset(t), ((jx + sx).clamp(-m, m), (jy + sy).clamp(-m, m)));
    }
}

/// EX4: a held face that changes its expression starts again (its effects run from the change), so a line that turns
/// hurt flashes and shakes then; the mark and box names parse.
#[test]
fn ex4_a_held_face_that_changes_starts_again() {
    let mut fx = PortraitFx::new("ex4");
    fx.hold(&[face_cue("angry")], 0);
    fx.hold(&[face_cue("angry")], 300);
    assert_eq!(fx.expression_since(400), Some(("angry", 0)), "the same face keeps its start");
    fx.hold(&[face_cue("hurt")], 500);
    assert_eq!(fx.expression_since(600), Some(("hurt", 500)));
    fx.set_expressions(&[ExpressionFx::new("hurt")]);
    assert_eq!(fx.expression_fx(600).map(|(r, s)| (r.expression.as_str(), s)), Some(("hurt", 500)));
    assert_eq!(MARKS.iter().map(|m| Mark::parse(m).unwrap().name()).collect::<Vec<_>>(), MARKS);
    assert_eq!(Mark::parse(""), None);
    assert_eq!((MarkBox::parse("eyes"), MarkBox::parse("face")), (MarkBox::Eyes, MarkBox::Face));
}

/// PORTRAITFX1-TL (4.6, 6.2 row 4): a tier cap keeps a portrait no richer than it (a timeline row's icon stays minimal
/// however large the window); without one the side decides, as before.
#[test]
fn tl_a_tier_cap_keeps_a_portrait_no_richer_than_it() {
    let sides = [FULL_SIDE + 40, FULL_SIDE, REDUCED_SIDE, REDUCED_SIDE - 1, 8];
    let plain = PortraitOpts::default();
    assert!(sides.iter().all(|&s| plain.tier(s) == Tier::of(s)), "no cap: the side decides");
    let minimal = PortraitOpts { tier_cap: Some(Tier::Minimal), ..PortraitOpts::default() };
    assert!(sides.iter().all(|&s| minimal.tier(s) == Tier::Minimal), "capped at minimal: minimal at every side");
    let reduced = PortraitOpts { tier_cap: Some(Tier::Reduced), ..PortraitOpts::default() };
    assert_eq!(sides.map(|s| reduced.tier(s)), [Tier::Reduced, Tier::Reduced, Tier::Reduced, Tier::Minimal, Tier::Minimal], "capped at reduced: never full");
}

// ---- PORTRAITFX2-M: motion per face (PORTRAITFX2.md 4 and 8, MO1-MO11) ---------------------------------------------

/// The motion context of seed `seed` at `t` (face start `since`) on a composed side `side` (its tier), motion seed of
/// the face `m`.
fn mctx(seed: u64, t: u64, since: u64, side: i32) -> MotionCtx {
    MotionCtx { seed, mseed: motion_seed(seed, "m"), t, since, side, tier: Tier::of(side), reduce: false, still: false, tinted: false }
}

/// A motion with no cycle (only the parts set on it).
fn steady() -> Motion {
    Motion { breath_amp: 0.0, ..Motion::default() }
}

/// (stretch, dx, dy) of an output.
fn sxy(o: &MotionOut) -> (i32, i32, i32) {
    (o.stretch, o.dx, o.dy)
}

/// The agent state's motion (stretch, dx, dy) and parts.
fn state_motion(fx: &PortraitFx, now: u64) -> ((i64, i64, i64), Vec<String>) {
    let s = fx.state(now);
    let m = &s["motion"];
    let parts = m["parts"].as_array().unwrap().iter().map(|p| p.as_str().unwrap().to_string()).collect();
    ((m["stretch"].as_i64().unwrap(), m["offset"][0].as_i64().unwrap(), m["offset"][1].as_i64().unwrap()), parts)
}

/// A face row of `expression` with motion `m` and nothing else.
fn moving(expression: &str, m: Motion) -> ExpressionFx {
    ExpressionFx { motion: m, ..ExpressionFx::new(expression) }
}

/// MO1: no profile and `Motion::default()` give `breath_px` exactly at every `t` and full-tier side (the face's start
/// plays no part), none below the full tier; a face whose row keeps the default motion, and a face with no row, compose
/// exactly as the plain portrait.
#[test]
fn mo1_the_default_motion_is_the_breath_exactly() {
    let d = Motion::default();
    assert_eq!((d.breath_amp, d.breath_ms, d.breath_in, d.breath_jitter, d.blink), (0.012 * MOTION_REF as f32, 3600, 0.5, 0.0, true));
    for key in ["mo1", "breath", "kit.sample", "a"] {
        let seed = seed_of(key);
        for s in FULL_SIDE..=COMPOSE_MAX {
            assert_eq!(motion_px(d.breath_amp, s), breath_amp(s), "px(2.88) is the breath amplitude at side {s}");
            for t in (0..12_000u64).step_by(37) {
                let o = d.eval(&mctx(seed, t, 0, s));
                assert_eq!(sxy(&o), (breath_px(seed, t, s), 0, 0), "{key} side {s} t {t}");
            }
        }
        assert_eq!(d.eval(&mctx(seed, 5000, 1234, 240)), d.eval(&mctx(seed, 5000, 0, 240)), "the start plays no part");
        for s in [16, REDUCED_SIDE - 1, REDUCED_SIDE, FULL_SIDE - 1] {
            assert_eq!(sxy(&d.eval(&mctx(seed, 1800, 0, s))), (0, 0, 0), "no breath below the full tier ({s})");
        }
    }
    let mut rig = Rig::new();
    let src = rows(240);
    for t in [0, 900, 1800, 2700, 4000] {
        let plain = rig.compose(&mut PortraitFx::new("mo1"), &src, None, 240, t, PortraitOpts::default());
        let mut row = PortraitFx::new("mo1");
        row.set_expressions(&[ExpressionFx::new("hurt")]);
        row.hold(&[face_cue("hurt")], 0);
        assert_eq!(rig.compose(&mut row, &src, None, 240, t, PortraitOpts::default()), plain, "a row at the default motion (t {t})");
        let mut none = PortraitFx::new("mo1");
        none.hold(&[face_cue("smug")], 0);
        assert_eq!(rig.compose(&mut none, &src, None, 240, t, PortraitOpts::default()), plain, "a face without a row takes the base motion (t {t})");
    }
}

/// MO2: `breath_in` r puts the peak (b = 1) at r P' with the rise and fall of `cycle_phase`; with no jitter the cycle runs
/// on the clock (the face's start and the motion seed play no part); with jitter j each cycle n (from the face's start)
/// lasts P' (1 + j (2 hm(4n) - 1)) and peaks at A (1 - j hm(4n + 1)), within [A (1 - j), A], the same for the same key.
#[test]
fn mo2_the_cycle_breathes_in_over_its_share_and_jitter_varies_each_cycle() {
    let seed = seed_of("mo2");
    for r in [0.3f32, 0.4, 0.5, 0.7] {
        let m = Motion { breath_in: r, ..Motion::default() };
        let p = m.period(seed);
        assert_eq!(p, 3600.0 * (0.9 + 0.2 * hash01(seed, 1)), "P' as the breath");
        let peak = (r * p).round() as u64;
        let (b, a) = m.cycle_at(seed, 0, peak, 0);
        assert!(b > 0.999 && a == m.breath_amp, "r {r}: the peak at r P' ({b})");
        assert_eq!(m.cycle_at(seed, 0, 0, 0).0, 0.0, "r {r}: the cycle starts at 0");
        for t in (0..(2.0 * p) as u64).step_by(53) {
            let f = ((t as f64 % p as f64) / p as f64) as f32;
            assert!((m.cycle_at(seed, 0, t, 0).0 - cycle_phase(f, r)).abs() < 1e-4, "r {r} t {t}");
        }
        for since in [0, 777, 5000] {
            assert_eq!(m.cycle_at(seed, 99, 6000, since), m.cycle_at(seed, 1, 6000, 0), "j = 0: the clock's closed form");
        }
    }
    let m = Motion { breath_amp: 4.0, breath_in: 0.4, breath_jitter: 0.35, ..Motion::default() };
    let mseed = motion_seed(seed, "hurt");
    let (p, j) = (m.period(seed) as f64, m.breath_jitter);
    let since = 2500u64;
    let mut start = 0f64;
    for n in 0..12u64 {
        let len = p * (1.0 + j as f64 * (2.0 * hash01(mseed, 4 * n) as f64 - 1.0));
        let want = m.breath_amp * (1.0 - j * hash01(mseed, 4 * n + 1));
        assert!(want >= m.breath_amp * (1.0 - j) && want <= m.breath_amp, "cycle {n} amplitude in range");
        let peak = since + (start + 0.4 * len).round() as u64;
        let (b, a) = m.cycle_at(seed, mseed, peak, since);
        assert!(b > 0.999, "cycle {n} peaks at r of its own length ({b})");
        assert_eq!(a, want, "cycle {n}'s amplitude");
        let (b0, _) = m.cycle_at(seed, mseed, since + start.ceil() as u64, since);
        assert!(b0 < 0.01, "cycle {n} starts low ({b0})");
        start += len;
    }
    let c = MotionCtx { mseed, ..mctx(seed, 9000, since, 240) };
    assert_eq!(m.eval(&c), m.eval(&c), "the same key and time, the same output");
    assert_eq!(m.cycle_at(seed, mseed, 9000 + 400, since + 400), m.cycle_at(seed, mseed, 9000, since), "counted from the face's start");
}

/// MO3: `enter_stretch` -2 at side 240 reaches -2 at 0.25 E and 0 at E; the cycle is 0 before E, then grows to full
/// scale at E + `settle_ms` (half way: half the stretch).
#[test]
fn mo3_the_entry_runs_first_and_the_cycle_settles_after_it() {
    let seed = seed_of("mo3");
    let m = Motion { breath_amp: 4.0, enter_stretch: -2, enter_ms: 400, settle_ms: 800, ..Motion::default() };
    let since = 1000;
    let at = |u: u64| m.eval(&mctx(seed, since + u, since, 240));
    assert_eq!(at(100).stretch, -2, "the full squeeze at 0.25 E");
    assert_eq!(at(400).stretch, 0, "nothing at E (the cycle not yet started)");
    for u in 0..400 {
        assert_eq!(at(u).stretch, (motion_px(-2.0, 240) as f32 * entry_env(u, 400)).round() as i32, "the cycle is 0 during the entry (u {u})");
    }
    assert!(at(100).parts.contains(&"entry") && !at(500).parts.contains(&"entry"));
    assert!(at(800).parts.contains(&"settle") && !at(1300).parts.contains(&"settle"));
    let plain = Motion { breath_amp: 4.0, ..Motion::default() };
    for u in [1200u64, 1500, 2000, 3100] {
        assert_eq!(at(u).stretch, plain.eval(&mctx(seed, since + u, since, 240)).stretch, "full scale from E + settle (u {u})");
    }
    let (b, _) = m.cycle_at(seed, 0, since + 800, since);
    assert_eq!(at(800).stretch, (motion_px(4.0, 240) as f32 * b * 0.5).round() as i32, "half way through the settle");
}

/// MO4: lean, bob, rest and shiver follow 4.1 (shiver 0 between bursts, within px(X) in one); the totals clamp to
/// [-px(3), px(4)] for the stretch and px(3) for each offset axis.
#[test]
fn mo4_lean_bob_rest_and_shiver_follow_their_rules_and_the_totals_clamp() {
    let seed = seed_of("mo4");
    let lean = Motion { lean_px: 2, ..Motion::default() };
    let p = lean.period(seed);
    for t in (0..8000u64).step_by(41) {
        let (b, _) = lean.cycle_at(seed, 0, t, 0);
        assert_eq!(lean.eval(&mctx(seed, t, 0, 240)).dx, (2.0 * b).round() as i32, "lean with the cycle (t {t})");
    }
    assert_eq!(lean.eval(&mctx(seed, (p / 2.0).round() as u64, 0, 240)).dx, 2, "the full lean at the peak");

    let bob = Motion { bob_px: 2, bob_ms: 1000, ..steady() };
    for t in (0..3000u64).step_by(29) {
        let want = -(2.0 * (std::f64::consts::PI * t as f64 / 1000.0).sin().abs() as f32).round() as i32;
        assert_eq!(bob.eval(&mctx(seed, t, 0, 240)).dy, want, "bob (t {t})");
    }
    assert_eq!((bob.eval(&mctx(seed, 500, 0, 240)).dy, bob.eval(&mctx(seed, 1000, 0, 240)).dy), (-2, 0));

    let rest = Motion { rest_dy: 2, rest_ms: 1000, ..steady() };
    for u in (0..2000u64).step_by(50) {
        let f = (u as f32 / 1000.0).min(1.0);
        assert_eq!(rest.eval(&mctx(seed, 300 + u, 300, 240)).dy, (2.0 * (1.0 - (1.0 - f).powi(2))).round() as i32, "rest (u {u})");
    }
    assert_eq!(Motion { rest_ms: 0, ..rest }.eval(&mctx(seed, 300, 300, 240)).dy, 2, "rest_ms 0: at once");

    let sh = Motion { shiver_px: 1, shiver_ms: 280, shiver_gap_ms: 1400, ..steady() };
    let ms = motion_seed(seed, "afraid");
    let c = |u: u64| MotionCtx { mseed: ms, ..mctx(seed, u, 0, 240) };
    let s0 = shiver_start(ms, 0, 280, 1400);
    assert_eq!(s0, (1400.0 * (0.5 + hash01(ms, 1_000_000) as f64)).round() as u64);
    let s1 = shiver_start(ms, 1, 280, 1400);
    assert_eq!(s1, s0 + 280 + (1400.0 * (0.5 + hash01(ms, 1_000_001) as f64)).round() as u64);
    let mut moved = 0;
    for u in 0..s1 + 280 {
        let o = sh.eval(&c(u));
        if u < s0 || (s0 + 280..s1).contains(&u) {
            assert_eq!((o.dx, o.dy), (0, 0), "still between bursts (u {u})");
        } else if u < s0 + 280 {
            let q = (u - s0) / SHIVER_STEP_MS;
            let step = |k: u64| (3.0 * hash01(ms, k)).floor() as i32 - 1;
            assert_eq!((o.dx, o.dy), (step(2_000_000 + 2 * q), step(2_000_001 + 2 * q)), "burst 0 step {q}");
            moved += (o.dx != 0 || o.dy != 0) as i32;
        } else {
            assert!(o.dx.abs() <= 1 && o.dy.abs() <= 1, "burst 1 within 1 px");
        }
    }
    assert!(moved > 0, "the burst moves");

    let down = Motion { rest_dy: 3, rest_ms: 0, enter_dy: 3, enter_ms: 400, ..steady() };
    assert_eq!(down.eval(&mctx(seed, 100, 0, 240)).dy, 3, "6 px down clamps at 3");
    let up = Motion { rest_dy: -3, enter_dy: -3, ..down };
    assert_eq!(up.eval(&mctx(seed, 100, 0, 240)).dy, -3, "6 px up clamps at -3");
    let wide = Motion { enter_stretch: 6, enter_ms: 400, ..steady() };
    assert_eq!(wide.eval(&mctx(seed, 100, 0, 240)).stretch, 4, "the stretch clamps at 4");
    assert_eq!(wide.eval(&mctx(seed, 100, 0, 120)).stretch, motion_px(4.0, 120), "at px(4) on a smaller side");
    let squeeze = Motion { enter_stretch: -6, ..wide };
    assert_eq!(squeeze.eval(&mctx(seed, 100, 0, 240)).stretch, -3, "the squeeze clamps at -3");
}

/// MO5: no border pixel changes under any sample profile (no jolt, no shake); content sampled past the inner edge takes
/// the edge pixel (a 3 px lift repeats the bottom row, a squeeze repeats the top row).
#[test]
fn mo5_the_border_never_moves_and_samples_past_the_edge_take_the_edge() {
    let mut rig = Rig::new();
    let n = 240;
    let src = rows(n);
    let b = Face::plain(n).border;
    for name in MOTION_SAMPLES {
        for t in [0u64, 100, 400, 900, 1700, 2600, 5000] {
            let mut fx = PortraitFx::new("mo5");
            fx.set_base_motion(Motion::sample(name).unwrap());
            let out = rig.compose(&mut fx, &src, None, n, t, PortraitOpts::default());
            for y in 0..n {
                for x in 0..n {
                    if in_border(n, b, x, y) {
                        assert_eq!(at(&out, x, y), at(&src, x, y), "{name} t {t}: the border at ({x}, {y})");
                    }
                }
            }
        }
    }
    let (lo, hi) = (b, n - b);
    let mut fx = PortraitFx::new("mo5");
    fx.set_base_motion(Motion { rest_dy: -3, ..steady() });
    let out = rig.compose(&mut fx, &src, None, n, 500, PortraitOpts::default());
    for x in [lo, n / 2, hi - 1] {
        for y in hi - 3..hi {
            assert_eq!(at(&out, x, y), at(&src, x, hi - 1), "lifted 3 px: the bottom rows repeat the edge ({x}, {y})");
        }
        assert_eq!(at(&out, x, lo), at(&src, x, lo + 3), "the top row shows the row 3 px below");
    }
    let mut fx = PortraitFx::new("mo5");
    fx.set_base_motion(Motion { enter_stretch: -3, enter_ms: 400, ..steady() });
    let out = rig.compose(&mut fx, &src, None, n, 100, PortraitOpts::default());
    assert_eq!(fx.motion(100).stretch, -3);
    for x in [lo, n / 2] {
        assert_eq!(at(&out, x, lo), at(&src, x, lo), "squeezed: the top row repeats the edge");
        assert_eq!(at(&out, x, hi - 1), at(&src, x, hi - 1), "anchored at the bottom");
    }
}

/// MO6: a face with `blink` false draws no lid while it holds (even at a blink's closed phase) and `live` drops `blink`;
/// blinks return once the face is gone.
#[test]
fn mo6_a_face_without_blinks_holds_the_eyes_open() {
    let mut rig = Rig::new();
    let (img, face) = eye_face(true);
    let mut fx = PortraitFx::new("mo6");
    fx.set_expressions(&[moving("hurt", Motion { blink: false, ..Motion::default() })]);
    fx.hold(&[face_cue("hurt")], 0);
    let starts = blink_starts(fx.seed, 60_000);
    let first = starts[0];
    assert_eq!(rig.compose(&mut fx, &img, Some(&face), 96, first + 75, reduce()), img, "no lid while hurt holds");
    assert!(!fx.live(first + 75).contains(&"blink"));
    fx.hold(&[], first + 200);
    let next = *starts.iter().find(|s| **s > first + 400).unwrap();
    let out = rig.compose(&mut fx, &img, Some(&face), 96, next + 75, reduce());
    assert_ne!(out, img, "the blink returns after the face");
    assert_eq!(at(&out, 42, 41), SKIN, "the lid over the eye");
    assert!(fx.live(next + 75).contains(&"blink"));
}

/// MO7: `tint_breath` 0.5 scales the face tint by (1 - F + F b): half at b = 0, full at b = 1; steady at full under
/// reduce motion and at the minimal tier.
#[test]
fn mo7_the_flush_pulses_with_the_cycle() {
    let m = Motion { tint_breath: 0.5, ..Motion::default() };
    let seed = seed_of("mo7");
    let p = m.period(seed);
    let tinted_ctx = |t: u64, side: i32| MotionCtx { tinted: true, ..mctx(seed, t, 0, side) };
    assert_eq!(m.eval(&tinted_ctx(0, 96)).flush, 0.5, "half at b = 0");
    let top = m.eval(&tinted_ctx((p / 2.0).round() as u64, 96));
    assert!((top.flush - 1.0).abs() < 1e-4, "full at b = 1 ({})", top.flush);
    assert!(top.parts.contains(&"flush"));
    assert_eq!(m.eval(&mctx(seed, 0, 0, 96)).flush, 1.0, "no tint, no flush");

    let mut rig = Rig::new();
    let (src, face) = ex_face();
    let row = ExpressionFx { tint: Some(RED), tint_strength: 0.5, motion: m, ..ExpressionFx::new("angry") };
    let mut fx = PortraitFx::new("mo7");
    fx.set_expressions(&[row]);
    fx.hold(&[face_cue("angry")], 0);
    for t in (300..4000u64).step_by(300) {
        let out = rig.compose(&mut fx, &src, Some(&face), 96, t, PortraitOpts::default());
        let o = fx.motion(t);
        assert!((o.flush - (0.5 + 0.5 * o.b)).abs() < 1e-6, "t {t}");
        assert_eq!(at(&out, 48, 48), tinted(SKIN, RED, 0.5 * o.flush), "the face box centre at t {t}");
        let out = rig.compose(&mut fx, &src, Some(&face), 96, t, reduce());
        assert_eq!((fx.motion(t).flush, at(&out, 48, 48)), (1.0, tinted(SKIN, RED, 0.5)), "steady under reduce motion (t {t})");
        rig.compose(&mut fx, &src, Some(&face), 32, t, PortraitOpts::default());
        assert_eq!(fx.motion(t).flush, 1.0, "steady at the minimal tier (t {t})");
    }
}

/// MO8: under reduce motion and at the minimal tier every sample profile gives stretch 0 and offset [0, 0]; at the
/// reduced tier there is no cycle, and the entry and rest are px(v / 2), at least 1 px.
#[test]
fn mo8_reduce_motion_and_the_tiers() {
    let seed = seed_of("mo8");
    for name in MOTION_SAMPLES {
        let m = Motion::sample(name).unwrap();
        for t in (0..6000u64).step_by(23) {
            assert_eq!(sxy(&m.eval(&MotionCtx { reduce: true, ..mctx(seed, t, 0, 240) })), (0, 0, 0), "{name}: none under reduce motion (t {t})");
            assert_eq!(sxy(&m.eval(&mctx(seed, t, 0, REDUCED_SIDE - 1))), (0, 0, 0), "{name}: none at the minimal tier (t {t})");
            let o = m.eval(&mctx(seed, t, 0, 64));
            let entry = (motion_px(m.enter_stretch as f32 / 2.0, 64) as f32 * entry_env(t, m.enter_ms)).round() as i32;
            assert_eq!(o.stretch, if m.has_entry() { entry } else { 0 }, "{name}: no cycle at the reduced tier (t {t})");
            assert!(!o.parts.contains(&"cycle") && !o.parts.contains(&"bob"));
        }
    }
    let lift = Motion::sample("lift").unwrap();
    assert_eq!(lift.eval(&mctx(seed, 2000, 0, 64)).dy, motion_px(lift.rest_dy as f32 / 2.0, 64), "the rest halved");
    assert_eq!(motion_px(lift.rest_dy as f32 / 2.0, 64), -1, "at least 1 px");
    let jolt = Motion::sample("jolt").unwrap();
    let q = jolt.enter_ms as u64 / 4;
    assert_eq!(jolt.eval(&mctx(seed, q, 0, 64)).dy, motion_px(jolt.enter_dy as f32 / 2.0, 64), "the entry halved at its peak");
    let sink = Motion::sample("sink").unwrap();
    assert_eq!(Motion { breath_amp: 0.0, ..sink }.eval(&mctx(seed, 4000, 0, 240)).dy, sink.rest_dy, "full size: the whole rest");
    let mut rig = Rig::new();
    let src = rows(96);
    let mut fx = PortraitFx::new("mo8");
    fx.set_base_motion(jolt);
    assert_eq!(rig.compose(&mut fx, &src, None, 96, q, reduce()), src, "a reduce-motion portrait does not move");
}

/// MO9: `still` stops every part of every sample; when the face changes its motion blends over MOTION_BLEND_MS: at
/// 100 ms the output is the rounded midpoint of the two faces' outputs.
#[test]
fn mo9_still_stops_the_motion_and_a_face_change_blends() {
    let mut rig = Rig::new();
    let src = rows(240);
    for name in MOTION_SAMPLES {
        let mut fx = PortraitFx::new("mo9");
        fx.set_base_motion(Motion::sample(name).unwrap());
        fx.hold(&[cue("still")], 0);
        for t in [300u64, 700, 1500, 2600] {
            assert_eq!(rig.compose(&mut fx, &src, None, 240, t, PortraitOpts::default()), src, "{name}: still (t {t})");
            assert_eq!(state_motion(&fx, t).0, (0, 0, 0));
        }
    }
    let (a, b) = (Motion { rest_dy: -3, ..steady() }, Motion { rest_dy: 2, ..steady() });
    let mut fx = PortraitFx::new("mo9");
    fx.set_expressions(&[moving("lifted", a), moving("sunk", b)]);
    fx.hold(&[face_cue("lifted")], 0);
    rig.compose(&mut fx, &src, None, 240, 1000, PortraitOpts::default());
    assert_eq!(state_motion(&fx, 1000).0, (0, 0, -3));
    fx.hold(&[face_cue("sunk")], 2000);
    rig.compose(&mut fx, &src, None, 240, 2000, PortraitOpts::default());
    assert_eq!(state_motion(&fx, 2000).0, (0, 0, -3), "the blend starts at the old face");
    rig.compose(&mut fx, &src, None, 240, 2100, PortraitOpts::default());
    let (m, parts) = state_motion(&fx, 2100);
    assert_eq!(m, (0, 0, ((-3.0f32 + 2.0) / 2.0).round() as i64), "the rounded midpoint at 100 ms");
    assert!(parts.contains(&"blend".to_string()), "{parts:?}");
    rig.compose(&mut fx, &src, None, 240, 2300, PortraitOpts::default());
    assert_eq!(state_motion(&fx, 2300).0, (0, 0, 2), "the new face after the blend");
}

/// MO10: the blink lids, the mark and the face tint box follow the motion (shifted by px(3) for a 3 px rest at side
/// 96); the ash does not.
#[test]
fn mo10_the_anchored_layers_follow_the_motion_and_ash_does_not() {
    let mut rig = Rig::new();
    let sh = motion_px(3.0, 96);
    let (calm, down) = (steady(), Motion { rest_dy: 3, ..steady() });
    let shifted = |b: Option<(i32, i32, i32, i32)>| b.map(|(x, y, w, h)| (x, y + sh, w, h));
    // the lids
    let (img, face) = eye_face(true);
    let first = blink_starts(seed_of("mo10"), 20_000)[0];
    let lid = |m: Motion, rig: &mut Rig| {
        let mut fx = PortraitFx::new("mo10");
        fx.set_base_motion(m);
        let open = rig.compose(&mut fx, &img, Some(&face), 96, first - 40, PortraitOpts::default());
        let closed = rig.compose(&mut fx, &img, Some(&face), 96, first + 75, PortraitOpts::default());
        changed(&closed, &open)
    };
    let still_lid = lid(calm, &mut rig);
    assert!(still_lid.is_some());
    assert_eq!(lid(down, &mut rig), shifted(still_lid), "the lid moves with the eyes");
    // the mark and the face tint
    let (src, face) = ex_face();
    let vein = ExpressionFx { mark: Some(Mark::Vein), mark_color: Some(GREEN), ..ExpressionFx::new("angry") };
    let tint = ExpressionFx { tint: Some(RED), tint_strength: 0.5, ..ExpressionFx::new("angry") };
    for row in [vein, tint] {
        let drawn = |m: Motion, rig: &mut Rig| {
            let mut fx = PortraitFx::new("mo10");
            fx.set_expressions(&[ExpressionFx { motion: m, ..row.clone() }]);
            fx.hold(&[face_cue("angry")], 0);
            changed(&rig.compose(&mut fx, &src, Some(&face), 96, 1000, PortraitOpts::default()), &src)
        };
        let base = drawn(calm, &mut rig);
        assert!(base.is_some());
        assert_eq!(drawn(down, &mut rig), shifted(base), "{:?} / {:?} moves with the motion", row.mark, row.tint);
    }
    // the ash stays where it falls
    let ash = |m: Motion, rig: &mut Rig| {
        let mut fx = PortraitFx::new("mo10");
        fx.set_base_motion(m);
        fx.hold(&[cue("ash")], 0);
        rig.compose(&mut fx, &src, None, 96, 1000, PortraitOpts::default())
    };
    let still_ash = ash(calm, &mut rig);
    assert_ne!(still_ash, src, "the ash draws");
    assert_eq!(ash(down, &mut rig), still_ash, "the ash does not follow the motion");
}

/// MO11: at side 120 each amplitude is px(v) (a 3 px rest is 2 px, a 1 px shiver stays 1 px, as at 96); the Budget
/// counts a moving face and not a rest-only face once settled.
#[test]
fn mo11_amplitudes_scale_and_the_budget_counts_moving_faces() {
    let seed = seed_of("mo11");
    let lift = Motion { rest_dy: -3, ..steady() };
    assert_eq!(lift.eval(&mctx(seed, 100, 0, 120)).dy, -((3.0f32 * 120.0 / MOTION_REF as f32).round() as i32));
    let sh = Motion { shiver_px: 1, shiver_ms: 280, shiver_gap_ms: 1400, ..steady() };
    for side in [120, 96] {
        let mut seen = 0;
        for u in 0..6000 {
            let o = sh.eval(&mctx(seed, u, 0, side));
            assert!(o.dx.abs() <= 1 && o.dy.abs() <= 1, "1 px at side {side}");
            seen += (o.dx != 0) as i32;
        }
        assert!(seen > 0, "the shiver shows at side {side}");
    }
    let mut rig = Rig::new();
    let src = rows(120);
    let used = |m: Motion, t: u64, rig: &mut Rig| {
        let mut fx = PortraitFx::new("mo11");
        fx.set_base_motion(m);
        let mut budget = Budget::default();
        budget.begin();
        rig.draw(&mut fx, &src, None, Rect::new(8, 8, 120, 120), t, &PortraitOpts::default(), &mut budget);
        budget.used
    };
    let rest = Motion { rest_dy: -1, rest_ms: 400, ..steady() };
    assert_eq!(used(sh, 1000, &mut rig), 1, "a shiver is time-varying");
    assert_eq!(used(rest, 200, &mut rig), 1, "a rest still moving");
    assert_eq!(used(rest, 1000, &mut rig), 0, "a settled rest-only face is not");
    assert_eq!(used(Motion::default(), 1000, &mut rig), 1, "the breath is");
}

/// The Kit widgets Portraits page offers the sample profiles (`kit.portrait.motion`) and names the live parts.
#[test]
fn mo_the_kit_page_offers_the_motion_samples() {
    assert_eq!(Motion::sample("breath"), Some(Motion::default()));
    assert!(MOTION_SAMPLES.iter().all(|n| Motion::sample(n).is_some()) && Motion::sample("smug").is_none());
    let mut k = Specimens::default();
    let (w, _, _, _) = draw_kit(&k);
    k.act(by_id(&w, "kit.page.3").unwrap());
    let (w, _, _, _) = draw_kit(&k);
    assert!(w.iter().any(|x| x.id == "kit.portrait.motion"), "the motion choice");
    let wince = MOTION_SAMPLES.iter().position(|n| *n == "wince").unwrap();
    k.act(KitAct::PortraitMotion(wince));
    assert_eq!(k.last, "portrait motion wince");
    k.advance(60.0);
    let (_, _, recs, audit) = draw_kit(&k);
    assert_eq!(audit["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "the page reads clean: {audit}");
    assert!(recs.iter().any(|t| t.text.starts_with("Motion wince:") && t.text.contains("entry")), "the foot names the entry");
    let (_, parts) = state_motion(&k.portrait.fx.borrow(), k.portrait.clock.now());
    assert!(parts.contains(&"entry".to_string()), "{parts:?}");
}
