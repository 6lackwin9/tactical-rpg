//! The layout specimen (LAYOUT720-E5, section 6.4): anchors, shares, fills, a stack and `pick`, on one screen on
//! `Grid::Native` with no fixed position, every rect cut from `Ui::screen`. It is what the kit's layout looks like, and the
//! layout audit's subject: it passes `desktop`, and on a smaller canvas it picks another arrangement until a stack of its
//! sections no longer fits, which it reports (`Ui::report_overflow`).

use crate::canvas::{Rect, Rgba};
use crate::painter::Grid;
use crate::screen::{Cx, Nav, Screen};
use crate::theme::Metrics;
use crate::ui::{Env, Gfx, Pointer, Ui, Widget};
use crate::widget_plumbing;
use aegis_layout::{fill, frac, pick, Anchor, Insets, Need, Part, SizeClass, Stack};
use serde_json::{json, Value};

pub const ID: &str = "layout_specimen";

/// The arrangement of the four sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arrange {
    /// Two across, two down.
    Grid,
    /// One above another, sharing the height.
    Rows,
    /// One above another, each its own height: past the area they overflow, and it is reported.
    Stacked,
}

impl Arrange {
    fn name(self) -> &'static str {
        match self {
            Arrange::Grid => "grid",
            Arrange::Rows => "rows",
            Arrange::Stacked => "stacked",
        }
    }
}

const ANCHORS: [(Anchor, &str); 9] = [
    (Anchor::TopLeft, "top_left"),
    (Anchor::Top, "top"),
    (Anchor::TopRight, "top_right"),
    (Anchor::Left, "left"),
    (Anchor::Center, "center"),
    (Anchor::Right, "right"),
    (Anchor::BottomLeft, "bottom_left"),
    (Anchor::Bottom, "bottom"),
    (Anchor::BottomRight, "bottom_right"),
];
/// Per mille of a row, the three cells of the Shares section.
const SHARES: [u16; 3] = [186, 314, 500];
/// The weights of the Fills section.
const FILLS: [u16; 2] = [2, 1];
/// The rows of the Stack section.
const STACK_ROWS: usize = 4;

#[derive(Debug, Clone, Copy)]
pub enum Act {
    Close,
}

/// What it last laid out, for the agent link and the tests.
#[derive(Default)]
pub struct LayoutSpecimen {
    arrange: Option<Arrange>,
    inner: Option<Arrange>,
    overflow: i32,
    w: Vec<Widget<Act>>,
}

fn class_name(c: SizeClass) -> &'static str {
    match c {
        SizeClass::Compact => "Compact",
        SizeClass::Medium => "Medium",
        SizeClass::Expanded => "Expanded",
    }
}

/// What the screen knows of where it runs, one line.
fn env_line(env: &Env) -> String {
    let pointer = if env.pointer == Pointer::Touch { "touch" } else { "mouse" };
    format!("Canvas {} x {}; {} by {}; {pointer}", env.canvas.0, env.canvas.1, class_name(env.width_class), class_name(env.height_class))
}

impl LayoutSpecimen {
    fn act<G>(&mut self, cx: &mut Cx<G>, a: Act) {
        match a {
            Act::Close => cx.nav.push(Nav::Pop),
        }
    }

    /// The label of a cell, centred in it and cut to it, over a plain fill.
    fn cell(ui: &mut Ui<Act>, id: String, r: Rect, label: &str, fill_with: Rgba) {
        let m = ui.m();
        ui.p.fill(r, fill_with);
        let (text, h) = (ui.t.text, ui.p.line_h(m.small));
        let shown = ui.p.fit(label, m.small, r.w as f32);
        ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - h) / 2.0, &shown, m.small, text);
        ui.push(id, "cell", label, r, true, false, None, Vec::new());
    }

    /// A section: its frame, its title and the rect left for its content.
    fn section(ui: &mut Ui<Act>, m: &Metrics, r: Rect, title: &str) -> Rect {
        let mut body = ui.frame_body("inset", r);
        let head = body.cut_top(ui.p.line_h(m.body).ceil() as i32 + m.u(2));
        let gold = ui.t.gold;
        ui.p.text_fit(head.x as f32, head.y as f32, title, m.body, head.w as f32, gold);
        body
    }

    fn anchors(ui: &mut Ui<Act>, m: &Metrics, r: Rect) {
        let body = Self::section(ui, m, r, "Anchors");
        let (panel, line) = (ui.t.panel, ui.p.line_h(m.small).ceil() as i32);
        let w = frac(250);
        for (at, name) in ANCHORS {
            let cell = body.place(at, w, line + m.u(2));
            Self::cell(ui, format!("layout.anchor.{name}"), cell, name, panel);
        }
    }

    fn shares(ui: &mut Ui<Act>, m: &Metrics, r: Rect) {
        let body = Self::section(ui, m, r, "Shares and fills");
        let [top, bottom] = body.split_y(&[fill(1), fill(1)], m.gutter)[..] else { return };
        let (panel, edge) = (ui.t.panel, ui.t.panel_edge);
        for (cell, share) in top.split_x(&SHARES.map(frac), m.gutter).into_iter().zip(SHARES) {
            Self::cell(ui, format!("layout.share.{share}"), cell, &format!("{}.{} %", share / 10, share % 10), panel);
        }
        let token: Part = m.u(30).into();
        let cells = bottom.split_x(&[token, fill(FILLS[0]), fill(FILLS[1])], m.gutter);
        Self::cell(ui, "layout.fill.token".into(), cells[0], "token", edge);
        for (cell, weight) in cells[1..].iter().zip(FILLS) {
            Self::cell(ui, format!("layout.fill.{weight}"), *cell, &format!("fill {weight}"), panel);
        }
    }

    /// A stack of rows; what does not fit goes to the audit, and drawing stays in the area.
    fn stack(&mut self, ui: &mut Ui<Act>, m: &Metrics, r: Rect) {
        let body = Self::section(ui, m, r, "Stack");
        let panel = ui.t.panel;
        let mut rows = Stack::new(body, m.u(2));
        ui.push_clip(body);
        for i in 0..STACK_ROWS {
            let row = rows.next(m.row_h);
            if row.h > 0 {
                Self::cell(ui, format!("layout.stack.{i}"), row, &format!("Row {}", i + 1), panel);
            }
        }
        ui.pop_clip();
        self.overflow += rows.overflow();
        ui.report_overflow("layout.stack", body, rows.overflow());
    }

    /// `pick` inside a pane: side by side while they fit, one above the other below that.
    fn picked(&mut self, ui: &mut Ui<Act>, m: &Metrics, r: Rect) {
        let body = Self::section(ui, m, r, "Pick");
        let panel = ui.t.panel;
        let across = pick(body, &[(Need { min_w: m.u(150), min_h: m.u(16) }, Arrange::Grid)], Arrange::Rows);
        self.inner = Some(across);
        let cells = if across == Arrange::Grid { body.split_x(&[fill(1), fill(1)], m.gutter) } else { body.split_y(&[fill(1), fill(1)], m.gutter) };
        let label = if across == Arrange::Grid { "side by side" } else { "above" };
        for (i, cell) in cells.into_iter().enumerate() {
            Self::cell(ui, format!("layout.pick.{i}"), cell, label, panel);
        }
    }
}

impl<G> Screen<G> for LayoutSpecimen {
    fn id(&self) -> &'static str {
        ID
    }
    fn grid(&self) -> Grid {
        Grid::Native
    }
    fn draw(&mut self, _cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<Act> = Ui::new(g, mouse, live);
        let (m, area, env) = (ui.m(), ui.screen(), ui.env());
        let mut body = ui.frame_body("panel", area.pad(Insets::all(m.margin)));
        let (gold, dim) = (ui.t.gold, ui.t.dim);
        let (line, small_line) = (ui.p.line_h(m.body).ceil() as i32, ui.p.line_h(m.small).ceil() as i32);
        // The head grows with the text: a title, a line under it, and the button at its right.
        let mut head = body.cut_top((line + small_line).max(m.tab_h));
        let close = head.cut_right(m.u(60)).place(Anchor::Right, frac(1000), m.tab_h);
        ui.button("layout.close", close, "Close", Act::Close, true);
        let mut lines = Stack::new(head, 0);
        let (title, about) = (lines.next(line), lines.next(small_line));
        ui.p.text_fit(title.x as f32, title.y as f32, "Layout", m.body, title.w as f32, gold);
        ui.p.text_fit(about.x as f32, about.y as f32, &env_line(&env), m.small, about.w as f32, dim);
        body.cut_top(m.gutter);
        let arrange = pick(body, &[(Need { min_w: m.u(300), min_h: m.u(120) }, Arrange::Grid), (Need { min_w: 0, min_h: m.u(240) }, Arrange::Rows)], Arrange::Stacked);
        self.arrange = Some(arrange);
        self.overflow = 0;
        let sections: Vec<Rect> = match arrange {
            Arrange::Grid => body.grid(2, 2, m.gutter),
            Arrange::Rows => body.split_y(&[fill(1), fill(1), fill(1), fill(1)], m.gutter),
            Arrange::Stacked => {
                // Each section its own height, from its rows: what no longer fits goes to the audit.
                let mut stack = Stack::new(body, m.gutter);
                let tall = m.u(60) + line;
                let cut: Vec<Rect> = (0..4).map(|_| stack.next(tall)).collect();
                self.overflow += stack.overflow();
                ui.report_overflow("layout.sections", body, stack.overflow());
                cut
            }
        };
        let [anchors, shares, stack, picked] = sections[..] else { return };
        for (r, draw) in [(anchors, 0), (shares, 1), (stack, 2), (picked, 3)] {
            // A section with no room left shows nothing: the overflow above says why.
            if r.w <= 0 || r.h <= 0 {
                continue;
            }
            match draw {
                0 => Self::anchors(&mut ui, &m, r),
                1 => Self::shares(&mut ui, &m, r),
                2 => self.stack(&mut ui, &m, r),
                _ => self.picked(&mut ui, &m, r),
            }
        }
        self.w = ui.widgets;
    }
    fn state(&self, _cx: &Cx<G>) -> Value {
        json!({"arrange": self.arrange.map(Arrange::name), "pick": self.inner.map(Arrange::name), "overflow": self.overflow})
    }
    widget_plumbing!(Cx<G>);
}
