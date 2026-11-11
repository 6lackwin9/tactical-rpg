//! The screen stack (MenuRouter): every screen draws with the widget kit, records its widgets for the agent link and
//! reacts to clicks, keys and the wheel. The shared context `Cx<G>` holds the engine's half (settings, art, audio, queued
//! navigation and notices, answers) and the game's half `G` (its data, the game in play, its log), which screens
//! reach through Deref. Screens never call each other; they queue navigation in `cx.nav`.

use crate::art::Art;
use crate::audio::Audio;
use crate::canvas::Rect;
use crate::pacing::FrameNeed;
pub use crate::painter::Grid;
use crate::painter::Painter;
use crate::ui::{self, Gfx, Ui, Widget, BODY, H, W};
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
    /// Shift+Tab.
    BackTab,
    PageUp,
    PageDown,
    /// Ctrl+Enter (submit a form, e.g. save a note).
    CtrlEnter,
    /// Shift with a digit key (0-9, the top row or the keypad), whatever character the layout types for it.
    ShiftDigit(u8),
    /// A function key a screen may use for debugging (F5, F6, F7, F9, F10, F11; the shell keeps F8 and F12 for itself).
    Fn(u8),
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
            "backtab" | "shift+tab" => Key::BackTab,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "ctrl+enter" => Key::CtrlEnter,
            d if d.len() == 7 && d.starts_with("shift+") && d.as_bytes()[6].is_ascii_digit() => Key::ShiftDigit(d.as_bytes()[6] - b'0'),
            "f5" => Key::Fn(5),
            "f6" => Key::Fn(6),
            "f7" => Key::Fn(7),
            "f9" => Key::Fn(9),
            "f10" => Key::Fn(10),
            "f11" => Key::Fn(11),
            c if c.chars().count() == 1 => Key::Char(c.chars().next().unwrap()),
            _ => return None,
        })
    }

    /// The key's name, as `parse` reads it ("up", "shift+tab", "a").
    pub fn name(self) -> String {
        match self {
            Key::Up => "up".into(),
            Key::Down => "down".into(),
            Key::Left => "left".into(),
            Key::Right => "right".into(),
            Key::Enter => "enter".into(),
            Key::Space => "space".into(),
            Key::Esc => "esc".into(),
            Key::Tab => "tab".into(),
            Key::BackTab => "shift+tab".into(),
            Key::PageUp => "pageup".into(),
            Key::PageDown => "pagedown".into(),
            Key::CtrlEnter => "ctrl+enter".into(),
            Key::ShiftDigit(d) => format!("shift+{d}"),
            Key::Fn(n) => format!("f{n}"),
            Key::Char(c) => c.to_string(),
        }
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
    /// The window sizes offered (`window_sizes`): the last list `keep_window_size` held to, else `WINDOW_SIZES`.
    offered: Vec<String>,
}

/// The engine's choices (Godot SettingsFile / DisplaySettings): window sizes, frame-rate caps. `WINDOW_SIZES` is the
/// default list; a game offers its own (its settings rows, `Settings::keep_window_size`).
pub const WINDOW_SIZES: [&str; 8] = ["960x540", "1024x576", "1152x648", "1280x720", "1366x768", "1600x900", "1920x1080", "2560x1440"];
pub const FPS_CAPS: [&str; 5] = ["Unlimited", "30", "60", "120", "144"];
/// Text size levels (Godot TEXT_LEVELS): label and font factor.
pub const TEXT_SIZES: [(&str, f64); 4] = [("Small", 0.85), ("Normal", 1.0), ("Large", 1.15), ("Extra large", 1.3)];
/// Typewriter speeds (STORYUI1-E, setting `text_speed`; `vignette::TextSpeed`): 30, 60, 120 characters a second, or
/// all at once.
pub const TEXT_SPEEDS: [&str; 4] = ["slow", "normal", "fast", "instant"];

/// How the settings ask the window to show (DISPLAY2, keys `fullscreen` and `borderless`; anything but `true` is off).
/// Fullscreen covers the monitor whatever `borderless` says; `borderless` waits for its turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowShow {
    Fullscreen,
    Borderless,
    Windowed,
}

impl WindowShow {
    pub fn of(settings: &Value) -> Self {
        let on = |k: &str| settings[k].as_bool().unwrap_or(false);
        match (on("fullscreen"), on("borderless")) {
            (true, _) => Self::Fullscreen,
            (false, true) => Self::Borderless,
            (false, false) => Self::Windowed,
        }
    }
}

/// The largest side a `window_size` value may name: past it the value is no size, so a bad file cannot ask the OS for
/// an absurd window on a platform that reports no monitor to cap it by.
pub const WINDOW_SIDE_MAX: u32 = 16384;

/// A `window_size` value's width and height in real pixels, at any display scale (None: not a size, or a side of 0 or past `WINDOW_SIDE_MAX`).
pub fn window_size_of(size: &Value) -> Option<(u32, u32)> {
    let (w, h) = size.as_str()?.split_once('x')?;
    let (w, h) = (w.parse::<u32>().ok()?, h.parse::<u32>().ok()?);
    ((1..=WINDOW_SIDE_MAX).contains(&w) && (1..=WINDOW_SIDE_MAX).contains(&h)).then_some((w, h))
}

/// The longest list of window sizes a game may offer: a settings row cycles through it.
pub const WINDOW_SIZES_MAX: usize = 32;

/// The manifest's `window_sizes` value (LAYOUT720-E5): absent (null) is `WINDOW_SIZES`; else a list of 1 to
/// `WINDOW_SIZES_MAX` `WxH` strings (`window_size_of`), none twice. Anything else is refused, with why.
pub fn read_window_sizes(v: &Value) -> Result<Vec<String>, String> {
    if v.is_null() {
        return Ok(WINDOW_SIZES.map(String::from).to_vec());
    }
    let Some(list) = v.as_array() else { return Err(format!("window_sizes must be a list of WxH strings, not {v}")) };
    if list.is_empty() || list.len() > WINDOW_SIZES_MAX {
        return Err(format!("window_sizes must list 1 to {WINDOW_SIZES_MAX} sizes, not {}", list.len()));
    }
    let mut sizes: Vec<String> = Vec::with_capacity(list.len());
    for size in list {
        let text = size.as_str().filter(|s| window_size_of(size).is_some() && !sizes.iter().any(|o| o == s));
        sizes.push(text.ok_or_else(|| format!("window_sizes: {size} is not a WxH size up to {WINDOW_SIDE_MAX}, or is listed twice"))?.to_string());
    }
    Ok(sizes)
}

/// An area of the desktop in physical px.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// The client size for a window that asks for `want`, with `frame` px around it, on `usable`: `want` when the whole
/// window fits, else `want` scaled down to fit with its shape kept, so the fixed canvas gains no bars. Never below 1 px.
pub fn window_capped(want: (u32, u32), frame: (u32, u32), usable: (u32, u32)) -> (u32, u32) {
    let room = (u64::from(usable.0.saturating_sub(frame.0)).max(1), u64::from(usable.1.saturating_sub(frame.1)).max(1));
    let asked = (u64::from(want.0).max(1), u64::from(want.1).max(1));
    if asked.0 <= room.0 && asked.1 <= room.1 {
        return want;
    }
    // The tighter side sets the scale; whole integers, so neither side can round past its room.
    let (num, den) = if room.0 * asked.1 <= room.1 * asked.0 { (room.0, asked.0) } else { (room.1, asked.1) };
    ((asked.0 * num / den).max(1) as u32, (asked.1 * num / den).max(1) as u32)
}

/// Where a window of `outer` size goes to stay inside `usable`, from `at`: as near as it can; the top left wins when it
/// is too big to fit.
pub fn window_kept_in(usable: ScreenRect, at: (i32, i32), outer: (u32, u32)) -> (i32, i32) {
    let axis = |at: i32, start: i32, room: u32, size: u32| {
        let last = i64::from(start) + i64::from(room) - i64::from(size);
        i64::from(at).min(last).max(i64::from(start)) as i32
    };
    (axis(at.0, usable.x, usable.w, outer.0), axis(at.1, usable.y, usable.h, outer.1))
}

/// Where a borderless window of `outer` size goes: centred on the monitor, then moved only as far as it must to stay
/// inside `usable` (a window as tall as the usable area sits above the taskbar, not under it).
pub fn window_centred(monitor: ScreenRect, usable: ScreenRect, outer: (u32, u32)) -> (i32, i32) {
    let middle = |start: i32, room: u32, size: u32| (i64::from(start) + (i64::from(room) - i64::from(size)) / 2) as i32;
    window_kept_in(usable, (middle(monitor.x, monitor.w, outer.0), middle(monitor.y, monitor.h, outer.1)), outer)
}

impl Settings {
    /// The engine's keys and their defaults. STORYUI1-E: `text_speed` (the vignette player's typewriter) and
    /// `reduce_motion` (no drifting or shaking effects; shared with FX1).
    pub fn engine_defaults() -> Value {
        json!({"mute": false, "master": 100, "music": 80, "effects": 80, "interface": 80, "fullscreen": false, "borderless": false, "window_size": "1280x720",
            "vsync": true, "fps_cap": "Unlimited", "text_scale": 1.0, "record_playtests": cfg!(debug_assertions), "text_speed": "normal", "reduce_motion": false})
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

    /// The shared reduce-motion setting (STORYUI1-E's key `vignette::REDUCE_MOTION`; false when missing): no camera or
    /// token shakes (FX1), no drifting ash; effects and hit-stop still show.
    pub fn reduce_motion(&self) -> bool {
        self.v[crate::vignette::REDUCE_MOTION].as_bool().unwrap_or(false)
    }

    /// The saved settings over the defaults (keys the file lacks keep their default). A file from before DISPLAY2 has
    /// `window_mode`: Borderless (it covered the monitor) and Fullscreen become `fullscreen` on, Windowed or anything
    /// else both off. The old key goes, so the next save does not write it; a file with the new keys keeps them.
    pub fn load(path: PathBuf, game_defaults: Value) -> Self {
        let mut v = Self::defaults(game_defaults);
        if let Ok(Value::Object(mut saved)) = aegis_save::store::current().read_to_string(&path).map(|t| serde_json::from_str::<Value>(&t).unwrap_or(Value::Null)) {
            if let Some(old) = saved.shift_remove("window_mode") {
                if !saved.contains_key("fullscreen") && !saved.contains_key("borderless") {
                    saved.insert("fullscreen".into(), Value::Bool(matches!(old.as_str(), Some("Borderless" | "Fullscreen"))));
                }
            }
            for (k, x) in saved {
                v[k] = x;
            }
        }
        Self { v, path, offered: WINDOW_SIZES.map(String::from).to_vec() }
    }

    /// The window sizes the game offers, for a settings row: what `keep_window_size` was last given, else `WINDOW_SIZES`.
    pub fn window_sizes(&self) -> &[String] {
        &self.offered
    }

    /// Holds `window_size` to the sizes a game offers: a listed value stays, a size off the list becomes the listed one
    /// nearest in pixels, anything else the first listed. An empty list changes nothing. Not written until the next save.
    pub fn keep_window_size(&mut self, offered: &[&str]) {
        if !offered.is_empty() {
            self.offered = offered.iter().map(|s| s.to_string()).collect();
        }
        let now = &self.v["window_size"];
        if now.as_str().is_some_and(|s| offered.contains(&s)) {
            return;
        }
        let area = |s: &Value| window_size_of(s).map(|(w, h)| i64::from(w) * i64::from(h));
        let nearest = area(now).and_then(|have| offered.iter().filter_map(|o| Some((*o, (area(&json!(o))? - have).abs()))).min_by_key(|c| c.1).map(|c| c.0));
        if let Some(to) = nearest.or(offered.first().copied()) {
            self.v["window_size"] = json!(to);
        }
    }

    /// Writes the settings file through the user store (`aegis_save::store`: the disk on native, `localStorage` on
    /// the web); a refused write (a full quota) is the error, never a panic.
    pub fn save(&self) -> Result<(), String> {
        let st = aegis_save::store::current();
        if let Some(dir) = self.path.parent() {
            st.create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        st.write(&self.path, serde_json::to_string_pretty(&self.v).unwrap().as_bytes()).map_err(|e| e.to_string())
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

/// The modifier keys held down right now (the shell keeps it current; all up while the window is out of focus), for
/// hold-to-show controls such as revealing what is hidden while a key is held.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Held {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// The step direction (`Key::dir`) of the movement key (arrows, W A S D) held down, the latest pressed when
    /// several are; None when none is. For screens that walk while a key is held (`Screen::hold_walk`).
    pub dir: Option<(i64, i64)>,
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
    /// The playtest hub (recordings of this run: sessions, trail, notes).
    pub playtest: crate::playtest::Playtests,
    /// The modifier keys held down right now.
    pub held: Held,
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
    /// A yes / no question with Yes as the default (Enter = Yes); `yes` runs on Yes.
    pub fn ask(&mut self, question: impl Into<String>, yes: impl FnOnce(&mut Cx<G>) + 'static) {
        self.nav.push(Nav::Push(Box::new(Ask::new(question.into(), Box::new(yes), false))));
    }

    /// A destructive question with No as the default (ASKNO1): No has the visible keyboard focus as it opens, Enter
    /// = No, `y` = Yes; `yes` runs on Yes.
    pub fn ask_no(&mut self, question: impl Into<String>, yes: impl FnOnce(&mut Cx<G>) + 'static) {
        self.nav.push(Nav::Push(Box::new(Ask::new(question.into(), Box::new(yes), true))));
    }

    /// A question whose Yes leaves `tag` in `answers` for the asking screen.
    pub fn ask_tag(&mut self, question: impl Into<String>, tag: &str) {
        let tag = tag.to_string();
        self.ask(question, move |cx| cx.answers.push(tag));
    }

    /// `ask_tag` with No as the default (`ask_no`).
    pub fn ask_tag_no(&mut self, question: impl Into<String>, tag: &str) {
        let tag = tag.to_string();
        self.ask_no(question, move |cx| cx.answers.push(tag));
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
    /// The units this screen writes its rects in, for as long as it lives (LAYOUT720-E1; the shell gives it the view
    /// and the mouse in them). The default keeps today's picture.
    fn grid(&self) -> Grid {
        Grid::Old
    }
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
    /// Whether the arrow keys move the keyboard focus (PORT-FOCUS); a screen that walks or steers with them says no
    /// (Tab and Shift+Tab still move it).
    fn arrow_focus(&self) -> bool {
        true
    }
    /// Whether the screen walks while a movement key is held (`Held::dir`): the shell then delivers such a key once
    /// (the press) and drops the operating system's key repeats, whose first comes ~0.5 s late.
    fn hold_walk(&self) -> bool {
        false
    }
    /// Whether the first widget takes a visible focus as the screen appears (a menu meant for the keyboard).
    fn auto_focus(&self) -> bool {
        false
    }
    /// The widget the opening focus lands on (`auto_focus`) when it is not the first one (ASKNO2).
    fn focus_first(&self) -> Option<&str> {
        None
    }
    /// A widget the windowed shell moves the OS mouse pointer onto as this screen opens (ASKNO2: a question's safe
    /// answer). Only for a person playing: never in tests, headless runs, agent-link commands or an unfocused window.
    fn pointer_to(&self) -> Option<&str> {
        None
    }
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
    /// What the frame loop owes this screen now (FPSCAP1). Default: every frame. Say less only when `tick` and `draw` have
    /// nothing to do until input or the time named; the shell asks every screen it draws, after each frame.
    fn frame_need(&self, _cx: &Cx<G>) -> FrameNeed {
        FrameNeed::Full
    }
    /// Extra state for the agent's ui_tree.
    fn state(&self, _cx: &Cx<G>) -> Value {
        Value::Null
    }
    /// Called when the screen becomes the top again.
    fn resume(&mut self, _cx: &mut Cx<G>) {}
    /// A text field of this screen changed while being typed into (`Ui::text_field`): the screen keeps the text.
    fn text_changed(&mut self, _cx: &mut Cx<G>, _id: &str, _text: &str) {}
    /// What a playtest note taken over this screen records about it (a game's play screen: its session, round, block,
    /// phase).
    fn note_context(&self, _cx: &Cx<G>) -> Value {
        Value::Null
    }
    /// Called when the screen leaves the stack (popped, a new stack, the window closing): its last chance to keep
    /// what it holds (a playtest session).
    fn leave(&mut self, _cx: &mut Cx<G>) {}
    /// An agent link command the shell does not know (a game screen's own command API); None when not handled.
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
    /// The screen's windows (UIKIT3), when it keeps a desk: the shell moves, resizes, raises, opens and closes them and
    /// saves the desk; the screen draws it (`Desk::draw`, `Desk::toolbar`).
    fn desk(&mut self) -> Option<&mut crate::window::Desk> {
        None
    }
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

/// The yes / no question (`Cx::ask`, `Cx::ask_no`): Yes then No, left to right. Esc, right click and `n` answer No,
/// `y` answers Yes; Enter answers the default: Yes, or No for a destructive question (`no_default`, which also opens
/// with the keyboard focus showing on No and asks the windowed shell to move the mouse pointer onto No).
pub struct Ask<G> {
    question: String,
    yes: Option<Box<dyn FnOnce(&mut Cx<G>)>>,
    no_default: bool,
    w: Vec<Widget<bool>>,
}

impl<G> Ask<G> {
    fn new(question: String, yes: Box<dyn FnOnce(&mut Cx<G>)>, no_default: bool) -> Self {
        Self { question, yes: Some(yes), no_default, w: Vec::new() }
    }
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
    fn frame_need(&self, _cx: &Cx<G>) -> FrameNeed {
        FrameNeed::Idle
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, _cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<bool> = Ui::new(g, mouse, live);
        let scrim = ui.t.scrim;
        ui.p.fill(Rect::new(0, 0, W, H), scrim);
        let lines = ui.p.wrap(&self.question, BODY, 250.0);
        let h = 40 + lines.len() as i32 * 10;
        let r = Rect::new(W / 2 - 135, H / 2 - h / 2, 270, h);
        ui.frame(r, "", false);
        let mut y = r.y as f32 + 8.0;
        for l in &lines {
            ui.p.text(r.x as f32 + 10.0, y, l, BODY, ui.t.text);
            y += 10.0;
        }
        ui.button("ask.yes", Rect::new(r.x + r.w - 120, r.y + r.h - 22, 50, 15), "Yes", true, true);
        ui.button("ask.no", Rect::new(r.x + r.w - 62, r.y + r.h - 22, 50, 15), "No", false, true);
        self.w = ui.widgets;
    }
    fn auto_focus(&self) -> bool {
        self.no_default
    }
    fn focus_first(&self) -> Option<&str> {
        self.no_default.then_some("ask.no")
    }
    fn pointer_to(&self) -> Option<&str> {
        self.no_default.then_some("ask.no")
    }
    fn key(&mut self, cx: &mut Cx<G>, k: Key) {
        match k {
            Key::Enter => self.act(cx, !self.no_default),
            Key::Char('y') => self.act(cx, true),
            Key::Char('n') => self.act(cx, false),
            _ => {}
        }
    }
    fn state(&self, _cx: &Cx<G>) -> Value {
        json!({"question": self.question, "default": if self.no_default { "no" } else { "yes" }})
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
    fn frame_need(&self, _cx: &Cx<G>) -> FrameNeed {
        FrameNeed::Idle
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
            ui.p.text(r.x as f32 + 10.0, y, l, BODY, ui.t.text);
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
    let gold = ui.t.gold;
    ui.p.text_center(W as f32 / 2.0, y as f32, text, ui::HEAD + 4.0, gold);
}
