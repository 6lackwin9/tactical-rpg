//! ShopRules runtime (scripts/dialogue/shop_rules.gd): open / closed, integer prices with reputation bands,
//! offers (stock, loot, travel supplies), buy and sell over GameState. Validation of shop entries is not ported.

use crate::data::Data;
use crate::dialogue::{self, rep};
use crate::quest;
use crate::state::{GameState, GOLD_MAX, SUPPLIES_MAX};
use aegis_core::json::{self, int, Dict};
use serde_json::Value;

pub const UNLIMITED: i64 = -1;
pub const DERIVED: i64 = -1;
pub const BUY_MAX: i64 = 99;
pub const SUPPLIES: &str = "supplies";
pub const REP_NEUTRAL: i64 = 100;

pub fn shop_of<'a>(d: &'a Data, gs: &GameState, r: &str) -> &'a Dict {
    if r.is_empty() {
        json::empty()
    } else {
        d.resolve(&gs.world_id, "shops", r)
    }
}

pub fn is_open(d: &Data, gs: &GameState, r: &str) -> bool {
    let s = shop_of(d, gs, r);
    !s.is_empty()
        && quest::all_hold(gs, json::get(s, "requires_flags"))
        && !dialogue::is_gone(d, gs, &json::s(s, "keeper"))
        && keeper_band(d, gs, s) != rep::HOSTILE
}

pub fn keeper_faction(d: &Data, gs: &GameState, s: &Dict) -> String {
    dialogue::faction_qid(d, gs, &json::s(s, "keeper"))
}

pub fn keeper_band(d: &Data, gs: &GameState, s: &Dict) -> &'static str {
    let fq = keeper_faction(d, gs, s);
    if fq.is_empty() {
        rep::NEUTRAL
    } else {
        rep::band(rep::standing(d, gs, &fq))
    }
}

fn mult(s: &Dict, key: &str, fallback: f64) -> i64 {
    json::roundi(json::f(s, key, fallback) * 100.0)
}

pub fn buy_price(s: &Dict, value: i64, fixed: i64, rb: i64) -> i64 {
    if fixed > 0 {
        return fixed;
    }
    let num = value.max(0) * mult(s, "buy_mult", 1.0) * rb;
    ((num + 9999) / 10000).max(1)
}

pub fn sell_price(s: &Dict, value: i64, rs: i64) -> i64 {
    value.max(0) * mult(s, "sell_mult", 0.5) * rs / 10000
}

fn value_of(d: &Data, item: &str) -> i64 {
    json::i(d.entry("items", item), "value", 0)
}

fn sold(gs: &GameState, sqid: &str, item: &str) -> i64 {
    gs.shops.get(sqid).map(json::dict).and_then(|m| m.get(item)).map(json::to_int).unwrap_or(0)
}

/// [{kind: item | loot | supplies, id, item, count, price, left}]; [] when closed.
pub fn offers(d: &Data, gs: &GameState, r: &str) -> Vec<Dict> {
    let mut out = Vec::new();
    if !is_open(d, gs, r) {
        return out;
    }
    let s = shop_of(d, gs, r);
    let sqid = json::s(s, "_qid");
    let rb = rep::buy_pct(keeper_band(d, gs, s));
    for raw in json::arr(&json::normalize(json::get(s, "stock"))) {
        let st = json::dict(raw);
        if !quest::all_hold(gs, json::get(st, "requires_flags")) {
            continue;
        }
        let item = json::s(st, "item");
        let count = json::i(st, "count", UNLIMITED);
        let left = if count == UNLIMITED { UNLIMITED } else { (count - sold(gs, &sqid, &item)).max(0) };
        let price = buy_price(s, value_of(d, &item), json::i(st, "price", DERIVED), rb);
        out.push(json::obj([("kind", "item".into()), ("id", item.clone().into()), ("item", item.into()), ("count", int(count)), ("price", int(price)), ("left", int(left))]));
    }
    for raw in json::arr(&json::normalize(json::get(s, "loot"))) {
        let l = json::dict(raw);
        let loot = d.resolve(&gs.world_id, "loot", &json::s(l, "loot"));
        let lqid = json::s(loot, "_qid");
        if lqid.is_empty() || gs.loot_claimed.contains(&lqid) || !quest::all_hold(gs, json::get(loot, "requires_flags")) {
            continue;
        }
        out.push(json::obj([
            ("kind", "loot".into()),
            ("id", lqid.into()),
            ("item", json::s(loot, "ref").into()),
            ("count", int(json::i(loot, "count", 1))),
            ("price", int(json::i(l, "price", 1))),
            ("left", int(1)),
        ]));
    }
    let supply_price = json::i(s, "supply_price", 0);
    if supply_price > 0 {
        out.push(json::obj([
            ("kind", SUPPLIES.into()),
            ("id", SUPPLIES.into()),
            ("item", "".into()),
            ("count", int(1)),
            ("price", int(supply_price)),
            ("left", int((SUPPLIES_MAX - gs.party.supplies).max(0))),
        ]));
    }
    out
}

/// {ok, errors, price, items | supplies}; all-or-nothing.
pub fn buy(d: &Data, gs: &mut GameState, r: &str, kind: &str, id: &str, count: i64) -> Dict {
    if !is_open(d, gs, r) {
        return json::fail("The shop is closed.");
    }
    if count < 1 || (kind == "loot" && count != 1) {
        return json::fail(if kind == "loot" { "Loot is sold one at a time." } else { "Buy at least one." });
    }
    if count > BUY_MAX {
        return json::fail(&format!("Buy at most {BUY_MAX} at a time."));
    }
    let Some(offer) = offers(d, gs, r).into_iter().find(|o| json::s(o, "kind") == kind && json::s(o, "id") == id) else {
        return json::fail(&format!("'{id}' is not for sale here."));
    };
    let left = json::i(&offer, "left", 0);
    let price = json::i(&offer, "price", 0);
    if kind == SUPPLIES {
        if count > left {
            return json::fail(&format!("You can carry at most {SUPPLIES_MAX} supplies."));
        }
        let total = price * count;
        if gs.party.gold < total {
            return json::fail("Not enough gold.");
        }
        gs.party.gold -= total;
        let added = gs.party.add_supplies(count);
        let mut out = json::ok_result(true, &[]);
        out.insert("price".into(), int(total));
        out.insert("supplies".into(), int(added));
        return out;
    }
    if left != UNLIMITED && left < count {
        return json::fail("Sold out.");
    }
    let total = price * count;
    if gs.party.gold < total {
        return json::fail("Not enough gold.");
    }
    let item = json::s(&offer, "item");
    let amount = if kind == "loot" { json::i(&offer, "count", 1) } else { count };
    if kind == "loot" {
        let why = quest::claim_loot(gs, id);
        if !why.is_empty() {
            return json::fail(&why);
        }
    }
    gs.party.gold -= total;
    gs.party.add_to_stash(&item, amount);
    if kind == "item" && json::i(&offer, "count", 0) != UNLIMITED {
        let sqid = json::s(shop_of(d, gs, r), "_qid");
        let mut sold_map = gs.shops.get(&sqid).map(json::dict).cloned().unwrap_or_default();
        let n = sold_map.get(&item).map(json::to_int).unwrap_or(0) + count;
        sold_map.insert(item.clone(), int(n));
        gs.shops.insert(sqid, Value::Object(sold_map));
    }
    let mut out = json::ok_result(true, &[]);
    out.insert("price".into(), int(total));
    out.insert("items".into(), Value::Object(json::obj([(item.as_str(), int(amount))])));
    out
}

/// Stash items the shop buys: [{item, count, price}], stash order.
pub fn sell_list(d: &Data, gs: &GameState, r: &str) -> Vec<Dict> {
    let mut out = Vec::new();
    if !is_open(d, gs, r) {
        return out;
    }
    let s = shop_of(d, gs, r);
    let buys = json::sget(s, "buys");
    let rs = rep::sell_pct(keeper_band(d, gs, s));
    for (id, n) in &gs.party.stash {
        let e = d.entry("items", id);
        let t = json::s(e, "item_type");
        if e.is_empty() || t == "QUEST" || !buys.contains(&t) {
            continue;
        }
        let price = sell_price(s, json::i(e, "value", 0), rs);
        if price >= 1 {
            out.push(json::obj([("item", id.clone().into()), ("count", int(*n)), ("price", int(price))]));
        }
    }
    out
}

/// {ok, errors, price, gold, notes}.
pub fn sell(d: &Data, gs: &mut GameState, r: &str, item: &str, count: i64) -> Dict {
    if !is_open(d, gs, r) {
        return json::fail("The shop is closed.");
    }
    let Some(row) = sell_list(d, gs, r).into_iter().filter(|x| json::s(x, "item") == item).last() else {
        return json::fail(&format!("The shop does not buy '{item}'."));
    };
    if count < 1 || json::i(&row, "count", 0) < count {
        return json::fail(&format!("Not enough {item}."));
    }
    let total = json::i(&row, "price", 0) * count;
    gs.party.remove_from_stash(item, count);
    let added = gs.party.add_gold(total);
    let mut notes = Vec::new();
    if added < total {
        notes.push(format!("Gold is capped at {GOLD_MAX} ({} not added).", total - added));
    }
    let mut out = json::ok_result(true, &[]);
    out.insert("price".into(), int(total));
    out.insert("gold".into(), int(added));
    out.insert("notes".into(), json::strs(&notes));
    out
}
