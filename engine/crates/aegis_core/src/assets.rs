//! Layered assets (ASSETORG2, docs/specs/ASSETORG2.md sections 2-4): a relative path such as `units/warrior/base/sheet.png`
//! resolves in the game's `assets/`, then its kit's (`kits/<kit>/assets/`, named by `game.json` `kit`), then the
//! engine's (`engine/assets/`); the first layer holding the file wins, so a game overrides a kit or engine file by
//! putting its own at the same path. Bodies are sheets: one row per clip of `clips.json`, frames left to right.
//! Playback (SPRITE1-L): `Clips::frame_at` picks a clip's frame by time (looping or holding the last frame, a missing
//! clip falling back to `idle`), `ClipPlayer` keeps a base loop with one shot at a time over it.

use crate::json::{self, Dict};
use indexmap::IndexMap;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Clip frame sizes, frame counts and durations a sheet may use.
pub const FRAME_MIN: u32 = 8;
pub const FRAME_MAX: u32 = 256;
pub const FRAMES_MAX: u32 = 64;
pub const MS_MIN: u32 = 16;
pub const MS_MAX: u32 = 2000;
/// Most named colours a body's palette or a palette file holds.
pub const PALETTE_SLOTS_MAX: usize = 16;

/// The asset roots of one game, in lookup order (game, kit, engine), and what was missing when they were found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layers {
    pub roots: Vec<PathBuf>,
    /// Start notices: a kit named by game.json without its folder, no engine assets above the game.
    pub notices: Vec<String>,
}

impl Layers {
    /// Exactly these roots, in this order (tests, tools).
    pub fn of(roots: Vec<PathBuf>) -> Self {
        Self { roots, notices: Vec::new() }
    }

    /// The layers of the game in `game_dir` (holds game.json and assets/): its own assets, its kit's (game.json `kit`),
    /// and the engine's, the workspace being the first ancestor of the game holding `engine/assets/`.
    pub fn for_game(game_dir: &Path) -> Self {
        let mut out = Self::default();
        out.roots.push(game_dir.join("assets"));
        let kit = std::fs::read_to_string(game_dir.join("game.json")).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).map(|v| json::s(json::dict(&v), "kit")).unwrap_or_default();
        let workspace = game_dir.ancestors().skip(1).find(|d| d.join("engine/assets").is_dir()).map(Path::to_path_buf);
        match &workspace {
            Some(ws) => {
                if !kit.is_empty() {
                    let k = ws.join("kits").join(&kit).join("assets");
                    if k.is_dir() {
                        out.roots.push(k);
                    } else {
                        out.notices.push(format!("Kit '{kit}' (game.json) has no folder kits/{kit}/assets: its assets are skipped."));
                    }
                }
                out.roots.push(ws.join("engine/assets"));
            }
            None => out.notices.push("No engine/assets above the game: only the game's own assets are used.".into()),
        }
        out
    }

    /// The first layer's file at `rel`.
    pub fn find(&self, rel: &str) -> Option<PathBuf> {
        self.roots.iter().map(|r| r.join(rel)).find(|p| p.is_file())
    }

    pub fn has(&self, rel: &str) -> bool {
        self.find(rel).is_some()
    }

    /// The tile variants of `dir` (holding v0.png, v1.png, ...): in the first layer holding `dir/v0.png`, the run of
    /// v0..v(n-1) files; 0 when no layer has v0.
    pub fn variants(&self, dir: &str) -> usize {
        let Some(root) = self.roots.iter().find(|r| r.join(dir).join("v0.png").is_file()) else { return 0 };
        (0..).take_while(|n| root.join(dir).join(format!("v{n}.png")).is_file()).count()
    }

    /// The names of the files directly in `dir` across every layer (a name once), sorted.
    pub fn list(&self, dir: &str) -> Vec<String> {
        self.names(dir, |p| p.is_file())
    }

    /// The names of the folders directly in `dir` across every layer (a name once), sorted.
    pub fn dirs(&self, dir: &str) -> Vec<String> {
        self.names(dir, |p| p.is_dir())
    }

    fn names(&self, dir: &str, keep: impl Fn(&Path) -> bool) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for r in &self.roots {
            let Ok(read) = std::fs::read_dir(r.join(dir)) else { continue };
            for e in read.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if keep(&e.path()) && !out.contains(&name) {
                    out.push(name);
                }
            }
        }
        out.sort();
        out
    }

    /// Every body key across the layers: `<class>` for `units/<class>/base/`, `<class>/<variant>` otherwise (a body is
    /// a folder holding clips.json).
    pub fn bodies(&self) -> Vec<String> {
        let mut out = Vec::new();
        for class in self.dirs("units") {
            for variant in self.dirs(&format!("units/{class}")) {
                if self.has(&body_dir(&format!("{class}/{variant}")).join("clips.json").to_string_lossy()) {
                    out.push(if variant == "base" { class.clone() } else { format!("{class}/{variant}") });
                }
            }
        }
        out
    }
}

/// The folder of body `key` (`<class>` = `<class>/base`) relative to a root: `units/<class>/<variant>`.
pub fn body_dir(key: &str) -> PathBuf {
    let (class, variant) = key.split_once('/').unwrap_or((key, "base"));
    PathBuf::from("units").join(class).join(variant)
}

/// The relative path of a body's file (`sheet.png`, `clips.json`, `palettes/<id>.json`), with forward slashes.
pub fn body_file(key: &str, file: &str) -> String {
    format!("{}/{file}", body_dir(key).to_string_lossy().replace('\\', "/"))
}

/// `#rrggbb` as [r, g, b].
pub fn hex_colour(s: &str) -> Option<[u8; 3]> {
    let h = s.strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

fn colours(raw: Option<&Value>, label: &str, problems: &mut Vec<String>) -> IndexMap<String, [u8; 3]> {
    let mut out = IndexMap::new();
    let Some(raw) = raw else { return out };
    let Some(o) = raw.as_object() else {
        problems.push(format!("{label} must map slot names to #rrggbb colours."));
        return out;
    };
    if o.len() > PALETTE_SLOTS_MAX {
        problems.push(format!("{label} holds at most {PALETTE_SLOTS_MAX} slots (has {}).", o.len()));
    }
    for (slot, v) in o {
        match v.as_str().and_then(hex_colour) {
            Some(c) => {
                out.insert(slot.clone(), c);
            }
            None => problems.push(format!("{label} slot '{slot}' must be a #rrggbb colour.")),
        }
    }
    out
}

/// One clip of a sheet: its row, frames left to right, each shown `ms[i]` milliseconds.
#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    pub id: String,
    pub frames: u32,
    pub ms: Vec<u32>,
    pub looped: bool,
}

/// A body's `clips.json`: the frame size, its palette slots and its clips (row order).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Clips {
    pub frame: (u32, u32),
    pub palette: IndexMap<String, [u8; 3]>,
    pub clips: Vec<Clip>,
}

fn is_snake(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') && s.starts_with(|c: char| c.is_ascii_lowercase())
}

impl Clips {
    /// Reads and checks a clips.json (spec 4.1): frame 8..256 each way, clips with unique snake_case ids and `idle`
    /// among them, 1..64 frames each with one duration 16..2000 ms per frame, `loop` a flag (default true), palette
    /// slots `#rrggbb`. Every problem is listed.
    pub fn parse(v: &Value) -> Result<Clips, Vec<String>> {
        let mut problems = Vec::new();
        let Some(o) = v.as_object() else { return Err(vec!["clips.json must be an object {frame, palette, clips}.".into()]) };
        let frame = json::arr(o.get("frame").unwrap_or(&Value::Null)).iter().map(|x| x.as_u64().unwrap_or(0) as u32).collect::<Vec<_>>();
        let frame = match frame.as_slice() {
            [w, h] if (FRAME_MIN..=FRAME_MAX).contains(w) && (FRAME_MIN..=FRAME_MAX).contains(h) => (*w, *h),
            _ => {
                problems.push(format!("frame must be [w, h], each {FRAME_MIN}..{FRAME_MAX}."));
                (0, 0)
            }
        };
        let palette = colours(o.get("palette"), "palette", &mut problems);
        let mut clips: Vec<Clip> = Vec::new();
        let list = json::arr(o.get("clips").unwrap_or(&Value::Null));
        if list.is_empty() {
            problems.push("clips must list at least one clip.".into());
        }
        for c in list {
            let c = json::dict(c);
            let id = json::s(c, "id");
            if !is_snake(&id) {
                problems.push(format!("clip id '{id}' must be snake_case."));
            } else if clips.iter().any(|x| x.id == id) {
                problems.push(format!("clip '{id}' is listed twice."));
            }
            let frames = c.get("frames").and_then(Value::as_u64).unwrap_or(0) as u32;
            if !(1..=FRAMES_MAX).contains(&frames) {
                problems.push(format!("clip '{id}': frames must be 1..{FRAMES_MAX}."));
            }
            let ms: Vec<u32> = json::arr(c.get("ms").unwrap_or(&Value::Null)).iter().map(|x| x.as_u64().unwrap_or(0) as u32).collect();
            if ms.len() != frames as usize || ms.iter().any(|m| !(MS_MIN..=MS_MAX).contains(m)) {
                problems.push(format!("clip '{id}': ms must hold one duration per frame, each {MS_MIN}..{MS_MAX}."));
            }
            let looped = match c.get("loop") {
                None => true,
                Some(Value::Bool(b)) => *b,
                Some(_) => {
                    problems.push(format!("clip '{id}': loop must be true or false."));
                    true
                }
            };
            clips.push(Clip { id, frames, ms, looped });
        }
        if !list.is_empty() && !clips.iter().any(|c| c.id == "idle") {
            problems.push("clips must include 'idle' (the token).".into());
        }
        if problems.is_empty() {
            Ok(Clips { frame, palette, clips })
        } else {
            Err(problems)
        }
    }

    /// The clip `id`'s row.
    pub fn row(&self, id: &str) -> Option<usize> {
        self.clips.iter().position(|c| c.id == id)
    }

    /// The sheet size the clips need: the longest clip's frames wide, one row per clip.
    pub fn needs(&self) -> (u32, u32) {
        (self.clips.iter().map(|c| c.frames).max().unwrap_or(0) * self.frame.0, self.clips.len() as u32 * self.frame.1)
    }

    /// "" or why a `w` x `h` sheet is too small for these clips.
    pub fn sheet_problem(&self, w: u32, h: u32) -> String {
        let (nw, nh) = self.needs();
        if w < nw || h < nh {
            format!("the sheet is {w}x{h}; its clips need at least {nw}x{nh}.")
        } else {
            String::new()
        }
    }

    /// The clip `id`.
    pub fn clip(&self, id: &str) -> Option<&Clip> {
        self.clips.iter().find(|c| c.id == id)
    }

    /// The row played for `id` (SPRITE1-L): the clip itself, else `idle`, else the first clip; None for no clips.
    pub fn resolve(&self, id: &str) -> Option<usize> {
        self.row(id).or_else(|| self.row("idle")).or(if self.clips.is_empty() { None } else { Some(0) })
    }

    /// Which frame shows `ms` after clip `id` started (fallback as `resolve`). A clip that is not `id` or `idle` (a
    /// body without idle) shows its first frame, still.
    pub fn frame_at(&self, id: &str, ms: u64) -> Option<ClipFrame> {
        let row = self.resolve(id)?;
        let c = &self.clips[row];
        let (frame, done) = if c.id == id || c.id == "idle" { c.frame_at(ms) } else { (0, true) };
        Some(ClipFrame { row, clip: c.id.clone(), frame, done })
    }
}

/// One frame of a sheet chosen by time: its row, the clip that row holds (after fallback), the frame, and whether a
/// one-shot clip has reached its end (always false for a looping one).
#[derive(Debug, Clone, PartialEq)]
pub struct ClipFrame {
    pub row: usize,
    pub clip: String,
    pub frame: u32,
    pub done: bool,
}

impl Clip {
    /// The clip's length: the sum of its frames' durations.
    pub fn total_ms(&self) -> u64 {
        self.ms.iter().map(|m| *m as u64).sum()
    }

    /// The frame shown `ms` after the clip started and whether it is over: a looping clip wraps around (never over);
    /// a one-shot clip holds its last frame once its durations have run.
    pub fn frame_at(&self, ms: u64) -> (u32, bool) {
        let total = self.total_ms();
        if total == 0 || self.ms.is_empty() {
            return (0, !self.looped);
        }
        let t = if self.looped {
            ms % total
        } else if ms >= total {
            return (self.ms.len() as u32 - 1, true);
        } else {
            ms
        };
        let mut acc = 0u64;
        for (i, m) in self.ms.iter().enumerate() {
            acc += *m as u64;
            if t < acc {
                return (i as u32, false);
            }
        }
        (self.ms.len() as u32 - 1, !self.looped)
    }
}

/// What a `ClipPlayer` plays over its base loop.
#[derive(Debug, Clone, PartialEq)]
struct Shot {
    clip: String,
    since: u64,
    /// Play for this long (looping the clip) instead of once.
    for_ms: Option<u64>,
    /// Keep the last frame when the clip is over (a death).
    hold: bool,
}

/// Plays a body's clips on a clock the caller keeps (SPRITE1-L; milliseconds, scaled by the caller for animation
/// speed): a base loop (`idle`, a stance's idle) and over it one shot at a time (an attack, a hit, a walk for a set
/// time, a death that holds its last frame). Clips the body lacks fall back as `Clips::frame_at` does.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipPlayer {
    base: String,
    base_since: u64,
    shot: Option<Shot>,
}

impl Default for ClipPlayer {
    fn default() -> Self {
        Self::new("idle")
    }
}

impl ClipPlayer {
    pub fn new(base: &str) -> Self {
        Self { base: base.to_string(), base_since: 0, shot: None }
    }

    /// The base loop.
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Loops `clip` between shots from `now` (unchanged when it already does).
    pub fn set_base(&mut self, clip: &str, now: u64) {
        if self.base != clip {
            self.base = clip.to_string();
            self.base_since = now;
        }
    }

    /// Plays `clip` once from `now`, then the base loop again.
    pub fn play(&mut self, clip: &str, now: u64) {
        self.shot = Some(Shot { clip: clip.to_string(), since: now, for_ms: None, hold: false });
    }

    /// Plays `clip` (looping) for `ms` from `now`, then the base loop again.
    pub fn play_for(&mut self, clip: &str, now: u64, ms: u64) {
        self.shot = Some(Shot { clip: clip.to_string(), since: now, for_ms: Some(ms), hold: false });
    }

    /// Plays `clip` once from `now` and keeps its last frame.
    pub fn play_and_hold(&mut self, clip: &str, now: u64) {
        self.shot = Some(Shot { clip: clip.to_string(), since: now, for_ms: None, hold: true });
    }

    /// Drops the shot: the base loop shows.
    pub fn stop(&mut self) {
        self.shot = None;
    }

    /// The shot playing at `now` ("" when the base loop shows).
    pub fn playing(&self, clips: &Clips, now: u64) -> &str {
        match &self.shot {
            Some(s) if s.hold || !self.shot_over(clips, s, now) => &s.clip,
            _ => "",
        }
    }

    fn shot_end(clips: &Clips, s: &Shot) -> u64 {
        let len = s.for_ms.unwrap_or_else(|| clips.resolve(&s.clip).map_or(0, |r| clips.clips[r].total_ms()));
        s.since.saturating_add(len)
    }

    fn shot_over(&self, clips: &Clips, s: &Shot, now: u64) -> bool {
        now >= Self::shot_end(clips, s)
    }

    /// How long a held shot has shown its last frame at `now` (None: no held shot, or still running).
    pub fn held_for(&self, clips: &Clips, now: u64) -> Option<u64> {
        let s = self.shot.as_ref().filter(|s| s.hold)?;
        let end = Self::shot_end(clips, s);
        (now >= end).then(|| now - end)
    }

    /// The frame to draw at `now`: the shot while it runs (a held one for good), else the base loop.
    pub fn frame(&self, clips: &Clips, now: u64) -> Option<ClipFrame> {
        if let Some(s) = &self.shot {
            let t = now.saturating_sub(s.since);
            if s.hold && self.shot_over(clips, s, now) {
                let mut f = clips.frame_at(&s.clip, u64::MAX)?;
                if let Some(c) = clips.clip(&f.clip).filter(|c| c.looped) {
                    // A held clip that loops (or the idle fallback) stops on its last frame.
                    f.frame = c.frames.saturating_sub(1);
                }
                f.done = true;
                return Some(f);
            }
            if !self.shot_over(clips, s, now) {
                let mut f = clips.frame_at(&s.clip, t)?;
                f.done = false;
                return Some(f);
            }
        }
        clips.frame_at(&self.base, now.saturating_sub(self.base_since))
    }
}

/// A palette file `{"slots": {"main": "#5a3a30"}}`: the new colour per slot.
pub fn parse_palette(v: &Value) -> Result<IndexMap<String, [u8; 3]>, Vec<String>> {
    let mut problems = Vec::new();
    let empty = Dict::new();
    let o = v.as_object().unwrap_or(&empty);
    if !o.contains_key("slots") {
        return Err(vec!["a palette file must be {\"slots\": {slot: #rrggbb}}.".into()]);
    }
    let slots = colours(o.get("slots"), "slots", &mut problems);
    if problems.is_empty() {
        Ok(slots)
    } else {
        Err(problems)
    }
}

/// Reads a JSON file of the layers at `rel`.
pub fn read_json(layers: &Layers, rel: &str) -> Option<Value> {
    let text = std::fs::read_to_string(layers.find(rel)?).ok()?;
    serde_json::from_str(&text).ok()
}

/// A kit's manifest, `kits/<id>/kit.json`: {id, title, version, engine}.
pub const KIT_KEYS: [&str; 4] = ["id", "title", "version", "engine"];

/// Why `v` is not a valid kit.json for folder `folder` (empty: valid): id = the folder name (snake_case, 2-24), a
/// title, a semver version, an engine range.
pub fn kit_problems(v: &Dict, folder: &str) -> Vec<String> {
    let mut out = Vec::new();
    for k in v.keys().filter(|k| !KIT_KEYS.contains(&k.as_str())) {
        out.push(format!("kit.json has no key '{k}' (keys: {}).", KIT_KEYS.join(", ")));
    }
    let id = json::s(v, "id");
    if !is_snake(&id) || !(2..=24).contains(&id.len()) {
        out.push(format!("kit.json: id '{id}' must be snake_case, 2-24 characters."));
    } else if id != folder {
        out.push(format!("kit.json: id '{id}' must equal its folder name '{folder}'."));
    }
    if json::s(v, "title").trim().is_empty() {
        out.push("kit.json: title is required.".into());
    }
    let version = json::s(v, "version");
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit())) {
        out.push(format!("kit.json: version '{version}' must be semver (1.2.3)."));
    }
    if json::s(v, "engine").trim().is_empty() {
        out.push("kit.json: engine (a version range, e.g. >=0.1) is required.".into());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tree(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("aegis_assets_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn touch(p: &Path) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"x").unwrap();
    }

    #[test]
    fn the_game_overrides_the_kit_which_overrides_the_engine() {
        let ws = tree("layers");
        let game = ws.join("games/demo");
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("game.json"), r#"{"id": "demo", "kit": "rpg"}"#).unwrap();
        for p in ["engine/assets/audio/sfx/click.wav", "kits/rpg/assets/audio/sfx/click.wav", "kits/rpg/assets/statuses/burn.png", "games/demo/assets/statuses/burn.png", "engine/assets/fonts/f.ttf"] {
            touch(&ws.join(p));
        }
        for n in 0..3 {
            touch(&ws.join(format!("engine/assets/tilesets/common/dirt/v{n}.png")));
        }
        touch(&ws.join("engine/assets/tilesets/common/dirt/v4.png"));
        let l = Layers::for_game(&game);
        assert!(l.notices.is_empty(), "{:?}", l.notices);
        assert_eq!(l.roots.len(), 3);
        assert_eq!(l.find("statuses/burn.png").unwrap(), game.join("assets/statuses/burn.png"), "the game first");
        assert_eq!(l.find("audio/sfx/click.wav").unwrap(), ws.join("kits/rpg/assets/audio/sfx/click.wav"), "then the kit");
        assert_eq!(l.find("fonts/f.ttf").unwrap(), ws.join("engine/assets/fonts/f.ttf"), "then the engine");
        assert_eq!(l.variants("tilesets/common/dirt"), 3, "a run of v0..v2 (v4 is past a gap)");
        assert_eq!(l.variants("tilesets/common/lava"), 0);
        assert_eq!(l.list("statuses"), ["burn.png"], "a name once");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn a_missing_kit_is_a_notice_and_bodies_are_listed() {
        let ws = tree("kitless");
        let game = ws.join("games/demo");
        std::fs::create_dir_all(ws.join("engine/assets")).unwrap();
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("game.json"), r#"{"id": "demo", "kit": "nope"}"#).unwrap();
        touch(&game.join("assets/units/warrior/base/clips.json"));
        touch(&game.join("assets/units/warrior/goblin/clips.json"));
        touch(&game.join("assets/units/warrior/empty/sheet.png"));
        let l = Layers::for_game(&game);
        assert_eq!(l.roots.len(), 2);
        assert!(l.notices[0].contains("Kit 'nope'"), "{:?}", l.notices);
        assert_eq!(l.bodies(), ["warrior", "warrior/goblin"], "a folder without clips.json is no body");
        assert_eq!(body_file("warrior", "sheet.png"), "units/warrior/base/sheet.png");
        assert_eq!(body_file("brawler/goblin", "clips.json"), "units/brawler/goblin/clips.json");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn clips_are_checked_and_measure_their_sheet() {
        let good = json!({"frame": [48, 48], "palette": {"main": "#a83028", "trim": "#E0B43C"},
            "clips": [{"id": "idle", "frames": 2, "ms": [120, 120]}, {"id": "attack", "frames": 4, "ms": [80, 80, 80, 120], "loop": false}]});
        let c = Clips::parse(&good).unwrap();
        assert_eq!((c.frame, c.needs(), c.row("attack")), ((48, 48), (192, 96), Some(1)));
        assert_eq!(c.palette["trim"], [0xe0, 0xb4, 0x3c]);
        assert!(c.clips[0].looped && !c.clips[1].looped);
        assert!(c.sheet_problem(192, 96).is_empty());
        assert_eq!(c.sheet_problem(96, 96), "the sheet is 96x96; its clips need at least 192x96.");
        let bad = Clips::parse(&json!({"frame": [4, 300], "palette": {"main": "red"}, "clips": [{"id": "Walk", "frames": 0, "ms": [], "loop": 1}, {"id": "Walk", "frames": 1, "ms": [5]}]})).unwrap_err();
        for m in [
            "frame must be [w, h], each 8..256.",
            "palette slot 'main' must be a #rrggbb colour.",
            "clip id 'Walk' must be snake_case.",
            "clip 'Walk': frames must be 1..64.",
            "clip 'Walk': loop must be true or false.",
            "clip 'Walk': ms must hold one duration per frame, each 16..2000.",
            "clips must include 'idle' (the token).",
        ] {
            assert!(bad.contains(&m.to_string()), "{m} in {bad:?}");
        }
        assert_eq!(parse_palette(&json!({"slots": {"main": "#5a3a30"}})).unwrap()["main"], [0x5a, 0x3a, 0x30]);
        assert!(parse_palette(&json!({"main": "#5a3a30"})).is_err());
    }

    #[test]
    fn a_kit_manifest_names_its_folder() {
        let ok = json!({"id": "rpg", "title": "RPG kit", "version": "0.1.0", "engine": ">=0.1"});
        assert!(kit_problems(ok.as_object().unwrap(), "rpg").is_empty());
        let bad = json!({"id": "rpg", "title": "", "version": "1.0", "engine": "", "x": 1});
        let p = kit_problems(bad.as_object().unwrap(), "other");
        assert_eq!(p.len(), 5, "{p:?}");
    }
}
