//! The shop (PLAYUI1-P, games/emberwake/docs/specs/PLAYUI1.md 4): the kit's trade screen (`rpgkit_ui::shop`, moved
//! there by KIT-UI1 KU2 with no change in behaviour) hosted by the game. `talk::Services` draws it while its
//! conversation has a shop open (over the region map, or the plain backdrop of `talk::ServiceScreen`). The game gives
//! it its data and game state, the purse words of the world being played, an item's detail line (`item_line`), its
//! icon files, the event log and the HUD height (`ShopGame for GameCx`).

use crate::hud::HUD_H;
use crate::screens::GameCx;
use aegis_core::json::{self, s, s_or};
use aegis_ui::canvas::Rect;
use aegis_ui::painter::Painter;
use emberwake_world::terms::UPKEEP;
use emberwake_world::{Data, GameState};
pub use rpgkit_ui::shop::{ShopAct, ShopPane, DISCARD};
use std::sync::Arc;

/// Term `id` of the world being played: its short form, or (`word`) its word inside a sentence (TERMS1 TU3, TU4).
fn world_term(cx: &GameCx, id: &str, short: bool) -> String {
    let w = cx.game.as_ref().map(|g| g.gs.world_id.clone()).unwrap_or_default();
    if short {
        emberwake_world::terms::short(&cx.data, &w, id)
    } else {
        emberwake_world::terms::word(&cx.data, &w, id)
    }
}

impl rpgkit_ui::shop::ShopGame for GameCx {
    fn data(&self) -> &Arc<Data> {
        &self.data
    }
    fn gs(&self) -> &GameState {
        GameCx::gs(self)
    }
    fn gs_mut(&mut self) -> &mut GameState {
        GameCx::gs_mut(self)
    }
    fn term_short(&self, id: &str) -> String {
        world_term(self, id, true)
    }
    fn term_word(&self, id: &str) -> String {
        world_term(self, id, false)
    }
    fn item_line(&self, item: &str) -> String {
        item_line(self, item)
    }
    fn item_icons(&self, id: &str) -> Vec<String> {
        crate::tokens::item_icons(&self.data, id)
    }
    fn log(&mut self, line: String) {
        GameCx::log(self, line);
    }
    fn hud_h(&self) -> i32 {
        HUD_H
    }
}

/// The shop's frame at the current text size, centred under the HUD (`rpgkit_ui::shop::frame_rect`); the board, the
/// recipe and facility panes share it.
pub fn frame_rect(p: &Painter) -> Rect {
    rpgkit_ui::shop::frame_rect(p, HUD_H)
}

/// "+90 HP · Owned 2 · Anyone can use it" style detail of an item ("" = travel supplies).
pub fn item_line(cx: &GameCx, item: &str) -> String {
    if item.is_empty() {
        return format!("Travel {}: one per day on the road.", world_term(cx, UPKEEP, false));
    }
    let e = cx.data.entry("items", item);
    let owned = cx.game.as_ref().map(|g| g.gs.party.stash.get(item).copied().unwrap_or(0)).unwrap_or(0);
    let mut parts = Vec::new();
    let desc = s(e, "description");
    if !desc.is_empty() {
        parts.push(desc);
    }
    // PR7-41 (PT-30): Owned counts the stash; who already wears it (bought copy or standard kit) is named apart
    let wearers = cx.game.as_ref().map(|g| worn_by(&cx.data, &g.gs, item)).unwrap_or_default();
    let worn = if wearers.is_empty() { String::new() } else { format!(" · Worn by {}", wearers.join(", ")) };
    parts.push(format!("{} · Owned {owned}{worn}", s_or(e, "item_type", "ITEM").to_lowercase()));
    if let Some(g) = cx.game.as_ref() {
        let note = item_use_note(&cx.data, &g.gs, item);
        if !note.is_empty() {
            parts.push(note);
        }
    }
    parts.join(" · ")
}

/// PR7-41 (PT-30): the party members wearing `item` now: in a slot, or as the slot's class default (standard kit).
pub fn worn_by(d: &Data, gs: &GameState, item: &str) -> Vec<String> {
    gs.party
        .members
        .iter()
        .filter(|m| {
            emberwake_world::state::LOADOUT_SLOTS.iter().any(|slot| match m.equipment.get(*slot) {
                Some(i) => i == item,
                None => m.default_item(d, &gs.world_id, slot).as_deref() == Some(item),
            })
        })
        .map(|m| emberwake_world::progression::display_name(d, m))
        .collect()
}

/// PR7-41 (PT-30): what a kept item is for, for its hover cards (stash, shop): the recipes whose cost takes it
/// (`recipe::uses_of`, at most 3 named), a story item kept for the story, an item no shop buys; "" for gear and
/// consumables with nothing to add.
pub fn item_use_note(d: &Data, gs: &GameState, item: &str) -> String {
    let e = d.entry("items", item);
    let uses = emberwake_world::recipe::uses_of(d, gs, item);
    let mut out = Vec::new();
    if !uses.is_empty() {
        let more = uses.len().saturating_sub(3);
        let named = uses.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        out.push(format!("Used for: {named}{}.", if more > 0 { format!(" and {more} more") } else { String::new() }));
    }
    if s(e, "item_type") == "QUEST" {
        out.push("Key item: kept for the story; no shop buys it.".to_string());
    } else if json::i(e, "value", 0) <= 0 && !e.is_empty() {
        out.push("No shop buys it.".to_string());
    }
    out.join(" ")
}

/// The frame at the current text size: 600 x 300 at Normal, grown with the text (never shrunk), centred under the HUD.

/// PLAYUI1-P (spec PLAYUI1 4.2-4.3): the trade screen on a debug game of the first playable world, its shops opened
/// the way a player opens them (the keeper's shop reply). Gold and the stash are set in memory; expectations come
/// from the shop's own offers and sell list. SERVICES1: the trade opens on the services' plain backdrop
/// (`talk::ServiceScreen`), no region map (the region map's own shop checks are in `region.rs`).
#[cfg(test)]
mod tests {
    use super::*;
    use aegis_ui::ui::{H, W};
    use emberwake_world::shop;
    use crate::screens::Cx;
    use rpgkit_ui::shop::{buy_cap, of_text, EDGE, SHOP_H, SHOP_W};
    use serde_json::{json, Value};
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use crate::talk::ServiceScreen;
    use crate::screens::{GameCx, Nav};
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::{Key, Screen, TEXT_SIZES};
    use aegis_ui::ui::Gfx;

    fn widget(map: &ServiceScreen, id: &str) -> Option<Value> {
        map.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn on(map: &ServiceScreen, id: &str) -> bool {
        widget(map, id).is_some_and(|w| w["enabled"] == json!(true))
    }

    fn frame_at(map: &mut ServiceScreen, cx: &mut Cx, g: &mut Gfx, at: (i32, i32), scale: f32) -> Vec<TextRec> {
        g.fonts.scale = scale;
        g.fonts.audit = Some(Vec::new());
        map.draw(cx, g, at, true);
        g.fonts.audit.take().unwrap_or_default()
    }

    /// Every catalog item put in the stash three times (what the shop does not buy too).
    fn fill_stash(cx: &mut Cx) {
        let ids: Vec<String> = cx.data.cat.items.keys().cloned().collect();
        for id in ids {
            cx.gs_mut().party.add_to_stash(&id, 3);
        }
    }

    /// Each shop open in the game, opened through its keeper's shop reply: `each` gets the map (shop open) and the
    /// shop's qid and says whether to go on; the game state is put back after each. The number of shops opened.
    fn each_shop(cx: &mut Cx, g: &mut Gfx, mut each: impl FnMut(&mut ServiceScreen, &mut Cx, &mut Gfx, &str) -> bool) -> usize {
        let d = cx.data.clone();
        let start = cx.gs().clone();
        let mut opened = 0;
        for sh in d.merged_entries(&start.world_id, "shops") {
            let q = s(sh, "_qid");
            if !shop::is_open(&d, &start, &q) {
                continue;
            }
            *cx.gs_mut() = start.clone();
            let mut map = ServiceScreen::default();
            map.talk(cx, &s(sh, "keeper"));
            frame(&mut map, cx, g, 1.0);
            let choices = json::aget(json::dict(&map.state(cx)["dialogue"]), "choices").to_vec();
            let Some(k) = choices.iter().position(|c| s(json::dict(c), "service") == "shop") else { continue };
            map.click_id(cx, &format!("choice.{k}")).expect("the shop reply");
            frame(&mut map, cx, g, 1.0);
            assert!(widget(&map, "shop").is_some() && map.state(cx)["shop"].is_object(), "{q}: the shop opens");
            opened += 1;
            if !each(&mut map, cx, g, &q) {
                break;
            }
        }
        *cx.gs_mut() = start;
        assert!(opened > 0, "no shop opens in the first playable world: nothing to check");
        opened
    }

    /// Row `id` of the shop view's rows field `list` (the agent link's `shop.view`).
    fn view_row(map: &ServiceScreen, cx: &Cx, list: &str, id: &str) -> Value {
        let v = map.state(cx)["shop"]["view"]["fields"][list]["rows"].clone();
        v.as_array().into_iter().flatten().find(|r| r["id"] == id).cloned().unwrap_or(Value::Null)
    }

    /// Text field `name` of the shop view.
    fn view_text(map: &ServiceScreen, cx: &Cx, name: &str) -> String {
        map.state(cx)["shop"]["view"]["fields"][name]["text"].as_str().unwrap_or("").to_string()
    }

    fn buys(map: &ServiceScreen, cx: &Cx) -> Vec<(String, String, i64)> {
        let b = map.state(cx)["shop"]["buys"].clone();
        b.as_array().into_iter().flatten().map(|b| (b[0].as_str().unwrap_or("").to_string(), b[1].as_str().unwrap_or("").to_string(), b[2].as_i64().unwrap_or(0))).collect()
    }

    fn sells(map: &ServiceScreen, cx: &Cx) -> Vec<(String, i64)> {
        let x = map.state(cx)["shop"]["sells"].clone();
        x.as_array().into_iter().flatten().map(|x| (x[0].as_str().unwrap_or("").to_string(), x[1].as_i64().unwrap_or(0))).collect()
    }

    /// 4.3 rule 1 and the 4.4 acceptance: a row click adds 1, `+5` / `+10` add at most what is left, the row reads
    /// "X of N"; at the cap the row and its buttons turn off. The stash side the same against what is owned.
    #[test]
    fn clicks_add_one_and_the_buttons_add_up_to_what_is_there() {
        let (mut cx, mut g) = (game_cx(), gfx());
        each_shop(&mut cx, &mut g, |map, cx, g, q| {
            cx.gs_mut().party.gold = 1_000_000;
            fill_stash(cx);
            frame(map, cx, g, 1.0);
            let d = cx.data.clone();
            for o in shop::offers(&d, cx.gs(), q).into_iter().filter(|o| buy_cap(o) > 1) {
                let (kind, id, cap) = (s(&o, "kind"), s(&o, "id"), buy_cap(&o));
                let n = (json::i(&o, "left", 0) != shop::UNLIMITED).then_some(cap);
                map.click_id(cx, &format!("shop.row.{id}")).expect("the row");
                map.click_id(cx, &format!("shop.row.{id}")).expect("the row again");
                map.click_id(cx, &format!("shop.row.{id}.buy5")).expect("+5");
                frame(map, cx, g, 1.0);
                let x = 7.min(cap);
                assert_eq!(buys(map, cx).last().cloned(), Some((kind.clone(), id.clone(), x)), "{id}");
                let count = view_row(map, cx, "offers", &id)["detail2"].as_str().unwrap_or("").to_string();
                assert_eq!(count, of_text(x, n), "{id}: the row reads {}", of_text(x, n));
                while on(map, &format!("shop.row.{id}.buy10")) {
                    map.click_id(cx, &format!("shop.row.{id}.buy10")).expect("+10");
                    frame(map, cx, g, 1.0);
                }
                assert_eq!(buys(map, cx).last().map(|b| b.2), Some(cap), "{id}: +10 stops at the cap");
                assert!(!on(map, &format!("shop.row.{id}")) && !on(map, &format!("shop.row.{id}.buy5")), "{id}: off at the cap");
                assert!(map.click_id(cx, &format!("shop.row.{id}")).is_err());
            }
            for r in shop::sell_list(&d, cx.gs(), q).into_iter().take(3) {
                let (item, owned) = (s(&r, "item"), json::i(&r, "count", 0));
                map.click_id(cx, &format!("stash.row.{item}")).expect("the stash row");
                if owned > 1 {
                    map.click_id(cx, &format!("stash.row.{item}.sell10")).expect("+10");
                }
                frame(map, cx, g, 1.0);
                assert_eq!(sells(map, cx).last().cloned(), Some((item.clone(), owned)), "{item}: at most what is owned");
                assert!(!on(map, &format!("stash.row.{item}")), "{item}: off when all of it is in the trade");
            }
            // loot is one at a time, without `+5` / `+10`
            for o in shop::offers(&d, cx.gs(), q).into_iter().filter(|o| s(o, "kind") == "loot") {
                let id = s(&o, "id");
                assert!(widget(map, &format!("shop.row.{id}.buy5")).is_none(), "{id}: no +5 on loot");
                map.click_id(cx, &format!("shop.row.{id}")).expect("the loot row");
                frame(map, cx, g, 1.0);
                assert!(!on(map, &format!("shop.row.{id}")), "{id}: one at most");
            }
            true
        });
    }

    /// 4.2: a click on a trade line puts the whole line back.
    #[test]
    fn a_trade_line_goes_back_whole() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut checked = false;
        each_shop(&mut cx, &mut g, |map, cx, g, q| {
            fill_stash(cx);
            frame(map, cx, g, 1.0);
            let d = cx.data.clone();
            let Some(o) = shop::offers(&d, cx.gs(), q).into_iter().find(|o| buy_cap(o) > 1) else { return true };
            let id = s(&o, "id");
            let item = shop::sell_list(&d, cx.gs(), q).first().map(|r| s(r, "item")).expect("something the shop buys");
            map.click_id(cx, &format!("shop.row.{id}")).expect("row");
            map.click_id(cx, &format!("shop.row.{id}")).expect("row");
            map.click_id(cx, &format!("stash.row.{item}")).expect("stash row");
            frame(map, cx, g, 1.0);
            map.click_id(cx, &format!("trade.row.buy.{id}")).expect("the buy line");
            frame(map, cx, g, 1.0);
            assert!(buys(map, cx).is_empty() && sells(map, cx).len() == 1, "the whole buy line went back");
            assert!(widget(map, &format!("trade.row.buy.{id}")).is_none());
            map.click_id(cx, &format!("trade.row.sell.{item}")).expect("the sale line");
            frame(map, cx, g, 1.0);
            assert!(sells(map, cx).is_empty());
            assert!(!on(map, "shop.confirm") && !on(map, "shop.clear"), "an empty trade: Confirm and Clear off");
            checked = true;
            false
        });
        assert!(checked, "a shop with an offer of more than one");
    }

    /// 4.2 / 4.3 rules 3-4: buy lines read `-price x n`, sale lines `+worth x n`; the total and the gold after; Confirm
    /// settles exactly what `shop::trade` does, clears the trade and logs it.
    #[test]
    fn the_total_shows_and_confirm_settles_the_trade() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut checked = false;
        each_shop(&mut cx, &mut g, |map, cx, g, q| {
            fill_stash(cx);
            cx.gs_mut().party.gold = 1_000;
            frame(map, cx, g, 1.0);
            let d = cx.data.clone();
            let Some(o) = shop::offers(&d, cx.gs(), q).into_iter().find(|o| buy_cap(o) > 1 && json::i(o, "price", 0) * 2 <= 1_000) else { return true };
            let r = shop::sell_list(&d, cx.gs(), q).into_iter().find(|r| json::i(r, "count", 0) >= 3 && s(r, "item") != s(&o, "item")).expect("three of something the shop buys");
            let (id, item, price, worth) = (s(&o, "id"), s(&r, "item"), json::i(&o, "price", 0), json::i(&r, "price", 0));
            map.click_id(cx, &format!("shop.row.{id}")).expect("row");
            map.click_id(cx, &format!("shop.row.{id}")).expect("row");
            for _ in 0..3 {
                map.click_id(cx, &format!("stash.row.{item}")).expect("stash row");
            }
            let recs = frame(map, cx, g, 1.0);
            let has = |t: &str| recs.iter().any(|r| r.text == t);
            let net = 3 * worth - 2 * price;
            assert!(has(&format!("{:+}", -2 * price)) && has(&format!("{:+}", 3 * worth)), "the lines' gold");
            let total = if net == 0 { "Total: 0 gold".to_string() } else { format!("Total: {net:+} gold") };
            assert!(has(&total) && has(&format!("Gold: 1000 -> {}", 1_000 + net)), "{total}: {:?}", recs.iter().map(|r| r.text.clone()).collect::<Vec<_>>());
            let mut want = cx.gs().clone();
            assert!(json::b(&shop::trade(&d, &mut want, q, &buys(map, cx), &sells(map, cx)), "ok", false));
            map.click_id(cx, "shop.confirm").expect("confirm");
            frame(map, cx, g, 1.0);
            assert!(*cx.gs() == want, "Confirm is shop::trade");
            assert!(buys(map, cx).is_empty() && sells(map, cx).is_empty(), "the trade clears");
            assert_eq!(cx.log.last().cloned(), Some(format!("Trade: bought 2 items for {} gold, sold 3 items for {} gold.", 2 * price, 3 * worth)));
            checked = true;
            false
        });
        assert!(checked, "a shop with an offer of more than one at 500 gold or less");
    }

    /// 4.3 rule 3 (Q1 net): Confirm is off while the gold after the trade is below 0 and its card says how much more is
    /// needed; a sale that covers the purchase turns it on. A row above the gold left still adds.
    #[test]
    fn confirm_is_off_below_zero_gold_after_and_sales_pay() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut checked = false;
        each_shop(&mut cx, &mut g, |map, cx, g, q| {
            fill_stash(cx);
            cx.gs_mut().party.gold = 0;
            frame(map, cx, g, 1.0);
            let d = cx.data.clone();
            let Some(o) = shop::offers(&d, cx.gs(), q).into_iter().filter(|o| buy_cap(o) > 0 && json::i(o, "price", 0) > 0).min_by_key(|o| json::i(o, "price", 0)) else { return true };
            let (id, price) = (s(&o, "id"), json::i(&o, "price", 0));
            map.click_id(cx, &format!("shop.row.{id}")).expect("a row above the gold left still adds");
            frame(map, cx, g, 1.0);
            assert!(!on(map, "shop.confirm"), "0 gold, a purchase: Confirm off");
            let why = view_text(map, cx, "errors");
            assert!(why.contains(&format!("Not enough gold: {price} more needed.")), "{why}");
            // sell enough of the dearest item the shop buys to cover it
            let r = shop::sell_list(&d, cx.gs(), q).into_iter().max_by_key(|r| json::i(r, "price", 0)).expect("something the shop buys");
            let worth = json::i(&r, "price", 1).max(1);
            let need = (price + worth - 1) / worth;
            if need > json::i(&r, "count", 0) {
                return true;
            }
            for _ in 0..need {
                map.click_id(cx, &format!("stash.row.{}", s(&r, "item"))).expect("stash row");
            }
            frame(map, cx, g, 1.0);
            assert!(on(map, "shop.confirm"), "the sales pay for the purchase");
            map.click_id(cx, "shop.confirm").expect("confirm");
            assert!(cx.gs().party.gold == need * worth - price && buys(map, cx).is_empty());
            checked = true;
            false
        });
        assert!(checked, "a shop where a sale covers the cheapest purchase");
    }

    /// 4.3 rule 5: Close, Esc or right click with lines asks "Discard this trade?"; Yes leaves the shop with nothing
    /// changed and the conversation back; an empty trade closes at once. Clear empties the trade.
    #[test]
    fn closing_with_lines_asks_and_discards() {
        let (mut cx, mut g) = (game_cx(), gfx());
        each_shop(&mut cx, &mut g, |map, cx, g, q| {
            fill_stash(cx);
            cx.gs_mut().party.gold = 1_000;
            frame(map, cx, g, 1.0);
            let d = cx.data.clone();
            let item = shop::sell_list(&d, cx.gs(), q).first().map(|r| s(r, "item")).expect("something the shop buys");
            map.click_id(cx, &format!("stash.row.{item}")).expect("stash row");
            frame(map, cx, g, 1.0);
            map.click_id(cx, "shop.clear").expect("clear");
            frame(map, cx, g, 1.0);
            assert!(sells(map, cx).is_empty(), "Clear empties the trade");
            map.click_id(cx, &format!("stash.row.{item}")).expect("stash row");
            frame(map, cx, g, 1.0);
            let before = cx.gs().clone();
            map.back(cx);
            let asked = matches!(cx.nav.last(), Some(Nav::Push(s)) if s.id() == "ask" && s.state(cx)["question"] == "Discard this trade?");
            assert!(asked, "Esc with a line asks first");
            cx.nav.clear();
            frame(map, cx, g, 1.0);
            assert!(widget(map, "shop").is_some(), "No: the shop stays");
            map.click_id(cx, "shop.close").expect("close");
            assert!(matches!(cx.nav.last(), Some(Nav::Push(s)) if s.id() == "ask"), "Close asks too");
            cx.nav.clear();
            cx.answers.push(DISCARD.into());
            frame(map, cx, g, 1.0);
            frame(map, cx, g, 1.0);
            assert!(widget(map, "shop").is_none() && widget(map, "dialogue").is_some(), "Yes: back in the conversation");
            assert!(*cx.gs() == before, "nothing traded");
            // again, with an empty trade: Esc closes at once
            let choices = json::aget(json::dict(&map.state(cx)["dialogue"]), "choices").to_vec();
            let k = choices.iter().position(|c| s(json::dict(c), "service") == "shop").expect("the shop reply");
            map.click_id(cx, &format!("choice.{k}")).expect("the shop reply");
            frame(map, cx, g, 1.0);
            assert!(sells(map, cx).is_empty(), "a reopened shop starts with an empty trade");
            map.back(cx);
            assert!(cx.nav.is_empty());
            frame(map, cx, g, 1.0);
            assert!(widget(map, "shop").is_none());
            false
        });
    }

    /// The question `back` pushed (ASKNO1), taken off `cx.nav`.
    fn take_question(cx: &mut Cx) -> Box<dyn Screen<GameCx>> {
        let Some(Nav::Push(q)) = cx.nav.pop() else { panic!("a question") };
        assert!(cx.nav.is_empty());
        q
    }

    /// 4.3 rule 5 "No focused" (ASKNO1): "Discard this trade?" has No as the default: Enter keeps the trade, `y`
    /// discards it.
    #[test]
    fn enter_at_discard_keeps_the_trade_and_y_discards() {
        let (mut cx, mut g) = (game_cx(), gfx());
        each_shop(&mut cx, &mut g, |map, cx, g, q| {
            fill_stash(cx);
            frame(map, cx, g, 1.0);
            let d = cx.data.clone();
            let item = shop::sell_list(&d, cx.gs(), q).first().map(|r| s(r, "item")).expect("something the shop buys");
            map.click_id(cx, &format!("stash.row.{item}")).expect("stash row");
            frame(map, cx, g, 1.0);
            let (before, trade) = (cx.gs().clone(), sells(map, cx));
            map.back(cx);
            let mut ask = take_question(cx);
            assert_eq!((ask.id(), ask.state(cx)["question"].clone(), ask.state(cx)["default"].clone()), ("ask", json!("Discard this trade?"), json!("no")));
            assert!(ask.auto_focus(), "No has the focus as it opens");
            ask.key(cx, Key::Enter);
            assert!(matches!(cx.nav.as_slice(), [Nav::Pop]) && cx.answers.is_empty(), "Enter: No");
            cx.nav.clear();
            frame(map, cx, g, 1.0);
            frame(map, cx, g, 1.0);
            assert!(widget(map, "shop").is_some() && sells(map, cx) == trade, "Enter keeps the shop and the trade");
            map.back(cx);
            let mut ask = take_question(cx);
            ask.key(cx, Key::Char('y'));
            assert!(matches!(cx.nav.as_slice(), [Nav::Pop]) && cx.answers == [DISCARD], "y: Yes");
            cx.nav.clear();
            frame(map, cx, g, 1.0);
            frame(map, cx, g, 1.0);
            assert!(widget(map, "shop").is_none() && widget(map, "dialogue").is_some(), "y discards the trade and leaves the shop");
            assert!(*cx.gs() == before, "nothing traded");
            false
        });
    }

    /// 4.2 layout and TEXTFIT1: 600 x 300 at Normal (and Small), grown with the text, centred under the HUD bar; every
    /// control inside; no "Esc or right click" hint; a mixed trade with a full stash (the lists scroll) and a hovered row
    /// reads clean at all four text sizes: nothing past the canvas, overlapping or out of its control; only names and
    /// the detail line shorten, with the full text in their control.
    #[test]
    fn the_trade_reads_clean_at_every_text_size() {
        let (mut cx, mut g) = (game_cx(), gfx());
        each_shop(&mut cx, &mut g, |map, cx, g, q| {
            fill_stash(cx);
            cx.gs_mut().party.gold = 40;
            frame(map, cx, g, 1.0);
            let d = cx.data.clone();
            for o in shop::offers(&d, cx.gs(), q).into_iter().take(3) {
                let _ = map.click_id(cx, &format!("shop.row.{}", s(&o, "id")));
            }
            for r in shop::sell_list(&d, cx.gs(), q).into_iter().take(2) {
                let _ = map.click_id(cx, &format!("stash.row.{}", s(&r, "item")));
            }
            for (size, scale) in TEXT_SIZES {
                frame(map, cx, g, scale as f32);
                // the pointer on the first shop row: its detail line shows
                let first = map.widgets().into_iter().find(|w| w["id"].as_str().is_some_and(|i| i.starts_with("shop.row."))).expect("a shop row");
                let (x, y, w, h) = rect(&first);
                let at = ((x + w / 2) as i32, (y + h / 2) as i32);
                frame_at(map, cx, g, at, scale as f32);
                let recs = frame_at(map, cx, g, at, scale as f32);
                let widgets = map.widgets();
                let (fx, fy, fw, fh) = rect(&widget(map, "shop").expect("the shop"));
                if scale <= 1.0 {
                    assert_eq!((fw, fh), (SHOP_W as i64, SHOP_H as i64), "{size}");
                }
                assert!((fx - (W as i64 - fx - fw)).abs() <= 1 && (fy - HUD_H as i64 - (H as i64 - fy - fh)).abs() <= 1, "{size}: centred under the HUD bar");
                assert!(fx >= EDGE as i64 && fy >= (HUD_H + EDGE) as i64, "{size}: inside the canvas under the HUD");
                // (a row scrolled out of its list is cut to nothing)
                for w in widgets.iter().filter(|w| w["id"] != "shop" && rect(w).2 > 0 && rect(w).3 > 0 && rect(w).1 >= fy) {
                    let (x, y, ww, hh) = rect(w);
                    assert!(x >= fx && y >= fy && x + ww <= fx + fw && y + hh <= fy + fh, "{size}: {w} inside the shop");
                }
                assert!(widget(map, "shop.detail").is_some(), "{size}: the hovered row's detail line");
                assert!(!recs.iter().any(|r| r.text.contains("right click")), "{size}: no close hint line");
                let in_shop: Vec<TextRec> = recs.into_iter().filter(|r| r.y >= fy as f32).collect();
                let boxes: Vec<(String, f32, f32, f32, f32)> = widgets
                    .iter()
                    .map(|w| {
                        let (x, y, ww, hh) = rect(w);
                        (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                    })
                    .collect();
                let v = aegis_ui::text_audit::issues(&in_shop, W as f32, H as f32, &boxes);
                let c = &v["counts"];
                assert_eq!((c["edge"].clone(), c["overlap"].clone(), c["spill"].clone()), (json!(0), json!(0), json!(0)), "{q} at {size}: {v}");
                for cut in v["cut"].as_array().into_iter().flatten() {
                    let full = cut["text"].as_str().unwrap_or("");
                    let named = widgets.iter().any(|w| w["text"].as_str().is_some_and(|t| t.starts_with(full)) || w["tip"][0].as_str() == Some(full));
                    assert!(named, "{q} at {size}: '{full}' is shortened without its full text in its control");
                }
            }
            true
        });
    }
}
