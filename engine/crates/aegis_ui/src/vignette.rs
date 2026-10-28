//! The vignette player (STORYUI1-E, docs/specs/STORYUI1.md 5.1): engine-only, no Godot source. A reel of still cards
//! (four styles: a picture and a caption, text on the backdrop, a title card, a speaker's line), each run through fade
//! in, typing, hold and fade out, with a typewriter at the player's text speed, skip, sound cues and drifting ash over
//! a code-drawn backdrop. The model (`Player`) moves only through `advance(ms)`, `press()` and `skip()`, so tests drive
//! it without a clock; `draw` paints a card on the 640 x 360 stage in the caller's `Palette`; `VignetteScreen` is the
//! full screen that measures the frame time, plays the cues and runs its `on_done` callback when the reel ends or is
//! skipped. What a reel holds (which cards, their tokens and conditions) is the game's or its kit's. A card may shake the
//! view (FX1-V, FX1 5.2): its `shake` preset, from the game's table (`with_shakes`), moves the card's picture and texts.

use crate::art::Art;
use crate::audio::Bus;
use crate::canvas::{rgb, rgba, Rect, Rgba};
use crate::fx::{Camera, ShakeLimits, ShakeTable};
use crate::painter::Painter;
use crate::screen::{Context, Cx, Key, Screen, Settings};
use crate::ui::{tip, Gfx, Ui, Widget, BODY, DIM, GOLD, RADIUS, SMALL, TEXT, H, W};
use aegis_core::json;
use serde_json::{json, Value};
use std::time::Instant;

/// The settings keys (engine keys, `Settings::engine_defaults`).
pub const TEXT_SPEED: &str = "text_speed";
pub const REDUCE_MOTION: &str = "reduce_motion";
/// The answer tag of the "Skip this scene?" question.
pub const SKIP_TAG: &str = "vignette.skip";
pub const SKIP_QUESTION: &str = "Skip this scene?";
/// The longest frame the screen feeds the model (a stall or a window drag does not jump the reel).
pub const MAX_FRAME_MS: u32 = 100;
/// The card text size (the caption, the line, the text card, the subtitle); the title is three times it.
pub const CARD_BODY: f32 = BODY + 1.0;
/// Layout on the 640 x 360 stage (V9).
pub const PICTURE: Rect = Rect::new(40, 20, 560, 236);
pub const CAPTION_Y: i32 = 266;
pub const CAPTION_X: i32 = 40;
pub const CAPTION_W: i32 = 560;
pub const CAPTION_PAD: i32 = 8;
pub const TEXT_BOX: Rect = Rect::new(80, 120, 480, 120);
pub const TITLE_Y: f32 = 140.0;
pub const RULE_W: i32 = 200;
pub const PORTRAIT: i32 = 64;
/// Room kept under a caption box for the "Click to continue" hint.
pub const HINT_ROOM: i32 = 18;
pub const HINT: &str = "Click to continue";
/// Ash (V11): specks per card and their fall in logical px per second.
pub const ASH_SPECKS: u32 = 24;
pub const ASH_FALL: f32 = 6.0;
/// The engine's frame colour when a reel names none (a near-black).
pub const DEFAULT_FRAME: Rgba = rgb(10, 11, 16);

/// `#rrggbb` as a colour (None for anything else).
pub fn hex(s: &str) -> Option<Rgba> {
    let h = s.strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let c = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some(rgb(c(0)?, c(2)?, c(4)?))
}

// ---- Settings: text speed, reduce motion ----------------------------------------------------------------------

/// The typewriter's speed (setting `text_speed`): 30, 60 or 120 characters a second, or all at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSpeed {
    Slow,
    Normal,
    Fast,
    Instant,
}

impl TextSpeed {
    pub const ALL: [TextSpeed; 4] = [TextSpeed::Slow, TextSpeed::Normal, TextSpeed::Fast, TextSpeed::Instant];

    /// The setting's value (`slow`, `normal`, `fast`, `instant`); anything else is `normal`.
    pub fn parse(s: &str) -> TextSpeed {
        match s {
            "slow" => TextSpeed::Slow,
            "fast" => TextSpeed::Fast,
            "instant" => TextSpeed::Instant,
            _ => TextSpeed::Normal,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TextSpeed::Slow => "slow",
            TextSpeed::Normal => "normal",
            TextSpeed::Fast => "fast",
            TextSpeed::Instant => "instant",
        }
    }

    /// The label a settings screen shows.
    pub fn label(self) -> &'static str {
        match self {
            TextSpeed::Slow => "Slow",
            TextSpeed::Normal => "Normal",
            TextSpeed::Fast => "Fast",
            TextSpeed::Instant => "Instant",
        }
    }

    /// Characters a second (None: the whole text at once).
    pub fn cps(self) -> Option<u32> {
        match self {
            TextSpeed::Slow => Some(30),
            TextSpeed::Normal => Some(60),
            TextSpeed::Fast => Some(120),
            TextSpeed::Instant => None,
        }
    }

    /// A card's fade time at this speed (V6): `fast` halves it, `instant` makes it 0.
    pub fn fade(self, ms: u32) -> u32 {
        match self {
            TextSpeed::Fast => ms / 2,
            TextSpeed::Instant => 0,
            _ => ms,
        }
    }

    /// The player's choice in the settings.
    pub fn of(settings: &Settings) -> TextSpeed {
        TextSpeed::parse(settings.v[TEXT_SPEED].as_str().unwrap_or(""))
    }
}

/// The shared reduce-motion setting (no drifting ash; other moving effects read the same key).
pub fn reduce_motion(settings: &Settings) -> bool {
    settings.v[REDUCE_MOTION].as_bool().unwrap_or(false)
}

// ---- Cards and reels ------------------------------------------------------------------------------------------

/// A card's style (STORYUI1 4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// A picture area and a caption.
    Image,
    /// Text centred on the backdrop.
    Text,
    /// A title and a subtitle.
    Title,
    /// A speaker's portrait, name and line.
    Line,
}

impl Style {
    pub fn parse(s: &str) -> Option<Style> {
        Some(match s {
            "image" => Style::Image,
            "text" => Style::Text,
            "title" => Style::Title,
            "line" => Style::Line,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Style::Image => "image",
            Style::Text => "text",
            Style::Title => "title",
            Style::Line => "line",
        }
    }
}

/// One card, resolved (tokens filled, conditions applied, the speaker's name and portrait chain looked up by the
/// caller).
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub style: Style,
    /// `image` only: a file under the asset layers; "" or a missing file draws the backdrop in the picture area.
    pub image: String,
    /// The code-drawn backdrop, a vertical gradient.
    pub top: Rgba,
    pub bottom: Rgba,
    /// Drifting ash specks over the backdrop.
    pub ash: bool,
    pub title: String,
    pub subtitle: String,
    /// The typed text (the caption, the line, the text card's text).
    pub text: String,
    /// `line`: the speaker's name and portrait chain (the first file a layer holds; none draws an empty frame).
    pub speaker: String,
    pub portrait: Vec<String>,
    pub fade_in_ms: u32,
    pub fade_out_ms: u32,
    /// 0 waits for a press; above 0 the card moves on by itself that long after its text is out.
    pub hold_ms: u32,
    /// A sound under `audio/`, played as the card starts.
    pub cue: String,
    /// A camera shake preset (`fx::SHAKE_PRESETS`) started with the card's fade in; "" none, an unknown name none.
    pub shake: String,
}

impl Default for Card {
    fn default() -> Self {
        Card {
            style: Style::Image,
            image: String::new(),
            top: DEFAULT_FRAME,
            bottom: DEFAULT_FRAME,
            ash: false,
            title: String::new(),
            subtitle: String::new(),
            text: String::new(),
            speaker: String::new(),
            portrait: Vec::new(),
            fade_in_ms: 600,
            fade_out_ms: 400,
            hold_ms: 0,
            cue: String::new(),
            shake: String::new(),
        }
    }
}

fn ms_of(d: &json::Dict, key: &str, fallback: u32, max: u32) -> u32 {
    json::i(d, key, fallback as i64).clamp(0, max as i64) as u32
}

impl Card {
    /// A card from its JSON form (the keys of STORYUI1 4.2: style, image, top, bottom, ash, title, subtitle, text,
    /// fade_in_ms, fade_out_ms, hold_ms, cue; FX1's shake); bad values fall back to the defaults. `speaker`, `portrait` and `when`
    /// are the caller's to resolve.
    pub fn from_json(v: &Value) -> Card {
        let d = json::dict(v);
        let base = Card::default();
        let top = hex(&json::s(d, "top")).unwrap_or(base.top);
        Card {
            style: Style::parse(&json::s(d, "style")).unwrap_or(Style::Image),
            image: json::s(d, "image"),
            top,
            bottom: hex(&json::s(d, "bottom")).unwrap_or(top),
            ash: json::b(d, "ash", false),
            title: json::s(d, "title"),
            subtitle: json::s(d, "subtitle"),
            text: json::s(d, "text"),
            fade_in_ms: ms_of(d, "fade_in_ms", base.fade_in_ms, 3000),
            fade_out_ms: ms_of(d, "fade_out_ms", base.fade_out_ms, 3000),
            hold_ms: ms_of(d, "hold_ms", base.hold_ms, 20000),
            cue: json::s(d, "cue"),
            shake: json::s(d, "shake"),
            ..base
        }
    }
}

/// A reel: the cards and how they play.
#[derive(Debug, Clone, PartialEq)]
pub struct Reel {
    pub id: String,
    /// "" keeps the music playing, `none` is silence, else a track under `audio/`.
    pub music: String,
    /// The letterbox and fade colour.
    pub frame: Rgba,
    /// False: Skip is hidden and refused.
    pub skippable: bool,
    pub cards: Vec<Card>,
}

impl Default for Reel {
    fn default() -> Self {
        Reel { id: String::new(), music: String::new(), frame: DEFAULT_FRAME, skippable: true, cards: Vec::new() }
    }
}

impl Reel {
    /// A reel from its JSON form (id, music, frame, skippable, cards; STORYUI1 4.1), every card through
    /// `Card::from_json`.
    pub fn from_json(v: &Value) -> Reel {
        let d = json::dict(v);
        Reel {
            id: json::s(d, "id"),
            music: json::s(d, "music"),
            frame: hex(&json::s(d, "frame")).unwrap_or(DEFAULT_FRAME),
            skippable: json::b(d, "skippable", true),
            cards: json::aget(d, "cards").iter().map(Card::from_json).collect(),
        }
    }

    /// The music wish (V8): "" none (the screen below decides), `none` silence (`Some("")`), else the track.
    pub fn music_wish(&self) -> Option<String> {
        match self.music.as_str() {
            "" => None,
            "none" => Some(String::new()),
            path => Some(path.to_string()),
        }
    }
}

/// The engine's sample reel (the UI Gallery's vignette entry): one card of each style on code-drawn backdrops.
pub fn sample_reel() -> Reel {
    let dusk = (rgb(10, 11, 16), rgb(44, 21, 70));
    let slate = (rgb(30, 35, 45), rgb(10, 11, 16));
    Reel {
        id: "sample".into(),
        cards: vec![
            Card { style: Style::Image, top: dusk.0, bottom: dusk.1, ash: true, text: "An image card: a picture area and a caption that types out letter by letter. No picture is named, so the backdrop fills the picture area, with drifting ash.".into(), fade_in_ms: 1000, hold_ms: 3000, cue: "sfx/ui_confirm.wav".into(), ..Card::default() },
            Card { style: Style::Text, top: slate.0, bottom: slate.1, text: "A text card: the text sits centred on the backdrop. This one waits for a click, Space or Enter; Esc or a right click skips the reel.".into(), ..Card::default() },
            Card { style: Style::Title, top: dusk.0, bottom: dusk.0, title: "Title card".into(), subtitle: "A subtitle under the title, between two rules".into(), fade_in_ms: 800, hold_ms: 2500, fade_out_ms: 600, ..Card::default() },
            Card { style: Style::Line, top: slate.0, bottom: slate.1, speaker: "A speaker".into(), text: "A line card: the speaker's portrait sits left of the caption box, the name above the line. Without a portrait file the frame stays empty.".into(), fade_in_ms: 300, fade_out_ms: 300, ..Card::default() },
        ],
        ..Reel::default()
    }
}

// ---- The model --------------------------------------------------------------------------------------------------

/// Where a card is (V1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    FadeIn,
    Typing,
    Hold,
    FadeOut,
    Done,
}

impl Phase {
    pub fn name(self) -> &'static str {
        match self {
            Phase::FadeIn => "fade_in",
            Phase::Typing => "typing",
            Phase::Hold => "hold",
            Phase::FadeOut => "fade_out",
            Phase::Done => "done",
        }
    }
}

/// The reel's playback: the card, its phase and the time spent in it. No clock: it moves only through `advance`,
/// `press` and `skip` (V12).
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub reel: Reel,
    pub speed: TextSpeed,
    /// The shake presets a card's `shake` names (the game's table; default the engine's) and their limits (default the
    /// engine ceiling).
    pub shakes: ShakeTable,
    pub shake_limits: ShakeLimits,
    card: usize,
    phase: Phase,
    /// Milliseconds in the current phase.
    t: u32,
    /// Milliseconds since the current card started (the ash drift).
    card_ms: u64,
    /// Cues of the cards started since the last `take_cues`.
    cues: Vec<String>,
    skipped: bool,
}

impl Player {
    /// The reel at its first card's fade in (its cue queued); an empty reel is done at once.
    pub fn new(reel: Reel, speed: TextSpeed) -> Player {
        let mut p = Player { reel, speed, shakes: ShakeTable::default(), shake_limits: ShakeLimits::CEILING, card: 0, phase: Phase::Done, t: 0, card_ms: 0, cues: Vec::new(), skipped: false };
        p.start_card(0);
        p.advance(0);
        p
    }

    fn start_card(&mut self, i: usize) {
        self.card = i;
        self.t = 0;
        self.card_ms = 0;
        match self.reel.cards.get(i) {
            Some(c) => {
                self.phase = Phase::FadeIn;
                if !c.cue.is_empty() {
                    self.cues.push(c.cue.clone());
                }
            }
            None => {
                self.card = self.reel.cards.len();
                self.phase = Phase::Done;
            }
        }
    }

    /// The current phase's length; None waits for a press (a hold of 0).
    fn phase_len(&self) -> Option<u32> {
        let c = self.reel.cards.get(self.card)?;
        match self.phase {
            Phase::FadeIn => Some(self.speed.fade(c.fade_in_ms)),
            Phase::Typing => Some(match self.speed.cps() {
                Some(cps) => (self.total() as u64 * 1000).div_ceil(cps as u64) as u32,
                None => 0,
            }),
            Phase::Hold => (c.hold_ms > 0).then_some(c.hold_ms),
            Phase::FadeOut => Some(self.speed.fade(c.fade_out_ms)),
            Phase::Done => None,
        }
    }

    /// The phase after the current one (the next card after a fade out).
    fn next_phase(&mut self) {
        self.t = 0;
        match self.phase {
            Phase::FadeIn => self.phase = Phase::Typing,
            Phase::Typing => self.phase = Phase::Hold,
            Phase::Hold => self.phase = Phase::FadeOut,
            Phase::FadeOut => self.start_card(self.card + 1),
            Phase::Done => {}
        }
    }

    /// `ms` of play: phases end when their time is up, the rest of the time going to the next one.
    pub fn advance(&mut self, ms: u32) {
        let mut left = ms;
        while self.phase != Phase::Done {
            match self.phase_len() {
                Some(len) if left >= len.saturating_sub(self.t) => {
                    let used = len.saturating_sub(self.t);
                    left -= used;
                    self.card_ms += used as u64;
                    self.next_phase();
                }
                _ => {
                    self.t = self.t.saturating_add(left);
                    self.card_ms += left as u64;
                    break;
                }
            }
        }
    }

    /// A press (V3) does one thing: ends the fade in, shows the whole text, starts the fade out or ends it.
    pub fn press(&mut self) {
        if self.phase != Phase::Done {
            self.next_phase();
            // phases of no length (an empty text, instant fades) pass at once
            self.advance(0);
        }
    }

    /// Skip (V5): a skippable reel is done at once and plays no further cue; true when it skipped.
    pub fn skip(&mut self) -> bool {
        if !self.reel.skippable || self.phase == Phase::Done {
            return false;
        }
        self.phase = Phase::Done;
        self.card = self.reel.cards.len();
        self.t = 0;
        self.cues.clear();
        self.skipped = true;
        true
    }

    pub fn done(&self) -> bool {
        self.phase == Phase::Done
    }

    /// Whether the reel ended through `skip`.
    pub fn skipped(&self) -> bool {
        self.skipped
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The milliseconds spent in the current phase.
    pub fn phase_ms(&self) -> u32 {
        self.t
    }

    /// The current card's index (the card count once done).
    pub fn card_index(&self) -> usize {
        self.card
    }

    pub fn card(&self) -> Option<&Card> {
        self.reel.cards.get(self.card)
    }

    /// The milliseconds since the current card started.
    pub fn card_ms(&self) -> u64 {
        self.card_ms
    }

    /// The characters of the current card's text.
    pub fn total(&self) -> usize {
        self.card().map_or(0, |c| c.text.chars().count())
    }

    /// The characters shown (V2): none before typing, the typed ones while typing, all after.
    pub fn shown(&self) -> usize {
        match self.phase {
            Phase::FadeIn | Phase::Done => 0,
            Phase::Typing => match self.speed.cps() {
                Some(cps) => ((self.t as u64 * cps as u64 / 1000) as usize).min(self.total()),
                None => self.total(),
            },
            Phase::Hold | Phase::FadeOut => self.total(),
        }
    }

    /// A hold that waits for the player (the "Click to continue" hint shows).
    pub fn waiting(&self) -> bool {
        self.phase == Phase::Hold && self.card().is_some_and(|c| c.hold_ms == 0)
    }

    /// The cover's alpha over the stage (V6): 255 x (1 - t / fade) in fade in, 255 x t / fade in fade out, full once
    /// done, else none.
    pub fn cover_alpha(&self) -> u8 {
        let part = |t: u32, len: u32| if len == 0 { 1.0 } else { (t as f64 / len as f64).clamp(0.0, 1.0) };
        match (self.phase, self.phase_len()) {
            (Phase::FadeIn, Some(len)) => (255.0 * (1.0 - part(self.t, len))).round() as u8,
            (Phase::FadeOut, Some(len)) => (255.0 * part(self.t, len)).round() as u8,
            (Phase::Done, _) => 255,
            _ => 0,
        }
    }

    /// The cues of the cards started since the last call (V7: each card's once, as it enters its fade in).
    pub fn take_cues(&mut self) -> Vec<String> {
        std::mem::take(&mut self.cues)
    }

    /// The shake presets and limits cards play with (FX1 5.2; the game's tuning table).
    pub fn with_shakes(mut self, table: ShakeTable, limits: ShakeLimits) -> Player {
        self.shakes = table;
        self.shake_limits = limits;
        self
    }

    /// The current card's camera (FX1 5.2): its `shake` preset, clamped to `shake_limits`, started as the card entered
    /// its fade in (card ms 0, the moment its cue plays) and seeded by the card's index. None runs once the reel is done
    /// or skipped, for "" or an unknown preset, or under `reduce_motion`.
    pub fn camera(&self, reduce_motion: bool) -> Camera {
        let mut cam = Camera::default();
        if let Some(shake) = self.card().filter(|_| !self.done()).and_then(|c| self.shakes.get(&c.shake)) {
            cam.start(shake, 0, self.card as u64 + 1, self.shake_limits, reduce_motion);
        }
        cam
    }

    /// The view offset the card's shake gives now, in whole logical px ((0, 0) when none runs).
    pub fn shake_offset(&self, reduce_motion: bool) -> (i32, i32) {
        self.camera(reduce_motion).offset_at(self.card_ms)
    }

    /// The agent link's view (V13).
    pub fn state(&self) -> Value {
        json!({"reel": self.reel.id, "card": self.card, "cards": self.reel.cards.len(), "phase": self.phase.name(), "shown": self.shown(), "total": self.total(), "done": self.done()})
    }
}

// ---- Drawing ----------------------------------------------------------------------------------------------------

/// The card colours (V10): the game passes its own; the defaults are the kit's neutral ones.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub text: Rgba,
    pub accent: Rgba,
    pub dim: Rgba,
}

impl Default for Palette {
    fn default() -> Self {
        Palette { text: TEXT, accent: GOLD, dim: DIM }
    }
}

/// The text wrapped once, in full, at `max_w`, with how many characters of each line `shown` reveals (V2: a word
/// never jumps lines while it types). The space a line break takes counts as typed with the line before it.
pub fn typed_lines(p: &mut Painter, text: &str, shown: usize, size: f32, max_w: f32) -> Vec<(String, usize)> {
    let lines = p.wrap(text, size, max_w);
    let chars: Vec<char> = text.chars().collect();
    let mut pos = 0usize;
    let mut out = Vec::new();
    for line in lines {
        // the space the wrap dropped at a break
        while pos < chars.len() && chars[pos] == ' ' && !line.starts_with(' ') {
            pos += 1;
        }
        let n = line.chars().count();
        let visible = shown.saturating_sub(pos).min(n);
        pos += n;
        out.push((line, visible));
    }
    out
}

fn prefix(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn lerp(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    rgb(m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]))
}

fn with_alpha(c: Rgba, a: u8) -> Rgba {
    rgba(c[0], c[1], c[2], a)
}

/// The backdrop: a vertical gradient from `top` to `bottom` over `r`.
fn backdrop(p: &mut Painter, r: Rect, top: Rgba, bottom: Rgba) {
    if top == bottom {
        p.fill(r, top);
        return;
    }
    for y in 0..r.h {
        p.fill(Rect::new(r.x, r.y + y, r.w, 1), lerp(top, bottom, y as f32 / (r.h - 1).max(1) as f32));
    }
}

/// A small deterministic hash (splitmix64) for the ash specks.
fn mixed(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Ash (V11): `ASH_SPECKS` specks seeded by the card index, falling `ASH_FALL` px a second, wrapping inside `r`.
fn ash(p: &mut Painter, r: Rect, card: usize, card_ms: u64) {
    let fall = card_ms as f32 / 1000.0 * ASH_FALL;
    for i in 0..ASH_SPECKS as u64 {
        let h = mixed(((card as u64) << 8) | i);
        let x = r.x + (h % r.w.max(1) as u64) as i32;
        let y0 = ((h >> 16) % r.h.max(1) as u64) as f32;
        let y = r.y + ((y0 + fall) % r.h.max(1) as f32) as i32;
        let size = 1 + ((h >> 32) % 2) as i32;
        let a = 90 + ((h >> 40) % 90) as u8;
        p.fill(Rect::new(x, y, size, size), rgba(206, 204, 200, a));
    }
}

/// A caption box at `x`, `w` wide, holding `head` (the speaker's name, when any) over the typed text; it starts at
/// `CAPTION_Y` and moves up when it would reach the hint's room. Returns its rect.
#[allow(clippy::too_many_arguments)]
fn caption<A: Clone>(ui: &mut Ui<A>, x: i32, w: i32, min_h: i32, head: &str, text: &str, shown: usize, frame: Rgba, pal: &Palette) -> Rect {
    let lh = ui.p.line_h(CARD_BODY);
    let lines = typed_lines(&mut ui.p, text, shown, CARD_BODY, (w - 2 * CAPTION_PAD) as f32);
    let rows = lines.len() + usize::from(!head.is_empty());
    let h = ((rows as f32 * lh).ceil() as i32 + 2 * CAPTION_PAD).max(min_h);
    let y = CAPTION_Y.min(H - HINT_ROOM - h);
    let r = Rect::new(x, y, w, h);
    ui.p.panel(r, with_alpha(lerp(frame, rgb(0, 0, 0), 0.3), 225), with_alpha(pal.dim, 160), RADIUS);
    let (tx, mut ty) = ((x + CAPTION_PAD) as f32, (y + CAPTION_PAD) as f32);
    if !head.is_empty() {
        ui.p.text(tx, ty, head, CARD_BODY, pal.accent);
        ty += lh;
    }
    for (line, n) in &lines {
        if *n > 0 {
            ui.p.text(tx, ty, &prefix(line, *n), CARD_BODY, pal.text);
        }
        ty += lh;
    }
    r
}

/// Draws the player's card on the whole stage (V9): the frame colour, the backdrop or the picture, the ash (unless
/// `reduce_motion`), the card's texts, the hint in a hold that waits, and the fade cover. Widgets are the caller's.
/// While the card's shake runs (FX1 5.2) the picture area, the caption box, the title, line and text-card texts and the
/// portrait move by its offset; the frame, the stage backdrop gradient, the hint and the fade cover stay put.
pub fn draw<A: Clone>(ui: &mut Ui<A>, art: &mut Art, pl: &Player, pal: &Palette, reduce_motion: bool) {
    let stage = Rect::new(0, 0, W, H);
    let frame = pl.reel.frame;
    ui.p.fill(stage, frame);
    let Some(c) = pl.card() else { return };
    let (dx, dy) = pl.shake_offset(reduce_motion);
    let view = (ui.p.v.ox, ui.p.v.oy);
    // The picture area shakes with the card; the stage's backdrop does not.
    if c.style == Style::Image {
        ui.p.shift(dx, dy);
    }
    // The backdrop, or the picture letterboxed in the frame colour.
    let area = if c.style == Style::Image { PICTURE } else { stage };
    let picture = (c.style == Style::Image && !c.image.is_empty()).then(|| art.size_of(&c.image)).flatten().filter(|s| s.0 > 0 && s.1 > 0);
    match picture {
        Some((iw, ih)) => {
            let k = (area.w as f32 / iw as f32).min(area.h as f32 / ih as f32);
            let (w, h) = ((iw as f32 * k).round() as i32, (ih as f32 * k).round() as i32);
            let r = Rect::new(area.x + (area.w - w) / 2, area.y + (area.h - h) / 2, w, h);
            let s = ui.p.v.s;
            let (pw, ph) = (((w as f32 * s).round() as i32).clamp(1, iw), ((h as f32 * s).round() as i32).clamp(1, ih));
            match art.image(&c.image, pw, ph) {
                Some(img) => ui.p.image(img, r),
                None => backdrop(&mut ui.p, area, c.top, c.bottom),
            }
        }
        None => backdrop(&mut ui.p, area, c.top, c.bottom),
    }
    if c.ash && !reduce_motion {
        ash(&mut ui.p, area, pl.card_index(), pl.card_ms());
    }
    if c.style != Style::Image {
        ui.p.shift(dx, dy);
    }
    let shown = pl.shown();
    match c.style {
        Style::Image => {
            if !c.text.is_empty() {
                caption(ui, CAPTION_X, CAPTION_W, 0, "", &c.text, shown, frame, pal);
            }
        }
        Style::Text => {
            let lh = ui.p.line_h(CARD_BODY);
            let lines = typed_lines(&mut ui.p, &c.text, shown, CARD_BODY, TEXT_BOX.w as f32);
            let mid = TEXT_BOX.y as f32 + TEXT_BOX.h as f32 / 2.0;
            let mut y = mid - lines.len() as f32 * lh / 2.0;
            for (line, n) in &lines {
                // placed by the full line's width, so a line does not slide while it types
                let x = W as f32 / 2.0 - ui.p.text_w(line, CARD_BODY) / 2.0;
                if *n > 0 {
                    ui.p.text(x, y, &prefix(line, *n), CARD_BODY, pal.text);
                }
                y += lh;
            }
        }
        Style::Title => {
            let big = CARD_BODY * 3.0;
            let (lt, ls) = (ui.p.line_h(big), ui.p.line_h(CARD_BODY));
            let titles = ui.p.wrap(&c.title, big, (W - 40) as f32);
            let subs = if c.subtitle.is_empty() { Vec::new() } else { ui.p.wrap(&c.subtitle, CARD_BODY, CAPTION_W as f32) };
            let top = TITLE_Y - titles.len() as f32 * lt / 2.0;
            let rule = |p: &mut Painter, y: f32| p.fill(Rect::new(W / 2 - RULE_W / 2, y.round() as i32, RULE_W, 1), pal.accent);
            rule(&mut ui.p, top - 8.0);
            let mut y = top;
            for l in &titles {
                ui.p.text_center(W as f32 / 2.0, y, l, big, pal.accent);
                y += lt;
            }
            y += 4.0;
            for l in &subs {
                ui.p.text_center(W as f32 / 2.0, y, l, CARD_BODY, pal.text);
                y += ls;
            }
            rule(&mut ui.p, y + 6.0);
        }
        Style::Line => {
            let x = CAPTION_X + PORTRAIT + 8;
            let r = caption(ui, x, CAPTION_X + CAPTION_W - x, PORTRAIT, &c.speaker, &c.text, shown, frame, pal);
            let face = Rect::new(CAPTION_X, r.y, PORTRAIT, PORTRAIT);
            ui.p.panel(face, with_alpha(lerp(frame, rgb(0, 0, 0), 0.3), 225), with_alpha(pal.dim, 160), RADIUS);
            let px = (PORTRAIT as f32 * ui.p.v.s).round() as i32;
            if let Some(img) = art.first(&c.portrait, px, px) {
                ui.p.image(img, face.inset(2));
            }
        }
    }
    // Back to the still view (restored, not shifted back, so no float drift).
    (ui.p.v.ox, ui.p.v.oy) = view;
    if pl.waiting() {
        ui.p.text_right((W - 12) as f32, (H - 14) as f32, HINT, SMALL, pal.dim);
    }
    let a = pl.cover_alpha();
    if a > 0 {
        ui.p.fill(stage, with_alpha(frame, a));
    }
}

// ---- The screen -------------------------------------------------------------------------------------------------

/// How a reel left the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    Played,
    Skipped,
}

/// What runs when a reel leaves the screen.
pub type OnDone<G> = Box<dyn FnOnce(&mut Cx<G>, Ended)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAct {
    Next,
    Skip,
}

/// The vignette screen (V15): a full screen playing one reel; `on_done` runs once, when the reel ends or is skipped,
/// and queues what comes next (usually `Nav::Pop`); the screen changes no game state itself.
pub struct VignetteScreen<G> {
    pub player: Player,
    pub palette: Palette,
    /// Skip asks "Skip this scene?" first (Yes the default); the caller sets it for a scene not yet seen.
    pub ask_first: bool,
    on_done: Option<OnDone<G>>,
    last: Option<Instant>,
    w: Vec<Widget<VAct>>,
}

impl<G: Context> VignetteScreen<G> {
    pub fn new(reel: Reel, speed: TextSpeed, on_done: impl FnOnce(&mut Cx<G>, Ended) + 'static) -> Self {
        Self { player: Player::new(reel, speed), palette: Palette::default(), ask_first: false, on_done: Some(Box::new(on_done)), last: None, w: Vec::new() }
    }

    /// The reel at the player's text speed (the settings' `text_speed`).
    pub fn with_settings(settings: &Settings, reel: Reel, on_done: impl FnOnce(&mut Cx<G>, Ended) + 'static) -> Self {
        Self::new(reel, TextSpeed::of(settings), on_done)
    }

    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    pub fn ask_first(mut self, ask: bool) -> Self {
        self.ask_first = ask;
        self
    }

    /// The game's shake presets and limits for the cards' `shake` (FX1 5.2; default the engine's table and ceiling).
    pub fn with_shakes(mut self, table: ShakeTable, limits: ShakeLimits) -> Self {
        self.player.shakes = table;
        self.player.shake_limits = limits;
        self
    }

    /// Plays the cues of the cards just started, and runs `on_done` once the reel is over.
    fn settle(&mut self, cx: &mut Cx<G>) {
        for cue in self.player.take_cues() {
            cx.audio.play(Bus::Sfx, &cue);
        }
        if self.player.done() {
            if let Some(f) = self.on_done.take() {
                f(cx, if self.player.skipped() { Ended::Skipped } else { Ended::Played });
            }
        }
    }

    /// `ms` of play, as a frame would feed it (for tools and tests that run the screen without a clock).
    pub fn advance(&mut self, cx: &mut Cx<G>, ms: u32) {
        self.player.advance(ms);
        self.settle(cx);
    }

    fn press(&mut self, cx: &mut Cx<G>) {
        self.player.press();
        self.settle(cx);
    }

    /// Skip asked for (V5): refused when the reel is not skippable, asked first when `ask_first`.
    fn want_skip(&mut self, cx: &mut Cx<G>) {
        if !self.player.reel.skippable || self.player.done() {
            return;
        }
        if self.ask_first {
            cx.ask_tag(SKIP_QUESTION, SKIP_TAG);
        } else {
            self.player.skip();
            self.settle(cx);
        }
    }

    /// A Yes to the question skips.
    fn take_answer(&mut self, cx: &mut Cx<G>) {
        if cx.take_answer(SKIP_TAG) {
            self.player.skip();
            self.settle(cx);
        }
    }

    fn act(&mut self, cx: &mut Cx<G>, a: VAct) {
        match a {
            VAct::Next => self.press(cx),
            VAct::Skip => self.want_skip(cx),
        }
    }
}

impl<G: Context> Screen<G> for VignetteScreen<G> {
    fn id(&self) -> &'static str {
        "vignette"
    }

    fn draw(&mut self, cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<VAct> = Ui::new(g, mouse, live);
        let still = reduce_motion(&cx.settings);
        draw(&mut ui, &mut cx.art, &self.player, &self.palette, still);
        let done = self.player.done();
        ui.push("vignette.next", "stage", "", Rect::new(0, 0, W, H), !done, false, Some(VAct::Next), tip("Continue", "A click, Space or Enter moves the scene on."));
        if self.player.reel.skippable && !done {
            let w = (ui.p.text_w("Skip", SMALL) + 14.0).ceil() as i32;
            ui.button_full("vignette.skip", Rect::new(W - w - 6, 6, w, 13), "Skip", VAct::Skip, true, false, SMALL, tip("Skip", "Esc or a right click skips the scene."));
        }
        self.w = ui.widgets;
    }

    fn arrow_focus(&self) -> bool {
        false
    }

    fn key(&mut self, cx: &mut Cx<G>, k: Key) {
        if matches!(k, Key::Space | Key::Enter) {
            self.press(cx);
        }
    }

    /// Right click and Esc: skip.
    fn back(&mut self, cx: &mut Cx<G>) {
        self.want_skip(cx);
    }

    /// The frame time (at most `MAX_FRAME_MS`) moves the reel.
    fn tick(&mut self, cx: &mut Cx<G>) {
        self.take_answer(cx);
        let now = Instant::now();
        let ms = self.last.map_or(0, |l| now.duration_since(l).as_millis().min(MAX_FRAME_MS as u128) as u32);
        self.last = Some(now);
        self.advance(cx, ms);
    }

    /// Back from the question: its answer, and no jump for the time it was open.
    fn resume(&mut self, cx: &mut Cx<G>) {
        self.last = None;
        self.take_answer(cx);
    }

    fn state(&self, cx: &Cx<G>) -> Value {
        let mut v = self.player.state();
        let (dx, dy) = self.player.shake_offset(reduce_motion(&cx.settings));
        v["shake"] = json!([dx, dy]);
        v["skippable"] = json!(self.player.reel.skippable);
        v["ask_first"] = json!(self.ask_first);
        v
    }

    fn music(&self, _cx: &Cx<G>) -> Option<String> {
        self.player.reel.music_wish()
    }

    crate::widget_plumbing!(Cx<G>);
}
