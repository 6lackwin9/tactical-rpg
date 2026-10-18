//! The widget kit every screen draws with: immediate-mode widgets on the logical 640x360 canvas, each recorded with an
//! id, kind, text, rect, enabled / active state, an optional action and a hover card, so the agent link can list,
//! hover and click exactly what the player sees. Colours follow the Godot HUD (dark navy, gold accents).

use crate::canvas::{rgb, rgba, Canvas, Rect, Rgba};
use crate::painter::{Fonts, Painter, View};
use serde_json::{json, Value};

pub const W: i32 = 640;
pub const H: i32 = 360;
pub const BODY: f32 = 7.0;
pub const SMALL: f32 = 6.0;
pub const TITLE: f32 = 7.6;
pub const HEAD: f32 = 12.0;
pub const LINE: f32 = 9.0;
pub const RADIUS: f32 = 2.5;

pub const BG: Rgba = rgb(10, 14, 24);
pub const PANEL_HI: Rgba = rgba(34, 50, 82, 250);
pub const CARD: Rgba = rgba(16, 24, 42, 250);
pub const MODAL: Rgba = rgba(18, 26, 44, 255);
pub const BORDER: Rgba = rgb(58, 78, 116);
pub const FRAME: Rgba = rgb(140, 196, 236);
pub const TEXT: Rgba = rgb(222, 228, 240);
pub const DIM: Rgba = rgb(132, 146, 172);
pub const GOLD: Rgba = rgb(244, 198, 84);
pub const INFO: Rgba = rgb(140, 196, 236);
pub const FOE: Rgba = rgb(240, 100, 92);
pub const GOOD: Rgba = rgb(96, 210, 110);
pub const SELECTED: Rgba = rgba(86, 68, 22, 250);

pub type Tip = Vec<(String, Rgba)>;

pub fn tip(title: &str, body: &str) -> Tip {
    let mut t = vec![(title.to_string(), GOLD)];
    if !body.is_empty() {
        t.push((body.to_string(), TEXT));
    }
    t
}

#[derive(Debug, Clone)]
pub struct Widget<A> {
    pub id: String,
    pub kind: &'static str,
    pub text: String,
    pub rect: Rect,
    pub enabled: bool,
    pub active: bool,
    pub action: Option<A>,
    pub tip: Tip,
}

impl<A> Widget<A> {
    pub fn info(&self) -> Value {
        json!({"id": self.id, "kind": self.kind, "text": self.text, "rect": [self.rect.x, self.rect.y, self.rect.w, self.rect.h],
            "enabled": self.enabled, "active": self.active, "clickable": self.action.is_some(), "tip": self.tip.iter().map(|t| t.0.clone()).collect::<Vec<_>>()})
    }
}

/// The drawing surface the shell lends a screen for one frame.
pub struct Gfx {
    pub canvas: Canvas,
    pub view: View,
    pub fonts: Fonts,
}

pub struct Ui<'a, A> {
    pub p: Painter<'a>,
    pub widgets: Vec<Widget<A>>,
    pub mouse: (i32, i32),
    /// Mouse position for hover effects; (-1, -1) while a screen is drawn under an overlay.
    pub live: bool,
}

impl<'a, A: Clone> Ui<'a, A> {
    pub fn new(g: &'a mut Gfx, mouse: (i32, i32), live: bool) -> Self {
        Ui { p: Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts }, widgets: Vec::new(), mouse, live }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push(&mut self, id: impl Into<String>, kind: &'static str, text: impl Into<String>, rect: Rect, enabled: bool, active: bool, action: Option<A>, tip: Tip) {
        self.widgets.push(Widget { id: id.into(), kind, text: text.into(), rect, enabled, active, action, tip });
    }

    pub fn hover(&self, r: Rect) -> bool {
        self.live && r.contains(self.mouse.0, self.mouse.1)
    }

    /// A plain button; `active` draws it selected (gold).
    #[allow(clippy::too_many_arguments)]
    pub fn button_full(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, active: bool, size: f32, t: Tip) {
        let hover = enabled && self.hover(r);
        let fill = if active { SELECTED } else if hover { PANEL_HI } else { CARD };
        let border = if active || (hover && enabled) { GOLD } else if enabled { rgb(100, 130, 186) } else { BORDER };
        self.p.panel(r, fill, border, RADIUS);
        let col = if enabled { if active { GOLD } else { TEXT } } else { DIM };
        let line = size * 1.28;
        let txt = self.p.fit(label, size, if r.w < 20 { r.w as f32 } else { r.w as f32 - 6.0 });
        self.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - line) / 2.0, &txt, size, col);
        self.push(id, "button", label, r, enabled, active, Some(action), t);
    }

    pub fn button(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool) {
        self.button_full(id, r, label, action, enabled, false, BODY, Vec::new());
    }

    pub fn button_tip(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, t: Tip) {
        self.button_full(id, r, label, action, enabled, false, BODY, t);
    }

    /// A tab: a button drawn selected when it is the open one.
    pub fn tab(&mut self, id: &str, r: Rect, label: &str, action: A, open: bool) {
        self.button_full(id, r, label, action, true, open, BODY, Vec::new());
    }

    /// A selectable multi-line card (world cards, slot rows, list entries).
    #[allow(clippy::too_many_arguments)]
    pub fn card(&mut self, id: &str, r: Rect, lines: &[(String, Rgba)], action: Option<A>, selected: bool, enabled: bool, t: Tip) {
        let hover = enabled && action.is_some() && self.hover(r);
        let fill = if selected { SELECTED } else if hover { PANEL_HI } else { CARD };
        let border = if selected { GOLD } else if hover { rgb(110, 140, 196) } else { BORDER };
        self.p.panel(r, fill, border, RADIUS);
        let mut y = r.y as f32 + 3.0;
        for (text, col) in lines {
            if y > (r.y + r.h) as f32 - 8.0 {
                break;
            }
            self.p.text_fit(r.x as f32 + 5.0, y, text, BODY, r.w as f32 - 10.0, if enabled { *col } else { DIM });
            y += 10.0;
        }
        let text = lines.iter().map(|l| l.0.clone()).collect::<Vec<_>>().join(" | ");
        self.push(id, "card", text, r, enabled, selected, action, t);
    }

    pub fn checkbox(&mut self, id: &str, x: i32, y: i32, label: &str, checked: bool, action: A, enabled: bool) {
        let b = Rect::new(x, y, 10, 10);
        let r = Rect::new(x, y - 1, 12 + self.p.text_w(label, BODY).ceil() as i32 + 4, 12);
        let hover = enabled && self.hover(r);
        self.p.panel(b, if checked { rgb(40, 62, 110) } else { CARD }, if hover { GOLD } else { TEXT }, 1.5);
        if checked {
            self.p.line(x as f32 + 2.0, y as f32 + 5.0, x as f32 + 4.2, y as f32 + 7.5, GOLD, 0.0);
            self.p.line(x as f32 + 4.2, y as f32 + 7.5, x as f32 + 8.2, y as f32 + 2.5, GOLD, 0.0);
        }
        self.p.text(x as f32 + 14.0, y as f32 + 0.5, label, BODY, if enabled { TEXT } else { DIM });
        self.push(id, "checkbox", format!("{label}: {}", if checked { "on" } else { "off" }), r, enabled, checked, Some(action), Vec::new());
    }

    /// A value bar with - / + steps (function over form: no dragging yet).
    #[allow(clippy::too_many_arguments)]
    pub fn stepper(&mut self, id: &str, r: Rect, value: f32, text: &str, minus: A, plus: A, enabled: bool) {
        self.button(&format!("{id}.minus"), Rect::new(r.x, r.y, 12, r.h), "-", minus, enabled);
        let bar = Rect::new(r.x + 15, r.y + r.h / 2 - 2, r.w - 30 - 44, 4);
        self.p.panel(bar, rgb(40, 46, 60), rgb(40, 46, 60), 2.0);
        let fill = Rect::new(bar.x, bar.y, ((bar.w as f32) * value.clamp(0.0, 1.0)) as i32, bar.h);
        self.p.panel(fill, rgb(200, 204, 214), rgb(200, 204, 214), 2.0);
        self.p.circle((fill.x + fill.w) as f32, bar.y as f32 + 2.0, 3.0, rgb(240, 242, 248));
        self.button(&format!("{id}.plus"), Rect::new(bar.x + bar.w + 3, r.y, 12, r.h), "+", plus, enabled);
        self.p.text(bar.x as f32 + bar.w as f32 + 19.0, r.y as f32 + (r.h as f32 - LINE) / 2.0, text, BODY, DIM);
        self.push(id, "slider", text, r, enabled, false, None, Vec::new());
    }

    /// A dropdown field: the current value and a chevron; the action opens the choice list.
    pub fn dropdown(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, t: Tip) {
        let hover = enabled && self.hover(r);
        self.p.panel(r, if hover { PANEL_HI } else { rgb(30, 44, 74) }, if hover { GOLD } else { rgb(100, 130, 186) }, RADIUS);
        let col = if enabled { TEXT } else { DIM };
        self.p.text_fit(r.x as f32 + 5.0, r.y as f32 + (r.h as f32 - LINE) / 2.0, label, BODY, r.w as f32 - 16.0, col);
        let (cx, cy) = (r.x as f32 + r.w as f32 - 7.0, r.y as f32 + r.h as f32 / 2.0);
        self.p.line(cx - 2.5, cy - 1.2, cx, cy + 1.3, col, 0.0);
        self.p.line(cx, cy + 1.3, cx + 2.5, cy - 1.2, col, 0.0);
        self.push(id, "dropdown", label, r, enabled, false, Some(action), t);
    }

    /// The X close button of a frame.
    pub fn close_x(&mut self, id: &str, r: Rect, action: A) {
        let hover = self.hover(r);
        self.p.panel(r, if hover { PANEL_HI } else { CARD }, if hover { GOLD } else { rgb(100, 130, 186) }, RADIUS);
        let (cx, cy, d) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0, 3.0);
        self.p.line(cx - d, cy - d, cx + d, cy + d, TEXT, 0.0);
        self.p.line(cx - d, cy + d, cx + d, cy - d, TEXT, 0.0);
        self.push(id, "button", "Close", r, true, false, Some(action), Vec::new());
    }

    /// A modal frame with a gold title; returns the content top y.
    pub fn frame(&mut self, r: Rect, title: &str, centered: bool) -> i32 {
        self.p.panel(r, MODAL, FRAME, 4.0);
        if !title.is_empty() {
            if centered {
                self.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + 6.0, title, HEAD, GOLD);
            } else {
                self.p.text(r.x as f32 + 9.0, r.y as f32 + 6.0, title, HEAD, GOLD);
            }
        }
        r.y + 26
    }

    /// Wrapped text; returns the y below it.
    pub fn para(&mut self, x: i32, y: i32, w: i32, text: &str, size: f32, col: Rgba) -> i32 {
        let mut yy = y as f32;
        let step = size * 1.35;
        for line in self.p.wrap(text, size, w as f32) {
            self.p.text(x as f32, yy, &line, size, col);
            yy += step;
        }
        yy.ceil() as i32
    }

    /// A list row: label text, optionally clickable, hover-highlighted.
    #[allow(clippy::too_many_arguments)]
    pub fn row(&mut self, id: &str, r: Rect, label: &str, col: Rgba, action: Option<A>, selected: bool, t: Tip) {
        let clickable = action.is_some();
        let hover = clickable && self.hover(r);
        if selected || hover {
            self.p.panel(r, if selected { SELECTED } else { PANEL_HI }, if selected { GOLD } else { rgb(110, 140, 196) }, RADIUS);
        }
        self.p.text_fit(r.x as f32 + 4.0, r.y as f32 + (r.h as f32 - LINE) / 2.0, label, BODY, r.w as f32 - 8.0, col);
        self.push(id, "row", label, r, true, selected, action, t);
    }
}

/// The hover card of the topmost hovered widget that has one.
pub fn draw_tip<A>(p: &mut Painter, widgets: &[Widget<A>], mouse: (i32, i32)) {
    let Some(wd) = widgets.iter().rev().find(|wd| wd.rect.contains(mouse.0, mouse.1) && (!wd.tip.is_empty() || wd.kind == "backdrop")) else { return };
    if wd.tip.is_empty() {
        return;
    }
    let max_w = 160.0;
    let mut lines: Vec<(String, Rgba, f32)> = Vec::new();
    for (i, (t, col)) in wd.tip.iter().enumerate() {
        let size = if i == 0 { BODY } else { SMALL };
        for part in p.wrap(t, size, max_w) {
            lines.push((part, *col, size));
        }
    }
    let tw = lines.iter().map(|l| p.text_w(&l.0, l.2)).fold(0.0f32, f32::max) + 10.0;
    let th = lines.iter().map(|l| if l.2 == BODY { 10.0 } else { 8.0 }).sum::<f32>() + 6.0;
    let (mut tx, mut ty) = (mouse.0 + 10, mouse.1 + 12);
    if tx as f32 + tw > W as f32 - 2.0 {
        tx = mouse.0 - tw as i32 - 6;
    }
    if ty as f32 + th > H as f32 - 2.0 {
        ty = mouse.1 - th as i32 - 6;
    }
    let r = Rect::new(tx.max(2), ty.max(2), tw.ceil() as i32, th.ceil() as i32);
    p.panel(r, rgba(12, 18, 30, 250), rgb(110, 140, 196), 3.0);
    let mut ly = r.y as f32 + 3.0;
    for (t, col, size) in lines {
        p.text(r.x as f32 + 5.0, ly, &t, size, col);
        ly += if size == BODY { 10.0 } else { 8.0 };
    }
}

/// The topmost enabled clickable widget under `pos`.
pub fn hit<A: Clone>(widgets: &[Widget<A>], pos: (i32, i32)) -> Option<A> {
    widgets.iter().rev().find(|w| w.enabled && w.action.is_some() && w.rect.contains(pos.0, pos.1)).and_then(|w| w.action.clone())
}

/// The action of widget `id`, or why it can't be clicked.
pub fn by_id<A: Clone>(widgets: &[Widget<A>], id: &str) -> Result<A, String> {
    match widgets.iter().find(|w| w.id == id) {
        Some(w) if w.enabled && w.action.is_some() => Ok(w.action.clone().unwrap()),
        Some(_) => Err(format!("widget '{id}' is disabled or not clickable")),
        None => Err(format!("no widget '{id}'")),
    }
}
