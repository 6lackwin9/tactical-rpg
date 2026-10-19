//! The screen stack (MenuRouter): every screen draws with the widget kit, records its widgets for the agent link and
//! reacts to clicks, keys and the wheel. The shared context holds the data, the game in play (CurrentGame), the save
//! folder, settings, art and the event log. Screens never call each other; they queue navigation in `cx.nav`.

use crate::art::Art;
use crate::canvas::Rect;
use crate::painter::Painter;
use crate::tuning::TuningFile;
use crate::ui::{self, Gfx, Ui, Widget, BODY, GOLD, H, TEXT, W};
use aegis_world::{save, Data, GameState};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Space,
    Esc,
    Tab,
    PageUp,
    PageDown,
    Char(char),
}

impl Key {
    pub fn parse(s: &str) -> Option<Key> {
        Some(match s.to_lowercase().as_str() {
            "up" => Key::Up,
            "down" => Key::Down,
            "left" => Key::Left,
            "right" => Key::Right,
            "enter" => Key::Enter,
            "space" => Key::Space,
            "esc" | "escape" => Key::Esc,
            "tab" => Key::Tab,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            c if c.chars().count() == 1 => Key::Char(c.chars().next().unwrap()),
            _ => return None,
        })
    }

    /// A step direction for the arrow keys and W A S D.
    pub fn dir(self) -> Option<(i64, i64)> {
        match self {
            Key::Up | Key::Char('w') => Some((0, -1)),
            Key::Down | Key::Char('s') => Some((0, 1)),
            Key::Left | Key::Char('a') => Some((-1, 0)),
            Key::Right | Key::Char('d') => Some((1, 0)),
            _ => None,
        }
    }
}

pub enum Nav {
    Push(Box<dyn Screen>),
    Pop,
    /// Drops every screen and shows this one (enter game, quit to menu).
    Reset(Box<dyn Screen>),
}

/// Player settings (SettingsFile): saved as JSON next to the engine's save folder.
#[derive(Debug, Clone)]
pub struct Settings {
    pub v: Value,
    path: PathBuf,
}

pub fn default_settings() -> Value {
    json!({"mute": false, "master": 100, "music": 80, "effects": 80, "interface": 80, "window_mode": "Windowed", "window_size": "1280x720",
        "vsync": true, "fps_cap": "Unlimited", "text_scale": 1.0, "anim_speed": 1.0, "skip_anim": false, "show_tells": true, "warn_missing": true})
}

impl Settings {
    pub fn load(path: PathBuf) -> Self {
        let mut v = default_settings();
        if let Ok(Value::Object(saved)) = std::fs::read_to_string(&path).map(|t| serde_json::from_str::<Value>(&t).unwrap_or(Value::Null)) {
            for (k, x) in saved {
                v[k] = x;
            }
        }
        Self { v, path }
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&self.path, serde_json::to_string_pretty(&self.v).unwrap()).map_err(|e| e.to_string())
    }
}

/// The game in play (CurrentGame): the state, the slot it came from and its copy at the last save or load.
pub struct Game {
    pub gs: GameState,
    pub slot: i64,
    pub clean: Value,
    /// A battle, conversation or shop is open (saving and leaving wait).
    pub busy: bool,
}

pub struct Cx {
    pub data: Arc<Data>,
    pub game: Option<Game>,
    pub save_dir: PathBuf,
    pub settings: Settings,
    pub art: Art,
    pub tuning: TuningFile,
    /// The region map's event log (newest last).
    pub log: Vec<String>,
    pub nav: Vec<Nav>,
    /// A one-shot message the shell shows as a notice popup.
    pub notice: Vec<String>,
    pub shot_dir: PathBuf,
    pub quit: bool,
    pub encounter: String,
    pub seed: Option<i64>,
    /// Tags of questions answered Yes (ask_tag); the asking screen takes them on its next frame.
    pub answers: Vec<String>,
    /// A debug game (Scene Runner): fights get Won / Fled / Lost buttons and nothing is saved.
    pub debug: bool,
    /// The Godot export folder the data was loaded from (Reload re-reads it).
    pub data_dir: PathBuf,
}

impl Cx {
    pub fn gs(&self) -> &GameState {
        &self.game.as_ref().expect("a game is running").gs
    }

    pub fn gs_mut(&mut self) -> &mut GameState {
        &mut self.game.as_mut().expect("a game is running").gs
    }

    pub fn has_game(&self) -> bool {
        self.game.is_some()
    }

    pub fn busy(&self) -> bool {
        self.game.as_ref().is_some_and(|g| g.busy)
    }

    pub fn set_busy(&mut self, b: bool) {
        if let Some(g) = self.game.as_mut() {
            g.busy = b;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.game.as_ref().is_some_and(|g| !aegis_core::json::deep_equal(&g.gs.to_value(), &g.clean))
    }

    pub fn mark_saved(&mut self, slot: i64) {
        if let Some(g) = self.game.as_mut() {
            g.slot = slot;
            g.clean = g.gs.to_value();
        }
    }

    pub fn log(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > 200 {
            self.log.remove(0);
        }
    }

    /// Writes the autosave (slot 0); problems go to the log.
    pub fn autosave(&mut self) {
        if self.debug {
            return;
        }
        let Some(g) = self.game.as_ref() else { return };
        let errors = save::write(&self.data, &self.save_dir, &g.gs, save::AUTOSAVE_SLOT, 0);
        if errors.is_empty() {
            self.mark_saved(save::AUTOSAVE_SLOT);
        } else {
            self.log(format!("Autosave failed: {}", errors.join("; ")));
        }
    }

    /// Starts playing `gs` (a new game or a loaded save) and shows the region map.
    pub fn enter_game(&mut self, gs: GameState, slot: i64, warnings: Vec<String>) {
        let clean = gs.to_value();
        self.game = Some(Game { gs, slot, clean, busy: false });
        self.log.clear();
        if !warnings.is_empty() {
            self.notice = warnings;
        }
        let map = crate::region::RegionMap::new(self);
        self.nav.push(Nav::Reset(Box::new(map)));
    }

    pub fn end_game(&mut self) {
        self.game = None;
        self.debug = false;
        self.log.clear();
    }

    pub fn ask(&mut self, question: impl Into<String>, yes: impl FnOnce(&mut Cx) + 'static) {
        self.nav.push(Nav::Push(Box::new(Ask { question: question.into(), yes: Some(Box::new(yes)), w: Vec::new() })));
    }

    /// A question whose Yes leaves `tag` in `answers` for the asking screen.
    pub fn ask_tag(&mut self, question: impl Into<String>, tag: &str) {
        let tag = tag.to_string();
        self.ask(question, move |cx| cx.answers.push(tag));
    }

    pub fn take_answer(&mut self, tag: &str) -> bool {
        match self.answers.iter().position(|a| a == tag) {
            Some(i) => {
                self.answers.remove(i);
                true
            }
            None => false,
        }
    }

    pub fn world_name(&self, world: &str) -> String {
        aegis_core::json::s_or(self.data.manifest(world), "name", world)
    }

    pub fn location_name(&self, world: &str, loc: &str) -> String {
        if loc.is_empty() {
            return String::new();
        }
        let hit = if self.data.has_world(world) { self.data.resolve(world, "locations", loc) } else { aegis_core::json::empty() };
        aegis_core::json::s_or(hit, "name", loc)
    }
}

pub trait Screen {
    fn id(&self) -> &'static str;
    /// Drawn over the screen below it (popups, menus over the map).
    fn overlay(&self) -> bool {
        false
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool);
    fn widgets(&self) -> Vec<Value>;
    fn draw_tip(&self, p: &mut Painter, mouse: (i32, i32));
    fn click(&mut self, cx: &mut Cx, pos: (i32, i32)) -> bool;
    fn click_id(&mut self, cx: &mut Cx, id: &str) -> Result<(), String>;
    fn widget_rect(&self, id: &str) -> Option<(Rect, Vec<String>)>;
    fn key(&mut self, _cx: &mut Cx, _k: Key) {}
    /// Right click (and Esc unless `escape` says otherwise).
    fn back(&mut self, cx: &mut Cx) {
        cx.nav.push(Nav::Pop);
    }
    /// The Esc key.
    fn escape(&mut self, cx: &mut Cx) {
        self.back(cx);
    }
    fn wheel(&mut self, _cx: &mut Cx, _lines: i32) {}
    fn tick(&mut self, _cx: &mut Cx) {}
    /// Extra state for the agent's ui_tree.
    fn state(&self, _cx: &Cx) -> Value {
        Value::Null
    }
    /// Called when the screen becomes the top again.
    fn resume(&mut self, _cx: &mut Cx) {}
    /// The battle screen exposes its command host to the agent link.
    fn as_battle(&mut self) -> Option<&mut crate::battle::BattleScreen> {
        None
    }
}

/// The widget plumbing every screen shares: its `w` list and `act(cx, action)`.
#[macro_export]
macro_rules! widget_plumbing {
    () => {
        fn widgets(&self) -> Vec<serde_json::Value> {
            self.w.iter().map(|w| w.info()).collect()
        }
        fn draw_tip(&self, p: &mut $crate::painter::Painter, mouse: (i32, i32)) {
            $crate::ui::draw_tip(p, &self.w, mouse)
        }
        fn click(&mut self, cx: &mut $crate::screens::Cx, pos: (i32, i32)) -> bool {
            match $crate::ui::hit(&self.w, pos) {
                Some(a) => {
                    self.act(cx, a);
                    true
                }
                None => false,
            }
        }
        fn click_id(&mut self, cx: &mut $crate::screens::Cx, id: &str) -> Result<(), String> {
            let a = $crate::ui::by_id(&self.w, id)?;
            self.act(cx, a);
            Ok(())
        }
        fn widget_rect(&self, id: &str) -> Option<($crate::canvas::Rect, Vec<String>)> {
            self.w.iter().find(|w| w.id == id).map(|w| (w.rect, w.tip.iter().map(|t| t.0.clone()).collect()))
        }
    };
}

// ---- Ask: the yes / no question (router.ask) -----------------------------------------------------------------

pub struct Ask {
    question: String,
    yes: Option<Box<dyn FnOnce(&mut Cx)>>,
    w: Vec<Widget<bool>>,
}

impl Ask {
    fn act(&mut self, cx: &mut Cx, yes: bool) {
        cx.nav.push(Nav::Pop);
        if yes {
            if let Some(f) = self.yes.take() {
                f(cx);
            }
        }
    }
}

impl Screen for Ask {
    fn id(&self) -> &'static str {
        "ask"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<bool> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), crate::canvas::rgba(0, 0, 0, 120));
        let lines = ui.p.wrap(&self.question, BODY, 250.0);
        let h = 40 + lines.len() as i32 * 10;
        let r = Rect::new(W / 2 - 135, H / 2 - h / 2, 270, h);
        ui.frame(r, "", false);
        let mut y = r.y as f32 + 8.0;
        for l in &lines {
            ui.p.text(r.x as f32 + 10.0, y, l, BODY, TEXT);
            y += 10.0;
        }
        ui.button("ask.no", Rect::new(r.x + r.w - 120, r.y + r.h - 22, 50, 15), "No", false, true);
        ui.button("ask.yes", Rect::new(r.x + r.w - 62, r.y + r.h - 22, 50, 15), "Yes", true, true);
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        match k {
            Key::Enter | Key::Char('y') => self.act(cx, true),
            Key::Char('n') => self.act(cx, false),
            _ => {}
        }
    }
    fn state(&self, _cx: &Cx) -> Value {
        json!({"question": self.question})
    }
    widget_plumbing!();
}

// ---- Notice: one-shot messages (load warnings, refused actions) ----------------------------------------------

pub struct Notice {
    pub lines: Vec<String>,
    w: Vec<Widget<()>>,
}

impl Notice {
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines, w: Vec::new() }
    }
    fn act(&mut self, cx: &mut Cx, _a: ()) {
        cx.nav.push(Nav::Pop);
    }
}

impl Screen for Notice {
    fn id(&self) -> &'static str {
        "notice"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<()> = Ui::new(g, mouse, live);
        let mut wrapped = Vec::new();
        for l in &self.lines {
            wrapped.extend(ui.p.wrap(l, BODY, 250.0));
        }
        let h = 40 + wrapped.len() as i32 * 10;
        let r = Rect::new(W / 2 - 135, H / 2 - h / 2, 270, h);
        ui.frame(r, "", false);
        let mut y = r.y as f32 + 8.0;
        for l in &wrapped {
            ui.p.text(r.x as f32 + 10.0, y, l, BODY, TEXT);
            y += 10.0;
        }
        ui.button("notice.ok", Rect::new(r.x + r.w - 62, r.y + r.h - 22, 50, 15), "OK", (), true);
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        if k == Key::Enter || k == Key::Space {
            self.act(cx, ());
        }
    }
    fn state(&self, _cx: &Cx) -> Value {
        json!({"lines": self.lines})
    }
    widget_plumbing!();
}

/// Errors as a notice (nothing when empty).
pub fn show_errors(cx: &mut Cx, errors: Vec<String>) {
    if !errors.is_empty() {
        cx.nav.push(Nav::Push(Box::new(Notice::new(errors))));
    }
}

/// A centred screen title.
pub fn title(ui: &mut Ui<impl Clone>, text: &str, y: i32) {
    ui.p.text_center(W as f32 / 2.0, y as f32, text, ui::HEAD + 4.0, GOLD);
}
