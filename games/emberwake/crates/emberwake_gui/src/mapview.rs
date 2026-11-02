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
use std::time::Instant;

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

/// The party's drawn position (REGIONVIEW1): straight from where it was to its tile in one step's time. A step's start is
/// its scheduled time, not the frame that noticed it, so steps every `STEP_MS` make one even line. Presentation only.
#[derive(Debug, Clone, Default)]
pub struct Glide {
    loc: String,
    to: Option<(i64, i64)>,
    from: (f32, f32),
    at: Option<Instant>,
}

impl Glide {
    /// The furthest the party glides (cells): one step, and one more if a key press met a step in the same frame.
    const REACH: f32 = 2.0;

    fn rest(&mut self, loc: &str, tile: (i64, i64)) -> ((f32, f32), bool) {
        let here = (tile.0 as f32, tile.1 as f32);
        *self = Glide { loc: loc.to_string(), to: Some(tile), from: here, at: None };
        (here, true)
    }

    fn position(&self, now: Instant, step_ms: f64) -> (f32, f32) {
        let (Some(to), Some(at)) = (self.to, self.at) else { return self.from };
        let t = (now.saturating_duration_since(at).as_secs_f64() * 1000.0 / step_ms).clamp(0.0, 1.0) as f32;
        (self.from.0 + (to.0 as f32 - self.from.0) * t, self.from.1 + (to.1 as f32 - self.from.1) * t)
    }

    /// The drawn position at `now` (None: no clock yet, so no time has passed: the tile), and whether it jumped: a first
    /// frame, a new location, or a tile too far to glide to (the camera then jumps with it). `step_at`: the scheduled time
    /// of a step taken since the last frame.
    pub fn update(&mut self, loc: &str, tile: (i64, i64), step_at: Option<Instant>, now: Option<Instant>, step_ms: f64) -> ((f32, f32), bool) {
        let Some(now) = now else { return self.rest(loc, tile) };
        if self.to.is_none() || self.loc != loc {
            return self.rest(loc, tile);
        }
        if self.to != Some(tile) {
            let start = step_at.unwrap_or(now);
            let was = self.position(start, step_ms);
            if (tile.0 as f32 - was.0).abs() + (tile.1 as f32 - was.1).abs() > Self::REACH {
                return self.rest(loc, tile);
            }
            (self.from, self.to, self.at) = (was, Some(tile), Some(start));
        }
        (self.position(now, step_ms), false)
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

    const SWEEP: (i32, i32) = (48, 30);

    fn cells() -> impl Iterator<Item = (i32, i32)> {
        (0..SWEEP.1).flat_map(|y| (0..SWEEP.0).map(move |x| (x, y)))
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
                    let shifted = aegis_ui::painter::View { s, ox: shift.0, oy: shift.1, g: 1 };
                    assert!(shift.0.abs() <= s / 2.0 + 0.01 && shift.1.abs() <= s / 2.0 + 0.01, "{s}: under a logical px from the geometry");
                    for (x, y) in [(50, 50), (51, 50), (52, 53)] {
                        let r = shifted.rect(g.cell(x, y));
                        let (cw, ch) = g.cell_size();
                        let want = |c: i32, n: i32, d: i32| c + ((n * d) as f32 * s).round() as i32;
                        assert_eq!((r.x, r.y), (want(cam.0, x, cw), want(cam.1, y, ch)), "{view:?} {s}");
                        assert_eq!((r.x + r.w, r.y + r.h), (want(cam.0, x + 1, cw), want(cam.1, y + 1, ch)), "{view:?} {s}");
                    }
                    let (pr, ps) = party_px(view, q, cam, at, s);
                    let drawn = aegis_ui::painter::View { s, ox: ps.0, oy: ps.1, g: 1 }.rect(pr);
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
        assert_eq!(g.update("a", (5, 5), None, Some(t0), 90.0), ((5.0, 5.0), true), "the first frame");
        let mut tile = 5;
        let mut frame = 0.0;
        while frame < 900.0 {
            let due = (tile - 5) as f64 * 90.0;
            // a step on the first frame at or after its time, as the tick takes it
            let step = (frame >= due).then(|| {
                tile += 1;
                ms(due)
            });
            let (at, jumped) = g.update("a", (tile, 5), step, Some(ms(frame)), 90.0);
            assert!(!jumped);
            assert!((at.0 as f64 - (5.0 + frame / 90.0)).abs() < 1e-3, "at {frame} ms: {}", at.0);
            frame += 16.0 + (frame as i64 % 7) as f64;
        }
        let (at, _) = g.update("a", (tile, 5), None, Some(ms(5000.0)), 90.0);
        assert_eq!(at, (tile as f32, 5.0), "it ends on the tile");
        assert_eq!(g.update("a", (tile + 9, 5), None, Some(ms(5001.0)), 90.0), (((tile + 9) as f32, 5.0), true), "too far");
        assert_eq!(g.update("b", (tile + 9, 5), None, Some(ms(5002.0)), 90.0), (((tile + 9) as f32, 5.0), true), "another location");
        assert_eq!(g.update("b", (tile, 5), None, None, 90.0), ((tile as f32, 5.0), true), "no clock");
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
}
