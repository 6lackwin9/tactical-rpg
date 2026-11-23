//! ENGINE-60 (PLAN-START2): a round's planning start sends a member's tactic draft without first working it out on a
//! copy when the copy can decide nothing (no buff a new intent would drop, no planned row yet). The plans, ghosts,
//! waiting drafts and the requests sent are exactly those of drafting every member through the copy.

use super::ghost_defer_tests::board;
use crate::battle::input::START_COPIES;
use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use aegis_ui::art::Art;
use aegis_core::clock::Instant;
use emberwake_battle::session::Phase;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::{json, Value};

fn start(encounter: Option<&str>, ruleset: Option<&str>, copies: bool) -> (Cx, BattleScreen) {
    START_COPIES.with(|c| c.set(copies));
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    cx.settings.v["skip_anim"] = json!(true);
    let encounter = encounter.map(String::from).unwrap_or_else(|| cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter"));
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    (cx, b)
}

fn tick_ready(b: &mut BattleScreen, cx: &mut Cx) {
    b.next_at = Instant::now();
    b.gap_until = None;
    b.tick(cx);
}

/// The board and every request sent so far.
fn seen(b: &mut BattleScreen) -> Value {
    json!({"board": board(b), "requests": b.host.history.clone()})
}

/// Plays the round planned and goes on to the next round's planning start.
fn next_round(b: &mut BattleScreen, cx: &mut Cx) {
    b.advanced_at = None;
    b.act(cx, Action::Confirm);
    let round = b.session().current_round;
    for _ in 0..2000 {
        if b.session().combat_over || (b.session().current_round > round && b.planning()) {
            return;
        }
        if b.session().phase == Phase::Replay && !b.pending_next_round {
            b.run(cx, Action::NextRound);
        }
        tick_ready(b, cx);
    }
    panic!("the next round did not come");
}

fn both(encounter: Option<&str>, ruleset: Option<&str>, rounds: usize) {
    let (mut cx, mut quick) = start(encounter, ruleset, false);
    let (mut cx2, mut copied) = start(encounter, ruleset, true);
    assert_eq!(seen(&mut quick), seen(&mut copied), "round 1's planning start");
    for r in 2..=rounds + 1 {
        START_COPIES.with(|c| c.set(false));
        next_round(&mut quick, &mut cx);
        START_COPIES.with(|c| c.set(true));
        next_round(&mut copied, &mut cx2);
        if quick.session().combat_over {
            break;
        }
        assert_eq!(quick.session().current_round as usize, r, "round {r} reached");
        assert!(quick.host.history.iter().filter(|q| q["cmd"] == json!("preset")).count() > 0, "drafts were sent");
        assert_eq!(seen(&mut quick), seen(&mut copied), "round {r}'s planning start");
    }
}

#[test]
fn a_timeline_fight_starts_each_round_with_the_same_ghosts() {
    both(None, Some(TIMELINE), 2);
}

/// The hold escape's pick (the Crucible's exit pick, here on the kit's catalog sample): the planning start runs while
/// the pick waits, and after it.
#[test]
fn a_hold_pick_starts_with_the_same_ghosts() {
    let (mut cx, mut quick) = start(Some("kit_winch_escape"), None, false);
    let (mut cx2, mut copied) = start(Some("kit_winch_escape"), None, true);
    assert!(quick.session().needs_stay_pick(), "the pick waits");
    assert_eq!(seen(&mut quick), seen(&mut copied), "the pick waiting");
    let pick = crate::battle::marks::hold_pick_ids(quick.session())[0];
    START_COPIES.with(|c| c.set(false));
    quick.run(&mut cx, Action::StayPick(pick));
    START_COPIES.with(|c| c.set(true));
    copied.run(&mut cx2, Action::StayPick(pick));
    assert_eq!(seen(&mut quick), seen(&mut copied), "the pick made");
    START_COPIES.with(|c| c.set(false));
    next_round(&mut quick, &mut cx);
    START_COPIES.with(|c| c.set(true));
    next_round(&mut copied, &mut cx2);
    assert!(quick.session().combat_over || quick.session().current_round == 2, "round 2 reached");
    assert_eq!(seen(&mut quick), seen(&mut copied), "round 2's planning start");
}
