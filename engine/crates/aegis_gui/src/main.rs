//! aegis-gui: the game's screens on the Aegis engine. A stack of screens (main menu, new game, slots, settings, region
//! map with dialogue and shops, world map, game menu, character, journal, battle results, battle) over one window; the
//! logical canvas is 640x360, pixel art scales with nearest sampling and the UI is drawn smooth at native resolution.
//! The agent link (TCP, `aegis link`) sees and drives exactly what the player sees: ui_tree, click, hover, key,
//! screenshot, game, plus the battle command API while a battle is open.
//!   aegis-gui [screen=menu|arena|runner] [encounter=goblin_skirmish] [seed=N] [port=47800] [data=<export dir>] [saves=<dir>]
//! Keys: arrows / W A S D move, E interact, Esc back or game menu, Enter confirm, F12 screenshot; battle keys as listed
//! in battle.rs. Right click goes back.

mod art;
mod battle;
mod bridge;
mod canvas;
mod dojo;
mod forge;
mod hub;
mod menus;
mod painter;
mod party;
mod region;
mod runner;
mod screens;
mod tuning;
mod ui;
mod worldmap;

use anyhow::{Context, Result};
use canvas::{Canvas, Image};
use painter::{Fonts, Painter, View};
use pixels::{Pixels, SurfaceTexture};
use screens::{Cx, Key, Nav, Notice, Screen, Settings};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use ui::{Gfx, BG, H, W};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key as WKey, NamedKey};
use winit::window::{Window, WindowId};

struct Shell {
    cx: Cx,
    gfx: Gfx,
    stack: Vec<Box<dyn Screen>>,
    mouse: (i32, i32),
}

impl Shell {
    /// Applies queued navigation and notices.
    fn settle(&mut self) {
        let mut guard = 0;
        while (!self.cx.nav.is_empty() || !self.cx.notice.is_empty()) && guard < 32 {
            guard += 1;
            let navs: Vec<Nav> = self.cx.nav.drain(..).collect();
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
        self.stack[n - 1].draw_tip(&mut p, self.mouse);
        self.settle();
    }

    fn key(&mut self, k: Key) {
        let cx = &mut self.cx;
        let top = self.stack.last_mut().unwrap();
        if k == Key::Esc {
            top.escape(cx);
        } else {
            top.key(cx, k);
        }
        self.settle();
    }

    fn click(&mut self) {
        let (cx, pos) = (&mut self.cx, self.mouse);
        self.stack.last_mut().unwrap().click(cx, pos);
        self.settle();
    }

    fn back(&mut self) {
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
                let game = cx.game.as_ref().map(|g| {
                    json!({"world": g.gs.world_id, "location": g.gs.location_id, "position": [g.gs.position.0, g.gs.position.1], "gold": g.gs.party.gold,
                        "supplies": g.gs.party.supplies, "time": g.gs.time, "dirty": cx.is_dirty(), "busy": g.busy})
                });
                json!({"ok": true, "screen": top.id(), "stack": self.stack.iter().map(|s| s.id()).collect::<Vec<_>>(), "canvas": [W, H], "mouse": [self.mouse.0, self.mouse.1],
                    "state": top.state(cx), "game": game, "widgets": top.widgets()})
            }
            "click" => {
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                let cx = &mut self.cx;
                match self.stack.last_mut().unwrap().click_id(cx, id) {
                    Ok(()) => json!({"ok": true, "clicked": id}),
                    Err(e) => json!({"ok": false, "error": e}),
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
            "game" => match &self.cx.game {
                Some(g) => json!({"ok": true, "state": g.gs.to_value()}),
                None => json!({"ok": false, "error": "no game in progress"}),
            },
            "_bad" => json!({"ok": false, "error": body["error"]}),
            _ => {
                let cx = &self.cx;
                match self.stack.last_mut().unwrap().as_battle() {
                    Some(b) => {
                        let reply = b.host.handle(body);
                        if b.host.session.is_some() {
                            b.after_change(cx);
                        }
                        reply
                    }
                    None => json!({"ok": false, "error": format!("unknown command '{cmd}' (battle commands work while a battle is open)")}),
                }
            }
        };
        self.settle();
        reply
    }
}

struct App {
    shell: Option<Shell>,
    args: Vec<(String, String)>,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    proxy: EventLoopProxy<()>,
    bridge: Option<Receiver<bridge::Request>>,
    engine_dir: Option<PathBuf>,
    project: Option<PathBuf>,
    data: Arc<aegis_world::Data>,
    data_dir: PathBuf,
}

fn arg<'a>(args: &'a [(String, String)], k: &str) -> Option<&'a str> {
    args.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
}

fn app_dir() -> PathBuf {
    std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(".")).join("Aegis")
}

impl App {
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
        if let Some(p) = self.pixels.as_mut() {
            if p.frame().len() == shell.gfx.canvas.px.len() {
                p.frame_mut().copy_from_slice(&shell.gfx.canvas.px);
                let _ = p.render();
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("Aegis - Tactical RPG").with_inner_size(winit::dpi::LogicalSize::new(1280, 720)).with_min_inner_size(winit::dpi::LogicalSize::new(640, 360));
        let window = Arc::new(el.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let (pw, ph) = (size.width.max(1), size.height.max(1));
        let pixels = Pixels::new(pw, ph, SurfaceTexture::new(pw, ph, window.clone())).expect("pixels");
        let engine = self.engine_dir.clone().unwrap_or_else(|| PathBuf::from("."));
        let font_bytes = std::fs::read(engine.join("assets/fonts/OpenSans-SemiBold.ttf")).expect("engine/assets/fonts/OpenSans-SemiBold.ttf");
        let save_dir = arg(&self.args, "saves").map(PathBuf::from).unwrap_or_else(|| app_dir().join("saves"));
        let cx = Cx {
            data: self.data.clone(),
            game: None,
            save_dir,
            settings: Settings::load(app_dir().join("settings.json")),
            art: art::Art::new(self.project.clone()),
            tuning: tuning::TuningFile::new(Some(engine.join("data/tuning.json"))),
            log: Vec::new(),
            nav: Vec::new(),
            notice: Vec::new(),
            shot_dir: engine.join("parity/out"),
            quit: false,
            encounter: arg(&self.args, "encounter").unwrap_or("goblin_skirmish").to_string(),
            seed: arg(&self.args, "seed").and_then(|s| s.parse().ok()),
            answers: Vec::new(),
            debug: false,
            data_dir: self.data_dir.clone(),
        };
        let gfx = Gfx { canvas: Canvas::new(pw as i32, ph as i32), view: View::fit(pw, ph), fonts: Fonts::new(&font_bytes) };
        let mut shell = Shell { cx, gfx, stack: vec![Box::new(menus::MainMenu::default())], mouse: (0, 0) };
        match arg(&self.args, "screen") {
            Some("arena") => {
                let b = battle::BattleScreen::sandbox(&shell.cx);
                shell.stack.push(Box::new(b));
            }
            Some("runner") => shell.stack.push(Box::new(runner::SceneRunner::new())),
            _ => {}
        }
        let port = arg(&self.args, "port").and_then(|p| p.parse().ok()).unwrap_or(bridge::DEFAULT_PORT);
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
        el.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16)));
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
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    s.mouse = s.gfx.view.to_logical(position.x as f32, position.y as f32);
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
                        s.cx.log(msg);
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

fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<(String, String)> = raw.iter().filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
    let cwd = std::env::current_dir()?;
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    let engine_dir = [Some(cwd.clone()), exe_dir].into_iter().flatten().find_map(|start| {
        let mut d = start;
        loop {
            if d.join("crates/aegis_gui").is_dir() {
                return Some(d);
            }
            if d.join("engine/crates/aegis_gui").is_dir() {
                return Some(d.join("engine"));
            }
            if !d.pop() {
                return None;
            }
        }
    });
    let data_dir = arg(&args, "data").map(PathBuf::from).or_else(|| engine_dir.as_ref().map(|d| d.join("parity/out"))).context("export folder not found: pass data=<dir>")?;
    let data = Arc::new(aegis_world::Data::load(&data_dir).with_context(|| format!("loading the Godot exports in {} (run engine/parity/export.sh)", data_dir.display()))?);
    let project = art::find_project(engine_dir.as_deref().unwrap_or(&cwd));
    let el: EventLoop<()> = EventLoop::with_user_event().build()?;
    let proxy = el.create_proxy();
    let mut app = App { shell: None, args, window: None, pixels: None, proxy, bridge: None, engine_dir, project, data, data_dir };
    el.run_app(&mut app)?;
    Ok(())
}
