//! The kit's widgets on one page (UIGALLERY1): every widget family of `ui.rs` in its states, on sample values, to be
//! judged on its own (a UI Gallery's "Kit widgets" entry). Game-agnostic: the gallery screen keeps a `Specimens`,
//! wraps `KitAct` in its own action and forwards drops and text changes to it. Pages, picked by the tabs at the
//! page's foot: the widgets, the block timeline with its segment parts, links and marks (RULESET1-E), and the effects
//! kit (FX1-E: each primitive and shake preset on a small stage, the clock's speed and hit-stop, a local reduce
//! motion; STATUSFX1-E: the orbit and a loop of the chosen primitive, its still look under reduce motion); the Effects
//! page runs on its own clock, fed by `tick` (wall time) or `advance` (tests). PORTRAITFX1-E: a
//! fourth page, Portraits, draws the engine's code-drawn sample face (`portrait_fx::sample`) through a `PortraitFx` at a
//! chosen size, with toggles for each overlay and state, buttons for the reactions, a local reduce motion and the
//! anchors check, on its own clock. PORTRAITFX1-TL: the Timeline page's Icons check (`kit.timeline.icons`) gives every
//! track its icon square (`Track.icon`), drawn by the page through `Timeline::draw_icons`. LOOK1-E1: a fifth page,
//! Theme, shows the theme in force (`theme::th()`): every colour role's swatch, name and value, the LOOK1 8.2 contrast
//! pairs with pass or fail, the three faces at the four text sizes, the header, badge and inset pieces, every frame
//! role at 40 x 16 and 120 x 40 and the picked one at 300 x 120, and a switch between the engine default, the layers'
//! theme (`theme::loaded()`) and that theme with frames off (it sets the theme in force for the whole window until the
//! page or the gallery is left: `leave_theme`). LOOK1-ME: a sixth page, Panes, shows each engine pane (`panes`) on
//! its own stage, picked by tabs: the backdrop (vignette and veil switches), the menu column under its title block,
//! the tab rail, three slot cards (a thumbnail, an initial, a long name cut; lock and delete) and the themed ask
//! (ornate or panel, either default); `Specimens::key` takes the page's keys. LOOK1-E2: a seventh page, Tiles, draws a
//! fixture map (`TILES_MAP`: a pond with an island, a wall run, a lone water cell, a 1-wide channel) from a set picked
//! among the asset layers' `tilesets/` (`theme::layers`), square or tilted at 1x-3x (its Ground switch, REGIONGROUND1-E1:
//! the same map through `ground::compose`), with the set's transition pieces
//! (`autotile`) when it has them, else the mask numbers. LOOK1-FT: an eighth page, Frames, the frame tuner (`frame_tuner`).

use crate::autotile::{block_edges, corner_pieces};
use crate::canvas::{rgb, rgba, Image, Rect, Rgba};
use crate::frame_tuner::{self, FrameTuner, TunerAct};
use crate::ground::{self, GroundJob, GroundKey, GroundLayer};
use crate::fx::{Beam, Camera, Clock, Effect, Flash, Fx, Layer, Look, Looped, Orbit, Projectile, Ring, ShakeLimits, ShakeTable, Sparks, Trail, PRIM_KINDS, SHAKE_PRESETS};
use crate::portrait_fx::{self, Budget, Cue, ExpressionFx, Motion, PortraitFx, PortraitOpts, Tier, MOTION_SAMPLES};
use aegis_core::assets::{Face, Layers};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;
use crate::painter::{Painter, Typeface};
use crate::panes::{ask_dialog, ask_key, backdrop, backdrop_pane, menu_column, menu_column_styled, menu_key, menu_title, slot_card, tab_key, tab_rail, AskView, Corner, MenuItem, MenuStyle, SlotView};
use crate::screen::Key;
use crate::theme::{self, Theme, FRAME_ROLES, ROLES};
use crate::timeline::{Link, Mark, Part, PartStyle, Section, Segment, Timeline, Track};
use crate::ui::{tip, Glyph, Ui, BODY, DIM, FOE, GOLD, GOOD, INFO, SMALL, TEXT};

#[derive(Debug, Clone, PartialEq)]
pub enum KitAct {
    Press(&'static str),
    Tab(usize),
    Check,
    Step(i32),
    Pick(usize),
    Slide(usize),
    Card(usize),
    Row(usize),
    Close,
    /// The lock icon button (toggles), any other icon button (its name).
    Lock,
    Icon(&'static str),
    /// The collapsible section header.
    Fold,
    /// The page tabs at the foot.
    Page(usize),
    /// The timeline specimen: a block, a track, a segment (track, segment).
    Block(i32),
    Track(String),
    Segment(String, String),
    /// The timeline specimen's icons check (PORTRAITFX1-TL: every track's icon square, drawn by the page).
    TimelineIcons,
    /// The Effects page: a primitive (`fx::PRIM_KINDS`), a shake preset (`fx::SHAKE_PRESETS`), the clock speed
    /// (`FX_SPEEDS` index), the hit-stop toggle, the page's own reduce-motion check, the loop toggle (STATUSFX1-E).
    Effect(&'static str),
    Shake(&'static str),
    FxSpeed(usize),
    HitStop,
    Reduce,
    Loop,
    /// The Portraits page: an overlay or state toggle (`PORTRAIT_OVERLAYS`, `PORTRAIT_STATES`), a reaction
    /// (`PORTRAIT_REACTIONS`), the size (`PORTRAIT_SIZES` index), its reduce-motion and anchors checks.
    PortraitToggle(&'static str),
    PortraitFire(&'static str),
    PortraitSize(usize),
    PortraitReduce,
    PortraitAnchors,
    /// PORTRAITFX2-M: the motion profile (`portrait_fx::MOTION_SAMPLES` index).
    PortraitMotion(usize),
    /// LOOK1-E1: the Theme page's switch (`THEME_VIEWS` index) and the frame role shown large (`FRAME_ROLES` index).
    ThemeView(usize),
    ThemeFrame(usize),
    /// LOOK1-ME: the Panes page's picker (`PANES` index), its tab rail's tab (`PANE_TABS` index), a slot card, the
    /// slot's lock, and a pane switch (`vignette`, `veil`, `ornate`, `destructive`).
    Pane(usize),
    PaneTab(usize),
    Slot(usize),
    SlotLock,
    PaneToggle(&'static str),
    /// LOOK1-E2: the Tiles page's set (index in the layers' sorted `tilesets/`), view (`TILE_VIEWS` index), zoom
    /// (`TILE_ZOOMS` index) and a pan in cells (x, y).
    TileSet(usize),
    /// The set's transition pair shown (index in its sorted `<upper>/over_<lower>/` folders).
    TilePair(usize),
    TileView(usize),
    TileZoom(usize),
    TilePan(i32, i32),
    /// REGIONGROUND1-E1: the Tiles page's Ground switch: the fixture map through `ground::compose`.
    TileGround,
    /// LOOK1-FT: the Frames page's frame tuner.
    Tuner(TunerAct),
}

pub const TABS: [&str; 3] = ["First", "Second", "Third"];
pub const PAGES: [&str; 8] = ["Widgets", "Timeline", "Effects", "Portraits", "Theme", "Panes", "Tiles", "Frames"];
/// LOOK1-E2: the Tiles page's views (name, logical cell width and height at 1x) and zooms.
pub const TILE_VIEWS: [(&str, i32, i32); 2] = [("Square 16 x 16", 16, 16), ("Tilted 32 x 24", 32, 24)];
pub const TILE_ZOOMS: [i32; 3] = [1, 2, 3];
/// The Tiles page's tile kinds: the Gallery fixture's defaults, and the file-name convention of LOOK1 5.4
/// (`tilesets/<set>/<kind>/v<n>.png`, a block's `wall_face`). The engine's rules know no terrain: only this fixture
/// names these; a set's own `<upper>/over_<lower>/` pair replaces `upper` and `lower`.
pub const TILES_KINDS: TileKinds = TileKinds { upper: "floor", lower: "shallow_water", block: "wall", face: "wall_face" };
/// The fixture map's kinds: `.` is `upper`, `~` is `lower`, `#` is `block` with `face` over an open south.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileKinds {
    pub upper: &'static str,
    pub lower: &'static str,
    pub block: &'static str,
    pub face: &'static str,
}
/// The Tiles page's fixture map (engine test data): `.` ground, `#` a block, `~` the lower kind. A pond with a 1-cell
/// island, a wall run, a lone water cell, a 1-wide channel; its corners hold all 16 masks.
pub const TILES_MAP: [&str; 9] = [
    "............",
    ".~~~~~..###.",
    ".~~.~~....#.",
    ".~~~~~....#.",
    "......~.....",
    "............",
    ".~..........",
    "....~~~~~~~~",
    "...~........",
];
/// LOOK1-ME: the Panes page's panes, and its tab rail's sample tabs.
pub const PANES: [&str; 5] = ["Backdrop", "Menu column", "Tab rail", "Slot card", "Ask"];
pub const PANE_TABS: [&str; 4] = ["Video", "Audio", "Controls", "Text"];
/// The Theme page's switch: the engine default, the theme the asset layers gave, that theme with frames off.
pub const THEME_VIEWS: [&str; 3] = ["Engine default", "Layers' theme", "Frames off"];
/// The text sizes the Theme page shows each face at (the settings' four, `screen::TEXT_SIZES`).
pub const THEME_TEXT_SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];
/// The Portraits page: the sizes on offer (logical px), the toggles, the reactions and the stage (the largest size
/// plus the jolt ceiling and a margin on every side).
pub const PORTRAIT_SIZES: [i32; 6] = [96, 64, 48, 32, 24, 16];
pub const PORTRAIT_OVERLAYS: [&str; 5] = ["glow", "veins", "eye_glow", "ash", "smoke"];
pub const PORTRAIT_STATES: [&str; 4] = ["desaturate", "tint", "fade", "still"];
pub const PORTRAIT_REACTIONS: [&str; 3] = ["flash", "jolt.hit", "jolt.crit"];
pub const PORTRAIT_STAGE: i32 = 96 + 2 * 6;
/// The Effects page: its stage (dark, the dummies 160 px apart), the clock speeds and the hit-stop.
pub const FX_STAGE_W: i32 = 240;
pub const FX_STAGE_H: i32 = 120;
pub const FX_SPEEDS: [(&str, f64); 3] = [("0.5x", 0.5), ("1x", 1.0), ("2x", 2.0)];
pub const FX_HIT_STOP_MS: f64 = 40.0;
/// STATUSFX1-E: a looped sample repeats every `ms` of its effect plus this gap.
pub const FX_LOOP_GAP_MS: u64 = 200;
/// The dummies' chest points and their foot line, in stage px.
pub const FX_FROM: (f32, f32) = (40.0, 82.0);
pub const FX_TO: (f32, f32) = (200.0, 82.0);
pub const FX_FOOT_Y: f32 = 96.0;
/// Palette hues of the samples (steel, linen, ash, amber, gold, orchid) and the stage.
const STEEL: Rgba = rgb(156, 163, 175);
const LINEN: Rgba = rgb(217, 203, 180);
const ASH: Rgba = rgb(107, 114, 128);
const AMBER: Rgba = rgb(217, 119, 6);
const CANDLE: Rgba = rgb(251, 191, 36);
const ORCHID: Rgba = rgb(192, 132, 252);
const STAGE_BG: Rgba = rgb(14, 15, 20);
/// The Tiles page's flat colours for a kind the set does not hold, and its block lines.
const TILE_UPPER: Rgba = rgb(70, 68, 62);
const TILE_LOWER: Rgba = rgb(34, 52, 78);
const TILE_BLOCK: Rgba = rgb(96, 98, 106);
const TILE_FACE: Rgba = rgb(52, 53, 60);
const TILE_LINE: Rgba = rgb(14, 15, 20);
pub const OPTIONS: [&str; 4] = ["Small", "Normal", "Large", "Extra large"];
pub const SLIDER_STEPS: usize = 5;

/// The page's sample values; what the last widget used did is shown at the foot of the page.
#[derive(Debug, Clone)]
pub struct Specimens {
    pub tab: usize,
    pub checked: bool,
    pub value: i32,
    pub pick: usize,
    pub slide: usize,
    pub card: usize,
    pub row: usize,
    pub text: String,
    /// The text area's sample (the kit's text editor, PLAYUI1-T).
    pub area: String,
    /// The icon buttons' lock (on: the delete icon is disabled) and whether the section is open.
    pub locked: bool,
    pub open: bool,
    /// The drag and drop specimen: the chips in its two boxes.
    pub left: Vec<String>,
    pub right: Vec<String>,
    pub last: String,
    /// The page on view (`PAGES`), and the timeline specimen's selected block and track.
    pub page: usize,
    pub block: Option<i32>,
    pub track: Option<String>,
    /// The timeline specimen's icon squares on (`Track.icon`, PORTRAITFX1-TL).
    pub icons: bool,
    /// The Effects page's state.
    pub fx: FxDemo,
    /// The Portraits page's state.
    pub portrait: PortraitDemo,
    /// The Theme page's switch (`THEME_VIEWS`) and its large frame role (`FRAME_ROLES` index).
    pub theme_view: usize,
    pub theme_frame: usize,
    /// The Panes page (LOOK1-ME): the pane on view (`PANES`), the tab rail's open tab, the selected slot card, the
    /// first slot's lock, and the switches.
    pub pane: usize,
    pub pane_tab: usize,
    pub slot: usize,
    pub slot_locked: bool,
    pub vignette: bool,
    pub veil: bool,
    pub ask_ornate: bool,
    /// LOOK1-FT: the Frames page.
    pub tuner: FrameTuner,
    pub ask_no_default: bool,
    /// The Tiles page's state (LOOK1-E2).
    pub tiles: TileDemo,
}

/// The cell of `TILES_MAP` at (x, y), an off-grid cell taking its nearest cell's kind (so the map's rim draws no edge).
pub fn tiles_map_cell(x: i32, y: i32) -> u8 {
    let row = TILES_MAP[y.clamp(0, TILES_MAP.len() as i32 - 1) as usize].as_bytes();
    row[x.clamp(0, row.len() as i32 - 1) as usize]
}

/// The Tiles page's choices, and what it read from the asset layers (behind a `RefCell`: the page draws through
/// `&self`).
#[derive(Debug, Clone, Default)]
pub struct TileDemo {
    /// Index in the layers' sorted sets, in the set's sorted pairs, `TILE_VIEWS` index, `TILE_ZOOMS` index, and the
    /// pan in cells.
    pub set: usize,
    pub pair: usize,
    pub view: usize,
    pub zoom: usize,
    pub pan: (i32, i32),
    /// The Ground switch (REGIONGROUND1-E1): the fixture map composed, no tile set needed.
    pub ground: bool,
    cache: RefCell<TileCache>,
}

/// What the Tiles page read from the layers `from`; a new `theme::reload` starts it over.
#[derive(Debug, Clone, Default)]
struct TileCache {
    from: Option<Rc<Layers>>,
    sets: Vec<String>,
    /// Set -> its transition pairs (upper, lower): every `<upper>/over_<lower>/` folder, sorted.
    pairs: HashMap<String, Vec<(String, String)>>,
    /// Layer-relative file -> its image.
    images: HashMap<String, Option<Image>>,
    /// The Ground switch's fixture textures by art px per cell (no file needed).
    ground_tex: HashMap<(i32, i32), [Image; 3]>,
    /// Their jobs' keys, made once (`GroundKey::of` hashes every texture pixel).
    ground_keys: HashMap<(i32, i32), GroundKey>,
}

impl TileCache {
    /// Starts over when the layers in force are not the ones read.
    fn sync(&mut self) {
        let now = theme::layers();
        if !self.from.as_ref().is_some_and(|f| Rc::ptr_eq(f, &now)) {
            *self = TileCache { sets: now.dirs("tilesets"), from: Some(now), ..TileCache::default() };
        }
    }

    fn pairs(&mut self, set: &str) -> Vec<(String, String)> {
        let Some(layers) = self.from.clone() else { return Vec::new() };
        self.pairs
            .entry(set.to_string())
            .or_insert_with(|| {
                let mut out = Vec::new();
                for upper in layers.dirs(&format!("tilesets/{set}")) {
                    for d in layers.dirs(&format!("tilesets/{set}/{upper}")) {
                        if let Some(lower) = d.strip_prefix("over_") {
                            out.push((upper.clone(), lower.to_string()));
                        }
                    }
                }
                out
            })
            .clone()
    }

    fn image(&mut self, rel: &str) -> Option<&Image> {
        let layers = self.from.clone()?;
        self.images.entry(rel.to_string()).or_insert_with(|| layers.find(rel).and_then(|p| crate::art::load_png(&p))).as_ref()
    }

    /// The first variant of tile `kind` in `set`, from the top layer holding it.
    fn tile(&mut self, set: &str, kind: &str) -> Option<&Image> {
        self.image(&format!("tilesets/{set}/{kind}/v0.png"))
    }
}

/// The Portraits page's clock, portrait (its cache behind a `RefCell`: the page draws through `&self`), sample face
/// and choices.
#[derive(Debug, Clone)]
pub struct PortraitDemo {
    pub clock: Clock,
    pub fx: RefCell<PortraitFx>,
    budget: RefCell<Budget>,
    /// The overlays and states toggled on.
    pub on: BTreeSet<&'static str>,
    /// `PORTRAIT_SIZES` index.
    pub size: usize,
    pub reduce: bool,
    pub anchors: bool,
    /// PORTRAITFX2-M: the motion profile shown (`MOTION_SAMPLES` index; 0, the breath, holds no face).
    pub motion: usize,
    pub image: Image,
    pub face: Face,
    last_wall: Option<aegis_core::clock::Instant>,
}

impl Default for PortraitDemo {
    fn default() -> Self {
        let (image, face) = portrait_fx::sample();
        let mut fx = PortraitFx::new("kit.sample");
        // one face per sample profile (named after it), so choosing one starts its entry and blends from the last
        let rows: Vec<ExpressionFx> = MOTION_SAMPLES.iter().filter_map(|n| Motion::sample(n).map(|m| ExpressionFx { motion: m, ..ExpressionFx::new(n) })).collect();
        fx.set_expressions(&rows);
        Self { clock: Clock::default(), fx: RefCell::new(fx), budget: RefCell::new(Budget::default()), on: BTreeSet::new(), size: 0, reduce: false, anchors: false, motion: 0, image, face, last_wall: None }
    }
}

impl PortraitDemo {
    /// Toggles overlay or state `e` and holds the new set; what it did.
    fn toggle(&mut self, e: &'static str) -> String {
        let on = !self.on.remove(e);
        if on {
            self.on.insert(e);
        }
        self.hold();
        format!("portrait {} {}", e, if on { "on" } else { "off" })
    }

    /// Holds the toggled set and the chosen motion's face.
    fn hold(&mut self) {
        let mut cues: Vec<Cue> = self.on.iter().filter_map(|x| Cue::parse(x, "").map(|c| c.with_key(&format!("kit.{x}")))).collect();
        let name = MOTION_SAMPLES[self.motion % MOTION_SAMPLES.len()];
        if self.motion > 0 {
            cues.extend(Cue::parse("face", "").map(|c| c.with_expression(name).with_key("kit.motion")));
        }
        self.fx.borrow_mut().hold(&cues, self.clock.now());
    }

    /// PORTRAITFX2-M: shows motion profile `i`; what it did.
    fn set_motion(&mut self, i: usize) -> String {
        self.motion = i % MOTION_SAMPLES.len();
        self.hold();
        format!("portrait motion {}", MOTION_SAMPLES[self.motion])
    }

    /// Fires reaction `what` (`flash`, `jolt.hit`, `jolt.crit`); what it did.
    fn fire(&mut self, what: &str) -> String {
        let cue = match what {
            "flash" => Cue::parse("flash", ""),
            _ => Cue::parse("jolt", what.trim_start_matches("jolt.")),
        };
        let Some(cue) = cue else { return format!("no reaction {what}") };
        self.fx.borrow_mut().fire(cue, self.clock.now());
        let still = self.reduce && what.starts_with("jolt");
        format!("portrait {what}{}", if still { ": no motion (reduce motion)" } else { "" })
    }

    fn opts(&self) -> PortraitOpts {
        PortraitOpts { reduce_motion: self.reduce, rim_default: None, show_anchors: self.anchors, tier_cap: None }
    }

    /// The portrait's agent state at the page's clock (`PortraitFx::state`).
    pub fn state(&self) -> serde_json::Value {
        self.fx.borrow().state(self.clock.now())
    }
}

/// The Effects page's clock, live list, camera and choices; impacts to come fire their sparks, flash and hit-stop
/// when the clock reaches them.
#[derive(Debug, Clone)]
pub struct FxDemo {
    pub clock: Clock,
    pub fx: Fx,
    pub camera: Camera,
    pub shakes: ShakeTable,
    /// `FX_SPEEDS` index.
    pub speed: usize,
    pub hit_stop: bool,
    pub reduce: bool,
    /// (clock ms, seed) of the impacts to come.
    pub impacts: Vec<(u64, u64)>,
    /// Seeds handed out so far (each press gets the next).
    pub seeds: u64,
    /// STATUSFX1-E: the primitive last pressed (`PRIM_KINDS`), whether it loops, and the loop's start and seed.
    pub chosen: &'static str,
    pub looping: bool,
    pub loop_start: u64,
    pub loop_seed: u64,
    last_wall: Option<aegis_core::clock::Instant>,
}

impl Default for FxDemo {
    fn default() -> Self {
        Self { clock: Clock::default(), fx: Fx::default(), camera: Camera::default(), shakes: ShakeTable::default(), speed: 1, hit_stop: true, reduce: false, impacts: Vec::new(), seeds: 0, chosen: "orbit", looping: false, loop_start: 0, loop_seed: 0, last_wall: None }
    }
}

impl FxDemo {
    fn seed(&mut self) -> u64 {
        self.seeds += 1;
        self.seeds
    }

    /// Plays primitive `kind` from the left dummy to the right one (while looping: loops it instead, from now); what it
    /// did.
    fn play(&mut self, kind: &'static str) -> String {
        let now = self.clock.now();
        let seed = self.seed();
        self.chosen = kind;
        if self.looping {
            self.loop_start = now;
            self.loop_seed = seed;
            return format!("{kind}: looped every {} ms", self.looped().period);
        }
        let (e, impact) = fx_sample(kind, now);
        let ms = e.ms();
        self.fx.push(e.seeded(seed));
        if let Some(t) = impact {
            self.impacts.push((t, seed));
        }
        format!("{kind}: {ms} ms{}", if impact.is_some() { ", then the impact" } else { "" })
    }

    /// Turns the loop of the chosen primitive on or off (on: it starts now).
    fn toggle_loop(&mut self) {
        self.looping = !self.looping;
        if self.looping {
            self.loop_start = self.clock.now();
            self.loop_seed = self.seed();
        }
    }

    /// The chosen primitive looped (period its `ms` + `FX_LOOP_GAP_MS`), from the loop's start, in stage px.
    pub fn looped(&self) -> Looped {
        let e = fx_sample(self.chosen, self.loop_start).0.seeded(self.loop_seed);
        Looped::new(e, e.ms() + FX_LOOP_GAP_MS)
    }
}

/// The page's sample of primitive `kind` starting at `now`, in stage px, and when its impact lands (if it has one).
fn fx_sample(kind: &str, now: u64) -> (Effect, Option<u64>) {
    let from = (FX_FROM.0 + 6.0, FX_FROM.1);
    match kind {
        "projectile" => {
            let travel = (((FX_TO.0 - from.0) / 560.0) * 1000.0).round() as u32;
            (Effect::new(Projectile { from, to: FX_TO, travel_ms: travel, arc_px: 6.0, look: Look::Arrow { len: 7.0, shaft: LINEN, head: STEEL } }, now), Some(now + travel as u64))
        }
        "beam" => (Effect::new(Beam { from, to: FX_TO, ms: 160, width: 2, core: LINEN, glow: ORCHID }, now), Some(now)),
        "trail" => (Effect::new(Trail { at: (FX_TO.0 - 14.0, FX_TO.1), radius: 13.0, from_deg: -60.0, to_deg: 50.0, ms: 150, width: 2, col: STEEL, mirror: false, squash: 0.75 }, now), Some(now + 90)),
        "sparks" => (Effect::new(Sparks { at: FX_TO, ms: 260, count: 10, dir_deg: -90.0, spread_deg: 160.0, speed: 70.0, gravity: 120.0, size: 1, cols: [AMBER, CANDLE] }, now), None),
        "flash" => (Effect::new(Flash { at: FX_TO, rx: 8.0, ry: 6.0, ms: 140, col: rgba(CANDLE[0], CANDLE[1], CANDLE[2], 160) }, now), None),
        "orbit" => (Effect::new(Orbit { at: (FX_TO.0, FX_FOOT_Y - 33.0), rx: 7.0, ry: 2.5, ms: 900, count: 3, size: 2, cols: [CANDLE, LINEN], phase_deg: 0.0, star: true }, now), None),
        _ => (Effect::new(Ring { at: (FX_TO.0, FX_FOOT_Y), r0: 3.0, r1: 12.0, ms: 260, col: rgba(ASH[0], ASH[1], ASH[2], 200), squash: 0.75 }, now).on(Layer::Ground), None),
    }
}

impl FxDemo {
    /// Starts preset `name` at the engine ceiling; what it did.
    fn shake(&mut self, name: &str) -> String {
        let Some(s) = self.shakes.get(name) else { return format!("no preset {name}") };
        let (now, seed) = (self.clock.now(), self.seed());
        if self.camera.start(s, now, seed, ShakeLimits::CEILING, self.reduce) {
            format!("shake {name}: {} px, {} ms, {}, {}", s.px, s.ms, s.axis.name(), s.envelope.name())
        } else if self.reduce {
            format!("shake {name}: none (reduce motion)")
        } else {
            format!("shake {name}: a stronger shake runs")
        }
    }

    /// `wall_ms` of wall time at the chosen speed: due impacts fire (their sparks and flash at the right dummy and,
    /// when on, the hit-stop), done effects and shakes end.
    pub fn advance(&mut self, wall_ms: f64) {
        self.clock.advance(wall_ms, FX_SPEEDS[self.speed.min(FX_SPEEDS.len() - 1)].1);
        let now = self.clock.now();
        let due: Vec<(u64, u64)> = self.impacts.iter().copied().filter(|(t, _)| *t <= now).collect();
        self.impacts.retain(|(t, _)| *t > now);
        for (t, seed) in due {
            self.fx.push(Effect::new(Sparks { at: FX_TO, ms: 260, count: 6, dir_deg: 0.0, spread_deg: 90.0, speed: 70.0, gravity: 120.0, size: 1, cols: [STEEL, LINEN] }, t).seeded(seed));
            self.fx.push(Effect::new(Flash { at: FX_TO, rx: 5.0, ry: 4.0, ms: 100, col: rgba(255, 255, 255, 150) }, t));
            if self.hit_stop {
                self.clock.hold(FX_HIT_STOP_MS);
            }
        }
        self.fx.prune(now);
        self.camera.offset(now);
    }
}

impl Default for Specimens {
    fn default() -> Self {
        Self {
            tab: 0,
            checked: true,
            value: 3,
            pick: 1,
            slide: 2,
            card: 0,
            row: 1,
            text: "Type here".into(),
            area: "A text area wraps its lines and scrolls. Shift with the arrows selects, so does a drag; a double click takes a word.\nCtrl with Left or Right jumps by word; Ctrl+Z undoes, Ctrl+Y redoes.".into(),
            locked: false,
            open: true,
            left: vec!["Sword".into(), "Shield".into()],
            right: vec!["Potion".into()],
            last: "nothing yet".into(),
            page: 0,
            block: None,
            track: None,
            icons: false,
            fx: FxDemo::default(),
            portrait: PortraitDemo::default(),
            theme_view: 1,
            theme_frame: 0,
            pane: 0,
            pane_tab: 0,
            slot: 0,
            slot_locked: false,
            vignette: true,
            veil: true,
            ask_ornate: true,
            tuner: FrameTuner::default(),
            ask_no_default: false,
            tiles: TileDemo::default(),
        }
    }
}

impl Specimens {
    pub fn act(&mut self, a: KitAct) {
        self.last = match &a {
            KitAct::Press(what) => format!("pressed {what}"),
            KitAct::Tab(i) => format!("tab {}", TABS[*i % TABS.len()]),
            KitAct::Check => format!("checkbox {}", if self.checked { "off" } else { "on" }),
            KitAct::Step(d) => format!("stepper {}", (self.value + d).clamp(0, 9)),
            KitAct::Pick(i) => format!("select {}", OPTIONS[*i % OPTIONS.len()]),
            KitAct::Slide(i) => format!("slider step {i}"),
            KitAct::Card(i) => format!("card {}", i + 1),
            KitAct::Row(i) => format!("row {}", i + 1),
            KitAct::Close => "close".into(),
            KitAct::Lock => format!("lock {}", if self.locked { "off" } else { "on" }),
            KitAct::Icon(what) => format!("icon {what}"),
            KitAct::Fold => format!("section {}", if self.open { "folded" } else { "opened" }),
            KitAct::Page(i) => format!("page {}", PAGES[*i % PAGES.len()]),
            KitAct::Block(b) => format!("block {b}"),
            KitAct::Track(t) => format!("track {t}"),
            KitAct::Segment(t, s) => format!("segment {s} on {t}"),
            KitAct::TimelineIcons => format!("timeline icons {}", if self.icons { "off" } else { "on" }),
            KitAct::Effect(k) => self.fx.play(k),
            KitAct::Shake(n) => self.fx.shake(n),
            KitAct::FxSpeed(i) => format!("speed {}", FX_SPEEDS[*i % FX_SPEEDS.len()].0),
            KitAct::HitStop => format!("hit-stop {}", if self.fx.hit_stop { "off" } else { "on" }),
            KitAct::Reduce => format!("reduce motion {}", if self.fx.reduce { "off" } else { "on" }),
            KitAct::Loop => format!("loop {} {}", self.fx.chosen, if self.fx.looping { "off" } else { "on" }),
            KitAct::PortraitToggle(e) => self.portrait.toggle(e),
            KitAct::PortraitFire(w) => self.portrait.fire(w),
            KitAct::PortraitSize(i) => format!("portrait size {} px", PORTRAIT_SIZES[*i % PORTRAIT_SIZES.len()]),
            KitAct::PortraitReduce => format!("portrait reduce motion {}", if self.portrait.reduce { "off" } else { "on" }),
            KitAct::PortraitAnchors => format!("portrait anchors {}", if self.portrait.anchors { "off" } else { "on" }),
            KitAct::PortraitMotion(i) => self.portrait.set_motion(*i),
            KitAct::ThemeView(i) => format!("theme {}", THEME_VIEWS[*i % THEME_VIEWS.len()]),
            KitAct::ThemeFrame(i) => format!("frame {}", FRAME_ROLES[*i % FRAME_ROLES.len()].0),
            KitAct::Pane(i) => format!("pane {}", PANES[*i % PANES.len()]),
            KitAct::PaneTab(i) => format!("rail tab {}", PANE_TABS[*i % PANE_TABS.len()]),
            KitAct::Slot(i) => format!("slot {}", i + 1),
            KitAct::SlotLock => format!("slot lock {}", if self.slot_locked { "off" } else { "on" }),
            KitAct::PaneToggle(w) => {
                let on = match *w {
                    "vignette" => self.vignette,
                    "veil" => self.veil,
                    "ornate" => self.ask_ornate,
                    _ => self.ask_no_default,
                };
                format!("{w} {}", if on { "off" } else { "on" })
            }
            KitAct::TileSet(i) => format!("tile set {i}"),
            KitAct::TilePair(i) => format!("tile pair {i}"),
            KitAct::TileView(i) => format!("tile view {}", TILE_VIEWS[*i % TILE_VIEWS.len()].0),
            KitAct::TileZoom(i) => format!("tile zoom {}x", TILE_ZOOMS[*i % TILE_ZOOMS.len()]),
            KitAct::TilePan(dx, dy) => format!("tile pan ({dx}, {dy})"),
            KitAct::TileGround => format!("composed ground {}", if self.tiles.ground { "off" } else { "on" }),
            KitAct::Tuner(_) => String::new(),
        };
        match a {
            KitAct::Tab(i) => self.tab = i % TABS.len(),
            KitAct::Check => self.checked = !self.checked,
            KitAct::Step(d) => self.value = (self.value + d).clamp(0, 9),
            KitAct::Pick(i) => self.pick = i % OPTIONS.len(),
            KitAct::Slide(i) => self.slide = i.min(SLIDER_STEPS - 1),
            KitAct::Card(i) => self.card = i,
            KitAct::Row(i) => self.row = i,
            KitAct::Lock => self.locked = !self.locked,
            KitAct::Fold => self.open = !self.open,
            KitAct::Page(i) => {
                self.page = i % PAGES.len();
                if self.page != 4 {
                    self.leave_theme();
                }
            }
            KitAct::Block(b) => self.block = Some(b),
            KitAct::Track(t) => self.track = Some(t),
            KitAct::TimelineIcons => self.icons = !self.icons,
            KitAct::FxSpeed(i) => self.fx.speed = i % FX_SPEEDS.len(),
            KitAct::HitStop => self.fx.hit_stop = !self.fx.hit_stop,
            KitAct::Reduce => {
                self.fx.reduce = !self.fx.reduce;
                if self.fx.reduce {
                    self.fx.camera.stop();
                }
            }
            KitAct::Loop => self.fx.toggle_loop(),
            KitAct::PortraitSize(i) => self.portrait.size = i % PORTRAIT_SIZES.len(),
            KitAct::PortraitReduce => self.portrait.reduce = !self.portrait.reduce,
            KitAct::PortraitAnchors => self.portrait.anchors = !self.portrait.anchors,
            KitAct::ThemeView(i) => {
                self.theme_view = i % THEME_VIEWS.len();
                theme::set_theme(theme_view(self.theme_view));
            }
            KitAct::ThemeFrame(i) => self.theme_frame = i % FRAME_ROLES.len(),
            KitAct::Pane(i) => self.pane = i % PANES.len(),
            KitAct::PaneTab(i) => self.pane_tab = i % PANE_TABS.len(),
            KitAct::Slot(i) => self.slot = i,
            KitAct::SlotLock => self.slot_locked = !self.slot_locked,
            KitAct::PaneToggle(w) => match w {
                "vignette" => self.vignette = !self.vignette,
                "veil" => self.veil = !self.veil,
                "ornate" => self.ask_ornate = !self.ask_ornate,
                _ => self.ask_no_default = !self.ask_no_default,
            },
            KitAct::TileSet(i) => {
                self.tiles.set = i;
                self.tiles.pair = 0;
            }
            KitAct::TilePair(i) => self.tiles.pair = i,
            KitAct::TileView(i) => {
                self.tiles.view = i % TILE_VIEWS.len();
                self.tiles.pan = (0, 0);
            }
            KitAct::TileZoom(i) => {
                self.tiles.zoom = i % TILE_ZOOMS.len();
                self.tiles.pan = (0, 0);
            }
            KitAct::TilePan(dx, dy) => self.tiles.pan = ((self.tiles.pan.0 + dx).max(0), (self.tiles.pan.1 + dy).max(0)),
            KitAct::TileGround => self.tiles.ground = !self.tiles.ground,
            KitAct::Tuner(t) => {
                self.tuner.act(t);
                self.last = self.tuner.note.clone();
            }
            KitAct::Press(_) | KitAct::Close | KitAct::Icon(_) | KitAct::Segment(..) | KitAct::Effect(_) | KitAct::Shake(_) | KitAct::PortraitToggle(_) | KitAct::PortraitFire(_) | KitAct::PortraitMotion(_) => {}
        }
    }

    /// The Theme page's switch back to the layers' theme (`theme::loaded()`): called when the page or the gallery is
    /// left, so a comparison never outlives the page (planner decision, LOOK1-E1 review).
    pub fn leave_theme(&mut self) {
        if self.theme_view != 1 {
            self.theme_view = 1;
            theme::set_theme(theme_view(1));
        }
    }

    /// The Effects or Portraits page's clock moves by the wall time since the last call (at most
    /// `vignette::MAX_FRAME_MS`), while that page is on view (the gallery screen calls it every frame).
    pub fn tick(&mut self) {
        let now = aegis_core::clock::Instant::now();
        let since = |last: Option<aegis_core::clock::Instant>| last.map_or(0.0, |l| (now.duration_since(l).as_secs_f64() * 1000.0).min(crate::vignette::MAX_FRAME_MS as f64));
        if self.page == 2 {
            let ms = since(self.fx.last_wall);
            self.fx.last_wall = Some(now);
            self.fx.advance(ms);
        } else {
            self.fx.last_wall = None;
        }
        if self.page == 3 {
            let ms = since(self.portrait.last_wall);
            self.portrait.last_wall = Some(now);
            self.portrait.clock.advance(ms, 1.0);
        } else {
            self.portrait.last_wall = None;
        }
    }

    /// The Effects page's clock moves by `wall_ms` (`FxDemo::advance`), and the Portraits page's clock with it.
    pub fn advance(&mut self, wall_ms: f64) {
        self.fx.advance(wall_ms);
        self.portrait.clock.advance(wall_ms, 1.0);
    }

    /// A chip (`chip.<name>`) onto the other box (`box.left` / `box.right`).
    pub fn can_drop(&self, from: &str, to: &str) -> bool {
        let Some(chip) = from.strip_prefix("chip.") else { return false };
        match to {
            "box.left" => self.right.iter().any(|c| c == chip),
            "box.right" => self.left.iter().any(|c| c == chip),
            _ => false,
        }
    }

    pub fn accept_drop(&mut self, from: &str, to: &str) {
        if !self.can_drop(from, to) {
            return;
        }
        let chip = from.trim_start_matches("chip.").to_string();
        let (src, dst) = if to == "box.left" { (&mut self.right, &mut self.left) } else { (&mut self.left, &mut self.right) };
        src.retain(|c| *c != chip);
        dst.push(chip.clone());
        self.last = format!("dropped {chip} on the {} box", to.trim_start_matches("box."));
    }

    pub fn text_changed(&mut self, id: &str, text: &str) {
        match id {
            "kit.text" => self.text = text.to_string(),
            "kit.area" => self.area = text.to_string(),
            frame_tuner::HEX_FIELD => self.tuner.hex_changed(text),
            _ => {}
        }
    }

    /// Draws the page on view in `r`, the page tabs (`kit.page.<i>`) and what happened last at its foot. Widget ids
    /// start with `kit.`.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: impl Fn(KitAct) -> A) {
        let mut tx = r.x + r.w - 6;
        for (i, p) in PAGES.iter().enumerate().rev() {
            let w = (ui.p.text_w(p, BODY) + 12.0).ceil() as i32;
            tx -= w;
            ui.tab(&format!("kit.page.{i}"), Rect::new(tx, r.y + r.h - 15, w, 14), p, wrap(KitAct::Page(i)), self.page == i);
            tx -= 2;
        }
        if self.page == 1 {
            self.draw_timeline(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else if self.page == 2 {
            self.draw_effects(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else if self.page == 3 {
            self.draw_portraits(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else if self.page == 4 {
            self.draw_theme(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else if self.page == 5 {
            self.draw_panes(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else if self.page == 6 {
            self.draw_tiles(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else if self.page == 7 {
            self.tuner.draw(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &|a| wrap(KitAct::Tuner(a)));
        } else {
            self.draw_widgets(ui, r, &wrap);
        }
        ui.p.text_fit((r.x + 6) as f32, (r.y + r.h - 12) as f32, &format!("Last: {}", self.last), SMALL, (tx - r.x - 10) as f32, GOOD);
    }

    /// The timeline page: a sample round on two sides, every part, a whole-hatched segment, links, marks (one on a
    /// masked track), then a legend.
    fn draw_timeline<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let x = r.x + 6;
        let w = r.w - 12;
        // PORTRAITFX1-TL: the icons check, at the header line's right end; on, every track has its icon square, drawn
        // here as a chip in the track's colour with its label's first letter (a screen draws a portrait, say).
        let check_w = (ui.p.text_w("Icons", BODY) + 16.0).ceil() as i32;
        ui.p.text_fit(x as f32, (r.y + 4) as f32, "Timeline: segment parts, hatching, links and marks (hover for the cards; clicks select)", SMALL, (w - check_w - 6) as f32, INFO);
        ui.checkbox("kit.timeline.icons", x + w - check_w, r.y + 2, "Icons", self.icons, wrap(KitAct::TimelineIcons), true);
        let y = r.y + 4 + ui.p.line_h(SMALL).ceil() as i32 + 2;
        let mut t = sample_timeline(self.block, self.track.clone());
        for track in t.sections.iter_mut().flat_map(|s| s.tracks.iter_mut()) {
            track.icon = self.icons;
        }
        let h = (crate::timeline::RULER_H + 2 + t.content_h()).min(r.y + r.h - y - 30);
        let chip = |ui: &mut Ui<A>, track: &Track, sq: Rect| {
            let c = track.colour;
            ui.p.panel(Rect::new(sq.x + 1, sq.y + 1, sq.w - 2, sq.h - 2), rgba(c[0] / 3, c[1] / 3, c[2] / 3, 255), c, 2.0);
            let initial: String = track.label.chars().take(1).collect();
            ui.p.text_center((sq.x + sq.w / 2) as f32, sq.y as f32 + 3.0, &initial, SMALL, c);
        };
        t.draw_icons(ui, "kit.timeline", Rect::new(x, y, w, h), |b| wrap(KitAct::Block(b)), |k| wrap(KitAct::Track(k.to_string())), |k, s| wrap(KitAct::Segment(k.to_string(), s.to_string())), chip);
        ui.para(
            x,
            y + h + 6,
            w,
            "A windup is shaded, an impact bright with an edge where it lands, a recovery hatched; a segment can be hatched whole. A link joins one segment's end to the next one's start. A mark is a glyph in a colour with a card, inside its block; it takes no click, the text beside it makes room, and it shows on a masked track too.",
            SMALL,
            DIM,
        );
    }

    /// The Effects page: the stage (`kit.fx.stage`) with its two dummies and the live effects, shaken by the camera
    /// (the panel and the controls stay put); beside it a button per primitive (`kit.fx.<kind>`) and per shake preset
    /// (`kit.fx.shake.<name>`), the speed (`kit.fx.speed`), the hit-stop (`kit.fx.hitstop`), reduce motion
    /// (`kit.fx.reduce`) and the loop of the chosen primitive (`kit.fx.loop`, STATUSFX1-E: drawn every frame outside
    /// the live list; under reduce motion its still look); under it the clock's state.
    fn draw_effects<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let x = r.x + 6;
        let line = ui.p.line_h(SMALL).ceil() as i32;
        ui.p.text(x as f32, (r.y + 4) as f32, "Effects: drawn primitives on the effect clock and camera shakes by preset", SMALL, INFO);
        let y = r.y + 4 + line + 2;
        let stage = Rect::new(x, y, FX_STAGE_W, FX_STAGE_H);
        let d = &self.fx;
        let now = d.clock.now();
        ui.p.panel(Rect::new(stage.x - 1, stage.y - 1, stage.w + 2, stage.h + 2), STAGE_BG, crate::ui::BORDER, 2.0);
        let (dx, dy) = d.camera.offset_at(now);
        let looping = if d.looping { format!(", loop {}{}", d.chosen, if d.reduce { " (still)" } else { "" }) } else { String::new() };
        ui.push("kit.fx.stage", "stage", format!("effects stage: {} live, shake ({dx}, {dy}){looping}", d.fx.kinds(now).len()), stage, true, false, None, Vec::new());
        let saved = ui.p.c.clip;
        ui.p.clip(Some(stage));
        ui.p.shift(dx, dy);
        let (ox, oy) = (stage.x as f32, stage.y as f32);
        ui.p.fill(Rect::new(stage.x - 8, stage.y + FX_FOOT_Y as i32 + 1, FX_STAGE_W + 16, 1), rgb(44, 48, 60));
        let placed = |layer: Layer| Fx { live: d.fx.live.iter().filter(|e| e.layer == layer).map(|e| e.moved(ox, oy)).collect() };
        let lp = d.looping.then(|| d.looped().moved(ox, oy));
        let draw_loop = |p: &mut crate::painter::Painter, layer: Layer| match lp.filter(|l| l.layer() == layer) {
            Some(l) if d.reduce => l.draw_still(p),
            Some(l) => l.draw(p, now),
            None => {}
        };
        draw_loop(&mut ui.p, Layer::Ground);
        placed(Layer::Ground).draw(&mut ui.p, now, Layer::Ground);
        for (cx, _) in [FX_FROM, FX_TO] {
            let (fx_, fy) = (ox + cx, oy + FX_FOOT_Y);
            ui.p.ellipse(fx_, fy, 8.0, 2.5, rgba(0, 0, 0, 120));
            ui.p.fill(Rect::new(fx_ as i32 - 4, fy as i32 - 22, 8, 22), rgb(70, 78, 96));
            ui.p.fill(Rect::new(fx_ as i32 - 3, fy as i32 - 29, 6, 6), rgb(96, 104, 122));
        }
        draw_loop(&mut ui.p, Layer::Air);
        placed(Layer::Air).draw(&mut ui.p, now, Layer::Air);
        ui.p.shift(-dx, -dy);
        ui.p.c.clip = saved;
        let state = format!("Clock {} ms, hold {} ms, {} live, impacts to come: {}", now, d.clock.hold.ceil(), d.fx.len(), d.impacts.len());
        ui.p.text_fit(x as f32, (stage.y + stage.h + 4) as f32, &state, SMALL, FX_STAGE_W as f32, DIM);
        ui.para(
            x,
            stage.y + stage.h + 6 + line,
            FX_STAGE_W,
            "Every piece is a pure function of the clock and a seed. A hit-stop holds the clock; the speed scales it. Shakes move the stage only, in whole pixels, never past the engine ceiling (6 px, 3 s); reduce motion stops them and keeps the effects.",
            SMALL,
            TEXT,
        );

        // -- Controls beside the stage
        let x2 = stage.x + stage.w + 10;
        let right = r.x + r.w - 6;
        let bw = |ui: &mut Ui<A>, t: &str| (ui.p.text_w(t, BODY) + 12.0).ceil() as i32;
        let head = |ui: &mut Ui<A>, y: i32, t: &str| -> i32 {
            ui.p.text(x2 as f32, y as f32, t, SMALL, INFO);
            y + line + 1
        };
        let mut yy = head(ui, y, "Primitives (left dummy to right)");
        let mut bx = x2;
        for kind in PRIM_KINDS {
            let label = cap(kind);
            let w = bw(ui, &label);
            if bx + w > right {
                bx = x2;
                yy += 16;
            }
            ui.button(&format!("kit.fx.{kind}"), Rect::new(bx, yy, w, 14), &label, wrap(KitAct::Effect(kind)), true);
            bx += w + 3;
        }
        yy += 14 + 5;
        yy = head(ui, yy, "Camera shakes (engine presets)");
        let mut bx = x2;
        for name in SHAKE_PRESETS {
            let w = bw(ui, name);
            if bx + w > right {
                bx = x2;
                yy += 16;
            }
            ui.button(&format!("kit.fx.shake.{name}"), Rect::new(bx, yy, w, 14), name, wrap(KitAct::Shake(name)), true);
            bx += w + 3;
        }
        yy += 14 + 5;
        yy = head(ui, yy, "Clock speed, hit-stop, motion");
        let speeds: Vec<String> = FX_SPEEDS.iter().map(|s| s.0.to_string()).collect();
        ui.select("kit.fx.speed", Rect::new(x2, yy, 56, 14), &speeds, d.speed, |i| wrap(KitAct::FxSpeed(i)), true, tip("Speed", "Scales the effect clock (hit-stops stay wall time)."));
        ui.checkbox("kit.fx.hitstop", x2 + 64, yy + 2, &format!("Hit-stop {} ms", FX_HIT_STOP_MS), d.hit_stop, wrap(KitAct::HitStop), true);
        yy += 14 + 5;
        ui.checkbox("kit.fx.reduce", x2, yy + 2, "Reduce motion (this page)", d.reduce, wrap(KitAct::Reduce), true);
        yy += 14 + 5;
        ui.checkbox("kit.fx.loop", x2, yy + 2, &format!("Loop {} (every ms + {FX_LOOP_GAP_MS})", d.chosen), d.looping, wrap(KitAct::Loop), true);
    }

    /// The Portraits page: the stage (`kit.portrait.face`: the portrait and its jolt room) with the sample face at the
    /// chosen size and, under it, the tier and what is live; beside it a toggle per overlay and state
    /// (`kit.portrait.<effect>`), a button per reaction (`kit.portrait.flash`, `kit.portrait.jolt.hit`,
    /// `kit.portrait.jolt.crit`), the size (`kit.portrait.size`), reduce motion (`kit.portrait.reduce`), the
    /// anchors check (`kit.portrait.anchors`) and (PORTRAITFX2-M) the motion profile (`kit.portrait.motion`, the foot
    /// line naming its live parts).
    fn draw_portraits<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let d = &self.portrait;
        let x = r.x + 6;
        let line = ui.p.line_h(SMALL).ceil() as i32;
        ui.p.text(x as f32, (r.y + 4) as f32, "Portraits: base life, overlays, states and reactions over a code-drawn face", SMALL, INFO);
        let y = r.y + 4 + line + 2;
        let stage = Rect::new(x, y, PORTRAIT_STAGE, PORTRAIT_STAGE);
        ui.p.panel(stage, STAGE_BG, crate::ui::BORDER, 2.0);
        let size = PORTRAIT_SIZES[d.size.min(PORTRAIT_SIZES.len() - 1)];
        let face = Rect::new(stage.x + (stage.w - size) / 2, stage.y + (stage.h - size) / 2, size, size);
        let now = d.clock.now();
        {
            let mut budget = d.budget.borrow_mut();
            budget.begin();
            d.fx.borrow_mut().draw_image(&mut ui.p, &d.image, "kit.sample", Some(&d.face), face, now, &d.opts(), &mut budget);
        }
        let (tier, live, parts) = {
            let fx = d.fx.borrow();
            (fx.tier().map_or("", Tier::name), fx.live(now), fx.motion(portrait_fx::tq(now)).parts)
        };
        let device = (size as f32 * ui.p.v.s).round() as i32;
        ui.push("kit.portrait.face", "portrait", format!("sample portrait: {size} px, tier {tier}"), stage, true, false, None, Vec::new());
        let col_w = 200;
        let named = |v: &[&str]| if v.is_empty() { "nothing".to_string() } else { v.join(", ") };
        let foot = format!("Tier {tier} ({device} device px). Live: {}", named(&live));
        let fy = ui.para(x, stage.y + stage.h + 4, col_w, &foot, SMALL, DIM);
        let fy = ui.para(x, fy, col_w, &format!("Motion {}: {}", MOTION_SAMPLES[d.motion % MOTION_SAMPLES.len()], named(&parts)), SMALL, DIM);
        ui.para(
            x,
            fy + 4,
            col_w,
            "Pure functions of the clock and a seed from the key, composed at the drawn size and cached. From 96 device px everything draws; 48-95 drops breath, veins and smoke; below 48 only flashes, colour states, fade and the eye glow's core. Reduce motion stops breath, jolts, flicker and drift.",
            SMALL,
            TEXT,
        );

        // -- Controls beside the stage
        let x2 = x + col_w + 10;
        let right = r.x + r.w - 6;
        let head = |ui: &mut Ui<A>, y: i32, t: &str| -> i32 {
            ui.p.text(x2 as f32, y as f32, t, SMALL, INFO);
            y + line + 1
        };
        let mut yy = y;
        for (title, names) in [("Overlays (held at full strength)", &PORTRAIT_OVERLAYS[..]), ("States", &PORTRAIT_STATES[..])] {
            yy = head(ui, yy, title);
            let mut bx = x2;
            for name in names.iter().copied() {
                let label = cap(&name.replace('_', " "));
                let w = 12 + ui.p.text_w(&label, BODY).ceil() as i32 + 4;
                if bx + w > right {
                    bx = x2;
                    yy += 16;
                }
                ui.checkbox(&format!("kit.portrait.{name}"), bx, yy + 2, &label, d.on.contains(name), wrap(KitAct::PortraitToggle(name)), true);
                bx += w + 8;
            }
            yy += 14 + 5;
        }
        yy = head(ui, yy, "Reactions");
        let mut bx = x2;
        for (what, label) in PORTRAIT_REACTIONS.iter().zip(["Flash", "Hit jolt", "Crit jolt"]) {
            let w = (ui.p.text_w(label, BODY) + 12.0).ceil() as i32;
            ui.button(&format!("kit.portrait.{what}"), Rect::new(bx, yy, w, 14), label, wrap(KitAct::PortraitFire(what)), true);
            bx += w + 3;
        }
        yy += 14 + 5;
        yy = head(ui, yy, "Size, motion, anchors");
        let sizes: Vec<String> = PORTRAIT_SIZES.iter().map(|s| format!("{s} px")).collect();
        ui.select("kit.portrait.size", Rect::new(x2, yy, 56, 14), &sizes, d.size, |i| wrap(KitAct::PortraitSize(i)), true, tip("Size", "The portrait's side in logical px; the tier follows the device px."));
        ui.checkbox("kit.portrait.reduce", x2 + 64, yy + 2, "Reduce motion (this page)", d.reduce, wrap(KitAct::PortraitReduce), true);
        yy += 14 + 5;
        ui.checkbox("kit.portrait.anchors", x2, yy + 2, "Anchors (eye boxes, glow, vein roots)", d.anchors, wrap(KitAct::PortraitAnchors), true);
        // PORTRAITFX2-M: the motion profile
        yy += 14 + 5;
        let names: Vec<String> = MOTION_SAMPLES.iter().map(|n| cap(n)).collect();
        ui.select("kit.portrait.motion", Rect::new(x2, yy, 72, 14), &names, d.motion, |i| wrap(KitAct::PortraitMotion(i)), true, tip("Motion", "The face's motion profile: parts combined by one function (cycle, lean, bob, entry, settle, rest, shiver, flush)."));
        ui.p.text((x2 + 80) as f32, (yy + 2) as f32, "Motion profile", SMALL, DIM);
    }

    /// The widgets page: two columns of named groups.
    fn draw_widgets<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let col_w = (r.w - 18) / 2;
        let (x1, x2) = (r.x + 6, r.x + 12 + col_w);
        let gap = 5;
        let head = |ui: &mut Ui<A>, x: i32, y: i32, t: &str| -> i32 {
            ui.p.text(x as f32, y as f32, t, SMALL, INFO);
            y + ui.p.line_h(SMALL).ceil() as i32 + 1
        };
        let bw = |ui: &mut Ui<A>, t: &str| (ui.p.text_w(t, BODY) + 12.0).ceil() as i32;

        // -- Left column
        let mut y = r.y + 4;
        y = head(ui, x1, y, "Buttons: normal, selected, disabled, with a hover card");
        let mut x = x1;
        for (id, label, active, enabled) in [("kit.button", "Button", false, true), ("kit.button.active", "Selected", true, true), ("kit.button.off", "Disabled", false, false)] {
            let w = bw(ui, label);
            ui.button_full(id, Rect::new(x, y, w, 14), label, wrap(KitAct::Press(label_key(id))), enabled, active, BODY, Vec::new());
            x += w + 4;
        }
        let w = bw(ui, "Hover me");
        ui.button_tip("kit.button.tip", Rect::new(x, y, w, 14), "Hover me", wrap(KitAct::Press("hover card")), true, tip("Hover card", "Every widget can carry one: a title line and its text, wrapped."));
        y += 14 + gap;
        y = head(ui, x1, y, "Tabs");
        let mut x = x1;
        for (i, t) in TABS.iter().enumerate() {
            let w = bw(ui, t);
            ui.tab(&format!("kit.tab.{i}"), Rect::new(x, y, w, 14), t, wrap(KitAct::Tab(i)), self.tab == i);
            x += w + 2;
        }
        y += 14 + gap;
        y = head(ui, x1, y, "Checkbox and stepper");
        ui.checkbox("kit.check", x1, y + 2, if self.checked { "Checked" } else { "Not checked" }, self.checked, wrap(KitAct::Check), true);
        ui.stepper("kit.stepper", Rect::new(x1 + 80, y, 130, 14), self.value as f32 / 9.0, &self.value.to_string(), wrap(KitAct::Step(-1)), wrap(KitAct::Step(1)), true);
        y += 14 + gap;
        y = head(ui, x1, y, "Dropdown (a menu) and select (a choice list)");
        let half = (col_w - 4) / 2;
        ui.dropdown("kit.dropdown", Rect::new(x1, y, half, 14), "Dropdown", wrap(KitAct::Press("dropdown")), true, tip("Dropdown", "Opens a menu of its own (the screen draws it)."));
        let options: Vec<String> = OPTIONS.iter().map(|o| o.to_string()).collect();
        ui.select("kit.select", Rect::new(x1 + half + 4, y, half, 14), &options, self.pick, |i| wrap(KitAct::Pick(i)), true, tip("Select", "Opens its choice list; arrows and Enter work too."));
        y += 14 + gap;
        y = head(ui, x1, y, "Slider");
        ui.slider("kit.slider", Rect::new(x1, y, col_w.min(160), 14), self.slide, SLIDER_STEPS, &format!("Step {} of {}", self.slide + 1, SLIDER_STEPS), |i| wrap(KitAct::Slide(i)), true);
        y += 14 + gap;
        y = head(ui, x1, y, "Text field (click to type)");
        ui.text_field("kit.text", Rect::new(x1, y, col_w.min(180), 14), &self.text, false, true);
        y += 14 + gap;
        y = head(ui, x1, y, "Text area: Up / Down, Shift selects, Ctrl+arrows by word, Ctrl+A, C, X, V, Z, Y");
        let ah = (ui.p.line_h(BODY) * 4.0).ceil() as i32 + 6;
        ui.text_field("kit.area", Rect::new(x1, y, col_w, ah), &self.area, true, true);
        y += ah + gap;
        y = head(ui, x1, y, "Frame, paragraph and close button");
        let fr = Rect::new(x1, y, col_w.min(200), (r.y + r.h - 18 - y).clamp(46, 70));
        let body_y = ui.frame(fr, "Frame", false);
        ui.close_x("kit.close", Rect::new(fr.x + fr.w - 16, fr.y + 4, 12, 12), wrap(KitAct::Close));
        ui.para(fr.x + 6, body_y, fr.w - 12, "A paragraph wraps to its width and returns where it ends, so what follows sits under it.", SMALL, TEXT);

        // -- Right column
        let mut y = r.y + 4;
        y = head(ui, x2, y, "Cards (click to select)");
        let cw = (col_w - 4) / 2;
        for i in 0..2 {
            let lines: Vec<(String, Rgba)> = vec![(format!("Card {}", i + 1), GOLD), ("Two lines of text".into(), TEXT)];
            ui.card(&format!("kit.card.{i}"), Rect::new(x2 + i as i32 * (cw + 4), y, cw, 26), &lines, Some(wrap(KitAct::Card(i))), self.card == i, true, Vec::new());
        }
        y += 26 + gap;
        y = head(ui, x2, y, "Rows: clickable, selected, info (hover card only)");
        for i in 0..2 {
            ui.row(&format!("kit.row.{i}"), Rect::new(x2, y, col_w, 13), &format!("Row {}", i + 1), if self.row == i { GOLD } else { TEXT }, Some(wrap(KitAct::Row(i))), self.row == i, Vec::new());
            y += 14;
        }
        ui.row("kit.row.info", Rect::new(x2, y, col_w, 13), "Info row", DIM, None, false, tip("Info row", "No action: hovering shows its card and an outline."));
        y += 13 + gap;
        y = head(ui, x2, y, "Drag and drop (drag a chip to the other box)");
        let bh = 34;
        for (side, chips, bx) in [("left", &self.left, x2), ("right", &self.right, x2 + cw + 4)] {
            let b = Rect::new(bx, y, cw, bh);
            ui.p.panel(b, crate::ui::CARD, crate::ui::BORDER, 2.0);
            ui.push(format!("kit.box.{side}"), "drop", format!("{side} box"), b, true, false, None, Vec::new());
            ui.drop_target(format!("box.{side}"));
            let mut cx = b.x + 3;
            for c in chips {
                let w = bw(ui, c);
                let chip = Rect::new(cx, b.y + 3, w, 12);
                ui.button_full(&format!("kit.chip.{c}"), chip, c, wrap(KitAct::Press("chip")), true, false, SMALL, Vec::new());
                ui.draggable(format!("chip.{c}"));
                cx += w + 3;
            }
        }
        y += bh + gap;
        y = head(ui, x2, y, "Icon buttons: image, drawn shapes (file missing), toggled, disabled");
        let ib = |i: i32| Rect::new(x2 + i * 19, y, 16, 16);
        let px = ui.icon_px(ib(0));
        let sample = sample_icon(px);
        ui.icon_button("kit.icon.image", ib(0), "Image icon", Some(&sample), Glyph::Letter('i'), wrap(KitAct::Icon("image")), true, false, tip("Image icon", "An image from the asset layers; a drawn shape stands in while the file is missing."));
        let (lock, lock_tip) = if self.locked { ("Unlock", "Unlock") } else { ("Lock", "Lock (protect from overwrite and delete)") };
        ui.icon_button("kit.icon.lock", ib(1), lock, None, if self.locked { Glyph::Lock } else { Glyph::Unlock }, wrap(KitAct::Lock), true, self.locked, tip(lock_tip, ""));
        let del_tip = if self.locked { tip("Delete", "Unlock it to delete it.") } else { Vec::new() };
        ui.icon_button("kit.icon.delete", ib(2), "Delete", None, Glyph::Delete, wrap(KitAct::Icon("delete")), !self.locked, false, del_tip);
        ui.icon_button("kit.icon.help", ib(3), "Help", None, Glyph::Help, wrap(KitAct::Icon("help")), true, false, tip("Help (?)", "A hover card can list the keys."));
        ui.icon_button("kit.icon.menu", ib(4), "Menu", None, Glyph::Menu, wrap(KitAct::Icon("menu")), true, false, Vec::new());
        ui.icon_button("kit.icon.letter", ib(5), "Party", None, Glyph::Letter('P'), wrap(KitAct::Icon("party")), true, false, tip("Party (P)", "A letter stands in for an icon with no shape of its own."));
        ui.icon_button("kit.icon.off", ib(6), "Disabled", Some(&sample), Glyph::Letter('i'), wrap(KitAct::Icon("disabled")), false, false, tip("Disabled", "Dimmed; takes no click."));
        y += 16 + gap;
        y = head(ui, x2, y, "Section header (click to fold)");
        let open = ui.section("kit.section", Rect::new(x2, y, col_w, 14), "Section", "2 rows", self.open, wrap(KitAct::Fold), tip("Section", "A click folds or opens it; the screen keeps the state."));
        y += 14 + 1;
        if open {
            for i in 0..2 {
                ui.row(&format!("kit.section.row.{i}"), Rect::new(x2 + 8, y, col_w - 8, 12), &format!("Content row {}", i + 1), TEXT, None, false, Vec::new());
                y += 13;
            }
        }
        y += gap - 1;
        y = head(ui, x2, y, "Scroll area (the wheel over it)");
        let view = Rect::new(x2, y, col_w, (r.y + r.h - 18 - y).max(24));
        ui.p.panel(view, crate::ui::CARD, crate::ui::BORDER, 2.0);
        let line = ui.p.line_h(SMALL).ceil() as i32 + 1;
        let off = ui.scroll_begin("kit.scroll", view, 12 * line + 4);
        for i in 0..12 {
            ui.p.text((view.x + 4) as f32, (view.y + 2 - off + i * line) as f32, &format!("Line {} of a list taller than its area", i + 1), SMALL, TEXT);
        }
        ui.scroll_end();
    }
}

/// LOOK1-ME: the Panes page's menu column items (generic menu words; Credits is disabled to show its reason).
pub fn pane_menu<A: Clone>(wrap: &impl Fn(KitAct) -> A) -> Vec<MenuItem<A>> {
    vec![
        MenuItem::new("continue", "Continue", Some('c'), wrap(KitAct::Press("continue"))).with_tip("Carries on from the last save."),
        MenuItem::new("new", "New game", Some('n'), wrap(KitAct::Press("new game"))),
        MenuItem::new("load", "Load", Some('l'), wrap(KitAct::Press("load"))),
        MenuItem::new("settings", "Settings", Some('s'), wrap(KitAct::Press("settings"))),
        MenuItem::new("credits", "Credits", None, wrap(KitAct::Press("credits"))).off("Nothing to show yet."),
        MenuItem::new("quit", "Quit", Some('q'), wrap(KitAct::Press("quit"))),
    ]
}

/// LOOK1-ME: the Panes page's sample picture, 320 x 180 (16:9): a dusk sky, a low sun and two hill lines, so the
/// cover fit's crop and the vignette show.
pub fn sample_backdrop() -> &'static Image {
    static IMG: std::sync::OnceLock<Image> = std::sync::OnceLock::new();
    IMG.get_or_init(|| {
        let (w, h) = (320, 180);
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let t = y as f32 / h as f32;
                let mut c = [(40.0 + 150.0 * t) as u8, (34.0 + 70.0 * t) as u8, (70.0 - 20.0 * t) as u8];
                let (dx, dy) = (x as f32 - 230.0, y as f32 - 110.0);
                if dx * dx + dy * dy < 22.0 * 22.0 {
                    c = [250, 200, 110];
                }
                let far = 118.0 + 10.0 * (x as f32 / 30.0).sin();
                let near = 140.0 + 14.0 * (x as f32 / 47.0 + 1.3).sin();
                if y as f32 > near {
                    c = [24, 22, 30];
                } else if y as f32 > far {
                    c = [52, 44, 58];
                }
                // a marker column at each edge, so the crop of the sides shows
                if x < 4 || x >= w - 4 {
                    c = [200, 60, 60];
                }
                px[((y * w + x) * 4) as usize..((y * w + x) * 4 + 4) as usize].copy_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
        Image { w, h, px }
    })
}

/// LOOK1-ME: five 20 x 20 sample portraits (a head and shoulders in a hue each).
pub fn sample_portraits() -> &'static [Image] {
    static IMGS: std::sync::OnceLock<Vec<Image>> = std::sync::OnceLock::new();
    IMGS.get_or_init(|| {
        [STEEL, AMBER, ORCHID, CANDLE, LINEN]
            .iter()
            .map(|hue| {
                let n = 20;
                let mut px = vec![0u8; (n * n * 4) as usize];
                for y in 0..n {
                    for x in 0..n {
                        let (fx, fy) = (x as f32 - 9.5, y as f32);
                        let head = fx * fx + (fy - 7.0) * (fy - 7.0) < 20.0;
                        let body = fy > 13.0 && fx.abs() < 8.0;
                        let c = if head { [hue[0], hue[1], hue[2], 255] } else if body { [hue[0] / 2, hue[1] / 2, hue[2] / 2, 255] } else { [30, 32, 40, 255] };
                        px[((y * n + x) * 4) as usize..((y * n + x) * 4 + 4) as usize].copy_from_slice(&c);
                    }
                }
                Image { w: n, h: n, px }
            })
            .collect()
    })
}

impl Specimens {
    /// The Panes page's keys (the gallery screen forwards them): the menu column's hotkeys, the tab rail's PageUp /
    /// PageDown and the ask's Enter, y, n and Esc. True when the key was used.
    pub fn key(&mut self, k: Key) -> bool {
        if self.page == 7 {
            let used = self.tuner.key(k);
            if used {
                self.last = self.tuner.note.clone();
            }
            return used;
        }
        if self.page != 5 {
            return false;
        }
        let a = match self.pane {
            1 => menu_key(&pane_menu(&|a| a), k),
            2 => tab_key(PANE_TABS.len(), self.pane_tab, k).map(KitAct::PaneTab),
            4 => ask_key(self.ask_no_default, k).map(|yes| KitAct::Press(if yes { "yes" } else { "no" })),
            _ => None,
        };
        match a {
            Some(a) => {
                self.act(a);
                true
            }
            None => false,
        }
    }

    /// The Panes page (LOOK1-ME): the pane picker (`kit.pane.<i>`), the pane's switches, then the pane on its stage.
    fn draw_panes<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let mut x = r.x + 6;
        for (i, p) in PANES.iter().enumerate() {
            let w = (ui.p.text_w(p, BODY) + 12.0).ceil() as i32;
            ui.tab(&format!("kit.pane.{i}"), Rect::new(x, r.y + 4, w, 14), p, wrap(KitAct::Pane(i)), self.pane == i);
            x += w + 2;
        }
        let stage = Rect::new(r.x + 6, r.y + 40, r.w - 12, r.h - 44);
        let row = r.y + 24;
        let checks = |ui: &mut Ui<A>, list: &[(&'static str, &str, bool)]| {
            let mut x = r.x + 6;
            for (key, label, on) in list {
                ui.checkbox(&format!("kit.pane.{key}"), x, row, label, *on, wrap(KitAct::PaneToggle(key)), true);
                x += (ui.p.text_w(label, BODY) + 30.0).ceil() as i32;
            }
        };
        match self.pane {
            0 => {
                checks(ui, &[("vignette", "Vignette", self.vignette), ("veil", "Veil 50 %", self.veil)]);
                // A 4:3 rect (the sides cropped) and a tall one beside it (cropped more), the same 16:9 picture.
                let (veil, vig) = (if self.veil { 128 } else { 0 }, if self.vignette { 0.8 } else { 0.0 });
                let wide = Rect::new(stage.x, stage.y, (stage.h * 4 / 3).min(stage.w), stage.h);
                backdrop_pane(ui, "kit.backdrop", Some(sample_backdrop()), "sample 320 x 180 (16:9) in 4:3", wide, veil, vig);
                let tall_x = wide.x + wide.w + 6;
                if stage.x + stage.w - tall_x > 20 {
                    backdrop_pane(ui, "kit.backdrop.tall", Some(sample_backdrop()), "sample 320 x 180 (16:9), tall", Rect::new(tall_x, stage.y, stage.x + stage.w - tall_x, stage.h), veil, vig);
                }
                let note = Rect::new(wide.x + 8, wide.y + wide.h - 40, wide.w - 16, 32);
                let inner = ui.inset(note, false);
                let ink = ui.t.text;
                ui.para(inner.x, inner.y, inner.w, "Cover fit: the 16:9 picture fills each rect, its overflow cropped evenly (the red edge columns fall outside).", SMALL, ink);
            }
            1 => {
                backdrop(&mut ui.p, Some(sample_backdrop()), stage, 128, 0.6);
                let x = stage.x + 16;
                let y = menu_title(ui, x, stage.y + 12, 200, "Title", "A subtitle under it");
                let items = pane_menu(wrap);
                let hovered = menu_column(ui, "kit.menu", x, y, 160, 18, 22, &items);
                // Both styles on one stage: the framed one and the text-only one over the same picture.
                let hovered = menu_column_styled(ui, "kit.menu_text", x + 180, y, 160, 18, 22, &items, MenuStyle::Text).or(hovered);
                let dim = ui.t.dim;
                let said = hovered.map_or("Hovered: none (letters are the hotkeys)".to_string(), |i| format!("Hovered: {}", items[i].label));
                ui.p.text_fit((r.x + 6) as f32, row as f32, &said, SMALL, (r.w - 12) as f32, dim);
            }
            2 => {
                let content = tab_rail(ui, "kit.rail", stage, &PANE_TABS, self.pane_tab, |i| wrap(KitAct::PaneTab(i)));
                let y = ui.header(content.x, content.y, PANE_TABS[self.pane_tab]);
                let ink = ui.t.text;
                ui.para(content.x, y + 4, content.w, "The open tab's form goes here. The rail takes the focus like any buttons; PageUp and PageDown change the tab.", BODY, ink);
            }
            3 => {
                let portraits: Vec<&Image> = sample_portraits().iter().collect();
                let cw = 290.min(stage.w);
                let views = [
                    SlotView { thumb: Some(sample_backdrop()), title: "First save".into(), lines: vec!["The harbour, day 3".into(), "Evening".into()], portraits: portraits[..3].to_vec(), stamp: "2026-11-02 18:40".into() },
                    SlotView { thumb: None, title: "A save with a very long name that cannot fit".into(), lines: vec!["A place whose name runs long as well, day 12".into()], portraits: portraits.clone(), stamp: "2026-10-30 09:05".into() },
                    SlotView { thumb: None, title: "Empty slot".into(), lines: vec!["Nothing saved here".into()], ..SlotView::default() },
                ];
                for (i, v) in views.iter().enumerate() {
                    let cr = Rect::new(stage.x, stage.y + i as i32 * (84 + 6), cw, 84);
                    let corner = if i == 2 {
                        Vec::new()
                    } else {
                        let locked = i == 0 && self.slot_locked;
                        vec![
                            Corner { key: "lock".into(), label: if locked { "Unlock".into() } else { "Lock".into() }, glyph: if locked { Glyph::Lock } else { Glyph::Unlock }, action: wrap(if i == 0 { KitAct::SlotLock } else { KitAct::Press("lock") }), enabled: true, toggled: locked, tip: String::new() },
                            Corner { key: "delete".into(), label: "Delete".into(), glyph: Glyph::Delete, action: wrap(KitAct::Press("delete")), enabled: !locked, toggled: false, tip: if locked { "Unlock it to delete it.".into() } else { String::new() } },
                        ]
                    };
                    slot_card(ui, &format!("kit.slot.{i}"), cr, v, Some(wrap(KitAct::Slot(i))), self.slot == i, &corner);
                }
                let nx = stage.x + cw + 8;
                if stage.x + stage.w - nx > 60 {
                    let inner = ui.inset(Rect::new(nx, stage.y, stage.x + stage.w - nx, 84 * 3 + 12), false);
                    let ink = ui.t.text;
                    ui.para(inner.x, inner.y, inner.w, "Slot cards: a thumbnail (or the title's initial), the title in the display face, place and time, the party's portraits, the timestamp, lock and delete top right. A cut name shows in full on hover.", SMALL, ink);
                }
            }
            _ => {
                checks(ui, &[("ornate", "Ornate frame", self.ask_ornate), ("destructive", "No is the default", self.ask_no_default)]);
                let under = ui.inset(stage, false);
                let ink = ui.t.text;
                ui.para(under.x, under.y, under.w, "The screen under the question: the scrim dims it and its widgets take no focus or click.", BODY, ink);
                let v = AskView { ornate: self.ask_ornate, no_default: self.ask_no_default, ..AskView::new("Leave this place?", "Anything not saved since the last save is lost. Enter answers the default, y and n answer, Esc is No.") };
                ask_dialog(ui, "kit.ask", stage, &v, wrap(KitAct::Press("yes")), wrap(KitAct::Press("no")));
            }
        }
    }
}

impl Specimens {
    /// The Tiles page (LOOK1-E2): the set picker (`kit.tiles.set`), the view (`kit.tiles.view.<i>`) and zoom
    /// (`kit.tiles.zoom.<i>`) tabs, the pair picker (`kit.tiles.pair`, for a set with several pairs), what the set
    /// gives and the pan buttons (`kit.tiles.pan.<dir>`, live when the map is larger than its stage), then the fixture
    /// map on its stage (`kit.tiles.map`).
    fn draw_tiles<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let d = &self.tiles;
        let mut cache = d.cache.borrow_mut();
        cache.sync();
        let sets = cache.sets.clone();
        let at = d.set.min(sets.len().saturating_sub(1));
        let set = sets.get(at).cloned().unwrap_or_default();
        let pairs = if sets.is_empty() { Vec::new() } else { cache.pairs(&set) };
        let pair_at = d.pair.min(pairs.len().saturating_sub(1));
        let pair = pairs.get(pair_at).cloned();
        let (upper, lower) = pair.clone().unwrap_or((TILES_KINDS.upper.to_string(), TILES_KINDS.lower.to_string()));
        let piece_rel = |mask: u8| format!("tilesets/{set}/{upper}/over_{lower}/m{mask:02}.png");

        // The controls.
        let row1 = r.y + 4;
        let options = if sets.is_empty() { vec!["no tile sets".to_string()] } else { sets.clone() };
        ui.select("kit.tiles.set", Rect::new(r.x + 6, row1, 110, 14), &options, at, |i| wrap(KitAct::TileSet(i)), !sets.is_empty(), tip("Tile set", "A folder of tilesets/ in the asset layers."));
        let mut x = r.x + 122;
        for (i, (name, _, _)) in TILE_VIEWS.iter().enumerate() {
            let w = (ui.p.text_w(name, BODY) + 12.0).ceil() as i32;
            ui.tab(&format!("kit.tiles.view.{i}"), Rect::new(x, row1, w, 14), name, wrap(KitAct::TileView(i)), d.view == i);
            x += w + 2;
        }
        x += 6;
        for (i, z) in TILE_ZOOMS.iter().enumerate() {
            let label = format!("{z}x");
            let w = (ui.p.text_w(&label, BODY) + 12.0).ceil() as i32;
            ui.tab(&format!("kit.tiles.zoom.{i}"), Rect::new(x, row1, w, 14), &label, wrap(KitAct::TileZoom(i)), d.zoom == i);
            x += w + 2;
        }
        x += 6;
        let ground_w = (ui.p.text_w("Ground", BODY) + 12.0).ceil() as i32;
        ui.tab("kit.tiles.ground", Rect::new(x, row1, ground_w, 14), "Ground", wrap(KitAct::TileGround), d.ground);

        // The map, in cells of the view at the zoom; the pan keeps it on the stage.
        let (_, vw, vh) = TILE_VIEWS[d.view % TILE_VIEWS.len()];
        let zoom = TILE_ZOOMS[d.zoom % TILE_ZOOMS.len()];
        let (cw, ch) = (vw * zoom, vh * zoom);
        let (cols, rows) = (TILES_MAP[0].len() as i32, TILES_MAP.len() as i32);
        let stage = Rect::new(r.x + 6, r.y + 40, r.w - 12, r.h - 44);
        let pan_max = (((cols * cw - stage.w).max(0) + cw - 1) / cw, ((rows * ch - stage.h).max(0) + ch - 1) / ch);
        let pan = (d.pan.0.min(pan_max.0), d.pan.1.min(pan_max.1));

        let row2 = r.y + 22;
        let buttons = [("left", "<", (-1, 0), pan.0 > 0), ("right", ">", (1, 0), pan.0 < pan_max.0), ("up", "^", (0, -1), pan.1 > 0), ("down", "v", (0, 1), pan.1 < pan_max.1)];
        let mut bx = r.x + r.w - 6 - 16 * buttons.len() as i32;
        for (name, label, (dx, dy), live) in buttons {
            ui.button(&format!("kit.tiles.pan.{name}"), Rect::new(bx, row2, 14, 14), label, wrap(KitAct::TilePan(dx, dy)), live);
            bx += 16;
        }
        let mut said_x = r.x + 6;
        if pairs.len() > 1 {
            let names: Vec<String> = pairs.iter().map(|(u, l)| format!("{u} over {l}")).collect();
            ui.select("kit.tiles.pair", Rect::new(said_x, row2, 110, 14), &names, pair_at, |i| wrap(KitAct::TilePair(i)), true, tip("Transition pair", "A folder <upper>/over_<lower>/ of this set."));
            said_x += 116;
        }
        let have = (1..=14u8).filter(|m| pair.is_some() && cache.image(&piece_rel(*m)).is_some()).count();
        let said = if d.ground {
            "Composed ground: three fixture layers, a generated edge, a sunken layer (needs no tile set).".to_string()
        } else if sets.is_empty() {
            "No tilesets/ in the asset layers: flat colours and the mask numbers.".to_string()
        } else if pair.is_some() {
            format!("{upper} over {lower}: {have} of 14 pieces (a number marks a corner whose piece is missing).")
        } else {
            format!("{set} has no transition pieces: the numbers are the corner masks ({upper}, {lower}).")
        };
        let dim = ui.t.dim;
        ui.p.text_fit(said_x as f32, row2 as f32 + (14.0 - ui.p.line_h(SMALL)) / 2.0, &said, SMALL, (r.x + r.w - 10 - 16 * buttons.len() as i32 - said_x) as f32, dim);

        // The stage.
        ui.push("kit.tiles.map", "stage", format!("tile map: {} {}x", TILE_VIEWS[d.view % TILE_VIEWS.len()].0, zoom), stage, true, false, None, Vec::new());
        ui.p.fill(stage, STAGE_BG);
        let map = Rect::new(stage.x - pan.0 * cw, stage.y - pan.1 * ch, cols * cw, rows * ch);
        let cell = |x: i32, y: i32| Rect::new(map.x + x * cw, map.y + y * ch, cw, ch);
        if d.ground {
            ui.push_clip(stage.intersect(map));
            draw_ground_fixture(&mut ui.p, &mut cache, map, (vw, vh));
            ui.pop_clip();
            return;
        }
        let is_upper = |x: i32, y: i32| tiles_map_cell(x, y) != b'~';
        let is_block = |x: i32, y: i32| (0..cols).contains(&x) && (0..rows).contains(&y) && tiles_map_cell(x, y) == b'#';
        ui.push_clip(stage.intersect(map));
        // The ground pass: a block stands on the upper kind.
        for y in 0..rows {
            for x in 0..cols {
                let (kind, flat) = if is_upper(x, y) { (upper.as_str(), TILE_UPPER) } else { (lower.as_str(), TILE_LOWER) };
                match cache.tile(&set, kind) {
                    Some(img) => ui.p.image(img, cell(x, y)),
                    None => ui.p.fill(cell(x, y), flat),
                }
            }
        }
        // The transition pieces, centred on their corners; the corners without one get their number last.
        let mut numbers = Vec::new();
        for (cx, cy, mask) in corner_pieces(cols, rows, is_upper) {
            match if pair.is_some() { cache.image(&piece_rel(mask)) } else { None } {
                Some(img) => ui.p.image(img, Rect::new(map.x + cx * cw - cw / 2, map.y + cy * ch - ch / 2, cw, ch)),
                None => numbers.push((cx, cy, mask)),
            }
        }
        // The blocks: the cap, the face over its lower half when the south is open, 1 px lines on the open north and
        // sides of the cap.
        for y in 0..rows {
            for x in 0..cols {
                if !is_block(x, y) {
                    continue;
                }
                let c = cell(x, y);
                match cache.tile(&set, TILES_KINDS.block) {
                    Some(img) => ui.p.image(img, c),
                    None => ui.p.fill(c, TILE_BLOCK),
                }
                let open = block_edges(is_block, x, y);
                let cap_h = if open.south { ch / 2 } else { ch };
                if open.south {
                    let face = Rect::new(c.x, c.y + cap_h, cw, ch - cap_h);
                    match cache.tile(&set, TILES_KINDS.face) {
                        Some(img) => ui.p.image(img, face),
                        None => ui.p.fill(face, TILE_FACE),
                    }
                }
                if open.north {
                    ui.p.fill(Rect::new(c.x, c.y, cw, 1), TILE_LINE);
                }
                if open.west {
                    ui.p.fill(Rect::new(c.x, c.y, 1, cap_h), TILE_LINE);
                }
                if open.east {
                    ui.p.fill(Rect::new(c.x + cw - 1, c.y, 1, cap_h), TILE_LINE);
                }
            }
        }
        ui.pop_clip();
        // Only a number wholly on the stage (the audit's texts stay inside what is seen).
        let lh = ui.p.line_h(SMALL).ceil() as i32;
        for (cx, cy, mask) in numbers {
            let text = mask.to_string();
            let w = ui.p.text_w(&text, SMALL).ceil() as i32 + 2;
            let plate = Rect::new(map.x + cx * cw - w / 2, map.y + cy * ch - lh / 2, w, lh);
            if plate.x >= stage.x && plate.y >= stage.y && plate.x + plate.w <= stage.x + stage.w && plate.y + plate.h <= stage.y + stage.h {
                ui.p.fill(plate, STAGE_BG);
                ui.p.text_center((plate.x + plate.w / 2) as f32, plate.y as f32, &text, SMALL, CANDLE);
            }
        }
    }
}

/// The Tiles page's Ground switch: `TILES_MAP` through the composer, at the art px per cell of the view's shape (16 x 16
/// square, 16 x 12 tilted), three layers of fixture textures: `.` the lowest, toned; `~` a sunken layer; `#` an upper
/// ground with wear and strays.
fn draw_ground_fixture(p: &mut Painter, cache: &mut TileCache, map: Rect, view: (i32, i32)) {
    let cell = (ground::CELL_PX, ground::CELL_PX * view.1 / view.0);
    let (tw, th) = (ground::TEX_CELLS * cell.0, ground::TEX_CELLS * cell.1);
    let TileCache { ground_tex, ground_keys, .. } = cache;
    let tex = ground_tex.entry(cell).or_insert_with(|| {
        [
            ground::two_tone(tw, th, rgb(74, 66, 56), rgb(70, 62, 53), 1),
            ground::two_tone(tw, th, rgb(38, 56, 80), rgb(35, 52, 75), 2),
            ground::two_tone(tw, th, rgb(112, 108, 100), rgb(100, 97, 90), 3),
        ]
    });
    let (cols, rows) = (TILES_MAP[0].len() as i32, TILES_MAP.len() as i32);
    let cells: Vec<u8> = (0..rows)
        .flat_map(|y| {
            (0..cols).map(move |x| match tiles_map_cell(x, y) {
                b'~' => 1,
                b'#' => 2,
                _ => 0,
            })
        })
        .collect();
    let blocked = vec![false; cells.len()];
    let layers = [
        GroundLayer { tex: Some(&tex[0]), tone: 0.13, ..GroundLayer::flat(0, TILE_UPPER) },
        GroundLayer { tex: Some(&tex[1]), tone: 0.05, sunken: true, ..GroundLayer::flat(1, TILE_LOWER) },
        GroundLayer { tex: Some(&tex[2]), tone: 0.13, wear: 0.1, strays: true, ..GroundLayer::flat(2, TILE_BLOCK) },
    ];
    let job = GroundJob { seed: 7, cols, rows, cell, cells: &cells, blocked: &blocked, layers: &layers, placed: &[] };
    let key = ground_keys.entry(cell).or_insert_with(|| GroundKey::of("gallery.tiles", &job)).clone();
    ground::draw(p, key, map, || ground::compose(&job));
}

/// The theme the Theme page's switch `view` puts in force: the engine default, the layers' theme, or it with frames
/// off.
pub fn theme_view(view: usize) -> Theme {
    match view {
        0 => Theme::default(),
        1 => (*theme::loaded()).clone(),
        _ => Theme { frames_on: false, ..(*theme::loaded()).clone() },
    }
}

impl Specimens {
    /// The Theme page (LOOK1-E1): the switch and the sources on top, the rest in a scroll area.
    fn draw_theme<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let t = ui.t.clone();
        let bw = |ui: &mut Ui<A>, s: &str| (ui.p.text_w(s, BODY) + 12.0).ceil() as i32;
        let mut x = r.x + 6;
        for (i, v) in THEME_VIEWS.iter().enumerate() {
            let w = bw(ui, v);
            ui.tab(&format!("kit.theme.view.{i}"), Rect::new(x, r.y + 4, w, 14), v, wrap(KitAct::ThemeView(i)), self.theme_view == i);
            x += w + 2;
        }
        let from = if t.sources.is_empty() && t.frames.is_empty() { "no theme file: the engine default".to_string() } else { t.sources.iter().cloned().chain(t.frames.values().map(|f| f.source.clone())).collect::<Vec<_>>().join(", ") };
        ui.p.text_fit((x + 6) as f32, r.y as f32 + 4.0 + (14.0 - ui.p.line_h(SMALL)) / 2.0, &format!("From: {from}"), SMALL, (r.x + r.w - x - 12) as f32, t.dim);
        let view = Rect::new(r.x + 4, r.y + 22, r.w - 8, r.h - 24);
        let content_h = self.theme_content(ui, &t, view, 0, wrap, false);
        let off = ui.scroll_begin("kit.theme.scroll", view, content_h);
        self.theme_content(ui, &t, view, off, wrap, true);
        ui.scroll_end();
    }

    /// Lays out (and when `draw`, draws) the Theme page's content in `view` scrolled by `off`; returns its height.
    #[allow(clippy::too_many_arguments)]
    fn theme_content<A: Clone>(&self, ui: &mut Ui<A>, t: &Theme, view: Rect, off: i32, wrap: &impl Fn(KitAct) -> A, draw: bool) -> i32 {
        let (x0, w) = (view.x + 2, view.w - 8);
        let top = view.y - off;
        let mut y = top + 2;
        let small_h = ui.p.line_h(SMALL).ceil() as i32;
        let head = |ui: &mut Ui<A>, y: i32, s: &str| -> i32 {
            if draw {
                ui.p.text(x0 as f32, y as f32, s, SMALL, t.info);
            }
            y + small_h + 2
        };
        // Colour roles: swatch, name, value, three columns.
        y = head(ui, y, &format!("Colour roles ({}; game colours {})", ROLES.len(), t.game_colors.len()));
        let cols = 3;
        let col_w = w / cols;
        let row_h = small_h + 2;
        let rows = ROLES.len().div_ceil(cols as usize) as i32;
        if draw {
            for (i, name) in ROLES.iter().enumerate() {
                let (c, rr) = (i as i32 / rows, i as i32 % rows);
                let (cx, cy) = (x0 + c * col_w, y + rr * row_h);
                let col = t.role(name).unwrap_or(t.text);
                ui.p.panel(Rect::new(cx, cy + 1, 10, small_h - 2), col, t.dim, 0.0);
                ui.p.text_fit((cx + 13) as f32, cy as f32, name, SMALL, (col_w - 62) as f32, t.text);
                ui.p.text_right((cx + col_w - 6) as f32, cy as f32, &theme::colour_hex(col), SMALL, t.dim);
                ui.push(format!("kit.theme.role.{name}"), "swatch", *name, Rect::new(cx, cy, col_w - 4, row_h), true, false, None, tip(name, theme::ROLE_DOCS[i]));
            }
        }
        y += rows * row_h + 4;
        // Contrast pairs (LOOK1 8.2): the text role on its ground, the ratio, pass or fail.
        y = head(ui, y, "Contrast (LOOK1 8.2: text, gold, info at least 4.5:1, dim 3:1; ink on parchment 4.5:1)");
        let pairs = theme::contrast_pairs();
        let prow = small_h + 4;
        let prows = pairs.len().div_ceil(cols as usize) as i32;
        if draw {
            for (i, (fg, bg, least)) in pairs.iter().enumerate() {
                let (c, rr) = (i as i32 / prows, i as i32 % prows);
                let (cx, cy) = (x0 + c * col_w, y + rr * prow);
                let (f, b) = (t.role(fg).unwrap_or(t.text), t.role(bg).unwrap_or(t.bg));
                let k = t.contrast(f, b);
                ui.p.panel(Rect::new(cx, cy, col_w - 50, prow - 1), b, b, 0.0);
                ui.p.text_fit((cx + 3) as f32, cy as f32 + 1.5, &format!("{fg} on {bg}"), SMALL, (col_w - 56) as f32, f);
                let ok = k >= *least;
                ui.p.text_right((cx + col_w - 6) as f32, cy as f32 + 1.5, &format!("{k:.1} {}", if ok { "ok" } else { "low" }), SMALL, if ok { t.good } else { t.foe });
            }
        }
        y += prows * prow + 4;
        // The three faces at the four text sizes.
        y = head(ui, y, "Faces at the four text sizes (body, display, mono)");
        for face in Typeface::ALL {
            let spec = t.face(face);
            let lh = ui.p.line_h_face(face, BODY * THEME_TEXT_SIZES[3]).ceil() as i32;
            if draw {
                let label = format!("{} x{}", face.name(), spec.scale);
                ui.p.text_fit(x0 as f32, (y + lh - small_h) as f32, &label, SMALL, 60.0, t.dim);
                ui.push(format!("kit.theme.face.{}", face.name()), "face", face.name(), Rect::new(x0, y, w, lh), true, false, None, tip(face.name(), &spec.file));
                let mut sx = (x0 + 66) as f32;
                for k in THEME_TEXT_SIZES {
                    let size = BODY * k;
                    let s = format!("Aegis {k}");
                    let sw = ui.p.text_w_face(face, &s, size);
                    ui.p.text_face(face, sx, (y + lh) as f32 - ui.p.line_h_face(face, size), &s, size, t.text);
                    sx += sw + 10.0;
                }
            }
            y += lh + 2;
        }
        y += 2;
        // The new pieces: a header, badges, an inset and a parchment inset.
        y = head(ui, y, "Pieces: Ui::header, Ui::badge, Ui::inset (plain, parchment)");
        let piece_h = 40;
        if draw {
            ui.header(x0, y, "Header");
            let mut bx = x0;
            for (i, role) in ["gold", "info", "good", "foe"].iter().enumerate() {
                ui.badge(Rect::new(bx, y + piece_h - 14, 18, 11), &(3 + 4 * i).to_string(), role);
                bx += 22;
            }
            for (i, parchment) in [false, true].into_iter().enumerate() {
                let ir = Rect::new(x0 + 110 + i as i32 * 150, y, 140, piece_h);
                let inner = ui.inset(ir, parchment);
                let role = if parchment { "parchment" } else { "inset" };
                let ink = t.ink_on(role);
                ui.p.text_fit(inner.x as f32, inner.y as f32, &format!("{role}: {}", t.frame_source(role)), SMALL, inner.w as f32, ink);
            }
        }
        y += piece_h + 6;
        // Every frame role at 40 x 16 and 120 x 40.
        y = head(ui, y, "Frame roles at 40 x 16 and 120 x 40 (hover: the file or fallback that draws it)");
        let per = (w / 44).max(1);
        let small_rows = (FRAME_ROLES.len() as i32 + per - 1) / per;
        if draw {
            for (i, (role, _)) in FRAME_ROLES.iter().enumerate() {
                let fr = Rect::new(x0 + (i as i32 % per) * 44, y + (i as i32 / per) * 20, 40, 16);
                draw_frame_role(ui, t, role, fr);
                ui.push(format!("kit.theme.frame16.{role}"), "frame", *role, fr, true, false, None, tip(role, &t.frame_source(role)));
            }
        }
        y += small_rows * 20 + 2;
        let per = (w / 126).max(1);
        let mid_rows = (FRAME_ROLES.len() as i32 + per - 1) / per;
        if draw {
            for (i, (role, _)) in FRAME_ROLES.iter().enumerate() {
                let fr = Rect::new(x0 + (i as i32 % per) * 126, y + (i as i32 / per) * 44, 120, 40);
                draw_frame_role(ui, t, role, fr);
                let inner = t.frame(role).map_or(fr.inset(4), |f| crate::painter::nine_content_rect(&f.nine, fr, ui.p.v.s));
                ui.p.text_fit(inner.x as f32, inner.y as f32, role, SMALL, inner.w as f32, t.ink_on(role));
                ui.push(format!("kit.theme.frame40.{role}"), "frame", *role, fr, true, false, None, tip(role, &t.frame_source(role)));
            }
        }
        y += mid_rows * 44 + 2;
        // The picked role at 300 x 120, with its picker.
        y = head(ui, y, "One role at 300 x 120");
        let names: Vec<String> = FRAME_ROLES.iter().map(|r| r.0.to_string()).collect();
        if draw {
            ui.select("kit.theme.frame", Rect::new(x0, y, 100, 14), &names, self.theme_frame, |i| wrap(KitAct::ThemeFrame(i)), true, tip("Frame role", "The role drawn at 300 x 120."));
            let role = FRAME_ROLES[self.theme_frame % FRAME_ROLES.len()].0;
            let fr = Rect::new(x0 + 106, y, 300, 120);
            draw_frame_role(ui, t, role, fr);
            let inner = t.frame(role).map_or(fr.inset(6), |f| crate::painter::nine_content_rect(&f.nine, fr, ui.p.v.s));
            ui.p.text_fit(inner.x as f32, inner.y as f32, &format!("{role}: {}", t.frame_source(role)), SMALL, inner.w as f32, t.ink_on(role));
        }
        y += 120 + 4;
        y - top
    }
}

/// Frame role `role` over `r` (its file, else the vector panel in its colours).
fn draw_frame_role<A: Clone>(ui: &mut Ui<A>, t: &Theme, role: &str, r: Rect) {
    let (fill, edge, radius) = t.vector(role);
    ui.skin(role, r, fill, edge, radius);
}

/// The timeline specimen's round: 8 blocks, three tracks on one side and two on the other (one masked).
fn sample_timeline(block: Option<i32>, track: Option<String>) -> Timeline {
    let part = |start, end, style| Part { start, end, style };
    let seg = |key: &str, start, end, label: &str, fill: Rgba, parts: Vec<Part>, card: &str| Segment { key: key.into(), start, end, label: label.into(), fill, parts, tip: tip(label, card), ..Segment::default() };
    let mark = |key: &str, block, glyph: &str, colour, title: &str, body: &str| Mark { key: key.into(), block, glyph: glyph.into(), colour, tip: tip(title, body) };
    let link = |from: &str, to: &str, colour, body: &str| Link { from: from.into(), to: to.into(), colour, tip: tip("Link", body) };
    let (blue, dim) = (rgb(70, 110, 170), rgb(44, 56, 78));
    let red = rgb(160, 70, 64);
    let lead = Track {
        key: "lead".into(),
        label: "Lead".into(),
        colour: TEXT,
        segments: vec![
            seg("strike", 1, 3, "Strike", blue, vec![part(1, 2, PartStyle::Windup), part(3, 3, PartStyle::Impact)], "Windup B1-B2 (shaded), impact B3 (bright, edged)."),
            seg("follow", 4, 5, "Follow-up", blue, vec![part(4, 4, PartStyle::Windup), part(5, 5, PartStyle::Impact)], "Linked to the strike before it."),
            seg("rest", 6, 8, "Rest", dim, Vec::new(), "A plain segment: no parts."),
        ],
        links: vec![link("strike", "follow", GOLD, "Chained: the follow-up starts as the strike lands.")],
        marks: vec![mark("gain", 8, "+", GOOD, "Mark", "A glyph in a colour at one block, with its card.")],
        ..Track::default()
    };
    let scout = Track {
        key: "scout".into(),
        label: "Scout".into(),
        colour: TEXT,
        segments: vec![
            Segment { dashed: true, ..seg("move", 1, 2, "Move", rgb(60, 80, 110), Vec::new(), "Dashed: a planned move.") },
            seg("shot", 3, 6, "Shot", blue, vec![part(3, 5, PartStyle::Windup), part(6, 6, PartStyle::Impact)], "A long windup, impact at B6."),
        ],
        links: vec![link("move", "shot", INFO, "Carried: the shot follows the move.")],
        marks: vec![mark("step", 2, ">", INFO, "Step", "A mark on the move's last block.")],
        ..Track::default()
    };
    let warden = Track {
        key: "warden".into(),
        label: "Warden".into(),
        colour: TEXT,
        segments: vec![
            seg("hold", 1, 4, "Hold", rgb(60, 120, 100), Vec::new(), "A plain segment."),
            Segment { hatched: true, ..seg("dropped", 5, 8, "Dropped", rgb(60, 120, 100), Vec::new(), "Hatched whole: a span that will not happen.") },
        ],
        marks: vec![mark("break", 5, "x", FOE, "Broken", "A mark at a segment's first block: its label starts after it."), mark("note", 5, "i", INFO, "Note", "Two marks in one block sit side by side.")],
        ..Track::default()
    };
    let brute = Track {
        key: "brute".into(),
        label: "Brute".into(),
        colour: FOE,
        segments: vec![seg("smash", 1, 7, "Heavy smash", red, vec![part(1, 3, PartStyle::Windup), part(4, 4, PartStyle::Impact), part(5, 7, PartStyle::Recovery)], "Windup B1-B3, impact B4, recovery B5-B7 (hatched).")],
        marks: vec![mark("impact", 4, "!", FOE, "Impact", "A badge where something lands.")],
        ..Track::default()
    };
    let lurker = Track {
        key: "lurker".into(),
        label: "Lurker".into(),
        colour: FOE,
        segments: vec![seg("hidden", 1, 8, "Hidden", red, Vec::new(), "")],
        masked: Some("Plans in secret".into()),
        marks: vec![mark("impact", 4, "!", FOE, "Impact", "Marks show on a masked track: something lands here, the rest is hidden."), mark("guess", 7, "?", DIM, "Unknown", "")],
        ..Track::default()
    };
    Timeline {
        blocks: 8,
        playhead: 0,
        selected_block: block,
        selected_track: track,
        sections: vec![Section { title: "Allies".into(), colour: INFO, tracks: vec![lead, scout, warden] }, Section { title: "Enemies".into(), colour: FOE, tracks: vec![brute, lurker] }],
    }
}

/// The image icon specimen: a gold diamond with a dark outline, `n` pixels square (an icon as the art would give it).
fn sample_icon(n: i32) -> Image {
    let n = n.max(4);
    let mut px = vec![0u8; (n * n * 4) as usize];
    let c = (n - 1) as f32 / 2.0;
    for y in 0..n {
        for x in 0..n {
            let d = (x as f32 - c).abs() + (y as f32 - c).abs();
            let col = if d <= c * 0.7 { [244, 198, 84, 255] } else if d <= c * 0.95 { [70, 52, 20, 255] } else { [0, 0, 0, 0] };
            px[((y * n + x) * 4) as usize..((y * n + x) * 4 + 4) as usize].copy_from_slice(&col);
        }
    }
    Image { w: n, h: n, px }
}

/// `kind` with its first letter in upper case.
fn cap(kind: &str) -> String {
    let mut c = kind.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

fn label_key(id: &str) -> &'static str {
    match id {
        "kit.button.active" => "the selected button",
        "kit.button.off" => "the disabled button",
        _ => "the button",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chip_moves_to_the_other_box_only() {
        let mut k = Specimens::default();
        assert!(k.can_drop("chip.Sword", "box.right"));
        assert!(!k.can_drop("chip.Sword", "box.left"), "already in the left box");
        assert!(!k.can_drop("Sword", "box.right"), "not a chip");
        k.accept_drop("chip.Sword", "box.right");
        assert_eq!((k.left.clone(), k.right.clone()), (vec!["Shield".to_string()], vec!["Potion".to_string(), "Sword".to_string()]));
        k.accept_drop("chip.Sword", "box.right");
        assert_eq!(k.right.len(), 2, "a refused drop changes nothing");
    }

    #[test]
    fn actions_update_the_sample_values_and_say_what_happened() {
        let mut k = Specimens::default();
        k.act(KitAct::Check);
        assert!(!k.checked);
        k.act(KitAct::Step(10));
        assert_eq!((k.value, k.last.as_str()), (9, "stepper 9"), "the stepper stops at 9");
        k.act(KitAct::Pick(3));
        assert_eq!((k.pick, k.last.as_str()), (3, "select Extra large"));
        k.act(KitAct::Slide(99));
        assert_eq!(k.slide, SLIDER_STEPS - 1);
    }
}
