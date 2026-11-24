//! ENGINE-64 (LIVE-RELOAD1): live reload in a dev run. With `Options::live` (a game turns it on for `dev=1` in a build
//! with its dev tools; `start::Start` reads the key, a game without dev tools of its own keeps it), a thread looks at
//! every file under the asset layers and the game's data folders (`Context::data_dirs`) every `POLL` (change time and
//! length; no file events, no new crate) and hands the shell the paths that changed, appeared or went. The shell
//! (`Shell::live_poll`, before a frame) reloads what they touch, keeping the game state: art files (`Art::forget`),
//! `ui/theme.json` (`theme::reload`), screen layouts (`screen_layout::Layouts` reads a changed file again while live
//! reload is on), sounds (`Audio::forget`) and data (`Context::reload_data`, then `Screen::data_reloaded` on each screen).
//! Each reload shows one line for a few seconds (`LINE_FOR`); a refused data reload is a notice naming the error. Nothing
//! reloads while a save is being written (`aegis_save::writing`). Never on the web (no dev files).

use aegis_core::clock::{Duration, SystemTime};
use aegis_core::vfs::dev;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;

/// How often the watcher looks at the files.
pub const POLL: Duration = Duration::from_millis(500);
/// How long a reload's line shows.
pub const LINE_FOR: Duration = Duration::from_secs(3);
/// The theme file over the layers.
pub const THEME_FILE: &str = "ui/theme.json";
/// The screen layouts' folder over the layers.
pub const LAYOUTS_DIR: &str = "ui/layouts/";
/// The most files one watcher follows (a folder past it is followed in part, with a log line).
pub const FILES_MAX: usize = 50_000;

/// Some watcher runs in this process: layouts read a changed file again (`screen_layout::Layouts`), as a debug build's do.
static ON: AtomicBool = AtomicBool::new(false);

/// Whether live reload runs in this process.
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Every file under `dirs` with its change time and length (hidden files and folders skipped).
pub fn scan(dirs: &[PathBuf]) -> BTreeMap<PathBuf, (SystemTime, u64)> {
    let mut out = BTreeMap::new();
    let mut todo: Vec<PathBuf> = dirs.to_vec();
    while let Some(d) = todo.pop() {
        let Ok(entries) = dev::list_dir(&d) else { continue };
        for p in entries {
            if p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.')) {
                continue;
            }
            if dev::is_dir(&p) {
                todo.push(p);
            } else if let Some(st) = dev::stamp(&p) {
                if out.len() >= FILES_MAX {
                    return out;
                }
                out.insert(p, st);
            }
        }
    }
    out
}

/// The files that differ between two scans: changed, new or gone.
pub fn changed(before: &BTreeMap<PathBuf, (SystemTime, u64)>, after: &BTreeMap<PathBuf, (SystemTime, u64)>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = after.iter().filter(|(p, st)| before.get(*p) != Some(*st)).map(|(p, _)| p.clone()).collect();
    out.extend(before.keys().filter(|p| !after.contains_key(*p)).cloned());
    out
}

/// What a batch of changed files touches.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Changes {
    /// Art files by their path over the layers (pictures and the sidecars beside them).
    pub art: Vec<String>,
    pub theme: bool,
    pub layouts: Vec<String>,
    /// Sound files by their path over the layers.
    pub sounds: Vec<String>,
    /// Changed files in the game's data folders.
    pub data: Vec<PathBuf>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.art.is_empty() && !self.theme && self.layouts.is_empty() && self.sounds.is_empty() && self.data.is_empty()
    }

    /// Sorts `paths` by what they touch: a file in a data folder is data; one under an asset layer is the theme, a
    /// layout, a sound (`.ogg`, `.wav`, `.mp3`, `.flac`) or else art; anything else is left out.
    pub fn of(paths: &[PathBuf], layers: &[PathBuf], data: &[PathBuf]) -> Changes {
        let mut c = Changes::default();
        for p in paths {
            if data.iter().any(|d| p.starts_with(d)) {
                c.data.push(p.clone());
                continue;
            }
            let Some(rel) = layers.iter().find_map(|r| p.strip_prefix(r).ok()).map(rel_of) else { continue };
            let ext = Path::new(&rel).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
            if rel == THEME_FILE {
                c.theme = true;
            } else if rel.starts_with(LAYOUTS_DIR) {
                c.layouts.push(rel);
            } else if matches!(ext.as_str(), "ogg" | "wav" | "mp3" | "flac") {
                c.sounds.push(rel);
            } else {
                c.art.push(rel);
            }
        }
        for v in [&mut c.art, &mut c.layouts, &mut c.sounds] {
            v.sort();
            v.dedup();
        }
        c.data.sort();
        c.data.dedup();
        c
    }

    /// The reload's line ("Reloaded 3 art files, the theme"; the data's own words come from the shell).
    pub fn line(&self) -> Option<String> {
        let n = |k: usize, one: &str, many: &str| (k > 0).then(|| if k == 1 { format!("1 {one}") } else { format!("{k} {many}") });
        let parts: Vec<String> = [n(self.art.len(), "art file", "art files"), self.theme.then(|| "the theme".to_string()), n(self.layouts.len(), "layout", "layouts"), n(self.sounds.len(), "sound", "sounds")].into_iter().flatten().collect();
        (!parts.is_empty()).then(|| format!("Reloaded {}", parts.join(", ")))
    }
}

/// A path over a layer as the art and sound caches name it (forward slashes).
fn rel_of(p: &Path) -> String {
    p.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/")
}

/// The watcher: its thread and what it found since the last `take`.
pub struct Watch {
    rx: mpsc::Receiver<Vec<PathBuf>>,
    pub layers: Vec<PathBuf>,
    pub data: Vec<PathBuf>,
    /// Found but not reloaded yet (a save was being written).
    held: BTreeSet<PathBuf>,
}

impl Watch {
    /// Starts watching `layers` (asset roots) and `data` (data folders); None where there are no dev files (the web).
    /// The thread stops once the watch is dropped.
    pub fn start(layers: Vec<PathBuf>, data: Vec<PathBuf>) -> Option<Watch> {
        if !dev::ON {
            return None;
        }
        let (tx, rx) = mpsc::channel();
        let dirs: Vec<PathBuf> = layers.iter().chain(&data).cloned().collect();
        std::thread::Builder::new()
            .name("aegis-live".into())
            .spawn(move || {
                let mut seen = scan(&dirs);
                loop {
                    std::thread::sleep(POLL);
                    let now = scan(&dirs);
                    let diff = changed(&seen, &now);
                    seen = now;
                    if diff.is_empty() {
                        // a closed watch ends the thread on the next change, or here (an empty batch is a probe)
                        if tx.send(Vec::new()).is_err() {
                            return;
                        }
                        continue;
                    }
                    if tx.send(diff).is_err() {
                        return;
                    }
                }
            })
            .ok()?;
        ON.store(true, Ordering::Relaxed);
        Some(Watch { rx, layers, data, held: BTreeSet::new() })
    }

    /// The changes found since the last call, unless `wait` (a save being written): then they are kept for later.
    pub fn take(&mut self, wait: bool) -> Changes {
        while let Ok(batch) = self.rx.try_recv() {
            self.held.extend(batch);
        }
        if wait || self.held.is_empty() {
            return Changes::default();
        }
        let paths: Vec<PathBuf> = std::mem::take(&mut self.held).into_iter().collect();
        Changes::of(&paths, &self.layers, &self.data)
    }
}
