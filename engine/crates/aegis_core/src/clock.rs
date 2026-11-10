//! The engine's one wall clock (WEBREL1-E1a, `docs/specs/WEBREL1.md` 2.3): `Instant`, `SystemTime`, `UNIX_EPOCH` and
//! `Duration`. Native: `std::time` itself (re-exported, so every reading and every sum is std's, bit for bit). Web
//! (`wasm32`): the `web-time` equivalents, because std's `Instant::now` / `SystemTime::now` panic on
//! `wasm32-unknown-unknown`. Runtime code reads the clock only through this module; the lint in
//! `tests/clock_lint.rs` refuses `std::time::{Instant, SystemTime}` anywhere else in non-test, non-CLI `src`.
//! Rules, sim and RNG code never read a clock (determinism, WEBREL1 2.4).

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
pub use web_time::{Duration, Instant, SystemTime, UNIX_EPOCH};
