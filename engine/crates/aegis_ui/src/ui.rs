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
    /// Drag and drop (UIKIT2): the payload this widget carries when dragged (`Ui::draggable`), and the key it
    /// accepts drops under (`Ui::drop_target`); both are the screen's own strings, e.g. "stash.3", "slot.weapon".
    pub drag: Option<String>,
    pub drop: Option<String>,
}

impl<A> Widget<A> {
    pub fn info(&self) -> Value {
        let mut info = json!({"id": self.id, "kind": self.kind, "text": self.text, "rect": [self.rect.x, self.rect.y, self.rect.w, self.rect.h],
            "enabled": self.enabled, "active": self.active, "clickable": self.action.is_some() || self.kind == "select", "tip": self.tip.iter().map(|t| t.0.clone()).collect::<Vec<_>>()});
        // Drag and drop keys only where set, so ui_tree stays small.
        for (k, v) in [("drag", &self.drag), ("drop", &self.drop)] {
            if let Some(v) = v {
                info[k] = json!(v);
            }
        }
        info
    }
}

/// Widget state that outlives a frame, kept by the shell: the select whose choice list is open, the drag in
/// progress, the offset of every scroll area (by id; cleared when a new stack starts, Nav::Reset) and the keyboard focus.
#[derive(Debug, Clone, Default)]
pub struct Kit {
    pub open: Option<String>,
    pub drag: Option<Drag>,
    pub scroll: std::collections::HashMap<String, i32>,
    /// The focused widget (PORT-FOCUS), and whether its ring shows: the keyboard moved the focus since the last click.
    pub focus: Option<String>,
    pub focus_visible: bool,
    /// The text field being typed into (its text and caret live here while it edits).
    pub edit: Option<Edit>,
}

/// A text field being edited: its id, the text so far, the caret (a character index) and whether Enter adds a line.
/// The shell types into it and hands every change to the screen (`Screen::text_changed`), which owns the value.
#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    pub id: String,
    pub text: String,
    pub caret: usize,
    pub multiline: bool,
}

impl Edit {
    /// Editing `text` with the caret at its end.
    pub fn new(id: &str, text: &str, multiline: bool) -> Self {
        Self { id: id.into(), text: text.into(), caret: text.chars().count(), multiline }
    }

    fn byte_at(&self, caret: usize) -> usize {
        self.text.char_indices().nth(caret).map_or(self.text.len(), |(i, _)| i)
    }

    /// Inserts typed text at the caret (control characters dropped; a newline only in a multiline field).
    pub fn insert(&mut self, s: &str) {
        for ch in s.chars().filter(|c| !c.is_control() || (*c == '\n' && self.multiline)) {
            let at = self.byte_at(self.caret);
            self.text.insert(at, ch);
            self.caret += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.caret > 0 {
            let at = self.byte_at(self.caret - 1);
            self.text.remove(at);
            self.caret -= 1;
        }
    }

    pub fn delete(&mut self) {
        if self.caret < self.text.chars().count() {
            let at = self.byte_at(self.caret);
            self.text.remove(at);
        }
    }

    pub fn left(&mut self) {
        self.caret = self.caret.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.caret = (self.caret + 1).min(self.text.chars().count());
    }

    pub fn home(&mut self) {
        self.caret = 0;
    }

    pub fn end(&mut self) {
        self.caret = self.text.chars().count();
    }
}

/// Pixels one wheel line scrolls a scroll area.
pub const SCROLL_STEP: i32 = 12;

/// A drag in progress (UIKIT2): the dragged widget's payload, id and text (the ghost under the cursor).
#[derive(Debug, Clone)]
pub struct Drag {
    pub key: String,
    pub id: String,
    pub text: String,
}

/// The drawing surface the shell lends a screen for one frame.
pub struct Gfx {
    pub canvas: Canvas,
    pub view: View,
    pub fonts: Fonts,
    pub kit: Kit,
}

pub struct Ui<'a, A> {
    pub p: Painter<'a>,
    pub widgets: Vec<Widget<A>>,
    pub mouse: (i32, i32),
    /// Mouse position for hover effects; (-1, -1) while a screen is drawn under an overlay.
    pub live: bool,
    /// The select whose list is open (`Kit::open`, read at the start of the frame).
    open: Option<String>,
    /// The drag in progress (`Kit::drag`, read at the start of the frame).
    drag: Option<Drag>,
    /// Widgets and drawing are kept inside this rect (a window's body, `set_clip`).
    clip: Option<Rect>,
    /// The clips `push_clip` replaced, restored by `pop_clip`.
    clips: Vec<Option<Rect>>,
    /// The scroll offsets at the start of the frame (`Kit::scroll`).
    scroll: std::collections::HashMap<String, i32>,
    /// How many scroll areas are open (`scroll_begin`), each closed by `scroll_end`.
    scrolls: usize,
    /// The text field being edited (`Kit::edit`, read at the start of the frame; only the live screen sees it).
    edit: Option<Edit>,
}

impl<'a, A: Clone> Ui<'a, A> {
    pub fn new(g: &'a mut Gfx, mouse: (i32, i32), live: bool) -> Self {
        // Only the live (top) screen sees the drag; screens under an overlay draw as if there were none.
        let (open, drag, scroll) = (g.kit.open.clone(), if live { g.kit.drag.clone() } else { None }, g.kit.scroll.clone());
        let edit = if live { g.kit.edit.clone() } else { None };
        g.canvas.clip = None;
        Ui { p: Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts }, widgets: Vec::new(), mouse, live, open, drag, clip: None, clips: Vec::new(), scroll, scrolls: 0, edit }
    }

    /// A text field (`multiline`: a text area, Enter adds a line). Clicking it starts editing: the shell types into it
    /// and hands each change to the screen's `text_changed(id, text)`; `value` is the screen's text otherwise. While it
    /// edits, a caret shows; the field scrolls to keep the caret's line in view.
    pub fn text_field(&mut self, id: &str, r: Rect, value: &str, multiline: bool, enabled: bool) {
        let editing = self.edit.as_ref().filter(|e| e.id == id).cloned();
        let hover = enabled && self.hover(r);
        let border = if editing.is_some() { GOLD } else if hover { rgb(110, 140, 196) } else if enabled { rgb(100, 130, 186) } else { BORDER };
        self.p.panel(r, rgb(22, 30, 48), border, RADIUS);
        let text = editing.as_ref().map_or(value, |e| e.text.as_str()).to_string();
        let line = self.p.line_h(BODY);
        let max_w = r.w as f32 - 10.0;
        let lines = |p: &mut Painter, s: &str| -> Vec<String> {
            let mut out = Vec::new();
            for para in s.split('\n') {
                let wrapped = p.wrap(para, BODY, max_w);
                if wrapped.is_empty() {
                    out.push(String::new());
                } else {
                    out.extend(wrapped);
                }
            }
            out
        };
        let shown = if multiline { lines(&mut self.p, &text) } else { vec![text.clone()] };
        // The caret: the line and x of the text before it.
        let caret = editing.as_ref().map(|e| {
            let before: String = e.text.chars().take(e.caret).collect();
            let ls = if multiline { lines(&mut self.p, &before) } else { vec![before] };
            let last = ls.last().cloned().unwrap_or_default();
            (ls.len().saturating_sub(1), self.p.text_w(&last, BODY))
        });
        let rows = ((r.h as f32 - 6.0) / line).floor().max(1.0) as usize;
        let first = caret.map_or(0, |(l, _)| l.saturating_sub(rows - 1));
        let col = if enabled { TEXT } else { DIM };
        let clip = Rect::new(r.x + 3, r.y + 2, r.w - 6, r.h - 4);
        self.push_clip(clip);
        for (i, l) in shown.iter().skip(first).take(rows).enumerate() {
            let y = r.y as f32 + 3.0 + i as f32 * line;
            if multiline {
                self.p.text(r.x as f32 + 5.0, y, l, BODY, col);
            } else {
                // A single line keeps its end (the caret) in view.
                let w = self.p.text_w(l, BODY);
                self.p.text(r.x as f32 + 5.0 - (w - max_w).max(0.0), y, l, BODY, col);
            }
        }
        if let Some((l, x)) = caret {
            let shift = if multiline { 0.0 } else { (self.p.text_w(&text, BODY) - max_w).max(0.0) };
            let cx = r.x as f32 + 5.0 + x - shift + 0.5;
            let cy = r.y as f32 + 3.0 + (l - first) as f32 * line;
            self.p.line(cx, cy, cx, cy + line - 1.0, GOLD, 0.0);
        }
        self.pop_clip();
        self.push(id, if multiline { "textarea" } else { "text" }, text, r, enabled, editing.is_some(), None, Vec::new());
    }

    /// Keeps the drawing and the recorded widgets inside `r` (None: the whole canvas); a widget partly outside keeps
    /// only its visible part, so it can't be clicked outside the window that shows it (UIKIT3).
    pub fn set_clip(&mut self, r: Option<Rect>) {
        self.clip = r;
        self.p.clip(r);
    }

    /// Narrows the clip to `r` (inside the current one); `pop_clip` restores the clip before it.
    pub fn push_clip(&mut self, r: Rect) {
        self.clips.push(self.clip);
        let inner = self.clip.map_or(r, |c| c.intersect(r));
        self.set_clip(Some(inner));
    }

    pub fn pop_clip(&mut self) {
        let prev = self.clips.pop().flatten();
        self.set_clip(prev);
    }

    /// Opens a scroll area (the kit keeps its offset; the wheel over it scrolls it): content `content_h` pixels tall
    /// shown in `view`, with a scrollbar on the right when it does not fit. Returns the offset to draw the content at
    /// (`view.y - offset` is the content's top); drawing and widgets are clipped to `view` until `scroll_end`.
    pub fn scroll_begin(&mut self, id: &str, view: Rect, content_h: i32) -> i32 {
        let max = (content_h - view.h).max(0);
        let at = self.scroll.get(id).copied().unwrap_or(0).clamp(0, max);
        // The area itself, under its content: the shell finds it for the wheel (`text` is the furthest offset).
        self.push(id, "scroll", max.to_string(), view, true, false, None, Vec::new());
        if max > 0 {
            let bar = Rect::new(view.x + view.w - 3, view.y, 3, view.h);
            self.p.fill(bar, rgba(40, 46, 60, 200));
            let knob_h = (view.h * view.h / content_h.max(1)).max(8);
            let knob_y = view.y + (view.h - knob_h) * at / max;
            self.p.fill(Rect::new(bar.x, knob_y, 3, knob_h), rgb(150, 160, 180));
        }
        self.push_clip(view);
        self.scrolls += 1;
        at
    }

    /// Closes the scroll area `scroll_begin` opened (the clip before it comes back).
    pub fn scroll_end(&mut self) {
        if self.scrolls > 0 {
            self.scrolls -= 1;
            self.pop_clip();
        }
    }

    /// Makes the last recorded widget draggable with payload `key` (UIKIT2): press and move picks it up, a press
    /// without a move stays a click. The widget being dragged is drawn dimmed.
    pub fn draggable(&mut self, key: impl Into<String>) {
        let Some(w) = self.widgets.last_mut() else { return };
        w.drag = Some(key.into());
        let r = w.rect;
        if self.drag.as_ref().is_some_and(|d| d.id == w.id) {
            self.p.fill(r, rgba(10, 14, 24, 150));
        }
    }

    /// Makes the last recorded widget accept drops under `key`; the screen's `can_drop` and `accept_drop` decide.
    pub fn drop_target(&mut self, key: impl Into<String>) {
        if let Some(w) = self.widgets.last_mut() {
            w.drop = Some(key.into());
        }
    }

    /// The payload being dragged, if any (to draw previews, e.g. the stat change over a hovered slot).
    pub fn dragging(&self) -> Option<&str> {
        self.drag.as_ref().map(|d| d.key.as_str())
    }

    /// Records a widget; an info row (a hover card but no action) gets a hover outline, so hovering shows it responds.
    #[allow(clippy::too_many_arguments)]
    pub fn push(&mut self, id: impl Into<String>, kind: &'static str, text: impl Into<String>, rect: Rect, enabled: bool, active: bool, action: Option<A>, tip: Tip) {
        if kind == "row" && action.is_none() && tip.len() > 1 && self.hover(rect) {
            self.p.panel(rect, rgba(0, 0, 0, 0), rgb(110, 140, 196), RADIUS);
        }
        let rect = match self.clip {
            // An open select's options are drawn over everything by the shell: not cut to a window body.
            Some(c) if kind != "option" => rect.intersect(c),
            _ => rect,
        };
        self.widgets.push(Widget { id: id.into(), kind, text: text.into(), rect, enabled, active, action, tip, drag: None, drop: None });
    }

    /// The mouse is over `r` (and inside the clip, so a widget scrolled out of its window shows no hover).
    pub fn hover(&self, r: Rect) -> bool {
        self.live && r.contains(self.mouse.0, self.mouse.1) && self.clip.is_none_or(|c| c.contains(self.mouse.0, self.mouse.1))
    }

    /// A plain button; `active` draws it selected (gold).
    #[allow(clippy::too_many_arguments)]
    pub fn button_full(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, active: bool, size: f32, t: Tip) {
        let hover = enabled && self.hover(r);
        let fill = if active { SELECTED } else if hover { PANEL_HI } else { CARD };
        let border = if active || (hover && enabled) { GOLD } else if enabled { rgb(100, 130, 186) } else { BORDER };
        self.p.panel(r, fill, border, RADIUS);
        let col = if enabled { if active { GOLD } else { TEXT } } else { DIM };
        let line = self.p.line_h(size);
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

    /// A value bar with - / + steps (`slider` drags; this one stays for fine steps).
    #[allow(clippy::too_many_arguments)]
    pub fn stepper(&mut self, id: &str, r: Rect, value: f32, text: &str, minus: A, plus: A, enabled: bool) {
        self.button(&format!("{id}.minus"), Rect::new(r.x, r.y, 12, r.h), "-", minus, enabled);
        let bar = Rect::new(r.x + 15, r.y + r.h / 2 - 2, r.w - 30 - 44, 4);
        self.p.panel(bar, rgb(40, 46, 60), rgb(40, 46, 60), 2.0);
        let fill = Rect::new(bar.x, bar.y, ((bar.w as f32) * value.clamp(0.0, 1.0)) as i32, bar.h);
        self.p.panel(fill, rgb(200, 204, 214), rgb(200, 204, 214), 2.0);
        self.p.circle((fill.x + fill.w) as f32, bar.y as f32 + 2.0, 3.0, rgb(240, 242, 248));
        self.button(&format!("{id}.plus"), Rect::new(bar.x + bar.w + 3, r.y, 12, r.h), "+", plus, enabled);
        self.p.text(bar.x as f32 + bar.w as f32 + 19.0, r.y as f32 + (r.h as f32 - self.p.line_h(BODY)) / 2.0, text, BODY, DIM);
        self.push(id, "slider", text, r, enabled, false, None, Vec::new());
    }

    /// A dropdown field: the current value and a chevron; the action opens the choice list.
    pub fn dropdown(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, t: Tip) {
        let hover = enabled && self.hover(r);
        self.p.panel(r, if hover { PANEL_HI } else { rgb(30, 44, 74) }, if hover { GOLD } else { rgb(100, 130, 186) }, RADIUS);
        let col = if enabled { TEXT } else { DIM };
        self.p.text_fit(r.x as f32 + 5.0, r.y as f32 + (r.h as f32 - self.p.line_h(BODY)) / 2.0, label, BODY, r.w as f32 - 16.0, col);
        let (cx, cy) = (r.x as f32 + r.w as f32 - 7.0, r.y as f32 + r.h as f32 / 2.0);
        self.p.line(cx - 2.5, cy - 1.2, cx, cy + 1.3, col, 0.0);
        self.p.line(cx, cy + 1.3, cx + 2.5, cy - 1.2, col, 0.0);
        self.push(id, "dropdown", label, r, enabled, false, Some(action), t);
    }

    /// A dropdown with its choice list: a click opens the list under it (above when there is no room), a pick sends
    /// `pick(index)`; the shell opens and closes the list and draws it over the screen (`kind` "select" / "option").
    #[allow(clippy::too_many_arguments)]
    pub fn select(&mut self, id: &str, r: Rect, options: &[String], selected: usize, pick: impl Fn(usize) -> A, enabled: bool, t: Tip) {
        let label = options.get(selected).cloned().unwrap_or_default();
        let open = enabled && self.open.as_deref() == Some(id);
        let hover = enabled && (open || self.hover(r));
        self.p.panel(r, if hover { PANEL_HI } else { rgb(30, 44, 74) }, if hover { GOLD } else { rgb(100, 130, 186) }, RADIUS);
        let col = if enabled { TEXT } else { DIM };
        self.p.text_fit(r.x as f32 + 5.0, r.y as f32 + (r.h as f32 - self.p.line_h(BODY)) / 2.0, &label, BODY, r.w as f32 - 16.0, col);
        let (cx, cy, d) = (r.x as f32 + r.w as f32 - 7.0, r.y as f32 + r.h as f32 / 2.0, if open { -1.0 } else { 1.0 });
        self.p.line(cx - 2.5, cy - 1.2 * d, cx, cy + 1.3 * d, col, 0.0);
        self.p.line(cx, cy + 1.3 * d, cx + 2.5, cy - 1.2 * d, col, 0.0);
        self.push(id, "select", label, r, enabled, open, None, t);
        if open {
            // Below the field, else above it; a list taller than the room flows into columns (shifted left to fit).
            let (h, n) = (r.h.max(14), options.len().max(1) as i32);
            let (below, above) = (H - 2 - (r.y + r.h + 1), r.y - 3);
            let per_col = (below.max(above) / h).clamp(1, n);
            let cols = (n + per_col - 1) / per_col;
            let w = r.w.min((W - 4) / cols);
            let y0 = if per_col * h <= below { r.y + r.h + 1 } else { r.y - per_col * h - 1 };
            let x0 = r.x.min(W - 2 - cols * w).max(2);
            for (i, o) in options.iter().enumerate() {
                let (col, row) = (i as i32 / per_col, i as i32 % per_col);
                self.push(format!("{id}.opt.{i}"), "option", o.clone(), Rect::new(x0 + col * w, y0 + row * h, w, h), true, i == selected, Some(pick(i)), Vec::new());
            }
        }
    }

    /// A slider over `steps` values (index 0..steps): click the track or drag along it (the shell drags through the
    /// per-step widgets, `kind` "slider_step"); `text` is shown to the right.
    #[allow(clippy::too_many_arguments)]
    pub fn slider(&mut self, id: &str, r: Rect, at: usize, steps: usize, text: &str, set: impl Fn(usize) -> A, enabled: bool) {
        let steps = steps.max(2);
        let area = Rect::new(r.x, r.y, r.w - 44, r.h);
        let bar = Rect::new(area.x + 4, r.y + r.h / 2 - 2, area.w - 8, 4);
        let hover = enabled && self.hover(area);
        let knob = bar.x as f32 + bar.w as f32 * at.min(steps - 1) as f32 / (steps - 1) as f32;
        self.p.panel(bar, rgb(40, 46, 60), rgb(40, 46, 60), 2.0);
        self.p.panel(Rect::new(bar.x, bar.y, (knob as i32 - bar.x).max(0), bar.h), rgb(200, 204, 214), rgb(200, 204, 214), 2.0);
        self.p.circle(knob, bar.y as f32 + 2.0, if hover { 4.0 } else { 3.0 }, if hover { GOLD } else if enabled { rgb(240, 242, 248) } else { DIM });
        self.p.text(area.x as f32 + area.w as f32 + 6.0, r.y as f32 + (r.h as f32 - self.p.line_h(BODY)) / 2.0, text, BODY, DIM);
        self.push(id, "slider", text, area, enabled, false, None, Vec::new());
        let seg = bar.w as f32 / (steps - 1) as f32;
        for i in 0..steps {
            let left = if i == 0 { area.x as f32 } else { bar.x as f32 + seg * (i as f32 - 0.5) };
            let right = if i == steps - 1 { (area.x + area.w) as f32 } else { bar.x as f32 + seg * (i as f32 + 0.5) };
            let rect = Rect::new(left.round() as i32, r.y, (right - left).round().max(1.0) as i32, r.h);
            self.push(format!("{id}.step.{i}"), "slider_step", i.to_string(), rect, enabled, i == at, Some(set(i)), Vec::new());
        }
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
        self.p.text_fit(r.x as f32 + 4.0, r.y as f32 + (r.h as f32 - self.p.line_h(BODY)) / 2.0, label, BODY, r.w as f32 - 8.0, col);
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

/// The topmost enabled clickable widget under `pos`; an open select's options lie over everything.
pub fn hit<A: Clone>(widgets: &[Widget<A>], pos: (i32, i32)) -> Option<A> {
    let under = |w: &&Widget<A>| w.enabled && w.action.is_some() && w.rect.contains(pos.0, pos.1);
    widgets.iter().rev().filter(|w| w.kind == "option").find(under).or_else(|| widgets.iter().rev().find(under)).and_then(|w| w.action.clone())
}

/// The action of widget `id`, or why it can't be clicked.
pub fn by_id<A: Clone>(widgets: &[Widget<A>], id: &str) -> Result<A, String> {
    match widgets.iter().find(|w| w.id == id) {
        Some(w) if w.enabled && w.action.is_some() => Ok(w.action.clone().unwrap()),
        Some(_) => Err(format!("widget '{id}' is disabled or not clickable")),
        None => Err(format!("no widget '{id}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::Edit;

    #[test]
    fn an_edit_types_deletes_and_moves_the_caret_by_characters() {
        let mut e = Edit::new("t", "héllo", false);
        assert_eq!(e.caret, 5);
        e.insert("!");
        e.left();
        e.left();
        e.backspace();
        assert_eq!((e.text.as_str(), e.caret), ("hélo!", 3), "keeps the multibyte é");
        e.home();
        e.delete();
        e.end();
        e.insert("\n x\t");
        assert_eq!(e.text, "élo! x", "a single-line field drops the newline and the tab");
        let mut m = Edit::new("m", "a", true);
        m.insert("\nb");
        assert_eq!((m.text.as_str(), m.caret), ("a\nb", 3));
    }
}
