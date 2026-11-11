//! Sound and music (Godot AudioDirector, MusicDirector's player, VolumePreview's throttle), game-agnostic: sound files
//! under the game's audio folder, addressed by relative path ("sfx/hit.wav", "music/town.ogg"); four levels
//! (master, music, effects, interface, 0-100, from the settings) and mute; a voice pool per bus (the oldest restarts
//! when all are busy); one music track wanted at a time, crossfaded and looped. The game decides which sound and which
//! track (screens name their music, `Screen::music`). Without an audio device (or headless) everything is silent. On
//! the web (WEBREL1-E2) the device opens at the player's first gesture (`unlock`): a page may not start audio before
//! one, and sounds asked for until then are dropped, not queued.
//!
//! MUSICVAR1: a track `music/<key>.ogg` may ship numbered variants beside it (`<key>_2.ogg`, `<key>_3.ogg`, ...;
//! `music_variants`); the game asks `music_variant` for one per request, picked from the seeded stream
//! `VARIANT_STREAM` (its own generator, so no gameplay roll moves) and never the previous request's pick. An Ogg Vorbis
//! track tagged `LOOPSTART` (and `LOOPLENGTH`, in sample frames) plays its intro once and then loops that span
//! (`loop_points`, `IntroLoop`); an untagged one loops whole as before.

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use aegis_core::clock::{Duration, Instant};
use aegis_core::ghash::{hash, GVar};
use aegis_core::rng::GodotRng;
use aegis_core::vfs;

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
    /// A track's variant pool (`music_variants`), listed once until the next mount.
    pools: HashMap<String, Vec<String>>,
    /// The last pick per track (`music_variant`), kept across requests so the next one differs.
    picks: HashMap<String, Pick>,
    /// The read seam's mount count the caches were filled under (`vfs::generation`).
    mounted: u64,
}

/// One music request's variant (`Audio::music_variant`).
struct Pick {
    /// The game's request id; None: the request lasts while its pick is the wanted music.
    request: Option<u64>,
    seed: i64,
    index: usize,
    rel: String,
    /// The wanted music when it was picked: while it is still wanted and the pick has not played, the pick is still
    /// this request's (asked again before the shell applied it).
    before: Option<String>,
    /// The pick became the wanted music (`Audio::music`): once something else is wanted, its request is over.
    played: bool,
}

struct Device {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

pub struct Audio {
    /// The audio folders in lookup order (ASSETORG2: the game's, its kit's, the engine's `assets/audio`).
    roots: Vec<PathBuf>,
    device: Option<Device>,
    /// The device opens at the first gesture (`unlock`; the web): until then nothing plays.
    waiting: bool,
    state: RefCell<State>,
}

/// The default output device; None when there is none.
fn open_device() -> Option<Device> {
    OutputStream::try_default().ok().map(|(stream, handle)| Device { _stream: stream, handle })
}

impl Audio {
    /// One audio folder (`root`) on the default output device; silent when there is none.
    pub fn new(root: Option<PathBuf>) -> Self {
        Self::layered(root.into_iter().collect())
    }

    /// The audio folders `roots` (the first holding a file wins) on the default output device.
    /// On the web the device waits for the first gesture (`unlock`).
    pub fn layered(roots: Vec<PathBuf>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let (device, waiting) = (open_device(), false);
        #[cfg(target_arch = "wasm32")]
        let (device, waiting) = (None, true);
        Self { roots, device, waiting, state: RefCell::new(State::new()) }
    }

    /// The player pressed a key or a button (the shell calls it on every press). On the web the first press opens the
    /// output device (a page may start audio only from a gesture) and the music wanted now starts, fading in as at the
    /// start; the sounds asked for before it were dropped. A no-op after that, offline and on the desktop.
    pub fn unlock(&mut self) {
        if !self.waiting {
            return;
        }
        self.waiting = false;
        self.device = open_device();
        let wanted = self.state.borrow_mut().wanted.take();
        self.music(wanted.as_deref());
    }

    /// The audio folders of asset `layers` (each root's `audio/`).
    pub fn of_layers(layers: &aegis_core::assets::Layers) -> Self {
        Self::layered(layers.roots.iter().map(|r| r.join("audio")).collect())
    }

    /// The files without a device: nothing plays, the choices and checks still work (tests, tools).
    pub fn offline(root: Option<PathBuf>) -> Self {
        Self::offline_layered(root.into_iter().collect())
    }

    /// Several audio folders without a device (tests, tools over the asset layers).
    pub fn offline_layered(roots: Vec<PathBuf>) -> Self {
        Self { roots, device: None, waiting: false, state: RefCell::new(State::new()) }
    }

    /// The audio folders of asset `layers` without a device (a run with no window: no sound device is opened or needed).
    pub fn offline_layers(layers: &aegis_core::assets::Layers) -> Self {
        Self::offline_layered(layers.roots.iter().map(|r| r.join("audio")).collect())
    }

    /// The first folder's file at `rel`.
    fn find(&self, rel: &str) -> Option<PathBuf> {
        self.roots.iter().map(|r| r.join(rel)).find(|p| vfs::is_file(p))
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
        let hit = self.find(rel).is_some();
        self.state.borrow_mut().found.insert(rel.to_string(), hit);
        hit
    }

    /// The four levels and mute from the settings (the engine's keys: master, music, effects, interface 0-100, mute).
    /// The music tracks take them at the next `tick`: the shell sets the saved levels and a screen may set draft ones
    /// in the same frame, and setting the sinks on each call made the music flip between both levels every frame.
    pub fn set_levels(&self, settings: &Value) {
        let level = |k: &str| (settings[k].as_f64().unwrap_or(100.0) / 100.0).clamp(0.0, 1.0) as f32;
        let mute = settings["mute"].as_bool().unwrap_or(false);
        self.state.borrow_mut().levels = Levels { master: if mute { 0.0 } else { level("master") }, music: level("music"), sfx: level("effects"), ui: level("interface") };
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

    /// An output device is open: sounds play (false offline, without a device, and on the web before `unlock`).
    pub fn live(&self) -> bool {
        self.device.is_some()
    }

    /// The track `rel` and its numbered variants beside it (MUSICVAR1): `music/x.ogg` first, then every
    /// `music/x_<n>.ogg` (n of 2 and up, no leading zero) in the folder that holds `x.ogg`, by number; a missing number is
    /// skipped, not the end of the pool. Empty when `rel` has no file. Listed through the read seam (`vfs::list_dir`,
    /// so the web bundle's files count), once per track until the next mount.
    pub fn music_variants(&self, rel: &str) -> Vec<String> {
        if let Some(pool) = self.state.borrow().pools.get(rel) {
            return pool.clone();
        }
        let pool = self.list_variants(rel);
        let mut st = self.state.borrow_mut();
        for v in &pool {
            st.found.insert(v.clone(), true);
        }
        st.pools.insert(rel.to_string(), pool.clone());
        pool
    }

    fn list_variants(&self, rel: &str) -> Vec<String> {
        let Some(base) = self.find(rel) else { return Vec::new() };
        let (Some(dir), Some(stem), Some(ext)) = (base.parent(), base.file_stem().and_then(|s| s.to_str()), base.extension().and_then(|e| e.to_str())) else {
            return vec![rel.to_string()];
        };
        let (prefix, suffix) = (format!("{stem}_"), format!(".{ext}"));
        let mut numbered: Vec<(u32, String)> = vfs::list_dir(dir)
            .unwrap_or_default()
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(String::from))
            .filter_map(|name| {
                let n = name.strip_prefix(&prefix)?.strip_suffix(&suffix)?;
                let whole = !n.is_empty() && !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit());
                let n = n.parse::<u32>().ok().filter(|&n| whole && n >= 2)?;
                Some((n, name))
            })
            .collect();
        numbered.sort();
        numbered.dedup_by_key(|(n, _)| *n);
        let folder = rel.rsplit_once('/').map(|(d, _)| format!("{d}/")).unwrap_or_default();
        std::iter::once(rel.to_string()).chain(numbered.into_iter().map(|(_, name)| format!("{folder}{name}"))).collect()
    }

    /// The track to play for one music request of `rel` (MUSICVAR1): `rel` itself when it has no variants, else one of
    /// `music_variants(rel)`. The same request keeps its pick; a new one picks with `pick_variant` from `seed` (the
    /// game's session seed, say), never the previous request's pick. `request`: the game's id of the request (a session);
    /// None: a request lasts while its pick is the wanted music (`music`; before the pick first plays, while the music
    /// wanted at the pick is still wanted), so the next one starts once another track (or silence) is wanted after the
    /// pick played, or the seed changed. Works without a device.
    pub fn music_variant(&self, rel: &str, request: Option<u64>, seed: i64) -> String {
        if !self.state.borrow().pools.contains_key(rel) {
            self.music_variants(rel);
        }
        let mut guard = self.state.borrow_mut();
        let st = &mut *guard;
        let Some(pool) = st.pools.get(rel).filter(|p| p.len() >= 2) else { return rel.to_string() };
        let last = st.picks.get(rel).filter(|p| pool.get(p.index) == Some(&p.rel));
        if let Some(p) = last {
            let same = match request {
                Some(id) => p.request == Some(id),
                None => {
                    let playing = st.wanted.as_deref() == Some(p.rel.as_str());
                    p.request.is_none() && p.seed == seed && (playing || (!p.played && st.wanted == p.before))
                }
            };
            if same {
                return p.rel.clone();
            }
        }
        let index = pick_variant(pool.len(), seed, last.map(|p| p.index));
        let pick = pool[index].clone();
        let before = st.wanted.clone();
        st.picks.insert(rel.to_string(), Pick { request, seed, index, rel: pick.clone(), before, played: false });
        pick
    }

    /// The music track wanted now (`None`: silence); a new track fades in over the old one, which fades out. A track
    /// tagged with loop points (`loop_points`) plays its intro once and then loops the tagged span; others loop whole.
    pub fn music(&self, rel: Option<&str>) {
        let mut st = self.state.borrow_mut();
        if st.wanted.as_deref() == rel {
            return;
        }
        let was_silent = st.wanted.is_none();
        st.wanted = rel.map(String::from);
        for p in st.picks.values_mut() {
            p.played |= rel == Some(p.rel.as_str());
        }
        for t in st.tracks.iter_mut() {
            t.target = 0.0;
            t.rate = 1.0 / CROSSFADE_SEC;
        }
        let Some(rel) = rel else { return };
        drop(st);
        let (Some(device), Some(bytes)) = (&self.device, self.bytes(rel)) else { return };
        let points = loop_points(&bytes);
        let (Ok(source), Ok(sink)) = (Decoder::new(Cursor::new(bytes)), Sink::try_new(&device.handle)) else { return };
        sink.set_volume(0.0);
        sink.append(looped(source, points));
        let mut st = self.state.borrow_mut();
        st.tracks.push(Track { sink, gain: 0.0, target: 1.0, rate: 1.0 / if was_silent { FIRST_FADE_SEC } else { CROSSFADE_SEC } });
    }

    /// The track wanted now (for the agent link and tests; also without a device).
    pub fn wanted_music(&self) -> Option<String> {
        self.state.borrow().wanted.clone()
    }

    /// A music fade is running: `tick` still has work to do.
    pub fn fading(&self) -> bool {
        self.state.borrow().tracks.iter().any(|t| t.gain != t.target)
    }

    /// Advances the fades; tracks faded to silence stop. The shell calls it every frame.
    pub fn tick(&self) {
        self.remounted();
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

    /// A bundle part was mounted since the caches were filled (WEBREL1-G1: the web build's audio comes as a later part,
    /// while the game already runs): the files remembered as missing are looked up again, and the wanted music, when
    /// it has no track yet (its file was missing), starts now.
    fn remounted(&self) {
        let now = vfs::generation();
        let wanted = {
            let mut st = self.state.borrow_mut();
            if st.mounted == now {
                return;
            }
            st.mounted = now;
            st.cache.retain(|_, read| read.is_some());
            st.found.retain(|_, hit| *hit);
            st.pools.clear();   // a new part may bring variants
            if st.tracks.iter().any(|t| t.target > 0.0) {
                return;
            }
            st.wanted.take()
        };
        self.music(wanted.as_deref());
    }

    /// The bytes of `rel`, read once (a missing or unreadable file is remembered as None, until the next mount).
    fn bytes(&self, rel: &str) -> Option<Arc<[u8]>> {
        if let Some(hit) = self.state.borrow().cache.get(rel) {
            return hit.clone();
        }
        let read = self.find(rel).and_then(|p| vfs::read(p).ok()).map(Arc::from);
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
            pools: HashMap::new(),
            picks: HashMap::new(),
            mounted: vfs::generation(),
        }
    }
}

/// The name of the music variant stream (`pick_variant`): its generator is seeded from the request's seed and this
/// name, so a pick never draws from (or moves) a gameplay stream.
pub const VARIANT_STREAM: &str = "music_variant";

/// MUSICVAR1: the variant (0-based, of `count`) a request seeded `seed` plays: a fresh `GodotRng` seeded with
/// `hash([seed, VARIANT_STREAM])`, one draw; with `last` (the previous request's pick) it draws among the others, so a
/// pool of two or more never plays the same variant twice in a row. 0 when `count` is below 2.
pub fn pick_variant(count: usize, seed: i64, last: Option<usize>) -> usize {
    if count < 2 {
        return 0;
    }
    let mut rng = GodotRng::new(hash(&GVar::Arr(vec![GVar::Int(seed), GVar::Str(VARIANT_STREAM.into())])) as u64);
    let top = i32::try_from(count).unwrap_or(i32::MAX) - 1;
    match last.filter(|&l| l < count) {
        Some(l) => {
            let r = rng.randi_range(0, top - 1) as usize;
            if r >= l { r + 1 } else { r }
        }
        None => rng.randi_range(0, top) as usize,
    }
}

/// A music file's loop span in sample frames (MUSICVAR1): the intro is `0..start`, then `start..start + length`
/// repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopPoints {
    pub start: u64,
    pub length: u64,
}

/// The loop span an Ogg Vorbis file's comment header names (`LOOPSTART`, `LOOPLENGTH`, sample frames, keys in any
/// case), checked against the file's length (its last page's granule position, itself refused when the bytes could not
/// hold that many frames): no `LOOPSTART`, a start at or past the end, or a span that is the whole file is None (loop
/// the whole file). `LOOPLENGTH` absent or 0 runs to the end; a
/// length past the end stops there. Read from the bytes directly (rodio's decoder does not expose the comments);
/// anything that is not Ogg Vorbis (WAV) is None.
pub fn loop_points(bytes: &[u8]) -> Option<LoopPoints> {
    let comments = vorbis_comments(bytes)?;
    let tag = |key: &str| {
        comments.iter().find_map(|c| {
            let (k, v) = c.split_once('=')?;
            if k.eq_ignore_ascii_case(key) { v.trim().parse::<u64>().ok() } else { None }
        })
    };
    let start = tag("LOOPSTART")?;
    // a Vorbis packet (at least a byte) decodes to at most MAX_FRAMES_PER_PACKET frames: a granule beyond that is bogus
    let total = last_granule(bytes).filter(|&t| t <= (bytes.len() as u64).saturating_mul(MAX_FRAMES_PER_PACKET))?;
    if start >= total {
        return None;
    }
    let length = tag("LOOPLENGTH").filter(|&l| l > 0).unwrap_or(total - start).min(total - start);
    (start > 0 || length < total).then_some(LoopPoints { start, length })
}

/// The most sample frames one Vorbis packet decodes to (half the largest block size the format allows, 8192).
const MAX_FRAMES_PER_PACKET: u64 = 4096;

/// The `KEY=value` comments of an Ogg Vorbis file (its second packet), None when the first packets are not Vorbis
/// headers.
fn vorbis_comments(bytes: &[u8]) -> Option<Vec<String>> {
    let packets = ogg_packets(bytes, 2);
    let (id, comment) = (packets.first()?, packets.get(1)?);
    if !id.starts_with(b"\x01vorbis") || !comment.starts_with(b"\x03vorbis") {
        return None;
    }
    // lengths come from the file: every sum is checked (an overflow, on wasm32 too, is no tags)
    let u32_at = |at: usize| comment.get(at..at.checked_add(4)?).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize);
    let field = |at: usize| -> Option<(usize, usize)> {
        let len = u32_at(at)?;
        let from = at.checked_add(4)?;
        Some((from, from.checked_add(len)?))
    };
    let (_, vendor_end) = field(7)?;
    let count = u32_at(vendor_end)?;
    let mut at = vendor_end.checked_add(4)?;
    let mut out = Vec::new();
    for _ in 0..count {
        let (from, to) = field(at)?;
        out.push(String::from_utf8_lossy(comment.get(from..to)?).into_owned());
        at = to;
    }
    Some(out)
}

/// The first `want` packets of the file's first logical stream (pages joined by their lacing; the CRC is not checked).
fn ogg_packets(bytes: &[u8], want: usize) -> Vec<Vec<u8>> {
    let (mut packets, mut part, mut at, mut serial) = (Vec::new(), Vec::new(), 0usize, None);
    while packets.len() < want {
        let Some(head) = bytes.get(at..at + 27).filter(|h| h.starts_with(b"OggS")) else { break };
        let page_serial = u32::from_le_bytes([head[14], head[15], head[16], head[17]]);
        let n = head[26] as usize;
        let Some(table) = bytes.get(at + 27..at + 27 + n) else { break };
        let ours = *serial.get_or_insert(page_serial) == page_serial;
        let mut data = at + 27 + n;
        for &lace in table {
            let Some(seg) = bytes.get(data..data + lace as usize) else { return packets };
            data += lace as usize;
            if ours && packets.len() < want {
                part.extend_from_slice(seg);
                if lace < 255 {
                    packets.push(std::mem::take(&mut part));
                }
            }
        }
        at = data;
    }
    packets
}

/// The granule position of the file's last Ogg page (a Vorbis stream's length in sample frames): the last `OggS` in
/// the final 64 KiB whose page ends exactly at the file's end.
fn last_granule(bytes: &[u8]) -> Option<u64> {
    let from = bytes.len().saturating_sub(65_536 + 27);
    (from..bytes.len().saturating_sub(26)).rev().find_map(|at| {
        let head = bytes.get(at..at + 27).filter(|h| h.starts_with(b"OggS") && h[4] == 0)?;
        let n = head[26] as usize;
        let body: usize = bytes.get(at + 27..at + 27 + n)?.iter().map(|&l| l as usize).sum();
        let granule = u64::from_le_bytes([head[6], head[7], head[8], head[9], head[10], head[11], head[12], head[13]]);
        (at + 27 + n + body == bytes.len() && granule != u64::MAX).then_some(granule)
    })
}

/// The music source of a decoded track: with loop points the intro once and then the span (`IntroLoop`), else the
/// whole file looped (rodio's `repeat_infinite`, as before MUSICVAR1).
pub fn looped<S>(source: S, points: Option<LoopPoints>) -> Box<dyn Source<Item = i16> + Send>
where
    S: Source<Item = i16> + Send + 'static,
{
    match points {
        Some(p) => Box::new(IntroLoop::new(source, p)),
        None => Box::new(source.repeat_infinite()),
    }
}

/// The most samples `IntroLoop::new` reserves for the span up front (8 MiB of i16; a 3 min stereo 44.1 kHz span grows
/// past it once or twice).
const BODY_RESERVE: u64 = 1 << 22;

/// Plays `source` up to the end of the loop span, keeping the span's samples as they pass, then repeats them forever
/// (sample-exact, no seeking, so it works the same on every backend, the web's included; it holds the span in memory,
/// as the whole-file loop holds the whole file). A source that ends before the span's end repeats what it had of the
/// span; one that ends before the span starts ends.
pub struct IntroLoop<S> {
    inner: Option<S>,
    channels: u16,
    rate: u32,
    /// Interleaved samples read so far.
    pos: u64,
    start: u64,
    end: u64,
    body: Vec<i16>,
    /// Repeating: the next sample of `body`.
    at: Option<usize>,
}

impl<S: Source<Item = i16>> IntroLoop<S> {
    pub fn new(source: S, points: LoopPoints) -> Self {
        let channels = source.channels().max(1);
        let rate = source.sample_rate();
        let start = points.start.saturating_mul(channels as u64);
        let end = start.saturating_add(points.length.saturating_mul(channels as u64));
        // the span may name any length (the points are public): reserve at most BODY_RESERVE samples, the rest grows
        // with what the source really decodes
        let reserve = usize::try_from((end - start).min(BODY_RESERVE)).unwrap_or(0);
        Self { inner: Some(source), channels, rate, pos: 0, start, end, body: Vec::with_capacity(reserve), at: None }
    }
}

impl<S: Source<Item = i16>> Iterator for IntroLoop<S> {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        if let Some(i) = self.at {
            self.at = Some((i + 1) % self.body.len());
            return Some(self.body[i]);
        }
        if self.pos < self.end {
            if let Some(s) = self.inner.as_mut().and_then(Iterator::next) {
                if self.pos >= self.start {
                    self.body.push(s);
                }
                self.pos += 1;
                return Some(s);
            }
        }
        // the span's end (or the file's): repeat the span from its start
        self.inner = None;
        let first = *self.body.first()?;
        self.at = Some(1 % self.body.len());
        Some(first)
    }
}

impl<S: Source<Item = i16>> Source for IntroLoop<S> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.rate
    }

    fn total_duration(&self) -> Option<core::time::Duration> {
        None
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
        let dir = aegis_core::scratch::Scratch::vacant("audio");
        std::fs::create_dir_all(dir.join("sfx")).unwrap();
        std::fs::write(dir.join("sfx/hit.wav"), b"x").unwrap();
        let a = Audio::offline(Some(dir.to_path_buf()));
        assert!(a.has("sfx/hit.wav"));
        assert!(!a.has("sfx/none.wav"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn layered_folders_resolve_in_order() {
        let base = aegis_core::scratch::Scratch::vacant("audio_layers");
        let (game, engine) = (base.join("game"), base.join("engine"));
        for (dir, file) in [(&game, "music/town.ogg"), (&engine, "sfx/ui_click.wav"), (&engine, "music/town.ogg")] {
            std::fs::create_dir_all(dir.join(file).parent().unwrap()).unwrap();
            std::fs::write(dir.join(file), dir.to_string_lossy().as_bytes()).unwrap();
        }
        let a = Audio { roots: vec![game.clone(), engine.clone()], device: None, waiting: false, state: RefCell::new(State::new()) };
        assert!(a.has("sfx/ui_click.wav") && a.has("music/town.ogg") && !a.has("sfx/none.wav"));
        assert_eq!(a.bytes("music/town.ogg").as_deref(), Some(game.to_string_lossy().as_bytes()), "the game's file wins");
        let _ = std::fs::remove_dir_all(base);
    }
}
