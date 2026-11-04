//! FPSCAP1: a battle slows its frames only while it waits for the player with nothing playing, and never stops them.

use crate::battle::{Action, BattleScreen, REST_AFTER};
use crate::screens::{Cx, Screen};
use aegis_ui::art::Art;
use aegis_ui::pacing::{FrameNeed, SLOW_LIFE};
use serde_json::json;
use std::time::Instant;

fn arena() -> (Cx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, None, "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    (cx, b)
}

fn rested(b: &mut BattleScreen) {
    b.astir_at = Instant::now() - REST_AFTER * 2;
}

#[test]
fn a_plan_at_rest_draws_slowly_and_only_after_the_last_stir_has_passed() {
    let (mut cx, mut b) = arena();
    assert!(b.planning() && b.at_rest(), "the plan is being made, nothing plays");
    b.astir_at = Instant::now();
    assert_eq!(b.frame_need(&cx), FrameNeed::Full, "just stirred");
    rested(&mut b);
    assert_eq!(b.frame_need(&cx), FrameNeed::Within(SLOW_LIFE));
    b.tick(&mut cx);
    assert_eq!(b.frame_need(&cx), FrameNeed::Within(SLOW_LIFE), "a tick at rest changes nothing");
}

#[test]
fn anything_playing_gets_every_frame() {
    let (cx, mut b) = arena();
    rested(&mut b);
    let flags: [fn(&mut BattleScreen, bool); 4] = [|b, on| b.playing = on, |b, on| b.auto = on, |b, on| b.pending_next_round = on, |b, on| b.viewing = on];
    for (i, set) in flags.iter().enumerate() {
        set(&mut b, true);
        assert_eq!(b.frame_need(&cx), FrameNeed::Full, "flag {i}");
        set(&mut b, false);
    }
    b.gap_until = Some(Instant::now() + REST_AFTER);
    assert_eq!(b.frame_need(&cx), FrameNeed::Full, "the impact gap is a timer");
    b.gap_until = None;
    assert_eq!(b.frame_need(&cx), FrameNeed::Within(SLOW_LIFE));
}

#[test]
fn a_confirmed_plan_gets_every_frame_and_the_stir_counts_from_it() {
    let (mut cx, mut b) = arena();
    rested(&mut b);
    b.act(&mut cx, Action::Confirm);
    assert!(!b.at_rest(), "the round plays");
    assert_eq!(b.frame_need(&cx), FrameNeed::Full);
    let before = b.astir_at;
    b.tick(&mut cx);
    assert!(b.astir_at > before, "a tick while it plays moves the last stir");
}

/// The playback's end waits for Next: at rest again, with the screen still ticking at the slow rate.
#[test]
fn a_finished_round_waiting_for_next_rests_again() {
    let (mut cx, mut b) = arena();
    b.act(&mut cx, Action::Confirm);
    b.playing = false;
    b.impacts.clear();
    b.gap_until = None;
    b.fx.clear();
    assert!(!b.planning(), "the plan is gone");
    assert!(b.at_rest(), "nothing moves by itself");
    rested(&mut b);
    assert_eq!(b.frame_need(&cx), FrameNeed::Within(SLOW_LIFE));
}

/// VISCLAR1-D: a NextRound after a long rest draws every frame until the round's banner and the refill are both done, on the
/// FX clock: at animation speed 0.5 a wall time past `REST_AFTER` is not enough.
#[test]
fn the_next_round_gets_every_frame_until_its_banner_and_refill_end() {
    use crate::battle::playback::ROUND_BANNER_MS;
    use crate::battle::tips::AP_REFILL_MS;
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(emberwake_rules::ruleset::TIMELINE.into()), "", false);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    let a = b.selected.unwrap();
    assert_eq!(b.send(&cx, json!({"cmd": "resolve"}))["ok"], json!(true));
    b.host.session.as_mut().unwrap().w.units[a].ap = 0;
    // the round over, long at rest: banner and pips landed
    b.draw(&mut cx, &mut g, (-500, -500), true);
    b.clock.advance(ROUND_BANNER_MS.0 as f64 + ROUND_BANNER_MS.1 as f64 + ROUND_BANNER_MS.2 as f64 + AP_REFILL_MS as f64, 1.0);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    b.impacts.clear();
    b.gap_until = None;
    b.fx.clear();
    b.floats.clear();
    b.flashes.clear();
    b.tracers.clear();
    b.sidesteps.clear();
    rested(&mut b);
    assert_eq!(b.frame_need(&cx), FrameNeed::Within(SLOW_LIFE), "resting at the round's end");
    b.act(&mut cx, Action::NextRound);
    assert_eq!(b.frame_need(&cx), FrameNeed::Full, "the advance is a stir");
    // no draw yet: the old long rest must not count
    assert!(b.astir_at.elapsed() < REST_AFTER);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(b.round_banner_alive() && b.pips_moving(), "both just started");
    rested(&mut b);
    // more wall time than REST_AFTER, at animation speed 0.5: under the refill's FX time when the rest is short
    b.clock.advance(AP_REFILL_MS as f64, 0.5);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(b.pips_moving(), "half the refill's FX time");
    assert_eq!(b.frame_need(&cx), FrameNeed::Full, "the pips still move");
    b.clock.advance(AP_REFILL_MS as f64, 0.5);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(!b.pips_moving(), "the refill landed");
    assert!(b.round_banner_alive());
    assert_eq!(b.frame_need(&cx), FrameNeed::Full, "the banner is still up");
    let (fade_in, hold, fade_out) = ROUND_BANNER_MS;
    b.clock.advance((fade_in + hold + fade_out) as f64, 1.0);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(!b.round_banner_alive());
    assert_eq!(b.frame_need(&cx), FrameNeed::Within(SLOW_LIFE), "both done: rest again");
    // the pips alone keep the frames: a fall with the banner long gone
    let ap = s_ap(&b, a);
    assert!(ap > 0, "the refill gave AP");
    b.host.session.as_mut().unwrap().w.units[a].ap = 0;
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(!b.round_banner_alive() && b.pips_moving());
    assert_eq!(b.frame_need(&cx), FrameNeed::Full, "the pips move, no banner");
    b.clock.advance(crate::battle::tips::AP_DRAIN_MS as f64, 1.0);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(b.frame_need(&cx), FrameNeed::Within(SLOW_LIFE), "landed");
}

fn s_ap(b: &BattleScreen, a: usize) -> i32 {
    b.host.session.as_ref().unwrap().w.units[a].ap
}
