//! FX1-E (FX1 section 3, checks E1-E13 of FX1_TASKS.md 1.1): the effect clock with
//! hit-stop, the six primitives as pure functions of the clock and a seed, the live list, the camera shake (presets,
//! limits, the engine ceiling, reduce motion), `Painter::shift`, the silhouette flash, `Settings::reduce_motion` and
//! the Effects page of the Kit widgets. Expectations follow from the spec's rules.

use aegis_core::assets::Layers;
use aegis_ui::art::Art;
use aegis_ui::canvas::{Canvas, Image, Rect, Rgba, OUTLINE_ALPHA};
use aegis_ui::fx::*;
use aegis_ui::painter::{Fonts, Painter, View};
use aegis_ui::screen::Settings;
use aegis_ui::specimens::{KitAct, Specimens, FX_SPEEDS};
use aegis_ui::text_audit;
use aegis_ui::ui::{by_id, Gfx, Kit, Ui, Widget, BG};
use aegis_ui::vignette;
use serde_json::json;
use std::path::Path;

const WHITE: Rgba = [255, 255, 255, 255];

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

/// A 640 x 360 canvas at scale 1 with `draw` run on it.
fn render(fonts: &mut Fonts, draw: impl FnOnce(&mut Painter)) -> Canvas {
    let mut c = Canvas::new(640, 360);
    {
        let mut p = Painter { c: &mut c, v: View::fit(640, 360), fonts };
        draw(&mut p);
    }
    c
}

fn blank(c: &Canvas) -> bool {
    c.px.iter().all(|b| *b == 0)
}

fn px(c: &Canvas, x: i32, y: i32) -> [u8; 4] {
    let i = ((y * c.w + x) * 4) as usize;
    [c.px[i], c.px[i + 1], c.px[i + 2], c.px[i + 3]]
}

/// One sample of each primitive, 200 ms long, around (100, 100).
fn samples() -> Vec<Prim> {
    vec![
        Projectile { from: (60.0, 100.0), to: (160.0, 100.0), travel_ms: 200, arc_px: 6.0, look: Look::Arrow { len: 7.0, shaft: WHITE, head: WHITE } }.into(),
        Beam { from: (60.0, 100.0), to: (160.0, 100.0), ms: 200, width: 2, core: WHITE, glow: [192, 132, 252, 255] }.into(),
        Trail { at: (100.0, 100.0), radius: 13.0, from_deg: -60.0, to_deg: 50.0, ms: 200, width: 2, col: WHITE, mirror: false, squash: 0.75 }.into(),
        Sparks { at: (100.0, 100.0), ms: 200, count: 8, dir_deg: 0.0, spread_deg: 90.0, speed: 70.0, gravity: 120.0, size: 1, cols: [WHITE, [217, 203, 180, 255]] }.into(),
        Flash { at: (100.0, 100.0), rx: 6.0, ry: 4.0, ms: 200, col: WHITE }.into(),
        Ring { at: (100.0, 100.0), r0: 3.0, r1: 12.0, ms: 200, col: WHITE, squash: 0.75 }.into(),
    ]
}

#[test]
fn e1_the_clock_runs_at_its_speed_and_a_hold_eats_wall_time() {
    let mut c = Clock::default();
    c.advance(100.0, 1.0);
    assert_eq!(c.now(), 100);
    c.hold(40.0);
    c.advance(100.0, 1.0);
    assert_eq!(c.now(), 160, "a 40 ms hold eats 40 of the 100 wall ms");
    c.hold(40.0);
    c.hold(30.0);
    assert_eq!(c.hold, 40.0, "holds do not add: the longer one stays");
    c.advance(100.0, 1.0);
    let t = c.now();
    c.advance(100.0, 0.5);
    assert_eq!(c.now(), t + 50, "half speed");
    c.advance(100.0, 2.0);
    assert_eq!(c.now(), t + 250, "double speed");
    let mut h = Clock::default();
    h.hold(150.0);
    h.advance(100.0, 2.0);
    assert_eq!((h.now(), h.hold), (0, 50.0), "a hold longer than the frame keeps the clock still");
}

#[test]
fn e2_every_primitive_draws_only_inside_its_life_and_the_same_way_twice() {
    let mut fonts = Fonts::new(&font());
    for prim in samples() {
        let e = Effect::new(prim, 100).seeded(7);
        assert_eq!((e.end(), e.u(99), e.u(300)), (300, None, None), "{}", e.kind());
        for now in [0, 99, 300, 400] {
            assert!(blank(&render(&mut fonts, |p| e.draw(p, now))), "{} draws nothing at {now}", e.kind());
        }
        let a = render(&mut fonts, |p| e.draw(p, 180));
        let b = render(&mut fonts, |p| e.draw(p, 180));
        assert!(!blank(&a), "{} shows during its life", e.kind());
        assert!(a.px == b.px, "{} draws the same pixels for the same now", e.kind());
    }
}

#[test]
fn e3_a_projectile_leaves_from_lands_on_and_arcs_its_height_half_way() {
    let p = Projectile { from: (10.0, 50.0), to: (110.0, 30.0), travel_ms: 300, arc_px: 6.0, look: Look::Bolt { len: 5.0, shaft: WHITE, head: WHITE } };
    assert_eq!((p.pos(0.0), p.pos(1.0)), ((10.0, 50.0), (110.0, 30.0)));
    let (x, y) = p.pos(0.5);
    let straight = (60.0, 40.0);
    assert!((x - straight.0).abs() < 1e-4 && (straight.1 - y - 6.0).abs() < 1e-4, "6 px above the straight line half way: {x}, {y}");
    let flat = Projectile { arc_px: 0.0, ..p };
    let d = flat.dir(0.3);
    let l = (100.0f32 * 100.0 + 20.0 * 20.0).sqrt();
    assert!((d.0 - 100.0 / l).abs() < 1e-4 && (d.1 + 20.0 / l).abs() < 1e-4, "a flat shot points along its path");
}

#[test]
fn e4_sparks_follow_their_seed_and_fade_to_nothing() {
    let mut fonts = Fonts::new(&font());
    let s = Sparks { at: (300.0, 180.0), ms: 400, count: 12, dir_deg: -90.0, spread_deg: 160.0, speed: 120.0, gravity: 80.0, size: 2, cols: [WHITE, [251, 191, 36, 255]] };
    let a = render(&mut fonts, |p| Effect::new(s, 0).seeded(1).draw(p, 200));
    let b = render(&mut fonts, |p| Effect::new(s, 0).seeded(1).draw(p, 200));
    let c = render(&mut fonts, |p| Effect::new(s, 0).seeded(2).draw(p, 200));
    assert!(a.px == b.px, "same seed, same pixels");
    assert!(a.px != c.px, "another seed scatters them elsewhere");
    let half = s.at_time(1, 200);
    assert_eq!(half.len(), 12);
    assert!(half.iter().all(|(_, col)| col[3] == 128), "half way the alpha is half: {:?}", half[0].1);
    assert!(s.at_time(1, 400).iter().all(|(_, col)| col[3] == 0), "the alpha falls to 0 at u 1");
    assert!(blank(&render(&mut fonts, |p| Effect::new(s, 0).seeded(1).draw(p, 400))));
    let start = s.at_time(1, 0);
    assert!(start.iter().all(|((x, y), _)| (*x, *y) == (300.0, 180.0)), "every spark leaves from `at`");
}

#[test]
fn e5_hash01_stays_in_the_unit_range_and_repeats() {
    for k in 0..10_000u64 {
        let h = hash01(42, k);
        assert!((0.0..1.0).contains(&h), "key {k}: {h}");
        assert_eq!(h, hash01(42, k));
    }
    assert_ne!(hash01(42, 1), hash01(43, 1), "the seed matters");
}

#[test]
fn e6_the_live_list_keeps_32_prunes_the_done_and_draws_by_layer() {
    let flash = |x: f32, ms: u32| Flash { at: (x, 100.0), rx: 4.0, ry: 4.0, ms, col: WHITE };
    let mut fx = Fx::default();
    for i in 0..MAX_LIVE as u64 {
        fx.push(Effect::new(flash(10.0, 100), 10 + i));
    }
    fx.push(Effect::new(flash(20.0, 100), 10));
    assert_eq!(fx.len(), MAX_LIVE);
    assert!(!fx.live.iter().any(|e| e.start == 10 && matches!(e.prim, Prim::Flash(f) if f.at.0 == 10.0)), "the 33rd push drops the oldest");
    assert_eq!(fx.live.last().unwrap().start, 10, "and keeps the new one");
    assert_eq!(fx.busy_until(), Some(10 + MAX_LIVE as u64 - 1 + 100), "the latest end");
    let mut ties = Fx::default();
    for i in 0..MAX_LIVE {
        ties.push(Effect::new(flash(i as f32, 100), 5));
    }
    ties.push(Effect::new(flash(99.0, 100), 5));
    let xs: Vec<f32> = ties.live.iter().map(|e| if let Prim::Flash(f) = e.prim { f.at.0 } else { -1.0 }).collect();
    assert_eq!((xs[0], *xs.last().unwrap(), xs.len()), (1.0, 99.0, MAX_LIVE), "same start: the first pushed goes");

    let mut two = Fx::default();
    two.push(Effect::new(flash(100.0, 100), 0).on(Layer::Ground));
    two.push(Effect::new(flash(300.0, 400), 0).on(Layer::Air));
    assert_eq!((two.busy_until(), two.kinds(50)), (Some(400), vec!["flash", "flash"]));
    let mut fonts = Fonts::new(&font());
    let ground = render(&mut fonts, |p| two.draw(p, 50, Layer::Ground));
    assert!(px(&ground, 100, 100)[3] > 0 && px(&ground, 300, 100)[3] == 0, "one layer draws only its own effects");
    let air = render(&mut fonts, |p| two.draw(p, 50, Layer::Air));
    assert!(px(&air, 100, 100)[3] == 0 && px(&air, 300, 100)[3] > 0);
    two.prune(100);
    assert_eq!((two.len(), two.kinds(100)), (1, vec!["flash"]), "prune drops the done");
    two.clear();
    assert_eq!((two.busy_until(), two.is_empty()), (None, true));
}

#[test]
fn e7_camera_offsets_stay_in_bounds_follow_their_axis_and_alternate() {
    let table = ShakeTable::default();
    for (name, s) in table.rows() {
        for seed in [1u64, 99, 12345] {
            let mut cam = Camera::default();
            assert!(cam.start(*s, 1000, seed, ShakeLimits::CEILING, false), "{name} starts");
            let mut seen: Vec<(u64, i32)> = Vec::new();
            for t in 0..s.ms as u64 + 50 {
                let (dx, dy) = cam.offset_at(1000 + t);
                assert!(dx.abs() as f32 <= s.px && dy.abs() as f32 <= s.px, "{name} at {t}: ({dx}, {dy})");
                match s.axis {
                    Axis::X => assert_eq!(dy, 0, "{name}: x only"),
                    Axis::Y => assert_eq!(dx, 0, "{name}: y only"),
                    Axis::Both => {}
                }
                if t >= s.ms as u64 {
                    assert_eq!((dx, dy), (0, 0), "{name}: still at and after its end");
                }
                let main = if s.axis == Axis::Y { dy } else { dx };
                if main != 0 {
                    seen.push((t / s.step_ms as u64, main.signum()));
                }
            }
            assert!(seen.len() > 1, "{name} moves");
            for w in seen.windows(2) {
                let ((k0, s0), (k1, s1)) = (w[0], w[1]);
                let flips = (k1 - k0) % 2 == 1;
                assert_eq!(s0 != s1, flips, "{name}: the sign flips with every step ({k0} -> {k1})");
            }
            if s.envelope == Envelope::Swell {
                assert_eq!(cam.offset_at(1000), (0, 0), "{name}: a swell starts still");
            }
            assert_eq!(cam.offset(1000 + s.ms as u64), (0, 0));
            assert!(cam.shake.is_none(), "{name} ends once its time is up");
        }
    }
}

#[test]
fn e8_limits_clamp_a_shake_and_the_ceiling_clamps_the_limits() {
    let big = Shake { px: 9.0, ms: 5000, axis: Axis::X, envelope: Envelope::Linear, step_ms: 30 };
    let mut cam = Camera::default();
    assert!(cam.start(big, 0, 3, ShakeLimits::new(3.0, 1500), false));
    let (s, start, _) = cam.shake.unwrap();
    assert_eq!((s.px, s.ms, start), (3.0, 1500, 0), "it runs at 3 px for 1500 ms");
    assert!(cam.active(1499) && !cam.active(1500));
    assert!((0..1500).map(|t| cam.offset_at(t).0.abs()).max() == Some(3), "at full strength it reaches the limit");
    assert_eq!(ShakeLimits::new(10.0, 9000), ShakeLimits { max_px: SHAKE_CEIL_PX, max_ms: SHAKE_CEIL_MS }, "limits never pass the ceiling");
    let capped = big.clamped(ShakeLimits { max_px: 10.0, max_ms: 9000 });
    assert_eq!((capped.px, capped.ms), (6.0, 3000), "unclamped limits are clamped to the ceiling first");
}

#[test]
fn e9_reduce_motion_starts_nothing_and_a_weaker_shake_does_not_replace_a_stronger() {
    let t = ShakeTable::default();
    let (jolt, quake) = (t.get("jolt").unwrap(), t.get("quake").unwrap());
    let mut cam = Camera::default();
    assert!(!cam.start(quake, 0, 1, ShakeLimits::CEILING, true));
    assert_eq!((cam.shake, cam.offset_at(10)), (None, (0, 0)), "reduce motion: no shake");
    assert!(cam.start(quake, 0, 1, ShakeLimits::CEILING, false));
    assert!(cam.amplitude(10) > jolt.px);
    assert!(!cam.start(jolt, 10, 2, ShakeLimits::CEILING, false), "a weaker shake is ignored");
    assert_eq!(cam.shake.unwrap().0, quake);
    let mut cam = Camera::default();
    cam.start(jolt, 0, 1, ShakeLimits::CEILING, false);
    assert!(cam.start(quake, 10, 2, ShakeLimits::CEILING, false), "a stronger shake replaces it");
    assert_eq!(cam.shake.unwrap(), (quake, 10, 2));
}

#[test]
fn e10_shift_moves_later_drawing_by_whole_px_and_keeps_an_earlier_clip() {
    let mut fonts = Fonts::new(&font());
    let c = render(&mut fonts, |p| {
        let v = p.v;
        p.clip(Some(Rect::new(0, 0, 21, 360)));
        p.shift(2, 0);
        p.fill(Rect::new(10, 10, 1, 1), WHITE);
        p.fill(Rect::new(19, 12, 1, 1), WHITE);
        p.shift(-2, 0);
        assert_eq!(p.v, v, "shift(-2, 0) restores the view");
        p.fill(Rect::new(30, 14, 1, 1), WHITE);
    });
    assert_eq!((px(&c, 12, 10), px(&c, 10, 10)), (WHITE, [0; 4]), "the fill lands 2 px right");
    assert_eq!(px(&c, 21, 12), [0; 4], "the clip set before the shift stays where it was");
    assert_eq!(px(&c, 30, 14), [0; 4], "the clip still holds after the undo");
    let mut c2 = Canvas::new(1280, 720);
    {
        let mut p = Painter { c: &mut c2, v: View::fit(1280, 720), fonts: &mut fonts };
        p.shift(2, 0);
        p.fill(Rect::new(10, 10, 1, 1), WHITE);
    }
    assert_eq!((px(&c2, 24, 20), px(&c2, 25, 21), px(&c2, 20, 20)), (WHITE, WHITE, [0; 4]), "at scale 2 a logical px is 2 physical");
}

/// A scratch asset root with body `hero`: one 8 x 8 `idle` frame holding an L (not symmetric left to right).
fn hero_root() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_ui_fx_body_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("units/hero/base");
    std::fs::create_dir_all(&dir).unwrap();
    let mut px = vec![0u8; 8 * 8 * 4];
    for (x, y, a) in [(1, 1, 255u8), (1, 2, 255), (1, 3, 255), (1, 4, 200), (2, 4, 255), (3, 4, 130), (4, 4, 90)] {
        let i = (y * 8 + x) * 4;
        px[i..i + 4].copy_from_slice(&[40, 80, 120, a]);
    }
    let file = std::fs::File::create(dir.join("sheet.png")).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 8, 8);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&px).unwrap();
    std::fs::write(dir.join("clips.json"), json!({"frame": [8, 8], "clips": [{"id": "idle", "frames": 1, "ms": [100], "loop": true}]}).to_string()).unwrap();
    root
}

#[test]
fn e11_a_silhouette_paints_the_solid_pixels_and_mirrors_with_flip() {
    let img = Image { w: 3, h: 2, px: vec![1, 2, 3, 255, 0, 0, 0, 0, 9, 9, 9, 200, 5, 5, 5, OUTLINE_ALPHA - 1, 7, 7, 7, OUTLINE_ALPHA, 1, 1, 1, 10] };
    let s = img.silhouette(WHITE);
    assert_eq!((s.w, s.h), (3, 2));
    let pixels: Vec<Rgba> = s.px.as_chunks::<4>().0.to_vec();
    assert_eq!(pixels, vec![WHITE, [0; 4], WHITE, [0; 4], WHITE, [0; 4]], "solid pixels take the colour, the rest are clear");

    let root = hero_root();
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    let col: Rgba = [255, 255, 255, 150];
    let plain = art.body_silhouette("hero", "", "idle", 0, 16, col, false).cloned().unwrap();
    let flipped = art.body_silhouette("hero", "", "idle", 0, 16, col, true).cloned().unwrap();
    let frame = art.body_clip("hero", "", "idle", 0, 16).cloned().unwrap();
    assert_eq!(plain, frame.silhouette(col), "the fitted frame's silhouette, same size and place");
    assert_eq!(flipped, plain.mirrored(), "flip mirrors it");
    assert_ne!(flipped, plain, "the fixture is not symmetric");
    assert!(art.body_silhouette("nobody", "", "idle", 0, 16, col, false).is_none());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn e12_reduce_motion_is_the_shared_setting() {
    let mut s = Settings::load(std::env::temp_dir().join("aegis_fx_no_settings.json"), json!({}));
    assert!(!s.reduce_motion(), "off on the defaults");
    s.v[vignette::REDUCE_MOTION] = json!(true);
    assert!(s.reduce_motion() && vignette::reduce_motion(&s), "on when set, the same key the vignette player reads");
    s.v.as_object_mut().unwrap().shift_remove(vignette::REDUCE_MOTION);
    assert!(!s.reduce_motion(), "false when missing");
}

fn gfx() -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.canvas.clear(BG);
    g
}

/// The kit page drawn as the gallery does: its widgets, its pixels and the text audit.
fn draw_kit(k: &Specimens) -> (Vec<Widget<KitAct>>, Canvas, serde_json::Value) {
    let mut g = gfx();
    g.fonts.audit = Some(Vec::new());
    let w = {
        let mut ui: Ui<KitAct> = Ui::new(&mut g, (-1, -1), true);
        k.draw(&mut ui, Rect::new(112, 22, 522, 332), |a| a);
        ui.widgets
    };
    let boxes: Vec<_> = w.iter().map(|x| (x.id.clone(), x.rect.x as f32, x.rect.y as f32, x.rect.w as f32, x.rect.h as f32)).collect();
    let issues = text_audit::issues(&g.fonts.audit.take().unwrap_or_default(), 640.0, 360.0, &boxes);
    (w, g.canvas, issues)
}

#[test]
fn e13_the_effects_page_lists_every_piece_and_its_shakes_move_the_stage_only() {
    let mut k = Specimens::default();
    let (w, _, _) = draw_kit(&k);
    k.act(by_id(&w, "kit.page.2").unwrap());
    let (w, before, audit) = draw_kit(&k);
    assert_eq!(audit["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "the page reads clean: {audit}");
    for kind in PRIM_KINDS {
        assert!(by_id(&w, &format!("kit.fx.{kind}")).is_ok(), "button for {kind}");
    }
    for name in SHAKE_PRESETS {
        assert!(by_id(&w, &format!("kit.fx.shake.{name}")).is_ok(), "button for {name}");
    }
    assert!(w.iter().any(|x| x.id == "kit.fx.speed") && by_id(&w, "kit.fx.hitstop").is_ok() && by_id(&w, "kit.fx.reduce").is_ok());
    let stage = w.iter().find(|x| x.id == "kit.fx.stage").unwrap().rect;

    k.act(by_id(&w, "kit.fx.shake.toll").unwrap());
    assert!(k.last.starts_with("shake toll"), "the foot line says what ran: {}", k.last);
    k.advance(20.0);
    let now = k.fx.clock.now();
    assert_ne!(k.fx.camera.offset_at(now), (0, 0), "the toll moves the camera");
    let (w2, shaken, _) = draw_kit(&k);
    let mut still = k.clone();
    still.fx.camera.stop();
    let (_, unshaken, _) = draw_kit(&still);
    let inside = |x: i32, y: i32| stage.contains(x, y);
    let (mut moved, mut outside_changed) = (0, 0);
    for y in 0..360 {
        for x in 0..640 {
            if px(&shaken, x, y) != px(&unshaken, x, y) {
                if inside(x, y) {
                    moved += 1;
                } else {
                    outside_changed += 1;
                }
            }
        }
    }
    assert!(moved > 0, "the stage shakes");
    assert_eq!(outside_changed, 0, "the panels around it do not");
    assert_eq!(w2.iter().find(|x| x.id == "kit.fx.stage").unwrap().rect, stage, "and no widget rect moves");
    assert!(before.px != shaken.px || moved > 0);

    k.act(KitAct::Reduce);
    assert!(k.fx.camera.shake.is_none(), "reduce motion stops a running shake");
    k.act(KitAct::Shake("quake"));
    assert!(k.fx.camera.shake.is_none() && k.last.contains("reduce motion"), "and starts none: {}", k.last);
}

#[test]
fn the_effects_page_fires_impacts_with_a_hit_stop_on_its_clock() {
    let mut k = Specimens { page: 2, ..Specimens::default() };
    k.act(KitAct::Effect("projectile"));
    let (t, _) = k.fx.impacts[0];
    assert!(t > 0 && k.fx.fx.kinds(0) == vec!["projectile"]);
    k.advance(t as f64);
    assert_eq!((k.fx.impacts.len(), k.fx.clock.hold), (0, 40.0), "the impact fired and holds the clock");
    assert_eq!(k.fx.fx.kinds(k.fx.clock.now()), vec!["sparks", "flash"], "its sparks and flash, the arrow done");
    let at = k.fx.clock.now();
    k.advance(40.0);
    assert_eq!(k.fx.clock.now(), at, "the hit-stop holds 40 wall ms");
    k.act(KitAct::HitStop);
    k.act(KitAct::FxSpeed(2));
    assert_eq!((k.fx.hit_stop, FX_SPEEDS[k.fx.speed].1), (false, 2.0));
    k.act(KitAct::Effect("beam"));
    let at = k.fx.clock.now();
    k.advance(10.0);
    assert_eq!((k.fx.clock.hold, k.fx.clock.now()), (0.0, at + 20), "no hit-stop when off; double speed");
    for kind in PRIM_KINDS {
        k.act(KitAct::Effect(kind));
    }
    assert!(k.fx.fx.len() <= MAX_LIVE);
}
