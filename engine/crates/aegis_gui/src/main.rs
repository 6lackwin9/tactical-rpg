//! aegis-gui: the battle screen (COMBAT-UI3 layout A) on a 640x360 pixel canvas, integer-scaled to the window.
//! Every change goes through the command API (aegis_host), the same one `aegis serve` and the agent link use, so
//! what the user clicks and what an agent sends are the same requests. Feel values come from engine/data/tuning.json.
//!   aegis-gui [encounter=goblin_skirmish] [seed=N] [port=47800] [catalog=<path>]
//! Mouse: click a hero to select; click a foe, ally, the hero itself or a tile for its options; right click closes a
//! menu or undoes the hero's last step. Keys: Enter confirm, Space step / next round, A autoplay, Z undo, L log,
//! R restart, F12 screenshot (engine/parity/out/gui_shot.png), Esc close.

mod art;
mod bridge;
mod canvas;
mod font;
mod tuning;

use aegis_battle::command::CmdKind;
use aegis_battle::grid::{V2, GRID_H, GRID_W};
use aegis_battle::plan::expected;
use aegis_battle::session::Phase;
use aegis_data::Catalog;
use aegis_host::Host;
use anyhow::{Context, Result};
use canvas::{rgb, rgba, Canvas, Rect, Rgba};
use pixels::{Pixels, SurfaceTexture};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const W: i32 = 640;
const H: i32 = 360;
const TILE: i32 = 32;
const GRID_X: i32 = 128;
const GRID_Y: i32 = 30;
const TACTICS: [&str; 6] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy"];
const TACTIC_LABELS: [&str; 6] = ["Balanced", "All-out", "Weakest", "Guard", "Kite", "Economy"];

// Palette (dark slate UI, team colours).
const BG: Rgba = rgb(12, 16, 26);
const PANEL: Rgba = rgb(20, 28, 46);
const PANEL_HI: Rgba = rgb(30, 42, 68);
const BORDER: Rgba = rgb(52, 70, 104);
const TEXT: Rgba = rgb(214, 222, 236);
const DIM: Rgba = rgb(128, 142, 168);
const GOLD: Rgba = rgb(242, 196, 82);
const ALLY: Rgba = rgb(92, 160, 255);
const FOE: Rgba = rgb(238, 92, 86);
const GOOD: Rgba = rgb(96, 210, 110);

#[derive(Debug, Clone, PartialEq)]
enum Action {
    Select(usize),
    UnitClick(usize),
    TileClick(V2),
    Choose(usize),
    ClosePopup,
    Confirm,
    Clear,
    Undo,
    Autoplay,
    PartyTactic,
    PartyCycle,
    HeroTactic,
    HeroCycle,
    NextRound,
    Step,
    Restart,
    ToggleLog,
}

#[derive(Debug, Clone)]
struct Widget {
    id: String,
    kind: &'static str,
    text: String,
    rect: Rect,
    enabled: bool,
    active: bool,
    action: Option<Action>,
}

#[derive(Debug, Clone)]
struct Row {
    label: String,
    detail: String,
    valid: bool,
    step: Value,
}

#[derive(Debug, Clone)]
struct Popup {
    title: String,
    x: i32,
    y: i32,
    rows: Vec<Row>,
}

struct Float {
    unit: usize,
    text: String,
    color: Rgba,
    born: Instant,
}

struct Gui {
    host: Host,
    art: art::Art,
    tuning: tuning::TuningFile,
    canvas: Canvas,
    widgets: Vec<Widget>,
    mouse: (i32, i32),
    selected: Option<usize>,
    popup: Option<Popup>,
    playing: bool,
    auto: bool,
    next_at: Instant,
    floats: Vec<Float>,
    flashes: Vec<(usize, Instant)>,
    last_hp: Vec<i32>,
    party_tactic: usize,
    hero_tactic: usize,
    message: String,
    show_log: bool,
    encounter: String,
    seed: Option<i64>,
    shot_dir: PathBuf,
    tuning_checked: Instant,
    /// Autoplay: the next round starts once the round pause is over.
    pending_next_round: bool,
}

impl Gui {
    fn session(&self) -> &aegis_battle::session::Session {
        self.host.session.as_ref().expect("a battle is running")
    }

    /// Sends one request through the command API and turns HP changes into floating numbers.
    fn send(&mut self, req: Value) -> Value {
        let reply = self.host.handle(&req);
        if reply["ok"] != json!(true) {
            self.message = reply["error"].as_str().unwrap_or("refused").to_string();
        }
        self.after_change();
        reply
    }

    fn after_change(&mut self) {
        let now = Instant::now();
        let hps: Vec<i32> = self.session().w.units.iter().map(|u| u.hp).collect();
        if hps.len() == self.last_hp.len() {
            for (i, (&old, &new)) in self.last_hp.iter().zip(hps.iter()).enumerate() {
                if new < old {
                    self.floats.push(Float { unit: i, text: format!("-{}", old - new), color: self.tuning.value.damage_color, born: now });
                    self.flashes.push((i, now));
                } else if new > old {
                    self.floats.push(Float { unit: i, text: format!("+{}", new - old), color: self.tuning.value.heal_color, born: now });
                }
            }
        }
        self.last_hp = hps;
    }

    fn start_battle(&mut self) {
        let mut req = json!({"cmd": "new", "encounter": self.encounter});
        if let Some(s) = self.seed {
            req["seed"] = json!(s);
        }
        let reply = self.host.handle(&req);
        if reply["ok"] != json!(true) {
            self.message = reply["error"].as_str().unwrap_or("could not start").to_string();
            return;
        }
        self.last_hp = self.session().w.units.iter().map(|u| u.hp).collect();
        self.selected = self.session().w.allies.first().copied();
        self.popup = None;
        self.playing = false;
        self.auto = false;
        self.floats.clear();
        self.message = "Plan your party's round, then confirm.".into();
    }

    fn planning(&self) -> bool {
        let s = self.session();
        s.phase == Phase::Planning && !s.plan_confirmed
    }

    // ---- Actions (all mutations are command-API requests) -----------------------------------------------------

    fn act(&mut self, a: Action) {
        match a {
            Action::Select(u) => {
                self.selected = Some(u);
                self.popup = None;
            }
            Action::UnitClick(u) => self.unit_click(u),
            Action::TileClick(c) => self.tile_click(c),
            Action::Choose(i) => {
                let Some(p) = self.popup.take() else { return };
                let Some(row) = p.rows.get(i).filter(|r| r.valid) else { return };
                if let Some(h) = self.selected {
                    let r = self.send(json!({"cmd": "add_step", "unit": h, "step": row.step}));
                    if r["ok"] == json!(true) {
                        self.message = format!("{}: {}", self.session().w.units[h].name, row.label);
                    }
                }
            }
            Action::ClosePopup => self.popup = None,
            Action::Confirm => {
                let r = self.send(json!({"cmd": "confirm"}));
                if r["ok"] == json!(true) {
                    self.playing = true;
                    self.next_at = Instant::now();
                    self.message = "Resolving the round...".into();
                }
            }
            Action::Clear => {
                if let Some(h) = self.selected {
                    self.send(json!({"cmd": "clear", "unit": h}));
                }
            }
            Action::Undo => {
                if let Some(h) = self.selected {
                    self.send(json!({"cmd": "undo", "unit": h}));
                }
            }
            Action::Autoplay => {
                self.auto = !self.auto;
                if self.auto && self.planning() {
                    self.send(json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
                    self.act(Action::Confirm);
                }
            }
            Action::PartyTactic => {
                self.send(json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
                self.message = format!("Party drafted: {}", TACTIC_LABELS[self.party_tactic]);
            }
            Action::PartyCycle => self.party_tactic = (self.party_tactic + 1) % TACTICS.len(),
            Action::HeroTactic => {
                if let Some(h) = self.selected {
                    self.send(json!({"cmd": "preset", "unit": h, "preset": TACTICS[self.hero_tactic]}));
                }
            }
            Action::HeroCycle => self.hero_tactic = (self.hero_tactic + 1) % TACTICS.len(),
            Action::NextRound => {
                let r = self.send(json!({"cmd": "advance"}));
                if r["ok"] == json!(true) {
                    self.message = format!("Round {}: plan your party.", self.session().current_round);
                    if self.auto {
                        self.send(json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
                        self.act(Action::Confirm);
                    }
                }
            }
            Action::Step => {
                if self.session().phase == Phase::Replay {
                    self.act(Action::NextRound);
                } else {
                    self.send(json!({"cmd": "step"}));
                }
            }
            Action::Restart => self.start_battle(),
            Action::ToggleLog => self.show_log = !self.show_log,
        }
    }

    fn unit_click(&mut self, u: usize) {
        let s = self.session();
        let is_ally = s.w.allies.contains(&u);
        let names: Vec<String> = s.w.units.iter().map(|x| x.name.clone()).collect();
        if !self.planning() {
            if is_ally {
                self.selected = Some(u);
            }
            return;
        }
        let Some(h) = self.selected else {
            if is_ally {
                self.selected = Some(u);
            }
            return;
        };
        if is_ally && u != h {
            // Another hero: select it, unless the selected hero has ally-target support skills to offer.
            let rows = self.options_for(h, "ally", Some(u), None);
            if rows.iter().any(|r| r.valid) {
                self.open_popup(format!("{} -> {}", names[h], names[u]), rows);
            } else {
                self.selected = Some(u);
            }
            return;
        }
        let (title, rows) = if u == h { (format!("{}: self", names[h]), self.options_for(h, "self", None, None)) } else { (format!("Attack {}", names[u]), self.options_for(h, "foe", Some(u), None)) };
        self.open_popup(title, rows);
    }

    fn tile_click(&mut self, c: V2) {
        if !self.planning() {
            return;
        }
        let Some(h) = self.selected else { return };
        let rows = self.options_for(h, "tile", None, Some(c));
        self.open_popup(format!("Tile {},{}", c.x, c.y), rows);
    }

    fn open_popup(&mut self, title: String, rows: Vec<Row>) {
        if rows.is_empty() {
            self.message = "Nothing to do there.".into();
            return;
        }
        let (mx, my) = self.mouse;
        let w = 176;
        let h = 14 + rows.len() as i32 * 11;
        self.popup = Some(Popup { title, x: mx.clamp(2, W - w - 2), y: my.clamp(2, H - h - 2), rows });
    }

    /// Candidate steps for hero `h` (`what`: foe / ally / self / tile), each probed through the API.
    fn options_for(&mut self, h: usize, what: &str, target: Option<usize>, cell: Option<V2>) -> Vec<Row> {
        let skills = self.host.handle(&json!({"cmd": "skills", "unit": h}))["skills"].as_array().cloned().unwrap_or_default();
        let mut steps: Vec<(String, Value)> = Vec::new();
        for sk in &skills {
            let id = sk["skill"].as_str().unwrap_or("").to_string();
            let name = sk["name"].as_str().unwrap_or("").to_string();
            let kind = sk["kind"].as_str().unwrap_or("");
            let area = sk["area"].as_str().unwrap_or("NONE") != "NONE";
            match what {
                "foe" => match kind {
                    "ATTACK" if area => steps.push((name, json!({"mode": "area", "skill": id, "cell": target.map(|t| { let p = self.session().w.pos(t); json!([p.x, p.y]) })}))),
                    "ATTACK" | "DISENGAGE" => steps.push((name, json!({"mode": "engage", "skill": id, "target": target}))),
                    _ => {}
                },
                "ally" if kind == "SUPPORT" => steps.push((name, json!({"mode": "ally", "skill": id, "target": target}))),
                "self" if matches!(kind, "DEFEND" | "FOCUS" | "REST" | "DISENGAGE" | "SUPPORT") => steps.push((name, json!({"mode": "self", "skill": id}))),
                "tile" if kind == "ATTACK" && area => steps.push((name, json!({"mode": "area", "skill": id, "cell": cell.map(|c| json!([c.x, c.y]))}))),
                _ => {}
            }
        }
        if what == "tile" {
            if let Some(c) = cell {
                steps.insert(0, ("Move here".into(), json!({"mode": "goto", "cell": [c.x, c.y]})));
            }
        }
        let mut rows = Vec::new();
        for (label, step) in steps {
            let p = self.host.handle(&json!({"cmd": "probe", "unit": h, "step": step}));
            let valid = p["valid"] == json!(true);
            let detail = if valid {
                let dmg: i64 = p["entries"].as_array().map(|es| es.iter().map(|e| e["est_damage"].as_i64().unwrap_or(0)).sum()).unwrap_or(0);
                let heal: i64 = p["entries"].as_array().map(|es| es.iter().map(|e| e["est_heal"].as_i64().unwrap_or(0)).sum()).unwrap_or(0);
                let mut d = format!("{}AP", p["ap"].as_i64().unwrap_or(0));
                if dmg > 0 {
                    d.push_str(&format!(" ~{dmg}"));
                }
                if heal > 0 {
                    d.push_str(&format!(" +{heal}"));
                }
                d
            } else {
                p["reason"].as_str().unwrap_or("").to_string()
            };
            rows.push(Row { label, detail, valid, step });
        }
        rows.sort_by_key(|r| !r.valid);
        rows
    }

    // ---- Per-frame update ---------------------------------------------------------------------------------------

    fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.tuning_checked) > Duration::from_millis(500) {
            self.tuning_checked = now;
            if self.tuning.reload_if_changed() {
                self.message = "Tuning reloaded.".into();
            }
        }
        self.floats.retain(|f| now.duration_since(f.born) < Duration::from_millis(self.tuning.value.float_ms));
        self.flashes.retain(|f| now.duration_since(f.1) < Duration::from_millis(self.tuning.value.hit_flash_ms));
        if self.host.session.is_none() {
            return;
        }
        if self.playing && now >= self.next_at {
            if self.session().phase == Phase::Replay {
                self.playing = false;
            } else {
                self.send(json!({"cmd": "step"}));
                self.next_at = now + Duration::from_millis(self.tuning.value.step_ms);
                if self.session().phase == Phase::Replay {
                    self.playing = false;
                    let s = self.session();
                    let can_advance = s.can_advance_round();
                    let message = if s.combat_over {
                        match s.outcome.as_str() {
                            "won" => "Victory!".into(),
                            "lost" => "Defeat.".into(),
                            o => format!("Battle over: {o}"),
                        }
                    } else if !can_advance {
                        "Round limit reached.".into()
                    } else {
                        "Round over. Space for the next round.".into()
                    };
                    self.message = message;
                    if self.auto && can_advance {
                        self.next_at = now + Duration::from_millis(self.tuning.value.round_pause_ms);
                        self.playing = true;
                        self.pending_next_round = true;
                    }
                }
            }
        }
        if self.pending_next_round && now >= self.next_at && self.session().phase == Phase::Replay {
            self.pending_next_round = false;
            self.playing = false;
            self.act(Action::NextRound);
        }
    }
}

struct App {
    gui: Option<Gui>,
    cat: Arc<Catalog>,
    args: Vec<(String, String)>,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    proxy: EventLoopProxy<()>,
    bridge: Option<Receiver<bridge::Request>>,
    project: Option<PathBuf>,
    engine_dir: Option<PathBuf>,
}

fn arg<'a>(args: &'a [(String, String)], k: &str) -> Option<&'a str> {
    args.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
}

impl App {
    fn redraw(&mut self) {
        let Some(gui) = self.gui.as_mut() else { return };
        if let Some(rx) = &self.bridge {
            while let Ok(req) = rx.try_recv() {
                let reply = bridge_request(gui, &req.body);
                let _ = req.reply.send(reply);
            }
        }
        gui.tick();
        draw(gui);
        if let Some(p) = self.pixels.as_mut() {
            p.frame_mut().copy_from_slice(&gui.canvas.px);
            let _ = p.render();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("Aegis - battle").with_inner_size(winit::dpi::LogicalSize::new(1280, 720)).with_min_inner_size(winit::dpi::LogicalSize::new(640, 360));
        let window = Arc::new(el.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let pixels = Pixels::new(W as u32, H as u32, SurfaceTexture::new(size.width, size.height, window.clone())).expect("pixels");
        let tuning_path = self.engine_dir.as_ref().map(|d| d.join("data/tuning.json"));
        let mut gui = Gui {
            host: Host::new(self.cat.clone()),
            art: art::Art::new(self.project.clone()),
            tuning: tuning::TuningFile::new(tuning_path),
            canvas: Canvas::new(W, H),
            widgets: Vec::new(),
            mouse: (0, 0),
            selected: None,
            popup: None,
            playing: false,
            auto: false,
            next_at: Instant::now(),
            floats: Vec::new(),
            flashes: Vec::new(),
            last_hp: Vec::new(),
            party_tactic: 0,
            hero_tactic: 0,
            message: String::new(),
            show_log: false,
            encounter: arg(&self.args, "encounter").unwrap_or("goblin_skirmish").to_string(),
            seed: arg(&self.args, "seed").and_then(|s| s.parse().ok()),
            shot_dir: self.engine_dir.clone().map(|d| d.join("parity/out")).unwrap_or_else(|| PathBuf::from(".")),
            tuning_checked: Instant::now(),
            pending_next_round: false,
        };
        gui.start_battle();
        let port = arg(&self.args, "port").and_then(|p| p.parse().ok()).unwrap_or(bridge::DEFAULT_PORT);
        self.bridge = bridge::start(port, self.proxy.clone());
        if self.bridge.is_none() {
            gui.message = format!("Agent link: port {port} is busy.");
        }
        self.gui = Some(gui);
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
                if let Some(p) = self.pixels.as_mut() {
                    let _ = p.resize_surface(size.width.max(1), size.height.max(1));
                }
            }
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => {
                if let (Some(p), Some(g)) = (self.pixels.as_ref(), self.gui.as_mut()) {
                    let (x, y) = p.window_pos_to_pixel((position.x as f32, position.y as f32)).unwrap_or_else(|(x, y)| (x.max(0) as usize, y.max(0) as usize));
                    g.mouse = (x as i32, y as i32);
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                let Some(g) = self.gui.as_mut() else { return };
                match button {
                    MouseButton::Left => {
                        if let Some(a) = hit(g, g.mouse) {
                            g.act(a);
                        }
                    }
                    MouseButton::Right => {
                        if g.popup.is_some() {
                            g.popup = None;
                        } else if g.planning() {
                            g.act(Action::Undo);
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. }, .. } => {
                let Some(g) = self.gui.as_mut() else { return };
                match logical_key {
                    Key::Named(NamedKey::Enter) if g.planning() => g.act(Action::Confirm),
                    Key::Named(NamedKey::Space) => g.act(Action::Step),
                    Key::Named(NamedKey::Escape) => g.popup = None,
                    Key::Named(NamedKey::F12) => {
                        let p = g.shot_dir.join("gui_shot.png");
                        g.message = match save_png(&g.canvas, &p, 2) {
                            Ok(()) => format!("Screenshot: {}", p.display()),
                            Err(e) => format!("Screenshot failed: {e}"),
                        };
                    }
                    Key::Character(c) => match c.as_str() {
                        "a" | "A" => g.act(Action::Autoplay),
                        "z" | "Z" => g.act(Action::Undo),
                        "l" | "L" => g.act(Action::ToggleLog),
                        "r" | "R" => g.act(Action::Restart),
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// The topmost enabled widget under `pos`.
fn hit(g: &Gui, pos: (i32, i32)) -> Option<Action> {
    g.widgets.iter().rev().find(|w| w.enabled && w.action.is_some() && w.rect.contains(pos.0, pos.1)).and_then(|w| w.action.clone())
}

/// Agent link requests: the command API, plus ui_tree / screenshot / click on the live screen.
fn bridge_request(g: &mut Gui, body: &Value) -> Value {
    match body.get("cmd").and_then(Value::as_str).unwrap_or("") {
        "ui_tree" => json!({"ok": true, "canvas": [W, H], "mouse": [g.mouse.0, g.mouse.1], "message": g.message, "playing": g.playing, "auto": g.auto,
            "popup": g.popup.as_ref().map(|p| json!({"title": p.title, "rows": p.rows.iter().map(|r| json!({"label": r.label, "detail": r.detail, "valid": r.valid})).collect::<Vec<_>>()})),
            "widgets": g.widgets.iter().map(|w| json!({"id": w.id, "kind": w.kind, "text": w.text, "rect": [w.rect.x, w.rect.y, w.rect.w, w.rect.h], "enabled": w.enabled, "active": w.active, "clickable": w.action.is_some()})).collect::<Vec<_>>()}),
        "screenshot" => {
            let path = body.get("path").and_then(Value::as_str).map(PathBuf::from).unwrap_or_else(|| g.shot_dir.join("gui_shot.png"));
            let scale = body.get("scale").and_then(Value::as_u64).unwrap_or(2) as i32;
            match save_png(&g.canvas, &path, scale) {
                Ok(()) => json!({"ok": true, "path": path.display().to_string()}),
                Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "click" => {
            let id = body.get("id").and_then(Value::as_str).unwrap_or("");
            match g.widgets.iter().find(|w| w.id == id).cloned() {
                Some(w) if w.enabled && w.action.is_some() => {
                    g.act(w.action.unwrap());
                    json!({"ok": true, "clicked": id})
                }
                Some(_) => json!({"ok": false, "error": format!("widget '{id}' is disabled or not clickable")}),
                None => json!({"ok": false, "error": format!("no widget '{id}'")}),
            }
        }
        "_bad" => json!({"ok": false, "error": body["error"]}),
        _ => {
            let reply = g.host.handle(body);
            if g.host.session.is_some() {
                g.after_change();
            }
            reply
        }
    }
}

fn save_png(c: &Canvas, path: &Path, scale: i32) -> Result<()> {
    let scale = scale.max(1);
    let (w, h) = (c.w * scale, c.h * scale);
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let s = (((y / scale) * c.w + x / scale) * 4) as usize;
            let d = ((y * w + x) * 4) as usize;
            data[d..d + 4].copy_from_slice(&c.px[s..s + 4]);
        }
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&data)?;
    Ok(())
}

// ---- Drawing ------------------------------------------------------------------------------------------------------

struct Ui<'a> {
    c: &'a mut Canvas,
    widgets: Vec<Widget>,
    mouse: (i32, i32),
}

impl<'a> Ui<'a> {
    fn push(&mut self, id: impl Into<String>, kind: &'static str, text: impl Into<String>, rect: Rect, enabled: bool, active: bool, action: Option<Action>) {
        self.widgets.push(Widget { id: id.into(), kind, text: text.into(), rect, enabled, active, action });
    }

    fn button(&mut self, id: &str, r: Rect, label: &str, action: Action, enabled: bool, active: bool) {
        let hover = enabled && r.contains(self.mouse.0, self.mouse.1);
        let fill = if active { rgb(70, 58, 24) } else if hover { PANEL_HI } else { PANEL };
        let border = if active { GOLD } else if hover { rgb(96, 124, 176) } else { BORDER };
        self.c.panel(r, fill, border);
        let avail = r.w - 2;
        let tw = font::text_width(label).min(avail);
        self.c.text_fit(r.x + (r.w - tw) / 2, r.y + (r.h - 7) / 2, label, avail, if enabled { if active { GOLD } else { TEXT } } else { DIM });
        self.push(id, "button", label, r, enabled, active, Some(action));
    }
}

fn kind_color(kind: &str) -> Rgba {
    match kind {
        "ATTACK" => rgb(150, 52, 52),
        "MOVE" => rgb(48, 86, 150),
        "DEFEND" => rgb(40, 116, 116),
        "SUPPORT" => rgb(48, 120, 64),
        "FOCUS" => rgb(120, 92, 32),
        "REST" => rgb(70, 74, 88),
        "STATUS" => rgb(96, 52, 120),
        "DISENGAGE" => rgb(52, 110, 150),
        _ => rgb(60, 66, 84),
    }
}

fn hp_bar(c: &mut Canvas, r: Rect, hp: i32, max: i32, color: Rgba) {
    c.fill(r, rgb(40, 22, 26));
    let w = if max > 0 { (r.w * hp.max(0) / max).max(if hp > 0 { 1 } else { 0 }) } else { 0 };
    c.fill(Rect::new(r.x, r.y, w, r.h), color);
}

fn cell_rect(c: V2) -> Rect {
    Rect::new(GRID_X + c.x * TILE, GRID_Y + c.y * TILE, TILE, TILE)
}

fn terrain_color(id: &str) -> Rgba {
    match id {
        "wall" => rgb(46, 46, 54),
        "tree" => rgb(30, 70, 40),
        "deep_water" => rgb(30, 54, 110),
        "shallow_water" => rgb(56, 92, 140),
        "lava" => rgb(150, 50, 20),
        "grass" => rgb(60, 104, 56),
        "dirt" => rgb(104, 84, 56),
        "sand" | "snow" => rgb(170, 160, 120),
        "stone" | "rubble" => rgb(92, 92, 100),
        _ => rgb(80, 82, 92),
    }
}

fn short_name(name: &str) -> String {
    name.strip_prefix("Goblin ").unwrap_or(name).to_string()
}

fn draw(g: &mut Gui) {
    let now = Instant::now();
    let mouse = g.mouse;
    let tuning = g.tuning.value.clone();
    g.canvas.clear(BG);
    let mut ui = Ui { c: &mut g.canvas, widgets: Vec::new(), mouse };
    let Some(s) = g.host.session.as_ref() else {
        ui.c.text(8, 8, &g.message, FOE);
        g.widgets = ui.widgets;
        return;
    };
    let w = &s.w;
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    let sel = g.selected.filter(|u| w.allies.contains(u));

    // -- Top strip: the selected hero's six blocks and the round controls.
    let top = Rect::new(2, 2, W - 4, 24);
    ui.c.panel(top, PANEL, BORDER);
    let hero_name = sel.map(|u| w.units[u].name.clone()).unwrap_or_else(|| "No hero".into());
    ui.c.text_fit(8, 10, &format!("{hero_name} plan"), 84, GOLD);
    ui.push("strip.title", "label", format!("{hero_name} plan"), Rect::new(8, 8, 84, 10), true, false, None);
    let entries: Vec<(i32, i32, String, String)> = match sel {
        Some(u) if s.plan_confirmed => committed(s, u).into_iter().map(|c| (c.0, c.1, c.2, c.3)).collect(),
        Some(u) if w.units[u].is_active() => s
            .plan
            .compile(w, u)
            .entries
            .iter()
            .map(|e| {
                let sk = &w.cat.skills[e.skill];
                (e.start_block, e.end_block(w), sk.name.clone(), sk.kind.clone())
            })
            .collect(),
        _ => Vec::new(),
    };
    let (bx, bw) = (96, 30);
    for b in 1..=s.def.total_blocks {
        let r = Rect::new(bx + (b - 1) * (bw + 2), 6, bw, 16);
        let entry = entries.iter().find(|e| b >= e.0 && b <= e.1);
        let live = !planning && s.phase == Phase::Resolving && s.current_block == b;
        match entry {
            Some((start, _, name, kind)) => {
                ui.c.fill(r, kind_color(kind));
                if *start == b {
                    ui.c.text_fit(r.x + 2, r.y + 5, name, bw - 3, TEXT);
                }
            }
            None => {
                ui.c.panel(r, rgb(16, 22, 36), BORDER);
                ui.c.text(r.x + 2, r.y + 5, &format!("{b}"), DIM);
            }
        }
        if live {
            ui.c.outline(r, GOLD);
        }
        ui.push(format!("strip.block{b}"), "block", entry.map(|e| e.2.clone()).unwrap_or_else(|| "empty".into()), r, true, live, None);
    }
    let mut x = bx + 6 * (bw + 2) + 6;
    ui.button("strip.party", Rect::new(x, 6, 70, 16), &format!("Party:{}", TACTIC_LABELS[g.party_tactic]), Action::PartyTactic, planning, false);
    ui.button("strip.party_next", Rect::new(x + 71, 6, 9, 16), ">", Action::PartyCycle, true, false);
    x += 84;
    ui.button("strip.hero", Rect::new(x, 6, 66, 16), &format!("Hero:{}", TACTIC_LABELS[g.hero_tactic]), Action::HeroTactic, planning && sel.is_some(), false);
    ui.button("strip.hero_next", Rect::new(x + 67, 6, 9, 16), ">", Action::HeroCycle, true, false);
    x += 80;
    ui.button("strip.auto", Rect::new(x, 6, 28, 16), "Auto", Action::Autoplay, !s.combat_over, g.auto);
    x += 30;
    ui.button("strip.clear", Rect::new(x, 6, 34, 16), "Clear", Action::Clear, planning && sel.is_some(), false);
    x += 36;
    let confirm_label = if planning { "Confirm" } else if s.phase == Phase::Replay && s.can_advance_round() { "Next" } else { "..." };
    let confirm_action = if planning { Action::Confirm } else { Action::NextRound };
    let confirm_enabled = planning || (s.phase == Phase::Replay && s.can_advance_round() && !g.playing);
    ui.button("strip.confirm", Rect::new(x, 6, W - 6 - x, 16), confirm_label, confirm_action, confirm_enabled, planning);

    // -- Left: party cards and the selected hero.
    let left = Rect::new(2, 28, 124, 290);
    ui.c.panel(left, PANEL, BORDER);
    let alive = w.allies.iter().filter(|a| w.units[**a].is_active()).count();
    ui.c.text(8, 33, &format!("Party {alive}/{}", w.allies.len()), ALLY);
    let mut y = 44;
    for &a in &w.allies {
        let u = &w.units[a];
        let r = Rect::new(6, y, 116, 30);
        let is_sel = Some(a) == sel;
        let hover = r.contains(mouse.0, mouse.1);
        ui.c.panel(r, if is_sel { PANEL_HI } else if hover { rgb(26, 36, 58) } else { rgb(16, 22, 36) }, if is_sel { GOLD } else { BORDER });
        ui.c.text_fit(r.x + 4, r.y + 3, &u.name, 108, if u.is_active() { ALLY } else { DIM });
        hp_bar(ui.c, Rect::new(r.x + 4, r.y + 12, 108, 4), u.hp, u.max_hp, GOOD);
        ui.c.text(r.x + 4, r.y + 19, &format!("HP {}/{}", u.hp, u.max_hp), TEXT);
        ui.c.text_right(r.x + r.w - 4, r.y + 19, &format!("AP {}", u.ap), ALLY);
        ui.push(format!("party.{a}"), "card", format!("{} HP {}/{} AP {}/{}", u.name, u.hp, u.max_hp, u.ap, u.max_ap), r, true, is_sel, Some(Action::Select(a)));
        y += 33;
    }
    if let Some(h) = sel {
        let u = &w.units[h];
        let r = Rect::new(6, y + 2, 116, 318 - y - 6);
        ui.c.panel(r, rgb(16, 22, 36), BORDER);
        let stance = ["Neutral", "Offensive", "Defensive", "Tactical"][u.stance as usize];
        ui.c.text_fit(r.x + 4, r.y + 4, &u.name, 64, GOLD);
        ui.c.text_right(r.x + r.w - 4, r.y + 4, stance, DIM);
        let mut ly = r.y + 15;
        let st = s.plan.compile(w, h);
        let weapon = u.inventory.main_weapon().map(|e| e.item.name.clone()).unwrap_or_else(|| "Bare hands".into());
        let mut lines: Vec<(String, Rgba)> = vec![
            (format!("Speed {} Reach {}", u.speed, u.attack_range), TEXT),
            (format!("AP left {}/{}", st.ap_left, u.max_ap), ALLY),
            (format!("Gear: {weapon}"), GOLD),
        ];
        if !u.st.active.is_empty() {
            lines.push((format!("Now: {}", u.st.active.iter().cloned().collect::<Vec<_>>().join(", ")), FOE));
        }
        if !u.st.pending.is_empty() {
            lines.push((format!("Next: {}", u.st.pending.iter().cloned().collect::<Vec<_>>().join(", ")), DIM));
        }
        if s.plan_confirmed {
            lines.push(("This round".into(), TEXT));
            for (start, _, name, _, tgt) in committed(s, h) {
                let tgt = tgt.map(|t| format!(" > {}", short_name(&t))).unwrap_or_default();
                lines.push((format!("B{start} {name}{tgt}"), TEXT));
            }
        } else {
            lines.push(("Plan".into(), TEXT));
            if st.entries.iter().all(|e| e.step_index < 0) {
                lines.push(("No plan yet".into(), GOLD));
            }
            for e in st.entries.iter().filter(|e| e.step_index >= 0) {
                let name = &w.cat.skills[e.skill].name;
                let tgt = e.target.filter(|t| *t != h).map(|t| format!(" > {}", short_name(&w.units[t].name))).unwrap_or_default();
                lines.push((format!("B{} {name}{tgt}", e.start_block), TEXT));
            }
            if st.error_index >= 0 {
                lines.push((st.error_text.clone(), FOE));
            }
        }
        for (text, col) in lines {
            if ly > r.y + r.h - 10 {
                break;
            }
            ui.c.text_fit(r.x + 4, ly, &text, 108, col);
            ly += font::LINE_H;
        }
        ui.push("hero.panel", "panel", u.name.clone(), r, true, false, None);
    }

    // -- Grid: terrain, exits, plan ghost, intents, units, floating numbers.
    let grid_r = Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4);
    ui.c.panel(grid_r, rgb(8, 10, 16), BORDER);
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let r = cell_rect(c);
            let tid = w.grid.terrain_id(c);
            match g.art.tile(tid, gx, gy, TILE) {
                Some(img) => ui.c.image(img, r.x, r.y),
                None => ui.c.fill(r, terrain_color(tid)),
            }
            ui.c.outline(r, rgba(0, 0, 0, 60));
            if w.grid.is_exit(c) {
                ui.c.fill(r.inset(8), rgba(70, 200, 90, 200));
                ui.c.text(r.x + 13, r.y + 13, ">", BG);
            }
            if planning && sel.is_some() && r.contains(mouse.0, mouse.1) && g.popup.is_none() {
                ui.c.outline(r.inset(1), rgba(255, 255, 255, 120));
            }
            ui.push(format!("tile.{gx}.{gy}"), "tile", tid, r, planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None });
        }
    }
    let center = |c: V2| (GRID_X + c.x * TILE + TILE / 2, GRID_Y + c.y * TILE + TILE / 2);
    if let Some(h) = sel {
        if w.units[h].is_active() && planning {
            let st = s.plan.compile(w, h);
            let mut from = w.pos(h);
            for e in &st.entries {
                if e.pos_after != from && e.pos_after.x >= 0 {
                    let (a, b) = (center(from), center(e.pos_after));
                    ui.c.line(a.0, a.1, b.0, b.1, GOLD, 3);
                    ui.c.outline(cell_rect(e.pos_after).inset(3), GOLD);
                    from = e.pos_after;
                }
            }
        }
    }
    if tuning.intent_arrows && planning {
        let mut seen = Vec::new();
        for a in &s.resolver.scheduled {
            if !w.enemies.contains(&a.unit) || seen.contains(&a.unit) {
                continue;
            }
            if let CmdKind::Attack { target: Some(t), .. } = a.command.kind {
                seen.push(a.unit);
                let (p, q) = (center(w.pos(a.unit)), center(w.pos(t)));
                ui.c.line(p.0, p.1, q.0, q.1, rgba(238, 92, 86, 200), 4);
            }
        }
    }
    for i in 0..w.units.len() {
        let Some(p) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let r = cell_rect(p);
        let ally = w.allies.contains(&i);
        let flash = g.flashes.iter().find(|f| f.0 == i).map(|f| now.duration_since(f.1).as_millis() as u64);
        let shake = if flash.is_some() { tuning.hit_shake_px * if (now.elapsed().as_millis() / 30) % 2 == 0 { 1 } else { -1 } } else { 0 };
        let tp = tuning.token_px;
        let (tx, ty) = (r.x + (TILE - tp) / 2 + shake, r.y + (TILE - tp) / 2);
        if !u.is_alive() {
            ui.c.text(r.x + 13, r.y + 12, "x", DIM);
            continue;
        }
        let ring = if ally { ALLY } else { FOE };
        ui.c.circle(r.x + TILE / 2 + shake, r.y + TILE / 2, tp / 2 + 1, ring);
        match g.art.token(&s.art[i], tp) {
            Some(img) => ui.c.image(img, tx, ty),
            None => {
                ui.c.fill(Rect::new(tx + 4, ty + 4, tp - 8, tp - 8), ring);
                ui.c.text(tx + tp / 2 - 3, ty + tp / 2 - 3, &u.name.chars().next().unwrap_or('?').to_string(), BG);
            }
        }
        if flash.is_some() {
            ui.c.fill(Rect::new(tx, ty, tp, tp), rgba(255, 255, 255, 110));
        }
        hp_bar(ui.c, Rect::new(r.x + 3, r.y + TILE - 3, TILE - 6, 2), u.hp, u.max_hp, if ally { GOOD } else { FOE });
        if Some(i) == sel {
            ui.c.outline(r, GOLD);
        }
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), r, true, Some(i) == sel, Some(Action::UnitClick(i)));
    }
    for f in &g.floats {
        if let Some(p) = w.grid.pos(f.unit) {
            let t = now.duration_since(f.born).as_millis() as i32;
            let rise = tuning.float_rise_px * t / tuning.float_ms.max(1) as i32;
            let (cx, cy) = center(p);
            let tw = font::text_width(&f.text);
            ui.c.text(cx - tw / 2 + 1, cy - 12 - rise + 1, &f.text, rgba(0, 0, 0, 200));
            ui.c.text(cx - tw / 2, cy - 12 - rise, &f.text, f.color);
        }
    }

    // -- Right: hostile cards and the hovered / first foe's detail.
    let right = Rect::new(W - 126, 28, 124, 290);
    ui.c.panel(right, PANEL, BORDER);
    let foes_active = w.enemies.iter().filter(|e| w.units[**e].is_active()).count();
    ui.c.text(right.x + 6, 33, &format!("Hostiles {foes_active}"), FOE);
    let next_of = |e: usize| -> String {
        if s.phase == Phase::Replay {
            return "-".into();
        }
        let a = s.resolver.scheduled.iter().find(|a| a.unit == e && !a.cancelled && (planning || a.end >= s.current_block));
        match a {
            Some(a) => match a.command.kind {
                CmdKind::Attack { target: Some(t), .. } => format!("{} on {}", a.command.name, short_name(&w.units[t].name)),
                _ => a.command.name.clone(),
            },
            None => "-".into(),
        }
    };
    let mut y = 44;
    let mut hovered_foe = None;
    for &e in &w.enemies {
        let u = &w.units[e];
        if w.grid.pos(e).is_none() && u.is_alive() && !u.is_withdrawn {
            continue; // reinforcements not on the field yet
        }
        if y > 210 {
            break;
        }
        let r = Rect::new(right.x + 4, y, 116, 21);
        let hover = r.contains(mouse.0, mouse.1) || w.grid.pos(e).is_some_and(|p| cell_rect(p).contains(mouse.0, mouse.1));
        if hover {
            hovered_foe = Some(e);
        }
        ui.c.panel(r, if hover { rgb(40, 26, 34) } else { rgb(16, 22, 36) }, if hover { FOE } else { BORDER });
        let col = if u.is_active() { FOE } else { DIM };
        ui.c.text_fit(r.x + 3, r.y + 3, &short_name(&u.name), 70, col);
        ui.c.text_right(r.x + r.w - 3, r.y + 3, &format!("{}/{}", u.hp, u.max_hp), if u.is_active() { GOOD } else { DIM });
        if u.is_active() {
            ui.c.text_fit(r.x + 3, r.y + 12, &format!("Next: {}", next_of(e)), 110, GOLD);
        } else {
            ui.c.text(r.x + 3, r.y + 12, if u.is_withdrawn { "Fled" } else { "Down" }, DIM);
        }
        ui.push(format!("foe.{e}"), "card", format!("{} {}/{} next {}", u.name, u.hp, u.max_hp, next_of(e)), r, true, hover, Some(Action::UnitClick(e)));
        y += 23;
    }
    let detail_foe = hovered_foe.or_else(|| w.enemies.iter().copied().find(|e| w.units[*e].is_active() && w.grid.pos(*e).is_some()));
    if let Some(e) = detail_foe {
        let u = &w.units[e];
        let r = Rect::new(right.x + 4, y + 2, 116, 318 - y - 6);
        ui.c.panel(r, rgb(16, 22, 36), FOE);
        ui.c.text_fit(r.x + 4, r.y + 4, &u.name, 108, TEXT);
        let mut lines: Vec<(String, Rgba)> = vec![(format!("HP {}/{}", u.hp, u.max_hp), TEXT), (format!("Speed {} Reach {}", u.speed, u.attack_range), TEXT), (format!("Next: {}", next_of(e)), GOLD)];
        if let Some(h) = sel.filter(|h| w.units[*h].is_active()) {
            let strike = w.book().lowest(aegis_battle::skill::Kind::Attack);
            let deal = expected(w, h, e, strike, None, 0.0, 1.0);
            let take = expected(w, e, h, strike, None, 0.0, 1.0);
            lines.push((format!("{} deals ~{deal}", short_name(&w.units[h].name)), ALLY));
            lines.push((format!("takes ~{take}"), ALLY));
        }
        if !u.st.active.is_empty() {
            lines.push((u.st.active.iter().cloned().collect::<Vec<_>>().join(", "), FOE));
        }
        let mut ly = r.y + 15;
        for (t, col) in lines {
            if ly > r.y + r.h - 10 {
                break;
            }
            ui.c.text_fit(r.x + 4, ly, &t, 108, col);
            ly += font::LINE_H;
        }
    }

    // -- Bottom bar: playback controls, round and status, the last log line.
    let bottom = Rect::new(2, 322, W - 4, 36);
    ui.c.panel(bottom, PANEL, BORDER);
    ui.button("bar.restart", Rect::new(6, 326, 18, 13), "R", Action::Restart, true, false);
    ui.button("bar.step", Rect::new(26, 326, 30, 13), "Step", Action::Step, !planning || s.plan_confirmed, false);
    let phase = match s.phase {
        Phase::Planning => "Planning",
        Phase::Resolving => "Resolving",
        Phase::Replay => "Round over",
    };
    ui.c.text(62, 329, &format!("Round {}/{}  {phase}", s.current_round, s.def.max_rounds), GOLD);
    ui.c.text_fit(200, 329, &g.message, 300, TEXT);
    ui.push("bar.status", "label", format!("Round {} {phase}: {}", s.current_round, g.message), Rect::new(62, 327, 440, 10), true, false, None);
    let last = s.resolver.log.last().cloned().unwrap_or_else(|| "No actions yet".into());
    ui.c.text_fit(8, 344, &last, W - 70, DIM);
    ui.button("bar.log", Rect::new(W - 52, 341, 46, 14), if g.show_log { "Close" } else { "Log" }, Action::ToggleLog, true, g.show_log);

    if s.combat_over {
        let r = Rect::new(GRID_X + 96, GRID_Y + 112, 192, 52);
        ui.c.panel(r, rgba(10, 14, 22, 235), GOLD);
        let title = match s.outcome.as_str() {
            "won" => "VICTORY",
            "lost" => "DEFEAT",
            _ => "BATTLE OVER",
        };
        ui.c.text(r.x + (r.w - font::text_width(title)) / 2, r.y + 10, title, if s.outcome == "won" { GOOD } else { FOE });
        ui.button("banner.restart", Rect::new(r.x + 60, r.y + 28, 72, 16), "Play again", Action::Restart, true, false);
    }
    if g.show_log {
        let r = Rect::new(GRID_X, GRID_Y + 8, GRID_W * TILE, GRID_H * TILE - 16);
        ui.c.panel(r, rgba(8, 10, 16, 240), BORDER);
        let lines = &s.resolver.log;
        let n = ((r.h - 8) / font::LINE_H) as usize;
        for (k, l) in lines.iter().rev().take(n).rev().enumerate() {
            ui.c.text_fit(r.x + 4, r.y + 4 + k as i32 * font::LINE_H, l, r.w - 8, TEXT);
        }
        ui.push("log.panel", "panel", format!("{} lines", lines.len()), r, true, true, Some(Action::ToggleLog));
    }
    if let Some(p) = &g.popup {
        let h = 14 + p.rows.len() as i32 * 11;
        let r = Rect::new(p.x, p.y, 176, h);
        ui.push("popup.backdrop", "backdrop", "", Rect::new(0, 0, W, H), true, false, Some(Action::ClosePopup));
        ui.c.panel(r, rgb(14, 20, 34), GOLD);
        ui.c.text_fit(r.x + 4, r.y + 3, &p.title, r.w - 8, GOLD);
        for (k, row) in p.rows.iter().enumerate() {
            let rr = Rect::new(r.x + 2, r.y + 12 + k as i32 * 11, r.w - 4, 11);
            let hover = row.valid && rr.contains(mouse.0, mouse.1);
            if hover {
                ui.c.fill(rr, PANEL_HI);
            }
            let col = if row.valid { TEXT } else { DIM };
            ui.c.text_fit(rr.x + 2, rr.y + 2, &row.label, 84, col);
            ui.c.text_fit(rr.x + 88, rr.y + 2, &row.detail, rr.w - 90, if row.valid { GOLD } else { DIM });
            ui.push(format!("popup.{k}"), "option", format!("{} {}", row.label, row.detail), rr, row.valid, hover, Some(Action::Choose(k)));
        }
    }
    g.widgets = ui.widgets;
}

fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<(String, String)> = raw.iter().filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
    let cwd = std::env::current_dir()?;
    let engine_dir = {
        let mut d = cwd.clone();
        loop {
            if d.join("crates/aegis_gui").is_dir() {
                break Some(d);
            }
            if d.join("engine/crates/aegis_gui").is_dir() {
                break Some(d.join("engine"));
            }
            if !d.pop() {
                break None;
            }
        }
    };
    let catalog = arg(&args, "catalog").map(PathBuf::from).or_else(|| std::env::var("AEGIS_CATALOG").ok().map(PathBuf::from)).or_else(|| engine_dir.as_ref().map(|d| d.join("parity/out/catalog.json"))).context("catalog.json not found: pass catalog=<path>")?;
    let cat = Arc::new(Catalog::load_resolved(&catalog).with_context(|| format!("loading {}", catalog.display()))?);
    let project = art::find_project(engine_dir.as_deref().unwrap_or(&cwd));
    let el: EventLoop<()> = EventLoop::with_user_event().build()?;
    let proxy = el.create_proxy();
    let mut app = App { gui: None, cat, args, window: None, pixels: None, proxy, bridge: None, project, engine_dir };
    el.run_app(&mut app)?;
    Ok(())
}

/// The actions the resolver holds for `u` this round (after confirm): (start, end, name, kind, target name).
fn committed(s: &aegis_battle::session::Session, u: usize) -> Vec<(i32, i32, String, String, Option<String>)> {
    s.resolver
        .scheduled
        .iter()
        .filter(|a| a.unit == u)
        .map(|a| {
            let (kind, target) = match &a.command.kind {
                CmdKind::Attack { target, .. } => ("ATTACK", *target),
                CmdKind::Area { .. } => ("ATTACK", None),
                CmdKind::Defend { .. } => ("DEFEND", None),
                CmdKind::Focus { .. } => ("FOCUS", None),
                CmdKind::Rest { .. } => ("REST", None),
                CmdKind::StatusBlock { .. } => ("STATUS", None),
                CmdKind::Support { target, .. } => ("SUPPORT", Some(*target)),
                CmdKind::Target { target } => ("TARGET", *target),
                CmdKind::Walk { .. } => ("MOVE", None),
                CmdKind::Disengage { .. } => ("DISENGAGE", None),
            };
            let name = if a.cancelled { format!("{} (x)", a.command.name) } else { a.command.name.clone() };
            (a.start, a.end, name, kind.to_string(), target.filter(|t| *t != u).map(|t| s.w.units[t].name.clone()))
        })
        .collect()
}
