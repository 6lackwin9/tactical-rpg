//! ASSETORG4 section 2.2 (FB-227): the portrait overlay FX (`glow`, `lines`, `point_glow`, `particles_rise`, `drift`)
//! are the opt-in layer `rpgkit_ui::portrait_overlays`. Unmounted (the default `PortraitOpts`), a portrait holding every
//! overlay composes exactly what it composes holding none: the base life (breath, rim, blink), states, reactions and
//! expressions still draw, and `live` lists no overlay. Mounted, the overlays draw. `Mount::resolve` takes the game's
//! switch and the player's setting (`SETTING`, on when missing). Images are built here (the engine's sample face).

use aegis_core::assets::Face;
use aegis_ui::canvas::{Canvas, Image, Rect};
use aegis_ui::painter::{Fonts, Painter, View};
use rpgkit_ui::portrait_fx::{idle_cues, sample, Budget, Cue, ExpressionFx, PortraitFx, PortraitOpts, Tier, BASE_LIFE};
use rpgkit_ui::portrait_overlays::{is_overlay, player_on, Mount, OVERLAYS, SETTING};
use aegis_ui::screen::Settings;
use serde_json::json;
use std::path::Path;

fn fonts() -> Fonts {
    Fonts::new(&std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap())
}

/// The composed image and live list of a fresh portrait holding `cues` from 0 (and the sample face's sidecar overlays
/// when `idle`), drawn at 96 px (the full tier) at `now` (held cues ease in over `EASE_MS`) with `opts`.
fn draw(cues: &[Cue], idle: bool, now: u64, opts: PortraitOpts) -> (Image, Vec<&'static str>) {
    let (src, face) = sample();
    let mut fx = PortraitFx::new("overlay_check");
    let mut held: Vec<Cue> = if idle { idle_cues(&face) } else { Vec::new() };
    held.extend(cues.iter().cloned());
    fx.hold(&held, 0);
    let mut c = Canvas::new(640, 360);
    let mut f = fonts();
    let mut p = Painter { c: &mut c, v: View::fit(640, 360), fonts: &mut f };
    fx.draw_image(&mut p, &src, "sample", Some(&face), Rect::new(8, 8, 96, 96), now, &opts, &mut Budget::default());
    (fx.composed().expect("composed").clone(), fx.live(now))
}

/// Every overlay held at full strength (the Kit widgets page's toggles).
fn every_overlay() -> Vec<Cue> {
    OVERLAYS.iter().map(|e| Cue::parse(e, "").unwrap().with_key(&format!("check.{e}"))).collect()
}

fn mounted() -> PortraitOpts {
    PortraitOpts::default().with_overlays(Mount::Mounted)
}

#[test]
fn the_overlays_are_the_sidecar_overlays_and_the_default_is_unmounted() {
    assert_eq!(OVERLAYS, aegis_core::assets::IDLE_OVERLAYS);
    for e in OVERLAYS {
        assert!(is_overlay(e) && Cue::parse(e, "").is_some(), "{e} is an overlay cue");
    }
    for e in BASE_LIFE.iter().chain(["flash", "jolt", "desaturate", "tint", "fade", "still", "face"].iter()) {
        assert!(!is_overlay(e), "{e} is not an overlay");
    }
    assert_eq!(PortraitOpts::default().overlays, Mount::Unmounted, "opt-in: unmounted unless a caller mounts it");
    assert_eq!(Mount::default(), Mount::Unmounted);
    assert!(Mount::of(true).is_mounted() && !Mount::of(false).is_mounted());
}

#[test]
fn unmounted_a_portrait_holding_every_overlay_draws_exactly_its_base_portrait() {
    let (_, face) = sample();
    assert!(!face.idle.is_empty() || !face.glow.is_empty(), "the sample face has anchors for the overlays");
    for now in [300, 700, 2400, 5000] {
        let base = draw(&[], false, now, PortraitOpts::default());
        let unmounted = draw(&every_overlay(), true, now, PortraitOpts::default());
        assert_eq!(unmounted.0, base.0, "t {now}: no overlay pixel is composed while unmounted");
        assert_eq!(unmounted.1, base.1, "t {now}: the same live list");
        assert!(unmounted.1.iter().all(|e| !is_overlay(e)), "t {now}: {:?}", unmounted.1);
        let on = draw(&every_overlay(), true, now, mounted());
        assert!(OVERLAYS.iter().all(|e| on.1.contains(e)), "t {now}: mounted, every overlay is live: {:?}", on.1);
        assert_ne!(on.0, base.0, "t {now}: mounted, the overlays draw");
    }
    // the base life still runs unmounted: the breath and blink of a full-tier portrait
    let live = draw(&every_overlay(), true, 300, PortraitOpts::default()).1;
    assert!(live.contains(&"breath") && live.contains(&"blink"), "{live:?}");
}

#[test]
fn unmounted_states_reactions_and_expressions_still_draw() {
    let tint = Cue::parse("tint", "").unwrap().with_intensity(0.6);
    let face = Cue::parse("face", "").unwrap().with_expression("angry").with_key("check.face");
    let with = |cues: Vec<Cue>, opts: PortraitOpts| {
        let (src, sface) = sample();
        let mut fx = PortraitFx::new("overlay_check");
        fx.set_expressions(&[ExpressionFx { tint: Some([200, 30, 30, 255]), tint_strength: 0.6, ..ExpressionFx::new("angry") }]);
        fx.hold(&cues, 0);
        let mut c = Canvas::new(640, 360);
        let mut f = fonts();
        let mut p = Painter { c: &mut c, v: View::fit(640, 360), fonts: &mut f };
        fx.draw_image(&mut p, &src, "sample", Some(&sface), Rect::new(8, 8, 96, 96), 1000, &opts, &mut Budget::default());
        (fx.composed().unwrap().clone(), fx.live(1000), fx.expression(1000).to_string(), fx.state(1000))
    };
    let mut cues = every_overlay();
    cues.extend([tint.clone(), face.clone()]);
    let off = with(cues, PortraitOpts::default());
    let base = with(vec![tint, face], PortraitOpts::default());
    assert_eq!(off.0, base.0, "the tint and the expression's tint draw; the overlays do not");
    assert_eq!(off.2, "angry", "the expression swap is kept");
    assert!(off.1.contains(&"tint") && off.1.contains(&"face"), "{:?}", off.1);
    assert!(off.1.iter().all(|e| !is_overlay(e)), "{:?}", off.1);
    assert_eq!(off.3["expression_fx"], json!(["tint"]), "{}", off.3);
}

#[test]
fn unmounted_overlays_draw_at_no_tier_and_never_on_the_minimal_core() {
    for cap in [Tier::Full, Tier::Reduced, Tier::Minimal] {
        let opts = PortraitOpts { tier_cap: Some(cap), ..PortraitOpts::default() };
        let base = draw(&[], false, 1200, opts);
        let off = draw(&every_overlay(), true, 1200, opts);
        assert_eq!(off.0, base.0, "{cap:?}");
        let on = draw(&every_overlay(), true, 1200, opts.with_overlays(Mount::Mounted));
        assert!(on.1.contains(&"point_glow"), "{cap:?}: the point glow's core draws at every tier when mounted: {:?}", on.1);
    }
}

#[test]
fn the_game_switch_and_the_player_setting_resolve_the_mount() {
    let mut s = Settings::load(std::env::temp_dir().join("aegis_portrait_overlays_no_settings.json"), json!({}));
    assert!(player_on(&s), "missing: on");
    assert_eq!(Mount::resolve(false, &s), Mount::Unmounted, "a game that does not mount the layer draws none");
    assert_eq!(Mount::resolve(true, &s), Mount::Mounted);
    s.v[SETTING] = json!(false);
    assert!(!player_on(&s));
    assert_eq!(Mount::resolve(true, &s), Mount::Unmounted, "the player turned it off");
    s.v[SETTING] = json!("yes");
    assert_eq!(Mount::resolve(true, &s), Mount::Mounted, "not a boolean: the default");
    assert_eq!(SETTING, "portrait_overlays");
}

#[test]
fn a_sidecar_overlay_is_held_and_returns_when_mounted_again() {
    let (src, face): (Image, Face) = sample();
    let mut fx = PortraitFx::new("overlay_check");
    fx.hold(&every_overlay(), 0);
    let mut c = Canvas::new(640, 360);
    let mut f = fonts();
    let mut p = Painter { c: &mut c, v: View::fit(640, 360), fonts: &mut f };
    let mut b = Budget::default();
    fx.draw_image(&mut p, &src, "sample", Some(&face), Rect::new(8, 8, 96, 96), 1000, &PortraitOpts::default(), &mut b);
    assert!(fx.live(1000).iter().all(|e| !is_overlay(e)));
    fx.draw_image(&mut p, &src, "sample", Some(&face), Rect::new(8, 8, 96, 96), 1000, &mounted(), &mut b);
    assert!(OVERLAYS.iter().all(|e| fx.live(1000).contains(e)), "the holds were kept: {:?}", fx.live(1000));
}
