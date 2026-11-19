//! BATTLE-UI2 part 2 (PR7-40, `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` 2, 4, 5): the foe groups open
//! and close on a click (contract 4), the docked card stays on the planning hero's target whatever plate the mouse is
//! over (contract 5), and the tactics are portrait-and-pennant buttons under the round intent (contract 2). Battles
//! are the first catalog encounter, edited in memory (no shipped data changes).

use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

/// An Arena battle of the first catalog encounter under the timeline; `foes` copies of its first enemy when set.
fn arena(foes: Option<usize>) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    if let Some(n) = foes {
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| {
            let row = c.encounters.get_mut(&encounter).expect("the row");
            let first = row["enemies"][0].clone();
            row.insert("enemies".into(), json!(vec![first; n]));
        });
    }
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

fn has(b: &BattleScreen, id: &str) -> bool {
    b.w.iter().any(|w| w.id == id)
}

/// Contract 4: from 11 foes the identical ones group; a click opens the group into a compact plate each, a second
/// click (on the open group's header) closes it again.
#[test]
fn a_group_opens_on_a_click_and_closes_on_the_second() {
    let n = crate::battle_view::GROUP_FROM + 1;
    let (mut cx, mut g, mut b) = arena(Some(n));
    let on_field: Vec<usize> = b.session().w.enemies.iter().copied().filter(|&e| b.session().w.grid.pos(e).is_some()).collect();
    assert!(on_field.len() >= crate::battle_view::GROUP_FROM, "the battle fields {} foes", on_field.len());
    let group = b.w.iter().find(|w| w.id.starts_with("foe.group.")).expect("a group row").clone();
    let key = match group.action.clone() {
        Some(Action::FoeGroup(k)) => k,
        other => panic!("a group click opens it, not {other:?}"),
    };
    let plates = |b: &BattleScreen| on_field.iter().filter(|e| has(b, &format!("foe.{e}"))).count();
    let closed = plates(&b);
    assert!(closed < on_field.len(), "a closed group hides its plates");
    b.act(&mut cx, Action::FoeGroup(key.clone()));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(plates(&b) > closed, "the open group shows its foes' plates");
    let header = b.w.iter().find(|w| w.id.starts_with("foe.group.")).expect("the open group's header");
    assert_eq!(header.action, Some(Action::FoeGroup(key.clone())), "the header closes it");
    assert!(header.rect.h < crate::battle::plates::COMPACT_H, "the header is slim");
    b.act(&mut cx, Action::FoeGroup(key));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(plates(&b), closed, "closed again");
}

/// Contract 5: while a hero plans, the card shows the hero's target and stays there with the mouse over another foe's
/// plate.
#[test]
fn the_card_stays_on_the_planning_heros_target() {
    let (mut cx, mut g, mut b) = arena(None);
    let h = b.selected.expect("a hero");
    let foes: Vec<usize> = b.session().w.enemies.iter().copied().filter(|&e| b.session().w.units[e].is_active() && b.session().w.grid.pos(e).is_some()).collect();
    assert!(foes.len() >= 2, "two foes");
    let (target, other) = (foes[0], foes[1]);
    assert_eq!(b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": target}))["ok"], json!(true));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let plate = b.w.iter().find(|w| w.id == format!("foe.{other}")).expect("the other foe's plate").rect;
    b.draw(&mut cx, &mut g, (plate.x + plate.w / 2, plate.y + plate.h / 2), true);
    let card = b.w.iter().find(|w| w.id == "intel.unit" || w.id == "card.unit").expect("the card");
    assert_eq!(card.text, b.session().w.units[target].name, "the card keeps the hero's target");
}

/// Contract 2: under the round intent the tactics are square portrait-and-pennant buttons (their widgets keep the
/// tactic's words) and Auto a play button.
#[test]
fn the_tactics_are_pennant_buttons_and_auto_a_play_button() {
    let (_cx, _g, b) = arena(None);
    let party = b.w.iter().find(|w| w.id == "strip.party").expect("the party tactic");
    let hero = b.w.iter().find(|w| w.id == "strip.hero").expect("the hero tactic");
    let auto = b.w.iter().find(|w| w.id == "strip.auto").expect("Auto");
    assert!(party.text.starts_with("Party: ") && hero.text.starts_with("Hero: "), "{} / {}", party.text, hero.text);
    assert_eq!((party.rect.w, hero.rect.w, auto.rect.w), (super::super::draw::TACTIC_W, super::super::draw::TACTIC_W, super::super::draw::TACTIC_W));
    assert!(party.rect.x + party.rect.w <= hero.rect.x && hero.rect.x + hero.rect.w <= auto.rect.x);
    assert_eq!(auto.action, Some(Action::Autoplay));
}
