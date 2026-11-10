//! The region map's two views (EXPLOREVIEW1): the debug setting `region_view` and the geometry both views share, pure
//! functions with no drawing. Square is the map seen from above (16 x 16 cells); Tilted is the battle stage's look at
//! half size (16 x 12 cells, no diamonds): `region.rs` and `props.rs` take every map position from `MapGeom`. A view
//! change only: rules, walking, saves and journeys never read it. In the square view at 1x every result is the formula
//! the map used before this module.
//!
//! REGIONVIEW1: the camera. The party's drawn position glides between its tiles (`Glide`), the camera follows that position
//! in whole physical px and stops at the map's edges (`camera_px`), and looks elsewhere by easing (`Follow`). `MapGeom` stays
//! whole logical px, the nearest to the camera, so every rect, click and hit area keeps its formula; `placed` gives the
//! small rest as a shift of the painter's view, under which each cell's edge is fixed on the map and only moves rigidly.
//!
//! The map's zoom (`map_zoom`, in quarters: `MapGeom::q`): a cell's size is whole logical px at every step, and everything on
//! the map takes its size from its cell (`sprite_rect`, `icon_rect`, `cap_rect`, ...), so one factor sizes the whole scene and
//! the rects stay exact for clicks. The constants below are their sizes at 1x.

use aegis_ui::canvas::Rect;
use serde_json::Value;
use aegis_core::clock::{Duration, Instant};

/// A map cell's width (logical px; 32 px at 1280 wide, MAPVIEW1), and its height in each view.
pub const CELL_W: i32 = 16;
pub const SQUARE_H: i32 = 16;
pub const TILTED_H: i32 = 12;
/// A character sprite (spec PLAYUI1 2.4; the 48 px frame at 24 logical px), standing bottom-centred on its cell.
pub const SPRITE: i32 = 24;
/// A marker icon: stands on its cell.
pub const ICON: i32 = 16;
/// How far a wall's cap is lifted over its cell, and the height of its south face (STAGE1 3.3 at half size).
pub const BLOCK_LIFT: i32 = 6;
#[cfg(test)]
pub const FACE_H: i32 = 6;
/// The shade a wall's face drops on the cell south.
pub const SHADE_H: i32 = 2;
/// How far water sinks: the band over its top is this plus one row (STAGE1 3.2 at half size).
pub const LIP_DEPTH: i32 = 2;

/// How much past a map's edge the camera shows, in cells (REGIONVIEW1; only boundary art is there). Maps ring themselves
/// with `#`, so with the ring that is 2 rows of wall past the last walkable row (user 2026-11-02).
pub const MARGIN_CELLS: i32 = 1;
/// How long the camera eases to a pan, a talk or back to the party (wall ms; none under reduce motion).
pub const CAM_EASE_MS: f64 = 260.0;
/// The map's zoom in quarters of 1x: (least, most, default). The one place the range lives. An accessibility setting (user
/// 2026-11-02): the default is today's view, and nothing goes below it.
pub const ZOOM_QUARTERS: (i32, i32, i32) = (4, 12, 4);
pub const ZOOM_TIP: &str = "Draws the region map larger: ground, walls, props and people together. The bar, the legend and the log keep their size.";

/// The debug setting of a walk's base step time (VIEWKEYS1; a release build uses the default).
pub const WALK_STEP_KEY: &str = "walk_step_ms";
/// The base walk step time in ms: (least, most, default).
pub const WALK_STEP_MS: (f64, f64, f64) = (60.0, 240.0, 200.0);
/// The base step time while running (WALKRUN1: Shift held on the region map), in every build.
pub const RUN_STEP_MS: f64 = 130.0;

/// The `anim_speed` setting as the map plays it (missing or no number: 1x).
pub fn anim_speed(settings: &Value) -> f64 {
    settings["anim_speed"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0)
}

/// The time between steps of a walked path and of a held movement key (VIEWKEYS1, user 2026-11-03: base / `anim_speed`, so
/// anyone preferring fast animation also walks faster, and the walk clip's pace per step stays the same; WALKRUN1: the
/// base is the walk's, or `RUN_STEP_MS` while running).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WalkStep {
    /// The base step time, ms: also the animation ms a step takes, whatever the speed.
    pub base: f64,
    /// The wall ms between steps.
    pub wall: f64,
}

impl WalkStep {
    /// `run`: Shift is held.
    pub fn of(settings: &Value, run: bool) -> Self {
        Self::read(settings, run, cfg!(debug_assertions))
    }

    /// `debug`: whether the debug setting (the walk's only) counts; a value that is no number in range is the default.
    fn read(settings: &Value, run: bool, debug: bool) -> Self {
        let (least, most, default) = WALK_STEP_MS;
        let saved = settings[WALK_STEP_KEY].as_f64().filter(|b| debug && (least..=most).contains(b));
        let base = if run { RUN_STEP_MS } else { saved.unwrap_or(default) };
        Self { base, wall: base / anim_speed(settings) }
    }

    pub fn gap(self) -> Duration {
        Duration::from_secs_f64(self.wall / 1000.0)
    }
}

/// TURNFEEL1's debug settings (F10), both off by default; a release build reads them as off.
pub const TURN_BEAT_KEY: &str = "walk_turn_beat";
pub const EQUAL_SPEED_KEY: &str = "walk_equal_speed";
pub const TURN_CORNER_KEY: &str = "walk_turn_corner";
/// With the quick finish, the most the rest of a leg takes after a perpendicular press (animation ms).
pub const QUICK_FINISH_MS: f64 = 60.0;
/// With the finish, how many steps' time the drawn position may run behind the tile: quick turns past it play faster, so
/// it never falls so far behind that it counts as a jump.
const FINISH_BEHIND_STEPS: f64 = 2.0;
/// How long the party stands, faced the new way, on a perpendicular turn with the beat on (animation ms).
pub const TURN_BEAT_MS: f64 = 60.0;

/// How a perpendicular press draws (TURNFEEL1). The press steps at once in all three: tile, events and the facing `E` uses,
/// so a tap released during a finish has already turned; only the drawn position and the sprite's facing wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TurnCorner {
    /// From wherever it had got to, straight to the new tile: the path cuts the corner.
    #[default]
    Cut,
    /// The current leg ends at its own speed, then the turn: an L, up to one step later.
    Finish,
    /// As `Finish`, with the rest of the leg squeezed into `QUICK_FINISH_MS`.
    Quick,
}

/// F5's order; the first is the default.
pub const TURN_CORNERS: [TurnCorner; 3] = [TurnCorner::Cut, TurnCorner::Finish, TurnCorner::Quick];

impl TurnCorner {
    /// The setting's value.
    pub fn name(self) -> &'static str {
        match self {
            TurnCorner::Cut => "cut",
            TurnCorner::Finish => "finish",
            TurnCorner::Quick => "quick",
        }
    }

    /// The setting's value read; anything unknown is `Cut`.
    fn of_setting(v: &Value) -> Self {
        TURN_CORNERS.into_iter().find(|c| v.as_str() == Some(c.name())).unwrap_or_default()
    }
}

/// The walk's feel options (TURNFEEL1; the user picks by feel). Presentation only: the party's tile changes when the
/// step is taken whatever they say; they set when its glide starts, how long it takes and when the next step comes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WalkFeel {
    /// A perpendicular turn stands for `TURN_BEAT_MS` before its glide.
    pub beat: bool,
    /// A step's time goes with its screen distance (`screen_scale`).
    pub equal: bool,
    /// How a perpendicular press draws.
    pub corner: TurnCorner,
}

impl WalkFeel {
    pub fn of(settings: &Value) -> Self {
        Self::read(settings, cfg!(debug_assertions))
    }

    /// `debug`: whether the settings count; anything but `true` is off.
    pub fn read(settings: &Value, debug: bool) -> Self {
        let on = |key: &str| debug && settings[key].as_bool() == Some(true);
        let corner = if debug { TurnCorner::of_setting(&settings[TURN_CORNER_KEY]) } else { TurnCorner::Cut };
        Self { beat: on(TURN_BEAT_KEY), equal: on(EQUAL_SPEED_KEY), corner }
    }

    /// A step's times: `step` the pace, `facing` the way the party faces before it, `dir` the step. The beat is on the
    /// animation clock, so it divides by `anim_speed` as the pace does (`wall / base`).
    pub fn leg(self, step: WalkStep, view: View, facing: (i64, i64), dir: (i64, i64)) -> Leg {
        let scale = if self.equal { screen_scale(view, dir) } else { 1.0 };
        let turn = is_turn(facing, dir);
        let beat = if self.beat && turn { TURN_BEAT_MS } else { 0.0 };
        let finish_cap = match self.corner {
            TurnCorner::Cut => 0.0,
            TurnCorner::Finish => FINISH_BEHIND_STEPS * step.wall,
            TurnCorner::Quick => QUICK_FINISH_MS * step.wall / step.base,
        };
        Leg {
            glide: step.wall * scale,
            delay: beat * step.wall / step.base,
            anim: step.base * scale + beat,
            reach: if self.beat { Glide::BEAT_REACH } else { Glide::REACH },
            corners: self.corner != TurnCorner::Cut,
            turned: self.corner != TurnCorner::Cut && turn,
            finish_cap,
        }
    }
}

/// Whether `dir` is a perpendicular turn from `facing` (a reversal or going straight on is not).
pub fn is_turn(facing: (i64, i64), dir: (i64, i64)) -> bool {
    facing.0 * dir.0 + facing.1 * dir.1 == 0
}

/// A step's screen distance over a horizontal step's, whatever the zoom (a cell's sides keep their ratio at every step of
/// it). So a step left or right keeps the pace's time, as every step does on square cells, and the speed in px is the
/// square view's in every direction.
pub fn screen_scale(view: View, dir: (i64, i64)) -> f64 {
    if dir.1 != 0 {
        view.cell_h() as f64 / CELL_W as f64
    } else {
        1.0
    }
}

/// One step's times (`WalkFeel::leg`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leg {
    /// The wall ms its glide takes.
    pub glide: f64,
    /// The wall ms it stands first.
    pub delay: f64,
    /// All of it on the animation clock (ms): how long the walk clip plays for this step.
    pub anim: f64,
    /// How far (cells) its glide may start from before it counts as a jump.
    reach: f32,
    /// A corner setting other than `Cut` is on.
    corners: bool,
    /// And this step is a perpendicular turn.
    turned: bool,
    /// The most the path still to play may take before this step's own glide (wall ms).
    finish_cap: f64,
}

impl Leg {
    /// A glide of `ms`, no feel option on.
    pub fn plain(ms: f64) -> Self {
        Self { glide: ms, delay: 0.0, anim: ms, reach: Glide::REACH, corners: false, turned: false, finish_cap: 0.0 }
    }

    #[cfg(test)]
    fn with_anim(self, anim: f64) -> Self {
        Self { anim, ..self }
    }

    /// The wall time to the next step: it waits for this one's beat and glide, so the glides chain with no change of speed.
    pub fn gap(self) -> Duration {
        Duration::from_secs_f64((self.delay + self.glide) / 1000.0)
    }
}

/// The setting's quarters: a number to the nearest step in range; anything else (missing, unknown) the default.
pub fn zoom_of(v: &Value) -> i32 {
    let (least, most, default) = ZOOM_QUARTERS;
    v.as_f64().filter(|z| z.is_finite()).map_or(default, |z| ((z * 4.0).round().clamp(least as f64, most as f64)) as i32)
}

/// Every step's setting value (a whole number of quarters as 1x, 1.25x, ...) in order.
pub fn zoom_steps() -> impl Iterator<Item = f64> {
    (ZOOM_QUARTERS.0..=ZOOM_QUARTERS.1).map(|q| q as f64 / 4.0)
}

/// The region view setting (`region_view`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Square,
    Tilted,
}

/// The setting's values and labels; the first is the default.
pub const VIEWS: [(&str, &str); 2] = [("square", "Square"), ("tilted", "Tilted")];
pub const VIEW_TIP: &str = "Square: the map seen from above. Tilted: shorter rows with standing walls, like the battle stage.";

impl View {
    /// The setting's value: `tilted` is Tilted, anything else (unknown, missing) Square.
    pub fn of_setting(v: &Value) -> View {
        if v.as_str() == Some("tilted") {
            View::Tilted
        } else {
            View::Square
        }
    }

    /// The setting's value of this view (the agent link's `state.view`).
    pub fn name(self) -> &'static str {
        match self {
            View::Square => VIEWS[0].0,
            View::Tilted => VIEWS[1].0,
        }
    }

    /// The other view (the debug build's F7).
    pub fn flipped(self) -> View {
        match self {
            View::Square => View::Tilted,
            View::Tilted => View::Square,
        }
    }

    pub fn cell_h(self) -> i32 {
        match self {
            View::Square => SQUARE_H,
            View::Tilted => TILTED_H,
        }
    }
}

/// The map's placement in a frame: its view and the screen position of cell (0, 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapGeom {
    pub view: View,
    pub ox: i32,
    pub oy: i32,
    /// The zoom in quarters of 1x.
    pub q: i32,
}

impl MapGeom {
    /// The camera: the midpoint of the centres of cells `one` and `other` (the party; the party and the speaker; the
    /// pan) on the centre of `focus`. REGIONVIEW1's `camera_px` took over; kept as the rule tests measure it against.
    #[cfg(test)]
    pub fn centred(view: View, focus: Rect, one: (i64, i64), other: (i64, i64)) -> MapGeom {
        let ch = view.cell_h();
        let ox = focus.x + focus.w / 2 - ((one.0 + other.0) as i32 * CELL_W + CELL_W) / 2;
        let oy = focus.y + focus.h / 2 - ((one.1 + other.1) as i32 * ch + ch) / 2;
        MapGeom { view, ox, oy, q: 4 }
    }

    /// The cell's size (w, h).
    pub fn cell_size(&self) -> (i32, i32) {
        cell_of(self.view, self.q)
    }

    /// Cell (x, y)'s rect.
    pub fn cell(&self, x: i32, y: i32) -> Rect {
        let (cw, ch) = self.cell_size();
        Rect::new(self.ox + x * cw, self.oy + y * ch, cw, ch)
    }

    /// `cell` of a map position.
    pub fn cell_of(&self, at: (i64, i64)) -> Rect {
        self.cell(at.0 as i32, at.1 as i32)
    }

    /// The cell under the point (mx, my), which may be off the map.
    pub fn cell_at(&self, mx: i32, my: i32) -> (i64, i64) {
        let (cw, ch) = self.cell_size();
        ((mx - self.ox).div_euclid(cw) as i64, (my - self.oy).div_euclid(ch) as i64)
    }

    /// The columns (first, last) meeting `area`.
    pub fn cols(&self, area: Rect) -> (i32, i32) {
        let cw = self.cell_size().0;
        ((area.x - self.ox).div_euclid(cw), (area.x + area.w - self.ox - 1).div_euclid(cw))
    }

    /// The rows (first, last) to draw for `area`: those meeting it; tilted, one more below (its lifted cap reaches in).
    pub fn rows(&self, area: Rect) -> (i32, i32) {
        let ch = self.cell_size().1;
        let last = (area.y + area.h - self.oy - 1).div_euclid(ch);
        ((area.y - self.oy).div_euclid(ch), if self.view == View::Tilted { last + 1 } else { last })
    }
}

impl Default for MapGeom {
    fn default() -> Self {
        MapGeom { view: View::Square, ox: 0, oy: 0, q: 4 }
    }
}

/// A cell's size in `view` at `q` quarters: whole logical px at every step.
pub fn cell_of(view: View, q: i32) -> (i32, i32) {
    (CELL_W * q / 4, view.cell_h() * q / 4)
}

/// One axis of `camera_px`: the screen px (physical, from the view's origin) of the map's first cell.
fn axis_px(start: i32, len: i32, cell: i32, count: i32, at: f32, s: f32) -> i32 {
    let px = |logical: i32| (logical as f32 * s).round() as i32;
    let map = count * cell;
    if map <= len {
        return px(start + (len - map) / 2);
    }
    let margin = MARGIN_CELLS * cell;
    let want = px(start + len / 2 - cell / 2) - (at * cell as f32 * s).round() as i32;
    want.clamp(px(start + len - map - margin), px(start + margin))
}

/// The camera (REGIONVIEW1): where cell (0, 0) goes, in whole physical px from the view's origin, with the point `at` (in
/// cells; a whole number is a cell's left and top edge) centred as its cell would be on `focus`. It stops so that past the
/// map only `MARGIN_CELLS` show; a map that fits in `focus` centres in it. `at` rounds to physical px before the
/// subtraction, so whatever follows `at` this way stays on the same physical px: see `party_px`.
pub fn camera_px(cell: (i32, i32), focus: Rect, map: (i32, i32), at: (f32, f32), s: f32) -> (i32, i32) {
    (axis_px(focus.x, focus.w, cell.0, map.0, at.0, s), axis_px(focus.y, focus.h, cell.1, map.1, at.1, s))
}

/// What a camera gives a frame: the geometry of whole logical px nearest to it, and the rest as a shift of the painter's
/// view (physical px). So a cell's physical edge is `camera + round(its map position x s)` wherever the camera is: the
/// map moves rigidly, at a fractional window scale too. A click reads the geometry: with the camera a fraction of a
/// logical px from it, that is as near as the logical mouse can tell.
pub fn placed(view: View, q: i32, camera: (i32, i32), s: f32) -> (MapGeom, (f32, f32)) {
    let near = |px: i32| (px as f32 / s).round() as i32;
    let (ox, oy) = (near(camera.0), near(camera.1));
    (MapGeom { view, ox, oy, q }, (camera.0 as f32 - ox as f32 * s, camera.1 as f32 - oy as f32 * s))
}

/// The party's cell rect and its view shift (as `placed`) when it is drawn at `at` (in cells) under `camera`: its own
/// physical px, rounded on the map as in `camera_px`, so while the camera follows it this never changes.
pub fn party_px(view: View, q: i32, camera: (i32, i32), at: (f32, f32), s: f32) -> (Rect, (f32, f32)) {
    let (cw, ch) = cell_of(view, q);
    let screen = (camera.0 + (at.0 * cw as f32 * s).round() as i32, camera.1 + (at.1 * ch as f32 * s).round() as i32);
    let (g, shift) = placed(view, q, screen, s);
    (Rect::new(g.ox, g.oy, cw, ch), shift)
}

/// A stretch of a drawn path: it stands at `from` until `at`, then goes to `to` in `ms`.
#[derive(Debug, Clone, Copy)]
struct Seg {
    from: (f32, f32),
    to: (f32, f32),
    at: Instant,
    ms: f64,
}

fn after_ms(at: Instant, ms: f64) -> Instant {
    at + Duration::from_secs_f64(ms.max(0.0) / 1000.0)
}

fn cells_apart(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

impl Seg {
    fn end(&self) -> Instant {
        after_ms(self.at, self.ms)
    }

    fn position(&self, now: Instant) -> (f32, f32) {
        let t = (now.saturating_duration_since(self.at).as_secs_f64() * 1000.0 / self.ms).clamp(0.0, 1.0) as f32;
        (self.from.0 + (self.to.0 - self.from.0) * t, self.from.1 + (self.to.1 - self.from.1) * t)
    }
}

/// The party's drawn position (REGIONVIEW1): straight from where it was to its tile in one step's time. A step's start is
/// its scheduled time, not the frame that noticed it, so steps every `WalkStep` make one even line. Presentation only.
#[derive(Debug, Clone, Default)]
pub struct Glide {
    loc: String,
    to: Option<(i64, i64)>,
    from: (f32, f32),
    at: Option<Instant>,
    /// The current step's own time (WALKRUN1: the pace may change between steps and must not move the party back).
    ms: f64,
    /// TURNFEEL1: what plays before that glide while a turn finishes its leg, oldest first.
    before: Vec<Seg>,
}

impl Glide {
    /// The furthest the party glides (cells): one step, and one more if a key press met a step in the same frame.
    const REACH: f32 = 2.0;
    /// With the beat on: a key press still steps at once, so quick turns stack more cells behind the drawn position, and
    /// those are steps to glide, not jumps (a tile this far off in one frame is no step).
    const BEAT_REACH: f32 = 4.0;

    fn rest(&mut self, loc: &str, tile: (i64, i64)) -> ((f32, f32), bool) {
        let here = (tile.0 as f32, tile.1 as f32);
        *self = Glide { loc: loc.to_string(), to: Some(tile), from: here, at: None, ms: 0.0, before: Vec::new() };
        (here, true)
    }

    fn position(&self, now: Instant) -> (f32, f32) {
        if let Some(s) = self.before.iter().find(|s| now < s.end()) {
            return s.position(now);
        }
        let (Some(to), Some(at)) = (self.to, self.at) else { return self.from };
        let t = (now.saturating_duration_since(at).as_secs_f64() * 1000.0 / self.ms).clamp(0.0, 1.0) as f32;
        (self.from.0 + (to.0 as f32 - self.from.0) * t, self.from.1 + (to.1 as f32 - self.from.1) * t)
    }

    /// The drawn position at `now` (None: no clock yet, so no time has passed: the tile), and whether it jumped: a first
    /// frame, a new location, or a tile too far to glide to (the camera then jumps with it). `step_at`: the scheduled time
    /// of a step taken since the last frame; `leg`: the times of a step taken now. A leg's delay starts its glide later, at
    /// the place the party has got to (TURNFEEL1); `position` stands there until then.
    pub fn update(&mut self, loc: &str, tile: (i64, i64), step_at: Option<Instant>, now: Option<Instant>, leg: Leg) -> ((f32, f32), bool) {
        let Some(now) = now else { return self.rest(loc, tile) };
        if self.to.is_none() || self.loc != loc {
            return self.rest(loc, tile);
        }
        if self.to != Some(tile) {
            let start = step_at.unwrap_or(now);
            let was = self.position(start);
            if (tile.0 as f32 - was.0).abs() + (tile.1 as f32 - was.1).abs() > leg.reach {
                return self.rest(loc, tile);
            }
            let begins = start + Duration::from_secs_f64(leg.delay / 1000.0);
            (self.from, self.to, self.at, self.ms) = (was, Some(tile), Some(begins), leg.glide);
            self.before.clear();
        }
        (self.position(now), false)
    }
}

impl Glide {
    /// The way the drawn party goes while a turn finishes its leg (None otherwise): the sprite faces it until the corner.
    pub fn heading(&self, now: Instant) -> Option<(i32, i32)> {
        let way = |d: f32| if d > 0.0 { 1 } else if d < 0.0 { -1 } else { 0 };
        self.before.iter().find(|s| now < s.end()).map(|s| (way(s.to.0 - s.from.0), way(s.to.1 - s.from.1)))
    }

    /// A step from tile `from` to `tile` was just taken at `at`: plays a clean corner when the leg's corner setting asks for
    /// one and a leg is still under way, and returns how much later than a plain step its own glide starts (wall ms).
    /// None leaves the step to `update`, which draws it as it always did: `Cut`, a straight step or a reversal on a leg with
    /// no turn pending, a step with the party already on its tile, or a change of map.
    pub fn corner_step(&mut self, loc: &str, from: (i64, i64), tile: (i64, i64), at: Instant, leg: Leg) -> Option<f64> {
        let adjacent = (from.0 - tile.0).abs() + (from.1 - tile.1).abs() == 1;
        if !leg.corners || !adjacent || self.loc != loc || self.to != Some(from) {
            return None;
        }
        let last_at = self.at?;
        if after_ms(last_at, self.ms) <= at {
            return None;
        }
        self.before.retain(|s| s.end() > at);
        let target = (tile.0 as f32, tile.1 as f32);
        if let Some(i) = self.before.iter().rposition(|s| s.to == target) {
            // back to a corner not yet reached: the path goes no further than it
            let s = self.before[i];
            self.before.truncate(i);
            (self.from, self.to, self.at, self.ms) = (s.from, Some(tile), Some(s.at), s.ms);
            return Some(0.0);
        }
        if !leg.turned && self.before.is_empty() {
            return None;
        }
        self.squeeze(at, leg.finish_cap);
        let end = after_ms(self.at?, self.ms);
        let corner = (from.0 as f32, from.1 as f32);
        self.before.push(Seg { from: self.from, to: corner, at: self.at?, ms: self.ms });
        (self.from, self.to, self.at, self.ms) = (corner, Some(tile), Some(after_ms(end, leg.delay)), leg.glide);
        Some(end.saturating_duration_since(at).as_secs_f64() * 1000.0)
    }

    /// Plays all that is left of the path from `at` in at most `cap` ms, each stretch by its length.
    fn squeeze(&mut self, at: Instant, cap: f64) {
        let (Some(to), Some(last_at)) = (self.to, self.at) else { return };
        if after_ms(last_at, self.ms).saturating_duration_since(at).as_secs_f64() * 1000.0 <= cap {
            return;
        }
        let was = self.position(at);
        let mut ends: Vec<(f32, f32)> = self.before.iter().map(|s| s.to).collect();
        ends.push((to.0 as f32, to.1 as f32));
        let total: f32 = ends.iter().fold((was, 0.0), |(p, d), e| (*e, d + cells_apart(p, *e))).1;
        if total <= 0.0 {
            return;
        }
        self.before.clear();
        let (mut p, mut t) = (was, at);
        for e in ends {
            let ms = (cap * (cells_apart(p, e) / total) as f64).max(1e-3);
            self.before.push(Seg { from: p, to: e, at: t, ms });
            (p, t) = (e, after_ms(t, ms));
        }
        if let Some(last) = self.before.pop() {
            (self.from, self.at, self.ms) = (last.from, Some(last.at), last.ms);
        }
    }
}

/// What the camera looks at; a change of it eases.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Look {
    Party,
    Pan((i64, i64)),
    /// The party and the speaker (their midpoint) over the focus rect above the box.
    Talk((i64, i64), Rect),
}

/// The camera over frames: exactly on its target, except for `CAM_EASE_MS` after what it looks at changed.
#[derive(Debug, Clone, Default)]
pub struct Follow {
    look: Option<Look>,
    lens: Option<(View, i32)>,
    /// Where the last frame's camera was, in logical px (a resize keeps it).
    last: Option<(f32, f32)>,
    ease: Option<((f32, f32), Instant)>,
}

impl Follow {
    /// This frame's camera. `lens`: the view and zoom. `snap`: no easing (a jump, reduce motion); no clock or a new lens
    /// does the same.
    pub fn update(&mut self, look: Look, lens: (View, i32), target: (i32, i32), s: f32, now: Option<Instant>, snap: bool) -> (i32, i32) {
        let changed = self.look != Some(look);
        let snap = snap || self.lens != Some(lens);
        (self.look, self.lens) = (Some(look), Some(lens));
        let out = match (now, self.last) {
            (Some(now), Some(last)) if !snap => {
                if changed {
                    self.ease = Some((last, now));
                }
                self.eased(target, s, now)
            }
            _ => {
                self.ease = None;
                target
            }
        };
        self.last = Some((out.0 as f32 / s, out.1 as f32 / s));
        out
    }

    fn eased(&mut self, target: (i32, i32), s: f32, now: Instant) -> (i32, i32) {
        let Some((from, at)) = self.ease else { return target };
        let t = now.saturating_duration_since(at).as_secs_f64() * 1000.0 / CAM_EASE_MS;
        if t >= 1.0 {
            self.ease = None;
            return target;
        }
        let k = (t * t * (3.0 - 2.0 * t)) as f32;
        let mix = |a: f32, b: i32| (a * s + (b as f32 - a * s) * k).round() as i32;
        (mix(from.0, target.0), mix(from.1, target.1))
    }
}

/// A sprite on `cell`: `SPRITE` square, bottom-centred on the cell's bottom edge.
pub fn sprite_rect(cell: Rect) -> Rect {
    let side = SPRITE * cell.w / CELL_W;
    Rect::new(cell.x + (cell.w - side) / 2, cell.y + cell.h - side, side, side)
}

/// A marker icon (or its stand-in) standing on `cell` (in the square view, the cell itself).
pub fn icon_rect(cell: Rect) -> Rect {
    let side = ICON * cell.w / CELL_W;
    Rect::new(cell.x, cell.y + cell.h - side, side, side)
}

/// A wall's cap on `cell` (tilted): a cell lifted by `BLOCK_LIFT`.
pub fn cap_rect(cell: Rect) -> Rect {
    Rect::new(cell.x, cell.y - lift_of(cell.h), cell.w, cell.h)
}

/// A wall's south face on `cell` (tilted): the cell's bottom `FACE_H`.
pub fn face_rect(cell: Rect) -> Rect {
    let face = lift_of(cell.h);
    Rect::new(cell.x, cell.y + cell.h - face, cell.w, face)
}

/// A wall's drawn rect on `cell` (tilted): its cap, and its face when it has one (an open cell south).
pub fn block_rect(cell: Rect, face: bool) -> Rect {
    let lift = lift_of(cell.h);
    Rect::new(cell.x, cell.y - lift, cell.w, cell.h + if face { lift } else { 0 })
}

/// A tilted cell `ch` tall: a wall's lift and face height (`BLOCK_LIFT` and `FACE_H` at 1x, always equal: the cap ends where
/// the face starts).
pub fn lift_of(ch: i32) -> i32 {
    BLOCK_LIFT * ch / TILTED_H
}

/// The shade under a wall's face on a tilted cell `ch` tall.
pub fn shade_of(ch: i32) -> i32 {
    SHADE_H * ch / TILTED_H
}

/// How far water sinks on a tilted cell `ch` tall.
pub fn lip_of(ch: i32) -> i32 {
    LIP_DEPTH * ch / TILTED_H
}

/// A thin line on a tilted cell `ch` tall: 1 logical px at 1x, by whole px with the zoom.
pub fn line_of(ch: i32) -> i32 {
    (ch / TILTED_H).max(1)
}

/// The smallest rect holding both (the tilted view's icon click areas: the cell and the icon standing on it).
pub fn union(a: Rect, b: Rect) -> Rect {
    let (x, y) = (a.x.min(b.x), a.y.min(b.y));
    Rect::new(x, y, (a.x + a.w).max(b.x + b.w) - x, (a.y + a.h).max(b.y + b.h) - y)
}

/// EXPLOREVIEW1 section 4, checks 1, 2 and the geometry behind 3, 5 and 6.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SPAN: (i32, i32) = (48, 30);

    fn cells() -> impl Iterator<Item = (i32, i32)> {
        (0..SPAN.1).flat_map(|y| (0..SPAN.0).map(move |x| (x, y)))
    }

    /// The origins the sweeps run at: negative, zero and positive, even and odd.
    const ORIGINS: [(i32, i32); 4] = [(0, 0), (7, 50), (-333, -129), (320, 195)];

    #[test]
    fn the_setting_reads_square_unless_it_says_tilted() {
        assert_eq!(View::of_setting(&json!("tilted")), View::Tilted);
        assert_eq!(View::of_setting(&json!("square")), View::Square);
        assert_eq!(View::of_setting(&json!("isometric")), View::Square, "an unknown value is Square");
        assert_eq!(View::of_setting(&Value::Null), View::Square, "a missing value is Square");
        assert_eq!(View::of_setting(&crate::screens::default_settings()["region_view"]), View::Square, "the default");
        assert_eq!(VIEWS[0].0, View::Square.name(), "the first value is the default");
        for (value, _) in VIEWS {
            assert_eq!(View::of_setting(&json!(value)).name(), value, "a value round trips");
        }
        assert_eq!(View::Square.flipped(), View::Tilted);
        assert_eq!(View::Tilted.flipped(), View::Square);
    }

    /// Check 1: in the square view every result is the formula the map used before `MapGeom`.
    #[test]
    fn square_results_are_the_old_formulas_over_a_sweep() {
        let ts = 16;
        for (ox, oy) in ORIGINS {
            let g = MapGeom { view: View::Square, ox, oy, q: 4 };
            assert_eq!(g.cell_size(), (ts, ts));
            for (x, y) in cells() {
                let old = Rect::new(ox + x * ts, oy + y * ts, ts, ts);
                assert_eq!(g.cell(x, y), old);
                assert_eq!(g.cell_of((x as i64, y as i64)), old);
                assert_eq!(sprite_rect(old), Rect::new(old.x + (old.w - ts * 3 / 2) / 2, old.y + old.h - ts * 3 / 2, ts * 3 / 2, ts * 3 / 2));
                assert_eq!(icon_rect(old), old, "the icon is the cell");
                for (mx, my) in [(old.x, old.y), (old.x + ts - 1, old.y + ts - 1), (old.x + ts / 2, old.y + ts / 2)] {
                    let was = (((mx - ox) as f32 / ts as f32).floor() as i64, ((my - oy) as f32 / ts as f32).floor() as i64);
                    assert_eq!(g.cell_at(mx, my), was);
                    assert_eq!(was, (x as i64, y as i64));
                }
            }
            let area = Rect::new(0, 24, 640, 336);
            assert_eq!(g.cols(area), ((area.x - ox).div_euclid(ts), (area.x + area.w - ox - 1).div_euclid(ts)));
            assert_eq!(g.rows(area), ((area.y - oy).div_euclid(ts), (area.y + area.h - oy - 1).div_euclid(ts)), "no extra row");
        }
        let focus = Rect::new(0, 24, 640, 336);
        for (one, other) in [((0, 0), (0, 0)), ((5, 9), (5, 9)), ((3, 4), (8, 1)), ((47, 29), (46, 29))] {
            let g = MapGeom::centred(View::Square, focus, one, other);
            assert_eq!(g.ox, focus.x + focus.w / 2 - ((one.0 + other.0) as i32 * ts + ts) / 2);
            assert_eq!(g.oy, focus.y + focus.h / 2 - ((one.1 + other.1) as i32 * ts + ts) / 2);
        }
    }

    /// Check 2: a cell's centre and the corners inside it all map back to that cell.
    #[test]
    fn tilted_cell_at_finds_each_cell_over_a_sweep() {
        for (ox, oy) in ORIGINS {
            let g = MapGeom { view: View::Tilted, ox, oy, q: 4 };
            assert_eq!(g.cell_size(), (16, 12));
            for (x, y) in cells() {
                let r = g.cell(x, y);
                assert_eq!(r, Rect::new(ox + 16 * x, oy + 12 * y, 16, 12));
                let want = (x as i64, y as i64);
                for (mx, my) in [(r.x + r.w / 2, r.y + r.h / 2), (r.x, r.y), (r.x + r.w - 1, r.y), (r.x, r.y + r.h - 1), (r.x + r.w - 1, r.y + r.h - 1)] {
                    assert_eq!(g.cell_at(mx, my), want, "({mx}, {my}) at origin ({ox}, {oy})");
                }
            }
            assert_eq!(g.cell_at(ox - 1, oy - 1), (-1, -1), "off the map, left of and above cell 0");
        }
    }

    /// Check 3: the camera centres the focus cell; two cells straddle the centre.
    #[test]
    fn the_tilted_camera_centres_the_focus_on_the_area() {
        let focus = Rect::new(0, 24, 640, 336);
        let centre = (focus.x + focus.w / 2, focus.y + focus.h / 2);
        for at in [(0, 0), (5, 9), (47, 29)] {
            let r = MapGeom::centred(View::Tilted, focus, at, at).cell_of(at);
            assert_eq!((r.x + r.w / 2, r.y + r.h / 2), centre, "{at:?}");
        }
        let (a, b) = ((3, 4), (8, 9));
        let g = MapGeom::centred(View::Tilted, focus, a, b);
        let (ra, rb) = (g.cell_of(a), g.cell_of(b));
        assert_eq!(((ra.x + rb.x + 16) / 2, (ra.y + rb.y + 12) / 2), centre, "the midpoint of the two centres");
    }

    /// One more row below the area: the row whose lifted cap reaches into it.
    #[test]
    fn tilted_draws_the_row_below_the_area() {
        let area = Rect::new(0, 24, 640, 336);
        let g = MapGeom { view: View::Tilted, ox: 0, oy: 24, q: 4 };
        let (first, last) = g.rows(area);
        assert_eq!(g.cell(0, first).y, 24, "the first row meets the area");
        let below = g.cell(0, last);
        assert!(below.y >= area.y + area.h, "the last row is below the area");
        assert!(cap_rect(below).y < area.y + area.h, "its cap reaches in");
    }

    /// Check 5 (geometry): the sprite and the icon stand on the cell's bottom edge in both views; the head rises into
    /// the row north; the sprite's bottom middle maps back to the cell.
    #[test]
    fn sprites_and_icons_stand_on_the_cell_bottom() {
        for view in [View::Square, View::Tilted] {
            let g = MapGeom { view, ox: 7, oy: 50, q: 4 };
            let cell = g.cell(4, 3);
            let s = sprite_rect(cell);
            assert_eq!((s.w, s.h), (SPRITE, SPRITE));
            assert_eq!(s.y + s.h, cell.y + cell.h);
            assert_eq!(s.x + s.w / 2, cell.x + cell.w / 2);
            assert_eq!(g.cell_at(s.x + s.w / 2, s.y + s.h - 1), (4, 3));
            let i = icon_rect(cell);
            assert_eq!((i.w, i.h, i.y + i.h, i.x), (ICON, ICON, cell.y + cell.h, cell.x));
        }
        let cell = MapGeom { view: View::Tilted, ox: 0, oy: 0, q: 4 }.cell(0, 5);
        assert_eq!(cell.y - sprite_rect(cell).y, 12, "the head rises 12 px into the row north");
    }

    /// 3.2: the cap is lifted, the face fills the cell's bottom, the drawn rect holds both.
    #[test]
    fn a_block_is_a_lifted_cap_and_a_south_face() {
        let cell = MapGeom { view: View::Tilted, ox: 0, oy: 0, q: 4 }.cell(2, 2);
        let (cap, face) = (cap_rect(cell), face_rect(cell));
        assert_eq!(cap, Rect::new(cell.x, cell.y - BLOCK_LIFT, 16, 12));
        assert_eq!(face, Rect::new(cell.x, cell.y + 6, 16, FACE_H));
        assert_eq!(cap.y + cap.h, face.y, "the face starts where the cap ends");
        assert_eq!(face.y + face.h, cell.y + cell.h, "and ends on the cell's bottom edge");
        assert_eq!(block_rect(cell, true), union(cap, face));
        assert_eq!(block_rect(cell, false), cap);
    }

    /// REGIONVIEW1: the clamp on all four sides in both views, at whole and fractional scales: with the point at any
    /// corner tile, the margin past the near edges is `MARGIN_CELLS` cells (within the rounding of one physical px).
    #[test]
    fn the_camera_stops_with_a_small_margin_at_every_corner() {
        let focus = Rect::new(0, 24, 640, 336);
        let (cols, rows) = (60, 50);
        for view in [View::Square, View::Tilted] {
            let ch = view.cell_h();
            for s in [1.0f32, 2.0, 3.0, 2.37] {
                for (x, y) in [(0, 0), (cols - 1, 0), (0, rows - 1), (cols - 1, rows - 1)] {
                    let (cx, cy) = camera_px(cell_of(view, 4), focus, (cols, rows), (x as f32, y as f32), s);
                    let near = |a: f32, b: f32, what: &str| assert!((a - b).abs() <= 1.0, "{view:?} {s} ({x}, {y}) {what}: {a} {b}");
                    if x == 0 {
                        near(cx as f32, (focus.x + MARGIN_CELLS * CELL_W) as f32 * s, "left");
                    } else {
                        near(cx as f32 + (cols * CELL_W) as f32 * s, (focus.x + focus.w - MARGIN_CELLS * CELL_W) as f32 * s, "right");
                    }
                    if y == 0 {
                        near(cy as f32, (focus.y + MARGIN_CELLS * ch) as f32 * s, "top");
                    } else {
                        near(cy as f32 + (rows * ch) as f32 * s, (focus.y + focus.h - MARGIN_CELLS * ch) as f32 * s, "bottom");
                    }
                }
            }
        }
    }

    /// Away from the edges the point is on the focus's centre, in the old formula's place at a whole scale; a map that
    /// fits centres in the focus, wherever the point is.
    #[test]
    fn the_camera_centres_inside_the_map_and_a_small_map_centres_in_the_view() {
        let focus = Rect::new(0, 24, 640, 336);
        for view in [View::Square, View::Tilted] {
            let ch = view.cell_h();
            let (cx, cy) = camera_px(cell_of(view, 4), focus, (200, 200), (100.0, 100.0), 2.0);
            let old = MapGeom::centred(view, focus, (100, 100), (100, 100));
            assert_eq!((cx, cy), (old.ox * 2, old.oy * 2), "{view:?}: at rest the old camera");
            let (small_w, small_h) = (16, 10);
            for at in [(0.0, 0.0), (15.0, 9.0), (7.5, 3.0)] {
                let (cx, cy) = camera_px(cell_of(view, 4), focus, (small_w, small_h), at, 1.0);
                assert_eq!(cx, focus.x + (focus.w - small_w * CELL_W) / 2, "{view:?} {at:?}");
                assert_eq!(cy, focus.y + (focus.h - small_h * ch) / 2, "{view:?} {at:?}");
            }
        }
    }

    /// What follows the camera's point stays on one physical px at any fraction of a cell, and the map's cells keep their
    /// shape: each edge is `camera + round(map position x s)`, at whole and fractional scales.
    #[test]
    fn the_map_moves_rigidly_and_the_party_stays_put_while_followed() {
        let focus = Rect::new(0, 24, 640, 336);
        for view in [View::Square, View::Tilted] {
            for (s, q) in [(1.0f32, 4), (2.0, 8), (3.0, 8), (2.37, 8), (2.0, 5), (3.0, 7), (2.37, 11), (2.0, 12)] {
                let mut party = None;
                for i in 0..400 {
                    let at = (50.0 + i as f32 * 0.037, 50.0 + i as f32 * 0.011);
                    let cam = camera_px(cell_of(view, q), focus, (200, 200), at, s);
                    let (g, shift) = placed(view, q, cam, s);
                    let shifted = aegis_ui::painter::View { s, ox: shift.0, oy: shift.1, ..aegis_ui::painter::View::fit(1, 1) };
                    assert!(shift.0.abs() <= s / 2.0 + 0.01 && shift.1.abs() <= s / 2.0 + 0.01, "{s}: under a logical px from the geometry");
                    for (x, y) in [(50, 50), (51, 50), (52, 53)] {
                        let r = shifted.rect(g.cell(x, y));
                        let (cw, ch) = g.cell_size();
                        let want = |c: i32, n: i32, d: i32| c + ((n * d) as f32 * s).round() as i32;
                        assert_eq!((r.x, r.y), (want(cam.0, x, cw), want(cam.1, y, ch)), "{view:?} {s}");
                        assert_eq!((r.x + r.w, r.y + r.h), (want(cam.0, x + 1, cw), want(cam.1, y + 1, ch)), "{view:?} {s}");
                    }
                    let (pr, ps) = party_px(view, q, cam, at, s);
                    let drawn = aegis_ui::painter::View { s, ox: ps.0, oy: ps.1, ..aegis_ui::painter::View::fit(1, 1) }.rect(pr);
                    assert_eq!(*party.get_or_insert(drawn), drawn, "{view:?} {s}: frame {i}");
                    assert_eq!((pr.w, pr.h), cell_of(view, q));
                }
            }
        }
    }

    /// A chain of steps on their schedule is one straight line whatever the frame times; a late frame neither stalls nor
    /// jumps it; a far tile or another location is a jump.
    #[test]
    fn a_glide_is_even_over_chained_steps_and_jumps_when_it_must() {
        let t0 = Instant::now();
        let ms = |m: f64| t0 + std::time::Duration::from_secs_f64(m / 1000.0);
        let mut g = Glide::default();
        assert_eq!(g.update("a", (5, 5), None, Some(t0), Leg::plain(90.0)), ((5.0, 5.0), true), "the first frame");
        let mut tile = 5;
        let mut frame = 0.0;
        while frame < 900.0 {
            let due = (tile - 5) as f64 * 90.0;
            // a step on the first frame at or after its time, as the tick takes it
            let step = (frame >= due).then(|| {
                tile += 1;
                ms(due)
            });
            let (at, jumped) = g.update("a", (tile, 5), step, Some(ms(frame)), Leg::plain(90.0));
            assert!(!jumped);
            assert!((at.0 as f64 - (5.0 + frame / 90.0)).abs() < 1e-3, "at {frame} ms: {}", at.0);
            frame += 16.0 + (frame as i64 % 7) as f64;
        }
        let (at, _) = g.update("a", (tile, 5), None, Some(ms(5000.0)), Leg::plain(90.0));
        assert_eq!(at, (tile as f32, 5.0), "it ends on the tile");
        assert_eq!(g.update("a", (tile + 9, 5), None, Some(ms(5001.0)), Leg::plain(90.0)), (((tile + 9) as f32, 5.0), true), "too far");
        assert_eq!(g.update("b", (tile + 9, 5), None, Some(ms(5002.0)), Leg::plain(90.0)), (((tile + 9) as f32, 5.0), true), "another location");
        assert_eq!(g.update("b", (tile, 5), None, None, Leg::plain(90.0)), ((tile as f32, 5.0), true), "no clock");
    }

    /// A change of look eases from where the camera was to a target that may move, then sits exactly on it; without a
    /// clock, on a snap or on a new view there is no easing.
    #[test]
    fn the_camera_eases_to_a_new_look_and_then_follows_exactly() {
        let t0 = Instant::now();
        let ms = |m: f64| t0 + std::time::Duration::from_secs_f64(m / 1000.0);
        let mut f = Follow::default();
        assert_eq!(f.update(Look::Party, (View::Square, 4), (100, 0), 2.0, Some(t0), false), (100, 0), "the first frame");
        assert_eq!(f.update(Look::Party, (View::Square, 4), (110, 0), 2.0, Some(ms(16.0)), false), (110, 0), "following is exact");
        let pan = Look::Pan((1, 1));
        let first = f.update(pan, (View::Square, 4), (510, 0), 2.0, Some(ms(32.0)), false);
        assert_eq!(first, (110, 0), "starts from where it was");
        let mut was = first.0;
        for i in 1..16 {
            let now = f.update(pan, (View::Square, 4), (510, 0), 2.0, Some(ms(32.0 + i as f64 * 16.0)), false).0;
            assert!(now >= was && now <= 510, "monotonic: {was} then {now}");
            was = now;
        }
        assert!(was > 110 && was < 510, "under way at 240 ms: {was}");
        assert_eq!(f.update(pan, (View::Square, 4), (510, 0), 2.0, Some(ms(32.0 + CAM_EASE_MS)), false).0, 510, "there on time");
        assert_eq!(f.update(pan, (View::Square, 4), (520, 0), 2.0, Some(ms(600.0)), false).0, 520, "and exact after");
        assert_eq!(f.update(Look::Party, (View::Square, 4), (40, 0), 2.0, Some(ms(700.0)), true).0, 40, "a snap");
        assert_eq!(f.update(pan, (View::Tilted, 4), (70, 0), 2.0, Some(ms(800.0)), false).0, 70, "a new view");
        assert_eq!(f.update(Look::Party, (View::Tilted, 4), (90, 0), 2.0, None, false).0, 90, "no clock");
    }

    /// The zoom's setting: a number to the nearest step in range, the default otherwise; every step's cell, and so every
    /// rect taken from it, is whole logical px; the wall's cap ends where its face starts at each.
    #[test]
    fn the_zoom_reads_its_setting_and_every_step_is_whole_px() {
        let (least, most, default) = ZOOM_QUARTERS;
        assert_eq!(zoom_of(&Value::Null), default, "missing");
        assert_eq!(zoom_of(&json!("big")), default, "not a number");
        assert_eq!(zoom_of(&crate::screens::default_settings()["map_zoom"]), default, "the default setting");
        assert_eq!(default, 4, "1x: an accessibility setting, today's view by default");
        assert_eq!(least, 4, "nothing below 1x");
        assert_eq!(zoom_of(&json!(0.1)), least, "under the range");
        assert_eq!(zoom_of(&json!(99)), most, "over it");
        assert_eq!(zoom_of(&json!(1.3)), 5, "to the nearest step");
        let steps: Vec<f64> = zoom_steps().collect();
        assert_eq!(steps.len() as i32, most - least + 1);
        assert!(steps.windows(2).all(|p| p[1] - p[0] == 0.25), "in quarters");
        for z in steps {
            let q = zoom_of(&json!(z));
            assert_eq!(q as f64 / 4.0, z, "a step round trips");
            for view in [View::Square, View::Tilted] {
                let (cw, ch) = cell_of(view, q);
                assert_eq!((cw * 4, ch * 4), (CELL_W * q, view.cell_h() * q), "{view:?} {z}: whole px");
                let c = Rect::new(0, 0, cw, ch);
                assert_eq!(cap_rect(c).y + cap_rect(c).h, face_rect(c).y, "{view:?} {z}");
                assert_eq!(block_rect(c, true), union(cap_rect(c), face_rect(c)), "{view:?} {z}");
                assert_eq!((sprite_rect(c).w * 4, icon_rect(c).w * 4), (SPRITE * q, ICON * q), "{view:?} {z}");
            }
        }
    }

    #[test]
    fn union_holds_both_rects() {
        let (a, b) = (Rect::new(10, 20, 16, 12), Rect::new(6, 8, 24, 24));
        assert_eq!(union(a, b), Rect::new(6, 8, 24, 24));
        assert_eq!(union(b, a), union(a, b));
        assert_eq!(union(a, Rect::new(30, 40, 2, 2)), Rect::new(10, 20, 22, 22));
    }

    /// VIEWKEYS1: the step time is the base over the animation speed, at every speed the Settings slider gives.
    #[test]
    fn the_walk_step_is_the_base_over_the_animation_speed() {
        let default = WALK_STEP_MS.2;
        for speed in [0.5, 1.0, 2.0] {
            let s = WalkStep::read(&json!({"anim_speed": speed}), false, true);
            assert_eq!(s, WalkStep { base: default, wall: default / speed }, "{speed}");
            assert_eq!(s.gap().as_secs_f64(), default / speed / 1000.0, "{speed}");
            let tuned = WalkStep::read(&json!({"anim_speed": speed, WALK_STEP_KEY: 150.0}), false, true);
            assert_eq!(tuned.wall, 150.0 / speed, "the slider's value: {speed}");
        }
        assert!(WalkStep::read(&json!({"anim_speed": 2.0}), false, true).wall < WalkStep::read(&json!({"anim_speed": 0.5}), false, true).wall, "faster animation walks faster");
        assert_eq!(WalkStep::read(&json!({"anim_speed": 99}), false, true).wall, default / 2.0, "the sprite's clamp");
        assert_eq!(WalkStep::read(&json!({"anim_speed": "fast"}), false, true).wall, default, "no number is 1x");
    }

    /// WALKRUN1: walk 200 ms and run 130 ms over the animation speed; the debug setting moves the walk only, and a run
    /// needs no debug build and takes no bad value.
    #[test]
    fn a_run_steps_at_its_own_time_at_every_speed() {
        assert_eq!((WALK_STEP_MS.2, RUN_STEP_MS), (200.0, 130.0));
        for speed in [0.5, 1.0, 2.0] {
            let v = json!({"anim_speed": speed});
            assert_eq!(WalkStep::read(&v, false, false), WalkStep { base: 200.0, wall: 200.0 / speed }, "walk at {speed}");
            for debug in [false, true] {
                assert_eq!(WalkStep::read(&v, true, debug), WalkStep { base: 130.0, wall: 130.0 / speed }, "run at {speed}");
            }
            let tuned = json!({"anim_speed": speed, WALK_STEP_KEY: 90.0});
            assert_eq!(WalkStep::read(&tuned, false, true).base, 90.0, "the slider and F11 set the walk");
            assert_eq!(WalkStep::read(&tuned, true, true).base, RUN_STEP_MS, "and not the run");
        }
        assert_eq!(WalkStep::read(&json!({WALK_STEP_KEY: "slow"}), true, true).base, RUN_STEP_MS, "a bad saved walk");
        assert_eq!(WalkStep::read(&json!({WALK_STEP_KEY: "slow"}), false, true).base, WALK_STEP_MS.2, "a bad saved walk");
    }

    /// WALKRUN1: a step glides in the time it started with: a change of pace during it or at the next step never moves
    /// the party back (the drawn position only goes forward through a run, a walk and a run again).
    #[test]
    fn a_change_of_pace_keeps_the_glide_going_forward() {
        let t0 = Instant::now();
        let at = |ms: u64| Some(t0 + Duration::from_millis(ms));
        let mut g = Glide::default();
        assert_eq!(g.update("m", (0, 0), None, at(0), Leg::plain(130.0)).0, (0.0, 0.0));
        // a run's step, half way
        assert_eq!(g.update("m", (1, 0), at(0), at(65), Leg::plain(130.0)).0, (0.5, 0.0));
        // Shift let go during it: the step keeps its own time
        assert_eq!(g.update("m", (1, 0), None, at(65), Leg::plain(200.0)).0, (0.5, 0.0), "no hop back when the pace changes");
        assert_eq!(g.update("m", (1, 0), None, at(130), Leg::plain(200.0)).0, (1.0, 0.0), "it ends on its time");
        // the next step is a walk's, from the tile
        assert_eq!(g.update("m", (2, 0), at(130), at(130), Leg::plain(200.0)).0, (1.0, 0.0), "from the tile, not from behind it");
        assert_eq!(g.update("m", (2, 0), None, at(230), Leg::plain(200.0)).0, (1.5, 0.0));
        // Shift pressed during it: no hop forward
        assert_eq!(g.update("m", (2, 0), None, at(230), Leg::plain(130.0)).0, (1.5, 0.0), "no hop on when the pace changes");
    }

    /// TURNFEEL1: a beat only on a perpendicular turn, in animation ms; off (and in a release build) a leg is the pace's.
    #[test]
    fn a_leg_has_a_beat_on_a_perpendicular_turn_only() {
        let (up, down, left, right) = ((0, -1), (0, 1), (-1, 0), (1, 0));
        for (from, to) in [(up, left), (up, right), (down, left), (down, right), (left, up), (left, down), (right, up), (right, down)] {
            assert!(is_turn(from, to), "{from:?} {to:?}");
        }
        for (from, to) in [(up, up), (up, down), (down, up), (left, right), (right, left), (right, right)] {
            assert!(!is_turn(from, to), "{from:?} {to:?}");
        }
        let beat = WalkFeel { beat: true, equal: false, corner: TurnCorner::Cut };
        for speed in [0.5, 1.0, 2.0] {
            let step = WalkStep::read(&json!({"anim_speed": speed}), false, true);
            let turn = beat.leg(step, View::Square, up, right);
            assert_eq!(turn.delay, TURN_BEAT_MS / speed, "the animation clock at {speed}x");
            assert_eq!(turn.anim, step.base + TURN_BEAT_MS);
            assert_eq!(turn.gap().as_secs_f64(), (step.wall + TURN_BEAT_MS / speed) / 1000.0, "the next step waits");
            assert_eq!(turn.glide, step.wall, "the glide keeps its time");
            for (from, to) in [(up, down), (right, right), (left, right)] {
                assert_eq!(beat.leg(step, View::Square, from, to).delay, 0.0, "{from:?} {to:?}");
            }
            let off = WalkFeel::default().leg(step, View::Tilted, up, right);
            assert_eq!(off, Leg::plain(step.wall).with_anim(step.base), "both off");
            assert_eq!(off.gap(), step.gap(), "today's schedule");
        }
        assert_eq!(TURN_BEAT_MS, 60.0, "the brief's default");
    }

    /// TURNFEEL1: with equal speed a step's px per ms is the same both ways in the tilted view at every zoom and pace, and
    /// is the square view's; on square cells it changes nothing.
    #[test]
    fn equal_speed_makes_the_screen_speed_the_same_every_way() {
        let (up, right) = ((0, -1), (1, 0));
        let feel = WalkFeel { beat: false, equal: true, corner: TurnCorner::Cut };
        for run in [false, true] {
            let step = WalkStep::read(&json!({}), run, true);
            for q in ZOOM_QUARTERS.0..=ZOOM_QUARTERS.1 {
                let (cw, ch) = cell_of(View::Tilted, q);
                let across = cw as f64 / feel.leg(step, View::Tilted, right, right).glide;
                let along = ch as f64 / feel.leg(step, View::Tilted, up, up).glide;
                assert!((across - along).abs() < 1e-9, "{q}: {across} {along}");
                let plain = WalkFeel::default();
                let was = (cw as f64 / plain.leg(step, View::Tilted, right, right).glide, ch as f64 / plain.leg(step, View::Tilted, up, up).glide);
                assert!((was.0 * 3.0 - was.1 * 4.0).abs() < 1e-9, "off: a third faster across than along");
                let square = cell_of(View::Square, q).0 as f64 / step.wall;
                assert!((across - square).abs() < 1e-9, "the square view's speed: {q}");
            }
            for dir in [up, right] {
                assert_eq!(feel.leg(step, View::Square, dir, dir), WalkFeel::default().leg(step, View::Square, dir, dir), "square cells");
            }
            let tilted = feel.leg(step, View::Tilted, up, up);
            assert_eq!(tilted.gap().as_secs_f64(), tilted.glide / 1000.0, "the next step comes as the glide ends");
            assert_eq!(tilted.anim, step.base * 0.75);
        }
    }

    fn corner_feel(corner: TurnCorner) -> WalkFeel {
        WalkFeel { beat: false, equal: false, corner }
    }

    /// Walks up from (5, 5), takes `steps` at the given times, and returns the path drawn each 4 ms to `until`.
    fn drawn_path(feel: WalkFeel, base: f64, steps: &[((i64, i64), f64)], until: f64) -> Vec<(f64, (f32, f32))> {
        let t0 = Instant::now();
        let ms = |m: f64| t0 + Duration::from_secs_f64(m / 1000.0);
        let step = WalkStep { base, wall: base };
        let mut g = Glide::default();
        g.update("m", (5, 5), None, Some(t0), Leg::plain(base));
        let mut tile = (5, 4);
        let mut facing = (0, -1);
        let first = feel.leg(step, View::Square, facing, facing);
        assert_eq!(g.corner_step("m", (5, 5), tile, t0, first), None, "from rest");
        g.update("m", tile, Some(t0), Some(t0), first);
        let mut out = Vec::new();
        let mut now = 0.0;
        let mut i = 0;
        while now <= until {
            while i < steps.len() && steps[i].1 <= now {
                let (dir, at) = steps[i];
                let leg = feel.leg(step, View::Square, facing, dir);
                let was = tile;
                tile = (tile.0 + dir.0, tile.1 + dir.1);
                facing = dir;
                let handled = g.corner_step("m", was, tile, ms(at), leg).is_some();
                assert!(!g.update("m", tile, (!handled).then(|| ms(at)), Some(ms(now)), leg).1, "no jump");
                i += 1;
            }
            out.push((now, g.update("m", tile, None, Some(ms(now)), Leg::plain(base)).0));
            now += 4.0;
        }
        out
    }

    /// Whether the path is on the column x = 5 down to the corner row, then on row `y`: no diagonal.
    fn is_clean_l(path: &[(f64, (f32, f32))], y: f32) -> bool {
        path.iter().all(|(_, p)| p.0 == 5.0 || p.1 == y)
    }

    /// TURNFEEL1: with finish and quick, a press at any point in a leg draws an L at walk and run pace, and ends on the new
    /// tile; cut draws the diagonal (what the options are there to compare).
    #[test]
    fn a_clean_corner_draws_an_l_and_a_cut_does_not() {
        for base in [200.0, RUN_STEP_MS] {
            for phase in [0.05, 0.25, 0.5, 0.75, 0.95] {
                let press = base * phase;
                for corner in [TurnCorner::Finish, TurnCorner::Quick] {
                    let path = drawn_path(corner_feel(corner), base, &[((1, 0), press)], 1000.0);
                    assert!(is_clean_l(&path, 4.0), "{corner:?} {base} {phase}");
                    assert_eq!(path.last().unwrap().1, (6.0, 4.0), "{corner:?} ends on the tile");
                }
                let cut = drawn_path(corner_feel(TurnCorner::Cut), base, &[((1, 0), press)], 1000.0);
                assert!(!is_clean_l(&cut, 4.0), "cut {base} {phase}");
                assert_eq!(cut.last().unwrap().1, (6.0, 4.0));
            }
        }
    }

    /// When the new way starts: the finish at the leg's end, the quick finish at most `QUICK_FINISH_MS` after the press, and
    /// no slower than the leg (a press near the end keeps the leg's own speed); then a full glide.
    #[test]
    fn the_new_way_starts_when_the_setting_says() {
        let base = 200.0;
        let start = |path: &[(f64, (f32, f32))]| path.iter().find(|(_, p)| p.0 > 5.0).expect("it turns").0;
        for (press, finish, quick) in [(40.0, 200.0, 100.0), (100.0, 200.0, 160.0), (180.0, 200.0, 200.0)] {
            let at = |c| start(&drawn_path(corner_feel(c), base, &[((1, 0), press)], 800.0));
            assert!((at(TurnCorner::Finish) - finish).abs() <= 4.0, "finish at {press}: {}", at(TurnCorner::Finish));
            assert!((at(TurnCorner::Quick) - quick).abs() <= 4.0, "quick at {press}: {}", at(TurnCorner::Quick));
        }
        let leg = corner_feel(TurnCorner::Quick).leg(WalkStep { base: 200.0, wall: 100.0 }, View::Square, (0, -1), (1, 0));
        assert_eq!(leg.finish_cap, QUICK_FINISH_MS / 2.0, "the cap is on the animation clock");
    }

    /// Straight walking and a reversal with no turn pending draw the same under every setting; a reversal during a finish
    /// goes back to the corner it had not reached, and no further; quick turns stack to no jump and end on the tile.
    #[test]
    fn straight_steps_and_reversals_stay_as_they_are() {
        for steps in [vec![((0, -1), 80.0), ((0, -1), 280.0)], vec![((0, 1), 80.0)], vec![((0, 1), 80.0), ((0, -1), 120.0)]] {
            let cut = drawn_path(corner_feel(TurnCorner::Cut), 200.0, &steps, 700.0);
            for corner in [TurnCorner::Finish, TurnCorner::Quick] {
                assert_eq!(cut, drawn_path(corner_feel(corner), 200.0, &steps, 700.0), "{corner:?} {steps:?}");
            }
        }
        let back = drawn_path(corner_feel(TurnCorner::Finish), 200.0, &[((1, 0), 100.0), ((-1, 0), 150.0)], 600.0);
        assert!(back.iter().all(|(_, p)| p.0 == 5.0), "never leaves the column");
        assert_eq!(back.last().unwrap().1, (5.0, 4.0));
        // a staircase of presses 30 ms apart: more cells behind than any reach, still a glide, and the drawn lag is bounded
        let stairs: Vec<((i64, i64), f64)> = (0..16).map(|i| (if i % 2 == 0 { (1, 0) } else { (0, -1) }, 40.0 + i as f64 * 30.0)).collect();
        for corner in [TurnCorner::Finish, TurnCorner::Quick] {
            let path = drawn_path(corner_feel(corner), 200.0, &stairs, 2000.0);
            assert_eq!(path.last().unwrap().1, (13.0, -4.0), "{corner:?}");
            let most = path.windows(2).map(|w| cells_apart(w[0].1, w[1].1)).fold(0.0, f32::max);
            assert!(most <= 1.0, "{corner:?}: {most} cells in a 4 ms frame is a jump");
        }
    }

    /// During a finish the sprite faces the way it is going; from the corner on, the new way.
    #[test]
    fn the_sprite_turns_at_the_corner() {
        let t0 = Instant::now();
        let ms = |m: f64| t0 + Duration::from_millis(m as u64);
        let feel = corner_feel(TurnCorner::Finish);
        let step = WalkStep { base: 200.0, wall: 200.0 };
        let mut g = Glide::default();
        g.update("m", (5, 5), None, Some(t0), Leg::plain(200.0));
        g.update("m", (5, 4), Some(t0), Some(t0), feel.leg(step, View::Square, (0, -1), (0, -1)));
        assert_eq!(g.heading(t0), None, "no turn yet");
        assert_eq!(g.corner_step("m", (5, 4), (6, 4), ms(50.0), feel.leg(step, View::Square, (0, -1), (1, 0))), Some(150.0));
        assert_eq!(g.heading(ms(50.0)), Some((0, -1)));
        assert_eq!(g.heading(ms(199.0)), Some((0, -1)));
        assert_eq!(g.heading(ms(200.0)), None);
    }

    /// TURNFEEL1: a release build reads every option off; only `true` is on; an unknown corner is the cut.
    #[test]
    fn the_feel_counts_in_a_debug_build_only() {
        let on = json!({TURN_BEAT_KEY: true, EQUAL_SPEED_KEY: true, TURN_CORNER_KEY: "quick"});
        assert_eq!(WalkFeel::read(&on, true).corner, TurnCorner::Quick);
        for c in TURN_CORNERS {
            assert_eq!(TurnCorner::of_setting(&json!(c.name())), c);
        }
        for bad in [json!("sharp"), json!(2), Value::Null] {
            assert_eq!(TurnCorner::of_setting(&bad), TurnCorner::Cut, "{bad}");
        }
        let on = json!({TURN_BEAT_KEY: true, EQUAL_SPEED_KEY: true});
        assert_eq!(WalkFeel::read(&on, true), WalkFeel { beat: true, equal: true, corner: TurnCorner::Cut });
        assert_eq!(WalkFeel::read(&on, false), WalkFeel::default(), "a release build");
        assert_eq!(WalkFeel::read(&json!({TURN_BEAT_KEY: 1, EQUAL_SPEED_KEY: "true"}), true), WalkFeel::default());
        assert_eq!(WalkFeel::read(&crate::screens::default_settings(), true), WalkFeel::default(), "the settings' default");
    }

    /// TURNFEEL1: with a delay the drawn position stands where the party had got to, then glides its full time from there;
    /// the camera's `jumped` stays false; stacked turns stay a glide with the beat on.
    #[test]
    fn a_beat_stands_then_glides_with_no_jump() {
        let t0 = Instant::now();
        let at = |ms: u64| Some(t0 + Duration::from_millis(ms));
        let step = WalkStep::read(&json!({}), false, true);
        let beat = WalkFeel { beat: true, equal: false, corner: TurnCorner::Cut };
        let leg = beat.leg(step, View::Square, (0, -1), (1, 0));
        let mut g = Glide::default();
        g.update("m", (5, 5), None, at(0), Leg::plain(200.0));
        // from rest: stands through the beat, half way at beat + 100, there at beat + 200
        assert_eq!(g.update("m", (6, 5), at(0), at(0), leg), ((5.0, 5.0), false));
        assert_eq!(g.update("m", (6, 5), None, at(59), leg).0, (5.0, 5.0), "still in the beat");
        assert_eq!(g.update("m", (6, 5), None, at(60), leg).0, (5.0, 5.0), "the glide starts");
        assert_eq!(g.update("m", (6, 5), None, at(160), leg).0, (5.5, 5.0));
        assert_eq!(g.update("m", (6, 5), None, at(260), leg).0, (6.0, 5.0), "its full time");
        // a straight step on: no stand, and from the tile
        let on = beat.leg(step, View::Square, (1, 0), (1, 0));
        assert_eq!(g.update("m", (7, 5), at(260), at(310), on).0, (6.25, 5.0));
        // a press half way: stands where it had got to, not on a tile
        let turn = beat.leg(step, View::Square, (1, 0), (0, 1));
        assert_eq!(g.update("m", (7, 6), at(360), at(360), turn).0, (6.5, 5.0));
        assert_eq!(g.update("m", (7, 6), None, at(419), turn).0, (6.5, 5.0));
        assert_eq!(g.update("m", (7, 6), None, at(620), turn).0, (7.0, 6.0));
        // three quick turns inside one beat: more cells behind than the plain reach, still a glide
        let mut x = Glide::default();
        x.update("m", (0, 0), None, at(0), Leg::plain(200.0));
        for (i, tile) in [(1, 0), (1, 1), (2, 1)].into_iter().enumerate() {
            let (_, jumped) = x.update("m", tile, at(i as u64 * 10), at(i as u64 * 10), turn);
            assert!(!jumped, "{tile:?}");
        }
        let mut y = Glide::default();
        y.update("m", (0, 0), None, at(0), Leg::plain(200.0));
        y.update("m", (9, 0), None, at(10), turn);
        assert_eq!(y.position(t0 + Duration::from_millis(10)), (9.0, 0.0), "a tile far off is still a jump");
    }

    /// A bad or missing saved base is the default; the range's ends are good; a release build reads the default always.
    #[test]
    fn a_bad_walk_step_falls_back_to_the_default() {
        let (least, most, default) = WALK_STEP_MS;
        let base = |v: Value, debug: bool| WalkStep::read(&json!({WALK_STEP_KEY: v}), false, debug).base;
        assert_eq!(WalkStep::read(&json!({}), false, true).base, default, "missing");
        for bad in [json!("slow"), json!(null), json!(true), json!(least - 1.0), json!(most + 1.0), json!(-5), json!(1e300)] {
            assert_eq!(base(bad.clone(), true), default, "{bad}");
        }
        assert_eq!(base(json!(least), true), least);
        assert_eq!(base(json!(most), true), most);
        assert_eq!(base(json!(most), false), default, "a release build");
        assert_eq!(crate::screens::default_settings()[WALK_STEP_KEY], json!(default), "the settings' default");
    }
}
