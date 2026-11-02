//! ENCOUNTER2-U (ENCOUNTER2 H1, H3, H7, S2): the hold escape's screen on the kit sample `kit_winch_escape` (a catalog
//! encounter). Every expectation is the kit's (`HoldGate`, `Session::needs_stay_pick`, `gate_note`, `final_outcome`);
//! classic and every fight without a hold escape draw as before.

use crate::battle::marks::{exit_sign, hold_line, hold_pick_ids, HoldMarks};
use crate::battle::overlays::Layers;
use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use aegis_ui::ui::{Gfx, Widget};
use emberwake_battle::grid::V2;
use emberwake_battle::session::{Phase, Session};
use aegis_ui::art::Art;
use emberwake_battle::world::World;
use serde_json::{json, Value};
const ENC: &str = "kit_winch_escape";

fn screen(encounter: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let b = BattleScreen::arena_with(&cx, None, encounter, None, "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn widget<'a>(b: &'a BattleScreen, id: &str) -> Option<&'a Widget<Action>> {
    b.w.iter().find(|x| x.id == id)
}

fn tip_text(b: &BattleScreen, id: &str) -> String {
    b.w.iter().filter(|x| x.id == id).flat_map(|x| x.tip.iter().map(|(t, _)| t.clone())).collect::<Vec<_>>().join(" | ")
}

fn exit_tile(w: &World) -> V2 {
    *w.grid.exits.iter().find(|c| w.grid.is_valid(**c)).expect("an exit")
}

/// H1: before the pick the panel offers each hero on the field, Confirm waits, the objective line asks for the pick
/// and the gate is shut; a pick names the holder in the objective line and the hold tiles' cards, frees Confirm and
/// closes the panel. H3: the hero panel shows the kit's gate note while the gate is shut.
#[test]
fn the_pick_comes_first_then_names_the_holder() {
    let (mut cx, mut g, mut b) = screen(ENC);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let gate = s(&b).w.hold.clone().expect("a hold escape");
    assert!(s(&b).needs_stay_pick());
    let offered: Vec<usize> = gate.heroes.iter().copied().filter(|&c| s(&b).w.has(c)).collect();
    for &c in &offered {
        assert!(widget(&b, &format!("hold.pick.{c}")).is_some_and(|x| x.enabled), "hero {c} offered");
    }
    assert!(!widget(&b, "strip.confirm").unwrap().enabled, "Confirm waits for the pick");
    assert!(b.message.contains(&format!("Pick who holds {} first.", gate.label)), "{}", b.message);
    let exit = exit_tile(&s(&b).w);
    assert!(tip_text(&b, &format!("tile.{}.{}", exit.x, exit.y)).contains("shut"));
    let st = b.state(&cx);
    assert_eq!((st["hold"]["needs_pick"].clone(), st["hold"]["open"].clone()), (json!(true), json!(false)));

    let pick = offered[0];
    b.run(&mut cx, Action::StayPick(pick));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(s(&b).w.hold.as_ref().unwrap().holder, Some(pick));
    assert!(b.message.contains(&hold_line(s(&b))) && !hold_line(s(&b)).is_empty(), "{}", b.message);
    assert!(widget(&b, &format!("hold.pick.{pick}")).is_none() && widget(&b, "hold.pick").is_none());
    assert!(widget(&b, "strip.confirm").unwrap().enabled);
    let tile = gate.tiles[0];
    assert!(tip_text(&b, &format!("tile.{}.{}", tile.x, tile.y)).contains(&format!("{} holds it", s(&b).w.units[pick].name)));
    let note = s(&b).gate_note();
    b.selected = Some(pick);
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    let words: String = recs.iter().filter(|r| r.x < 126.0).map(|r| r.text.clone()).collect::<Vec<_>>().join(" ");
    assert_eq!(words.contains("The gate is shut"), !note.is_empty(), "the hero panel shows the kit's note {note:?}: {words}");
    assert_eq!(b.state(&cx)["hold"]["holder"], json!(pick));
}

/// H2, H7: with the holder on a hold tile the gate opens (the exit's card says so and the sign reads OPEN); with the
/// `hold` and `gate` layers off the cards and the frame lose the marks.
#[test]
fn the_gate_follows_the_holder_and_the_layers_switch_it() {
    let (mut cx, mut g, mut b) = screen(ENC);
    let pick = hold_pick_ids(s(&b))[0];
    b.run(&mut cx, Action::StayPick(pick));
    let tile = s(&b).w.hold.as_ref().unwrap().tiles[0];
    {
        let w = &mut b.host.session.as_mut().unwrap().w;
        if let Some(o) = w.occupant(tile) {
            w.grid.positions.swap(o, pick);
        } else {
            w.grid.positions[pick] = Some(tile);
        }
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    assert!(w.hold.as_ref().unwrap().is_open(w), "the holder on its tile opens the gate");
    let exit = exit_tile(w);
    assert!(tip_text(&b, &format!("tile.{}.{}", exit.x, exit.y)).contains("The gate is open."));
    assert_eq!(exit_sign(HoldMarks::of(s(&b), &Layers::of(&Value::Null)).as_ref()).0, "OPEN");
    let on = g.canvas.px.clone();
    cx.settings.v["battle_overlays"] = json!({"hold": false, "gate": false});
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_ne!(on, g.canvas.px);
    assert!(!tip_text(&b, &format!("tile.{}.{}", exit.x, exit.y)).contains("gate"));
    assert!(!tip_text(&b, &format!("tile.{}.{}", tile.x, tile.y)).contains("holds it"));
    assert_eq!(exit_sign(HoldMarks::of(s(&b), &Layers::of(&cx.settings.v["battle_overlays"])).as_ref()).0, "EXIT");
}

/// S2: the screen's outcome is the session's `final_outcome`: after the last round a `timeout_outcome` of lost is
/// honoured (the old screen rule read fled).
#[test]
fn the_outcome_is_the_sessions_final_outcome() {
    let (_cx, _g, mut b) = screen(ENC);
    assert_eq!(b.outcome(), s(&b).final_outcome());
    let sess = b.host.session.as_mut().unwrap();
    sess.timeout_outcome = "lost".into();
    sess.phase = Phase::Replay;
    sess.current_round = sess.def.max_rounds;
    assert!(!s(&b).can_advance_round());
    assert_eq!((b.outcome(), s(&b).final_outcome()), ("lost".to_string(), "lost".to_string()));
}

/// Classic and a fight without a hold escape: no pick panel, no `hold` key, exits read EXIT.
#[test]
fn without_a_hold_escape_nothing_changes() {
    let mut cx = crate::hud::hud_tests::game_cx();
    let encounter = cx.data.cat.encounters.keys().find(|k| k.as_str() != ENC).cloned().unwrap();
    cx.settings.v["battle_view"] = json!("flat");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, None, "", false);
    let mut g = crate::hud::hud_tests::gfx();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    if s(&b).w.hold.is_some() {
        return;
    }
    assert!(widget(&b, "hold.pick").is_none() && b.state(&cx).get("hold").is_none());
    assert!(HoldMarks::of(s(&b), &Layers::of(&Value::Null)).is_none());
    assert_eq!(exit_sign(None).0, "EXIT");
}

/// Decision "RULESET1-A3 loose ends": the battle screen's combat log panel reads the kit's `Session::log_lines`, so
/// an encounter line (a phase's `line`, a phase kill; the kit test `phase_kill.rs` says them) shows in it where it
/// was said, while the resolver log stays as it is.
#[test]
fn phase_lines_reach_the_combat_log_panel() {
    let (mut cx, mut g, mut b) = screen(ENC);
    let line = "[Phase] Test: The first captive falls.".to_string();
    b.host.session.as_mut().unwrap().encounter_lines.push((0, line.clone()));
    b.show_log = true;
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    assert!(recs.iter().any(|r| r.text.starts_with("[Phase] Test")), "the panel draws the phase line");
    assert!(widget(&b, "log.panel").is_some_and(|x| x.text == format!("{} lines", s(&b).resolver.log.len() + 1)));
    assert!(!s(&b).resolver.log.contains(&line));
}
