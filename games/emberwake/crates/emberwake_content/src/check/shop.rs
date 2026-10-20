//! Shop rules of the world checks (ShopRules.shop_errors): keeper, stock, item loot for sale, the buy / sell
//! multipliers, the supply price, the item types bought and the open flags.

use super::quest::condition_errors;
use super::world::ref_errors;
use super::View;
use aegis_core::fmt::fixed;
use aegis_core::json::{self, gstr, Dict};
use serde_json::Value;

const MAX_STOCK: usize = 32;
const MAX_LOOT: usize = 8;
const STOCK_KEYS: [&str; 4] = ["item", "count", "price", "requires_flags"];
const LOOT_KEYS: [&str; 2] = ["loot", "price"];
const COUNT_MAX: i64 = 99;
const PRICE_MAX: i64 = 99999;
const SUPPLY_PRICE_MAX: i64 = 999;
/// Every item type but QUEST (ItemFamilies.ITEM_TYPES order).
const BUY_TYPES: [&str; 6] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY", "CONSUMABLE", "MATERIAL"];

fn int_in(v: &Value, lo: i64, hi: i64) -> bool {
    json::is_int(v) && (lo..=hi).contains(&json::to_int(v))
}

/// ShopRules.shop_errors of a normalized shop.
pub fn shop_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if json::s(e, "keeper").is_empty() {
        errors.push("Choose the shop keeper.".into());
    }
    let empty = Value::Array(Vec::new());
    errors.extend(stock_errors(v, e.get("stock").unwrap_or(&empty)));
    errors.extend(loot_errors(v, w, e.get("loot").unwrap_or(&empty)));
    errors.extend(mult_errors(e.get("buy_mult").unwrap_or(&Value::from(1.0)), "Buy multiplier", 0.5, 3.0));
    errors.extend(mult_errors(e.get("sell_mult").unwrap_or(&Value::from(0.5)), "Sell multiplier", 0.0, 1.0));
    if !int_in(e.get("supply_price").unwrap_or(&json::int(0)), 0, SUPPLY_PRICE_MAX) {
        errors.push(format!("Supply price must be 0 (not sold) or 1..{SUPPLY_PRICE_MAX}."));
    }
    errors.extend(buys_errors(e.get("buys").unwrap_or(&empty)));
    errors.extend(condition_errors(e.get("requires_flags").unwrap_or(&empty), "Open when"));
    errors
}

fn stock_errors(v: &View, raw: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    if !raw.is_array() {
        return vec!["Stock must be a list of {item, count, price, requires_flags}.".into()];
    }
    let list = json::normalize(raw);
    let list = json::arr(&list);
    if list.len() > MAX_STOCK {
        errors.push(format!("Stock holds at most {MAX_STOCK} entries (has {}).", list.len()));
    }
    let mut seen: Vec<String> = Vec::new();
    for (i, raw) in list.iter().enumerate() {
        let Some(s) = raw.as_object() else {
            errors.push(format!("Stock {} must be an object {{item, count, price, requires_flags}}.", i + 1));
            continue;
        };
        let item = json::s(s, "item");
        let label = if item.is_empty() { format!("Stock {}", i + 1) } else { format!("Stock '{item}'") };
        for key in s.keys().filter(|k| !STOCK_KEYS.contains(&k.as_str())) {
            errors.push(format!("{label}: unknown key '{key}'."));
        }
        let base = if s.get("item").is_some_and(Value::is_string) { v.get("items", &item) } else { json::empty() };
        if base.is_empty() {
            errors.push(format!("{label}: unknown base item '{item}'."));
        } else if json::s(base, "item_type") == "QUEST" {
            errors.push(format!("{label}: Quest items can't be stocked."));
        }
        if seen.contains(&item) {
            errors.push(format!("Stock lists '{item}' twice."));
        }
        seen.push(item);
        let count = s.get("count").cloned().unwrap_or(json::int(-1));
        if !json::is_int(&count) || !(json::to_int(&count) == -1 || int_in(&count, 1, COUNT_MAX)) {
            errors.push(format!("{label}: count must be -1 (unlimited) or 1..{COUNT_MAX}."));
        }
        let price = s.get("price").cloned().unwrap_or(json::int(-1));
        if !json::is_int(&price) || !(json::to_int(&price) == -1 || int_in(&price, 1, PRICE_MAX)) {
            errors.push(format!("{label}: price must be -1 (derived) or 1..{PRICE_MAX}."));
        }
        errors.extend(condition_errors(s.get("requires_flags").unwrap_or(&Value::Array(Vec::new())), &format!("{label} requires flags")));
    }
    errors
}

fn loot_errors(v: &View, w: &str, raw: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    if !raw.is_array() {
        return vec!["Loot for sale must be a list of {loot, price}.".into()];
    }
    let list = json::normalize(raw);
    let list = json::arr(&list);
    if list.len() > MAX_LOOT {
        errors.push(format!("Loot for sale holds at most {MAX_LOOT} entries (has {}).", list.len()));
    }
    let mut seen: Vec<String> = Vec::new();
    for (i, raw) in list.iter().enumerate() {
        let Some(l) = raw.as_object() else {
            errors.push(format!("Loot for sale {} must be an object {{loot, price}}.", i + 1));
            continue;
        };
        let r = json::s(l, "loot");
        let label = if r.is_empty() { format!("Loot for sale {}", i + 1) } else { format!("Loot for sale '{r}'") };
        for key in l.keys().filter(|k| !LOOT_KEYS.contains(&k.as_str())) {
            errors.push(format!("{label}: unknown key '{key}'."));
        }
        if !l.get("loot").is_some_and(Value::is_string) || r.is_empty() {
            errors.push(format!("{label}: choose a loot entry."));
        } else {
            let found = ref_errors(v, w, &label, "loot", &r);
            let ok = found.is_empty();
            errors.extend(found);
            let hit = v.worlds.resolve(w, "loot", &r);
            if ok && json::s_or(hit, "reward_kind", "item") != "item" {
                errors.push(format!("{label}: Shops sell item loot only."));
            }
            let key = if hit.is_empty() { r.clone() } else { json::s(hit, "_qid") };
            if seen.contains(&key) {
                errors.push(format!("Loot for sale lists '{r}' twice."));
            }
            seen.push(key);
        }
        if !int_in(json::get(l, "price"), 1, PRICE_MAX) {
            errors.push(format!("{label}: price must be 1..{PRICE_MAX}."));
        }
    }
    errors
}

/// lo..hi with at most two decimals.
fn mult_errors(raw: &Value, label: &str, lo: f64, hi: f64) -> Vec<String> {
    let Some(x) = raw.as_f64() else {
        return vec![format!("{label} must be a number.")];
    };
    let mut errors = Vec::new();
    if x < lo - 0.00001 || x > hi + 0.00001 {
        errors.push(format!("{label} must be {}..{}.", fixed(lo, 1), fixed(hi, 1)));
    }
    if (x * 100.0 - (x * 100.0).round()).abs() > 0.001 {
        errors.push(format!("{label} takes at most two decimals."));
    }
    errors
}

fn buys_errors(raw: &Value) -> Vec<String> {
    let Some(list) = raw.as_array() else {
        return vec!["Buys types must be a list of item types.".into()];
    };
    let mut errors = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for x in list {
        let t = gstr(x);
        if !x.is_string() || !BUY_TYPES.contains(&t.as_str()) {
            errors.push(format!("Buys types: '{t}' is not an item type a shop buys ({}).", BUY_TYPES.join(", ")));
        }
        if seen.contains(&t) {
            errors.push(format!("Buys types lists '{t}' twice."));
        }
        seen.push(t);
    }
    errors
}
