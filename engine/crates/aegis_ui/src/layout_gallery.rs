//! UISPLIT1 US6 (PR7-38): the UI Gallery's layout preview. Every screen with a layout file or a view fixture in the
//! asset layers, picked on the left; the fixture drawn through its layout (or the default, on a toggle) on the right, with
//! the problems that sent it to the default under it. In a native debug build a layout file reloads when it is saved
//! (`screen_layout::Layouts`). Clicks on the preview's own widgets do nothing but show the action they would run.

use crate::painter::Grid;
use crate::screen::{Cx, Nav, Screen};
use crate::screen_layout::{self, Act, Layouts, View};
use crate::ui::{Gfx, Ui, Widget};
use crate::widget_plumbing;
use aegis_layout::Insets;
use serde_json::{json, Value};

pub const ID: &str = "layout_gallery";
/// The gallery's own actions, apart from the previewed screen's.
const PICK: &str = "gallery.pick";
const DEFAULT: &str = "gallery.default";
const CLOSE: &str = "gallery.close";

#[derive(Default)]
pub struct LayoutGallery {
    screens: Vec<String>,
    picked: usize,
    default: bool,
    layouts: Option<Layouts>,
    /// The problems shown under the preview: the fixture's, or the layout's that sent it to the default.
    problems: Vec<String>,
    /// The last action a click on the preview would have run.
    last: Option<Act>,
    w: Vec<Widget<Act>>,
}

impl LayoutGallery {
    fn act<G>(&mut self, cx: &mut Cx<G>, a: Act) {
        match a.action.as_str() {
            CLOSE => cx.nav.push(Nav::Pop),
            PICK => {
                self.picked = a.arg.and_then(|i| i.parse().ok()).unwrap_or(0);
                self.problems.clear();
                self.last = None;
            }
            DEFAULT => self.default = !self.default,
            _ => self.last = Some(a),
        }
    }
}

impl<G> Screen<G> for LayoutGallery {
    fn id(&self) -> &'static str {
        ID
    }
    fn grid(&self) -> Grid {
        Grid::Native
    }
    fn draw(&mut self, cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let layers = cx.art.layers().clone();
        if self.layouts.is_none() {
            self.screens = screen_layout::screens(&layers);
            self.layouts = Some(Layouts::new(layers.clone()));
        }
        let screen = self.screens.get(self.picked).cloned();
        let view: Option<Result<View, Vec<String>>> = screen.as_ref().map(|s| screen_layout::load_fixture(&layers, s).unwrap_or_else(|| Err(vec![format!("no fixture {}/{s}{}", screen_layout::VIEWS_DIR, screen_layout::VIEW_SUFFIX)])));
        let mut ui: Ui<Act> = Ui::new(g, mouse, live);
        let (m, area) = (ui.m(), ui.screen());
        let bg = ui.t.bg;
        ui.p.fill(area, bg);
        let mut body = area.pad(Insets::all(m.margin));
        let mut side = body.cut_left(m.u(110));
        body.cut_left(m.gutter);
        let close = side.cut_bottom(m.u(12));
        ui.button("gallery.close", close.pad(Insets { l: 0, t: m.u(2), r: 0, b: 0 }), "Close", Act { action: CLOSE.into(), arg: None }, true);
        let toggle = side.cut_bottom(m.u(12));
        ui.button("gallery.default", toggle.pad(Insets { l: 0, t: m.u(2), r: 0, b: 0 }), if self.default { "Showing the default" } else { "Showing the layout" }, Act { action: DEFAULT.into(), arg: None }, true);
        let gold = ui.t.gold;
        let head = side.cut_top(ui.p.line_h(m.body).ceil() as i32 + m.u(2));
        ui.p.text_fit(head.x as f32, head.y as f32, "Screen layouts", m.body, head.w as f32, gold);
        for (i, s) in self.screens.iter().enumerate() {
            let r = side.cut_top(m.row_h.min(side.h));
            let text = ui.t.text;
            ui.row(&format!("gallery.screen.{i}"), r, s, text, Some(Act { action: PICK.into(), arg: Some(i.to_string()) }), i == self.picked, Vec::new());
        }
        if self.screens.is_empty() {
            let dim = ui.t.dim;
            ui.para(side.x, side.y, side.w, "No layout file or fixture in the asset layers (ui/layouts/, ui/views/).", m.small, dim);
        }
        let notes_h = if self.problems.is_empty() && self.last.is_none() { 0 } else { m.u(24) };
        let notes = body.cut_bottom(notes_h);
        match view {
            Some(Ok(v)) => {
                let layout = if self.default { None } else { self.layouts.as_mut().and_then(|l| l.get(&v)).cloned() };
                if let Some(l) = self.layouts.as_mut() {
                    let fresh = l.take_problems();
                    if !fresh.is_empty() {
                        self.problems = fresh;
                    } else if layout.is_some() {
                        // a fixed layout that loaded again clears the old problems
                        self.problems.clear();
                    }
                }
                screen_layout::draw(&mut ui, &mut cx.art, layout.as_ref(), &v, body);
            }
            Some(Err(ps)) => self.problems = ps,
            None => {}
        }
        if notes_h > 0 {
            let (dim, bad) = (ui.t.dim, ui.t.foe);
            let mut lines: Vec<(String, _)> = self.problems.iter().map(|p| (p.clone(), bad)).collect();
            if let Some(a) = &self.last {
                lines.push((format!("Would run: {}{}", a.action, a.arg.as_ref().map(|x| format!(" ({x})")).unwrap_or_default()), dim));
            }
            let mut y = notes.y + m.u(2);
            for (l, col) in lines {
                y = ui.para(notes.x, y, notes.w, &l, m.small, col);
            }
        }
        self.w = ui.widgets;
    }
    fn state(&self, _cx: &Cx<G>) -> Value {
        json!({"screens": self.screens, "picked": self.screens.get(self.picked), "default": self.default, "problems": self.problems,
            "last": self.last.as_ref().map(|a| json!({"action": a.action, "arg": a.arg}))})
    }
    widget_plumbing!(Cx<G>);
}
