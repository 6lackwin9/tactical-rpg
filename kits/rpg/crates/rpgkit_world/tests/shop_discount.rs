//! ACT1-HOOKS1 (FB-241 c): a shop's flag-gated `discount` `{"requires_flags", "mult"}`: while every condition holds, each
//! stock and loot-for-sale price is ceil(price x mult) (at least 1); the supply price stays fixed (it ignores every
//! multiplier) and the sell prices are unchanged. Buying charges the price offered. Every expected price is derived from
//! the shop's own prices without the flag.

use aegis_core::json::{self, Dict};
use rpgkit_world::shop;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{Data, GameState};
use serde_json::{json, Value};

const FLAG: &str = "w.friend";
const MULT_PCT: i64 = 80;

fn with_ids(w: &str, rows: Value) -> Value {
    Value::Array(
        json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = json::s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect(),
    )
}

/// World `w`: the smith's shop `forge` (a derived-price tonic, a fixed-price blade, a 1-gold pebble, loot for sale and
/// supplies) with the discount `MULT_PCT` % while `FLAG` holds, and `plain` (the same stock, `{}`).
fn data() -> Data {
    let stock = json!([{"item": "tonic", "count": -1, "price": -1}, {"item": "blade", "count": -1, "price": 55}, {"item": "pebble", "count": -1, "price": 1}]);
    let shops = json!([
        {"id": "forge", "name": "Forge", "keeper": "smith", "stock": stock, "loot": [{"loot": "relic", "price": 33}], "buy_mult": 1.0, "sell_mult": 0.5, "supply_price": 7,
         "discount": {"requires_flags": [{"key": FLAG, "value": true}], "mult": MULT_PCT as f64 / 100.0}},
        {"id": "plain", "name": "Plain", "keeper": "smith", "stock": stock, "loot": [], "buy_mult": 1.0, "sell_mult": 0.5, "supply_price": 7, "discount": {}}
    ]);
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero"]}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub"}])),
        "characters": with_ids("w", json!([{"id": "smith", "name": "Smith", "role": "npc"}])),
        "loot": with_ids("w", json!([{"id": "relic", "ref": "blade", "reward_kind": "item", "count": 1}])),
        "shops": with_ids("w", shops)
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}],
        "items": [
            {"id": "tonic", "name": "Tonic", "item_type": "CONSUMABLE", "value": 23},
            {"id": "blade", "name": "Blade", "item_type": "MATERIAL", "value": 40},
            {"id": "pebble", "name": "Pebble", "item_type": "MATERIAL", "value": 1}
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    party.gold = 1000;
    party.stash.insert("blade".into(), 2);
    GameState::create(d, "w", party)
}

fn prices(offers: &[Dict]) -> Vec<(String, String, i64)> {
    offers.iter().map(|o| (json::s(o, "kind"), json::s(o, "id"), json::i(o, "price", -1))).collect()
}

#[test]
fn the_discount_lowers_stock_and_loot_prices_only_while_its_flags_hold() {
    let d = data();
    let mut gs = state(&d);
    let full = prices(&shop::offers(&d, &gs, "forge"));
    let no_loot: Vec<_> = full.iter().filter(|p| p.0 != "loot").cloned().collect();
    assert_eq!(no_loot, prices(&shop::offers(&d, &gs, "plain")), "flag unset: the prices of the shop with no discount");
    assert_eq!(shop::discount_pct(&gs, d.resolve("w", "shops", "forge")), 100, "flag unset: 100 %");
    let sells = shop::sell_list(&d, &gs, "forge");
    gs.set_flag(FLAG, Value::Bool(true));
    assert_eq!(shop::discount_pct(&gs, d.resolve("w", "shops", "forge")), MULT_PCT);
    let cut = prices(&shop::offers(&d, &gs, "forge"));
    assert_eq!(cut.len(), full.len());
    for ((kind, id, was), (_, _, now)) in full.iter().zip(&cut) {
        let want = if kind == shop::SUPPLIES { *was } else { ((was * MULT_PCT + 99) / 100).max(1) };
        assert_eq!(*now, want, "{kind} {id}: {was} -> {now}");
    }
    assert!(full.iter().zip(&cut).any(|(a, b)| b.2 < a.2), "some price fell");
    assert_eq!(shop::sell_list(&d, &gs, "forge"), sells, "sell prices unchanged");
    assert_eq!(prices(&shop::offers(&d, &gs, "plain")), prices(&shop::offers(&d, &state(&d), "plain")), "a shop with {{}} ignores the flag");
}

#[test]
fn buying_charges_the_discounted_price() {
    let d = data();
    let mut gs = state(&d);
    gs.set_flag(FLAG, Value::Bool(true));
    let tonic = shop::offers(&d, &gs, "forge").into_iter().find(|o| json::s(o, "id") == "tonic").expect("tonic");
    let price = json::i(&tonic, "price", 0);
    let gold = gs.party.gold;
    let res = shop::buy(&d, &mut gs, "forge", "item", "tonic", 2);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!((json::i(&res, "price", 0), gs.party.gold), (price * 2, gold - price * 2));
    let trade = shop::trade(&d, &mut gs, "forge", &[("item".into(), "tonic".into(), 1)], &[]);
    assert!(json::b(&trade, "ok", false), "{trade:?}");
    assert_eq!(gs.party.gold, gold - price * 3, "the trade pays the same price");
}

#[test]
fn a_discounted_price_rounds_up_and_never_reaches_zero() {
    assert_eq!(shop::discounted(10, 80), 8);
    assert_eq!(shop::discounted(23, 80), 19, "18.4 rounds up");
    assert_eq!(shop::discounted(1, 10), 1, "at least 1");
    assert_eq!(shop::discounted(55, 100), 55, "100 %: unchanged");
}
