//! The talk box (scripts/ui/overworld/dialogue_panel.gd's layout as PLAYUI1-D's 400 px box): one `DialogueRunner::view`
//! measured (`talk_layout`), drawn with its living portrait (`draw_talk`, PORTRAITFX1-D: `portraits::Faces`, the
//! speaker's look from `look_of_member` or `look_of_character`; PORTRAITFX1-X: the node's `expression` picks the
//! frame along `face_chain`), its keys (`talk_key`) and the agent link's reading of
//! it (`talk_state`). Shared by the region map's conversations and the battle's phase talks (ENCOUNTER2-C DT7: moved
//! out of `region.rs`, the look unchanged); each screen maps `TalkAct` to its own action and owns its runner.

use crate::portraits::{Faces, Who};
use crate::screens::{Cx, Key};
use aegis_core::json::{self, s, Dict};
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Gfx, Ui, BODY, H, SMALL, W};
use emberwake_world::GameState;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::{game_cx, gfx};
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
}
