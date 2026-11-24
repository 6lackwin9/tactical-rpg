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

    /// CONTROLS-MIGRATE1: each action a fixed key gives (a map saved before `fixed` refused it) goes back to its own key,
    /// by `bind_key`'s swap (so the key that held that key gets its old action back), and the fixed key gives itself
    /// again; returns those actions in the map's order.
    pub fn release_fixed(&mut self, fixed: &[Fixed]) -> Vec<Key> {
        let mut out = Vec::new();
        // each pass frees one fixed key; a swap can re-take another fixed key, so the passes are bounded
        for _ in 0..=self.keys.len() + fixed.len() {
            let Some(&(f, t)) = self.keys.iter().find(|(f, _)| fixed.iter().any(|x| x.key == *f)) else { break };
            self.bind_key(t, t);
            if self.key(f) != f {
                self.set_key(f, f);
            }
            out.push(t);
        }
        out
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

    /// CONTROLS-UI1B: the key that gives `action`, as an on-screen hint reads it ("P", "Enter"); "no key" when none
    /// does. A screen's hint goes through here (or `hint`), never a fixed letter, so a rebind shows everywhere.
    pub fn key_hint(&self, action: Key) -> String {
        self.key_for(action).map_or_else(|| "no key".to_string(), key_label)
    }

    /// CONTROLS-UI1B: the key and, when the action has one, the pad button that give `action` ("Esc / Pad B", "P").
    pub fn hint(&self, action: Key) -> String {
        match self.pad_for(action) {
            Some(p) => format!("{} / {}", self.key_hint(action), p.label()),
            None => self.key_hint(action),
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

/// CONTROLS-UI1B: a key a game's screens read that its table does not list (a question's yes, a number row): a rebind
/// may not take it, and the refusal names `label` (what it does) and `screen` (where).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fixed {
    pub key: Key,
    pub label: &'static str,
    pub screen: &'static str,
}

/// CONTROLS-MIGRATE1: the settings' control map with every action on a fixed key put back on its own key
/// (`Controls::release_fixed`), written back only when something moved; returns the actions moved (empty: the settings
/// are untouched). A game calls it when its settings load, and saves them when it returns any.
pub fn release_fixed(settings: &mut Value, fixed: &[Fixed]) -> Vec<Key> {
    let mut c = Controls::from_settings(settings).0;
    let moved = c.release_fixed(fixed);
    if !moved.is_empty() {
        settings[SETTINGS_KEY] = c.to_json();
    }
    moved
}

/// CONTROLS-UI1B: how long a rebind waits with no input before it ends with nothing changed, so a player whose every
/// button is bindable (a pad-only player) can always leave it.
pub const WAIT: Duration = Duration::from_secs(5);

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
    /// (`Rebind::refused` names that action), or a fixed key (`Rebind::refused_fixed`); the rebind still waits.
    Refused,
    /// Ended with nothing changed: Esc, the input the action already has, or `WAIT` with no input (`Rebind::expire`).
    Cancelled,
    /// The input is another action's: the swap question is open.
    Asked,
    /// The binding changed (or was already so); the settings hold the new map.
    Bound,
}

/// CONTROLS-UI1: the rebinding flow over the settings key `controls`, game-agnostic: a game lists its actions (`Action`)
/// and labels; a click starts a rebind, the next raw key or pad button binds it (Esc, the input the action already has,
/// or `WAIT` with no input ends it unchanged), an input that is another action's asks first and then swaps the two, a
/// fixed key (`Fixed`) is refused; Reset drops the player's map.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rebind {
    pub step: Step,
    /// The listed action whose pad button the last input asked for and could not take (the action waiting has no
    /// button to give it in trade); cleared by the next input, start or cancel.
    pub refused: Option<usize>,
    /// CONTROLS-UI1B: the fixed key (an index into the game's `Fixed` list) the last input was; cleared as `refused`.
    pub refused_fixed: Option<usize>,
    /// CONTROLS-UI1B: when the open rebind ends by itself (`WAIT` after its start or its last input; the shot clock, so
    /// a stepped run is frame-exact).
    pub until: Option<Instant>,
}

impl Rebind {
    pub fn start(&mut self, at: usize) {
        self.step = Step::Waiting(at);
        self.refused = None;
        self.refused_fixed = None;
        self.until = Some(Instant::now() + WAIT);
    }

    pub fn cancel(&mut self) {
        self.step = Step::Idle;
        self.refused = None;
        self.refused_fixed = None;
        self.until = None;
    }

    /// CONTROLS-UI1B: the open rebind at `now` (a screen's tick): past `until` it ends with nothing changed. True: it
    /// ended now.
    pub fn expire(&mut self, now: Instant) -> bool {
        if self.capturing() && self.until.is_some_and(|u| now >= u) {
            self.cancel();
            return true;
        }
        false
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

    /// A raw input while a rebind is open (see `Outcome`); `fixed` the game's fixed keys; `settings` is the whole
    /// settings object. Any input restarts the `WAIT`.
    pub fn input(&mut self, actions: &[Action], fixed: &[Fixed], settings: &mut Value, input: Input) -> Outcome {
        let c = Controls::from_settings(settings).0;
        if self.step != Step::Idle {
            self.refused = None;
            self.refused_fixed = None;
            self.until = Some(Instant::now() + WAIT);
        }
        match self.step {
            Step::Idle => Outcome::Ignored,
            Step::Waiting(_) if input == Input::Key(Key::Esc) => {
                self.cancel();
                Outcome::Cancelled
            }
            Step::Waiting(_) if matches!(input, Input::Key(k) if !capturable(k)) => Outcome::Held,
            Step::Waiting(at) if at >= actions.len() || c.gives(input) == actions[at].key => {
                // the input the action already has: nothing to change (a pad-only player's way out)
                self.cancel();
                Outcome::Cancelled
            }
            Step::Waiting(_) if matches!(input, Input::Key(k) if fixed.iter().any(|f| f.key == k)) => {
                self.refused_fixed = fixed.iter().position(|f| Input::Key(f.key) == input);
                Outcome::Refused
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
/// ENGINE-63 (PAD-TUNE1): its delay and rate are the player's (`PadTuning`, `tune`); the defaults are today's.
#[derive(Debug, Default)]
pub struct Repeat {
    held: Option<(Key, Instant)>,
    tuning: PadTuning,
}

impl Repeat {
    /// The delay and the rate from now on (a direction held already keeps the wait it has).
    pub fn tune(&mut self, t: PadTuning) {
        self.tuning = t;
    }
    /// The direction changed to `k` (None: let go). Returns the key to send now.
    pub fn set(&mut self, k: Option<Key>, now: Instant) -> Option<Key> {
        if self.held.map(|(h, _)| h) == k {
            return None;
        }
        self.held = k.map(|k| (k, now + self.tuning.repeat_delay));
        k
    }
    /// The key a held direction repeats now, if its time has come.
    pub fn poll(&mut self, now: Instant) -> Option<Key> {
        let (k, at) = self.held?;
        if now < at {
            return None;
        }
        self.held = Some((k, now + self.tuning.repeat_every));
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

/// ENGINE-63 (PAD-TUNE1): the settings key of the gamepad's tuning, apart from `controls` (the bindings):
/// `{"gamepad": {"dead_zone": 0.5, "repeat_delay_ms": 350, "repeat_every_ms": 110}}`. A missing entry is its default; a
/// value out of range is clamped onto its steps. The keyboard's own repeat stays the system's.
pub const PAD_TUNING_KEY: &str = "gamepad";

/// One tunable of the gamepad, for a settings screen: its key in `PAD_TUNING_KEY`, words, range, step, default and unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PadSetting {
    pub key: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub default: f64,
    /// "ms", or "" for a share of the stick's reach.
    pub unit: &'static str,
}

impl PadSetting {
    /// `v` clamped to the range and put on the nearest step.
    pub fn snap(&self, v: f64) -> f64 {
        let n = ((v.clamp(self.min, self.max) - self.min) / self.step).round();
        ((self.min + n * self.step) * 1000.0).round() / 1000.0
    }
    /// How many steps the range has (a slider's positions).
    pub fn steps(&self) -> usize {
        ((self.max - self.min) / self.step).round() as usize + 1
    }
    /// The step `v` is on (a slider's index).
    pub fn index(&self, v: f64) -> usize {
        ((self.snap(v) - self.min) / self.step).round() as usize
    }
    /// The value at step `i`.
    pub fn at(&self, i: usize) -> f64 {
        self.snap(self.min + i as f64 * self.step)
    }
    /// `v` in the setting's words ("0.50", "350 ms").
    pub fn text(&self, v: f64) -> String {
        if self.unit.is_empty() {
            format!("{v:.2}")
        } else {
            format!("{v:.0} {}", self.unit)
        }
    }
}

/// The gamepad's tunables, in the order a settings screen lists them (ranges from the PAD-TUNE1 row: the dead zone 0.15
/// to 0.6; the repeat's delay and rate around today's 350 ms and 110 ms).
pub const PAD_SETTINGS: [PadSetting; 3] = [
    PadSetting { key: "dead_zone", label: "Stick Dead Zone", help: "How far the left stick goes before it counts as a direction", min: 0.15, max: 0.6, step: 0.05, default: STICK_DEAD as f64, unit: "" },
    PadSetting { key: "repeat_delay_ms", label: "Repeat Delay", help: "How long a held d-pad or stick direction waits before it repeats", min: 150.0, max: 600.0, step: 25.0, default: REPEAT_FIRST.as_millis() as f64, unit: "ms" },
    PadSetting { key: "repeat_every_ms", label: "Repeat Rate", help: "The time between repeats of a held d-pad or stick direction", min: 50.0, max: 250.0, step: 10.0, default: REPEAT_EVERY.as_millis() as f64, unit: "ms" },
];

/// ENGINE-63 (PAD-TUNE1): the gamepad's tuning in force: the stick's dead zone and the held direction's repeat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PadTuning {
    pub dead_zone: f32,
    pub repeat_delay: Duration,
    pub repeat_every: Duration,
}

impl Default for PadTuning {
    fn default() -> Self {
        PadTuning { dead_zone: STICK_DEAD, repeat_delay: REPEAT_FIRST, repeat_every: REPEAT_EVERY }
    }
}

impl PadTuning {
    /// The tuning `settings` (the whole object) gives: each entry clamped onto its steps, a missing one its default.
    pub fn of(settings: &Value) -> PadTuning {
        let v = |i: usize| PadTuning::value(settings, PAD_SETTINGS[i].key);
        PadTuning { dead_zone: v(0) as f32, repeat_delay: Duration::from_millis(v(1) as u64), repeat_every: Duration::from_millis(v(2) as u64) }
    }
    /// One tunable's value in `settings` (snapped; its default when unset or no number). An unknown key is 0.
    pub fn value(settings: &Value, key: &str) -> f64 {
        let Some(p) = PAD_SETTINGS.iter().find(|p| p.key == key) else { return 0.0 };
        settings.get(PAD_TUNING_KEY).and_then(|g| g.get(key)).and_then(Value::as_f64).map_or(p.default, |v| p.snap(v))
    }
    /// Writes one tunable into `settings` (snapped). False for a key `PAD_SETTINGS` does not name, or no change.
    pub fn set(settings: &mut Value, key: &str, v: f64) -> bool {
        let Some(p) = PAD_SETTINGS.iter().find(|p| p.key == key) else { return false };
        let v = p.snap(v);
        if settings.get(PAD_TUNING_KEY).and_then(|g| g.get(key)).and_then(Value::as_f64) == Some(v) {
            return false;
        }
        if !settings.is_object() {
            *settings = json!({});
        }
        if !settings[PAD_TUNING_KEY].is_object() {
            settings[PAD_TUNING_KEY] = json!({});
        }
        settings[PAD_TUNING_KEY][key] = if p.unit.is_empty() { json!(v) } else { json!(v as i64) };
        true
    }
    /// The left stick's direction under this dead zone (`stick_dir`).
    pub fn stick_dir(&self, x: f32, y: f32) -> Option<Key> {
        stick_dir_past(x, y, self.dead_zone)
    }
    /// The live Test line: where the stick is and the direction it reads ("none" inside the dead zone).
    pub fn test_line(&self, stick: (f32, f32)) -> String {
        let (x, y) = stick;
        let dir = match self.stick_dir(x, y) {
            Some(k) => key_label(k),
            None => "none".to_string(),
        };
        format!("Stick x {x:+.2}  y {y:+.2}  reads {dir}  (dead zone {:.2})", self.dead_zone)
    }
}

/// The left stick's direction: the larger axis past the default dead zone (y up is positive, as pads report it).
pub fn stick_dir(x: f32, y: f32) -> Option<Key> {
    stick_dir_past(x, y, STICK_DEAD)
}

/// The left stick's direction: the larger axis past the dead zone `dead`.
pub fn stick_dir_past(x: f32, y: f32, dead: f32) -> Option<Key> {
    if x.abs() < dead && y.abs() < dead {
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
