//! aegis-gui: the battle screen (COMBAT-UI3 layout A). Two layers: the battlefield is pixel art scaled with nearest
//! sampling; panels, text, hover cards and menus are drawn smooth at the window's real resolution (painter.rs).
//! Every change goes through the command API (aegis_host), the same one `aegis serve` and the agent link use.
//! Feel values come from engine/data/tuning.json (hot-reloaded).
//!   aegis-gui [encounter=goblin_skirmish] [seed=N] [port=47800] [catalog=<path>]
//! Mouse: click a hero to select; click a foe, an ally, the hero itself or a tile for its options; hover anything for
//! details; right click closes a menu or undoes the hero's last step; the wheel scrolls the log. Keys: Enter confirm,
//! Space step / next round, A autoplay, Z undo, L log, R restart, F12 screenshot, Esc close.

mod art;
mod bridge;
mod canvas;
mod painter;
mod tuning;

use aegis_battle::command::CmdKind;
use aegis_battle::grid::{V2, GRID_H, GRID_W};
use aegis_battle::plan::expected;
use aegis_battle::session::{Phase, Session};
use aegis_battle::skill::Kind;
use aegis_data::Catalog;
use aegis_host::Host;
use anyhow::{Context, Result};
use canvas::{rgb, rgba, Canvas, Image, Rect, Rgba};
use painter::{Fonts, Painter, View};
use pixels::{Pixels, SurfaceTexture};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const W: i32 = 640;
const H: i32 = 360;
const TILE: i32 = 32;
const GRID_X: i32 = 128;
const GRID_Y: i32 = 30;
const BODY: f32 = 7.0;
const SMALL: f32 = 6.0;
const TITLE: f32 = 7.6;
const LINE: f32 = 9.0;
const TACTICS: [&str; 6] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy"];
const TACTIC_LABELS: [&str; 6] = ["Balanced", "All-out Attack", "Focus Weakest", "Guard & Counter", "Kite at Range", "Economy"];
const TACTIC_HELP: [&str; 6] = [
    "The unit's own role: fights, guards when hurt, flees when nearly down.",
    "Nearest enemy, strongest affordable skills, no guarding or fleeing.",
    "Goes for the enemy that is closest to falling.",
    "Guards first, then trades blows.",
    "Holds maximum range and backs away from melee.",
    "Uses level 1 skills to stretch AP over more actions.",
];

// Palette (Godot HUD colours: dark navy panels, gold accents, team colours).
const BG: Rgba = rgb(10, 14, 24);
const PANEL: Rgba = rgba(22, 32, 54, 245);
const PANEL_HI: Rgba = rgba(34, 50, 82, 250);
const CARD: Rgba = rgba(16, 24, 42, 250);
const BORDER: Rgba = rgb(58, 78, 116);
const TEXT: Rgba = rgb(222, 228, 240);
const DIM: Rgba = rgb(132, 146, 172);
const GOLD: Rgba = rgb(244, 198, 84);
const ALLY: Rgba = rgb(110, 170, 255);
const FOE: Rgba = rgb(240, 100, 92);
const GOOD: Rgba = rgb(96, 210, 110);
const RADIUS: f32 = 2.5;

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
    PartyMenu,
    HeroMenu,
    NextRound,
    Step,
    Restart,
    ToggleLog,
}

#[derive(Debug, Clone)]
enum RowAct {
    Step(Value),
    PartyTactic(usize),
    HeroTactic(usize),
}

type Tip = Vec<(String, Rgba)>;

#[derive(Debug, Clone)]
struct Widget {
    id: String,
    kind: &'static str,
    text: String,
    rect: Rect,
    enabled: bool,
    active: bool,
    action: Option<Action>,
    tip: Tip,
}

#[derive(Debug, Clone)]
struct Row {
    label: String,
    detail: String,
    valid: bool,
    act: RowAct,
    tip: Tip,
}

#[derive(Debug, Clone)]
struct Popup {
    title: String,
    x: i32,
    y: i32,
    w: i32,
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
    fonts: Fonts,
    canvas: Canvas,
    view: View,
    widgets: Vec<Widget>,
    mouse: (i32, i32),
    selected: Option<usize>,
    popup: Option<Popup>,
    playing: bool,
    auto: bool,
    next_at: Instant,
    pending_next_round: bool,
    floats: Vec<Float>,
    flashes: Vec<(usize, Instant)>,
    last_hp: Vec<i32>,
    party_tactic: usize,
    hero_tactic: usize,
    message: String,
    show_log: bool,
    log_scroll: i32,
    encounter: String,
    seed: Option<i64>,
    shot_dir: PathBuf,
    tuning_checked: Instant,
}

impl Gui {
    fn session(&self) -> &Session {
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
        self.pending_next_round = false;
        self.floats.clear();
        self.log_scroll = 0;
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
                let Some(row) = p.rows.get(i).filter(|r| r.valid).cloned() else { return };
                match row.act {
                    RowAct::Step(step) => {
                        if let Some(h) = self.selected {
                            let r = self.send(json!({"cmd": "add_step", "unit": h, "step": step}));
                            if r["ok"] == json!(true) {
                                self.message = format!("{}: {}", self.session().w.units[h].name, row.label);
                            }
                        }
                    }
                    RowAct::PartyTactic(t) => {
                        self.party_tactic = t;
                        self.send(json!({"cmd": "preset", "preset": TACTICS[t]}));
                        self.message = format!("Party drafted: {}", TACTIC_LABELS[t]);
                    }
                    RowAct::HeroTactic(t) => {
                        self.hero_tactic = t;
                        if let Some(h) = self.selected {
                            self.send(json!({"cmd": "preset", "unit": h, "preset": TACTICS[t]}));
                            self.message = format!("{} drafted: {}", self.session().w.units[h].name, TACTIC_LABELS[t]);
                        }
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
            Action::PartyMenu => self.tactic_menu(true),
            Action::HeroMenu => self.tactic_menu(false),
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
            Action::ToggleLog => {
                self.show_log = !self.show_log;
                self.log_scroll = 0;
            }
        }
    }

    /// Dropdown of the six tactics under the Party / Hero picker.
    fn tactic_menu(&mut self, party: bool) {
        let current = if party { self.party_tactic } else { self.hero_tactic };
        let rows = (0..TACTICS.len())
            .map(|i| Row {
                label: TACTIC_LABELS[i].to_string(),
                detail: if i == current { "current".into() } else { String::new() },
                valid: true,
                act: if party { RowAct::PartyTactic(i) } else { RowAct::HeroTactic(i) },
                tip: vec![(TACTIC_LABELS[i].to_string(), GOLD), (TACTIC_HELP[i].to_string(), TEXT)],
            })
            .collect();
        let anchor = self.widgets.iter().find(|w| w.id == if party { "strip.party" } else { "strip.hero" }).map(|w| w.rect).unwrap_or_default();
        self.popup = Some(Popup { title: if party { "Draft the whole party".into() } else { "Draft the selected hero".into() }, x: anchor.x, y: anchor.y + anchor.h + 1, w: 120, rows });
    }

    fn unit_click(&mut self, u: usize) {
        let s = self.session();
        let is_ally = s.w.allies.contains(&u);
        let names: Vec<String> = s.w.units.iter().map(|x| x.name.clone()).collect();
        if !self.planning() || self.selected.is_none() {
            if is_ally {
                self.selected = Some(u);
            }
            return;
        }
        let h = self.selected.unwrap();
        if is_ally && u != h {
            let rows = self.options_for(h, "ally", Some(u), None);
            if rows.iter().any(|r| r.valid) {
                self.open_popup(format!("{} -> {}", names[h], names[u]), rows);
            } else {
                self.selected = Some(u);
            }
            return;
        }
        let (title, rows) = if u == h { (format!("{}: own actions", names[h]), self.options_for(h, "self", None, None)) } else { (format!("Attack {}", names[u]), self.options_for(h, "foe", Some(u), None)) };
        self.open_popup(title, rows);
    }

    fn tile_click(&mut self, c: V2) {
        if !self.planning() {
            return;
        }
        let Some(h) = self.selected else { return };
        let rows = self.options_for(h, "tile", None, Some(c));
        self.open_popup(format!("Tile {}, {}", c.x, c.y), rows);
    }

    fn open_popup(&mut self, title: String, rows: Vec<Row>) {
        if rows.is_empty() {
            self.message = "Nothing to do there.".into();
            return;
        }
        let (mx, my) = self.mouse;
        let w = 170;
        let h = 15 + rows.len() as i32 * 11;
        self.popup = Some(Popup { title, x: mx.clamp(2, W - w - 2), y: my.clamp(2, H - h - 2), w, rows });
    }

    /// Candidate steps for hero `h` (`what`: foe / ally / self / tile), each probed through the API.
    fn options_for(&mut self, h: usize, what: &str, target: Option<usize>, cell: Option<V2>) -> Vec<Row> {
        let skills = self.host.handle(&json!({"cmd": "skills", "unit": h}))["skills"].as_array().cloned().unwrap_or_default();
        let mut steps: Vec<(String, Value, Tip)> = Vec::new();
        let target_cell = target.map(|t| {
            let p = self.session().w.pos(t);
            json!([p.x, p.y])
        });
        for sk in &skills {
            let id = sk["skill"].as_str().unwrap_or("").to_string();
            let name = sk["name"].as_str().unwrap_or("").to_string();
            let kind = sk["kind"].as_str().unwrap_or("");
            let area = sk["area"].as_str().unwrap_or("NONE") != "NONE";
            let mut tip: Tip = vec![(name.clone(), GOLD)];
            if let Some(d) = sk["description"].as_str().filter(|d| !d.is_empty()) {
                tip.push((d.to_string(), TEXT));
            }
            tip.push((format!("{} AP, {} block{}, reach {}-{}", sk["ap"], sk["blocks"], if sk["blocks"] == json!(1) { "" } else { "s" }, sk["reach"][0], sk["reach"][1]), DIM));
            match what {
                "foe" => match kind {
                    "ATTACK" if area => steps.push((name, json!({"mode": "area", "skill": id, "cell": target_cell}), tip)),
                    "ATTACK" | "DISENGAGE" => steps.push((name, json!({"mode": "engage", "skill": id, "target": target}), tip)),
                    _ => {}
                },
                "ally" if kind == "SUPPORT" => steps.push((name, json!({"mode": "ally", "skill": id, "target": target}), tip)),
                "self" if matches!(kind, "DEFEND" | "FOCUS" | "REST" | "DISENGAGE" | "SUPPORT") => steps.push((name, json!({"mode": "self", "skill": id}), tip)),
                "tile" if kind == "ATTACK" && area => steps.push((name, json!({"mode": "area", "skill": id, "cell": cell.map(|c| json!([c.x, c.y]))}), tip)),
                _ => {}
            }
        }
        if what == "tile" {
            if let Some(c) = cell {
                steps.insert(0, ("Move here".into(), json!({"mode": "goto", "cell": [c.x, c.y]}), vec![("Move here".into(), GOLD), ("1 AP per tile; moving must come first in a round.".into(), TEXT)]));
            }
        }
        let mut rows = Vec::new();
        for (label, step, mut tip) in steps {
            let p = self.host.handle(&json!({"cmd": "probe", "unit": h, "step": step}));
            let valid = p["valid"] == json!(true);
            let detail = if valid {
                let es = p["entries"].as_array().cloned().unwrap_or_default();
                let dmg: i64 = es.iter().map(|e| e["est_damage"].as_i64().unwrap_or(0)).sum();
                let heal: i64 = es.iter().map(|e| e["est_heal"].as_i64().unwrap_or(0)).sum();
                let first = es.first().map(|e| e["start"].as_i64().unwrap_or(0)).unwrap_or(0);
                let last = es.last().map(|e| e["end"].as_i64().unwrap_or(0)).unwrap_or(0);
                tip.push((format!("Blocks {first}-{last}, AP left after: {}", p["ap_left"]), ALLY));
                let mut d = format!("{} AP", p["ap"].as_i64().unwrap_or(0));
                if dmg > 0 {
                    d.push_str(&format!("  ~{dmg} dmg"));
                }
                if heal > 0 {
                    d.push_str(&format!("  +{heal} HP"));
                }
                d
            } else {
                let r = p["reason"].as_str().unwrap_or("").to_string();
                tip.push((r.clone(), FOE));
                r
            };
            rows.push(Row { label, detail, valid, act: RowAct::Step(step), tip });
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
        if self.pending_next_round {
            if now >= self.next_at && self.session().phase == Phase::Replay {
                self.pending_next_round = false;
                self.act(Action::NextRound);
            }
            return;
        }
        if self.playing && now >= self.next_at {
            if self.session().phase == Phase::Replay {
                self.playing = false;
                return;
            }
            self.send(json!({"cmd": "step"}));
            self.next_at = now + Duration::from_millis(self.tuning.value.step_ms);
            if self.session().phase == Phase::Replay {
                self.playing = false;
                let s = self.session();
                let can_advance = s.can_advance_round();
                let message = if s.combat_over {
                    match s.outcome.as_str() {
                        "won" => "Victory!".to_string(),
                        "lost" => "Defeat.".to_string(),
                        o => format!("Battle over: {o}"),
                    }
                } else if !can_advance {
                    "Round limit reached.".into()
                } else {
                    "Round over. Space (or Next) for the next round.".into()
                };
                self.message = message;
                if self.auto && can_advance {
                    self.next_at = now + Duration::from_millis(self.tuning.value.round_pause_ms);
                    self.pending_next_round = true;
                }
            }
        }
    }

    fn resize(&mut self, pw: u32, ph: u32) {
        self.canvas = Canvas::new(pw as i32, ph as i32);
        self.view = View::fit(pw, ph);
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
            if p.frame().len() == gui.canvas.px.len() {
                p.frame_mut().copy_from_slice(&gui.canvas.px);
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
        let attrs = Window::default_attributes().with_title("Aegis - battle").with_inner_size(winit::dpi::LogicalSize::new(1280, 720)).with_min_inner_size(winit::dpi::LogicalSize::new(640, 360));
        let window = Arc::new(el.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let pixels = Pixels::new(size.width.max(1), size.height.max(1), SurfaceTexture::new(size.width.max(1), size.height.max(1), window.clone())).expect("pixels");
        let tuning_path = self.engine_dir.as_ref().map(|d| d.join("data/tuning.json"));
        let font_bytes = self.engine_dir.as_ref().and_then(|d| std::fs::read(d.join("assets/fonts/OpenSans-SemiBold.ttf")).ok()).expect("engine/assets/fonts/OpenSans-SemiBold.ttf");
        let mut gui = Gui {
            host: Host::new(self.cat.clone()),
            art: art::Art::new(self.project.clone()),
            tuning: tuning::TuningFile::new(tuning_path),
            fonts: Fonts::new(&font_bytes),
            canvas: Canvas::new(size.width.max(1) as i32, size.height.max(1) as i32),
            view: View::fit(size.width.max(1), size.height.max(1)),
            widgets: Vec::new(),
            mouse: (0, 0),
            selected: None,
            popup: None,
            playing: false,
            auto: false,
            next_at: Instant::now(),
            pending_next_round: false,
            floats: Vec::new(),
            flashes: Vec::new(),
            last_hp: Vec::new(),
            party_tactic: 0,
            hero_tactic: 0,
            message: String::new(),
            show_log: false,
            log_scroll: 0,
            encounter: arg(&self.args, "encounter").unwrap_or("goblin_skirmish").to_string(),
            seed: arg(&self.args, "seed").and_then(|s| s.parse().ok()),
            shot_dir: self.engine_dir.clone().map(|d| d.join("parity/out")).unwrap_or_else(|| PathBuf::from(".")),
            tuning_checked: Instant::now(),
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
                let (w, h) = (size.width.max(1), size.height.max(1));
                if let Some(p) = self.pixels.as_mut() {
                    let _ = p.resize_surface(w, h);
                    let _ = p.resize_buffer(w, h);
                }
                if let Some(g) = self.gui.as_mut() {
                    g.resize(w, h);
                }
            }
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(g) = self.gui.as_mut() {
                    g.mouse = g.view.to_logical(position.x as f32, position.y as f32);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(g) = self.gui.as_mut() {
                    let lines = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y.round() as i32,
                        MouseScrollDelta::PixelDelta(p) => (p.y / 20.0).round() as i32,
                    };
                    if g.show_log {
                        g.log_scroll = (g.log_scroll + lines * 3).max(0);
                    }
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
                        } else if g.show_log {
                            g.show_log = false;
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
                    Key::Named(NamedKey::Escape) => {
                        g.popup = None;
                        g.show_log = false;
                    }
                    Key::Named(NamedKey::PageUp) if g.show_log => g.log_scroll += 10,
                    Key::Named(NamedKey::PageDown) if g.show_log => g.log_scroll = (g.log_scroll - 10).max(0),
                    Key::Named(NamedKey::F12) => {
                        let p = g.shot_dir.join("gui_shot.png");
                        g.message = match save_png(&g.canvas, &p, 0) {
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
        "ui_tree" => json!({"ok": true, "canvas": [W, H], "mouse": [g.mouse.0, g.mouse.1], "message": g.message, "playing": g.playing, "auto": g.auto, "log_open": g.show_log,
            "popup": g.popup.as_ref().map(|p| json!({"title": p.title, "rows": p.rows.iter().map(|r| json!({"label": r.label, "detail": r.detail, "valid": r.valid})).collect::<Vec<_>>()})),
            "widgets": g.widgets.iter().map(|w| json!({"id": w.id, "kind": w.kind, "text": w.text, "rect": [w.rect.x, w.rect.y, w.rect.w, w.rect.h], "enabled": w.enabled, "active": w.active, "clickable": w.action.is_some(), "tip": w.tip.iter().map(|t| t.0.clone()).collect::<Vec<_>>()})).collect::<Vec<_>>()}),
        "screenshot" => {
            let path = body.get("path").and_then(Value::as_str).map(PathBuf::from).unwrap_or_else(|| g.shot_dir.join("gui_shot.png"));
            let max_w = body.get("max_w").and_then(Value::as_u64).unwrap_or(1280) as i32;
            match save_png(&g.canvas, &path, max_w) {
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
        "hover" => {
            let id = body.get("id").and_then(Value::as_str).unwrap_or("");
            match g.widgets.iter().find(|w| w.id == id) {
                Some(w) => {
                    g.mouse = (w.rect.x + w.rect.w / 2, w.rect.y + w.rect.h / 2);
                    json!({"ok": true, "tip": w.tip.iter().map(|t| t.0.clone()).collect::<Vec<_>>()})
                }
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

// ---- Drawing ------------------------------------------------------------------------------------------------------

struct Ui<'a> {
    p: Painter<'a>,
    widgets: Vec<Widget>,
    mouse: (i32, i32),
}

impl<'a> Ui<'a> {
    #[allow(clippy::too_many_arguments)]
    fn push(&mut self, id: impl Into<String>, kind: &'static str, text: impl Into<String>, rect: Rect, enabled: bool, active: bool, action: Option<Action>, tip: Tip) {
        self.widgets.push(Widget { id: id.into(), kind, text: text.into(), rect, enabled, active, action, tip });
    }

    fn hover(&self, r: Rect) -> bool {
        r.contains(self.mouse.0, self.mouse.1)
    }

    #[allow(clippy::too_many_arguments)]
    fn button(&mut self, id: &str, r: Rect, label: &str, action: Action, enabled: bool, active: bool, tip: Tip) {
        let hover = enabled && self.hover(r);
        let fill = if active { rgba(86, 68, 22, 250) } else if hover { PANEL_HI } else { CARD };
        let border = if active { GOLD } else if hover { rgb(110, 140, 196) } else { BORDER };
        self.p.panel(r, fill, border, RADIUS);
        let col = if enabled { if active { GOLD } else { TEXT } } else { DIM };
        let t = self.p.fit(label, BODY, r.w as f32 - 6.0);
        self.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - LINE) / 2.0, &t, BODY, col);
        self.push(id, "button", label, r, enabled, active, Some(action), tip);
    }
}

impl<'a> Ui<'a> {
    /// A dropdown picker: label on the left, a chevron on the right.
    fn dropdown(&mut self, id: &str, r: Rect, label: &str, action: Action, enabled: bool, tip: Tip) {
        let hover = enabled && self.hover(r);
        self.p.panel(r, if hover { PANEL_HI } else { CARD }, if hover { rgb(110, 140, 196) } else { BORDER }, RADIUS);
        let col = if enabled { TEXT } else { DIM };
        self.p.text_fit(r.x as f32 + 4.0, r.y as f32 + (r.h as f32 - LINE) / 2.0, label, BODY, r.w as f32 - 16.0, col);
        let (cx, cy) = (r.x as f32 + r.w as f32 - 7.0, r.y as f32 + r.h as f32 / 2.0);
        self.p.line(cx - 2.5, cy - 1.2, cx, cy + 1.3, col, 0.0);
        self.p.line(cx, cy + 1.3, cx + 2.5, cy - 1.2, col, 0.0);
        self.push(id, "dropdown", label, r, enabled, false, Some(action), tip);
    }
}

fn kind_color(kind: &str) -> Rgba {
    match kind {
        "ATTACK" => rgb(160, 56, 56),
        "MOVE" => rgb(52, 92, 160),
        "DEFEND" => rgb(40, 122, 122),
        "SUPPORT" => rgb(52, 128, 68),
        "FOCUS" => rgb(128, 98, 34),
        "REST" => rgb(74, 80, 96),
        "STATUS" => rgb(104, 56, 128),
        "TARGET" => rgb(92, 76, 120),
        "DISENGAGE" => rgb(56, 116, 160),
        _ => rgb(60, 66, 84),
    }
}

fn hp_bar(p: &mut Painter, r: Rect, hp: i32, max: i32, color: Rgba) {
    p.fill(r, rgb(46, 24, 30));
    let w = if max > 0 { (r.w * hp.max(0) + max - 1) / max } else { 0 };
    p.fill(Rect::new(r.x, r.y, w.min(r.w), r.h), color);
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

fn stance_name(u: &aegis_rules::Unit) -> &'static str {
    ["Neutral", "Offensive", "Defensive", "Tactical"][u.stance as usize]
}

/// The actions the resolver holds for `u` this round (after confirm): (start, end, name, kind, target name).
fn committed(s: &Session, u: usize) -> Vec<(i32, i32, String, String, Option<String>)> {
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
            let name = if a.cancelled { format!("{} (cancelled)", a.command.name) } else { a.command.name.clone() };
            (a.start, a.end, name, kind.to_string(), target.filter(|t| *t != u).map(|t| s.w.units[t].name.clone()))
        })
        .collect()
}

/// A unit's next (or current) action this round, for cards and hover cards.
fn next_action(s: &Session, e: usize) -> String {
    if s.phase == Phase::Replay {
        return "-".into();
    }
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    match s.resolver.scheduled.iter().find(|a| a.unit == e && !a.cancelled && (planning || a.end >= s.current_block)) {
        Some(a) => match a.command.kind {
            CmdKind::Attack { target: Some(t), .. } => format!("{} on {} (B{}-{})", a.command.name, short_name(&s.w.units[t].name), a.start, a.end),
            _ => format!("{} (B{}-{})", a.command.name, a.start, a.end),
        },
        None => "-".into(),
    }
}

/// Hover card of unit `i`.
fn unit_tip(s: &Session, i: usize, sel: Option<usize>) -> Tip {
    let w = &s.w;
    let u = &w.units[i];
    let ally = w.allies.contains(&i);
    let mut t: Tip = vec![(u.name.clone(), if ally { ALLY } else { FOE })];
    t.push((format!("HP {}/{}   AP {}/{}   {}", u.hp, u.max_hp, u.ap, u.max_ap, stance_name(u)), TEXT));
    t.push((format!("Level {} {}   Speed {}   Reach {}", u.stats.level, u.tier, u.speed, u.attack_range), DIM));
    if !u.st.active.is_empty() {
        t.push((format!("Now: {}", u.st.active.iter().map(|x| aegis_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), FOE));
    }
    if !u.st.pending.is_empty() {
        t.push((format!("Next round: {}", u.st.pending.iter().map(|x| aegis_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), GOLD));
    }
    if !ally && u.is_active() {
        t.push((format!("Next: {}", next_action(s, i)), GOLD));
        if let Some(h) = sel.filter(|h| w.units[*h].is_active()) {
            let strike = w.book().lowest(Kind::Attack);
            t.push((format!("{} deals about {} with a Strike, takes about {}", w.units[h].name, expected(w, h, i, strike, None, 0.0, 1.0), expected(w, i, h, strike, None, 0.0, 1.0)), ALLY));
        }
    }
    if !u.is_alive() {
        t.push(("Down".into(), DIM));
    } else if u.is_withdrawn {
        t.push(("Left the battle".into(), DIM));
    }
    t
}

fn draw(g: &mut Gui) {
    let now = Instant::now();
    let mouse = g.mouse;
    let tuning = g.tuning.value.clone();
    g.canvas.clear(BG);
    let view = g.view;
    let mut ui = Ui { p: Painter { c: &mut g.canvas, v: view, fonts: &mut g.fonts }, widgets: Vec::new(), mouse };
    let Some(s) = g.host.session.as_ref() else {
        ui.p.text(8.0, 8.0, &g.message, BODY, FOE);
        g.widgets = ui.widgets;
        return;
    };
    let w = &s.w;
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    let sel = g.selected.filter(|u| w.allies.contains(u));

    // -- Top strip: the selected hero's six blocks and the round controls.
    ui.p.panel(Rect::new(2, 2, W - 4, 24), PANEL, BORDER, RADIUS);
    let hero_name = sel.map(|u| w.units[u].name.clone()).unwrap_or_else(|| "No hero".into());
    ui.p.text_fit(8.0, 9.0, &format!("{hero_name} plan"), TITLE, 80.0, GOLD);
    ui.push("strip.title", "label", format!("{hero_name} plan"), Rect::new(8, 8, 80, 10), true, false, None, Vec::new());
    let entries: Vec<(i32, i32, String, String, Option<String>, i32, i32)> = match sel {
        Some(u) if s.plan_confirmed => committed(s, u).into_iter().map(|c| (c.0, c.1, c.2, c.3, c.4, 0, 0)).collect(),
        Some(u) if w.units[u].is_active() => s
            .plan
            .compile(w, u)
            .entries
            .iter()
            .map(|e| {
                let sk = &w.cat.skills[e.skill];
                (e.start_block, e.end_block(w), sk.name.clone(), sk.kind.clone(), e.target.filter(|t| *t != u).map(|t| w.units[t].name.clone()), e.total_ap(), e.est_damage)
            })
            .collect(),
        _ => Vec::new(),
    };
    let (bx, bw) = (92, 34);
    let live_block = if !planning && s.phase == Phase::Resolving { s.current_block } else { 0 };
    for b in 1..=s.def.total_blocks {
        let r = Rect::new(bx + (b - 1) * bw, 6, bw - 1, 16);
        let entry = entries.iter().find(|e| b >= e.0 && b <= e.1);
        let mut tip: Tip = Vec::new();
        match entry {
            Some(e) => {
                tip.push((e.2.clone(), GOLD));
                tip.push((format!("Blocks {}-{}", e.0, e.1), TEXT));
                if let Some(t) = &e.4 {
                    tip.push((format!("Target: {t}"), FOE));
                }
                if e.5 > 0 {
                    tip.push((format!("{} AP", e.5), ALLY));
                }
                if e.6 > 0 {
                    tip.push((format!("Expected damage about {}", e.6), GOLD));
                }
            }
            None => {
                ui.p.panel(r, rgba(14, 20, 34, 255), if live_block == b { GOLD } else { BORDER }, RADIUS);
                ui.p.text(r.x as f32 + 3.0, r.y as f32 + 4.0, &format!("{b} Idle"), SMALL, DIM);
                tip.push((format!("Block {b}: nothing planned"), DIM));
            }
        }
        ui.push(format!("strip.block{b}"), "block", entry.map(|e| e.2.clone()).unwrap_or_else(|| "empty".into()), r, true, live_block == b, None, tip);
    }
    // Each action is one continuous span over its blocks, coloured by kind (COMBAT-UI3).
    for e in &entries {
        let r = Rect::new(bx + (e.0 - 1) * bw, 6, (e.1 - e.0 + 1) * bw - 1, 16);
        let live = live_block >= e.0 && live_block <= e.1;
        ui.p.panel(r, kind_color(&e.3), if live { GOLD } else { rgba(255, 255, 255, 50) }, RADIUS);
        ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 4.0, &e.2, SMALL, r.w as f32 - 6.0, TEXT);
    }
    let mut x = bx + 6 * bw + 6;
    ui.dropdown("strip.party", Rect::new(x, 6, 82, 16), &format!("Party: {}", TACTIC_LABELS[g.party_tactic]), Action::PartyMenu, planning, vec![("Party tactic".into(), GOLD), ("AI-drafts every hero with a tactic (you can edit the steps after).".into(), TEXT)]);
    x += 84;
    ui.dropdown("strip.hero", Rect::new(x, 6, 80, 16), &format!("Hero: {}", TACTIC_LABELS[g.hero_tactic]), Action::HeroMenu, planning && sel.is_some(), vec![("Hero tactic".into(), GOLD), ("AI-drafts the selected hero only.".into(), TEXT)]);
    x += 82;
    ui.button("strip.auto", Rect::new(x, 6, 34, 16), "Auto", Action::Autoplay, !s.combat_over, g.auto, vec![("Autoplay".into(), GOLD), ("Drafts with the party tactic, confirms and plays rounds until the battle ends. Click again to stop.".into(), TEXT)]);
    x += 36;
    ui.button("strip.clear", Rect::new(x, 6, 34, 16), "Clear", Action::Clear, planning && sel.is_some(), false, vec![("Clears the selected hero's steps.".into(), TEXT)]);
    x += 36;
    let (label, action, enabled) = if planning {
        ("Confirm round", Action::Confirm, true)
    } else if s.phase == Phase::Replay && s.can_advance_round() {
        ("Next round", Action::NextRound, !g.pending_next_round)
    } else {
        (if s.combat_over { "Battle over" } else if s.phase == Phase::Replay { "Round limit" } else { "Resolving" }, Action::NextRound, false)
    };
    ui.button("strip.confirm", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled, planning, vec![(label.into(), GOLD)]);

    // -- Left: party cards and the selected hero.
    ui.p.panel(Rect::new(2, 28, 124, 290), PANEL, BORDER, RADIUS);
    let alive = w.allies.iter().filter(|a| w.units[**a].is_active()).count();
    ui.p.text(8.0, 32.0, &format!("Party {alive}/{}", w.allies.len()), TITLE, ALLY);
    let mut y = 44;
    for &a in &w.allies {
        let u = &w.units[a];
        let r = Rect::new(6, y, 116, 30);
        let is_sel = Some(a) == sel;
        let hover = ui.hover(r);
        ui.p.panel(r, if is_sel { PANEL_HI } else if hover { rgba(28, 40, 66, 255) } else { CARD }, if is_sel { GOLD } else { BORDER }, RADIUS);
        ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 1.5, &u.name, BODY, 108.0, if u.is_active() { ALLY } else { DIM });
        hp_bar(&mut ui.p, Rect::new(r.x + 4, r.y + 12, 108, 4), u.hp, u.max_hp, GOOD);
        ui.p.text(r.x as f32 + 4.0, r.y as f32 + 18.0, &format!("HP {}/{}", u.hp, u.max_hp), SMALL, TEXT);
        ui.p.text_right(r.x as f32 + r.w as f32 - 4.0, r.y as f32 + 18.0, &format!("AP {}/{}", u.ap, u.max_ap), SMALL, ALLY);
        let tip = unit_tip(s, a, sel);
        ui.push(format!("party.{a}"), "card", format!("{} HP {}/{} AP {}/{}", u.name, u.hp, u.max_hp, u.ap, u.max_ap), r, true, is_sel, Some(Action::Select(a)), tip);
        y += 33;
    }
    if let Some(h) = sel {
        let u = &w.units[h];
        let r = Rect::new(6, y + 2, 116, 318 - y - 6);
        ui.p.panel(r, CARD, BORDER, RADIUS);
        ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 2.0, &u.name, BODY, 64.0, GOLD);
        ui.p.text_right(r.x as f32 + r.w as f32 - 4.0, r.y as f32 + 2.0, stance_name(u), SMALL, DIM);
        let st = s.plan.compile(w, h);
        let weapon = u.inventory.main_weapon().map(|e| e.item.name.clone()).unwrap_or_else(|| "Bare hands".into());
        let mut lines: Vec<(String, Rgba)> = vec![
            (format!("Speed {}  Reach {}", u.speed, u.attack_range), TEXT),
            (format!("AP left {}/{}", if s.plan_confirmed { u.ap } else { st.ap_left }, u.max_ap), ALLY),
            (format!("Gear: {weapon}"), GOLD),
        ];
        if !u.st.active.is_empty() {
            lines.push((format!("Now: {}", u.st.active.iter().map(|x| aegis_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), FOE));
        }
        if !u.st.pending.is_empty() {
            lines.push((format!("Next: {}", u.st.pending.iter().map(|x| aegis_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), DIM));
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
            let empty: Vec<String> = (1..=s.def.total_blocks).filter(|b| !st.entries.iter().any(|e| *b >= e.start_block && *b <= e.end_block(w))).map(|b| b.to_string()).collect();
            if !empty.is_empty() && u.is_active() {
                lines.push((format!("Empty: B{}", empty.join(", B")), GOLD));
            }
            if st.error_index >= 0 {
                lines.push((st.error_text.clone(), FOE));
            }
        }
        let mut ly = r.y as f32 + 12.0;
        for (text, col) in lines {
            for part in ui.p.wrap(&text, SMALL, 108.0) {
                if ly > (r.y + r.h) as f32 - 9.0 {
                    break;
                }
                ui.p.text(r.x as f32 + 4.0, ly, &part, SMALL, col);
                ly += 8.0;
            }
        }
        ui.push("hero.panel", "panel", u.name.clone(), r, true, false, None, Vec::new());
    }

    // -- Grid (pixel layer): terrain, exits, plan ghost, intents, units, floating numbers.
    ui.p.panel(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4), rgb(8, 10, 16), BORDER, RADIUS);
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let r = cell_rect(c);
            let tid = w.grid.terrain_id(c);
            match g.art.tile(tid, gx, gy, TILE) {
                Some(img) => ui.p.image(img, r),
                None => ui.p.fill(r, terrain_color(tid)),
            }
            if w.grid.is_exit(c) {
                ui.p.panel(r.inset(7), rgba(70, 200, 90, 220), rgba(200, 255, 210, 255), 2.0);
                ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "EXIT", SMALL, BG);
            }
            if planning && sel.is_some() && ui.hover(r) && g.popup.is_none() {
                ui.p.panel(r.inset(1), rgba(255, 255, 255, 30), rgba(255, 255, 255, 160), 1.5);
            }
            let tip: Tip = if w.grid.is_exit(c) { vec![("Exit".into(), GOOD), ("A unit that steps here leaves the battle.".into(), TEXT)] } else { Vec::new() };
            ui.push(format!("tile.{gx}.{gy}"), "tile", tid, r, planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None }, tip);
        }
    }
    let center = |c: V2| ((GRID_X + c.x * TILE + TILE / 2) as f32, (GRID_Y + c.y * TILE + TILE / 2) as f32);
    if let Some(h) = sel {
        if w.units[h].is_active() && planning {
            let st = s.plan.compile(w, h);
            let mut from = w.pos(h);
            for e in &st.entries {
                if e.pos_after != from && e.pos_after.x >= 0 {
                    let (a, b) = (center(from), center(e.pos_after));
                    ui.p.line(a.0, a.1, b.0, b.1, GOLD, 3.0);
                    ui.p.panel(cell_rect(e.pos_after).inset(3), rgba(244, 198, 84, 40), GOLD, 2.0);
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
                ui.p.line(p.0, p.1, q.0, q.1, rgba(240, 100, 92, 210), 4.0);
            }
        }
    }
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let r = cell_rect(pos);
        let ally = w.allies.contains(&i);
        if !u.is_alive() {
            ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "x", BODY, DIM);
            continue;
        }
        let flash = g.flashes.iter().any(|f| f.0 == i);
        let shake = if flash { tuning.hit_shake_px as f32 * if (now.elapsed().as_millis() / 30) % 2 == 0 { 1.0 } else { -1.0 } } else { 0.0 };
        let tp = tuning.token_px;
        let tr = Rect::new(r.x + (TILE - tp) / 2 + shake as i32, r.y + (TILE - tp) / 2, tp, tp);
        let ring = if ally { ALLY } else { FOE };
        ui.p.circle(r.x as f32 + 16.0 + shake, r.y as f32 + 16.0, tp as f32 / 2.0 + 0.5, ring);
        match g.art.token(&s.art[i], tp) {
            Some(img) => ui.p.image(img, tr),
            None => {
                ui.p.panel(tr.inset(4), ring, ring, 6.0);
                ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, &u.name.chars().next().unwrap_or('?').to_string(), BODY, BG);
            }
        }
        if flash {
            ui.p.fill(tr, rgba(255, 255, 255, 110));
        }
        hp_bar(&mut ui.p, Rect::new(r.x + 3, r.y + TILE - 3, TILE - 6, 2), u.hp, u.max_hp, if ally { GOOD } else { FOE });
        if Some(i) == sel {
            ui.p.panel(r, rgba(0, 0, 0, 0), GOLD, 2.0);
        }
        let tip = unit_tip(s, i, sel);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), r, true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    for f in &g.floats {
        if let Some(pos) = w.grid.pos(f.unit) {
            let t = now.duration_since(f.born).as_millis() as f32 / tuning.float_ms.max(1) as f32;
            let (cx, cy) = center(pos);
            let yy = cy - 14.0 - tuning.float_rise_px as f32 * t;
            ui.p.text_center(cx + 0.6, yy + 0.6, &f.text, TITLE, rgba(0, 0, 0, 220));
            ui.p.text_center(cx, yy, &f.text, TITLE, f.color);
        }
    }

    // -- Right: hostile cards and the hovered (or first) foe's detail card.
    let right = Rect::new(W - 126, 28, 124, 290);
    ui.p.panel(right, PANEL, BORDER, RADIUS);
    let foes_active = w.enemies.iter().filter(|e| w.units[**e].is_active()).count();
    ui.p.text(right.x as f32 + 6.0, 32.0, &format!("Hostiles {foes_active}"), TITLE, FOE);
    let mut y = 44;
    let mut hovered_foe = None;
    for &e in &w.enemies {
        let u = &w.units[e];
        if w.grid.pos(e).is_none() && u.is_alive() && !u.is_withdrawn {
            continue; // reinforcements not on the field yet
        }
        if y > 206 {
            break;
        }
        let r = Rect::new(right.x + 4, y, 116, 22);
        let hover = ui.hover(r) || w.grid.pos(e).is_some_and(|p| ui.hover(cell_rect(p)));
        if hover {
            hovered_foe = Some(e);
        }
        ui.p.panel(r, if hover { rgba(48, 28, 38, 255) } else { CARD }, if hover { FOE } else { BORDER }, RADIUS);
        let col = if u.is_active() { FOE } else { DIM };
        ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 1.5, &short_name(&u.name), BODY, 70.0, col);
        ui.p.text_right(r.x as f32 + r.w as f32 - 3.0, r.y as f32 + 1.5, &format!("{}/{}", u.hp, u.max_hp), BODY, if u.is_active() { GOOD } else { DIM });
        if u.is_active() {
            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 11.5, &format!("Next: {}", next_action(s, e)), SMALL, 110.0, GOLD);
        } else {
            ui.p.text(r.x as f32 + 3.0, r.y as f32 + 11.5, if u.is_withdrawn { "Fled" } else { "Down" }, SMALL, DIM);
        }
        let tip = unit_tip(s, e, sel);
        ui.push(format!("foe.{e}"), "card", format!("{} {}/{} next {}", u.name, u.hp, u.max_hp, next_action(s, e)), r, true, hover, Some(Action::UnitClick(e)), tip);
        y += 24;
    }
    let detail_foe = hovered_foe.or_else(|| w.enemies.iter().copied().find(|e| w.units[*e].is_active() && w.grid.pos(*e).is_some()));
    if let Some(e) = detail_foe {
        let r = Rect::new(right.x + 4, y + 2, 116, 318 - y - 6);
        ui.p.panel(r, CARD, FOE, RADIUS);
        let mut ly = r.y as f32 + 3.0;
        for (i, (t, col)) in unit_tip(s, e, sel).into_iter().enumerate() {
            for part in ui.p.wrap(&t, if i == 0 { BODY } else { SMALL }, 108.0) {
                if ly > (r.y + r.h) as f32 - 9.0 {
                    break;
                }
                ui.p.text(r.x as f32 + 4.0, ly, &part, if i == 0 { BODY } else { SMALL }, col);
                ly += if i == 0 { 10.0 } else { 8.0 };
            }
        }
    }

    // -- Bottom bar: playback, round and status, the last log line.
    ui.p.panel(Rect::new(2, 322, W - 4, 36), PANEL, BORDER, RADIUS);
    ui.button("bar.restart", Rect::new(6, 325, 40, 14), "Restart", Action::Restart, true, false, vec![("Restart the battle (R)".into(), TEXT)]);
    ui.button("bar.step", Rect::new(48, 325, 34, 14), "Step", Action::Step, !planning || s.plan_confirmed, false, vec![("Resolve one action (Space)".into(), TEXT)]);
    let phase = match s.phase {
        Phase::Planning => "Planning",
        Phase::Resolving => "Resolving",
        Phase::Replay => "Round over",
    };
    ui.p.text(88.0, 326.0, &format!("Round {}/{}  {phase}", s.current_round, s.def.max_rounds), BODY, GOLD);
    ui.p.text_fit(200.0, 326.0, &g.message, BODY, 430.0, TEXT);
    ui.push("bar.status", "label", format!("Round {} {phase}: {}", s.current_round, g.message), Rect::new(88, 325, 540, 12), true, false, None, Vec::new());
    let last = s.resolver.log.last().cloned().unwrap_or_else(|| "No actions yet".into());
    let log_r = Rect::new(6, 341, W - 64, 14);
    ui.p.panel(log_r, CARD, BORDER, RADIUS);
    ui.p.text_fit(10.0, 342.5, &last, SMALL, (W - 72) as f32, DIM);
    ui.push("bar.last", "label", last.clone(), log_r, true, false, Some(Action::ToggleLog), vec![(last, TEXT), ("Click to open the full log.".into(), DIM)]);
    ui.button("bar.log", Rect::new(W - 54, 341, 48, 14), if g.show_log { "Close" } else { "Expand" }, Action::ToggleLog, true, g.show_log, vec![("Full combat log (L); the wheel scrolls".into(), TEXT)]);

    if s.combat_over {
        let r = Rect::new(GRID_X + 92, GRID_Y + 110, 200, 56);
        ui.p.panel(r, rgba(10, 14, 22, 240), GOLD, 4.0);
        let (title, col) = match s.outcome.as_str() {
            "won" => ("Victory", GOOD),
            "lost" => ("Defeat", FOE),
            _ => ("Battle over", GOLD),
        };
        ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + 8.0, title, 11.0, col);
        ui.button("banner.restart", Rect::new(r.x + 64, r.y + 32, 72, 16), "Play again", Action::Restart, true, false, Vec::new());
    }

    if g.show_log {
        let r = Rect::new(GRID_X, GRID_Y + 4, GRID_W * TILE, GRID_H * TILE - 8);
        ui.push("log.panel", "panel", format!("{} lines", s.resolver.log.len()), r, true, true, Some(Action::Step), Vec::new());
        ui.p.panel(r, rgba(10, 14, 24, 246), BORDER, 3.0);
        ui.p.text(r.x as f32 + 6.0, r.y as f32 + 3.0, &format!("Combat log ({} entries)", s.resolver.log.len()), TITLE, GOLD);
        let mut wrapped: Vec<(String, Rgba)> = Vec::new();
        for l in &s.resolver.log {
            let col = if l.starts_with("[Attack]") { TEXT } else if l.starts_with("[Support]") { GOOD } else if l.contains("Failed") || l.contains("Out of Range") { FOE } else { DIM };
            for (k, part) in ui.p.wrap(l, SMALL, (r.w - 22) as f32).into_iter().enumerate() {
                wrapped.push((if k == 0 { part } else { format!("   {part}") }, col));
            }
        }
        let visible = ((r.h - 18) as f32 / 8.0) as i32;
        let max_scroll = (wrapped.len() as i32 - visible).max(0);
        g.log_scroll = g.log_scroll.min(max_scroll);
        let start = (wrapped.len() as i32 - visible - g.log_scroll).max(0) as usize;
        for (k, (t, col)) in wrapped.iter().skip(start).take(visible as usize).enumerate() {
            ui.p.text(r.x as f32 + 6.0, r.y as f32 + 15.0 + k as f32 * 8.0, t, SMALL, *col);
        }
        if max_scroll > 0 {
            let track = Rect::new(r.x + r.w - 8, r.y + 15, 4, r.h - 19);
            ui.p.panel(track, rgba(30, 40, 60, 255), rgba(30, 40, 60, 255), 1.5);
            let thumb_h = (track.h * visible / wrapped.len() as i32).max(8);
            let pos = (track.h - thumb_h) * (max_scroll - g.log_scroll) / max_scroll;
            ui.p.panel(Rect::new(track.x, track.y + pos, 4, thumb_h), GOLD, GOLD, 1.5);
        }
    }

    if let Some(pop) = g.popup.clone() {
        let h = 15 + pop.rows.len() as i32 * 11;
        let r = Rect::new(pop.x, pop.y, pop.w, h);
        ui.push("popup.backdrop", "backdrop", "", Rect::new(0, 0, W, H), true, false, Some(Action::ClosePopup), Vec::new());
        ui.p.panel(r, rgba(14, 20, 34, 252), GOLD, 3.0);
        ui.p.text_fit(r.x as f32 + 5.0, r.y as f32 + 2.0, &pop.title, BODY, r.w as f32 - 10.0, GOLD);
        for (k, row) in pop.rows.iter().enumerate() {
            let rr = Rect::new(r.x + 2, r.y + 13 + k as i32 * 11, r.w - 4, 11);
            let hover = ui.hover(rr);
            if hover && row.valid {
                ui.p.panel(rr, PANEL_HI, rgb(110, 140, 196), 1.5);
            }
            let col = if row.valid { TEXT } else { DIM };
            ui.p.text_fit(rr.x as f32 + 4.0, rr.y as f32 + 1.5, &row.label, BODY, 78.0, col);
            ui.p.text_fit(rr.x as f32 + 84.0, rr.y as f32 + 2.0, &row.detail, SMALL, rr.w as f32 - 86.0, if row.valid { GOLD } else { DIM });
            ui.push(format!("popup.{k}"), "option", format!("{} {}", row.label, row.detail), rr, row.valid, hover, Some(Action::Choose(k)), row.tip.clone());
        }
    }

    // Hover card of the topmost hovered widget that has one.
    let hovered = ui.widgets.iter().rev().find(|wd| wd.rect.contains(mouse.0, mouse.1) && (!wd.tip.is_empty() || wd.kind == "backdrop")).cloned();
    if let Some(wd) = hovered.filter(|wd| !wd.tip.is_empty()) {
        let max_w = 150.0;
        let mut lines: Vec<(String, Rgba, f32)> = Vec::new();
        for (i, (t, col)) in wd.tip.iter().enumerate() {
            let size = if i == 0 { BODY } else { SMALL };
            for part in ui.p.wrap(t, size, max_w) {
                lines.push((part, *col, size));
            }
        }
        let tw = lines.iter().map(|l| ui.p.text_w(&l.0, l.2)).fold(0.0f32, f32::max) + 10.0;
        let th = lines.iter().map(|l| if l.2 == BODY { 10.0 } else { 8.0 }).sum::<f32>() + 6.0;
        let (mut tx, mut ty) = (mouse.0 + 10, mouse.1 + 12);
        if tx as f32 + tw > W as f32 - 2.0 {
            tx = mouse.0 - tw as i32 - 6;
        }
        if ty as f32 + th > H as f32 - 2.0 {
            ty = mouse.1 - th as i32 - 6;
        }
        let r = Rect::new(tx.max(2), ty.max(2), tw.ceil() as i32, th.ceil() as i32);
        ui.p.panel(r, rgba(12, 18, 30, 250), rgb(110, 140, 196), 3.0);
        let mut ly = r.y as f32 + 3.0;
        for (t, col, size) in lines {
            ui.p.text(r.x as f32 + 5.0, ly, &t, size, col);
            ly += if size == BODY { 10.0 } else { 8.0 };
        }
    }
    g.widgets = ui.widgets;
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
    let catalog = arg(&args, "catalog").map(PathBuf::from).or_else(|| std::env::var("AEGIS_CATALOG").ok().map(PathBuf::from)).or_else(|| engine_dir.as_ref().map(|d| d.join("parity/out/catalog.json"))).context("catalog.json not found: pass catalog=<path>")?;
    let cat = Arc::new(Catalog::load_resolved(&catalog).with_context(|| format!("loading {}", catalog.display()))?);
    let project = art::find_project(engine_dir.as_deref().unwrap_or(&cwd));
    let el: EventLoop<()> = EventLoop::with_user_event().build()?;
    let proxy = el.create_proxy();
    let mut app = App { gui: None, cat, args, window: None, pixels: None, proxy, bridge: None, project, engine_dir };
    el.run_app(&mut app)?;
    Ok(())
}
