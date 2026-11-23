//! ENGINE-58 (BLOCK-CLICK2): a ghost block's click takes the clicked member's ghost at once and drafts the later
//! members' ghosts on the next frames (`reghost`, `ghost_tick`); the plans and the ghosts end exactly as they did when
//! every ghost was drafted in the click.

use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use aegis_ui::art::Art;
use aegis_ui::ui::Gfx;
use emberwake_battle::round_intent_plan as rip;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::{json, Value};

/// The Arena battle of the first catalog encounter under the timeline, the first hero selected, drawn once.
fn arena() -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    b.selected = b.host.session.as_ref().unwrap().w.allies.first().copied();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

/// Everything a plan and its ghosts show: the host's plan view, each member's picks, ghost entries and round state,
/// and the ghosts waiting for Confirm.
fn board(b: &mut BattleScreen) -> Value {
    let plan = b.host.handle(&json!({"cmd": "plan_view"}));
    let s = b.host.session.as_ref().unwrap();
    let w = &s.w;
    let members: Vec<Value> = w.allies.iter().map(|&m| json!({
        "unit": m,
        "picks": format!("{:?}", rip::picks_of(w, m)),
        "ghost": rip::ghost_entries(w, &s.plan, m),
        "round": format!("{:?}", w.round_states.get(&m)),
    })).collect();
    json!({"plan": plan, "members": members, "deferred": b.deferred.iter().map(|d| d.0).collect::<Vec<_>>()})
}

fn ghost_slot(b: &BattleScreen) -> Action {
    b.w.iter().filter(|w| w.id.starts_with("strip.block")).find_map(|w| w.action.clone().filter(|a| matches!(a, Action::GhostSlot(_)))).expect("a ghost block under the auto-fill")
}

#[test]
fn a_ghost_click_drafts_later_ghosts_on_the_next_frames_and_ends_on_the_same_board() {
    let (mut cx, mut g, mut now) = arena();
    let (mut cx2, _, mut later) = arena();
    assert!(now.host.session.as_ref().unwrap().w.allies.len() > 1, "a party of more than one");
    let slot = ghost_slot(&now);
    assert_eq!(slot, ghost_slot(&later), "the same board");
    let h = now.selected.unwrap();
    // drafted at once (as before ENGINE-58)
    now.act(&mut cx, slot.clone());
    now.settle_ghosts(&cx);
    // drafted on the frames after the click
    later.act(&mut cx2, slot);
    assert!(!later.reghost.is_empty(), "the later members wait for the next frames");
    let s = later.host.session.as_ref().unwrap();
    assert_eq!(format!("{:?}", rip::picks_of(&s.w, h)), format!("{:?}", rip::picks_of(&now.host.session.as_ref().unwrap().w, h)), "the clicked member's change is made in the click");
    let mut frames = 0;
    while !later.reghost.is_empty() {
        later.tick(&mut cx2);
        later.draw(&mut cx2, &mut g, (-500, -500), true);
        frames += 1;
        assert!(frames < 20, "the waiting ghosts are drafted");
    }
    assert_eq!(board(&mut later), board(&mut now), "the same plans and ghosts");
}
