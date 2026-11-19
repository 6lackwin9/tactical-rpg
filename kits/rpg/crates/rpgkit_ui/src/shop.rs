//! The shop as a trade (PLAYUI1-P; KIT-UI1 KU2, ENGINE-8: moved from the game's GUI with no change in behaviour):
//! three columns over the screen the game puts under it, the shop's offers, the trade and the party's stash. Clicks
//! only: a row click adds 1, `+5` / `+10` add more (at most what is there), a click on a trade line puts the whole
//! line back; Confirm settles the trade at once through `rpgkit_world::shop::trade` (ShopRules), Close or Esc with
//! lines in the trade asks first. The game hosts it (`ShopGame`): its data and game state, its purse words, an item's
//! detail line and icon files, its log and the HUD height above the frame. Its view-model is `crate::shop_view`, drawn
//! through the layout `ui/layouts/shop.json` (UISPLIT1-G Stage B).

use aegis_core::json::{self, s, Dict};
use aegis_ui::canvas::Rect;
use aegis_ui::painter::Painter;
use aegis_ui::screen_layout::{self, Act, Field, Layout, Layouts};
use aegis_ui::ui::{Ui, W, H};
use aegis_ui::Cx;
use rpgkit_world::terms::CURRENCY;
use rpgkit_world::{shop, Data, GameState};
use serde_json::{json, Value};
use std::sync::Arc;

/// What a game gives the shop: the screen context's game half (`aegis_ui::Cx::app`) implements it.
pub trait ShopGame: aegis_ui::Context {
    /// The game's data.
    fn data(&self) -> &Arc<Data>;
    /// The running game's state.
    fn gs(&self) -> &GameState;
    fn gs_mut(&mut self) -> &mut GameState;
    /// Term `id`'s short form ("Gold", "Coin") and its word inside a sentence, in the world being played
    /// (`rpgkit_world::terms`).
    fn term_short(&self, id: &str) -> String;
    fn term_word(&self, id: &str) -> String;
    /// An item's detail line ("" = travel supplies): its hover card and the detail line under the columns.
    fn item_line(&self, item: &str) -> String;
    /// The icon files to try for item `id`, best first.
    fn item_icons(&self, id: &str) -> Vec<String>;
    /// A line for the game's event log (the settled trade).
    fn log(&mut self, line: String);
    /// The height of the HUD over the screen: the frame is centred under it.
    fn hud_h(&self) -> i32;
}

/// The tag of the "Discard this trade?" question (`Cx::ask_tag_no`: No is the default, PLAYUI1 4.3 rule 5);
/// the game's screen takes the Yes and leaves the shop.
pub const DISCARD: &str = "shop.discard";
/// The frame at text size Normal (spec 4.2); bigger text grows it, centred, up to `EDGE` from the canvas and the HUD.
pub const SHOP_W: i32 = 600;
pub const SHOP_H: i32 = 300;
pub const EDGE: i32 = 8;
/// A click in the shop.
#[derive(Debug, Clone)]
pub enum ShopAct {
    /// Up to n more of an offer (kind, id) into the trade.
    Buy(String, String, i64),
    /// Up to n more of a stash item into the trade.
    Sell(String, i64),
    /// The whole buy line of an offer (kind, id) back out.
    ReturnBuy(String, String),
    /// The whole sale line of an item back out.
    ReturnSell(String),
    Clear,
    Confirm,
    Close,
}

/// The trade being put together: buy lines (kind, id, count) and sale lines (item, count), each in the order added.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trade {
    pub buys: Vec<(String, String, i64)>,
    pub sells: Vec<(String, i64)>,
}

impl Trade {
    pub fn is_empty(&self) -> bool {
        self.buys.is_empty() && self.sells.is_empty()
    }

    /// How many of an offer the trade buys.
    pub fn bought(&self, kind: &str, id: &str) -> i64 {
        self.buys.iter().find(|b| b.0 == kind && b.1 == id).map_or(0, |b| b.2)
    }

    /// How many of a stash item the trade sells.
    pub fn sold(&self, item: &str) -> i64 {
        self.sells.iter().find(|x| x.0 == item).map_or(0, |x| x.1)
    }

    /// Adds `min(n, cap - in the trade)` of an offer (none at or past the cap); the number added.
    pub fn add_buy(&mut self, kind: &str, id: &str, n: i64, cap: i64) -> i64 {
        let add = n.min(cap - self.bought(kind, id)).max(0);
        if add > 0 {
            match self.buys.iter_mut().find(|b| b.0 == kind && b.1 == id) {
                Some(b) => b.2 += add,
                None => self.buys.push((kind.into(), id.into(), add)),
            }
        }
        add
    }

    /// Adds `min(n, owned - in the trade)` of a stash item; the number added.
    pub fn add_sell(&mut self, item: &str, n: i64, owned: i64) -> i64 {
        let add = n.min(owned - self.sold(item)).max(0);
        if add > 0 {
            match self.sells.iter_mut().find(|x| x.0 == item) {
                Some(x) => x.1 += add,
                None => self.sells.push((item.into(), add)),
            }
        }
        add
    }

    /// (spent, earned) at the shop's prices: `offers` (`shop::offers`) and `sale` (`shop::sell_list`).
    pub fn sums(&self, offers: &[Dict], sale: &[Dict]) -> (i64, i64) {
        let price = |kind: &str, id: &str| offers.iter().find(|o| s(o, "kind") == kind && s(o, "id") == id).map_or(0, |o| json::i(o, "price", 0));
        let worth = |item: &str| sale.iter().find(|r| s(r, "item") == item).map_or(0, |r| json::i(r, "price", 0));
        (self.buys.iter().map(|b| price(&b.0, &b.1) * b.2).sum(), self.sells.iter().map(|x| worth(&x.0) * x.1).sum())
    }
}

/// How many of an offer one trade may hold: what is left (unlimited stock: BUY_MAX), at most BUY_MAX; loot has one.
pub fn buy_cap(o: &Dict) -> i64 {
    let left = json::i(o, "left", 0);
    if left == shop::UNLIMITED {
        shop::BUY_MAX
    } else {
        left.clamp(0, shop::BUY_MAX)
    }
}

/// "X of N" (an unlimited offer shows X only, blank at 0).
pub fn of_text(x: i64, n: Option<i64>) -> String {
    match n {
        Some(n) => format!("{x} of {n}"),
        None if x > 0 => x.to_string(),
        None => String::new(),
    }
}

/// The frame at the current text size: 600 x 300 at Normal, grown with the text (never shrunk), centred under the HUD.
pub fn frame_rect(p: &Painter, hud_h: i32) -> Rect {
    let sc = p.fonts.scale;
    let fw = ((SHOP_W as f32 * sc).round() as i32).clamp(SHOP_W, W - 2 * EDGE);
    let fh = ((SHOP_H as f32 * sc).round() as i32).clamp(SHOP_H, H - hud_h - 2 * EDGE);
    Rect::new((W - fw) / 2, hud_h + (H - hud_h - fh) / 2, fw, fh)
}

/// The trade pane: the trade, the detail line of the row last under the pointer (or focused), the last refusal and
/// the layouts it draws with (PR7-39 Stage B: `ui/layouts/shop.json`, the kit's unless the game has its own).
#[derive(Debug, Default)]
pub struct ShopPane {
    pub trade: Trade,
    /// The note of the row last under the pointer or focused (its detail line), kept after the pointer leaves.
    pub(crate) detail: Option<String>,
    pub(crate) errors: Vec<String>,
    layouts: Option<PaneLayouts>,
}

/// The kit's shop layout (`kits/rpg/assets/ui/layouts/shop.json`), built in for a context with no asset layers (tests,
/// tools) or a layer file that does not fit the view.
pub fn builtin_layout() -> Option<Layout> {
    static L: std::sync::OnceLock<Option<Layout>> = std::sync::OnceLock::new();
    L.get_or_init(|| serde_json::from_str(include_str!("../../../assets/ui/layouts/shop.json")).ok().and_then(|v| Layout::from_json(&v).ok())).clone()
}

/// The pane's layout cache (made on its first draw, from the art's layers).
#[derive(Default)]
struct PaneLayouts(Layouts);

impl std::fmt::Debug for PaneLayouts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Layouts")
    }
}

impl ShopPane {
    /// The last refusal's lines (empty: none).
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// The agent link's view: the trade's lines and the last refusal.
    pub fn state(&self) -> Value {
        json!({"buys": self.trade.buys.iter().map(|b| json!([b.0, b.1, b.2])).collect::<Vec<_>>(),
            "sells": self.trade.sells.iter().map(|x| json!([x.0, x.1])).collect::<Vec<_>>(), "errors": self.errors})
    }

    /// Applies a click in shop `sq`; true when the shop should close (Close or Esc with an empty trade). Close with lines
    /// asks "Discard this trade?" (tag `DISCARD`).
    pub fn act<G: ShopGame>(&mut self, cx: &mut Cx<G>, sq: &str, a: ShopAct) -> bool {
        let d = cx.app.data().clone();
        match a {
            ShopAct::Buy(kind, id, n) => {
                if let Some(o) = shop::offers(&d, cx.app.gs(), sq).into_iter().find(|o| s(o, "kind") == kind && s(o, "id") == id) {
                    self.trade.add_buy(&kind, &id, n, buy_cap(&o));
                }
                self.errors.clear();
            }
            ShopAct::Sell(item, n) => {
                if let Some(r) = shop::sell_list(&d, cx.app.gs(), sq).into_iter().find(|r| s(r, "item") == item) {
                    self.trade.add_sell(&item, n, json::i(&r, "count", 0));
                }
                self.errors.clear();
            }
            ShopAct::ReturnBuy(kind, id) => {
                self.trade.buys.retain(|b| !(b.0 == kind && b.1 == id));
                self.errors.clear();
            }
            ShopAct::ReturnSell(item) => {
                self.trade.sells.retain(|x| x.0 != item);
                self.errors.clear();
            }
            ShopAct::Clear => {
                self.trade = Trade::default();
                self.errors.clear();
            }
            ShopAct::Confirm => {
                let (bought, sold): (i64, i64) = (self.trade.buys.iter().map(|b| b.2).sum(), self.trade.sells.iter().map(|x| x.1).sum());
                let res = shop::trade(&d, cx.app.gs_mut(), sq, &self.trade.buys, &self.trade.sells);
                if json::b(&res, "ok", false) {
                    let (spent, earned) = (json::i(&res, "spent", 0), json::i(&res, "earned", 0));
                    let mut parts = Vec::new();
                    let cur = cx.app.term_word(CURRENCY);
                    if bought > 0 {
                        parts.push(format!("bought {bought} {} for {spent} {cur}", if bought == 1 { "item" } else { "items" }));
                    }
                    if sold > 0 {
                        parts.push(format!("sold {sold} {} for {earned} {cur}", if sold == 1 { "item" } else { "items" }));
                    }
                    cx.app.log(format!("Trade: {}.", parts.join(", ")));
                    for n in json::sget(&res, "notes") {
                        cx.app.log(n);
                    }
                    self.trade = Trade::default();
                    self.errors.clear();
                } else {
                    self.errors = json::sget(&res, "errors");
                }
            }
            ShopAct::Close => {
                if self.trade.is_empty() {
                    return true;
                }
                cx.ask_tag_no("Discard this trade?", DISCARD);
            }
        }
        false
    }

    /// Draws the trade of shop `sq` (spec 4.2) through its layout (UISPLIT1-G Stage B): the view `shop_view::view` in
    /// the frame (`frame_rect`), each click mapped to its `ShopAct`; `wrap` makes a screen action of a shop click. The
    /// layout's problems go to the shell's notice; the row under the pointer (or focused) gives the detail line.
    pub fn draw<A: Clone, G: ShopGame>(&mut self, ui: &mut Ui<A>, cx: &mut Cx<G>, sq: &str, wrap: impl Fn(ShopAct) -> A) {
        let fr = frame_rect(&ui.p, cx.app.hud_h());
        let art = &mut cx.art;
        let view = crate::shop_view::view_with(&cx.app, self, sq, &mut |chain| art.first_rel(chain));
        let layers = cx.art.layers().clone();
        let layouts = &mut self.layouts.get_or_insert_with(|| PaneLayouts(Layouts::new(layers))).0;
        // the asset layers' layout (a game's own, else the kit's file), else the kit's built in: the same file
        let layout = layouts.get(&view).cloned().or_else(|| builtin_layout().filter(|l| l.check(&view).is_empty()));
        cx.notice.extend(layouts.take_problems());
        let title = view.get("title").and_then(Field::as_text).unwrap_or_default();
        ui.push("shop", "panel", title, fr, true, false, None, Vec::new());
        let kinds: Vec<(String, String)> = shop::offers(cx.app.data(), cx.app.gs(), sq).iter().map(|o| (s(o, "id"), s(o, "kind"))).collect();
        let art = &mut cx.art;
        let drawn = ui.nest(|a: Act| wrap(crate::shop_view::shop_act_of(&kinds, &a)), |sub| screen_layout::draw(sub, art, layout.as_ref(), &view, fr));
        if let Some((_, note)) = drawn.hot {
            self.detail = Some(note);
        }
    }
}

/// The trade's sums and caps, with no game: what the shop's clicks rely on.
#[cfg(test)]
mod tests {
    use super::*;

    /// The kit's layout file parses (the built-in copy is that file).
    #[test]
    fn the_builtin_layout_parses() {
        let l = builtin_layout().expect("kits/rpg/assets/ui/layouts/shop.json parses");
        assert_eq!(l.screen, "shop");
    }

    #[test]
    fn a_trade_adds_up_to_the_cap_and_sums_at_the_shops_prices() {
        let mut t = Trade::default();
        assert!(t.is_empty());
        assert_eq!((t.add_buy("stock", "potion", 5, 6), t.add_buy("stock", "potion", 5, 6), t.add_buy("stock", "potion", 1, 6)), (5, 1, 0));
        assert_eq!(t.add_sell("pelt", 10, 3), 3);
        assert_eq!(t.add_sell("pelt", 1, 3), 0, "no more than is owned");
        assert_eq!((t.bought("stock", "potion"), t.sold("pelt"), t.bought("loot", "potion")), (6, 3, 0));
        let offer = |kind: &str, id: &str, price: i64| json!({"kind": kind, "id": id, "price": price}).as_object().cloned().unwrap();
        let offers = vec![offer("stock", "potion", 30)];
        let sale = vec![json!({"item": "pelt", "price": 12}).as_object().cloned().unwrap()];
        assert_eq!(t.sums(&offers, &sale), (180, 36));
        let cap = |left: i64| buy_cap(&json!({"left": left}).as_object().cloned().unwrap());
        assert_eq!((cap(shop::UNLIMITED), cap(3), cap(0), cap(shop::BUY_MAX + 5)), (shop::BUY_MAX, 3, 0, shop::BUY_MAX));
        assert_eq!((of_text(2, Some(6)), of_text(2, None), of_text(0, None)), ("2 of 6".to_string(), "2".to_string(), String::new()));
    }
}
