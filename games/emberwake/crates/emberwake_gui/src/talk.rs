//! The talk box (scripts/ui/overworld/dialogue_panel.gd's layout as PLAYUI1-D's 400 px box): one `DialogueRunner::view`
//! measured (`talk_layout`), drawn with its living portrait (`draw_talk`, PORTRAITFX1-D: `portraits::Faces`, the
//! speaker's look from `look_of_member` or `look_of_character`; PORTRAITFX1-X: the node's `expression` picks the
//! frame along `face_chain`), its keys (`talk_key`) and the agent link's reading of
//! it (`talk_state`). Shared by the region map's conversations and the battle's phase talks (ENCOUNTER2-C DT7: moved
//! out of `region.rs`, the look unchanged); each screen maps `TalkAct` to its own action and owns its runner.
//! SERVICES1 (SILOS1 T2): `Services`, the conversation with its shop, board, turn-in picker and rest ask, moved out of
//! `region.rs` (the region map holds one); `ServiceScreen` hosts one on a plain backdrop (the UI Gallery). PK2-S3: and
//! the recipe pane (`recipe.rs`) over the conversation, like the shop.

use crate::board::{BoardAct, BoardPane};
use crate::hud::{purse_word, PurseWord, HUD_H};
use crate::portraits::{Faces, Who};
use crate::recipe::{RecipeAct, RecipePane};
use crate::screens::{Cx, GameCx, Key, Nav, Screen};
use crate::shop::{ShopAct, ShopPane};
use aegis_core::json::{self, s, s_or, Dict};
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Gfx, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use emberwake_world::dialogue::DialogueRunner;
use emberwake_world::{quest, Data, GameState};
use serde_json::{json, Value};

/// The dialogue box (spec PLAYUI1 3.1, logical px at text size Normal): its width (times the text size, at most the
/// canvas less 40), its gap above the bottom edge, its least height (its most is `TALK_MAX_PCT` % of the canvas).
pub(crate) const TALK_W: i32 = 400;
pub(crate) const TALK_GAP: i32 = 6;
pub(crate) const TALK_MIN_H: i32 = 104;
pub(crate) const TALK_MAX_PCT: i32 = 45;
/// The left column (portrait, name, faction band) and the portrait in it.
pub(crate) const TALK_LEFT: i32 = 76;
pub(crate) const PORTRAIT: i32 = 64;
/// The box's inner margin.
pub(crate) const TALK_PAD: i32 = 6;
/// The NPC's line scrolls past this many lines, the replies past this many rows; a reply row is at least this tall.
const TALK_LINES: i32 = 4;
const TALK_ROWS: usize = 5;
const REPLY_H: i32 = 14;
/// The scroll views of the line and the replies (a new node starts both at the top: `follow_node`).
const TEXT_VIEW: &str = "dialogue.text";
const ROWS_VIEW: &str = "dialogue.choices";
/// The box's frame line.

/// What a click or a key on the box asks for: reply `i` (0-based), or Continue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TalkAct {
    Choose(i64),
    Advance,
}

/// A reply row of the dialogue box: its widget id, the number shown (1-9 choose it), the text the agent link reads,
/// the wrapped lines, locked, its action and its height.
pub(crate) struct Reply {
    id: String,
    num: String,
    label: String,
    lines: Vec<String>,
    locked: bool,
    act: TalkAct,
    h: i32,
}

/// The dialogue box measured for one frame (spec PLAYUI1 3.1): the box, its left column's width, the speaker's name
/// and faction band lines, the NPC's line in its (scrolling) view, and the replies in theirs.
pub(crate) struct TalkBox {
    pub(crate) r: Rect,
    left: i32,
    name: Vec<String>,
    band: Vec<String>,
    text: Vec<String>,
    text_view: Rect,
    replies: Vec<Reply>,
    num_w: i32,
    rows_view: Rect,
    rows_h: i32,
}

/// The rows of `view` (a `DialogueRunner::view`) in order: (widget id, text shown, locked, action): each choice (a
/// locked one with its reason), then Continue when the node goes on (or ends with no reply).
fn reply_rows(view: &Dict) -> Vec<(String, String, bool, TalkAct)> {
    let choices = json::aget(view, "choices");
    let mut out: Vec<(String, String, bool, TalkAct)> = choices
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let c = json::dict(c);
            let locked = json::b(c, "locked", false);
            let shown = if locked { format!("{}  ({})", s(c, "text"), s(c, "reason")) } else { s(c, "text") };
            (format!("choice.{i}"), shown, locked, TalkAct::Choose(i as i64))
        })
        .collect();
    if json::b(view, "next", false) || (json::b(view, "end", false) && choices.is_empty()) {
        out.push(("dialogue.continue".into(), "Continue".into(), false, TalkAct::Advance));
    }
    out
}

/// At most `n` lines of `lines`, the rest joined into the last one (drawn shortened, with the full text in the audit).
fn capped(mut lines: Vec<String>, n: usize) -> Vec<String> {
    if lines.len() > n {
        let rest = lines.split_off(n - 1).join(" ");
        lines.push(rest);
    }
    lines
}

/// Measures the dialogue box for `view` (a `DialogueRunner::view`) at the current text size: `TALK_W` wide (times the
/// text size, at most the canvas less 40), centred, `TALK_GAP` above the bottom; its height from the content, at least
/// `TALK_MIN_H`, at most `TALK_MAX_PCT` % of the canvas (the replies, then the line, scroll when they do not fit).
pub(crate) fn talk_layout(p: &mut Painter, view: &Dict) -> TalkBox {
    let sc = p.fonts.scale;
    let bw = (W - 40).min((TALK_W as f32 * sc).round() as i32);
    let left = TALK_LEFT.max((TALK_LEFT as f32 * sc).round() as i32);
    let (lb, ls) = (p.line_h(BODY).ceil() as i32, p.line_h(SMALL).ceil() as i32);
    let name = capped(p.wrap(&s(view, "speaker_name"), BODY, (left - 6) as f32), 2);
    let band_text = if s(view, "faction").is_empty() { String::new() } else { format!("{}: {}", s(view, "faction_name"), s(view, "band")) };
    let band = if band_text.is_empty() { Vec::new() } else { capped(p.wrap(&band_text, SMALL, (left - 6) as f32), 2) };
    let left_h = TALK_PAD + PORTRAIT + 3 + name.len() as i32 * lb + if band.is_empty() { 0 } else { 1 + band.len() as i32 * ls } + TALK_PAD;
    // the right part: the line on top, a 1 px divider, the replies below (3 px kept for a scroll bar)
    let rw = bw - left - 2 - TALK_PAD;
    let text = p.wrap(&s(view, "text"), BODY, (rw - 5) as f32);
    let text_h = text.len() as i32 * lb;
    let mut text_view_h = text_h.min(TALK_LINES * lb).max(lb);
    let num_w = p.text_w("8", BODY).ceil() as i32 + 6;
    let line_w = (rw - 4 - 5 - num_w - 4) as f32;
    let mut replies = Vec::new();
    for (i, (id, shown, locked, act)) in reply_rows(view).into_iter().enumerate() {
        let lines = p.wrap(&shown, BODY, line_w);
        let h = REPLY_H.max(lines.len() as i32 * lb + 5);
        let (num, label) = match act {
            TalkAct::Choose(_) => (if i < 9 { (i + 1).to_string() } else { String::new() }, format!("{}  {shown}", i + 1)),
            TalkAct::Advance => (String::new(), shown.clone()),
        };
        replies.push(Reply { id, num, label, lines, locked, act, h });
    }
    let rows_h: i32 = replies.iter().map(|r| r.h + 2).sum::<i32>() - if replies.is_empty() { 0 } else { 2 };
    let mut rows_view_h: i32 = replies.iter().take(TALK_ROWS).map(|r| r.h + 2).sum::<i32>() - if replies.is_empty() { 0 } else { 2 };
    let max_h = H * TALK_MAX_PCT / 100;
    let content = |text_view_h: i32, rows_view_h: i32| TALK_PAD + text_view_h + 9 + rows_view_h + TALK_PAD;
    let mut over = content(text_view_h, rows_view_h).max(left_h) - max_h;
    if over > 0 {
        let least = replies.first().map_or(0, |r| r.h);
        let cut = over.min(rows_view_h - least).max(0);
        rows_view_h -= cut;
        over -= cut;
        text_view_h -= over.min(text_view_h - lb).max(0);
    }
    let bh = content(text_view_h, rows_view_h).max(left_h).max(TALK_MIN_H).min(max_h);
    let r = Rect::new((W - bw) / 2, H - TALK_GAP - bh, bw, bh);
    let rx = r.x + left + 2;
    let text_view = Rect::new(rx, r.y + TALK_PAD, rw, text_view_h);
    let rows_view = Rect::new(rx, text_view.y + text_view.h + 9, rw, rows_view_h);
    TalkBox { r, left, name, band, text, text_view, replies, num_w, rows_view, rows_h }
}

/// Text centred on `cx`, shortened to `max_w` (the audit keeps the full text).
fn text_center_fit(p: &mut Painter, cx: f32, y: f32, text: &str, size: f32, max_w: f32, col: Rgba) {
    let shown = p.fit(text, size, max_w);
    p.text_center(cx, y, &shown, size, col);
    if shown != text {
        p.mark_cut(text);
    }
}

/// Draws the dialogue box `b` (spec PLAYUI1 3.1): the portrait at the top of the left column with the speaker's name
/// (GOLD) and faction band (DIM) under it; at the right the NPC's line, a divider and the numbered replies (locked
/// ones dim with their reason; all off while `busy`, a turn-in picker over it). No key hint line: the numbers show
/// 1-9, Esc is on the Help button's card. PORTRAITFX1-D (6.2 row 1): the portrait lives (`portraits::Faces` on surface
/// `dialogue`: base life, the sidecar's `idle`, the state rows of the speaker's look); the initial letter without a file.
/// PORTRAITFX1-X (7): the node's `expression` (the view's, absent = none) is the speaker's face for the node
/// (`Faces::say`), so its frame draws when it exists, else the neutral frame, else the single portrait.
/// `gs` is the game state the conversation runs in (the game in play, or the Arena's scratch state); `act` maps a
/// row's `TalkAct` to the screen's action.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_talk<A: Clone>(ui: &mut Ui<A>, cx: &mut Cx, gs: &GameState, faces: &mut Faces, view: &Dict, b: &TalkBox, busy: bool, act: fn(TalkAct) -> A) {
    ui.p.panel(b.r, th().game("talk_box", rgba(14, 20, 34, 250)), th().panel_edge, 4.0);
    ui.push("dialogue", "panel", s(view, "speaker_name"), b.r, true, false, None, Vec::new());
    let pr = Rect::new(b.r.x + (b.left - PORTRAIT) / 2, b.r.y + TALK_PAD, PORTRAIT, PORTRAIT);
    ui.p.panel(pr, th().game("portrait_well", rgb(22, 30, 48)), th().edge, 2.0);
    let d = cx.data.clone();
    let speaker = d.resolve(&gs.world_id, "characters", &s(view, "speaker"));
    let (who, look) = talk_portrait(&d, gs, speaker, view);
    faces.say(&who.key, &s(view, "expression"));
    if !faces.draw(&mut ui.p, cx, &who, &look, "dialogue", pr) {
        // the missing art goes to the playtest trail as before (nothing loads)
        let _ = cx.art.first(&crate::screens::character_art(&d, speaker).2, 1, 1);
    }
    let mid = b.r.x as f32 + b.left as f32 / 2.0;
    let max_w = (b.left - 6) as f32;
    let (lb, ls) = (ui.p.line_h(BODY).ceil(), ui.p.line_h(SMALL).ceil());
    let mut y = (pr.y + PORTRAIT + 3) as f32;
    for l in &b.name {
        text_center_fit(&mut ui.p, mid, y, l, BODY, max_w, th().gold);
        y += lb;
    }
    y += 1.0;
    for l in &b.band {
        text_center_fit(&mut ui.p, mid, y, l, SMALL, max_w, th().dim);
        y += ls;
    }
    // the NPC's line (scrolls past TALK_LINES lines)
    let tv = b.text_view;
    let off = ui.scroll_begin(TEXT_VIEW, tv, b.text.len() as i32 * lb as i32);
    for (i, l) in b.text.iter().enumerate() {
        ui.p.text(tv.x as f32, (tv.y - off) as f32 + i as f32 * lb, l, BODY, th().text);
    }
    ui.scroll_end();
    let dy = (tv.y + tv.h + 4) as f32 + 0.5;
    ui.p.line(tv.x as f32, dy, (tv.x + tv.w) as f32, dy, th().edge, 0.0);
    // the replies (scroll past TALK_ROWS rows)
    let rv = b.rows_view;
    let off = ui.scroll_begin(ROWS_VIEW, rv, b.rows_h);
    let mut ry = rv.y - off;
    for rep in &b.replies {
        let rr = Rect::new(rv.x, ry, rv.w - 4, rep.h);
        let on = !rep.locked && !busy;
        let hover = on && ui.hover(rr);
        ui.p.panel(rr, if hover { th().card_hot } else { th().card }, if hover { th().gold } else if rep.locked { th().edge } else { th().edge_live }, 2.0);
        let ty = rr.y as f32 + (rep.h as f32 - rep.lines.len() as f32 * lb) / 2.0;
        if !rep.num.is_empty() {
            ui.p.text(rr.x as f32 + 4.0, ty, &rep.num, BODY, if on { th().gold } else { th().dim });
        }
        for (i, l) in rep.lines.iter().enumerate() {
            ui.p.text_fit((rr.x + 4 + b.num_w) as f32, ty + i as f32 * lb, l, BODY, (rr.w - 8 - b.num_w) as f32, if rep.locked { th().dim } else { th().text });
        }
        ui.push(rep.id.clone(), "choice", rep.label.clone(), rr, on, false, Some(act(rep.act)), Vec::new());
        ry += rep.h + 2;
    }
    ui.scroll_end();
}

/// The talk box's portrait (PORTRAITFX1-D, 5.2 and 6.2): whose it is (keyed by the character id, the initial letter
/// from the shown name) and the look its state rows read: a party member's when a member plays the speaker (its carried
/// HP, its conditions), else the world character's (no HP, so no `hp` or `down` row matches).
fn talk_portrait(d: &emberwake_world::Data, gs: &GameState, speaker: &Dict, view: &Dict) -> (Who, emberwake_battle::portrait::Look) {
    let mut who = Who::of_character(d, speaker);
    who.name = s(view, "speaker_name");
    if who.key.is_empty() {
        who.key = format!("speaker:{}", s(view, "speaker"));
    }
    let qid = s(speaker, "_qid");
    let member = gs.party.members.iter().find(|m| !qid.is_empty() && s(emberwake_world::progression::character_of(d, m), "_qid") == qid);
    let look = match member {
        Some(m) => crate::portraits::look_of_member(d, gs, m),
        None => crate::portraits::look_of_character(d, gs, speaker),
    };
    (who, look)
}

/// The box's keys over `view`: 1-9 pick that reply; Enter or Space continue when the node goes on or ends.
pub(crate) fn talk_key(k: Key, view: &Dict) -> Option<TalkAct> {
    match k {
        Key::Char(c) if c.is_ascii_digit() && c != '0' => Some(TalkAct::Choose(c as i64 - '1' as i64)),
        Key::Enter | Key::Space if json::b(view, "next", false) || json::b(view, "end", false) => Some(TalkAct::Advance),
        _ => None,
    }
}

/// A new node (`node`, "" when none shows) starts its line and its replies scrolled to the top; `seen` keeps the last.
pub(crate) fn follow_node(g: &mut Gfx, seen: &mut String, node: String) {
    if node != *seen {
        g.kit.scroll.remove(TEXT_VIEW);
        g.kit.scroll.remove(ROWS_VIEW);
        *seen = node;
    }
}

/// ENCOUNTER2-C DT11: the agent link's reading of an open talk of tree `dialogue`: the node, the speaker (the
/// character's qualified id), the node's expression ("" none, PORTRAITFX1-X), the line and the reply rows as the box
/// draws them (`id` is the widget to click).
pub(crate) fn talk_state(dialogue: &str, view: &Dict) -> Value {
    let replies: Vec<Value> = reply_rows(view).into_iter().map(|(id, text, locked, _)| json!({"id": id, "text": text, "locked": locked})).collect();
    json!({"dialogue": dialogue, "node": s(view, "node"), "speaker": s(view, "speaker"), "expression": s(view, "expression"), "text": s(view, "text"), "replies": replies})
}

/// The dim over the backdrop while the shop or the board is open (spec PLAYUI1 3.3).
const SHOP_DIM: Rgba = rgba(0, 0, 0, 150);

/// A click or key on the services (`Services::act`): the talk box's replies and Continue, Farewell, the shop's trade
/// (PLAYUI1-P, `shop.rs`) and its close, the board (STORYUI1-U2, `board.rs`) and its close, the turn-in picker.
#[derive(Debug, Clone)]
pub enum SvcAct {
    Choose(i64),
    Advance,
    Farewell,
    /// A click in the shop's trade.
    Shop(ShopAct),
    /// Leaves the shop for the conversation (the trade is empty or discarded).
    CloseShop,
    /// Closes the open board back to the conversation (STORYUI1 B11).
    CloseBoard,
    /// A click on the board pane.
    Board(BoardAct),
    /// A click on the recipe pane (PK2-S3; its Close goes back to the conversation).
    Recipe(RecipeAct),
    TogglePick(String),
    PickRecipient(usize),
    ConfirmTurnIn,
    CancelTurnIn,
}

/// SERVICES1 (SILOS1 T2): what a person opens: the conversation (`DialogueRunner`), its shop's trade, its board, the
/// turn-in picker and the ask before a resting reply (CAMPAIGN1 W3), with the talk box's portrait. The region map holds
/// one and draws the map around it; `ServiceScreen` hosts one on a plain backdrop. Methods that can start a fight say
/// so (true), so a host can stop what it was doing (the region map's walk).
#[derive(Default)]
pub struct Services {
    runner: Option<DialogueRunner>,
    /// The open shop's trade (PLAYUI1-P).
    shop: ShopPane,
    /// The open board's pick and refusals (STORYUI1-U2).
    board: BoardPane,
    /// PK2-S3: the open recipe pane's picks and last run.
    recipe: RecipePane,
    pending: Option<Dict>,
    /// CAMPAIGN1 W3: the choice waiting on the ask before its `rest` would end the run (a Yes picks it).
    rest_ask: Option<i64>,
    picks: Vec<String>,
    recipient: usize,
    /// The conversation node shown last ("character/node"): a new node starts its line and replies scrolled to the top.
    talk_node: String,
    /// PORTRAITFX1-D: the talk box's animated portrait (its clock moves with the frame's wall time).
    pub(crate) faces: Faces,
}

impl Services {
    /// A conversation is open.
    pub fn talking(&self) -> bool {
        self.runner.as_ref().is_some_and(|r| !r.closed)
    }

    /// The open shop of the conversation ("" when none).
    pub fn shop_qid(&self) -> String {
        self.runner.as_ref().filter(|r| !r.closed).map(|r| r.shop_qid.clone()).unwrap_or_default()
    }

    /// The open board of the conversation ("" when none; STORYUI1 B3).
    pub fn board_qid(&self) -> String {
        self.runner.as_ref().filter(|r| !r.closed).map(|r| r.board_qid.clone()).unwrap_or_default()
    }

    /// PK2-S3: the character whose recipe pane is open ("" when none).
    pub fn recipe_npc(&self) -> String {
        self.runner.as_ref().filter(|r| !r.closed).map(|r| r.recipe_npc.clone()).unwrap_or_default()
    }

    /// The character talked to last ("" when none).
    pub(crate) fn talked(&self) -> String {
        self.runner.as_ref().map(|r| r.character_qid.clone()).unwrap_or_default()
    }

    /// Opens the conversation with `character`.
    pub fn talk(&mut self, cx: &mut Cx, character: &str) {
        let d = cx.data.clone();
        let r = DialogueRunner::open(&d, cx.gs_mut(), character);
        if r.closed {
            for e in &r.errors {
                cx.log(e.clone());
            }
            return;
        }
        cx.log(format!("You speak with {}.", r.character_name));
        cx.set_busy(true);
        self.runner = Some(r);
    }

    /// CAMPAIGN1 T2: a queued `{dialogue}` scene opens as the conversation (`runover::pending_talk`). True when it did.
    pub fn open_pending(&mut self, cx: &mut Cx) -> bool {
        if cx.busy() || !cx.nav.is_empty() {
            return false;
        }
        let Some(r) = crate::runover::pending_talk(cx) else { return false };
        cx.set_busy(true);
        self.runner = Some(r);
        true
    }

    /// Closes everything open.
    pub fn end(&mut self, cx: &mut Cx) {
        self.runner = None;
        self.pending = None;
        self.rest_ask = None;
        self.shop = ShopPane::default();
        self.board = BoardPane::default();
        self.recipe = RecipePane::default();
        cx.set_busy(false);
    }

    /// A frame's tick: the portrait clock moves (PORTRAITFX1 4.5: by the frame's wall time, at most 100 ms a frame);
    /// a Yes to the ask before a resting reply picks it (CAMPAIGN1 W3). None when no such answer came, else whether
    /// a fight started.
    pub fn tick(&mut self, cx: &mut Cx) -> Option<bool> {
        self.faces.tick();
        if !cx.take_answer(crate::runover::ASK_TALK_REST) {
            return None;
        }
        Some(self.rest_ask.take().is_some_and(|i| self.choose(cx, i)))
    }

    /// Yes to "Discard this trade?" (PLAYUI1-P): back to the conversation. Whether a fight started.
    pub fn take_discard(&mut self, cx: &mut Cx) -> bool {
        cx.take_answer(crate::shop::DISCARD) && self.act(cx, SvcAct::CloseShop)
    }

    /// Before a frame's `Ui`: the open conversation's view (a new node starts its line and its replies scrolled to the
    /// top); the portraits left out of the last frame are dropped.
    pub(crate) fn begin_frame(&mut self, g: &mut Gfx, d: &Data, gs: &GameState) -> Option<Dict> {
        let view = self.runner.as_ref().filter(|r| !r.closed).map(|r| r.view(d, gs));
        let node = self.runner.as_ref().filter(|r| !r.closed).map(|r| format!("{}/{}", r.character_qid, r.node_id)).unwrap_or_default();
        follow_node(g, &mut self.talk_node, node);
        self.faces.begin();
        view
    }

    /// Draws the open conversation's `view` (measured as `b`): the talk box; while the shop is open (3.3) the box hides
    /// and the shop sits centred over `veil` dimmed; an open board (STORYUI1 B11) takes the shop's place; the turn-in
    /// picker over all.
    pub(crate) fn draw<A: Clone + From<SvcAct>>(&mut self, ui: &mut Ui<A>, cx: &mut Cx, view: &Dict, b: &TalkBox, veil: Rect) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let (shop_qid, board_qid, recipe_npc) = (self.shop_qid(), self.board_qid(), self.recipe_npc());
        if !shop_qid.is_empty() {
            ui.p.fill(veil, th().game("shop_veil", SHOP_DIM));
            self.shop.draw(ui, cx, &shop_qid, |a| A::from(SvcAct::Shop(a)));
        } else if !recipe_npc.is_empty() {
            ui.p.fill(veil, th().game("shop_veil", SHOP_DIM));
            self.recipe.draw(ui, cx, &recipe_npc, |a| A::from(SvcAct::Recipe(a)));
        } else if !board_qid.is_empty() {
            ui.p.fill(veil, th().game("shop_veil", SHOP_DIM));
            let bv = emberwake_world::board::view(&d, &gs, &board_qid);
            let pick = self.board.select(&bv);
            let det = if pick.is_empty() { Dict::new() } else { emberwake_world::board::detail(&d, &gs, &pick) };
            self.board.draw(ui, cx, &bv, &det, |a| A::from(SvcAct::Board(a)));
        } else {
            draw_talk(ui, cx, &gs, &mut self.faces, view, b, self.pending.is_some(), svc_talk_act::<A>);
        }
        if let Some(p) = self.pending.clone() {
            self.draw_turn_in(ui, cx, &p);
        }
    }

    fn draw_turn_in<A: Clone + From<SvcAct>>(&mut self, ui: &mut Ui<A>, cx: &Cx, p: &Dict) {
        let d = &cx.data;
        let gs = cx.gs();
        let qid = s(p, "quest");
        let q = quest::resolved(d, gs, &qid);
        let fr = Rect::new(160, 60, 320, 230);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("map_veil", rgba(0, 0, 0, 100)));
        ui.frame(fr, &format!("Turn in: {}", s_or(&q, "name", &qid)), false);
        let mut y = fr.y + 30;
        let rewards = crate::hub::reward_lines(cx, json::dget(&q, "rewards"));
        if !rewards.is_empty() {
            ui.p.text_fit(fr.x as f32 + 10.0, y as f32, &format!("Rewards: {}", rewards.join(", ")), BODY, (fr.w - 20) as f32, th().text);
            y += 14;
        }
        let allowed = json::i(p, "picks_allowed", 0);
        let offers = json::sget(p, "offers");
        if allowed > 0 {
            ui.p.text(fr.x as f32 + 10.0, y as f32, &format!("Choose {allowed} of:"), BODY, th().gold);
            y += 13;
            for lq in offers {
                let loot = d.resolve(&gs.world_id, "loot", &lq);
                let on = self.picks.contains(&lq);
                ui.checkbox(&format!("turn_in.pick.{lq}"), fr.x + 14, y, &s_or(loot, "name", &lq), on, A::from(SvcAct::TogglePick(lq.clone())), true);
                y += 14;
            }
        }
        let names: Vec<String> = gs.party.members.iter().map(|m| crate::hub::member_name(cx, m)).collect();
        ui.p.text(fr.x as f32 + 10.0, y as f32 + 5.0, "Passive / skill rewards go to", BODY, th().dim);
        ui.select("turn_in.recipient", Rect::new(fr.x + 150, y + 2, 120, 15), &names, self.recipient, |i| A::from(SvcAct::PickRecipient(i)), gs.party.members.len() > 1, Vec::new());
        ui.button("turn_in.cancel", Rect::new(fr.x + fr.w - 120, fr.y + fr.h - 24, 52, 16), "Cancel", A::from(SvcAct::CancelTurnIn), true);
        ui.button("turn_in.confirm", Rect::new(fr.x + fr.w - 62, fr.y + fr.h - 24, 52, 16), "Confirm", A::from(SvcAct::ConfirmTurnIn), self.picks.len() as i64 <= allowed);
    }

    /// A key while a conversation is open: Enter confirms the turn-in picker; the shop and the board take none; else
    /// the talk box's keys. None when no conversation is open (the host goes on), else whether a fight started.
    pub fn key(&mut self, cx: &mut Cx, k: Key) -> Option<bool> {
        let r = self.runner.as_ref().filter(|r| !r.closed)?;
        if self.pending.is_some() {
            return Some(k == Key::Enter && self.act(cx, SvcAct::ConfirmTurnIn));
        }
        if !r.shop_qid.is_empty() || !r.board_qid.is_empty() || !r.recipe_npc.is_empty() {
            return Some(false);
        }
        let view = r.view(&cx.data, cx.gs());
        Some(talk_key(k, &view).is_some_and(|a| self.act(cx, svc_talk_act::<SvcAct>(a))))
    }

    /// Back (right click, Esc): cancels the turn-in picker, closes the shop (asking first with a trade), closes the
    /// board, or says farewell. None when nothing is open, else whether a fight started.
    pub fn back(&mut self, cx: &mut Cx) -> Option<bool> {
        let a = if self.pending.is_some() {
            SvcAct::CancelTurnIn
        } else if !self.shop_qid().is_empty() {
            SvcAct::Shop(ShopAct::Close)
        } else if !self.board_qid().is_empty() {
            SvcAct::CloseBoard
        } else if !self.recipe_npc().is_empty() {
            SvcAct::Recipe(RecipeAct::Close)
        } else if self.talking() {
            SvcAct::Farewell
        } else {
            return None;
        };
        Some(self.act(cx, a))
    }

    /// Picks reply `i` of the conversation. Whether a fight started.
    fn choose(&mut self, cx: &mut Cx, i: i64) -> bool {
        let d = cx.data.clone();
        let Some(r) = self.runner.as_mut() else { return false };
        let res = r.choose(&d, cx.gs_mut(), i);
        self.dialogue_result(cx, res)
    }

    /// CAMPAIGN1 W3, R3: the question before reply `i` when it rests (the `rest` effect) and the rest's days would end
    /// the run; None otherwise (also when the rest is refused: the choice says why).
    fn rest_question(&self, cx: &Cx, i: i64) -> Option<String> {
        let r = self.runner.as_ref()?;
        let d = &cx.data;
        if !r.choice_effects(d, cx.gs(), i).iter().any(|fx| s(fx, "effect") == "rest") || !emberwake_world::campaign::purse::rest_refusal(d, cx.gs()).is_empty() {
            return None;
        }
        let days = emberwake_world::campaign::purse::rest_rule(d, &cx.gs().world_id).days as i64;
        crate::runover::days_ask(cx, days, "Resting", "Rest")
    }

    /// Handles a dialogue result: fights, the turn-in picker, notes, the end. Whether a fight started.
    fn dialogue_result(&mut self, cx: &mut Cx, res: Dict) -> bool {
        if !json::b(&res, "ok", false) {
            for e in json::sget(&res, "errors") {
                cx.log(e);
            }
        }
        crate::runover::take(cx, &res);
        // CAMPAIGN1 F3: a choice that ended the run ends the talk (the run-over screen follows)
        if crate::runover::is_over(cx) {
            self.end(cx);
            return false;
        }
        for n in json::sget(&res, "notes") {
            cx.log(n);
        }
        if let Some(g) = res.get("granted").and_then(Value::as_object) {
            let mut got = Vec::new();
            if json::i(g, "xp", 0) > 0 {
                got.push(format!("{} XP", json::i(g, "xp", 0)));
            }
            if json::i(g, "gold", 0) > 0 {
                got.push(format!("{} {}", json::i(g, "gold", 0), purse_word(cx, PurseWord::Currency)));
            }
            for (id, n) in json::ints(json::get(g, "items")) {
                got.push(format!("{} x{n}", cx.data.item_name(&id)));
            }
            if !got.is_empty() {
                cx.log(format!("Quest complete: {}.", got.join(", ")));
            }
        }
        if res.contains_key("pending") {
            self.picks.clear();
            self.recipient = 0;
            self.pending = Some(res);
            return false;
        }
        if let Some(f) = res.get("fight").and_then(Value::as_object).cloned() {
            self.end(cx);
            cx.set_busy(false);
            // CAMPAIGN1 W3: a fight that could end the run asks first (No: the talk is over, nothing fought)
            return match crate::runover::fight_ask(cx, &s(&f, "encounter")) {
                Some(q) => {
                    cx.ask_no(q, move |cx| crate::region::fight_now(cx, &f, "dialogue"));
                    false
                }
                None => {
                    crate::region::fight_now(cx, &f, "dialogue");
                    true
                }
            };
        }
        if self.runner.as_ref().is_some_and(|r| r.closed) {
            self.end(cx);
        }
        false
    }

    /// One click or key on the services. Whether a fight started.
    pub fn act(&mut self, cx: &mut Cx, a: SvcAct) -> bool {
        let d = cx.data.clone();
        match a {
            SvcAct::Choose(i) => {
                // CAMPAIGN1 W3, R3: a choice whose rest would end the run asks first (No: the talk goes on)
                if let Some(q) = self.rest_question(cx, i) {
                    self.rest_ask = Some(i);
                    cx.ask_tag_no(q, crate::runover::ASK_TALK_REST);
                    return false;
                }
                self.choose(cx, i)
            }
            SvcAct::Advance => {
                let Some(r) = self.runner.as_mut() else { return false };
                let res = r.advance(&d, cx.gs_mut());
                self.dialogue_result(cx, res)
            }
            SvcAct::Farewell => {
                let Some(r) = self.runner.as_ref() else { return false };
                let view = r.view(&d, cx.gs());
                let farewell = json::aget(&view, "choices").iter().position(|c| s(json::dict(c), "service") == "farewell");
                match farewell {
                    Some(i) => self.act(cx, SvcAct::Choose(i as i64)),
                    None => {
                        self.end(cx);
                        false
                    }
                }
            }
            SvcAct::Shop(a) => {
                let sq = self.shop_qid();
                !sq.is_empty() && self.shop.act(cx, &sq, a) && self.act(cx, SvcAct::CloseShop)
            }
            SvcAct::CloseShop => {
                self.shop = ShopPane::default();
                let Some(r) = self.runner.as_mut() else { return false };
                let res = r.close_shop(&d, cx.gs_mut());
                self.dialogue_result(cx, res)
            }
            SvcAct::CloseBoard => {
                self.board = BoardPane::default();
                let Some(r) = self.runner.as_mut() else { return false };
                let res = r.close_board(&d, cx.gs_mut());
                self.dialogue_result(cx, res)
            }
            // PK2-S3: a pick or a run stays on the pane; Close goes back to the conversation
            SvcAct::Recipe(a) => {
                let npc = self.recipe_npc();
                if npc.is_empty() || !self.recipe.act(cx, &npc, a) {
                    return false;
                }
                self.recipe = RecipePane::default();
                let Some(r) = self.runner.as_mut() else { return false };
                let res = r.close_recipes(&d, cx.gs_mut());
                self.dialogue_result(cx, res)
            }
            SvcAct::Board(a) => match a {
                BoardAct::Pick(q) => {
                    self.board.pick(q);
                    false
                }
                BoardAct::Close => self.act(cx, SvcAct::CloseBoard),
                // B8: the runner's `start_quest` path; a refusal stays on the pane, a taken quest is logged
                BoardAct::Accept(q) => {
                    let Some(r) = self.runner.as_mut() else { return false };
                    let res = r.accept(&d, cx.gs_mut(), &q);
                    if json::b(&res, "ok", false) {
                        self.board.errors.clear();
                        self.dialogue_result(cx, res)
                    } else {
                        self.board.errors = json::sget(&res, "errors");
                        false
                    }
                }
            },
            SvcAct::TogglePick(id) => {
                let allowed = self.pending.as_ref().map(|p| json::i(p, "picks_allowed", 0)).unwrap_or(0) as usize;
                if let Some(i) = self.picks.iter().position(|p| *p == id) {
                    self.picks.remove(i);
                } else if self.picks.len() < allowed {
                    self.picks.push(id);
                }
                false
            }
            SvcAct::PickRecipient(i) => {
                self.recipient = i;
                false
            }
            SvcAct::ConfirmTurnIn => {
                let rec = cx.gs().party.members.get(self.recipient).map(|m| m.id.clone()).unwrap_or_default();
                let picks = self.picks.clone();
                let Some(r) = self.runner.as_mut() else { return false };
                let res = r.confirm_turn_in(&d, cx.gs_mut(), &rec, &picks);
                if json::b(&res, "ok", false) {
                    self.pending = None;
                }
                self.dialogue_result(cx, res)
            }
            SvcAct::CancelTurnIn => {
                if let Some(r) = self.runner.as_mut() {
                    r.cancel();
                }
                self.pending = None;
                false
            }
        }
    }

    /// The agent link's reading, in the region map's key order: `dialogue` (the open conversation's view), `turn_in`,
    /// `shop` (the trade), `board` (its qid), `board_pane` and `portraits` (PORTRAITFX1 4.11: the speaker's portrait
    /// while the talk box shows; the shop or the board hides it).
    pub fn state(&self, cx: &Cx) -> Value {
        let gs = cx.gs();
        let (shop_qid, board_qid) = (self.shop_qid(), self.board_qid());
        json!({"dialogue": self.runner.as_ref().filter(|r| !r.closed).map(|r| Value::Object(r.view(&cx.data, gs))),
            "turn_in": self.pending.as_ref().map(|p| json!({"quest": p["quest"], "offers": p["offers"], "picks": self.picks})),
            "shop": if shop_qid.is_empty() { Value::Null } else { self.shop.state() }, "board": Some(board_qid.clone()).filter(|b| !b.is_empty()),
            "board_pane": if board_qid.is_empty() { Value::Null } else { self.board.state() },
            "recipe": Some(self.recipe_npc()).filter(|n| !n.is_empty()),
            "recipe_pane": if self.recipe_npc().is_empty() { Value::Null } else { self.recipe.state() },
            "portraits": if self.talking() && shop_qid.is_empty() && board_qid.is_empty() && self.recipe_npc().is_empty() { self.faces.state() } else { json!([]) }})
    }
}

/// The services' action of a talk box row or key.
pub(crate) fn svc_talk_act<A: From<SvcAct>>(a: TalkAct) -> A {
    A::from(match a {
        TalkAct::Choose(i) => SvcAct::Choose(i),
        TalkAct::Advance => SvcAct::Advance,
    })
}

/// SERVICES1 (SILOS1 T2): `Services` on a plain backdrop, with no map: the UI Gallery's dialogue and shop entries
/// (and the shop's tests). The box and the panes draw as over the region map, the backdrop in the map's ground colour.
#[derive(Default)]
pub struct ServiceScreen {
    pub(crate) svc: Services,
    w: Vec<Widget<SvcAct>>,
}

impl ServiceScreen {
    /// Opens the conversation with `character`.
    pub fn talk(&mut self, cx: &mut Cx, character: &str) {
        self.svc.talk(cx, character);
    }

    fn act(&mut self, cx: &mut Cx, a: SvcAct) {
        self.svc.act(cx, a);
    }
}

impl Screen<GameCx> for ServiceScreen {
    fn id(&self) -> &'static str {
        "services"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.svc.take_discard(cx);
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let view = self.svc.begin_frame(g, &d, &gs);
        let mut ui: Ui<SvcAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("map_ground", rgb(14, 18, 26)));
        match view {
            Some(v) => {
                let b = talk_layout(&mut ui.p, &v);
                self.svc.draw(&mut ui, cx, &v, &b, Rect::new(0, HUD_H, W, H - HUD_H));
            }
            None => {
                ui.p.text(20.0, (HUD_H + 10) as f32, "No conversation is open.", BODY, th().dim);
            }
        }
        self.w = ui.widgets;
    }

    fn tick(&mut self, cx: &mut Cx) {
        self.svc.tick(cx);
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        self.svc.key(cx, k);
    }

    /// Back as over the map; with nothing open it leaves.
    fn back(&mut self, cx: &mut Cx) {
        if self.svc.back(cx).is_none() {
            cx.nav.push(Nav::Pop);
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        self.svc.state(cx)
    }

    widget_plumbing!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use crate::portraits::tests::{drawn_fill, frame_layers, BASE_FILL, FRAME_FILL, NEUTRAL_FILL};
    use emberwake_content::art as kart;

    /// X2 (PORTRAITFX1 5.4, 7): a node with `angry` draws the speaker's `faces/<id>/angry.png` when it exists, else its
    /// `neutral` frame, else its single portrait; a node without the key draws the neutral frame (the frames' plain
    /// face). The talk box is the region map's and the battle's (one `draw_talk`); the agent link reads the expression.
    #[test]
    fn x2_a_nodes_expression_picks_the_speakers_frame() {
        let mut cx = game_cx();
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = Vec::new());
        cx.settings.v["reduce_motion"] = json!(true);
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        // a speaker with a unique portrait tier (it has frames), picked by rule
        let character = d.merged_entries(&gs.world_id, "characters").into_iter().find(|c| kart::tier_folder(&Who::of_character(&d, c).tier).is_some()).expect("a character with a unique portrait").clone();
        let who = Who::of_character(&d, &character);
        let base = kart::portrait_chain(&who.tier, &who.id, &who.body)[0].clone();
        let view = |expression: Option<&str>| -> Dict {
            let mut v = json::dict(&json!({"speaker": s(&character, "_qid"), "speaker_name": s(&character, "name"), "node": "n", "text": "A line.", "choices": [], "end": true, "next": false, "faction": "", "faction_name": "", "band": ""})).clone();
            if let Some(e) = expression {
                v.insert("expression".into(), e.into());
            }
            v
        };
        let mut g = gfx();
        let mut drawn = |cx: &mut Cx, view: &Dict| {
            let mut faces = Faces::default();
            let mut ui: Ui<TalkAct> = Ui::new(&mut g, (-500, -500), true);
            faces.begin();
            let b = talk_layout(&mut ui.p, view);
            draw_talk(&mut ui, cx, &gs, &mut faces, view, &b, false, |a| a);
            drop(ui);
            drawn_fill(&faces, &who.key)
        };
        cx.art = frame_layers("talk_x2_all", &base, true, Some("angry"));
        assert_eq!(drawn(&mut cx, &view(Some("angry"))), Some(FRAME_FILL), "the angry frame");
        assert_eq!(drawn(&mut cx, &view(None)), Some(NEUTRAL_FILL), "no expression: the neutral frame");
        cx.art = frame_layers("talk_x2_neutral", &base, true, None);
        assert_eq!(drawn(&mut cx, &view(Some("angry"))), Some(NEUTRAL_FILL), "no angry frame: the neutral frame");
        cx.art = frame_layers("talk_x2_none", &base, false, None);
        assert_eq!(drawn(&mut cx, &view(Some("angry"))), Some(BASE_FILL), "no frames: the single portrait");
        assert_eq!(talk_state("w:t", &view(Some("angry")))["expression"], json!("angry"));
        assert_eq!(talk_state("w:t", &view(None))["expression"], json!(""));
    }

    /// The widgets of `screen` whose id starts with one of `prefixes`: (id, rect, enabled).
    fn panes(screen: &dyn Screen<GameCx>, prefixes: &[&str]) -> Vec<(String, (i64, i64, i64, i64), Value)> {
        screen.widgets().into_iter().filter(|w| w["id"].as_str().is_some_and(|i| prefixes.iter().any(|p| i.starts_with(p)))).map(|w| (s(json::dict(&w), "id"), rect(&w), w["enabled"].clone())).collect()
    }

    /// SERVICES1 (SILOS1 T2): the plain-backdrop host opens a conversation and its shop with no region map, and they
    /// read and act as over the map: the same conversation state, the same shop state and controls in the same
    /// places; back closes the empty trade (the box comes back), says farewell, then leaves the host. The keeper is
    /// the first open shop's (by rule), the state put back between the two hosts.
    #[test]
    fn the_service_screen_opens_a_conversation_and_its_shop_as_the_region_map_does() {
        let (mut cx, mut g) = (game_cx(), gfx());
        // the region map enters the start location first; both hosts start from that state
        let mut map = crate::region::RegionMap::new(&mut cx);
        cx.nav.clear();
        let start = cx.gs().clone();
        let d = cx.data.clone();
        let keepers: Vec<String> = d.merged_entries(&start.world_id, "shops").into_iter().filter(|sh| emberwake_world::shop::is_open(&d, &start, &s(sh, "_qid"))).map(|sh| s(sh, "keeper")).collect();
        let shop_reply = |screen: &dyn Screen<GameCx>, cx: &Cx| json::aget(json::dict(&screen.state(cx)["dialogue"]), "choices").iter().position(|c| s(json::dict(c), "service") == "shop");
        let mut checked = false;
        for keeper in keepers {
            *cx.gs_mut() = start.clone();
            map.talk(&mut cx, &keeper);
            frame(&mut map, &mut cx, &mut g, 1.0);
            let Some(k) = shop_reply(&map, &cx) else {
                map.svc.end(&mut cx);
                continue;
            };
            let on_map = map.state(&cx);
            let box_on_map = panes(&map, &["dialogue", "choice."]);
            map.click_id(&mut cx, &format!("choice.{k}")).expect("the shop reply on the map");
            frame(&mut map, &mut cx, &mut g, 1.0);
            let shop_on_map = (map.state(&cx)["shop"].clone(), panes(&map, &["shop", "stash.", "trade."]), cx.gs().clone());
            map.svc.end(&mut cx);
            // the same through the plain backdrop
            *cx.gs_mut() = start.clone();
            let mut host = ServiceScreen::default();
            host.talk(&mut cx, &keeper);
            assert!(cx.busy(), "a conversation holds the game busy");
            frame(&mut host, &mut cx, &mut g, 1.0);
            let st = host.state(&cx);
            assert!(st["dialogue"].is_object() && st["shop"].is_null(), "{keeper}: the conversation opens: {st}");
            assert_eq!(st["dialogue"], on_map["dialogue"], "{keeper}: the same conversation");
            assert_eq!(st["portraits"], on_map["portraits"], "{keeper}: the same portrait reading");
            assert_eq!(panes(&host, &["dialogue", "choice."]), box_on_map, "{keeper}: the box and its replies in the same places");
            assert!(!panes(&host, &["choice."]).is_empty(), "{keeper}: the replies show");
            host.click_id(&mut cx, &format!("choice.{k}")).expect("the shop reply");
            frame(&mut host, &mut cx, &mut g, 1.0);
            assert_eq!((host.state(&cx)["shop"].clone(), panes(&host, &["shop", "stash.", "trade."]), cx.gs().clone()), shop_on_map, "{keeper}: the same trade in the same places");
            assert!(panes(&host, &["dialogue"]).is_empty(), "{keeper}: the shop hides the box");
            // back: the empty trade closes at once and the box comes back; again: farewell; again: the host leaves
            host.back(&mut cx);
            assert!(cx.nav.is_empty(), "an empty trade closes without asking");
            frame(&mut host, &mut cx, &mut g, 1.0);
            assert!(host.state(&cx)["shop"].is_null() && !panes(&host, &["dialogue"]).is_empty(), "{keeper}: back in the conversation");
            host.back(&mut cx);
            frame(&mut host, &mut cx, &mut g, 1.0);
            assert!(!cx.busy() && host.state(&cx)["dialogue"].is_null(), "{keeper}: the conversation is over");
            host.back(&mut cx);
            assert!(matches!(cx.nav.as_slice(), [Nav::Pop]), "nothing open: back leaves the host");
            cx.nav.clear();
            checked = true;
            break;
        }
        *cx.gs_mut() = start;
        assert!(checked, "no open shop's keeper offers a shop reply in the first playable world: nothing to check");
    }

    /// SERVICES1: the keys over the plain backdrop are the region map's: a number key picks that reply (the last, a
    /// farewell, ends the conversation as on the map), and nothing open takes no key.
    #[test]
    fn a_number_key_picks_a_reply_on_the_plain_backdrop() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let d = cx.data.clone();
        let _ = emberwake_world::explore::resume(&d, cx.gs_mut());
        let start = cx.gs().clone();
        let mut host = ServiceScreen::default();
        host.key(&mut cx, Key::Char('1'));
        assert!(*cx.gs() == start && host.state(&cx)["dialogue"].is_null(), "nothing open: no key does anything");
        let people: Vec<String> = d.merged_entries(&start.world_id, "characters").iter().map(|c| s(c, "id")).collect();
        let mut checked = false;
        for who in people {
            *cx.gs_mut() = start.clone();
            host.talk(&mut cx, &who);
            if !host.svc.talking() {
                continue;
            }
            frame(&mut host, &mut cx, &mut g, 1.0);
            let choices = json::aget(json::dict(&host.state(&cx)["dialogue"]), "choices").to_vec();
            let Some(i) = choices.iter().position(|c| s(json::dict(c), "service") == "farewell").filter(|i| *i < 9) else {
                host.svc.end(&mut cx);
                continue;
            };
            host.key(&mut cx, Key::Char((b'1' + i as u8) as char));
            assert!(!host.svc.talking() && !cx.busy(), "{who}: key {} (farewell) ends the conversation", i + 1);
            checked = true;
            break;
        }
        *cx.gs_mut() = start;
        assert!(checked, "nobody in the first playable world has a farewell reply: nothing to check");
    }

    /// PR5-38 ACT1-STORY (shipped data): a New game in `sunk_ward` played through its Prologue (story fights settled
    /// `won` with no battle) stands in the hub; the hub's region map draws; Talia's board reply opens the BoardPane, the
    /// Slab's and the Hearth's shop replies the ShopPane; no recipe reply before M5, and with the effects of the
    /// post-M5 sequence applied Sabina's recipe reply opens the RecipePane; the WorldMap shows the hub and the way out,
    /// not Tier 2.
    #[test]
    fn the_sunk_ward_hub_opens_its_panes_and_the_world_map() {
        use emberwake_world::{battle, new_game, story};
        const W: &str = "sunk_ward";
        let (mut cx, mut g) = (game_cx(), gfx());
        let d = cx.data.clone();
        let gs = new_game::create_for_world(&d, W, 1).expect("a new sunk_ward game");
        crate::screens::enter_game(&mut cx, gs, -1, Vec::new());
        cx.nav.clear();
        {
            let gs = cx.gs_mut();
            for _ in 0..200 {
                if !story::is_running(gs) {
                    break;
                }
                if let Some(fight) = story::fight(&d, gs) {
                    let r = battle::finish_world(&d, gs, story::FIGHT_SOURCE, &fight, "won", None);
                    assert!(json::b(&r, "ok", false), "{r:?}");
                } else if !json::b(&story::step_done(&d, gs), "ok", false) {
                    let r = story::finish(&d, gs);
                    assert!(json::b(&r, "ok", false), "{r:?}");
                }
            }
        }
        assert!(!story::is_running(cx.gs()), "the Prologue ends");
        assert_eq!(cx.gs().location_id, format!("{W}:sluice_undercroft"));
        let hub = cx.data.resolve(W, "locations", "sluice_undercroft").clone();
        let mut map = crate::region::RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let npc_markers: Vec<String> = emberwake_world::explore::markers_of(&hub).iter().filter(|m| s(m, "kind") == "npc").map(|m| format!("marker.{}", s(m, "id"))).collect();
        assert!(map.widgets().iter().any(|w| npc_markers.iter().any(|id| w["id"] == json!(id))), "the hub map draws its people: {npc_markers:?}");
        // a service reply of `who`, clicked: the host with its pane open; None when `who` offers no such reply
        let open = |cx: &mut Cx, g: &mut Gfx, who: &str, service: &str| -> Option<ServiceScreen> {
            let mut host = ServiceScreen::default();
            host.talk(cx, who);
            frame(&mut host, cx, g, 1.0);
            let choices = json::aget(json::dict(&host.state(cx)["dialogue"]), "choices").to_vec();
            let k = choices.iter().position(|c| s(json::dict(c), "service") == service)?;
            host.click_id(cx, &format!("choice.{k}")).expect("the service reply");
            frame(&mut host, cx, g, 1.0);
            Some(host)
        };
        let has = |host: &ServiceScreen, id: &str| host.widgets().iter().any(|w| w["id"] == id);
        let host = open(&mut cx, &mut g, "talia_rusk", "board").expect("Talia's board reply");
        assert_eq!(host.state(&cx)["board"], json!(format!("{W}:talia_board")));
        assert!(has(&host, "board"), "the BoardPane draws");
        for (who, shop) in [("sabina_kress", "sabina_slab"), ("berek_dorn", "berek_hearth")] {
            let host = open(&mut cx, &mut g, who, "shop").unwrap_or_else(|| panic!("{who}'s shop reply"));
            assert!(host.state(&cx)["shop"].is_object() && has(&host, "shop"), "{shop}: the ShopPane opens");
            assert!(open(&mut cx, &mut g, who, "recipe").is_none(), "{who}: no recipe reply before M5");
        }
        // the post-M5 sequence's own effects (the PK2 flags), applied as its effects step applies them
        let after_m5 = cx.data.resolve(W, "sequences", "seq_after_m5").clone();
        let effects: Vec<Dict> = json::aget(&after_m5, "steps")
            .iter()
            .map(json::dict)
            .filter(|st| s(st, "kind") == "effects")
            .flat_map(|st| json::aget(st, "effects").iter().map(|fx| json::dict(fx).clone()).collect::<Vec<_>>())
            .collect();
        assert!(!effects.is_empty(), "seq_after_m5 has an effects step");
        let (mut notes, mut events) = (Vec::new(), Vec::new());
        emberwake_world::dialogue::apply_effects(&d, cx.gs_mut(), &effects, &mut notes, &mut events);
        for who in ["sabina_kress", "berek_dorn"] {
            let host = open(&mut cx, &mut g, who, "recipe").unwrap_or_else(|| panic!("{who}'s recipe reply after M5"));
            assert_eq!(host.state(&cx)["recipe"], json!(format!("{W}:{who}")));
            assert!(has(&host, "recipe"), "{who}: the RecipePane draws");
        }
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let node = |id: &str| format!("node.{W}.{id}");
        assert!(wm.widgets().iter().any(|w| w["id"] == json!(node("sluice_undercroft"))), "the WorldMap shows the hub");
        assert!(wm.widgets().iter().any(|w| w["id"] == json!(node("drowned_runoff_grate"))), "and the way out");
        assert!(!wm.widgets().iter().any(|w| w["id"] == json!(node("fourth_kiln"))), "not Tier 2");
    }
}
