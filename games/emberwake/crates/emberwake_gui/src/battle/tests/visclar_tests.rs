//! VISCLAR1 slice A (INPUT1.md 10: VC1-VC6, AR1-AR6): the calmer board of a timeline battle. A catalog encounter on
//! the catalog's timeline row; every expectation is a rule of the spec or a kit query (the plan, the records, the
//! positions), never a shipped number. Classic keeps today's board.

use crate::battle::marks::{arrows_of, bars_shown, party_foresight_off, party_leg, seen_intents, Arrow, FORESIGHT_OFF};
use crate::battle::overlays::{overlay_of, strike_tile, Layers, OVERLAY_GROUPS};
use crate::battle::BattleScreen;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::round_intent::{self as ri, RoundTarget};
use emberwake_battle::session::Session;
use aegis_ui::art::Art;
use serde_json::{json, Value};
use emberwake_rules::ruleset::TIMELINE;

fn arena(ruleset: Option<&str>, view: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!(view);
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

/// The selected hero takes the first foe on the field as its Assault target.
fn assault(cx: &Cx, b: &mut BattleScreen) -> (usize, usize) {
    let h = b.selected.unwrap();
    let foe = s(b).w.enemies.iter().copied().find(|&e| s(b).w.units[e].is_active() && s(b).w.has(e)).unwrap();
    let r = b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
    assert_eq!(r["ok"], json!(true), "{r}");
    (h, foe)
}

fn arrows(b: &BattleScreen, cx: &Cx) -> Vec<Value> {
    b.state(cx)["overlay"]["arrows"].as_array().cloned().unwrap_or_default()
}

fn units_of(a: &[Value]) -> Vec<usize> {
    a.iter().map(|x| x["unit"].as_u64().unwrap() as usize).collect()
}

/// VC6, VC5, decision "Battle view answers": `default_settings` holds every group on but `all_bars` and the testing
/// groups, and no debug key; a saved `battle_overlays` without a group takes the group's default (a file from before
/// the decision, which holds the old ids on, included); Restore defaults returns to them.
#[test]
fn the_defaults_are_the_calmer_view() {
    let d = crate::screens::default_settings();
    for (id, _, _, on) in OVERLAY_GROUPS {
        assert_eq!(d["battle_overlays"][id], json!(on), "{id}");
        assert_eq!(on, !matches!(id, "all_bars" | "test_hazards" | "test_arena"), "{id}");
    }
    assert!(d.get(FORESIGHT_OFF).is_none());
    let older = Layers::of(&json!({"band": true, "moves": true, "first_move": true, "hazards": true, "arena": true}));
    assert!(!older.all_bars && !older.hazards && !older.arena && older.facings && older.glyphs, "an older file shows the calmer view");
    let mut cx = crate::hud::hud_tests::game_cx();
    let mut g = crate::hud::hud_tests::gfx();
    let mut st = crate::menus::SettingsScreen::new(&cx);
    st.draw(&mut cx, &mut g, (-500, -500), true);
    st.click_id(&mut cx, "settings.tab.battle").unwrap();
    st.draw(&mut cx, &mut g, (-500, -500), true);
    st.click_id(&mut cx, "settings.overlay.test_hazards").unwrap();
    st.click_id(&mut cx, "settings.overlay.all_bars").unwrap();
    let draft = st.state(&cx)["draft"].clone();
    assert!(Layers::of(&draft["battle_overlays"]).hazards && Layers::of(&draft["battle_overlays"]).all_bars, "a click turns each on");
    st.click_id(&mut cx, "settings.defaults").unwrap();
    let draft = st.state(&cx)["draft"].clone();
    let l = Layers::of(&draft["battle_overlays"]);
    assert!(!l.hazards && !l.arena && !l.all_bars && l.moves && l.walks, "Restore defaults");
}

/// VC4, VC5: the reader is false outside a debug build whatever the file says, and the checkbox (a debug build)
/// sets the key.
#[test]
fn party_foresight_off_is_a_debug_build_reader() {
    let on = json!({FORESIGHT_OFF: true});
    assert!(!party_foresight_off(&on, false), "a release build never reads it");
    assert!(party_foresight_off(&on, true));
    assert!(!party_foresight_off(&json!({}), true) && !party_foresight_off(&json!({FORESIGHT_OFF: false}), true));
    let mut cx = crate::hud::hud_tests::game_cx();
    let mut g = crate::hud::hud_tests::gfx();
    let mut st = crate::menus::SettingsScreen::new(&cx);
    st.draw(&mut cx, &mut g, (-500, -500), true);
    st.click_id(&mut cx, "settings.tab.battle").unwrap();
    st.draw(&mut cx, &mut g, (-500, -500), true);
    let has = st.widgets().iter().any(|w| w["id"] == json!("settings.debug.party_foresight_off"));
    assert_eq!(has, cfg!(debug_assertions), "VC5: a debug build only");
    if has {
        st.click_id(&mut cx, "settings.debug.party_foresight_off").unwrap();
        assert_eq!(st.state(&cx)["draft"][FORESIGHT_OFF], json!(true));
    }
}

/// VC2, VC3: the token bars draw for the focus set (the selected hero and its target here, nothing hovered),
/// for every unit with Alt or `all_bars`; classic draws every unit's.
#[test]
fn bars_follow_the_focus_set() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "flat");
    let (h, foe) = assault(&cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let l = Layers::of(&cx.settings.v["battle_overlays"]);
    let st = b.state(&cx);
    assert_eq!(st["focus"], json!([h, foe]), "the selected hero and its round target");
    let focus: Vec<usize> = vec![h, foe];
    let n = s(&b).w.units.len();
    assert!(n > 2);
    for i in 0..n {
        assert_eq!(bars_shown(true, &l, false, &focus, i), focus.contains(&i), "unit {i}");
        assert!(bars_shown(true, &l, true, &focus, i), "Alt: unit {i}");
        assert!(bars_shown(true, &Layers::of(&json!({"all_bars": true})), false, &focus, i), "all_bars: unit {i}");
        assert!(bars_shown(false, &l, false, &[], i), "classic: unit {i}");
    }
    // the inspected unit joins it
    let other = s(&b).w.enemies.iter().copied().find(|&e| e != foe && s(&b).w.units[e].is_active() && s(&b).w.has(e));
    if let Some(e) = other {
        b.inspect = Some((e, s(&b).current_round));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(b.state(&cx)["focus"], json!([h, foe, e]));
    }
    // classic: no focus or overlay key
    let (mut cx, mut g, mut b) = arena(None, "flat");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let st = b.state(&cx);
    assert!(st.get("focus").is_none() && st.get("overlay").is_none());
}

/// AR1-AR4 and the acceptance: by default the agent link's `overlay.arrows` holds the selected hero's arrow only
/// (to its target, alpha 90); with Alt one per unit with a target, never two for a unit; an unrevealed enemy's has
/// two points (no move leg); the inspected unit's shows by default (Q15); with the tells hidden no enemy's.
#[test]
fn one_quiet_arrow_per_unit() {
    for view in ["stage", "flat"] {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), view);
        let (h, foe) = assault(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let a = arrows(&b, &cx);
        assert_eq!(units_of(&a), vec![h], "{view}: the selected hero's only");
        let w = &s(&b).w;
        let fp = w.pos(foe);
        assert_eq!(a[0]["points"][0], json!([w.pos(h).x, w.pos(h).y]));
        assert_eq!(*a[0]["points"].as_array().unwrap().last().unwrap(), json!([fp.x, fp.y]), "{view}: it ends at the target");
        assert_eq!(a[0]["alpha"], json!(90));
        cx.held.alt = true;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let a = arrows(&b, &cx);
        let mut seen = units_of(&a);
        let total = seen.len();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), total, "{view}: at most one arrow a unit: {a:?}");
        assert!(seen.len() > 1 && seen.contains(&h), "{view}: Alt shows every unit's: {a:?}");
        assert!(a.iter().all(|x| x["alpha"] == json!(90) && x["points"].as_array().unwrap().len() >= 2));
        let tells = s(&b).enemy_intents();
        for x in a.iter().filter(|x| tells.iter().any(|t| x["unit"] == json!(t.unit) && !t.revealed)) {
            assert_eq!(x["points"].as_array().unwrap().len(), 2, "{view}: an unrevealed enemy goes straight: {x}");
        }
        // every party member on the field with a unit or tile target has one
        let w = &s(&b).w;
        for &m in w.allies.iter().filter(|&&m| w.units[m].is_active() && w.has(m)) {
            if let Some(RoundTarget::Unit(_) | RoundTarget::Cell(_)) = w.round_states.get(&m).and_then(|r| r.target) {
                assert!(seen.contains(&m), "{view}: member {m}: {a:?}");
            }
        }
        // Q15: the inspected unit's arrow beside the selected hero's, without Alt
        cx.held.alt = false;
        let enemy_arrow = seen.iter().copied().find(|u| w.enemies.contains(u));
        if let Some(e) = enemy_arrow {
            b.inspect = Some((e, s(&b).current_round));
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert_eq!(units_of(&arrows(&b, &cx)), vec![h, e], "{view}");
        }
        // the tells hidden: no enemy arrow even with Alt
        cx.held.alt = true;
        cx.settings.v["show_tells"] = json!(false);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(units_of(&arrows(&b, &cx)).iter().all(|u| s(&b).w.allies.contains(u)), "{view}");
    }
}

/// AR1, AR5: the move leg: the hero's arrow bends where its moves before its first command end (with the group
/// `moves` on; off: straight); an enemy's leg needs `walks`.
#[test]
fn the_move_leg_follows_the_plan_and_the_groups() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "stage");
    let (h, foe) = assault(&cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let sess = s(&b);
    let leg = party_leg(sess, h);
    let tells = sess.enemy_intents();
    let rounds = sess.round_intents();
    let on = arrows_of(sess, &tells, &rounds, &Layers::of(&Value::Null), true);
    let off = arrows_of(sess, &tells, &rounds, &Layers::of(&json!({"moves": false})), true);
    let mine = |v: &[Arrow]| v.iter().find(|a| a.unit == h).cloned().expect("the hero's arrow");
    let (hp, fp) = (sess.w.pos(h), sess.w.pos(foe));
    let want = if leg == hp { vec![hp, fp] } else { vec![hp, leg, fp] };
    assert_eq!(mine(&on).points, want, "the move leg ends where the moves before the first command end");
    assert_eq!(mine(&off).points, vec![hp, fp], "moves off: straight");
    // the leg's end is where a planned move ends or the first command strikes from (or the hero's own tile)
    let o = overlay_of(sess, Some(h), true).unwrap();
    let entries = sess.plan.compile(&sess.w, h).entries;
    assert!(leg == hp || o.moves.iter().any(|(t, _)| t.last() == Some(&leg)) || entries.iter().any(|e| strike_tile(e) == leg), "{leg:?} {:?}", o.moves);
    let foes_off = arrows_of(sess, &tells, &rounds, &Layers::of(&json!({"walks": false})), true);
    assert!(foes_off.iter().filter(|a| !a.ally).all(|a| a.points.len() == 2));
}

/// AR6 (Q6, Q16): during playback the board draws no target arrow and no destination plate (the overlay's moves
/// are empty), with Alt too; the facing chevrons stay (VC6).
#[test]
fn playback_draws_no_arrow_and_no_plate() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "stage");
    assault(&cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(!arrows(&b, &cx).is_empty());
    let r = b.send(&cx, json!({"cmd": "confirm"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert!(!s(&b).is_planning());
    cx.held.alt = true;
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let st = b.state(&cx);
    assert_eq!(st["overlay"]["arrows"], json!([]), "{}", st["overlay"]);
    assert_eq!(st["overlay"]["moves"], json!([]));
    let w = &s(&b).w;
    let facing = w.enemies.iter().filter(|&&e| w.units[e].is_active() && w.has(e) && ri::facing(w, e) != emberwake_battle::grid::ZERO).count();
    assert_eq!(st["overlay"]["facings"].as_array().unwrap().len(), facing, "the chevrons stay");
    assert!(Layers::of(&cx.settings.v["battle_overlays"]).facings);
}

/// VC4: with Party foresight off a Hawk Eye reveal reads unrevealed on the screen (no "Read by" line on the
/// enemy's hover card; its record `revealed` false with no plan lines, so the badge row is not filled) while the
/// kit's records keep it; off, the screen shows it. Its arrow goes straight (AR2).
#[test]
fn party_foresight_off_hides_foresight_reveals() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "flat");
    let h = b.selected.unwrap();
    {
        let sess = b.host.session.as_mut().unwrap();
        sess.w.units[h].foresight_source = "t_sight".into();
        sess.w.units[h].foresight_range = emberwake_rules::unit::FORESIGHT_RANGE_MAX;
        sess.measure_reveals();
    }
    let tells = s(&b).enemy_intents();
    let read: Vec<usize> = tells.iter().filter(|t| t.revealed && !t.readers.is_empty() && !t.enraged).map(|t| t.unit).collect();
    assert!(!read.is_empty(), "the range reveals a foe");
    let seen = seen_intents(&s(&b).w, tells.clone(), true);
    for t in seen.iter().filter(|t| read.contains(&t.unit)) {
        assert!(!t.revealed && t.plan.is_empty() && t.readers.is_empty(), "{t:?}");
    }
    assert_eq!(seen_intents(&s(&b).w, tells.clone(), false), tells, "off: unchanged");
    let tip_of = |b: &BattleScreen, e: usize| b.w.iter().find(|x| x.id == format!("unit.{e}")).map(|x| x.tip.iter().map(|l| l.0.clone()).collect::<Vec<_>>()).unwrap_or_default();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(read.iter().all(|&e| tip_of(&b, e).iter().any(|l| l.starts_with("Read by "))), "the reveal shows");
    cx.settings.v[FORESIGHT_OFF] = json!(true);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    for &e in &read {
        assert!(!tip_of(&b, e).iter().any(|l| l.starts_with("Read by ")), "foe {e}: {:?}", tip_of(&b, e));
    }
    assert!(s(&b).enemy_intents().iter().filter(|t| read.contains(&t.unit)).all(|t| t.revealed), "display only: the kit still reveals");
    cx.held.alt = true;
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let a = arrows(&b, &cx);
    assert!(a.iter().filter(|x| read.iter().any(|e| x["unit"] == json!(e))).all(|x| x["points"].as_array().unwrap().len() == 2), "{a:?}");
}
