//! PLAYUI1-P: `shop::trade` (games/emberwake/docs/specs/PLAYUI1.md 4.4), one whole trade settled at once. Checked
//! against today's `buy` and `sell` (a one-line trade leaves the same state) and the trade rules (net gold, summed
//! counts, all-or-nothing, refusal reasons). Read-only over the shipped data: the party's gold and stash are set in
//! memory and the expectations come from the shop's own offers and sell list, not from content numbers.

use aegis_core::json::{self, Dict};
use emberwake_world::state::{GOLD_MAX, SUPPLIES_MAX};
use emberwake_world::{new_game, shop, Data, GameState};
use serde_json::Value;
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// The `==` conditions of a `requires_flags` list set in memory, so the stock or loot behind them is offered.
fn meet(gs: &mut GameState, raw: &Value) {
    for c in json::arr(&json::normalize(raw)) {
        let c = json::dict(c);
        if json::s_or(c, "op", "==") == "==" {
            gs.set_flag(&json::s(c, "key"), c.get("value").cloned().unwrap_or(Value::Bool(true)));
        }
    }
}

/// (game, shop qid) for every shop open at the start of a playable world, and again with the flags its stock and loot
/// ask for set (more stock and the loot offers).
fn open_shops(d: &Data) -> Vec<(GameState, String)> {
    let mut out = Vec::new();
    for w in new_game::playable_worlds(d) {
        let gs = new_game::create_for_world(d, &w, 1).expect("a new game");
        for sh in d.merged_entries(&gs.world_id, "shops") {
            let q = json::s(sh, "_qid");
            if !shop::is_open(d, &gs, &q) {
                continue;
            }
            out.push((gs.clone(), q.clone()));
            let mut all = gs.clone();
            for st in json::arr(&json::normalize(json::get(sh, "stock"))) {
                meet(&mut all, json::get(json::dict(st), "requires_flags"));
            }
            for l in json::arr(&json::normalize(json::get(sh, "loot"))) {
                let loot = d.resolve(&gs.world_id, "loot", &json::s(json::dict(l), "loot"));
                meet(&mut all, json::get(loot, "requires_flags"));
            }
            if all != gs && shop::is_open(d, &all, &q) {
                out.push((all, q));
            }
        }
    }
    assert!(!out.is_empty(), "no shop is open at the start of any playable world: nothing to check");
    out
}

/// Every catalog item the shop buys, put in the stash `n` times each (the stash order is the catalog's).
fn stock_stash(d: &Data, gs: &mut GameState, q: &str, n: i64) {
    for id in d.cat.items.keys() {
        gs.party.add_to_stash(id, n);
    }
    let keep: Vec<String> = shop::sell_list(d, gs, q).iter().map(|r| json::s(r, "item")).collect();
    gs.party.stash.retain(|id, _| keep.contains(id));
}

fn ok(r: &Dict) -> bool {
    json::b(r, "ok", false)
}

fn errors(r: &Dict) -> Vec<String> {
    json::sget(r, "errors")
}

fn buy_line(o: &Dict, n: i64) -> (String, String, i64) {
    (json::s(o, "kind"), json::s(o, "id"), n)
}

/// 4.4 acceptance: a one-line trade leaves the same state as today's `buy` or `sell`, with the same gold.
#[test]
fn a_one_line_trade_matches_buy_and_sell() {
    let d = data();
    let mut buys = 0;
    let mut sells = 0;
    for (start, q) in open_shops(&d) {
        let mut gs = start.clone();
        gs.party.gold = 100_000;
        for o in shop::offers(&d, &gs, &q) {
            let n = if json::s(&o, "kind") == shop::SUPPLIES { 5.min(json::i(&o, "left", 0)) } else { 1 };
            if n < 1 || json::i(&o, "left", 0) == 0 {
                continue;
            }
            let (mut a, mut b) = (gs.clone(), gs.clone());
            let old = shop::buy(&d, &mut a, &q, &json::s(&o, "kind"), &json::s(&o, "id"), n);
            let new = shop::trade(&d, &mut b, &q, &[buy_line(&o, n)], &[]);
            assert!(ok(&old) && ok(&new), "{q} {o:?}: {old:?} / {new:?}");
            assert_eq!(a, b, "{q}: buying {} x{n}", json::s(&o, "id"));
            assert_eq!((json::i(&new, "spent", -1), json::i(&new, "gold", 0)), (json::i(&old, "price", 0), -json::i(&old, "price", 0)));
            buys += 1;
        }
        stock_stash(&d, &mut gs, &q, 3);
        for row in shop::sell_list(&d, &gs, &q) {
            let item = json::s(&row, "item");
            let (mut a, mut b) = (gs.clone(), gs.clone());
            let old = shop::sell(&d, &mut a, &q, &item, 2);
            let new = shop::trade(&d, &mut b, &q, &[], &[(item.clone(), 2)]);
            assert!(ok(&old) && ok(&new), "{q} {item}: {old:?} / {new:?}");
            assert_eq!(a, b, "{q}: selling {item} x2");
            assert_eq!((json::i(&new, "earned", -1), json::i(&new, "gold", -1)), (json::i(&old, "price", 0), json::i(&old, "gold", 0)));
            sells += 1;
        }
    }
    assert!(buys > 0 && sells > 0, "an offer and a sale were checked ({buys}, {sells})");
}

/// Q1 (net): sales in the same trade pay for purchases. With gold exactly `price - sales` the trade goes through and
/// leaves 0 gold; one gold less and it fails with nothing changed.
#[test]
fn sales_pay_for_purchases_and_one_gold_short_changes_nothing() {
    let d = data();
    let mut checked = false;
    for (start, q) in open_shops(&d) {
        let mut gs = start.clone();
        stock_stash(&d, &mut gs, &q, 2);
        // the dearest offer with stock, against two of the cheapest item the shop buys
        let Some(offer) = shop::offers(&d, &gs, &q).into_iter().filter(|o| json::i(o, "left", 0) != 0).max_by_key(|o| json::i(o, "price", 0)) else { continue };
        let Some(sale) = shop::sell_list(&d, &gs, &q).into_iter().filter(|r| json::s(r, "item") != json::s(&offer, "item")).min_by_key(|r| json::i(r, "price", 0)) else { continue };
        let (price, earned) = (json::i(&offer, "price", 0), 2 * json::i(&sale, "price", 0));
        if price <= earned {
            continue;
        }
        let buys = [buy_line(&offer, 1)];
        let sells = [(json::s(&sale, "item"), 2)];
        gs.party.gold = price - earned - 1;
        let before = gs.clone();
        let r = shop::trade(&d, &mut gs, &q, &buys, &sells);
        assert!(!ok(&r) && errors(&r) == ["Not enough gold."], "{q}: {r:?}");
        assert_eq!(gs, before, "{q}: a refused trade changes nothing");
        gs.party.gold = price - earned;
        assert!(gs.party.gold < price, "the purchase alone does not fit the gold held");
        let r = shop::trade(&d, &mut gs, &q, &buys, &sells);
        assert!(ok(&r), "{q}: {r:?}");
        assert_eq!((json::i(&r, "spent", 0), json::i(&r, "earned", 0), json::i(&r, "gold", 0)), (price, earned, earned - price));
        assert_eq!(gs.party.gold, 0, "{q}: the gold left is gold + sales - purchases");
        assert_eq!(gs.party.stash.get(&json::s(&sale, "item")).copied().unwrap_or(0), 0, "{q}: both sold");
        checked = true;
    }
    assert!(checked, "no open shop sells something dearer than two of what it buys");
}

/// 4.4: counts are summed per id and side before the stock, supplies and stash checks; stock and stash move by the
/// summed counts; the shop's sold count rises for limited stock.
#[test]
fn summed_counts_meet_stock_supplies_and_the_stash() {
    let d = data();
    let (mut limited, mut supplies, mut loot) = (false, false, false);
    for (start, q) in open_shops(&d) {
        let mut gs = start.clone();
        gs.party.gold = 1_000_000;
        stock_stash(&d, &mut gs, &q, 3);
        for o in shop::offers(&d, &gs, &q) {
            let (kind, id, left) = (json::s(&o, "kind"), json::s(&o, "id"), json::i(&o, "left", 0));
            let before = gs.clone();
            match kind.as_str() {
                "item" if left > 0 => {
                    // one more than is left, in two lines: refused, nothing changes
                    let r = shop::trade(&d, &mut gs, &q, &[buy_line(&o, left), buy_line(&o, 1)], &[]);
                    assert!(!ok(&r) && errors(&r).iter().any(|e| e.starts_with("Sold out")), "{q} {id}: {r:?}");
                    assert_eq!(gs, before);
                    // all that is left, in two lines when it can be
                    let lines = if left > 1 { vec![buy_line(&o, left - 1), buy_line(&o, 1)] } else { vec![buy_line(&o, 1)] };
                    let r = shop::trade(&d, &mut gs, &q, &lines, &[]);
                    assert!(ok(&r), "{q} {id}: {r:?}");
                    let had = before.party.stash.get(&id).copied().unwrap_or(0);
                    assert_eq!(gs.party.stash.get(&id).copied().unwrap_or(0), had + left);
                    let now = shop::offers(&d, &gs, &q).into_iter().find(|x| json::s(x, "id") == id).expect("the offer stays");
                    assert_eq!(json::i(&now, "left", -1), 0, "{q} {id}: sold out");
                    assert_eq!(gs.party.gold, before.party.gold - left * json::i(&o, "price", 0));
                    limited = true;
                }
                "loot" => {
                    let r = shop::trade(&d, &mut gs, &q, &[buy_line(&o, 1), buy_line(&o, 1)], &[]);
                    assert!(!ok(&r) && errors(&r) == ["Loot is sold one at a time."], "{q} {id}: {r:?}");
                    assert_eq!(gs, before);
                    let r = shop::trade(&d, &mut gs, &q, &[buy_line(&o, 1)], &[]);
                    assert!(ok(&r) && gs.loot_claimed.contains(&id), "{q} {id}: {r:?}");
                    assert!(!shop::offers(&d, &gs, &q).iter().any(|x| json::s(x, "id") == id), "claimed loot leaves the offers");
                    loot = true;
                }
                k if k == shop::SUPPLIES => {
                    let room = SUPPLIES_MAX - gs.party.supplies;
                    assert_eq!(left, room.max(0));
                    let r = shop::trade(&d, &mut gs, &q, &[buy_line(&o, room.max(0) + 1)], &[]);
                    assert!(!ok(&r) && errors(&r) == [format!("You can carry at most {SUPPLIES_MAX} supplies.")], "{q}: {r:?}");
                    assert_eq!(gs, before);
                    if room > 0 {
                        let r = shop::trade(&d, &mut gs, &q, &[buy_line(&o, room)], &[]);
                        assert!(ok(&r), "{q}: {r:?}");
                        assert_eq!(gs.party.supplies, SUPPLIES_MAX);
                    }
                    supplies = true;
                }
                _ => {}
            }
        }
        // the stash: two lines summing past what is owned are refused; what is owned sells and leaves the stash
        if let Some(row) = shop::sell_list(&d, &gs, &q).first() {
            let (item, owned) = (json::s(row, "item"), json::i(row, "count", 0));
            let before = gs.clone();
            let r = shop::trade(&d, &mut gs, &q, &[], &[(item.clone(), owned), (item.clone(), 1)]);
            assert!(!ok(&r) && errors(&r) == [format!("Not enough {}.", d.item_name(&item))], "{q} {item}: {r:?}");
            assert_eq!(gs, before);
            let r = shop::trade(&d, &mut gs, &q, &[], &[(item.clone(), owned)]);
            assert!(ok(&r), "{q} {item}: {r:?}");
            assert_eq!(gs.party.stash.get(&item).copied().unwrap_or(0), 0);
            assert_eq!(gs.party.gold, before.party.gold + owned * json::i(row, "price", 0));
        }
    }
    eprintln!("checked: limited stock {limited}, supplies {supplies}, loot {loot}");
    assert!(limited || supplies || loot, "no open shop has limited stock, loot or supplies: nothing to check");
}

/// All-or-nothing: one bad line refuses the whole trade with every reason, and nothing changes.
#[test]
fn one_bad_line_refuses_the_whole_trade() {
    let d = data();
    for (start, q) in open_shops(&d) {
        let mut gs = start.clone();
        gs.party.gold = 1_000_000;
        stock_stash(&d, &mut gs, &q, 1);
        let good: Vec<(String, String, i64)> = shop::offers(&d, &gs, &q).iter().filter(|o| json::i(o, "left", 0) != 0).take(1).map(|o| buy_line(o, 1)).collect();
        let sale: Vec<(String, i64)> = shop::sell_list(&d, &gs, &q).iter().take(1).map(|r| (json::s(r, "item"), 1)).collect();
        let before = gs.clone();
        let mut sells = sale.clone();
        sells.push(("no_such_item".into(), 1));
        let mut buys = good.clone();
        buys.push(("item".into(), "no_such_item".into(), 1));
        let r = shop::trade(&d, &mut gs, &q, &buys, &sells);
        let errs = errors(&r);
        assert!(!ok(&r) && errs.len() == 2, "{q}: {r:?}");
        assert!(errs.contains(&"'no_such_item' is not for sale here.".to_string()) && errs.iter().any(|e| e.starts_with("The shop does not buy")), "{errs:?}");
        assert_eq!(gs, before, "{q}: nothing changes");
        // the good lines alone go through
        if !good.is_empty() || !sale.is_empty() {
            assert!(ok(&shop::trade(&d, &mut gs, &q, &good, &sale)), "{q}: the good lines alone");
        }
    }
}

/// Refusal reasons: an empty trade, a closed shop, counts below one or above BUY_MAX, items the shop does not buy
/// (quest items never sell).
#[test]
fn refusals_name_their_reason() {
    let d = data();
    let (start, q) = open_shops(&d).remove(0);
    let mut gs = start.clone();
    gs.party.gold = 1_000_000;
    let before = gs.clone();
    let reason = |gs: &mut GameState, r: &str, buys: &[(String, String, i64)], sells: &[(String, i64)]| -> Vec<String> {
        let res = shop::trade(&d, gs, r, buys, sells);
        assert!(!ok(&res));
        errors(&res)
    };
    assert_eq!(reason(&mut gs, &q, &[], &[]), ["Nothing to trade."]);
    assert_eq!(reason(&mut gs, "", &[], &[("x".into(), 1)]), ["The shop is closed."]);
    let o = shop::offers(&d, &gs, &q).into_iter().find(|o| json::s(o, "kind") == "item").expect("an item offer");
    assert_eq!(reason(&mut gs, &q, &[buy_line(&o, 0)], &[]), ["Buy at least one."]);
    let loot = ("loot".to_string(), "w:loot".to_string(), 1);
    assert_eq!(reason(&mut gs, &q, &[loot.clone(), loot], &[]), ["Loot is sold one at a time."], "loot lines are summed too");
    if json::i(&o, "left", 0) == shop::UNLIMITED {
        assert_eq!(reason(&mut gs, &q, &[buy_line(&o, shop::BUY_MAX + 1)], &[]), [format!("Buy at most {} at a time.", shop::BUY_MAX)]);
    }
    let quest_item = d.cat.items.keys().find(|id| json::s(d.entry("items", id), "item_type") == "QUEST").cloned();
    if let Some(id) = quest_item {
        gs.party.add_to_stash(&id, 1);
        let snap = gs.clone();
        assert_eq!(reason(&mut gs, &q, &[], &[(id.clone(), 1)]), [format!("The shop does not buy {}.", d.item_name(&id))]);
        assert_eq!(gs, snap);
        gs.party.remove_from_stash(&id, 1);
    }
    assert_eq!(gs.party.gold, before.party.gold);
}

/// A trade that earns more than it spends is capped at GOLD_MAX with `sell`'s note.
#[test]
fn a_gain_past_the_gold_cap_is_noted() {
    let d = data();
    for (start, q) in open_shops(&d) {
        let mut gs = start.clone();
        stock_stash(&d, &mut gs, &q, 1);
        let Some(row) = shop::sell_list(&d, &gs, &q).into_iter().find(|r| json::i(r, "price", 0) >= 2) else { continue };
        gs.party.gold = GOLD_MAX - 1;
        let r = shop::trade(&d, &mut gs, &q, &[], &[(json::s(&row, "item"), 1)]);
        assert!(ok(&r), "{r:?}");
        assert_eq!((gs.party.gold, json::i(&r, "gold", 0)), (GOLD_MAX, 1));
        assert_eq!(json::sget(&r, "notes"), [format!("Gold is capped at {GOLD_MAX} ({} not added).", json::i(&row, "price", 0) - 1)]);
        return;
    }
    panic!("no open shop buys anything worth 2 gold or more");
}
