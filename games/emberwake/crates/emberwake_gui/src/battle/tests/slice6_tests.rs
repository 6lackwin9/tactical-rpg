//! RULESET1-U slice 6: the overlay groups on the Settings Battle tab, the round summary in the combat log, and the
//! battle screen's text at the four text sizes (TEXTFIT1: no strip label or hero panel line spills, no intent button
//! label is cut). Expectations come from `OVERLAY_GROUPS`, the kit's `summary_line` and the text audit.

use crate::battle::overlays::{Layers, OVERLAY_GROUPS};
use crate::battle::BattleScreen;
use crate::battle::tokens::Float;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::{Gfx, H, W};
use emberwake_battle::round_intent::{self as ri};
use aegis_ui::art::Art;
use serde_json::{json, Value};
use std::time::Instant;
use emberwake_rules::ruleset::TIMELINE;

fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

/// The preference (user 2026-10-30): the Settings Battle tab lists every overlay group, at its default; a click
/// switches one off in the draft (Apply saves it under `battle_overlays`, which the battle screen reads).
#[test]
fn the_settings_list_every_overlay_group() {
    let mut cx = crate::hud::hud_tests::game_cx();
    let mut g = crate::hud::hud_tests::gfx();
    let defaults = crate::screens::default_settings();
    assert!(OVERLAY_GROUPS.iter().all(|(id, _, _, on)| defaults["battle_overlays"][*id] == json!(on)), "each at its default");
    assert_eq!(Layers::of(&defaults["battle_overlays"]), Layers::of(&Value::Null));
    let mut st = crate::menus::SettingsScreen::new(&cx);
    st.draw(&mut cx, &mut g, (-500, -500), true);
    st.click_id(&mut cx, "settings.tab.battle").unwrap();
    st.draw(&mut cx, &mut g, (-500, -500), true);
    let ids: Vec<String> = st.widgets().iter().filter_map(|w| w["id"].as_str().filter(|i| i.starts_with("settings.overlay.")).map(String::from)).collect();
    assert_eq!(ids, OVERLAY_GROUPS.iter().map(|g| format!("settings.overlay.{}", g.0)).collect::<Vec<_>>());
    st.click_id(&mut cx, "settings.overlay.band").unwrap();
    let draft = st.state(&cx)["draft"]["battle_overlays"].clone();
    assert_eq!(draft["band"], json!(false));
    assert!(!Layers::of(&draft).band && Layers::of(&draft).front, "the battle screen's reading of it");
}

/// Decision "RULESET1-U slice 5 answers": a timeline round closes the combat log with its `[Round]` summary line;
/// a classic round logs none.
#[test]
fn the_round_summary_closes_the_combat_log() {
    for (rules, logged) in [(Some(TIMELINE), true), (None, false)] {
        let (cx, _g, mut b) = arena(rules);
        let r = b.send(&cx, json!({"cmd": "resolve"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        let s = b.host.session.as_ref().unwrap();
        let lines: Vec<&String> = s.resolver.log.iter().filter(|l| l.starts_with("[Round]")).collect();
        if logged {
            assert_eq!(lines, [&ri::summary_line(1, &ri::round_summary(&s.w, &s.resolver.scheduled))]);
        } else {
            assert!(lines.is_empty(), "classic: {lines:?}");
        }
    }
}

/// TEXTFIT1 at the four text sizes, 1280x720: in a planned and a played timeline round no strip label spills past
/// its widget, no hero panel line spills, and no intent button label is cut.
#[test]
fn the_timeline_screen_fits_at_every_text_size() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    let h = b.selected.unwrap();
    let foe = b.host.session.as_ref().unwrap().w.enemies[0];
    b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
    for played in [false, true] {
        if played {
            b.send(&cx, json!({"cmd": "resolve"}));
        }
        for scale in [0.85f32, 1.0, 1.15, 1.3] {
            let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let ours = |k: &str| v[k].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("strip.") || w.starts_with("hero."))).cloned().collect::<Vec<_>>();
            assert!(ours("spill").is_empty(), "played {played} scale {scale}: {:?}", ours("spill"));
            let cut: Vec<&Value> = v["cut"].as_array().unwrap().iter().filter(|e| ["Assault", "Guard", "Support", "Maneuver"].contains(&e["text"].as_str().unwrap_or(""))).collect();
            assert!(cut.is_empty(), "played {played} scale {scale}: {cut:?}");
        }
    }
}

/// TEXTFIT2 (decision "RULESET1-U slice 6 answers"): a played round's frame with a damage or heal float over every
/// living unit, on the stage and on the flat board, audits clean of the floats at the four text sizes: every float
/// is recorded as transient, no issue names one, and the frame reports what it reports without them.
#[test]
fn a_playback_frame_with_floats_audits_clean_at_every_text_size() {
    for view in ["stage", "flat"] {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        cx.settings.v["battle_view"] = json!(view);
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        let r = b.send(&cx, json!({"cmd": "resolve"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        let living: Vec<usize> = b.host.session.as_ref().unwrap().w.units.iter().enumerate().filter(|(_, u)| u.hp > 0).map(|(i, _)| i).collect();
        assert!(!living.is_empty());
        let (damage, heal) = (cx.tuning.value.damage_color, cx.tuning.value.heal_color);
        let texts: Vec<String> = living.iter().map(|i| if i % 2 == 0 { format!("-{}", 10 + i) } else { format!("+{}", 10 + i) }).collect();
        for scale in [0.85f32, 1.0, 1.15, 1.3] {
            let audit = |b: &mut BattleScreen, cx: &mut Cx, g: &mut Gfx, floats: bool| {
                b.floats.clear();
                if floats {
                    let now = Instant::now();
                    b.floats.extend(living.iter().zip(&texts).map(|(&unit, text)| Float { unit, text: text.clone(), color: if text.starts_with('-') { damage } else { heal }, born: now, ms: cx.tuning.value.float_ms }));
                }
                let recs = crate::hud::hud_tests::frame(b, cx, g, scale);
                let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                (recs, v)
            };
            let (_, bare) = audit(&mut b, &mut cx, &mut g, false);
            let (recs, v) = audit(&mut b, &mut cx, &mut g, true);
            for text in &texts {
                let drawn: Vec<bool> = recs.iter().filter(|r| &r.text == text).map(|r| r.transient).collect();
                assert!(!drawn.is_empty() && drawn.iter().all(|&x| x), "{view} scale {scale}: float {text} drawn transient: {drawn:?}");
            }
            let named = |e: &Value| ["text", "shown", "a", "b"].iter().any(|k| e[*k].as_str().is_some_and(|s| texts.iter().any(|t| t == s)));
            for kind in ["cut", "edge", "overlap", "spill"] {
                let hits: Vec<&Value> = v[kind].as_array().unwrap().iter().filter(|e| named(e)).collect();
                assert!(hits.is_empty(), "{view} scale {scale}: {kind} {hits:?}");
            }
            assert_eq!(v["counts"], bare["counts"], "{view} scale {scale}: the floats add no issue");
        }
    }
}
