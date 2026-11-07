//! The layout audit (LAYOUT720-E5, spec section 10): what one frame of a screen on `Grid::Native` does wrong with its
//! space, read without eyes. It sits beside `text_audit`, reads the same frame and answers under its own key, so the text
//! audit's counts stay as they are. The painter and `Ui` record what they place and squeeze while `Fonts::layout` is on
//! (`LayoutRec`); `issues` reads the records and the screen's widgets. Size profiles (`PROFILES`) run a screen on other
//! windows and canvases: `desktop` must pass, the others only report, until the minimums of section 11.1 exist.
//!
//! The kinds, each an issue with `kind`, `what` (a widget id, a frame role or what the screen reported), `at` (`[x, y, w,
//! h]` in the screen's units) and `why` (one sentence with the numbers):
//!
//! - `overflow`: a stack or list needs more room than its area; `by` is how many units (`Ui::report_overflow`).
//! - `off_canvas`: a widget, frame or image runs past the canvas; `by` is the most units past any edge.
//! - `zero_size`: a widget, frame, frame body or image got no room, so it does not show; where caps will be needed.
//! - `corner_shrunk`: a panel too small for its frame's corners; `what` is the frame's role, `widget` the smallest widget
//!   at its middle and `share` the part of their size the corners draw at.
//! - `hover_only`: a tip on a widget that can take neither the keyboard's focus nor a press, so only a mouse reads it.

use crate::canvas::Rect;
use crate::painter::{CanvasPolicy, Grid, TextRec, View};
use crate::screen::{Cx, Screen, TEXT_SIZES};
use crate::text_audit;
use crate::ui::{size_classes, Gfx, Pointer};
use aegis_layout::SizeClass;
use serde_json::{json, Value};

/// Every kind, in the order counts and lists hold them.
pub const KINDS: [&str; 5] = ["overflow", "off_canvas", "zero_size", "corner_shrunk", "hover_only"];
/// Problems listed per answer (the counts are always complete).
pub const LIST_MAX: usize = 60;
/// Where a frame's mouse sits while it is audited: off every widget, so no hover card or hot state.
pub const NO_MOUSE: (i32, i32) = (-500, -500);

/// What a frame recorded while the audit was on, in the screen's units.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutRec {
    /// A frame, frame body or image was given `at`.
    Placed { what: String, at: Rect },
    /// A widget was pushed with no room, before any scroll area's clip (a row scrolled out of its area is not a problem).
    NoRoom { what: String, at: Rect },
    /// A stack or list in `at` needs `by` units more.
    Overflow { what: String, at: Rect, by: i32 },
    /// A frame's corners draw at `share` of their size on `at`. `what`: the role, "" where only the painter knew.
    CornerShrunk { what: String, at: Rect, share: f32 },
}

fn rect_of(w: &Value) -> Rect {
    let n = |i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
    Rect::new(n(0), n(1), n(2), n(3))
}

/// The most units `r` runs past any edge of the canvas, 0 inside it. An empty rect is `zero_size`'s, not this kind's.
fn past_canvas(r: Rect, (w, h): (i32, i32)) -> i32 {
    if r.w <= 0 || r.h <= 0 {
        return 0;
    }
    (-r.x).max(-r.y).max(r.x + r.w - w).max(r.y + r.h - h).max(0)
}

/// The smallest widget holding the middle of `r`: what an agent looks up for a frame the painter drew.
fn holder(r: Rect, widgets: &[Value]) -> Option<&str> {
    let (mx, my) = (r.x + r.w / 2, r.y + r.h / 2);
    widgets.iter().filter(|w| rect_of(w).contains(mx, my)).min_by_key(|w| {
        let b = rect_of(w);
        i64::from(b.w) * i64::from(b.h)
    }).and_then(|w| w["id"].as_str())
}

fn issue(kind: &str, what: &str, at: Rect, why: String) -> Value {
    json!({"kind": kind, "what": what, "at": [at.x, at.y, at.w, at.h], "why": why})
}

/// The problems of one frame on a `canvas`, in the screen's units, from what it recorded (`recs`) and its widgets
/// (`Screen::widgets`): {audited: true, canvas, widgets, clean, counts: {kind: n}, issues: [...]}, the issues in `KINDS`
/// order (the kinds are in the module's notes).
pub fn issues(recs: &[LayoutRec], widgets: &[Value], canvas: (i32, i32)) -> Value {
    answer(all_issues(recs, widgets, canvas), widgets.len(), canvas)
}

/// The answer for one frame: the counts of all of `found`, and a bounded list.
fn answer(mut found: Vec<Value>, widgets: usize, (cw, ch): (i32, i32)) -> Value {
    let counts: serde_json::Map<String, Value> = KINDS.iter().map(|k| (k.to_string(), json!(found.iter().filter(|i| i["kind"] == *k).count()))).collect();
    let clean = found.is_empty();
    found.truncate(LIST_MAX);
    json!({"audited": true, "canvas": [cw, ch], "widgets": widgets, "clean": clean, "counts": counts, "issues": found})
}

/// Every problem of one frame, in `KINDS` order.
fn all_issues(recs: &[LayoutRec], widgets: &[Value], canvas: (i32, i32)) -> Vec<Value> {
    let mut found: Vec<Value> = Vec::new();
    for rec in recs {
        match rec {
            LayoutRec::Overflow { what, at, by } if *by > 0 => {
                let mut one = issue("overflow", what, *at, format!("needs {by} units more than its {} x {} area", at.w, at.h));
                one["by"] = json!(by);
                found.push(one);
            }
            LayoutRec::CornerShrunk { what, at, share } => {
                let name = if what.is_empty() { "frame" } else { what };
                let mut one = issue("corner_shrunk", name, *at, format!("a {} x {} panel draws its frame's corners at {:.0} % of their size", at.w, at.h, share * 100.0));
                one["share"] = json!((share * 100.0).round() / 100.0);
                one["widget"] = json!(holder(*at, widgets));
                found.push(one);
            }
            LayoutRec::NoRoom { what, at } => found.push(issue("zero_size", what, *at, format!("given {} x {} units, so it does not show", at.w, at.h))),
            LayoutRec::Placed { what, at } if at.w <= 0 || at.h <= 0 => found.push(issue("zero_size", what, *at, format!("given {} x {} units, so it does not show", at.w, at.h))),
            LayoutRec::Placed { what, at } => push_past(&mut found, what, *at, canvas),
            LayoutRec::Overflow { .. } => {}
        }
    }
    for w in widgets {
        let (r, id) = (rect_of(w), w["id"].as_str().unwrap_or(""));
        push_past(&mut found, id, r, canvas);
        let tip = w["tip"].as_array().and_then(|t| t.first());
        if let Some(tip) = tip.filter(|_| r.w > 0 && r.h > 0 && !crate::focus::focusable(w, false)) {
            found.push(issue("hover_only", id, r, format!("its tip ({tip}) shows only under a mouse: it takes no focus or press")));
        }
    }
    found.sort_by_key(|i| KINDS.iter().position(|k| i["kind"] == *k));
    found
}

fn push_past(found: &mut Vec<Value>, what: &str, at: Rect, canvas: (i32, i32)) {
    let by = past_canvas(at, canvas);
    if by > 0 {
        let mut one = issue("off_canvas", what, at, format!("runs up to {by} units past the {} x {} canvas", canvas.0, canvas.1));
        one["by"] = json!(by);
        found.push(one);
    }
}

/// The answer for a screen the audit does not read: `Old` and `Quadrant` screens are written in old numbers.
pub fn not_audited(grid: Grid) -> Value {
    json!({"audited": false, "why": format!("the screen is on the {} grid; only native screens are audited for layout", grid.name())})
}

/// How a profile relates its canvas to the window; a game's own `w` and `h` come in from its policy (`Profile::policy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfilePolicy {
    Fixed,
    Fit,
    Dp,
}

/// A window and device a screen is checked on (LAYOUT720 section 10).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Profile {
    pub name: &'static str,
    pub window: (u32, u32),
    pub dpr: f32,
    pub ui_scale: f32,
    pub pointer: Pointer,
    pub policy: ProfilePolicy,
    /// A native screen must pass this one; the others report.
    pub must_pass: bool,
}

const fn profile(name: &'static str, window: (u32, u32), dpr: f32, ui_scale: f32, pointer: Pointer, policy: ProfilePolicy, must_pass: bool) -> Profile {
    Profile { name, window, dpr, ui_scale, pointer, policy, must_pass }
}

pub const PROFILES: [Profile; 7] = [
    profile("desktop", (1280, 720), 1.0, 1.0, Pointer::Mouse, ProfilePolicy::Fixed, true),
    profile("wide", (1920, 1200), 1.0, 1.0, Pointer::Mouse, ProfilePolicy::Fit, false),
    profile("ultrawide", (3440, 1440), 1.0, 1.0, Pointer::Mouse, ProfilePolicy::Fit, false),
    profile("handheld", (1280, 800), 1.0, 1.5, Pointer::Touch, ProfilePolicy::Fit, false),
    profile("browser_small", (800, 600), 1.0, 1.0, Pointer::Mouse, ProfilePolicy::Dp, false),
    profile("phone_landscape", (2532, 1170), 3.0, 1.0, Pointer::Touch, ProfilePolicy::Dp, false),
    profile("phone_portrait", (1170, 2532), 3.0, 1.0, Pointer::Touch, ProfilePolicy::Dp, false),
];

/// The profile named `name`, or why not.
pub fn profile_named(name: &str) -> Result<&'static Profile, String> {
    PROFILES.iter().find(|p| p.name == name).ok_or_else(|| format!("unknown profile '{name}' ({})", PROFILES.map(|p| p.name).join(", ")))
}

/// What `Profile::enter` changed, to put back.
#[derive(Debug)]
pub struct Entered {
    window: (i32, i32),
    device: crate::ui::Device,
    view: View,
    /// A widget a smaller canvas has no room for loses the focus or its open list: both come back with the window.
    focus: Option<String>,
    focus_visible: bool,
    open: Option<String>,
}

impl Profile {
    /// This profile's policy for a game whose own is `game`: its `w` and `h` kept, `Dp` having none.
    pub fn policy_for(&self, game: CanvasPolicy) -> CanvasPolicy {
        let (w, h) = match game {
            CanvasPolicy::Fixed { w, h } | CanvasPolicy::Fit { w, h } => (w, h),
            CanvasPolicy::Dp => {
                let CanvasPolicy::Fixed { w, h } = CanvasPolicy::default() else { unreachable!("the default is Fixed") };
                (w, h)
            }
        };
        match self.policy {
            ProfilePolicy::Fixed => CanvasPolicy::Fixed { w, h },
            ProfilePolicy::Fit => CanvasPolicy::Fit { w, h },
            ProfilePolicy::Dp => CanvasPolicy::Dp,
        }
    }

    /// The view of a native screen under this profile.
    pub fn view(&self, game: CanvasPolicy) -> View {
        View::native(self.policy_for(game), self.window.0, self.window.1, self.dpr, self.ui_scale)
    }

    /// `g` takes this profile's window and device, and the view of a native screen; `Entered::leave` puts them back, with
    /// the keyboard's focus and any open list as they were.
    pub fn enter(&self, g: &mut Gfx, game: CanvasPolicy) -> Entered {
        let was = Entered { window: (g.canvas.w, g.canvas.h), device: g.kit.device, view: g.view, focus: g.kit.focus.clone(), focus_visible: g.kit.focus_visible, open: g.kit.open.clone() };
        g.canvas = crate::canvas::Canvas::new(self.window.0 as i32, self.window.1 as i32);
        g.kit.device = crate::ui::Device { dpr: self.dpr, ui_scale: self.ui_scale, pointer: self.pointer };
        g.view = self.view(game);
        was
    }
}

impl Entered {
    /// Puts back what `Profile::enter` changed. The canvas comes back at its size but blank: the caller draws a frame.
    pub fn leave(self, g: &mut Gfx) {
        g.canvas = crate::canvas::Canvas::new(self.window.0, self.window.1);
        g.kit.device = self.device;
        g.view = self.view;
        g.kit.focus = self.focus;
        g.kit.focus_visible = self.focus_visible;
        g.kit.open = self.open;
    }
}

fn class_name(c: SizeClass) -> &'static str {
    match c {
        SizeClass::Compact => "compact",
        SizeClass::Medium => "medium",
        SizeClass::Expanded => "expanded",
    }
}

/// One frame's two audits: `report` as a screen's text audit reports (`text_audit::issues_on`, the widgets being its boxes
/// as in the shell's command) with the layout audit under `layout`, and `found`, every layout issue, which `sweep` reads
/// because the report's list is bounded.
pub struct FrameAudit {
    pub report: Value,
    pub found: Vec<Value>,
}

/// The audits of one frame.
pub fn frame_report(view: View, grid: Grid, texts: &[TextRec], recs: &[LayoutRec], widgets: &[Value]) -> FrameAudit {
    let boxes: Vec<(String, f32, f32, f32, f32)> = widgets.iter().filter(|w| w["kind"] != "scroll").map(|w| {
        let r = rect_of(w);
        (w["id"].as_str().unwrap_or("").to_string(), r.x as f32, r.y as f32, r.w as f32, r.h as f32)
    }).collect();
    let (cw, ch) = view.canvas();
    let mut report = text_audit::issues_on(texts, cw as f32, ch as f32, &boxes, view.g);
    if grid != Grid::Native {
        report["layout"] = not_audited(grid);
        return FrameAudit { report, found: Vec::new() };
    }
    let found = all_issues(recs, widgets, view.canvas());
    report["layout"] = answer(found.clone(), widgets.len(), view.canvas());
    FrameAudit { report, found }
}

/// One profile's frames at every text size, read together: the same problem at the same place at several text sizes is
/// one line, with the sizes it shows at, and the counts count those lines, all of them.
struct Merged {
    counts: serde_json::Map<String, Value>,
    text_counts: serde_json::Map<String, Value>,
    issues: Vec<Value>,
}

impl Merged {
    fn new() -> Self {
        Self { counts: KINDS.iter().map(|k| (k.to_string(), json!(0))).collect(), text_counts: serde_json::Map::new(), issues: Vec::new() }
    }

    fn add(&mut self, scale: f64, frame: &FrameAudit) {
        for one in &frame.found {
            let known = self.issues.iter_mut().find(|i| i["kind"] == one["kind"] && i["what"] == one["what"] && i["at"] == one["at"]);
            match known {
                Some(i) => {
                    let sizes = i["text_sizes"].as_array_mut().unwrap();
                    if sizes.last() != Some(&json!(scale)) {
                        sizes.push(json!(scale));
                    }
                }
                None => {
                    let mut new = one.clone();
                    new["text_sizes"] = json!([scale]);
                    let kind = one["kind"].as_str().unwrap_or("");
                    let n = self.counts[kind].as_u64().unwrap_or(0) + 1;
                    self.counts[kind] = json!(n);
                    self.issues.push(new);
                }
            }
        }
        for (k, n) in frame.report["counts"].as_object().unwrap_or(&serde_json::Map::new()) {
            let sum = self.text_counts.get(k).and_then(Value::as_u64).unwrap_or(0) + n.as_u64().unwrap_or(0);
            self.text_counts.insert(k.clone(), json!(sum));
        }
    }
}

/// Runs `frame` once per profile (`only`: that one) and text size, each giving what `frame_report` gives, and reads them
/// together: {audited: true, policy, pass, profiles: [{profile, must_pass, window, policy, dpr, ui_scale, pointer, canvas,
/// classes: [width, height], clean, counts, text_counts, issues: [{.., text_sizes}]}]}. `pass` is every `must_pass` profile
/// run being clean of layout issues; the others report only. `text_counts` adds up the text audit over the four sizes. The
/// counts are complete, the list bounded by `LIST_MAX`.
pub fn sweep(game: CanvasPolicy, only: Option<&Profile>, breakpoints: crate::theme::Breakpoints, mut frame: impl FnMut(&Profile, f32) -> FrameAudit) -> Value {
    let mut pass = true;
    let mut profiles = Vec::new();
    for p in PROFILES.iter().filter(|p| only.is_none_or(|o| o.name == p.name)) {
        let mut merged = Merged::new();
        for (_, scale) in TEXT_SIZES {
            merged.add(scale, &frame(p, scale as f32));
        }
        let clean = merged.issues.is_empty();
        pass &= clean || !p.must_pass;
        let view = p.view(game);
        let mut listed = merged.issues;
        listed.sort_by_key(|i| KINDS.iter().position(|k| i["kind"] == *k));
        listed.truncate(LIST_MAX);
        let (width, height) = size_classes(view.canvas, view.g, breakpoints);
        let classes = [class_name(width), class_name(height)];
        profiles.push(json!({"profile": p.name, "must_pass": p.must_pass, "window": [p.window.0, p.window.1], "policy": p.policy_for(game).name(), "dpr": p.dpr, "ui_scale": p.ui_scale,
            "pointer": if p.pointer == Pointer::Touch { "touch" } else { "mouse" }, "canvas": [view.canvas.0, view.canvas.1], "classes": classes, "clean": clean, "counts": merged.counts, "text_counts": merged.text_counts, "issues": listed}));
    }
    json!({"audited": true, "policy": game.name(), "pass": pass, "profiles": profiles})
}

/// A native screen alone, drawn with `mouse` through the profiles (`only`: that one) at every text size, with no shell:
/// what the shell's `layout_audit` command answers, for a test or a tool. `g` is left as it was found but for its picture,
/// which is blank: the caller draws a frame.
pub fn audit_screen<G>(screen: &mut dyn Screen<G>, cx: &mut Cx<G>, g: &mut Gfx, game: CanvasPolicy, mouse: (i32, i32), only: Option<&Profile>) -> Value {
    let grid = screen.grid();
    if grid != Grid::Native {
        return not_audited(grid);
    }
    let was_scale = g.fonts.scale;
    let breakpoints = crate::theme::Metrics::of(&crate::theme::th(), g.view).breakpoints;
    let out = sweep(game, only, breakpoints, |p, scale| {
        let entered = p.enter(g, game);
        g.fonts.scale = scale;
        g.fonts.audit = Some(Vec::new());
        g.fonts.layout = Some(Vec::new());
        screen.draw(cx, g, mouse, true);
        let (texts, recs) = (g.fonts.audit.take().unwrap_or_default(), g.fonts.layout.take().unwrap_or_default());
        let report = frame_report(g.view, grid, &texts, &recs, &screen.widgets());
        entered.leave(g);
        report
    });
    g.fonts.scale = was_scale;
    out
}
