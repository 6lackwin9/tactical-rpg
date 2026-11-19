//! INPUT-MAP1 (ENGINE-7): the gamepad, on Windows (`gilrs`, Windows.Gaming.Input). Buttons go through the control map
//! (`controls::Controls::pad`); the d-pad and the left stick are a held direction that repeats as arrow keys
//! (`controls::Repeat`) and sets `Held::dir` for screens that walk on a held key. With no pad connected nothing runs.

use super::*;
use crate::controls::{stick_dir, Pad, Repeat};

/// A pad's state between events.
pub(super) struct Gamepad {
    gilrs: gilrs::Gilrs,
    /// The d-pad directions held, in press order (the last one counts), and the left stick.
    dpad: Vec<Key>,
    stick: (f32, f32),
    repeat: Repeat,
}

fn pad_of(b: gilrs::Button) -> Option<Pad> {
    use gilrs::Button as B;
    Some(match b {
        B::South => Pad::South,
        B::East => Pad::East,
        B::West => Pad::West,
        B::North => Pad::North,
        B::LeftTrigger => Pad::LeftBumper,
        B::RightTrigger => Pad::RightBumper,
        B::Select => Pad::Select,
        B::Start => Pad::Start,
        B::DPadUp => Pad::DUp,
        B::DPadDown => Pad::DDown,
        B::DPadLeft => Pad::DLeft,
        B::DPadRight => Pad::DRight,
        _ => return None,
    })
}

impl Gamepad {
    /// None when the system offers no gamepad input.
    pub(super) fn open() -> Option<Gamepad> {
        gilrs::Gilrs::new().ok().map(|gilrs| Gamepad { gilrs, dpad: Vec::new(), stick: (0.0, 0.0), repeat: Repeat::default() })
    }

    /// A pad is connected: the loop wakes at least this often to read it.
    pub(super) fn connected(&self) -> bool {
        self.gilrs.gamepads().next().is_some()
    }

    /// When a held direction repeats next.
    pub(super) fn due(&self) -> Option<Instant> {
        self.repeat.due()
    }

    /// Reads the pad's events into `s`. Returns whether anything happened (the pacer wakes for it).
    pub(super) fn poll<G: Context>(&mut self, s: &mut Shell<G>) -> bool {
        let now = Instant::now();
        let mut any = false;
        let mut dir_changed = false;
        while let Some(ev) = self.gilrs.next_event() {
            any = true;
            match ev.event {
                gilrs::EventType::ButtonPressed(b, _) => {
                    let Some(p) = pad_of(b) else { continue };
                    let k = s.controls().pad(p);
                    if k.dir().is_some() && matches!(p, Pad::DUp | Pad::DDown | Pad::DLeft | Pad::DRight) {
                        self.dpad.retain(|d| *d != k);
                        self.dpad.push(k);
                        dir_changed = true;
                    } else {
                        s.pad(p);
                    }
                }
                gilrs::EventType::ButtonReleased(b, _) => {
                    if let Some(p @ (Pad::DUp | Pad::DDown | Pad::DLeft | Pad::DRight)) = pad_of(b) {
                        let k = s.controls().pad(p);
                        self.dpad.retain(|d| *d != k);
                        dir_changed = true;
                    }
                }
                gilrs::EventType::AxisChanged(gilrs::Axis::LeftStickX, v, _) => {
                    self.stick.0 = v;
                    dir_changed = true;
                }
                gilrs::EventType::AxisChanged(gilrs::Axis::LeftStickY, v, _) => {
                    self.stick.1 = v;
                    dir_changed = true;
                }
                gilrs::EventType::Disconnected => {
                    self.dpad.clear();
                    self.stick = (0.0, 0.0);
                    dir_changed = true;
                }
                _ => {}
            }
        }
        if dir_changed {
            let want = self.dpad.last().copied().or_else(|| stick_dir(self.stick.0, self.stick.1));
            if let Some(k) = self.repeat.set(want, now) {
                s.key(k);
            }
            s.cx.held.dir = self.repeat.held().and_then(Key::dir);
        }
        if let Some(k) = self.repeat.poll(now) {
            if !s.stack.last().is_some_and(|t| t.hold_walk()) {
                s.key(k);
            }
            any = true;
        }
        any
    }
}
