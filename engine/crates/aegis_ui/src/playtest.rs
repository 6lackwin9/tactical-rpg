//! The playtest hub (PLAYTEST1 PlaytestHub on the engine): while recording is on, one recording per app run, opened
//! on the first thing worth keeping and saved after every addition into the game's playtest folder (atomic, so the
//! file is whole whenever the app stops). The game hands it each command-host session, again after every round while
//! it runs (`put_session`); the shell feeds the UI trail and the notes into the same recording.

use aegis_host::playtest::{self, Recording, Session};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
}

impl Playtests {
    pub fn new(dir: PathBuf, on: bool, content: Value) -> Self {
        Self { dir, on, content, rec: None, started: Instant::now(), error: String::new() }
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
        let mut off = Playtests::new(dir.join("off"), false, Value::Null);
        off.put_session(&mut None, session(3));
        assert!(off.current().is_none() && !dir.join("off").exists(), "recording off writes nothing");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
