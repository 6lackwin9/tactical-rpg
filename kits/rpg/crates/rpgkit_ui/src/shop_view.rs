//! UISPLIT1-G (PR7-39; KIT-UI1 KU2 moved it to the kit): the shop's view-model (UISPLIT1 US1, US4, US8). Its view
//! `shop` holds what the shop shows (the shop's offers, the trade, the stash, the totals, the detail line and the
//! refusal line) and its actions are the pane's clicks (`ShopAct`). Stage B: `ShopPane::draw` draws it through the
//! layout `ui/layouts/shop.json` (the kit's file, built in too; a game's own replaces it). A game ships its fixture
//! `ui/views/shop.view.json` and checks its shape.

use crate::shop::{buy_cap, of_text, ShopAct, ShopGame, ShopPane};
use aegis_core::json::{self, s, s_or};
use aegis_ui::screen_layout::{Act, Field, RowButton, RowView, View};
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

/// Row buttons: `+5` and `+10` of the `buy` or `sell` actions, on (`on`) while there is more to add.
fn more(acts: &[(&str, i64); 3], on: bool) -> Vec<RowButton> {
    acts[1..].iter().map(|(a, n)| RowButton { action: a.to_string(), label: format!("+{n}"), enabled: on }).collect()
}

/// The view of shop `sq` with the pane's trade, detail line and refusals. Each row carries its detail line (`note`:
/// the item's name and `ShopGame::item_line`); offers and stash rows in the trade are selected; prices are toned
/// (`foe` above the gold left, `gold` while on, `dim` off), trade lines by sign; `+5` / `+10` on a row with more than
/// one to add (never on loot).
pub fn view<G: ShopGame>(cx: &G, pane: &ShopPane, sq: &str) -> View {
    view_with(cx, pane, sq, &mut |chain| chain.first().cloned())
}

/// `view` with each row's picture chosen from its item's icon files by `pick` (the drawing picks the first file
/// there, `aegis_ui::art::Art::first_rel`; the agent link's view takes the first named).
pub fn view_with<G: ShopGame>(cx: &G, pane: &ShopPane, sq: &str, pick: &mut dyn FnMut(&[String]) -> Option<String>) -> View {
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
    let name = |item: &str| if item.is_empty() { upkeep.clone() } else { d.item_name(item) };
    let note = |item: &str| format!("{}: {}", name(item), cx.item_line(item));
    let mut picture = |item: &str| pick(&cx.item_icons(item));
    let offer_rows: Vec<RowView> = offers
        .iter()
        .map(|o| {
            let (kind, id) = (s(o, "kind"), s(o, "id"));
            let item = if kind == shop::SUPPLIES { String::new() } else { s(o, "item") };
            let mut label = name(&item);
            if kind == "loot" && json::i(o, "count", 1) > 1 {
                label = format!("{label} x{}", json::i(o, "count", 1));
            }
            let (cap, x, price) = (buy_cap(o), trade.bought(&kind, &id), json::i(o, "price", 0));
            let n = (json::i(o, "left", 0) != shop::UNLIMITED).then_some(cap);
            let on = cap - x > 0;
            let tone = if price > after { "foe" } else if on { "gold" } else { "dim" };
            let buttons = if kind != "loot" && cap > 1 { more(&BUY, on) } else { Vec::new() };
            RowView { id, label, detail: format!("{price} {cur}"), detail2: of_text(x, n), picture: picture(&item), enabled: on, selected: x > 0, tone: tone.into(), note: note(&item), buttons, ..Default::default() }
        })
        .collect();
    let mut lines = Vec::new();
    for (kind, id, n) in &trade.buys {
        let Some(o) = offers.iter().find(|o| s(o, "kind") == *kind && s(o, "id") == *id) else { continue };
        let item = if kind == shop::SUPPLIES { String::new() } else { s(o, "item") };
        let gold = -json::i(o, "price", 0) * n;
        lines.push(RowView { id: format!("buy.{id}"), label: format!("{} x{n}", name(&item)), detail: format!("{gold:+}"), picture: picture(&item), enabled: true, tone: "foe".into(), note: note(&item), ..Default::default() });
    }
    for (item, n) in &trade.sells {
        let gold = sale.iter().find(|r| s(r, "item") == *item).map_or(0, |r| json::i(r, "price", 0)) * n;
        lines.push(RowView { id: format!("sell.{item}"), label: format!("{} x{n}", d.item_name(item)), detail: format!("{gold:+}"), picture: picture(item), enabled: true, tone: "good".into(), note: note(item), ..Default::default() });
    }
    let stash_rows: Vec<RowView> = sale
        .iter()
        .map(|r| {
            let item = s(r, "item");
            let (owned, x) = (json::i(r, "count", 0), trade.sold(&item));
            let on = owned - x > 0;
            let buttons = if owned > 1 { more(&SELL, on) } else { Vec::new() };
            RowView { id: item.clone(), label: d.item_name(&item), detail: format!("{} {cur}", json::i(r, "price", 0)), detail2: of_text(x, Some(owned)), picture: picture(&item), enabled: on, selected: x > 0, tone: if on { "gold" } else { "dim" }.into(), note: note(&item), buttons, ..Default::default() }
        })
        .collect();
    let others: Vec<RowView> = gs
        .party
        .stash
        .iter()
        .filter(|(id, n)| **n > 0 && !sale.iter().any(|r| s(r, "item") == **id))
        .map(|(id, n)| RowView { id: id.clone(), label: d.item_name(id), detail: format!("x{n}"), picture: picture(id), enabled: false, note: format!("{} Not bought here.", note(id)), ..Default::default() })
        .collect();
    let net = earned - spent;
    let total = if net == 0 { format!("Total: 0 {cur_word}") } else { format!("Total: {net:+} {cur_word}") };
    let empty = trade.is_empty();
    // the refusal line: the last refusal, else why Confirm is off with lines in the trade
    let errors = if !pane.errors.is_empty() {
        pane.errors.join(" ")
    } else if !empty && after < 0 {
        format!("Not enough {cur_word}: {} more needed.", -after)
    } else {
        String::new()
    };
    let mut v = View::new(SCREEN)
        .field("title", Field::Text(s_or(sh, "name", sq)))
        .field("band", Field::Text(band))
        .field(OFFERS, Field::Rows(offer_rows))
        .field(TRADE, Field::Rows(lines))
        .field("hint", Field::Text(if empty { "Click a row to add it.".into() } else { String::new() }))
        .field("total", Field::Text(total))
        .field("purse", Field::Text(format!("{cur}: {} -> {after}", gs.party.gold)))
        .field(STASH, Field::Rows(stash_rows))
        .field(OTHERS, Field::Rows(others))
        .field("detail", Field::Text(pane.detail.clone().unwrap_or_default()))
        .field("errors", Field::Text(errors));
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
    let kinds: Vec<(String, String)> = shop::offers(cx.data(), cx.gs(), sq).iter().map(|o| (s(o, "id"), s(o, "kind"))).collect();
    let arg = a.arg.as_deref().unwrap_or("");
    let known = |id: &str| kinds.iter().any(|k| k.0 == id);
    let buys = BUY.iter().any(|(x, _)| *x == a.action);
    if (buys && !known(arg)) || (a.action == RETURN && arg.strip_prefix("buy.").is_some_and(|id| !known(id))) {
        return None;
    }
    let x = shop_act_of(&kinds, a);
    (!matches!(&x, ShopAct::Buy(_, id, 0) if id.is_empty())).then_some(x)
}

/// The pane's click for a view action, with the shop's offers as (id, kind): an action it does not know, or an empty
/// sale, is a click that adds nothing (`Buy` of 0 of no offer).
pub fn shop_act_of(kinds: &[(String, String)], a: &Act) -> ShopAct {
    let arg = a.arg.as_deref().unwrap_or("");
    let kind = |id: &str| kinds.iter().find(|k| k.0 == id).map_or_else(String::new, |k| k.1.clone());
    let none = ShopAct::Buy(String::new(), String::new(), 0);
    if let Some((_, n)) = BUY.iter().find(|(id, _)| *id == a.action) {
        return ShopAct::Buy(kind(arg), arg.to_string(), *n);
    }
    if let Some((_, n)) = SELL.iter().find(|(id, _)| *id == a.action) {
        return if arg.is_empty() { none } else { ShopAct::Sell(arg.to_string(), *n) };
    }
    match a.action.as_str() {
        RETURN => match (arg.strip_prefix("buy."), arg.strip_prefix("sell.")) {
            (Some(id), _) => ShopAct::ReturnBuy(kind(id), id.to_string()),
            (_, Some(item)) => ShopAct::ReturnSell(item.to_string()),
            _ => none,
        },
        CLEAR => ShopAct::Clear,
        CONFIRM => ShopAct::Confirm,
        CLOSE => ShopAct::Close,
        _ => none,
    }
}

/// Runs a view action on the pane; true when the shop should close (as `ShopPane::act`).
pub fn act<G: ShopGame>(cx: &mut Cx<G>, pane: &mut ShopPane, sq: &str, a: &Act) -> bool {
    match shop_act(&cx.app, sq, a) {
        Some(x) => pane.act(cx, sq, x),
        None => false,
    }
}
