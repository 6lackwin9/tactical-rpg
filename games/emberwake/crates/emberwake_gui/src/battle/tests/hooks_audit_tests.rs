//! HOOKS1-UI slice 5: the text audit of every HOOKS1-UI addition at 1280 x 720 and the four text sizes (TEXTFIT1), and
//! the intel panel that fits its lines (decision "RULESET1-B2 tuning and HOOKS1-UI slice 3 answers"). One timeline
//! battle with every hook rule on and every slice's test rows: ground hazards and the arena (slice 1), meters, a held
//! guard and HP costs (slice 3), a planned push (slice 4).

use crate::battle::overlays::OVERLAY_GROUPS;
use crate::battle::BattleScreen;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::{Gfx, H, W};
use aegis_ui::art::Art;
use serde_json::{json, Value};
use emberwake_rules::ruleset::TIMELINE;

const ALL: &str = "+ground_hazards,+arena_hazards,+unit_meters,+triggers,+hp_cost,+forced_move";
const SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];

fn marked(view: &str) -> (Cx, Gfx, BattleScreen, usize, usize) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!(view);
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), ALL, false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    crate::battle::tests::hook_marks_tests::mark_up(&mut b);
    crate::battle::tests::meter_guard_tests::hook_up(&mut b);
    let (h, f) = crate::battle::tests::push_forecast_tests::plan_push(&cx, &mut b);
    let mut g = crate::hud::hud_tests::gfx();
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    (cx, g, b, h, f)
}

/// The problems of the screen's own texts: spills out of the strip, the cards, the tiles and the intel panel; cut
/// texts on the board and the side panels; texts off the canvas.
fn faults(b: &mut BattleScreen, cx: &mut Cx, g: &mut Gfx, scale: f32) -> Vec<Value> {
    let recs = crate::hud::hud_tests::frame(b, cx, g, scale);
    let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
    let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
    // spills: only out of the widgets the HOOKS1-UI marks live in or add lines to (the strip, the party and hostile
    // cards, the board tiles, the intel panel); other screens' widgets have their own audits
    let ours = |e: &Value| e["widget"].as_str().is_some_and(|w| ["strip.", "party.", "foe.", "tile.", "intel."].iter().any(|p| w.starts_with(p)));
    let mut out: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| ours(e)).cloned().collect();
    // cut by design and not HOOKS1-UI's (TEXTFIT1, RULESET1-U): a strip span's label (`strip.block<n>`), the bar's
    // status line (`bar.status`) and its ruleset chip (`bar.chip`); the cut text's widget is the smallest box
    // holding its start
    let widget_at = |x: f32, y: f32| boxes.iter().filter(|b| x >= b.1 && y >= b.2 && x < b.1 + b.3 && y < b.2 + b.4).min_by(|a, b| (a.3 * a.4).total_cmp(&(b.3 * b.4))).map(|b| b.0.as_str());
    let by_design = |e: &Value| {
        let (x, y) = (e["at"][0].as_f64().unwrap_or(-1.0) as f32 + 0.5, e["at"][1].as_f64().unwrap_or(-1.0) as f32 + 0.5);
        widget_at(x, y).is_some_and(|id| id == "bar.status" || id == "bar.chip" || id.starts_with("strip.block"))
    };
    out.extend(v["cut"].as_array().unwrap().iter().filter(|e| !by_design(e)).cloned());
    out.extend(v["edge"].as_array().unwrap().iter().cloned());
    out
}

/// TEXTFIT1: in both views, planning and played, at the four sizes, no HOOKS1-UI text spills, is cut or leaves the
/// canvas.
#[test]
fn every_hook_addition_fits_at_every_text_size() {
    for view in ["flat", "stage"] {
        let (mut cx, mut g, mut b, _, _) = marked(view);
        for played in [false, true] {
            if played {
                b.send(&cx, json!({"cmd": "resolve"}));
            }
            for scale in SIZES {
                let f = faults(&mut b, &mut cx, &mut g, scale);
                assert!(f.is_empty(), "{view} played {played} scale {scale}: {f:?}");
            }
        }
    }
}

/// The Settings Battle tab lists the overlay groups (and, in a debug build, VISCLAR1's Debug row) without a cut,
/// spilled, overlapping or off-canvas label at any size.
#[test]
fn the_overlay_settings_fit_at_every_text_size() {
    let mut cx = crate::hud::hud_tests::game_cx();
    let mut g = crate::hud::hud_tests::gfx();
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    let mut st = crate::menus::SettingsScreen::new(&cx);
    st.draw(&mut cx, &mut g, (-500, -500), true);
    st.click_id(&mut cx, "settings.tab.battle").unwrap();
    // SETTINGS-MOCK1: the overlay groups sit under the mock's battle rows; the body scrolls down to them
    g.kit.scroll.insert("settings.body".to_string(), i32::MAX / 2);
    for scale in SIZES {
        let recs = crate::hud::hud_tests::frame(&mut st, &mut cx, &mut g, scale);
        let boxes: Vec<(String, f32, f32, f32, f32)> = st.widgets().iter().map(|w| {
            let r = crate::hud::hud_tests::rect(w);
            (w["id"].as_str().unwrap_or("").to_string(), r.0 as f32, r.1 as f32, r.2 as f32, r.3 as f32)
        }).collect();
        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        // (INPUT1-PB: the playback speed select at the heading row's end too)
        let spill: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("settings.overlay.") || w.starts_with("settings.debug.") || w == "settings.playback_speed")).collect();
        assert!(spill.is_empty() && v["cut"].as_array().unwrap().is_empty() && v["edge"].as_array().unwrap().is_empty(), "scale {scale}: {v}");
        // VISCLAR1-A: the tighter rows (24 groups and the Debug row) do not overlap
        assert_eq!(v["counts"]["overlap"], json!(0), "scale {scale}: {}", v["overlap"]);
        let ids = st.widgets().iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("settings.overlay."))).count();
        assert_eq!(ids, OVERLAY_GROUPS.len());
        let debug = st.widgets().iter().filter(|w| w["id"] == json!("settings.debug.party_foresight_off")).count();
        assert_eq!(debug, usize::from(cfg!(debug_assertions)), "VC5: the debug row only in a debug build");
        assert!(st.widgets().iter().any(|w| w["id"] == json!("settings.playback_speed")), "PB3: the playback speed");
    }
}

