//! PORTRAITFX1-TL (docs/specs/PORTRAITFX1.md 6.2 row 4, 6.5, 4.6; PORTRAITFX1_TASKS.md TL2): the full timeline view's
//! track icons: every unit's track leads with its portrait in the widget's icon square, in the minimal tier however
//! large the window (flash, tints, fade; no jolt, no breath), fading for a stealthed unit and flashing at a struck
//! unit's impact. Cue rows are this module's fixtures, portraits scratch layers.

use crate::battle::{Action, BattleScreen, TIMELINE_SURFACE};
use aegis_ui::canvas::Rect;
use crate::portraits::Faces;
use crate::timeline_view;
use aegis_ui::fx::Clock;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::{Gfx, H, W};
use aegis_ui::art::Art;
use serde_json::{json, Value};
use crate::portraits::tests::{layer, row};
use aegis_core::assets::Layers;
use aegis_ui::portrait_fx::{Tier, EASE_MS};
use aegis_ui::timeline::ROW_H;
use emberwake_data::PortraitCue;
use emberwake_rules::ruleset::TIMELINE;

/// The fixture rows: a blow's flash and jolt on every surface, stealth's fade.
fn rows() -> Vec<PortraitCue> {
    vec![
        row(json!({"id": "hit_flash", "event": "hit", "intensity": 0.6, "ms": 140, "color": "#d9cbb4"})),
        row(json!({"id": "hit_jolt", "event": "hit", "effect": "jolt", "preset": "hit"})),
        row(json!({"id": "crit_flash", "event": "crit", "intensity": 0.9, "ms": 180, "color": "#e11d48"})),
        row(json!({"id": "crit_jolt", "event": "crit", "effect": "jolt", "preset": "crit"})),
        row(json!({"id": "guard_flash", "event": "guard_hit", "intensity": 0.4, "ms": 120, "color": "#fbbf24"})),
        row(json!({"id": "heal_glow", "event": "heal", "intensity": 0.3, "ms": 420, "color": "#fbbf24"})),
        row(json!({"id": "stealthed", "trigger": "status", "status": "stealth", "effect": "fade", "intensity": 0.6})),
    ]
}

/// The flash colour (rgb) the fixture rows give `event` on the timeline surface.
fn flash_of(event: &str) -> [u8; 3] {
    let rows = rows();
    let r = emberwake_battle::portrait::reactions(&rows, &[event], TIMELINE_SURFACE, TIMELINE).into_iter().find(|r| r.effect == "flash").expect("a flash row").clone();
    let c = aegis_core::assets::hex_colour(&r.color).expect("a colour");
    [c[0], c[1], c[2]]
}

/// The first catalog encounter under the timeline ruleset on the stage view (its steps wait for their impacts),
/// the fixture rows, a scratch portrait for every unit (layers named after `tag`: one per test, as they run side
/// by side), the timeline view open, drawn once.
fn battle(tag: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = rows());
    cx.settings.v["battle_view"] = json!("stage");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let n = b.session().w.units.len();
    let layers: Vec<_> = (0..n).map(|u| layer(&format!("tl_{tag}_{u}"), &b.unit_portrait(&cx, u).0.chain("")[0], 32, [180, 90, 40], None)).collect();
    cx.art = Art::layered(Layers::of(layers));
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    b.tv_open = true;
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(b.timeline_view_open(), "the timeline view opens");
    (cx, g, b)
}

fn widget<'a>(b: &'a BattleScreen, id: &str) -> Option<&'a aegis_ui::ui::Widget<Action>> {
    b.w.iter().find(|w| w.id == id)
}

/// TL2 placement (6.5, TEXTFIT1): at the four text sizes every unit's track has its icon square (row height,
/// left of its label, on its row), the portrait kept for it is in the minimal tier, and no timeline text spills or
/// is cut without its full text on hover.
#[test]
fn tl2_every_track_leads_with_its_portrait_in_the_minimal_tier() {
    let (mut cx, mut g, mut b) = battle("place");
    let n = b.session().w.units.len();
    for (_, scale) in aegis_ui::screen::TEXT_SIZES {
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale as f32);
        let view = b.tv_shown.clone().expect("the view is drawn");
        let mut seen = 0;
        for t in view.sections.iter().flat_map(|s| &s.tracks) {
            let u = timeline_view::unit_of(&t.key).expect("a unit's track");
            assert!(t.icon, "{scale}: {} has an icon square", t.key);
            let (Some(sq), Some(label)) = (widget(&b, &format!("timeline.icon.{}", t.key)), widget(&b, &format!("timeline.track.{}", t.key))) else { panic!("{scale}: {}'s square and label", t.key) };
            let (sq, label) = (sq.rect, label.rect);
            if sq.h < ROW_H {
                continue; // scrolled partly out of the rows' view (its widget is cut to it)
            }
            assert_eq!((sq.w, sq.h, sq.y), (ROW_H, ROW_H, label.y - 1), "{scale}: {}'s square is the row's height, on its row", t.key);
            assert!(sq.x + sq.w <= label.x, "{scale}: {}'s square is left of its label", t.key);
            let key = b.unit_portrait(&cx, u).0.key;
            let f = b.timeline_faces.get(&key).unwrap_or_else(|| panic!("{scale}: {key} drawn on the timeline"));
            assert_eq!(f.tier(), Some(Tier::Minimal), "{scale}: {key}");
            seen += 1;
        }
        assert!(seen > 0 && seen <= n, "{scale}: {seen} squares in view");
        let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        let spills: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("timeline."))).collect();
        assert!(spills.is_empty(), "{scale}: {spills:?}");
        for c in v["cut"].as_array().unwrap() {
            let (x, y) = (c["at"][0].as_f64().unwrap() as i32 + 1, c["at"][1].as_f64().unwrap() as i32 + 2);
            let full = c["text"].as_str().unwrap();
            let Some(wd) = b.w.iter().rev().find(|w| w.rect.contains(x, y) && w.id.starts_with("timeline.") && w.id != "timeline.backdrop") else { continue };
            assert!(wd.text == full || wd.tip.iter().any(|l| l.0.contains(full)), "{scale}: {full:?} cut on {} without its full text on hover", wd.id);
        }
    }
    assert!(b.state(&cx)["timeline_portraits"].as_array().is_some_and(|a| !a.is_empty()), "the agent link sees the timeline portraits");
}

/// TL2 tier (4.6): on a window four times the canvas the 15 px square is 60 device px (reduced by its side) and
/// still draws minimal, while the 32 px panel portrait is full: a blow flashes the row in its row's colour with no
/// jolt and no breath; the panel jolts.
#[test]
fn tl2_a_blow_flashes_the_row_without_a_jolt_at_any_window_size() {
    let (mut cx, _g, mut b) = battle("tier");
    let mut big = crate::hud::hud_tests::gfx();
    big.canvas = aegis_ui::canvas::Canvas::new(W * 4, H * 4);
    big.view = aegis_ui::painter::View::fit((W * 4) as u32, (H * 4) as u32);
    assert_eq!(Tier::of(big.view.rect(Rect::new(0, 0, ROW_H, ROW_H)).w), Tier::Reduced, "the square's side alone would be reduced");
    let u = b.session().w.allies[0];
    b.selected = Some(u);
    let key = b.unit_portrait(&cx, u).0.key;
    b.clock = Clock { ms: 1000.0, hold: 0.0 };
    b.react_portraits(&cx, &[(u, "hit")], 1000);
    b.clock.advance(40.0, 1.0);
    b.draw(&mut cx, &mut big, (-500, -500), true);
    let f = b.timeline_faces.get(&key).expect("the row's portrait");
    let now = b.timeline_faces.now();
    assert_eq!(f.tier(), Some(Tier::Minimal));
    let flash = f.level("flash", now).expect("the row flashes");
    assert_eq!((flash.start, flash.color[..3].to_vec()), (1000, flash_of("hit").to_vec()));
    assert_eq!(f.jolt_offset(now), (0, 0), "no jolt in the row");
    assert!(!f.live(now).contains(&"breath"), "no breath in the row: {:?}", f.live(now));
    let panel = b.panel_faces.get(&key).expect("the hero panel's portrait");
    assert_eq!(panel.tier(), Some(Tier::Full), "the panel portrait is uncapped");
    assert!(panel.jolt_amplitude(now) > 0.0, "the panel jolts for the same blow");
}

/// TL2 impact (FX1 rule I, 5.3): a step's portrait events wait in its impact; at the impact each struck unit's row
/// flashes from the impact's ms in its event's colour.
#[test]
fn tl2_a_struck_units_row_flashes_at_the_impact_not_at_the_pop() {
    let (mut cx, _g, mut b) = battle("impact");
    let found = crate::battle::tests::step_until(&mut cx, &mut b, |_, i| i.portrait.iter().any(|(_, e)| ["hit", "crit", "guard_hit"].contains(e)));
    assert!(found, "the battle has a blow at an impact");
    b.resolve_impacts(&mut cx);
    b.timeline_faces = Faces::capped(Tier::Minimal);
    let i = b.impacts[0].clone();
    let at = i.at.expect("resolved");
    // the blows and heals (PORTRAITFX2: a step may carry `ally_down` too, which no flash row answers)
    let keys: Vec<(String, &str)> = i.portrait.iter().filter(|(_, e)| ["hit", "crit", "guard_hit", "heal"].contains(e)).map(|&(u, e)| (b.unit_portrait(&cx, u).0.key, e)).collect();
    assert!(keys.iter().all(|(k, _)| b.timeline_faces.get(k).is_none()), "nothing reacts at the pop");
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    for (k, ev) in &keys {
        let f = b.timeline_faces.get(k).unwrap_or_else(|| panic!("{k}'s row reacted to {ev}"));
        let flash = f.level("flash", at).unwrap_or_else(|| panic!("{k}: a flash for {ev}"));
        assert_eq!((flash.start, flash.color[..3].to_vec()), (at, flash_of(ev).to_vec()), "{k}: {ev}'s flash from the impact");
    }
    assert_eq!(b.timeline_faces.clock, b.clock, "the timeline clock is the FX clock again after a reaction");
}

/// TL2 fade (4.2, 5.2): a stealthed unit's row draws at alpha 0.58 (`fade` 0.6) once its hold has eased in.
#[test]
fn tl2_a_stealthed_units_row_fades() {
    let (mut cx, mut g, mut b) = battle("fade");
    let u = b.session().w.allies[0];
    let key = b.unit_portrait(&cx, u).0.key;
    let centre = |b: &BattleScreen| {
        let img = b.timeline_faces.get(&key).and_then(|f| f.composed()).expect("composed").clone();
        let i = ((img.h / 2 * img.w + img.w / 2) * 4) as usize;
        img.px[i + 3]
    };
    b.clock = Clock { ms: 1000.0, hold: 0.0 };
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!((b.timeline_faces.get(&key).and_then(|f| f.level("fade", 1000)).map(|l| l.i), centre(&b)), (None, 255), "no state: no fade");
    b.host.session.as_mut().unwrap().w.units[u].st.active.insert("stealth".into());
    for ms in [2000, 2000 + EASE_MS + 100] {
        b.clock = Clock { ms: ms as f64, hold: 0.0 };
        b.draw(&mut cx, &mut g, (-500, -500), true);
    }
    let now = b.timeline_faces.now();
    assert!(b.timeline_faces.get(&key).and_then(|f| f.level("fade", now)).is_some_and(|l| (l.i - 0.6).abs() < 1e-3));
    let want = (255.0 * (1.0 - 0.7 * 0.6f32)).round() as i32;
    assert!((centre(&b) as i32 - want).abs() <= 1, "stealth draws the row at alpha 0.58: {} vs {want}", centre(&b));
}

/// TL2 enemy rows (user 2026-11-01): a stealthed enemy's row stays plain (no fade) while a stealthed party member's
/// fades; an enemy's row reacts only to a blow the board shows on it (no heal, no unshown blow), a party row to
/// every event; no row reacts while the view is closed.
#[test]
fn tl2_a_stealthed_enemys_row_is_plain_while_a_stealthed_members_fades() {
    let (mut cx, mut g, mut b) = battle("enemy");
    let (a, e) = (b.session().w.allies[0], b.session().w.enemies[0]);
    let (ka, ke) = (b.unit_portrait(&cx, a).0.key, b.unit_portrait(&cx, e).0.key);
    for u in [a, e] {
        b.host.session.as_mut().unwrap().w.units[u].st.active.insert("stealth".into());
    }
    for ms in [2000, 2000 + EASE_MS + 100] {
        b.clock = Clock { ms: ms as f64, hold: 0.0 };
        b.draw(&mut cx, &mut g, (-500, -500), true);
    }
    let now = b.timeline_faces.now();
    let level = |b: &BattleScreen, k: &str, eff: &str, t: u64| b.timeline_faces.get(k).and_then(|f| f.level(eff, t)).map(|l| l.i);
    assert!(level(&b, &ka, "fade", now).is_some_and(|i| (i - 0.6).abs() < 1e-3), "the stealthed member's row fades");
    assert_eq!(level(&b, &ke, "fade", now), None, "the stealthed enemy's row stays plain");
    assert!(b.timeline_faces.get(&ke).is_some_and(|f| f.composed().is_some()), "the enemy's row is drawn");

    b.react_portraits_shown(&cx, &[(e, "hit"), (a, "heal")], 5000, &[]);
    assert_eq!(level(&b, &ke, "flash", 5000), None, "a blow the board does not show: no flash on the enemy's row");
    assert!(level(&b, &ka, "flash", 5000).is_some(), "a party row reacts to every event");
    b.react_portraits_shown(&cx, &[(e, "heal")], 6000, &[e]);
    assert_eq!(level(&b, &ke, "flash", 6000), None, "an enemy's row reacts to blows only");
    b.react_portraits_shown(&cx, &[(e, "hit")], 7000, &[e]);
    assert!(level(&b, &ke, "flash", 7000).is_some_and(|_| b.timeline_faces.get(&ke).unwrap().level("flash", 7000).unwrap().color[..3] == flash_of("hit")), "a shown blow flashes the enemy's row");

    b.tv_open = false;
    b.draw(&mut cx, &mut g, (-500, -500), true);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    b.react_portraits_shown(&cx, &[(a, "hit")], 8000, &[a]);
    assert!(b.timeline_faces.get(&ka).is_none() && b.panel_faces.get(&ka).is_some(), "the view closed: the rows do not react, the panel does");
    assert!(b.state(&cx)["timeline_portraits"].as_array().is_some_and(|v| v.is_empty()), "and the agent state lists none");
}
