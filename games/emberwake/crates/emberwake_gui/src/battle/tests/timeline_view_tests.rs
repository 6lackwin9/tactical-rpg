//! INPUT1-T (INPUT1.md 8, TV1-TV7): the full timeline view over the grid panel (key T), built by
//! `timeline_view::build`, the Gallery's Battle timeline's builder too.

use crate::battle::marks::FORESIGHT_OFF;
use crate::battle::{Action, BattleScreen, TILE, TV_PAD};
use crate::battle::tips::ap_lines;
use crate::intent_text;
use crate::screens::{Cx, Key, Screen};
use aegis_ui::theme::th;
use aegis_ui::ui::{Gfx, H, W};
use emberwake_battle::grid::GRID_H;
use emberwake_battle::round_intent::{self as ri};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use aegis_ui::art::Art;
use serde_json::{json, Value};
use std::time::Instant;
use crate::timeline_view::{masked_note, track_key, unit_of};
use aegis_ui::timeline::{Timeline, Track, HEADER_H, ROW_H, RULER_H};
use emberwake_rules::ruleset::TIMELINE;

fn arena(ruleset: Option<&str>, reveal: bool) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", reveal);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    // the planning start drafts the ghosts (GH2 a)
    b.tick(&mut cx);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn drawn(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
    b.draw(cx, g, (-500, -500), true);
}

/// The view as last drawn (it must be open).
fn view(b: &BattleScreen) -> &Timeline {
    b.tv_shown.as_ref().expect("the view is open")
}

fn tracks(t: &Timeline, section: usize) -> &[Track] {
    &t.sections[section].tracks
}

fn track(t: &Timeline, u: usize) -> &Track {
    t.sections.iter().flat_map(|x| &x.tracks).find(|x| x.key == track_key(u)).unwrap_or_else(|| panic!("no track for unit {u}"))
}

/// TV1: T opens the view over the grid panel's rect and closes it; Esc closes it first (the inspected unit stays,
/// SL3's steps after); while open the board takes no click (the view's widgets are on top); classic: T does nothing.
#[test]
fn t_opens_the_view_over_the_grid_and_t_or_esc_closes_it() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    drawn(&mut cx, &mut g, &mut b);
    assert_eq!(b.state(&cx)["timeline_view"], Value::Null, "closed at the start");
    let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.has(e)).unwrap();
    let cell = b.w.iter().find(|x| x.id == format!("unit.{foe}")).map(|x| x.rect).expect("the foe's token");
    b.key(&mut cx, Key::Char('t'));
    drawn(&mut cx, &mut g, &mut b);
    assert!(b.state(&cx)["timeline_view"].is_object());
    let grid = b.panels.iter().find(|p| p.0 == "grid").unwrap().1;
    assert_eq!((grid.x, grid.y, grid.w, grid.h), (126, 28, 388, 292), "the grid panel's rect");
    let back = b.w.iter().find(|x| x.id == "timeline.backdrop").expect("the backdrop").rect;
    assert_eq!(back, grid, "the view takes the grid panel's rect");
    assert!(matches!(aegis_ui::ui::hit(&b.w, (cell.x + cell.w / 2, cell.y + cell.h / 2)), Some(Action::TimelinePick(_))), "the board takes no click");
    b.key(&mut cx, Key::Char('t'));
    drawn(&mut cx, &mut g, &mut b);
    assert_eq!(b.state(&cx)["timeline_view"], Value::Null, "T closes it");
    assert!(b.w.iter().all(|x| !x.id.starts_with("timeline.")));
    b.key(&mut cx, Key::Char('t'));
    b.inspect_unit(foe);
    b.escape(&mut cx);
    assert!(!b.tv_open && b.inspected() == Some(foe), "Esc closes the view first");
    b.escape(&mut cx);
    assert_eq!(b.inspected(), None, "then SL3's steps");
    let (mut cx, mut g, mut b) = arena(None, false);
    b.key(&mut cx, Key::Char('t'));
    drawn(&mut cx, &mut g, &mut b);
    assert!(!b.tv_open && b.state(&cx).get("timeline_view").is_none(), "classic: no view, no key");
}

/// TV2: "Party" (allies in roster order, present or acting) above "Enemies" (the hostile list's order); the ruler
/// is the round's blocks; no playhead while planning.
#[test]
fn the_sections_hold_one_track_per_unit_in_order() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    b.tv_open = true;
    drawn(&mut cx, &mut g, &mut b);
    let t = view(&b);
    let w = &s(&b).w;
    assert_eq!(t.sections.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(), ["Party", "Enemies"]);
    let keys = |side: &[usize]| side.iter().copied().filter(|&u| w.has(u) || s(&b).resolver.scheduled.iter().any(|a| a.unit == u)).map(track_key).collect::<Vec<_>>();
    assert_eq!(tracks(t, 0).iter().map(|x| x.key.clone()).collect::<Vec<_>>(), keys(&w.allies));
    assert_eq!(tracks(t, 1).iter().map(|x| x.key.clone()).collect::<Vec<_>>(), keys(&w.enemies));
    assert_eq!((t.blocks, t.playhead), (s(&b).def.total_blocks, 0));
    assert!(tracks(t, 0).iter().chain(tracks(t, 1)).all(|x| !x.hatched), "nobody is down yet");
}

/// TV3: a party track is the member's plan (each compiled entry a span, the planner's ghost in the ghost style),
/// its label leads with the intent's letter, its card holds AP9's lines; the selected hero's track is selected.
#[test]
fn a_party_track_is_the_plan_with_its_ghost_and_ap_card() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    b.tv_open = true;
    drawn(&mut cx, &mut g, &mut b);
    let t = view(&b).clone();
    let sess = s(&b);
    let h = b.selected.expect("a hero is selected");
    assert_eq!(t.selected_track, Some(track_key(h)));
    for &u in sess.w.allies.iter().filter(|&&u| sess.w.has(u)) {
        let tr = track(&t, u);
        let entries = sess.plan.compile(&sess.w, u).entries;
        let ghosts = rip::ghost_entries(&sess.w, &sess.plan, u);
        let spans: Vec<(i32, i32, bool)> = entries.iter().enumerate().map(|(i, e)| (e.start_block, e.end_block(&sess.w), ghosts[i] && sess.w.cat.skills[e.skill].kind != "REST")).collect();
        assert_eq!(tr.segments.iter().map(|x| (x.start, x.end, x.ghost)).collect::<Vec<_>>(), spans, "unit {u}");
        assert!(tr.segments.iter().filter(|x| x.ghost).all(|x| x.tip[0].0.starts_with("Auto-filled")), "GH8: a ghost's card");
        let i = ri::intent_of(&sess.w, u).expect("the ghost's intent");
        assert_eq!(tr.label, format!("{} {}", &i.name()[..1], sess.w.units[u].name));
        let card: Vec<String> = tr.tip.iter().map(|l| l.0.clone()).collect();
        for l in ap_lines(&rip::ap_forecast(&sess.w, &sess.plan, u)) {
            assert!(card.contains(&l), "AP9 line {l:?} in {card:?}");
        }
    }
    assert!(t.sections[0].tracks.iter().any(|x| x.segments.iter().any(|g| g.ghost)), "the planner's ghost shows");
}

/// TV4: while planning an unrevealed enemy is masked: its note is the intent and target and the tell (no other
/// skill of its plan), its impact badges are marks, it holds no span; with the tells hidden the note drops the tell.
#[test]
fn an_unrevealed_enemy_is_masked_with_its_intent_tell_and_badges() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    b.tv_open = true;
    drawn(&mut cx, &mut g, &mut b);
    let t = view(&b).clone();
    let sess = s(&b);
    let w = &sess.w;
    let tells = sess.enemy_intents();
    let mut masked = 0;
    for tr in tracks(&t, 1) {
        let e = unit_of(&tr.key).unwrap();
        let rec = tells.iter().find(|r| r.unit == e);
        if rec.is_some_and(|r| r.revealed) {
            continue;
        }
        masked += 1;
        let note = tr.masked.clone().expect("masked");
        assert_eq!(note, masked_note(w, rec, true));
        assert!(tr.segments.is_empty(), "no span of its plan in the value");
        let r = rec.expect("a tell");
        if let Some(head) = intent_text::round_head(w, r) {
            assert!(note.starts_with(&head), "{note}");
        }
        for a in sess.resolver.scheduled.iter().filter(|a| a.unit == e && a.command.name != r.line.skill) {
            assert!(!note.contains(&a.command.name), "{note} names {}", a.command.name);
        }
        let impacts: Vec<(i32, String)> = r.impacts.iter().map(|(b, k)| (*b, if *k == emberwake_battle::intent::HIT { "!" } else { "*" }.to_string())).collect();
        assert_eq!(tr.marks.iter().map(|m| (m.block, m.glyph.clone())).collect::<Vec<_>>(), impacts, "V1's badges as marks");
        assert!(tr.marks.iter().all(|m| m.colour == if m.glyph == "!" { th().foe } else { th().gold }));
    }
    assert!(masked > 0, "an enemy plans in secret");
    cx.settings.v["show_tells"] = json!(false);
    drawn(&mut cx, &mut g, &mut b);
    for tr in tracks(view(&b), 1).iter().filter(|x| x.masked.is_some()) {
        let e = unit_of(&tr.key).unwrap();
        assert_eq!(tr.masked.clone().unwrap(), masked_note(&s(&b).w, s(&b).enemy_intents().iter().find(|r| r.unit == e), false));
        assert!(!tr.masked.clone().unwrap().contains("Next:"));
    }
}

/// TV4: during playback an unrevealed enemy's track holds only its actions that have started (block <= the
/// playhead); the playhead is the resolving block (INPUT1 PB2: the strip's cursor, the block the last step
/// resolved; the session's current block before the first step).
#[test]
fn during_playback_an_unrevealed_enemy_shows_only_what_has_started() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    b.act(&mut cx, Action::Confirm);
    b.tv_open = true;
    let mut checked = 0;
    assert!(s(&b).plan_confirmed, "{}", b.message);
    while s(&b).phase != Phase::Replay {
        drawn(&mut cx, &mut g, &mut b);
        let t = view(&b).clone();
        let at = b.cursor_block(Instant::now()).unwrap_or(s(&b).current_block);
        let sess = s(&b);
        assert_eq!(t.playhead, at);
        let rounds = sess.round_intents();
        for tr in tracks(&t, 1) {
            let e = unit_of(&tr.key).unwrap();
            assert!(tr.masked.is_none(), "no mask during playback");
            if rounds.iter().find(|r| r.unit == e).is_some_and(|r| r.revealed) {
                continue;
            }
            assert!(tr.segments.iter().all(|x| x.start <= at), "block {at}: {:?}", tr.segments);
            let started = sess.resolver.scheduled.iter().filter(|a| a.unit == e && a.start <= at).count();
            assert_eq!(tr.segments.len(), started, "every started action");
            checked += 1;
        }
        b.send(&cx, json!({"cmd": "step"}));
    }
    assert!(checked > 0, "an unrevealed enemy was checked during playback");
}

/// TV4: a revealed enemy (the battle's plan reveal here) is unmasked with its whole plan and the card "Plan
/// revealed (...)"; VC4: a foresight reveal is masked again with Party foresight off (debug builds).
#[test]
fn a_revealed_enemy_is_unmasked_and_foresight_off_masks_it_again() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), true);
    b.tv_open = true;
    drawn(&mut cx, &mut g, &mut b);
    let t = view(&b).clone();
    let sess = s(&b);
    for tr in tracks(&t, 1).iter().filter(|x| sess.w.units[unit_of(&x.key).unwrap()].is_active()) {
        let e = unit_of(&tr.key).unwrap();
        assert!(tr.masked.is_none(), "revealed: unmasked");
        assert_eq!(tr.segments.len(), sess.resolver.scheduled.iter().filter(|a| a.unit == e).count(), "its whole plan");
        assert!(tr.tip.iter().any(|l| l.0 == "Plan revealed (plan reveal)"), "{:?}", tr.tip);
    }
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    let h = b.selected.unwrap();
    {
        let sess = b.host.session.as_mut().unwrap();
        sess.w.units[h].foresight_source = "t_sight".into();
        sess.w.units[h].foresight_range = emberwake_rules::unit::FORESIGHT_RANGE_MAX;
        sess.measure_reveals();
    }
    let read: Vec<usize> = s(&b).enemy_intents().iter().filter(|t| t.revealed && !t.readers.is_empty() && !t.enraged).map(|t| t.unit).collect();
    assert!(!read.is_empty(), "the range reveals a foe");
    b.tv_open = true;
    drawn(&mut cx, &mut g, &mut b);
    for &e in &read {
        let tr = track(view(&b), e);
        assert!(tr.masked.is_none() && tr.tip.iter().any(|l| l.0.starts_with("Plan revealed (Read by ")), "{:?}", tr.tip);
    }
    cx.settings.v[FORESIGHT_OFF] = json!(true);
    drawn(&mut cx, &mut g, &mut b);
    for &e in &read {
        assert_eq!(track(view(&b), e).masked.is_some(), cfg!(debug_assertions), "VC4 masks it in a debug build");
    }
}

/// TV5: a click on a party track selects that hero and the view stays open; an enemy track or a block selects
/// nothing.
#[test]
fn a_party_track_click_selects_its_hero() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    b.tv_open = true;
    drawn(&mut cx, &mut g, &mut b);
    let h = b.selected.unwrap();
    let other = s(&b).w.allies.iter().copied().find(|&a| a != h && s(&b).w.has(a)).expect("a second hero");
    b.click_id(&mut cx, &format!("timeline.track.{}", track_key(other))).unwrap();
    assert_eq!((b.selected, b.tv_open), (Some(other), true));
    let foe = unit_of(&tracks(view(&b), 1)[0].key).unwrap();
    b.click_id(&mut cx, &format!("timeline.track.{}", track_key(foe))).unwrap();
    b.click_id(&mut cx, "timeline.block.1").unwrap();
    assert_eq!((b.selected, b.tv_open, b.popup.is_none()), (Some(other), true, true), "nothing else happens");
    drawn(&mut cx, &mut g, &mut b);
    assert_eq!(view(&b).selected_track, Some(track_key(other)));
}

/// TV6: 16 tracks fit in the view before the widget scrolls; at the four text sizes no label or note spills, and a
/// shortened one reads in full on its widget's hover card.
#[test]
fn the_view_fits_at_every_text_size() {
    let inner = GRID_H * TILE + 4 - 2 * TV_PAD;
    assert!(RULER_H + 2 + 2 * (HEADER_H + 3) + 16 * ROW_H <= inner, "16 tracks before the scroll");
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    b.tv_open = true;
    for scale in [0.85f32, 1.0, 1.15, 1.3] {
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
        let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        let spills: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("timeline."))).collect();
        assert!(spills.is_empty(), "scale {scale}: {spills:?}");
        for c in v["cut"].as_array().unwrap() {
            let (x, y) = (c["at"][0].as_f64().unwrap() as i32 + 1, c["at"][1].as_f64().unwrap() as i32 + 2);
            let full = c["text"].as_str().unwrap();
            let Some(wd) = b.w.iter().rev().find(|w| w.rect.contains(x, y) && w.id.starts_with("timeline.") && w.id != "timeline.backdrop") else { continue };
            assert!(wd.text == full || wd.tip.iter().any(|l| l.0.contains(full)), "scale {scale}: {full:?} cut on {} without its full text on hover", wd.id);
        }
    }
}

/// TV7: the battle's view and the Gallery's Battle timeline build the same tracks for the same session (one
/// builder), planning and during playback; the battle's view alone gives every track its portrait square
/// (PORTRAITFX1-TL).
#[test]
fn the_battle_view_and_the_gallery_build_the_same_timeline() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    b.tv_open = true;
    for _ in 0..2 {
        drawn(&mut cx, &mut g, &mut b);
        let sess = s(&b);
        let gallery = crate::gallery::battle_timeline(&cx, sess, sess.current_round, false, None, b.selected.map(track_key));
        let mut shown = view(&b).clone();
        for t in shown.sections.iter_mut().flat_map(|sec| sec.tracks.iter_mut()) {
            assert!(t.icon, "{}: the battle's view has the portrait square", t.key);
            t.icon = false;
        }
        assert_eq!(&shown, &gallery);
        b.act(&mut cx, Action::Confirm);
        b.send(&cx, json!({"cmd": "step"}));
    }
}
