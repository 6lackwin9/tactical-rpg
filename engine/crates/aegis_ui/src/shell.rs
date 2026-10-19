//! The window shell: one window over the screen stack; the logical canvas is 640x360, pixel art scales with nearest
//! sampling and the UI is drawn smooth at native resolution. The agent link (TCP, `aegis link`) sees and drives
//! exactly what the player sees: ui_tree, click, drag, hover, key, back, screenshot, game, plus whatever the top screen
//! answers (`Screen::command`). Keys: arrows, Enter, Space, Esc, Tab, Page Up / Down and characters go to the top
//! screen; F12 takes a screenshot; right click goes back.

use crate::bridge;
use crate::canvas::{Canvas, Image};
use crate::painter::{Fonts, Painter, View};
use crate::screen::{Context, Cx, Key, Nav, Notice, Screen};
use crate::canvas::{rgb, rgba, Rect};
use crate::ui::{Drag, Gfx, Kit, BG, BODY, GOLD, H, MODAL, PANEL_HI, RADIUS, TEXT, W};
use anyhow::Result;
use pixels::{Pixels, SurfaceTexture};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key as WKey, NamedKey};
use winit::dpi::LogicalSize;
use winit::window::{Fullscreen, Window, WindowId};

/// The window the shell opens.
pub struct Options {
    pub title: String,
    /// The UI font (a TTF file).
    pub font: PathBuf,
    /// The agent link's TCP port.
    pub port: u16,
}

struct Shell<G> {
    cx: Cx<G>,
    gfx: Gfx,
    stack: Vec<Box<dyn Screen<G>>>,
    mouse: (i32, i32),
    /// The slider being dragged (its id) and the step widget it last set.
    drag: Option<(String, String)>,
    /// A press on a draggable widget (UIKIT2), waiting to become a drag (a move) or a click (a release).
    press: Option<Press>,
}

/// A press on a draggable widget: its id, payload and text, and where the press was.
struct Press {
    id: String,
    key: String,
    text: String,
    at: (i32, i32),
}

/// How far (canvas pixels) the mouse moves from the press before a draggable widget is picked up.
const DRAG_START: i32 = 3;

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

impl<G: Context> Shell<G> {
    /// Applies queued navigation and notices (a screen change closes an open choice list).
    fn settle(&mut self) {
        let mut guard = 0;
        while (!self.cx.nav.is_empty() || !self.cx.notice.is_empty()) && guard < 32 {
            guard += 1;
            self.gfx.kit.open = None;
            self.drag = None;
            self.press = None;
            self.gfx.kit.drag = None;
            let navs: Vec<Nav<G>> = self.cx.nav.drain(..).collect();
            for n in navs {
                match n {
                    Nav::Push(s) => self.stack.push(s),
                    Nav::Pop => {
                        if self.stack.len() > 1 {
                            self.stack.pop();
                            let cx = &mut self.cx;
                            self.stack.last_mut().unwrap().resume(cx);
                        }
                    }
                    Nav::Reset(s) => {
                        self.stack.clear();
                        self.stack.push(s);
                    }
                }
            }
            if !self.cx.notice.is_empty() {
                let lines: Vec<String> = self.cx.notice.drain(..).collect();
                self.stack.push(Box::new(Notice::new(lines)));
            }
        }
    }

    fn frame(&mut self) {
        self.settle();
        // The saved sound levels; a screen may play draft ones for this frame (the settings popup).
        self.cx.audio.set_levels(&self.cx.settings.v);
        let cx = &mut self.cx;
        self.stack.last_mut().unwrap().tick(cx);
        self.settle();
        self.gfx.canvas.clear(BG);
        let mut base = self.stack.len() - 1;
        while base > 0 && self.stack[base].overlay() {
            base -= 1;
        }
        let n = self.stack.len();
        for i in base..n {
            let live = i == n - 1;
            let mouse = if live { self.mouse } else { (-1, -1) };
            self.stack[i].draw(&mut self.cx, &mut self.gfx, mouse, live);
        }
        let mut p = Painter { c: &mut self.gfx.canvas, v: self.gfx.view, fonts: &mut self.gfx.fonts };
        if self.gfx.kit.open.is_some() {
            // The open choice list goes over everything the screen drew; no hover card while it is open.
            let infos = self.stack[n - 1].widgets();
            let opts: Vec<&Value> = infos.iter().filter(|w| w["kind"] == "option").collect();
            if opts.is_empty() {
                self.gfx.kit.open = None;
            } else {
                draw_options(&mut p, &opts, self.mouse);
            }
        } else if let Some(d) = &self.gfx.kit.drag {
            // Dragging: valid targets outlined (the one under the cursor in gold) and the ghost; no hover card.
            let top = &self.stack[n - 1];
            for w in top.widgets().iter().filter(|w| w["enabled"] == json!(true) && w["drop"].is_string()) {
                if top.can_drop(&self.cx, &d.key, w["drop"].as_str().unwrap_or("")) {
                    let r = info_rect(w);
                    let over = r.contains(self.mouse.0, self.mouse.1);
                    p.panel(r, if over { rgba(244, 198, 84, 40) } else { rgba(0, 0, 0, 0) }, if over { GOLD } else { rgb(110, 140, 196) }, RADIUS);
                }
            }
            draw_ghost(&mut p, &d.text, self.mouse);
        } else {
            self.stack[n - 1].draw_tip(&mut p, self.mouse);
        }
        self.settle();
        let cx = &self.cx;
        let track = self.stack.iter().rev().find_map(|s| s.music(cx));
        cx.audio.music(track.as_deref());
        cx.audio.tick();
    }

    /// Drops a press or drag in progress; true when there was one.
    fn cancel_drag(&mut self) -> bool {
        let had = self.press.take().is_some();
        self.gfx.kit.drag.take().is_some() || had
    }

    fn key(&mut self, k: Key) {
        // Any key drops a press or drag in progress (what it changes could move the target); Esc does only that.
        let cancelled = self.cancel_drag();
        if k == Key::Esc && (cancelled || self.gfx.kit.open.take().is_some()) {
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

    /// A left click: opens or closes a choice list, picks from it, starts a slider drag, or clicks the widget.
    fn click(&mut self) {
        // A press or drag whose release was lost (focus moved away mid-drag) ends here.
        self.cancel_drag();
        let pos = self.mouse;
        let infos = self.stack.last().unwrap().widgets();
        let at = info_at(&infos, pos).cloned();
        let kind = at.as_ref().map(|w| w["kind"].as_str().unwrap_or("").to_string()).unwrap_or_default();
        if self.gfx.kit.open.take().is_some() {
            // An open list: a pick goes through, any other click only closes it.
            if kind == "option" {
                let cx = &mut self.cx;
                self.stack.last_mut().unwrap().click(cx, pos);
            }
            self.settle();
            return;
        }
        let id = at.as_ref().map(|w| w["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
        if kind == "select" {
            self.gfx.kit.open = Some(id);
            return;
        }
        if let Some(key) = at.as_ref().and_then(|w| w["drag"].as_str()) {
            // A draggable widget: a move picks it up (`drag_to`), a release without one clicks it (`release`).
            let text = at.as_ref().and_then(|w| w["text"].as_str()).unwrap_or("").to_string();
            self.press = Some(Press { id, key: key.to_string(), text, at: pos });
            return;
        }
        if kind == "slider_step" {
            if let Some((slider, _)) = id.rsplit_once(".step.") {
                self.drag = Some((slider.to_string(), id.clone()));
            }
        }
        let cx = &mut self.cx;
        self.stack.last_mut().unwrap().click(cx, pos);
        self.settle();
    }

    /// The mouse moved: a press on a draggable widget becomes a drag once it moved DRAG_START pixels; a slider drag
    /// follows it along the track (the step under its x).
    fn drag_to(&mut self) {
        if let Some(p) = &self.press {
            if self.gfx.kit.drag.is_none() && ((self.mouse.0 - p.at.0).abs() > DRAG_START || (self.mouse.1 - p.at.1).abs() > DRAG_START) {
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
        let hit = infos.iter().rev().find(|w| w["enabled"] == json!(true) && w["drop"].is_string() && info_rect(w).contains(self.mouse.0, self.mouse.1))?;
        let to = hit["drop"].as_str()?;
        top.can_drop(&self.cx, from, to).then(|| to.to_string())
    }

    fn back(&mut self) {
        if self.cancel_drag() || self.gfx.kit.open.take().is_some() {
            return;
        }
        let cx = &mut self.cx;
        self.stack.last_mut().unwrap().back(cx);
        self.settle();
    }

    fn shot(&mut self, path: &Path, max_w: i32) -> Result<()> {
        save_png(&self.gfx.canvas, path, max_w)
    }

    /// Agent link requests.
    fn bridge(&mut self, body: &Value) -> Value {
        let cmd = body.get("cmd").and_then(Value::as_str).unwrap_or("");
        let reply = match cmd {
            "ui_tree" => {
                let cx = &self.cx;
                let top = self.stack.last().unwrap();
                json!({"ok": true, "screen": top.id(), "stack": self.stack.iter().map(|s| s.id()).collect::<Vec<_>>(), "canvas": [W, H], "mouse": [self.mouse.0, self.mouse.1],
                    "music": cx.audio.wanted_music(), "state": top.state(cx), "game": cx.app.agent_summary(), "widgets": top.widgets()})
            }
            "click" => {
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                let infos = self.stack.last().unwrap().widgets();
                let target = infos.iter().find(|w| w["id"] == id).cloned();
                if target.as_ref().is_some_and(|w| w["kind"] == "select" && w["enabled"] == json!(true)) {
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
            "hover" => {
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                match self.stack.last().unwrap().widget_rect(id) {
                    Some((r, tip)) => {
                        self.mouse = (r.x + r.w / 2, r.y + r.h / 2);
                        json!({"ok": true, "tip": tip})
                    }
                    None => json!({"ok": false, "error": format!("no widget '{id}'")}),
                }
            }
            "key" => match body.get("key").and_then(Value::as_str).and_then(Key::parse) {
                Some(k) => {
                    self.key(k);
                    json!({"ok": true})
                }
                None => json!({"ok": false, "error": "key needs one of up, down, left, right, enter, space, esc, tab, pageup, pagedown or a character"}),
            },
            "back" => {
                self.back();
                json!({"ok": true})
            }
            "screenshot" => {
                let path = body.get("path").and_then(Value::as_str).map(PathBuf::from).unwrap_or_else(|| self.cx.shot_dir.join("gui_shot.png"));
                let max_w = body.get("max_w").and_then(Value::as_u64).unwrap_or(1280) as i32;
                match self.shot(&path, max_w) {
                    Ok(()) => json!({"ok": true, "path": path.display().to_string()}),
                    Err(e) => json!({"ok": false, "error": e.to_string()}),
                }
            }
            "game" => self.cx.app.agent_game(),
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
}

impl<G: Context> App<G> {
    fn redraw(&mut self) {
        let Some(shell) = self.shell.as_mut() else { return };
        if let Some(rx) = &self.bridge {
            while let Ok(req) = rx.try_recv() {
                let reply = shell.bridge(&req.body);
                let _ = req.reply.send(reply);
            }
        }
        shell.frame();
        if shell.cx.quit {
            std::process::exit(0);
        }
        self.apply_settings();
        let Some(shell) = self.shell.as_mut() else { return };
        if let Some(p) = self.pixels.as_mut() {
            if p.frame().len() == shell.gfx.canvas.px.len() {
                p.frame_mut().copy_from_slice(&shell.gfx.canvas.px);
                let _ = p.render();
            }
        }
    }

    /// Applies the engine's settings (DisplaySettings) when they changed since the last time: window mode (windowed
    /// at the chosen size, borderless fullscreen, exclusive fullscreen at the monitor's best mode), vsync, the
    /// frame-rate cap and the text size. The window size is set only when windowed and only when the mode or the size
    /// setting changed, so a window the player resized keeps its size.
    fn apply_settings(&mut self) {
        let (Some(shell), Some(window)) = (self.shell.as_mut(), self.window.as_ref()) else { return };
        let v = shell.cx.settings.v.clone();
        if self.applied.as_ref() == Some(&v) {
            return;
        }
        let was = |k: &str| self.applied.as_ref().map(|a| a[k].clone());
        let mode = v["window_mode"].as_str().unwrap_or("Windowed");
        if was("window_mode").as_ref() != Some(&v["window_mode"]) || was("window_size").as_ref() != Some(&v["window_size"]) {
            match mode {
                "Borderless" => window.set_fullscreen(Some(Fullscreen::Borderless(None))),
                "Fullscreen" => {
                    let best = window.current_monitor().and_then(|m| m.video_modes().max_by_key(|vm| (vm.size().width * vm.size().height, vm.refresh_rate_millihertz())));
                    window.set_fullscreen(Some(best.map(Fullscreen::Exclusive).unwrap_or(Fullscreen::Borderless(None))));
                }
                _ => {
                    window.set_fullscreen(None);
                    if let Some((w, h)) = v["window_size"].as_str().and_then(|s| s.split_once('x')).and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))) {
                        let _ = window.request_inner_size(LogicalSize::new(w, h));
                    }
                }
            }
        }
        if let Some(p) = self.pixels.as_mut() {
            p.enable_vsync(v["vsync"].as_bool().unwrap_or(true));
        }
        self.frame_every = v["fps_cap"].as_str().and_then(|c| c.parse::<u64>().ok()).filter(|n| *n > 0).map(|n| Duration::from_micros(1_000_000 / n));
        shell.gfx.fonts.scale = v["text_scale"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0) as f32;
        self.applied = Some(v);
    }
}

impl<G: Context> ApplicationHandler for App<G> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title(self.options.title.as_str()).with_inner_size(winit::dpi::LogicalSize::new(1280, 720)).with_min_inner_size(winit::dpi::LogicalSize::new(640, 360));
        let window = Arc::new(el.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let (pw, ph) = (size.width.max(1), size.height.max(1));
        let pixels = Pixels::new(pw, ph, SurfaceTexture::new(pw, ph, window.clone())).expect("pixels");
        let font_bytes = std::fs::read(&self.options.font).unwrap_or_else(|e| panic!("{}: {e}", self.options.font.display()));
        let (cx, stack) = self.start.take().expect("the shell starts once");
        let gfx = Gfx { canvas: Canvas::new(pw as i32, ph as i32), view: View::fit(pw, ph), fonts: Fonts::new(&font_bytes), kit: Kit::default() };
        let mut shell = Shell { cx, gfx, stack, mouse: (0, 0), drag: None, press: None };
        let port = self.options.port;
        self.bridge = bridge::start(port, self.proxy.clone());
        if self.bridge.is_none() {
            shell.cx.notice.push(format!("Agent link: port {port} is busy."));
        }
        self.shell = Some(shell);
        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn user_event(&mut self, _el: &ActiveEventLoop, _e: ()) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        // The frame-rate cap paces the redraws; unlimited redraws as fast as presenting allows (vsync waits for the screen).
        el.set_control_flow(match self.frame_every {
            Some(every) => ControlFlow::WaitUntil(Instant::now() + every),
            None => ControlFlow::Poll,
        });
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(size) => {
                let (w, h) = (size.width.max(1), size.height.max(1));
                if let Some(p) = self.pixels.as_mut() {
                    let _ = p.resize_surface(w, h);
                    let _ = p.resize_buffer(w, h);
                }
                if let Some(s) = self.shell.as_mut() {
                    s.gfx.canvas = Canvas::new(w as i32, h as i32);
                    s.gfx.view = View::fit(w, h);
                }
            }
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Focused(false) => {
                // The release may never arrive (alt-tab mid-drag): drop the drag.
                if let Some(s) = self.shell.as_mut() {
                    s.cancel_drag();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    s.mouse = s.gfx.view.to_logical(position.x as f32, position.y as f32);
                    s.drag_to();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    let lines = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y.round() as i32,
                        MouseScrollDelta::PixelDelta(p) => (p.y / 20.0).round() as i32,
                    };
                    let cx = &mut s.cx;
                    s.stack.last_mut().unwrap().wheel(cx, lines);
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
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. }, .. } => {
                let Some(s) = self.shell.as_mut() else { return };
                let k = match logical_key {
                    WKey::Named(NamedKey::ArrowUp) => Some(Key::Up),
                    WKey::Named(NamedKey::ArrowDown) => Some(Key::Down),
                    WKey::Named(NamedKey::ArrowLeft) => Some(Key::Left),
                    WKey::Named(NamedKey::ArrowRight) => Some(Key::Right),
                    WKey::Named(NamedKey::Enter) => Some(Key::Enter),
                    WKey::Named(NamedKey::Space) => Some(Key::Space),
                    WKey::Named(NamedKey::Escape) => Some(Key::Esc),
                    WKey::Named(NamedKey::Tab) => Some(Key::Tab),
                    WKey::Named(NamedKey::PageUp) => Some(Key::PageUp),
                    WKey::Named(NamedKey::PageDown) => Some(Key::PageDown),
                    WKey::Named(NamedKey::F12) => {
                        let p = s.cx.shot_dir.join("gui_shot.png");
                        let msg = match s.shot(&p, 0) {
                            Ok(()) => format!("Screenshot: {}", p.display()),
                            Err(e) => format!("Screenshot failed: {e}"),
                        };
                        s.cx.app.note(msg);
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

/// The dragged widget's text in a small card under the cursor.
fn draw_ghost(p: &mut Painter, text: &str, mouse: (i32, i32)) {
    let w = (p.text_w(text, BODY).ceil() as i32 + 10).clamp(24, 160);
    let h = p.line_h(BODY).ceil() as i32 + 4;
    let r = Rect::new((mouse.0 + 6).min(W - w - 2), (mouse.1 + 6).min(H - h - 2), w, h);
    p.panel(r, rgba(18, 26, 44, 220), GOLD, RADIUS);
    p.text_fit(r.x as f32 + 5.0, r.y as f32 + 2.0, text, BODY, (w - 10) as f32, TEXT);
}

/// An open select's choice list, over the screen: one row per option, the chosen one in gold, the hovered one lit.
fn draw_options(p: &mut Painter, opts: &[&Value], mouse: (i32, i32)) {
    let rects: Vec<Rect> = opts.iter().map(|w| info_rect(w)).collect();
    let (x0, y0) = (rects.iter().map(|r| r.x).min().unwrap_or(0), rects.iter().map(|r| r.y).min().unwrap_or(0));
    let (x1, y1) = (rects.iter().map(|r| r.x + r.w).max().unwrap_or(0), rects.iter().map(|r| r.y + r.h).max().unwrap_or(0));
    p.panel(Rect::new(x0 - 1, y0 - 1, x1 - x0 + 2, y1 - y0 + 2), MODAL, GOLD, RADIUS);
    for (w, r) in opts.iter().zip(&rects) {
        if r.contains(mouse.0, mouse.1) {
            p.panel(*r, PANEL_HI, rgb(110, 140, 196), RADIUS);
        }
        let chosen = w["active"] == json!(true);
        p.text_fit(r.x as f32 + 5.0, r.y as f32 + (r.h as f32 - p.line_h(BODY)) / 2.0, w["text"].as_str().unwrap_or(""), BODY, r.w as f32 - 10.0, if chosen { GOLD } else { TEXT });
    }
}

/// Saves the canvas as PNG; `max_w` > 0 shrinks it (box filter) to at most that width.
fn save_png(c: &Canvas, path: &Path, max_w: i32) -> Result<()> {
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

/// Opens the window and runs the screens until the window closes or a screen sets `cx.quit`.
pub fn run<G: Context>(cx: Cx<G>, stack: Vec<Box<dyn Screen<G>>>, options: Options) -> Result<()> {
    let el: EventLoop<()> = EventLoop::with_user_event().build()?;
    let proxy = el.create_proxy();
    let mut app = App { start: Some((cx, stack)), shell: None, options, window: None, pixels: None, proxy, bridge: None, applied: None, frame_every: Some(Duration::from_millis(16)) };
    el.run_app(&mut app)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::Art;
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
            app: App::default(),
        };
        let gfx = Gfx { canvas: Canvas::new(W, H), view: View::fit(W as u32, H as u32), fonts: Fonts::new(&font), kit: Kit::default() };
        let mut s = Shell { cx, gfx, stack: vec![Box::new(Board::default())], mouse: (0, 0), drag: None, press: None };
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
}
