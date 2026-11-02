//! The region map's two views (EXPLOREVIEW1): the debug setting `region_view` and the geometry both views share, pure
//! functions with no drawing. Square is the map seen from above (16 x 16 cells); Tilted is the battle stage's look at
//! half size (16 x 12 cells, no diamonds): `region.rs` and `props.rs` take every map position from `MapGeom`. A view
//! change only: rules, walking, saves and journeys never read it. In the square view every result is the formula the
//! map used before this module.

use aegis_ui::canvas::Rect;
use serde_json::Value;

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
pub const FACE_H: i32 = 6;
/// The shade a wall's face drops on the cell south.
pub const SHADE_H: i32 = 2;
/// How far water sinks: the band over its top is this plus one row (STAGE1 3.2 at half size).
pub const LIP_DEPTH: i32 = 2;

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
}

impl MapGeom {
    /// The camera: the midpoint of the centres of cells `one` and `other` (the party; the party and the speaker; the
    /// pan) on the centre of `focus`.
    pub fn centred(view: View, focus: Rect, one: (i64, i64), other: (i64, i64)) -> MapGeom {
        let ch = view.cell_h();
        let ox = focus.x + focus.w / 2 - ((one.0 + other.0) as i32 * CELL_W + CELL_W) / 2;
        let oy = focus.y + focus.h / 2 - ((one.1 + other.1) as i32 * ch + ch) / 2;
        MapGeom { view, ox, oy }
    }

    /// The cell's size (w, h).
    pub fn cell_size(&self) -> (i32, i32) {
        (CELL_W, self.view.cell_h())
    }

    /// Cell (x, y)'s rect.
    pub fn cell(&self, x: i32, y: i32) -> Rect {
        let ch = self.view.cell_h();
        Rect::new(self.ox + x * CELL_W, self.oy + y * ch, CELL_W, ch)
    }

    /// `cell` of a map position.
    pub fn cell_of(&self, at: (i64, i64)) -> Rect {
        self.cell(at.0 as i32, at.1 as i32)
    }

    /// The cell under the point (mx, my), which may be off the map.
    pub fn cell_at(&self, mx: i32, my: i32) -> (i64, i64) {
        ((mx - self.ox).div_euclid(CELL_W) as i64, (my - self.oy).div_euclid(self.view.cell_h()) as i64)
    }

    /// The columns (first, last) meeting `area`.
    pub fn cols(&self, area: Rect) -> (i32, i32) {
        ((area.x - self.ox).div_euclid(CELL_W), (area.x + area.w - self.ox - 1).div_euclid(CELL_W))
    }

    /// The rows (first, last) to draw for `area`: those meeting it; tilted, one more below (its lifted cap reaches in).
    pub fn rows(&self, area: Rect) -> (i32, i32) {
        let ch = self.view.cell_h();
        let last = (area.y + area.h - self.oy - 1).div_euclid(ch);
        ((area.y - self.oy).div_euclid(ch), if self.view == View::Tilted { last + 1 } else { last })
    }
}

impl Default for MapGeom {
    fn default() -> Self {
        MapGeom { view: View::Square, ox: 0, oy: 0 }
    }
}

/// A sprite on `cell`: `SPRITE` square, bottom-centred on the cell's bottom edge.
pub fn sprite_rect(cell: Rect) -> Rect {
    Rect::new(cell.x + (cell.w - SPRITE) / 2, cell.y + cell.h - SPRITE, SPRITE, SPRITE)
}

/// A marker icon (or its stand-in) standing on `cell` (in the square view, the cell itself).
pub fn icon_rect(cell: Rect) -> Rect {
    Rect::new(cell.x, cell.y + cell.h - ICON, ICON, ICON)
}

/// A wall's cap on `cell` (tilted): a cell lifted by `BLOCK_LIFT`.
pub fn cap_rect(cell: Rect) -> Rect {
    Rect::new(cell.x, cell.y - BLOCK_LIFT, cell.w, cell.h)
}

/// A wall's south face on `cell` (tilted): the cell's bottom `FACE_H`.
pub fn face_rect(cell: Rect) -> Rect {
    Rect::new(cell.x, cell.y + cell.h - FACE_H, cell.w, FACE_H)
}

/// A wall's drawn rect on `cell` (tilted): its cap, and its face when it has one (an open cell south).
pub fn block_rect(cell: Rect, face: bool) -> Rect {
    Rect::new(cell.x, cell.y - BLOCK_LIFT, cell.w, cell.h + if face { BLOCK_LIFT } else { 0 })
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
            let g = MapGeom { view: View::Square, ox, oy };
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
            let g = MapGeom { view: View::Tilted, ox, oy };
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
        let g = MapGeom { view: View::Tilted, ox: 0, oy: 24 };
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
            let g = MapGeom { view, ox: 7, oy: 50 };
            let cell = g.cell(4, 3);
            let s = sprite_rect(cell);
            assert_eq!((s.w, s.h), (SPRITE, SPRITE));
            assert_eq!(s.y + s.h, cell.y + cell.h);
            assert_eq!(s.x + s.w / 2, cell.x + cell.w / 2);
            assert_eq!(g.cell_at(s.x + s.w / 2, s.y + s.h - 1), (4, 3));
            let i = icon_rect(cell);
            assert_eq!((i.w, i.h, i.y + i.h, i.x), (ICON, ICON, cell.y + cell.h, cell.x));
        }
        let cell = MapGeom { view: View::Tilted, ox: 0, oy: 0 }.cell(0, 5);
        assert_eq!(cell.y - sprite_rect(cell).y, 12, "the head rises 12 px into the row north");
    }

    /// 3.2: the cap is lifted, the face fills the cell's bottom, the drawn rect holds both.
    #[test]
    fn a_block_is_a_lifted_cap_and_a_south_face() {
        let cell = MapGeom { view: View::Tilted, ox: 0, oy: 0 }.cell(2, 2);
        let (cap, face) = (cap_rect(cell), face_rect(cell));
        assert_eq!(cap, Rect::new(cell.x, cell.y - BLOCK_LIFT, 16, 12));
        assert_eq!(face, Rect::new(cell.x, cell.y + 6, 16, FACE_H));
        assert_eq!(cap.y + cap.h, face.y, "the face starts where the cap ends");
        assert_eq!(face.y + face.h, cell.y + cell.h, "and ends on the cell's bottom edge");
        assert_eq!(block_rect(cell, true), union(cap, face));
        assert_eq!(block_rect(cell, false), cap);
    }

    #[test]
    fn union_holds_both_rects() {
        let (a, b) = (Rect::new(10, 20, 16, 12), Rect::new(6, 8, 24, 24));
        assert_eq!(union(a, b), Rect::new(6, 8, 24, 24));
        assert_eq!(union(b, a), union(a, b));
        assert_eq!(union(a, Rect::new(30, 40, 2, 2)), Rect::new(10, 20, 22, 22));
    }
}
