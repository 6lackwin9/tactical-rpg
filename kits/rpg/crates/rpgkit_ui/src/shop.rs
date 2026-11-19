//! The shop as a trade (PLAYUI1-P; KIT-UI1 KU2, ENGINE-8: moved from the game's GUI with no change in behaviour):
//! three columns over the screen the game puts under it, the shop's offers, the trade and the party's stash. Clicks
//! only: a row click adds 1, `+5` / `+10` add more (at most what is there), a click on a trade line puts the whole
//! line back; Confirm settles the trade at once through `rpgkit_world::shop::trade` (ShopRules), Close or Esc with
//! lines in the trade asks first. The game hosts it (`ShopGame`): its data and game state, its purse words, an item's
//! detail line and icon files, its log and the HUD height above the frame. Its view-model is `crate::shop_view`.

use aegis_core::json::{self, s, s_or, Dict};
use aegis_ui::canvas::{rgb, Rect, Rgba};
use aegis_ui::painter::{Painter, Typeface};
use aegis_ui::theme::th;
use aegis_ui::ui::{tip, Ui, BODY, RADIUS, SMALL, W, H};
use aegis_ui::Cx;
use rpgkit_world::terms::{CURRENCY, UPKEEP};
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
/// The frame's inner margin and the gap between the columns.
const PAD: i32 = 7;
const GAP: i32 = 6;
/// The columns' shares (shop, trade, stash; spec 4.2 at Normal).
const COLS: [i32; 3] = [230, 150, 200];
/// A row's least height and its icon slot (PR7-36, ART-05: a 32 px I2 tile at 1280 x 720).
const ROW_H: i32 = 18;
const ICON: i32 = 16;
/// The columns' fill.
const COL_BG: Rgba = rgb(14, 20, 34);
/// Buy lines and prices above the gold left: the kit has no warning colour of its own, its problem red stands in.

/// PR7-36 (ART-05): item `id`'s I2 tile in `r` (its icon chain; the bare tile when no file is there).

/// PR7-36 (ART-05): item `id`'s I2 tile in `r` (its icon chain; the bare tile when no file is there).
pub fn item_tile<A: Clone, G: ShopGame>(ui: &mut Ui<A>, cx: &mut Cx<G>, r: Rect, id: &str) {
    let px = ui.icon_px(r);
    let icons = cx.app.item_icons(id);
    let img = cx.art.first(&icons, px, px).cloned();
    aegis_ui::panes::icon_tile(ui, r, img.as_ref(), "");
}

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

/// "+90 HP · Owned 2 · Anyone can use it" style detail of an item ("" = travel supplies).
pub fn frame_rect(p: &Painter, hud_h: i32) -> Rect {
    let sc = p.fonts.scale;
    let fw = ((SHOP_W as f32 * sc).round() as i32).clamp(SHOP_W, W - 2 * EDGE);
    let fh = ((SHOP_H as f32 * sc).round() as i32).clamp(SHOP_H, H - hud_h - 2 * EDGE);
    Rect::new((W - fw) / 2, hud_h + (H - hud_h - fh) / 2, fw, fh)
}

/// One list row of the shop or stash column, as drawn.
struct Row {
    id: String,
    label: String,
    item: String,
    price: i64,
    x: i64,
    /// What is there (None: unlimited).
    n: Option<i64>,
    /// How many the trade may hold.
    cap: i64,
    /// `+5` / `+10` shown.
    more: bool,
    add: Box<dyn Fn(i64) -> ShopAct>,
}

/// The trade pane: the trade, the item under the pointer (its detail line) and the last refusal.
#[derive(Debug, Default)]
pub struct ShopPane {
    pub trade: Trade,
    /// The item last under the pointer (Some("") = supplies).
    pub(crate) detail: Option<String>,
    pub(crate) errors: Vec<String>,
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

    /// Draws the trade of shop `sq` (spec 4.2): header, the three columns, the detail line, refusals and the action
    /// bar; `wrap` makes a screen action of a shop click.
    pub fn draw<A: Clone, G: ShopGame>(&mut self, ui: &mut Ui<A>, cx: &mut Cx<G>, sq: &str, wrap: impl Fn(ShopAct) -> A) {
        let data = cx.app.data().clone();
        let d = &*data;
        let state = cx.app.gs().clone();
        let gs = &state;
        let sh = shop::shop_of(d, gs, sq);
        let name = s_or(sh, "name", sq);
        let offers = shop::offers(d, gs, sq);
        let sale = shop::sell_list(d, gs, sq);
        let (spent, earned) = self.trade.sums(&offers, &sale);
        let after = gs.party.gold + earned - spent;
        // TERMS1 TU4: the purse names of the world being played (labels `short`, sentences `word`)
        let (cur, cur_word, upkeep) = (cx.app.term_short(CURRENCY), cx.app.term_word(CURRENCY), cx.app.term_short(UPKEEP));
        let fr = frame_rect(&ui.p, cx.app.hud_h());
        // PR7-36 (ART-02): a main panel, the ornate tier
        ui.p.fill(fr, th().panel);
        ui.frame_body("ornate", fr);
        ui.push("shop", "panel", name.clone(), fr, true, false, None, Vec::new());
        let (lb, ls) = (ui.p.line_h(BODY).ceil() as i32, ui.p.line_h(SMALL).ceil() as i32);
        let rh = ROW_H.max(lb + 3);
        // -- header: the shop's name and its keeper's faction band
        let (x0, right) = (fr.x + PAD, fr.x + fr.w - PAD);
        let fq = shop::keeper_faction(d, gs, sh);
        let band = if fq.is_empty() { String::new() } else { format!("{}: {}", s(d.resolve(&gs.world_id, "factions", &fq), "name"), shop::keeper_band(d, gs, sh)) };
        let band_w = if band.is_empty() { 0.0 } else { ui.p.text_w(&band, BODY) + 12.0 };
        ui.p.text_fit_face(Typeface::Display, x0 as f32, fr.y as f32 + 5.0, &name, 10.0, (fr.w - 2 * PAD) as f32 - band_w, th().gold);
        let head_h = ui.p.line_h(10.0).ceil() as i32;
        if !band.is_empty() {
            ui.p.text_right(right as f32, (fr.y + 5 + head_h - lb) as f32, &band, BODY, th().dim);
        }
        // -- the columns' place: under the headings, over the detail line, the refusals and the action bar
        let bh = 17.max(lb + 6);
        let bar_y = fr.y + fr.h - PAD - bh;
        let err_y = bar_y - 3 - lb;
        let detail_y = err_y - lb - 1;
        let heading_y = fr.y + 5 + head_h + 4;
        let top = heading_y + ls + 2;
        let ch = detail_y - 4 - top;
        let avail = fr.w - 2 * PAD - 2 * GAP;
        let total: i32 = COLS.iter().sum();
        let w0 = avail * COLS[0] / total;
        let w1 = avail * COLS[1] / total;
        let w2 = avail - w0 - w1;
        let (cx0, cx1, cx2) = (x0, x0 + w0 + GAP, x0 + w0 + GAP + w1 + GAP);
        for (x, label) in [(cx0, "Shop"), (cx1, "Trade"), (cx2, "Stash")] {
            ui.p.text(x as f32 + 2.0, heading_y as f32, label, SMALL, th().dim);
        }
        // -- the shop column: one row per offer
        let shop_rows: Vec<Row> = offers
            .iter()
            .map(|o| {
                let (kind, id) = (s(o, "kind"), s(o, "id"));
                let supplies = kind == shop::SUPPLIES;
                let item = if supplies { String::new() } else { s(o, "item") };
                let mut label = if supplies { upkeep.clone() } else { d.item_name(&item) };
                if kind == "loot" && json::i(o, "count", 1) > 1 {
                    label = format!("{label} x{}", json::i(o, "count", 1));
                }
                let cap = buy_cap(o);
                let unlimited = json::i(o, "left", 0) == shop::UNLIMITED;
                let (k, i) = (kind.clone(), id.clone());
                Row { id, label, item, price: json::i(o, "price", 0), x: self.trade.bought(&kind, &i), n: (!unlimited).then_some(cap), cap, more: kind != "loot" && cap > 1, add: Box::new(move |n| ShopAct::Buy(k.clone(), i.clone(), n)) }
            })
            .collect();
        self.list(ui, cx, "shop", Rect::new(cx0, top, w0, ch), &shop_rows, Some(after), &[], &wrap);
        // -- the trade column: buy lines, then sale lines; the totals under them
        let tot_h = 2 * lb + 4;
        let lines_r = Rect::new(cx1, top, w1, ch - tot_h);
        ui.p.panel(Rect::new(cx1, top, w1, ch), th().game("column", COL_BG), th().edge, RADIUS);
        let mut lines: Vec<(String, String, i64, String, ShopAct)> = Vec::new();
        for (kind, id, n) in &self.trade.buys {
            let Some(o) = offers.iter().find(|o| s(o, "kind") == *kind && s(o, "id") == *id) else { continue };
            let label = if kind == shop::SUPPLIES { upkeep.clone() } else { d.item_name(&s(o, "item")) };
            let item = if kind == shop::SUPPLIES { String::new() } else { s(o, "item") };
            lines.push((format!("trade.buy.{id}"), format!("{label} x{n}"), -json::i(o, "price", 0) * n, item, ShopAct::ReturnBuy(kind.clone(), id.clone())));
        }
        for (item, n) in &self.trade.sells {
            let price = sale.iter().find(|r| s(r, "item") == *item).map_or(0, |r| json::i(r, "price", 0));
            lines.push((format!("trade.sell.{item}"), format!("{} x{n}", d.item_name(item)), price * n, item.clone(), ShopAct::ReturnSell(item.clone())));
        }
        let off = ui.scroll_begin("trade.list", lines_r, lines.len() as i32 * (rh + 1) + 2);
        if lines.is_empty() {
            ui.p.text_fit(lines_r.x as f32 + 4.0, lines_r.y as f32 + 3.0, "Click a row to add it.", SMALL, (lines_r.w - 8) as f32, th().dim);
        }
        for (k, (id, label, gold, item, act)) in lines.into_iter().enumerate() {
            let r = Rect::new(lines_r.x + 1, lines_r.y + 1 - off + k as i32 * (rh + 1), lines_r.w - 5, rh);
            let hover = ui.hover(r);
            if hover {
                ui.p.panel(r, th().card_hot, th().edge_hot, RADIUS);
                self.detail = Some(item.clone());
            }
            let amount = format!("{gold:+}");
            let aw = ui.p.text_w(&amount, BODY);
            let ty = r.y as f32 + (r.h as f32 - ui.p.line_h(BODY)) / 2.0;
            ui.p.text_fit(r.x as f32 + 3.0, ty, &label, BODY, r.w as f32 - aw - 10.0, th().text);
            ui.p.text_right((r.x + r.w) as f32 - 3.0, ty, &amount, BODY, if gold < 0 { th().foe } else { th().good });
            ui.push(id, "row", format!("{label} {amount}"), r, true, false, Some(wrap(act)), tip(&label, "Click to put the whole line back."));
        }
        ui.scroll_end();
        let net = earned - spent;
        let ty = (top + ch - tot_h + 2) as f32;
        let net_text = if net == 0 { format!("Total: 0 {cur_word}") } else { format!("Total: {net:+} {cur_word}") };
        ui.p.text_fit(cx1 as f32 + 4.0, ty, &net_text, BODY, (w1 - 8) as f32, if net < 0 { th().foe } else if net > 0 { th().good } else { th().text });
        ui.p.text_fit(cx1 as f32 + 4.0, ty + lb as f32, &format!("{cur}: {} -> {after}", gs.party.gold), BODY, (w1 - 8) as f32, if after < 0 { th().foe } else { th().text });
        // -- the stash column: what the shop buys, then (below a divider, dimmed) what it does not
        let stash_rows: Vec<Row> = sale
            .iter()
            .map(|r| {
                let item = s(r, "item");
                let owned = json::i(r, "count", 0);
                let i = item.clone();
                Row { id: item.clone(), label: d.item_name(&item), item: item.clone(), price: json::i(r, "price", 0), x: self.trade.sold(&item), n: Some(owned), cap: owned, more: owned > 1, add: Box::new(move |n| ShopAct::Sell(i.clone(), n)) }
            })
            .collect();
        let others: Vec<(String, i64)> = gs.party.stash.iter().filter(|(id, n)| **n > 0 && !sale.iter().any(|r| s(r, "item") == **id)).map(|(id, n)| (id.clone(), *n)).collect();
        self.list(ui, cx, "stash", Rect::new(cx2, top, w2, ch), &stash_rows, None, &others, &wrap);
        // -- the detail line of the item last under the pointer, the refusals, the action bar
        if let Some(item) = self.detail.clone() {
            let label = if item.is_empty() { upkeep.clone() } else { d.item_name(&item) };
            let text = format!("{label}: {}", cx.app.item_line(&item));
            let r = Rect::new(x0, detail_y, fr.w - 2 * PAD, lb);
            ui.p.text_fit(r.x as f32, r.y as f32, &text, BODY, r.w as f32, th().info);
            ui.push("shop.detail", "label", text.clone(), r, true, false, None, Vec::new());
        }
        if !self.errors.is_empty() {
            let text = self.errors.join(" ");
            let r = Rect::new(x0, err_y, fr.w - 2 * PAD, lb);
            ui.p.text_fit(r.x as f32, r.y as f32, &text, BODY, r.w as f32, th().foe);
            ui.push("trade.errors", "label", text.clone(), r, true, false, None, Vec::new());
        }
        let bw = |ui: &mut Ui<A>, t: &str| (ui.p.text_w(t, BODY).ceil() as i32 + 16).max(44);
        let (confirm_w, clear_w, close_w) = (bw(ui, "Confirm trade"), bw(ui, "Clear"), bw(ui, "Close"));
        let empty = self.trade.is_empty();
        let why = if empty {
            "Nothing to trade.".to_string()
        } else if after < 0 {
            format!("Not enough {cur_word}: {} more needed.", -after)
        } else {
            String::new()
        };
        let confirm_r = Rect::new(right - confirm_w, bar_y, confirm_w, bh);
        let clear_r = Rect::new(confirm_r.x - 6 - clear_w, bar_y, clear_w, bh);
        ui.button_full("trade.clear", clear_r, "Clear", wrap(ShopAct::Clear), !empty, false, BODY, tip("Clear", "Put every line back."));
        let confirm_tip = if why.is_empty() { tip("Confirm trade", &format!("{cur}: {} -> {after}", gs.party.gold)) } else { tip("Confirm trade", &why) };
        ui.button_full("trade.confirm", confirm_r, "Confirm trade", wrap(ShopAct::Confirm), why.is_empty(), false, BODY, confirm_tip);
        ui.button_full("shop.close", Rect::new(x0, bar_y, close_w, bh), "Close", wrap(ShopAct::Close), true, false, BODY, tip("Close", "Back to the conversation (Esc)."));
    }

    /// A shop or stash column (`side` "shop" | "stash"): its rows (icon slot, name, unit price, "X of N", `+5` / `+10`)
    /// in a scroll area; `gold_left` colours shop prices above it; `others` are stash items the shop does not buy,
    /// listed dimmed under a divider.
    #[allow(clippy::too_many_arguments)]
    fn list<A: Clone, G: ShopGame>(&mut self, ui: &mut Ui<A>, cx: &mut Cx<G>, side: &str, area: Rect, rows: &[Row], gold_left: Option<i64>, others: &[(String, i64)], wrap: &impl Fn(ShopAct) -> A) {
        let (lb, rh) = {
            let lb = ui.p.line_h(BODY).ceil() as i32;
            (lb, ROW_H.max(lb + 3))
        };
        let step = rh + 1;
        ui.p.panel(area, th().game("column", COL_BG), th().edge, RADIUS);
        let divider = if others.is_empty() { 0 } else { 6 };
        let content = (rows.len() + others.len()) as i32 * step + divider + 2;
        let off = ui.scroll_begin(&format!("{side}.list"), area, content);
        // the parts' widths from the texts at this size
        // PR7-41 (PT-30): prices in the world's currency ("30 Gold")
        let cur = cx.app.term_short(CURRENCY);
        let price_w = rows.iter().map(|r| ui.p.text_w(&format!("{} {cur}", r.price), BODY)).fold(0.0f32, f32::max).ceil() as i32;
        let count_w = rows.iter().map(|r| ui.p.text_w(&of_text(r.cap, r.n), BODY)).fold(0.0f32, f32::max).ceil() as i32;
        let (b5, b10) = (ui.p.text_w("+5", BODY).ceil() as i32 + 8, ui.p.text_w("+10", BODY).ceil() as i32 + 8);
        let full_w = area.w - 5;
        // prices and counts line up in every row, also in rows without `+5` / `+10`
        let button_w = if rows.iter().any(|r| r.more) { b5 + b10 + 4 } else { 0 };
        let count_right = area.x + 1 + full_w - button_w - 3;
        let mut y = area.y + 1 - off;
        for r in rows {
            let free = r.cap - r.x;
            let buttons = if r.more { b5 + b10 + 4 } else { 0 };
            let rr = Rect::new(area.x + 1, y, full_w - buttons, rh);
            let on = free > 0;
            let hover = ui.hover(rr);
            if r.x > 0 || (hover && on) {
                ui.p.panel(rr, if r.x > 0 { th().selected } else { th().card_hot }, if hover && on { th().gold } else { th().edge_hot }, RADIUS);
            }
            if hover {
                self.detail = Some(r.item.clone());
            }
            let ty = rr.y as f32 + (rh as f32 - ui.p.line_h(BODY)) / 2.0;
            let price = format!("{} {cur}", r.price);
            let price_col = if gold_left.is_some_and(|g| r.price > g) { th().foe } else if on { th().gold } else { th().dim };
            let count = of_text(r.x, r.n);
            let name_x = rr.x + 2 + ICON + 3;
            item_tile(ui, cx, Rect::new(rr.x + 1, rr.y + (rh - ICON) / 2, ICON, ICON), &r.item);
            let price_right = count_right - count_w - 6;
            ui.p.text_fit(name_x as f32, ty, &r.label, BODY, (price_right - price_w - 6 - name_x) as f32, if on || r.x > 0 { th().text } else { th().dim });
            ui.p.text_right(price_right as f32, ty, &price, BODY, price_col);
            if !count.is_empty() {
                ui.p.text_right(count_right as f32, ty, &count, BODY, th().info);
            }
            let mut t = tip(&r.label, &cx.app.item_line(&r.item));
            t.push((format!("{price} each"), th().gold));
            ui.push(format!("{side}.row.{}", r.id), "row", format!("{}, {price}, {count}", r.label), rr, on, r.x > 0, Some(wrap((r.add)(1))), t);
            if r.more {
                let r5 = Rect::new(rr.x + rr.w + 2, y + 1, b5, rh - 2);
                let r10 = Rect::new(r5.x + b5 + 2, y + 1, b10, rh - 2);
                ui.button_full(&format!("{side}.add5.{}", r.id), r5, "+5", wrap((r.add)(5)), on, false, BODY, tip(&format!("Add 5 {}", r.label), &format!("At most {free} more.")));
                ui.button_full(&format!("{side}.add10.{}", r.id), r10, "+10", wrap((r.add)(10)), on, false, BODY, tip(&format!("Add 10 {}", r.label), &format!("At most {free} more.")));
            }
            y += step;
        }
        if !others.is_empty() {
            ui.p.fill(Rect::new(area.x + 4, y + 2, full_w - 6, 1), th().edge);
            y += divider;
            for (item, n) in others {
                let rr = Rect::new(area.x + 1, y, full_w, rh);
                let ty = rr.y as f32 + (rh as f32 - lb as f32) / 2.0;
                let label = cx.app.data().item_name(item);
                let count = format!("x{n}");
                let cw = ui.p.text_w(&count, BODY).ceil() as i32;
                let name_x = rr.x + 2 + ICON + 3;
                item_tile(ui, cx, Rect::new(rr.x + 1, rr.y + (rh - ICON) / 2, ICON, ICON), item);
                ui.p.text_fit(name_x as f32, ty, &label, BODY, (rr.x + rr.w - 3 - cw - 6 - name_x) as f32, th().dim);
                ui.p.text_right((rr.x + rr.w - 3) as f32, ty, &count, BODY, th().dim);
                if ui.hover(rr) {
                    self.detail = Some(item.clone());
                }
                ui.push(format!("stash.item.{item}"), "row", format!("{label}, {count}"), rr, true, false, None, tip(&label, "Not bought here."));
                y += step;
            }
        }
        ui.scroll_end();
    }
}

/// The trade's sums and caps, with no game: what the shop's clicks rely on.
#[cfg(test)]
mod tests {
    use super::*;

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
