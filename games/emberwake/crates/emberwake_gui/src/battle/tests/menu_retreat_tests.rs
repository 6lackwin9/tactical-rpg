//! PR7-41 (PT-27, PT-32, PT-34) on the scratch world of `transitions_tests` and a catalog arena (no shipped data): a
//! world battle's bar shows no Restart, and Step only once a round runs; its Menu offers Retreat while the fight allows
//! it (the kit's `can_retreat`), and Yes to the question ends the battle as fled; Clear on an auto-filled round (no
//! picks) empties it and the hero stays empty (no ghost drafts it again); AP never reads below 0 (a debt is named);
//! the bar's message does not repeat its phase word; kit lines read in player words.

use super::transitions_tests::{battle, beat_cx};
use crate::battle::draw::without_phase;
use crate::battle::{Action, BattleScreen};
use crate::hud::hud_tests::{frame, gfx};
use crate::intent_text::{ap_text, plain};
use crate::screens::{Cx, GameCx, Nav, Screen};
use emberwake_battle::round_intent_plan as rip;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

fn pushed(cx: &mut Cx) -> Option<Box<dyn Screen<GameCx>>> {
    match cx.nav.pop() {
        Some(Nav::Push(s)) => Some(s),
        _ => None,
    }
}

fn ids(b: &BattleScreen) -> Vec<String> {
    b.widgets().iter().map(|w| w["id"].as_str().unwrap_or_default().to_string()).collect()
}

#[test]
fn the_world_bar_hides_restart_and_step_and_the_menu_retreats() {
    let mut cx = beat_cx(false, false);
    let mut g = gfx();
    let mut b = battle(&mut cx, "map");
    b.tick(&mut cx);
    frame(&mut b, &mut cx, &mut g, 1.0);
    let now = ids(&b);
    assert!(!now.iter().any(|i| i == "bar.restart"), "a world battle never restarts: {now:?}");
    assert!(!now.iter().any(|i| i == "bar.step"), "no Step while planning: {now:?}");
    let can = b.host.session.as_ref().unwrap().can_retreat();
    b.act(&mut cx, Action::Settings);
    let mut s = pushed(&mut cx).expect("the Menu popup");
    s.draw(&mut cx, &mut g, (-1, -1), true);
    let popup: Vec<String> = s.widgets().iter().map(|w| w["id"].as_str().unwrap_or_default().to_string()).collect();
    assert!(can, "the scratch world's fight allows a retreat (its timeout outcome is fled)");
    assert!(popup.iter().any(|i| i == "settings.retreat"), "Retreat when the fight allows it: {popup:?}");
    s.click_id(&mut cx, "settings.retreat").expect("Retreat");
    assert!(cx.nav.iter().any(|n| matches!(n, Nav::Push(x) if x.id() == "ask")), "it asks first");
    cx.nav.clear();
    cx.answers.push(crate::menus::RETREAT_ASK.to_string());
    b.tick(&mut cx);
    let sess = b.host.session.as_ref().unwrap();
    assert_eq!((sess.outcome.as_str(), sess.combat_over), ("fled", true), "{}", b.message);
    assert!(b.host.history.iter().any(|r| r["cmd"] == json!("retreat")), "recorded");
}

#[test]
fn clear_on_an_auto_filled_round_empties_it_and_keeps_it_empty() {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    b.tick(&mut cx);
    let mut g = gfx();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let h = b.selected.expect("a selected hero");
    let sess = b.host.session.as_ref().unwrap();
    assert!(rip::picks_of(&sess.w, h).is_empty(), "no picks: the round is the auto fill");
    assert!(!sess.plan.steps_of(h).is_empty(), "the auto fill planned the hero's round");
    b.act(&mut cx, Action::Clear);
    assert!(b.host.session.as_ref().unwrap().plan.steps_of(h).is_empty(), "{}", b.message);
    assert!(b.message.contains("auto plan cleared"), "{}", b.message);
    // a later member's redraft does not fill it again
    b.tick(&mut cx);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(b.host.session.as_ref().unwrap().plan.steps_of(h).is_empty(), "it stays empty this round");
}

#[test]
fn ap_bar_message_and_kit_lines_read_in_player_words() {
    assert_eq!(ap_text(-2, 10, "/"), "AP 0/10 (2 owed)");
    assert_eq!(ap_text(4, 10, " of "), "AP 4 of 10");
    assert_eq!(without_phase("Round over: 1 break. Space (or Next) for the next round.", "Round over"), "1 break. Space (or Next) for the next round.");
    assert_eq!(without_phase("Round over.", "Round over"), "");
    assert_eq!(without_phase("Victory!", "Round over"), "Victory!");
    assert_eq!(plain("Gruk: The fire. (Fortified, new tactics from the next round)"), "Gruk: The fire. (Fortified, fights differently from the next round)");
    assert_eq!(plain("Stopped at (2, 2) (blocked by no route)"), "Stopped at (2, 2) (no way through)");
}
