//! PR6-8 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 3-4): the `facilities` world kind and the shop's act supply price.
//! A facility starts at level 1 and is raised one level at a time (`facility::upgrade`: the level's flags, its purse
//! cost and materials, its `set_flag` effects; a refusal changes nothing); the level is saved as `facilities` only when
//! raised; the keeper offers the `facility` service (badge order and the conversation's pane); the passive hooks
//! (`shop_pct` on the keeper's shop prices, `recipe_pct` on the keeper's recipe purse costs, `rest_upkeep` on a campaign
//! rest) add up level by level. `supply_price_mult_by_act` prices supplies at round(base x mult) in the act it names.
//! Every expected number is derived from the fixture's own constants and the shop's prices without the hook.

use aegis_core::json::{self, Dict};
use rpgkit_data::facility::{supply_price_in_act, LEVEL_MIN, SUPPLY_MULT_KEY};
use rpgkit_world::dialogue::{DialogueRunner, OPEN_FACILITIES};
use rpgkit_world::recipe::{self, Pick};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{campaign, explore, facility, shop, Data, GameState};
use serde_json::{json, Value};

const FUNDS: i64 = 1000;
const COST_2: i64 = 40;
const COST_3: i64 = 90;
const TIMBER_2: i64 = 2;
const SHOP_PCT_2: i64 = 80;
const SHOP_PCT_3: i64 = 50;
const RECIPE_PCT_3: i64 = 75;
const RECIPE_GOLD: i64 = 30;
const REST_COST: i64 = 3;
const REST_DELTA_3: i64 = -2;
const GATE: &str = "w.big_job";
const BUNKS: &str = "w.bunks_more";
const SUPPLY: i64 = 8;

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

/// The smith's forge facility: level 2 costs `COST_2` and `TIMBER_2` timber (`shop_pct` `SHOP_PCT_2`, sets `BUNKS`);
/// level 3 costs `COST_3`, opens with `GATE` (`shop_pct` `SHOP_PCT_3`, `recipe_pct` `RECIPE_PCT_3` on `metal`
/// recipes, `rest_upkeep` `REST_DELTA_3`).
fn forge() -> Value {
    json!({"id": "forge", "name": "Forge", "keeper_character": "smith", "levels": [
        {"level": 1, "name": "Cold", "scrap_cost": 0, "effects": []},
        {"level": 2, "name": "Warm", "scrap_cost": COST_2, "materials": {"timber": TIMBER_2},
         "effects": [{"effect": "shop_pct", "pct": SHOP_PCT_2}, {"effect": "set_flag", "key": BUNKS, "value": true}]},
        {"level": 3, "name": "Hot", "scrap_cost": COST_3, "requires_flags": [{"key": GATE, "value": true}],
         "effects": [{"effect": "shop_pct", "pct": SHOP_PCT_3}, {"effect": "recipe_pct", "pct": RECIPE_PCT_3, "recipe_tags": ["metal"]}, {"effect": "rest_upkeep", "delta": REST_DELTA_3}]}
    ]})
}

/// World `w` (a campaign; a rest costs `REST_COST` upkeep): Smith keeps the forge, a shop (a derived-price tonic, a
/// fixed blade, supplies at `SUPPLY`) and two recipes (`temper`, tagged metal, and `mend`, untagged); Clerk keeps
/// nothing.
fn data_with(shop_extra: Value) -> Data {
    let mut shop_row = json!({"id": "stall", "name": "Stall", "keeper": "smith", "stock": [{"item": "tonic", "count": -1, "price": -1}, {"item": "blade", "count": -1, "price": 55}],
        "loot": [], "buy_mult": 1.0, "sell_mult": 0.5, "supply_price": SUPPLY});
    for (k, v) in json::dict(&shop_extra) {
        shop_row[k] = v.clone();
    }
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1},
        "rest": {"days": 1, "cost": REST_COST, "where": "rest_places", "on_enter": false, "needs_upkeep": true}}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub", "rest": true}])),
        "characters": with_ids("w", json!([{"id": "smith", "name": "Smith", "role": "npc"}, {"id": "clerk", "name": "Clerk", "role": "npc"}])),
        "shops": with_ids("w", json!([shop_row])),
        "facilities": with_ids("w", json!([forge()]))
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}],
        "items": [
            {"id": "tonic", "name": "Tonic", "item_type": "CONSUMABLE", "value": 23},
            {"id": "blade", "name": "Blade", "item_type": "MATERIAL", "value": 40},
            {"id": "timber", "name": "Timber", "item_type": "MATERIAL", "value": 2}
        ],
        "recipes": [
            {"id": "temper", "name": "Temper", "npc": "smith", "requires": [], "cost": {"gold": RECIPE_GOLD}, "target": "stash", "tags": ["metal"], "effects": [{"effect": "craft", "item": "blade", "count": 1}]},
            {"id": "mend", "name": "Mend", "npc": "smith", "requires": [], "cost": {"gold": RECIPE_GOLD}, "target": "stash", "effects": [{"effect": "craft", "item": "tonic", "count": 1}]}
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn data() -> Data {
    data_with(json!({}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    gs.party.gold = FUNDS;
    gs.party.supplies = 10;
    gs.party.add_to_stash("timber", TIMBER_2);
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    gs
}

const FQ: &str = "w:forge";

fn prices(offers: &[Dict]) -> Vec<(String, i64)> {
    offers.iter().filter(|o| json::s(o, "kind") != shop::SUPPLIES).map(|o| (json::s(o, "id"), json::i(o, "price", -1))).collect()
}

fn supply_offer(d: &Data, gs: &GameState) -> i64 {
    shop::offers(d, gs, "stall").iter().find(|o| json::s(o, "kind") == shop::SUPPLIES).map_or(-1, |o| json::i(o, "price", -1))
}

fn ceil_pct(price: i64, pct: i64) -> i64 {
    ((price * pct + 99) / 100).max(1)
}

#[test]
fn a_facility_starts_at_level_one_and_is_saved_only_when_raised() {
    let d = data();
    let gs = state(&d);
    assert_eq!(facility::level(&gs, FQ), LEVEL_MIN);
    assert!(gs.facilities.is_empty());
    assert!(!gs.to_value().as_object().unwrap().contains_key("facilities"), "an unraised state writes no facilities key");
    let back = GameState::from_dict(&d, json::dict(&gs.to_value()));
    assert_eq!(back.to_value(), gs.to_value(), "the save reads back byte for byte");
    assert!(back.validate(&d).is_empty(), "{:?}", back.validate(&d));
}

#[test]
fn an_upgrade_pays_its_cost_raises_one_level_and_sets_its_flags() {
    let d = data();
    let mut gs = state(&d);
    let r = facility::upgrade(&d, &mut gs, FQ);
    assert!(json::b(&r, "ok", false), "{r:?}");
    assert_eq!((facility::level(&gs, FQ), gs.party.gold, gs.party.stash.get("timber").copied()), (2, FUNDS - COST_2, None), "level 2 paid with purse and timber");
    assert!(gs.flag_true(BUNKS), "the level's set_flag");
    assert_eq!(gs.facilities.get(FQ).copied(), Some(2));
    // the save carries it and reads back
    let v = gs.to_value();
    assert_eq!(v["facilities"], json!({FQ: 2}));
    let back = GameState::from_dict(&d, json::dict(&v));
    assert_eq!((back.to_value(), back.validate(&d)), (v, Vec::<String>::new()));
    // level 3 is gated: refused, nothing changes
    let before = gs.clone();
    let r = facility::upgrade(&d, &mut gs, FQ);
    assert_eq!(json::sget(&r, "errors"), [facility::NOT_YET_TEXT]);
    assert_eq!(gs, before);
    gs.set_flag(GATE, json!(true));
    // too little purse: refused with the currency's words, nothing changes
    gs.party.gold = COST_3 - 1;
    let before = gs.clone();
    let r = facility::upgrade(&d, &mut gs, FQ);
    assert_eq!(json::sget(&r, "errors"), [rpgkit_world::terms::not_enough_currency(&d, "w")]);
    assert_eq!(gs, before);
    gs.party.gold = COST_3;
    assert!(json::b(&facility::upgrade(&d, &mut gs, FQ), "ok", false));
    assert_eq!((facility::level(&gs, FQ), gs.party.gold), (3, 0));
    let r = facility::upgrade(&d, &mut gs, FQ);
    assert_eq!(json::sget(&r, "errors"), [facility::TOP_TEXT], "no level past the top");
    assert_eq!(json::sget(&facility::upgrade(&d, &mut gs, "w:nothing"), "errors"), [facility::UNKNOWN_TEXT]);
}

#[test]
fn a_save_with_a_bad_level_is_refused() {
    let d = data();
    let gs = state(&d);
    let mut v = gs.to_value();
    v["facilities"] = json!({FQ: 1});
    assert!(GameState::from_dict(&d, json::dict(&v)).validate(&d).iter().any(|e| e.starts_with("facilities.")), "level 1 is never stored");
    v["facilities"] = json!({FQ: "two"});
    assert!(!GameState::from_dict(&d, json::dict(&v)).validate(&d).is_empty());
    v["facilities"] = json!([]);
    assert!(!GameState::from_dict(&d, json::dict(&v)).validate(&d).is_empty());
}

#[test]
fn the_keeper_offers_the_facility_service_and_its_pane() {
    let d = data();
    let mut gs = state(&d);
    let services = explore::npc_services(&d, &gs, "w:smith");
    assert!(services.contains(&"facility") && services.contains(&"shop") && services.contains(&"recipe"), "{services:?}");
    let order: Vec<&str> = explore::SERVICES.iter().copied().filter(|s| services.contains(s)).collect();
    assert_eq!(order, services, "badge order");
    assert!(!explore::npc_services(&d, &gs, "w:clerk").contains(&"facility"));
    let mut r = DialogueRunner::open(&d, &mut gs, "smith");
    let view = r.view(&d, &gs);
    let choices = json::aget(&view, "choices");
    let k = choices.iter().position(|c| json::s(json::dict(c), "service") == "facility").expect("the facility reply");
    let res = r.choose(&d, &mut gs, k as i64);
    assert_eq!(json::s(&res, "facility"), "w:smith", "{res:?}");
    assert_eq!(json::s(&r.view(&d, &gs), "facility"), "w:smith");
    assert!(!json::b(&r.choose(&d, &mut gs, 0), "ok", true), "the conversation waits while the pane is open");
    let back = r.close_facilities(&d, &mut gs);
    assert!(json::b(&back, "ok", false) && !r.view(&d, &gs).contains_key("facility"), "{back:?}");
    // the pane's reading: level 1 of 3, the next level's cost and effects, ready
    let v = facility::view(&d, &gs, "w:smith");
    let row = json::dict(&v[0]);
    assert_eq!((json::i(row, "level", 0), json::i(row, "top", 0), json::b(row, "ready", false)), (1, 3, true));
    let next = json::dget(row, "next");
    assert_eq!(json::sget(next, "cost"), [format!("{COST_2} gold"), format!("Timber x{TIMBER_2}")]);
    assert_eq!(json::sget(next, "effects"), [format!("Shop prices -{}%", 100 - SHOP_PCT_2)], "set_flag shows no line");
    // the runner never takes the open effect from data
    assert_eq!(OPEN_FACILITIES, "open_facilities");
}

#[test]
fn shop_and_recipe_and_rest_hooks_add_up_level_by_level() {
    let d = data();
    let mut gs = state(&d);
    let base = prices(&shop::offers(&d, &gs, "stall"));
    let temper = d.cat.recipes.get("temper").unwrap().clone();
    let mend = d.cat.recipes.get("mend").unwrap().clone();
    assert_eq!((recipe::gold_cost(&d, &gs, &temper), campaign::purse::rest_cost(&d, &gs)), (RECIPE_GOLD, REST_COST), "level 1: no change");
    facility::upgrade(&d, &mut gs, FQ);
    let want2: Vec<(String, i64)> = base.iter().map(|(id, p)| (id.clone(), ceil_pct(*p, SHOP_PCT_2))).collect();
    assert_eq!(prices(&shop::offers(&d, &gs, "stall")), want2, "level 2: shop_pct");
    assert_eq!(supply_offer(&d, &gs), SUPPLY, "the supply price ignores shop_pct");
    assert_eq!(recipe::gold_cost(&d, &gs, &temper), RECIPE_GOLD, "no recipe_pct yet");
    gs.set_flag(GATE, json!(true));
    facility::upgrade(&d, &mut gs, FQ);
    let pct3 = SHOP_PCT_2 * SHOP_PCT_3 / 100;
    let want3: Vec<(String, i64)> = base.iter().map(|(id, p)| (id.clone(), ceil_pct(*p, pct3))).collect();
    assert_eq!(prices(&shop::offers(&d, &gs, "stall")), want3, "level 3: both levels' shop_pct");
    assert_eq!(recipe::gold_cost(&d, &gs, &temper), ceil_pct(RECIPE_GOLD, RECIPE_PCT_3), "a metal recipe");
    assert_eq!(recipe::gold_cost(&d, &gs, &mend), RECIPE_GOLD, "recipe_tags: an untagged recipe keeps its cost");
    assert_eq!(recipe::cost_lines(&d, &gs, &temper), [format!("{} gold", ceil_pct(RECIPE_GOLD, RECIPE_PCT_3))]);
    // buying charges the offered price; running the recipe pays the lowered cost
    gs.party.gold = FUNDS;
    let blade = want3.iter().find(|(id, _)| id == "blade").unwrap().1;
    let r = shop::buy(&d, &mut gs, "stall", "item", "blade", 1);
    assert_eq!((json::i(&r, "price", 0), gs.party.gold), (blade, FUNDS - blade));
    let gold = gs.party.gold;
    let r = recipe::run(&d, &mut gs, "w:smith", "temper", &Pick::default());
    assert!(json::b(&r, "ok", false), "{r:?}");
    assert_eq!(gs.party.gold, gold - ceil_pct(RECIPE_GOLD, RECIPE_PCT_3));
    // the rest spends the lowered upkeep
    assert_eq!(campaign::purse::rest_cost(&d, &gs), (REST_COST + REST_DELTA_3).max(0));
    gs.party.members[0].hp_lost = 1;
    let supplies = gs.party.supplies;
    let r = campaign::purse::rest(&d, &mut gs);
    assert!(json::b(&r, "ok", false), "{r:?}");
    assert_eq!((json::i(&r, "spent", -1), gs.party.supplies), ((REST_COST + REST_DELTA_3).max(0), supplies - (REST_COST + REST_DELTA_3).max(0)));
}

#[test]
fn the_supply_price_follows_the_act_multiplier_in_offers_buy_and_trade() {
    const MULT: f64 = 2.0;
    let d = data_with(json!({SUPPLY_MULT_KEY: {"2": MULT}}));
    let mut gs = state(&d);
    assert_eq!(supply_offer(&d, &gs), SUPPLY, "act 1: not named, the supply price itself");
    gs.campaign.as_mut().unwrap().act = 2;
    let want = json::roundi(SUPPLY as f64 * MULT);
    assert_eq!(supply_offer(&d, &gs), want, "act 2: round(base x mult)");
    assert_eq!(supply_price_in_act(d.resolve("w", "shops", "stall"), 2), want);
    let mut a = gs.clone();
    let r = shop::buy(&d, &mut a, "stall", shop::SUPPLIES, shop::SUPPLIES, 1);
    assert_eq!((json::i(&r, "price", 0), a.party.gold), (want, FUNDS - want), "buy charges it");
    let mut b = gs.clone();
    let r = shop::trade(&d, &mut b, "stall", &[(shop::SUPPLIES.into(), shop::SUPPLIES.into(), 1)], &[]);
    assert_eq!((json::i(&r, "spent", 0), b.party.gold), (want, FUNDS - want), "trade charges it");
    assert_eq!(a, b);
    // the stock prices are unchanged by it
    assert_eq!(prices(&shop::offers(&d, &gs, "stall")), prices(&shop::offers(&data(), &state(&data()), "stall")));
}
