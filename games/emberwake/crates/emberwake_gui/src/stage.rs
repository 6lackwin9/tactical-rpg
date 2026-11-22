//! The battle stage (STAGE1 option B, game-only, new in the engine port): the board tilted to 32 x 24 cells so the
//! upright side-view sprites belong on it. Its geometry (cells, foot and chest points, token rects, the point-to-cell
//! map), the terrain stage table (ground, sunken, standing, block), the ground under a standing prop, the lips of
//! sunken cells, the blocks, props, backdrop and front face, the draw passes, the ground arrows, the feet-sorted order
//! with the outlines drawn through what covers a unit, and the units' hit rects. The pieces of the board itself are
//! drawn here; `battle/board.rs` runs the passes and draws the units and markers, `battle/draw.rs` the HUD. A view change only: rules, grid,
//! plans and every battle number stay as they are. The Flat view (the board seen from above, as before) stays a
//! setting until the user's review (STAGE1-C removes it). LOOK1-L1: the world's stage set (`stage_set`, LOOK1 5.2-5.4)
//! over `common_stage`, and its transition pieces on the tile corners. PR5-33 (LAYER-UP-DATA): the stage table is the
//! terrain rows' look fields (`relief`, `relief_depth`, `prop`, `color`, `face`; `emberwake_data::looks::TerrainLook`),
//! which the grid's terrain table carries per type; no table here names a terrain id.

use crate::tuning::Tuning;
use aegis_ui::art::Art;
use aegis_ui::autotile::corner_mask;
use aegis_ui::canvas::{rgb, rgba, Image, Rect, Rgba};
use aegis_ui::depth::DepthList;
use aegis_ui::painter::Painter;
use emberwake_battle::grid::{data_zone, Grid, V2, GRID_H, GRID_W};
use indexmap::IndexMap;
use emberwake_content::art::{battle_backdrop, battle_prop_dirs, battle_stage_dirs, transition_piece, zone_backdrop, BATTLE_SETS};
use emberwake_data::ZoneDef;
use serde_json::Value;

/// The grid panel's interior (every stage pass is clipped to it).
pub const INTERIOR: Rect = Rect::new(128, 30, 384, 288);
/// The zone backdrop over the board's far edge.
pub const BACKDROP: Rect = Rect::new(128, 30, 384, 52);
/// DIORAMA2 R4: the stage's clip inside the board panel's frame (the panel `Rect(98, 28, 444, 290)` inset 2): the
/// backdrop, floor and front face reach into the side gutters (`WING` px each side) and fade there, and nothing of the
/// stage covers the panel's rim.
pub const STAGE_CLIP: Rect = Rect::new(100, 30, 440, 286);
pub const WING: i32 = INTERIOR.x - STAGE_CLIP.x;
pub const BOARD_X: i32 = 128;
pub const BOARD_Y: i32 = 82;
pub const CELL_W: i32 = 32;
pub const CELL_H: i32 = 24;
/// The cells' area (the transition pieces stay inside it).
pub const BOARD: Rect = Rect::new(BOARD_X, BOARD_Y, GRID_W * CELL_W, GRID_H * CELL_H);
/// LOOK1 5.1: a stage set's art holds 2 art px per logical px.
pub const SET_ART_PX: i32 = 2;
/// The front face under the near edge (`FACE_H` tall) and the drop shadow under it.
pub const FACE_H: i32 = 12;
/// DIORAMA1 D4: the wall foot's height (the top of a row 0 cell, where the backdrop meets the board).
pub const WALL_FOOT_H: i32 = 8;
pub const FRONT_FACE: Rect = Rect::new(128, BOARD_Y + GRID_H * CELL_H, 384, FACE_H);
pub const DROP_SHADOW: Rect = Rect::new(128, BOARD_Y + GRID_H * CELL_H + FACE_H, 384, 8);
/// How far a block's cap is lifted above its cell.
pub const BLOCK_LIFT: i32 = 12;
/// DIORAMA1B (D5): how many cells (`CELL_W`) a body with a `foot` row stands per footprint cell, its frame's top to
/// its foot row (contract 8.1: about 1.15; a 2 x 2 unit about 2.3).
pub const PAINTED_CELLS: f32 = 1.15;
/// DIORAMA1B (D5), PAINTED-MAPS1: the sprite square of a body with `frame` and a `foot` row standing `PAINTED_CELLS`
/// tiles `tile_w` wide tall per footprint cell (`n`), its frame's top to its foot row (`fit_square` keeps the scale: the
/// square's side is the frame's longer side), and how far its bottom edge drops under the foot point's row (the frame
/// rows under the foot row). None for a body without a `foot` row (it keeps its token square). The battle stage
/// (`Geom::sprite_px`, `CELL_W`) and the hub and region maps (`mapview::body_rect`, their own tile width) share it.
pub fn painted_px(tile_w: i32, n: i32, frame: (u32, u32), foot: Option<u32>) -> Option<(i32, i32)> {
    let foot = foot.filter(|&f| f > 0 && frame.0 > 0 && frame.1 > 0)?;
    let long = frame.0.max(frame.1) as f32;
    let sp = (PAINTED_CELLS * tile_w as f32 * n.max(1) as f32 * long / foot as f32).round() as i32;
    let drop = ((frame.1 - 1).saturating_sub(foot) as f32 * sp as f32 / long).round() as i32;
    Some((sp, drop))
}

/// The Flat view: square cells at the panel's interior top.
pub const FLAT_Y: i32 = 30;
pub const FLAT_CELL: i32 = 32;
/// Ground arrows are shortened in flat space (y / `ARROW_Y`) at the start and the end (3.7 pass 4).
pub const ARROW_Y: f32 = 0.75;
pub const ARROW_START: f32 = 10.0;
pub const ARROW_END: f32 = 12.0;

const NIGHT: Rgba = rgb(10, 14, 24);

/// The battle view setting (`battle_view`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Stage,
    Flat,
}

/// The setting's values and labels; the first is the default.
pub const VIEWS: [(&str, &str); 2] = [("stage", "Stage"), ("flat", "Flat")];
pub const VIEW_TIP: &str = "Stage: a tilted board with standing trees and walls. Flat: the board seen from above, as before.";

impl View {
    /// The setting's value: `flat` is Flat, anything else (unknown, missing) Stage.
    pub fn of_setting(v: &Value) -> View {
        if v.as_str() == Some("flat") {
            View::Flat
        } else {
            View::Stage
        }
    }
}

/// The board geometry of a view at the tuning's sizes (section 3, 4.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geom {
    pub view: View,
    /// The unit sprite square (`token_px` on the Stage, `flat_token_px` Flat).
    pub token_px: i32,
    /// The Stage foot point's height above its cell's bottom edge.
    pub foot_inset: i32,
}

impl Geom {
    pub fn new(view: View, t: &Tuning) -> Geom {
        Geom { view, token_px: if view == View::Stage { t.token_px } else { t.flat_token_px }, foot_inset: t.foot_inset_px }
    }

    /// Cell (x, y)'s rect: 32 x 24 on the board (Stage), 32 x 32 from the panel's top (Flat).
    pub fn cell_rect(&self, c: V2) -> Rect {
        match self.view {
            View::Stage => Rect::new(BOARD_X + c.x * CELL_W, BOARD_Y + c.y * CELL_H, CELL_W, CELL_H),
            View::Flat => Rect::new(BOARD_X + c.x * FLAT_CELL, FLAT_Y + c.y * FLAT_CELL, FLAT_CELL, FLAT_CELL),
        }
    }

    /// BIGUNIT1 BU14: the rect of the `n` x `n` square anchored at `c` (`cell_rect` for one cell).
    pub fn square_rect(&self, c: V2, n: i32) -> Rect {
        let (a, n) = (self.cell_rect(c), n.max(1));
        Rect::new(a.x, a.y, a.w * n, a.h * n)
    }

    /// BIGUNIT1 BU14: `foot` of a unit filling the `n` x `n` square at `c`: centred on the square across, `foot_inset`
    /// above its bottom edge (Stage); the square's centre (Flat).
    pub fn foot_n(&self, c: V2, n: i32) -> (i32, i32) {
        let r = self.square_rect(c, n);
        match self.view {
            View::Stage => (r.x + r.w / 2, r.y + r.h - self.foot_inset),
            View::Flat => (r.x + r.w / 2, r.y + r.h / 2),
        }
    }

    /// BIGUNIT1 BU14: `token_rect` of a unit filling the `n` x `n` square at `c`: its sprite scaled `n` times (the
    /// existing sheet until a 2 x 2 sheet lands), on the square's foot point (Stage) or centred on it (Flat).
    pub fn token_rect_n(&self, c: V2, n: i32, dx: i32, dy: i32) -> Rect {
        if n <= 1 {
            return self.token_rect(c, dx, dy);
        }
        let tp = self.token_px * n;
        let (fx, fy) = self.foot_n(c, n);
        match self.view {
            View::Stage => Rect::new(fx - tp / 2 + dx, fy + 1 - tp + dy, tp, tp),
            View::Flat => Rect::new(fx - tp / 2 + dx, fy - tp / 2 + dy, tp, tp),
        }
    }

    /// DIORAMA1B (D5, contract 8.1): the sprite square of a unit filling the `n` x `n` square whose body has `frame`
    /// and a `foot` row, and how far its bottom edge drops under the foot point's row. On the Stage a body with a
    /// `foot` row is scaled so its frame from the top to the foot row stands `PAINTED_CELLS` cells (`CELL_W`) tall per
    /// footprint cell (`fit_square` keeps the scale: the square's side is the frame's longer side), the drop being the
    /// frame rows under the foot row; any other body (and every body Flat) keeps the token square (`token_px` x `n`,
    /// no drop), as before.
    pub fn sprite_px(&self, n: i32, frame: (u32, u32), foot: Option<u32>) -> (i32, i32) {
        let n = n.max(1);
        match self.view {
            View::Stage => painted_px(CELL_W, n, frame, foot).unwrap_or((self.token_px * n, 0)),
            View::Flat => (self.token_px * n, 0),
        }
    }

    /// DIORAMA1B: `token_rect_n` for a sprite square of side `sp` dropping `drop` px (`sprite_px`): centred on the
    /// square's foot point across, its bottom edge `1 + drop` px under it (Stage); centred on the square (Flat). With
    /// the token square and no drop it is `token_rect_n`.
    pub fn sprite_rect_n(&self, c: V2, n: i32, (sp, drop): (i32, i32), dx: i32, dy: i32) -> Rect {
        if sp == self.token_px * n.max(1) && drop == 0 {
            return self.token_rect_n(c, n, dx, dy);
        }
        let (fx, fy) = self.foot_n(c, n);
        match self.view {
            View::Stage => Rect::new(fx - sp / 2 + dx, fy + 1 + drop - sp + dy, sp, sp),
            View::Flat => Rect::new(fx - sp / 2 + dx, fy - sp / 2 + dy, sp, sp),
        }
    }

    /// The cell under logical (mx, my) (None: off the board; the backdrop and the front face hold no cell).
    pub fn cell_at(&self, mx: i32, my: i32) -> Option<V2> {
        let (ch, top) = match self.view {
            View::Stage => (CELL_H, BOARD_Y),
            View::Flat => (FLAT_CELL, FLAT_Y),
        };
        let (dx, dy) = (mx - BOARD_X, my - top);
        if dx < 0 || dy < 0 {
            return None;
        }
        let c = V2::new(dx / CELL_W, dy / ch);
        (c.x < GRID_W && c.y < GRID_H).then_some(c)
    }

    /// Where a unit on `c` stands: centred across, `foot_inset` above the cell's bottom edge (Stage); the cell's
    /// centre (Flat).
    pub fn foot(&self, c: V2) -> (i32, i32) {
        let r = self.cell_rect(c);
        match self.view {
            View::Stage => (r.x + r.w / 2, r.y + r.h - self.foot_inset),
            View::Flat => (r.x + r.w / 2, r.y + r.h / 2),
        }
    }

    /// The foot point raised a third of the sprite (where floats and effects start; FX1's anchor).
    pub fn chest(&self, c: V2) -> (i32, i32) {
        let (x, y) = self.foot(c);
        match self.view {
            View::Stage => (x, y - self.token_px / 3),
            View::Flat => (x, y),
        }
    }

    /// The sprite square of a unit on `c` moved by (dx, dy): its bottom edge one px under the foot point, centred on it
    /// (Stage); centred in the cell (Flat, as before).
    pub fn token_rect(&self, c: V2, dx: i32, dy: i32) -> Rect {
        let tp = self.token_px;
        match self.view {
            View::Stage => {
                let (fx, fy) = self.foot(c);
                Rect::new(fx - tp / 2 + dx, fy + 1 - tp + dy, tp, tp)
            }
            View::Flat => {
                let r = self.cell_rect(c);
                Rect::new(r.x + (r.w - tp) / 2 + dx, r.y + (r.h - tp) / 2 + dy, tp, tp)
            }
        }
    }

    /// The offset of a unit sliding from `from` to `to` with `left` (1 to 0) of the slide to go: a cell's width and
    /// height per step.
    pub fn slide(&self, from: V2, to: V2, left: f32) -> (i32, i32) {
        let ch = if self.view == View::Stage { CELL_H } else { FLAT_CELL };
        (((from.x - to.x) as f32 * CELL_W as f32 * left).round() as i32, ((from.y - to.y) as f32 * ch as f32 * left).round() as i32)
    }
}

// ---- The terrain stage table (3.1) -------------------------------------------------------------------------------

/// How a terrain stands on the stage (its row's `relief`): `Ground`; `Sunken`, lower than the ground (its depth in px:
/// its lips show the step); `Standing`, a prop on the ground of its neighbours; `Block`, lifted `BLOCK_LIFT` with a
/// cap and a face.
pub use emberwake_data::looks::Relief as Kind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageRow<'a> {
    pub kind: Kind,
    /// Px down (sunken), else 0.
    pub depth: i32,
    /// The prop it shows (its row's `prop`): a ground prop or the standing prop.
    pub prop: Option<&'a str>,
}

/// A terrain's stage row (3.1, PR5-33: its look on `grid`'s terrain table; an unknown id is ground, depth 0).
pub fn stage_of<'a>(grid: &'a Grid, terrain: &str) -> StageRow<'a> {
    let look = grid.table.look(terrain);
    StageRow { kind: look.relief, depth: look.depth, prop: (!look.prop.is_empty()).then_some(look.prop.as_str()) }
}

/// The terrain on `c` (None off the grid).
fn at(grid: &Grid, c: V2) -> Option<&str> {
    grid.is_valid(c).then(|| grid.terrain_id(c))
}

fn is_ground(grid: &Grid, t: &str) -> bool {
    matches!(stage_of(grid, t).kind, Kind::Ground | Kind::Sunken)
}

fn is_block(grid: &Grid, c: V2) -> bool {
    at(grid, c).is_some_and(|t| stage_of(grid, t).kind == Kind::Block)
}

/// The four neighbours in tie order: north, east, south, west.
fn neighbours(c: V2) -> [V2; 4] {
    [V2::new(c.x, c.y - 1), V2::new(c.x + 1, c.y), V2::new(c.x, c.y + 1), V2::new(c.x - 1, c.y)]
}

/// The most common of `ids` (ties: the first seen); None when empty.
fn most_common<'a>(ids: &[&'a str]) -> Option<&'a str> {
    let mut best: Option<(&'a str, usize)> = None;
    for id in ids {
        let n = ids.iter().filter(|o| *o == id).count();
        if best.is_none_or(|(_, m)| n > m) {
            best = Some((id, n));
        }
    }
    best.map(|(id, _)| id)
}

/// 3.2.1: the ground a standing cell shows under its prop: the terrain most of its four neighbours have among ground
/// and sunken cells (ties: first in the order north, east, south, west); none, `grass`.
pub fn ground_under(grid: &Grid, c: V2) -> &str {
    let ids: Vec<&str> = neighbours(c).iter().filter_map(|n| at(grid, *n)).filter(|t| is_ground(grid, t)).collect();
    most_common(&ids).unwrap_or("grass")
}

/// The depth a lip compares with (block and standing count as 0).
fn depth_of(grid: &Grid, t: &str) -> i32 {
    stage_of(grid, t).depth
}

/// 3.2.2: a sunken cell's lips: the band over its top (px; 0 none) when its north neighbour is shallower, and whether
/// its bottom row is shaded (its south neighbour is shallower).
pub fn lips(grid: &Grid, c: V2) -> (i32, bool) {
    let Some(t) = at(grid, c) else { return (0, false) };
    let row = stage_of(grid, t);
    if row.kind != Kind::Sunken {
        return (0, false);
    }
    let d = row.depth;
    let band = at(grid, V2::new(c.x, c.y - 1)).map(|n| depth_of(grid, n)).filter(|n| *n < d).map_or(0, |n| d - n + 1);
    let bottom = at(grid, V2::new(c.x, c.y + 1)).map(|n| depth_of(grid, n)).is_some_and(|n| n < d);
    (band, bottom)
}

/// 3.3: a block draws its face over its own bottom half unless the cell south is a block too (row 8 always does).
pub fn block_face(grid: &Grid, c: V2) -> bool {
    !is_block(grid, V2::new(c.x, c.y + 1))
}

/// 3.3: a block's drawn rect: the cap lifted `BLOCK_LIFT` over its cell, and its face when it has one.
pub fn block_rect(grid: &Grid, c: V2) -> Rect {
    let r = Geom { view: View::Stage, token_px: 0, foot_inset: 0 }.cell_rect(c);
    Rect::new(r.x, r.y - BLOCK_LIFT, r.w, r.h + if block_face(grid, c) { BLOCK_LIFT } else { 0 })
}

/// 3.5: a prop of `size` px (its PNG, 1:1 at 32 px of art per cell) stands bottom-centred on the cell rect `cell`.
pub fn prop_rect(cell: Rect, size: (i32, i32)) -> Rect {
    Rect::new(cell.x + (cell.w - size.0) / 2, cell.y + cell.h - size.1, size.0, size.1)
}

/// 3.4: the backdrop fallback's ground: the most common ground terrain of row 0 (ties: the first from the left);
/// none, `grass`.
pub fn far_ground(grid: &Grid) -> &str {
    let ids: Vec<&str> = (0..GRID_W).filter_map(|x| at(grid, V2::new(x, 0))).filter(|t| is_ground(grid, t)).collect();
    most_common(&ids).unwrap_or("grass")
}

/// The ground a cell shows (a standing cell the ground under its prop).
fn shown_ground(grid: &Grid, c: V2) -> &str {
    let t = grid.terrain_id(c);
    if stage_of(grid, t).kind == Kind::Standing {
        ground_under(grid, c)
    } else {
        t
    }
}

/// The flat colour of a terrain without a `color` of its own.
pub const DEFAULT_TERRAIN_COLOR: Rgba = rgb(80, 82, 92);

/// A terrain's flat colour (the fallback when no tile loads; the backdrop and face colours): PR5-33, its row's `color`
/// on `grid`'s terrain table, else `DEFAULT_TERRAIN_COLOR`.
pub fn terrain_color(grid: &Grid, id: &str) -> Rgba {
    grid.table.look(id).color.map_or(DEFAULT_TERRAIN_COLOR, |[r, g, b]| rgb(r, g, b))
}

// ---- Art (3.2-3.5, 4.4) ------------------------------------------------------------------------------------------

/// Which art a ground cell draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileArt {
    /// A stage tile (`tilesets/common_stage/<id>/v<n>.png`), drawn 32 x 24.
    Stage(String),
    /// The square `common` tile squashed into the cell (no stage tile).
    Common(String),
    /// Neither: the terrain's colour.
    Colour,
}

fn dirs_ref(dirs: &[String]) -> Vec<&str> {
    dirs.iter().map(String::as_str).collect()
}

/// 3.2.1: terrain `id`'s tile on cell (x, y), varied by position (LOOK1 5.4: `set`, the world's stage set, first).
pub fn tile_art(art: &mut Art, id: &str, set: &str, x: i32, y: i32) -> TileArt {
    if let Some(rel) = art.pick(&dirs_ref(&battle_stage_dirs(id, set)), None, x, y) {
        return TileArt::Stage(rel);
    }
    let common: Vec<String> = BATTLE_SETS.iter().map(|s| format!("tilesets/{s}/{id}")).collect();
    match art.pick(&dirs_ref(&common), None, x, y) {
        Some(rel) => TileArt::Common(rel),
        None => TileArt::Colour,
    }
}

/// 3.5: a prop's art on cell (x, y) (the PROPS1 position pick) with its PNG size; None: no art.
pub fn prop_art(art: &mut Art, prop: &str, x: i32, y: i32) -> Option<(String, (i32, i32))> {
    let rel = art.pick(&dirs_ref(&battle_prop_dirs(prop)), None, x, y)?;
    let size = art.size_of(&rel)?;
    Some((rel, size))
}

/// 3.3: a block's face art on cell (x, y); None: the cap's top rows darkened.
pub fn face_art(art: &mut Art, set: &str, x: i32, y: i32) -> Option<String> {
    art.pick(&dirs_ref(&battle_stage_dirs("wall_face", set)), None, x, y)
}

/// DIORAMA1 D4 (SCREEN_CONTRACT_BATTLE 8.1): the wall foot strip on row 0's cell `x`, `tilesets/<set>/wall_foot/v*`
/// (the world's set first, then the common stage set); None: no file, nothing drawn.
pub fn wall_foot_art(art: &mut Art, set: &str, x: i32) -> Option<String> {
    art.pick(&dirs_ref(&battle_stage_dirs("wall_foot", set)), None, x, 0)
}

/// DIORAMA1 D4: the strip over row 0's ground cells (`WALL_FOOT_H` tall, at the set's art px), so the floor runs into
/// the wall band instead of stopping at a line; blocks and standing cells draw their own.
fn draw_wall_foot(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str) {
    for x in 0..GRID_W {
        let c = V2::new(x, 0);
        if !at(grid, c).is_some_and(|t| is_ground(grid, t)) {
            continue;
        }
        let Some(rel) = wall_foot_art(art, set, x) else { continue };
        let r = stage_cell(c);
        if let Some(img) = stage_image(art, &rel, set, CELL_W, WALL_FOOT_H) {
            p.image(img, Rect::new(r.x, r.y, r.w, WALL_FOOT_H));
        }
    }
}

/// 3.4: zone `zone`'s backdrop file (None: no zone, or no file: the gradient).
pub fn backdrop_art(art: &mut Art, zone: &str) -> Option<String> {
    let rel = battle_backdrop(zone);
    (!zone.is_empty() && art.has(&rel)).then_some(rel)
}

/// The standing prop a cell shows with its art (None: not a standing cell, or no art: the flat tile instead).
fn standing_art(art: &mut Art, grid: &Grid, c: V2) -> Option<(String, (i32, i32))> {
    let row = stage_of(grid, grid.terrain_id(c));
    if row.kind != Kind::Standing {
        return None;
    }
    prop_art(art, row.prop?, c.x, c.y)
}

fn stage_cell(c: V2) -> Rect {
    Rect::new(BOARD_X + c.x * CELL_W, BOARD_Y + c.y * CELL_H, CELL_W, CELL_H)
}

/// A stage file for a `w` x `h` logical rect: a file of the world's set at its art resolution (LOOK1 5.1, so a large
/// window shows its art px), any other as before.
fn stage_image<'a>(art: &'a mut Art, rel: &str, set: &str, w: i32, h: i32) -> Option<&'a Image> {
    let own = !set.is_empty() && rel.strip_prefix("tilesets/").and_then(|r| r.strip_prefix(set)).is_some_and(|r| r.starts_with('/'));
    let k = if own { SET_ART_PX } else { 1 };
    art.image(rel, w * k, h * k)
}

fn draw_tile(p: &mut Painter, art: &mut Art, grid: &Grid, id: &str, set: &str, c: V2, r: Rect) {
    let terrain_color = |id: &str| terrain_color(grid, id);
    match tile_art(art, id, set, c.x, c.y) {
        TileArt::Stage(rel) => match stage_image(art, &rel, set, CELL_W, CELL_H) {
            Some(img) => p.image(img, r),
            None => p.fill(r, terrain_color(id)),
        },
        TileArt::Common(rel) => match art.image(&rel, FLAT_CELL, FLAT_CELL) {
            Some(img) => p.image(img, r),
            None => p.fill(r, terrain_color(id)),
        },
        TileArt::Colour => p.fill(r, terrain_color(id)),
    }
}

// ---- Drawing the board's pieces (3.2-3.5) -------------------------------------------------------------------------

fn lerp(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let m = |i: usize| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8;
    [m(0), m(1), m(2), 255]
}

fn scaled(c: Rgba, k: f32) -> Rgba {
    [(c[0] as f32 * k).round() as u8, (c[1] as f32 * k).round() as u8, (c[2] as f32 * k).round() as u8, c[3]]
}

/// 3.4, HOOKS1 ZN3: what the backdrop pass draws for zone `zone`: its file (None: the gradient) and the gradient's
/// ground. A code zone (or an unknown one): `backdrop_art` and the far ground, whatever `zones` holds; a data zone row
/// (`data_zone` over `zones`): its backdrop key (`zone_backdrop`) when the file exists, and its `ground`.
pub fn backdrop_source<'a>(art: &mut Art, grid: &'a Grid, zone: &str, zones: &'a IndexMap<String, ZoneDef>) -> (Option<String>, &'a str) {
    match data_zone(zone, zones) {
        Some(z) => (Some(zone_backdrop(zone, Some(z))).filter(|rel| art.has(rel)), z.ground.as_str()),
        None => (backdrop_art(art, zone), far_ground(grid)),
    }
}

/// Pass 1: the zone's backdrop, else a gradient from the night sky to the far ground's colour at 40 % (a data zone
/// row in `zones`: its backdrop key and `ground`, `backdrop_source`).
pub fn draw_backdrop(p: &mut Painter, art: &mut Art, grid: &Grid, zone: &str, zones: &IndexMap<String, ZoneDef>) {
    let (file, ground) = backdrop_source(art, grid, zone, zones);
    if let Some(img) = file.and_then(|rel| art.image(&rel, BACKDROP.w, BACKDROP.h)) {
        p.image(img, BACKDROP);
        return;
    }
    let bottom = lerp(NIGHT, terrain_color(grid, ground), 0.4);
    for k in 0..BACKDROP.h {
        let t = k as f32 / (BACKDROP.h - 1) as f32;
        p.fill(Rect::new(BACKDROP.x, BACKDROP.y + k, BACKDROP.w, 1), lerp(NIGHT, bottom, t));
    }
}

/// The ground a cell's tile shows in the ground pass: a standing cell with prop art the ground under it, else its own.
fn tile_shown<'g>(art: &mut Art, grid: &'g Grid, c: V2) -> &'g str {
    if standing_art(art, grid, c).is_some() {
        ground_under(grid, c)
    } else {
        grid.terrain_id(c)
    }
}

/// LOOK1 5.3: the transition pieces of stage set `set` on this board, as (corner x, corner y, file); none without a
/// set. A corner takes a piece when its cells show exactly two grounds and the set has that pair in one order (off-grid
/// cells read as the nearest cell). A block counts as the upper ground: its cell is hidden under the block. A corner
/// with a third ground takes none, since a piece is opaque and would paint over it.
pub fn stage_pieces(art: &mut Art, grid: &Grid, set: &str) -> Vec<(i32, i32, String)> {
    let mut out = Vec::new();
    if set.is_empty() {
        return out;
    }
    let mut cells = Vec::with_capacity((GRID_W * GRID_H) as usize);
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            cells.push(tile_shown(art, grid, V2::new(x, y)));
        }
    }
    let at = |x: i32, y: i32| cells[(y.clamp(0, GRID_H - 1) * GRID_W + x.clamp(0, GRID_W - 1)) as usize];
    let block = |t: &str| stage_of(grid, t).kind == Kind::Block;
    for cy in 0..=GRID_H {
        for cx in 0..=GRID_W {
            let mut grounds: Vec<&str> = [at(cx - 1, cy - 1), at(cx, cy - 1), at(cx - 1, cy), at(cx, cy)].into_iter().filter(|t| !block(t)).collect();
            grounds.sort_unstable();
            grounds.dedup();
            let [a, b] = grounds[..] else { continue };
            for (upper, lower) in [(a, b), (b, a)] {
                let mask = corner_mask(|x, y| at(x, y) == upper || block(at(x, y)), cx, cy);
                let rel = transition_piece(set, upper, lower, mask);
                if art.has(&rel) {
                    out.push((cx, cy, rel));
                    break;
                }
            }
        }
    }
    out
}

/// LOOK1 5.3: a transition piece's rect, one cell centred on corner (`cx`, `cy`).
pub fn piece_rect(cx: i32, cy: i32) -> Rect {
    Rect::new(BOARD_X + cx * CELL_W - CELL_W / 2, BOARD_Y + cy * CELL_H - CELL_H / 2, CELL_W, CELL_H)
}

/// The pieces of `stage_pieces`, clipped to the board (which a camera shake moves) inside the clip already set.
fn draw_pieces(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str) {
    let pieces = stage_pieces(art, grid, set);
    if pieces.is_empty() {
        return;
    }
    let outer = p.c.clip;
    let board = p.v.rect(BOARD);
    p.c.clip = Some(outer.map_or(board, |o| o.intersect(board)));
    for (cx, cy, rel) in &pieces {
        if let Some(img) = stage_image(art, rel, set, CELL_W, CELL_H) {
            p.image(img, piece_rect(*cx, *cy));
        }
    }
    p.c.clip = outer;
}

/// Pass 2: every cell's ground tile (a standing cell with prop art the ground under it, without art the flat tile),
/// the stage set's transition pieces, the lips of sunken cells, the ground props and the grid lines at `grid_alpha`.
pub fn draw_ground(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str, grid_alpha: u8) {
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            let shown = tile_shown(art, grid, c);
            draw_tile(p, art, grid, shown, set, c, stage_cell(c));
        }
    }
    draw_pieces(p, art, grid, set);
    draw_wall_foot(p, art, grid, set);
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            let r = stage_cell(c);
            let (band, bottom) = lips(grid, c);
            if band > 0 {
                p.fill(Rect::new(r.x, r.y, r.w, 1), rgb(118, 116, 62));
                if band > 1 {
                    p.fill(Rect::new(r.x, r.y + 1, r.w, band - 1), rgb(66, 54, 30));
                }
                p.fill(Rect::new(r.x, r.y + band, r.w, 1), rgba(60, 40, 26, 110));
            }
            if bottom {
                p.fill(Rect::new(r.x, r.y + r.h - 1, r.w, 1), rgba(48, 34, 22, 120));
            }
            let row = stage_of(grid, grid.terrain_id(c));
            if let (Kind::Ground, Some(prop)) = (row.kind, row.prop) {
                if let Some((rel, size)) = prop_art(art, prop, x, y) {
                    if let Some(img) = art.image(&rel, size.0, size.1) {
                        p.image(img, prop_rect(r, size));
                    }
                }
            }
        }
    }
    let line = rgba(0, 0, 0, grid_alpha);
    for k in 0..=GRID_H {
        p.fill(Rect::new(BOARD_X, BOARD_Y + k * CELL_H, GRID_W * CELL_W, 1), line);
    }
    for k in 0..=GRID_W {
        p.fill(Rect::new(BOARD_X + k * CELL_W, BOARD_Y, 1, GRID_H * CELL_H), line);
    }
}

/// Pass 5 (props): the contact shadow under each standing prop that has art.
pub fn draw_prop_shadows(p: &mut Painter, art: &mut Art, grid: &Grid) {
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            if standing_art(art, grid, c).is_some() {
                let r = stage_cell(c);
                p.ellipse((r.x + r.w / 2 + 2) as f32, (r.y + r.h - 5) as f32, 10.0, 2.7, rgba(10, 16, 6, 95));
            }
        }
    }
}

fn lighten(c: Rgba, k: f32) -> Rgba {
    let m = |i: usize| (c[i] as f32 + (255.0 - c[i] as f32) * k).round() as u8;
    [m(0), m(1), m(2), 255]
}

/// The front face's body under a column (3.4; PR5-33: its bottom row terrain's `face`).
pub use emberwake_data::looks::Face as FaceBody;

/// 3.4: column `x`'s front face: the ground its top row lightens (row 8's shown ground) and its body.
pub fn face_column(grid: &Grid, x: i32) -> (&str, FaceBody) {
    let c = V2::new(x, GRID_H - 1);
    (shown_ground(grid, c), grid.table.look(grid.terrain_id(c)).face)
}

/// Pass 6: the front face (per column: the top row the ground's colour lightened 25 %, the body wall art, water,
/// lava or earth with flecks), then the drop shadow.
pub fn draw_face(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str) {
    for x in 0..GRID_W {
        let r = Rect::new(FRONT_FACE.x + x * CELL_W, FRONT_FACE.y, CELL_W, FACE_H);
        draw_face_cell(p, art, grid, set, x, x, r);
    }
    for k in 0..DROP_SHADOW.h {
        p.fill(Rect::new(DROP_SHADOW.x, DROP_SHADOW.y + k, DROP_SHADOW.w, 1), rgba(0, 0, 0, (120 - 15 * k).max(0) as u8));
    }
}

/// DIORAMA2 R2: the front face's art for column `x`'s top terrain `top`: the stage set's
/// `tilesets/<set>/front_face/<top>/v*` (picked by the column); None: no set, or no file for that terrain.
pub fn front_face_art(art: &mut Art, set: &str, top: &str, x: i32) -> Option<String> {
    if set.is_empty() {
        return None;
    }
    art.pick(&[&format!("tilesets/{set}/front_face/{top}")], None, x, GRID_H)
}

/// The front face of column `src` (its terrains) drawn in `r`, the art picked by column `x`: the stage set's front face
/// for the column's top terrain (DIORAMA2 R2), else today's body by the column's face kind with the top's light line.
fn draw_face_cell(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str, src: i32, x: i32, r: Rect) {
    let (top, body) = face_column(grid, src);
    if let Some(img) = front_face_art(art, set, top, x).and_then(|rel| stage_image(art, &rel, set, CELL_W, FACE_H)) {
        p.image(img, r);
        return;
    }
    let flecks = |p: &mut Painter, col: Rgba| {
        let mut fx = 0;
        while fx < CELL_W {
            let bx = x * CELL_W + fx;
            p.fill(Rect::new(r.x + fx, r.y + 2 + (bx * 7).rem_euclid(6), 2, 1), col);
            fx += 7;
        }
    };
    match body {
        FaceBody::Wall => match face_art(art, set, x, GRID_H).and_then(|rel| stage_image(art, &rel, set, CELL_W, FACE_H)) {
            Some(img) => p.image(img, r),
            None => p.fill(r, scaled(terrain_color(grid, grid.terrain_id(V2::new(src, GRID_H - 1))), 0.75)),
        },
        FaceBody::Water => p.fill(r, scaled(rgb(30, 54, 110), 0.7)),
        FaceBody::Lava => {
            p.fill(r, rgb(150, 50, 20));
            flecks(p, rgb(255, 160, 60));
        }
        FaceBody::Earth => {
            p.fill(r, rgb(70, 50, 32));
            flecks(p, rgb(52, 36, 24));
        }
    }
    p.fill(Rect::new(r.x, r.y, r.w, 1), lighten(terrain_color(grid, top), 0.25));
}

/// DIORAMA3 R5: a cover prop's height (64 x 72 art at 2 art px: a cell and half a cell above it, the block's own box).
pub const COVER_H: i32 = CELL_H + BLOCK_LIFT;

/// DIORAMA3 R5: whether block `c` belongs to the arena's outer wall: it is on the grid's edge, or joined to the edge
/// through other blocks (side by side).
pub fn outer_wall(grid: &Grid, c: V2) -> bool {
    let mut seen = vec![c];
    let mut todo = vec![c];
    while let Some(at) = todo.pop() {
        if at.x == 0 || at.y == 0 || at.x == GRID_W - 1 || at.y == GRID_H - 1 {
            return true;
        }
        for n in neighbours(at) {
            if is_block(grid, n) && !seen.contains(&n) {
                seen.push(n);
                todo.push(n);
            }
        }
    }
    false
}

/// DIORAMA3 R5: the cover prop block `c` shows, `tilesets/<set>/cover/v*` picked by the cell (v0 crates, v1 barrels,
/// v2 pillar, v3 lamp post); None: no set, no files, not a block, or a block of the outer wall (its wall cap).
pub fn cover_art(art: &mut Art, grid: &Grid, set: &str, c: V2) -> Option<String> {
    if set.is_empty() || !is_block(grid, c) || outer_wall(grid, c) {
        return None;
    }
    art.pick(&[&format!("tilesets/{set}/cover")], None, c.x, c.y)
}

/// DIORAMA3 R3: the light spots of stage set `set` for battle zone `zone` from its `stage.fx.json`: `zones.<zone>.spots`
/// first, else the top-level `spots`; each `at` is [u, v], fractions of the backdrop. Read once per set and zone.
pub fn light_spots(art: &mut Art, set: &str, zone: &str) -> Vec<(f32, f32)> {
    if set.is_empty() {
        return Vec::new();
    }
    thread_local! {
        static SPOTS: std::cell::RefCell<std::collections::HashMap<(String, String), Vec<(f32, f32)>>> = Default::default();
    }
    let key = (set.to_string(), zone.to_string());
    if let Some(v) = SPOTS.with(|m| m.borrow().get(&key).cloned()) {
        return v;
    }
    let v = aegis_core::assets::read_json(art.layers(), &format!("tilesets/{set}/stage.fx.json")).map_or_else(Vec::new, |fx| {
        let own = &fx["zones"][zone]["spots"];
        let list = if own.is_array() { own } else { &fx["spots"] };
        list.as_array()
            .map(|a| a.iter().filter_map(|s| Some((s["at"][0].as_f64()? as f32, s["at"][1].as_f64()? as f32))).filter(|(u, v)| (0.0..=1.0).contains(u) && (0.0..=1.0).contains(v)).collect())
            .unwrap_or_default()
    });
    SPOTS.with(|m| m.borrow_mut().insert(key, v.clone()));
    v
}

/// DIORAMA3 R3: the amber of a lantern's pool.
pub const POOL: Rgba = rgb(255, 170, 80);
/// The pool's half sizes on the floor (logical px) and its rings (alpha each; the centre gathers them).
pub const POOL_RX: f32 = 46.0;
pub const POOL_RY: f32 = 18.0;
const POOL_RINGS: [(f32, u8); 10] = [(1.0, 5), (0.9, 6), (0.8, 6), (0.7, 7), (0.6, 7), (0.5, 8), (0.4, 8), (0.3, 9), (0.2, 9), (0.12, 10)];

/// DIORAMA3 R3: the light pass after the ground: an amber pool on the floor below each lantern spot (centred on its
/// `u`, on row 0 and into row 1), then a soft vignette at the stage's edges. Static (no flicker), so it stays on under
/// Reduce motion.
pub fn draw_light(p: &mut Painter, spots: &[(f32, f32)]) {
    for &(u, _) in spots {
        let cx = BACKDROP.x as f32 + u * BACKDROP.w as f32;
        let cy = BOARD_Y as f32 + CELL_H as f32 * 0.7;
        for (k, a) in POOL_RINGS {
            p.ellipse(cx, cy, POOL_RX * k, POOL_RY * k, rgba(POOL[0], POOL[1], POOL[2], a));
        }
    }
    let clip = STAGE_CLIP;
    for k in 0..VIGNETTE_PX {
        let a = ((1.0 - k as f32 / VIGNETTE_PX as f32).powf(2.0) * VIGNETTE_ALPHA as f32).round() as u8;
        let col = rgba(0, 0, 0, a);
        p.fill(Rect::new(clip.x, BOARD_Y + k, clip.w, 1), col);
        p.fill(Rect::new(clip.x, FRONT_FACE.y - 1 - k, clip.w, 1), col);
        p.fill(Rect::new(INTERIOR.x + k, BOARD_Y, 1, FRONT_FACE.y - BOARD_Y), col);
        p.fill(Rect::new(INTERIOR.x + INTERIOR.w - 1 - k, BOARD_Y, 1, FRONT_FACE.y - BOARD_Y), col);
    }
}

/// DIORAMA3 R3: the vignette's depth at each edge of the board and its darkest alpha.
pub const VIGNETTE_PX: i32 = 18;
pub const VIGNETTE_ALPHA: u8 = 70;

/// Today's lip on a wall cell's cap (a set with no `lip` of its own and no set named).
pub const LIP_TODAY: Rgba = rgb(170, 172, 186);
/// DIORAMA2 R1 (Fable review): the lip of a stage set with a `stage.fx.json` but no `lip` in it: bronze, not white.
pub const LIP_BRONZE: Rgba = rgb(112, 88, 54);

/// DIORAMA2 R1: the lip line on a wall cell's cap: the stage set's `lip` (a "#rrggbb" in its `stage.fx.json`), else
/// `LIP_BRONZE` when the set has a `stage.fx.json`, else `LIP_TODAY` (no set, or a set without the file: today's
/// frame). Read once per set.
pub fn lip_colour(art: &mut Art, set: &str) -> Rgba {
    if set.is_empty() {
        return LIP_TODAY;
    }
    thread_local! {
        static LIPS: std::cell::RefCell<std::collections::HashMap<String, Rgba>> = Default::default();
    }
    if let Some(c) = LIPS.with(|l| l.borrow().get(set).copied()) {
        return c;
    }
    let c = match aegis_core::assets::read_json(art.layers(), &format!("tilesets/{set}/stage.fx.json")) {
        Some(v) => v["lip"].as_str().and_then(rpgkit_ui::vignette::hex).unwrap_or(LIP_BRONZE),
        None => LIP_TODAY,
    };
    LIPS.with(|l| l.borrow_mut().insert(set.to_string(), c));
    c
}

/// DIORAMA2 R4: the stage's wings, the side gutters of the board panel (`STAGE_CLIP` beyond `INTERIOR`), drawn before
/// the passes: the backdrop repeated beside itself, the edge columns' ground (and row 0's wall foot) one cell further
/// out, their front face and drop shadow, then a fade to the panel colour `panel` at the outer edge.
pub fn draw_wings(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str, zone: &str, zones: &IndexMap<String, ZoneDef>, panel: Rgba) {
    let (file, ground) = backdrop_source(art, grid, zone, zones);
    match file.and_then(|rel| art.image(&rel, BACKDROP.w, BACKDROP.h)) {
        Some(img) => {
            for dx in [-BACKDROP.w, BACKDROP.w] {
                p.image(img, Rect::new(BACKDROP.x + dx, BACKDROP.y, BACKDROP.w, BACKDROP.h));
            }
        }
        None => {
            let bottom = lerp(NIGHT, terrain_color(grid, ground), 0.4);
            for k in 0..BACKDROP.h {
                let t = k as f32 / (BACKDROP.h - 1) as f32;
                for x0 in [STAGE_CLIP.x, INTERIOR.x + INTERIOR.w] {
                    p.fill(Rect::new(x0, BACKDROP.y + k, WING, 1), lerp(NIGHT, bottom, t));
                }
            }
        }
    }
    for (x, src) in [(-1, 0), (GRID_W, GRID_W - 1)] {
        for y in 0..GRID_H {
            let edge = V2::new(src, y);
            let t = if is_block(grid, edge) { ground_under(grid, edge) } else { tile_shown(art, grid, edge) };
            let r = stage_cell(V2::new(x, y));
            draw_tile(p, art, grid, t, set, V2::new(x, y), r);
            if y == 0 && is_ground(grid, t) {
                if let Some(img) = wall_foot_art(art, set, x).and_then(|rel| stage_image(art, &rel, set, CELL_W, WALL_FOOT_H)) {
                    p.image(img, Rect::new(r.x, r.y, r.w, WALL_FOOT_H));
                }
            }
        }
        let r = Rect::new(FRONT_FACE.x + x * CELL_W, FRONT_FACE.y, CELL_W, FACE_H);
        draw_face_cell(p, art, grid, set, src, x, r);
    }
    for k in 0..DROP_SHADOW.h {
        let a = rgba(0, 0, 0, (120 - 15 * k).max(0) as u8);
        p.fill(Rect::new(STAGE_CLIP.x, DROP_SHADOW.y + k, WING, 1), a);
        p.fill(Rect::new(INTERIOR.x + INTERIOR.w, DROP_SHADOW.y + k, WING, 1), a);
    }
    // the fade: the panel colour, full at the panel's edge, gone where the board starts
    for k in 0..WING {
        let a = (255.0 * (1.0 - k as f32 / WING as f32).powf(1.4)).round() as u8;
        let col = rgba(panel[0], panel[1], panel[2], a);
        p.fill(Rect::new(STAGE_CLIP.x + k, STAGE_CLIP.y, 1, STAGE_CLIP.h), col);
        p.fill(Rect::new(STAGE_CLIP.x + STAGE_CLIP.w - 1 - k, STAGE_CLIP.y, 1, STAGE_CLIP.h), col);
    }
}

/// The feet pass: block `c` (3.3): its cap (the stage tile lifted `BLOCK_LIFT`), its face (the wall face art, else the
/// cap's top rows at 75 %) with its top line and the shade on the cell south, the cap's highlight and side lines.
pub fn draw_block(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str, c: V2) {
    let r = stage_cell(c);
    // DIORAMA3 R5: a block inside the arena stands as the stage set's cover prop (display only), feet on its bottom edge
    if let Some(img) = cover_art(art, grid, set, c).and_then(|rel| stage_image(art, &rel, set, CELL_W, COVER_H)) {
        p.image(img, Rect::new(r.x, r.y + r.h - COVER_H, CELL_W, COVER_H));
        if c.y + 1 < GRID_H {
            p.fill(Rect::new(r.x, r.y + r.h, r.w, 3), rgba(0, 0, 0, 70));
        }
        return;
    }
    let cap = Rect::new(r.x, r.y - BLOCK_LIFT, CELL_W, CELL_H);
    let t = grid.terrain_id(c);
    draw_tile(p, art, grid, t, set, c, cap);
    if block_face(grid, c) {
        let face = Rect::new(r.x, r.y + CELL_H - FACE_H, CELL_W, FACE_H);
        let drawn = match face_art(art, set, c.x, c.y).and_then(|rel| stage_image(art, &rel, set, CELL_W, FACE_H)) {
            Some(img) => {
                p.image(img, face);
                true
            }
            None => false,
        };
        if !drawn {
            match darkened_cap(art, t, set, c) {
                Some(img) => p.image(&img, face),
                None => p.fill(face, scaled(terrain_color(grid, t), 0.75)),
            }
        }
        p.fill(Rect::new(face.x, face.y, face.w, 1), rgb(24, 24, 30));
        if c.y + 1 < GRID_H {
            p.fill(Rect::new(r.x, r.y + r.h, r.w, 3), rgba(0, 0, 0, 70));
        }
    }
    if !is_block(grid, V2::new(c.x, c.y - 1)) {
        p.fill(Rect::new(cap.x, cap.y, cap.w, 1), lip_colour(art, set));
    }
    let side = rgb(30, 30, 38);
    let h = block_rect(grid, c).h;
    if !is_block(grid, V2::new(c.x - 1, c.y)) {
        p.fill(Rect::new(cap.x, cap.y, 1, h), side);
    }
    if !is_block(grid, V2::new(c.x + 1, c.y)) {
        p.fill(Rect::new(cap.x + cap.w - 1, cap.y, 1, h), side);
    }
}

/// The wall face's fallback: the cap tile's top 12 rows at 75 % brightness.
fn darkened_cap(art: &mut Art, t: &str, set: &str, c: V2) -> Option<Image> {
    let img = match tile_art(art, t, set, c.x, c.y) {
        TileArt::Stage(rel) => art.image(&rel, CELL_W, CELL_H)?.clone(),
        TileArt::Common(rel) => art.image(&rel, CELL_W, CELL_H)?.clone(),
        TileArt::Colour => return None,
    };
    let mut top = img.crop(0, 0, img.w, (img.h / 2).max(1));
    for (k, v) in top.px.iter_mut().enumerate() {
        if k % 4 != 3 {
            *v = (*v as f32 * 0.75).round() as u8;
        }
    }
    Some(top)
}

/// The feet pass: standing prop `c`'s art at its PNG size, bottom-centred on the cell (mirrored never; varied by
/// position).
pub fn draw_prop(p: &mut Painter, art: &mut Art, grid: &Grid, c: V2) {
    if let Some((rel, size)) = standing_art(art, grid, c) {
        if let Some(img) = art.image(&rel, size.0, size.1) {
            p.image(img, prop_rect(stage_cell(c), size));
        }
    }
}

// ---- The feet pass and the outlines through (3.7 passes 7 and 8) ------------------------------------------------

/// What the feet pass draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Block(V2),
    Prop(V2),
    /// A unit by its battle index.
    Unit(usize),
}

/// One thing in the feet pass: its sort keys and drawn rect (a unit's body rect, 3.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub item: Item,
    pub foot_y: i32,
    pub x: i32,
    pub rect: Rect,
}

/// Blocks and standing props sort before units at equal feet.
const GROUP_BOARD: u8 = 0;
const GROUP_UNIT: u8 = 1;

/// The board's blocks and standing props (with art) in the feet pass: foot y the cell's bottom edge, x its left edge.
pub fn board_items(art: &mut Art, grid: &Grid) -> Vec<Placed> {
    let mut out = Vec::new();
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            let r = stage_cell(c);
            match stage_of(grid, grid.terrain_id(c)).kind {
                Kind::Block => out.push(Placed { item: Item::Block(c), foot_y: r.y + r.h, x: r.x, rect: block_rect(grid, c) }),
                Kind::Standing => {
                    if let Some((_, size)) = standing_art(art, grid, c) {
                        out.push(Placed { item: Item::Prop(c), foot_y: r.y + r.h, x: r.x, rect: prop_rect(r, size) });
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// A unit in the feet pass: its foot point (with the slide) and body rect.
pub fn unit_item(i: usize, foot: (i32, i32), body: Rect) -> Placed {
    Placed { item: Item::Unit(i), foot_y: foot.1, x: foot.0, rect: body }
}

/// Back to front by feet, then group (board pieces before units), then x, then push order (`DepthList`).
pub fn feet_order(items: &[Placed]) -> Vec<Placed> {
    let mut list = DepthList::new();
    for p in items {
        let group = if matches!(p.item, Item::Unit(_)) { GROUP_UNIT } else { GROUP_BOARD };
        list.push(p.foot_y as f32, group, p.x as f32, *p);
    }
    list.into_sorted()
}

fn meets(a: Rect, b: Rect) -> bool {
    let m = a.intersect(b);
    m.w > 0 && m.h > 0
}

/// Why a unit's outline is drawn again over what covers it (pass 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Through {
    /// (a) Behind a block or standing prop: in its side colour.
    Side,
    /// (b) The selected unit with anything in front of it: gold.
    Selected,
    /// (b) The hovered unit with anything in front of it: white.
    Hovered,
}

/// When pass 8 reveals a covered unit (STAGE1-O, the Gameplay setting `see_through` and the held Alt key).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reveal {
    /// The default: only the selected unit, so the unit lower on the board covers the one behind it.
    Selected,
    /// STAGE1's look: the selected and hovered unit, and any unit behind a block or standing prop.
    Always,
    /// While Alt is held: every unit outlined in its side colour, covered or not (other units count as cover).
    Held,
}

/// The setting's values and labels; the first is the default.
pub const SEE_THROUGH: [(&str, &str); 2] = [("selected", "When selected or Alt held"), ("always", "Always")];
pub const SEE_THROUGH_TIP: &str =
    "Outlines of units hidden behind others, trees or walls. When selected or Alt held: only the selected unit, and every unit outlined while you hold Alt. Always: the selected and hovered unit and any unit behind cover.";

impl Reveal {
    /// The setting's value and the Alt key: Alt held wins, `always` is Always, anything else (unknown, missing) the
    /// default.
    pub fn of(setting: &Value, alt: bool) -> Reveal {
        if alt {
            Reveal::Held
        } else if setting.as_str() == Some("always") {
            Reveal::Always
        } else {
            Reveal::Selected
        }
    }
}

/// Pass 8 for the item at `at` in `order` (a unit), by `reveal`: (b) the selected unit (and the hovered one, except
/// by default) when anything sorted after it meets its body rect; else while Alt is held every unit (Side); else (a)
/// with Always any unit a block or standing prop sorted after it meets; None: nothing covers it or it stays hidden.
pub fn through(order: &[Placed], at: usize, selected: bool, hovered: bool, reveal: Reveal) -> Option<Through> {
    let me = order.get(at)?;
    let after = &order[at + 1..];
    let hovered = hovered && reveal != Reveal::Selected;
    if (selected || hovered) && after.iter().any(|o| meets(o.rect, me.rect)) {
        return Some(if selected { Through::Selected } else { Through::Hovered });
    }
    match reveal {
        Reveal::Selected => None,
        Reveal::Held => Some(Through::Side),
        Reveal::Always => after.iter().any(|o| !matches!(o.item, Item::Unit(_)) && meets(o.rect, me.rect)).then_some(Through::Side),
    }
}

/// 3.8: a unit's widget rect: the bounding rect of its cell rect and its body rect.
pub fn unit_rect(cell: Rect, body: Rect) -> Rect {
    let (x0, y0) = (cell.x.min(body.x), cell.y.min(body.y));
    let (x1, y1) = ((cell.x + cell.w).max(body.x + body.w), (cell.y + cell.h).max(body.y + body.h));
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

// ---- Passes and ground arrows ------------------------------------------------------------------------------------

/// The Stage's draw passes, back to front (3.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pass {
    Backdrop,
    Ground,
    Markers,
    Arrows,
    Shadows,
    Face,
    Feet,
    Through,
    Top,
}

const PASSES: [Pass; 9] = [Pass::Backdrop, Pass::Ground, Pass::Markers, Pass::Arrows, Pass::Shadows, Pass::Face, Pass::Feet, Pass::Through, Pass::Top];

/// The order `battle/board.rs` runs the Stage's passes in.
pub fn passes() -> &'static [Pass] {
    &PASSES
}

/// 3.7 pass 4: a ground arrow from foot point `from` to foot point `to`, shortened in flat space (y / `ARROW_Y`) by
/// `ARROW_START` at the start and `ARROW_END` at the end, so its head stops at the target's foot ring; None when the
/// points are too close for that.
pub fn ground_arrow(from: (f32, f32), to: (f32, f32)) -> Option<(f32, f32, f32, f32)> {
    let (dx, dy) = (to.0 - from.0, (to.1 - from.1) / ARROW_Y);
    let len = (dx * dx + dy * dy).sqrt();
    if len <= ARROW_START + ARROW_END {
        return None;
    }
    let (ux, uy) = (dx / len, dy / len);
    Some((from.0 + ux * ARROW_START, from.1 + uy * ARROW_START * ARROW_Y, to.0 - ux * ARROW_END, to.1 - uy * ARROW_END * ARROW_Y))
}

/// A tell's arrow is dashed (3 px) for a move or a flee, else solid.
pub fn arrow_dash(kind: &str) -> f32 {
    if matches!(kind, "move" | "flee") {
        3.0
    } else {
        0.0
    }
}

/// A line `width` logical px wide (`Painter::line` is about one): parallel strokes one physical px apart, offset
/// across the line's minor axis so no gaps open.
#[allow(clippy::too_many_arguments)]
pub fn wide_line(p: &mut Painter, x0: f32, y0: f32, x1: f32, y1: f32, col: Rgba, width: f32, dash: f32) {
    let v = p.v;
    let (a, b, c, d) = (v.x(x0), v.y(y0), v.x(x1), v.y(y1));
    let n = (width * v.s).round().max(1.0) as i32;
    let dash_px = (dash * v.s).round() as i32;
    let steep = (d - b).abs() > (c - a).abs();
    for k in 0..n {
        let o = k - n / 2;
        if steep {
            p.c.line(a + o, b, c + o, d, col, dash_px);
        } else {
            p.c.line(a, b + o, c, d + o, col, dash_px);
        }
    }
}

/// An arrow with a `width` shaft and a head of `head` px (as `Painter::arrow`).
#[allow(clippy::too_many_arguments)]
pub fn wide_arrow(p: &mut Painter, x0: f32, y0: f32, x1: f32, y1: f32, col: Rgba, width: f32, dash: f32, head: f32) {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    wide_line(p, x0, y0, x1, y1, col, width, dash);
    for side in [-1.0f32, 1.0] {
        let (c, s) = (0.866f32, 0.5 * side);
        let (bx, by) = (-(ux * c - uy * s), -(uy * c + ux * s));
        wide_line(p, x1, y1, x1 + bx * head, y1 + by * head, col, width, 0.0);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use aegis_core::assets::Layers;
    use emberwake_battle::grid::TerrainTable;
    use serde_json::json;
    use std::path::Path;

    /// PR5-33: a fixture catalog's terrain rows with the look fields (the stage table 3.1 as the board was drawn when
    /// it was a table in this file: sunken dirt, mud, shallow and deep water, lava and toxic bog; rubble's stones; the
    /// standing tree; the wall block; the flat colours; the wall, water and lava faces).
    fn looks() -> emberwake_data::Catalog {
        emberwake_data::Catalog::from_value(&json!({"terrain": [
            {"id": "grass", "color": "#3c6838"},
            {"id": "dirt", "relief": "sunken", "relief_depth": 2, "color": "#685438"},
            {"id": "stone", "color": "#5c5c64"},
            {"id": "sand", "color": "#aaa078"},
            {"id": "snow", "color": "#aaa078"},
            {"id": "shallow_water", "relief": "sunken", "relief_depth": 3, "color": "#385c8c", "face": "water"},
            {"id": "deep_water", "relief": "sunken", "relief_depth": 5, "color": "#1e366e", "face": "water"},
            {"id": "mud", "relief": "sunken", "relief_depth": 1},
            {"id": "lava", "relief": "sunken", "relief_depth": 5, "color": "#963214", "face": "lava"},
            {"id": "wall", "relief": "block", "color": "#2e2e36", "face": "wall"},
            {"id": "tree", "relief": "standing", "prop": "tree", "color": "#1e4628"},
            {"id": "rubble", "prop": "stones", "color": "#5c5c64"},
            {"id": "toxic_bog", "relief": "sunken", "relief_depth": 2},
        ]}))
    }

    fn zone(id: &str) -> Grid {
        let mut g = Grid::new(false, Vec::new(), TerrainTable::from_catalog(&looks()), 0);
        g.apply_zone(id);
        g
    }

    fn stage() -> Geom {
        Geom::new(View::Stage, &Tuning::default())
    }

    fn game_art() -> Art {
        Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")))
    }

    /// Check 1: the cells, the foot point, a row-0 token inside the interior, and the point-to-cell map.
    #[test]
    fn the_stage_geometry_maps_cells_feet_and_points() {
        let g = stage();
        assert_eq!(g.foot_inset, 5, "the default foot inset");
        assert_eq!(g.token_px, 40, "the default token size");
        assert_eq!(g.cell_rect(V2::new(0, 0)), Rect::new(128, 82, 32, 24));
        assert_eq!(g.cell_rect(V2::new(11, 8)), Rect::new(480, 274, 32, 24));
        assert_eq!(g.foot(V2::new(5, 4)), (304, 197));
        let tr = g.token_rect(V2::new(3, 0), 0, 0);
        assert_eq!(tr.y, 62, "a row-0 token starts at y 62");
        assert!(INTERIOR.contains(tr.x, tr.y) && tr.y + tr.h <= INTERIOR.y + INTERIOR.h, "inside the interior");
        assert_eq!(g.chest(V2::new(5, 4)), (304, 197 - 40 / 3));
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let r = g.cell_rect(V2::new(x, y));
                assert_eq!(g.cell_at(r.x + r.w / 2, r.y + r.h / 2), Some(V2::new(x, y)));
            }
        }
        assert_eq!(g.cell_at(300, 50), None, "the backdrop holds no cell");
        assert_eq!(g.cell_at(300, 305), None, "the front face holds no cell");
        assert_eq!(g.slide(V2::new(4, 3), V2::new(5, 4), 0.5), (-16, -12), "half a step back: half a cell each way");
        assert_eq!(FRONT_FACE, Rect::new(128, 298, 384, 12));
        assert_eq!(DROP_SHADOW.y + DROP_SHADOW.h, INTERIOR.y + INTERIOR.h, "the drop shadow ends at the interior's edge");
    }

    /// Check 10 (geometry): Flat is today's board (32 x 32 at (128, 30), tokens `flat_token_px` centred in the cell);
    /// the setting reads `flat` as Flat and anything else as Stage.
    #[test]
    fn the_flat_view_keeps_todays_board_and_the_setting_defaults_to_stage() {
        let t = Tuning::default();
        let f = Geom::new(View::Flat, &t);
        assert_eq!(f.cell_rect(V2::new(0, 0)), Rect::new(128, 30, 32, 32));
        assert_eq!(f.cell_rect(V2::new(11, 8)), Rect::new(480, 286, 32, 32));
        let tr = f.token_rect(V2::new(2, 1), 0, 0);
        assert_eq!((tr.w, tr.h), (t.flat_token_px, t.flat_token_px));
        assert_eq!((tr.x, tr.y), (128 + 64 + (32 - t.flat_token_px) / 2, 30 + 32 + (32 - t.flat_token_px) / 2));
        assert_eq!(f.cell_at(130, 32), Some(V2::new(0, 0)));
        assert_eq!(View::of_setting(&Value::from("flat")), View::Flat);
        assert_eq!(View::of_setting(&Value::from("stage")), View::Stage);
        assert_eq!(View::of_setting(&Value::from("isometric")), View::Stage, "an unknown value is Stage");
        assert_eq!(View::of_setting(&Value::Null), View::Stage);
        assert_eq!(View::of_setting(&crate::screens::default_settings()["battle_view"]), View::Stage, "the default");
        assert_eq!(VIEWS[0].0, "stage");
    }

    /// PR5-33: the shipped terrain rows give the fixture's looks (so `TODAYS_FRAME`, drawn on `looks()`, stays the
    /// shipped board): every terrain type of the shipped table has the look the fixture table gives it.
    #[test]
    fn the_shipped_terrain_rows_give_the_fixture_looks() {
        let dir = emberwake_world::Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let shipped = TerrainTable::from_catalog(&emberwake_world::Data::open(&dir).expect("data").0.cat);
        let fixture = TerrainTable::from_catalog(&looks());
        for id in fixture.ids.iter().chain(&shipped.ids) {
            assert_eq!(shipped.look(id), fixture.look(id), "{id}");
        }
    }

    #[test]
    fn the_stage_table_sorts_every_terrain() {
        let g = zone("stone_arena");
        for id in emberwake_battle::grid::TERRAIN_IDS {
            let row = stage_of(&g, id);
            assert_eq!(row.depth > 0, row.kind == Kind::Sunken, "{id}: only sunken cells have a depth");
        }
        assert_eq!(stage_of(&g, "lava"), StageRow { kind: Kind::Sunken, depth: 5, prop: None });
        assert_eq!(stage_of(&g, "rubble").prop, Some("stones"));
        assert_eq!(stage_of(&g, "tree").kind, Kind::Standing);
        assert_eq!(stage_of(&g, "wall").kind, Kind::Block);
        assert_eq!(stage_of(&g, "no_such_terrain"), StageRow { kind: Kind::Ground, depth: 0, prop: None });
        assert_eq!((terrain_color(&g, "lava"), terrain_color(&g, "no_such_terrain")), (rgb(150, 50, 20), DEFAULT_TERRAIN_COLOR));
        let bare = Grid::new(false, Vec::new(), TerrainTable::builtin(), 0);
        assert!(emberwake_battle::grid::TERRAIN_IDS.iter().all(|id| stage_of(&bare, id) == StageRow { kind: Kind::Ground, depth: 0, prop: None }), "rows without looks: flat ground");
        assert_eq!(terrain_color(&bare, "lava"), DEFAULT_TERRAIN_COLOR);
    }

    /// Check 2: the ground under a standing prop.
    #[test]
    fn a_tree_stands_on_the_ground_most_of_its_neighbours_have() {
        let forest = zone("forest_clearing");
        assert_eq!(forest.terrain_id(V2::new(0, 0)), "tree");
        assert_eq!(ground_under(&forest, V2::new(0, 0)), "grass", "only trees around: grass");
        assert_eq!(ground_under(&forest, V2::new(10, 2)), "grass");
        let pass = zone("frozen_pass");
        assert_eq!(pass.terrain_id(V2::new(5, 0)), "tree");
        assert_eq!(ground_under(&pass, V2::new(5, 0)), "snow");
        // a tree with only block and standing neighbours: walls around a tree
        let mut boxed = zone("stone_arena");
        let idx = |x: i32, y: i32| (y * GRID_W + x) as usize;
        let id = |t: &str| emberwake_battle::grid::TERRAIN_IDS.iter().position(|x| *x == t).unwrap();
        boxed.terrain[idx(5, 5)] = id("tree");
        boxed.terrain[idx(5, 4)] = id("wall");
        boxed.terrain[idx(6, 5)] = id("tree");
        boxed.terrain[idx(5, 6)] = id("wall");
        boxed.terrain[idx(4, 5)] = id("wall");
        assert_eq!(ground_under(&boxed, V2::new(5, 5)), "grass");
        // ties: the first in the order north, east, south, west
        boxed.terrain[idx(5, 4)] = id("sand");
        boxed.terrain[idx(5, 6)] = id("snow");
        assert_eq!(ground_under(&boxed, V2::new(5, 5)), "sand");
    }

    /// Check 3: the lips of sunken cells (band = depth - north depth + 1).
    #[test]
    fn sunken_cells_show_a_lip_under_a_shallower_north_neighbour() {
        let forest = zone("forest_clearing");
        assert_eq!((forest.terrain_id(V2::new(0, 4)), forest.terrain_id(V2::new(0, 3))), ("dirt", "grass"));
        assert_eq!(lips(&forest, V2::new(0, 4)).0, stage_of(&forest, "dirt").depth + 1, "a 3 px band");
        assert_eq!(lips(&forest, V2::new(0, 4)).0, 3);
        assert_eq!((forest.terrain_id(V2::new(4, 3)), forest.terrain_id(V2::new(4, 2))), ("dirt", "dirt"));
        assert_eq!(lips(&forest, V2::new(4, 3)).0, 0, "dirt under dirt: none");
        assert!(lips(&forest, V2::new(4, 2)).0 > 0, "dirt under grass: a band");
        let cavern = zone("lava_cavern");
        assert_eq!((cavern.terrain_id(V2::new(1, 1)), cavern.terrain_id(V2::new(1, 0))), ("lava", "wall"));
        assert_eq!(lips(&cavern, V2::new(1, 1)).0, 6, "lava under a wall");
        assert_eq!(lips(&forest, V2::new(1, 3)), (0, false), "ground has no lips");
        // the bottom row: dirt over grass to the south
        assert_eq!(forest.terrain_id(V2::new(0, 5)), "grass");
        assert!(lips(&forest, V2::new(0, 4)).1);
    }

    /// Check 8 (faces): a wall over a wall draws no face; a row-8 wall does; the rects follow.
    #[test]
    fn a_block_shows_its_face_unless_a_block_stands_south() {
        let cavern = zone("lava_cavern");
        assert_eq!((cavern.terrain_id(V2::new(0, 0)), cavern.terrain_id(V2::new(0, 1))), ("wall", "wall"));
        assert!(!block_face(&cavern, V2::new(0, 0)), "a wall with a wall south");
        assert!(block_face(&cavern, V2::new(0, 8)), "a row-8 wall");
        assert!(block_face(&cavern, V2::new(1, 0)), "a wall over lava");
        assert_eq!(block_rect(&cavern, V2::new(0, 0)), Rect::new(128, 70, 32, 24), "the cap only, lifted 12 into the backdrop");
        assert_eq!(block_rect(&cavern, V2::new(1, 0)), Rect::new(160, 70, 32, 36), "cap and face");
    }

    /// One PNG of `w` x `h` (opaque grey) at `root/rel`.
    fn png(root: &Path, rel: &str, w: u32, h: u32) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&vec![128u8; (w * h * 4) as usize]).unwrap();
    }

    fn scratch(tag: &str) -> aegis_core::scratch::Scratch {
        let root = aegis_core::scratch::Scratch::new(&format!("emberwake_stage_{tag}"));
        root
    }

    /// Check 8 (art): a missing stage tile uses the squashed `common` tile; a missing `wall_face` the darkened cap; a
    /// missing tree prop the flat tree tile (nothing stands); with the art present each draws its own.
    #[test]
    fn missing_stage_art_falls_back_to_the_flat_pieces() {
        let root = scratch("fallback");
        png(&root, "tilesets/common/grass/v0.png", 32, 32);
        png(&root, "tilesets/common/tree/v0.png", 32, 32);
        png(&root, "tilesets/common/wall/v0.png", 32, 32);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(tile_art(&mut art, "grass", "", 2, 3), TileArt::Common("tilesets/common/grass/v0.png".into()));
        assert_eq!(tile_art(&mut art, "lava", "", 2, 3), TileArt::Colour, "neither: the colour");
        assert_eq!(face_art(&mut art, "", 2, 3), None, "the face falls back to the darkened cap");
        assert!(darkened_cap(&mut art, "wall", "", V2::new(2, 3)).is_some_and(|img| img.h == CELL_H / 2 && img.px[0] == 96), "the cap's top rows at 75 %");
        assert_eq!(prop_art(&mut art, "tree", 2, 3), None);
        let forest = zone("forest_clearing");
        assert!(board_items(&mut art, &forest).is_empty(), "no tree art: nothing stands");
        assert_eq!(tile_art(&mut art, "tree", "", 0, 0), TileArt::Common("tilesets/common/tree/v0.png".into()), "the flat tree tile");
        // with the stage art
        let full = scratch("full");
        png(&full, "tilesets/common_stage/grass/v0.png", 128, 96);
        png(&full, "tilesets/common_stage/wall_face/v0.png", 128, 48);
        png(&full, "props/common/tree/v0.png", 40, 52);
        let mut art = Art::layered(Layers::of(vec![full.to_path_buf(), root.to_path_buf()]));
        assert_eq!(tile_art(&mut art, "grass", "", 2, 3), TileArt::Stage("tilesets/common_stage/grass/v0.png".into()));
        assert_eq!(face_art(&mut art, "", 2, 3), Some("tilesets/common_stage/wall_face/v0.png".into()));
        assert_eq!(prop_art(&mut art, "tree", 2, 3), Some(("props/common/tree/v0.png".into(), (40, 52))));
        let items = board_items(&mut art, &forest);
        let tree = items.iter().find(|p| p.item == Item::Prop(V2::new(10, 2))).expect("the tree at (10, 2) stands");
        assert_eq!(tree.rect, Rect::new(128 + 320 - 4, 82 + 72 - 52, 40, 52), "bottom-centred on its cell");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&full);
    }

    /// Check 9: the zone's backdrop, else the gradient (no file).
    #[test]
    fn a_zone_loads_its_backdrop_and_an_unknown_zone_draws_the_gradient() {
        let mut art = game_art();
        assert_eq!(backdrop_art(&mut art, "forest_clearing"), Some("backgrounds/battle/forest_clearing.png".into()));
        assert_eq!(art.size_of("backgrounds/battle/forest_clearing.png"), Some((BACKDROP.w, BACKDROP.h)), "drawn 1:1");
        assert_eq!(backdrop_art(&mut art, "no_such_zone"), None);
        assert_eq!(backdrop_art(&mut art, ""), None);
        assert_eq!(far_ground(&zone("forest_clearing")), "grass");
        assert_eq!(far_ground(&zone("frozen_pass")), "snow");
        assert_eq!(far_ground(&zone("lava_cavern")), "grass", "row 0 all walls: none, grass");
    }

    /// HOOKS1 ZN3: a data zone row draws its backdrop key when the file exists, else the gradient to its `ground`
    /// (not the far ground of the board); a code zone as before, also when a (bad) row reuses its id.
    #[test]
    fn a_data_zone_keys_its_backdrop_and_its_gradient_ground() {
        let mut art = game_art();
        // `zz_test_zone`: a test-only zone id with no file under backgrounds/battle/
        let one = |id: &str, backdrop: &str, ground: &str| {
            let z = ZoneDef::from_obj(json!({"id": id, "name": "Test", "backdrop": backdrop, "ground": ground}).as_object().unwrap());
            IndexMap::from([(id.to_string(), z)])
        };
        let board = zone("forest_clearing");
        assert!(!art.has("backgrounds/battle/zz_test_zone.png"), "fixture");
        let missing = one("zz_test_zone", "", "stone");
        assert_eq!(backdrop_source(&mut art, &board, "zz_test_zone", &missing), (None, "stone"), "no file: the gradient to the row's ground");
        let keyed = one("zz_test_zone", "backgrounds/battle/forest_clearing.png", "bridge");
        assert_eq!(backdrop_source(&mut art, &board, "zz_test_zone", &keyed), (Some("backgrounds/battle/forest_clearing.png".into()), "bridge"), "the row's key");
        let gone = one("zz_test_zone", "backgrounds/battle/zz_none.png", "sand");
        assert_eq!(backdrop_source(&mut art, &board, "zz_test_zone", &gone), (None, "sand"), "a missing key file: the gradient");
        let code = (Some("backgrounds/battle/forest_clearing.png".to_string()), "grass");
        assert_eq!(backdrop_source(&mut art, &board, "forest_clearing", &IndexMap::new()), code, "a code zone unchanged");
        let shadow = one("forest_clearing", "backgrounds/battle/zz_none.png", "sand");
        assert_eq!(backdrop_source(&mut art, &board, "forest_clearing", &shadow), code, "a row reusing a code id is ignored");
    }

    #[test]
    fn the_front_face_follows_row_8() {
        let cavern = zone("lava_cavern");
        assert_eq!(face_column(&cavern, 0), ("wall", FaceBody::Wall));
        let river = zone("river_crossing");
        assert_eq!(river.terrain_id(V2::new(8, 8)), "shallow_water");
        assert_eq!(face_column(&river, 8).1, FaceBody::Water);
        let forest = zone("forest_clearing");
        assert_eq!(face_column(&forest, 0), ("grass", FaceBody::Earth), "a tree's column shows the ground under it");
        assert_eq!(face_column(&forest, 4), ("grass", FaceBody::Earth));
    }

    /// A body rect of a 40 px token on `c` from a real body's solid bounds.
    fn body_on(art: &mut Art, g: &Geom, key: &str, c: V2) -> Rect {
        let b = art.body_bounds(key, "", g.token_px, false).expect("the body loads");
        let tr = g.token_rect(c, 0, 0);
        Rect::new(tr.x + b.x, tr.y + b.y, b.w, b.h)
    }

    /// Checks 4 and 5: the feet order (unit north of a tree, the tree, the unit south of it); a block before a unit at
    /// equal feet; the southern of two units last. The unit behind the tree is outlined in its side colour, the one
    /// in front is not; a selected hero north of another unit is outlined gold; an unselected, unhovered unit behind
    /// a unit is not.
    #[test]
    fn the_feet_pass_sorts_by_feet_and_outlines_what_is_covered() {
        let mut art = game_art();
        let g = stage();
        let forest = zone("forest_clearing");
        assert_eq!((forest.terrain_id(V2::new(10, 1)), forest.terrain_id(V2::new(10, 2)), forest.terrain_id(V2::new(10, 3))), ("grass", "tree", "grass"));
        let board = board_items(&mut art, &forest);
        assert!(board.iter().any(|p| p.item == Item::Prop(V2::new(10, 2))), "the tree stands (STAGE1-T art)");
        let (n, s) = (V2::new(10, 1), V2::new(10, 3));
        let mut items = board.clone();
        items.push(unit_item(0, g.foot(n), body_on(&mut art, &g, "warrior", n)));
        items.push(unit_item(1, g.foot(s), body_on(&mut art, &g, "warrior", s)));
        let order = feet_order(&items);
        let pos = |it: Item| order.iter().position(|p| p.item == it).unwrap();
        assert!(pos(Item::Unit(0)) < pos(Item::Prop(V2::new(10, 2))) && pos(Item::Prop(V2::new(10, 2))) < pos(Item::Unit(1)));
        assert_eq!(through(&order, pos(Item::Unit(0)), false, false, Reveal::Always), Some(Through::Side), "behind the canopy");
        assert_eq!(through(&order, pos(Item::Unit(1)), false, false, Reveal::Always), None, "in front of the tree");
        // a block and a unit at equal feet: the block first
        let cell = g.cell_rect(V2::new(3, 3));
        let block = Placed { item: Item::Block(V2::new(3, 3)), foot_y: cell.y + cell.h, x: cell.x, rect: cell };
        let unit = unit_item(2, (cell.x + 16, cell.y + cell.h), cell);
        assert_eq!(feet_order(&[unit, block]).iter().map(|p| p.item).collect::<Vec<_>>(), vec![block.item, unit.item]);
        // two units in one column: the southern last; the northern selected is outlined gold, else not at all
        let open = zone("stone_arena");
        let (a, b) = (V2::new(5, 3), V2::new(5, 4));
        let two = [unit_item(1, g.foot(b), body_on(&mut art, &g, "warrior", b)), unit_item(0, g.foot(a), body_on(&mut art, &g, "warrior", a))];
        let mut with_board = board_items(&mut art, &open);
        with_board.extend(two);
        let order = feet_order(&with_board);
        assert_eq!(order.iter().map(|p| p.item).collect::<Vec<_>>(), vec![Item::Unit(0), Item::Unit(1)]);
        assert_eq!(through(&order, 0, true, false, Reveal::Always), Some(Through::Selected), "a selected hero north of a unit");
        assert_eq!(through(&order, 0, false, true, Reveal::Always), Some(Through::Hovered));
        assert_eq!(through(&order, 0, false, false, Reveal::Always), None, "unselected and unhovered behind a unit");
        assert_eq!(through(&order, 1, true, false, Reveal::Always), None, "nothing in front of the southern unit");
        // STAGE1-O: by default only the selected unit shows through; Alt held shows every covered unit, units included
        assert_eq!(through(&order, 0, true, false, Reveal::Selected), Some(Through::Selected), "selected, by default");
        assert_eq!(through(&order, 0, false, true, Reveal::Selected), None, "hovered alone stays hidden by default");
        assert_eq!(through(&order, 0, false, false, Reveal::Held), Some(Through::Side), "Alt held: behind a unit");
        assert_eq!(through(&order, 1, false, false, Reveal::Held), Some(Through::Side), "Alt held: uncovered units are outlined too");
    }

    /// STAGE1-O: the canopy hides a unit by default and reveals it while Alt is held or with the setting Always; the
    /// setting's values map to the reveal, Alt winning.
    #[test]
    fn units_behind_cover_show_only_when_selected_alt_held_or_always() {
        let mut art = game_art();
        let g = stage();
        let forest = zone("forest_clearing");
        let n = V2::new(10, 1);
        let mut items = board_items(&mut art, &forest);
        items.push(unit_item(0, g.foot(n), body_on(&mut art, &g, "warrior", n)));
        let order = feet_order(&items);
        let at = order.iter().position(|p| p.item == Item::Unit(0)).unwrap();
        assert_eq!(through(&order, at, false, false, Reveal::Selected), None, "behind the canopy, hidden by default");
        assert_eq!(through(&order, at, true, false, Reveal::Selected), Some(Through::Selected));
        assert_eq!(through(&order, at, false, false, Reveal::Held), Some(Through::Side));
        assert_eq!(through(&order, at, false, false, Reveal::Always), Some(Through::Side));
        assert_eq!(Reveal::of(&Value::from("selected"), false), Reveal::Selected);
        assert_eq!(Reveal::of(&Value::from("always"), false), Reveal::Always);
        assert_eq!(Reveal::of(&Value::from("always"), true), Reveal::Held, "Alt wins");
        assert_eq!(Reveal::of(&Value::Null, false), Reveal::Selected, "missing: the default");
        assert_eq!(Reveal::of(&crate::screens::default_settings()["see_through"], false), Reveal::Selected, "the default setting");
        assert_eq!(SEE_THROUGH[0].0, "selected", "the first value is the default");
    }

    /// Check 7: an arrow ends `ARROW_END` flat px short of the target's foot point and starts `ARROW_START` from its
    /// own; move and flee tells are dashed; arrows draw before the feet pass.
    #[test]
    fn ground_arrows_stop_at_the_foot_ring_and_draw_under_the_units() {
        let g = stage();
        let flat = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).powi(2) + ((b.1 - a.1) / ARROW_Y).powi(2)).sqrt();
        for (from, to) in [(V2::new(2, 4), V2::new(7, 4)), (V2::new(9, 1), V2::new(3, 6)), (V2::new(5, 2), V2::new(5, 7))] {
            let (f, t) = (g.foot(from), g.foot(to));
            let (f, t) = ((f.0 as f32, f.1 as f32), (t.0 as f32, t.1 as f32));
            let (x0, y0, x1, y1) = ground_arrow(f, t).expect("far enough for an arrow");
            assert!((flat((x1, y1), t) - ARROW_END).abs() < 0.01, "ends 12 flat px short of the foot point");
            assert!((flat(f, (x0, y0)) - ARROW_START).abs() < 0.01, "starts 10 flat px out");
        }
        assert_eq!(ground_arrow((10.0, 10.0), (20.0, 10.0)), None, "too close");
        assert!(arrow_dash("move") > 0.0 && arrow_dash("flee") > 0.0 && arrow_dash("attack") == 0.0);
        let at = |p: Pass| passes().iter().position(|x| *x == p).unwrap();
        assert!(at(Pass::Arrows) < at(Pass::Feet), "arrows before the feet pass");
        assert!(at(Pass::Ground) < at(Pass::Markers) && at(Pass::Feet) < at(Pass::Through) && at(Pass::Through) < at(Pass::Top));
        assert_eq!(passes().first(), Some(&Pass::Backdrop));
    }

    // ---- LOOK1-L1: stage sets ---------------------------------------------------------------------------------------

    /// One PNG at `root/rel`, its colour stepped across both axes (a flat picture would hide a sampling change).
    fn ramp(root: &Path, rel: &str, w: u32, h: u32, col: [u8; 3]) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                px.extend([col[0].wrapping_add((x * 3) as u8), col[1].wrapping_add((y * 3) as u8), col[2], 255]);
            }
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
    }

    /// A fixture layer of today's art: stage tiles for most terrains (two variants of grass), a `common` tile for the
    /// rest, the wall face, the tree and the stones.
    fn todays_layer(root: &Path) {
        for (k, id) in emberwake_battle::grid::TERRAIN_IDS.iter().enumerate() {
            let col = [(k * 13) as u8, (k * 29) as u8, (k * 47) as u8];
            if matches!(*id, "lava" | "ice") {
                ramp(root, &format!("tilesets/common/{id}/v0.png"), 32, 32, col);
            } else {
                ramp(root, &format!("tilesets/common_stage/{id}/v0.png"), 64, 48, col);
            }
        }
        ramp(root, "tilesets/common_stage/grass/v1.png", 64, 48, [40, 200, 90]);
        ramp(root, "tilesets/common_stage/wall_face/v0.png", 64, 24, [200, 90, 40]);
        ramp(root, "props/common/tree/v0.png", 40, 52, [20, 120, 30]);
        ramp(root, "props/common/stones/v0.png", 32, 24, [120, 120, 130]);
    }

    /// FNV-1a over the canvas of the board's passes on each zone, with stage set `set`.
    fn frame_hash(art: &mut Art, zones: &[&str], set: &str) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for id in zones {
            let g = paint(art, &zone(id), set, 40, 1, (0, 0));
            for b in &g.canvas.px {
                h = (h ^ *b as u64).wrapping_mul(0x100000001b3);
            }
        }
        h
    }

    /// The board's passes on `grid` with stage set `set`, on a canvas `scale` x 640 x 360, shifted by `shift`.
    fn paint(art: &mut Art, grid: &Grid, set: &str, grid_alpha: u8, scale: i32, shift: (i32, i32)) -> aegis_ui::ui::Gfx {
        let mut g = crate::hud::hud_tests::gfx();
        g.canvas = aegis_ui::canvas::Canvas::new(640 * scale, 360 * scale);
        g.view = aegis_ui::painter::View::fit((640 * scale) as u32, (360 * scale) as u32);
        let mut p = Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
        p.clip(Some(INTERIOR));
        let interior = p.c.clip;
        p.shift(shift.0, shift.1);
        draw_backdrop(&mut p, art, grid, "", &IndexMap::new());
        draw_ground(&mut p, art, grid, set, grid_alpha);
        assert_eq!(p.c.clip, interior, "the ground pass gives the clip back");
        draw_prop_shadows(&mut p, art, grid);
        draw_face(&mut p, art, grid, set);
        for it in feet_order(&board_items(art, grid)) {
            match it.item {
                Item::Block(c) => draw_block(&mut p, art, grid, set, c),
                Item::Prop(c) => draw_prop(&mut p, art, grid, c),
                Item::Unit(_) => {}
            }
        }
        g
    }

    const ZONES: [&str; 5] = ["forest_clearing", "lava_cavern", "river_crossing", "frozen_pass", "stone_arena"];
    /// The fixture layer's frame over `ZONES` before LOOK1-L1 (printed on the code before the change).
    const TODAYS_FRAME: u64 = 0xe6c3440ef5f1669f;

    /// An empty `stage_set` draws today's frame; so does a set no layer holds, and a `stage_set` draws its own.
    #[test]
    fn an_empty_stage_set_draws_todays_frame() {
        let root = scratch("todays");
        todays_layer(&root);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(frame_hash(&mut art, &ZONES, ""), TODAYS_FRAME);
        assert_eq!(frame_hash(&mut art, &ZONES, "no_such_set"), TODAYS_FRAME, "a set with no files: common_stage");
        ramp(&root, "tilesets/zz_set/grass/v0.png", 64, 48, [250, 0, 250]);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_ne!(frame_hash(&mut art, &ZONES, "zz_set"), TODAYS_FRAME);
        assert_eq!(frame_hash(&mut art, &ZONES, ""), TODAYS_FRAME, "the same layer without the set named");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// DIORAMA1 D4: a set's `wall_foot` strip changes only row 0's top `WALL_FOOT_H` px of ground cells; a set without
    /// one draws today's frame.
    #[test]
    fn the_wall_foot_draws_on_row_0_only() {
        let root = scratch("wall_foot");
        todays_layer(&root);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        let grid = zone("stone_arena");
        let before = paint(&mut art, &grid, "zz_set", 0, 1, (0, 0));
        assert_eq!(wall_foot_art(&mut art, "zz_set", 0), None);
        ramp(&root, "tilesets/zz_set/wall_foot/v0.png", 64, 16, [250, 0, 250]);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(wall_foot_art(&mut art, "zz_set", 3).as_deref(), Some("tilesets/zz_set/wall_foot/v0.png"));
        let after = paint(&mut art, &grid, "zz_set", 0, 1, (0, 0));
        let (w, mut changed) = (before.canvas.w, Vec::new());
        for (i, (a, b)) in before.canvas.px.chunks(4).zip(after.canvas.px.chunks(4)).enumerate() {
            if a != b {
                changed.push(((i as i32) % w, (i as i32) / w));
            }
        }
        assert!(!changed.is_empty(), "the strip shows");
        assert!(changed.iter().all(|&(x, y)| (BOARD_Y..BOARD_Y + WALL_FOOT_H).contains(&y) && (BOARD_X..BOARD_X + GRID_W * CELL_W).contains(&x)), "only the top of row 0");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The pixels where two canvases of the same size differ, as (x, y) in canvas px.
    fn diff(a: &aegis_ui::ui::Gfx, b: &aegis_ui::ui::Gfx) -> Vec<(i32, i32)> {
        let w = a.canvas.w;
        a.canvas.px.chunks(4).zip(b.canvas.px.chunks(4)).enumerate().filter(|(_, (x, y))| x != y).map(|(i, _)| ((i as i32) % w, (i as i32) / w)).collect()
    }

    /// DIORAMA2 R2: a stage set's `front_face/<terrain>/` replaces the front face under the columns whose top is that
    /// terrain, and only there (the face band); a terrain without one keeps today's face.
    #[test]
    fn the_front_face_comes_from_the_stage_set_with_todays_fallback() {
        let root = scratch("front_face");
        todays_layer(&root);
        let grid = zone("stone_arena");
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        let before = paint(&mut art, &grid, "zz_face", 0, 1, (0, 0));
        let (top, _) = face_column(&grid, 0);
        let top = top.to_string();
        assert_eq!(front_face_art(&mut art, "zz_face", &top, 0), None);
        ramp(&root, &format!("tilesets/zz_face/front_face/{top}/v0.png"), 64, 24, [250, 0, 250]);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(front_face_art(&mut art, "zz_face", &top, 3), Some(format!("tilesets/zz_face/front_face/{top}/v0.png")));
        assert_eq!(front_face_art(&mut art, "", &top, 3), None, "no set named: today's face");
        let after = paint(&mut art, &grid, "zz_face", 0, 1, (0, 0));
        let changed = diff(&before, &after);
        assert!(!changed.is_empty(), "the face shows");
        assert!(changed.iter().all(|&(_, y)| (FRONT_FACE.y..FRONT_FACE.y + FACE_H).contains(&y)), "only the face band");
        let cols: Vec<i32> = (0..GRID_W).filter(|x| face_column(&grid, *x).0 == top).collect();
        assert!(changed.iter().all(|&(x, _)| cols.contains(&((x - BOARD_X) / CELL_W))), "only the columns topped by {top}: {cols:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// DIORAMA2 R1: the wall cap's lip: today's with no set or a set without a `stage.fx.json`, the set's own `lip`
    /// from it, bronze when it names none.
    #[test]
    fn the_wall_lip_takes_the_stage_sets_tone() {
        let root = scratch("lip");
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(lip_colour(&mut art, ""), LIP_TODAY);
        assert_eq!(lip_colour(&mut art, "zz_lip_none"), LIP_TODAY, "a set without the file: today's frame");
        for (set, body) in [("zz_lip_own", r##"{"lip": "#4a5060", "spots": []}"##), ("zz_lip_bare", r##"{"spots": []}"##)] {
            let fx = root.join(format!("tilesets/{set}/stage.fx.json"));
            std::fs::create_dir_all(fx.parent().unwrap()).unwrap();
            std::fs::write(&fx, body).unwrap();
        }
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(lip_colour(&mut art, "zz_lip_own"), rgb(0x4a, 0x50, 0x60));
        assert_eq!(lip_colour(&mut art, "zz_lip_bare"), LIP_BRONZE);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// DIORAMA2 R4: the wings draw only in the side gutters (`STAGE_CLIP` beyond `INTERIOR`), full panel colour at the
    /// panel's edge and the stage's own colours where the board starts.
    #[test]
    fn the_wings_fill_the_gutters_and_fade_to_the_panel() {
        let root = scratch("wings");
        todays_layer(&root);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        let grid = zone("stone_arena");
        let panel = rgb(20, 22, 30);
        let mut g = crate::hud::hud_tests::gfx();
        g.canvas = aegis_ui::canvas::Canvas::new(640, 360);
        g.view = aegis_ui::painter::View::fit(640, 360);
        let blank = g.canvas.px.clone();
        let mut p = Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
        p.clip(Some(STAGE_CLIP));
        draw_wings(&mut p, &mut art, &grid, "", "", &IndexMap::new(), panel);
        let px = |x: i32, y: i32| g.canvas.px[((y * 640 + x) * 4) as usize..((y * 640 + x) * 4 + 3) as usize].to_vec();
        let painted: Vec<(i32, i32)> = g.canvas.px.chunks(4).zip(blank.chunks(4)).enumerate().filter(|(_, (a, b))| a != b).map(|(i, _)| ((i as i32) % 640, (i as i32) / 640)).collect();
        assert!(!painted.is_empty());
        assert!(painted.iter().all(|&(x, y)| STAGE_CLIP.contains(x, y) && !INTERIOR.contains(x, y)), "only the gutters");
        let mid = BOARD_Y + 3 * CELL_H;
        assert_eq!(px(STAGE_CLIP.x, mid), vec![20, 22, 30], "the panel colour at the edge");
        assert_ne!(px(INTERIOR.x - 1, mid), vec![20, 22, 30], "the floor where the board starts");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// DIORAMA3 R5: a block joined to the grid's edge (through blocks) is the outer wall; one standing free inside is
    /// cover, drawn as the stage set's cover prop picked by the cell, in the block's own box; no set: its wall cap.
    #[test]
    fn a_block_inside_the_arena_stands_as_cover() {
        let root = scratch("cover");
        todays_layer(&root);
        let mut grid = zone("stone_arena");
        let idx = |x: i32, y: i32| (y * GRID_W + x) as usize;
        let id = |t: &str| emberwake_battle::grid::TERRAIN_IDS.iter().position(|x| *x == t).unwrap();
        for c in 0..(GRID_W * GRID_H) as usize {
            grid.terrain[c] = id("stone");
        }
        grid.terrain[idx(0, 3)] = id("wall");
        grid.terrain[idx(1, 3)] = id("wall");
        grid.terrain[idx(6, 4)] = id("wall");
        assert!(outer_wall(&grid, V2::new(1, 3)), "joined to the edge");
        assert!(!outer_wall(&grid, V2::new(6, 4)), "free inside");
        ramp(&root, "tilesets/zz_cover/cover/v0.png", 64, 72, [250, 0, 250]);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(cover_art(&mut art, &grid, "zz_cover", V2::new(6, 4)).as_deref(), Some("tilesets/zz_cover/cover/v0.png"));
        assert_eq!(cover_art(&mut art, &grid, "zz_cover", V2::new(1, 3)), None, "the outer wall keeps its cap");
        assert_eq!(cover_art(&mut art, &grid, "zz_cover", V2::new(5, 4)), None, "not a block");
        assert_eq!(cover_art(&mut art, &grid, "", V2::new(6, 4)), None, "no set named");
        let plain = paint(&mut art, &grid, "zz_none", 0, 1, (0, 0));
        let covered = paint(&mut art, &grid, "zz_cover", 0, 1, (0, 0));
        let r = stage_cell(V2::new(6, 4));
        let changed = diff(&plain, &covered);
        assert!(!changed.is_empty());
        assert!(changed.iter().all(|&(x, y)| x >= r.x && x < r.x + CELL_W && y >= r.y + r.h - COVER_H && y < r.y + r.h), "only the block's box (its contact shadow stays)");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// DIORAMA3 R3: the light spots come from the stage set's `stage.fx.json`, the zone's own first, else the default;
    /// an `at` off the backdrop is dropped; no file or no set: none. A pool lights the floor below its `u`.
    #[test]
    fn the_lanterns_light_pools_under_their_spots() {
        let root = scratch("light");
        let fx = root.join("tilesets/zz_light/stage.fx.json");
        std::fs::create_dir_all(fx.parent().unwrap()).unwrap();
        std::fs::write(&fx, r#"{"spots": [{"kind": "lantern", "at": [0.25, 0.3]}, {"kind": "lantern", "at": [1.5, 0.3]}], "zones": {"canal": {"spots": [{"kind": "lantern", "at": [0.75, 0.3]}]}}}"#).unwrap();
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(light_spots(&mut art, "zz_light", "alley"), [(0.25, 0.3)]);
        assert_eq!(light_spots(&mut art, "zz_light", "canal"), [(0.75, 0.3)]);
        assert_eq!(light_spots(&mut art, "zz_light_none", "alley"), []);
        assert_eq!(light_spots(&mut art, "", "alley"), []);
        let mut g = crate::hud::hud_tests::gfx();
        g.canvas = aegis_ui::canvas::Canvas::new(640, 360);
        g.view = aegis_ui::painter::View::fit(640, 360);
        let blank = g.canvas.px.clone();
        let mut p = Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
        p.clip(Some(STAGE_CLIP));
        draw_light(&mut p, &[(0.25, 0.3)]);
        let at = |px: &[u8], x: i32, y: i32| px[((y * 640 + x) * 4) as usize..((y * 640 + x) * 4 + 3) as usize].to_vec();
        let (cx, cy) = (BACKDROP.x + BACKDROP.w / 4, BOARD_Y + CELL_H * 7 / 10);
        let lit = at(&g.canvas.px, cx, cy);
        assert!(lit[0] > lit[2] && lit != at(&blank, cx, cy), "amber under the lantern: {lit:?}");
        let far = BACKDROP.x + BACKDROP.w * 3 / 4;
        assert_eq!(at(&g.canvas.px, far, cy + 40), at(&blank, far, cy + 40), "no light far from it (inside the vignette's margin)");
        let _ = std::fs::remove_dir_all(&root);
    }

    const UPPER: [u8; 3] = [200, 30, 30];
    const LOWER: [u8; 3] = [30, 30, 200];
    const PIECE_UPPER: [u8; 3] = [240, 200, 40];
    const PIECE_LOWER: [u8; 3] = [40, 220, 220];

    fn flat_png(root: &Path, rel: &str, w: u32, h: u32, pixel: impl Fn(u32, u32) -> [u8; 3]) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let c = pixel(x, y);
                px.extend([c[0], c[1], c[2], 255]);
            }
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
    }

    /// A two-colour set `zz_set` with the pair `floor` over `shallow_water`: each piece's quarters by its mask's bits,
    /// in colours of their own so a piece reads apart from the tiles under it.
    fn two_colour_set(root: &Path) {
        flat_png(root, "tilesets/zz_set/floor/v0.png", 64, 48, |_, _| UPPER);
        flat_png(root, "tilesets/zz_set/shallow_water/v0.png", 64, 48, |_, _| LOWER);
        for m in 1..=14u8 {
            let rel = transition_piece("zz_set", "floor", "shallow_water", m);
            flat_png(root, &rel, 64, 48, |x, y| if m & (1 << ((y >= 24) as u8 * 2 + (x >= 32) as u8)) != 0 { PIECE_UPPER } else { PIECE_LOWER });
        }
    }

    /// A board of `floor` with the given cells set.
    fn board_of(cells: &[(i32, i32, &str)]) -> Grid {
        let mut g = zone("stone_arena");
        let id = |t: &str| emberwake_battle::grid::TERRAIN_IDS.iter().position(|x| *x == t).unwrap();
        g.terrain.iter_mut().for_each(|t| *t = id("floor"));
        for (x, y, t) in cells {
            g.terrain[(y * GRID_W + x) as usize] = id(t);
        }
        g
    }

    fn at_px(g: &aegis_ui::ui::Gfx, x: i32, y: i32) -> [u8; 3] {
        let i = ((y * g.canvas.w + x) * 4) as usize;
        [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2]]
    }

    /// LOOK1 5.3: a lone water cell takes a piece on each of its 4 corners, centred there, each quarter by its bit;
    /// lips and grid lines draw over the pieces; the other cells keep their tiles.
    #[test]
    fn a_lone_water_cell_takes_pieces_on_its_four_corners() {
        let root = scratch("lone");
        two_colour_set(&root);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        let (x, y) = (5, 4);
        let grid = board_of(&[(x, y, "shallow_water")]);
        // Each corner's mask: all bits but the one where the water cell sits.
        let corners = [(x, y, 8u8), (x + 1, y, 4), (x, y + 1, 2), (x + 1, y + 1, 1)];
        let mut found = stage_pieces(&mut art, &grid, "zz_set");
        found.sort();
        let mut want: Vec<(i32, i32, String)> = corners.iter().map(|&(cx, cy, bit)| (cx, cy, transition_piece("zz_set", "floor", "shallow_water", 15 - bit))).collect();
        want.sort();
        assert_eq!(found, want);
        let g = paint(&mut art, &grid, "zz_set", 0, 1, (0, 0));
        let cell = stage_cell(V2::new(x, y));
        for (cx, cy, bit) in corners {
            let r = piece_rect(cx, cy);
            assert_eq!((r.x + r.w / 2, r.y + r.h / 2), (BOARD_X + cx * CELL_W, BOARD_Y + cy * CELL_H), "centred on the corner");
            for (b, qx, qy) in [(1u8, 8, 6), (2, 24, 6), (4, 8, 18), (8, 24, 18)] {
                let want = if b == bit { PIECE_LOWER } else { PIECE_UPPER };
                assert_eq!(at_px(&g, r.x + qx, r.y + qy), want, "corner ({cx}, {cy}) quarter {b}");
            }
        }
        assert_eq!(at_px(&g, cell.x - 40, cell.y + 12), UPPER, "a cell away keeps its tile");
        assert_eq!(at_px(&g, cell.x + 8, cell.y), [118, 116, 62], "the lip draws over the pieces");
        let lined = paint(&mut art, &grid, "zz_set", 255, 1, (0, 0));
        assert_eq!(at_px(&lined, cell.x + 8, cell.y), [0, 0, 0], "the grid draws over the lip");
        // the set named without pieces, and no set: the water's own tile
        let plain = scratch("lone_plain");
        flat_png(&plain, "tilesets/zz_set/floor/v0.png", 64, 48, |_, _| UPPER);
        flat_png(&plain, "tilesets/zz_set/shallow_water/v0.png", 64, 48, |_, _| LOWER);
        let mut art = Art::layered(Layers::of(vec![plain.to_path_buf()]));
        assert!(stage_pieces(&mut art, &grid, "zz_set").is_empty(), "no pieces in the set");
        assert_eq!(at_px(&paint(&mut art, &grid, "zz_set", 0, 1, (0, 0)), cell.x + 8, cell.y + 12), LOWER);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert!(stage_pieces(&mut art, &grid, "").is_empty(), "no set, no pieces");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&plain);
    }

    /// A piece on the board's edge is clipped to the board, not spilled on the backdrop, a shake moving the clip with
    /// the board; off-grid cells read as the nearest cell.
    #[test]
    fn a_piece_on_the_edge_stays_on_the_board() {
        let root = scratch("edge");
        two_colour_set(&root);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        let grid = board_of(&[(5, 0, "shallow_water")]);
        let piece = |m: u8| transition_piece("zz_set", "floor", "shallow_water", m);
        let found = stage_pieces(&mut art, &grid, "zz_set");
        // The top corners read the row above as row 0: floor on the outer side, water on the inner.
        assert!(found.contains(&(5, 0, piece(1 | 4))) && found.contains(&(6, 0, piece(2 | 8))), "{found:?}");
        let bare = paint(&mut art, &grid, "", 0, 1, (0, 0));
        let g = paint(&mut art, &grid, "zz_set", 0, 1, (0, 0));
        let (x, top) = (BOARD_X + 5 * CELL_W - 8, BOARD_Y);
        assert_eq!(at_px(&g, x, top - 1), at_px(&bare, x, top - 1), "nothing over the backdrop");
        assert_eq!(at_px(&g, x, top), PIECE_UPPER, "the piece from the board's first row");
        let shaken = paint(&mut art, &grid, "zz_set", 0, 1, (0, 2));
        assert_eq!(at_px(&shaken, x, top + 1), at_px(&paint(&mut art, &grid, "", 0, 1, (0, 2)), x, top + 1), "the clip moves with the board");
        assert_eq!(at_px(&shaken, x, top + 2), PIECE_UPPER);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A corner of a third ground takes no piece (an opaque piece would paint over it); a block counts as upper; a
    /// corner of only water and blocks takes none; a tree with art shows the ground under it; a pair the set lacks
    /// takes none.
    #[test]
    fn only_a_corner_of_the_sets_pair_takes_a_piece() {
        let root = scratch("third");
        two_colour_set(&root);
        ramp(&root, "props/common/tree/v0.png", 40, 52, [20, 120, 30]);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        let corner = |art: &mut Art, g: &Grid, cx: i32, cy: i32| stage_pieces(art, g, "zz_set").into_iter().find(|p| (p.0, p.1) == (cx, cy)).map(|p| p.2);
        let piece = |m: u8| Some(transition_piece("zz_set", "floor", "shallow_water", m));
        let w = (5, 4, "shallow_water");
        let third = board_of(&[w, (4, 3, "grass")]);
        assert_eq!(corner(&mut art, &third, 5, 4), None, "floor, water and grass");
        assert_eq!(corner(&mut art, &third, 6, 5), piece(15 - 1), "the far corner keeps its piece");
        let walled = board_of(&[w, (4, 3, "wall")]);
        assert_eq!(corner(&mut art, &walled, 5, 4), piece(15 - 8), "a wall counts as upper");
        let tree = board_of(&[w, (4, 3, "tree")]);
        assert_eq!(corner(&mut art, &tree, 5, 4), piece(15 - 8), "a tree stands on the floor around it");
        let shut = board_of(&[w, (4, 3, "wall"), (5, 3, "wall"), (4, 4, "wall")]);
        assert_eq!(corner(&mut art, &shut, 5, 4), None, "water and walls only");
        let other = board_of(&[(5, 4, "lava")]);
        assert!(stage_pieces(&mut art, &other, "zz_set").is_empty(), "the set lacks floor over lava");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// LOOK1 5.4: a terrain the set lacks falls back to `common_stage` (tile and wall face), and a set's own file draws
    /// at its art px: on a 2x canvas each of its 64 x 48 px shows, a fallback tile as before.
    #[test]
    fn a_terrain_missing_in_the_set_falls_back_to_common_stage() {
        let root = scratch("missing");
        two_colour_set(&root);
        flat_png(&root, "tilesets/common_stage/floor/v0.png", 64, 48, |_, _| [9, 9, 9]);
        flat_png(&root, "tilesets/common_stage/grass/v0.png", 64, 48, |x, _| if x % 2 == 0 { [10, 90, 10] } else { [90, 10, 90] });
        flat_png(&root, "tilesets/common_stage/wall_face/v0.png", 64, 24, |_, _| [90, 90, 10]);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(tile_art(&mut art, "floor", "zz_set", 2, 3), TileArt::Stage("tilesets/zz_set/floor/v0.png".into()), "the set first");
        assert_eq!(tile_art(&mut art, "floor", "", 2, 3), TileArt::Stage("tilesets/common_stage/floor/v0.png".into()));
        assert_eq!(tile_art(&mut art, "grass", "zz_set", 2, 3), TileArt::Stage("tilesets/common_stage/grass/v0.png".into()), "missing in the set");
        assert_eq!(face_art(&mut art, "zz_set", 2, 3), Some("tilesets/common_stage/wall_face/v0.png".into()), "the face too");
        let c = stage_cell(V2::new(3, 3));
        let grid = board_of(&[(3, 3, "grass")]);
        assert_eq!(at_px(&paint(&mut art, &grid, "zz_set", 0, 1, (0, 0)), c.x + 8, c.y + 8), at_px(&paint(&mut art, &grid, "", 0, 1, (0, 0)), c.x + 8, c.y + 8), "the fallback draws as today");
        flat_png(&root, "tilesets/zz_set/wall_face/v0.png", 64, 24, |_, _| [77, 7, 7]);
        flat_png(&root, "tilesets/zz_set/stone/v0.png", 64, 48, |x, _| if x % 2 == 0 { [1, 2, 3] } else { [200, 201, 202] });
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        assert_eq!(face_art(&mut art, "zz_set", 2, 3), Some("tilesets/zz_set/wall_face/v0.png".into()), "the set's own face");
        let walled = board_of(&[(3, 3, "wall")]);
        let g = paint(&mut art, &walled, "zz_set", 0, 1, (0, 0));
        assert_eq!(at_px(&g, c.x + 8, c.y + CELL_H - 4), [77, 7, 7], "the block draws the set's face");
        let stone = board_of(&[(3, 3, "stone")]);
        let big = paint(&mut art, &stone, "zz_set", 0, 2, (0, 0));
        let (px, py) = ((c.x + 8) * 2, (c.y + 8) * 2);
        assert_eq!((at_px(&big, px, py), at_px(&big, px + 1, py)), ([1, 2, 3], [200, 201, 202]), "2 art px per logical px");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_unit_rect_spans_its_cell_and_its_body() {
        let cell = Rect::new(128, 106, 32, 24);
        let body = Rect::new(139, 89, 17, 36);
        assert_eq!(unit_rect(cell, body), Rect::new(128, 89, 32, 41));
        assert_eq!(unit_rect(cell, Rect::new(140, 110, 4, 4)), cell, "a body inside the cell");
    }

    /// DIORAMA1B: a scratch root with body `units/painted/base`: a `w` x `h` frame (one `idle` row, 2 frames) whose
    /// figure fills columns w/4..3w/4 from row h/8 down to its `foot` row, painted.
    pub(crate) fn painted_root(name: &str, (w, h): (u32, u32), foot: u32) -> aegis_core::scratch::Scratch {
        let root = aegis_core::scratch::Scratch::vacant(&format!("stage_painted_{name}"));
        let sw = 2 * w;
        let mut px = vec![0u8; (sw * h * 4) as usize];
        for y in h / 8..=foot {
            for x in (w / 4..3 * w / 4).chain(w + w / 4..w + 3 * w / 4) {
                let i = ((y * sw + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[90, 70, 50, 255]);
            }
        }
        let dir = root.join("units/painted/base");
        std::fs::create_dir_all(&dir).unwrap();
        let file = std::fs::File::create(dir.join("sheet.png")).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), sw, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        let clips = json!({"frame": [w, h], "foot": foot, "painted": true, "clips": [{"id": "idle", "frames": 2, "ms": [200, 200]}]});
        std::fs::write(dir.join("clips.json"), clips.to_string()).unwrap();
        root
    }

    /// DIORAMA1B (D5): the drawn frame of a painted body filling the `n` x `n` square at `c`: (frame top to foot row
    /// in px, the figure's solid bounds placed on the board).
    fn painted_on(art: &mut Art, g: &Geom, n: i32, c: V2) -> (f32, Rect) {
        let clips = art.clips("painted/base").expect("the body loads").clone();
        let foot = clips.foot.expect("a foot row");
        let sprite = g.sprite_px(n, clips.frame, clips.foot);
        let tall = foot as f32 * sprite.0 as f32 / clips.frame.0.max(clips.frame.1) as f32;
        let r = g.sprite_rect_n(c, n, sprite, 0, 0);
        let b = art.body_clip("painted/base", "", "idle", 0, sprite.0).and_then(|f| f.opaque_bounds(aegis_ui::canvas::OUTLINE_ALPHA)).expect("a figure");
        (tall, Rect::new(r.x + b.x, r.y + b.y, b.w, b.h))
    }

    /// DIORAMA1B (D5, contract 8.1): a painted 112 x 96 frame (feet on row 93) stands `PAINTED_CELLS` cells
    /// (`CELL_W`) tall from its top to its foot row, centred on the cell's foot point with its feet on it; a 2 x 2
    /// unit's 224 x 192 frame stands twice that on its square's centre foot point.
    #[test]
    fn a_painted_frame_stands_1_15_cells_on_its_foot_point() {
        let g = stage();
        let c = V2::new(4, 3);
        let root = painted_root("one", (112, 96), 93);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        let (tall, body) = painted_on(&mut art, &g, 1, c);
        let want = PAINTED_CELLS * CELL_W as f32;
        assert!((tall - want).abs() <= 1.0, "about 1.15 cells tall: {tall} against {want}");
        let (fx, fy) = g.foot(c);
        assert!((body.y + body.h - 1 - fy).abs() <= 1, "the feet on the foot point: bottom row {} against {fy}", body.y + body.h - 1);
        assert!((body.x + body.w / 2 - fx).abs() <= 1, "centred on the foot point: {} against {fx}", body.x + body.w / 2);
        let root = painted_root("big", (224, 192), 189);
        let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
        let (tall, body) = painted_on(&mut art, &g, 2, c);
        assert!((tall - 2.0 * want).abs() <= 1.0, "a 2 x 2 unit about 2.3 cells tall: {tall} against {}", 2.0 * want);
        let (fx, fy) = g.foot_n(c, 2);
        assert!((body.y + body.h - 1 - fy).abs() <= 1, "on the square's foot point: bottom row {} against {fy}", body.y + body.h - 1);
        assert!((body.x + body.w / 2 - fx).abs() <= 1, "centred on the square: {} against {fx}", body.x + body.w / 2);
    }

    /// DIORAMA1B: a body without a `foot` row keeps the token square on the Stage (as before), and the Flat view keeps
    /// the token square for every body (a painted frame fits it like any sheet).
    #[test]
    fn a_sheet_without_a_foot_row_and_the_flat_view_keep_the_token_square() {
        let g = stage();
        let c = V2::new(2, 5);
        for n in [1, 2] {
            let sprite = g.sprite_px(n, (48, 48), None);
            assert_eq!(sprite, (g.token_px * n, 0));
            assert_eq!(g.sprite_rect_n(c, n, sprite, 3, -2), g.token_rect_n(c, n, 3, -2), "the old sheet's rect, n = {n}");
        }
        let flat = Geom::new(View::Flat, &Tuning::default());
        let sprite = flat.sprite_px(1, (112, 96), Some(93));
        assert_eq!(sprite, (flat.token_px, 0), "Flat: the token square");
        assert_eq!(flat.sprite_rect_n(c, 1, sprite, 0, 0), flat.token_rect(c, 0, 0));
    }
}
