//! Sound and music (Godot AudioDirector, MusicDirector's player, VolumePreview's throttle), game-agnostic: sound files
//! under the game's audio folder, addressed by relative path ("sfx/hit.wav", "music/town.ogg"); four levels
//! (master, music, effects, interface, 0-100, from the settings) and mute; a voice pool per bus (the oldest restarts
//! when all are busy); one music track wanted at a time, crossfaded and looped. The game decides which sound and which
//! track (screens name their music, `Screen::music`). Without an audio device (or headless) everything is silent.

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

const SFX_VOICES: usize = 8;
const UI_VOICES: usize = 2;
/// Music plays 6 dB under the effects (Godot MUSIC_DB -6).
const MUSIC_GAIN: f32 = 0.501;
const CROSSFADE_SEC: f32 = 1.0;
const FIRST_FADE_SEC: f32 = 2.0;
/// A volume slider's sample plays at most this often while it is dragged (Godot VolumePreview.INTERVAL).
const PREVIEW_INTERVAL: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bus {
    Music,
    Sfx,
    Ui,
}

#[derive(Debug, Clone, Copy)]
struct Levels {
    master: f32,
    music: f32,
    sfx: f32,
    ui: f32,
}

impl Levels {
    fn of(&self, bus: Bus) -> f32 {
        self.master
            * match bus {
                Bus::Music => self.music,
                Bus::Sfx => self.sfx,
                Bus::Ui => self.ui,
            }
    }
}

struct Track {
    sink: Sink,
    gain: f32,
    target: f32,
    /// Gain change per second toward `target`.
    rate: f32,
}

struct State {
    levels: Levels,
    sfx: Vec<(Sink, Instant)>,
    ui: Vec<(Sink, Instant)>,
    tracks: Vec<Track>,
    wanted: Option<String>,
    last_tick: Instant,
    last_preview: Option<Instant>,
    cache: HashMap<String, Option<Arc<[u8]>>>,
    found: HashMap<String, bool>,
}

struct Device {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

pub struct Audio {
    root: Option<PathBuf>,
    device: Option<Device>,
    state: RefCell<State>,
}

impl Audio {
    /// The game's audio folder (`root`) on the default output device; silent when there is none.
    pub fn new(root: Option<PathBuf>) -> Self {
        let device = OutputStream::try_default().ok().map(|(stream, handle)| Device { _stream: stream, handle });
        Self { root, device, state: RefCell::new(State::new()) }
    }

    /// The files without a device: nothing plays, the choices and checks still work (tests, tools).
    pub fn offline(root: Option<PathBuf>) -> Self {
        Self { root, device: None, state: RefCell::new(State::new()) }
    }

    /// Whether `rel` reads and decodes (a format the engine plays: WAV or Ogg Vorbis).
    pub fn check(&self, rel: &str) -> Result<(), String> {
        let bytes = self.bytes(rel).ok_or_else(|| format!("{rel}: not found"))?;
        Decoder::new(Cursor::new(bytes)).map(|_| ()).map_err(|e| format!("{rel}: {e}"))
    }

    /// Whether the sound file `rel` exists (works without a device, so the game's choices are the same everywhere);
    /// looked up once per path, since screens ask every frame.
    pub fn has(&self, rel: &str) -> bool {
        if let Some(&hit) = self.state.borrow().found.get(rel) {
            return hit;
        }
        let hit = self.root.as_ref().is_some_and(|r| r.join(rel).is_file());
        self.state.borrow_mut().found.insert(rel.to_string(), hit);
        hit
    }

    /// The four levels and mute from the settings (the engine's keys: master, music, effects, interface 0-100, mute).
    pub fn set_levels(&self, settings: &Value) {
        let level = |k: &str| (settings[k].as_f64().unwrap_or(100.0) / 100.0).clamp(0.0, 1.0) as f32;
        let mute = settings["mute"].as_bool().unwrap_or(false);
        let mut st = self.state.borrow_mut();
        st.levels = Levels { master: if mute { 0.0 } else { level("master") }, music: level("music"), sfx: level("effects"), ui: level("interface") };
        let music = st.levels.of(Bus::Music) * MUSIC_GAIN;
        for t in &st.tracks {
            t.sink.set_volume(t.gain * music);
        }
    }

    /// Plays `rel` once on `bus` at that bus's level; a missing file is silent.
    pub fn play(&self, bus: Bus, rel: &str) {
        let level = self.state.borrow().levels.of(bus);
        self.play_at(bus, rel, level);
    }

    /// A volume slider moved: its sample plays on `bus` at the levels set now (the screen sets the draft levels
    /// first), at most every PREVIEW_INTERVAL. Like Godot's bus, the music sample plays without the music track's -6 dB.
    pub fn preview(&self, bus: Bus, rel: &str) {
        let level = {
            let mut st = self.state.borrow_mut();
            let now = Instant::now();
            if st.last_preview.is_some_and(|t| now.duration_since(t) < PREVIEW_INTERVAL) {
                return;
            }
            st.last_preview = Some(now);
            st.levels.of(bus)
        };
        self.play_at(bus, rel, level);
    }

    fn play_at(&self, bus: Bus, rel: &str, volume: f32) {
        let Some(device) = &self.device else { return };
        if volume <= 0.0 {
            return;
        }
        let Some(bytes) = self.bytes(rel) else { return };
        let Ok(source) = Decoder::new(Cursor::new(bytes)) else { return };
        let Ok(sink) = Sink::try_new(&device.handle) else { return };
        sink.set_volume(volume);
        sink.append(source);
        let mut st = self.state.borrow_mut();
        let (voices, max) = match bus {
            Bus::Sfx => (&mut st.sfx, SFX_VOICES),
            _ => (&mut st.ui, UI_VOICES),
        };
        voices.retain(|(s, _)| !s.empty());
        if voices.len() >= max {
            voices.remove(0);   // dropping a sink stops it: the oldest makes room
        }
        voices.push((sink, Instant::now()));
    }

    /// The music track wanted now (`None`: silence); a new track fades in over the old one, which fades out.
    pub fn music(&self, rel: Option<&str>) {
        let mut st = self.state.borrow_mut();
        if st.wanted.as_deref() == rel {
            return;
        }
        let was_silent = st.wanted.is_none();
        st.wanted = rel.map(String::from);
        for t in st.tracks.iter_mut() {
            t.target = 0.0;
            t.rate = 1.0 / CROSSFADE_SEC;
        }
        let Some(rel) = rel else { return };
        drop(st);
        let (Some(device), Some(bytes)) = (&self.device, self.bytes(rel)) else { return };
        let (Ok(source), Ok(sink)) = (Decoder::new(Cursor::new(bytes)), Sink::try_new(&device.handle)) else { return };
        sink.set_volume(0.0);
        sink.append(source.repeat_infinite());
        let mut st = self.state.borrow_mut();
        st.tracks.push(Track { sink, gain: 0.0, target: 1.0, rate: 1.0 / if was_silent { FIRST_FADE_SEC } else { CROSSFADE_SEC } });
    }

    /// The track wanted now (for the agent link and tests; also without a device).
    pub fn wanted_music(&self) -> Option<String> {
        self.state.borrow().wanted.clone()
    }

    /// Advances the fades; tracks faded to silence stop. The shell calls it every frame.
    pub fn tick(&self) {
        let mut st = self.state.borrow_mut();
        let now = Instant::now();
        let dt = now.duration_since(st.last_tick).as_secs_f32().min(0.25);
        st.last_tick = now;
        let level = st.levels.of(Bus::Music) * MUSIC_GAIN;
        for t in st.tracks.iter_mut() {
            let step = t.rate * dt;
            t.gain = if t.gain < t.target { (t.gain + step).min(t.target) } else { (t.gain - step).max(t.target) };
            t.sink.set_volume(t.gain * level);
        }
        st.tracks.retain(|t| !(t.target == 0.0 && t.gain <= 0.0));
    }

    /// The bytes of `rel`, read once (a missing or unreadable file is remembered as None).
    fn bytes(&self, rel: &str) -> Option<Arc<[u8]>> {
        if let Some(hit) = self.state.borrow().cache.get(rel) {
            return hit.clone();
        }
        let read = self.root.as_ref().and_then(|r| std::fs::read(r.join(rel)).ok()).map(Arc::from);
        self.state.borrow_mut().cache.insert(rel.to_string(), read.clone());
        read
    }
}

impl State {
    fn new() -> Self {
        Self {
            levels: Levels { master: 1.0, music: 0.8, sfx: 0.8, ui: 0.8 },
            sfx: Vec::new(),
            ui: Vec::new(),
            tracks: Vec::new(),
            wanted: None,
            last_tick: Instant::now(),
            last_preview: None,
            cache: HashMap::new(),
            found: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn silent_audio_tracks_the_wanted_music_and_ignores_sounds() {
        let a = Audio::offline(None);
        assert_eq!(a.wanted_music(), None);
        a.music(Some("music/town.ogg"));
        a.music(Some("music/town.ogg"));
        assert_eq!(a.wanted_music().as_deref(), Some("music/town.ogg"));
        a.play(Bus::Sfx, "sfx/hit.wav");
        a.preview(Bus::Ui, "sfx/ui_click.wav");
        a.tick();
        a.music(None);
        assert_eq!(a.wanted_music(), None);
        assert!(!a.has("sfx/hit.wav"));
    }

    #[test]
    fn levels_multiply_by_master_and_mute_silences() {
        let a = Audio::offline(None);
        a.set_levels(&json!({"master": 50, "music": 80, "effects": 100, "interface": 20, "mute": false}));
        let l = a.state.borrow().levels;
        assert_eq!((l.of(Bus::Music), l.of(Bus::Sfx), l.of(Bus::Ui)), (0.4, 0.5, 0.1));
        a.set_levels(&json!({"master": 50, "music": 80, "effects": 100, "interface": 20, "mute": true}));
        assert_eq!(a.state.borrow().levels.of(Bus::Sfx), 0.0);
    }

    #[test]
    fn has_finds_files_under_the_root() {
        let dir = std::env::temp_dir().join(format!("aegis_audio_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sfx")).unwrap();
        std::fs::write(dir.join("sfx/hit.wav"), b"x").unwrap();
        let a = Audio::offline(Some(dir.clone()));
        assert!(a.has("sfx/hit.wav"));
        assert!(!a.has("sfx/none.wav"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
