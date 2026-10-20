//! The playtest hub (PLAYTEST1 PlaytestHub on the engine): while recording is on, one recording per app run, opened
//! on the first thing worth keeping and saved after every addition into the game's playtest folder (atomic, so the
//! file is whole whenever the app stops). The game hands it each command-host session, again after every round while
//! it runs (`put_session`); the shell feeds the UI trail and the notes into the same recording.

use aegis_host::playtest::{self, Recording, Session};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A hub with recording on exists in this process (the report functions do nothing otherwise).
static RECORDING: AtomicBool = AtomicBool::new(false);

/// How often a changed trail is written (sessions and notes save at once).
pub const TRAIL_SAVE_EVERY: Duration = Duration::from_secs(5);
/// A hover is logged when the pointer rested this long on one control.
pub const HOVER_MS: u64 = 700;
/// Errors kept per recording.
pub const ERRORS_MAX: usize = 500;

/// Errors reported from anywhere (a panic hook, code that hits a problem), drained into the trail each frame.
static ERRORS: Mutex<Vec<Value>> = Mutex::new(Vec::new());

/// Tests that report into or drain the shared queues hold this lock (the lib tests run in parallel).
#[cfg(test)]
pub(crate) static TEST_QUEUES: Mutex<()> = Mutex::new(());

/// Reports an error or warning to the playtest trail (`level`: error, warning, panic; `place`: "file:line" or "").
/// A panic that ends the process (on the event loop) is reported but never written: the next frame never comes.
pub fn report_error(level: &str, msg: &str, place: &str) {
    if !RECORDING.load(Ordering::Relaxed) {
        return;
    }
    if let Ok(mut q) = ERRORS.lock() {
        if q.len() < ERRORS_MAX {
            q.push(json!({"type": "error", "level": level, "msg": msg.chars().take(300).collect::<String>(), "where": place}));
        }
    }
}

/// Art the loader could not find or read: the keys seen (one event each per run) and the events not yet drained.
static ASSETS: Mutex<(Vec<String>, Vec<Value>)> = Mutex::new((Vec::new(), Vec::new()));

/// Reports art that fell back (`source`: missing_file or load_failed), once per category / id / variant / source.
pub fn report_asset(category: &str, id: &str, variant: &str, source: &str) {
    if !RECORDING.load(Ordering::Relaxed) {
        return;
    }
    let key = format!("{category}/{id}/{variant}/{source}");
    if let Ok(mut q) = ASSETS.lock() {
        if !q.0.contains(&key) {
            q.0.push(key);
            q.1.push(json!({"type": "asset", "ns": category, "key": id, "variant": variant, "source": source, "screen": ""}));
        }
    }
}

/// Sends panics to the trail too (the previous hook still runs).
fn hook_panics() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let place = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let msg = info.payload().downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_default();
        report_error("panic", &msg, &place);
        previous(info);
    }));
}

pub struct Playtests {
    /// Where recordings go (the game's user folder `playtests/`).
    pub dir: PathBuf,
    /// Recording on (a debug build's default; the Scene Runner toggles it).
    pub on: bool,
    /// The content fingerprint recordings carry (the game's data folder at start).
    pub content: Value,
    rec: Option<Recording>,
    started: Instant,
    /// The last save problem (shown by whoever asks).
    pub error: String,
    /// The trail changed since the last save.
    dirty: bool,
    last_save: Instant,
    errors: usize,
    /// The screen of the last `screen` event (stamped on asset events).
    screen: String,
}

impl Playtests {
    pub fn new(dir: PathBuf, on: bool, content: Value) -> Self {
        if on && !RECORDING.swap(true, Ordering::Relaxed) {
            hook_panics();
        }
        Self { dir, on, content, rec: None, started: Instant::now(), error: String::new(), dirty: false, last_save: Instant::now(), errors: 0, screen: String::new() }
    }

    /// Adds a trail event (`t` is set here): {type, ...} (the event types: `aegis_host::trail`); returns its index.
    pub fn trail(&mut self, mut event: Value) -> Option<usize> {
        if !self.on {
            return None;
        }
        if event["type"] == "screen" {
            self.screen = event["path"].as_str().unwrap_or("").to_string();
        }
        event["t"] = json!(self.ms());
        let rec = self.open();
        let at = rec.trail.len();
        rec.push_trail(event);
        let kept = at < rec.trail.len();
        self.dirty = true;
        kept.then_some(at)
    }

    /// Sets `key` of the trail event at `index` (a click's `handled`, known once the click ran).
    pub fn set_trail(&mut self, index: usize, key: &str, value: Value) {
        if let Some(e) = self.rec.as_mut().and_then(|r| r.trail.get_mut(index)) {
            e[key] = value;
            self.dirty = true;
        }
    }

    /// Each frame: drains the reported errors into the trail and writes a changed trail every TRAIL_SAVE_EVERY.
    pub fn tick(&mut self) {
        if !self.on {
            return;
        }
        let drained: Vec<Value> = ERRORS.lock().map(|mut q| q.drain(..).collect()).unwrap_or_default();
        for e in drained {
            if self.errors < ERRORS_MAX {
                self.errors += 1;
                self.trail(e);
            }
        }
        let assets: Vec<Value> = ASSETS.lock().map(|mut q| q.1.drain(..).collect()).unwrap_or_default();
        for mut e in assets {
            e["screen"] = json!(self.screen);
            self.trail(e);
        }
        if self.dirty && self.last_save.elapsed() >= TRAIL_SAVE_EVERY {
            self.save();
        }
    }

    /// Recording off, nowhere to write (tests, tools).
    pub fn off() -> Self {
        Self::new(PathBuf::new(), false, Value::Null)
    }

    /// Milliseconds since the hub started (trail and note times).
    pub fn ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    /// The recording of this run, when one is open.
    pub fn current(&self) -> Option<&Recording> {
        self.rec.as_ref()
    }

    fn open(&mut self) -> &mut Recording {
        if self.rec.is_none() {
            let id = playtest::new_id(&self.dir, playtest::now_secs());
            self.rec = Some(Recording::new(&id, "user", self.content.clone()));
        }
        self.rec.as_mut().expect("just opened")
    }

    /// Puts a session of the game's command host into this run's recording and saves: in place of the one at `slot`
    /// (the same session put earlier, while it runs: a crash keeps it up to its last save), else appended (`slot`
    /// then names it). Empty sessions are dropped.
    pub fn put_session(&mut self, slot: &mut Option<usize>, session: Session) {
        if !self.on || session.requests.is_empty() {
            return;
        }
        let battles = &mut self.open().battles;
        match slot.filter(|i| *i < battles.len()) {
            Some(i) => battles[i] = session,
            None => {
                battles.push(session);
                *slot = Some(battles.len() - 1);
            }
        }
        self.save();
    }

    /// Writes the open recording (nothing when none is open or recording is off).
    pub fn save(&mut self) -> Option<PathBuf> {
        if !self.on {
            return None;
        }
        let dir = self.dir.clone();
        let rec = self.rec.as_ref()?;
        self.dirty = false;
        self.last_save = Instant::now();
        match rec.save(&dir) {
            Ok(p) => {
                self.error.clear();
                Some(p)
            }
            Err(e) => {
                self.error = e;
                None
            }
        }
    }

    /// The recordings in the folder, newest first.
    pub fn list(&self) -> Vec<(PathBuf, Result<Recording, String>)> {
        playtest::list(&self.dir)
    }

    pub fn path_of(&self, id: &str) -> PathBuf {
        Path::new(&self.dir).join(format!("{id}.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn session(n: usize) -> Session {
        Session { requests: (0..n).map(|i| json!({"cmd": "step", "i": i})).collect(), checkpoints: Vec::new(), end: json!({"result": "won", "at": n}) }
    }

    #[test]
    fn one_recording_per_run_keeps_every_session_and_saves() {
        let _queues = TEST_QUEUES.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("aegis_hub_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut hub = Playtests::new(dir.clone(), true, json!({"total": "abc"}));
        assert!(hub.current().is_none(), "nothing opened before a session");
        let (mut a, mut b) = (None, None);
        hub.put_session(&mut a, session(0));
        assert!(hub.current().is_none() && a.is_none(), "an empty session is dropped");
        hub.put_session(&mut a, session(3));
        hub.put_session(&mut b, session(2));
        hub.put_session(&mut a, session(5));
        assert_eq!((a, b), (Some(0), Some(1)));
        let id = hub.current().unwrap().id.clone();
        let saved = Recording::load(&hub.path_of(&id)).unwrap();
        assert_eq!(saved.battles.len(), 2);
        assert_eq!(saved.battles[0].requests.len(), 5, "a running session is saved again in place");
        assert_eq!(saved.content["total"], json!("abc"));
        assert_eq!(hub.list().len(), 1);
        // Reported errors and missing art reach the trail on the next tick, art once per key, stamped with the screen.
        hub.trail(json!({"type": "screen", "path": "arena"}));
        report_error("error", "boom", "a.rs:1");
        report_asset("unit", "zz_hub_test", "token", "missing_file");
        report_asset("unit", "zz_hub_test", "token", "missing_file");
        hub.tick();
        let trail = &hub.current().unwrap().trail;
        assert!(trail.iter().any(|e| e["type"] == "error" && e["msg"] == "boom" && e["where"] == "a.rs:1"), "{trail:?}");
        let art: Vec<&Value> = trail.iter().filter(|e| e["type"] == "asset" && e["key"] == "zz_hub_test").collect();
        assert_eq!(art.len(), 1, "one event per key: {trail:?}");
        assert_eq!(art[0]["screen"], json!("arena"));
        let mut off = Playtests::new(dir.join("off"), false, Value::Null);
        off.tick();
        assert!(off.current().is_none(), "a hub with recording off drains nothing");
        off.put_session(&mut None, session(3));
        assert!(off.current().is_none() && !dir.join("off").exists(), "recording off writes nothing");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
