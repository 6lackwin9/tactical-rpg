//! The engine's one wall clock (WEBREL1-E1a, `docs/specs/WEBREL1.md` 2.3): `Instant`, `SystemTime`, `UNIX_EPOCH` and
//! `Duration`. Native: `std::time` underneath; web (`wasm32`): the `web-time` equivalents, because std's `Instant::now` /
//! `SystemTime::now` panic on `wasm32-unknown-unknown`. Runtime code reads the clock only through this module; the lint in
//! `tests/clock_lint.rs` refuses `std::time::{Instant, SystemTime}` anywhere else in non-test, non-CLI `src`.
//! Rules, sim and RNG code never read a clock (determinism, WEBREL1 2.4).
//!
//! The shot clock (ENGINE-17, SHOT-CLOCK1): `Instant` wraps the platform's instant so that a run can switch the clock
//! to stepped time (`step_on`). From then on `Instant::now()` reads a time that moves only by `advance` (a headless
//! scenario run moves it one fixed frame step per frame the driver asks for), so every animation a screen reads (idle
//! bobs, portrait loops, transitions, bar fills, floats) is at the same point in two runs of the same build, and their
//! shots are byte-identical. `Instant::wall()` always reads the real clock (the frame pacer, idle limits, perf timing).
//! Off (every window, every shipped run), `now()` is the platform's `now()` itself.

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant as Raw;

#[cfg(target_arch = "wasm32")]
pub use web_time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(target_arch = "wasm32")]
use web_time::Instant as Raw;

use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;

static STEPPED: AtomicBool = AtomicBool::new(false);
/// Stepped time: nanoseconds past `BASE`.
static OFFSET_NS: AtomicU64 = AtomicU64::new(0);
static BASE: OnceLock<Raw> = OnceLock::new();

/// A point in time, as the platform's `Instant`; `now()` follows the shot clock when it is stepped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(Raw);

impl Instant {
    /// Now: the real clock, or the stepped time once `step_on` was called.
    pub fn now() -> Self {
        if STEPPED.load(Ordering::Acquire) {
            Self(*BASE.get_or_init(Raw::now) + Duration::from_nanos(OFFSET_NS.load(Ordering::Acquire)))
        } else {
            Self(Raw::now())
        }
    }

    /// The real clock, stepped or not: for what measures or waits on real time (pacing, idle limits, perf).
    pub fn wall() -> Self {
        Self(Raw::now())
    }

    /// How long since this instant, on the same clock as `now()`; zero for an instant still ahead.
    pub fn elapsed(&self) -> Duration {
        Self::now().0.saturating_duration_since(self.0)
    }

    /// Real time since this instant (an instant read with `wall()`).
    pub fn wall_elapsed(&self) -> Duration {
        Raw::now().saturating_duration_since(self.0)
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.saturating_duration_since(earlier.0)
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.0.saturating_duration_since(earlier.0)
    }

    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.0.checked_duration_since(earlier.0)
    }

    pub fn checked_add(&self, d: Duration) -> Option<Instant> {
        self.0.checked_add(d).map(Self)
    }

    pub fn checked_sub(&self, d: Duration) -> Option<Instant> {
        self.0.checked_sub(d).map(Self)
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, d: Duration) -> Instant {
        Self(self.0 + d)
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, d: Duration) {
        self.0 += d;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, d: Duration) -> Instant {
        Self(self.0 - d)
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, d: Duration) {
        self.0 -= d;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, earlier: Instant) -> Duration {
        self.0.saturating_duration_since(earlier.0)
    }
}

/// Switches this process to stepped time, starting where the real clock is now; stays on until the process ends.
pub fn step_on() {
    if !STEPPED.load(Ordering::Acquire) {
        let base = *BASE.get_or_init(Raw::now);
        OFFSET_NS.store(Raw::now().saturating_duration_since(base).as_nanos() as u64, Ordering::Release);
        STEPPED.store(true, Ordering::Release);
    }
}

/// Moves stepped time on by `d` (nothing when the clock is real).
pub fn advance(d: Duration) {
    if STEPPED.load(Ordering::Acquire) {
        OFFSET_NS.fetch_add(d.as_nanos() as u64, Ordering::AcqRel);
    }
}

/// Whether the clock is stepped.
pub fn stepped() -> bool {
    STEPPED.load(Ordering::Acquire)
}

/// The platform's instant, for an API that takes one (a window's `WaitUntil`); real time only.
impl From<Instant> for Raw {
    fn from(i: Instant) -> Raw {
        i.0
    }
}

impl From<Raw> for Instant {
    fn from(r: Raw) -> Instant {
        Self(r)
    }
}

/// What the shot clock calls the date when it starts: 2026-01-01 00:00 UTC, in seconds after `UNIX_EPOCH`.
pub const STEP_EPOCH_S: u64 = 1_767_225_600;

/// The date and time now: `SystemTime::now()`, or on the shot clock `STEP_EPOCH_S` plus the stepped time, so what a run
/// derives from the date (a new game's clock seed) is the same in every run. For anything a run must not repeat (file
/// names, crash stamps) read `SystemTime::now()` itself.
pub fn system_now() -> SystemTime {
    if stepped() {
        let base = *BASE.get_or_init(Raw::now);
        let since = (Instant::now().0).saturating_duration_since(base);
        UNIX_EPOCH + Duration::from_secs(STEP_EPOCH_S) + since
    } else {
        SystemTime::now()
    }
}
