//! Live tuning files: a JSON file of feel values (timings, sizes, feedback colours) read into the game's own type and
//! re-read when the file changes, so the user can tune while playing and the agent reads the exact values.

use aegis_core::clock::SystemTime;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Native: the file's modified time, so an edit while playing reloads it (hot reload). Web (WEBREL1-E1b): a fixed
/// stamp while the bundled file exists, so it is read once through the read seam and never again.
#[cfg(not(target_arch = "wasm32"))]
fn stamp_of(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

#[cfg(target_arch = "wasm32")]
fn stamp_of(p: &Path) -> Option<SystemTime> {
    aegis_core::vfs::is_file(p).then_some(aegis_core::clock::UNIX_EPOCH)
}

/// A game's tuning values: the defaults, and the values a tuning file gives.
pub trait Tuned: Default {
    fn from_json(v: &Value) -> Self;
}

/// The tuning file plus change detection.
pub struct TuningFile<T> {
    pub path: Option<PathBuf>,
    pub value: T,
    pub raw: Value,
    stamp: Option<SystemTime>,
}

impl<T: Tuned> TuningFile<T> {
    pub fn new(path: Option<PathBuf>) -> Self {
        let mut t = Self { path, value: T::default(), raw: Value::Null, stamp: None };
        t.reload_if_changed();
        t
    }

    /// Re-reads the file when its timestamp moved; true when the values changed.
    pub fn reload_if_changed(&mut self) -> bool {
        let Some(p) = &self.path else { return false };
        let stamp = stamp_of(p);
        if stamp.is_none() || stamp == self.stamp {
            return false;
        }
        self.stamp = stamp;
        let Ok(text) = aegis_core::vfs::read_to_string(p) else { return false };
        let Ok(v) = serde_json::from_str::<Value>(&text) else { return false };
        self.value = T::from_json(&v);
        self.raw = v;
        true
    }
}
