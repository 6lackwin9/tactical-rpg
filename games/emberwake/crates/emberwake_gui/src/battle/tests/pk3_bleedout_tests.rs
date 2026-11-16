//! PK3 battle GUI tests: bleed-out tooltip & party card banner, and clicking a downed bleeding ally's party card
//! or unoccupied token targets it for revive rather than switching selection to a downed unit or failing behind an
//! occupying token on the board.

use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use aegis_ui::theme::th;
use aegis_ui::ui::Gfx;
use emberwake_battle::round_intent::RoundTarget;
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

fn setup_bleed_battle() -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    // Pick a catalog encounter with >= 2 party members.
    let encounter = cx
        .data
        .cat
        .encounters
        .values()
        .find(|e| e.get("party").and_then(|p| p.as_array()).is_some_and(|a| a.len() >= 2))
        .and_then(|e| e.get("id").and_then(|v| v.as_str()))
        .unwrap_or("courtyard_skirmish")
        .to_string();
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    {
        let s = b.host.session.as_mut().unwrap();
        s.w.bleed_out_rounds = 3;
        s.w.bleed_heroes = s.w.allies.clone();
        // Ensure living hero (index 0) knows Rally, meets level/active-family checks, and carries Smelling Salts.
        s.w.units[0].stats.level = s.w.units[0].stats.level.max(3);
        s.w.units[0].ap = s.w.units[0].ap.max(6);
        if !s.w.units[0].known_skills.iter().any(|x| x == "rally") {
            s.w.units[0].known_skills.push("rally".into());
        }
        let fam = s.w.cat.family_of("rally").to_string();
        if !s.w.units[0].active_class_families.is_empty() && !s.w.units[0].active_families().contains(&fam) {
            s.w.units[0].active_extra_families.push(fam);
        }
        assert!(s.w.units[0].can_use(&s.w.cat, s.w.cat.skill("rally").unwrap()), "hero 0 must be able to cast Rally");
        if let Some(salts) = cx.data.cat.items.get("smelling_salts") {
            s.w.units[0].inventory.add_to_pouch(salts);
            s.w.add_item_skills();
        }
        // Knock down ally 1 so it enters 3-round bleed-out, and move an enemy onto ally 1's tile to verify card bypasses.
        let ally = s.w.allies[1];
        s.w.damage(ally, 9999, true);
        if let (Some(foe), Some(pos)) = (s.w.enemies.first().copied(), s.w.grid.positions[ally]) {
            s.w.grid.positions[foe] = Some(pos);
        }
    }
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn card_tips(b: &BattleScreen, unit: usize) -> Vec<(String, aegis_ui::canvas::Rgba)> {
    b.w.iter().find(|w| w.id == format!("party.{unit}")).expect("party card").tip.clone()
}

#[test]
fn downed_bleed_tooltip_shows_countdown_and_danger_tone_at_one_round() {
    let (mut cx, mut g, mut b) = setup_bleed_battle();
    let ally = s(&b).w.allies[1];
    b.selected = Some(s(&b).w.allies[0]);

    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(card_tips(&b, ally).contains(&("Bleeds out in 3 rounds".into(), th().dim)));

    b.host.session.as_mut().unwrap().w.bleed_remaining.insert(ally, 1);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(card_tips(&b, ally).contains(&("Bleeds out in 1 round".into(), th().foe)), "1 round uses danger foe tone");

    {
        let sess = b.host.session.as_mut().unwrap();
        sess.w.bleed_remaining.shift_remove(&ally);
        sess.w.dead.push(ally);
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(card_tips(&b, ally).contains(&("Dead".into(), th().foe)), "bled out unit reads Dead");
}

#[test]
fn clicking_downed_bleeding_party_card_or_token_targets_for_revive() {
    let (mut cx, mut g, mut b) = setup_bleed_battle();
    let healer = s(&b).w.allies[0];
    let ally = s(&b).w.allies[1];
    assert_eq!(s(&b).phase, Phase::Planning);

    // Select healer, then click downed ally's party card even while an enemy occupies its board tile.
    b.selected = Some(healer);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    b.act(&mut cx, Action::Select(ally));

    assert_eq!(b.selected, Some(healer), "must not switch selection to downed ally");
    assert_eq!(s(&b).w.round_states.get(&healer).and_then(|r| r.target), Some(RoundTarget::Unit(ally)), "party card click sets downed ally as Support target");
    let popup = b.popup.as_ref().expect("revive skill menu opens on downed ally");
    assert!(popup.rows.iter().any(|r| r.valid && r.label.contains("Urgent Triage")), "popup offers Urgent Triage revive skill: {:?}", popup.rows.iter().map(|r| &r.label).collect::<Vec<_>>());

    // Now move the enemy off ally's tile and test clicking the board token (`UnitClick`) without Ctrl.
    b.popup = None;
    {
        let sess = b.host.session.as_mut().unwrap();
        let foe = sess.w.enemies[0];
        sess.w.grid.positions[foe] = Some(emberwake_battle::grid::V2::new(8, 4));
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let unit_widget = b.w.iter().find(|w| w.id == format!("unit.{ally}")).expect("unoccupied downed unit marker pushes unit widget");
    assert_eq!(unit_widget.action, Some(Action::UnitClick(ally)));
    b.act(&mut cx, Action::UnitClick(ally));

    assert_eq!(b.selected, Some(healer), "board token click must not switch selection to downed ally");
    let popup2 = b.popup.as_ref().expect("board token click opens revive menu");
    assert!(popup2.rows.iter().any(|r| r.valid && !rip::picks_of(&s(&b).w, healer).contains(&emberwake_battle::round_intent::Pick::Rest)));
}
