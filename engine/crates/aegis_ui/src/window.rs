//! Windows in the UI kit (UIKIT3; Godot's UiWindow and the dojo's workspace desk, game-agnostic): a screen keeps a
//! `Desk` of windows inside an area of the canvas. Each window has a title bar (drag it to move the window), a grip in
//! its lower right corner (drag it to resize), collapse and close buttons, and a body the screen draws into, clipped to
//! the body. Pressing anywhere on a window raises it; a window covers what lies under it (clicks, hover, drops). The
//! desk's toolbar opens and closes windows and applies layout presets. The shell handles every desk widget itself (ids
//! `win.<id>.<part>` and `desk.<what>.<name>`, `Screen::desk`), so a screen only names its windows, draws their bodies
//! and keeps its desk. Geometry is kept as fractions of the area, so a layout survives a different area; the shell saves
//! the desk in the settings under `desks.<screen id>` after every change; the screen reads it back when it builds the
//! desk (`restore(&settings.v["desks"][id])`). Ports UiWindow (`_clamp_to_parent`, `commit_fraction`) and
//! DojoWorkspace (`apply_layout`, `save_layout`, `_load_layout`). LAYOUT720-E4: a desk draws in the units of the grid it is
//! on (`Desk::on_grid`, old units by default); the settings hold fractions, so `desks.<id>` means the same on every grid.

use crate::canvas::{rgba, Rect};
use crate::painter::Grid;
use crate::ui::{tip, Ui, RADIUS};
use serde_json::{json, Value};

/// Height of a window's title bar (old units).
pub const TITLE_H: i32 = 13;
const GRIP: i32 = 8;
const BUTTON: i32 = 11;

/// A rect as fractions (0-1) of the desk's area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frac {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Frac {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Win {
    /// No dots: widget ids are `win.<id>.<part>`.
    pub id: String,
    pub title: String,
    pub frac: Frac,
    pub open: bool,
    pub collapsed: bool,
    /// The smallest size (old units) a resize leaves.
    pub min: (i32, i32),
}

impl Win {
    pub fn new(id: &str, title: &str, min: (i32, i32)) -> Self {
        assert!(!id.contains('.') && !id.is_empty(), "window id '{id}': no dots (widget ids are win.<id>.<part>)");
        Self { id: id.into(), title: title.into(), frac: Frac::new(0.0, 0.0, 0.3, 0.4), open: false, collapsed: false, min }
    }
}

/// A named layout: where each window it lists goes (the others close).
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub name: String,
    pub places: Vec<(String, Frac)>,
}

/// What a drag on a window does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grab {
    Move,
    Resize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Desk {
    pub area: Rect,
    /// Back to front: the last open window is on top.
    pub wins: Vec<Win>,
    pub layouts: Vec<Layout>,
    /// The layout applied last ("" after a free change is kept as the last name).
    pub layout: String,
    /// The windows in the order they were given (the toolbar's order).
    order: Vec<String>,
    /// Units per old unit (`View::g`) of the grid `area` and the rects are in.
    g: i32,
}

impl Desk {
    /// The windows over `area`, arranged by the first layout.
    pub fn new(area: Rect, wins: Vec<Win>, layouts: Vec<Layout>) -> Self {
        let order = wins.iter().map(|w| w.id.clone()).collect();
        let mut d = Self { area, wins, layouts, layout: String::new(), order, g: 1 };
        if let Some(first) = d.layouts.first().map(|l| l.name.clone()) {
            d.apply_layout(&first);
        }
        d
    }

    /// The desk on `grid`: `area`, every rect and a drag are in its units, as the screen drawing it is (`Screen::grid`). Sizes
    /// in old units (`Win::min`, the title bar) follow; the fractions, and so the saved desk, do not change.
    pub fn on_grid(mut self, grid: Grid) -> Self {
        self.g = grid.g();
        self
    }

    pub fn win(&self, id: &str) -> Option<&Win> {
        self.wins.iter().find(|w| w.id == id)
    }

    fn win_mut(&mut self, id: &str) -> Option<&mut Win> {
        self.wins.iter_mut().find(|w| w.id == id)
    }

    /// Places every window the layout lists (open, expanded, in the layout's order on top) and closes the rest.
    pub fn apply_layout(&mut self, name: &str) -> bool {
        let Some(l) = self.layouts.iter().find(|l| l.name == name).cloned() else { return false };
        for w in self.wins.iter_mut() {
            w.open = false;
        }
        for (id, frac) in &l.places {
            if let Some(w) = self.win_mut(id) {
                w.frac = *frac;
                w.open = true;
                w.collapsed = false;
            }
            self.raise(id);
        }
        self.layout = l.name;
        true
    }

    /// A window's rect on the canvas (only the title bar when collapsed).
    pub fn rect(&self, id: &str) -> Option<Rect> {
        let w = self.win(id)?;
        let a = self.area;
        let x = a.x + (w.frac.x * a.w as f32).round() as i32;
        let y = a.y + (w.frac.y * a.h as f32).round() as i32;
        let r = Rect::new(x, y, (w.frac.w * a.w as f32).round() as i32, (w.frac.h * a.h as f32).round() as i32);
        Some(if w.collapsed { Rect::new(r.x, r.y, r.w, TITLE_H * self.g) } else { r })
    }

    /// The rect a window's content is drawn in.
    pub fn body(&self, id: &str) -> Option<Rect> {
        let r = self.rect(id)?;
        let g = self.g;
        Some(Rect::new(r.x + 2 * g, r.y + (TITLE_H + 1) * g, (r.w - 4 * g).max(0), (r.h - (TITLE_H + 3) * g).max(0)))
    }

    /// The topmost open window under `pos`.
    pub fn top_at(&self, pos: (i32, i32)) -> Option<&str> {
        self.wins.iter().rev().filter(|w| w.open).find(|w| self.rect(&w.id).is_some_and(|r| r.contains(pos.0, pos.1))).map(|w| w.id.as_str())
    }

    pub fn raise(&mut self, id: &str) {
        if let Some(i) = self.wins.iter().position(|w| w.id == id) {
            let w = self.wins.remove(i);
            self.wins.push(w);
        }
    }

    /// Opens (on top) or closes a window.
    pub fn toggle(&mut self, id: &str) {
        let open = self.win(id).is_some_and(|w| !w.open);
        self.set_open(id, open);
    }

    pub fn set_open(&mut self, id: &str, open: bool) {
        if let Some(w) = self.win_mut(id) {
            w.open = open;
        }
        if open {
            self.raise(id);
        }
    }

    pub fn collapse(&mut self, id: &str) {
        if let Some(w) = self.win_mut(id) {
            w.collapsed = !w.collapsed;
        }
    }

    /// Moves or resizes a window by (dx, dy) units of its grid, kept inside the area and at least its minimum size.
    pub fn drag(&mut self, id: &str, grab: Grab, dx: i32, dy: i32) {
        let (a, g, Some(cur)) = (self.area, self.g, self.rect(id)) else { return };
        if a.w <= 0 || a.h <= 0 {
            return;
        }
        let Some(w) = self.win_mut(id) else { return };
        let full_h = (w.frac.h * a.h as f32).round() as i32;
        let mut r = Rect::new(cur.x, cur.y, cur.w, full_h);
        match grab {
            Grab::Move => {
                // The whole window stays inside the area (UiWindow._clamp_to_parent), a collapsed one by its title bar.
                r.x = fit(r.x + dx, a.x, a.x + a.w - r.w);
                r.y = fit(r.y + dy, a.y, a.y + a.h - cur.h);
            }
            Grab::Resize => {
                // The minimum size wins over the room left: a window near the far edge moves back inside.
                let (least_w, least_h) = ((w.min.0 * g).min(a.w), (w.min.1 * g).min(a.h));
                r.w = fit(r.w + dx, least_w, a.x + a.w - r.x).max(least_w);
                r.h = fit(r.h + dy, least_h, a.y + a.h - r.y).max(least_h);
                r.x = fit(r.x, a.x, a.x + a.w - r.w);
                r.y = fit(r.y, a.y, a.y + a.h - r.h);
            }
        }
        w.frac = Frac::new((r.x - a.x) as f32 / a.w as f32, (r.y - a.y) as f32 / a.h as f32, r.w as f32 / a.w as f32, r.h as f32 / a.h as f32);
    }

    /// The desk as saved in the settings: the layout name and every window back to front.
    pub fn to_value(&self) -> Value {
        let wins: Vec<Value> = self.wins.iter().map(|w| json!({"id": w.id, "frac": [w.frac.x, w.frac.y, w.frac.w, w.frac.h], "open": w.open, "collapsed": w.collapsed})).collect();
        json!({"layout": self.layout, "wins": wins})
    }

    /// Takes a saved desk (`to_value`) back: known windows get their place, state and order; unknown ones are
    /// ignored and windows the save lacks keep the layout's place. Anything malformed is skipped.
    pub fn restore(&mut self, v: &Value) {
        let Some(saved) = v["wins"].as_array() else { return };
        for s in saved {
            let Some(id) = s["id"].as_str() else { continue };
            let f: Vec<f32> = s["frac"].as_array().map(|a| a.iter().filter_map(|x| x.as_f64()).map(|x| x as f32).collect()).unwrap_or_default();
            let Some(w) = self.win_mut(id) else { continue };
            if f.len() == 4 && f.iter().all(|x| x.is_finite() && (0.0..=1.0).contains(x)) && f[2] > 0.0 && f[3] > 0.0 && f[0] + f[2] <= 1.001 && f[1] + f[3] <= 1.001 {
                w.frac = Frac::new(f[0], f[1], f[2], f[3]);
            }
            w.open = s["open"].as_bool().unwrap_or(w.open);
            w.collapsed = s["collapsed"].as_bool().unwrap_or(false);
            self.raise(id);
        }
        // A saved size below a window's minimum grows back to it (inside the area).
        let ids: Vec<String> = self.wins.iter().map(|w| w.id.clone()).collect();
        for id in ids {
            self.drag(&id, Grab::Resize, 0, 0);
        }
        // A layout name the screen no longer has is not kept (DojoWorkspace._load_layout falls back).
        if let Some(l) = v["layout"].as_str().filter(|l| self.layouts.iter().any(|x| x.name == *l)) {
            self.layout = l.to_string();
        }
    }

    /// For the agent link: every window with its rect and state, back to front.
    pub fn info(&self) -> Value {
        let wins: Vec<Value> = self
            .wins
            .iter()
            .map(|w| {
                let r = self.rect(&w.id).unwrap_or_default();
                json!({"id": w.id, "title": w.title, "open": w.open, "collapsed": w.collapsed, "rect": [r.x, r.y, r.w, r.h]})
            })
            .collect();
        json!({"layout": self.layout, "layouts": self.layouts.iter().map(|l| l.name.clone()).collect::<Vec<_>>(), "windows": wins})
    }

    /// Draws the open windows back to front: frame, title bar, buttons and grip (shell-handled widgets) and the body,
    /// which `body(ui, id, rect)` fills, clipped to it. Under a window the mouse counts as elsewhere, so covered
    /// widgets show no hover.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, mut body: impl FnMut(&mut Ui<A>, &str, Rect)) {
        let mouse = ui.mouse;
        let top = self.top_at(mouse).map(String::from);
        let front = self.wins.iter().rev().find(|w| w.open).map(|w| w.id.clone());
        let (t, m) = (ui.t.clone(), ui.m());
        for w in self.wins.iter().filter(|w| w.open) {
            let Some(r) = self.rect(&w.id) else { continue };
            ui.mouse = if top.as_deref() == Some(w.id.as_str()) { mouse } else { (-1, -1) };
            let lit = front.as_deref() == Some(w.id.as_str());
            ui.p.panel(r, t.card, if lit { t.panel_edge } else { t.edge }, m.uf(RADIUS));
            // The window blocks what is under it: a backdrop over the whole rect, its controls on top.
            ui.push(format!("win.{}.body", w.id), "backdrop", w.title.clone(), r, true, false, None, Vec::new());
            let bar = Rect::new(r.x + m.u(1), r.y + m.u(1), r.w - m.u(2), m.u(TITLE_H - 1));
            ui.p.fill(bar, if lit { t.card_hot } else { t.title_idle });
            ui.p.text_fit(bar.x as f32 + m.uf(4.0), bar.y as f32 + m.uf(1.5), &w.title, m.body, (bar.w - m.u(2 * BUTTON + 10)) as f32, if lit { t.gold } else { t.text });
            ui.push(format!("win.{}.title", w.id), "win_title", w.title.clone(), Rect::new(bar.x, bar.y, bar.w - m.u(2 * BUTTON + 4), bar.h), true, false, None, tip(&w.title, "Drag to move; click to bring it to the front."));
            for (i, (part, label, help)) in [("close", "x", "Close (the toolbar opens it again)"), ("collapse", if w.collapsed { "+" } else { "-" }, if w.collapsed { "Expand" } else { "Collapse to the title bar" })].into_iter().enumerate() {
                let b = Rect::new(bar.x + bar.w - m.u((i as i32 + 1) * (BUTTON + 1)), bar.y + m.u(1), m.u(BUTTON), m.u(BUTTON - 1));
                let hover = ui.hover(b);
                ui.p.panel(b, if hover { t.card_hot } else { rgba(0, 0, 0, 0) }, if hover { t.focus } else { rgba(0, 0, 0, 0) }, m.uf(2.0));
                // Drawn as strokes, not text, so the text size never pushes them out of the bar (TEXTFIT1).
                let (col, cx, cy, d) = (if hover { t.focus } else { t.dim }, b.x as f32 + b.w as f32 / 2.0, b.y as f32 + b.h as f32 / 2.0, m.uf(2.5));
                match label {
                    "x" => {
                        ui.p.line(cx - d, cy - d, cx + d, cy + d, col, 0.0);
                        ui.p.line(cx - d, cy + d, cx + d, cy - d, col, 0.0);
                    }
                    _ => {
                        ui.p.line(cx - d, cy, cx + d, cy, col, 0.0);
                        if label == "+" {
                            ui.p.line(cx, cy - d, cx, cy + d, col, 0.0);
                        }
                    }
                }
                ui.push(format!("win.{}.{part}", w.id), "win_button", label, b, true, false, None, tip(label, help));
            }
            if !w.collapsed {
                if let Some(b) = self.body(&w.id) {
                    ui.push_clip(b);
                    body(ui, &w.id, b);
                    ui.pop_clip();
                }
                let g = Rect::new(r.x + r.w - m.u(GRIP), r.y + r.h - m.u(GRIP), m.u(GRIP), m.u(GRIP));
                let col = if ui.hover(g) { t.focus } else { t.grip };
                for k in [2, 5] {
                    ui.p.fill(Rect::new(g.x + g.w - m.u(2), g.y + g.h - m.u(k + 1), m.u(1), m.u(k)), col);
                    ui.p.fill(Rect::new(g.x + g.w - m.u(k + 1), g.y + g.h - m.u(2), m.u(k), m.u(1)), col);
                }
                ui.push(format!("win.{}.grip", w.id), "win_grip", "resize", g, true, false, None, tip("Resize", "Drag to resize the window."));
            }
        }
        ui.mouse = mouse;
    }

    /// The toolbar at (x, y): a toggle per window (gold while open) and a button per layout; returns the right edge.
    pub fn toolbar<A: Clone>(&self, ui: &mut Ui<A>, x: i32, y: i32) -> i32 {
        let mut x = x;
        for id in &self.order {
            let Some(w) = self.win(id) else { continue };
            x = desk_button(ui, x, y, format!("desk.toggle.{id}"), &w.title, w.open, "Open or close this window.");
        }
        x += ui.m().u(6);
        for l in &self.layouts {
            x = desk_button(ui, x, y, format!("desk.layout.{}", l.name), &l.name, self.layout == l.name, "Arrange the windows by this layout (it reopens its windows).");
        }
        x
    }
}

/// `v` within lo..=hi; `lo` wins when the range is empty (no panic).
fn fit(v: i32, lo: i32, hi: i32) -> i32 {
    v.min(hi).max(lo)
}

/// One toolbar button (a shell-handled `desk_button`); returns the next x.
fn desk_button<A: Clone>(ui: &mut Ui<A>, x: i32, y: i32, id: String, label: &str, active: bool, help: &str) -> i32 {
    let (t, m) = (ui.t.clone(), ui.m());
    let w = ui.p.text_w(label, m.small).ceil() as i32 + m.u(10);
    let r = Rect::new(x, y, w, m.u(12));
    let hover = ui.hover(r);
    ui.p.panel(r, if active { t.selected } else if hover { t.card_hot } else { t.card }, if active || hover { t.focus } else { t.edge }, m.uf(2.0));
    ui.p.text_center((r.x + r.w / 2) as f32, r.y as f32 + m.uf(2.0), label, m.small, if active { t.focus } else { t.text });
    ui.push(id, "desk_button", label, r, true, active, None, tip(label, help));
    x + w + m.u(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desk() -> Desk {
        let wins = vec![Win::new("a", "Alpha", (60, 40)), Win::new("b", "Beta", (60, 40)), Win::new("c", "Gamma", (60, 40))];
        let layouts = vec![
            Layout { name: "Two".into(), places: vec![("a".into(), Frac::new(0.0, 0.0, 0.5, 1.0)), ("b".into(), Frac::new(0.5, 0.0, 0.5, 1.0))] },
            Layout { name: "One".into(), places: vec![("c".into(), Frac::new(0.25, 0.25, 0.5, 0.5))] },
        ];
        Desk::new(Rect::new(0, 20, 400, 200), wins, layouts)
    }

    #[test]
    fn the_first_layout_places_and_opens_its_windows_and_closes_the_rest() {
        let d = desk();
        assert_eq!(d.layout, "Two");
        assert_eq!(d.rect("a"), Some(Rect::new(0, 20, 200, 200)));
        assert_eq!(d.rect("b"), Some(Rect::new(200, 20, 200, 200)));
        assert!(!d.win("c").unwrap().open);
        assert_eq!(d.top_at((250, 50)), Some("b"));
        assert_eq!(d.top_at((5, 5)), None, "outside the area");
    }

    #[test]
    fn moving_and_resizing_stay_inside_the_area_and_above_the_minimum() {
        let mut d = desk();
        d.drag("a", Grab::Move, -50, -50);
        assert_eq!(d.rect("a").unwrap(), Rect::new(0, 20, 200, 200), "already at the corner");
        d.drag("a", Grab::Resize, -500, -500);
        assert_eq!(d.rect("a").unwrap(), Rect::new(0, 20, 60, 40), "the minimum size");
        d.drag("a", Grab::Move, 1000, 1000);
        let r = d.rect("a").unwrap();
        assert_eq!((r.x + r.w, r.y + r.h), (400, 220), "the whole window stays inside");
        d.drag("a", Grab::Resize, 1000, 1000);
        let r = d.rect("a").unwrap();
        assert!(r.x + r.w <= 400 && r.y + r.h <= 220);
    }

    #[test]
    fn raising_toggling_collapsing_and_layouts_change_the_order_and_state() {
        let mut d = desk();
        d.raise("a");
        assert_eq!(d.top_at((199, 50)), Some("a"));
        d.toggle("c");
        assert_eq!(d.wins.last().unwrap().id, "c", "an opened window comes to the front");
        d.collapse("c");
        assert_eq!(d.rect("c").unwrap().h, TITLE_H);
        assert!(d.apply_layout("One"));
        assert!(d.win("c").unwrap().open && !d.win("c").unwrap().collapsed && !d.win("a").unwrap().open);
        assert!(!d.apply_layout("None"));
    }

    #[test]
    fn a_saved_desk_comes_back_and_bad_entries_are_skipped() {
        let mut d = desk();
        d.drag("b", Grab::Move, -40, 30);
        d.toggle("c");
        d.collapse("c");
        let saved = d.to_value();
        let mut back = desk();
        back.restore(&saved);
        assert_eq!(back, d);
        let mut odd = desk();
        odd.restore(&json!({"wins": [{"id": "zzz"}, {"id": "a", "frac": [0.5, "x"]}, {"id": "b", "frac": [2.0, 0.0, 0.5, 0.5]}]}));
        assert_eq!(odd.rect("a"), desk().rect("a"));
        assert_eq!(odd.rect("b"), desk().rect("b"));
        // Past the area, below the minimum size, or an unknown layout name: not taken as saved.
        let mut checked = desk();
        checked.restore(&json!({"layout": "Gone", "wins": [{"id": "a", "frac": [0.9, 0.0, 0.5, 0.5]}, {"id": "b", "frac": [0.0, 0.0, 0.01, 0.01]}]}));
        assert_eq!(checked.rect("a"), desk().rect("a"), "x + w past the area");
        let b = checked.rect("b").unwrap();
        assert_eq!((b.w, b.h), (60, 40), "grown to the minimum size");
        assert_eq!(checked.layout, "Two", "an unknown layout name is ignored");
    }

    #[test]
    fn a_collapsed_window_moves_down_to_the_area_bottom() {
        let mut d = desk();
        d.collapse("a");
        d.drag("a", Grab::Move, 0, 1000);
        let r = d.rect("a").unwrap();
        assert_eq!(r.y + r.h, 220, "the title bar reaches the bottom");
        d.collapse("a");
        assert!(d.rect("a").unwrap().h > TITLE_H);
    }
}
