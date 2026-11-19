//! INPUT-MAP1 (ENGINE-7): the control map. Every key a screen gets passes through it, so a player can rebind keys and a
//! gamepad drives the same screens as the keyboard. Game-agnostic: the map turns an input (a keyboard key by its
//! `Key` name, or a pad button) into the `Key` the screens already read; a game's own keys are just characters.
//!
//! Settings key `controls` (the player's file, over the defaults): `{"keys": {"i": "up", "k": "down"}, "pad":
//! {"south": "enter", "east": "esc"}}`. A key bound to itself, or not named, is itself. Problems in the settings are
//! reported and the default stays for that entry. The pad is read on Windows (`gilrs`, the native build); elsewhere
//! and with no window, the agent link's `pad` request presses a button through the same map.

use crate::screen::Key;
use aegis_core::clock::{Duration, Instant};
use serde_json::{json, Map, Value};

/// The settings key that holds the player's bindings.
pub const SETTINGS_KEY: &str = "controls";

/// A gamepad button, by its place (the south face button is A on an Xbox pad, Cross on a PlayStation pad).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pad {
    South,
    East,
    West,
    North,
    LeftBumper,
    RightBumper,
    Select,
    Start,
    DUp,
    DDown,
    DLeft,
    DRight,
}

/// Every button with its name, in order.
pub const PADS: [(Pad, &str); 12] = [
    (Pad::South, "south"),
    (Pad::East, "east"),
    (Pad::West, "west"),
    (Pad::North, "north"),
    (Pad::LeftBumper, "left_bumper"),
    (Pad::RightBumper, "right_bumper"),
    (Pad::Select, "select"),
    (Pad::Start, "start"),
    (Pad::DUp, "dpad_up"),
    (Pad::DDown, "dpad_down"),
    (Pad::DLeft, "dpad_left"),
    (Pad::DRight, "dpad_right"),
];

impl Pad {
    pub fn parse(s: &str) -> Option<Pad> {
        PADS.iter().find(|(_, n)| *n == s).map(|(p, _)| *p)
    }
    pub fn name(self) -> &'static str {
        PADS.iter().find(|(p, _)| *p == self).map(|(_, n)| *n).unwrap_or("?")
    }
    /// The button as the player reads it, by the Xbox pad's names (`[PROVISIONAL]`, FB-327).
    pub fn label(self) -> &'static str {
        match self {
            Pad::South => "Pad A",
            Pad::East => "Pad B",
            Pad::West => "Pad X",
            Pad::North => "Pad Y",
            Pad::LeftBumper => "LB",
            Pad::RightBumper => "RB",
            Pad::Select => "View",
            Pad::Start => "Menu",
            Pad::DUp => "D-pad Up",
            Pad::DDown => "D-pad Down",
            Pad::DLeft => "D-pad Left",
            Pad::DRight => "D-pad Right",
        }
    }
}

/// What each button does with no setting: confirm, back, Space, Tab; the bumpers page; Select and Start are Shift+Tab
/// and Esc (the menu); the d-pad is the arrows.
pub const PAD_DEFAULTS: [(Pad, Key); 12] = [
    (Pad::South, Key::Enter),
    (Pad::East, Key::Esc),
    (Pad::West, Key::Space),
    (Pad::North, Key::Tab),
    (Pad::LeftBumper, Key::PageUp),
    (Pad::RightBumper, Key::PageDown),
    (Pad::Select, Key::BackTab),
    (Pad::Start, Key::Esc),
    (Pad::DUp, Key::Up),
    (Pad::DDown, Key::Down),
    (Pad::DLeft, Key::Left),
    (Pad::DRight, Key::Right),
];

/// The keys the shell keeps for itself and a binding may not take or give (F8 note, F12 shot are not `Key`s at all).
fn bindable(k: Key) -> bool {
    !matches!(k, Key::CtrlEnter)
}

/// CONTROLS-UI1: the keys a rebind may take: not Esc (it cancels the rebind), not the shell's own, the debug
/// function keys or Shift with a digit (a screen's own chord).
pub fn capturable(k: Key) -> bool {
    bindable(k) && !matches!(k, Key::Esc | Key::Fn(_) | Key::ShiftDigit(_))
}

/// A key as the player reads it: "Enter", "Shift+Tab", "Page Up", "E", "?".
pub fn key_label(k: Key) -> String {
    match k {
        Key::Up => "Up".into(),
        Key::Down => "Down".into(),
        Key::Left => "Left".into(),
        Key::Right => "Right".into(),
        Key::Enter => "Enter".into(),
        Key::Space => "Space".into(),
        Key::Esc => "Esc".into(),
        Key::Tab => "Tab".into(),
        Key::BackTab => "Shift+Tab".into(),
        Key::PageUp => "Page Up".into(),
        Key::PageDown => "Page Down".into(),
        Key::CtrlEnter => "Ctrl+Enter".into(),
        Key::ShiftDigit(d) => format!("Shift+{d}"),
        Key::Fn(n) => format!("F{n}"),
        Key::Char(c) => c.to_uppercase().collect(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Controls {
    /// (pressed, what screens get), only where they differ.
    keys: Vec<(Key, Key)>,
    pad: Vec<(Pad, Key)>,
}

impl Default for Controls {
    fn default() -> Self {
        Controls { keys: Vec::new(), pad: PAD_DEFAULTS.to_vec() }
    }
}

impl Controls {
    /// The map in `settings` (the whole settings object; its `controls` key, if any) over the defaults, and the
    /// problems found in it.
    pub fn from_settings(settings: &Value) -> (Controls, Vec<String>) {
        let mut c = Controls::default();
        let mut problems = Vec::new();
        let Some(v) = settings.get(SETTINGS_KEY) else { return (c, problems) };
        let Some(o) = v.as_object() else {
            return (c, vec![format!("settings '{SETTINGS_KEY}' is an object with 'keys' and 'pad'")]);
        };
        for k in o.keys().filter(|k| !["keys", "pad"].contains(&k.as_str())) {
            problems.push(format!("settings '{SETTINGS_KEY}': unknown key '{k}' (keys, pad)"));
        }
        let key = |s: &Value, at: &str, problems: &mut Vec<String>| match s.as_str().and_then(Key::parse) {
            Some(k) if bindable(k) => Some(k),
            _ => {
                problems.push(format!("settings '{SETTINGS_KEY}' {at}: '{}' is not a key name (up, enter, esc, tab, a, ...)", s.as_str().unwrap_or("?")));
                None
            }
        };
        for (from, to) in o.get("keys").and_then(Value::as_object).into_iter().flatten() {
            let at = format!("keys '{from}'");
            let (Some(f), Some(t)) = (key(&json!(from), &at, &mut problems), key(to, &at, &mut problems)) else { continue };
            c.keys.retain(|(k, _)| *k != f);
            if f != t {
                c.keys.push((f, t));
            }
        }
        for (button, to) in o.get("pad").and_then(Value::as_object).into_iter().flatten() {
            let Some(p) = Pad::parse(button) else {
                problems.push(format!("settings '{SETTINGS_KEY}' pad: '{button}' is not a button ({})", PADS.iter().map(|(_, n)| *n).collect::<Vec<_>>().join(", ")));
                continue;
            };
            if let Some(t) = key(to, &format!("pad '{button}'"), &mut problems) {
                if let Some(slot) = c.pad.iter_mut().find(|(q, _)| *q == p) {
                    slot.1 = t;
                }
            }
        }
        (c, problems)
    }

    /// What a pressed key `k` gives the screens.
    pub fn key(&self, k: Key) -> Key {
        self.keys.iter().find(|(f, _)| *f == k).map_or(k, |(_, t)| *t)
    }

    /// What a pad button gives the screens.
    pub fn pad(&self, p: Pad) -> Key {
        self.pad.iter().find(|(q, _)| *q == p).map(|(_, k)| *k).unwrap_or(Key::Enter)
    }

    /// The map as the settings write it: every pad button, and the rebound keys only.
    pub fn to_json(&self) -> Value {
        let keys: Map<String, Value> = self.keys.iter().map(|(f, t)| (f.name(), json!(t.name()))).collect();
        let pad: Map<String, Value> = self.pad.iter().map(|(p, k)| (p.name().to_string(), json!(k.name()))).collect();
        json!({"keys": keys, "pad": pad})
    }

    /// CONTROLS-UI1: the pressed key that gives the screens `action` (a rebound one first, else the key itself unless it
    /// was rebound away). None: no key gives it.
    pub fn key_for(&self, action: Key) -> Option<Key> {
        self.keys.iter().find(|(_, t)| *t == action).map(|(f, _)| *f).or_else(|| (self.key(action) == action).then_some(action))
    }

    /// The first pad button (in `PADS` order) that gives `action`.
    pub fn pad_for(&self, action: Key) -> Option<Pad> {
        PADS.iter().map(|(p, _)| *p).find(|p| self.pad(*p) == action)
    }

    fn set_key(&mut self, from: Key, to: Key) {
        self.keys.retain(|(f, _)| *f != from);
        if from != to {
            self.keys.push((from, to));
        }
        self.keys.sort_by_key(|(f, _)| f.name());
    }

    /// Key `new` gives `action` from now on; the key that gave it takes what `new` gave (a swap, so no input is lost and
    /// none gives two things).
    pub fn bind_key(&mut self, action: Key, new: Key) {
        let (cur, displaced) = (self.key_for(action), self.key(new));
        if displaced == action {
            return;
        }
        self.set_key(new, action);
        if let Some(c) = cur.filter(|c| *c != new) {
            self.set_key(c, displaced);
        }
    }

    /// Button `new` gives `action` from now on; the button that gave it takes what `new` gave.
    pub fn bind_pad(&mut self, action: Key, new: Pad) {
        let (cur, displaced) = (self.pad_for(action), self.pad(new));
        if displaced == action {
            return;
        }
        if let Some(slot) = self.pad.iter_mut().find(|(p, _)| *p == new) {
            slot.1 = action;
        }
        if let Some(c) = cur.filter(|c| *c != new) {
            if let Some(slot) = self.pad.iter_mut().find(|(p, _)| *p == c) {
                slot.1 = displaced;
            }
        }
    }

    /// What `input` gives the screens now.
    pub fn gives(&self, input: Input) -> Key {
        match input {
            Input::Key(k) => self.key(k),
            Input::Pad(p) => self.pad(p),
        }
    }
}

/// CONTROLS-UI1: a raw input, before the map: what a rebind captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Key(Key),
    Pad(Pad),
}

/// One action a game lists for rebinding: its id (widget and view row ids), the player's label, and the key its screens
/// read (`Key`). A game's actions name distinct keys.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Action {
    pub id: &'static str,
    pub label: &'static str,
    pub key: Key,
}

/// Where a rebind is: nothing asked, waiting for the input of action `at` (an index into the game's list), or asking
/// whether `input` may move from action `with` to `at` (a swap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Step {
    #[default]
    Idle,
    Waiting(usize),
    Clash { at: usize, input: Input, with: usize },
}

/// What an input did to a rebind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// No rebind is open: the input goes on to the screen.
    Ignored,
    /// Taken, nothing changed (a key a rebind may not take, or a key the swap question does not answer).
    Held,
    /// Taken and refused: a pad button another listed action holds, for an action with no button to trade
    /// (`Rebind::refused` names that action); the rebind still waits.
    Refused,
    Cancelled,
    /// The input is another action's: the swap question is open.
    Asked,
    /// The binding changed (or was already so); the settings hold the new map.
    Bound,
}

/// CONTROLS-UI1: the rebinding flow over the settings key `controls`, game-agnostic: a game lists its actions (`Action`)
/// and labels; a click starts a rebind, the next raw key or pad button binds it (Esc cancels), an input that is another
/// action's asks first and then swaps the two; Reset drops the player's map.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rebind {
    pub step: Step,
    /// The listed action whose pad button the last input asked for and could not take (the action waiting has no
    /// button to give it in trade); cleared by the next input, start or cancel.
    pub refused: Option<usize>,
}

impl Rebind {
    pub fn start(&mut self, at: usize) {
        self.step = Step::Waiting(at);
        self.refused = None;
    }

    pub fn cancel(&mut self) {
        self.step = Step::Idle;
        self.refused = None;
    }

    /// A rebind is open: the shell's raw input comes here first.
    pub fn capturing(&self) -> bool {
        self.step != Step::Idle
    }

    /// The listed action other than `at` that `input` gives now, if any.
    pub fn clash(c: &Controls, actions: &[Action], at: usize, input: Input) -> Option<usize> {
        let d = c.gives(input);
        actions.iter().enumerate().find(|(i, a)| *i != at && a.key == d).map(|(i, _)| i)
    }

    /// A raw input while a rebind is open (see `Outcome`); `settings` is the whole settings object.
    pub fn input(&mut self, actions: &[Action], settings: &mut Value, input: Input) -> Outcome {
        let c = Controls::from_settings(settings).0;
        if self.step != Step::Idle {
            self.refused = None;
        }
        match self.step {
            Step::Idle => Outcome::Ignored,
            Step::Waiting(_) if input == Input::Key(Key::Esc) => {
                self.cancel();
                Outcome::Cancelled
            }
            Step::Waiting(_) if matches!(input, Input::Key(k) if !capturable(k)) => Outcome::Held,
            Step::Waiting(at) if at >= actions.len() => {
                self.cancel();
                Outcome::Cancelled
            }
            Step::Waiting(at) => match Self::clash(&c, actions, at, input) {
                // a swap would leave `with` with no pad button: refused, rebind `with` first
                Some(with) if matches!(input, Input::Pad(_)) && c.pad_for(actions[at].key).is_none() => {
                    self.refused = Some(with);
                    Outcome::Refused
                }
                Some(with) => {
                    self.step = Step::Clash { at, input, with };
                    Outcome::Asked
                }
                None => self.bind(actions, settings, at, input),
            },
            // The question takes Enter (or what gives Enter: the pad's confirm) as Swap, Esc (or the pad's back) as Cancel.
            // A raw Esc always cancels, as the prompt says.
            Step::Clash { .. } if input == Input::Key(Key::Esc) => {
                self.cancel();
                Outcome::Cancelled
            }
            Step::Clash { .. } => match c.gives(input) {
                Key::Enter => self.swap(actions, settings),
                Key::Esc => {
                    self.cancel();
                    Outcome::Cancelled
                }
                _ => Outcome::Held,
            },
        }
    }

    /// The swap question's Yes: the input moves to the action asked for, the other action takes its old one.
    pub fn swap(&mut self, actions: &[Action], settings: &mut Value) -> Outcome {
        match self.step {
            Step::Clash { at, input, .. } if at < actions.len() => self.bind(actions, settings, at, input),
            _ => Outcome::Held,
        }
    }

    fn bind(&mut self, actions: &[Action], settings: &mut Value, at: usize, input: Input) -> Outcome {
        let mut c = Controls::from_settings(settings).0;
        match input {
            Input::Key(k) => c.bind_key(actions[at].key, k),
            Input::Pad(p) => c.bind_pad(actions[at].key, p),
        }
        settings[SETTINGS_KEY] = c.to_json();
        self.cancel();
        Outcome::Bound
    }

    /// Reset to defaults: the player's map is dropped (the defaults are what no setting gives).
    pub fn reset(&mut self, settings: &mut Value) {
        self.cancel();
        if let Some(o) = settings.as_object_mut() {
            // the file keeps its order (preserve_order: `remove` would swap the last key in)
            o.shift_remove(SETTINGS_KEY);
        }
    }
}

/// How long a held direction waits before it repeats, and then between repeats.
pub const REPEAT_FIRST: Duration = Duration::from_millis(350);
pub const REPEAT_EVERY: Duration = Duration::from_millis(110);
/// How far a stick goes before it counts as a direction (of 1.0).
pub const STICK_DEAD: f32 = 0.5;

/// A held pad direction (the d-pad or the left stick) as repeated arrow keys: `press` when it starts, `poll` each frame.
#[derive(Debug, Default)]
pub struct Repeat {
    held: Option<(Key, Instant)>,
}

impl Repeat {
    /// The direction changed to `k` (None: let go). Returns the key to send now.
    pub fn set(&mut self, k: Option<Key>, now: Instant) -> Option<Key> {
        if self.held.map(|(h, _)| h) == k {
            return None;
        }
        self.held = k.map(|k| (k, now + REPEAT_FIRST));
        k
    }
    /// The key a held direction repeats now, if its time has come.
    pub fn poll(&mut self, now: Instant) -> Option<Key> {
        let (k, at) = self.held?;
        if now < at {
            return None;
        }
        self.held = Some((k, now + REPEAT_EVERY));
        Some(k)
    }
    /// The held direction, for the walk (`Held::dir`).
    pub fn held(&self) -> Option<Key> {
        self.held.map(|(k, _)| k)
    }
    /// When the next repeat is due, so the loop can wake for it.
    pub fn due(&self) -> Option<Instant> {
        self.held.map(|(_, at)| at)
    }
}

/// The left stick's direction: the larger axis past the dead zone (y up is positive, as pads report it).
pub fn stick_dir(x: f32, y: f32) -> Option<Key> {
    if x.abs() < STICK_DEAD && y.abs() < STICK_DEAD {
        return None;
    }
    Some(if x.abs() > y.abs() {
        if x > 0.0 { Key::Right } else { Key::Left }
    } else if y > 0.0 {
        Key::Up
    } else {
        Key::Down
    })
}
