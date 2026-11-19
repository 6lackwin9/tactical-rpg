//! PASSIVE-ICON1 (ENGINE-24, `docs/specs/STATUSVIS1.md` SV10, SV12): a hero's slotted loadout passives show as two
//! small icons under the portrait on its plate, apart from the status column, each with the passive's name on its
//! hover card and its archetype's icon file (`passive_icon::of`). The battle is the first catalog encounter under the
//! timeline (no shipped data changes).

use crate::battle::BattleScreen;
use crate::screens::Screen;
use crate::battle_view::{plate_passives, PLATE_PASSIVES};
use emberwake_data::passive_icon;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

#[test]
fn a_heros_slotted_passives_show_under_its_portrait() {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    let slotted: Vec<String> = cx.data.cat.passives.values().filter(|p| p.slot == "passive").take(PLATE_PASSIVES + 1).map(|p| p.id.clone()).collect();
    assert_eq!(slotted.len(), PLATE_PASSIVES + 1, "the catalog has slot passives");
    let sess = b.host.session.as_mut().expect("a session");
    let hero = sess.w.allies[0];
    assert!(sess.w.units[hero].deck_rule.is_some(), "the timeline has the deck rule");
    sess.w.units[hero].deck.passive = slotted.clone();
    let shown = plate_passives(&sess.w, hero);
    assert_eq!(shown.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), slotted[..PLATE_PASSIVES].iter().map(String::as_str).collect::<Vec<_>>(), "the first two slots");
    for p in &shown {
        let def = &cx.data.cat.passives[&p.id];
        assert_eq!(p.icon, passive_icon::path(&passive_icon::of(def)), "{}: its archetype's file", p.id);
        assert!(!def.icon.is_empty(), "{}: the shipped passive names its archetype", p.id);
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let rect = |id: &str| b.w.iter().find(|w| w.id == id).map(|w| w.rect);
    let portrait = rect(&format!("party.{hero}.portrait")).expect("the portrait");
    let plate = rect(&format!("party.{hero}")).unwrap_or(portrait);
    for (i, p) in shown.iter().enumerate() {
        let id = format!("party.{hero}.passive.{i}");
        let w = b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no {id}"));
        assert_eq!(w.text, p.name);
        assert!(w.tip.iter().any(|t| t.0 == p.name), "the hover card names it: {:?}", w.tip);
        let r = w.rect;
        assert!(r.y >= portrait.y + portrait.h && r.x >= portrait.x && r.x + r.w <= portrait.x + portrait.w, "{id} under the portrait: {r:?} {portrait:?}");
        assert!(r.y + r.h <= plate.y + plate.h || plate == portrait, "{id} inside the plate");
    }
    assert!(rect(&format!("party.{hero}.passive.{PLATE_PASSIVES}")).is_none(), "only the loadout's two slots");
    // a foe shows none
    let foe = b.session().w.enemies[0];
    assert!(rect(&format!("foe.{foe}.passive.0")).is_none());
}
