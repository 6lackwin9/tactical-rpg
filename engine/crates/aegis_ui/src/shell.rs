//! The window shell: one window over the screen stack; each screen draws in the units of its `Grid` (the old canvas is
//! 640x360), pixel art scales with nearest sampling and the UI is drawn smooth at native resolution. The agent link (TCP, `aegis link`) sees and drives
//! exactly what the player sees: ui_tree, click, drag, hover, key, back, screenshot, game, plus whatever the top screen
//! answers (`Screen::command`). Keys: arrows, Enter, Space, Esc, Tab, Page Up / Down and characters go to the top
//! screen; F12 takes a screenshot; right click goes back. While a text field edits (PLAYUI1-T) the editing keys, the
//! clipboard keys, clicks and drags in it go to the field (`ui::Edit`).

use crate::bridge;
use crate::focus;
use crate::canvas::{Canvas, Image};
use crate::layout_audit::{self, Profile};
use crate::painter::{CanvasPolicy, Fonts, Grid, Painter, View};
use crate::screen::{read_window_sizes, Settings, WINDOW_SIZES, window_capped, window_centred, window_kept_in, window_size_of, Context, Cx, Held, Key, Nav, Notice, ScreenRect, Screen, WindowShow};
use crate::canvas::{rgba, Rect};
use crate::pacing::{FrameNeed, Pace, Pacer};
use crate::window::Grab;
use crate::theme::{th, Metrics};
use crate::ui::{clipboard, field_content_h, field_layout, field_scroll_id, field_shift, reveal_offset, Drag, Edit, EditKey, Gfx, Kit, Layout, FIELD_PAD_X, FIELD_PAD_Y};
use anyhow::Result;
use pixels::Pixels;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use aegis_core::clock::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key as WKey, KeyCode, NamedKey, PhysicalKey};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::monitor::MonitorHandle;
use winit::window::{Fullscreen, Window, WindowId};

pub mod headless;

/// What the shell opens.
pub struct Options {
    pub title: String,
    /// The UI font (a TTF file).
    pub font: PathBuf,
    /// The agent link's TCP port.
    pub port: u16,
    /// The window's icon: a PNG path over the asset layers ("" none; ICON1, `crate::icon`).
    pub icon: String,
    /// The manifest's `canvas` as written (null: `CanvasPolicy`'s default; a bad value is a notice and the default).
    pub canvas: Value,
    /// The manifest's `window_sizes` as written (null: the shell holds nothing, and what the game told its settings stands;
    /// a bad value is a notice and `WINDOW_SIZES`).
    pub window_sizes: Value,
    /// Some: no window at all, the frames stay in memory (HEADLESS1, `headless`). None: the player's window.
    pub headless: Option<headless::Headless>,
}

/// Holds the saved window size to what the manifest declared (`declared` as written, `offered` as `read_window_options`
/// read it). A manifest with no such key says nothing, so it overrides nothing the game gave `keep_window_size` itself.
fn hold_declared_sizes(settings: &mut Settings, declared: &Value, offered: &[String]) {
    if !declared.is_null() {
        settings.keep_window_size(&offered.iter().map(String::as_str).collect::<Vec<_>>());
    }
}

/// What a game's options say of its canvas and windows (LAYOUT720-E5): the policy, the sizes it offers and a notice for
/// each value refused.
pub fn read_window_options(canvas: &Value, window_sizes: &Value) -> (CanvasPolicy, Vec<String>, Vec<String>) {
    let mut notices = Vec::new();
    let policy = CanvasPolicy::read(canvas).unwrap_or_else(|e| {
        notices.push(format!("game.json: {e}; using the default canvas."));
        CanvasPolicy::default()
    });
    let sizes = read_window_sizes(window_sizes).unwrap_or_else(|e| {
        notices.push(format!("game.json: {e}; using the default window sizes."));
        WINDOW_SIZES.map(String::from).to_vec()
    });
    (policy, sizes, notices)
}

struct Shell<G> {
    cx: Cx<G>,
    gfx: Gfx,
    stack: Vec<Box<dyn Screen<G>>>,
    /// The mouse in the top screen's units, as every path that follows the pointer reads it.
    mouse: (i32, i32),
    /// Where the OS last put the pointer, in physical px: what `mouse` is worked out from when a screen on another grid
    /// comes to the top. None until the pointer moved.
    mouse_px: Option<(f32, f32)>,
    /// The top screen has `text_button` rows (noted each frame by `settle_focus`; `mouse_moved` needs nothing without).
    text_rows: bool,
    /// The slider being dragged (its id) and the step widget it last set.
    drag: Option<(String, String)>,
    /// A press on a draggable widget (UIKIT2), waiting to become a drag (a move) or a click (a release).
    press: Option<Press>,
    /// A window being moved or resized (UIKIT3): its id, the grab and the last mouse position.
    win_drag: Option<(String, Grab, (i32, i32))>,
    /// The choice field whose list the keyboard opened (its option takes the focus once the list is drawn).
    open_focus: Option<String>,
    /// Playtest trail: the control under the pointer, since when, and its trail form.
    hover: Option<(String, aegis_core::clock::Instant, Value)>,
    /// Shift is held (a Shift+click in a text field extends its selection).
    shift: bool,
    /// The left button went down in the text field being edited: moving the mouse selects (PLAYUI1-T).
    text_press: bool,
    /// The last click in a text field (when and where), to tell a double click (it selects a word).
    text_click: Option<(Instant, (i32, i32))>,
    /// When the shell started: the clock a text field's undo groups typing by (`Edit::now_ms`).
    born: Instant,
    /// A screen was pushed: once it drew, its `Screen::pointer_to` widget (if any) becomes `pointer_req` (ASKNO2).
    pointer_pending: bool,
    /// The canvas point the windowed shell should move the OS mouse pointer to (taken by `App::redraw`).
    pointer_req: Option<(i32, i32)>,
    /// An agent-link command is being run: screens it opens never move the pointer.
    agent: bool,
    /// Frames drawn so far (`ui_tree`'s `frame`): a link reply comes before the frame that shows it, so a driver waits on this.
    frames: u64,
    /// How native screens' canvas relates to the window (the game's `Options::canvas`).
    policy: CanvasPolicy,
}

/// A press on a draggable widget: its id, payload and text, and where the press was.
struct Press {
    id: String,
    key: String,
    text: String,
    at: (i32, i32),
}

/// How far (old units) the mouse moves from the press before a draggable widget is picked up.
const DRAG_START: i32 = 3;
/// A second click in a text field this soon (milliseconds) and this near (old units) is a double click.
const DOUBLE_CLICK_MS: u128 = 400;
const DOUBLE_CLICK_PX: i32 = 3;

/// The drop key under `pos`: the topmost enabled widget that is a drop target or a window's backdrop (a window covers
/// what lies under it), when it is a target.
fn drop_key_at(infos: &[Value], pos: (i32, i32)) -> Option<&str> {
    let hit = infos.iter().rev().find(|w| w["enabled"] == json!(true) && (w["drop"].is_string() || w["kind"] == "backdrop") && info_rect(w).contains(pos.0, pos.1))?;
    hit["drop"].as_str()
}

/// The rect of a widget's agent info.
fn info_rect(w: &Value) -> Rect {
    let r = |i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
    Rect::new(r(0), r(1), r(2), r(3))
}

/// The topmost enabled widget info under `pos`; an open select's options lie over everything (like `ui::hit`).
fn info_at(infos: &[Value], pos: (i32, i32)) -> Option<&Value> {
    let under = |w: &&Value| w["enabled"] == json!(true) && info_rect(w).contains(pos.0, pos.1);
    infos.iter().rev().filter(|w| w["kind"] == "option").find(under).or_else(|| infos.iter().rev().find(under))
}

/// A widget's agent info as a playtest trail control: {path (its id), cls (its kind), text (40 chars), tip (60),
/// interactive}.
fn trail_ctl(w: &Value) -> Value {
    let cut = |v: &Value, n: usize| v.as_str().unwrap_or("").chars().take(n).collect::<String>();
    let tip = w["tip"].as_array().and_then(|t| t.first()).cloned().unwrap_or(Value::Null);
    json!({"path": w["id"], "cls": w["kind"], "text": cut(&w["text"], 40), "tip": cut(&tip, 60), "interactive": w["clickable"] == json!(true)})
}

/// A screen as a trail control (overlays open and close as modals).
fn screen_ctl(id: &str) -> Value {
    json!({"path": id, "cls": "screen", "text": "", "tip": "", "interactive": false})
}

impl<G: Context> Shell<G> {
    /// The shell on a canvas of `pw` x `ph` px: everything but where the frames go, so a window and a run without one
    /// start the same. `icon_notice` is what the manifest's icon came to.
    fn open(cx: Cx<G>, stack: Vec<Box<dyn Screen<G>>>, options: &Options, (pw, ph): (u32, u32), dpr: f32, icon_notice: Option<String>) -> Self {
        let font_bytes = std::fs::read(&options.font).unwrap_or_else(|e| panic!("{}: {e}", options.font.display()));
        let mut gfx = Gfx { canvas: Canvas::new(pw as i32, ph as i32), view: View::fit(pw, ph), fonts: Fonts::new(&font_bytes), kit: Kit::default() };
        gfx.kit.device.dpr = dpr;
        let (policy, offered, window_notices) = read_window_options(&options.canvas, &options.window_sizes);
        let mut shell = Shell { frames: 0, policy, cx, gfx, stack, mouse: (0, 0), mouse_px: None, text_rows: false, drag: None, press: None, win_drag: None, open_focus: None, hover: None, shift: false, text_press: false, text_click: None, born: Instant::now(), pointer_pending: false, pointer_req: None, agent: false };
        shell.cx.notice.extend(icon_notice);
        shell.cx.notice.extend(window_notices);
        hold_declared_sizes(&mut shell.cx.settings, &options.window_sizes, &offered);
        shell
    }

    /// The canvas is now `w` x `h` px.
    fn resize_canvas(&mut self, w: u32, h: u32) {
        self.gfx.canvas = Canvas::new(w as i32, h as i32);
        self.gfx.view = self.top_view();
    }

    /// Applies queued navigation and notices (a screen change closes an open choice list).
    fn settle(&mut self) {
        let mut guard = 0;
        while (!self.cx.nav.is_empty() || !self.cx.notice.is_empty()) && guard < 32 {
            guard += 1;
            self.gfx.kit.open = None;
            self.drag = None;
            self.press = None;
            self.win_drag = None;
            self.gfx.kit.drag = None;
            // The keyboard focus starts over on the new top, hidden unless that screen asks for it (auto_focus): a
            // screen's own Enter / Space (confirm, continue, yes) stay its own.
            self.gfx.kit.focus = None;
            self.gfx.kit.edit = None;
            self.gfx.kit.focus_visible = false;
            self.open_focus = None;
            let navs: Vec<Nav<G>> = self.cx.nav.drain(..).collect();
            for n in navs {
                match n {
                    Nav::Push(s) => {
                        // Playtest trail: an overlay opens as a modal, any other screen is a screen change.
                        let event = if s.overlay() { json!({"type": "modal", "ctl": screen_ctl(s.id()), "open": true}) } else { json!({"type": "screen", "path": s.id(), "leaked": 0}) };
                        self.cx.playtest.trail(event);
                        self.stack.push(s);
                        self.pointer_pending = true;
                    }
                    Nav::Pop => {
                        if self.stack.len() > 1 {
                            if let Some(mut gone) = self.stack.pop() {
                                gone.leave(&mut self.cx);
                                let event = if gone.overlay() { json!({"type": "modal", "ctl": screen_ctl(gone.id()), "open": false}) } else { json!({"type": "screen", "path": self.stack.last().map(|t| t.id()).unwrap_or(""), "leaked": 0}) };
                                self.cx.playtest.trail(event);
                            }
                            let cx = &mut self.cx;
                            self.stack.last_mut().unwrap().resume(cx);
                        }
                    }
                    Nav::Reset(s) => {
                        // A new stack: scroll offsets start over (an overlay or a return keeps them).
                        self.gfx.kit.scroll.clear();
                        while let Some(mut gone) = self.stack.pop() {
                            gone.leave(&mut self.cx);
                        }
                        self.cx.playtest.trail(json!({"type": "screen", "path": s.id(), "leaked": 0}));
                        self.stack.push(s);
                    }
                }
            }
            if !self.cx.notice.is_empty() {
                let lines: Vec<String> = self.cx.notice.drain(..).collect();
                self.cx.playtest.trail(json!({"type": "modal", "ctl": screen_ctl("notice"), "open": true}));
                self.stack.push(Box::new(Notice::new(lines)));
            }
        }
        self.follow_top();
    }

    /// The whole window's view, the one every screen's view comes from.
    fn whole_view(&self) -> View {
        View::fit(self.gfx.canvas.w as u32, self.gfx.canvas.h as u32)
    }

    /// The view of a screen on `grid`: a native screen's comes from the game's policy, the others' from the whole window.
    fn view_of(&self, grid: Grid) -> View {
        if grid == Grid::Native {
            let d = self.gfx.kit.device;
            View::native(self.policy, self.gfx.canvas.w as u32, self.gfx.canvas.h as u32, d.dpr, d.ui_scale)
        } else {
            self.whole_view().for_grid(grid)
        }
    }

    /// The top screen's view.
    fn top_view(&self) -> View {
        self.stack.last().map_or_else(|| self.whole_view(), |t| self.view_of(t.grid()))
    }

    /// The stack changed: the view and the mouse become the new top's. Nothing moves between screens with one view; the
    /// pointer has not moved, so across grids the mouse is read again from where it is (an agent-link position, which no
    /// physical move goes with, from its own middle).
    fn follow_top(&mut self) {
        let (from, to) = (self.gfx.view, self.top_view());
        if from == to {
            return;
        }
        let at = self.mouse_px.filter(|p| from.to_logical(p.0, p.1) == self.mouse).unwrap_or_else(|| from.to_physical(self.mouse.0, self.mouse.1));
        self.mouse = to.to_logical(at.0, at.1);
        self.gfx.view = to;
    }

    /// The OS moved the pointer to physical (`px`, `py`).
    fn pointer_moved(&mut self, px: f32, py: f32) {
        self.mouse_px = Some((px, py));
        self.mouse = self.gfx.view.to_logical(px, py);
        self.mouse_moved();
        self.drag_to();
    }

    /// Every screen leaves, top first (the app closes).
    fn leave_all(&mut self) {
        while let Some(mut gone) = self.stack.pop() {
            gone.leave(&mut self.cx);
        }
        self.cx.playtest.tick();
        self.cx.playtest.save();
    }

    /// Types into the text field being edited (`f` changes its text, caret or selection), hands a change to the screen
    /// and scrolls the field to keep the caret's line in view.
    fn edit(&mut self, f: impl FnOnce(&mut Edit)) {
        let now = self.born.elapsed().as_millis() as u64;
        let Some(e) = self.gfx.kit.edit.as_mut() else { return };
        e.now_ms = now;
        let before = e.text.clone();
        f(e);
        if e.text != before {
            let (id, text) = (e.id.clone(), e.text.clone());
            let cx = &mut self.cx;
            self.stack.last_mut().unwrap().text_changed(cx, &id, &text);
        }
        self.reveal();
    }

    /// The text field being edited as the top screen last drew it, in its units: its layout, its rect and its line height.
    /// None when nothing edits or the field is not drawn (yet).
    fn edit_geometry(&mut self) -> Option<(Layout, Rect, f32)> {
        let (id, text, multiline) = self.gfx.kit.edit.as_ref().map(|e| (e.id.clone(), e.text.clone(), e.multiline))?;
        let infos = self.stack.last()?.widgets();
        let r = infos.iter().find(|w| w["id"] == id.as_str() && (w["kind"] == "text" || w["kind"] == "textarea")).map(info_rect)?;
        let m = Metrics::of(&th(), self.gfx.view);
        let mut p = Painter { c: &mut self.gfx.canvas, v: self.gfx.view, fonts: &mut self.gfx.fonts };
        let line = p.line_h(m.body);
        Some((field_layout(&mut p, &text, r, multiline), r, line))
    }

    /// Scrolls the text area being edited so the caret's line shows.
    fn reveal(&mut self) {
        let Some((lay, r, line)) = self.edit_geometry() else { return };
        let Some(e) = self.gfx.kit.edit.as_ref().filter(|e| e.multiline) else { return };
        let id = field_scroll_id(&e.id);
        let off = self.gfx.kit.scroll.get(&id).copied().unwrap_or(0);
        let g = self.gfx.view.g;
        let to = reveal_offset(off, lay.line_of(e.caret), line, r.h, field_content_h(lay.lines.len(), line, g), g);
        self.gfx.kit.scroll.insert(id, to);
    }

    /// The caret position under canvas point `pos` in the text field being edited (as drawn: scrolled or slid).
    fn caret_at(&mut self, pos: (i32, i32)) -> Option<usize> {
        let (lay, r, line) = self.edit_geometry()?;
        let g = self.gfx.view.g;
        let e = self.gfx.kit.edit.as_ref()?;
        let (dx, off) = if e.multiline {
            let max = (field_content_h(lay.lines.len(), line, g) - r.h).max(0);
            (0.0, self.gfx.kit.scroll.get(&field_scroll_id(&e.id)).copied().unwrap_or(0).clamp(0, max))
        } else {
            (field_shift(&lay, e.caret, r, g), 0)
        };
        let (px, py) = (FIELD_PAD_X * g as f32, FIELD_PAD_Y * g as f32);
        Some(lay.pos_at(pos.0 as f32 - r.x as f32 - px + dx, pos.1 as f32 - r.y as f32 - py + off as f32, line))
    }

    /// An editing key in the text field being edited (PLAYUI1-T): moves, selection, deletes, undo and the clipboard.
    fn edit_key(&mut self, k: EditKey, ctrl: bool, shift: bool) {
        let g = self.gfx.view.g as f32;
        let (lay, rows) = match self.edit_geometry() {
            Some((lay, r, line)) => (lay, ((r.h as f32 - 2.0 * FIELD_PAD_Y * g) / line).floor().max(1.0) as usize),
            // Not drawn yet: one line of the text as it is (no wrapping).
            None => (self.gfx.kit.edit.as_ref().map(|e| Layout::new(&e.text, f32::MAX, false, &mut |_| 1.0)).unwrap_or_default(), 1),
        };
        let mut copied = None;
        self.edit(|e| copied = e.key(k, ctrl, shift, &lay, rows, clipboard::get));
        if let Some(t) = copied {
            clipboard::set(&t);
        }
    }

    /// F8: a playtest note. The screen is captured first, then the note box opens over it with its text field typing
    /// (recording off: a notice says so).
    fn note(&mut self) {
        if !self.cx.playtest.on {
            self.cx.notice.push("Playtest recording is off: turn it on (Scene Runner) to take notes with F8.".into());
            self.settle();
            return;
        }
        if self.stack.last().is_some_and(|t| t.id() == crate::notes::ID) {
            return;
        }
        let c = &self.gfx.canvas;
        let shot = Some((c.w as usize, c.h as usize, c.px.clone()));
        let top = self.stack.last().unwrap();
        let infos = top.widgets();
        let ctl = info_at(&infos, self.mouse).map(trail_ctl).unwrap_or(Value::Null);
        let mut stamp = json!({"scene": top.id(), "battle": -1, "round": 0, "block": 0, "phase": "", "mouse": [self.mouse.0, self.mouse.1], "viewport": self.top_view().canvas(), "ctl": ctl});
        if let Some(ctx) = top.note_context(&self.cx).as_object() {
            for (k, v) in ctx {
                stamp[k] = v.clone();
            }
        }
        self.cx.nav.push(Nav::Push(Box::new(crate::notes::NoteBox::new(shot, stamp))));
        self.settle();
        self.gfx.kit.edit = Some(Edit::new(crate::notes::TEXT_ID, "", true));
    }

    /// Playtest trail: a hover is logged when the pointer leaves a control it rested on for HOVER_MS or longer.
    fn track_hover(&mut self) {
        if !self.cx.playtest.on {
            return;
        }
        let infos = self.stack.last().unwrap().widgets();
        let under = info_at(&infos, self.mouse).map(|w| (w["id"].as_str().unwrap_or("").to_string(), trail_ctl(w)));
        let same = matches!((&self.hover, &under), (Some((a, _, _)), Some((b, _))) if a == b);
        if same {
            return;
        }
        if let Some((_, since, ctl)) = self.hover.take() {
            let ms = since.elapsed().as_millis() as u64;
            if ms >= crate::playtest::HOVER_MS {
                self.cx.playtest.trail(json!({"type": "hover", "ctl": ctl, "ms": ms}));
            }
        }
        self.hover = under.map(|(id, ctl)| (id, aegis_core::clock::Instant::now(), ctl));
    }

    /// The lowest screen drawn: the top one and those under it down to the first that is not an overlay.
    fn drawn_from(&self) -> usize {
        let mut base = self.stack.len() - 1;
        while base > 0 && self.stack[base].overlay() {
            base -= 1;
        }
        base
    }

    /// What the frame loop owes what is on screen now (FPSCAP1): every frame while the shell's own work runs (a text field
    /// and its caret, a drag, a held walk key, a music fade, a queued screen or notice), else the most any drawn screen says.
    fn frame_need(&self) -> FrameNeed {
        let kit = &self.gfx.kit;
        let own = kit.edit.is_some() || kit.drag.is_some() || self.drag.is_some() || self.press.is_some() || self.win_drag.is_some() || self.cx.held.dir.is_some();
        if own || self.cx.audio.fading() || !self.cx.nav.is_empty() || !self.cx.notice.is_empty() {
            return FrameNeed::Full;
        }
        self.stack[self.drawn_from()..].iter().fold(FrameNeed::Idle, |need, s| need.and(s.frame_need(&self.cx)))
    }

    fn frame(&mut self) {
        self.settle();
        self.cx.playtest.tick();
        self.track_hover();
        self.gfx.canvas.clip = None;
        // The saved sound levels; a screen may play draft ones for this frame (the settings popup).
        self.cx.audio.set_levels(&self.cx.settings.v);
        let cx = &mut self.cx;
        self.stack.last_mut().unwrap().tick(cx);
        self.settle();
        // LOOK1-E1: the theme in force (the faces follow it; the screens' `Ui` sync them too).
        let theme = th();
        self.gfx.fonts.sync(&theme);
        self.gfx.canvas.clear(theme.bg);
        let n = self.stack.len();
        for i in self.drawn_from()..n {
            let live = i == n - 1;
            self.gfx.view = self.view_of(self.stack[i].grid());
            let mouse = if live { self.mouse } else { (-1, -1) };
            // the audits record the top screen only (what lies under a popup is covered, not cramped)
            let paused = if live { (None, None) } else { (self.gfx.fonts.audit.take(), self.gfx.fonts.layout.take()) };
            self.stack[i].draw(&mut self.cx, &mut self.gfx, mouse, live);
            if paused.0.is_some() {
                self.gfx.fonts.audit = paused.0;
            }
            if paused.1.is_some() {
                self.gfx.fonts.layout = paused.1;
            }
        }
        self.gfx.canvas.clip = None;
        // The focus ring, the open list, the ghost and the tip are the top screen's: its view stays until the stack changes.
        self.gfx.view = self.view_of(self.stack[n - 1].grid());
        self.settle_focus();
        self.settle_pointer();
        let mut p = Painter { c: &mut self.gfx.canvas, v: self.gfx.view, fonts: &mut self.gfx.fonts };
        if self.gfx.kit.focus_visible && self.gfx.kit.open.is_none() {
            let infos = self.stack[n - 1].widgets();
            if let Some(w) = focus::live(&infos, self.gfx.kit.focus.as_deref(), false).filter(|w| focus::shows_ring(w)) {
                draw_ring(&mut p, info_rect(w));
            }
        }
        if self.gfx.kit.open.is_some() {
            // The open choice list goes over everything the screen drew; no hover card while it is open.
            let infos = self.stack[n - 1].widgets();
            let opts: Vec<&Value> = infos.iter().filter(|w| w["kind"] == "option").collect();
            if opts.is_empty() {
                self.gfx.kit.open = None;
            } else {
                let keyed = self.gfx.kit.focus_visible.then(|| self.gfx.kit.focus.clone()).flatten();
                draw_options(&mut p, &opts, self.mouse, keyed.as_deref());
            }
        } else if let Some(d) = &self.gfx.kit.drag {
            // Dragging: valid targets outlined (the one under the cursor in gold) and the ghost; no hover card.
            let top = &self.stack[n - 1];
            let infos = top.widgets();
            for (i, w) in infos.iter().enumerate().filter(|(_, w)| w["enabled"] == json!(true) && w["drop"].is_string()) {
                let r = info_rect(w);
                // A target whose centre a later window backdrop covers takes no drop there (`drop_at`): no outline.
                let covered = infos[i + 1..].iter().any(|o| o["kind"] == "backdrop" && info_rect(o).contains(r.x + r.w / 2, r.y + r.h / 2));
                if !covered && top.can_drop(&self.cx, &d.key, w["drop"].as_str().unwrap_or("")) {
                    let over = r.contains(self.mouse.0, self.mouse.1) && drop_key_at(&infos, self.mouse) == w["drop"].as_str();
                    draw_drop_target(&mut p, r, over);
                }
            }
            draw_ghost(&mut p, &d.text, self.mouse);
        } else {
            // A tip is also reachable by focus (LAYOUT720 section 11); a screen still on the old numbers keeps today's
            // picture, where only the mouse shows one, until its own task moves it.
            let keyed = self.gfx.kit.focus.as_deref().filter(|_| self.gfx.kit.focus_visible && self.stack[n - 1].grid() == Grid::Native);
            let at = keyed.map_or(self.mouse, |f| focus::tip_point(&self.stack[n - 1].widgets(), self.mouse, Some(f)));
            self.stack[n - 1].draw_tip(&mut p, at);
        }
        self.settle();
        let cx = &self.cx;
        let track = self.stack.iter().rev().find_map(|s| s.music(cx));
        cx.audio.music(track.as_deref());
        cx.audio.tick();
        self.frames += 1;
    }

    /// After the top screen drew: a focus whose widget is gone moves to the first one (when it was showing), an open
    /// list the keyboard opened focuses its picked option, a screen that wants it starts with a visible focus.
    fn settle_focus(&mut self) {
        let top = self.stack.last().unwrap();
        let infos = top.widgets();
        self.text_rows = infos.iter().any(|w| !focus::shows_ring(w));
        let open = self.gfx.kit.open.is_some();
        if open && self.open_focus.is_some() && !self.gfx.kit.focus.as_deref().is_some_and(|f| infos.iter().any(|w| w["id"] == f && w["kind"] == "option")) {
            let picked = infos.iter().find(|w| w["kind"] == "option" && w["active"] == json!(true)).and_then(|w| w["id"].as_str()).map(String::from);
            self.gfx.kit.focus = picked.or_else(|| focus::step(&infos, None, focus::Move::Next, true));
            return;
        }
        let present = focus::live(&infos, self.gfx.kit.focus.as_deref(), open).is_some();
        if !present && (self.gfx.kit.focus_visible || (top.auto_focus() && self.gfx.kit.focus.is_none())) {
            let first = top.focus_first().filter(|f| focus::live(&infos, Some(f), open).is_some()).map(String::from);
            self.gfx.kit.focus = first.or_else(|| focus::step(&infos, None, focus::Move::Next, open));
            self.gfx.kit.focus_visible = self.gfx.kit.focus.is_some();
        }
        // No focus showing, and a menu row lit under a mouse that has not moved: it is the active item, so Enter presses it.
        if !self.gfx.kit.focus_visible {
            self.gfx.kit.mouse_moved(&infos, self.mouse);
        }
    }

    /// After a pushed screen first drew (ASKNO2): the centre of its `Screen::pointer_to` widget becomes the pointer
    /// request the windowed shell carries out; a screen an agent-link command opened asks for nothing.
    fn settle_pointer(&mut self) {
        if !std::mem::take(&mut self.pointer_pending) || self.agent {
            return;
        }
        let top = self.stack.last().unwrap();
        self.pointer_req = top.pointer_to().and_then(|id| top.widget_rect(id)).map(|(r, _)| (r.x + r.w / 2, r.y + r.h / 2));
    }

    /// The wheel: the topmost scroll area under the mouse scrolls (a window above it stops the search), else the
    /// screen gets the lines (`Screen::wheel`).
    fn wheel(&mut self, lines: i32) {
        // An open choice list closes rather than the area under it scrolling.
        if self.gfx.kit.open.take().is_some() {
            return;
        }
        let infos = self.stack.last().unwrap().widgets();
        let under = infos.iter().rev().find(|w| (w["kind"] == "scroll" || w["kind"] == "backdrop") && info_rect(w).contains(self.mouse.0, self.mouse.1));
        if let Some(area) = under.filter(|w| w["kind"] == "scroll") {
            let id = area["id"].as_str().unwrap_or("").to_string();
            let max = area["text"].as_str().and_then(|t| t.parse::<i32>().ok()).unwrap_or(0);
            // An offset set past the end ("scroll to the end") counts from the end.
            let at = self.gfx.kit.scroll.get(&id).copied().unwrap_or(0).clamp(0, max);
            self.gfx.kit.scroll.insert(id, (at - lines * self.gfx.view.u(crate::ui::SCROLL_STEP)).clamp(0, max));
            return;
        }
        let cx = &mut self.cx;
        self.stack.last_mut().unwrap().wheel(cx, lines);
    }

    /// Drops a press or drag in progress; true when there was one.
    fn cancel_drag(&mut self) -> bool {
        self.text_press = false;
        let moved = self.win_drag.take().is_some();
        if moved {
            self.save_desk();
        }
        let had = self.press.take().is_some() | moved;
        self.gfx.kit.drag.take().is_some() || had
    }

    /// The mouse moved: a menu row without a panel under it becomes the active item (`Kit::mouse_moved`).
    fn mouse_moved(&mut self) {
        // Most screens have none: no widget list built on every move (`settle_focus` notes it each frame).
        if !self.text_rows {
            return;
        }
        let infos = self.stack.last().unwrap().widgets();
        self.gfx.kit.mouse_moved(&infos, self.mouse);
    }

    fn key(&mut self, k: Key) {
        // Keys typed into a text field are not part of the trail (PLAYTEST1: skipped while a text field has focus).
        if self.gfx.kit.edit.is_none() {
            self.cx.playtest.trail(json!({"type": "key", "key": k.name(), "mods": ""}));
        }
        // Any key drops a press or drag in progress (what it changes could move the target); Esc does only that.
        let cancelled = self.cancel_drag();
        if k == Key::Esc && cancelled {
            return;
        }
        if k == Key::Esc {
            if let Some(field) = self.gfx.kit.open.take() {
                // Back on the field the list belongs to.
                self.open_focus = None;
                if self.gfx.kit.focus_visible {
                    self.gfx.kit.focus = Some(field);
                }
                return;
            }
        }
        if k != Key::Esc && self.focus_key(k) {
            self.settle();
            return;
        }
        let cx = &mut self.cx;
        let top = self.stack.last_mut().unwrap();
        if k == Key::Esc {
            top.escape(cx);
        } else {
            top.key(cx, k);
        }
        self.settle();
    }

    /// The keyboard focus (PORT-FOCUS): Tab / Shift+Tab move it, the arrows too unless the screen steers with them
    /// (`Screen::arrow_focus`; inside an open choice list they always do), Enter or Space on a visible focus presses
    /// the widget (a choice field opens its list, an option is picked), Left / Right step a focused slider. True when
    /// the key was used here (the screen does not get it).
    fn focus_key(&mut self, k: Key) -> bool {
        let infos = self.stack.last().unwrap().widgets();
        let open = self.gfx.kit.open.is_some();
        let kind_of = |id: &str| infos.iter().find(|w| w["id"] == id).and_then(|w| w["kind"].as_str()).unwrap_or("").to_string();
        let current = focus::live(&infos, self.gfx.kit.focus.as_deref(), open).and_then(|w| w["id"].as_str()).map(String::from);
        let visible = self.gfx.kit.focus_visible && current.is_some();
        let slider = current.as_deref().filter(|f| visible && !open && kind_of(f) == "slider").map(String::from);
        if let (Some(id), Key::Left | Key::Right) = (slider, k) {
            // The step beside the active one.
            let prefix = format!("{id}.step.");
            let steps: Vec<&Value> = infos.iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with(&prefix))).collect();
            let at = steps.iter().position(|w| w["active"] == json!(true)).unwrap_or(0);
            let to = if k == Key::Left { at.saturating_sub(1) } else { (at + 1).min(steps.len().saturating_sub(1)) };
            if let Some(step) = steps.get(to).and_then(|w| w["id"].as_str()).map(String::from) {
                let cx = &mut self.cx;
                let _ = self.stack.last_mut().unwrap().click_id(cx, &step);
            }
            return true;
        }
        let arrows = open || self.stack.last().unwrap().arrow_focus();
        let m = match k {
            Key::Tab => Some(focus::Move::Next),
            Key::BackTab => Some(focus::Move::Prev),
            Key::Up if arrows => Some(focus::Move::Dir(0, -1)),
            Key::Down if arrows => Some(focus::Move::Dir(0, 1)),
            Key::Left if arrows => Some(focus::Move::Dir(-1, 0)),
            Key::Right if arrows => Some(focus::Move::Dir(1, 0)),
            _ => None,
        };
        if let Some(m) = m {
            // The first key only shows where the focus is.
            let from = if visible { current.as_deref() } else { None };
            let next = match (from, current.as_deref()) {
                (None, Some(c)) if !matches!(m, focus::Move::Next | focus::Move::Prev) => Some(c.to_string()),
                _ => focus::step(&infos, from, m, open),
            };
            if next.is_some() {
                self.gfx.kit.focus = next;
                self.gfx.kit.focus_visible = true;
                return true;
            }
            return false;
        }
        if !matches!(k, Key::Enter | Key::Space) || !visible {
            return false;
        }
        let id = current.unwrap();
        match kind_of(&id).as_str() {
            "select" => {
                self.gfx.kit.open = Some(id.clone());
                self.gfx.kit.focus = None;
                self.open_focus = Some(id);
            }
            "slider" => {}
            kind => {
                let picked = kind == "option";
                let cx = &mut self.cx;
                let _ = self.stack.last_mut().unwrap().click_id(cx, &id);
                if picked {
                    // Back on the field the list belongs to.
                    self.gfx.kit.focus = self.gfx.kit.open.take().or(self.open_focus.take());
                }
            }
        }
        true
    }

    /// A left click with its playtest trail events, logged before the click runs (as Godot's `pressed` comes before the
    /// panel it opens): `click` (the control; `handled` set once the click ran) and `button` when the click goes to a
    /// clickable widget (not a click that only closes an open list or starts a drag).
    fn click(&mut self) {
        if !self.cx.playtest.on {
            self.click_inner();
            return;
        }
        let infos = self.stack.last().unwrap().widgets();
        let at = info_at(&infos, self.mouse).cloned();
        let ctl = at.as_ref().map(trail_ctl).unwrap_or(Value::Null);
        let list_open = self.gfx.kit.open.is_some();
        let to_widget = at.as_ref().is_some_and(|w| w["clickable"] == json!(true) && !w["drag"].is_string() && (!list_open || w["kind"] == "option"));
        let at_click = self.cx.playtest.trail(json!({"type": "click", "button": "left", "pos": [self.mouse.0, self.mouse.1], "ctl": ctl, "handled": at.is_some()}));
        if to_widget {
            self.cx.playtest.trail(json!({"type": "button", "ctl": ctl}));
        }
        if self.click_inner() && at.is_none() {
            if let Some(i) = at_click {
                self.cx.playtest.set_trail(i, "handled", json!(true));
            }
        }
    }

    /// A left click: opens or closes a choice list, picks from it, starts a slider drag, or clicks the widget; true
    /// when something took it.
    fn click_inner(&mut self) -> bool {
        // A press or drag whose release was lost (focus moved away mid-drag) ends here.
        self.cancel_drag();
        let pos = self.mouse;
        let infos = self.stack.last().unwrap().widgets();
        let at = info_at(&infos, pos).cloned();
        let kind = at.as_ref().map(|w| w["kind"].as_str().unwrap_or("").to_string()).unwrap_or_default();
        // A text field starts editing on a click, the caret where the click was (Shift+click in the field being edited
        // extends the selection, a double click selects a word, a drag from the press selects); a click anywhere else
        // ends an edit.
        if (kind == "text" || kind == "textarea") && at.as_ref().is_some_and(|w| w["enabled"] == json!(true)) {
            let w = at.as_ref().unwrap();
            let id = w["id"].as_str().unwrap_or("").to_string();
            let same = self.gfx.kit.edit.as_ref().is_some_and(|e| e.id == id);
            if !same {
                self.gfx.kit.edit = Some(Edit::new(&id, w["text"].as_str().unwrap_or(""), kind == "textarea"));
            }
            let near = self.gfx.view.u(DOUBLE_CLICK_PX);
            let double = same && self.text_click.is_some_and(|(t, p)| t.elapsed().as_millis() < DOUBLE_CLICK_MS && (p.0 - pos.0).abs() <= near && (p.1 - pos.1).abs() <= near);
            self.text_click = if double { None } else { Some((Instant::now(), pos)) };
            let extend = self.shift && same;
            if let Some(c) = self.caret_at(pos) {
                self.edit(|e| {
                    e.goal_x = None;
                    if double {
                        e.select_word(c);
                    } else {
                        e.move_to(c, extend);
                    }
                });
            }
            self.text_press = !double;
            return true;
        }
        self.gfx.kit.edit = None;
        let keep = self.gfx.kit.focus_visible && focus::click_keeps_focus(&infos, self.gfx.kit.focus.as_deref(), at.as_ref());
        self.gfx.kit.focus_visible = keep;
        self.open_focus = None;
        if self.gfx.kit.open.take().is_some() {
            // An open list: a pick goes through, any other click only closes it.
            if kind == "option" {
                let cx = &mut self.cx;
                self.stack.last_mut().unwrap().click(cx, pos);
            }
            self.settle();
            return true;
        }
        let id = at.as_ref().map(|w| w["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
        if at.as_ref().is_some_and(|w| focus::focusable(w, false)) {
            self.gfx.kit.focus = Some(id.clone());
        }
        if self.desk_press(&id, pos) {
            return true;
        }
        // A press on a window's content brings the window to the front (the content still gets the click).
        if let Some(d) = self.stack.last_mut().unwrap().desk() {
            if let Some(w) = d.top_at(pos).map(String::from) {
                d.raise(&w);
            }
        }
        if kind == "select" {
            self.gfx.kit.open = Some(id);
            return true;
        }
        if let Some(key) = at.as_ref().and_then(|w| w["drag"].as_str()) {
            // A draggable widget: a move picks it up (`drag_to`), a release without one clicks it (`release`).
            let text = at.as_ref().and_then(|w| w["text"].as_str()).unwrap_or("").to_string();
            self.press = Some(Press { id, key: key.to_string(), text, at: pos });
            return true;
        }
        if kind == "slider_step" {
            if let Some((slider, _)) = id.rsplit_once(".step.") {
                self.drag = Some((slider.to_string(), id.clone()));
            }
        }
        let cx = &mut self.cx;
        let handled = self.stack.last_mut().unwrap().click(cx, pos);
        self.settle();
        handled
    }

    /// The mouse moved: a press on a draggable widget becomes a drag once it moved DRAG_START pixels; a slider drag
    /// follows it along the track (the step under its x).
    fn drag_to(&mut self) {
        if self.text_press {
            // Selecting in a text field: the caret follows the mouse, the selection from the press.
            if let Some(c) = self.caret_at(self.mouse) {
                self.edit(|e| {
                    e.goal_x = None;
                    e.move_to(c, true);
                });
            }
            return;
        }
        if let Some((win, grab, last)) = self.win_drag.clone() {
            if let Some(d) = self.stack.last_mut().unwrap().desk() {
                d.drag(&win, grab, self.mouse.0 - last.0, self.mouse.1 - last.1);
            }
            self.win_drag = Some((win, grab, self.mouse));
            return;
        }
        if let Some(p) = &self.press {
            let start = self.gfx.view.u(DRAG_START);
            if self.gfx.kit.drag.is_none() && ((self.mouse.0 - p.at.0).abs() > start || (self.mouse.1 - p.at.1).abs() > start) {
                self.gfx.kit.drag = Some(Drag { key: p.key.clone(), id: p.id.clone(), text: p.text.clone() });
            }
            return;
        }
        let Some((slider, last)) = self.drag.clone() else { return };
        let prefix = format!("{slider}.step.");
        let infos = self.stack.last().unwrap().widgets();
        let step = infos.iter().find(|w| w["id"].as_str().is_some_and(|i| i.starts_with(&prefix)) && {
            let r = info_rect(w);
            self.mouse.0 >= r.x && self.mouse.0 < r.x + r.w
        });
        let step = step.or_else(|| {
            // Past either end: the first or last step.
            let steps: Vec<&Value> = infos.iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with(&prefix))).collect();
            let first = steps.first().map(|w| info_rect(w))?;
            if self.mouse.0 < first.x { steps.first().copied() } else { steps.last().copied() }
        });
        let Some(step) = step else { return };
        let id = step["id"].as_str().unwrap_or("").to_string();
        if id != last {
            let r = info_rect(step);
            let cx = &mut self.cx;
            self.stack.last_mut().unwrap().click(cx, (r.x + r.w / 2, r.y + r.h / 2));
            self.drag = Some((slider, id));
            self.settle();
        }
    }

    /// The left button came up: a drag drops on the target under the cursor when the screen allows it (else it is
    /// cancelled), a press on a draggable widget that never moved clicks it, a slider drag ends.
    fn release(&mut self) {
        self.drag = None;
        self.text_press = false;
        if self.win_drag.take().is_some() {
            self.save_desk();
        }
        let Some(p) = self.press.take() else { return };
        match self.gfx.kit.drag.take() {
            Some(d) => {
                if let Some(to) = self.drop_at(&d.key) {
                    let cx = &mut self.cx;
                    self.stack.last_mut().unwrap().accept_drop(cx, &d.key, &to);
                }
            }
            None => {
                let cx = &mut self.cx;
                self.stack.last_mut().unwrap().click(cx, p.at);
            }
        }
        self.settle();
    }

    /// The drop key of the topmost enabled drop target under the mouse, when it accepts payload `from`.
    fn drop_at(&self, from: &str) -> Option<String> {
        let top = self.stack.last().unwrap();
        let infos = top.widgets();
        let to = drop_key_at(&infos, self.mouse)?;
        top.can_drop(&self.cx, from, to).then(|| to.to_string())
    }

    /// A press on a desk widget (UIKIT3): `win.<id>.<part>` raises the window and moves it (title), resizes it (grip),
    /// closes or collapses it; `desk.toggle.<id>` and `desk.layout.<name>` are the toolbar's. False when `id` is none of
    /// these or the top screen has no desk.
    fn desk_press(&mut self, id: &str, pos: (i32, i32)) -> bool {
        let Some(desk) = self.stack.last_mut().unwrap().desk() else { return false };
        if let Some((win, part)) = id.strip_prefix("win.").and_then(|r| r.rsplit_once('.')) {
            if desk.win(win).is_none() {
                return false;
            }
            desk.raise(win);
            match part {
                "title" => self.win_drag = Some((win.to_string(), Grab::Move, pos)),
                "grip" => self.win_drag = Some((win.to_string(), Grab::Resize, pos)),
                "close" => desk.set_open(win, false),
                "collapse" => desk.collapse(win),
                _ => {}
            }
        } else if let Some((what, name)) = id.strip_prefix("desk.").and_then(|r| r.split_once('.')) {
            match what {
                "toggle" => desk.toggle(name),
                "layout" => {
                    desk.apply_layout(name);
                }
                _ => return false,
            }
        } else {
            return false;
        }
        if self.win_drag.is_none() && !id.ends_with(".body") {
            self.save_desk();
        }
        true
    }

    /// Saves the top screen's desk in the settings (`desks.<screen id>`).
    fn save_desk(&mut self) {
        let top = self.stack.last_mut().unwrap();
        let id = top.id();
        let Some(d) = top.desk() else { return };
        if !self.cx.settings.v["desks"].is_object() {
            self.cx.settings.v["desks"] = json!({});
        }
        self.cx.settings.v["desks"][id] = d.to_value();
        let _ = self.cx.settings.save();
    }

    fn back(&mut self) {
        if self.cx.playtest.on {
            let infos = self.stack.last().unwrap().widgets();
            let ctl = info_at(&infos, self.mouse).map(trail_ctl).unwrap_or(Value::Null);
            self.cx.playtest.trail(json!({"type": "click", "button": "right", "pos": [self.mouse.0, self.mouse.1], "ctl": ctl, "handled": true}));
        }
        if self.cancel_drag() || self.gfx.kit.open.take().is_some() {
            return;
        }
        let cx = &mut self.cx;
        self.stack.last_mut().unwrap().back(cx);
        self.settle();
    }

    /// TEXTFIT1: draws one frame with the painter's text audit on, the mouse off the canvas (no hover card) and the text
    /// size at `scale` when given; reports the screen, the scale and `text_audit::issues` (saving that frame to `shot`
    /// when given), then draws again as it was.
    fn text_audit(&mut self, scale: Option<f32>, shot: Option<&Path>) -> Value {
        let out = self.audit_frame(scale, shot).report;
        self.frame();
        out
    }

    /// One frame with both audits on (`text_audit`'s report, the layout audit under `layout`), the scale and the mouse
    /// put back. The picture is left as the audit drew it, for the caller's last frame to replace.
    fn audit_frame(&mut self, scale: Option<f32>, shot: Option<&Path>) -> layout_audit::FrameAudit {
        let (old_scale, old_mouse) = (self.gfx.fonts.scale, self.mouse);
        if let Some(s) = scale {
            self.gfx.fonts.scale = s;
        }
        self.mouse = layout_audit::NO_MOUSE;
        self.gfx.fonts.audit = Some(Vec::new());
        self.gfx.fonts.layout = Some(Vec::new());
        self.frame();
        let (recs, laid) = (self.gfx.fonts.audit.take().unwrap_or_default(), self.gfx.fonts.layout.take().unwrap_or_default());
        let top = self.stack.last();
        let (screen, grid, widgets) = (top.map_or("", |s| s.id()), top.map_or(Grid::Old, |s| s.grid()), top.map(|s| s.widgets()).unwrap_or_default());
        let mut audit = layout_audit::frame_report(self.top_view(), grid, &recs, &laid, &widgets);
        let out = &mut audit.report;
        out["screen"] = json!(screen);
        out["scale"] = json!(self.gfx.fonts.scale);
        if let Some(path) = shot {
            out["shot"] = match save_png(&self.gfx.canvas, path, 1280) {
                Ok(()) => json!(path.display().to_string()),
                Err(e) => json!(format!("not saved: {e}")),
            };
        }
        self.gfx.fonts.scale = old_scale;
        self.mouse = old_mouse;
        audit
    }

    /// Runs `f` with the window, the device and the policy of `profile`, then puts them back, with the keyboard's focus and
    /// any open list (`Profile::enter`). The mouse stays in the units it was in, and the caller draws a frame once it is done.
    fn on_profile<R>(&mut self, profile: &Profile, f: impl FnOnce(&mut Self) -> R) -> R {
        let game = self.policy;
        let entered = profile.enter(&mut self.gfx, game);
        self.policy = profile.policy_for(game);
        self.gfx.view = self.top_view();
        let out = f(self);
        self.policy = game;
        entered.leave(&mut self.gfx);
        out
    }

    /// LAYOUT720-E5: the top screen through the size profiles (`only`: that one) at the four text sizes, then drawn as it
    /// was (`layout_audit::sweep`'s answer). A screen in old numbers is not audited, and nothing is drawn for it.
    fn layout_audit(&mut self, only: Option<&Profile>) -> Value {
        let grid = self.stack.last().map_or(Grid::Old, |t| t.grid());
        if grid != Grid::Native {
            return layout_audit::not_audited(grid);
        }
        let breakpoints = Metrics::of(&th(), self.gfx.view).breakpoints;
        let game = self.policy;
        let out = layout_audit::sweep(game, only, breakpoints, |p, scale| self.on_profile(p, |s| s.audit_frame(Some(scale), None)));
        self.frame();
        out
    }

    fn shot(&mut self, path: &Path, max_w: i32) -> Result<()> {
        save_png(&self.gfx.canvas, path, max_w)
    }

    /// F12: the shot, noted; one that fails also shows, since nothing else says that no file came.
    fn f12_shot(&mut self) {
        let p = self.cx.shot_dir.join("gui_shot.png");
        let msg = match self.shot(&p, 0) {
            Ok(()) => format!("Screenshot: {}", p.display()),
            Err(e) => {
                self.cx.notice.push(format!("Screenshot failed: {e}"));
                format!("Screenshot failed: {e}")
            }
        };
        self.cx.app.note(msg);
    }

    /// Agent link requests.
    fn bridge(&mut self, body: &Value) -> Value {
        self.agent = true;
        let reply = self.bridge_cmd(body);
        // Screens the command opened never move the pointer (ASKNO2).
        self.pointer_pending = false;
        self.agent = false;
        reply
    }

    fn bridge_cmd(&mut self, body: &Value) -> Value {
        let cmd = body.get("cmd").and_then(Value::as_str).unwrap_or("");
        let reply = match cmd {
            "ui_tree" => {
                let cx = &self.cx;
                let top = self.stack.last().unwrap();
                json!({"ok": true, "screen": top.id(), "stack": self.stack.iter().map(|s| s.id()).collect::<Vec<_>>(), "canvas": self.top_view().canvas(), "grid": top.grid().name(), "mouse": [self.mouse.0, self.mouse.1],
                    "frame": self.frames, "pid": std::process::id(), "px": [self.gfx.canvas.w, self.gfx.canvas.h],
                    "music": cx.audio.wanted_music(), "focus": self.gfx.kit.focus, "focus_visible": self.gfx.kit.focus_visible, "state": top.state(cx), "game": cx.app.agent_summary(), "widgets": top.widgets(),
                    "edit": self.gfx.kit.edit.as_ref().map(|e| json!({"id": e.id, "caret": e.caret, "selection": e.selection(), "selected": e.selected(), "undo": e.history().0}))})
            }
            "click" => {
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                let infos = self.stack.last().unwrap().widgets();
                let target = infos.iter().find(|w| w["id"] == id).cloned();
                // As a mouse click: the ring hides, the clicked widget takes the focus.
                let keep = self.gfx.kit.focus_visible && focus::click_keeps_focus(&infos, self.gfx.kit.focus.as_deref(), target.as_ref());
                self.gfx.kit.focus_visible = keep;
                self.open_focus = None;
                if target.as_ref().is_some_and(|w| focus::focusable(w, false)) {
                    self.gfx.kit.focus = Some(id.to_string());
                }
                if target.as_ref().is_some_and(|w| w["kind"].as_str().is_some_and(|k| k.starts_with("win_") || k == "desk_button")) {
                    // A window or toolbar control: what the press does (no window drag through a click).
                    let r = target.as_ref().map(info_rect).unwrap_or_default();
                    self.desk_press(id, (r.x + r.w / 2, r.y + r.h / 2));
                    self.win_drag = None;
                    json!({"ok": true, "clicked": id})
                } else if target.as_ref().is_some_and(|w| w["kind"] == "select" && w["enabled"] == json!(true)) {
                    // A select opens (or closes) its list; the reply names the options to click.
                    let open = self.gfx.kit.open.as_deref() != Some(id);
                    self.gfx.kit.open = open.then(|| id.to_string());
                    self.frame();
                    let options: Vec<Value> = self.stack.last().unwrap().widgets().into_iter().filter(|w| w["kind"] == "option").map(|w| json!({"id": w["id"], "text": w["text"]})).collect();
                    json!({"ok": true, "clicked": id, "open": open, "options": options})
                } else {
                    // Any other click closes an open list (a pick from it still goes through).
                    self.gfx.kit.open = None;
                    let cx = &mut self.cx;
                    match self.stack.last_mut().unwrap().click_id(cx, id) {
                        Ok(()) => json!({"ok": true, "clicked": id}),
                        Err(e) => json!({"ok": false, "error": e}),
                    }
                }
            }
            "drag" => {
                // Press on widget `id`, move to `to` ([x, y] or a widget id, its centre) in 8 steps, release: the real
                // mouse path (slider drags, drag and drop).
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                let infos = self.stack.last().unwrap().widgets();
                let to = match body["to"].as_str() {
                    Some(target) => match infos.iter().find(|w| w["id"] == target).map(info_rect) {
                        Some(r) => (r.x + r.w / 2, r.y + r.h / 2),
                        None => return json!({"ok": false, "error": format!("no widget '{target}'")}),
                    },
                    None => (body["to"][0].as_i64().unwrap_or(0) as i32, body["to"][1].as_i64().unwrap_or(0) as i32),
                };
                match infos.iter().find(|w| w["id"] == id).map(info_rect) {
                    Some(r) => {
                        let from = (r.x + r.w / 2, r.y + r.h / 2);
                        self.mouse = from;
                        self.click();
                        for t in 1..=8 {
                            self.mouse = (from.0 + (to.0 - from.0) * t / 8, from.1 + (to.1 - from.1) * t / 8);
                            self.drag_to();
                        }
                        self.release();
                        json!({"ok": true, "dragged": id, "to": [to.0, to.1]})
                    }
                    None => json!({"ok": false, "error": format!("no widget '{id}'")}),
                }
            }
            "scroll" => {
                // {id, by: lines} (positive scrolls up like the wheel) or {id, to: offset}.
                let id = body["id"].as_str().unwrap_or("");
                let infos = self.stack.last().unwrap().widgets();
                match infos.iter().find(|w| w["id"] == id && w["kind"] == "scroll") {
                    Some(area) => {
                        let max = area["text"].as_str().and_then(|t| t.parse::<i32>().ok()).unwrap_or(0);
                        let at = self.gfx.kit.scroll.get(id).copied().unwrap_or(0).clamp(0, max);
                        let next = match body["to"].as_i64() {
                            Some(to) => to as i32,
                            None => at - body["by"].as_i64().unwrap_or(0) as i32 * self.gfx.view.u(crate::ui::SCROLL_STEP),
                        }
                        .clamp(0, max);
                        self.gfx.kit.scroll.insert(id.to_string(), next);
                        json!({"ok": true, "offset": next, "max": max})
                    }
                    None => json!({"ok": false, "error": format!("no scroll area '{id}'")}),
                }
            }
            "windows" => match self.stack.last_mut().unwrap().desk() {
                Some(d) => json!({"ok": true, "desk": d.info()}),
                None => json!({"ok": false, "error": "this screen has no windows"}),
            },
            "window" => {
                // {id, op: open | close | raise | collapse | move | size, by: [dx, dy]} or {layout}.
                let id = body["id"].as_str().unwrap_or("").to_string();
                let op = body["op"].as_str().unwrap_or("raise");
                let by = (body["by"][0].as_i64().unwrap_or(0) as i32, body["by"][1].as_i64().unwrap_or(0) as i32);
                let reply = match self.stack.last_mut().unwrap().desk() {
                    None => json!({"ok": false, "error": "this screen has no windows"}),
                    Some(d) => {
                        if let Some(l) = body["layout"].as_str() {
                            if d.apply_layout(l) {
                                json!({"ok": true, "desk": d.info()})
                            } else {
                                json!({"ok": false, "error": format!("no layout '{l}'")})
                            }
                        } else if d.win(&id).is_none() {
                            json!({"ok": false, "error": format!("no window '{id}'")})
                        } else {
                            match op {
                                "open" => d.set_open(&id, true),
                                "close" => d.set_open(&id, false),
                                "collapse" => d.collapse(&id),
                                "move" => d.drag(&id, Grab::Move, by.0, by.1),
                                "size" => d.drag(&id, Grab::Resize, by.0, by.1),
                                "raise" => d.raise(&id),
                                _ => return json!({"ok": false, "error": format!("unknown op '{op}' (open, close, raise, collapse, move, size)")}),
                            }
                            json!({"ok": true, "desk": d.info()})
                        }
                    }
                };
                if reply["ok"] == json!(true) {
                    self.save_desk();
                }
                reply
            }
            "hover" => {
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                match self.stack.last().unwrap().widget_rect(id) {
                    Some((r, tip)) => {
                        self.mouse = (r.x + r.w / 2, r.y + r.h / 2);
                        self.mouse_moved();
                        json!({"ok": true, "tip": tip})
                    }
                    None => json!({"ok": false, "error": format!("no widget '{id}'")}),
                }
            }
            "key" => {
                let name = body.get("key").and_then(Value::as_str).unwrap_or("");
                // While a text field edits, its editing keys go to it (`shift+left`, `ctrl+c`, `ctrl+z`, `home`...).
                match (self.gfx.kit.edit.is_some().then(|| EditKey::parse(name)).flatten(), Key::parse(name)) {
                    (Some((k, ctrl, shift)), _) => {
                        self.edit_key(k, ctrl, shift);
                        json!({"ok": true})
                    }
                    (None, Some(k)) => {
                        // As the keyboard does while a field edits: Enter adds a line to a text area; Enter, Esc and Tab
                        // end the edit and reach the screen (Ctrl+Enter submits and keeps it).
                        let multiline = self.gfx.kit.edit.as_ref().map(|e| e.multiline);
                        match (k, multiline) {
                            (Key::Enter, Some(true)) => self.edit(|e| e.insert("\n")),
                            (Key::Enter | Key::Esc | Key::Tab | Key::BackTab, Some(_)) => {
                                self.gfx.kit.edit = None;
                                self.key(k);
                            }
                            _ => self.key(k),
                        }
                        json!({"ok": true})
                    }
                    (None, None) => json!({"ok": false, "error": "key needs one of up, down, left, right, enter, space, esc, tab, backtab, pageup, pagedown or a character; while a text field edits also home, end, backspace, delete, ctrl+a / c / x / v / z / y with ctrl+ and shift+"}),
                }
            }
            "back" => {
                self.back();
                json!({"ok": true})
            }
            // The text field being edited (a click on it starts one; F8 starts the note's) takes `text` as typed.
            "type" => match (self.gfx.kit.edit.is_some(), body.get("text").and_then(Value::as_str)) {
                (true, Some(t)) => {
                    let t = t.to_string();
                    self.edit(|e| e.insert(&t));
                    json!({"ok": true})
                }
                (false, _) => json!({"ok": false, "error": "no text field is being edited: click one first"}),
                (true, None) => json!({"ok": false, "error": "type needs text"}),
            },
            // F8: a playtest note on the screen as it is (recording on).
            "note" => {
                self.note();
                let open = self.stack.last().is_some_and(|t| t.id() == crate::notes::ID);
                if open {
                    json!({"ok": true})
                } else {
                    json!({"ok": false, "error": self.cx.notice.last().cloned().unwrap_or_else(|| "no note box".into())})
                }
            }
            // TEXTFIT1: one frame drawn with the text audit on (at `scale` when given, the saved text size after); the layout
            // audit reads the same frame, under `layout` (LAYOUT720-E5). `profile`: on that window and device.
            "text_audit" => {
                let scale = body.get("scale").and_then(Value::as_f64).map(|s| s.clamp(0.5, 2.0) as f32);
                let shot = body.get("shot").and_then(Value::as_str).map(PathBuf::from);
                match body.get("profile").and_then(Value::as_str).map(layout_audit::profile_named).transpose() {
                    Err(e) => json!({"ok": false, "error": e}),
                    Ok(profile) => {
                        let mut out = match profile {
                            None => self.text_audit(scale, shot.as_deref()),
                            Some(p) => {
                                let out = self.on_profile(p, |s| s.audit_frame(scale, shot.as_deref())).report;
                                self.frame();
                                out
                            }
                        };
                        out["ok"] = json!(true);
                        out
                    }
                }
            }
            // LAYOUT720-E5: the top screen through every size profile at the four text sizes (`profile`: that one).
            "layout_audit" => match body.get("profile").and_then(Value::as_str).map(layout_audit::profile_named).transpose() {
                Err(e) => json!({"ok": false, "error": e}),
                Ok(only) => {
                    let mut out = self.layout_audit(only);
                    out["ok"] = json!(true);
                    out["screen"] = json!(self.stack.last().map_or("", |s| s.id()));
                    out
                }
            },
            "screenshot" => {
                let path = body.get("path").and_then(Value::as_str).map(PathBuf::from).unwrap_or_else(|| self.cx.shot_dir.join("gui_shot.png"));
                let max_w = body.get("max_w").and_then(Value::as_u64).unwrap_or(1280) as i32;
                match self.shot(&path, max_w) {
                    Ok(()) => json!({"ok": true, "path": path.display().to_string()}),
                    Err(e) => json!({"ok": false, "error": e.to_string()}),
                }
            }
            "game" => self.cx.app.agent_game(),
            // As the Quit button: the screens leave and the recording is saved after this frame (the driver's clean stop).
            "quit" => {
                self.cx.quit = true;
                json!({"ok": true})
            }
            "_bad" => json!({"ok": false, "error": body["error"]}),
            _ => {
                let cx = &self.cx;
                match self.stack.last_mut().unwrap().command(cx, body) {
                    Some(reply) => reply,
                    None => json!({"ok": false, "error": format!("unknown command '{cmd}' ({})", G::MORE_COMMANDS)}),
                }
            }
        };
        self.settle();
        reply
    }
}

/// The part of `monitor` a window may use, in physical px. Windows: the monitor's work area (`GetMonitorInfoW`'s
/// `rcWork`: the monitor without the taskbar and docked bars; a hidden taskbar leaves it whole). Elsewhere winit offers
/// no work area, so it is the whole monitor and a panel may still cover a window's edge.
fn usable_area(monitor: &MonitorHandle) -> ScreenRect {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFO};
        use winit::platform::windows::MonitorHandleExtWindows;
        // SAFETY: MONITORINFO is plain integers, so all zeroes is valid; `cbSize` is set as the call requires, and the
        // call only writes through the pointer, which lives for the call. A stale handle returns 0 and writes nothing.
        let (ok, work) = unsafe {
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            let ok = GetMonitorInfoW(monitor.hmonitor(), &mut info);
            (ok != 0, info.rcWork)
        };
        let (w, h) = (i64::from(work.right) - i64::from(work.left), i64::from(work.bottom) - i64::from(work.top));
        if ok && w > 0 && h > 0 {
            return ScreenRect { x: work.left, y: work.top, w: w as u32, h: h as u32 };
        }
    }
    whole_monitor(monitor)
}

fn whole_monitor(monitor: &MonitorHandle) -> ScreenRect {
    let (at, size) = (monitor.position(), monitor.size());
    ScreenRect { x: at.x, y: at.y, w: size.width, h: size.height }
}

/// Sizes a window that is not fullscreen (`window_size` is real pixels, so 1280 x 720 is 1280 x 720 px at any display
/// scale) to fit the usable area of the monitor it is on, then places it: a borderless one centred, a framed
/// one left where it is unless it must move to stay on the usable area. The setting itself never changes: a bigger
/// monitor gets the full size back. A value that is no size leaves the size alone; no monitor reported (Wayland can
/// report none) means no cap and no move. Returns a line for the log when the size was capped.
fn size_and_place(window: &Window, want: &Value, show: WindowShow) -> Option<String> {
    let (inner, outer) = (window.inner_size(), window.outer_size());
    let frame = (outer.width.saturating_sub(inner.width), outer.height.saturating_sub(inner.height));
    let area = window.current_monitor().map(|m| (whole_monitor(&m), usable_area(&m)));
    let asked = window_size_of(want);
    let client = match (asked, area) {
        (Some(a), Some((_, usable))) => window_capped(a, frame, (usable.w, usable.h)),
        (Some(a), None) => a,
        (None, _) => (inner.width, inner.height),
    };
    if asked.is_some() {
        let _ = window.request_inner_size(PhysicalSize::new(client.0, client.1));
    }
    if let (Some((monitor, usable)), Ok(at)) = (area, window.outer_position()) {
        let (now, size) = ((at.x, at.y), (client.0 + frame.0, client.1 + frame.1));
        let to = if show == WindowShow::Borderless { window_centred(monitor, usable, size) } else { window_kept_in(usable, now, size) };
        if to != now {
            window.set_outer_position(PhysicalPosition::new(to.0, to.1));
        }
    }
    asked.filter(|a| *a != client).map(|a| format!("Window capped to {} x {} px: {} x {} px does not fit the usable screen area.", client.0, client.1, a.0, a.1))
}

/// The settings' text size.
fn text_scale_of(v: &Value) -> f32 {
    v["text_scale"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0) as f32
}

struct App<G> {
    /// The context and the first screens, until the window exists.
    start: Option<(Cx<G>, Vec<Box<dyn Screen<G>>>)>,
    shell: Option<Shell<G>>,
    options: Options,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    proxy: EventLoopProxy<()>,
    bridge: Option<Receiver<bridge::Request>>,
    /// The settings last applied to the window (None: not yet), and the frame interval the cap asks for (None:
    /// unlimited).
    applied: Option<Value>,
    frame_every: Option<Duration>,
    /// FPSCAP1: when the next frame comes.
    pacer: Pacer,
    /// The movement keys held down, in press order (the last one walks).
    walk_keys: Vec<Key>,
    /// Shift is held (Shift+Tab).
    shift: bool,
    /// Ctrl is held (Ctrl+Enter, the text field's Ctrl keys).
    ctrl: bool,
    /// Alt is held (AltGr reports Ctrl+Alt: the character it types is text, not a Ctrl key).
    alt: bool,
}

impl<G: Context> App<G> {
    fn redraw(&mut self) {
        let started = Instant::now();
        let Some(shell) = self.shell.as_mut() else { return };
        if let Some(rx) = &self.bridge {
            while let Ok(req) = rx.try_recv() {
                let reply = shell.bridge(&req.body);
                let _ = req.reply.send(reply);
            }
        }
        shell.frame();
        if shell.cx.quit {
            shell.leave_all();
            std::process::exit(0);
        }
        // A screen asked for the pointer on one of its widgets (ASKNO2): moved only while the window has the focus;
        // a platform that refuses is ignored. The shell's mouse follows at once so the hover shows.
        if let (Some(at), Some(window)) = (shell.pointer_req.take(), self.window.as_ref()) {
            let v = shell.gfx.view;
            let px = winit::dpi::PhysicalPosition::new(v.ox as f64 + (at.0 as f64 + 0.5) * v.s as f64, v.oy as f64 + (at.1 as f64 + 0.5) * v.s as f64);
            if window.has_focus() && window.set_cursor_position(px).is_ok() {
                shell.mouse = at;
            }
        }
        self.apply_settings();
        let Some(shell) = self.shell.as_mut() else { return };
        if let Some(p) = self.pixels.as_mut() {
            if p.frame().len() == shell.gfx.canvas.px.len() {
                p.frame_mut().copy_from_slice(&shell.gfx.canvas.px);
                let _ = p.render();
            }
        }
        self.pacer.framed(started, shell.frame_need());
    }

    /// Applies the engine's settings (DisplaySettings) when they changed since the last time: the window's show
    /// (DISPLAY2: borderless fullscreen over the monitor, a frameless centred window, or a framed one, the last two at the
    /// chosen size capped to the usable area), vsync, the frame-rate cap and the text size. The window is sized only when
    /// its show or the size setting changed, so a window the player resized keeps its size.
    fn apply_settings(&mut self) {
        let (Some(shell), Some(window)) = (self.shell.as_mut(), self.window.as_ref()) else { return };
        let v = shell.cx.settings.v.clone();
        if self.applied.as_ref() == Some(&v) {
            return;
        }
        let show = WindowShow::of(&v);
        if self.applied.as_ref().is_none_or(|a| WindowShow::of(a) != show || a["window_size"] != v["window_size"]) {
            if show == WindowShow::Fullscreen {
                if window.fullscreen().is_none() {
                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
            } else {
                window.set_fullscreen(None);
                window.set_decorations(show == WindowShow::Windowed);
                if let Some(note) = size_and_place(window, &v["window_size"], show) {
                    shell.cx.app.note(note);
                }
            }
        }
        if let Some(p) = self.pixels.as_mut() {
            p.enable_vsync(v["vsync"].as_bool().unwrap_or(true));
        }
        self.frame_every = v["fps_cap"].as_str().and_then(|c| c.parse::<u64>().ok()).filter(|n| *n > 0).map(|n| Duration::from_micros(1_000_000 / n));
        shell.gfx.fonts.scale = text_scale_of(&v);
        self.applied = Some(v);
    }
}

/// The renderer on the window: pixels' synchronous init (the desktop).
#[cfg(not(target_arch = "wasm32"))]
fn open_pixels(pw: u32, ph: u32, window: Arc<Window>) -> Result<Pixels<'static>, pixels::Error> {
    Pixels::new(pw, ph, pixels::SurfaceTexture::new(pw, ph, window))
}

/// The web (WEBREL1-E1a, compile only): pixels has only `new_async` there, which cannot run inside `resumed`. The web
/// start path (`spawn_app`, the async init handed back as a user event) is PR4-5 `WEBREL1-E2`; until then this errors.
#[cfg(target_arch = "wasm32")]
fn open_pixels(_pw: u32, _ph: u32, _window: Arc<Window>) -> Result<Pixels<'static>, pixels::Error> {
    Err(pixels::Error::UserDefined("the web renderer starts in WEBREL1-E2 (PR4-5)".into()))
}

impl<G: Context> ApplicationHandler for App<G> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (cx, stack) = self.start.take().expect("the shell starts once");
        // The setting `start_inactive` (a scenario run's scratch profile): the window opens without taking the focus.
        let active = cx.settings.v["start_inactive"] != json!(true);
        let attrs = Window::default_attributes().with_title(self.options.title.as_str()).with_inner_size(PhysicalSize::new(1280, 720)).with_min_inner_size(PhysicalSize::new(640, 360)).with_active(active);
        let (attrs, icon_notice) = match crate::icon::window_icons(cx.art.layers(), &self.options.icon) {
            Ok(Some(icons)) => {
                #[cfg(windows)]
                let attrs = winit::platform::windows::WindowAttributesExtWindows::with_taskbar_icon(attrs, Some(icons.taskbar));
                (attrs.with_window_icon(Some(icons.window)), None)
            }
            Ok(None) => (attrs, None),
            Err(notice) => (attrs, Some(notice)),
        };
        let window = Arc::new(el.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let (pw, ph) = (size.width.max(1), size.height.max(1));
        let pixels = open_pixels(pw, ph, window.clone()).expect("pixels");
        let mut shell = Shell::open(cx, stack, &self.options, (pw, ph), window.scale_factor() as f32, icon_notice);
        let port = self.options.port;
        let proxy = self.proxy.clone();
        self.bridge = bridge::start(port, move || {
            let _ = proxy.send_event(());
        });
        if self.bridge.is_none() {
            shell.cx.notice.push(format!("Agent link: port {port} is busy."));
        }
        self.shell = Some(shell);
        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    /// The agent link woke the loop: its requests are answered in a redraw, at once, whatever the cap says.
    fn user_event(&mut self, _el: &ActiveEventLoop, _e: ()) {
        self.pacer.touch(Instant::now());
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// FPSCAP1: asked after every wake. A flood of events only asks again: the cap's deadline counts from the last frame.
    /// Unlimited draws as fast as presenting allows (vsync waits for the screen); the redraw itself wakes the loop.
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        match self.pacer.decide(Instant::now(), self.frame_every) {
            Pace::Redraw => {
                el.set_control_flow(ControlFlow::Wait);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            Pace::WaitUntil(at) => el.set_control_flow(ControlFlow::WaitUntil(at)),
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        if !matches!(ev, WindowEvent::RedrawRequested) {
            self.pacer.touch(Instant::now());
        }
        match ev {
            WindowEvent::CloseRequested => {
                // Taken out first: no event after the close reaches a shell whose screens have left.
                if let Some(mut shell) = self.shell.take() {
                    shell.leave_all();
                }
                el.exit()
            }
            WindowEvent::Resized(size) => {
                let (w, h) = (size.width.max(1), size.height.max(1));
                if let Some(p) = self.pixels.as_mut() {
                    let _ = p.resize_surface(w, h);
                    let _ = p.resize_buffer(w, h);
                }
                if let Some(s) = self.shell.as_mut() {
                    s.resize_canvas(w, h);
                }
                // The new surface shows nothing until a frame: at once, also inside the system's own resize loop.
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, mut inner_size_writer } => {
                if let Some(s) = self.shell.as_mut() {
                    s.gfx.kit.device.dpr = scale_factor as f32;
                }
                // Sizes are real pixels: a window moved to a monitor of another scale keeps its pixels.
                if let Some(w) = self.window.as_ref().filter(|w| w.fullscreen().is_none()) {
                    let _ = inner_size_writer.request_inner_size(w.inner_size());
                }
            }
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::ModifiersChanged(m) => {
                self.shift = m.state().shift_key();
                self.ctrl = m.state().control_key();
                self.alt = m.state().alt_key();
                if let Some(s) = self.shell.as_mut() {
                    s.shift = self.shift;
                    s.cx.held = Held { shift: self.shift, ctrl: self.ctrl, alt: self.alt, dir: s.cx.held.dir };
                }
            }
            WindowEvent::Focused(false) => {
                // The release may never arrive (alt-tab mid-drag): drop the drag; no key counts as held.
                if let Some(s) = self.shell.as_mut() {
                    s.cancel_drag();
                    s.cx.held = Held::default();
                }
                self.walk_keys.clear();
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Released, .. }, .. } => {
                if let Some(k) = walk_key(&logical_key) {
                    self.walk_keys.retain(|w| *w != k);
                    if let Some(s) = self.shell.as_mut() {
                        s.cx.held.dir = self.walk_keys.last().and_then(|k| k.dir());
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    s.pointer_moved(position.x as f32, position.y as f32);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    let lines = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y.round() as i32,
                        MouseScrollDelta::PixelDelta(p) => (p.y / 20.0).round() as i32,
                    };
                    s.wheel(lines);
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                let Some(s) = self.shell.as_mut() else { return };
                match button {
                    MouseButton::Left => s.click(),
                    MouseButton::Right => s.back(),
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    s.release();
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, physical_key, text, state: ElementState::Pressed, repeat, .. }, .. } => {
                let Some(s) = self.shell.as_mut() else { return };
                if s.gfx.kit.edit.is_none() {
                    if let Some(k) = walk_key(&logical_key) {
                        if !self.walk_keys.contains(&k) {
                            self.walk_keys.push(k);
                        }
                        s.cx.held.dir = self.walk_keys.last().and_then(|k| k.dir());
                        if repeat && s.stack.last().is_some_and(|t| t.hold_walk()) {
                            return;
                        }
                    }
                }
                if s.gfx.kit.edit.is_some() {
                    // A text field types: the editing keys and the typed text go to it, not to the screen. Ctrl+Enter
                    // still submits; Enter (one line), Esc and Tab end the edit and reach the screen.
                    let multiline = s.gfx.kit.edit.as_ref().is_some_and(|e| e.multiline);
                    let ctrl_key = |c: &str| if self.ctrl && !self.alt { EditKey::parse(&format!("ctrl+{}", c.to_lowercase())).map(|k| k.0) } else { None };
                    let named = match &logical_key {
                        WKey::Named(NamedKey::Backspace) => Some(EditKey::Backspace),
                        WKey::Named(NamedKey::Delete) => Some(EditKey::Delete),
                        WKey::Named(NamedKey::ArrowLeft) => Some(EditKey::Left),
                        WKey::Named(NamedKey::ArrowRight) => Some(EditKey::Right),
                        WKey::Named(NamedKey::ArrowUp) => Some(EditKey::Up),
                        WKey::Named(NamedKey::ArrowDown) => Some(EditKey::Down),
                        WKey::Named(NamedKey::Home) => Some(EditKey::Home),
                        WKey::Named(NamedKey::End) => Some(EditKey::End),
                        WKey::Named(NamedKey::PageUp) => Some(EditKey::PageUp),
                        WKey::Named(NamedKey::PageDown) => Some(EditKey::PageDown),
                        // Ctrl+Shift+Z is redo (the letter arrives as itself, Shift aside).
                        WKey::Character(c) if self.ctrl && self.shift && !self.alt && c.eq_ignore_ascii_case("z") => Some(EditKey::Redo),
                        WKey::Character(c) => ctrl_key(c),
                        _ => None,
                    };
                    if let Some(k) = named {
                        s.edit_key(k, self.ctrl, self.shift);
                        return;
                    }
                    match logical_key {
                        WKey::Named(NamedKey::Enter) if self.ctrl => s.key(Key::CtrlEnter),
                        WKey::Named(NamedKey::Enter) if multiline => s.edit(|e| e.insert("\n")),
                        WKey::Named(named @ (NamedKey::Enter | NamedKey::Escape | NamedKey::Tab)) => {
                            s.gfx.kit.edit = None;
                            s.key(match named {
                                NamedKey::Enter => Key::Enter,
                                NamedKey::Escape => Key::Esc,
                                _ if self.shift => Key::BackTab,
                                _ => Key::Tab,
                            });
                        }
                        WKey::Named(NamedKey::F8) => {}
                        // Ctrl+letter types a control character (dropped by the field); AltGr reports Ctrl+Alt and
                        // types the character its layout gives.
                        _ => {
                            if let Some(t) = text {
                                s.edit(|e| e.insert(&t));
                            }
                        }
                    }
                    return;
                }
                // Shift with a digit key reads by its place on the keyboard (the layout types "!" or "1" for it)
                let shift_digit = match physical_key {
                    PhysicalKey::Code(code) if self.shift && !self.ctrl && !self.alt => shift_digit(code),
                    _ => None,
                };
                let k = match logical_key {
                    _ if shift_digit.is_some() => shift_digit.map(Key::ShiftDigit),
                    WKey::Named(NamedKey::ArrowUp) => Some(Key::Up),
                    WKey::Named(NamedKey::ArrowDown) => Some(Key::Down),
                    WKey::Named(NamedKey::ArrowLeft) => Some(Key::Left),
                    WKey::Named(NamedKey::ArrowRight) => Some(Key::Right),
                    WKey::Named(NamedKey::Enter) if self.ctrl => Some(Key::CtrlEnter),
                    WKey::Named(NamedKey::F5 | NamedKey::F6 | NamedKey::F9 | NamedKey::F10 | NamedKey::F11) if repeat => None,
                    WKey::Named(NamedKey::F5) => Some(Key::Fn(5)),
                    WKey::Named(NamedKey::F6) => Some(Key::Fn(6)),
                    WKey::Named(NamedKey::F7) => Some(Key::Fn(7)),
                    WKey::Named(NamedKey::F9) => Some(Key::Fn(9)),
                    WKey::Named(NamedKey::F10) => Some(Key::Fn(10)),
                    WKey::Named(NamedKey::F11) => Some(Key::Fn(11)),
                    WKey::Named(NamedKey::F8) => {
                        s.note();
                        None
                    }
                    WKey::Named(NamedKey::Enter) => Some(Key::Enter),
                    WKey::Named(NamedKey::Space) => Some(Key::Space),
                    WKey::Named(NamedKey::Escape) => Some(Key::Esc),
                    WKey::Named(NamedKey::Tab) => Some(if self.shift { Key::BackTab } else { Key::Tab }),
                    WKey::Named(NamedKey::PageUp) => Some(Key::PageUp),
                    WKey::Named(NamedKey::PageDown) => Some(Key::PageDown),
                    WKey::Named(NamedKey::F12) => {
                        s.f12_shot();
                        None
                    }
                    WKey::Character(c) => c.chars().next().map(|ch| Key::Char(ch.to_ascii_lowercase())),
                    _ => None,
                };
                if let Some(k) = k {
                    s.key(k);
                }
            }
            _ => {}
        }
    }
}

/// The movement key (arrows, W A S D) a logical key is, if it is one.
fn walk_key(k: &WKey) -> Option<Key> {
    let key = match k {
        WKey::Named(NamedKey::ArrowUp) => Key::Up,
        WKey::Named(NamedKey::ArrowDown) => Key::Down,
        WKey::Named(NamedKey::ArrowLeft) => Key::Left,
        WKey::Named(NamedKey::ArrowRight) => Key::Right,
        WKey::Character(c) => Key::Char(c.chars().next()?.to_ascii_lowercase()),
        _ => return None,
    };
    key.dir().map(|_| key)
}

/// The digit of a top-row or keypad digit key.
fn shift_digit(code: KeyCode) -> Option<u8> {
    use KeyCode::*;
    let d = match code {
        Digit0 | Numpad0 => 0,
        Digit1 | Numpad1 => 1,
        Digit2 | Numpad2 => 2,
        Digit3 | Numpad3 => 3,
        Digit4 | Numpad4 => 4,
        Digit5 | Numpad5 => 5,
        Digit6 | Numpad6 => 6,
        Digit7 | Numpad7 => 7,
        Digit8 | Numpad8 => 8,
        Digit9 | Numpad9 => 9,
        _ => return None,
    };
    Some(d)
}

/// The keyboard focus's ring around a widget's rect.
fn draw_ring(p: &mut Painter, r: Rect) {
    let t = th();
    let m = Metrics::of(&t, p.v);
    let (out, wide, radius) = (m.u(2), m.u(4), m.radius + m.uf(1.0));
    p.panel(Rect::new(r.x - out, r.y - out, r.w + wide, r.h + wide), rgba(0, 0, 0, 0), t.ring, radius);
}

/// A valid drop target's outline while dragging, the one under the cursor lit in gold.
fn draw_drop_target(p: &mut Painter, r: Rect, over: bool) {
    let t = th();
    p.panel(r, if over { t.drop_hot } else { rgba(0, 0, 0, 0) }, if over { t.focus } else { t.edge_hot }, Metrics::of(&t, p.v).radius);
}

/// The dragged widget's text in a small card under the cursor.
fn draw_ghost(p: &mut Painter, text: &str, mouse: (i32, i32)) {
    let t = th();
    let (v, m) = (p.v, Metrics::of(&t, p.v));
    let w = (p.text_w(text, m.body).ceil() as i32 + m.u(10)).clamp(m.u(24), m.u(160));
    let h = p.line_h(m.body).ceil() as i32 + m.u(4);
    let (cw, ch) = v.canvas();
    let r = Rect::new((mouse.0 + m.u(6)).min(cw - w - m.u(2)), (mouse.1 + m.u(6)).min(ch - h - m.u(2)), w, h);
    p.panel(r, t.ghost, t.focus, m.radius);
    p.text_fit(r.x as f32 + m.uf(5.0), r.y as f32 + m.uf(2.0), text, m.body, (w - m.u(10)) as f32, t.text);
}

/// An open select's choice list, over the screen: one row per option, the chosen one in gold, the hovered one lit.
/// The open choice list; the option under the mouse or the keyboard's (`keyed`) is highlighted, the keyboard's ringed.
fn draw_options(p: &mut Painter, opts: &[&Value], mouse: (i32, i32), keyed: Option<&str>) {
    let rects: Vec<Rect> = opts.iter().map(|w| info_rect(w)).collect();
    let (x0, y0) = (rects.iter().map(|r| r.x).min().unwrap_or(0), rects.iter().map(|r| r.y).min().unwrap_or(0));
    let (x1, y1) = (rects.iter().map(|r| r.x + r.w).max().unwrap_or(0), rects.iter().map(|r| r.y + r.h).max().unwrap_or(0));
    let t = th();
    let m = Metrics::of(&t, p.v);
    p.panel(Rect::new(x0 - m.u(1), y0 - m.u(1), x1 - x0 + m.u(2), y1 - y0 + m.u(2)), t.panel, t.focus, m.radius);
    for (w, r) in opts.iter().zip(&rects) {
        let on_key = keyed.is_some_and(|k| w["id"] == k);
        if r.contains(mouse.0, mouse.1) || on_key {
            p.panel(*r, t.card_hot, if on_key { t.ring } else { t.edge_hot }, m.radius);
        }
        let chosen = w["active"] == json!(true);
        p.text_fit(r.x as f32 + m.uf(5.0), r.y as f32 + (r.h as f32 - p.line_h(m.body)) / 2.0, w["text"].as_str().unwrap_or(""), m.body, r.w as f32 - m.uf(10.0), if chosen { t.focus } else { t.text });
    }
}

/// The smallest canvas a shot is taken of: below this it is a minimized window's canvas of nothing, not a picture.
pub const SHOT_SIDE_MIN: i32 = 64;
/// How far from the first pixel, on any channel, a pixel of a blank frame may be.
pub const BLANK_TOLERANCE: u8 = 3;

/// Why this canvas is no picture of anything, if it is not: too small, or one colour all over (nothing drawn, or a
/// cleared frame).
pub fn blank_reason(c: &Canvas) -> Option<String> {
    if c.w < SHOT_SIDE_MIN || c.h < SHOT_SIDE_MIN {
        return Some(format!("the canvas is {} x {} px: a window that is minimized, or no frame", c.w, c.h));
    }
    let first = c.px.first_chunk::<4>()?;
    let same = c.px.chunks_exact(4).all(|p| p.iter().zip(first).all(|(a, b)| a.abs_diff(*b) <= BLANK_TOLERANCE));
    same.then(|| format!("the frame is blank: all {} x {} px are one colour, {first:?}", c.w, c.h))
}

/// Saves the canvas as PNG; `max_w` > 0 shrinks it (box filter) to at most that width. A blank canvas (`blank_reason`) is
/// refused and nothing is written, so a shot on disk always shows something; an older shot at `path` goes, which this one
/// would have replaced, so it cannot pass for this one.
pub fn save_png(c: &Canvas, path: &Path, max_w: i32) -> Result<()> {
    if let Some(why) = blank_reason(c) {
        let _ = std::fs::remove_file(path);
        anyhow::bail!("no shot: {why}");
    }
    let mut img = Image { w: c.w, h: c.h, px: c.px.clone() };
    if max_w > 0 && c.w > max_w {
        img = img.downsample(max_w, c.h * max_w / c.w);
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&img.px)?;
    Ok(())
}

/// Opens the window (or, with `Options::headless`, none) and runs the screens until the window closes or a screen sets
/// `cx.quit`. The theme is loaded from the game's asset layers first (LOOK1-E1; its problems show as a notice over the
/// first screen).
pub fn run<G: Context>(mut cx: Cx<G>, stack: Vec<Box<dyn Screen<G>>>, options: Options) -> Result<()> {
    crate::theme::reload(&mut cx);
    if let Some(h) = options.headless {
        return headless::run(cx, stack, &options, h);
    }
    let el: EventLoop<()> = EventLoop::with_user_event().build()?;
    let proxy = el.create_proxy();
    let mut app = App { start: Some((cx, stack)), shell: None, options, window: None, pixels: None, proxy, bridge: None, applied: None, frame_every: Some(Duration::from_millis(16)), pacer: Pacer::default(), walk_keys: Vec::new(), shift: false, ctrl: false, alt: false };
    el.run_app(&mut app)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::Art;
    use crate::canvas::rgb;
    use crate::ui::{BODY, H, TEXT, W};
    use crate::audio::Audio;
    use crate::screen::Settings;
    use crate::ui::{Tip, Ui, Widget};

    #[derive(Default)]
    struct App {
        log: Vec<String>,
    }

    impl Context for App {
        const MORE_COMMANDS: &'static str = "none";
        fn note(&mut self, _line: String) {}
        fn agent_summary(&self) -> Value {
            Value::Null
        }
        fn agent_game(&self) -> Value {
            Value::Null
        }
    }

    type Cx = crate::screen::Cx<App>;

    /// An item that can be dragged (and clicked), a slot that takes it and a locked slot that refuses it.
    #[derive(Default)]
    struct Board {
        w: Vec<Widget<&'static str>>,
    }

    impl Board {
        fn act(&mut self, cx: &mut Cx, a: &'static str) {
            cx.app.log.push(format!("click {a}"));
        }
    }

    impl Screen<App> for Board {
        fn id(&self) -> &'static str {
            "board"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            let no_tip: Tip = Vec::new();
            ui.row("item", Rect::new(10, 10, 60, 14), "Sword", TEXT, Some("item"), false, no_tip.clone());
            ui.draggable("item.sword");
            ui.row("slot", Rect::new(100, 10, 60, 14), "Weapon", TEXT, None, false, no_tip.clone());
            ui.drop_target("slot.weapon");
            ui.row("locked", Rect::new(100, 40, 60, 14), "Locked", TEXT, None, false, no_tip);
            ui.drop_target("slot.locked");
            self.w = ui.widgets;
        }
        fn can_drop(&self, _cx: &Cx, _from: &str, to: &str) -> bool {
            to != "slot.locked"
        }
        fn accept_drop(&mut self, cx: &mut Cx, from: &str, to: &str) {
            cx.app.log.push(format!("drop {from} on {to}"));
        }
        crate::widget_plumbing!(Cx);
    }

    fn shell() -> Shell<App> {
        let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        let cx = Cx {
            settings: Settings::load(std::env::temp_dir().join("aegis_no_settings.json"), json!({})),
            art: Art::new(None),
            audio: Audio::offline(None),
            nav: Vec::new(),
            notice: Vec::new(),
            shot_dir: std::env::temp_dir(),
            quit: false,
            answers: Vec::new(),
            playtest: crate::playtest::Playtests::off(),
            held: Default::default(),
            app: App::default(),
        };
        let gfx = Gfx { canvas: Canvas::new(W, H), view: View::fit(W as u32, H as u32), fonts: Fonts::new(&font), kit: Kit::default() };
        let mut s = Shell { frames: 0, policy: CanvasPolicy::default(), cx, gfx, stack: vec![Box::new(Board::default())], mouse: (0, 0), mouse_px: None, text_rows: false, drag: None, press: None, win_drag: None, open_focus: None, hover: None, shift: false, text_press: false, text_click: None, born: Instant::now(), pointer_pending: false, pointer_req: None, agent: false };
        s.frame();
        s
    }

    /// Press at `from`, move to `to` in steps, release (the window's mouse path).
    fn drag(s: &mut Shell<App>, from: (i32, i32), to: (i32, i32)) {
        s.mouse = from;
        s.click();
        for t in 1..=4 {
            s.mouse = (from.0 + (to.0 - from.0) * t / 4, from.1 + (to.1 - from.1) * t / 4);
            s.drag_to();
            s.frame();
        }
        s.release();
    }

    /// Two buttons, a choice field and a slider (PORT-FOCUS); `steer` says the screen walks with the arrows.
    #[derive(Default)]
    struct Form {
        w: Vec<Widget<(&'static str, usize)>>,
        pick: usize,
        vol: usize,
        steer: bool,
        keyboard: bool,
    }

    impl Form {
        fn act(&mut self, cx: &mut Cx, a: (&'static str, usize)) {
            match a.0 {
                "pick" => self.pick = a.1,
                "vol" => self.vol = a.1,
                _ => {}
            }
            cx.app.log.push(format!("{} {}", a.0, a.1));
        }
    }

    impl Screen<App> for Form {
        fn id(&self) -> &'static str {
            "form"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui: Ui<(&'static str, usize)> = Ui::new(g, mouse, live);
            ui.button("ok", Rect::new(10, 10, 40, 14), "OK", ("ok", 0), true);
            ui.button("no", Rect::new(60, 10, 40, 14), "No", ("no", 0), true);
            ui.button("off", Rect::new(110, 10, 40, 14), "Off", ("off", 0), false);
            let opts: Vec<String> = ["one", "two", "three"].iter().map(|s| s.to_string()).collect();
            ui.select("pick", Rect::new(10, 40, 60, 14), &opts, self.pick, |i| ("pick", i), true, Vec::new());
            ui.slider("vol", Rect::new(10, 80, 140, 14), self.vol, 5, "vol", |i| ("vol", i), true);
            self.w = ui.widgets;
        }
        fn key(&mut self, cx: &mut Cx, k: Key) {
            cx.app.log.push(format!("key {k:?}"));
        }
        fn arrow_focus(&self) -> bool {
            !self.steer
        }
        fn auto_focus(&self) -> bool {
            self.keyboard
        }
        crate::widget_plumbing!(Cx);
    }

    fn form(f: Form) -> Shell<App> {
        let mut s = shell();
        s.stack = vec![Box::new(f)];
        s.frame();
        s
    }

    fn press(s: &mut Shell<App>, k: Key) {
        s.key(k);
        s.frame();
    }

    /// A screen with one label that fits at Normal text size and is cut at Extra large (TEXTFIT1 audit).
    #[derive(Default)]
    struct Cramped {
        w: Vec<Widget<&'static str>>,
    }

    impl Cramped {
        fn act(&mut self, _cx: &mut Cx, _a: &'static str) {}
    }

    impl Screen<App> for Cramped {
        fn id(&self) -> &'static str {
            "cramped"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            let label = "Twelve letters";
            let fit_at_normal = ui.p.text_w(label, BODY) / ui.p.fonts.scale + 2.0;
            ui.p.text_fit(10.0, 10.0, label, BODY, fit_at_normal, TEXT);
            self.w = ui.widgets;
        }
        crate::widget_plumbing!(Cx);
    }

    #[test]
    fn the_text_audit_finds_a_label_cut_only_at_a_larger_text_size() {
        let mut s = shell();
        s.stack = vec![Box::new(Cramped::default())];
        s.frame();
        let normal = s.bridge(&json!({"cmd": "text_audit", "scale": 1.0}));
        assert_eq!((normal["ok"].clone(), normal["screen"].clone(), normal["counts"]["cut"].clone()), (json!(true), json!("cramped"), json!(0)), "{normal}");
        let large = s.bridge(&json!({"cmd": "text_audit", "scale": 1.3}));
        assert_eq!(large["counts"]["cut"], json!(1), "{large}");
        assert_eq!(large["cut"][0]["text"], json!("Twelve letters"));
        assert!((s.gfx.fonts.scale - 1.0).abs() < 1e-6 && s.gfx.fonts.audit.is_none(), "the audit leaves the text size and the recorder as they were");
    }

    /// A screen whose only button opens a notice.
    #[derive(Default)]
    struct Opener {
        w: Vec<Widget<&'static str>>,
    }

    impl Opener {
        fn act(&mut self, cx: &mut Cx, _a: &'static str) {
            cx.notice.push("Opened.".into());
        }
    }

    impl Screen<App> for Opener {
        fn id(&self) -> &'static str {
            "opener"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            ui.button("open", Rect::new(10, 10, 40, 14), "Open", "open", true);
            self.w = ui.widgets;
        }
        crate::widget_plumbing!(Cx);
    }

    /// A click that opens an overlay is in the trail before the overlay (so the report reads it as prompted), and a
    /// click that only closes an open choice list presses no button.
    #[test]
    fn a_click_comes_before_what_it_opens() {
        let _queues = crate::playtest::test_queues();
        let dir = aegis_core::scratch::Scratch::vacant("shell_open");
        let mut s = shell();
        s.stack = vec![Box::new(Opener::default())];
        s.frame();
        s.cx.playtest = crate::playtest::Playtests::new(dir.to_path_buf(), true, Value::Null);
        s.mouse = (20, 17);
        s.click();
        s.frame();
        let trail = s.cx.playtest.current().unwrap().trail.clone();
        let types: Vec<&str> = trail.iter().map(|e| e["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["click", "button", "modal"], "{trail:?}");
        let report = aegis_host::trail::build(&trail[..3], 0, &[]);
        assert_eq!((report["counts"]["unprompted_open"].clone(), report["counts"]["slow_open"].clone()), (json!(0), json!(0)), "{report}");

        let mut f = form(Form::default());
        f.cx.playtest = crate::playtest::Playtests::new(dir.to_path_buf(), true, Value::Null);
        f.mouse = (30, 47);
        f.click();
        f.frame();
        f.mouse = (30, 17);
        f.click();
        let trail = f.cx.playtest.current().unwrap().trail.clone();
        let types: Vec<&str> = trail.iter().map(|e| e["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["click", "button", "click"], "the click closing the list presses nothing: {trail:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PORT-PLAYTEST notes: F8 captures the screen and opens the note box with its field typing (keys typed there stay
    /// out of the trail); Ctrl+Enter saves the note (stamp, text, tags, a JPEG next to the recording) and closes it.
    #[test]
    fn f8_takes_a_note_with_a_screenshot() {
        let _queues = crate::playtest::test_queues();
        let dir = aegis_core::scratch::Scratch::vacant("shell_note");
        let mut s = form(Form::default());
        s.note();
        assert!(s.stack.iter().all(|t| t.id() != crate::notes::ID), "recording off: no note box");
        s.cx.notice.clear();
        s.stack.truncate(1);
        s.cx.playtest = crate::playtest::Playtests::new(dir.to_path_buf(), true, Value::Null);
        s.mouse = (30, 17);
        s.note();
        s.frame();
        assert_eq!(s.stack.last().unwrap().id(), crate::notes::ID);
        assert_eq!(s.gfx.kit.edit.as_ref().map(|e| e.id.as_str()), Some(crate::notes::TEXT_ID), "the text field types at once");
        s.edit(|e| e.insert("The OK button is hard to read"));
        s.key(Key::CtrlEnter);
        s.frame();
        assert_ne!(s.stack.last().unwrap().id(), crate::notes::ID, "saved and closed");
        let rec = s.cx.playtest.current().unwrap().clone();
        let note = &rec.notes[0];
        assert_eq!((note["n"].clone(), note["text"].clone(), note["scene"].clone(), note["ctl"]["path"].clone()), (json!(1), json!("The OK button is hard to read"), json!("form"), json!("ok")));
        let image = s.cx.playtest.dir.join(&rec.id).join(note["image"].as_str().unwrap());
        assert_eq!(&std::fs::read(&image).unwrap()[..2], &[0xff, 0xd8], "a JPEG");
        assert!(rec.trail.iter().any(|e| e["type"] == "note" && e["n"] == json!(1)));
        assert!(!rec.trail.iter().any(|e| e["type"] == "key" && e["key"] == "ctrl+enter"), "the save key was typed in the field");
        // The agent link takes notes the same way.
        assert_eq!(s.bridge(&json!({"cmd": "type", "text": "x"}))["ok"], json!(false), "nothing is being edited");
        assert_eq!(s.bridge(&json!({"cmd": "note"}))["ok"], json!(true));
        s.frame();
        assert_eq!(s.bridge(&json!({"cmd": "type", "text": "Agent note"}))["ok"], json!(true));
        assert_eq!(s.bridge(&json!({"cmd": "key", "key": "ctrl+enter"}))["ok"], json!(true));
        s.frame();
        assert_eq!(s.cx.playtest.current().unwrap().notes[1]["text"], json!("Agent note"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A text area and a one-line field whose values the screen keeps (PLAYUI1-T).
    struct Pad {
        w: Vec<Widget<&'static str>>,
        area: String,
        line: String,
    }

    const PAD_AREA: Rect = Rect::new(10, 10, 120, 30);

    impl Pad {
        fn act(&mut self, cx: &mut Cx, a: &'static str) {
            cx.app.log.push(a.to_string());
        }
    }

    impl Screen<App> for Pad {
        fn id(&self) -> &'static str {
            "pad"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            ui.text_field("area", PAD_AREA, &self.area, true, true);
            ui.text_field("line", Rect::new(10, 80, 100, 14), &self.line, false, true);
            self.w = ui.widgets;
        }
        fn text_changed(&mut self, _cx: &mut Cx, id: &str, text: &str) {
            match id {
                "area" => self.area = text.to_string(),
                _ => self.line = text.to_string(),
            }
        }
        fn state(&self, _cx: &Cx) -> Value {
            json!({"area": self.area, "line": self.line})
        }
        crate::widget_plumbing!(Cx);
    }

    const PAD_TEXT: &str = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega";

    fn pad() -> Shell<App> {
        let mut s = shell();
        s.stack = vec![Box::new(Pad { w: Vec::new(), area: PAD_TEXT.into(), line: "one line".into() })];
        s.frame();
        s
    }

    /// The canvas point of caret position `pos` in the field being edited (as drawn, scroll included).
    fn point_of(s: &mut Shell<App>, pos: usize) -> (i32, i32) {
        let (lay, r, line) = s.edit_geometry().unwrap();
        let off = s.gfx.kit.scroll.get("area.scroll").copied().unwrap_or(0);
        let g = s.gfx.view.g as f32;
        let x = r.x as f32 + FIELD_PAD_X * g + lay.x_of(pos);
        let y = r.y as f32 + FIELD_PAD_Y * g + lay.line_of(pos) as f32 * line + line / 2.0 - off as f32;
        (x.round() as i32, y as i32)
    }

    /// A text field on any grid: the Pad screen with its rects in that grid's units.
    struct Fields {
        grid: Grid,
        w: Vec<Widget<&'static str>>,
        area: String,
        line: String,
    }

    impl Fields {
        fn act(&mut self, _cx: &mut Cx, _a: &'static str) {}
    }

    impl Screen<App> for Fields {
        fn id(&self) -> &'static str {
            "fields"
        }
        fn grid(&self) -> Grid {
            self.grid
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let u = self.grid.g();
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            ui.text_field("area", Rect::new(20 * u, 20 * u, 80 * u, 40 * u), &self.area, true, true);
            ui.text_field("line", Rect::new(20 * u, 80 * u, 60 * u, 14 * u), &self.line, false, true);
            self.w = ui.widgets;
        }
        fn text_changed(&mut self, _cx: &mut Cx, id: &str, text: &str) {
            match id {
                "area" => self.area = text.into(),
                "line" => self.line = text.into(),
                _ => {}
            }
        }
        crate::widget_plumbing!(Cx);
    }

    /// Clicks through the physical pointer on a field land the caret where the text is drawn, on every grid and window: the
    /// same character whichever units the field is written in, and the view's scroll and slide are in those units too.
    #[test]
    fn a_click_puts_the_caret_on_the_same_character_on_every_grid() {
        for (pw, ph) in [(1280, 720), (1920, 1080), (1366, 768)] {
            for (field, chars) in [("area", PAD_TEXT.chars().count()), ("line", 40)] {
                let carets: Vec<Vec<usize>> = [Grid::Old, Grid::Native].iter().map(|grid| {
                    let screen = Fields { grid: *grid, w: Vec::new(), area: PAD_TEXT.into(), line: "a line far too long to show in its one line field".into() };
                    let mut s = window(pw, ph, vec![Box::new(screen)]);
                    s.gfx.kit.edit = Some(Edit::new(field, if field == "area" { PAD_TEXT } else { "a line far too long to show in its one line field" }, field == "area"));
                    s.frame();
                    (0..chars).step_by(7).map(|at| {
                        let e = s.gfx.kit.edit.as_mut().unwrap();
                        e.caret = 0;
                        let (x, y) = point_of(&mut s, at);
                        s.caret_at((x, y)).unwrap()
                    }).collect()
                }).collect();
                assert_eq!(carets[0], carets[1], "{pw}x{ph} {field}");
                let hits = carets[1].iter().enumerate().filter(|(i, c)| i * 7 == **c).count();
                assert!(hits * 2 > carets[1].len(), "{pw}x{ph} {field}: the caret lands on the character clicked: {:?}", carets[1]);
            }
        }
    }

    /// The caret's line stays in view, and the page keys step by the rows that show, in the field's own units: a text area
    /// scrolled for the same caret shows the same line on either grid.
    #[test]
    fn a_text_area_scrolls_the_same_lines_on_the_native_grid() {
        let offset = |grid: Grid| {
            let screen = Fields { grid, w: Vec::new(), area: PAD_TEXT.into(), line: String::new() };
            let mut s = window(1920, 1080, vec![Box::new(screen)]);
            s.gfx.kit.edit = Some(Edit::new("area", PAD_TEXT, true));
            s.frame();
            s.gfx.kit.edit.as_mut().unwrap().caret = PAD_TEXT.chars().count();
            s.reveal();
            s.gfx.kit.scroll["area.scroll"]
        };
        let (old, native) = (offset(Grid::Old), offset(Grid::Native));
        assert!(old > 0, "the text is taller than its field");
        assert!((native - 2 * old).abs() <= 2, "{native} against twice {old}");
    }

    /// A screen on the native grid shows a focused widget's tip with no mouse over it; a screen on the old numbers keeps
    /// today's picture.
    #[test]
    fn a_tip_follows_the_keyboard_focus_on_the_native_grid_only() {
        struct Tips {
            grid: Grid,
            w: Vec<Widget<&'static str>>,
        }
        impl Tips {
            fn act(&mut self, _cx: &mut Cx, _a: &'static str) {}
        }
        impl Screen<App> for Tips {
            fn id(&self) -> &'static str {
                "tips"
            }
            fn grid(&self) -> Grid {
                self.grid
            }
            fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
                let u = self.grid.g();
                let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
                ui.button_tip("tipped", Rect::new(100 * u, 100 * u, 60 * u, 16 * u), "Tipped", "a", true, crate::ui::tip("A title", "A body."));
                self.w = ui.widgets;
            }
            crate::widget_plumbing!(Cx);
        }
        let lit = |s: &Shell<App>| s.gfx.canvas.px.clone();
        for (grid, shows) in [(Grid::Native, true), (Grid::Old, false), (Grid::Quadrant, false)] {
            let mut s = window(1280, 720, vec![Box::new(Tips { grid, w: Vec::new() })]);
            s.mouse = (-1, -1);
            s.frame();
            let plain = lit(&s);
            s.gfx.kit.focus = Some("tipped".into());
            s.gfx.kit.focus_visible = true;
            s.frame();
            // The ring is around the button; the tip is what reaches past the ring, below it.
            let v = s.gfx.view;
            let below = v.rect(Rect::new(110 * grid.g(), 122 * grid.g(), 40 * grid.g(), 6 * grid.g()));
            let differs = (below.y..below.y + below.h).any(|y| (below.x..below.x + below.w).any(|x| {
                let i = ((y * s.gfx.canvas.w + x) * 4) as usize;
                s.gfx.canvas.px[i..i + 4] != plain[i..i + 4]
            }));
            assert_eq!(differs, shows, "{grid:?}");
        }
    }

    #[test]
    fn a_click_places_the_caret_and_a_drag_or_shift_click_selects() {
        let mut s = pad();
        s.mouse = (PAD_AREA.x + 2, PAD_AREA.y + 6);
        s.click();
        s.release();
        s.frame();
        assert_eq!(s.gfx.kit.edit.as_ref().map(|e| (e.id.as_str(), e.caret)), Some(("area", 0)), "a click at the left of line 1 puts the caret at the start");
        let (lay, _, _) = s.edit_geometry().unwrap();
        assert!(lay.lines.len() > 3, "the sample wraps to more lines than the area shows");
        let (end1, start2) = (lay.last_pos(0), lay.lines[1].start);
        // A click right of line 1's last word: the end of that line.
        s.mouse = (PAD_AREA.x + PAD_AREA.w - 4, PAD_AREA.y + 6);
        s.click();
        s.release();
        assert_eq!(s.gfx.kit.edit.as_ref().unwrap().caret, end1);
        // Press at the start, drag to line 2's start: the selection runs between them.
        s.mouse = point_of(&mut s, 0);
        s.click();
        s.mouse = point_of(&mut s, start2);
        s.drag_to();
        s.release();
        assert_eq!(s.gfx.kit.edit.as_ref().unwrap().selection(), Some((0, start2)));
        // Shift+click moves the caret and keeps the selection's start.
        s.shift = true;
        s.mouse = point_of(&mut s, end1);
        s.click();
        s.release();
        s.shift = false;
        assert_eq!(s.gfx.kit.edit.as_ref().unwrap().selection(), Some((0, end1)));
        // A click in the other field edits that one.
        s.mouse = (60, 87);
        s.click();
        s.release();
        assert_eq!(s.gfx.kit.edit.as_ref().map(|e| e.id.as_str()), Some("line"));
        s.mouse = (300, 300);
        s.click();
        assert!(s.gfx.kit.edit.is_none(), "a click elsewhere ends the edit");
    }

    #[test]
    fn editing_keys_over_the_agent_link_select_copy_paste_undo_and_move_by_line() {
        let mut s = pad();
        s.mouse = point_of_start();
        s.click();
        s.release();
        s.frame();
        for k in ["ctrl+a", "ctrl+c", "ctrl+end", "ctrl+v"] {
            assert_eq!(s.bridge(&json!({"cmd": "key", "key": k}))["ok"], json!(true), "{k}");
        }
        assert_eq!(clipboard::get(), PAD_TEXT, "copied to the in-app clipboard (unit tests never touch the system's)");
        let state = s.stack.last().unwrap().state(&s.cx);
        assert_eq!(state["area"], json!(format!("{PAD_TEXT}{PAD_TEXT}")), "the paste reached the screen");
        s.bridge(&json!({"cmd": "key", "key": "ctrl+z"}));
        assert_eq!(s.stack.last().unwrap().state(&s.cx)["area"], json!(PAD_TEXT), "undone");
        s.frame();
        let (lay, _, _) = s.edit_geometry().unwrap();
        let last = lay.lines.len() - 1;
        assert_eq!(lay.line_of(s.gfx.kit.edit.as_ref().unwrap().caret), last, "the caret at the end");
        s.bridge(&json!({"cmd": "key", "key": "up"}));
        assert_eq!(lay.line_of(s.gfx.kit.edit.as_ref().unwrap().caret), last - 1, "Up goes one visual line up");
        s.bridge(&json!({"cmd": "key", "key": "shift+home"}));
        let tree = s.bridge(&json!({"cmd": "ui_tree"}));
        assert_eq!(tree["edit"]["id"], json!("area"));
        assert!(tree["edit"]["selected"].as_str().is_some_and(|t| !t.is_empty()), "Shift+Home selected to the line's start: {}", tree["edit"]);
        // Enter adds a line to a text area; Esc ends the edit, as the keyboard does.
        s.bridge(&json!({"cmd": "key", "key": "ctrl+end"}));
        s.bridge(&json!({"cmd": "key", "key": "enter"}));
        assert!(s.stack.last().unwrap().state(&s.cx)["area"].as_str().is_some_and(|t| t.ends_with('\n')));
        s.bridge(&json!({"cmd": "key", "key": "esc"}));
        assert!(s.gfx.kit.edit.is_none(), "Esc ended the edit");
    }

    fn point_of_start() -> (i32, i32) {
        (PAD_AREA.x + 2, PAD_AREA.y + 6)
    }

    #[test]
    fn the_wheel_scrolls_a_text_area_and_a_caret_move_brings_its_line_back() {
        let mut s = pad();
        s.mouse = (PAD_AREA.x + 20, PAD_AREA.y + 20);
        s.wheel(-1);
        let off = s.gfx.kit.scroll.get("area.scroll").copied().unwrap_or(0);
        assert!(off > 0, "the wheel scrolled the text area");
        s.frame();
        // A click at the top of the scrolled view lands on the line drawn there, not line 1.
        s.mouse = (PAD_AREA.x + 2, PAD_AREA.y + 4);
        s.click();
        s.release();
        let (lay, _, _) = s.edit_geometry().unwrap();
        let caret = s.gfx.kit.edit.as_ref().unwrap().caret;
        assert!(lay.line_of(caret) > 0, "the click took the scroll into account");
        s.bridge(&json!({"cmd": "key", "key": "ctrl+home"}));
        assert_eq!(s.gfx.kit.scroll.get("area.scroll").copied(), Some(0), "the caret's line scrolled back into view");
        s.bridge(&json!({"cmd": "key", "key": "ctrl+end"}));
        let max = s.stack.last().unwrap().widgets().iter().find(|w| w["id"] == "area.scroll").and_then(|w| w["text"].as_str().and_then(|t| t.parse::<i32>().ok())).unwrap();
        assert_eq!(s.gfx.kit.scroll.get("area.scroll").copied(), Some(max), "the end shows");
    }

    /// PLAYUI1-T: a note's row in the Playtests browser opens it whole; Copy puts its text on the clipboard.
    #[test]
    fn the_recordings_browser_shows_a_note_whole_and_copies_it() {
        let _queues = crate::playtest::test_queues();
        let dir = aegis_core::scratch::Scratch::vacant("shell_note_view");
        let mut s = shell();
        s.cx.playtest = crate::playtest::Playtests::new(dir.to_path_buf(), true, Value::Null);
        let text = "The OK button is hard to read.\nSecond line: the gold on navy is too dim at Small.";
        s.cx.playtest.add_note(json!({"scene": "form", "text": text, "tags": ["ui"]}), None).unwrap();
        let browser = crate::recordings::Recordings::new(&s.cx, launch, |_| Value::Null, Vec::new());
        s.stack.push(Box::new(browser));
        s.frame();
        let row = s.stack.last().unwrap().widgets().into_iter().find(|w| w["id"] == "recordings.note.0").expect("the note's row");
        assert!(!row["text"].as_str().unwrap().contains('\n'), "the row shows the note on one line");
        assert_eq!(s.bridge(&json!({"cmd": "click", "id": "recordings.note.0"}))["ok"], json!(true));
        s.frame();
        assert_eq!(s.stack.last().unwrap().id(), crate::recordings::NOTE_ID);
        assert_eq!(s.stack.last().unwrap().state(&s.cx)["text"], json!(text), "the whole text, line breaks kept");
        assert_eq!(s.bridge(&json!({"cmd": "click", "id": "recordings.note_view.copy"}))["ok"], json!(true));
        assert_eq!(clipboard::get(), text);
        press(&mut s, Key::Esc);
        assert_eq!(s.stack.last().unwrap().id(), crate::recordings::ID, "Esc goes back to the browser");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PLAYUI1-T, TEXTFIT1: the taller note box (its text area at least 160 tall at Normal) and the note view read
    /// clean at every text size.
    #[test]
    fn the_note_box_and_the_note_view_read_clean_at_every_text_size() {
        let _queues = crate::playtest::test_queues();
        let dir = aegis_core::scratch::Scratch::vacant("shell_note_sizes");
        let mut s = form(Form::default());
        s.cx.playtest = crate::playtest::Playtests::new(dir.to_path_buf(), true, Value::Null);
        s.note();
        s.frame();
        s.edit(|e| e.insert("The OK button is hard to read at Small text size; the gold on navy loses contrast.\nSecond paragraph."));
        s.frame();
        let area = s.stack.last().unwrap().widgets().into_iter().find(|w| w["id"] == crate::notes::TEXT_ID).map(|w| info_rect(&w)).unwrap();
        assert!(area.h >= 160, "the note's text area is {} tall", area.h);
        let clean = |s: &mut Shell<App>| {
            for (name, scale) in crate::screen::TEXT_SIZES {
                let audit = s.text_audit(Some(scale as f32), None);
                assert_eq!(audit["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "{} at {name}: {audit}", audit["screen"]);
            }
        };
        clean(&mut s);
        s.key(Key::CtrlEnter);
        s.frame();
        let browser = crate::recordings::Recordings::new(&s.cx, launch, |_| Value::Null, Vec::new());
        s.stack.push(Box::new(browser));
        s.frame();
        s.bridge(&json!({"cmd": "click", "id": "recordings.note.0"}));
        s.frame();
        assert_eq!(s.stack.last().unwrap().id(), crate::recordings::NOTE_ID);
        clean(&mut s);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// What the recordings browser handed the game's launcher: (requests, take-over).
    static LAUNCHED: std::sync::Mutex<Vec<(usize, Option<(Value, usize)>)>> = std::sync::Mutex::new(Vec::new());

    fn launch(_cx: &mut Cx, session: &aegis_host::playtest::Session, take: Option<(Value, usize)>) -> Option<Box<dyn Screen<App>>> {
        LAUNCHED.lock().unwrap().push((session.requests.len(), take));
        Some(Box::new(Opener::default()))
    }

    /// PORT-PLAYTEST playback: the browser lists the run's recording, takes over at a picked round (the parent named),
    /// plays back to the end, and copies the recording to the repo inbox, where it lists with the repo badge.
    #[test]
    fn the_recordings_browser_plays_back_takes_over_and_copies() {
        let _queues = crate::playtest::test_queues();
        let dir = aegis_core::scratch::Scratch::vacant("shell_recordings");
        let mut s = shell();
        s.cx.playtest = crate::playtest::Playtests::new(dir.to_path_buf(), true, json!({"total": "a"}));
        let session = aegis_host::playtest::Session {
            requests: vec![json!({"cmd": "x"}); 6],
            checkpoints: vec![json!({"at": 3, "state": {"round": 1, "over": false}}), json!({"at": 6, "state": {"round": 2, "over": true}})],
            end: json!({"result": "won", "at": 6}),
        };
        s.cx.playtest.put_session(&mut None, session);
        let id = s.cx.playtest.current().unwrap().id.clone();
        let inbox = dir.join("inbox");
        let browser = crate::recordings::Recordings::new(&s.cx, launch, |_| Value::Null, vec![inbox.clone()]);
        s.stack.push(Box::new(browser));
        s.frame();
        LAUNCHED.lock().unwrap().clear();
        for id in ["recordings.round", "recordings.round.opt.1", "recordings.take_over"] {
            assert_eq!(s.bridge(&json!({"cmd": "click", "id": id}))["ok"], json!(true), "{id}");
            s.frame();
        }
        assert_eq!(s.stack.last().unwrap().id(), "opener", "the launched screen is on top");
        s.stack.pop();
        s.frame();
        assert_eq!(s.bridge(&json!({"cmd": "click", "id": "recordings.play"}))["ok"], json!(true));
        s.frame();
        assert_eq!(*LAUNCHED.lock().unwrap(), vec![(6, Some((json!({"recording": id, "battle": 0, "at": 3}), 3))), (6, None)]);
        s.stack.pop();
        s.frame();
        assert_eq!(s.bridge(&json!({"cmd": "click", "id": "recordings.copy"}))["ok"], json!(true));
        s.frame();
        assert!(inbox.join(&id).join(format!("{id}.json")).is_file());
        let state = s.stack.last().unwrap().state(&s.cx);
        assert_eq!(state["recordings"].as_array().map(Vec::len), Some(2), "the user copy and the repo copy: {state}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PORT-PLAYTEST trail: with recording on, the shell logs clicks (the control and whether anything took it), the
    /// widget a click pressed, keys and a notice opening as a modal; leaving writes the recording.
    #[test]
    fn the_shell_feeds_the_playtest_trail() {
        let _queues = crate::playtest::test_queues();
        let dir = aegis_core::scratch::Scratch::vacant("shell_trail");
        let mut s = form(Form::default());
        s.cx.playtest = crate::playtest::Playtests::new(dir.to_path_buf(), true, Value::Null);
        s.mouse = (30, 17);
        s.click();
        s.frame();
        s.mouse = (500, 500);
        s.click();
        press(&mut s, Key::Tab);
        s.cx.notice.push("Saved.".into());
        s.frame();
        let trail = s.cx.playtest.current().unwrap().trail.clone();
        let types: Vec<&str> = trail.iter().map(|e| e["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["click", "button", "click", "key", "modal"], "{trail:?}");
        assert_eq!((trail[0]["ctl"]["path"].clone(), trail[0]["handled"].clone()), (json!("ok"), json!(true)));
        assert_eq!((trail[2]["ctl"].clone(), trail[2]["handled"].clone()), (Value::Null, json!(false)), "a click on nothing");
        assert_eq!(trail[3]["key"], json!("tab"));
        assert_eq!(trail[4]["ctl"]["path"], json!("notice"));
        // The notice opened in the same millisecond would count as a reaction: the report reads the clicks and key.
        let report = aegis_host::trail::build(&trail[..4], 0, &[]);
        assert_eq!(report["counts"]["dead_click"], json!(1), "{report}");
        s.leave_all();
        let id = s.cx.playtest.current().unwrap().id.clone();
        assert_eq!(aegis_host::playtest::Recording::load(&s.cx.playtest.path_of(&id)).unwrap().trail.len(), 5);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tab_moves_the_focus_and_enter_presses_it() {
        let mut s = form(Form::default());
        assert_eq!(s.gfx.kit.focus, None, "no focus until a key moves it");
        press(&mut s, Key::Tab);
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (Some("ok"), true));
        press(&mut s, Key::Tab);
        press(&mut s, Key::Tab);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("pick"), "the disabled button is skipped");
        press(&mut s, Key::BackTab);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("no"));
        press(&mut s, Key::Enter);
        assert_eq!(s.cx.app.log.last().map(String::as_str), Some("no 0"), "Enter presses the focused button");
        assert!(!s.cx.app.log.iter().any(|l| l.starts_with("key")), "the screen got none of these keys");
        let tree = s.bridge(&json!({"cmd": "ui_tree"}));
        assert_eq!(tree["focus"], json!("no"));
        // A click hides the ring and focuses what it hit; Enter then goes to the screen.
        s.mouse = (30, 17);
        s.click();
        s.frame();
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (Some("ok"), false));
        press(&mut s, Key::Enter);
        assert_eq!(s.cx.app.log.last().map(String::as_str), Some("key Enter"));
    }

    #[test]
    fn arrows_move_the_focus_unless_the_screen_steers() {
        let mut s = form(Form::default());
        press(&mut s, Key::Down);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("ok"), "the first arrow shows the focus");
        press(&mut s, Key::Right);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("no"));
        press(&mut s, Key::Down);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("pick"));
        let mut s = form(Form { steer: true, ..Form::default() });
        press(&mut s, Key::Down);
        assert_eq!(s.gfx.kit.focus, None);
        assert_eq!(s.cx.app.log.last().map(String::as_str), Some("key Down"), "a steering screen keeps its arrows");
        press(&mut s, Key::Tab);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("ok"), "Tab still moves the focus");
    }

    #[test]
    fn a_choice_field_and_a_slider_work_from_the_keyboard() {
        let mut s = form(Form::default());
        for _ in 0..3 {
            press(&mut s, Key::Tab);
        }
        press(&mut s, Key::Enter);
        assert_eq!(s.gfx.kit.open.as_deref(), Some("pick"), "Enter opens the list");
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("pick.opt.0"), "on the chosen option");
        press(&mut s, Key::Down);
        press(&mut s, Key::Down);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("pick.opt.2"));
        press(&mut s, Key::Enter);
        assert_eq!(s.cx.app.log.last().map(String::as_str), Some("pick 2"));
        assert_eq!((s.gfx.kit.open.as_deref(), s.gfx.kit.focus.as_deref()), (None, Some("pick")), "closed, back on the field");
        press(&mut s, Key::Tab);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("vol"));
        press(&mut s, Key::Right);
        press(&mut s, Key::Right);
        press(&mut s, Key::Left);
        assert_eq!(s.cx.app.log.last().map(String::as_str), Some("vol 1"), "Left / Right step the slider");
    }

    #[test]
    fn a_keyboard_screen_starts_focused_and_another_screen_keeps_its_keys() {
        let mut s = form(Form { keyboard: true, ..Form::default() });
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (Some("ok"), true));
        press(&mut s, Key::Tab);
        s.cx.nav.push(Nav::Push(Box::new(Form::default())));
        s.frame();
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (None, false), "the focus does not carry over");
        press(&mut s, Key::Enter);
        assert_eq!(s.cx.app.log.last().map(String::as_str), Some("key Enter"), "the new screen's Enter is its own");
        s.cx.nav.push(Nav::Pop);
        s.frame();
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (Some("ok"), true), "back on a keyboard screen: focused again");
    }

    /// ASKNO1 through the shell: a destructive question opens with the focus ring on No and Enter answers No (Tab to
    /// Yes, then Enter, answers Yes); the default question opens without a ring and Enter answers Yes.
    #[test]
    fn a_destructive_question_opens_on_no_and_the_default_one_on_nothing() {
        let mut s = form(Form::default());
        s.cx.ask_tag_no("Discard?", "gone");
        s.frame();
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (Some("ask.no"), true), "the ring on No");
        press(&mut s, Key::Enter);
        assert!(!s.cx.take_answer("gone") && s.stack.last().unwrap().id() == "form", "Enter: No, the question closed");
        s.cx.ask_tag_no("Discard?", "gone");
        s.frame();
        press(&mut s, Key::Tab);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("ask.yes"));
        press(&mut s, Key::Enter);
        assert!(s.cx.take_answer("gone"), "Enter on a focused Yes: Yes");
        s.cx.ask_tag_no("Discard?", "gone");
        s.frame();
        press(&mut s, Key::Char('y'));
        assert!(s.cx.take_answer("gone"), "y: Yes");
        s.cx.ask_tag("Discard?", "kept");
        s.frame();
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (None, false), "the default question shows no ring");
        press(&mut s, Key::Enter);
        assert!(s.cx.take_answer("kept") && s.stack.last().unwrap().id() == "form", "Enter: Yes, the question closed");
    }

    #[test]
    fn esc_closes_a_keyboard_list_back_to_its_field_and_a_link_click_hides_the_ring() {
        let mut s = form(Form::default());
        for _ in 0..3 {
            press(&mut s, Key::Tab);
        }
        press(&mut s, Key::Enter);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("pick.opt.0"));
        press(&mut s, Key::Esc);
        assert_eq!((s.gfx.kit.open.as_deref(), s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (None, Some("pick"), true));
        s.bridge(&json!({"cmd": "click", "id": "no"}));
        s.frame();
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (Some("no"), false), "the link's click is a click");
        press(&mut s, Key::Space);
        assert_eq!(s.cx.app.log.last().map(String::as_str), Some("key Space"), "no ring: the screen gets Space");
    }

    #[test]
    fn a_press_without_a_move_is_a_click() {
        let mut s = shell();
        s.mouse = (20, 15);
        s.click();
        assert!(s.cx.app.log.is_empty(), "a draggable widget clicks on release");
        s.mouse = (22, 16);
        s.drag_to();
        s.release();
        assert_eq!(s.cx.app.log, ["click item"]);
    }

    #[test]
    fn a_drag_onto_a_target_that_accepts_drops_it() {
        let mut s = shell();
        drag(&mut s, (20, 15), (120, 15));
        assert_eq!(s.cx.app.log, ["drop item.sword on slot.weapon"]);
        assert!(s.gfx.kit.drag.is_none() && s.press.is_none());
    }

    #[test]
    fn a_refused_target_or_empty_space_cancels() {
        let mut s = shell();
        drag(&mut s, (20, 15), (120, 45));
        drag(&mut s, (20, 15), (300, 200));
        assert!(s.cx.app.log.is_empty());
    }

    #[test]
    fn esc_cancels_a_drag() {
        let mut s = shell();
        s.mouse = (20, 15);
        s.click();
        s.mouse = (60, 15);
        s.drag_to();
        assert!(s.gfx.kit.drag.is_some());
        s.key(Key::Esc);
        s.mouse = (120, 15);
        s.release();
        assert!(s.cx.app.log.is_empty());
    }

    #[test]
    fn right_click_navigation_or_a_new_press_end_a_drag() {
        let mut s = shell();
        let pick_up = |s: &mut Shell<App>| {
            s.mouse = (20, 15);
            s.click();
            s.mouse = (60, 15);
            s.drag_to();
            assert!(s.gfx.kit.drag.is_some());
        };
        pick_up(&mut s);
        s.back();
        assert!(s.gfx.kit.drag.is_none() && s.press.is_none(), "right click cancels");
        pick_up(&mut s);
        s.cx.nav.push(Nav::Pop);
        s.settle();
        assert!(s.gfx.kit.drag.is_none() && s.press.is_none(), "navigation clears the drag");
        pick_up(&mut s);
        s.mouse = (300, 200);
        s.click();
        assert!(s.gfx.kit.drag.is_none() && s.press.is_none(), "a lost release ends at the next press");
        s.mouse = (120, 15);
        s.release();
        assert!(s.cx.app.log.is_empty());
    }

    #[test]
    fn the_agent_link_drags_by_widget_ids_and_sees_the_keys() {
        let mut s = shell();
        let tree = s.bridge(&json!({"cmd": "ui_tree"}));
        let w = |id: &str| tree["widgets"].as_array().unwrap().iter().find(|w| w["id"] == id).cloned().unwrap();
        assert_eq!((w("item")["drag"].clone(), w("slot")["drop"].clone()), (json!("item.sword"), json!("slot.weapon")));
        assert_eq!(s.bridge(&json!({"cmd": "drag", "id": "item", "to": "slot"}))["ok"], json!(true));
        assert_eq!(s.cx.app.log, ["drop item.sword on slot.weapon"]);
        assert_eq!(s.bridge(&json!({"cmd": "drag", "id": "item", "to": "nowhere"}))["ok"], json!(false));
    }

    /// INTEG1-E1: a link reply comes before the frame that shows it, so `ui_tree` counts the frames for a driver to wait on,
    /// and names the process, so a driver knows it reached the game it started.
    #[test]
    fn the_link_tree_counts_frames_and_names_its_process() {
        let mut s = shell();
        let frame = |s: &mut Shell<App>| s.bridge(&json!({"cmd": "ui_tree"}))["frame"].as_u64().expect("a frame count");
        let before = frame(&mut s);
        assert_eq!(frame(&mut s), before, "asking draws nothing");
        s.frame();
        s.frame();
        assert_eq!(frame(&mut s), before + 2);
        let tree = s.bridge(&json!({"cmd": "ui_tree"}));
        assert_eq!(tree["pid"], json!(std::process::id()));
        assert_eq!(tree["px"], json!([s.gfx.canvas.w, s.gfx.canvas.h]));
    }

    /// A screen that says what it needs from the frame loop, over the screen below when it is an overlay.
    struct Rests(FrameNeed, bool);

    impl Screen<App> for Rests {
        fn id(&self) -> &'static str {
            "rests"
        }
        fn overlay(&self) -> bool {
            self.1
        }
        fn frame_need(&self, _cx: &Cx) -> FrameNeed {
            self.0
        }
        fn draw(&mut self, _cx: &mut Cx, _g: &mut Gfx, _mouse: (i32, i32), _live: bool) {}
        fn widgets(&self) -> Vec<Value> {
            Vec::new()
        }
        fn draw_tip(&self, _p: &mut Painter, _mouse: (i32, i32)) {}
        fn click(&mut self, _cx: &mut Cx, _pos: (i32, i32)) -> bool {
            false
        }
        fn click_id(&mut self, _cx: &mut Cx, _id: &str) -> Result<(), String> {
            Ok(())
        }
        fn widget_rect(&self, _id: &str) -> Option<(Rect, Vec<String>)> {
            None
        }
    }

    /// FPSCAP1: the shell's answer to the frame loop.
    #[test]
    fn a_screen_that_has_not_said_keeps_drawing() {
        let s = shell();
        assert_eq!(s.frame_need(), FrameNeed::Full, "the Board says nothing");
    }

    #[test]
    fn the_shell_asks_every_screen_it_draws_and_no_other() {
        let slow = FrameNeed::Within(Duration::from_millis(66));
        let mut s = shell();
        s.stack.push(Box::new(Rests(FrameNeed::Idle, true)));
        assert_eq!(s.frame_need(), FrameNeed::Full, "an idle popup over a screen that keeps drawing");
        s.stack.clear();
        s.stack.push(Box::new(Rests(slow, false)));
        s.stack.push(Box::new(Rests(FrameNeed::Idle, true)));
        assert_eq!(s.frame_need(), slow, "an idle popup over a slow screen");
        s.stack.push(Box::new(Rests(FrameNeed::Idle, false)));
        assert_eq!(s.frame_need(), FrameNeed::Idle, "a full screen hides what is under it");
    }

    #[test]
    fn the_shells_own_work_keeps_an_idle_screen_drawing() {
        let mut s = shell();
        s.stack.clear();
        s.stack.push(Box::new(Rests(FrameNeed::Idle, false)));
        assert_eq!(s.frame_need(), FrameNeed::Idle);
        s.gfx.kit.edit = Some(Edit::new("field", "", false));
        assert_eq!(s.frame_need(), FrameNeed::Full, "a text field's caret");
        s.gfx.kit.edit = None;
        s.cx.held.dir = Some((1, 0));
        assert_eq!(s.frame_need(), FrameNeed::Full, "a held walk key");
        s.cx.held.dir = None;
        s.cx.notice.push("A notice".into());
        assert_eq!(s.frame_need(), FrameNeed::Full, "a notice waiting for the next frame");
        s.cx.notice.clear();
        s.cx.nav.push(Nav::Pop);
        assert_eq!(s.frame_need(), FrameNeed::Full, "a screen change waiting for the next frame");
        s.cx.nav.clear();
        s.win_drag = Some(("win".into(), Grab::Move, (0, 0)));
        assert_eq!(s.frame_need(), FrameNeed::Full, "a window being dragged");
    }

    /// A screen's own commands are answered by it: the shell's names hide them, so its own must stay few.
    struct OwnStep;

    impl Screen<App> for OwnStep {
        fn id(&self) -> &'static str {
            "own_step"
        }
        fn draw(&mut self, _cx: &mut Cx, _g: &mut Gfx, _mouse: (i32, i32), _live: bool) {}
        fn command(&mut self, _cx: &Cx, body: &Value) -> Option<Value> {
            (body["cmd"] == "step").then(|| json!({"ok": true, "stepped": true}))
        }
        fn widgets(&self) -> Vec<Value> {
            Vec::new()
        }
        fn draw_tip(&self, _p: &mut Painter, _mouse: (i32, i32)) {}
        fn click(&mut self, _cx: &mut Cx, _pos: (i32, i32)) -> bool {
            false
        }
        fn click_id(&mut self, _cx: &mut Cx, _id: &str) -> Result<(), String> {
            Ok(())
        }
        fn widget_rect(&self, _id: &str) -> Option<(Rect, Vec<String>)> {
            None
        }
    }

    #[test]
    fn a_screens_own_step_reaches_it_through_the_shared_path() {
        let mut s = shell();
        s.stack.push(Box::new(OwnStep));
        assert_eq!(s.bridge(&json!({"cmd": "step"}))["stepped"], json!(true));
        s.stack.pop();
        assert_eq!(s.bridge(&json!({"cmd": "step"}))["ok"], json!(false), "no screen answers it here");
    }

    /// F12 on a blank frame: it shows, no file comes, and an older shot does not pass for this one.
    #[test]
    fn a_refused_f12_shows_and_leaves_no_old_shot() {
        let mut s = shell();
        let shots = aegis_core::scratch::Scratch::vacant("f12");
        s.cx.shot_dir = shots.to_path_buf();
        std::fs::create_dir_all(&s.cx.shot_dir).unwrap();
        let path = s.cx.shot_dir.join("gui_shot.png");
        s.f12_shot();
        assert!(path.is_file(), "a drawn frame is saved");
        assert!(s.cx.notice.is_empty());
        s.gfx.canvas.clear(rgb(9, 9, 9));
        s.f12_shot();
        assert!(!path.exists(), "the old shot would look current");
        assert!(s.cx.notice.iter().any(|n| n.contains("Screenshot failed")), "{:?}", s.cx.notice);
        std::fs::remove_dir_all(&s.cx.shot_dir).unwrap();
    }

    #[test]
    fn the_link_quit_asks_for_what_the_quit_button_does() {
        let mut s = shell();
        assert!(!s.cx.quit);
        assert_eq!(s.bridge(&json!({"cmd": "quit"}))["ok"], json!(true));
        assert!(s.cx.quit);
    }

    thread_local! {
        static TEMP_FILES: std::cell::RefCell<Vec<std::path::PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    /// Removes the settings files the desktop tests wrote (call at the end of a test).
    fn clean() {
        TEMP_FILES.with(|t| t.borrow_mut().drain(..).for_each(|p| {
            let _ = std::fs::remove_file(p);
        }));
    }

    /// Two overlapping windows, each with a button: "a" at the left, "b" over a's right part (b on top).
    struct Desktop {
        w: Vec<Widget<&'static str>>,
        desk: crate::window::Desk,
    }

    impl Desktop {
        fn new() -> Self {
            use crate::window::{Desk, Frac, Layout, Win};
            let wins = vec![Win::new("a", "Alpha", (40, 30)), Win::new("b", "Beta", (40, 30))];
            let layouts = vec![
                Layout { name: "Both".into(), places: vec![("a".into(), Frac::new(0.0, 0.0, 0.5, 0.5)), ("b".into(), Frac::new(0.25, 0.0, 0.5, 0.5))] },
                Layout { name: "Solo".into(), places: vec![("a".into(), Frac::new(0.0, 0.0, 1.0, 1.0))] },
            ];
            Self { w: Vec::new(), desk: Desk::new(Rect::new(0, 20, 400, 200), wins, layouts) }
        }
        fn act(&mut self, cx: &mut Cx, a: &'static str) {
            cx.app.log.push(format!("click {a}"));
        }
    }

    impl Screen<App> for Desktop {
        fn id(&self) -> &'static str {
            "desktop"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            self.desk.toolbar(&mut ui, 2, 2);
            self.desk.draw(&mut ui, |ui, id, body| {
                // A button across the whole body width, so part of a's is under b.
                let (label, act, dy) = if id == "a" { ("A button", "a", 4) } else { ("B button", "b", 40) };
                ui.row(&format!("{id}.button"), Rect::new(body.x, body.y + dy, body.w + 40, 14), label, TEXT, Some(act), false, Vec::new());
                if id == "b" {
                    // A select at the bottom of the body: its open list reaches below the window.
                    let opts: Vec<String> = ["One", "Two", "Three"].iter().map(|s| s.to_string()).collect();
                    ui.select("b.pick", Rect::new(body.x, body.y + body.h - 14, 60, 12), &opts, 0, |_| "pick", true, Vec::new());
                }
            });
            self.w = ui.widgets;
        }
        fn desk(&mut self) -> Option<&mut crate::window::Desk> {
            Some(&mut self.desk)
        }
        crate::widget_plumbing!(Cx);
    }

    /// A menu of two `MenuStyle::Text` rows; what it was told to do.
    #[derive(Default)]
    struct TextMenu {
        w: Vec<crate::ui::Widget<&'static str>>,
        did: Vec<&'static str>,
    }

    impl TextMenu {
        fn act(&mut self, _cx: &mut Cx, a: &'static str) {
            self.did.push(a);
        }
    }

    impl Screen<App> for TextMenu {
        fn id(&self) -> &'static str {
            "text_menu"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui = Ui::new(g, mouse, live);
            let items = [crate::panes::MenuItem::new("a", "First", None, "a"), crate::panes::MenuItem::new("b", "Second", None, "b")];
            crate::panes::menu_column_styled(&mut ui, "m", 40, 40, 160, 18, 22, &items, crate::panes::MenuStyle::Text);
            self.w = ui.widgets;
        }
        fn state(&self, _cx: &Cx) -> Value {
            json!(self.did)
        }
        crate::widget_plumbing!();
    }

    /// The real input path of a text menu: a mouse move makes the row under it the active item, whatever the keyboard
    /// did before; Enter presses what is lit; leaving the labels keeps it; a mouse that rests there does not take the focus
    /// back from the keyboard; past the label's end the mouse takes nothing and a click does nothing.
    #[test]
    fn a_mouse_move_makes_a_text_row_the_one_active_item() {
        let mut s = shell();
        s.stack = vec![Box::new(TextMenu::default())];
        s.frame();
        let focus = |s: &Shell<App>| (s.gfx.kit.focus.clone(), s.gfx.kit.focus_visible);
        let infos = s.stack[0].widgets();
        let centre = |id: &str| {
            let r = info_rect(infos.iter().find(|w| w["id"] == id).unwrap());
            (r.x + r.w / 2, r.y + r.h / 2)
        };
        s.key(Key::Tab);
        s.frame();
        assert_eq!(focus(&s), (Some("m.a".to_string()), true), "the keyboard on the first");
        s.mouse = centre("m.b");
        s.mouse_moved();
        s.frame();
        assert_eq!(focus(&s), (Some("m.b".to_string()), true), "the mouse moved it");
        s.key(Key::Up);
        s.frame();
        s.frame();
        assert_eq!(focus(&s).0.as_deref(), Some("m.a"), "the mouse rests there: the keyboard's move stays");
        s.mouse = (600, 300);
        s.mouse_moved();
        s.frame();
        assert_eq!(focus(&s), (Some("m.a".to_string()), true), "off the labels the last one stays lit");
        s.key(Key::Enter);
        assert_eq!(s.stack[0].state(&s.cx), json!(["a"]), "Enter presses what is lit");
        let b = info_rect(infos.iter().find(|w| w["id"] == "m.b").unwrap());
        s.mouse = (b.x + b.w + 2, b.y + b.h / 2);
        s.mouse_moved();
        assert_eq!(focus(&s).0.as_deref(), Some("m.a"), "just past the label's end: not hovered");
        s.click();
        assert_eq!(s.stack[0].state(&s.cx), json!(["a"]), "and not clicked");
        s.frame();
        assert_eq!(focus(&s), (Some("m.a".to_string()), true), "a click off the labels leaves the active item lit");
        s.key(Key::Enter);
        assert_eq!(s.stack[0].state(&s.cx), json!(["a", "a"]), "and Enter still presses it");
    }

    /// A row lit under a mouse that has not moved (no focus showing yet) is the active item: Enter presses it.
    #[test]
    fn a_text_row_lit_under_a_resting_mouse_can_be_pressed() {
        let mut s = shell();
        s.stack = vec![Box::new(TextMenu::default())];
        s.frame();
        assert_eq!((s.gfx.kit.focus.clone(), s.gfx.kit.focus_visible), (None, false), "nothing under the mouse");
        let r = info_rect(s.stack[0].widgets().iter().find(|w| w["id"] == "m.b").unwrap());
        s.mouse = (r.x + r.w / 2, r.y + r.h / 2);
        s.frame();
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (Some("m.b"), true));
        s.key(Key::Enter);
        assert_eq!(s.stack[0].state(&s.cx), json!(["b"]));
    }

    /// A screen without menu rows; counts how many times its widget list is built.
    #[derive(Default)]
    struct Counting(std::rc::Rc<std::cell::Cell<u32>>);

    impl Screen<App> for Counting {
        fn id(&self) -> &'static str {
            "counting"
        }
        fn draw(&mut self, _cx: &mut Cx, _g: &mut Gfx, _mouse: (i32, i32), _live: bool) {}
        fn widgets(&self) -> Vec<Value> {
            self.0.set(self.0.get() + 1);
            vec![json!({"id": "b", "kind": "button", "rect": [0, 0, 40, 14], "enabled": true, "clickable": true})]
        }
        fn draw_tip(&self, _p: &mut Painter, _mouse: (i32, i32)) {}
        fn click(&mut self, _cx: &mut Cx, _pos: (i32, i32)) -> bool {
            false
        }
        fn click_id(&mut self, _cx: &mut Cx, _id: &str) -> Result<(), String> {
            Ok(())
        }
        fn widget_rect(&self, _id: &str) -> Option<(Rect, Vec<String>)> {
            None
        }
    }

    /// A mouse move on a screen without menu rows builds no widget list; one with them does its work.
    #[test]
    fn a_mouse_move_costs_nothing_on_a_screen_without_text_rows() {
        let mut s = shell();
        let built = std::rc::Rc::new(std::cell::Cell::new(0));
        s.stack = vec![Box::new(Counting(built.clone()))];
        s.frame();
        assert!(built.get() > 0, "the frame looks");
        built.set(0);
        s.mouse = (10, 5);
        s.mouse_moved();
        assert_eq!((built.get(), s.gfx.kit.focus.clone()), (0, None));
        s.stack = vec![Box::new(TextMenu::default())];
        s.frame();
        assert!(s.text_rows, "the frame noted them");
    }

    /// A shell over the two-window screen with its own settings file in the temp dir (removed by `clean`).
    fn desktop(name: &str) -> Shell<App> {
        let mut s = shell();
        let path = std::env::temp_dir().join(format!("aegis_desk_{name}_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        TEMP_FILES.with(|t| t.borrow_mut().push(path.clone()));
        s.cx.settings = Settings::load(path, json!({}));
        s.stack = vec![Box::new(Desktop::new())];
        s.frame();
        s
    }

    fn rect_of(s: &mut Shell<App>, id: &str) -> Rect {
        s.stack.last_mut().unwrap().desk().unwrap().rect(id).unwrap()
    }

    #[test]
    fn dragging_the_title_moves_and_the_grip_resizes_and_the_desk_is_saved() {
        let mut s = desktop("drag");
        let a = rect_of(&mut s, "a");
        drag(&mut s, (a.x + 10, a.y + 5), (a.x + 40, a.y + 35));
        let moved = rect_of(&mut s, "a");
        assert_eq!((moved.x, moved.y), (a.x + 30, a.y + 30));
        let saved = s.cx.settings.v["desks"]["desktop"].clone();
        assert!(saved["wins"].as_array().is_some_and(|w| w.len() == 2), "{saved}");
        let g = (moved.x + moved.w - 3, moved.y + moved.h - 3);
        drag(&mut s, g, (g.0 - 20, g.1 - 10));
        let sized = rect_of(&mut s, "a");
        assert_eq!((sized.w, sized.h), (moved.w - 20, moved.h - 10));
        assert!(s.cx.app.log.is_empty(), "no button was clicked");
        clean();
    }

    #[test]
    fn a_window_covers_the_widgets_under_it_and_a_press_raises_it() {
        let mut s = desktop("cover");
        let b = rect_of(&mut s, "b");
        // a's button reaches under b: a press there lands on b (raised, its backdrop), not on a's button.
        let under = (b.x + 4, b.y + 20);
        s.mouse = under;
        s.click();
        s.release();
        assert!(s.cx.app.log.is_empty(), "{:?}", s.cx.app.log);
        let a = rect_of(&mut s, "a");
        s.mouse = (a.x + 6, a.y + 20);
        s.click();
        s.release();
        assert_eq!(s.cx.app.log, ["click a"]);
        s.frame();
        assert_eq!(s.stack.last_mut().unwrap().desk().unwrap().top_at(under), Some("a"), "the press raised a");
        clean();
    }

    #[test]
    fn the_toolbar_and_window_buttons_open_close_collapse_and_lay_out() {
        let mut s = desktop("toolbar");
        assert_eq!(s.bridge(&json!({"cmd": "click", "id": "win.b.close"}))["ok"], json!(true));
        assert!(!s.stack.last_mut().unwrap().desk().unwrap().win("b").unwrap().open);
        s.bridge(&json!({"cmd": "click", "id": "desk.toggle.b"}));
        assert!(s.stack.last_mut().unwrap().desk().unwrap().win("b").unwrap().open);
        s.bridge(&json!({"cmd": "click", "id": "win.a.collapse"}));
        assert_eq!(rect_of(&mut s, "a").h, crate::window::TITLE_H);
        s.bridge(&json!({"cmd": "click", "id": "desk.layout.Solo"}));
        let d = s.stack.last_mut().unwrap().desk().unwrap();
        assert!(d.win("a").unwrap().open && !d.win("b").unwrap().open && d.layout == "Solo");
        assert_eq!(s.cx.settings.v["desks"]["desktop"]["layout"], json!("Solo"));
        clean();
    }

    #[test]
    fn a_select_in_a_window_opens_its_whole_list() {
        let mut s = desktop("select");
        s.gfx.kit.open = Some("b.pick".into());
        s.frame();
        let body = s.stack.last_mut().unwrap().desk().unwrap().body("b").unwrap();
        let opts: Vec<Rect> = s.stack.last().unwrap().widgets().iter().filter(|w| w["kind"] == "option").map(info_rect).collect();
        assert_eq!(opts.len(), 3);
        assert!(opts.iter().all(|r| r.w > 0 && r.h > 0), "{opts:?}");
        assert!(opts.iter().any(|r| r.y + r.h > body.y + body.h), "the list reaches past the body, uncut");
        clean();
    }

    #[test]
    fn a_window_backdrop_stops_a_drop_on_a_target_it_covers() {
        let target = json!({"id": "t", "kind": "row", "rect": [0, 0, 50, 20], "enabled": true, "drop": "slot"});
        let backdrop = json!({"id": "win.w.body", "kind": "backdrop", "rect": [10, 0, 50, 50], "enabled": true});
        assert_eq!(drop_key_at(&[target.clone(), backdrop.clone()], (20, 10)), None, "covered");
        assert_eq!(drop_key_at(&[target.clone(), backdrop.clone()], (5, 10)), Some("slot"), "beside the window");
        assert_eq!(drop_key_at(&[backdrop, target], (20, 10)), Some("slot"), "a target inside the window");
    }

    #[test]
    fn esc_mid_move_keeps_and_saves_the_window_and_a_bad_desks_value_is_replaced() {
        let mut s = desktop("esc");
        s.cx.settings.v["desks"] = json!([]);
        let a = rect_of(&mut s, "a");
        s.mouse = (a.x + 10, a.y + 5);
        s.click();
        s.mouse = (a.x + 30, a.y + 25);
        s.drag_to();
        s.key(Key::Esc);
        let moved = rect_of(&mut s, "a");
        assert_eq!((moved.x, moved.y), (a.x + 20, a.y + 20));
        let saved = s.cx.settings.v["desks"]["desktop"]["wins"].as_array().cloned().unwrap_or_default();
        let x = saved.iter().find(|w| w["id"] == "a").and_then(|w| w["frac"][0].as_f64()).unwrap();
        assert!((x * 400.0 - 20.0).abs() < 0.5, "saved where it was left: {x}");
        clean();
    }

    #[test]
    fn the_agent_link_lists_and_drives_windows_and_widgets_stay_in_their_window() {
        let mut s = desktop("agent");
        let list = s.bridge(&json!({"cmd": "windows"}));
        assert_eq!(list["desk"]["windows"].as_array().map(|w| w.len()), Some(2));
        let before = rect_of(&mut s, "a");
        assert_eq!(s.bridge(&json!({"cmd": "window", "id": "a", "op": "move", "by": [20, 10]}))["ok"], json!(true));
        assert_eq!(rect_of(&mut s, "a"), Rect::new(before.x + 20, before.y + 10, before.w, before.h));
        assert_eq!(s.bridge(&json!({"cmd": "window", "id": "zz"}))["ok"], json!(false));
        assert_eq!(s.bridge(&json!({"cmd": "window", "id": "a", "op": "spin"}))["ok"], json!(false));
        assert_eq!(s.bridge(&json!({"cmd": "window", "layout": "Both"}))["ok"], json!(true));
        s.frame();
        let tree = s.bridge(&json!({"cmd": "ui_tree"}));
        let a_body = rect_of(&mut s, "a");
        let button = tree["widgets"].as_array().unwrap().iter().find(|w| w["id"] == "a.button").cloned().unwrap();
        assert!(button["rect"][0].as_i64().unwrap() + button["rect"][2].as_i64().unwrap() <= (a_body.x + a_body.w) as i64, "clipped to the window: {button}");
        clean();
    }

    /// A list of 20 clickable rows (14 px each, 280 px of content) in a 100 px tall scroll area.
    #[derive(Default)]
    struct List {
        w: Vec<Widget<&'static str>>,
    }

    const ROWS: [&str; 20] = ["r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15", "r16", "r17", "r18", "r19"];

    impl List {
        fn act(&mut self, cx: &mut Cx, a: &'static str) {
            cx.app.log.push(format!("click {a}"));
        }
    }

    impl Screen<App> for List {
        fn id(&self) -> &'static str {
            "list"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            let view = Rect::new(10, 20, 120, 100);
            let off = ui.scroll_begin("list.rows", view, 14 * ROWS.len() as i32);
            for (i, id) in ROWS.iter().enumerate() {
                ui.row(id, Rect::new(view.x, view.y - off + i as i32 * 14, view.w, 14), id, TEXT, Some(*id), false, Vec::new());
            }
            ui.scroll_end();
            self.w = ui.widgets;
        }
        fn wheel(&mut self, cx: &mut Cx, lines: i32) {
            cx.app.log.push(format!("wheel {lines}"));
        }
        crate::widget_plumbing!(Cx);
    }

    fn list() -> Shell<App> {
        let mut s = shell();
        s.stack = vec![Box::new(List::default())];
        s.frame();
        s
    }

    fn row_rect(s: &Shell<App>, id: &str) -> Rect {
        s.stack.last().unwrap().widgets().iter().find(|w| w["id"] == id).map(info_rect).unwrap()
    }

    #[test]
    fn the_wheel_scrolls_the_area_under_the_mouse_within_its_content() {
        let mut s = list();
        s.mouse = (20, 30);
        s.wheel(-2);
        s.frame();
        assert_eq!(s.gfx.kit.scroll["list.rows"], 2 * crate::ui::SCROLL_STEP);
        assert_eq!(row_rect(&s, "r2").y, 20 + 28 - 24);
        s.wheel(-100);
        assert_eq!(s.gfx.kit.scroll["list.rows"], 280 - 100, "stops at the end of the content");
        s.wheel(100);
        assert_eq!(s.gfx.kit.scroll["list.rows"], 0);
        s.mouse = (300, 300);
        s.wheel(-1);
        assert_eq!(s.cx.app.log, ["wheel -1"], "outside any area the screen gets the wheel");
    }

    #[test]
    fn rows_scrolled_out_of_view_cannot_be_clicked_and_visible_ones_can() {
        let mut s = list();
        assert_eq!(row_rect(&s, "r10").h, 0, "below the view: clipped to nothing");
        assert_eq!(s.bridge(&json!({"cmd": "scroll", "id": "list.rows", "to": 140}))["offset"], json!(140));
        s.frame();
        let r = row_rect(&s, "r10");
        assert!(r.h > 0 && r.y >= 20);
        s.mouse = (r.x + 5, r.y + 3);
        s.click();
        assert_eq!(s.cx.app.log, ["click r10"]);
        assert_eq!(row_rect(&s, "r0").h, 0, "above the view now");
        assert_eq!(s.bridge(&json!({"cmd": "scroll", "id": "nope", "by": 1}))["ok"], json!(false));
    }

    #[test]
    fn an_overlay_keeps_the_scroll_offsets_and_a_new_stack_clears_them() {
        let mut s = desktop("wheel");
        s.gfx.kit.scroll.insert("x".into(), 5);
        s.cx.notice.push("an overlay".into());
        s.settle();
        assert_eq!(s.gfx.kit.scroll.get("x"), Some(&5), "a notice over the screen keeps them");
        s.cx.nav.push(Nav::Pop);
        s.settle();
        assert_eq!(s.gfx.kit.scroll.get("x"), Some(&5), "and so does coming back");
        s.cx.nav.push(Nav::Reset(Box::new(List::default())));
        s.settle();
        assert!(s.gfx.kit.scroll.is_empty(), "a new stack starts over");
        clean();
    }

    /// A timeline of six blocks: a hero acting in blocks 2-4 and 5, a masked foe.
    #[derive(Default)]
    struct Plan {
        w: Vec<Widget<String>>,
    }

    impl Plan {
        fn act(&mut self, cx: &mut Cx, a: String) {
            cx.app.log.push(a);
        }
    }

    impl Screen<App> for Plan {
        fn id(&self) -> &'static str {
            "plan"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            use crate::timeline::{Section, Segment, Timeline, Track};
            let mut ui: Ui<String> = Ui::new(g, mouse, live);
            let seg = |key: &str, start, end| Segment { key: key.into(), start, end, label: key.into(), fill: rgb(60, 90, 140), ..Segment::default() };
            let t = Timeline {
                blocks: 6,
                playhead: 3,
                selected_block: None,
                selected_track: Some("hero".into()),
                sections: vec![
                    Section { title: "Allies".into(), colour: TEXT, tracks: vec![Track { key: "hero".into(), label: "Hero".into(), colour: TEXT, segments: vec![seg("slash", 2, 4), seg("guard", 5, 5)], ..Track::default() }] },
                    Section { title: "Enemies".into(), colour: TEXT, tracks: vec![Track { key: "foe".into(), label: "Foe".into(), colour: TEXT, segments: vec![seg("hidden", 1, 6)], masked: Some("Plans in secret".into()), ..Track::default() }] },
                ],
            };
            t.draw(&mut ui, "tl", Rect::new(10, 10, 74 + 4 + 300, 120), |b| format!("block {b}"), |k| format!("track {k}"), |k, s| format!("seg {k} {s}"));
            self.w = ui.widgets;
        }
        crate::widget_plumbing!(Cx);
    }

    #[test]
    fn a_timeline_draws_segments_over_their_blocks_masks_tracks_and_reports_clicks() {
        let mut s = shell();
        s.stack = vec![Box::new(Plan::default())];
        s.frame();
        let infos = s.stack.last().unwrap().widgets();
        let rect = |id: &str| infos.iter().find(|w| w["id"] == id).map(info_rect);
        let (b2, b4, slash) = (rect("tl.block.2").unwrap(), rect("tl.block.4").unwrap(), rect("tl.seg.hero.slash").unwrap());
        assert!(slash.x >= b2.x - 1 && slash.x + slash.w <= b4.x + b4.w + 1 && slash.w > 2 * b2.w, "{slash:?} spans B2-B4");
        assert!(rect("tl.seg.foe.hidden").is_none(), "a masked track hides its segments");
        assert!(rect("tl.track.foe.masked").is_some());
        for id in ["tl.block.4", "tl.track.foe", "tl.seg.hero.guard"] {
            assert_eq!(s.bridge(&json!({"cmd": "click", "id": id}))["ok"], json!(true));
        }
        assert_eq!(s.cx.app.log, ["block 4", "track foe", "seg hero guard"]);
    }

    /// Two buttons that ask a question: the default form (Yes) and the destructive one (No, ASKNO2).
    #[derive(Default)]
    struct Asker {
        w: Vec<Widget<bool>>,
    }

    impl Asker {
        fn act(&mut self, cx: &mut Cx, no_default: bool) {
            if no_default {
                cx.ask_tag_no("Delete it?", "asker.yes");
            } else {
                cx.ask_tag("Keep it?", "asker.yes");
            }
        }
    }

    impl Screen<App> for Asker {
        fn id(&self) -> &'static str {
            "asker"
        }
        fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            let mut ui: Ui<bool> = Ui::new(g, mouse, live);
            ui.button("asker.plain", Rect::new(10, 10, 60, 14), "Plain", false, true);
            ui.button("asker.risky", Rect::new(10, 30, 60, 14), "Risky", true, true);
            self.w = ui.widgets;
        }
        crate::widget_plumbing!(Cx);
    }

    /// A person clicks one of the Asker's buttons: the question opens and draws once.
    fn asked_by_click(id: &str) -> Shell<App> {
        let mut s = shell();
        s.stack = vec![Box::new(Asker::default())];
        s.frame();
        let r = s.stack.last().unwrap().widget_rect(id).unwrap().0;
        s.mouse = (r.x + 2, r.y + 2);
        s.click();
        s.frame();
        assert_eq!(s.stack.last().unwrap().id(), "ask");
        s
    }

    #[test]
    fn a_destructive_question_asks_for_the_pointer_on_no() {
        let s = asked_by_click("asker.risky");
        let r = s.stack.last().unwrap().widget_rect("ask.no").unwrap().0;
        assert_eq!(s.pointer_req, Some((r.x + r.w / 2, r.y + r.h / 2)), "the centre of No");
        assert_eq!((s.gfx.kit.focus.as_deref(), s.gfx.kit.focus_visible), (Some("ask.no"), true), "No has the focus");
        // the request is made once, as the question opens
        let mut s = s;
        s.pointer_req = None;
        s.frame();
        assert_eq!(s.pointer_req, None);
    }

    #[test]
    fn a_default_question_leaves_the_pointer_alone() {
        let s = asked_by_click("asker.plain");
        assert_eq!(s.pointer_req, None);
        assert!(!s.gfx.kit.focus_visible, "no focus shown");
    }

    #[test]
    fn a_question_the_agent_link_opens_leaves_the_pointer_alone() {
        let mut s = shell();
        s.stack = vec![Box::new(Asker::default())];
        s.frame();
        assert_eq!(s.bridge(&json!({"cmd": "click", "id": "asker.risky"}))["ok"], json!(true));
        s.frame();
        assert_eq!(s.stack.last().unwrap().id(), "ask");
        assert_eq!(s.pointer_req, None);
        assert_eq!(s.gfx.kit.focus.as_deref(), Some("ask.no"), "the focus still opens on No");
        // Enter on the focused No answers No
        s.key(Key::Enter);
        s.frame();
        assert_eq!(s.stack.last().unwrap().id(), "asker");
        assert!(!s.cx.take_answer("asker.yes"));
    }

    // ---- LAYOUT720-E1: each screen's grid (the painter's half: tests/layout_grid.rs) ------------------------------

    use crate::painter::Grid;

    /// Widgets in old numbers times the grid's unit: two rows to click, a draggable and a drop target, a scroll area.
    struct OnGrid {
        id: &'static str,
        grid: Grid,
        overlay: bool,
        /// A text past the old canvas's corner but inside the grid's.
        far_text: bool,
        w: Vec<Widget<&'static str>>,
    }

    fn on_grid(id: &'static str, grid: Grid, overlay: bool) -> Box<OnGrid> {
        Box::new(OnGrid { id, grid, overlay, far_text: false, w: Vec::new() })
    }

    impl OnGrid {
        fn act(&mut self, cx: &mut Cx, a: &'static str) {
            cx.app.log.push(format!("click {a}"));
        }
    }

    impl Screen<App> for OnGrid {
        fn id(&self) -> &'static str {
            self.id
        }
        fn grid(&self) -> Grid {
            self.grid
        }
        fn overlay(&self) -> bool {
            self.overlay
        }
        fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
            cx.app.log.push(format!("draw {} s={} g={} mouse={mouse:?} live={live}", self.id, g.view.s, g.view.g));
            let u = self.grid.g();
            let at = |x: i32, y: i32, w: i32, h: i32| Rect::new(x * u, y * u, w * u, h * u);
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            ui.row("left", at(20, 30, 100, 30), "Left", TEXT, Some("left"), false, Vec::new());
            ui.row("far", at(500, 300, 100, 30), "Far", TEXT, Some("far"), false, Vec::new());
            ui.row("item", at(20, 100, 60, 20), "Item", TEXT, Some("item"), false, Vec::new());
            ui.draggable("item.x");
            ui.row("slot", at(200, 100, 60, 20), "Slot", TEXT, None, false, Vec::new());
            ui.drop_target("slot.x");
            let view = at(300, 20, 100, 100);
            ui.scroll_begin("area", view, at(0, 0, 0, 1000).h);
            ui.scroll_end();
            if self.far_text {
                ui.p.text(800.0, 400.0, "Far text", BODY, TEXT);
            }
            self.w = ui.widgets;
        }
        fn accept_drop(&mut self, cx: &mut Cx, from: &str, to: &str) {
            cx.app.log.push(format!("drop {from} on {to}"));
        }
        crate::widget_plumbing!(Cx);
    }

    const GRIDS: [Grid; 3] = [Grid::Old, Grid::Quadrant, Grid::Native];

    /// A shell on a window of (`pw`, `ph`) with these screens, one frame drawn, the log empty.
    fn window(pw: i32, ph: i32, stack: Vec<Box<dyn Screen<App>>>) -> Shell<App> {
        let mut s = shell();
        s.gfx.canvas = Canvas::new(pw, ph);
        s.stack = stack;
        s.frame();
        s.cx.app.log.clear();
        s
    }

    /// Where the OS pointer sits over the middle of widget `id`.
    fn middle_px(s: &Shell<App>, id: &str) -> (f32, f32) {
        let p = s.gfx.view.rect(row_rect(s, id));
        ((p.x + p.w / 2) as f32, (p.y + p.h / 2) as f32)
    }

    #[test]
    fn a_physical_pointer_clicks_the_widget_under_it_on_every_grid() {
        for (pw, ph) in [(1280, 720), (1920, 1080), (1366, 768), (2000, 900)] {
            for grid in GRIDS {
                let mut s = window(pw, ph, vec![on_grid("top", grid, false)]);
                assert_eq!(s.gfx.view, View::fit(pw as u32, ph as u32).for_grid(grid), "{grid:?}");
                for id in ["left", "far"] {
                    let (x, y) = middle_px(&s, id);
                    s.pointer_moved(x, y);
                    let r = row_rect(&s, id);
                    assert!(r.contains(s.mouse.0, s.mouse.1), "{grid:?} {pw}x{ph} {id}: {:?} in {r:?}", s.mouse);
                    s.click();
                    assert_eq!(s.cx.app.log, [format!("click {id}")], "{grid:?} {pw}x{ph}");
                    s.cx.app.log.clear();
                    // Just past its left edge: nothing there.
                    s.pointer_moved(s.gfx.view.rect(r).x as f32 - 2.0, y);
                    s.click();
                    assert!(s.cx.app.log.is_empty(), "{grid:?} {pw}x{ph} {id} {:?}", s.cx.app.log);
                }
            }
        }
    }

    /// Past the quarter nothing is drawn and nothing is hit: where the wrapper's widget would be is blank.
    #[test]
    fn a_click_past_a_quadrant_screen_hits_nothing() {
        let mut s = window(1280, 720, vec![on_grid("top", Grid::Quadrant, false)]);
        let wrapper = View::fit(1280, 720).rect(row_rect(&s, "far"));
        s.pointer_moved((wrapper.x + wrapper.w / 2) as f32, (wrapper.y + wrapper.h / 2) as f32);
        s.click();
        assert!(s.cx.app.log.is_empty(), "{:?}", s.cx.app.log);
        assert!(s.mouse.0 >= W, "{:?} is past its old numbers", s.mouse);
    }

    #[test]
    fn every_screen_of_the_stack_draws_on_its_own_grid_and_the_top_keeps_its_view() {
        let mut s = window(1920, 1080, vec![on_grid("base", Grid::Native, false), on_grid("mid", Grid::Quadrant, true), on_grid("top", Grid::Old, true)]);
        s.mouse = (11, 22);
        s.frame();
        assert_eq!(s.cx.app.log, [
            "draw base s=1.5 g=2 mouse=(-1, -1) live=false",
            "draw mid s=1.5 g=1 mouse=(-1, -1) live=false",
            "draw top s=3 g=1 mouse=(11, 22) live=true",
        ]);
        assert_eq!(s.gfx.view, View::fit(1920, 1080), "the top's, for the ring, the lists and the tip");
    }

    #[test]
    fn the_mouse_follows_the_top_screen_when_the_stack_changes() {
        let mut s = window(1280, 720, vec![on_grid("base", Grid::Native, false)]);
        s.pointer_moved(400.0, 300.0);
        assert_eq!(s.mouse, (400, 300));
        let open = |s: &mut Shell<App>, grid: Grid| {
            s.cx.nav.push(Nav::Push(on_grid("over", grid, true)));
            s.settle();
        };
        open(&mut s, Grid::Old);
        assert_eq!((s.mouse, s.gfx.view), ((200, 150), View::fit(1280, 720)), "the pointer has not moved: old units");
        s.cx.nav.push(Nav::Pop);
        s.settle();
        assert_eq!(s.mouse, (400, 300), "and back, to the unit, from the pointer's own place");
        open(&mut s, Grid::Quadrant);
        assert_eq!(s.mouse, (400, 300), "one view scale: the quadrant's old numbers");
        s.cx.nav.push(Nav::Reset(on_grid("old", Grid::Old, false)));
        s.settle();
        assert_eq!(s.mouse, (200, 150));
        // An agent-link position has no physical move with it: the stale pointer is not what it is read from.
        s.mouse = (50, 60);
        open(&mut s, Grid::Native);
        assert_eq!((s.mouse.0 / 2, s.mouse.1 / 2), (50, 60), "{:?}: inside that old unit", s.mouse);
    }

    /// The old path: a stack change between screens with one view leaves the mouse alone, even off the canvas.
    #[test]
    fn a_stack_change_on_one_grid_leaves_the_mouse_alone() {
        let mut s = window(1920, 1080, vec![on_grid("base", Grid::Old, false)]);
        s.pointer_moved(900.0, 500.0);
        for at in [(7, 9), (-500, -500), (900, 900)] {
            s.mouse = at;
            s.cx.nav.push(Nav::Push(on_grid("over", Grid::Old, true)));
            s.settle();
            assert_eq!(s.mouse, at);
            s.cx.nav.push(Nav::Pop);
            s.settle();
            assert_eq!(s.mouse, at);
        }
    }

    #[test]
    fn a_resize_keeps_the_top_screens_grid() {
        let mut s = window(1280, 720, vec![on_grid("top", Grid::Native, false)]);
        s.gfx.canvas = Canvas::new(1920, 1080);
        s.gfx.view = s.top_view();
        assert_eq!(s.gfx.view, View::fit(1920, 1080).for_grid(Grid::Native));
        let (x, y) = middle_px(&s, "far");
        s.pointer_moved(x, y);
        s.click();
        assert_eq!(s.cx.app.log, ["click far"]);
    }

    #[test]
    fn the_agent_link_reports_the_grid_and_works_in_the_top_screens_units() {
        for (grid, canvas, name) in [(Grid::Old, json!([640, 360]), "old"), (Grid::Quadrant, json!([640, 360]), "quadrant"), (Grid::Native, json!([1280, 720]), "native")] {
            let mut s = window(1920, 1080, vec![on_grid("top", grid, false)]);
            let tree = s.bridge(&json!({"cmd": "ui_tree"}));
            assert_eq!((&tree["canvas"], tree["grid"].as_str()), (&canvas, Some(name)));
            let u = grid.g();
            let far = tree["widgets"].as_array().unwrap().iter().find(|w| w["id"] == "far").cloned().unwrap();
            assert_eq!(far["rect"], json!([500 * u, 300 * u, 100 * u, 30 * u]), "{name}: rects in its units");
            assert_eq!(s.bridge(&json!({"cmd": "click", "id": "left"}))["ok"], json!(true));
            s.bridge(&json!({"cmd": "hover", "id": "far"}));
            assert_eq!(s.bridge(&json!({"cmd": "ui_tree"}))["mouse"], json!([550 * u, 315 * u]), "{name}");
            let slot = row_rect(&s, "slot");
            s.cx.app.log.clear();
            assert_eq!(s.bridge(&json!({"cmd": "drag", "id": "item", "to": [slot.x + 2, slot.y + 2]}))["ok"], json!(true));
            assert_eq!(s.cx.app.log, ["drop item.x on slot.x"], "{name}: [x, y] in its units");
        }
    }

    /// The wheel, the agent's scroll and the drag threshold are old numbers times the unit.
    #[test]
    fn scroll_steps_and_the_drag_threshold_are_in_the_top_screens_units() {
        for grid in GRIDS {
            let u = grid.g();
            let mut s = window(1920, 1080, vec![on_grid("top", grid, false)]);
            let area = s.stack.last().unwrap().widgets().iter().find(|w| w["id"] == "area").map(info_rect).unwrap();
            s.mouse = (area.x + 1, area.y + 1);
            s.wheel(-2);
            assert_eq!(s.gfx.kit.scroll["area"], 2 * crate::ui::SCROLL_STEP * u, "{grid:?}");
            assert_eq!(s.bridge(&json!({"cmd": "scroll", "id": "area", "by": -1}))["offset"], json!(3 * crate::ui::SCROLL_STEP * u), "{grid:?}");
            let item = row_rect(&s, "item");
            let from = (item.x + 2, item.y + 2);
            s.mouse = from;
            s.click();
            s.mouse = (from.0 + DRAG_START * u, from.1);
            s.drag_to();
            assert!(s.gfx.kit.drag.is_none(), "{grid:?}: at the threshold");
            s.mouse = (from.0 + DRAG_START * u + 1, from.1);
            s.drag_to();
            assert!(s.gfx.kit.drag.is_some(), "{grid:?}: past it");
        }
    }

    // ---- LAYOUT720-E5: the game's canvas and the layout audit (the rest: tests/canvas_policy.rs, tests/layout_audit.rs) ---

    use crate::layout_specimen::LayoutSpecimen;

    const FIT: CanvasPolicy = CanvasPolicy::Fit { w: 1280, h: 720 };

    fn specimen() -> Box<LayoutSpecimen> {
        Box::new(LayoutSpecimen::default())
    }

    #[test]
    fn a_games_policy_reaches_its_native_screens_and_no_other() {
        let mut s = window(3440, 1440, vec![on_grid("top", Grid::Native, false)]);
        assert_eq!(s.gfx.view.canvas(), crate::painter::GRID, "the default: Fixed");
        s.policy = FIT;
        s.frame();
        assert_eq!(s.gfx.view, View::native(FIT, 3440, 1440, 1.0, 1.0));
        assert_eq!(s.bridge(&json!({"cmd": "ui_tree"}))["canvas"], json!([1720, 720]));
        let (s_fit, whole) = (s.gfx.view.s, View::fit(3440, 1440));
        s.cx.nav.push(Nav::Push(on_grid("old", Grid::Old, false)));
        s.frame();
        assert_eq!(s.gfx.view, whole, "an old screen keeps the whole window's view, whatever the game declares");
        s.cx.nav.push(Nav::Pop);
        s.frame();
        assert_eq!(s.gfx.view.s, s_fit, "and the native one its own");
        // A native screen under an old overlay: each draws with its own view, the top's stays.
        s.cx.nav.push(Nav::Push(on_grid("over", Grid::Old, true)));
        s.frame();
        assert_eq!(s.gfx.view, whole);
        assert!(s.cx.app.log.iter().any(|l| l.starts_with(&format!("draw top s={s_fit} g=2"))), "{:?}", s.cx.app.log);
    }

    #[test]
    fn a_physical_pointer_clicks_the_widget_under_it_on_a_fit_canvas() {
        for (pw, ph) in [(3440, 1440), (1920, 1200), (1280, 800)] {
            let mut s = window(pw, ph, vec![on_grid("top", Grid::Native, false)]);
            s.policy = FIT;
            s.frame();
            for id in ["left", "far"] {
                let (x, y) = middle_px(&s, id);
                s.pointer_moved(x, y);
                assert!(row_rect(&s, id).contains(s.mouse.0, s.mouse.1), "{pw}x{ph} {id}: {:?}", s.mouse);
            }
        }
    }

    #[test]
    fn a_resize_re_resolves_a_native_screen_with_no_screen_edit() {
        let mut s = window(1280, 720, vec![specimen()]);
        s.policy = FIT;
        s.frame();
        let wide = |s: &mut Shell<App>, pw: i32, ph: i32| {
            s.gfx.canvas = Canvas::new(pw, ph);
            s.gfx.view = s.top_view();
            s.frame();
            let t = s.bridge(&json!({"cmd": "ui_tree"}));
            (t["canvas"].clone(), row_rect(s, "layout.close"))
        };
        let (narrow, a) = wide(&mut s, 1280, 720);
        let (broad, b) = wide(&mut s, 3440, 1440);
        assert_eq!((narrow, broad), (json!([1280, 720]), json!([1720, 720])));
        assert!(b.x + b.w > a.x + a.w, "the close button follows the canvas's right edge: {a:?} {b:?}");
    }

    #[test]
    fn the_layout_audit_answers_under_its_own_key() {
        // The text audit's counts keep their exact form.
        let mut s = window(1280, 720, vec![specimen()]);
        let out = s.bridge(&json!({"cmd": "text_audit"}));
        assert_eq!(out["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "{out}");
        assert_eq!((out["layout"]["audited"].clone(), out["layout"]["clean"].clone(), out["layout"]["canvas"].clone()), (json!(true), json!(true), json!([1280, 720])), "{out}");
        assert_eq!(out["layout"]["counts"].as_object().unwrap().len(), layout_audit::KINDS.len());
        // A screen in old numbers keeps its answer, and is not audited for layout.
        for grid in [Grid::Old, Grid::Quadrant] {
            let mut s = window(1280, 720, vec![on_grid("top", grid, false)]);
            let out = s.bridge(&json!({"cmd": "text_audit"}));
            assert_eq!(out["layout"]["audited"], json!(false), "{grid:?}: {out}");
            assert!(out["counts"].is_object());
            let out = s.bridge(&json!({"cmd": "layout_audit"}));
            assert_eq!((out["ok"].clone(), out["audited"].clone(), out["screen"].clone()), (json!(true), json!(false), json!("top")), "{grid:?}: {out}");
            assert!(out["why"].as_str().unwrap().contains(grid.name()), "{out}");
        }
    }

    #[test]
    fn the_audit_records_the_top_screen_only() {
        // An overlay over a native screen with a zero-size widget: only what the top screen drew is read.
        struct Bare {
            over: bool,
            w: Vec<Widget<()>>,
        }
        impl Bare {
            fn act(&mut self, _: &mut Cx, _: ()) {}
        }
        impl Screen<App> for Bare {
            fn id(&self) -> &'static str {
                "bare"
            }
            fn grid(&self) -> Grid {
                Grid::Native
            }
            fn overlay(&self) -> bool {
                self.over
            }
            fn draw(&mut self, _: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
                let mut ui: Ui<()> = Ui::new(g, mouse, live);
                if !self.over {
                    ui.push("under.gone", "row", "", Rect::new(0, 0, 0, 0), true, false, None, Vec::new());
                }
                self.w = ui.widgets;
            }
            crate::widget_plumbing!(Cx);
        }
        let mut s = window(1280, 720, vec![Box::new(Bare { over: false, w: Vec::new() }), Box::new(Bare { over: true, w: Vec::new() })]);
        assert_eq!(s.bridge(&json!({"cmd": "text_audit"}))["layout"]["clean"], json!(true));
        s.stack.pop();
        let out = s.bridge(&json!({"cmd": "text_audit"}));
        assert_eq!(out["layout"]["counts"]["zero_size"], json!(1), "alone, it is read: {out}");
    }

    #[test]
    fn the_sweep_passes_on_desktop_and_leaves_the_window_as_it_was() {
        let mut s = window(1920, 1080, vec![specimen()]);
        s.gfx.kit.device.dpr = 1.25;
        s.frame();
        s.mouse = (321, 123);
        s.gfx.fonts.scale = 1.15;
        s.frame();
        let before = (s.gfx.canvas.px.clone(), s.gfx.view, s.gfx.kit.device, s.policy);
        let out = s.bridge(&json!({"cmd": "layout_audit"}));
        assert_eq!((out["ok"].clone(), out["audited"].clone(), out["pass"].clone(), out["screen"].clone()), (json!(true), json!(true), json!(true), json!(crate::layout_specimen::ID)), "{out}");
        assert_eq!(out["profiles"].as_array().unwrap().len(), layout_audit::PROFILES.len());
        let names: Vec<&str> = out["profiles"].as_array().unwrap().iter().map(|p| p["profile"].as_str().unwrap()).collect();
        assert_eq!(names, layout_audit::PROFILES.map(|p| p.name));
        assert!(before.0 == s.gfx.canvas.px, "the picture is back");
        assert_eq!((s.gfx.view, s.gfx.kit.device, s.policy), (before.1, before.2, before.3));
        assert_eq!((s.gfx.canvas.w, s.gfx.canvas.h, s.gfx.fonts.scale, s.mouse), (1920, 1080, 1.15, (321, 123)));
        assert_eq!((s.gfx.fonts.audit.is_none(), s.gfx.fonts.layout.is_none()), (true, true), "no audit stays on");
    }

    #[test]
    fn one_profile_runs_alone_and_a_wrong_name_says_what_there_is() {
        let mut s = window(1280, 720, vec![specimen()]);
        let out = s.bridge(&json!({"cmd": "layout_audit", "profile": "phone_portrait"}));
        assert_eq!(out["profiles"].as_array().unwrap().len(), 1, "{out}");
        assert_eq!((out["profiles"][0]["canvas"].clone(), out["profiles"][0]["must_pass"].clone(), out["pass"].clone()), (json!([390, 844]), json!(false), json!(true)), "reports only: {out}");
        let one = s.bridge(&json!({"cmd": "text_audit", "profile": "ultrawide", "scale": 1.3}));
        assert_eq!((one["ok"].clone(), one["layout"]["canvas"].clone(), one["scale"].clone()), (json!(true), json!([1720, 720]), json!(1.3f32)), "{one}");
        assert_eq!((s.gfx.canvas.w, s.gfx.canvas.h, s.gfx.view), (1280, 720, View::fit(1280, 720).for_grid(Grid::Native)));
        for cmd in ["layout_audit", "text_audit"] {
            let out = s.bridge(&json!({"cmd": cmd, "profile": "watch"}));
            assert_eq!(out["ok"], json!(false), "{out}");
            assert!(out["error"].as_str().unwrap().contains("desktop"), "{out}");
        }
    }

    /// The manifest's list holds the saved size; with no such key the game's own list stands.
    #[test]
    fn only_a_declared_list_replaces_the_games_own() {
        let game = ["1280x720", "1920x1080"];
        let settings = || {
            let mut s = Settings::load(std::env::temp_dir().join("aegis_no_settings.json"), json!({}));
            s.keep_window_size(&game);
            s
        };
        let hold = |declared: Value| {
            let mut s = settings();
            s.v["window_size"] = json!("2560x1440");
            let (_, offered, notices) = read_window_options(&Value::Null, &declared);
            hold_declared_sizes(&mut s, &declared, &offered);
            (s.window_sizes().to_vec(), s.v["window_size"].clone(), notices.len())
        };
        assert_eq!(hold(Value::Null), (game.map(String::from).to_vec(), json!("2560x1440"), 0), "absent: nothing is touched");
        assert_eq!(hold(json!(["1366x768"])), (vec!["1366x768".to_string()], json!("1366x768"), 0), "declared");
        assert_eq!(hold(json!(7)), (WINDOW_SIZES.map(String::from).to_vec(), json!("2560x1440"), 1), "refused: a notice and the default");
    }

    /// A widget a profile's canvas has no room for must not take the focus or an open list with it for good.
    #[test]
    fn a_profile_leaves_the_focus_where_it_was() {
        struct Shrinks {
            w: Vec<Widget<()>>,
        }
        impl Shrinks {
            fn act(&mut self, _: &mut Cx, _: ()) {}
        }
        impl Screen<App> for Shrinks {
            fn id(&self) -> &'static str {
                "shrinks"
            }
            fn grid(&self) -> Grid {
                Grid::Native
            }
            fn draw(&mut self, _: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
                let mut ui: Ui<()> = Ui::new(g, mouse, live);
                ui.row("always", Rect::new(0, 0, 100, 40), "Always", TEXT, Some(()), false, Vec::new());
                if ui.screen().w >= 1000 {
                    ui.row("wide.only", Rect::new(0, 50, 100, 40), "Wide", TEXT, Some(()), false, Vec::new());
                }
                self.w = ui.widgets;
            }
            crate::widget_plumbing!(Cx);
        }
        let mut s = window(1280, 720, vec![Box::new(Shrinks { w: Vec::new() })]);
        s.gfx.kit.focus = Some("wide.only".into());
        s.gfx.kit.focus_visible = true;
        let keep = |s: &Shell<App>| (s.gfx.kit.focus.clone(), s.gfx.kit.focus_visible);
        let was = keep(&s);
        for cmd in [json!({"cmd": "layout_audit"}), json!({"cmd": "layout_audit", "profile": "phone_portrait"}), json!({"cmd": "text_audit", "profile": "phone_portrait"})] {
            s.bridge(&cmd);
            assert_eq!(keep(&s), was, "{cmd}");
        }
        // Moved by the frames, as `settle_focus` does when the widget is gone: the profile's own leaving puts it back.
        let mut g = Gfx { canvas: Canvas::new(1280, 720), view: s.gfx.view, fonts: Fonts::new(&std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()), kit: Kit::default() };
        g.kit.open = Some("a list".into());
        g.kit.focus = Some("wide.only".into());
        let entered = layout_audit::profile_named("phone_portrait").unwrap().enter(&mut g, CanvasPolicy::default());
        g.kit.open = None;
        g.kit.focus = Some("always".into());
        entered.leave(&mut g);
        assert_eq!((g.kit.open.as_deref(), g.kit.focus.as_deref()), (Some("a list"), Some("wide.only")));
    }

    /// The text audit reads the top screen's canvas: a text inside the grid is no problem on a native screen and past the
    /// edge on an old one.
    #[test]
    fn the_text_audit_measures_against_the_top_screens_canvas() {
        for (grid, edges) in [(Grid::Native, 0), (Grid::Old, 1), (Grid::Quadrant, 1)] {
            let mut top = on_grid("top", grid, false);
            top.far_text = true;
            let mut s = window(1280, 720, vec![top]);
            let out = s.bridge(&json!({"cmd": "text_audit"}));
            assert_eq!(out["edge"].as_array().unwrap().len(), edges, "{grid:?}: {}", out["edge"]);
        }
    }

    /// What the shell draws over the top screen, at twice the numbers on the native grid: the ring, the outlines and the
    /// lists are the old picture to the pixel. The ghost sizes itself from measured text, rounded up in units, so its card
    /// may end up to a unit away (`shot_diff`, spec 6.3).
    #[test]
    fn the_ring_the_lists_and_the_ghost_are_the_old_picture_on_the_native_grid() {
        let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        for (pw, ph) in [(1280u32, 720u32), (1920, 1080), (1366, 768)] {
            let shot = |g: Grid, what: &dyn Fn(&mut Painter, i32)| {
                let mut c = Canvas::new(pw as i32, ph as i32);
                let mut fonts = Fonts::new(&font);
                what(&mut Painter { c: &mut c, v: View::fit(pw, ph).for_grid(g), fonts: &mut fonts }, g.g());
                Image { w: c.w, h: c.h, px: c.px }
            };
            let exact = |p: &mut Painter, u: i32| {
                draw_ring(p, Rect::new(30 * u, 30 * u, 50 * u, 20 * u));
                draw_drop_target(p, Rect::new(100 * u, 30 * u, 50 * u, 20 * u), true);
                draw_drop_target(p, Rect::new(160 * u, 30 * u, 50 * u, 20 * u), false);
                let opts = [
                    json!({"id": "a", "text": "First", "active": true, "rect": [30 * u, 200 * u, 90 * u, 14 * u]}),
                    json!({"id": "b", "text": "Second, a longer one than fits its row", "active": false, "rect": [30 * u, 214 * u, 90 * u, 14 * u]}),
                ];
                draw_options(p, &opts.iter().collect::<Vec<_>>(), (40 * u, 205 * u), Some("b"));
            };
            assert!(shot(Grid::Old, &exact).px == shot(Grid::Native, &exact).px, "{pw}x{ph}");
            // Near the middle, then held inside the canvas's far corner, on either grid.
            let ghost = |p: &mut Painter, u: i32| {
                draw_ghost(p, "A dragged thing", (300 * u, 100 * u));
                draw_ghost(p, "In the corner", (639 * u, 359 * u));
            };
            let (old, native) = (shot(Grid::Old, &ghost), shot(Grid::Native, &ghost));
            assert!(old.px != vec![0; old.px.len()]);
            let diff = crate::shot_diff::shot_diff(&old, &native, View::fit(pw, ph).for_grid(Grid::Native).s).unwrap();
            assert!(diff.passed(), "{pw}x{ph}: {diff}");
        }
    }
}
