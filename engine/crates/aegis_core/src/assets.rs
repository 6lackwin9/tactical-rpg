//! Layered assets (ASSETORG2, docs/specs/ASSETORG2.md sections 2-4): a relative path such as `units/warrior/base/sheet.png`
//! resolves in the game's `assets/`, then its kit's (`kits/<kit>/assets/`, named by `game.json` `kit`), then the
//! engine's (`engine/assets/`); the first layer holding the file wins, so a game overrides a kit or engine file by
//! putting its own at the same path. Bodies are sheets: one row per clip of `clips.json`, frames left to right.
//! Playback (SPRITE1-L): `Clips::frame_at` picks a clip's frame by time (looping or holding the last frame, a missing
//! clip falling back to `idle`), `ClipPlayer` keeps a base loop with one shot at a time over it.
//! Direction views (DIR4-E, spec DIR4 section 3): a row with `of` and `view` (`down`, `up`) is
//! the front or back drawing of the side clip `of`; `Facing` is four-way facing on a square grid (`Facing::toward`), and
//! `Clips::resolve_view` / `frame_at_view` / `ClipPlayer::frame_facing` pick the row for a view, falling back to the
//! side clip, then idle in that view, then side idle. Only side rows are ever mirrored. Engine-only (no Godot source).
//! Wind-ups (WINDUP1-E, spec WINDUP1 section 2): a looping row may play an intro once (`loop_from`) and a row may mark
//! its `strike` frame; `ClipPlayer::play_from` starts a shot at a frame and `ClipPlayer::set_base_from` starts the base
//! at a frame, optionally holding it still after the intro.
//! Face sidecars (PORTRAITFX1-E, spec 4.1): `<portrait>.face.json` beside a portrait PNG holds its anchors in image
//! px (eye boxes, glow anchors, line roots, the rim key, idle overlays): `face_file`, `parse_face`, `face_problems`.
//! FB-206 (2b): the overlay primitives have generic names (`lines`, `point_glow`, `particles_rise`, `drift`); a sidecar
//! or cue written with the old names (`veins`, `eye_glow`, `ash`, `smoke`) still loads (`PRIMITIVE_ALIASES`).

use crate::json::{self, Dict};
use crate::vfs;
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
        let kit = vfs::read_to_string(game_dir.join("game.json")).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).map(|v| json::s(json::dict(&v), "kit")).unwrap_or_default();
        let workspace = game_dir.ancestors().skip(1).find(|d| vfs::is_dir(d.join("engine/assets"))).map(Path::to_path_buf);
        match &workspace {
            Some(ws) => {
                if !kit.is_empty() {
                    let k = ws.join("kits").join(&kit).join("assets");
                    if vfs::is_dir(&k) {
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
        self.roots.iter().map(|r| r.join(rel)).find(|p| vfs::is_file(p))
    }

    pub fn has(&self, rel: &str) -> bool {
        self.find(rel).is_some()
    }

    /// The tile variants of `dir` (holding v0.png, v1.png, ...): in the first layer holding `dir/v0.png`, the run of
    /// v0..v(n-1) files; 0 when no layer has v0.
    pub fn variants(&self, dir: &str) -> usize {
        let Some(root) = self.roots.iter().find(|r| vfs::is_file(r.join(dir).join("v0.png"))) else { return 0 };
        (0..).take_while(|n| vfs::is_file(root.join(dir).join(format!("v{n}.png")))).count()
    }

    /// The names of the files directly in `dir` across every layer (a name once), sorted.
    pub fn list(&self, dir: &str) -> Vec<String> {
        self.names(dir, |p| vfs::is_file(p))
    }

    /// The names of the folders directly in `dir` across every layer (a name once), sorted.
    pub fn dirs(&self, dir: &str) -> Vec<String> {
        self.names(dir, |p| vfs::is_dir(p))
    }

    fn names(&self, dir: &str, keep: impl Fn(&Path) -> bool) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for r in &self.roots {
            let Ok(read) = vfs::list_dir(r.join(dir)) else { continue };
            for e in read {
                let name = e.file_name().unwrap_or_default().to_string_lossy().to_string();
                if keep(&e) && !out.contains(&name) {
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

/// Which drawing a clip row is (DIR4 3.1): the side view (the sheets face right; mirrored to face left), the front
/// (`down`, toward the bottom of the screen) or the back (`up`). Front and back rows are never mirrored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum View {
    #[default]
    Side,
    Down,
    Up,
}

impl View {
    /// The clips.json name of a view (`down`, `up`; "" for the side view).
    pub fn name(self) -> &'static str {
        match self {
            View::Side => "",
            View::Down => "down",
            View::Up => "up",
        }
    }

    /// A clips.json `view` value (`down` or `up`).
    pub fn parse(s: &str) -> Option<View> {
        match s {
            "down" => Some(View::Down),
            "up" => Some(View::Up),
            _ => None,
        }
    }
}

/// Four-way facing on a square grid (DIR4 3.2): the view drawn and the last horizontal facing (`left`), kept while
/// the unit faces up or down and used for side rows (mirrored when left) and side fallbacks. Default: side, right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Facing {
    pub view: View,
    pub left: bool,
}

impl Facing {
    /// Side view facing left (`left`) or right.
    pub fn side(left: bool) -> Facing {
        Facing { view: View::Side, left }
    }

    /// The facing toward an offset of `dx`, `dy` tiles (+y down the screen): unchanged for (0, 0); up or down when the
    /// vertical distance is larger (keeping `left`); otherwise (horizontal, or a diagonal tie) the side view, left
    /// when `dx < 0`.
    pub fn toward(self, dx: i32, dy: i32) -> Facing {
        if dx == 0 && dy == 0 {
            self
        } else if dy.abs() > dx.abs() {
            Facing { view: if dy > 0 { View::Down } else { View::Up }, left: self.left }
        } else {
            Facing::side(dx < 0)
        }
    }

    /// Whether `frame` (drawn for this facing) shows mirrored: a side row while facing left. Pass it as the `flip` of
    /// `body_outline` so a highlight rings the frame as drawn.
    pub fn mirrors(self, frame: &ClipFrame) -> bool {
        frame.side && self.left
    }
}

/// One clip of a sheet: its row, frames left to right, each shown `ms[i]` milliseconds. A view clip (DIR4) also names
/// the side clip it draws (`of`) and its `view`; a side clip has `of` "" and `View::Side`. `impact` (FX1-L1, optional)
/// is the frame where an attack lands or its shot leaves; the caller times the hit at `ms_before(impact)`. WINDUP1:
/// `loop_from` (0: none) is the first looped frame of a looping clip (the frames before it play once, the intro);
/// `strike` (0: none) is the first frame after the clip's own wind-up frames, where it starts after a held wind-up.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Clip {
    pub id: String,
    pub frames: u32,
    pub ms: Vec<u32>,
    pub looped: bool,
    pub of: String,
    pub view: View,
    pub impact: Option<u32>,
    pub loop_from: u32,
    pub strike: u32,
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
    /// slots `#rrggbb`. DIR4 3.1: a view row carries `of` and `view` together, `view` is `down` or `up`, `of` names a
    /// side clip of the list and the id is `<of>_<view>`; `idle` is a side clip. FX1-L1: `impact`, when present, is a
    /// frame of the clip (0 to frames - 1). WINDUP1 2: `loop_from` (1 to frames - 1, only with `loop` true) and
    /// `strike` (0 to frames - 1, at most `impact`), both optional. Every problem is listed.
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
        // Per clip: whether it carries both `of` and `view`, and its view when that is valid.
        let mut views: Vec<(bool, Option<View>)> = Vec::new();
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
            // FX1-L1: the frame an attack lands on (optional), a frame of the clip.
            let impact = match c.get("impact") {
                None => None,
                Some(v) => match v.as_u64().filter(|i| *i < frames as u64) {
                    Some(i) => Some(i as u32),
                    None => {
                        problems.push(format!("clip '{id}': impact must be 0 to {}.", frames.saturating_sub(1)));
                        None
                    }
                },
            };
            // WINDUP1 2 (L1, L2): the first looped frame of a looping clip; the frames before it are its intro.
            let loop_from = match c.get("loop_from") {
                None => 0,
                Some(_) if !looped => {
                    problems.push(format!("clip '{id}': loop_from needs loop true."));
                    0
                }
                Some(v) => match v.as_u64().filter(|l| *l >= 1 && *l < frames as u64) {
                    Some(l) => l as u32,
                    None => {
                        problems.push(format!("clip '{id}': loop_from must be 1 to {}.", frames.saturating_sub(1)));
                        0
                    }
                },
            };
            // WINDUP1 2 (S1, S2): where the clip starts after a held wind-up; never after its impact.
            let strike = match c.get("strike") {
                None => 0,
                Some(v) => match v.as_u64().filter(|f| *f < frames as u64) {
                    Some(f) => {
                        if impact.is_some_and(|i| f > i as u64) {
                            problems.push(format!("clip '{id}': strike must not come after impact."));
                        }
                        f as u32
                    }
                    None => {
                        problems.push(format!("clip '{id}': strike must be 0 to {}.", frames.saturating_sub(1)));
                        0
                    }
                },
            };
            // DIR4 3.1: a view row names its side clip (`of`) and its view (`down`, `up`), both or neither.
            let of = c.get("of").map(|v| v.as_str().unwrap_or("").to_string());
            let view = match c.get("view") {
                None => None,
                Some(v) => {
                    let parsed = v.as_str().and_then(View::parse);
                    if parsed.is_none() {
                        problems.push(format!("clip '{id}': view must be down or up."));
                    }
                    Some(parsed)
                }
            };
            if of.is_some() != view.is_some() {
                problems.push(format!("clip '{id}': of and view go together."));
            }
            views.push((of.is_some() && view.is_some(), view.flatten()));
            clips.push(Clip { id, frames, ms, looped, of: of.unwrap_or_default(), view: view.flatten().unwrap_or_default(), impact, loop_from, strike });
        }
        // Rows may come in any order: `of` is checked against the whole list.
        for (c, (both, view)) in clips.iter().zip(&views) {
            if !*both {
                continue;
            }
            if !clips.iter().any(|x| x.id == c.of && x.view == View::Side && x.of.is_empty()) {
                problems.push(format!("clip '{}': of '{}' is not a side clip of this body.", c.id, c.of));
            }
            if let Some(v) = view {
                let want = format!("{}_{}", c.of, v.name());
                if c.id != want {
                    problems.push(format!("clip '{}': a view clip's id must be '{want}'.", c.id));
                }
            }
        }
        if !list.is_empty() && !clips.iter().any(|c| c.id == "idle" && c.view == View::Side && c.of.is_empty()) {
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
        self.frame_at_view(id, View::Side, ms)
    }

    /// WINDUP1 2: the ms before `frame` in the row played for `id` (fallback as `resolve`; 0 for no clips).
    pub fn ms_before(&self, id: &str, frame: u32) -> u64 {
        self.resolve(id).map_or(0, |r| self.clips[r].ms_before(frame))
    }

    /// The view row drawing side clip `of` in `view` (DIR4).
    pub fn view_row(&self, of: &str, view: View) -> Option<usize> {
        self.clips.iter().position(|c| c.view == view && c.view != View::Side && c.of == of)
    }

    /// The row played for side clip `id` in `view` and whether it is a side row (DIR4 3.2): the view row of `id`, the
    /// side row of `id`, the view row of `idle`, the side row of `idle`, the first row; None for no clips.
    /// `View::Side` skips the view steps (`resolve`).
    pub fn resolve_view(&self, id: &str, view: View) -> Option<(usize, bool)> {
        let viewed = |of: &str| if view == View::Side { None } else { self.view_row(of, view) };
        let row = viewed(id).or_else(|| self.row(id)).or_else(|| viewed("idle")).or_else(|| self.resolve("idle"))?;
        Some((row, self.clips[row].view == View::Side))
    }

    /// `frame_at` in `view` (DIR4 3.2): the frame of the row `resolve_view` picks; a row of `id` or `idle` (side or
    /// view) animates, any other shows its first frame, still.
    pub fn frame_at_view(&self, id: &str, view: View, ms: u64) -> Option<ClipFrame> {
        self.frame_at_view_still(id, view, ms, false)
    }

    /// `frame_at_view`; `still` (WINDUP1 2) plays a looping row's intro, then holds its frame `loop_from` (0 without
    /// one) instead of looping.
    pub fn frame_at_view_still(&self, id: &str, view: View, ms: u64, still: bool) -> Option<ClipFrame> {
        let (row, side) = self.resolve_view(id, view)?;
        let c = &self.clips[row];
        let of_view = view != View::Side && c.view == view && (c.of == id || c.of == "idle");
        let (frame, done) = if c.id == id || c.id == "idle" || of_view {
            if still { c.frame_at_still(ms) } else { c.frame_at(ms) }
        } else {
            (0, true)
        };
        Some(ClipFrame { row, clip: c.id.clone(), frame, done, side })
    }
}

/// One frame of a sheet chosen by time: its row, the clip that row holds (after fallback), the frame, whether a
/// one-shot clip has reached its end (always false for a looping one) and whether the row is a side row (DIR4: only
/// side rows are mirrored for a left facing).
#[derive(Debug, Clone, PartialEq)]
pub struct ClipFrame {
    pub row: usize,
    pub clip: String,
    pub frame: u32,
    pub done: bool,
    pub side: bool,
}

impl Clip {
    /// The clip's length: the sum of its frames' durations.
    pub fn total_ms(&self) -> u64 {
        self.ms.iter().map(|m| *m as u64).sum()
    }

    /// The ms from the clip's start to frame `frame` (the sum of the durations before it; the whole clip past its end).
    pub fn ms_before(&self, frame: u32) -> u64 {
        self.ms.iter().take(frame as usize).map(|m| *m as u64).sum()
    }

    /// FX1-L1: the ms from the clip's start to its `impact` frame (None: the clip marks none).
    pub fn impact_ms(&self) -> Option<u64> {
        self.impact.map(|f| self.ms_before(f))
    }

    /// The frame shown `ms` after the clip started and whether it is over: a looping clip wraps around (never over;
    /// with `loop_from` L it plays frames 0..L once, then loops L..frames); a one-shot clip holds its last frame once
    /// its durations have run.
    pub fn frame_at(&self, ms: u64) -> (u32, bool) {
        let total = self.total_ms();
        if total == 0 || self.ms.is_empty() {
            return (0, !self.looped);
        }
        let intro = self.intro_ms();
        let t = if self.looped && intro > 0 {
            let span = total.saturating_sub(intro);
            if ms < intro || span == 0 { ms.min(total - 1) } else { intro + (ms - intro) % span }
        } else if self.looped {
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

    /// WINDUP1 2: the ms of a looping clip's intro (the frames before `loop_from`; 0 without one or for a one-shot).
    pub fn intro_ms(&self) -> u64 {
        if self.looped && self.loop_from > 0 { self.ms_before(self.loop_from) } else { 0 }
    }

    /// `frame_at` held still (WINDUP1 2, reduced motion): a looping clip plays its intro, then holds frame
    /// `loop_from` (0 without one); a one-shot clip plays as `frame_at`.
    pub fn frame_at_still(&self, ms: u64) -> (u32, bool) {
        if !self.looped || ms < self.intro_ms() {
            return self.frame_at(ms);
        }
        (self.loop_from.min(self.frames.saturating_sub(1)), false)
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
    /// WINDUP1: the frame the shot starts at (`play_from`; 0 otherwise).
    from: u32,
}

/// Plays a body's clips on a clock the caller keeps (SPRITE1-L; milliseconds, scaled by the caller for animation
/// speed): a base loop (`idle`, a stance's idle) and over it one shot at a time (an attack, a hit, a walk for a set
/// time, a death that holds its last frame). Clips the body lacks fall back as `Clips::frame_at` does.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipPlayer {
    base: String,
    base_since: u64,
    /// WINDUP1: the frame the base starts at and whether it holds still after its intro (`set_base_from`).
    base_from: u32,
    base_still: bool,
    shot: Option<Shot>,
}

impl Default for ClipPlayer {
    fn default() -> Self {
        Self::new("idle")
    }
}

impl ClipPlayer {
    pub fn new(base: &str) -> Self {
        Self { base: base.to_string(), base_since: 0, base_from: 0, base_still: false, shot: None }
    }

    /// The base loop.
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Loops `clip` between shots from `now` (unchanged when it already does).
    pub fn set_base(&mut self, clip: &str, now: u64) {
        self.set_base_from(clip, now, 0, false);
    }

    /// WINDUP1 2: loops `clip` between shots from `now`, starting at frame `from` (its clock offset by the ms before
    /// it); `still` stops the loop part: the intro plays, then frame `loop_from` (0 without one) holds. An unchanged
    /// base keeps its clock and start frame (only `still` follows the call).
    pub fn set_base_from(&mut self, clip: &str, now: u64, from: u32, still: bool) {
        if self.base != clip {
            self.base = clip.to_string();
            self.base_since = now;
            self.base_from = from;
        }
        self.base_still = still;
    }

    /// Plays `clip` once from `now`, then the base loop again.
    pub fn play(&mut self, clip: &str, now: u64) {
        self.play_from(clip, now, 0);
    }

    /// WINDUP1 2: plays `clip` once from `now` starting at frame `frame` (its clock offset by the ms before it; its
    /// length is the clip's total less that offset), then the base loop again.
    pub fn play_from(&mut self, clip: &str, now: u64, frame: u32) {
        self.shot = Some(Shot { clip: clip.to_string(), since: now, for_ms: None, hold: false, from: frame });
    }

    /// Plays `clip` (looping) for `ms` from `now`, then the base loop again.
    pub fn play_for(&mut self, clip: &str, now: u64, ms: u64) {
        self.shot = Some(Shot { clip: clip.to_string(), since: now, for_ms: Some(ms), hold: false, from: 0 });
    }

    /// Plays `clip` once from `now` and keeps its last frame.
    pub fn play_and_hold(&mut self, clip: &str, now: u64) {
        self.shot = Some(Shot { clip: clip.to_string(), since: now, for_ms: None, hold: true, from: 0 });
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
        let len = s.for_ms.unwrap_or_else(|| clips.resolve(&s.clip).map_or(0, |r| clips.clips[r].total_ms()).saturating_sub(Self::shot_offset(clips, s)));
        s.since.saturating_add(len)
    }

    /// The ms a `play_from` shot skips (the frames before its start frame).
    fn shot_offset(clips: &Clips, s: &Shot) -> u64 {
        if s.from == 0 { 0 } else { clips.ms_before(&s.clip, s.from) }
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
        self.frame_facing(clips, now, View::Side)
    }

    /// `frame` in `view` (DIR4 3.2): the same clips on the same clocks drawn by `Clips::frame_at_view`. The player
    /// keeps side clip ids (`playing`, `held_for` and a shot's length are the side clip's), so a turn in mid-clip keeps
    /// the clip's clock; view rows have their side clip's frames and durations (DIR4 rule R3).
    pub fn frame_facing(&self, clips: &Clips, now: u64, view: View) -> Option<ClipFrame> {
        if let Some(s) = &self.shot {
            let t = now.saturating_sub(s.since) + Self::shot_offset(clips, s);
            if s.hold && self.shot_over(clips, s, now) {
                let mut f = clips.frame_at_view(&s.clip, view, u64::MAX)?;
                if let Some(c) = clips.clip(&f.clip).filter(|c| c.looped) {
                    // A held clip that loops (or the idle fallback) stops on its last frame.
                    f.frame = c.frames.saturating_sub(1);
                }
                f.done = true;
                return Some(f);
            }
            if !self.shot_over(clips, s, now) {
                let mut f = clips.frame_at_view(&s.clip, view, t)?;
                f.done = false;
                return Some(f);
            }
        }
        let offset = if self.base_from == 0 { 0 } else { clips.ms_before(&self.base, self.base_from) };
        clips.frame_at_view_still(&self.base, view, now.saturating_sub(self.base_since) + offset, self.base_still)
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

/// Every palette file (`units/<class>/<variant>/palettes/*.json`) in every layer, referenced or not and shadowed or
/// not, read with the loader's rules (`parse_palette`): one line per problem, `<path>: <reason>`, in layer order and
/// sorted by path within a layer. Empty: every palette file parses.
pub fn palette_problems(layers: &Layers) -> Vec<String> {
    let mut out = Vec::new();
    for root in &layers.roots {
        let mut files: Vec<PathBuf> = Vec::new();
        for class in sorted_entries(&root.join("units")).into_iter().filter(|p| vfs::is_dir(p)) {
            for variant in sorted_entries(&class).into_iter().filter(|p| vfs::is_dir(p)) {
                files.extend(sorted_entries(&variant.join("palettes")).into_iter().filter(|p| vfs::is_file(p) && p.extension().is_some_and(|x| x == "json")));
            }
        }
        for file in files {
            let shown = file.to_string_lossy().replace('\\', "/");
            let reasons = match vfs::read_to_string(&file) {
                Err(e) => vec![format!("unreadable ({e}).")],
                Ok(text) => match serde_json::from_str::<Value>(&text) {
                    Err(e) => vec![format!("not JSON ({e}).")],
                    Ok(v) => parse_palette(&v).err().unwrap_or_default(),
                },
            };
            out.extend(reasons.into_iter().map(|r| format!("{shown}: {r}")));
        }
    }
    out
}

/// The entries directly in `dir`, sorted by path (none when it is not a readable folder).
fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = vfs::list_dir(dir).unwrap_or_default();
    out.sort();
    out
}

/// Reads a JSON file of the layers at `rel`.
pub fn read_json(layers: &Layers, rel: &str) -> Option<Value> {
    let text = vfs::read_to_string(layers.find(rel)?).ok()?;
    serde_json::from_str(&text).ok()
}

// ---- Portrait face sidecars (PORTRAITFX1 4.1) ---------------------------------------------------------------------

/// The keys a face sidecar may hold.
pub const FACE_KEYS: [&str; 11] = ["size", "border", "eyes", "blink", "lid", "glow", "lines", "mouth", "face", "rim", "idle"];
/// FB-206 (2b): the old sidecar keys a reader still takes, (old, key of `FACE_KEYS`); a sidecar holding both names of
/// one key is refused.
pub const FACE_KEY_ALIASES: [(&str, &str); 1] = [("veins", "lines")];
/// PORTRAITFX1-EX: the face box a sidecar without its own `face` takes from its eye boxes, in units of the eyes' span
/// (the union's width, at least `size / 5`) around the union's centre and top: x from -0.9 span, y from -0.6 span,
/// 1.8 span wide, 1.9 span tall (clamped to the inner rect).
pub const FACE_FROM_EYES: [f32; 4] = [0.9, 0.6, 1.8, 1.9];
/// The keys of one `idle` entry.
pub const IDLE_KEYS: [&str; 3] = ["effect", "color", "intensity"];
/// The overlays a sidecar's `idle` may name.
pub const IDLE_OVERLAYS: [&str; 5] = ["glow", "lines", "point_glow", "particles_rise", "drift"];
/// FB-206 (2b): the old names of the overlay primitives, (old, canonical): growing lines from roots, a glow on the eye
/// boxes, falling specks, rising puffs. Readers of sidecars and cue rows map them (`canonical_primitive`); writers
/// write the canonical name.
pub const PRIMITIVE_ALIASES: [(&str, &str); 4] = [("veins", "lines"), ("eye_glow", "point_glow"), ("ash", "particles_rise"), ("smoke", "drift")];

/// FB-206 (2b): the canonical name of an effect: an old primitive name's new one (`PRIMITIVE_ALIASES`), any other name
/// unchanged.
pub fn canonical_primitive(name: &str) -> &str {
    PRIMITIVE_ALIASES.iter().find(|(old, _)| *old == name).map_or(name, |&(_, new)| new)
}
/// A sidecar's `size` range (image px).
pub const FACE_SIZE_MIN: i32 = 16;
pub const FACE_SIZE_MAX: i32 = 1024;

/// One idle overlay of a sidecar: an effect of `IDLE_OVERLAYS`, its colour (None: the effect's default) and intensity.
#[derive(Debug, Clone, PartialEq)]
pub struct IdleOverlay {
    pub effect: String,
    pub color: Option<[u8; 3]>,
    pub intensity: f32,
}

/// A portrait's face sidecar (`<portrait>.face.json`): anchors in image px of a `size`-wide PNG. Boxes are
/// `[x, y, w, h]`, glow anchors `[x, y, r]`, line roots `[x, y, deg, len]` (0 deg = right, 90 = down).
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub size: i32,
    /// The outer frame no effect draws on or moves (default `round(size / 30)`).
    pub border: i32,
    pub eyes: Vec<[i32; 4]>,
    pub blink: bool,
    /// The lid colour (None: sampled 1 px above each eye box).
    pub lid: Option<[u8; 3]>,
    pub glow: Vec<[i32; 3]>,
    /// The `lines` overlay's roots (sidecar key `lines`, old name `veins`).
    pub lines: Vec<[i32; 4]>,
    /// Reserved for layered puppets: parsed and checked, read by nothing.
    pub mouth: Option<[i32; 4]>,
    /// PORTRAITFX1-EX: the face box the expression effects tint and place their marks by (None: `face_box` derives it
    /// from the eye boxes).
    pub face: Option<[i32; 4]>,
    /// The rim light's key colour (None: the caller's default).
    pub rim: Option<[u8; 3]>,
    pub idle: Vec<IdleOverlay>,
}

impl Face {
    /// A sidecar with no anchors for a `size` portrait (the default border, blinking allowed, no overlays).
    pub fn plain(size: i32) -> Face {
        Face { size, border: default_border(size), eyes: Vec::new(), blink: true, lid: None, glow: Vec::new(), lines: Vec::new(), mouth: None, face: None, rim: None, idle: Vec::new() }
    }

    /// The inner rect's bounds on each axis, [lo, hi) in image px (inside the border).
    pub fn inner(&self) -> (i32, i32) {
        (self.border, self.size - self.border)
    }

    /// The union of the eye boxes `[x, y, w, h]` (None without one).
    pub fn eyes_box(&self) -> Option<[i32; 4]> {
        let x0 = self.eyes.iter().map(|e| e[0]).min()?;
        let y0 = self.eyes.iter().map(|e| e[1]).min()?;
        let x1 = self.eyes.iter().map(|e| e[0] + e[2]).max()?;
        let y1 = self.eyes.iter().map(|e| e[1] + e[3]).max()?;
        Some([x0, y0, x1 - x0, y1 - y0])
    }

    /// PORTRAITFX1-EX: the face box `[x, y, w, h]` in image px: the sidecar's `face`, else one derived from the eye boxes
    /// (`FACE_FROM_EYES`), clamped to the inner rect; None with neither (the expression tint is skipped).
    pub fn face_box(&self) -> Option<[i32; 4]> {
        let (lo, hi) = self.inner();
        let clamp = |b: [i32; 4]| {
            let (x0, y0) = (b[0].clamp(lo, hi - 1), b[1].clamp(lo, hi - 1));
            let (x1, y1) = ((b[0] + b[2]).clamp(x0 + 1, hi), (b[1] + b[3]).clamp(y0 + 1, hi));
            [x0, y0, x1 - x0, y1 - y0]
        };
        if let Some(b) = self.face {
            return Some(clamp(b));
        }
        let [x, y, w, _] = self.eyes_box()?;
        let span = (w as f32).max((self.size as f32 / 5.0).round());
        let cx = x as f32 + w as f32 / 2.0;
        let [fx, fy, fw, fh] = FACE_FROM_EYES;
        Some(clamp([(cx - fx * span).round() as i32, (y as f32 - fy * span).round() as i32, (fw * span).round() as i32, (fh * span).round() as i32]))
    }
}

/// `round(size / 30)`: a sidecar's border when it names none (8 for a 240 px portrait).
pub fn default_border(size: i32) -> i32 {
    (size as f64 / 30.0).round() as i32
}

/// The sidecar path of a portrait: `.png` replaced by `.face.json` (`portraits/npcs/innkeeper.png` ->
/// `portraits/npcs/innkeeper.face.json`).
pub fn face_file(rel: &str) -> String {
    format!("{}.face.json", rel.strip_suffix(".png").unwrap_or(rel))
}

/// A JSON number with no fraction as an i32.
fn whole(v: &Value) -> Option<i32> {
    let f = v.as_f64()?;
    (f.fract() == 0.0 && f.abs() <= i32::MAX as f64).then_some(f as i32)
}

/// "" (None) or a `#rrggbb` colour; a problem otherwise.
fn colour_or_empty(raw: Option<&Value>, label: &str, problems: &mut Vec<String>) -> Option<[u8; 3]> {
    match raw {
        None => None,
        Some(Value::String(s)) if s.is_empty() => None,
        Some(v) => {
            let c = v.as_str().and_then(hex_colour);
            if c.is_none() {
                problems.push(format!("{label} must be \"\" or a #rrggbb colour."));
            }
            c
        }
    }
}

/// A list of `N`-number lists of whole numbers (`shape` names them in the message).
fn tuples<const N: usize>(raw: Option<&Value>, label: &str, shape: &str, problems: &mut Vec<String>) -> Vec<[i32; N]> {
    let Some(raw) = raw else { return Vec::new() };
    let bad = |problems: &mut Vec<String>| problems.push(format!("{label} must be a list of {shape} lists of whole numbers."));
    let Some(list) = raw.as_array() else {
        bad(problems);
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in list {
        let nums: Option<Vec<i32>> = item.as_array().filter(|a| a.len() == N).and_then(|a| a.iter().map(whole).collect());
        match nums.and_then(|n| <[i32; N]>::try_from(n).ok()) {
            Some(t) => out.push(t),
            None => {
                bad(problems);
                return out;
            }
        }
    }
    out
}

/// Reads a face sidecar (PORTRAITFX1 4.1): the keys and their types (unknown keys refused, `size` required, colours
/// "" or `#rrggbb`, whole numbers for every anchor); the ranges and anchor rules are `face_problems`. Every problem is
/// listed. FB-206 (2b): an old key (`FACE_KEY_ALIASES`) reads as its new one (both in one sidecar: refused), and an
/// idle effect's old name as its canonical one (`canonical_primitive`), so the parsed face holds canonical names only.
pub fn parse_face(v: &Value) -> Result<Face, Vec<String>> {
    let mut p: Vec<String> = Vec::new();
    let Some(o) = v.as_object() else { return Err(vec!["a face sidecar must be an object {size, eyes, ...}.".into()]) };
    let old_key = |k: &str| FACE_KEY_ALIASES.iter().any(|(old, _)| *old == k);
    for k in o.keys().filter(|k| !FACE_KEYS.contains(&k.as_str()) && !old_key(k)) {
        p.push(format!("the face sidecar has no key '{k}' (keys: {}).", FACE_KEYS.join(", ")));
    }
    for (old, new) in FACE_KEY_ALIASES {
        if o.contains_key(old) && o.contains_key(new) {
            p.push(format!("the face sidecar holds both '{new}' and its old name '{old}' (keep '{new}')."));
        }
    }
    // a key, or its old name when the sidecar was written before the rename
    let key = |k: &str| o.get(k).or_else(|| FACE_KEY_ALIASES.iter().find(|(_, new)| *new == k).and_then(|(old, _)| o.get(*old)));
    let size = match o.get("size").map(whole) {
        Some(Some(s)) => s,
        Some(None) => {
            p.push("size must be a whole number.".into());
            0
        }
        None => {
            p.push("size is required.".into());
            0
        }
    };
    let border = match o.get("border").map(whole) {
        None => default_border(size),
        Some(Some(b)) => b,
        Some(None) => {
            p.push("border must be a whole number.".into());
            default_border(size)
        }
    };
    let eyes = tuples::<4>(o.get("eyes"), "eyes", "[x, y, w, h]", &mut p);
    let blink = match o.get("blink") {
        None => true,
        Some(Value::Bool(b)) => *b,
        Some(_) => {
            p.push("blink must be true or false.".into());
            true
        }
    };
    let lid = colour_or_empty(o.get("lid"), "lid", &mut p);
    let glow = tuples::<3>(o.get("glow"), "glow", "[x, y, r]", &mut p);
    let lines = tuples::<4>(key("lines"), "lines", "[x, y, deg, len]", &mut p);
    let one_box = |key: &str, p: &mut Vec<String>| match o.get(key) {
        None => None,
        Some(Value::Array(a)) if a.is_empty() => None,
        Some(m) => {
            let nums: Option<Vec<i32>> = m.as_array().filter(|a| a.len() == 4).and_then(|a| a.iter().map(whole).collect());
            let t = nums.and_then(|n| <[i32; 4]>::try_from(n).ok());
            if t.is_none() {
                p.push(format!("{key} must be [] or [x, y, w, h] of whole numbers."));
            }
            t
        }
    };
    let mouth = one_box("mouth", &mut p);
    let face = one_box("face", &mut p);
    let rim = colour_or_empty(o.get("rim"), "rim", &mut p);
    let mut idle = Vec::new();
    match o.get("idle") {
        None => {}
        Some(Value::Array(list)) => {
            for (i, e) in list.iter().enumerate() {
                let n = i + 1;
                let Some(e) = e.as_object() else {
                    p.push(format!("idle entry {n} must be an object {{effect, color, intensity}}."));
                    continue;
                };
                for k in e.keys().filter(|k| !IDLE_KEYS.contains(&k.as_str())) {
                    p.push(format!("idle entry {n} has no key '{k}' (keys: {}).", IDLE_KEYS.join(", ")));
                }
                let effect = match e.get("effect") {
                    Some(Value::String(s)) => canonical_primitive(s).to_string(),
                    _ => {
                        p.push(format!("idle entry {n}: effect must be a name."));
                        String::new()
                    }
                };
                let color = colour_or_empty(e.get("color"), &format!("idle entry {n}: color"), &mut p);
                let intensity = match e.get("intensity") {
                    None => 1.0,
                    Some(x) => x.as_f64().map(|f| f as f32).unwrap_or_else(|| {
                        p.push(format!("idle entry {n}: intensity must be a number."));
                        1.0
                    }),
                };
                idle.push(IdleOverlay { effect, color, intensity });
            }
        }
        Some(_) => p.push("idle must be a list of {effect, color, intensity}.".into()),
    }
    if p.is_empty() {
        Ok(Face { size, border, eyes, blink, lid, glow, lines, mouth, face, rim, idle })
    } else {
        Err(p)
    }
}

/// Every range and anchor rule of a parsed sidecar (PORTRAITFX1 4.1; the kit's asset check A11 calls it), one line
/// each: `size` 16-1024 and equal to the PNG's width `png_w` (0: not checked; else a warning, the anchors being
/// scaled by width / size), `border` 0-size/8, at most 2 eye boxes inside the inner rect (w 2-size/6, h 1-size/10),
/// at most 3 glow anchors (centre inside, r 4-size/2), at most 3 line roots (root inside, deg -180-360, len
/// 6-size/2), the mouth inside, the face box inside and at least size/8 a side (PORTRAITFX1-EX), at most 4 idle overlays named by `IDLE_OVERLAYS` with their anchors and an intensity
/// 0.05-1.0. Empty: the sidecar is sound.
pub fn face_problems(face: &Face, png_w: i32) -> Vec<String> {
    let mut out = Vec::new();
    let s = face.size;
    if !(FACE_SIZE_MIN..=FACE_SIZE_MAX).contains(&s) {
        out.push(format!("size must be {FACE_SIZE_MIN}-{FACE_SIZE_MAX} (is {s})."));
    }
    if png_w > 0 && png_w != s {
        out.push(format!("size {s} differs from the PNG's width {png_w}: the anchors are scaled by {png_w} / {s}."));
    }
    if !(0..=s / 8).contains(&face.border) {
        out.push(format!("border must be 0-{} (is {}).", s / 8, face.border));
    }
    let (lo, hi) = face.inner();
    let inside = |x: i32, y: i32| x >= lo && y >= lo && x < hi && y < hi;
    if face.eyes.len() > 2 {
        out.push(format!("eyes holds at most 2 boxes (has {}).", face.eyes.len()));
    }
    for (i, &[x, y, w, h]) in face.eyes.iter().enumerate() {
        let n = i + 1;
        if x < lo || y < lo || x + w > hi || y + h > hi {
            out.push(format!("eye box {n} [{x}, {y}, {w}, {h}] must lie inside the inner rect ({lo}-{hi})."));
        }
        if !(2..=s / 6).contains(&w) {
            out.push(format!("eye box {n} width must be 2-{} (is {w}).", s / 6));
        }
        if !(1..=s / 10).contains(&h) {
            out.push(format!("eye box {n} height must be 1-{} (is {h}).", s / 10));
        }
    }
    if face.glow.len() > 3 {
        out.push(format!("glow holds at most 3 anchors (has {}).", face.glow.len()));
    }
    for (i, &[x, y, r]) in face.glow.iter().enumerate() {
        let n = i + 1;
        if !inside(x, y) {
            out.push(format!("glow anchor {n} centre ({x}, {y}) must lie inside the inner rect ({lo}-{hi})."));
        }
        if !(4..=s / 2).contains(&r) {
            out.push(format!("glow anchor {n} radius must be 4-{} (is {r}).", s / 2));
        }
    }
    if face.lines.len() > 3 {
        out.push(format!("lines holds at most 3 roots (has {}).", face.lines.len()));
    }
    for (i, &[x, y, deg, len]) in face.lines.iter().enumerate() {
        let n = i + 1;
        if !inside(x, y) {
            out.push(format!("line root {n} ({x}, {y}) must lie inside the inner rect ({lo}-{hi})."));
        }
        if !(-180..=360).contains(&deg) {
            out.push(format!("line root {n} deg must be -180-360 (is {deg})."));
        }
        if !(6..=s / 2).contains(&len) {
            out.push(format!("line root {n} len must be 6-{} (is {len}).", s / 2));
        }
    }
    if let Some([x, y, w, h]) = face.mouth {
        if x < lo || y < lo || x + w > hi || y + h > hi || w < 1 || h < 1 {
            out.push(format!("mouth [{x}, {y}, {w}, {h}] must lie inside the inner rect ({lo}-{hi})."));
        }
    }
    if let Some([x, y, w, h]) = face.face {
        if x < lo || y < lo || x + w > hi || y + h > hi {
            out.push(format!("face box [{x}, {y}, {w}, {h}] must lie inside the inner rect ({lo}-{hi})."));
        }
        if w < s / 8 || h < s / 8 {
            out.push(format!("face box width and height must be at least {} (is {w} x {h}).", s / 8));
        }
    }
    if face.idle.len() > 4 {
        out.push(format!("idle holds at most 4 overlays (has {}).", face.idle.len()));
    }
    for o in &face.idle {
        let e = o.effect.as_str();
        if !IDLE_OVERLAYS.contains(&e) {
            out.push(format!("idle '{e}' is not an overlay (one of {}).", IDLE_OVERLAYS.join(", ")));
        }
        match e {
            "glow" if face.glow.is_empty() => out.push("idle glow needs a glow anchor.".into()),
            "lines" if face.lines.is_empty() => out.push("idle lines needs a line root.".into()),
            "point_glow" if face.eyes.is_empty() => out.push("idle point_glow needs an eye box.".into()),
            _ => {}
        }
        if !(0.05..=1.0).contains(&o.intensity) {
            out.push(format!("idle '{e}' intensity must be 0.05-1.0 (is {}).", o.intensity));
        }
    }
    out
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

    fn tree(name: &str) -> crate::scratch::Scratch {
        let root = crate::scratch::Scratch::vacant(&format!("assets_{name}"));
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

    /// The sidecar example of PORTRAITFX1 4.1.
    fn face_example() -> Value {
        json!({"size": 240, "border": 8, "eyes": [[98, 92, 12, 6], [128, 90, 12, 6]], "blink": true, "lid": "",
            "glow": [[120, 200, 30]], "lines": [[140, 120, 200, 40]], "mouth": [], "rim": "",
            "idle": [{"effect": "point_glow", "color": "#c084fc", "intensity": 0.8}]})
    }

    /// The example with `key` set to `v`, parsed (it must parse), then its range and anchor problems on a 240 px PNG.
    fn face_with(key: &str, v: Value) -> Vec<String> {
        let mut f = face_example();
        f[key] = v;
        face_problems(&parse_face(&f).unwrap(), 240)
    }

    #[test]
    fn e1_the_sidecar_example_parses_and_each_broken_rule_names_itself() {
        let f = parse_face(&face_example()).unwrap();
        assert_eq!((f.size, f.border, f.eyes.len(), f.blink, f.lid, f.rim, f.mouth), (240, 8, 2, true, None, None, None));
        assert_eq!((f.glow.clone(), f.lines.clone()), (vec![[120, 200, 30]], vec![[140, 120, 200, 40]]));
        assert_eq!(f.idle, vec![IdleOverlay { effect: "point_glow".into(), color: Some([0xc0, 0x84, 0xfc]), intensity: 0.8 }]);
        assert!(face_problems(&f, 240).is_empty(), "{:?}", face_problems(&f, 240));
        assert_eq!(parse_face(&json!({"size": 90})).unwrap().border, 3, "the border defaults to round(size / 30)");

        // An eye box outside the inner rect (8..232): x 4 < 8.
        assert_eq!(face_with("eyes", json!([[4, 92, 12, 6]])), ["eye box 1 [4, 92, 12, 6] must lie inside the inner rect (8-232)."]);
        assert_eq!(face_with("eyes", json!([[98, 92, 12, 6], [128, 90, 12, 6], [60, 90, 12, 6]])), ["eyes holds at most 2 boxes (has 3)."]);
        assert_eq!(face_with("eyes", json!([])), ["idle point_glow needs an eye box."]);
        let mut no_root = face_example();
        no_root["lines"] = json!([]);
        no_root["idle"] = json!([{"effect": "lines"}]);
        assert_eq!(face_problems(&parse_face(&no_root).unwrap(), 240), ["idle lines needs a line root."]);
        let mut unknown = face_example();
        unknown["nose"] = json!([1, 2]);
        assert_eq!(parse_face(&unknown).unwrap_err(), [format!("the face sidecar has no key 'nose' (keys: {}).", FACE_KEYS.join(", "))]);
        let mut bad_colour = face_example();
        bad_colour["rim"] = json!("amber");
        assert_eq!(parse_face(&bad_colour).unwrap_err(), ["rim must be \"\" or a #rrggbb colour."]);
        assert_eq!(face_with("border", json!(31)), ["border must be 0-30 (is 31)."], "240 / 8 = 30");
        let f200 = parse_face(&json!({"size": 200})).unwrap();
        assert_eq!(face_problems(&f200, 240), ["size 200 differs from the PNG's width 240: the anchors are scaled by 240 / 200."]);
        assert!(face_problems(&f200, 0).is_empty(), "width 0 skips the scale check");
        assert_eq!(face_file("portraits/npcs/innkeeper.png"), "portraits/npcs/innkeeper.face.json");
    }

    #[test]
    fn face_ranges_follow_the_size() {
        assert_eq!(face_with("glow", json!([[120, 200, 3]])), ["glow anchor 1 radius must be 4-120 (is 3)."]);
        assert_eq!(face_with("lines", json!([[140, 120, 400, 40]])), ["line root 1 deg must be -180-360 (is 400)."]);
        assert_eq!(face_with("eyes", json!([[98, 92, 41, 6]])), ["eye box 1 width must be 2-40 (is 41)."]);
        assert_eq!(face_with("idle", json!([{"effect": "sparkle", "intensity": 2.0}])), ["idle 'sparkle' is not an overlay (one of glow, lines, point_glow, particles_rise, drift).", "idle 'sparkle' intensity must be 0.05-1.0 (is 2)."]);
        assert!(parse_face(&json!({"eyes": [[1, 2, 3]]})).unwrap_err().contains(&"size is required.".to_string()));
        assert!(parse_face(&json!({"size": 96, "eyes": [[1.5, 2, 3, 4]]})).is_err(), "anchors are whole numbers");
    }

    /// PORTRAITFX1-EX: the `face` box is checked like the mouth (inside the inner rect) and is at least size / 8 on each
    /// side; without one the box comes from the eye boxes (`FACE_FROM_EYES`); with neither there is none.
    #[test]
    fn the_face_box_is_its_own_or_the_eyes() {
        assert!(face_with("face", json!([70, 60, 100, 110])).is_empty());
        assert!(face_with("face", json!([])).is_empty(), "[] is no box");
        assert_eq!(face_with("face", json!([4, 60, 100, 110])), ["face box [4, 60, 100, 110] must lie inside the inner rect (8-232)."]);
        assert_eq!(face_with("face", json!([70, 60, 29, 110])), ["face box width and height must be at least 30 (is 29 x 110)."], "240 / 8 = 30");
        assert_eq!(parse_face(&json!({"size": 96, "face": [1, 2]})).unwrap_err(), ["face must be [] or [x, y, w, h] of whole numbers."]);
        let mut own = face_example();
        own["face"] = json!([70, 60, 100, 110]);
        assert_eq!(parse_face(&own).unwrap().face_box(), Some([70, 60, 100, 110]));
        // from the eyes: the union [98, 90, 42, 8], span max(42, round(240 / 5)) = 48 around its centre x 119 and top y 90
        let f = parse_face(&face_example()).unwrap();
        assert_eq!(f.eyes_box(), Some([98, 90, 42, 8]));
        let [fx, fy, fw, fh] = FACE_FROM_EYES;
        let r = |v: f32| v.round() as i32;
        assert_eq!(f.face_box(), Some([r(119.0 - fx * 48.0), r(90.0 - fy * 48.0), r(fw * 48.0), r(fh * 48.0)]));
        assert_eq!(Face::plain(96).face_box(), None, "no face box and no eyes: none");
        // a box past the border is clamped to the inner rect
        let edge = Face { face: Some([0, 0, 300, 300]), ..Face::plain(240) };
        assert_eq!(edge.face_box(), Some([8, 8, 224, 224]));
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
