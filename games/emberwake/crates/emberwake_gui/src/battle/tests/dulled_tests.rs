//! PR7-3 (review N3, `docs/sessions/PK2_GRAFTS.md` 7) battle GUI audit: a party member whose tracked weapon is at 0
//! durability (dulled) shows a `Dulled` badge on its card (widget `party.{unit}.dulled`, its tip the penalty from
//! `DULLED_POWER_MULT`) and the penalty line in its card tip; a weapon with durability left, and a unit that tracks
//! no wear, show neither. The wear is set on the session (the battle's own `Wear`), so no shipped number is assumed.

use crate::battle::BattleScreen;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_rules::gear::{Slot, Wear};
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

fn setup() -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.values().find_map(|e| e.get("id").and_then(|v| v.as_str()).map(str::to_string)).expect("an encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn penalty_line(weapon: &str) -> String {
    format!("{weapon} dulled: -{:.0}% weapon power until repaired.", (1.0 - emberwake_rules::gear::DULLED_POWER_MULT) * 100.0)
}

fn card_tip(b: &BattleScreen, unit: usize) -> Vec<String> {
    b.w.iter().find(|w| w.id == format!("party.{unit}")).expect("party card").tip.iter().map(|x| x.0.clone()).collect()
}

fn badge(b: &BattleScreen, unit: usize) -> Option<Vec<String>> {
    b.w.iter().find(|w| w.id == format!("party.{unit}.dulled")).map(|w| w.tip.iter().map(|x| x.0.clone()).collect())
}

#[test]
fn a_dulled_weapon_shows_its_badge_and_tip_only_when_dulled() {
    let (mut cx, mut g, mut b) = setup();
    let hero = b.host.session.as_ref().unwrap().w.allies[0];
    let weapon = b.host.session.as_ref().unwrap().w.units[hero].inventory.get(Slot::MainHand).map_or_else(|| "Weapon".to_string(), |e| e.item.name.clone());
    let line = penalty_line(&weapon);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(badge(&b, hero).is_none() && !card_tip(&b, hero).contains(&line), "no wear tracked: no badge, no line");
    // durability left: worn but not dulled
    b.host.session.as_mut().unwrap().w.units[hero].wear.insert(Slot::MainHand, Wear { start: 3, spent: 2 });
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(badge(&b, hero).is_none() && !card_tip(&b, hero).iter().any(|l| l.contains("dulled")), "durability left: not dulled");
    // spent to 0: dulled
    b.host.session.as_mut().unwrap().w.units[hero].wear.insert(Slot::MainHand, Wear { start: 3, spent: 3 });
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(badge(&b, hero), Some(vec!["Weapon dulled".to_string(), line.clone()]), "the badge names the penalty");
    assert!(card_tip(&b, hero).contains(&line), "the card tip names the penalty: {:?}", card_tip(&b, hero));
    for &other in b.host.session.as_ref().unwrap().w.allies.iter().filter(|&&a| a != hero) {
        assert!(badge(&b, other).is_none(), "only the dulled unit carries the badge");
    }
}
