//! Frame pacing (FPSCAP1, engine-only): when the shell draws its next frame. Nothing here touches a window or reads a
//! clock, so a loop with no window can drive it the same way, and the rule is tested in `tests/pacing.rs`.
//!
//! Two rules. The cap holds: a frame comes only when the last one is at least the cap's interval old, however many events
//! wake the loop, because the deadline counts from the last frame and not from the wake. Nothing changes, nothing draws: a
//! screen that says so (`Screen::frame_need`, default `FrameNeed::Full`: a screen that has not said keeps drawing) waits
//! for input, the agent link or its own next change, with `HEARTBEAT` as the limit.

use aegis_core::clock::{Duration, Instant};

/// After input, or the last frame a screen needed in full, frames go on this long: a screen drawn each frame shows what
/// a click changed a frame or two later, and a short ease nobody declared still finishes.
pub const SETTLE: Duration = Duration::from_millis(250);

/// Even an idle screen gets a frame this often, so work the shell or a screen does per frame (a playtest trail save, a
/// tuning file reload) is late at worst, never lost.
pub const HEARTBEAT: Duration = Duration::from_secs(1);

/// For `FrameNeed::Within`: the frame interval of a screen at rest whose only motion is slow life (a portrait's breath
/// and blink, a body's idle clip), 15 frames a second.
pub const SLOW_LIFE: Duration = Duration::from_micros(66_667);

/// What a screen needs from the frame loop now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameNeed {
    /// Every frame the cap allows: the default, and the answer to any doubt.
    Full,
    /// Nothing it draws changes sooner than this.
    Within(Duration),
    /// Nothing it draws changes until input or the agent link wakes it.
    Idle,
}

impl FrameNeed {
    /// The more demanding of two needs (the screens drawn together share one frame loop).
    pub fn and(self, other: FrameNeed) -> FrameNeed {
        match (self, other) {
            (FrameNeed::Full, _) | (_, FrameNeed::Full) => FrameNeed::Full,
            (FrameNeed::Within(a), FrameNeed::Within(b)) => FrameNeed::Within(a.min(b)),
            (FrameNeed::Within(a), FrameNeed::Idle) | (FrameNeed::Idle, FrameNeed::Within(a)) => FrameNeed::Within(a),
            (FrameNeed::Idle, FrameNeed::Idle) => FrameNeed::Idle,
        }
    }
}

/// What the loop does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    /// Draw a frame now.
    Redraw,
    /// Sleep until then, unless an event wakes the loop first (then ask again).
    WaitUntil(Instant),
}

/// The pacing rule. `needed_at` is when the next frame is wanted (at or before `now`: it is wanted now); `cap` is the
/// smallest time between frames (None: unlimited, a frame as soon as it is wanted).
pub fn pace(last_frame: Option<Instant>, now: Instant, cap: Option<Duration>, needed_at: Instant) -> Pace {
    let allowed = match (last_frame, cap) {
        (Some(last), Some(every)) => last + every,
        _ => needed_at,
    };
    let at = needed_at.max(allowed);
    if at <= now {
        Pace::Redraw
    } else {
        Pace::WaitUntil(at)
    }
}

/// The frame loop's memory: the last frame and when the next one is wanted.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pacer {
    last_frame: Option<Instant>,
    /// Frames are wanted until then (input, a screen in full need).
    busy_until: Option<Instant>,
    /// When a resting screen wants its next frame (None: no frame drawn yet, one is wanted now).
    next: Option<Instant>,
}

impl Pacer {
    /// Something happened that a frame must show (input, focus, a resize, the agent link).
    pub fn touch(&mut self, now: Instant) {
        self.busy_until = Some(now + SETTLE);
    }

    /// A frame started at `started`, and what is on screen now needs `need`.
    pub fn framed(&mut self, started: Instant, need: FrameNeed) {
        self.last_frame = Some(started);
        match need {
            FrameNeed::Full => {
                self.busy_until = Some(started + SETTLE);
                self.next = None; // a resting screen's old heartbeat must not outlive it
            }
            FrameNeed::Within(d) => self.next = Some(started + d.min(HEARTBEAT)),
            FrameNeed::Idle => self.next = Some(started + HEARTBEAT),
        }
    }

    /// When the next frame is wanted: now while busy, else when the resting screen asked for it.
    pub fn needed_at(&self, now: Instant) -> Instant {
        match (self.busy_until, self.next) {
            (Some(until), _) if now < until => now,
            (_, Some(next)) => next,
            _ => now,
        }
    }

    /// Draw now, or sleep until when. Ask again after every wake.
    pub fn decide(&self, now: Instant, cap: Option<Duration>) -> Pace {
        pace(self.last_frame, now, cap, self.needed_at(now))
    }
}
