//! The screen stack (MenuRouter): every screen draws with the widget kit, records its widgets for the agent link and
//! reacts to clicks, keys and the wheel. The shared context `Cx<G>` holds the engine's half (settings, art, audio, queued
//! navigation and notices, answers) and the game's half `G` (its data, the game in play, its log), which screens
//! reach through Deref. Screens never call each other; they queue navigation in `cx.nav`.

use crate::art::Art;
use crate::audio::Audio;
use crate::canvas::Rect;
use crate::painter::Painter;
use crate::ui::{self, Gfx, Ui, Widget, BODY, GOLD, H, TEXT, W};
use serde_json::{json, Value};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

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

pub enum Nav<G> {
    Push(Box<dyn Screen<G>>),
    Pop,
    /// Drops every screen and shows this one (enter game, quit to menu).
    Reset(Box<dyn Screen<G>>),
}

/// Player settings (SettingsFile): a JSON object of the engine's keys (sound, display, text size; the shell applies
/// them) and the game's own keys, over their defaults, saved where the game says.
#[derive(Debug, Clone)]
pub struct Settings {
    pub v: Value,
    path: PathBuf,
}

/// The engine's choices (Godot SettingsFile / DisplaySettings): window modes, window sizes, frame-rate caps.
pub const WINDOW_MODES: [&str; 3] = ["Windowed", "Borderless", "Fullscreen"];
pub const WINDOW_SIZES: [&str; 8] = ["960x540", "1024x576", "1152x648", "1280x720", "1366x768", "1600x900", "1920x1080", "2560x1440"];
pub const FPS_CAPS: [&str; 5] = ["Unlimited", "30", "60", "120", "144"];
/// Text size levels (Godot TEXT_LEVELS): label and font factor.
pub const TEXT_SIZES: [(&str, f64); 4] = [("Small", 0.85), ("Normal", 1.0), ("Large", 1.15), ("Extra large", 1.3)];

impl Settings {
    /// The engine's keys and their defaults.
    pub fn engine_defaults() -> Value {
        json!({"mute": false, "master": 100, "music": 80, "effects": 80, "interface": 80, "window_mode": "Windowed", "window_size": "1280x720",
            "vsync": true, "fps_cap": "Unlimited", "text_scale": 1.0})
    }

    /// Every default: the engine's keys, then the game's own (`game_defaults`).
    pub fn defaults(game_defaults: Value) -> Value {
        let mut v = Self::engine_defaults();
        if let Value::Object(g) = game_defaults {
            for (k, x) in g {
                v[k] = x;
            }
        }
        v
    }

    /// The saved settings over the defaults (keys the file lacks keep their default).
    pub fn load(path: PathBuf, game_defaults: Value) -> Self {
        let mut v = Self::defaults(game_defaults);
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

/// The game's half of the context, as the shell needs it.
pub trait Context: Sized + 'static {
    /// What the agent can send besides the shell's commands (named when a command is unknown).
    const MORE_COMMANDS: &'static str;
    /// A line for the game's event log (the shell's screenshot note).
    fn note(&mut self, line: String);
    /// The game in play for the agent's ui_tree (null when none).
    fn agent_summary(&self) -> Value;
    /// The reply to the agent's `game` command.
    fn agent_game(&self) -> Value;
}

/// The shared context: the engine's half here, the game's half in `app` (reached through Deref).
pub struct Cx<G> {
    pub settings: Settings,
    pub art: Art,
    pub audio: Audio,
    pub nav: Vec<Nav<G>>,
    /// A one-shot message the shell shows as a notice popup.
    pub notice: Vec<String>,
    pub shot_dir: PathBuf,
    pub quit: bool,
    /// Tags of questions answered Yes (ask_tag); the asking screen takes them on its next frame.
    pub answers: Vec<String>,
    pub app: G,
}

impl<G> Deref for Cx<G> {
    type Target = G;

    fn deref(&self) -> &G {
        &self.app
    }
}

impl<G> DerefMut for Cx<G> {
    fn deref_mut(&mut self) -> &mut G {
        &mut self.app
    }
}

impl<G: Context> Cx<G> {
    pub fn ask(&mut self, question: impl Into<String>, yes: impl FnOnce(&mut Cx<G>) + 'static) {
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
}

pub trait Screen<G> {
    fn id(&self) -> &'static str;
    /// Drawn over the screen below it (popups, menus over the map).
    fn overlay(&self) -> bool {
        false
    }
    fn draw(&mut self, cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool);
    fn widgets(&self) -> Vec<Value>;
    fn draw_tip(&self, p: &mut Painter, mouse: (i32, i32));
    fn click(&mut self, cx: &mut Cx<G>, pos: (i32, i32)) -> bool;
    fn click_id(&mut self, cx: &mut Cx<G>, id: &str) -> Result<(), String>;
    fn widget_rect(&self, id: &str) -> Option<(Rect, Vec<String>)>;
    fn key(&mut self, _cx: &mut Cx<G>, _k: Key) {}
    /// Right click (and Esc unless `escape` says otherwise).
    fn back(&mut self, cx: &mut Cx<G>) {
        cx.nav.push(Nav::Pop);
    }
    /// The Esc key.
    fn escape(&mut self, cx: &mut Cx<G>) {
        self.back(cx);
    }
    fn wheel(&mut self, _cx: &mut Cx<G>, _lines: i32) {}
    fn tick(&mut self, _cx: &mut Cx<G>) {}
    /// Extra state for the agent's ui_tree.
    fn state(&self, _cx: &Cx<G>) -> Value {
        Value::Null
    }
    /// Called when the screen becomes the top again.
    fn resume(&mut self, _cx: &mut Cx<G>) {}
    /// An agent link command the shell does not know (the battle screen's command API); None when not handled.
    fn command(&mut self, _cx: &Cx<G>, _body: &Value) -> Option<Value> {
        None
    }
    /// The music this screen wants (a path under the game's audio folder); the shell plays the topmost screen's
    /// wish, None lets the screen below choose.
    fn music(&self, _cx: &Cx<G>) -> Option<String> {
        None
    }
    /// Drag and drop (UIKIT2): whether payload `from` (`Ui::draggable`) may be dropped on the target `to`
    /// (`Ui::drop_target`); valid targets are outlined while dragging.
    fn can_drop(&self, _cx: &Cx<G>, _from: &str, _to: &str) -> bool {
        true
    }
    /// Payload `from` was released over target `to` (only when `can_drop` allowed it).
    fn accept_drop(&mut self, _cx: &mut Cx<G>, _from: &str, _to: &str) {}
}

/// The widget plumbing every screen shares: its `w` list and `act(cx, action)`. The context type is `Cx` as named
/// where the macro is used, or the one given.
#[macro_export]
macro_rules! widget_plumbing {
    () => {
        $crate::widget_plumbing!(Cx);
    };
    ($cx:ty) => {
        fn widgets(&self) -> Vec<serde_json::Value> {
            self.w.iter().map(|w| w.info()).collect()
        }
        fn draw_tip(&self, p: &mut $crate::painter::Painter, mouse: (i32, i32)) {
            $crate::ui::draw_tip(p, &self.w, mouse)
        }
        fn click(&mut self, cx: &mut $cx, pos: (i32, i32)) -> bool {
            match $crate::ui::hit(&self.w, pos) {
                Some(a) => {
                    self.act(cx, a);
                    true
                }
                None => false,
            }
        }
        fn click_id(&mut self, cx: &mut $cx, id: &str) -> Result<(), String> {
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

pub struct Ask<G> {
    question: String,
    yes: Option<Box<dyn FnOnce(&mut Cx<G>)>>,
    w: Vec<Widget<bool>>,
}

impl<G> Ask<G> {
    fn act(&mut self, cx: &mut Cx<G>, yes: bool) {
        cx.nav.push(Nav::Pop);
        if yes {
            if let Some(f) = self.yes.take() {
                f(cx);
            }
        }
    }
}

impl<G> Screen<G> for Ask<G> {
    fn id(&self) -> &'static str {
        "ask"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, _cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
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
    fn key(&mut self, cx: &mut Cx<G>, k: Key) {
        match k {
            Key::Enter | Key::Char('y') => self.act(cx, true),
            Key::Char('n') => self.act(cx, false),
            _ => {}
        }
    }
    fn state(&self, _cx: &Cx<G>) -> Value {
        json!({"question": self.question})
    }
    widget_plumbing!(Cx<G>);
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
    fn act<G>(&mut self, cx: &mut Cx<G>, _a: ()) {
        cx.nav.push(Nav::Pop);
    }
}

impl<G> Screen<G> for Notice {
    fn id(&self) -> &'static str {
        "notice"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, _cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
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
    fn key(&mut self, cx: &mut Cx<G>, k: Key) {
        if k == Key::Enter || k == Key::Space {
            self.act(cx, ());
        }
    }
    fn state(&self, _cx: &Cx<G>) -> Value {
        json!({"lines": self.lines})
    }
    widget_plumbing!(Cx<G>);
}

/// Errors as a notice (nothing when empty).
pub fn show_errors<G: 'static>(cx: &mut Cx<G>, errors: Vec<String>) {
    if !errors.is_empty() {
        cx.nav.push(Nav::Push(Box::new(Notice::new(errors))));
    }
}

/// A centred screen title.
pub fn title(ui: &mut Ui<impl Clone>, text: &str, y: i32) {
    ui.p.text_center(W as f32 / 2.0, y as f32, text, ui::HEAD + 4.0, GOLD);
}
