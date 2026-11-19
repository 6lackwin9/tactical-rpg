//! UISPLIT1-G Stage A (PR7-39): the shop's view-model, born beside today's drawing (`docs/specs/UISPLIT1.md` US1, US4,
//! US8). Its view `shop` names what `ShopPane::draw` shows (the shop's offers, the trade, the stash, the totals, the
//! detail line and the refusals) and its actions are the pane's clicks (`ShopAct`). Today's drawing stays until
//! ENGINE-19 (LAYOUT-KIT3: per-row buttons, the selected row, toned values, a dimmed section, a hover detail) lets a
//! layout file match it (Stage B). Its fixture is `ui/views/shop.view.json` (`tests::the_fixture_has_the_view_shape`).

use crate::hud::{term_short, term_word};
use crate::screens::Cx;
use crate::shop::{buy_cap, item_line, of_text, ShopAct, ShopPane};
use aegis_core::json::{self, s, s_or};
use aegis_ui::screen_layout::{Act, Field, RowView, View};
use emberwake_world::shop;
use emberwake_world::terms::{CURRENCY, UPKEEP};

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
fn picture(cx: &Cx, item: &str) -> Option<String> {
    if item.is_empty() {
        return None;
    }
    crate::tokens::item_icons(&cx.data, item).into_iter().next()
}

/// The view of shop `sq` with the pane's trade, detail line and refusals.
pub fn view(cx: &Cx, pane: &ShopPane, sq: &str) -> View {
    let d = &*cx.data;
    let gs = cx.gs();
    let sh = shop::shop_of(d, gs, sq);
    let offers = shop::offers(d, gs, sq);
    let sale = shop::sell_list(d, gs, sq);
    let trade = &pane.trade;
    let (spent, earned) = trade.sums(&offers, &sale);
    let after = gs.party.gold + earned - spent;
    let (cur, cur_word, upkeep) = (term_short(cx, CURRENCY), term_word(cx, CURRENCY), term_short(cx, UPKEEP));
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
        format!("{label}: {}", item_line(cx, item))
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
/// Stage B's layout draw clicks through it; until then only the tests do.
#[cfg_attr(not(test), allow(dead_code))]
pub fn shop_act(cx: &Cx, sq: &str, a: &Act) -> Option<ShopAct> {
    let arg = a.arg.as_deref().unwrap_or("");
    if let Some((_, n)) = BUY.iter().find(|(id, _)| *id == a.action) {
        let o = shop::offers(&cx.data, cx.gs(), sq).into_iter().find(|o| s(o, "id") == arg)?;
        return Some(ShopAct::Buy(s(&o, "kind"), arg.to_string(), *n));
    }
    if let Some((_, n)) = SELL.iter().find(|(id, _)| *id == a.action) {
        return (!arg.is_empty()).then(|| ShopAct::Sell(arg.to_string(), *n));
    }
    match a.action.as_str() {
        RETURN => {
            if let Some(id) = arg.strip_prefix("buy.") {
                let o = shop::offers(&cx.data, cx.gs(), sq).into_iter().find(|o| s(o, "id") == id)?;
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
/// Stage B's layout draw clicks through it; until then only the tests do.
#[cfg_attr(not(test), allow(dead_code))]
pub fn act(cx: &mut Cx, pane: &mut ShopPane, sq: &str, a: &Act) -> bool {
    match shop_act(cx, sq, a) {
        Some(x) => pane.act(cx, sq, x),
        None => false,
    }
}

/// The view against the shop's own offers and sell list on a debug game of the first playable world, every open shop.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::game_cx;

    fn rows<'a>(v: &'a View, name: &str) -> &'a [RowView] {
        match v.get(name) {
            Some(Field::Rows(r)) => r,
            _ => panic!("no rows {name}"),
        }
    }

    fn row<'a>(v: &'a View, name: &str, id: &str) -> &'a RowView {
        rows(v, name).iter().find(|r| r.id == id).unwrap_or_else(|| panic!("{name}: no row {id}"))
    }

    fn go(cx: &mut Cx, pane: &mut ShopPane, sq: &str, action: &str, arg: &str) -> bool {
        act(cx, pane, sq, &Act { action: action.into(), arg: (!arg.is_empty()).then(|| arg.to_string()) })
    }

    /// The open shops' qids, with a rich party: much gold and three of every catalog item.
    fn shops(cx: &mut Cx) -> Vec<String> {
        cx.gs_mut().party.gold = 1_000_000;
        let ids: Vec<String> = cx.data.cat.items.keys().cloned().collect();
        for id in ids {
            cx.gs_mut().party.add_to_stash(&id, 3);
        }
        let d = cx.data.clone();
        let q: Vec<String> = d.merged_entries(&cx.gs().world_id, "shops").into_iter().map(|sh| s(sh, "_qid")).filter(|q| shop::is_open(&d, cx.gs(), q)).collect();
        assert!(!q.is_empty(), "no shop opens in the first playable world: nothing to check");
        q
    }

    /// The rows are the shop's offers and sell list in their order, every stash item the shop does not buy listed apart
    /// and off; an empty trade has nothing to put back, clear or confirm.
    #[test]
    fn the_rows_are_the_shops_offers_and_sell_list() {
        let mut cx = game_cx();
        for q in shops(&mut cx) {
            let pane = ShopPane::default();
            let v = view(&cx, &pane, &q);
            let d = cx.data.clone();
            let ids = |r: &[RowView]| r.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
            assert_eq!(ids(rows(&v, OFFERS)), shop::offers(&d, cx.gs(), &q).iter().map(|o| s(o, "id")).collect::<Vec<_>>(), "{q}");
            let sale: Vec<String> = shop::sell_list(&d, cx.gs(), &q).iter().map(|r| s(r, "item")).collect();
            assert_eq!(ids(rows(&v, STASH)), sale, "{q}");
            assert!(rows(&v, OTHERS).iter().all(|r| !r.enabled && !sale.contains(&r.id)), "{q}");
            assert!(rows(&v, TRADE).is_empty());
            for a in [RETURN, CLEAR, CONFIRM] {
                assert!(!v.act(a).unwrap().enabled, "{q}: {a} off with nothing in the trade");
            }
            assert!(matches!(v.get("title"), Some(Field::Text(t)) if !t.is_empty()));
        }
    }

    /// Buy adds 1, `+5` adds up to what is there, the row reads "X of N" and turns off at the cap; a trade line goes
    /// back whole; Sell takes from the stash; Clear empties the trade; Confirm settles it and the purse moves.
    #[test]
    fn the_actions_are_the_panes_clicks() {
        let mut cx = game_cx();
        for q in shops(&mut cx) {
            let mut pane = ShopPane::default();
            let d = cx.data.clone();
            let offers = shop::offers(&d, cx.gs(), &q);
            let Some(o) = offers.iter().find(|o| buy_cap(o) > 1) else { continue };
            let (kind, id, cap) = (s(o, "kind"), s(o, "id"), buy_cap(o));
            let n = (json::i(o, "left", 0) != shop::UNLIMITED).then_some(cap);
            assert!(!go(&mut cx, &mut pane, &q, "buy", &id));
            assert!(!go(&mut cx, &mut pane, &q, "buy5", &id));
            let x = 6.min(cap);
            assert_eq!(pane.trade.bought(&kind, &id), x, "{q} {id}");
            let v = view(&cx, &pane, &q);
            assert_eq!(row(&v, OFFERS, &id).detail2, of_text(x, n));
            assert_eq!(row(&v, TRADE, &format!("buy.{id}")).detail, format!("{:+}", -json::i(o, "price", 0) * x));
            assert!(v.act(CONFIRM).unwrap().enabled && v.act(CLEAR).unwrap().enabled);
            while pane.trade.bought(&kind, &id) < cap {
                go(&mut cx, &mut pane, &q, "buy10", &id);
            }
            assert!(!row(&view(&cx, &pane, &q), OFFERS, &id).enabled, "{id}: off at the cap");
            go(&mut cx, &mut pane, &q, RETURN, &format!("buy.{id}"));
            assert!(pane.trade.is_empty(), "{q}: the line goes back whole");
            // sell, clear, then buy one and confirm
            let sale = shop::sell_list(&d, cx.gs(), &q);
            if let Some(r) = sale.first() {
                let item = s(r, "item");
                go(&mut cx, &mut pane, &q, "sell10", &item);
                assert_eq!(pane.trade.sold(&item), json::i(r, "count", 0), "{item}: at most what is owned");
                assert!(!row(&view(&cx, &pane, &q), STASH, &item).enabled);
                go(&mut cx, &mut pane, &q, CLEAR, "");
                assert!(pane.trade.is_empty());
            }
            let gold = cx.gs().party.gold;
            go(&mut cx, &mut pane, &q, "buy", &id);
            assert!(!go(&mut cx, &mut pane, &q, CONFIRM, ""));
            assert!(pane.trade.is_empty() && pane.errors.is_empty(), "{q}: the trade settles");
            assert_eq!(cx.gs().party.gold, gold - json::i(o, "price", 0), "{q}: the purse pays");
            // an unknown row is no click; Close with an empty trade closes
            assert!(shop_act(&cx, &q, &Act { action: "buy".into(), arg: Some("no_such_offer".into()) }).is_none());
            assert!(go(&mut cx, &mut pane, &q, CLOSE, ""));
            return;
        }
        panic!("no shop has an offer of more than one");
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids), so a layout drawn against it fits the
    /// running shop.
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let mut cx = game_cx();
        let q = shops(&mut cx).remove(0);
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&cx, &ShopPane::default(), &q)));
    }
}
