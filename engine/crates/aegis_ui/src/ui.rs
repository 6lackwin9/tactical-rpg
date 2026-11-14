//! The widget kit every screen draws with: immediate-mode widgets on the logical 640x360 canvas, each recorded with an
//! id, kind, text, rect, enabled / active state, an optional action and a hover card, so the agent link can list,
//! hover and click exactly what the player sees. Colours follow the Godot HUD (dark navy, gold accents). LOOK1-E1: the
//! widgets draw through the runtime theme (`theme::th()`: colour roles, layout values, frame roles, the display face
//! for frame titles); the colour constants below are its defaults (they stay until the game reads the theme, LOOK1-G2).
//! LAYOUT720-E3: a widget's inside is worked out from its own rect, in the units of the screen's grid (`Ui::m`), so a
//! widget draws right on every grid; the numbers it still writes are old units through `Metrics::u` and `uf`. Every picture
//! on `Grid::Old` stays as it was.

use crate::canvas::{rgb, rgba, Canvas, Image, Rect, Rgba};
use crate::layout_audit::LayoutRec;
use crate::painter::{Fonts, Grid, Painter, Typeface, View};
use crate::theme::{self, Breakpoints, Frame, Metrics, Theme, CORNER_SIZE};
use aegis_layout::{frac, Anchor, Insets, SizeClass};
use serde_json::{json, Value};
use std::rc::Rc;

/// The old canvas in old units. For the numbers of a screen on `Grid::Old`; widgets read the view's canvas.
pub const W: i32 = 640;
pub const H: i32 = 360;
/// Text sizes and the line step in old units. For a screen on `Grid::Old`; widgets read `Ui::m`.
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
    let th = theme::th();
    let mut t = vec![(title.to_string(), th.gold)];
    if !body.is_empty() {
        t.push((body.to_string(), th.text));
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
    /// What the shell knows of the device (`Ui::env`).
    pub device: Device,
}

/// What points at the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    Mouse,
    /// No shell produces it yet (LAYOUT720 section 11).
    Touch,
}

/// The device under the canvas, as the shell knows it: part of `Env`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Device {
    /// Window px per CSS px (the window system's scale factor). Not a true measurement: a handheld can report 1.
    pub dpr: f32,
    /// No setting yet: 1 in a window; the layout audit's profiles set others (`layout_audit::PROFILES`).
    pub ui_scale: f32,
    pub pointer: Pointer,
}

impl Default for Device {
    fn default() -> Self {
        Self { dpr: 1.0, ui_scale: 1.0, pointer: Pointer::Mouse }
    }
}

/// What a screen may ask of where it runs (`Ui::env`; LAYOUT720 6.4). It comes in as values: no file, thread or window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Env {
    /// The canvas in the screen's units.
    pub canvas: (i32, i32),
    pub dpr: f32,
    pub ui_scale: f32,
    pub pointer: Pointer,
    pub width_class: SizeClass,
    pub height_class: SizeClass,
}

/// An axis of `units` against `[compact, medium]`.
fn size_class(units: i32, [compact, medium]: [i32; 2]) -> SizeClass {
    if units < compact {
        SizeClass::Compact
    } else if units < medium {
        SizeClass::Medium
    } else {
        SizeClass::Expanded
    }
}

/// The width and height classes of `canvas` in a view of `g` units per old unit. They read the canvas in grid units, so
/// an old-numbers screen (`g` 1) of a window gets the same ones as a native screen of it.
pub(crate) fn size_classes(canvas: (i32, i32), g: i32, limits: Breakpoints) -> (SizeClass, SizeClass) {
    let per = if g <= 1 { Grid::Native.g() } else { 1 };
    (size_class(canvas.0 * per, limits.width), size_class(canvas.1 * per, limits.height))
}

impl Kit {
    /// The mouse moved to `mouse` over a screen's widgets `infos`: a `text_button` under it takes the visible focus
    /// (`focus::hover_target`). The shell calls it for every mouse move.
    pub fn mouse_moved(&mut self, infos: &[Value], mouse: (i32, i32)) {
        if self.open.is_none() && self.edit.is_none() {
            if let Some(id) = crate::focus::hover_target(infos, mouse) {
                self.focus = Some(id);
                self.focus_visible = true;
            }
        }
    }
}

/// The text editor's model, layout, keys and clipboard live in `edit` (PLAYUI1-T); re-exported here for the kit's
/// callers.
pub use crate::edit::{clipboard, field_content_h, field_layout, field_scroll_id, field_shift, reveal_offset, word_end, word_start, Edit, EditKey, Layout, VLine, FIELD_PAD_X, FIELD_PAD_Y, UNDO_MAX, UNDO_PAUSE_MS};

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
    edit: Option<EditView>,
    /// The theme in force when the frame began (`theme::th()`).
    pub t: Rc<Theme>,
    /// The widget with a visible keyboard focus (`Kit::focus`, read at the start of the frame; only the live screen).
    focus: Option<String>,
    /// The device (`Kit::device`, read at the start of the frame).
    device: Device,
    /// One corner size for what this frame still draws (`Ui::corner_size`).
    corner: Option<f32>,
}

/// What a frame draws of the text field being edited: its id, text, caret and selection anchor (not its history).
#[derive(Debug, Clone)]
struct EditView {
    id: String,
    text: String,
    caret: usize,
    anchor: Option<usize>,
}

impl<'a, A: Clone> Ui<'a, A> {
    pub fn new(g: &'a mut Gfx, mouse: (i32, i32), live: bool) -> Self {
        // Only the live (top) screen sees the drag; screens under an overlay draw as if there were none.
        let (open, drag, scroll) = (g.kit.open.clone(), if live { g.kit.drag.clone() } else { None }, g.kit.scroll.clone());
        let edit = g.kit.edit.as_ref().filter(|_| live).map(|e| EditView { id: e.id.clone(), text: e.text.clone(), caret: e.caret, anchor: e.anchor });
        let focus = g.kit.focus.clone().filter(|_| live && g.kit.focus_visible);
        let device = g.kit.device;
        g.canvas.clip = None;
        let t = theme::th();
        g.fonts.sync(&t);
        Ui { p: Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts }, widgets: Vec::new(), mouse, live, open, drag, clip: None, clips: Vec::new(), scroll, scrolls: 0, edit, t, focus, device, corner: None }
    }

    /// The layout's tokens for this screen's grid (LAYOUT720 6.2, 14): text sizes, gaps, small control heights and `u`, the
    /// one read path for them.
    pub fn m(&self) -> Metrics {
        Metrics::of(&self.t, self.p.v)
    }

    /// The canvas as a rect, in this screen's units: where a screen's layout starts. Read it each frame and assume no aspect.
    pub fn screen(&self) -> Rect {
        let (w, h) = self.p.v.canvas();
        Rect::new(0, 0, w, h)
    }

    /// What a screen may ask of where it runs. The classes read the canvas in grid units, so every grid of one window gets
    /// the same ones.
    pub fn env(&self) -> Env {
        let (m, canvas) = (self.m(), self.p.v.canvas());
        let (width_class, height_class) = size_classes(canvas, m.g, m.breakpoints);
        Env { canvas, dpr: self.device.dpr, ui_scale: self.device.ui_scale, pointer: self.device.pointer, width_class, height_class }
    }

    /// An image in the rect the layout gives. It draws as `Painter::image` does; the per-image rule attaches here later
    /// (LAYOUT720 11.1).
    pub fn image_in(&mut self, img: &Image, r: Rect) {
        self.p.note_layout(|| LayoutRec::Placed { what: "image".into(), at: r });
        self.p.image(img, r);
    }

    /// Tells the layout audit that what `what` lays out in `area` needs `by` units more (`Stack::overflow`, a list's
    /// height past its area). `by` of 0 or less is nothing, so a list can report each frame. Clipping what is drawn past
    /// the area stays the screen's.
    pub fn report_overflow(&mut self, what: &str, area: Rect, by: i32) {
        if by > 0 {
            self.p.note_layout(|| LayoutRec::Overflow { what: what.into(), at: area, by });
        }
    }

    /// One corner size for every frame this `Ui` still draws (LAYOUT720 section 9): cut every pane first, then call this with
    /// the size the smallest pane allows, then draw, so corners match where panes meet. The size is against the art, as the
    /// sidecar's `corner_size`, which it replaces. Outside `CORNER_SIZE`, or not a number, is refused: the corners stay as
    /// they were. A panel too small for two corners still shrinks them, so that they never overlap.
    pub fn corner_size(&mut self, size: f32) {
        if (CORNER_SIZE.0..=CORNER_SIZE.1).contains(&size) {
            self.corner = Some(size);
        }
    }

    /// `frame` as this `Ui` draws it: under `corner_size`, its corners at that size. None: as the sidecar says.
    pub(crate) fn cornered(&self, frame: &Frame) -> Option<Frame> {
        self.corner.map(|size| {
            let mut f = frame.clone();
            f.nine.corner_size = size;
            f
        })
    }

    /// Frame role `role` over `r`, under `corner_size`.
    fn draw_role(&mut self, t: &Theme, role: &str, r: Rect, fill: Rgba, edge: Rgba, radius: f32) {
        let frame = t.frame(role);
        let own = frame.and_then(|f| self.cornered(f));
        let before = self.p.fonts.layout.as_ref().map_or(0, Vec::len);
        self.p.skin(own.as_ref().or(frame), r, fill, edge, radius);
        // What the painter recorded for this frame takes the role it could not know.
        for rec in self.p.fonts.layout.iter_mut().flat_map(|recs| recs[before..].iter_mut()) {
            if let LayoutRec::CornerShrunk { what, .. } = rec {
                *what = role.to_string();
            }
        }
    }

    /// Draws frame role `role` (`theme::FRAME_ROLES`) over `r` in its own colours and returns its content rect: past the
    /// frame's pad and corners at the size they draw, else `Metrics::padding` inside. Content reads no slice number.
    pub fn frame_body(&mut self, role: &str, r: Rect) -> Rect {
        let (t, m) = (self.t.clone(), self.m());
        let (fill, edge, radius) = t.vector_in(role, &m);
        self.p.note_layout(|| LayoutRec::Placed { what: role.into(), at: r });
        self.draw_role(&t, role, r, fill, edge, radius);
        let body = self.frame_content(role, r);
        if r.w > 0 && r.h > 0 && (body.w <= 0 || body.h <= 0) {
            self.p.note_layout(|| LayoutRec::Placed { what: format!("{role} body"), at: body });
        }
        body
    }

    /// The content rect `frame_body` returns for `r`, with nothing drawn: what a panel sized by its content measures first.
    pub fn frame_content(&self, role: &str, r: Rect) -> Rect {
        match self.t.frame(role) {
            Some(f) => self.p.v.nine_content(&self.cornered(f).as_ref().unwrap_or(f).nine, r),
            None => r.pad(Insets::all(self.m().padding)),
        }
    }

    /// Whether any widget shows the keyboard focus this frame.
    pub fn focus_shown(&self) -> bool {
        self.focus.is_some()
    }

    /// Whether widget `id` shows the keyboard focus this frame (the shell's ring goes around it unless it draws its own:
    /// `focus::shows_ring`).
    pub fn focused(&self, id: &str) -> bool {
        self.focus.as_deref() == Some(id)
    }

    /// LOOK1-E1: draws frame role `role` (`theme::FRAME_ROLES`) over `r`: its frame file, else the vector panel
    /// `fill` / `edge` / `radius` exactly as the widget drew it before the theme.
    pub fn skin(&mut self, role: &str, r: Rect, fill: Rgba, edge: Rgba, radius: f32) {
        let t = self.t.clone();
        self.draw_role(&t, role, r, fill, edge, radius);
    }

    /// LOOK1-E1: a header in the display face (`Metrics::head`, colour role `gold`) at (x, y); returns the y below it.
    pub fn header(&mut self, x: i32, y: i32, text: &str) -> i32 {
        let (col, head) = (self.t.gold, self.m().head);
        self.p.text_face(Typeface::Display, x as f32, y as f32, text, head, col);
        y + self.p.line_h_face(Typeface::Display, head).ceil() as i32
    }

    /// LOOK1-E1: a badge (numbers on a count or a cost): `text` in the mono face centred on a fill in colour role
    /// `role` (unknown: `card`), the text in whichever of `text` and `ink` stands out more on it. Draws only.
    pub fn badge(&mut self, r: Rect, text: &str, role: &str) {
        let (t, m) = (self.t.clone(), self.m());
        let fill = t.role(role).unwrap_or(t.card);
        let ink = if t.contrast(t.text, fill) >= t.contrast(t.ink, fill) { t.text } else { t.ink };
        self.p.panel(r, fill, fill, m.radius);
        let lh = self.p.line_h_face(Typeface::Mono, m.small);
        self.p.text_center_face(Typeface::Mono, r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - lh) / 2.0, text, m.small, ink);
    }

    /// LOOK1-E1: a section inside a panel (frame role `inset`, or `parchment` when `parchment`; its text colour is
    /// `Theme::ink_on(role)`); returns the content rect (`frame_body`).
    pub fn inset(&mut self, r: Rect, parchment: bool) -> Rect {
        self.frame_body(if parchment { "parchment" } else { "inset" }, r)
    }

    /// A text field (`multiline`: a text area, Enter adds a line). Clicking it starts editing (the caret where the click
    /// was): the shell types into it and hands each change to the screen's `text_changed(id, text)`; `value` is the
    /// screen's text otherwise. While it edits, the caret and the selection show. A text area wraps at spaces and
    /// scrolls (the wheel, a scroll bar when the text is taller; the shell keeps the caret's line in view); a one-line
    /// field slides to keep the caret in view (PLAYUI1-T).
    pub fn text_field(&mut self, id: &str, r: Rect, value: &str, multiline: bool, enabled: bool) {
        let (t, m) = (self.t.clone(), self.m());
        let editing = self.edit.as_ref().filter(|e| e.id == id).cloned();
        let hover = enabled && self.hover(r);
        let border = if editing.is_some() { t.focus } else if hover { t.edge_hot } else if enabled { t.edge_live } else { t.edge };
        self.p.panel(r, t.input, border, m.radius);
        let text = editing.as_ref().map_or(value, |e| e.text.as_str()).to_string();
        let line = self.p.line_h(m.body);
        let lay = field_layout(&mut self.p, &text, r, multiline);
        let chars: Vec<char> = text.chars().collect();
        let caret = editing.as_ref().map_or(chars.len(), |e| e.caret.min(chars.len()));
        let selection = editing.as_ref().and_then(|e| e.anchor.map(|a| a.min(chars.len()))).filter(|a| *a != caret).map(|a| (a.min(caret), a.max(caret)));
        // Where the text's top left is drawn: a text area scrolls down by its offset, a one-line field slides left.
        let (dx, off) = if multiline {
            let off = self.scroll_begin(&field_scroll_id(id), r, field_content_h(lay.lines.len(), line, m.g));
            (0.0, off)
        } else {
            (field_shift(&lay, caret, r, m.g), 0)
        };
        let (tx, ty) = (r.x as f32 + m.uf(FIELD_PAD_X) - dx, r.y as f32 + m.uf(FIELD_PAD_Y) - off as f32);
        let col = if enabled { t.text } else { t.dim };
        self.push_clip(r.pad(Insets::xy(m.u(3), m.u(2))));
        for (i, l) in lay.lines.iter().enumerate() {
            let y = ty + i as f32 * line;
            if y + line < r.y as f32 || y > (r.y + r.h) as f32 {
                continue;
            }
            if let Some((a, b)) = selection {
                let (lo, hi) = (a.max(l.start), b.min(l.end));
                // A selection that goes on past the line's end also marks the break.
                let tail = if b > l.end && a <= l.end { m.uf(3.0) } else { 0.0 };
                if lo < hi || tail > 0.0 {
                    let (x0, x1) = (l.xs[lo - l.start], l.xs[hi - l.start] + tail);
                    self.p.fill(Rect::new((tx + x0).floor() as i32, y.floor() as i32, ((x1 - x0).ceil() as i32).max(m.u(1)), line.ceil() as i32), t.selection);
                }
            }
            let s: String = chars[l.start..l.end].iter().collect();
            self.p.text(tx, y, &s, m.body, col);
        }
        if editing.is_some() {
            let cx = tx + lay.x_of(caret) + m.uf(0.5);
            let cy = ty + lay.line_of(caret) as f32 * line;
            self.p.line(cx, cy, cx, cy + line - m.uf(1.0), t.focus, 0.0);
        }
        self.pop_clip();
        if multiline {
            self.scroll_end();
        }
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
            let (track, knob, m) = (self.t.scroll_track, self.t.scroll_knob, self.m());
            let bar = view.place(Anchor::Right, m.u(3), frac(1000));
            self.p.fill(bar, track);
            let knob_h = (view.h * view.h / content_h.max(1)).max(m.u(8));
            let knob_y = view.y + (view.h - knob_h) * at / max;
            self.p.fill(Rect::new(bar.x, knob_y, bar.w, knob_h), knob);
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
            self.p.fill(r, self.t.drag_shade);
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

    /// PR5-48: a free-form drag handle (a layout editor's marker or point): a press starts a pointer drag the screen follows
    /// itself (`Screen::pointer`: down, moves, up), with no ghost and no drop targets; `action` is what a click by id does
    /// (the agent's `click`, a screen that keeps the default `pointer`).
    pub fn handle(&mut self, id: &str, r: Rect, text: &str, action: Option<A>, t: Tip) {
        self.push(id, "handle", text, r, true, false, action, t);
    }

    /// Records a widget; an info row (a hover card but no action) gets a hover outline, so hovering shows it responds.
    #[allow(clippy::too_many_arguments)]
    pub fn push(&mut self, id: impl Into<String>, kind: &'static str, text: impl Into<String>, rect: Rect, enabled: bool, active: bool, action: Option<A>, tip: Tip) {
        if kind == "row" && action.is_none() && tip.len() > 1 && self.hover(rect) {
            let (edge, radius) = (self.t.edge_hot, self.m().radius);
            self.p.panel(rect, rgba(0, 0, 0, 0), edge, radius);
        }
        let id = id.into();
        if rect.w <= 0 || rect.h <= 0 {
            self.p.note_layout(|| LayoutRec::NoRoom { what: id.clone(), at: rect });
        }
        let rect = match self.clip {
            // An open select's options are drawn over everything by the shell: not cut to a window body.
            Some(c) if kind != "option" => rect.intersect(c),
            _ => rect,
        };
        self.widgets.push(Widget { id, kind, text: text.into(), rect, enabled, active, action, tip, drag: None, drop: None });
    }

    /// The mouse is over `r` (and inside the clip, so a widget scrolled out of its window shows no hover).
    pub fn hover(&self, r: Rect) -> bool {
        self.live && r.contains(self.mouse.0, self.mouse.1) && self.clip.is_none_or(|c| c.contains(self.mouse.0, self.mouse.1))
    }

    /// A plain button; `active` draws it selected (gold).
    #[allow(clippy::too_many_arguments)]
    pub fn button_full(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, active: bool, size: f32, t: Tip) {
        self.button_in(id, r, label, action, enabled, active, size, t, false);
    }

    /// `button_full` with the button's or (`tab`) the tab's frame roles; `Metrics::focus_glow` rings a hovered or
    /// active one in `focus`.
    #[allow(clippy::too_many_arguments)]
    fn button_in(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, active: bool, size: f32, t: Tip, tab: bool) {
        let (th, m) = (self.t.clone(), self.m());
        let hover = enabled && self.hover(r);
        let fill = if active { th.selected } else if hover { th.card_hot } else { th.card };
        let border = if active || (hover && enabled) { th.focus } else if enabled { th.edge_live } else { th.edge };
        let role = match (tab, enabled, active, hover) {
            (true, _, true, _) => "tab_on",
            (true, ..) => "tab",
            (false, false, ..) => "button_off",
            (false, true, true, _) => "button_down",
            (false, true, false, true) => "button_hot",
            (false, true, false, false) => "button",
        };
        let glow = m.focus_glow;
        if glow > 0 && enabled && (hover || active) {
            self.p.panel(r.pad(Insets::all(-glow)), rgba(0, 0, 0, 0), th.focus, m.radius + glow as f32);
        }
        self.draw_role(&th, role, r, fill, border, m.radius);
        let col = if enabled { if active { th.focus } else { th.ink_on(role) } } else { th.dim };
        let line = self.p.line_h(size);
        let txt = self.p.fit(label, size, if r.w < m.u(20) { r.w as f32 } else { r.w as f32 - m.uf(6.0) });
        self.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - line) / 2.0, &txt, size, col);
        if txt != label {
            self.p.mark_cut(label);
        }
        self.push(id, "button", label, r, enabled, active, Some(action), t);
    }

    pub fn button(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool) {
        let size = self.m().body;
        self.button_full(id, r, label, action, enabled, false, size, Vec::new());
    }

    pub fn button_tip(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, t: Tip) {
        let size = self.m().body;
        self.button_full(id, r, label, action, enabled, false, size, t);
    }

    /// A tab: a button drawn selected when it is the open one (frame roles `tab`, `tab_on`).
    pub fn tab(&mut self, id: &str, r: Rect, label: &str, action: A, open: bool) {
        let size = self.m().body;
        self.button_in(id, r, label, action, true, open, size, Vec::new(), true);
    }

    /// A selectable multi-line card (world cards, slot rows, list entries).
    #[allow(clippy::too_many_arguments)]
    pub fn card(&mut self, id: &str, r: Rect, lines: &[(String, Rgba)], action: Option<A>, selected: bool, enabled: bool, t: Tip) {
        let (th, m) = (self.t.clone(), self.m());
        let hover = enabled && action.is_some() && self.hover(r);
        let fill = if selected { th.selected } else if hover { th.card_hot } else { th.card };
        let border = if selected { th.focus } else if hover { th.edge_hot } else { th.edge };
        self.draw_role(&th, if selected { "card_on" } else if hover { "card_hot" } else { "card" }, r, fill, border, m.radius);
        let mut y = r.y as f32 + m.uf(3.0);
        for (text, col) in lines {
            if y > (r.y + r.h) as f32 - m.uf(8.0) {
                break;
            }
            self.p.text_fit(r.x as f32 + m.uf(5.0), y, text, m.body, r.w as f32 - m.uf(10.0), if enabled { *col } else { th.dim });
            y += m.uf(10.0);
        }
        let text = lines.iter().map(|l| l.0.clone()).collect::<Vec<_>>().join(" | ");
        self.push(id, "card", text, r, enabled, selected, action, t);
    }

    pub fn checkbox(&mut self, id: &str, x: i32, y: i32, label: &str, checked: bool, action: A, enabled: bool) {
        let m = self.m();
        let b = Rect::new(x, y, m.u(10), m.u(10));
        let r = Rect::new(x, y - m.u(1), m.u(12) + self.p.text_w(label, m.body).ceil() as i32 + m.u(4), m.u(12));
        let t = self.t.clone();
        let hover = enabled && self.hover(r);
        self.p.panel(b, if checked { t.check_on } else { t.card }, if hover { t.focus } else { t.text }, m.uf(1.5));
        let (fx, fy) = (x as f32, y as f32);
        if checked {
            self.p.line(fx + m.uf(2.0), fy + m.uf(5.0), fx + m.uf(4.2), fy + m.uf(7.5), t.focus, 0.0);
            self.p.line(fx + m.uf(4.2), fy + m.uf(7.5), fx + m.uf(8.2), fy + m.uf(2.5), t.focus, 0.0);
        }
        self.p.text(fx + m.uf(14.0), fy + m.uf(0.5), label, m.body, if enabled { t.text } else { t.dim });
        self.push(id, "checkbox", format!("{label}: {}", if checked { "on" } else { "off" }), r, enabled, checked, Some(action), Vec::new());
    }

    /// A value bar with - / + steps (`slider` drags; this one stays for fine steps).
    #[allow(clippy::too_many_arguments)]
    pub fn stepper(&mut self, id: &str, r: Rect, value: f32, text: &str, minus: A, plus: A, enabled: bool) {
        let m = self.m();
        self.button(&format!("{id}.minus"), Rect::new(r.x, r.y, m.u(12), r.h), "-", minus, enabled);
        let t = self.t.clone();
        let bar = Rect::new(r.x + m.u(15), r.y + r.h / 2 - m.u(2), r.w - m.u(30 + 44), m.u(4));
        self.p.panel(bar, t.track, t.track, m.uf(2.0));
        let fill = Rect::new(bar.x, bar.y, ((bar.w as f32) * value.clamp(0.0, 1.0)) as i32, bar.h);
        self.p.panel(fill, t.track_fill, t.track_fill, m.uf(2.0));
        self.p.circle((fill.x + fill.w) as f32, bar.y as f32 + m.uf(2.0), m.uf(3.0), t.knob);
        self.button(&format!("{id}.plus"), Rect::new(bar.x + bar.w + m.u(3), r.y, m.u(12), r.h), "+", plus, enabled);
        self.p.text(bar.x as f32 + bar.w as f32 + m.uf(19.0), r.y as f32 + (r.h as f32 - self.p.line_h(m.body)) / 2.0, text, m.body, t.dim);
        self.push(id, "slider", text, r, enabled, false, None, Vec::new());
    }

    /// A dropdown field: the current value and a chevron; the action opens the choice list.
    pub fn dropdown(&mut self, id: &str, r: Rect, label: &str, action: A, enabled: bool, t: Tip) {
        let (th, m) = (self.t.clone(), self.m());
        let hover = enabled && self.hover(r);
        self.p.panel(r, if hover { th.card_hot } else { th.field }, if hover { th.focus } else { th.edge_live }, m.radius);
        let col = if enabled { th.text } else { th.dim };
        self.p.text_fit(r.x as f32 + m.uf(5.0), r.y as f32 + (r.h as f32 - self.p.line_h(m.body)) / 2.0, label, m.body, r.w as f32 - m.uf(16.0), col);
        self.chevron(r, 1.0, col);
        self.push(id, "dropdown", label, r, enabled, false, Some(action), t);
    }

    /// The chevron at the right of a choice field `r`: pointing down, or up when `d` is -1.
    fn chevron(&mut self, r: Rect, d: f32, col: Rgba) {
        let m = self.m();
        let (cx, cy) = (r.x as f32 + r.w as f32 - m.uf(7.0), r.y as f32 + r.h as f32 / 2.0);
        self.p.line(cx - m.uf(2.5), cy - m.uf(1.2) * d, cx, cy + m.uf(1.3) * d, col, 0.0);
        self.p.line(cx, cy + m.uf(1.3) * d, cx + m.uf(2.5), cy - m.uf(1.2) * d, col, 0.0);
    }

    /// A dropdown with its choice list: a click opens the list under it (above when there is no room), a pick sends
    /// `pick(index)`; the shell opens and closes the list and draws it over the screen (`kind` "select" / "option").
    #[allow(clippy::too_many_arguments)]
    pub fn select(&mut self, id: &str, r: Rect, options: &[String], selected: usize, pick: impl Fn(usize) -> A, enabled: bool, t: Tip) {
        let label = options.get(selected).cloned().unwrap_or_default();
        let open = enabled && self.open.as_deref() == Some(id);
        let (th, m) = (self.t.clone(), self.m());
        let hover = enabled && (open || self.hover(r));
        self.p.panel(r, if hover { th.card_hot } else { th.field }, if hover { th.focus } else { th.edge_live }, m.radius);
        let col = if enabled { th.text } else { th.dim };
        self.p.text_fit(r.x as f32 + m.uf(5.0), r.y as f32 + (r.h as f32 - self.p.line_h(m.body)) / 2.0, &label, m.body, r.w as f32 - m.uf(16.0), col);
        self.chevron(r, if open { -1.0 } else { 1.0 }, col);
        self.push(id, "select", label, r, enabled, open, None, t);
        if open {
            // Below the field, else above it; a list taller than the room flows into columns (shifted left to fit).
            let (cw, ch) = self.p.v.canvas();
            let (h, n) = (r.h.max(m.u(14)), options.len().max(1) as i32);
            let (below, above) = (ch - m.u(2) - (r.y + r.h + m.u(1)), r.y - m.u(3));
            let per_col = (below.max(above) / h).clamp(1, n);
            let cols = (n + per_col - 1) / per_col;
            let w = r.w.min((cw - m.u(4)) / cols);
            let y0 = if per_col * h <= below { r.y + r.h + m.u(1) } else { r.y - per_col * h - m.u(1) };
            let x0 = r.x.min(cw - m.u(2) - cols * w).max(m.u(2));
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
        let m = self.m();
        let area = Rect::new(r.x, r.y, r.w - m.u(44), r.h);
        let bar = Rect::new(area.x + m.u(4), r.y + r.h / 2 - m.u(2), area.w - m.u(8), m.u(4));
        let t = self.t.clone();
        let hover = enabled && self.hover(area);
        let knob = bar.x as f32 + bar.w as f32 * at.min(steps - 1) as f32 / (steps - 1) as f32;
        self.p.panel(bar, t.track, t.track, m.uf(2.0));
        self.p.panel(Rect::new(bar.x, bar.y, (knob as i32 - bar.x).max(0), bar.h), t.track_fill, t.track_fill, m.uf(2.0));
        self.p.circle(knob, bar.y as f32 + m.uf(2.0), m.uf(if hover { 4.0 } else { 3.0 }), if hover { t.focus } else if enabled { t.knob } else { t.dim });
        self.p.text(area.x as f32 + area.w as f32 + m.uf(6.0), r.y as f32 + (r.h as f32 - self.p.line_h(m.body)) / 2.0, text, m.body, t.dim);
        self.push(id, "slider", text, area, enabled, false, None, Vec::new());
        let seg = bar.w as f32 / (steps - 1) as f32;
        for i in 0..steps {
            let left = if i == 0 { area.x as f32 } else { bar.x as f32 + seg * (i as f32 - 0.5) };
            let right = if i == steps - 1 { (area.x + area.w) as f32 } else { bar.x as f32 + seg * (i as f32 + 0.5) };
            let rect = Rect::new(left.round() as i32, r.y, (right - left).round().max(m.uf(1.0)) as i32, r.h);
            self.push(format!("{id}.step.{i}"), "slider_step", i.to_string(), rect, enabled, i == at, Some(set(i)), Vec::new());
        }
    }

    /// The X close button of a frame.
    pub fn close_x(&mut self, id: &str, r: Rect, action: A) {
        let (t, m) = (self.t.clone(), self.m());
        let hover = self.hover(r);
        self.p.panel(r, if hover { t.card_hot } else { t.card }, if hover { t.focus } else { t.edge_live }, m.radius);
        let (cx, cy, d) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0, m.uf(3.0));
        self.p.line(cx - d, cy - d, cx + d, cy + d, t.text, 0.0);
        self.p.line(cx - d, cy + d, cx + d, cy - d, t.text, 0.0);
        self.push(id, "button", "Close", r, true, false, Some(action), Vec::new());
    }

    /// A modal frame with a gold title (the display face; frame role `panel`); returns the content top y
    /// (`Metrics::frame_title_h` below the top).
    pub fn frame(&mut self, r: Rect, title: &str, centered: bool) -> i32 {
        let (t, m) = (self.t.clone(), self.m());
        self.draw_role(&t, "panel", r, t.panel, t.panel_edge, m.frame_radius);
        if !title.is_empty() {
            if centered {
                self.p.text_center_face(Typeface::Display, r.x as f32 + r.w as f32 / 2.0, r.y as f32 + m.uf(6.0), title, m.head, t.gold);
            } else {
                self.p.text_face(Typeface::Display, r.x as f32 + m.uf(9.0), r.y as f32 + m.uf(6.0), title, m.head, t.gold);
            }
        }
        r.y + m.frame_title_h
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
        let m = self.m();
        let hover = clickable && self.hover(r);
        if selected || hover {
            let t = self.t.clone();
            self.p.panel(r, if selected { t.selected } else { t.card_hot }, if selected { t.focus } else { t.edge_hot }, m.radius);
        }
        self.p.text_fit(r.x as f32 + m.uf(4.0), r.y as f32 + (r.h as f32 - self.p.line_h(m.body)) / 2.0, label, m.body, r.w as f32 - m.uf(8.0), col);
        self.push(id, "row", label, r, true, selected, action, t);
    }

    /// The icon square of an icon button in hit box `r` on an unmoved screen (centred, 2 px inside the smaller side; a
    /// 16 px box holds a 12 px icon).
    pub fn icon_rect(r: Rect) -> Rect {
        Self::icon_rect_on(r, 1)
    }

    /// `icon_rect` on any grid, `g` being the view's units per old unit. It stays a sum, not `Rect::place`: a box under the
    /// least icon holds it past its edges.
    pub fn icon_rect_on(r: Rect, g: i32) -> Rect {
        let s = (r.w.min(r.h) - 4 * g).max(4 * g);
        Rect::new(r.x + (r.w - s) / 2, r.y + (r.h - s) / 2, s, s)
    }

    /// The physical pixel size to ask the art for an icon of hit box `r` in (`Art::image(rel, n, n)`), so the image is
    /// drawn one source pixel per screen pixel.
    pub fn icon_px(&self, r: Rect) -> i32 {
        (Self::icon_rect_on(r, self.p.v.g).w as f32 * self.p.v.s).round().max(1.0) as i32
    }

    /// An icon button (PLAYUI1-K): `image` (a file from the asset layers the caller looked up, e.g. with
    /// `Art::image(rel, icon_px, icon_px)`) or, when the file is missing, the drawn `glyph`; `label` names it for the
    /// agent link and is the hover card's title when `t` is empty (a disabled button's card says why). `toggled` draws
    /// it on (gold, a lock that is locked); disabled draws it dimmed and takes no click. It takes the keyboard focus
    /// like any button (Enter or Space press it).
    #[allow(clippy::too_many_arguments)]
    pub fn icon_button(&mut self, id: &str, r: Rect, label: &str, image: Option<&Image>, glyph: Glyph, action: A, enabled: bool, toggled: bool, t: Tip) {
        let (th, m) = (self.t.clone(), self.m());
        let hover = enabled && self.hover(r);
        if toggled || hover {
            self.p.panel(r, if toggled { th.selected } else { th.card_hot }, if hover { th.focus } else { th.toggled_edge }, m.radius);
        }
        let ir = Self::icon_rect_on(r, m.g);
        match image {
            Some(img) if enabled => self.image_in(img, ir),
            Some(img) => self.image_in(&faded(img, 0.4), ir),
            None => {
                let col = if !enabled { th.dim } else if toggled { th.focus } else { th.text };
                draw_glyph(&mut self.p, glyph, ir, col);
            }
        }
        let t = if t.is_empty() { tip(label, "") } else { t };
        self.push(id, "icon_button", label, r, enabled, toggled, Some(action), t);
    }

    /// A collapsible section header (PLAYUI1-K): a chevron (down while open, right while closed), the title and a note
    /// at the right (a count, a hint; may be empty). A click anywhere on it, or Enter / Space on its focus, sends
    /// `toggle`; the screen keeps whether it is open and draws the section's content only then. Returns `open`.
    #[allow(clippy::too_many_arguments)]
    pub fn section(&mut self, id: &str, r: Rect, title: &str, note: &str, open: bool, toggle: A, t: Tip) -> bool {
        let (th, m) = (self.t.clone(), self.m());
        let hover = self.hover(r);
        self.p.panel(r, if hover { th.card_hot } else { th.section }, if hover { th.focus } else { th.edge }, m.radius);
        let ty = r.y as f32 + (r.h as f32 - self.p.line_h(m.body)) / 2.0;
        let c = r.pad(Insets { l: m.u(3), ..Insets::default() }).place(Anchor::Left, m.u(8), m.u(8));
        draw_glyph(&mut self.p, if open { Glyph::ChevronDown } else { Glyph::ChevronRight }, c, if hover { th.focus } else { th.text });
        let note_w = if note.is_empty() { 0.0 } else { self.p.text_w(note, m.small) + m.uf(6.0) };
        self.p.text_fit(r.x as f32 + m.uf(14.0), ty, title, m.body, r.w as f32 - m.uf(18.0) - note_w, th.gold);
        if !note.is_empty() {
            self.p.text_right((r.x + r.w) as f32 - m.uf(4.0), ty + m.uf(0.5), note, m.small, th.dim);
        }
        self.push(id, "section", title, r, true, open, Some(toggle), t);
        open
    }
}

/// A drawn icon (PLAYUI1-K): what an icon button shows while its image file is missing, and the chevron of a section
/// header. Shapes only, so every game has them before its art lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    /// A padlock, shut.
    Lock,
    /// A padlock with its shackle open.
    Unlock,
    /// A bin.
    Delete,
    /// A question mark in a ring.
    Help,
    ChevronDown,
    ChevronRight,
    /// Three bars.
    Menu,
    Plus,
    Close,
    /// A letter in a ring (an icon with no shape of its own, e.g. 'P' for a party button).
    Letter(char),
}

/// Draws `g` inside the square `r` in `col`.
pub fn draw_glyph(p: &mut Painter, g: Glyph, r: Rect, col: Rgba) {
    let (x, y, s) = (r.x as f32, r.y as f32, r.w.min(r.h) as f32);
    let (cx, cy) = (x + s / 2.0, y + s / 2.0);
    let m = Metrics::of(&theme::th(), p.v);
    let ring = |p: &mut Painter, rad: f32| {
        let n = 20;
        for i in 0..n {
            let (a0, a1) = (i as f32 / n as f32 * std::f32::consts::TAU, (i + 1) as f32 / n as f32 * std::f32::consts::TAU);
            p.line(cx + rad * a0.cos(), cy + rad * a0.sin(), cx + rad * a1.cos(), cy + rad * a1.sin(), col, 0.0);
        }
    };
    match g {
        Glyph::Lock | Glyph::Unlock => {
            // The body in the lower half, the shackle over it (lifted to the right when open).
            // Cut to old units, so the padlock has one shape on every grid.
            let old = s / m.uf(1.0);
            let body = Rect::new(r.x + m.u((old * 0.15) as i32), r.y + m.u((old * 0.45) as i32), m.u((old * 0.7).round() as i32), m.u((old * 0.5).round() as i32));
            p.panel(body, col, col, m.uf(1.0));
            let (l, rr, top) = (x + s * 0.3, x + s * 0.7, y + s * 0.1);
            let lift = if g == Glyph::Unlock { s * 0.15 } else { 0.0 };
            p.line(l, y + s * 0.45, l, top + s * 0.12 - lift, col, 0.0);
            p.line(l, top + s * 0.12 - lift, l + s * 0.08, top - lift, col, 0.0);
            p.line(l + s * 0.08, top - lift, rr - s * 0.08, top - lift, col, 0.0);
            p.line(rr - s * 0.08, top - lift, rr, top + s * 0.12 - lift, col, 0.0);
            let right_end = if g == Glyph::Unlock { y + s * 0.3 } else { y + s * 0.45 };
            p.line(rr, top + s * 0.12 - lift, rr, right_end, col, 0.0);
        }
        Glyph::Delete => {
            p.line(x + s * 0.12, y + s * 0.2, x + s * 0.88, y + s * 0.2, col, 0.0);
            p.line(x + s * 0.38, y + s * 0.08, x + s * 0.62, y + s * 0.08, col, 0.0);
            let (l, rr, top, bot) = (x + s * 0.22, x + s * 0.78, y + s * 0.3, y + s * 0.92);
            p.line(l, top, l + s * 0.05, bot, col, 0.0);
            p.line(rr, top, rr - s * 0.05, bot, col, 0.0);
            p.line(l + s * 0.05, bot, rr - s * 0.05, bot, col, 0.0);
            p.line(cx, top + s * 0.08, cx, bot - s * 0.1, col, 0.0);
        }
        Glyph::Help => {
            ring(p, s * 0.45);
            let h = p.line_h(m.small);
            p.text_center(cx, cy - h / 2.0, "?", m.small, col);
        }
        Glyph::ChevronDown => {
            p.line(x + s * 0.2, y + s * 0.35, cx, y + s * 0.65, col, 0.0);
            p.line(cx, y + s * 0.65, x + s * 0.8, y + s * 0.35, col, 0.0);
        }
        Glyph::ChevronRight => {
            p.line(x + s * 0.35, y + s * 0.2, x + s * 0.65, cy, col, 0.0);
            p.line(x + s * 0.65, cy, x + s * 0.35, y + s * 0.8, col, 0.0);
        }
        Glyph::Menu => {
            for f in [0.25, 0.5, 0.75] {
                p.line(x + s * 0.15, y + s * f, x + s * 0.85, y + s * f, col, 0.0);
            }
        }
        Glyph::Plus => {
            p.line(cx, y + s * 0.15, cx, y + s * 0.85, col, 0.0);
            p.line(x + s * 0.15, cy, x + s * 0.85, cy, col, 0.0);
        }
        Glyph::Close => {
            p.line(x + s * 0.2, y + s * 0.2, x + s * 0.8, y + s * 0.8, col, 0.0);
            p.line(x + s * 0.2, y + s * 0.8, x + s * 0.8, y + s * 0.2, col, 0.0);
        }
        Glyph::Letter(c) => {
            ring(p, s * 0.45);
            let h = p.line_h(m.small);
            p.text_center(cx, cy - h / 2.0, &c.to_string(), m.small, col);
        }
    }
}

/// `img` with its alpha scaled by `a` (a disabled icon).
fn faded(img: &Image, a: f32) -> Image {
    let mut out = img.clone();
    for alpha in out.px.iter_mut().skip(3).step_by(4) {
        *alpha = (*alpha as f32 * a).round() as u8;
    }
    out
}

/// The hover card of the topmost hovered widget that has one.
pub fn draw_tip<A>(p: &mut Painter, widgets: &[Widget<A>], mouse: (i32, i32)) {
    let Some(wd) = widgets.iter().rev().find(|wd| wd.rect.contains(mouse.0, mouse.1) && (!wd.tip.is_empty() || wd.kind == "backdrop")) else { return };
    if wd.tip.is_empty() {
        return;
    }
    let t = theme::th();
    let m = Metrics::of(&t, p.v);
    let max_w = m.uf(160.0);
    // Each line with its size and the step to the next: the title's lines, then the body's.
    let mut lines: Vec<(String, Rgba, f32, f32)> = Vec::new();
    for (i, (text, col)) in wd.tip.iter().enumerate() {
        let (size, step) = if i == 0 { (m.body, m.uf(10.0)) } else { (m.small, m.uf(8.0)) };
        for part in p.wrap(text, size, max_w) {
            lines.push((part, *col, size, step));
        }
    }
    let tw = lines.iter().map(|l| p.text_w(&l.0, l.2)).fold(0.0f32, f32::max) + m.uf(10.0);
    let th = lines.iter().map(|l| l.3).sum::<f32>() + m.uf(6.0);
    let (cw, ch) = p.v.canvas();
    let (mut tx, mut ty) = (mouse.0 + m.u(10), mouse.1 + m.u(12));
    if tx as f32 + tw > (cw - m.u(2)) as f32 {
        tx = mouse.0 - tw as i32 - m.u(6);
    }
    if ty as f32 + th > (ch - m.u(2)) as f32 {
        ty = mouse.1 - th as i32 - m.u(6);
    }
    let r = Rect::new(tx.max(m.u(2)), ty.max(m.u(2)), tw.ceil() as i32, th.ceil() as i32);
    let (fill, edge, radius) = t.vector_in("tip", &m);
    p.skin(t.frame("tip"), r, fill, edge, radius);
    let mut ly = r.y as f32 + m.uf(3.0);
    for (text, col, size, step) in lines {
        p.text(r.x as f32 + m.uf(5.0), ly, &text, size, col);
        ly += step;
    }
}

/// The topmost enabled clickable widget under `pos`; an open select's options lie over everything. A disabled icon
/// button still takes its click (and does nothing), so a row or card drawn under it never fires (PLAYUI1 section 1).
pub fn hit<A: Clone>(widgets: &[Widget<A>], pos: (i32, i32)) -> Option<A> {
    let under = |w: &&Widget<A>| (w.enabled || w.kind == "icon_button") && w.action.is_some() && w.rect.contains(pos.0, pos.1);
    let top = widgets.iter().rev().filter(|w| w.kind == "option").find(under).or_else(|| widgets.iter().rev().find(under));
    top.filter(|w| w.enabled).and_then(|w| w.action.clone())
}

/// The action of widget `id`, or why it can't be clicked.
pub fn by_id<A: Clone>(widgets: &[Widget<A>], id: &str) -> Result<A, String> {
    match widgets.iter().find(|w| w.id == id) {
        Some(w) if w.enabled && w.action.is_some() => Ok(w.action.clone().unwrap()),
        Some(_) => Err(format!("widget '{id}' is disabled or not clickable")),
        None => Err(format!("no widget '{id}'")),
    }
}
