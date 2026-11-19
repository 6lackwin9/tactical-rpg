//! UISPLIT1-G (PR7-39): the shop's view-model, now the kit's (`rpgkit_ui::shop_view`, KIT-UI1 KU2), on the game's
//! context, drawn through the kit's layout `ui/layouts/shop.json` (Stage B); its fixture is `ui/views/shop.view.json`
//! (`tests::the_fixture_has_the_view_shape`).

use crate::screens::Cx;
use crate::shop::ShopPane;
use aegis_ui::screen_layout::View;

/// The view of shop `sq` with the pane's trade, detail line and refusals (`rpgkit_ui::shop_view::view`).
pub fn view(cx: &Cx, pane: &ShopPane, sq: &str) -> View {
    rpgkit_ui::shop_view::view(&cx.app, pane, sq)
}

/// The view against the shop's own offers and sell list on a debug game of the first playable world, every open shop.
#[cfg(test)]
mod tests {
    use super::*;
    use rpgkit_ui::shop::{buy_cap, of_text};
    use aegis_core::json::{self, s};
    use aegis_ui::screen_layout::{Act, Field, RowView};
    use emberwake_world::shop;
    use rpgkit_ui::shop_view::{act, shop_act, CLEAR, CLOSE, CONFIRM, OFFERS, OTHERS, RETURN, SCREEN, STASH, TRADE};
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
            // ENGINE-20 (FB-361): a buy makes the total negative: toned foe, the purse untoned while it is affordable
            assert_eq!((v.tone_of("totals", 0), v.tone_of("totals", 1), v.tone_of("total", 0)), (Some("foe"), None, Some("foe")), "{q}");
            assert_eq!(view(&cx, &ShopPane::default(), &q).tone_of("totals", 0), None, "an empty trade: no tone");
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
            // a trade the purse cannot pay: the purse line in foe too
            cx.gs_mut().party.gold = 0;
            go(&mut cx, &mut pane, &q, "buy", &id);
            assert_eq!(view(&cx, &pane, &q).tone_of("totals", 1), Some("foe"), "{q}: cannot be afforded");
            go(&mut cx, &mut pane, &q, CLEAR, "");
            cx.gs_mut().party.gold = gold;
            go(&mut cx, &mut pane, &q, "buy", &id);
            assert!(!go(&mut cx, &mut pane, &q, CONFIRM, ""));
            assert!(pane.trade.is_empty() && pane.errors().is_empty(), "{q}: the trade settles");
            assert_eq!(cx.gs().party.gold, gold - json::i(o, "price", 0), "{q}: the purse pays");
            // an unknown row is no click; Close with an empty trade closes
            assert!(shop_act(&cx.app, &q, &Act { action: "buy".into(), arg: Some("no_such_offer".into()) }).is_none());
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
