//! PR5-48 (MAP-LAYOUT-ED): the layout editor, a pane in the mould of the frame tuner (`frame_tuner`). The caller hands it
//! a background image, a viewport (a rect and its padding) and markers (an id, an image, a label, a position and a
//! scale), with optional polylines (an id and its points). Positions are fractions of the padded viewport
//! (`Viewport::to_screen`: x = rect.x + pad.x + fx * (rect.w - 2 pad.x)) and are never clamped, so an edge anchor such as
//! 1.042 works. A marker's image is drawn in the box of the caller's ground-point rule (`MarkerBox`: its side at scale
//! 1, centred on the point, its bottom `foot` below it, so a larger marker grows upward).
//!
//! The pane draws the viewport geometry (its rect, the padded area positions span, a grid of fractions), drags markers and
//! polyline points (`Ui::handle` widgets, followed through `pointer`), inserts points (a segment's middle handle, or
//! Insert point) and removes them, nudges the selection with the arrows (Shift: coarse, `NUDGE`), sizes a marker with a
//! slider and + / - (`SCALE_RANGE`), toggles the grid, the labels and the ground-point rings, and undoes (Ctrl+Z, a
//! button): every edit is one undo step (a drag at its release; a slider drag on one marker is one step). It never
//! writes: each act returns the values it changed (`LayoutEdit`), `changes` holds what differs from the values loaded or
//! last saved (`dirty`), and Save, in a native debug build only (`writes_allowed`, as the frame tuner), leaves a request
//! the caller takes (`take_save`) and acknowledges (`mark_saved`). The agent link: `state` (a screen's
//! `layout_markers`, `STATE_KEY`) and `command` (`layout {id, pos, scale}`, `layout {route, points}`, `layout_markers`).

use crate::canvas::{rgb, rgba, Image, Rect, Rgba};
use crate::screen::{Key, PointerPhase};
use crate::ui::{tip, Ui, BODY, SMALL};
use serde_json::{json, Value};
use std::cell::Cell;
use std::rc::Rc;

pub use crate::frame_tuner::writes_allowed;

/// The key a screen's agent state keeps `LayoutEditor::state` under.
pub const STATE_KEY: &str = "layout_markers";
/// The lowest and highest marker scale an edit sets.
pub const SCALE_RANGE: (f32, f32) = (0.25, 4.0);
/// The scale slider's step and what + and - add.
pub const SCALE_STEP: f32 = 0.05;
/// An arrow key's nudge, in fractions of the padded viewport: (fine, coarse with Shift).
pub const NUDGE: (f32, f32) = (0.0025, 0.025);
/// The most undo steps kept (the oldest go first).
pub const UNDO_MAX: usize = 256;
/// The grid's spacing in fractions; 0, 0.5 and 1 are drawn brighter.
pub const GRID_STEP: f32 = 0.1;
/// The prefix of the pane's widget ids.
pub const ID: &str = "layout";

/// A position kept to 4 decimals (a saved file reads as typed, no float noise).
fn round4(v: f32) -> f32 {
    (v * 10_000.0).round() / 10_000.0
}

/// A scale kept to 3 decimals.
fn round3(v: f32) -> f32 {
    (v * 1_000.0).round() / 1_000.0
}

/// A value for the agent link and the read-out, at the precision it is kept to.
fn num(v: f32) -> f64 {
    (v as f64 * 10_000.0).round() / 10_000.0
}

fn pair(p: [f32; 2]) -> Value {
    json!([num(p[0]), num(p[1])])
}

/// A two-number position, both finite; None for anything else.
fn read_point(v: &Value) -> Option<[f32; 2]> {
    match v.as_array()?.as_slice() {
        [x, y] => {
            let (x, y) = (x.as_f64()? as f32, y.as_f64()? as f32);
            (x.is_finite() && y.is_finite()).then_some([x, y])
        }
        _ => None,
    }
}

/// The screen area markers live on: `rect` (logical px) and the padding (x, y) around the area positions 0..1 span.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub rect: Rect,
    pub pad: (f32, f32),
}

impl Default for Viewport {
    fn default() -> Self {
        Self { rect: Rect::new(0, 0, 320, 200), pad: (0.0, 0.0) }
    }
}

impl Viewport {
    pub fn new(rect: Rect, pad: (f32, f32)) -> Self {
        Self { rect, pad }
    }

    /// The largest rect of aspect `aspect` (w, h) centred in `stage`, padded by `pad_share` of its own size.
    pub fn fit(stage: Rect, aspect: (i32, i32), pad_share: (f32, f32)) -> Self {
        let (aw, ah) = (aspect.0.max(1) as f32, aspect.1.max(1) as f32);
        let k = (stage.w as f32 / aw).min(stage.h as f32 / ah).max(0.0);
        let (w, h) = ((aw * k).floor() as i32, (ah * k).floor() as i32);
        let rect = Rect::new(stage.x + (stage.w - w) / 2, stage.y + (stage.h - h) / 2, w, h);
        Self { rect, pad: ((w as f32 * pad_share.0).round(), (h as f32 * pad_share.1).round()) }
    }

    /// The area positions 0..1 span (x, y, w, h): the rect less its padding on each side.
    pub fn area(&self) -> (f32, f32, f32, f32) {
        let r = self.rect;
        (r.x as f32 + self.pad.0, r.y as f32 + self.pad.1, r.w as f32 - 2.0 * self.pad.0, r.h as f32 - 2.0 * self.pad.1)
    }

    /// The screen point (logical px) of position `p`.
    pub fn to_screen(&self, p: [f32; 2]) -> (f32, f32) {
        let (x, y, w, h) = self.area();
        (x + p[0] * w, y + p[1] * h)
    }

    /// The position of screen point `s`, the inverse of `to_screen` (an area with no size counts as 1 px).
    pub fn to_norm(&self, s: (f32, f32)) -> [f32; 2] {
        let (x, y, w, h) = self.area();
        [(s.0 - x) / w.max(1.0), (s.1 - y) / h.max(1.0)]
    }
}

/// The ground-point rule a marker is drawn by: a `side * scale` square centred on the point, its bottom `foot` below it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkerBox {
    pub side: f32,
    pub foot: f32,
}

impl Default for MarkerBox {
    fn default() -> Self {
        Self { side: 48.0, foot: 0.0 }
    }
}

impl MarkerBox {
    /// The box of a marker on ground point `g` (logical px) at `scale`.
    pub fn rect(&self, g: (f32, f32), scale: f32) -> Rect {
        let side = self.side * scale;
        Rect::new((g.0 - side / 2.0).round() as i32, (g.1 + self.foot - side).round() as i32, side.round() as i32, side.round() as i32)
    }
}

/// A placed picture: its id (the caller's key), image (None draws a pin), label, position and scale.
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub id: String,
    pub image: Option<Rc<Image>>,
    pub label: String,
    pub pos: [f32; 2],
    pub scale: f32,
}

impl Marker {
    pub fn new(id: &str, label: &str, pos: [f32; 2], scale: f32) -> Self {
        Self { id: id.into(), image: None, label: label.into(), pos, scale }
    }
}

/// A line through points (positions as markers have).
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    pub id: String,
    pub points: Vec<[f32; 2]>,
}

impl Polyline {
    pub fn new(id: &str, points: Vec<[f32; 2]>) -> Self {
        Self { id: id.into(), points }
    }
}

/// What is selected: a marker (index) or a polyline's point (polyline index, point index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sel {
    Marker(usize),
    Point(usize, usize),
}

/// A switch of what the stage shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Show {
    Grid,
    Labels,
    Rings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutAct {
    Select(Option<Sel>),
    /// The scale slider's step (`scale_at`).
    Slide(usize),
    /// + (1) or - (-1): `SCALE_STEP` on the selected marker's scale.
    ScaleStep(i32),
    /// An arrow: the direction (x, y) and whether Shift was held (coarse).
    Nudge(i32, i32, bool),
    Toggle(Show),
    /// The polyline Insert point works on (index).
    Line(usize),
    InsertPoint,
    RemovePoint,
    Undo,
    Save,
}

/// A changed value, as the caller keeps it: a marker's position and scale, or a polyline's points.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutEdit {
    Marker { id: String, pos: [f32; 2], scale: f32 },
    Points { id: String, points: Vec<[f32; 2]> },
}

impl LayoutEdit {
    pub fn to_json(&self) -> Value {
        match self {
            LayoutEdit::Marker { id, pos, scale } => json!({"id": id, "pos": pair(*pos), "scale": num(*scale)}),
            LayoutEdit::Points { id, points } => json!({"route": id, "points": points.iter().map(|p| pair(*p)).collect::<Vec<_>>()}),
        }
    }
}

/// The scale slider's steps.
pub fn scale_steps() -> usize {
    ((SCALE_RANGE.1 - SCALE_RANGE.0) / SCALE_STEP).round() as usize + 1
}

/// The scale at slider step `i` (past the last, the last).
pub fn scale_at(i: usize) -> f32 {
    round3(SCALE_RANGE.0 + i.min(scale_steps() - 1) as f32 * SCALE_STEP)
}

/// The slider step nearest scale `s`, inside the range.
pub fn step_of(s: f32) -> usize {
    (((s - SCALE_RANGE.0) / SCALE_STEP).round().max(0.0) as usize).min(scale_steps() - 1)
}

/// `s` inside `SCALE_RANGE`, at the kept precision.
pub fn clamp_scale(s: f32) -> f32 {
    round3(s.clamp(SCALE_RANGE.0, SCALE_RANGE.1))
}

/// What an undo step puts back: a marker's values (`slide`: set by the slider, so the next slider move on it joins the
/// step) or a polyline's points.
#[derive(Debug, Clone, PartialEq)]
enum Step {
    Marker { i: usize, pos: [f32; 2], scale: f32, slide: bool },
    Points { i: usize, points: Vec<[f32; 2]> },
}

/// A drag in progress: what is dragged, the grab's offset from the dragged point (screen px) and the values before.
#[derive(Debug, Clone, PartialEq)]
struct Grab {
    sel: Sel,
    offset: (f32, f32),
    before: Step,
}

#[derive(Debug, Clone)]
pub struct LayoutEditor {
    markers: Vec<Marker>,
    polylines: Vec<Polyline>,
    /// The values loaded or last saved, for `changes`.
    base_markers: Vec<([f32; 2], f32)>,
    base_lines: Vec<Vec<[f32; 2]>>,
    pub background: Option<Rc<Image>>,
    pub marker_box: MarkerBox,
    /// The viewport last set (`set_view`); a pointer is read through it.
    view: Cell<Viewport>,
    pub selected: Option<Sel>,
    /// The polyline Insert point works on when no point of one is selected.
    pub line: usize,
    pub grid: bool,
    pub labels: bool,
    pub rings: bool,
    /// What the last action did, for the pane's foot.
    pub note: String,
    undo: Vec<Step>,
    grab: Option<Grab>,
    save: Option<Vec<LayoutEdit>>,
}

impl LayoutEditor {
    pub fn new(markers: Vec<Marker>, polylines: Vec<Polyline>) -> Self {
        let base_markers = markers.iter().map(|m| (m.pos, m.scale)).collect();
        let base_lines = polylines.iter().map(|l| l.points.clone()).collect();
        Self {
            markers,
            polylines,
            base_markers,
            base_lines,
            background: None,
            marker_box: MarkerBox::default(),
            view: Cell::new(Viewport::default()),
            selected: None,
            line: 0,
            grid: true,
            labels: true,
            rings: true,
            note: String::new(),
            undo: Vec::new(),
            grab: None,
            save: None,
        }
    }

    pub fn with_view(self, v: Viewport) -> Self {
        self.view.set(v);
        self
    }

    /// The viewport the stage draws and a pointer is read through (a screen sets it where it draws, even mid-frame).
    pub fn set_view(&self, v: Viewport) {
        self.view.set(v);
    }

    pub fn view(&self) -> Viewport {
        self.view.get()
    }

    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    pub fn polylines(&self) -> &[Polyline] {
        &self.polylines
    }

    pub fn marker(&self, id: &str) -> Option<&Marker> {
        self.markers.iter().find(|m| m.id == id)
    }

    pub fn polyline(&self, id: &str) -> Option<&Polyline> {
        self.polylines.iter().find(|l| l.id == id)
    }

    /// The image marker `id` draws (None: a pin); false when there is no such marker. Not an edit.
    pub fn set_image(&mut self, id: &str, image: Option<Rc<Image>>) -> bool {
        match self.markers.iter_mut().find(|m| m.id == id) {
            Some(m) => {
                m.image = image;
                true
            }
            None => false,
        }
    }

    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Whether a drag is in progress.
    pub fn dragging(&self) -> bool {
        self.grab.is_some()
    }

    /// Every marker and polyline whose values differ from those loaded or last saved, at their values now.
    pub fn changes(&self) -> Vec<LayoutEdit> {
        let marks = self.markers.iter().enumerate().filter(|(i, m)| self.base_markers.get(*i) != Some(&(m.pos, m.scale))).map(|(i, _)| self.marker_edit(i));
        let lines = self.polylines.iter().enumerate().filter(|(i, l)| self.base_lines.get(*i) != Some(&l.points)).map(|(i, _)| self.line_edit(i));
        marks.chain(lines).collect()
    }

    /// Whether anything differs from the values loaded or last saved.
    pub fn dirty(&self) -> bool {
        !self.changes().is_empty()
    }

    fn marker_edit(&self, i: usize) -> LayoutEdit {
        let m = &self.markers[i];
        LayoutEdit::Marker { id: m.id.clone(), pos: m.pos, scale: m.scale }
    }

    fn line_edit(&self, i: usize) -> LayoutEdit {
        let l = &self.polylines[i];
        LayoutEdit::Points { id: l.id.clone(), points: l.points.clone() }
    }

    fn marker_step(&self, i: usize, slide: bool) -> Step {
        Step::Marker { i, pos: self.markers[i].pos, scale: self.markers[i].scale, slide }
    }

    fn line_step(&self, i: usize) -> Step {
        Step::Points { i, points: self.polylines[i].points.clone() }
    }

    fn push(&mut self, s: Step) {
        self.undo.push(s);
        if self.undo.len() > UNDO_MAX {
            self.undo.remove(0);
        }
    }

    /// Puts `s` back without an undo step; the edit it made.
    fn restore(&mut self, s: Step) -> Option<LayoutEdit> {
        match s {
            Step::Marker { i, pos, scale, .. } => {
                let m = self.markers.get_mut(i)?;
                (m.pos, m.scale) = (pos, scale);
                Some(self.marker_edit(i))
            }
            Step::Points { i, points } => {
                let l = self.polylines.get_mut(i)?;
                l.points = points;
                if let Some(Sel::Point(li, k)) = self.selected {
                    if li == i && k >= self.polylines[i].points.len() {
                        self.selected = self.polylines[i].points.len().checked_sub(1).map(|k| Sel::Point(i, k));
                    }
                }
                Some(self.line_edit(i))
            }
        }
    }

    /// The selected marker's index.
    fn sel_marker(&self) -> Option<usize> {
        match self.selected {
            Some(Sel::Marker(i)) if i < self.markers.len() => Some(i),
            _ => None,
        }
    }

    /// The selected point (polyline, point).
    fn sel_point(&self) -> Option<(usize, usize)> {
        match self.selected {
            Some(Sel::Point(li, k)) if self.polylines.get(li).is_some_and(|l| k < l.points.len()) => Some((li, k)),
            _ => None,
        }
    }

    fn sel_name(&self) -> String {
        if let Some(i) = self.sel_marker() {
            return format!("marker {}", self.markers[i].id);
        }
        match self.sel_point() {
            Some((li, k)) => format!("point {k} of {}", self.polylines[li].id),
            None => "nothing".into(),
        }
    }

    /// Sets marker `i`'s scale to `s` (clamped) as one undo step (a slider move on the marker the last step slid joins
    /// it). None when nothing changed.
    fn scale_to(&mut self, i: usize, s: f32, slide: bool) -> Option<LayoutEdit> {
        let s = clamp_scale(s);
        if self.markers[i].scale == s {
            return None;
        }
        let joins = slide && matches!(self.undo.last(), Some(Step::Marker { i: at, slide: true, .. }) if *at == i);
        if !joins {
            let step = self.marker_step(i, slide);
            self.push(step);
        }
        self.markers[i].scale = s;
        Some(self.marker_edit(i))
    }

    /// Moves the selection by (`dx`, `dy`) fractions as one undo step.
    fn nudge(&mut self, dx: f32, dy: f32) -> Option<LayoutEdit> {
        if let Some(i) = self.sel_marker() {
            let step = self.marker_step(i, false);
            self.push(step);
            let m = &mut self.markers[i];
            m.pos = [round4(m.pos[0] + dx), round4(m.pos[1] + dy)];
            return Some(self.marker_edit(i));
        }
        let (li, k) = self.sel_point()?;
        let step = self.line_step(li);
        self.push(step);
        let p = &mut self.polylines[li].points[k];
        *p = [round4(p[0] + dx), round4(p[1] + dy)];
        Some(self.line_edit(li))
    }

    /// Inserts a point into polyline `li` at `at` (one undo step) and selects it.
    fn insert_at(&mut self, li: usize, at: usize, p: [f32; 2]) -> LayoutEdit {
        let step = self.line_step(li);
        self.push(step);
        let pts = &mut self.polylines[li].points;
        let at = at.min(pts.len());
        pts.insert(at, [round4(p[0]), round4(p[1])]);
        self.selected = Some(Sel::Point(li, at));
        self.line = li;
        self.line_edit(li)
    }

    /// Insert point: after the selected point of a polyline (halfway to the next, or half a segment on past the last),
    /// else at the end of the picked polyline; an empty polyline starts at the middle.
    fn insert_point(&mut self) -> Option<LayoutEdit> {
        let (li, k) = match self.sel_point() {
            Some(at) => at,
            None => {
                let li = self.line.min(self.polylines.len().checked_sub(1)?);
                (li, self.polylines[li].points.len().wrapping_sub(1))
            }
        };
        let pts = &self.polylines[li].points;
        let p = match (pts.get(k), pts.get(k.wrapping_add(1)), k.checked_sub(1).and_then(|j| pts.get(j))) {
            (Some(a), Some(b), _) => [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0],
            (Some(a), None, Some(z)) => [a[0] + (a[0] - z[0]) / 2.0, a[1] + (a[1] - z[1]) / 2.0],
            (Some(a), None, None) => [a[0] + 0.05, a[1]],
            _ => [0.5, 0.5],
        };
        Some(self.insert_at(li, k.wrapping_add(1), p))
    }

    /// Remove point: the selected point goes (one undo step), the selection to the one before it.
    fn remove_point(&mut self) -> Option<LayoutEdit> {
        let (li, k) = self.sel_point()?;
        let step = self.line_step(li);
        self.push(step);
        self.polylines[li].points.remove(k);
        let n = self.polylines[li].points.len();
        self.selected = (n > 0).then(|| Sel::Point(li, k.saturating_sub(1).min(n - 1)));
        Some(self.line_edit(li))
    }

    /// Takes back the last edit; the values it put back.
    pub fn undo(&mut self) -> Option<LayoutEdit> {
        let s = self.undo.pop()?;
        self.restore(s)
    }

    pub fn act(&mut self, a: LayoutAct) -> Vec<LayoutEdit> {
        let edit = match a {
            LayoutAct::Select(s) => {
                self.selected = s.filter(|s| match *s {
                    Sel::Marker(i) => i < self.markers.len(),
                    Sel::Point(li, k) => self.polylines.get(li).is_some_and(|l| k < l.points.len()),
                });
                if let Some(Sel::Point(li, _)) = self.selected {
                    self.line = li;
                }
                self.note = format!("selected {}", self.sel_name());
                None
            }
            LayoutAct::Slide(step) => {
                let e = self.sel_marker().and_then(|i| self.scale_to(i, scale_at(step), true));
                self.note = self.scale_note(e.is_some());
                e
            }
            LayoutAct::ScaleStep(d) => {
                let e = self.sel_marker().and_then(|i| self.scale_to(i, self.markers[i].scale + d.signum() as f32 * SCALE_STEP, false));
                self.note = self.scale_note(e.is_some());
                e
            }
            LayoutAct::Nudge(dx, dy, coarse) => {
                let n = if coarse { NUDGE.1 } else { NUDGE.0 };
                let e = self.nudge(dx.signum() as f32 * n, dy.signum() as f32 * n);
                self.note = match &e {
                    Some(_) => format!("nudged {} {}", self.sel_name(), if coarse { "(coarse)" } else { "" }).trim_end().to_string(),
                    None => "select a marker or a point to nudge".into(),
                };
                e
            }
            LayoutAct::Toggle(s) => {
                let (flag, name) = match s {
                    Show::Grid => (&mut self.grid, "grid"),
                    Show::Labels => (&mut self.labels, "labels"),
                    Show::Rings => (&mut self.rings, "ground-point rings"),
                };
                *flag = !*flag;
                self.note = format!("{name} {}", if *flag { "on" } else { "off" });
                None
            }
            LayoutAct::Line(i) => {
                if !self.polylines.is_empty() {
                    self.line = i % self.polylines.len();
                    self.note = format!("polyline {}", self.polylines[self.line].id);
                }
                None
            }
            LayoutAct::InsertPoint => {
                let e = self.insert_point();
                self.note = if e.is_some() { format!("inserted {}", self.sel_name()) } else { "no polyline to insert into".into() };
                e
            }
            LayoutAct::RemovePoint => {
                let gone = self.sel_name();
                let e = self.remove_point();
                self.note = if e.is_some() { format!("removed {gone}") } else { "select a point to remove".into() };
                e
            }
            LayoutAct::Undo => {
                let e = self.undo();
                self.note = if e.is_some() { format!("undone ({} left)", self.undo.len()) } else { "nothing to undo".into() };
                e
            }
            LayoutAct::Save => {
                self.note = self.request_save().unwrap_or_else(|e| e);
                None
            }
        };
        edit.into_iter().collect()
    }

    fn scale_note(&self, changed: bool) -> String {
        match self.sel_marker() {
            Some(i) if changed => format!("{} scale {}", self.markers[i].id, num(self.markers[i].scale)),
            Some(i) => format!("{} scale stays {}", self.markers[i].id, num(self.markers[i].scale)),
            None => "select a marker to size".into(),
        }
    }

    /// The keys beyond the shell's focus: the arrows nudge the selection (Shift: coarse), + and - size the selected
    /// marker, Ctrl+Z undoes, PageUp and PageDown select the previous or next marker. True when the key was used.
    pub fn key(&mut self, k: Key, shift: bool, ctrl: bool) -> bool {
        let has = self.sel_marker().is_some() || self.sel_point().is_some();
        let a = match k {
            Key::Up if has => LayoutAct::Nudge(0, -1, shift),
            Key::Down if has => LayoutAct::Nudge(0, 1, shift),
            Key::Left if has => LayoutAct::Nudge(-1, 0, shift),
            Key::Right if has => LayoutAct::Nudge(1, 0, shift),
            Key::Char('+' | '=') if self.sel_marker().is_some() => LayoutAct::ScaleStep(1),
            Key::Char('-' | '_') if self.sel_marker().is_some() => LayoutAct::ScaleStep(-1),
            Key::Char('z' | 'Z') if ctrl => LayoutAct::Undo,
            Key::PageUp | Key::PageDown if !self.markers.is_empty() => {
                let n = self.markers.len();
                let i = match (self.sel_marker(), k) {
                    (Some(i), Key::PageUp) => (i + n - 1) % n,
                    (Some(i), _) => (i + 1) % n,
                    (None, Key::PageUp) => n - 1,
                    (None, _) => 0,
                };
                LayoutAct::Select(Some(Sel::Marker(i)))
            }
            _ => return false,
        };
        self.act(a);
        true
    }

    /// The selection's point on screen.
    fn screen_of(&self, s: Sel) -> Option<(f32, f32)> {
        let v = self.view();
        match s {
            Sel::Marker(i) => self.markers.get(i).map(|m| v.to_screen(m.pos)),
            Sel::Point(li, k) => self.polylines.get(li).and_then(|l| l.points.get(k)).map(|p| v.to_screen(*p)),
        }
    }

    fn line_by_id(&self, id: &str) -> Option<usize> {
        self.polylines.iter().position(|l| l.id == id)
    }

    /// A pointer drag on one of the pane's handles (`Screen::pointer`, positions in logical px through `view`): a press on
    /// a marker or a point selects it and grabs it, a press on a segment's middle inserts a point there and grabs it; moves
    /// follow the pointer; the release makes the drag one undo step (none when it ended where it started); a cancel puts
    /// it back. True when the event was the pane's.
    pub fn pointer(&mut self, id: &str, phase: PointerPhase, pos: (i32, i32)) -> bool {
        let at = (pos.0 as f32, pos.1 as f32);
        match phase {
            PointerPhase::Down => {
                let Some(rest) = id.strip_prefix(ID).and_then(|r| r.strip_prefix('.')) else { return false };
                let sel = if let Some(mid) = rest.strip_prefix("marker.") {
                    let Some(i) = self.markers.iter().position(|m| m.id == mid) else { return false };
                    Sel::Marker(i)
                } else if let Some((k, line)) = rest.strip_prefix("point.").and_then(|r| r.split_once('.')) {
                    let (Ok(k), Some(li)) = (k.parse::<usize>(), self.line_by_id(line)) else { return false };
                    if k >= self.polylines[li].points.len() {
                        return false;
                    }
                    Sel::Point(li, k)
                } else if let Some((k, line)) = rest.strip_prefix("mid.").and_then(|r| r.split_once('.')) {
                    let (Ok(k), Some(li)) = (k.parse::<usize>(), self.line_by_id(line)) else { return false };
                    let pts = &self.polylines[li].points;
                    let (Some(a), Some(b)) = (pts.get(k), pts.get(k + 1)) else { return false };
                    let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
                    self.insert_at(li, k + 1, mid);
                    Sel::Point(li, k + 1)
                } else {
                    return false;
                };
                self.act(LayoutAct::Select(Some(sel)));
                let Some(p) = self.screen_of(sel) else { return false };
                let before = match sel {
                    Sel::Marker(i) => self.marker_step(i, false),
                    Sel::Point(li, _) => self.line_step(li),
                };
                self.grab = Some(Grab { sel, offset: (p.0 - at.0, p.1 - at.1), before });
                true
            }
            PointerPhase::Move => {
                let Some(g) = self.grab.clone() else { return false };
                let n = self.view().to_norm((at.0 + g.offset.0, at.1 + g.offset.1));
                let n = [round4(n[0]), round4(n[1])];
                match g.sel {
                    Sel::Marker(i) => self.markers[i].pos = n,
                    Sel::Point(li, k) => self.polylines[li].points[k] = n,
                }
                true
            }
            PointerPhase::Up => {
                if self.grab.is_some() {
                    self.pointer(id, PointerPhase::Move, pos);
                }
                let Some(g) = self.grab.take() else { return false };
                let now = match g.sel {
                    Sel::Marker(i) => self.marker_step(i, false),
                    Sel::Point(li, _) => self.line_step(li),
                };
                if now != g.before {
                    self.push(g.before);
                    self.note = format!("moved {}", self.sel_name());
                }
                true
            }
            PointerPhase::Cancel => {
                let Some(g) = self.grab.take() else { return false };
                self.restore(g.before);
                self.note = "drag cancelled".into();
                true
            }
        }
    }

    /// Sets marker `id`'s position and / or scale (clamped to `SCALE_RANGE`) as one undo step; the values it has then.
    pub fn set_marker(&mut self, id: &str, pos: Option<[f32; 2]>, scale: Option<f32>) -> Result<LayoutEdit, String> {
        let i = self.markers.iter().position(|m| m.id == id).ok_or_else(|| format!("no marker '{id}'"))?;
        if pos.is_some_and(|p| !p[0].is_finite() || !p[1].is_finite()) || scale.is_some_and(|s| !s.is_finite()) {
            return Err(format!("marker '{id}': positions and scale must be finite numbers"));
        }
        let m = &self.markers[i];
        let pos = pos.map_or(m.pos, |p| [round4(p[0]), round4(p[1])]);
        let scale = scale.map_or(m.scale, clamp_scale);
        if (pos, scale) != (m.pos, m.scale) {
            let step = self.marker_step(i, false);
            self.push(step);
            (self.markers[i].pos, self.markers[i].scale) = (pos, scale);
        }
        Ok(self.marker_edit(i))
    }

    /// Sets polyline `id`'s points as one undo step; the points it has then.
    pub fn set_points(&mut self, id: &str, points: Vec<[f32; 2]>) -> Result<LayoutEdit, String> {
        let li = self.line_by_id(id).ok_or_else(|| format!("no polyline '{id}'"))?;
        if points.iter().any(|p| !p[0].is_finite() || !p[1].is_finite()) {
            return Err(format!("polyline '{id}': points must be finite numbers"));
        }
        let points: Vec<[f32; 2]> = points.iter().map(|p| [round4(p[0]), round4(p[1])]).collect();
        if points != self.polylines[li].points {
            let step = self.line_step(li);
            self.push(step);
            self.polylines[li].points = points;
            if matches!(self.selected, Some(Sel::Point(sl, k)) if sl == li && k >= self.polylines[li].points.len()) {
                self.selected = None;
            }
        }
        Ok(self.line_edit(li))
    }

    /// Save, under this build's rule.
    pub fn request_save(&mut self) -> Result<String, String> {
        self.request_save_as(writes_allowed())
    }

    /// Leaves a save request with the changed values for the caller (`take_save`); refused unless `allowed` and this is a
    /// native debug build. The pane writes nothing itself.
    pub fn request_save_as(&mut self, allowed: bool) -> Result<String, String> {
        if !(allowed && writes_allowed()) {
            return Err("a release build shows the layout only: nothing saved".into());
        }
        let changes = self.changes();
        if changes.is_empty() {
            return Ok("nothing changed, nothing to save".into());
        }
        let n = changes.len();
        self.save = Some(changes);
        Ok(format!("save requested ({n} changed)"))
    }

    /// The pending save request, taken (the caller writes it, then calls `mark_saved`).
    pub fn take_save(&mut self) -> Option<Vec<LayoutEdit>> {
        self.save.take()
    }

    /// Whether a save request waits for the caller.
    pub fn save_pending(&self) -> bool {
        self.save.is_some()
    }

    /// The caller wrote the values: they become what `changes` compares with. The undo steps stay.
    pub fn mark_saved(&mut self) {
        self.base_markers = self.markers.iter().map(|m| (m.pos, m.scale)).collect();
        self.base_lines = self.polylines.iter().map(|l| l.points.clone()).collect();
        self.note = "saved".into();
    }

    fn sel_json(&self) -> Value {
        if let Some(i) = self.sel_marker() {
            return json!({"marker": self.markers[i].id});
        }
        match self.sel_point() {
            Some((li, k)) => json!({"route": self.polylines[li].id, "point": k}),
            None => Value::Null,
        }
    }

    /// The agent link's view of the pane (a screen's `layout_markers`).
    pub fn state(&self) -> Value {
        let v = self.view();
        json!({
            "markers": self.markers.iter().map(|m| json!({"id": m.id, "label": m.label, "pos": pair(m.pos), "scale": num(m.scale), "image": m.image.is_some()})).collect::<Vec<_>>(),
            "routes": self.polylines.iter().map(|l| json!({"id": l.id, "points": l.points.iter().map(|p| pair(*p)).collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "selected": self.sel_json(),
            "route": self.polylines.get(self.line).map(|l| l.id.clone()),
            "grid": self.grid, "labels": self.labels, "rings": self.rings,
            "dirty": self.dirty(), "undo": self.undo.len(), "dragging": self.grab.is_some(), "save_pending": self.save.is_some(),
            "writes": writes_allowed(), "note": self.note,
            "view": {"rect": [v.rect.x, v.rect.y, v.rect.w, v.rect.h], "pad": [num(v.pad.0), num(v.pad.1)]},
        })
    }

    /// The pane's agent commands (a screen's `Screen::command` hands them on): `{"cmd": "layout_markers"}` (the state),
    /// `{"cmd": "layout", "id", "pos"?, "scale"?}` (a marker) and `{"cmd": "layout", "route", "points"}` (a polyline's
    /// points); each set is one undo step. None for any other command.
    pub fn command(&mut self, body: &Value) -> Option<Value> {
        match body["cmd"].as_str()? {
            STATE_KEY => Some(json!({"ok": true, STATE_KEY: self.state()})),
            "layout" => Some(self.layout_op(body).unwrap_or_else(|e| json!({"ok": false, "error": e}))),
            _ => None,
        }
    }

    fn layout_op(&mut self, body: &Value) -> Result<Value, String> {
        let (id, route) = (body.get("id").and_then(Value::as_str), body.get("route").and_then(Value::as_str));
        let mut clamped = false;
        let edit = match (id, route) {
            (Some(_), Some(_)) => return Err("layout takes either {id, pos, scale} or {route, points}, not both".into()),
            (None, Some(route)) => {
                let pts = body.get("points").and_then(Value::as_array).ok_or_else(|| format!("layout {{route: {route}}} needs points: [[x, y], ...]"))?;
                let points = pts.iter().map(read_point).collect::<Option<Vec<_>>>().ok_or_else(|| format!("route '{route}': every point must be two finite numbers"))?;
                self.set_points(route, points)?
            }
            (Some(id), None) => {
                let pos = match body.get("pos") {
                    None | Some(Value::Null) => None,
                    Some(p) => Some(read_point(p).ok_or_else(|| format!("marker '{id}': pos must be two finite numbers"))?),
                };
                let scale = match body.get("scale") {
                    None | Some(Value::Null) => None,
                    Some(s) => Some(s.as_f64().map(|s| s as f32).filter(|s| s.is_finite()).ok_or_else(|| format!("marker '{id}': scale must be a finite number"))?),
                };
                if pos.is_none() && scale.is_none() {
                    return Err(format!("layout {{id: {id}}} needs pos and / or scale"));
                }
                clamped = scale.is_some_and(|s| clamp_scale(s) != round3(s));
                self.set_marker(id, pos, scale)?
            }
            (None, None) => return Err("layout needs id (a marker) or route (a polyline)".into()),
        };
        self.note = format!("agent set {}", edit.to_json());
        Ok(json!({"ok": true, "edit": edit.to_json(), "clamped": clamped, "dirty": self.dirty(), "undo": self.undo.len()}))
    }

    /// The read-out: the selection with its values, and what the last action did.
    pub fn read_out(&self) -> Vec<String> {
        let what = if let Some(i) = self.sel_marker() {
            let m = &self.markers[i];
            format!("{} ({}): pos {}, {} scale {}", m.label, m.id, num(m.pos[0]), num(m.pos[1]), num(m.scale))
        } else if let Some((li, k)) = self.sel_point() {
            let p = self.polylines[li].points[k];
            format!("{} point {k}: {}, {}", self.polylines[li].id, num(p[0]), num(p[1]))
        } else {
            "Nothing selected: click a marker or a point".into()
        };
        let state = format!("{} changed, {} undo steps{}", self.changes().len(), self.undo.len(), if self.save.is_some() { ", save requested" } else { "" });
        vec![what, state, self.note.clone()]
    }

    /// The controls column's width in a pane `r` wide.
    pub fn controls_w(r: Rect) -> i32 {
        180.min(r.w / 3)
    }

    /// The pane `r` split into the controls column and the stage beside it.
    pub fn split(r: Rect) -> (Rect, Rect) {
        let cw = Self::controls_w(r);
        (Rect::new(r.x + 2, r.y + 2, cw, r.h - 4), Rect::new(r.x + cw + 6, r.y + 2, r.w - cw - 8, r.h - 4))
    }

    /// The pane in `r`: the controls on the left, the stage at `view` (which the caller sets, `set_view`).
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(LayoutAct) -> A) {
        let (controls, _) = Self::split(r);
        self.draw_controls(ui, controls, wrap);
        self.draw_stage(ui, wrap);
    }

    /// The stage at `view`: the background covering the rect, the padded area and the grid, the polylines with their
    /// points and segment middles, the markers by the ground-point rule with their rings and labels. Nothing draws
    /// outside the rect.
    pub fn draw_stage<A: Clone>(&self, ui: &mut Ui<A>, wrap: &impl Fn(LayoutAct) -> A) {
        let t = ui.t.clone();
        let v = self.view();
        let r = v.rect;
        if r.w <= 0 || r.h <= 0 {
            return;
        }
        ui.p.fill(r, t.stage);
        ui.push_clip(r);
        if let Some(bg) = &self.background {
            ui.p.image(bg, cover_rect(bg.w, bg.h, r));
        }
        let (ax, ay, aw, ah) = v.area();
        let (x0, y0, x1, y1) = (r.x as f32, r.y as f32, (r.x + r.w - 1) as f32, (r.y + r.h - 1) as f32);
        if self.grid {
            let lines = (1.0 / GRID_STEP).round() as i32;
            for i in -lines..=2 * lines {
                let f = i as f32 * GRID_STEP;
                let major = i % (lines / 2).max(1) == 0;
                let col = if major { with_alpha(t.info, 150) } else { with_alpha(t.info, 60) };
                let (x, y) = (ax + f * aw, ay + f * ah);
                if (x0..=x1).contains(&x) {
                    ui.p.line(x, y0, x, y1, col, if major { 0.0 } else { 2.0 });
                }
                if (y0..=y1).contains(&y) {
                    ui.p.line(x0, y, x1, y, col, if major { 0.0 } else { 2.0 });
                }
            }
        }
        let gold = with_alpha(t.gold, 200);
        for (a, b) in [((ax, ay), (ax + aw, ay)), ((ax + aw, ay), (ax + aw, ay + ah)), ((ax + aw, ay + ah), (ax, ay + ah)), ((ax, ay + ah), (ax, ay))] {
            ui.p.line(a.0, a.1, b.0, b.1, gold, 3.0);
        }
        for (li, l) in self.polylines.iter().enumerate() {
            let col = if li == self.line { t.info } else { with_alpha(t.info, 140) };
            for w in l.points.windows(2) {
                let (a, b) = (v.to_screen(w[0]), v.to_screen(w[1]));
                ui.p.line(a.0, a.1, b.0, b.1, col, 0.0);
            }
        }
        let small = ui.p.line_h(SMALL);
        for (i, m) in self.markers.iter().enumerate() {
            let g = v.to_screen(m.pos);
            let b = self.marker_box.rect(g, m.scale);
            match &m.image {
                Some(img) => ui.p.image(img, b),
                None => {
                    ui.p.panel(b, rgba(0, 0, 0, 0), with_alpha(t.dim, 160), 0.0);
                    ui.p.ellipse(g.0, g.1, 4.0, 4.0, t.gold);
                }
            }
            if self.rings {
                let rx = (b.w as f32 / 5.0).max(4.0);
                ui.p.ellipse_ring(g.0, g.1, rx, rx / 2.0, t.gold);
                ui.p.fill(Rect::new(g.0.round() as i32 - 1, g.1.round() as i32 - 1, 2, 2), t.text);
            }
            if self.selected == Some(Sel::Marker(i)) {
                ui.p.panel(b, rgba(0, 0, 0, 0), t.focus, 0.0);
            }
            if self.labels {
                ui.p.text_center(g.0, b.y as f32 - small - 1.0, &m.label, SMALL, t.text);
            }
            ui.handle(&format!("{ID}.marker.{}", m.id), b, &m.label, Some(wrap(LayoutAct::Select(Some(Sel::Marker(i))))), tip(&m.label, "Drag to move it; the arrows nudge it (Shift: more); + and - size it."));
        }
        for (li, l) in self.polylines.iter().enumerate() {
            for (k, w) in l.points.windows(2).enumerate() {
                let (a, b) = (v.to_screen(w[0]), v.to_screen(w[1]));
                let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
                ui.p.circle(mx, my, 2.5, with_alpha(t.info, 200));
                ui.handle(&format!("{ID}.mid.{k}.{}", l.id), Rect::new(mx.round() as i32 - 3, my.round() as i32 - 3, 7, 7), "+", None, tip("Insert", "Drag from a segment's middle to add a point there."));
            }
            for (k, p) in l.points.iter().enumerate() {
                let s = v.to_screen(*p);
                let on = self.selected == Some(Sel::Point(li, k));
                let (cx, cy) = (s.0.round() as i32, s.1.round() as i32);
                ui.p.fill(Rect::new(cx - 2, cy - 2, 5, 5), if on { t.focus } else { t.info });
                ui.handle(&format!("{ID}.point.{k}.{}", l.id), Rect::new(cx - 4, cy - 4, 9, 9), &format!("{} {k}", l.id), Some(wrap(LayoutAct::Select(Some(Sel::Point(li, k))))), tip(&l.id, "Drag the point; the arrows nudge it (Shift: more)."));
            }
        }
        ui.pop_clip();
        ui.p.panel(r, rgba(0, 0, 0, 0), t.edge, 0.0);
    }

    /// The controls in `r`: the selection, the scale slider with - and +, the switches, the polyline with Insert and
    /// Remove point, Undo, and Save (a debug build only), over the read-out.
    pub fn draw_controls<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(LayoutAct) -> A) {
        let t = ui.t.clone();
        let row = (ui.p.line_h(BODY) + 3.0).ceil() as i32;
        let small = ui.p.line_h(SMALL);
        let foot = (small * 3.0).ceil() as i32 + 4;
        let area = Rect::new(r.x, r.y, r.w, (r.h - foot).max(row));
        let rows = 9;
        let top = area.y - ui.scroll_begin(&format!("{ID}.controls"), area, rows * (row + 1));
        let (x, w) = (area.x, area.w - 5);
        let mut y = top;
        let marker = self.sel_marker();
        let title = match marker {
            Some(i) => format!("Marker: {}", self.markers[i].label),
            None => match self.sel_point() {
                Some((li, k)) => format!("Point {k} of {}", self.polylines[li].id),
                None => "Nothing selected".into(),
            },
        };
        ui.p.text_fit(x as f32, y as f32 + 1.0, &title, BODY, w as f32, t.gold);
        y += row + 1;
        let label_w = 34;
        ui.p.text_fit(x as f32, y as f32 + 1.0, "Scale", BODY, label_w as f32, t.dim);
        let scale = marker.map(|i| self.markers[i].scale);
        let text = scale.map_or("none".to_string(), |s| format!("{}", num(s)));
        ui.slider(&format!("{ID}.scale"), Rect::new(x + label_w, y, w - label_w, row), step_of(scale.unwrap_or(1.0)), scale_steps(), &text, |i| wrap(LayoutAct::Slide(i)), marker.is_some());
        y += row + 1;
        let bw = (w - 2) / 2;
        ui.button_tip(&format!("{ID}.smaller"), Rect::new(x, y, bw, row), "-", wrap(LayoutAct::ScaleStep(-1)), marker.is_some(), tip("Smaller", "The marker's scale less a step (also the - key)."));
        ui.button_tip(&format!("{ID}.larger"), Rect::new(x + bw + 2, y, bw, row), "+", wrap(LayoutAct::ScaleStep(1)), marker.is_some(), tip("Larger", "The marker's scale more a step (also the + key)."));
        y += row + 1;
        ui.checkbox(&format!("{ID}.grid"), x, y + 1, "Grid", self.grid, wrap(LayoutAct::Toggle(Show::Grid)), true);
        ui.checkbox(&format!("{ID}.labels"), x + w / 2, y + 1, "Labels", self.labels, wrap(LayoutAct::Toggle(Show::Labels)), true);
        y += row + 1;
        ui.checkbox(&format!("{ID}.rings"), x, y + 1, "Ground rings", self.rings, wrap(LayoutAct::Toggle(Show::Rings)), true);
        y += row + 1;
        let names: Vec<String> = self.polylines.iter().map(|l| l.id.clone()).collect();
        if !names.is_empty() {
            ui.select(&format!("{ID}.route"), Rect::new(x, y, w, row), &names, self.line.min(names.len() - 1), |i| wrap(LayoutAct::Line(i)), true, tip("Polyline", "The polyline Insert point adds to when no point is selected."));
        }
        y += row + 1;
        ui.button_tip(&format!("{ID}.insert"), Rect::new(x, y, bw, row), "Insert point", wrap(LayoutAct::InsertPoint), !names.is_empty(), tip("Insert point", "After the selected point, or at the end of the polyline."));
        ui.button_tip(&format!("{ID}.remove"), Rect::new(x + bw + 2, y, bw, row), "Remove point", wrap(LayoutAct::RemovePoint), self.sel_point().is_some(), tip("Remove point", "The selected point goes."));
        y += row + 1;
        ui.button_tip(&format!("{ID}.undo"), Rect::new(x, y, bw, row), &format!("Undo ({})", self.undo.len()), wrap(LayoutAct::Undo), !self.undo.is_empty(), tip("Undo", "Takes back the last edit (also Ctrl+Z)."));
        if writes_allowed() {
            ui.button_tip(&format!("{ID}.save"), Rect::new(x + bw + 2, y, bw, row), "Save", wrap(LayoutAct::Save), self.dirty(), tip("Save", "Hands the changed values to the screen, which saves them. A debug build only."));
        } else {
            ui.p.text_fit((x + bw + 2) as f32, y as f32 + 1.0, "Read-only build", BODY, bw as f32, t.dim);
        }
        ui.scroll_end();
        ui.push_clip(Rect::new(r.x, area.y + area.h, r.w, foot));
        for (i, l) in self.read_out().iter().enumerate() {
            ui.p.text_fit(r.x as f32, (area.y + area.h + 2) as f32 + small * i as f32, l, SMALL, r.w as f32, t.dim);
        }
        ui.pop_clip();
    }
}

/// `c` at alpha `a`.
fn with_alpha(c: Rgba, a: u8) -> Rgba {
    [c[0], c[1], c[2], a]
}

/// The rect an `iw` x `ih` picture covers `r` with, centred (the overflow cut by the caller's clip).
pub fn cover_rect(iw: i32, ih: i32, r: Rect) -> Rect {
    if iw <= 0 || ih <= 0 {
        return r;
    }
    let k = (r.w as f32 / iw as f32).max(r.h as f32 / ih as f32);
    let (w, h) = ((iw as f32 * k).round() as i32, (ih as f32 * k).round() as i32);
    Rect::new(r.x + (r.w - w) / 2, r.y + (r.h - h) / 2, w, h)
}

/// A code-drawn sample for the UI Gallery's Layout page: a 248 x 164 background, three markers (one past the right
/// edge, as an edge anchor) and a polyline of two points.
pub fn sample() -> LayoutEditor {
    let (w, h) = (248, 164);
    let mut bg = Image { w, h, px: vec![0; (w * h * 4) as usize] };
    for y in 0..h {
        for x in 0..w {
            let band = ((x + 2 * y) / 24) % 2 == 0;
            let c = if band { rgb(46, 58 + (y / 4) as u8, 70) } else { rgb(40, 52 + (y / 4) as u8, 62) };
            let at = ((y * w + x) * 4) as usize;
            bg.px[at..at + 4].copy_from_slice(&c);
        }
    }
    let shape = |c: Rgba| {
        let s = 32;
        let mut img = Image { w: s, h: s, px: vec![0; (s * s * 4) as usize] };
        for y in 0..s {
            let half = y / 2;
            for x in (s / 2 - half).max(0)..(s / 2 + half).min(s) {
                let at = ((y * s + x) * 4) as usize;
                img.px[at..at + 4].copy_from_slice(&c);
            }
        }
        Rc::new(img)
    };
    let mut markers = vec![Marker::new("a", "Marker A", [0.15, 0.8], 1.0), Marker::new("b", "Marker B", [0.5, 0.3], 1.5), Marker::new("c", "Marker C", [1.04, 0.6], 0.75)];
    markers[0].image = Some(shape(rgb(200, 160, 90)));
    markers[1].image = Some(shape(rgb(150, 180, 220)));
    let mut e = LayoutEditor::new(markers, vec![Polyline::new("p1", vec![[0.25, 0.65], [0.4, 0.45]])]);
    e.background = Some(Rc::new(bg));
    e.marker_box = MarkerBox { side: 32.0, foot: 4.0 };
    e
}

/// The sample's viewport on `stage`: its background's aspect, padded by 7 % of each side.
pub fn sample_view(e: &LayoutEditor, stage: Rect) -> Viewport {
    let aspect = e.background.as_ref().map_or((3, 2), |b| (b.w, b.h));
    Viewport::fit(stage.inset(4), aspect, (0.07, 0.07))
}
