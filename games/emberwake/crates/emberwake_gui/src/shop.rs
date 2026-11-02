//! The shop as a trade (PLAYUI1-P, games/emberwake/docs/specs/PLAYUI1.md 4; it replaces the Buy / Sell tabs ported
//! from scripts/ui/overworld/shop_panel.gd): three columns over the dimmed region map, the shop's offers, the trade and
//! the party's stash. Clicks only: a row click adds 1, `+5` / `+10` add more (at most what is there), a click on a
//! trade line puts the whole line back; Confirm settles the trade at once through `shop::trade` (ShopRules), Close or
//! Esc with lines in the trade asks first. `RegionMap` draws it while its conversation has a shop open.

use crate::hud::{term_short, term_word, HUD_H};
use crate::screens::Cx;
use aegis_core::json::{self, s, s_or, Dict};
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::ui::{tip, Ui, BODY, RADIUS, SMALL, W, H};
use emberwake_world::shop;
use emberwake_world::terms::{CURRENCY, UPKEEP};
use serde_json::{json, Value};

/// The tag of the "Discard this trade?" question (`Cx::ask_tag_no`: No is the default, PLAYUI1 4.3 rule 5); `RegionMap`
/// takes the Yes and leaves the shop.
pub const DISCARD: &str = "shop.discard";
/// The frame at text size Normal (spec 4.2); bigger text grows it, centred, up to `EDGE` from the canvas and the HUD.
const SHOP_W: i32 = 600;
const SHOP_H: i32 = 300;
const EDGE: i32 = 8;
/// The frame's inner margin and the gap between the columns.
const PAD: i32 = 7;
const GAP: i32 = 6;
/// The columns' shares (shop, trade, stash; spec 4.2 at Normal).
const COLS: [i32; 3] = [230, 150, 200];
/// A row's least height and its icon slot (filled when ICONUI1 lands).
const ROW_H: i32 = 14;
const ICON: i32 = 12;
/// The columns' fill.
const COL_BG: Rgba = rgb(14, 20, 34);
/// Buy lines and prices above the gold left: the kit has no warning colour of its own, its problem red stands in.

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
fn of_text(x: i64, n: Option<i64>) -> String {
    match n {
        Some(n) => format!("{x} of {n}"),
        None if x > 0 => x.to_string(),
        None => String::new(),
    }
}

/// "+90 HP · Owned 2 · Anyone can use it" style detail of an item ("" = travel supplies).
pub fn item_line(cx: &Cx, item: &str) -> String {
    if item.is_empty() {
        return format!("Travel {}: one per day on the road.", term_word(cx, UPKEEP));
    }
    let e = cx.data.entry("items", item);
    let owned = cx.game.as_ref().map(|g| g.gs.party.stash.get(item).copied().unwrap_or(0)).unwrap_or(0);
    let mut parts = Vec::new();
    let desc = s(e, "description");
    if !desc.is_empty() {
        parts.push(desc);
    }
    parts.push(format!("{} · Owned {owned}", s_or(e, "item_type", "ITEM").to_lowercase()));
    parts.join(" · ")
}

/// The frame at the current text size: 600 x 300 at Normal, grown with the text (never shrunk), centred under the HUD.
pub fn frame_rect(p: &Painter) -> Rect {
    let sc = p.fonts.scale;
    let fw = ((SHOP_W as f32 * sc).round() as i32).clamp(SHOP_W, W - 2 * EDGE);
    let fh = ((SHOP_H as f32 * sc).round() as i32).clamp(SHOP_H, H - HUD_H - 2 * EDGE);
    Rect::new((W - fw) / 2, HUD_H + (H - HUD_H - fh) / 2, fw, fh)
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
    detail: Option<String>,
    errors: Vec<String>,
}

impl ShopPane {
    /// The agent link's view: the trade's lines and the last refusal.
    pub fn state(&self) -> Value {
        json!({"buys": self.trade.buys.iter().map(|b| json!([b.0, b.1, b.2])).collect::<Vec<_>>(),
            "sells": self.trade.sells.iter().map(|x| json!([x.0, x.1])).collect::<Vec<_>>(), "errors": self.errors})
    }

    /// Applies a click in shop `sq`; true when the shop should close (Close or Esc with an empty trade). Close with lines
    /// asks "Discard this trade?" (tag `DISCARD`).
    pub fn act(&mut self, cx: &mut Cx, sq: &str, a: ShopAct) -> bool {
        let d = cx.data.clone();
        match a {
            ShopAct::Buy(kind, id, n) => {
                if let Some(o) = shop::offers(&d, cx.gs(), sq).into_iter().find(|o| s(o, "kind") == kind && s(o, "id") == id) {
                    self.trade.add_buy(&kind, &id, n, buy_cap(&o));
                }
                self.errors.clear();
            }
            ShopAct::Sell(item, n) => {
                if let Some(r) = shop::sell_list(&d, cx.gs(), sq).into_iter().find(|r| s(r, "item") == item) {
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
                let res = shop::trade(&d, cx.gs_mut(), sq, &self.trade.buys, &self.trade.sells);
                if json::b(&res, "ok", false) {
                    let (spent, earned) = (json::i(&res, "spent", 0), json::i(&res, "earned", 0));
                    let mut parts = Vec::new();
                    let cur = term_word(cx, CURRENCY);
                    if bought > 0 {
                        parts.push(format!("bought {bought} {} for {spent} {cur}", if bought == 1 { "item" } else { "items" }));
                    }
                    if sold > 0 {
                        parts.push(format!("sold {sold} {} for {earned} {cur}", if sold == 1 { "item" } else { "items" }));
                    }
                    cx.log(format!("Trade: {}.", parts.join(", ")));
                    for n in json::sget(&res, "notes") {
                        cx.log(n);
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
    pub fn draw<A: Clone>(&mut self, ui: &mut Ui<A>, cx: &Cx, sq: &str, wrap: impl Fn(ShopAct) -> A) {
        let d = &cx.data;
        let gs = cx.gs();
        let sh = shop::shop_of(d, gs, sq);
        let name = s_or(sh, "name", sq);
        let offers = shop::offers(d, gs, sq);
        let sale = shop::sell_list(d, gs, sq);
        let (spent, earned) = self.trade.sums(&offers, &sale);
        let after = gs.party.gold + earned - spent;
        // TERMS1 TU4: the purse names of the world being played (labels `short`, sentences `word`)
        let (cur, cur_word, upkeep) = (term_short(cx, CURRENCY), term_word(cx, CURRENCY), term_short(cx, UPKEEP));
        let fr = frame_rect(&ui.p);
        ui.p.panel(fr, th().panel, th().panel_edge, 4.0);
        ui.push("shop", "panel", name.clone(), fr, true, false, None, Vec::new());
        let (lb, ls) = (ui.p.line_h(BODY).ceil() as i32, ui.p.line_h(SMALL).ceil() as i32);
        let rh = ROW_H.max(lb + 3);
        // -- header: the shop's name and its keeper's faction band
        let (x0, right) = (fr.x + PAD, fr.x + fr.w - PAD);
        let fq = shop::keeper_faction(d, gs, sh);
        let band = if fq.is_empty() { String::new() } else { format!("{}: {}", s(d.resolve(&gs.world_id, "factions", &fq), "name"), shop::keeper_band(d, gs, sh)) };
        let band_w = if band.is_empty() { 0.0 } else { ui.p.text_w(&band, BODY) + 12.0 };
        ui.p.text_fit(x0 as f32, fr.y as f32 + 5.0, &name, 10.0, (fr.w - 2 * PAD) as f32 - band_w, th().gold);
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
            let text = format!("{label}: {}", item_line(cx, &item));
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
    fn list<A: Clone>(&mut self, ui: &mut Ui<A>, cx: &Cx, side: &str, area: Rect, rows: &[Row], gold_left: Option<i64>, others: &[(String, i64)], wrap: &impl Fn(ShopAct) -> A) {
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
        let price_w = rows.iter().map(|r| ui.p.text_w(&format!("{}g", r.price), BODY)).fold(0.0f32, f32::max).ceil() as i32;
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
            let price = format!("{}g", r.price);
            let price_col = if gold_left.is_some_and(|g| r.price > g) { th().foe } else if on { th().gold } else { th().dim };
            let count = of_text(r.x, r.n);
            let name_x = rr.x + 2 + ICON + 3;
            let price_right = count_right - count_w - 6;
            ui.p.text_fit(name_x as f32, ty, &r.label, BODY, (price_right - price_w - 6 - name_x) as f32, if on || r.x > 0 { th().text } else { th().dim });
            ui.p.text_right(price_right as f32, ty, &price, BODY, price_col);
            if !count.is_empty() {
                ui.p.text_right(count_right as f32, ty, &count, BODY, th().info);
            }
            let mut t = tip(&r.label, &item_line(cx, &r.item));
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
                let label = cx.data.item_name(item);
                let count = format!("x{n}");
                let cw = ui.p.text_w(&count, BODY).ceil() as i32;
                let name_x = rr.x + 2 + ICON + 3;
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

/// PLAYUI1-P (spec PLAYUI1 4.2-4.3): the trade screen on a debug game of the first playable world, its shops opened
/// the way a player opens them (the keeper's shop reply). Gold and the stash are set in memory; expectations come
/// from the shop's own offers and sell list.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use crate::region::RegionMap;
    use crate::screens::{GameCx, Nav};
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::{Key, Screen, TEXT_SIZES};
    use aegis_ui::ui::Gfx;

    fn widget(map: &RegionMap, id: &str) -> Option<Value> {
        map.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn on(map: &RegionMap, id: &str) -> bool {
        widget(map, id).is_some_and(|w| w["enabled"] == json!(true))
    }

    fn frame_at(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, at: (i32, i32), scale: f32) -> Vec<TextRec> {
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
    fn each_shop(cx: &mut Cx, g: &mut Gfx, mut each: impl FnMut(&mut RegionMap, &mut Cx, &mut Gfx, &str) -> bool) -> usize {
        let d = cx.data.clone();
        let start = cx.gs().clone();
        let mut opened = 0;
        for sh in d.merged_entries(&start.world_id, "shops") {
            let q = s(sh, "_qid");
            if !shop::is_open(&d, &start, &q) {
                continue;
            }
            *cx.gs_mut() = start.clone();
            let mut map = RegionMap::new(cx);
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

    fn buys(map: &RegionMap, cx: &Cx) -> Vec<(String, String, i64)> {
        let b = map.state(cx)["shop"]["buys"].clone();
        b.as_array().into_iter().flatten().map(|b| (b[0].as_str().unwrap_or("").to_string(), b[1].as_str().unwrap_or("").to_string(), b[2].as_i64().unwrap_or(0))).collect()
    }

    fn sells(map: &RegionMap, cx: &Cx) -> Vec<(String, i64)> {
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
                map.click_id(cx, &format!("shop.add5.{id}")).expect("+5");
                frame(map, cx, g, 1.0);
                let x = 7.min(cap);
                assert_eq!(buys(map, cx).last().cloned(), Some((kind.clone(), id.clone(), x)), "{id}");
                let text = widget(map, &format!("shop.row.{id}")).expect("the row")["text"].as_str().unwrap_or("").to_string();
                assert!(text.ends_with(&format!(", {}", of_text(x, n))), "{id}: '{text}' reads {}", of_text(x, n));
                while on(map, &format!("shop.add10.{id}")) {
                    map.click_id(cx, &format!("shop.add10.{id}")).expect("+10");
                    frame(map, cx, g, 1.0);
                }
                assert_eq!(buys(map, cx).last().map(|b| b.2), Some(cap), "{id}: +10 stops at the cap");
                assert!(!on(map, &format!("shop.row.{id}")) && !on(map, &format!("shop.add5.{id}")), "{id}: off at the cap");
                assert!(map.click_id(cx, &format!("shop.row.{id}")).is_err());
            }
            for r in shop::sell_list(&d, cx.gs(), q).into_iter().take(3) {
                let (item, owned) = (s(&r, "item"), json::i(&r, "count", 0));
                map.click_id(cx, &format!("stash.row.{item}")).expect("the stash row");
                if owned > 1 {
                    map.click_id(cx, &format!("stash.add10.{item}")).expect("+10");
                }
                frame(map, cx, g, 1.0);
                assert_eq!(sells(map, cx).last().cloned(), Some((item.clone(), owned)), "{item}: at most what is owned");
                assert!(!on(map, &format!("stash.row.{item}")), "{item}: off when all of it is in the trade");
            }
            // loot is one at a time, without `+5` / `+10`
            for o in shop::offers(&d, cx.gs(), q).into_iter().filter(|o| s(o, "kind") == "loot") {
                let id = s(&o, "id");
                assert!(widget(map, &format!("shop.add5.{id}")).is_none(), "{id}: no +5 on loot");
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
            map.click_id(cx, &format!("trade.buy.{id}")).expect("the buy line");
            frame(map, cx, g, 1.0);
            assert!(buys(map, cx).is_empty() && sells(map, cx).len() == 1, "the whole buy line went back");
            assert!(widget(map, &format!("trade.buy.{id}")).is_none());
            map.click_id(cx, &format!("trade.sell.{item}")).expect("the sale line");
            frame(map, cx, g, 1.0);
            assert!(sells(map, cx).is_empty());
            assert!(!on(map, "trade.confirm") && !on(map, "trade.clear"), "an empty trade: Confirm and Clear off");
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
            map.click_id(cx, "trade.confirm").expect("confirm");
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
            assert!(!on(map, "trade.confirm"), "0 gold, a purchase: Confirm off");
            let card = widget(map, "trade.confirm").expect("confirm")["tip"].to_string();
            assert!(card.contains(&format!("Not enough gold: {price} more needed.")), "{card}");
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
            assert!(on(map, "trade.confirm"), "the sales pay for the purchase");
            map.click_id(cx, "trade.confirm").expect("confirm");
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
            map.click_id(cx, "trade.clear").expect("clear");
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
