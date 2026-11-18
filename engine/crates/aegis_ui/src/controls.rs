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
