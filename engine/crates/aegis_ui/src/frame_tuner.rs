//! LOOK1-FT: the frame tuner, a pane of the UI Gallery: pick a frame role, change its corner size, tip size, line width and
//! colour, content padding and straddle, and a panel's size, with sliders on a live preview through the view in force (so
//! at the window's real pixels), read the frame's sidecar and the panel's values as text. The sliders change an override
//! held here (`Frame` copies drawn by this pane only), never the theme in force or a file. Save, in a debug build only,
//! merges what changed into the sidecar of the file that draws the role, keeping its other keys and their order, and
//! installs the theme again (`theme::reinstall`); Revert drops the override, then the file's bytes from before the
//! session's first save. A release build shows the read-out and refuses both writes.

use crate::canvas::{rgb, Image, Rect};
use crate::painter::{nine_art_line, nine_content_rect, Typeface};
use crate::screen::Key;
use crate::theme::{self, line_colour, Frame, Nine, NineFill, Theme, CORNER_SIZE, FRAME_ROLES, LINE_WIDTH, ROLES};
use crate::ui::{tip, Ui, BODY, SMALL};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The hex field's widget id (`Specimens::text_changed` hands its text to `hex_changed`).
pub const HEX_FIELD: &str = "kit.frame.hex";
/// The longest text the hex field keeps (`#rrggbbaa`).
const HEX_MAX: usize = 9;
/// How many times wider than its end a head's base is, at least, for `find_tip` to call it a head.
const TIP_POINT: i32 = 3;

/// A slider of the pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Param {
    Width,
    Height,
    Corner,
    Tip,
    Line,
    Pad,
}

pub const PARAMS: [Param; 6] = [Param::Width, Param::Height, Param::Corner, Param::Tip, Param::Line, Param::Pad];

impl Param {
    /// The lowest value, the highest and the step (logical px, or a share of the art's size for Corner and Tip).
    pub fn range(self) -> (f32, f32, f32) {
        match self {
            Param::Width => (16.0, 640.0, 8.0),
            Param::Height => (16.0, 360.0, 8.0),
            Param::Corner | Param::Tip => (CORNER_SIZE.0, CORNER_SIZE.1, 0.05),
            Param::Line => (LINE_WIDTH.0, LINE_WIDTH.1, 0.5),
            Param::Pad => (0.0, 32.0, 1.0),
        }
    }

    pub fn steps(self) -> usize {
        let (lo, hi, step) = self.range();
        ((hi - lo) / step).round() as usize + 1
    }

    /// The value at step `i`, past the last one the last; rounded, so a value met twice is the same number.
    pub fn value(self, i: usize) -> f32 {
        let (lo, _, step) = self.range();
        ((lo + i.min(self.steps() - 1) as f32 * step) * 100.0).round() / 100.0
    }

    /// The step nearest `v`, inside the range.
    pub fn step_of(self, v: f32) -> usize {
        let (lo, _, step) = self.range();
        (((v - lo) / step).round().max(0.0) as usize).min(self.steps() - 1)
    }

    pub fn label(self) -> &'static str {
        match self {
            Param::Width => "Width",
            Param::Height => "Height",
            Param::Corner => "Corner",
            Param::Tip => "Tip",
            Param::Line => "Line",
            Param::Pad => "Padding",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TunerAct {
    /// The frame role (`FRAME_ROLES` index).
    Role(usize),
    Slide(Param, usize),
    /// The line colour: 0 the art's own, else `ROLES[i - 1]`.
    Colour(usize),
    Straddle,
    /// The preview over the whole page.
    Big,
    Save,
    Revert,
}

/// Whether this build may write a sidecar: a debug build only.
pub const fn writes_allowed() -> bool {
    cfg!(debug_assertions)
}

/// What Save read before its first write to a file, for Revert.
#[derive(Debug, Clone)]
struct Backup {
    role: String,
    file: PathBuf,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FrameTuner {
    /// `FRAME_ROLES` index.
    pub role: usize,
    pub panel: (i32, i32),
    pub big: bool,
    /// The hex field's text.
    pub hex: String,
    /// What the last action did, for the page's foot.
    pub note: String,
    /// The overrides, by the role of the file they tune.
    tuned: BTreeMap<String, Nine>,
    backups: Vec<Backup>,
}

impl Default for FrameTuner {
    fn default() -> Self {
        let role = FRAME_ROLES.iter().position(|r| r.0 == "ornate").unwrap_or(0);
        Self { role, panel: (240, 120), big: false, hex: String::new(), note: String::new(), tuned: BTreeMap::new(), backups: Vec::new() }
    }
}

/// The role whose file draws `role`, along its fallbacks.
fn file_role(t: &Theme, role: &str) -> Option<String> {
    if !t.frames_on {
        return None;
    }
    let mut at = Some(role);
    while let Some(r) = at {
        if t.frames.contains_key(r) {
            return Some(r.to_string());
        }
        at = FRAME_ROLES.iter().find(|f| f.0 == r).and_then(|f| f.1);
    }
    None
}

/// The head at the end of a straddling corner's arm, if the art has one, as the sidecar's `tip`: walking in from the arm's
/// end along the anchor's row, the art widens from a point to a base and then narrows into the arm. An arm that runs on
/// to the slice's end at its full width has none.
pub fn find_tip(img: &Image, nine: &Nine) -> Option<[i32; 2]> {
    let (kw, kh) = (nine.slice[0].min(nine.slice[2]), nine.slice[1].min(nine.slice[3]));
    let [ax, ay] = nine.corner_anchor.unwrap_or([nine.slice[0] / 2, nine.slice[1] / 2]);
    if !nine.straddle || kw > img.w || kh > img.h {
        return None;
    }
    let drawn = |x: i32, y: i32| y >= 0 && y < kh && img.px[((y * img.w + x) * 4 + 3) as usize] > 128;
    let across = |x: i32| -> i32 {
        if !drawn(x, ay) {
            return 0;
        }
        (1..).take_while(|d| drawn(x, ay - d)).count() as i32 + (1..).take_while(|d| drawn(x, ay + d)).count() as i32 + 1
    };
    let end = (ax..kw).rev().find(|x| across(*x) > 0)?;
    let (mut base, mut most, mut narrows) = (end, across(end), false);
    for x in (ax..end).rev() {
        let w = across(x);
        if w >= most {
            (base, most) = (x, w);
        } else if w + 2 < most {
            narrows = true;
            break;
        }
    }
    let half = most / 2 + 2;
    (narrows && most >= TIP_POINT * across(end) && end - base >= 4 && half <= kw.min(kh)).then_some([base, half])
}

/// The sidecar keys of `n` that differ from a bare frame, in the order the art tool writes them.
pub fn sidecar_of(n: &Nine) -> Map<String, Value> {
    let mut m = Map::new();
    let num = |f: f32| json!((f as f64 * 100.0).round() / 100.0);
    m.insert("slice".into(), json!(n.slice));
    if n.edges != NineFill::Stretch {
        m.insert("edges".into(), json!(n.edges.name()));
    }
    if n.center != NineFill::Stretch {
        m.insert("center".into(), json!(n.center.name()));
    }
    if let Some(a) = n.corner_anchor {
        m.insert("corner_anchor".into(), json!(a));
    }
    if let Some(a) = n.corner_inner {
        m.insert("corner_inner".into(), json!(a));
    }
    m.insert("scale".into(), json!(n.scale));
    m.insert("pad".into(), json!(n.pad));
    if n.ink != "text" {
        m.insert("ink".into(), json!(n.ink));
    }
    if n.reduce {
        m.insert("reduce".into(), json!("average"));
    }
    if n.straddle {
        m.insert("straddle".into(), json!(true));
    }
    if n.sharpen != 0.0 {
        m.insert("sharpen".into(), num(n.sharpen));
    }
    if n.corner_size != 1.0 {
        m.insert("corner_size".into(), num(n.corner_size));
    }
    if let Some(a) = n.tip {
        m.insert("tip".into(), json!(a));
    }
    if n.tip_size != 1.0 {
        m.insert("tip_size".into(), num(n.tip_size));
    }
    if let Some(w) = n.line_width {
        m.insert("line_width".into(), num(w));
    }
    if let Some(c) = &n.line_color {
        m.insert("line_color".into(), json!(c.name));
    }
    m
}

/// `v` on one line in the art tool's form: `, ` and `: ` between items.
pub fn sidecar_text(v: &Value) -> String {
    match v {
        Value::Array(a) => format!("[{}]", a.iter().map(sidecar_text).collect::<Vec<_>>().join(", ")),
        Value::Object(m) => format!("{{{}}}", m.iter().map(|(k, x)| format!("{}: {}", json!(k), sidecar_text(x))).collect::<Vec<_>>().join(", ")),
        other => other.to_string(),
    }
}

/// The sidecar file's JSON `file` with what `tuned` changes against `was`, its other keys and their order kept. None: no
/// key changes.
pub fn merged_sidecar(file: &Value, was: &Nine, tuned: &Nine) -> Option<Value> {
    let (before, after) = (sidecar_of(was), sidecar_of(tuned));
    let mut out = file.as_object()?.clone();
    let mut changed = false;
    for key in before.keys().chain(after.keys()) {
        if before.get(key) == after.get(key) {
            continue;
        }
        match after.get(key) {
            Some(v) => out.insert(key.clone(), v.clone()),
            None => out.shift_remove(key),
        };
        changed = true;
    }
    changed.then_some(Value::Object(out))
}

impl FrameTuner {
    pub fn role_name(&self) -> &'static str {
        FRAME_ROLES[self.role % FRAME_ROLES.len()].0
    }

    /// The file's frame that draws the picked role, in the layers' theme, or None for a vector panel.
    fn file_frame(&self) -> Option<(String, Frame)> {
        let t = theme::loaded();
        let own = file_role(&t, self.role_name())?;
        let f = t.frames.get(&own)?.clone();
        Some((own, f))
    }

    /// The picked role's frame with the override on it, as this pane draws it.
    pub fn frame(&self) -> Option<Frame> {
        let (own, f) = self.file_frame()?;
        let nine = self.tuned.get(&own).cloned().unwrap_or_else(|| f.nine.clone());
        Some(Frame { nine, ..f })
    }

    /// Whether the picked frame has an unsaved override.
    pub fn changed(&self) -> bool {
        self.file_frame().is_some_and(|(own, _)| self.tuned.contains_key(&own))
    }

    fn edit(&mut self, change: impl FnOnce(&mut Nine, &Frame)) {
        let Some((own, f)) = self.file_frame() else {
            self.note = format!("{} has no frame file", self.role_name());
            return;
        };
        let mut n = self.tuned.get(&own).cloned().unwrap_or_else(|| f.nine.clone());
        change(&mut n, &f);
        if n == f.nine {
            self.tuned.remove(&own);
        } else {
            self.tuned.insert(own, n);
        }
    }

    /// The value a slider shows, None where the picked frame cannot use it. The Line slider shows the art's width until moved.
    pub fn value(&self, p: Param) -> Option<f32> {
        match p {
            Param::Width => return Some(self.panel.0 as f32),
            Param::Height => return Some(self.panel.1 as f32),
            _ => {}
        }
        let f = self.frame()?;
        match p {
            Param::Corner => Some(f.nine.corner_size),
            Param::Tip => (f.nine.tip.is_some() || find_tip(&f.img, &f.nine).is_some()).then_some(f.nine.tip_size),
            Param::Line => nine_art_line(&f.img, &f.nine).map(|a| f.nine.line_width.unwrap_or(a.0)),
            Param::Pad => Some(f.nine.pad[0] as f32),
            _ => None,
        }
    }

    fn slide(&mut self, p: Param, i: usize) {
        let v = p.value(i);
        match p {
            Param::Width => self.panel.0 = v as i32,
            Param::Height => self.panel.1 = v as i32,
            Param::Corner => self.edit(|n, _| n.corner_size = v),
            Param::Pad => self.edit(|n, _| n.pad = [v as i32; 4]),
            Param::Tip => self.edit(|n, f| {
                let Some(found) = n.tip.or_else(|| find_tip(&f.img, n)) else { return };
                n.tip_size = v;
                n.tip = if v == 1.0 { f.nine.tip } else { Some(found) };
            }),
            Param::Line => self.edit(|n, f| {
                let Some(art) = nine_art_line(&f.img, n) else { return };
                n.line_width = (v != art.0).then_some(v);
            }),
        }
    }

    /// The line colour `name` (a role or a hex colour) on the override; false when the frame has no line.
    fn set_colour(&mut self, name: Option<&str>) -> bool {
        let mut done = false;
        let t = theme::loaded();
        self.edit(|n, f| {
            if nine_art_line(&f.img, n).is_none() {
                return;
            }
            n.line_color = name.and_then(line_colour).map(|mut c| {
                if let Some(rgba) = t.role(&c.name) {
                    c.rgba = rgba;
                }
                c
            });
            done = true;
        });
        done
    }

    /// The hex field's text changed: kept (at most `HEX_MAX` characters), and in force once it is a colour.
    pub fn hex_changed(&mut self, text: &str) {
        if text.chars().count() > HEX_MAX {
            return;
        }
        self.hex = text.to_string();
        if theme::parse_colour(text).is_some() {
            self.note = if self.set_colour(Some(text)) { format!("line colour {text}") } else { "this frame has no plain line".into() };
        }
    }

    pub fn act(&mut self, a: TunerAct) {
        match a {
            TunerAct::Role(i) => {
                self.role = i % FRAME_ROLES.len();
                self.hex.clear();
                self.note = format!("frame {}", self.role_name());
            }
            TunerAct::Slide(p, i) => {
                self.slide(p, i);
                self.note = format!("{} {}", p.label(), p.value(i));
            }
            TunerAct::Colour(i) => {
                let name = i.checked_sub(1).and_then(|r| ROLES.get(r).copied());
                self.hex.clear();
                self.note = if self.set_colour(name) { format!("line colour {}", name.unwrap_or("of the art")) } else { "this frame has no plain line".into() };
            }
            TunerAct::Straddle => {
                self.edit(|n, f| {
                    n.straddle = !n.straddle;
                    // What only a straddling frame draws goes and comes with it, so the sidecar always loads.
                    (n.tip, n.tip_size, n.line_width, n.line_color) = if n.straddle && f.nine.straddle { (f.nine.tip, f.nine.tip_size, f.nine.line_width, f.nine.line_color.clone()) } else { (None, 1.0, None, None) };
                });
                self.note = format!("straddle {}", if self.frame().is_some_and(|f| f.nine.straddle) { "on" } else { "off" });
            }
            TunerAct::Big => {
                self.big = !self.big;
                self.note = format!("big preview {}", if self.big { "on" } else { "off" });
            }
            TunerAct::Save => self.note = self.save().unwrap_or_else(|e| e),
            TunerAct::Revert => self.note = self.revert().unwrap_or_else(|e| e),
        }
    }

    /// The keys beyond the shell's focus (Tab and the arrows walk the widgets, Left and Right move a focused slider, Enter
    /// presses): PageUp and PageDown go through the frames, Esc leaves the big preview. True when the key was used.
    pub fn key(&mut self, k: Key) -> bool {
        let n = FRAME_ROLES.len();
        match k {
            Key::PageUp => self.act(TunerAct::Role((self.role + n - 1) % n)),
            Key::PageDown => self.act(TunerAct::Role((self.role + 1) % n)),
            Key::Esc if self.big => self.act(TunerAct::Big),
            _ => return false,
        }
        true
    }

    /// Save, under this build's rule.
    pub fn save(&mut self) -> Result<String, String> {
        self.save_as(writes_allowed())
    }

    /// Writes the override into the sidecar of the file that draws the role and installs the theme again; a file the theme
    /// will not load goes back as it was. Refused unless `allowed` and this is a debug build.
    pub fn save_as(&mut self, allowed: bool) -> Result<String, String> {
        if !(allowed && writes_allowed()) {
            return Err("a release build shows the read-out only: nothing written".into());
        }
        let (own, f) = self.file_frame().ok_or_else(|| format!("{} has no frame file", self.role_name()))?;
        let Some(tuned) = self.tuned.get(&own).cloned() else { return Ok(format!("{own}: nothing changed, nothing written")) };
        let rel = format!("{}/{own}.json", theme::FRAMES_DIR);
        let file = theme::layers().find(&rel).ok_or_else(|| format!("no file {rel} in the layers"))?;
        let bytes = std::fs::read(&file).map_err(|e| format!("{rel}: {e}"))?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|e| format!("{rel}: {e}"))?;
        let was = Nine::parse(&value, f.img.w, f.img.h, &rel).map_err(|e| e.join("; "))?;
        let Some(merged) = merged_sidecar(&value, &was, &tuned) else {
            self.tuned.remove(&own);
            return Ok(format!("{own}: nothing changed, nothing written"));
        };
        Nine::parse(&merged, f.img.w, f.img.h, &rel).map_err(|e| e.join("; "))?;
        std::fs::write(&file, sidecar_text(&merged) + "\n").map_err(|e| format!("{rel}: {e}"))?;
        if let Err(e) = theme::reinstall() {
            std::fs::write(&file, &bytes).map_err(|w| format!("{rel}: {w}"))?;
            return Err(format!("{rel} put back: {}", e.join("; ")));
        }
        if !self.backups.iter().any(|b| b.role == own) {
            self.backups.push(Backup { role: own.clone(), file, bytes });
        }
        self.tuned.remove(&own);
        Ok(format!("saved {rel}"))
    }

    /// Drops the picked frame's override; with none, puts its file back as it was before the session's first save.
    pub fn revert(&mut self) -> Result<String, String> {
        let Some((own, _)) = self.file_frame() else { return Err(format!("{} has no frame file", self.role_name())) };
        if self.tuned.remove(&own).is_some() {
            return Ok(format!("{own}: back to the file"));
        }
        let Some(at) = self.backups.iter().position(|b| b.role == own) else { return Ok(format!("{own}: nothing to revert")) };
        if !writes_allowed() {
            return Err("a release build shows the read-out only: nothing written".into());
        }
        let b = self.backups.remove(at);
        std::fs::write(&b.file, &b.bytes).map_err(|e| format!("{own}: {e}"))?;
        theme::reinstall().map_err(|e| e.join("; "))?;
        Ok(format!("{own}: the file as it was before the first save"))
    }

    /// Whether Revert would do anything.
    fn can_revert(&self) -> bool {
        self.changed() || self.file_frame().is_some_and(|(own, _)| self.backups.iter().any(|b| b.role == own))
    }

    /// The read-out: where the frame comes from, its sidecar as it stands here, and the panel's values at this view.
    pub fn read_out(&self, view: f32) -> Vec<String> {
        let (pw, ph) = self.panel;
        let Some(f) = self.frame() else {
            return vec![format!("{}: no frame file, a vector panel", self.role_name()), format!("panel {pw} x {ph}")];
        };
        let inner = nine_content_rect(&f.nine, Rect::new(0, 0, pw, ph), view);
        vec![
            format!("{}{}", f.source, if self.changed() { ", not saved" } else { "" }),
            sidecar_text(&Value::Object(sidecar_of(&f.nine))),
            format!("panel {pw} x {ph}, content {} x {}, at {} x {} px in the window", inner.w, inner.h, (pw as f32 * view).round(), (ph as f32 * view).round()),
        ]
    }

    /// The agent link's view of the pane.
    pub fn state(&self) -> Value {
        json!({ "role": self.role_name(), "panel": [self.panel.0, self.panel.1], "big": self.big, "changed": self.changed(), "sidecar": self.frame().map(|f| Value::Object(sidecar_of(&f.nine))), "writes": writes_allowed(), "note": self.note })
    }

    /// The panel on `stage`, centred, with a portrait box and sample text in its content rect. Nothing draws outside `stage`.
    pub fn draw_preview<A: Clone>(&self, ui: &mut Ui<A>, stage: Rect) {
        let t = ui.t.clone();
        ui.p.fill(stage, rgb(40, 40, 46));
        ui.push_clip(stage);
        let (w, h) = self.panel;
        let panel = Rect::new(stage.x + (stage.w - w) / 2, stage.y + (stage.h - h) / 2, w, h);
        let frame = self.frame();
        let (fill, edge, radius) = t.vector(self.role_name());
        ui.p.skin(frame.as_ref(), panel, fill, edge, radius);
        let view = ui.p.v.s;
        let inner = frame.as_ref().map_or(panel.inset(t.layout.padding), |f| nine_content_rect(&f.nine, panel, view));
        let ink = frame.as_ref().and_then(|f| t.role(&f.nine.ink)).unwrap_or(t.text);
        ui.push_clip(inner);
        let side = (inner.h - 2).clamp(0, 48).min(inner.w / 3);
        if let Some(img) = crate::specimens::sample_portraits().first().filter(|_| side >= 8) {
            ui.p.image(img, Rect::new(inner.x, inner.y, side, side));
        }
        let x = inner.x + if side >= 8 { side + 4 } else { 0 };
        let line = ui.p.line_h(BODY);
        ui.p.text_face(Typeface::Display, x as f32, inner.y as f32, "Sample title", BODY, t.gold);
        let lines = ui.p.wrap("The quick brown fox waits by the old gate, and the rain has not stopped for days.", BODY, (inner.x + inner.w - x).max(1) as f32);
        for (i, l) in lines.iter().enumerate() {
            let y = inner.y as f32 + line * (i + 1) as f32;
            if y > (inner.y + inner.h) as f32 {
                break;
            }
            ui.p.text(x as f32, y, l, BODY, ink);
        }
        ui.pop_clip();
        ui.pop_clip();
    }

    /// The pane in `r`: the controls in a scroll area on the left, the stage on the right over the read-out; with the big
    /// preview, the stage on the whole page and one switch back.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(TunerAct) -> A) {
        let t = ui.t.clone();
        let row = (ui.p.line_h(BODY) + 3.0).ceil() as i32;
        if self.big {
            ui.checkbox("kit.frame.big", r.x + 4, r.y + 3, "Big preview (Esc)", true, wrap(TunerAct::Big), true);
            self.draw_preview(ui, Rect::new(r.x, r.y + 16, r.w, r.h - 16));
            return;
        }
        let side = 200.min(r.w / 2);
        let controls = Rect::new(r.x + 2, r.y + 2, side, r.h - 4);
        let rows = 3 + PARAMS.len() as i32 + 2;
        let top = controls.y - ui.scroll_begin("kit.frame.controls", controls, rows * (row + 1));
        let (x, w) = (controls.x, controls.w - 5);
        let mut y = top;
        let frames: Vec<String> = FRAME_ROLES.iter().map(|f| f.0.to_string()).collect();
        ui.select("kit.frame.role", Rect::new(x, y, w, row), &frames, self.role, |i| wrap(TunerAct::Role(i)), true, tip("Frame", "The frame role to tune. PageUp and PageDown go through them."));
        y += row + 1;
        let label_w = 40;
        for p in PARAMS {
            ui.p.text_fit(x as f32, y as f32 + 1.0, p.label(), BODY, label_w as f32, t.dim);
            let v = self.value(p);
            let text = v.map_or("none".to_string(), |v| if matches!(p, Param::Corner | Param::Tip) { format!("{:.0} %", v * 100.0) } else { format!("{v}") });
            let id = format!("kit.frame.{}", p.label().to_lowercase());
            ui.slider(&id, Rect::new(x + label_w, y, w - label_w, row), p.step_of(v.unwrap_or(1.0)), p.steps(), &text, |i| wrap(TunerAct::Slide(p, i)), v.is_some());
            y += row + 1;
        }
        let f = self.frame();
        let colour = f.as_ref().and_then(|f| f.nine.line_color.as_ref().map(|c| c.name.clone()));
        let mut colours: Vec<String> = std::iter::once("Art's own".to_string()).chain(ROLES.iter().map(|r| r.to_string())).collect();
        let picked = match &colour {
            None => 0,
            Some(name) => ROLES.iter().position(|r| r == name).map_or_else(
                || {
                    colours.push(name.clone());
                    colours.len() - 1
                },
                |i| i + 1,
            ),
        };
        let has_line = f.as_ref().is_some_and(|f| nine_art_line(&f.img, &f.nine).is_some());
        let hex_w = (ui.p.text_w("#rrggbbaa", BODY).ceil() as i32 + 8).min(w / 2);
        let n = ROLES.len() + 1;
        ui.select("kit.frame.colour", Rect::new(x, y, w - hex_w - 2, row), &colours, picked, |i| wrap(TunerAct::Colour(if i < n { i } else { picked })), has_line, tip("Line colour", "A colour role, or the art's own. The field takes #rrggbb."));
        ui.text_field(HEX_FIELD, Rect::new(x + w - hex_w, y, hex_w, row), &self.hex, false, has_line);
        y += row + 1;
        let straddle = f.as_ref().is_some_and(|f| f.nine.straddle);
        ui.checkbox("kit.frame.straddle", x, y + 1, "Straddle", straddle, wrap(TunerAct::Straddle), f.is_some());
        ui.checkbox("kit.frame.big", x + w / 2, y + 1, "Big preview", false, wrap(TunerAct::Big), true);
        y += row + 1;
        let bw = (w - 2) / 2;
        if writes_allowed() {
            ui.button_tip("kit.frame.save", Rect::new(x, y, bw, row), "Save", wrap(TunerAct::Save), self.changed(), tip("Save", "Writes the changed keys to the frame's sidecar and loads the theme again. A debug build only."));
            ui.button_tip("kit.frame.revert", Rect::new(x + bw + 2, y, bw, row), "Revert", wrap(TunerAct::Revert), self.can_revert(), tip("Revert", "Back to the file; with nothing unsaved, the file as it was before the first save."));
        } else {
            ui.p.text_fit(x as f32, y as f32 + 1.0, "Release build: read-out only", BODY, w as f32, t.dim);
        }
        ui.scroll_end();
        let (sx, sw) = (controls.x + controls.w + 4, r.x + r.w - (controls.x + controls.w + 4) - 2);
        let small = ui.p.line_h(SMALL);
        let view = ui.p.v.s;
        let mut lines = Vec::new();
        for l in self.read_out(view) {
            lines.extend(ui.p.wrap(&l, SMALL, sw.max(1) as f32));
        }
        let text_h = ((lines.len() as f32 * small).ceil() as i32 + 2).min(r.h / 3);
        let stage = Rect::new(sx, r.y + 2, sw, r.h - 4 - text_h);
        self.draw_preview(ui, stage);
        ui.push_clip(Rect::new(sx, stage.y + stage.h, sw, text_h + 2));
        for (i, l) in lines.iter().enumerate() {
            ui.p.text(sx as f32, (stage.y + stage.h + 2) as f32 + small * i as f32, l, SMALL, t.dim);
        }
        ui.pop_clip();
    }
}
