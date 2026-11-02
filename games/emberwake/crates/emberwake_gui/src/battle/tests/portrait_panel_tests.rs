//! PORTRAITFX1-P (docs/specs/PORTRAITFX1.md 6.2 row 3, PORTRAITFX1_TASKS.md P1-P5): the hero and intel panels'
//! portraits, their state holds and the step events at FX1's impacts. Cue rows are this module's fixtures (the shipped
//! table's shape), portraits scratch layers where pixels are read.

use crate::battle::{Action, BattleScreen, PANEL_INSET, PANEL_PORTRAIT, PANEL_SURFACE, PANEL_TEXT_X};
use aegis_ui::canvas::Rect;
use crate::portraits::Faces;
use aegis_ui::fx::Clock;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::{Gfx, Ui, H, W};
use emberwake_battle::session::Phase;
use aegis_ui::art::Art;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use crate::portraits::tests::{layer, row};
use aegis_core::assets::Layers;
use aegis_ui::portrait_fx::{PortraitOpts, EASE_MS};
use emberwake_data::PortraitCue;
use emberwake_rules::ruleset::TIMELINE;

/// The fixture cue rows: the reactions of 5.3 and the holds P3 names (stealth, low HP, down).
fn rows() -> Vec<PortraitCue> {
    vec![
        row(json!({"id": "hit_flash", "event": "hit", "intensity": 0.6, "ms": 140, "color": "#d9cbb4"})),
        row(json!({"id": "hit_jolt", "event": "hit", "effect": "jolt", "preset": "hit"})),
        row(json!({"id": "crit_flash", "event": "crit", "intensity": 0.9, "ms": 180, "color": "#e11d48"})),
        row(json!({"id": "crit_jolt", "event": "crit", "effect": "jolt", "preset": "crit"})),
        row(json!({"id": "guard_flash", "event": "guard_hit", "intensity": 0.4, "ms": 120, "color": "#fbbf24"})),
        row(json!({"id": "guard_jolt", "event": "guard_hit", "effect": "jolt", "intensity": 0.5, "preset": "hit"})),
        row(json!({"id": "heal_glow", "event": "heal", "intensity": 0.3, "ms": 420, "color": "#fbbf24"})),
        row(json!({"id": "stealthed", "trigger": "status", "status": "stealth", "effect": "fade", "intensity": 0.6})),
        row(json!({"id": "hp_low", "trigger": "hp", "below": 25, "effect": "desaturate", "intensity": 0.5})),
        row(json!({"id": "downed", "trigger": "down", "effect": "desaturate"})),
        row(json!({"id": "downed_still", "trigger": "down", "effect": "still"})),
    ]
}

/// The game's cx with the fixture rows and the repo's art.
fn cx_with(rows: Vec<PortraitCue>) -> Cx {
    let mut cx = crate::hud::hud_tests::game_cx();
    std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = rows);
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx
}

/// The ambush (FX1's impact tests' battle) with the fixture rows.
fn ambush() -> (Cx, Gfx, BattleScreen) {
    let mut cx = cx_with(rows());
    cx.app.encounter = "goblin_ambush".into();
    cx.app.seed = Some(1);
    cx.settings.v["battle_view"] = json!("stage");
    let b = BattleScreen::sandbox(&cx);
    assert!(b.host.session.is_some(), "the ambush starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

/// An Arena battle of the first catalog encounter under `ruleset` (None: classic), ticked and drawn, a hero selected.
fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
    let mut cx = cx_with(rows());
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

fn rect_of(b: &BattleScreen, id: &str) -> Rect {
    b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}")).rect
}

/// The flash colour (rgb) the fixture rows give `event` on the panel.
fn flash_of(event: &str) -> [u8; 3] {
    let rows = rows();
    let r = emberwake_battle::portrait::reactions(&rows, &[event], PANEL_SURFACE, "classic").into_iter().find(|r| r.effect == "flash").expect("a flash row").clone();
    let c = aegis_core::assets::hex_colour(&r.color).expect("a colour");
    [c[0], c[1], c[2]]
}

/// P1 (6.2 row 3, TEXTFIT1): at the four text sizes, classic and timeline, both panels draw a 32 px portrait at
/// their top-left (inset 4); every text beside it starts 40 px in, none crosses it, and no text of either panel
/// spills or leaves the canvas.
#[test]
fn p1_the_panels_draw_their_portraits_and_the_lines_beside_start_40_in() {
    for ruleset in [None, Some(TIMELINE)] {
        let (mut cx, mut g, mut b) = arena(ruleset);
        for (_, scale) in aegis_ui::screen::TEXT_SIZES {
            let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale as f32);
            for (panel, portrait) in [("hero", "hero.portrait"), ("intel", "intel.portrait")] {
                let r = b.panels.iter().find(|p| p.0 == panel).unwrap_or_else(|| panic!("{ruleset:?}: the {panel} panel")).1;
                let pr = rect_of(&b, portrait);
                assert_eq!(pr, Rect::new(r.x + PANEL_INSET, r.y + PANEL_INSET, PANEL_PORTRAIT, PANEL_PORTRAIT), "{ruleset:?} {scale}: the {panel} portrait");
                for t in recs.iter().filter(|t| !t.covered && t.x >= r.x as f32 && t.x < (r.x + r.w) as f32 && t.y >= r.y as f32 && t.y < (r.y + r.h) as f32) {
                    let inside = t.x >= pr.x as f32 && t.x + t.w <= (pr.x + pr.w) as f32 && t.y >= pr.y as f32 && t.y + t.h <= (pr.y + pr.h) as f32;
                    if inside {
                        assert_eq!(t.text.chars().count(), 1, "{ruleset:?} {scale}: only the initial letter inside the portrait: {:?}", t.text);
                        continue;
                    }
                    let rows_beside = t.y < (pr.y + pr.h) as f32 && t.y + t.h > pr.y as f32;
                    if rows_beside {
                        assert!(t.x >= (r.x + PANEL_TEXT_X) as f32 - 0.5, "{ruleset:?} {scale}: {:?} beside the {panel} portrait starts 40 px in (x {})", t.text, t.x);
                    }
                }
            }
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let ours = |e: &Value| e["widget"].as_str().is_some_and(|w| w.starts_with("hero.") || w.starts_with("intel."));
            let spills: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| ours(e)).collect();
            assert!(spills.is_empty(), "{ruleset:?} {scale}: {spills:?}");
            let panel_rects: Vec<Rect> = b.panels.iter().filter(|p| p.0 == "hero" || p.0 == "intel").map(|p| p.1).collect();
            let in_panels = |e: &Value| {
                let (x, y) = (e["at"][0].as_f64().unwrap_or(-1.0) as i32, e["at"][1].as_f64().unwrap_or(-1.0) as i32);
                panel_rects.iter().any(|r| r.contains(x, y))
            };
            let edges: Vec<&Value> = v["edge"].as_array().unwrap().iter().filter(|e| in_panels(e)).collect();
            assert!(edges.is_empty(), "{ruleset:?} {scale}: {edges:?}");
        }
    }
}

/// PORTRAITFX2 5.3: the battle screen's first frame fires `battle_start` on every living unit's panel portrait,
/// once, and `ambush` on the player side's only in an ambush deployment (fixture rows: a flash and a tint).
#[test]
fn fx2_the_first_frame_fires_battle_start_once_and_ambush_on_the_player_side() {
    let mut fixture = rows();
    fixture.push(row(json!({"id": "start_flash", "event": "battle_start", "ms": 300, "color": "#fbbf24"})));
    fixture.push(row(json!({"id": "ambush_tint", "event": "ambush", "effect": "tint", "ms": 300, "color": "#e11d48"})));
    for deployment in ["standard", emberwake_battle::portrait::AMBUSH_DEPLOYMENT] {
        let mut cx = cx_with(fixture.clone());
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let mut b = BattleScreen::arena_with(&cx, None, &encounter, None, "", false);
        assert!(b.host.session.is_some(), "{encounter} starts: {}", b.message);
        // the deployment as the encounter row would set it (only `start_events` reads it after the start)
        b.host.session.as_mut().expect("a session").def.deployment = deployment.into();
        let encounter = format!("{encounter} ({deployment})");
        assert!(b.panel_faces.get(&b.unit_portrait(&cx, 0).0.key).is_none(), "{encounter}: nothing before the first frame");
        let mut g = crate::hud::hud_tests::gfx();
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let now = b.clock.now();
        let s = b.session();
        let ambush = s.def.deployment == "ambush";
        for u in (0..s.w.units.len()).filter(|&u| s.w.units[u].hp > 0 && !s.w.units[u].is_withdrawn) {
            let key = b.unit_portrait(&cx, u).0.key;
            let f = b.panel_faces.get(&key).unwrap_or_else(|| panic!("{encounter}: unit {u}'s portrait reacted"));
            assert_eq!(f.level("flash", now).map(|l| l.start), Some(now), "{encounter}: unit {u}'s battle_start flash");
            let party = s.w.allies.contains(&u);
            assert_eq!(f.level("tint", now).is_some(), ambush && party, "{encounter}: unit {u} (party {party}, ambush {ambush})");
        }
        assert!(!b.portrait_start, "{encounter}: the start events fire once");
    }
}

/// P2 (5.3, 6.2 row 3; FX1 rule I): a step's portrait events wait in its impact: before it lands no struck
/// portrait reacts; at the impact each fires its event's rows from the impact's ms (the flash in its row's colour,
/// a jolt for a blow). Each event through the binding: `crit` jolts harder than `hit`, `guard_hit` flashes the
/// guard gold, `heal` glows gold without a jolt.
#[test]
fn p2_the_panel_portraits_react_at_the_impact_not_at_the_pop() {
    let (mut cx, _g, mut b) = ambush();
    let found = crate::battle::tests::step_until(&mut cx, &mut b, |_, i| !i.portrait.is_empty());
    assert!(found, "the ambush has a step with portrait events at an impact");
    b.resolve_impacts(&mut cx);
    // earlier impacts fired as the steps went (rule F): start the portraits afresh
    b.panel_faces = Faces::default();
    let i = b.impacts[0].clone();
    let at = i.at.expect("resolved");
    let s = b.host.session.as_ref().unwrap();
    for &u in &i.struck {
        let blows = i.portrait.iter().filter(|(v, e)| *v == u && ["hit", "crit", "guard_hit"].contains(e)).count();
        assert_eq!(blows, 1, "{} lost HP: one of hit, crit, guard_hit", s.w.units[u].name);
    }
    // the blows and heals (PORTRAITFX2: a step may carry `ally_down` too, which no flash row answers)
    let keys: Vec<(String, &str)> = i.portrait.iter().filter(|(_, e)| ["hit", "crit", "guard_hit", "heal"].contains(e)).map(|&(u, e)| (b.unit_portrait(&cx, u).0.key, e)).collect();
    assert!(keys.iter().all(|(k, _)| b.panel_faces.get(k).is_none()), "nothing reacts at the pop");
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    assert!(b.impacts.is_empty());
    for (k, ev) in &keys {
        let f = b.panel_faces.get(k).unwrap_or_else(|| panic!("{k} reacted to {ev}"));
        let flash = f.level("flash", at).unwrap_or_else(|| panic!("{k}: a flash for {ev}"));
        assert_eq!((flash.start, flash.color[..3].to_vec()), (at, flash_of(ev).to_vec()), "{k}: {ev}'s flash from the impact");
        assert_eq!(f.jolt_amplitude(at) > 0.0, *ev != "heal", "{k}: {ev}'s jolt");
    }

    // each event through the binding
    let u = b.session().w.allies[0];
    let key = b.unit_portrait(&cx, u).0.key;
    let mut amp = std::collections::BTreeMap::new();
    for ev in ["hit", "crit", "guard_hit", "heal"] {
        b.panel_faces = Faces::default();
        b.react_portraits(&cx, &[(u, ev)], 5000);
        let f = b.panel_faces.get(&key).expect("reacted");
        assert_eq!(f.level("flash", 5000).map(|l| l.color[..3].to_vec()), Some(flash_of(ev).to_vec()), "{ev}");
        amp.insert(ev, f.jolt_amplitude(5000));
    }
    assert!(amp["crit"] > amp["hit"] && amp["hit"] > 0.0 && amp["guard_hit"] > 0.0 && amp["heal"] == 0.0, "{amp:?}");
    assert_eq!(b.panel_faces.clock, b.clock, "the panel clock is the FX clock again after a reaction");
}

/// Draws unit `u`'s panel portrait at `t` and past `t + EASE_MS` (its holds eased in) from the scratch file in `art`;
/// its composed centre pixel.
fn settled(cx: &Cx, b: &mut BattleScreen, art: &mut Art, g: &mut Gfx, u: usize, t: u64) -> [u8; 4] {
    let rows = rows();
    let opts = PortraitOpts { reduce_motion: true, ..PortraitOpts::default() };
    // the second draw a frame step past the ease (time-varying effects read the 30 a second steps, 4.5)
    for ms in [t, t + EASE_MS + 100] {
        b.panel_faces.clock = Clock { ms: ms as f64, hold: 0.0 };
        let (who, look) = b.unit_portrait(cx, u);
        let mut ui: Ui<Action> = Ui::new(g, (-500, -500), true);
        b.panel_faces.begin();
        assert!(b.panel_faces.draw_parts(&mut ui.p, art, &rows, &who, &look, PANEL_SURFACE, Rect::new(4, 4, PANEL_PORTRAIT, PANEL_PORTRAIT), &opts));
    }
    let key = b.unit_portrait(cx, u).0.key;
    let img = b.panel_faces.get(&key).and_then(|f| f.composed()).expect("composed").clone();
    let i = ((img.h / 2 * img.w + img.w / 2) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]
}

/// P3 (4.2, 5.2): a unit with `stealth` draws at alpha 0.58 (`fade` 0.6); below 25 % HP `desaturate` 0.5; down:
/// `still` and `desaturate` 1.0 (a grey pixel). The HP is the board's (rule I).
#[test]
fn p3_the_panel_holds_follow_the_units_state() {
    let (cx, mut g, mut b) = ambush();
    let u = b.session().w.allies[0];
    let rel = b.unit_portrait(&cx, u).0.chain("")[0].clone();
    let mut art = Art::layered(Layers::of(vec![layer("p3", &rel, 32, [180, 90, 40], None)]));
    let level = |b: &BattleScreen, e: &str| b.panel_faces.get(&b.unit_portrait(&cx, u).0.key).and_then(|f| f.level(e, b.panel_faces.now())).map(|l| l.i);
    let plain = settled(&cx, &mut b, &mut art, &mut g, u, 1000);
    assert_eq!((level(&b, "fade"), level(&b, "desaturate"), plain[3]), (None, None, 255), "no state: no hold");

    b.host.session.as_mut().unwrap().w.units[u].st.active.insert("stealth".into());
    let px = settled(&cx, &mut b, &mut art, &mut g, u, 2000);
    assert!(level(&b, "fade").is_some_and(|i| (i - 0.6).abs() < 1e-3), "{:?}", level(&b, "fade"));
    let want = (255.0 * (1.0 - 0.7 * 0.6f32)).round() as i32;
    assert!((px[3] as i32 - want).abs() <= 1, "stealth draws at alpha 0.58: {} vs {want}", px[3]);
    b.host.session.as_mut().unwrap().w.units[u].st.active.shift_remove("stealth");

    let max = b.session().w.units[u].max_hp;
    let low = (max * 25 - 1) / 100;
    assert!(low > 0, "a max HP the 25 % row can match above 0");
    b.host.session.as_mut().unwrap().w.units[u].hp = low;
    b.after_change(&cx);
    settled(&cx, &mut b, &mut art, &mut g, u, 3000);
    assert!(level(&b, "desaturate").is_some_and(|i| (i - 0.5).abs() < 1e-3), "below 25 %: {:?}", level(&b, "desaturate"));
    assert_eq!(level(&b, "still"), None);

    b.host.session.as_mut().unwrap().w.units[u].hp = 0;
    b.after_change(&cx);
    let px = settled(&cx, &mut b, &mut art, &mut g, u, 4000);
    assert!(level(&b, "desaturate").is_some_and(|i| (i - 1.0).abs() < 1e-3), "down: {:?}", level(&b, "desaturate"));
    assert!(level(&b, "still").is_some(), "down holds still");
    assert!(px[0] == px[1] && px[1] == px[2], "down draws grey: {px:?}");
}

/// P4 (4.5): the panel portraits run on the FX clock: a hit-stop holds them still (a flash keeps its level), and
/// the animation speed 0.5 moves them at half the wall time.
#[test]
fn p4_hit_stop_pauses_the_panel_clock_and_anim_speed_scales_it() {
    let (mut cx, mut g, mut b) = ambush();
    let u = b.session().w.allies[0];
    let key = b.unit_portrait(&cx, u).0.key;
    b.selected = Some(u);
    b.clock = Clock { ms: 1000.0, hold: 0.0 };
    b.react_portraits(&cx, &[(u, "hit")], 1000);
    b.clock.advance(40.0, 1.0);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(b.panel_faces.now(), 1040, "the panels draw at the FX clock");
    let before = b.panel_faces.get(&key).and_then(|f| f.level("flash", b.panel_faces.now())).expect("the flash runs").i;
    b.clock.hold(120.0);
    b.clock.advance(100.0, 1.0);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(b.panel_faces.now(), 1040, "a hit-stop holds the portraits");
    assert_eq!(b.panel_faces.get(&key).and_then(|f| f.level("flash", b.panel_faces.now())).map(|l| l.i), Some(before), "the flash keeps its level");

    // the screen's tick: the FX clock moves by the wall time at the animation speed (at most 100 ms a frame)
    for (speed, lo, hi) in [(1.0, 60.0, 100.0), (0.5, 30.0, 50.0)] {
        cx.settings.v["anim_speed"] = json!(speed);
        b.clock = Clock { ms: 0.0, hold: 0.0 };
        b.clock_at = Instant::now() - Duration::from_millis(60);
        b.tick(&mut cx);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let moved = b.panel_faces.now() as f64;
        assert!(moved >= lo && moved <= hi, "anim_speed {speed}: {moved} ms for 60+ wall ms");
    }
}

/// P5 (8): presentation only: the session, the battle's RNG state and the request trace after a scripted battle are
/// equal with the panel portraits drawn every step and never drawn; two drawn runs give the same portrait pixels
/// for the same frames.
#[test]
fn p5_portraits_never_touch_the_battle_and_replay_the_same_pixels() {
    let run = |draw: bool| {
        let (mut cx, mut g, mut b) = ambush();
        let mut pixels = Vec::new();
        for k in 0..40u64 {
            let (over, phase, planning, can) = {
                let s = b.host.session.as_ref().unwrap();
                (s.combat_over, s.phase, s.is_planning(), s.can_advance_round())
            };
            if over || (phase == Phase::Replay && !can) {
                break;
            }
            if phase == Phase::Replay {
                b.send(&cx, json!({"cmd": "advance"}));
                continue;
            }
            if planning {
                b.send(&cx, json!({"cmd": "preset", "preset": "aggressive"}));
            }
            b.send(&cx, json!({"cmd": "step"}));
            if draw {
                b.resolve_impacts(&mut cx);
                b.clock = Clock { ms: (k * 700 + 600) as f64, hold: 0.0 };
                b.fire_due(&mut cx);
                b.clock.hold = 0.0;
                b.draw(&mut cx, &mut g, (-500, -500), true);
                for id in ["hero.portrait", "intel.portrait"] {
                    if let Some(r) = b.w.iter().find(|w| w.id == id).map(|w| w.rect) {
                        let c = &g.canvas;
                        let crop: Vec<u8> = (r.y..r.y + r.h).flat_map(|y| (r.x..r.x + r.w).flat_map(move |x| c.px[((y * c.w + x) * 4) as usize..((y * c.w + x) * 4 + 4) as usize].to_vec())).collect();
                        pixels.push((k, id, crop));
                    }
                }
            }
        }
        b.flush(&cx);
        let state = b.host.handle(&json!({"cmd": "state"}));
        let s = b.host.session.as_ref().unwrap();
        (state, format!("{:?}", s.resolver.rng), s.resolver.log.clone(), b.host.history.clone(), pixels)
    };
    let drawn = run(true);
    let plain = run(false);
    assert_eq!(drawn.0, plain.0, "the session state");
    assert_eq!(drawn.1, plain.1, "the RNG state");
    assert_eq!((&drawn.2, &drawn.3), (&plain.2, &plain.3), "the log and the request trace");
    assert!(!drawn.4.is_empty(), "the panels drew portraits");
    let again = run(true);
    assert_eq!(drawn.4, again.4, "a replay draws the same portrait pixels for the same frames");
}
