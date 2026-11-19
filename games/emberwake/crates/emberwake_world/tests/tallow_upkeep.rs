//! PR7-42 (PACE1, PT-19) on the shipped data (read only): the Act 1 upkeep has teeth. A shop of the world sells the
//! upkeep for the currency at its `supply_price`; with none left a rest (Make camp, the hub's rest point) is refused
//! when the rest rule needs upkeep and allowed again after buying one; at zero the fights' start-AP change is what the
//! world map shows (`campaign::purse::starved_ap`), and a trip that empties the purse shows it too. The shop, the
//! price, the rest place and the penalty are found by rule.

use aegis_core::json;
use emberwake_world::campaign::purse;
use emberwake_world::{new_game, progression, shop, Data, GameState};
use std::path::Path;

const WORLD: &str = "sunk_ward";

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

/// A new campaign game standing in a rest place of the world, hurt, with no upkeep and `gold` currency.
fn hurt_at_rest(d: &Data, gold: i64) -> GameState {
    let mut gs = new_game::create_for_world(d, WORLD, 1).expect("a new game");
    assert!(gs.campaign.is_some(), "{WORLD} runs a campaign");
    let rest = d.catalogs.own_entries(WORLD, "locations").iter().map(|l| json::s(l, "id")).find(|id| progression::is_rest_place(d, &gs, id)).expect("a rest place");
    gs.location_id = json::s(emberwake_world::explore::resolve_location(d, &gs, &rest), "_qid");
    gs.party.members[0].hp_lost = 1;
    gs.party.supplies = 0;
    gs.party.gold = gold;
    gs
}

/// The world's shop selling the upkeep (by rule: a positive `supply_price`) and its price now.
fn upkeep_shop(d: &Data, gs: &GameState) -> (String, i64) {
    for s in d.catalogs.own_entries(WORLD, "shops") {
        let id = json::s(s, "id");
        if let Some(o) = shop::offers(d, gs, &id).into_iter().find(|o| json::s(o, "kind") == shop::SUPPLIES) {
            return (id, json::i(&o, "price", 0));
        }
    }
    panic!("a {WORLD} shop sells the upkeep");
}

#[test]
fn the_upkeep_is_bought_and_a_rest_at_zero_is_refused() {
    let d = data();
    let rule = purse::rest_rule(&d, WORLD);
    assert!(rule.needs_upkeep, "{WORLD}'s rest needs upkeep");
    let probe = hurt_at_rest(&d, 0);
    let (shop_id, price) = upkeep_shop(&d, &probe);
    assert!(price > 0, "{shop_id} sells the upkeep for currency");
    let cost = purse::rest_cost(&d, &probe);
    assert!(cost > 0, "a rest costs upkeep");
    let mut gs = hurt_at_rest(&d, price * cost);
    assert_eq!(progression::can_camp(&d, &gs), purse::REFUSE_UPKEEP, "no rest at 0 upkeep");
    let refused = progression::make_camp(&d, &mut gs);
    assert!(!json::b(&refused, "ok", false), "Make camp is refused at 0");
    assert_eq!(gs.party.members[0].hp_lost, 1, "nobody healed");
    let short = shop::buy(&d, &mut hurt_at_rest(&d, price - 1), &shop_id, shop::SUPPLIES, shop::SUPPLIES, 1);
    assert!(!json::b(&short, "ok", false), "one short of the price buys nothing");
    let bought = shop::buy(&d, &mut gs, &shop_id, shop::SUPPLIES, shop::SUPPLIES, cost);
    assert!(json::b(&bought, "ok", false), "{:?}", json::sget(&bought, "errors"));
    assert_eq!(gs.party.supplies, cost, "bought {cost}");
    assert_eq!(gs.party.gold, 0, "for {price} each");
    assert_eq!(progression::can_camp(&d, &gs), "", "with the upkeep the rest is allowed");
    assert!(json::b(&progression::make_camp(&d, &mut gs), "ok", false));
    assert_eq!((gs.party.members[0].hp_lost, gs.party.supplies), (0, 0), "healed for the upkeep");
}

#[test]
fn zero_upkeep_shows_its_fight_penalty() {
    let d = data();
    let gs = hurt_at_rest(&d, 0);
    let ap = purse::purse(&d, WORLD).start_ap as i64;
    assert!(ap < 0, "{WORLD}'s fights start with less AP at zero upkeep");
    assert_eq!(purse::starved_ap(&d, &gs, gs.party.supplies), ap, "the legend's penalty at 0");
    assert_eq!(purse::battle_mods(&d, &gs).start_ap, ap, "the same as the battle's");
    let mut full = gs.clone();
    full.party.supplies = 3;
    assert_eq!(purse::starved_ap(&d, &full, purse::upkeep_after(&full, 2)), 0, "a trip that leaves some: no penalty");
    assert_eq!(purse::starved_ap(&d, &full, purse::upkeep_after(&full, 3)), ap, "a trip that empties it: the penalty");
    assert_eq!(purse::starved_ap(&d, &full, purse::upkeep_after(&full, 5)), ap, "a trip longer than the purse: the penalty");
}
