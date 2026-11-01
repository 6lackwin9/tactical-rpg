//! The mission board pane (STORYUI1-U2, docs/specs/STORYUI1.md 5.5 B5-B8, B11; engine-only, no Godot original;
//! story-free, marked for KIT2-UI): the keeper's board over the dimmed region map in the shop's frame, its intro on
//! top, the sections on the left (Main, Jobs, Under way, Locked, Closed; each row the quest's name, its node, the
//! deadline in Arterial Red and the section's line), the picked quest's detail on the right (description, steps,
//! rewards with the purse's name, loot picks, reputation, node, deadline), Accept and Close. The rules are the kit's
//! (`board::view`, `board::detail`, the runner's `accept` and `close_board`); the pane only draws them. `RegionMap`
//! draws it while its conversation has a board open; the UI Gallery shows it on a fixture board (`BoardSample`).

use crate::hud::{purse_word, PurseWord};
use crate::screens::{Cx, GameCx, Nav, Screen};
use aegis_core::json::{self, s, s_or, Dict};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::ui::{tip, Gfx, Tip, Ui, Widget, BODY, BORDER, DIM, FOE, FRAME, GOLD, GOOD, H, INFO, MODAL, PANEL_HI, RADIUS, SELECTED, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use emberwake_world::board;
use serde_json::{json, Value};

/// B6, B7, B10: deadline lines are Arterial Red (the art bible's `#e11d48`).
pub const DEADLINE: Rgba = rgb(0xe1, 0x1d, 0x48);
/// B11: the section list's width at text size Normal (it grows with the text).
const LIST_W: i32 = 240;
/// The frame's inner margin and the gap between the list and the detail (the shop's).
const PAD: i32 = 7;
const GAP: i32 = 6;
/// The columns' fill (the shop's).
const COL_BG: Rgba = rgb(14, 20, 34);
/// The board's name in the header.
const HEAD: f32 = 10.0;
/// The dim over the map (or the gallery's backdrop) under the pane.
pub const DIM_FILL: Rgba = rgba(0, 0, 0, 150);

/// A click on the board pane.
#[derive(Debug, Clone)]
pub enum BoardAct {
    /// Shows a quest's detail.
    Pick(String),
    /// Takes an available quest (B8).
    Accept(String),
    /// Back to the conversation (B11).
    Close,
}

/// The pane's own state: the quest whose detail shows and the last refusal of Accept.
#[derive(Debug, Default)]
pub struct BoardPane {
    pub selected: String,
    pub errors: Vec<String>,
}

/// Every row of a board view, in section order (each with its section id under `_section`).
pub fn rows(view: &Dict) -> Vec<Dict> {
    let mut out = Vec::new();
    for sec in json::aget(view, "sections") {
        let sec = json::dict(sec);
        for r in json::aget(sec, "rows") {
            let mut r = json::dict(r).clone();
            r.insert("_section".into(), s(sec, "id").into());
            out.push(r);
        }
    }
    out
}

/// Why Accept is off for a quest of `status` ("" when it is on: an available quest).
fn accept_why(status: &str, line: &str) -> String {
    match status {
        "available" => String::new(),
        "active" => "Already under way.".into(),
        "ready" => line.to_string(),
        "locked" if !line.is_empty() => line.to_string(),
        "locked" => "Not open yet.".into(),
        "" => "Pick a quest first.".into(),
        _ => "No longer on offer.".into(),
    }
}

/// The detail's status line.
fn status_text(status: &str) -> &'static str {
    match status {
        "available" => "On offer",
        "active" => "Under way",
        "ready" => "Done: report back",
        "locked" => "Locked",
        "completed" => "Completed",
        _ => "Closed",
    }
}

impl BoardPane {
    /// The agent link's view: the picked quest and the last refusal.
    pub fn state(&self) -> Value {
        json!({"selected": self.selected, "errors": self.errors})
    }

    /// Keeps the pick on a listed quest (the first row when it is gone) and gives it ("" for an empty board); the
    /// caller fetches its detail (`board::detail`) for `draw`.
    pub fn select(&mut self, view: &Dict) -> String {
        let all = rows(view);
        if !all.iter().any(|r| s(r, "quest") == self.selected) {
            self.selected = all.first().map(|r| s(r, "quest")).unwrap_or_default();
        }
        self.selected.clone()
    }

    /// Applies a pick (Accept and Close are the caller's: they go through the conversation).
    pub fn pick(&mut self, quest: String) {
        self.selected = quest;
        self.errors.clear();
    }

    /// The detail's paragraphs (text, size, colour) of `det` (`board::detail`) for the picked row `row` (B7): the
    /// status and the row's line, the description, the steps (done, now, then), the rewards in the purse's name, the
    /// loot picks, reputation, the node and the deadline.
    fn detail_paras(cx: &Cx, det: &Dict, row: &Dict) -> Vec<(String, f32, Rgba)> {
        let d = &cx.data;
        let world = cx.game.as_ref().map(|g| g.gs.world_id.clone()).unwrap_or_default();
        let mut out = Vec::new();
        let status = s_or(det, "status", &s(row, "status"));
        let line = s(row, "line");
        out.push((if line.is_empty() { status_text(&status).to_string() } else { format!("{}: {line}", status_text(&status)) }, SMALL, DIM));
        let desc = s(det, "description");
        if !desc.is_empty() {
            out.push((desc, BODY, TEXT));
        }
        for st in json::aget(det, "steps") {
            let st = json::dict(st);
            let (mark, col) = match s(st, "state").as_str() {
                "done" => ("Done", DIM),
                "current" => ("Now", GOOD),
                _ => ("Then", TEXT),
            };
            out.push((format!("{mark}: {}", s(st, "text")), BODY, col));
        }
        // the template's rewards; the currency in the purse's name (CAMPAIGN1 P1)
        let mut rewards = json::dget(det, "rewards").clone();
        let gold = rewards.shift_remove("gold").map(|g| json::to_int(&g)).unwrap_or(0);
        let mut lines = crate::hub::reward_lines(cx, &rewards);
        if gold > 0 {
            lines.push(format!("{gold} {}", purse_word(cx, PurseWord::Currency)));
        }
        if !lines.is_empty() {
            out.push((format!("Rewards: {}", lines.join(", ")), BODY, TEXT));
        }
        let loot: Vec<String> = json::sget(det, "loot").iter().map(|l| s_or(d.resolve(&world, "loot", l), "name", l)).collect();
        let picks = json::i(det, "loot_picks", 0);
        if picks > 0 && !loot.is_empty() {
            out.push((format!("Pick {picks} of: {}", loot.join(", ")), BODY, TEXT));
        }
        let rep: Vec<String> = json::dict(json::get(det, "reputation"))
            .iter()
            .filter(|(_, v)| json::is_int(v) && json::to_int(v) != 0)
            .map(|(f, v)| format!("{} {:+}", s_or(d.resolve(&world, "factions", f), "name", f), json::to_int(v)))
            .collect();
        if !rep.is_empty() {
            out.push((format!("Reputation: {}", rep.join(", ")), BODY, TEXT));
        }
        let node = s_or(det, "node_name", &s(row, "node_name"));
        if !node.is_empty() {
            out.push((format!("Where: {node}"), BODY, INFO));
        }
        let due = json::dict(det.get("deadline").or_else(|| row.get("deadline")).unwrap_or(&Value::Null)).clone();
        if !due.is_empty() {
            out.push((s(&due, "text"), BODY, DEADLINE));
        }
        out
    }

    /// Draws board `view` (`board::view`) with the picked quest's detail `det` (`board::detail` of `select`'s pick) in
    /// the shop's frame (B11); `wrap` makes a screen action of a pane click. Accept is on only for an offered quest
    /// (its hover card says why otherwise).
    pub fn draw<A: Clone>(&mut self, ui: &mut Ui<A>, cx: &Cx, view: &Dict, det: &Dict, wrap: impl Fn(BoardAct) -> A) {
        let fr = crate::shop::frame_rect(&ui.p);
        let name = s_or(view, "name", "Board");
        ui.p.panel(fr, MODAL, FRAME, 4.0);
        ui.push("board", "panel", name.clone(), fr, true, false, None, Vec::new());
        let sc = ui.p.fonts.scale;
        let (lb, ls) = (ui.p.line_h(BODY).ceil() as i32, ui.p.line_h(SMALL).ceil() as i32);
        let (x0, right, inner) = (fr.x + PAD, fr.x + fr.w - PAD, fr.w - 2 * PAD);
        // -- header: the board's name and its keeper
        let keeper = s(view, "keeper_name");
        let keeper_w = if keeper.is_empty() { 0.0 } else { ui.p.text_w(&keeper, BODY) + 12.0 };
        let head_h = ui.p.line_h(HEAD).ceil() as i32;
        ui.p.text_fit(x0 as f32, fr.y as f32 + 5.0, &name, HEAD, inner as f32 - keeper_w, GOLD);
        if !keeper.is_empty() {
            ui.p.text_right(right as f32, (fr.y + 5 + head_h - lb) as f32, &keeper, BODY, DIM);
        }
        let mut top = fr.y + 5 + head_h + 4;
        // -- the keeper's intro (B11), wrapped
        let intro = s(view, "intro");
        if !intro.is_empty() {
            for l in ui.p.wrap(&intro, BODY, inner as f32) {
                ui.p.text(x0 as f32, top as f32, &l, BODY, TEXT);
                top += lb;
            }
            top += 3;
        }
        // -- the action bar and the refusals at the bottom; the list and the detail between
        let bh = 17.max(lb + 6);
        let bar_y = fr.y + fr.h - PAD - bh;
        let err_y = bar_y - 3 - lb;
        let bottom = if self.errors.is_empty() { bar_y - 4 } else { err_y - 3 };
        let list_w = ((LIST_W as f32 * sc).round() as i32).min((inner - GAP) / 2 + 40);
        let list = Rect::new(x0, top, list_w, bottom - top);
        let det_r = Rect::new(x0 + list_w + GAP, top, inner - list_w - GAP, bottom - top);
        let all = rows(view);
        self.list(ui, &all, view, list, lb, ls, &wrap);
        let row = all.iter().find(|r| s(r, "quest") == self.selected).cloned().unwrap_or_default();
        Self::detail(ui, cx, det, &row, det_r);
        // -- the refusal line and the bar: Close at the left, Accept at the right
        if !self.errors.is_empty() {
            let text = self.errors.join(" ");
            let r = Rect::new(x0, err_y, inner, lb);
            ui.p.text_fit(r.x as f32, r.y as f32, &text, BODY, r.w as f32, FOE);
            ui.push("board.errors", "label", text.clone(), r, true, false, None, Vec::new());
        }
        let bw = |ui: &mut Ui<A>, t: &str| (ui.p.text_w(t, BODY).ceil() as i32 + 16).max(44);
        let (accept_w, close_w) = (bw(ui, "Accept"), bw(ui, "Close"));
        let why = accept_why(&s(&row, "status"), &s(&row, "line"));
        let accept_tip: Tip = if why.is_empty() { tip("Accept", &format!("Take {} on.", s(&row, "name"))) } else { tip("Accept", &why) };
        ui.button_full("board.accept", Rect::new(right - accept_w, bar_y, accept_w, bh), "Accept", wrap(BoardAct::Accept(self.selected.clone())), why.is_empty(), false, BODY, accept_tip);
        ui.button_full("board.close", Rect::new(x0, bar_y, close_w, bh), "Close", wrap(BoardAct::Close), true, false, BODY, tip("Close", "Back to the conversation (Esc)."));
    }

    /// B5, B6: the sections, each a heading and its rows (the name; the node with the deadline at the right; the
    /// section's line), in a scroll area.
    #[allow(clippy::too_many_arguments)]
    fn list<A: Clone>(&self, ui: &mut Ui<A>, all: &[Dict], view: &Dict, area: Rect, lb: i32, ls: i32, wrap: &impl Fn(BoardAct) -> A) {
        ui.p.panel(area, COL_BG, BORDER, RADIUS);
        let row_h = |r: &Dict| {
            let second = !s(r, "node_name").is_empty() || r.contains_key("deadline");
            4 + lb + if second { ls } else { 0 } + if s(r, "line").is_empty() { 0 } else { ls }
        };
        let head_h = ls + 4;
        let sections = json::aget(view, "sections");
        let content = sections.len() as i32 * head_h + all.iter().map(|r| row_h(r) + 1).sum::<i32>() + 4;
        let off = ui.scroll_begin("board.list", area, content);
        if all.is_empty() {
            ui.p.text_fit(area.x as f32 + 4.0, area.y as f32 + 3.0, "Nothing on the board.", SMALL, (area.w - 8) as f32, DIM);
        }
        let w = area.w - 5;
        let mut y = area.y + 2 - off;
        for sec in sections {
            let sec = json::dict(sec);
            ui.p.text(area.x as f32 + 4.0, y as f32 + 2.0, &s(sec, "title"), SMALL, INFO);
            y += head_h;
            for r in all.iter().filter(|r| s(r, "_section") == s(sec, "id")) {
                let q = s(r, "quest");
                let h = row_h(r);
                let rr = Rect::new(area.x + 1, y, w, h);
                let picked = q == self.selected;
                let hover = ui.hover(rr);
                if picked || hover {
                    ui.p.panel(rr, if picked { SELECTED } else { PANEL_HI }, if picked { GOLD } else { rgb(110, 140, 196) }, RADIUS);
                }
                let (tx, tw) = (rr.x as f32 + 4.0, (rr.w - 8) as f32);
                let name = s(r, "name");
                let mut ty = rr.y as f32 + 2.0;
                ui.p.text_fit(tx, ty, &name, BODY, tw, if picked { GOLD } else { TEXT });
                ty += lb as f32;
                let node = s(r, "node_name");
                let due = r.get("deadline").map(|v| s(json::dict(v), "text")).unwrap_or_default();
                if !node.is_empty() || !due.is_empty() {
                    let dw = if due.is_empty() { 0.0 } else { ui.p.text_w(&due, SMALL) };
                    if !due.is_empty() {
                        ui.p.text_right(tx + tw, ty, &due, SMALL, DEADLINE);
                    }
                    if !node.is_empty() {
                        ui.p.text_fit(tx, ty, &node, SMALL, tw - dw - if due.is_empty() { 0.0 } else { 6.0 }, INFO);
                    }
                    ty += ls as f32;
                }
                let line = s(r, "line");
                if !line.is_empty() {
                    ui.p.text_fit(tx, ty, &line, SMALL, tw, DIM);
                }
                let mut t = tip(&name, &line);
                if !node.is_empty() {
                    t.push((node.clone(), INFO));
                }
                if !due.is_empty() {
                    t.push((due.clone(), DEADLINE));
                }
                ui.push(format!("board.row.{q}"), "row", name.clone(), rr, true, picked, Some(wrap(BoardAct::Pick(q.clone()))), t);
                y += h + 1;
            }
        }
        ui.scroll_end();
    }

    /// B7: the picked quest's name and paragraphs, wrapped, in a scroll area.
    fn detail<A: Clone>(ui: &mut Ui<A>, cx: &Cx, det: &Dict, row: &Dict, area: Rect) {
        ui.p.panel(area, COL_BG, BORDER, RADIUS);
        if det.is_empty() && row.is_empty() {
            ui.scroll_begin("board.detail", area, 0);
            ui.scroll_end();
            return;
        }
        let tw = (area.w - 12) as f32;
        let mut lines: Vec<(String, f32, Rgba)> = Vec::new();
        let name = s_or(det, "name", &s(row, "name"));
        for l in ui.p.wrap(&name, BODY + 1.0, tw) {
            lines.push((l, BODY + 1.0, GOLD));
        }
        let paras = Self::detail_paras(cx, det, row);
        for (text, size, col) in paras {
            for l in ui.p.wrap(&text, size, tw) {
                lines.push((l, size, col));
            }
        }
        let gap = 2.0;
        let content = lines.iter().map(|(_, size, _)| ui.p.line_h(*size) + gap).sum::<f32>().ceil() as i32 + 6;
        let off = ui.scroll_begin("board.detail", area, content);
        let mut y = (area.y + 3 - off) as f32;
        for (l, size, col) in &lines {
            ui.p.text(area.x as f32 + 5.0, y, l, *size, *col);
            y += ui.p.line_h(*size) + gap;
        }
        ui.scroll_end();
    }
}

// ---- The UI Gallery's board (a fixture view) ----------------------------------------------------------------

/// A fixture row (`board::view`'s shape).
fn sample_row(q: &str, name: &str, status: &str, node: &str, line: &str, due: Option<(i64, i64)>) -> Value {
    let node_qid = if node.is_empty() { String::new() } else { format!("sample:{}", node.to_lowercase().replace(' ', "_")) };
    let mut r = json!({"quest": format!("sample:{q}"), "name": name, "status": status, "node": node_qid, "node_name": node, "line": line});
    if let Some((day, left)) = due {
        r["deadline"] = json!({"day": day, "left": left, "text": board::deadline_text(day, left)});
    }
    r
}

/// The gallery's fixture board: every section (a main mission with a deadline, a job, one under way, one locked with
/// its reason, one closed), as `board::view` and `board::detail` give them, in no world.
pub fn sample() -> (Dict, Vec<Dict>) {
    let rows = [
        ("main", vec![sample_row("m1", "Clear the Old Mill", "available", "Old Mill", "", Some((14, 6)))]),
        ("jobs", vec![sample_row("j1", "Escort the Salt Cart", "available", "North Road", "", None), sample_row("j2", "Hunt the Marsh Hounds", "available", "Fen Edge", "", None)]),
        ("under_way", vec![sample_row("u1", "Find the Lost Patrol", "active", "Watchtower", "Search the watchtower", Some((20, 12)))]),
        ("locked", vec![sample_row("l1", "Storm the Keep", "locked", "Keep", "Needs Clear the Old Mill (completed)", None)]),
        ("closed", vec![sample_row("c1", "Save the Ferryman", "unavailable", "", "The ferry burned before you came.", None)]),
    ];
    let sections: Vec<Value> = rows
        .iter()
        .map(|(id, list)| {
            let title = board::SECTIONS.iter().find(|s| s.0 == *id).map_or(*id, |s| s.1);
            json!({"id": id, "title": title, "rows": list})
        })
        .collect();
    let view = json!({"board": "sample:board", "name": "Notice board", "keeper": "sample:clerk", "keeper_name": "The Clerk", "open": true,
        "intro": "Work for anyone who can hold a blade. Take only what you can finish; the board remembers who failed it.", "sections": sections});
    let detail = |q: &str, name: &str, status: &str, desc: &str, steps: &[(&str, &str)], rewards: Value, node: &str, due: Option<(i64, i64)>| {
        let mut d = json!({"quest": format!("sample:{q}"), "name": name, "status": status, "description": desc,
            "steps": steps.iter().map(|(t, st)| json!({"text": t, "state": st})).collect::<Vec<_>>(), "node_name": node, "rewards": rewards,
            "loot": [], "loot_picks": 0, "reputation": {}});
        if let Some((day, left)) = due {
            d["deadline"] = json!({"day": day, "left": left, "text": board::deadline_text(day, left)});
        }
        json::dict(&d).clone()
    };
    let details = vec![
        detail("m1", "Clear the Old Mill", "available", "Raiders hold the old mill on the river. Drive them out before the grain carts come.", &[("Reach the Old Mill", "to_come"), ("Drive out the raiders", "to_come")], json!({"xp": 40, "gold": 25}), "Old Mill", Some((14, 6))),
        detail("j1", "Escort the Salt Cart", "available", "The salt cart leaves at dusk. See it to the north gate in one piece.", &[("Guard the cart on the North Road", "to_come")], json!({"xp": 20, "gold": 15}), "North Road", None),
        detail("j2", "Hunt the Marsh Hounds", "available", "A pack has taken three goats this week.", &[("Clear the pack at the Fen Edge", "to_come")], json!({"xp": 15}), "Fen Edge", None),
        detail("u1", "Find the Lost Patrol", "active", "Four scouts went up to the watchtower and did not come back.", &[("Reach the Watchtower", "done"), ("Search the watchtower", "current"), ("Report back", "to_come")], json!({"xp": 30, "gold": 10}), "Watchtower", Some((20, 12))),
        detail("l1", "Storm the Keep", "locked", "Once the mill is free, the keep is next.", &[("Breach the gate", "to_come"), ("Take the keep", "to_come")], json!({"xp": 80, "gold": 60}), "Keep", None),
        detail("c1", "Save the Ferryman", "unavailable", "The ferryman was taken across the river.", &[("Reach the ferry", "to_come")], json!({"xp": 10}), "", None),
    ];
    (json::dict(&view).clone(), details)
}

/// The UI Gallery's board pane (STORYUI1-U2): the pane on the fixture board over a plain backdrop; picks show their
/// detail, Accept moves an offered quest to Under way (as B8 does in a game), Close and Esc go back.
pub struct BoardSample {
    pane: BoardPane,
    view: Dict,
    details: Vec<Dict>,
    w: Vec<Widget<BoardAct>>,
}

impl Default for BoardSample {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardSample {
    pub fn new() -> Self {
        let (view, details) = sample();
        Self { pane: BoardPane::default(), view, details, w: Vec::new() }
    }

    /// Accept on the fixture: an offered row moves to Under way with its first step as the line.
    fn accept(&mut self, q: &str) {
        let Some(det) = self.details.iter_mut().find(|d| s(d, "quest") == q) else { return };
        if s(det, "status") != "available" {
            self.pane.errors = vec![accept_why(&s(det, "status"), "")];
            return;
        }
        det.insert("status".into(), "active".into());
        let step = json::aget(det, "steps").first().map(|st| s(json::dict(st), "text")).unwrap_or_default();
        if let Some(Value::Array(steps)) = det.get_mut("steps") {
            if let Some(Value::Object(first)) = steps.first_mut() {
                first.insert("state".into(), "current".into());
            }
        }
        let mut sections: Vec<Value> = json::aget(&self.view, "sections").to_vec();
        let mut moved = None;
        for sec in sections.iter_mut() {
            if let Some(Value::Array(list)) = sec.get_mut("rows") {
                if let Some(i) = list.iter().position(|r| r["quest"] == json!(q)) {
                    let mut r = list.remove(i);
                    r["status"] = json!("active");
                    r["line"] = json!(step);
                    moved = Some(r);
                }
            }
        }
        let Some(r) = moved else { return };
        match sections.iter_mut().find(|sec| sec["id"] == json!("under_way")) {
            Some(sec) => sec["rows"].as_array_mut().into_iter().for_each(|l| l.insert(0, r.clone())),
            None => sections.push(json!({"id": "under_way", "title": "Under way", "rows": [r]})),
        }
        let order: Vec<&str> = board::SECTIONS.iter().map(|s| s.0).collect();
        sections.retain(|sec| sec["rows"].as_array().is_some_and(|l| !l.is_empty()));
        sections.sort_by_key(|sec| order.iter().position(|id| sec["id"] == json!(id)).unwrap_or(order.len()));
        self.view.insert("sections".into(), Value::Array(sections));
        self.pane.errors.clear();
    }

    fn act(&mut self, cx: &mut Cx, a: BoardAct) {
        match a {
            BoardAct::Pick(q) => self.pane.pick(q),
            BoardAct::Accept(q) => self.accept(&q),
            BoardAct::Close => cx.nav.push(Nav::Pop),
        }
    }
}

impl Screen<GameCx> for BoardSample {
    fn id(&self) -> &'static str {
        "board_sample"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<BoardAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgb(14, 18, 26));
        ui.p.fill(Rect::new(0, 0, W, H), DIM_FILL);
        let sel = self.pane.select(&self.view);
        let det = self.details.iter().find(|d| s(d, "quest") == sel).cloned().unwrap_or_default();
        let view = self.view.clone();
        self.pane.draw(&mut ui, cx, &view, &det, |a| a);
        self.w = ui.widgets;
    }

    fn back(&mut self, cx: &mut Cx) {
        cx.nav.push(Nav::Pop);
    }

    fn state(&self, _cx: &Cx) -> Value {
        json!({"pane": self.pane.state(), "sections": json::aget(&self.view, "sections").iter().map(|s| json::s(json::dict(s), "id")).collect::<Vec<_>>()})
    }

    widget_plumbing!();
}

/// STORYUI1-U2 tests (STORYUI1_TASKS 2: TU4-TU6) on a scratch world `ward`: Clerk keeps a board (open from act 1) with
/// a main mission, a second one locked behind it (shown), a job given by no one, a job given by Smith (Accept refuses
/// it) and a long-named job (for the text sizes); four nodes on routes from the hub. A debug game (the story tests'
/// context). Expectations follow from the rules, the kit's own views and the fixture's data.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use crate::region::RegionMap;
    use crate::screens::enter_game;
    use crate::story::tests::story_cx;
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::TEXT_SIZES;
    use emberwake_world::{explore, new_game, quest, Data};

    fn with_ids(w: &str, rows: Value) -> Value {
        let rows = json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect();
        Value::Array(rows)
    }

    const LONG: &str = "The Long Road Through the Drowned Marshes to the Old Salt Works and Back Again";

    fn data() -> Data {
        let quests = json!([
            {"id": "m1", "name": "First Mission", "giver": "clerk", "board": "clerk_board", "track": "main", "template": "reach_and_clear",
             "description": "Raiders hold the kiln. Drive them out.",
             "steps": [{"text": "Reach the kiln", "location": "kiln"}, {"text": "Raid the kiln", "location": "kiln", "encounter": "enc_a"}]},
            {"id": "m2", "name": "Second Mission", "giver": "clerk", "board": "clerk_board", "track": "main", "template": "clear_site", "requires": ["m1"], "show_locked": true,
             "steps": [{"text": "Hold the watch", "encounter": "enc_w"}]},
            {"id": "job1", "name": "Job One", "board": "clerk_board", "template": "clear_site", "steps": [{"text": "Clear the watch", "encounter": "enc_w"}]},
            {"id": "foreign", "name": "Smith Job", "giver": "smith", "board": "clerk_board", "template": "clear_site", "steps": [{"text": "Work for Smith", "encounter": "enc_w"}]},
            {"id": "long", "name": LONG, "board": "clerk_board", "template": "clear_site",
             "description": "A very long description that wraps over several lines in the detail pane, so the pane's scroll area and its wrapping are both used at the larger text sizes as well.",
             "steps": [{"text": "Walk the long road through the drowned marshes all the way to the old salt works", "encounter": "enc_m"}]}
        ]);
        let worlds = json!({
            "ward": {"manifest": {"id": "ward", "name": "Ward", "start_location": "hub", "start_party": ["hero"]}, "own": {
                "locations": with_ids("ward", json!([{"id": "hub", "name": "Hub"}, {"id": "kiln", "name": "Kiln"}, {"id": "watch", "name": "Watch"},
                    {"id": "marsh", "name": "The Drowned Marshes Beyond the Old Salt Works"}])),
                "routes": with_ids("ward", json!([{"id": "r1", "from": "hub", "to": "kiln", "time": 1}, {"id": "r2", "from": "hub", "to": "watch", "time": 1},
                    {"id": "r3", "from": "hub", "to": "marsh", "time": 2}])),
                "characters": with_ids("ward", json!([{"id": "clerk", "name": "Clerk", "role": "npc"}, {"id": "smith", "name": "Smith", "role": "npc"}])),
                "encounters": with_ids("ward", json!([{"id": "enc_a", "name": "A", "enemies": []}, {"id": "enc_w", "name": "W", "location": "watch", "enemies": []},
                    {"id": "enc_m", "name": "M", "location": "marsh", "enemies": []}])),
                "boards": with_ids("ward", json!([{"id": "clerk_board", "name": "Notice board", "keeper": "clerk",
                    "intro": "Work for anyone who can hold a blade. Take only what you can finish; the board remembers every name that failed it, and so do I.",
                    "requires_flags": [{"key": "act", "op": ">=", "value": 1}]}])),
                "quests": with_ids("ward", quests)
            }}
        });
        let catalog = json!({
            "classes": [{"id": "fighter", "name": "Fighter"}],
            "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
        });
        let game = json!({"quests": [
            {"id": "reach_and_clear", "objectives": [{"kind": "reach_location"}, {"kind": "win_encounter"}], "rewards": {"xp": 10, "gold": 5}},
            {"id": "clear_site", "objectives": [{"kind": "win_encounter"}], "rewards": {"xp": 5}}
        ]});
        Data::from_values(&catalog, &worlds, &game)
    }

    /// A debug game in `ward` at act 1 (the board open), the hub visited, on its region map.
    fn ward_cx() -> Cx {
        let mut cx = story_cx(data());
        let mut gs = new_game::create_for_world(&cx.data.clone(), "ward", 1).expect("a new game");
        gs.set_flag("act", json!(1));
        gs.set_flag(&explore::visited_flag("ward:hub"), json!(true));
        enter_game(&mut cx, gs, -1, Vec::new());
        cx.nav.clear();
        cx
    }

    fn widget(s: &dyn Screen<GameCx>, id: &str) -> Option<Value> {
        s.widgets().into_iter().find(|w| w["id"] == id)
    }

    /// A conversation with Clerk with the board opened through its service reply (B3).
    fn open_board(cx: &mut Cx, g: &mut Gfx) -> RegionMap {
        let mut map = RegionMap::new(cx);
        map.talk(cx, "clerk");
        frame(&mut map, cx, g, 1.0);
        let choices = json::aget(json::dict(&map.state(cx)["dialogue"]), "choices").to_vec();
        let k = choices.iter().position(|c| s(json::dict(c), "service") == "board").expect("B3: the keeper's board reply");
        map.click_id(cx, &format!("choice.{k}")).expect("the board reply");
        frame(&mut map, cx, g, 1.0);
        map
    }

    /// The quest qids of a section of the kit's board view.
    fn section(cx: &Cx, id: &str) -> Vec<String> {
        let v = emberwake_world::board::view(&cx.data, cx.gs(), "ward:clerk_board");
        rows(&v).into_iter().filter(|r| s(r, "_section") == id).map(|r| s(&r, "quest")).collect()
    }

    /// TU4 (B3, B5, B8, B9, B11): the keeper's board reply opens the pane in the dialogue box's place, one row per
    /// quest the kit lists; a pick shows its detail; Accept takes an offered quest (it moves to Under way and the log
    /// says so), refuses another giver's quest on the pane and is off for a locked one; right click goes back to the
    /// conversation.
    #[test]
    fn the_board_reply_opens_the_pane_and_accept_takes_a_quest() {
        let (mut cx, mut g) = (ward_cx(), gfx());
        assert!(explore::npc_services(&cx.data, cx.gs(), "ward:clerk").contains(&"board"), "B4: the keeper wears the board badge");
        let mut map = open_board(&mut cx, &mut g);
        assert_eq!(map.state(&cx)["board"], json!("ward:clerk_board"));
        assert!(widget(&map, "board").is_some() && widget(&map, "dialogue").is_none(), "B11: the pane in the dialogue box's place");
        let b = cx.data.resolve("ward", "boards", "clerk_board").clone();
        assert!(emberwake_world::board::was_opened(cx.gs(), &b), "B9: the first open sets the board's flag");
        let view = emberwake_world::board::view(&cx.data, cx.gs(), "ward:clerk_board");
        let listed: Vec<String> = rows(&view).iter().map(|r| s(r, "quest")).collect();
        assert!(!listed.is_empty());
        for q in &listed {
            assert!(widget(&map, &format!("board.row.{q}")).is_some(), "B5: a row for {q}");
        }
        // the first row is picked; a click picks another and shows its detail
        assert_eq!(map.state(&cx)["board_pane"]["selected"], json!(listed[0]));
        map.click_id(&mut cx, "board.row.ward:job1").expect("the job's row");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(map.state(&cx)["board_pane"]["selected"], json!("ward:job1"));
        assert!(widget(&map, "board.accept").is_some_and(|w| w["enabled"] == json!(true)), "Accept on an offered quest");
        // B8: Accept takes it; it moves to Under way; the log names it
        map.click_id(&mut cx, "board.accept").expect("Accept");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(quest::status(&cx.data, cx.gs(), "ward:job1"), "active");
        assert!(section(&cx, "under_way").contains(&"ward:job1".to_string()));
        let name = s(&quest::resolved(&cx.data, cx.gs(), "ward:job1"), "name");
        assert!(cx.log.iter().any(|l| *l == format!("Quest taken: {name}.")), "{:?}", cx.log);
        assert!(widget(&map, "board.accept").is_some_and(|w| w["enabled"] == json!(false)), "off once under way");
        // a quest given by someone else: the kit's refusal shows on the pane, nothing starts
        map.click_id(&mut cx, "board.row.ward:foreign").expect("Smith's row");
        frame(&mut map, &mut cx, &mut g, 1.0);
        map.click_id(&mut cx, "board.accept").expect("Accept");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(quest::status(&cx.data, cx.gs(), "ward:foreign"), "available");
        let errors = map.state(&cx)["board_pane"]["errors"].clone();
        assert!(errors.as_array().is_some_and(|e| !e.is_empty()) && widget(&map, "board.errors").is_some(), "{errors}");
        // a locked quest: Accept is off and its card gives the lock reason
        map.click_id(&mut cx, "board.row.ward:m2").expect("the locked row");
        frame(&mut map, &mut cx, &mut g, 1.0);
        let accept = widget(&map, "board.accept").expect("Accept");
        let reason = emberwake_world::board::lock_reason(&cx.data, cx.gs(), "ward:m2");
        assert!(accept["enabled"] == json!(false) && accept["tip"].to_string().contains(&reason), "{accept}");
        // B11: right click (back) closes the pane back to the conversation
        map.back(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let st = map.state(&cx);
        assert!(st["board"].is_null() && st["board_pane"].is_null() && !st["dialogue"].is_null());
        assert!(widget(&map, "board").is_none() && widget(&map, "dialogue").is_some());
        // Close does the same
        let mut map = open_board(&mut cx, &mut g);
        map.click_id(&mut cx, "board.close").expect("Close");
        assert!(map.state(&cx)["board"].is_null() && !map.state(&cx)["dialogue"].is_null());
    }

    /// B4: the board service's badge is the quest badge with the kit's hover text.
    #[test]
    fn the_board_service_wears_the_quest_badge() {
        assert_eq!(crate::region::service_icon("board"), emberwake_content::art::badge("quest"));
        assert_eq!(crate::region::services_text(&["board"]), emberwake_world::board::BADGE_TEXT);
    }

    /// The texts drawn inside the pane checked by TEXTFIT1's audit against `screen`'s widgets: every pane control
    /// inside the frame; nothing past the canvas, overlapping or spilling out of its control; a shortened text keeps
    /// its full text in a control's text or hover card.
    fn audit(screen: &dyn Screen<GameCx>, recs: Vec<TextRec>, what: &str) {
        let widgets = screen.widgets();
        let (fx, fy, fw, fh) = rect(&widget(screen, "board").expect("the pane"));
        for w in widgets.iter().filter(|w| w["id"] != "board" && rect(w).2 > 0 && rect(w).3 > 0 && w["id"].as_str().is_some_and(|i| i.starts_with("board."))) {
            let (x, y, ww, hh) = rect(w);
            assert!(x >= fx && y >= fy && x + ww <= fx + fw && y + hh <= fy + fh, "{what}: {w} inside the pane");
        }
        let inside: Vec<TextRec> = recs.into_iter().filter(|r| r.x >= fx as f32 && r.y >= fy as f32 && r.x < (fx + fw) as f32 && r.y < (fy + fh) as f32).collect();
        assert!(!inside.is_empty(), "{what}: the pane draws its texts");
        let boxes: Vec<(String, f32, f32, f32, f32)> = widgets
            .iter()
            .map(|w| {
                let (x, y, ww, hh) = rect(w);
                (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
            })
            .collect();
        let v = aegis_ui::text_audit::issues(&inside, W as f32, H as f32, &boxes);
        let c = &v["counts"];
        assert_eq!((c["edge"].clone(), c["overlap"].clone(), c["spill"].clone()), (json!(0), json!(0), json!(0)), "{what}: {v}");
        for cut in v["cut"].as_array().into_iter().flatten() {
            let full = cut["text"].as_str().unwrap_or("");
            let named = widgets.iter().any(|w| w["text"].as_str().is_some_and(|t| t.starts_with(full)) || w["tip"].as_array().into_iter().flatten().any(|t| t.as_str() == Some(full)));
            assert!(named, "{what}: '{full}' is shortened without its full text in its control");
        }
    }

    /// TU5 (B11, TEXTFIT1): the pane in the region map (the shop's frame) and the gallery's sample read clean at the
    /// four text sizes, with a long row picked so its detail scrolls; the sample's deadline line shows.
    #[test]
    fn the_board_pane_reads_clean_at_every_text_size() {
        let (mut cx, mut g) = (ward_cx(), gfx());
        let mut map = open_board(&mut cx, &mut g);
        map.click_id(&mut cx, "board.row.ward:long").expect("the long row");
        for &(size, scale) in TEXT_SIZES.iter() {
            let recs = frame(&mut map, &mut cx, &mut g, scale as f32);
            let (_, _, fw, fh) = rect(&widget(&map, "board").expect("the pane"));
            if scale <= 1.0 {
                assert_eq!((fw, fh), (600, 300), "{size}: the shop's frame");
            }
            audit(&map, recs, &format!("region at {size}"));
        }
        let mut sample = BoardSample::new();
        for &(size, scale) in TEXT_SIZES.iter() {
            let recs = frame(&mut sample, &mut cx, &mut g, scale as f32);
            if scale == 1.0 {
                let (view, _) = super::sample();
                let due = rows(&view).iter().find_map(|r| r.get("deadline").map(|v| s(json::dict(v), "text"))).expect("a fixture deadline");
                assert!(recs.iter().any(|r| r.text == due), "B6: the deadline line {due}");
            }
            audit(&sample, recs, &format!("sample at {size}"));
        }
    }

    /// The gallery's sample: Accept moves an offered quest to Under way; Close and Esc go back.
    #[test]
    fn the_sample_board_moves_an_accepted_quest_under_way() {
        let (mut cx, mut g) = (story_cx(data()), gfx());
        let mut sample = BoardSample::new();
        frame(&mut sample, &mut cx, &mut g, 1.0);
        let first = sample.state(&cx)["pane"]["selected"].as_str().unwrap_or("").to_string();
        assert!(!first.is_empty(), "the first row is picked");
        sample.click_id(&mut cx, "board.accept").expect("Accept");
        frame(&mut sample, &mut cx, &mut g, 1.0);
        let under: Vec<String> = rows(&sample.view).into_iter().filter(|r| s(r, "_section") == "under_way").map(|r| s(&r, "quest")).collect();
        assert!(under.contains(&first), "{under:?}");
        assert!(widget(&sample, "board.accept").is_some_and(|w| w["enabled"] == json!(false)));
        sample.click_id(&mut cx, "board.close").expect("Close");
        assert!(matches!(cx.nav.last(), Some(Nav::Pop)));
        cx.nav.clear();
        sample.back(&mut cx);
        assert!(matches!(cx.nav.last(), Some(Nav::Pop)));
    }

    /// TU6 (B10): once the board was opened, the world map wears the quest badge on a job's node and its card lists
    /// "Job: <name>" per job there; none before; a taken job leaves its node; the first playable world's map has no
    /// job badge or line.
    #[test]
    fn the_world_map_marks_job_nodes_after_the_board_was_opened() {
        let (mut cx, mut g) = (ward_cx(), gfx());
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert!(!wm.widgets().iter().any(|w| w["kind"] == "badge"), "B10: no jobs before the first open");
        let d = cx.data.clone();
        emberwake_world::board::mark_opened(&d, cx.gs_mut(), "clerk_board");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let jobs = emberwake_world::board::jobs(&d, cx.gs());
        assert!(!jobs.is_empty());
        for (node, name) in &jobs {
            let id = format!("node.{}", node.replace(':', "."));
            let badge = widget(&wm, &format!("{id}.job")).unwrap_or_else(|| panic!("{id}: the job badge"));
            let (bx, by, bw, bh) = rect(&badge);
            let (nx, ny, ..) = rect(&widget(&wm, &id).expect("the node"));
            assert!(bw == 9 && bh == 9 && bx <= nx && by <= ny + 2, "{id}: a 9 px badge at the pin's top left: {badge}");
            let card = widget(&wm, &id).expect("the node")["tip"].to_string();
            assert!(card.contains(&format!("Job: {name}")), "{card}");
        }
        // a node with one job: taking it takes the badge off (an active target keeps its "!")
        let one = jobs.iter().find(|(n, _)| jobs.iter().filter(|(m, _)| m == n).count() == 1).map(|(n, _)| n.clone()).expect("a node with one job");
        let q = emberwake_world::board::quests(&d, cx.gs(), "ward:clerk_board").into_iter().find(|q| quest::status(&d, cx.gs(), q) == "available" && emberwake_world::board::node_of(&d, cx.gs(), q) == one).expect("its quest");
        quest::start(&d, cx.gs_mut(), &q);
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert!(widget(&wm, &format!("node.{}.job", one.replace(':', "."))).is_none(), "a taken job leaves its node");
        // the first playable world draws as before: no job badge, no job line
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert!(!wm.widgets().iter().any(|w| w["kind"] == "badge" || w["tip"].to_string().contains("\"Job: ")), "no jobs in the first playable world");
    }
}
