//! UISPLIT1-G Stage A (PR7-39; KIT-UI1 KU2 moved it to the kit): the shop's view-model, born beside today's drawing
//! (UISPLIT1 US1, US4, US8). Its view `shop` names what `ShopPane::draw` shows (the shop's offers, the trade, the stash, the totals, the
//! detail line and the refusals) and its actions are the pane's clicks (`ShopAct`). Today's drawing stays until
//! ENGINE-19 (LAYOUT-KIT3: per-row buttons, the selected row, toned values, a dimmed section, a hover detail) lets a
//! layout file match it (Stage B). A game ships its fixture `ui/views/shop.view.json` and checks its shape.

use crate::shop::{buy_cap, of_text, ShopAct, ShopGame, ShopPane};
use aegis_core::json::{self, s, s_or};
use aegis_ui::screen_layout::{Act, Field, RowView, View};
use aegis_ui::Cx;
use rpgkit_world::shop;
use rpgkit_world::terms::{CURRENCY, UPKEEP};

/// The view's screen name (its layout and fixture files).
pub const SCREEN: &str = "shop";
/// Row lists: the shop's offers (row id the offer id), the trade's lines (`buy.<offer id>`, `sell.<item>`), the stash
/// items the shop buys (row id the item) and the ones it does not (shown, never clicked).
pub const OFFERS: &str = "offers";
pub const TRADE: &str = "trade";
pub const STASH: &str = "stash";
pub const OTHERS: &str = "others";
/// Actions (arg the row id): 1 / 5 / 10 more of an offer or a stash item, a trade line back; then the bar's.
pub const BUY: [(&str, i64); 3] = [("buy", 1), ("buy5", 5), ("buy10", 10)];
pub const SELL: [(&str, i64); 3] = [("sell", 1), ("sell5", 5), ("sell10", 10)];
pub const RETURN: &str = "return";
pub const CLEAR: &str = "clear";
pub const CONFIRM: &str = "confirm";
pub const CLOSE: &str = "close";

/// The first icon file of an item's chain (None for supplies, which have none).
fn picture<G: ShopGame>(cx: &G, item: &str) -> Option<String> {
    if item.is_empty() {
        return None;
    }
    cx.item_icons(item).into_iter().next()
}

/// The view of shop `sq` with the pane's trade, detail line and refusals.
pub fn view<G: ShopGame>(cx: &G, pane: &ShopPane, sq: &str) -> View {
    let d = &**cx.data();
    let gs = cx.gs();
    let sh = shop::shop_of(d, gs, sq);
    let offers = shop::offers(d, gs, sq);
    let sale = shop::sell_list(d, gs, sq);
    let trade = &pane.trade;
    let (spent, earned) = trade.sums(&offers, &sale);
    let after = gs.party.gold + earned - spent;
    let (cur, cur_word, upkeep) = (cx.term_short(CURRENCY), cx.term_word(CURRENCY), cx.term_short(UPKEEP));
    let fq = shop::keeper_faction(d, gs, sh);
    let band = if fq.is_empty() { String::new() } else { format!("{}: {}", s(d.resolve(&gs.world_id, "factions", &fq), "name"), shop::keeper_band(d, gs, sh)) };
    let offer_rows: Vec<RowView> = offers
        .iter()
        .map(|o| {
            let (kind, id) = (s(o, "kind"), s(o, "id"));
            let supplies = kind == shop::SUPPLIES;
            let item = if supplies { String::new() } else { s(o, "item") };
            let mut label = if supplies { upkeep.clone() } else { d.item_name(&item) };
            if kind == "loot" && json::i(o, "count", 1) > 1 {
                label = format!("{label} x{}", json::i(o, "count", 1));
            }
            let (cap, x) = (buy_cap(o), trade.bought(&kind, &id));
            let n = (json::i(o, "left", 0) != shop::UNLIMITED).then_some(cap);
            RowView { id, label, detail: format!("{} {cur}", json::i(o, "price", 0)), detail2: of_text(x, n), sub: String::new(), picture: picture(cx, &item), enabled: cap - x > 0, ..Default::default() }
        })
        .collect();
    let mut lines = Vec::new();
    for (kind, id, n) in &trade.buys {
        let Some(o) = offers.iter().find(|o| s(o, "kind") == *kind && s(o, "id") == *id) else { continue };
        let item = if kind == shop::SUPPLIES { String::new() } else { s(o, "item") };
        let label = if item.is_empty() { upkeep.clone() } else { d.item_name(&item) };
        let gold = -json::i(o, "price", 0) * n;
        lines.push(RowView { id: format!("buy.{id}"), label: format!("{label} x{n}"), detail: format!("{gold:+}"), detail2: String::new(), sub: String::new(), picture: picture(cx, &item), enabled: true, ..Default::default() });
    }
    for (item, n) in &trade.sells {
        let gold = sale.iter().find(|r| s(r, "item") == *item).map_or(0, |r| json::i(r, "price", 0)) * n;
        lines.push(RowView { id: format!("sell.{item}"), label: format!("{} x{n}", d.item_name(item)), detail: format!("{gold:+}"), detail2: String::new(), sub: String::new(), picture: picture(cx, item), enabled: true, ..Default::default() });
    }
    let stash_rows: Vec<RowView> = sale
        .iter()
        .map(|r| {
            let item = s(r, "item");
            let (owned, x) = (json::i(r, "count", 0), trade.sold(&item));
            RowView { id: item.clone(), label: d.item_name(&item), detail: format!("{} {cur}", json::i(r, "price", 0)), detail2: of_text(x, Some(owned)), sub: String::new(), picture: picture(cx, &item), enabled: owned - x > 0, ..Default::default() }
        })
        .collect();
    let others: Vec<RowView> = gs
        .party
        .stash
        .iter()
        .filter(|(id, n)| **n > 0 && !sale.iter().any(|r| s(r, "item") == **id))
        .map(|(id, n)| RowView { id: id.clone(), label: d.item_name(id), detail: format!("x{n}"), detail2: String::new(), sub: String::new(), picture: picture(cx, id), enabled: false, ..Default::default() })
        .collect();
    let net = earned - spent;
    let total = if net == 0 { format!("Total: 0 {cur_word}") } else { format!("Total: {net:+} {cur_word}") };
    let detail = pane.detail.as_ref().map_or_else(String::new, |item| {
        let label = if item.is_empty() { upkeep.clone() } else { d.item_name(item) };
        format!("{label}: {}", cx.item_line(item))
    });
    let empty = trade.is_empty();
    let mut v = View::new(SCREEN)
        .field("title", Field::Text(s_or(sh, "name", sq)))
        .field("band", Field::Text(band))
        .field(OFFERS, Field::Rows(offer_rows))
        .field(TRADE, Field::Rows(lines))
        .field("total", Field::Text(total))
        .field("purse", Field::Text(format!("{cur}: {} -> {after}", gs.party.gold)))
        .field(STASH, Field::Rows(stash_rows))
        .field(OTHERS, Field::Rows(others))
        .field("detail", Field::Text(detail))
        .field("errors", Field::Text(pane.errors.join(" ")));
    for (id, n) in BUY {
        v = v.action(id, &if n == 1 { "Buy".to_string() } else { format!("+{n}") }, true);
    }
    for (id, n) in SELL {
        v = v.action(id, &if n == 1 { "Sell".to_string() } else { format!("+{n}") }, true);
    }
    v.action(RETURN, "Put back", !empty).action(CLEAR, "Clear", !empty).action(CONFIRM, "Confirm trade", !empty && after >= 0).action(CLOSE, "Close", true)
}

/// The pane's click for a view action (None: no such action, or a row arg that is not in the shop).
pub fn shop_act<G: ShopGame>(cx: &G, sq: &str, a: &Act) -> Option<ShopAct> {
    let arg = a.arg.as_deref().unwrap_or("");
    if let Some((_, n)) = BUY.iter().find(|(id, _)| *id == a.action) {
        let o = shop::offers(cx.data(), cx.gs(), sq).into_iter().find(|o| s(o, "id") == arg)?;
        return Some(ShopAct::Buy(s(&o, "kind"), arg.to_string(), *n));
    }
    if let Some((_, n)) = SELL.iter().find(|(id, _)| *id == a.action) {
        return (!arg.is_empty()).then(|| ShopAct::Sell(arg.to_string(), *n));
    }
    match a.action.as_str() {
        RETURN => {
            if let Some(id) = arg.strip_prefix("buy.") {
                let o = shop::offers(cx.data(), cx.gs(), sq).into_iter().find(|o| s(o, "id") == id)?;
                Some(ShopAct::ReturnBuy(s(&o, "kind"), id.to_string()))
            } else {
                arg.strip_prefix("sell.").map(|item| ShopAct::ReturnSell(item.to_string()))
            }
        }
        CLEAR => Some(ShopAct::Clear),
        CONFIRM => Some(ShopAct::Confirm),
        CLOSE => Some(ShopAct::Close),
        _ => None,
    }
}

/// Runs a view action on the pane; true when the shop should close (as `ShopPane::act`).
pub fn act<G: ShopGame>(cx: &mut Cx<G>, pane: &mut ShopPane, sq: &str, a: &Act) -> bool {
    match shop_act(&cx.app, sq, a) {
        Some(x) => pane.act(cx, sq, x),
        None => false,
    }
}
