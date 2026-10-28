//! STATUSFX1-E (STATUSFX1 section 2, checks E1-E5): `fx::Looped` (reseeded per cycle, a gap after the effect's `ms`,
//! `draw_still` for reduce motion), the `Orbit` primitive, a closing `Ring`, `Painter::image_alpha` and the Effects
//! page's orbit button and loop toggle. Expectations follow from the spec's rules.

use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::fx::*;
use aegis_ui::painter::{Fonts, Painter, View};
use aegis_ui::specimens::{KitAct, Specimens, FX_LOOP_GAP_MS};
use aegis_ui::text_audit;
use aegis_ui::ui::{by_id, Gfx, Kit, Ui, Widget, BG};
use serde_json::json;
use std::path::Path;

const WHITE: Rgba = [255, 255, 255, 255];
const GOLD: Rgba = [251, 191, 36, 255];
const LINEN: Rgba = [217, 203, 180, 255];

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

fn sparks() -> Sparks {
    Sparks { at: (300.0, 180.0), ms: 300, count: 8, dir_deg: -90.0, spread_deg: 120.0, speed: 60.0, gravity: -20.0, size: 1, cols: [GOLD, LINEN] }
}

fn ring() -> Ring {
    Ring { at: (300.0, 180.0), r0: 2.0, r1: 9.0, ms: 300, col: WHITE, squash: 0.75 }
}

fn orbit() -> Orbit {
    Orbit { at: (300.0, 180.0), rx: 20.0, ry: 8.0, ms: 1000, count: 3, size: 1, cols: [GOLD, LINEN], phase_deg: 0.0, star: false }
}

#[test]
fn e1_a_looped_effect_waits_for_its_start_rests_after_its_ms_and_reseeds_each_cycle() {
    let mut fonts = Fonts::new(&font());
    let (start, period) = (100u64, 500u64);
    let l = Looped::new(Effect::new(sparks(), start).seeded(9), period);
    assert_eq!((l.kind(), l.phase(99), l.phase(start + 1234)), ("sparks", None, Some((2, 234))));
    for now in [0, start - 1, start + 300, start + 499, start + period + 300, start + 2 * period + 450] {
        assert!(!l.showing(now) && blank(&render(&mut fonts, |p| l.draw(p, now))), "nothing at {now}");
    }
    let c0 = render(&mut fonts, |p| l.draw(p, start + 150));
    let c1 = render(&mut fonts, |p| l.draw(p, start + period + 150));
    assert!(!blank(&c0) && !blank(&c1), "each cycle shows inside its ms");
    assert!(c0.px != c1.px, "cycle 1 scatters its sparks elsewhere");
    assert_ne!(sparks().at_time(l.seed_at(0), 150), sparks().at_time(l.seed_at(1), 150));
    let again = Looped::new(Effect::new(sparks(), start).seeded(9), period);
    assert!(render(&mut fonts, |p| again.draw(p, start + period + 150)).px == c1.px, "the same in a second run");
    let one_shot = render(&mut fonts, |p| Effect::new(sparks(), start).seeded(9).draw(p, start + 150));
    assert!(one_shot.px == c0.px, "cycle 0 draws as the one-shot");

    let r = Looped::new(Effect::new(ring(), start), period);
    let first = render(&mut fonts, |p| r.draw(p, start + 120));
    assert!(!blank(&first));
    for k in 1..4 {
        assert!(render(&mut fonts, |p| r.draw(p, start + k * period + 120)).px == first.px, "a ring is the same in cycle {k}");
    }
    let short = Looped::new(Effect::new(ring(), 0), 200);
    assert!(short.showing(199) && short.phase(250) == Some((1, 50)), "a period shorter than the ms cuts each cycle at the period");
}

#[test]
fn e2_draw_still_is_the_inner_effect_half_way_whatever_the_clock() {
    let mut fonts = Fonts::new(&font());
    let prims: Vec<Prim> = vec![sparks().into(), ring().into(), orbit().into(), Flash { at: (300.0, 180.0), rx: 6.0, ry: 4.0, ms: 300, col: WHITE }.into()];
    for prim in prims {
        let e = Effect::new(prim, 700).seeded(4);
        let l = Looped::new(e, e.ms() + 200);
        let still = render(&mut fonts, |p| l.draw_still(p));
        let half = render(&mut fonts, |p| e.draw(p, e.start + e.ms() / 2));
        assert!(e.ms().is_multiple_of(2) && e.u(e.start + e.ms() / 2) == Some(0.5));
        assert!(!blank(&still) && still.px == half.px, "{}: draw_still is the effect at u 0.5", e.kind());
    }
}

#[test]
fn e3_orbit_dots_circle_evenly_and_dim_in_the_back_half() {
    let o = orbit();
    assert_eq!((o.angle(0, 0.0), o.angle(0, 0.25), o.angle(1, 0.0), o.angle(2, 0.0)), (0.0, 90.0, 120.0, 240.0));
    let near = |a: Pt, b: Pt| (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3;
    let (p0, c0, back0) = o.dot(0, 0.0);
    assert!(near(p0, (o.at.0 + o.rx, o.at.1)) && !back0 && c0 == GOLD, "dot 0 at angle 0: right of the centre, front, full alpha");
    let (p90, _, back90) = o.dot(0, 0.25);
    assert!(near(p90, (o.at.0, o.at.1 + o.ry)) && !back90, "at u 0.25 it is at 90 deg: below the centre, the front");
    let (_, c1, back1) = o.dot(1, 0.0);
    let (p2, c2, back2) = o.dot(2, 0.0);
    assert!(!back1 && c1 == LINEN, "dot 1 at 120 deg: front, cols[1]");
    assert!(back2 && p2.1 < o.at.1 && c2 == [GOLD[0], GOLD[1], GOLD[2], ORBIT_BACK_ALPHA], "dot 2 at 240 deg: the back half at alpha 110");
    let shifted = Orbit { phase_deg: 90.0, ..o };
    assert!(near(shifted.dot(0, 0.0).0, p90), "the phase turns every dot");
    assert_eq!(Orbit { count: 0, ..o }.dots(), 1);
    assert_eq!(Orbit { count: 12, ..o }.dots(), 8);

    let mut fonts = Fonts::new(&font());
    let e = Effect::new(o, 0);
    assert_eq!((e.kind(), e.ms()), ("orbit", 1000));
    assert!(PRIM_KINDS.contains(&"orbit"));
    let c = render(&mut fonts, |p| e.draw(p, 0));
    let (x0, y0) = (p0.0.round() as i32, p0.1.round() as i32);
    assert_eq!(px(&c, x0, y0), GOLD, "a front dot draws in its colour");
    let (x2, y2) = (p2.0.round() as i32, p2.1.round() as i32);
    let dim = |v: u8| ((v as u32 * ORBIT_BACK_ALPHA as u32) / 255) as u8;
    assert_eq!(px(&c, x2, y2), [dim(GOLD[0]), dim(GOLD[1]), dim(GOLD[2]), 255], "a back dot blends at alpha 110");
    let star = render(&mut fonts, |p| Effect::new(Orbit { star: true, ..o }, 0).draw(p, 0));
    for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        assert_eq!(px(&star, x0 + dx, y0 + dy), GOLD, "a star is a 3 px plus ({dx}, {dy})");
    }
    assert_eq!(px(&star, x0 + 1, y0 + 1), [0; 4], "and no square");
    assert_eq!((px(&star, x0 + 2, y0), px(&star, x0, y0 - 2)), ([0; 4], [0; 4]), "size 1: arms of 1 px");
    // STATUSFX1-L (from STATUSFX1-E): a star's size is its arm, so stun's size 2 is a 5 px plus
    let big = render(&mut fonts, |p| Effect::new(Orbit { star: true, size: 2, ..o }, 0).draw(p, 0));
    for (dx, dy) in [(0, 0), (-2, 0), (-1, 0), (1, 0), (2, 0), (0, -2), (0, -1), (0, 1), (0, 2)] {
        assert_eq!(px(&big, x0 + dx, y0 + dy), GOLD, "a size 2 star is a 5 px plus ({dx}, {dy})");
    }
    assert_eq!((px(&big, x0 + 3, y0), px(&big, x0 + 1, y0 + 1)), ([0; 4], [0; 4]), "no further, no square");
    let moved = e.moved(5.0, -3.0);
    assert!(matches!(moved.prim, Prim::Orbit(m) if m.at == (o.at.0 + 5.0, o.at.1 - 3.0)));
}

#[test]
fn a_ring_closes_when_r0_is_above_r1() {
    let mut fonts = Fonts::new(&font());
    let closing = Ring { r0: 9.0, r1: 4.0, ms: 1000, ..ring() };
    let right_edge = |c: &Canvas| (300..340).rev().find(|x| px(c, *x, 180)[3] > 0);
    let early = render(&mut fonts, |p| Effect::new(closing, 0).draw(p, 0));
    let late = render(&mut fonts, |p| Effect::new(closing, 0).draw(p, 800));
    assert!(right_edge(&early).unwrap() > right_edge(&late).unwrap(), "the ring shrinks over its life");
    let fixed = Ring { r0: 7.0, r1: 7.0, ..closing };
    let mid = render(&mut fonts, |p| Effect::new(closing, 0).draw(p, 400));
    assert!(mid.px == render(&mut fonts, |p| Effect::new(fixed, 0).draw(p, 400)).px, "at u 0.4 its radius is 9 - 5 x 0.4 = 7");
}

#[test]
fn e4_image_alpha_scales_every_pixels_alpha() {
    let img = Image { w: 2, h: 2, px: vec![200, 40, 40, 255, 40, 200, 40, 200, 40, 40, 200, 101, 9, 9, 9, 0] };
    let scaled = |a: u8| Image { px: img.px.chunks(4).flat_map(|p| [p[0], p[1], p[2], ((p[3] as u32 * a as u32 + 127) / 255) as u8]).collect(), ..img.clone() };
    let mut fonts = Fonts::new(&font());
    let on_grey = |fonts: &mut Fonts, f: &dyn Fn(&mut Painter)| {
        render(fonts, |p| {
            p.fill(Rect::new(0, 0, 40, 40), [60, 60, 60, 255]);
            f(p);
        })
    };
    let r = Rect::new(4, 4, 8, 8);
    let half = on_grey(&mut fonts, &|p| p.image_alpha(&img, r, 128));
    assert!(half.px == on_grey(&mut fonts, &|p| p.image(&scaled(128), r)).px, "128 halves every pixel's alpha");
    assert_eq!(scaled(128).px[3], 128, "a solid pixel at half alpha");
    assert!(half.px != on_grey(&mut fonts, &|p| p.image(&img, r)).px);
    assert!(on_grey(&mut fonts, &|p| p.image_alpha(&img, r, 255)).px == on_grey(&mut fonts, &|p| p.image(&img, r)).px, "255 is `image`");
    assert!(on_grey(&mut fonts, &|p| p.image_alpha(&img, r, 0)).px == on_grey(&mut fonts, &|_| {}).px, "0 draws nothing");
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

/// The stage's pixels (the page around it shows the clock, which moves).
fn stage_px(w: &[Widget<KitAct>], c: &Canvas) -> Vec<[u8; 4]> {
    let r = w.iter().find(|x| x.id == "kit.fx.stage").unwrap().rect;
    (r.y..r.y + r.h).flat_map(|y| (r.x..r.x + r.w).map(move |x| (x, y))).map(|(x, y)| px(c, x, y)).collect()
}

fn stage_text(w: &[Widget<KitAct>]) -> String {
    w.iter().find(|x| x.id == "kit.fx.stage").unwrap().text.clone()
}

#[test]
fn e5_the_effects_page_plays_an_orbit_and_loops_the_chosen_primitive() {
    let mut k = Specimens { page: 2, ..Specimens::default() };
    let (w, _, _) = draw_kit(&k);
    k.act(by_id(&w, "kit.fx.orbit").unwrap());
    assert!(k.last.starts_with("orbit: ") && k.fx.fx.kinds(k.fx.clock.now()) == vec!["orbit"], "a one-shot orbit: {}", k.last);
    k.fx.fx.clear();

    let (w, _, _) = draw_kit(&k);
    k.act(by_id(&w, "kit.fx.loop").unwrap());
    assert!(k.fx.looping && k.last == "loop orbit on", "{}", k.last);
    k.advance(300.0);
    let (w, looped, audit) = draw_kit(&k);
    assert_eq!(audit["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "the page reads clean with the loop: {audit}");
    assert!(stage_text(&w).contains("loop orbit"), "the stage names the loop");
    let mut unlooped = k.clone();
    unlooped.fx.looping = false;
    let (_, plain, _) = draw_kit(&unlooped);
    assert!(k.fx.fx.is_empty() && stage_px(&w, &looped) != stage_px(&w, &plain), "the loop draws outside the live list");
    let l = k.fx.looped();
    assert_eq!((l.kind(), l.period), ("orbit", l.effect.ms() + FX_LOOP_GAP_MS));

    k.act(KitAct::Effect("sparks"));
    assert!(k.fx.fx.is_empty() && k.fx.chosen == "sparks" && k.last.contains("looped every"), "while looping a press switches the loop: {}", k.last);
    let l = k.fx.looped();
    assert_eq!((k.fx.loop_start, l.kind(), l.period), (k.fx.clock.now(), "sparks", l.effect.ms() + FX_LOOP_GAP_MS));

    k.act(KitAct::Effect("orbit"));
    k.act(KitAct::Reduce);
    let (w, a, _) = draw_kit(&k);
    k.advance(370.0);
    let (_, b, _) = draw_kit(&k);
    assert!(stage_text(&w).contains("(still)") && stage_px(&w, &a) == stage_px(&w, &b), "reduce motion draws the still look, whatever the clock");
    k.act(KitAct::Loop);
    assert!(!k.fx.looping && k.last == "loop orbit off");
    let (_, off, _) = draw_kit(&k);
    assert!(stage_px(&w, &off) != stage_px(&w, &b), "off: no loop drawn");
}
