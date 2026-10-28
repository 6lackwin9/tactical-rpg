//! The battle stage (STAGE1 option B, game-only, new in the engine port): the board tilted to 32 x 24 cells so the
//! upright side-view sprites belong on it. Its geometry (cells, foot and chest points, token rects, the point-to-cell
//! map), the terrain stage table (ground, sunken, standing, block), the ground under a standing prop, the lips of
//! sunken cells, the blocks, props, backdrop and front face, the draw passes, the ground arrows, the feet-sorted order
//! with the outlines drawn through what covers a unit, and the units' hit rects. The pieces of the board itself are
//! drawn here; `battle.rs` runs the passes and draws the units, markers and HUD. A view change only: rules, grid,
//! plans and every battle number stay as they are. The Flat view (the board seen from above, as before) stays a
//! setting until the user's review (STAGE1-C removes it).

use crate::tuning::Tuning;
use aegis_ui::art::Art;
use aegis_ui::canvas::{rgb, rgba, Image, Rect, Rgba};
use aegis_ui::depth::DepthList;
use aegis_ui::painter::Painter;
use emberwake_battle::grid::{data_zone, Grid, V2, GRID_H, GRID_W};
use indexmap::IndexMap;
use emberwake_content::art::{battle_backdrop, battle_prop_dirs, battle_stage_dirs, zone_backdrop, BATTLE_SETS};
use emberwake_data::ZoneDef;
use serde_json::Value;

/// The grid panel's interior (every stage pass is clipped to it).
pub const INTERIOR: Rect = Rect::new(128, 30, 384, 288);
/// The zone backdrop over the board's far edge.
pub const BACKDROP: Rect = Rect::new(128, 30, 384, 52);
pub const BOARD_X: i32 = 128;
pub const BOARD_Y: i32 = 82;
pub const CELL_W: i32 = 32;
pub const CELL_H: i32 = 24;
/// The front face under the near edge (`FACE_H` tall) and the drop shadow under it.
pub const FACE_H: i32 = 12;
pub const FRONT_FACE: Rect = Rect::new(128, BOARD_Y + GRID_H * CELL_H, 384, FACE_H);
pub const DROP_SHADOW: Rect = Rect::new(128, BOARD_Y + GRID_H * CELL_H + FACE_H, 384, 8);
/// How far a block's cap is lifted above its cell.
pub const BLOCK_LIFT: i32 = 12;
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

/// How a terrain stands on the stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Ground,
    /// Lower than the ground (its depth in px): its lips show the step.
    Sunken,
    /// A standing prop on the ground of its neighbours.
    Standing,
    /// A block lifted `BLOCK_LIFT` with a cap and a face.
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageRow {
    pub kind: Kind,
    /// Px down (sunken), else 0.
    pub depth: i32,
    /// The prop it shows: a ground prop (`stones` on rubble) or the standing prop (`tree`).
    pub prop: Option<&'static str>,
}

/// A terrain's stage row (3.1; an unknown id is ground, depth 0).
pub fn stage_of(terrain: &str) -> StageRow {
    let (kind, depth, prop) = match terrain {
        "dirt" | "toxic_bog" => (Kind::Sunken, 2, None),
        "mud" => (Kind::Sunken, 1, None),
        "shallow_water" => (Kind::Sunken, 3, None),
        "deep_water" | "lava" => (Kind::Sunken, 5, None),
        "rubble" => (Kind::Ground, 0, Some("stones")),
        "tree" => (Kind::Standing, 0, Some("tree")),
        "wall" => (Kind::Block, 0, None),
        _ => (Kind::Ground, 0, None),
    };
    StageRow { kind, depth, prop }
}

/// The terrain on `c` (None off the grid).
fn at(grid: &Grid, c: V2) -> Option<&'static str> {
    grid.is_valid(c).then(|| grid.terrain_id(c))
}

fn is_ground(t: &str) -> bool {
    matches!(stage_of(t).kind, Kind::Ground | Kind::Sunken)
}

fn is_block(grid: &Grid, c: V2) -> bool {
    at(grid, c).is_some_and(|t| stage_of(t).kind == Kind::Block)
}

/// The four neighbours in tie order: north, east, south, west.
fn neighbours(c: V2) -> [V2; 4] {
    [V2::new(c.x, c.y - 1), V2::new(c.x + 1, c.y), V2::new(c.x, c.y + 1), V2::new(c.x - 1, c.y)]
}

/// The most common of `ids` (ties: the first seen); None when empty.
fn most_common(ids: &[&'static str]) -> Option<&'static str> {
    let mut best: Option<(&'static str, usize)> = None;
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
pub fn ground_under(grid: &Grid, c: V2) -> &'static str {
    let ids: Vec<&'static str> = neighbours(c).iter().filter_map(|n| at(grid, *n)).filter(|t| is_ground(t)).collect();
    most_common(&ids).unwrap_or("grass")
}

/// The depth a lip compares with (block and standing count as 0).
fn depth_of(t: &str) -> i32 {
    stage_of(t).depth
}

/// 3.2.2: a sunken cell's lips: the band over its top (px; 0 none) when its north neighbour is shallower, and whether
/// its bottom row is shaded (its south neighbour is shallower).
pub fn lips(grid: &Grid, c: V2) -> (i32, bool) {
    let Some(t) = at(grid, c) else { return (0, false) };
    let row = stage_of(t);
    if row.kind != Kind::Sunken {
        return (0, false);
    }
    let d = row.depth;
    let band = at(grid, V2::new(c.x, c.y - 1)).map(depth_of).filter(|n| *n < d).map_or(0, |n| d - n + 1);
    let bottom = at(grid, V2::new(c.x, c.y + 1)).map(depth_of).is_some_and(|n| n < d);
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
pub fn far_ground(grid: &Grid) -> &'static str {
    let ids: Vec<&'static str> = (0..GRID_W).filter_map(|x| at(grid, V2::new(x, 0))).filter(|t| is_ground(t)).collect();
    most_common(&ids).unwrap_or("grass")
}

/// The ground a cell shows (a standing cell the ground under its prop).
fn shown_ground(grid: &Grid, c: V2) -> &'static str {
    let t = grid.terrain_id(c);
    if stage_of(t).kind == Kind::Standing {
        ground_under(grid, c)
    } else {
        t
    }
}

/// A terrain's flat colour (the fallback when no tile loads; the backdrop and face colours).
pub fn terrain_color(id: &str) -> Rgba {
    match id {
        "wall" => rgb(46, 46, 54),
        "tree" => rgb(30, 70, 40),
        "deep_water" => rgb(30, 54, 110),
        "shallow_water" => rgb(56, 92, 140),
        "lava" => rgb(150, 50, 20),
        "grass" => rgb(60, 104, 56),
        "dirt" => rgb(104, 84, 56),
        "sand" | "snow" => rgb(170, 160, 120),
        "stone" | "rubble" => rgb(92, 92, 100),
        _ => rgb(80, 82, 92),
    }
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

/// 3.2.1: terrain `id`'s tile on cell (x, y), varied by position.
pub fn tile_art(art: &mut Art, id: &str, x: i32, y: i32) -> TileArt {
    if let Some(rel) = art.pick(&dirs_ref(&battle_stage_dirs(id)), None, x, y) {
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
pub fn face_art(art: &mut Art, x: i32, y: i32) -> Option<String> {
    art.pick(&dirs_ref(&battle_stage_dirs("wall_face")), None, x, y)
}

/// 3.4: zone `zone`'s backdrop file (None: no zone, or no file: the gradient).
pub fn backdrop_art(art: &mut Art, zone: &str) -> Option<String> {
    let rel = battle_backdrop(zone);
    (!zone.is_empty() && art.has(&rel)).then_some(rel)
}

/// The standing prop a cell shows with its art (None: not a standing cell, or no art: the flat tile instead).
fn standing_art(art: &mut Art, grid: &Grid, c: V2) -> Option<(String, (i32, i32))> {
    let row = stage_of(grid.terrain_id(c));
    if row.kind != Kind::Standing {
        return None;
    }
    prop_art(art, row.prop?, c.x, c.y)
}

fn stage_cell(c: V2) -> Rect {
    Rect::new(BOARD_X + c.x * CELL_W, BOARD_Y + c.y * CELL_H, CELL_W, CELL_H)
}

fn draw_tile(p: &mut Painter, art: &mut Art, id: &str, c: V2, r: Rect) {
    match tile_art(art, id, c.x, c.y) {
        TileArt::Stage(rel) => match art.image(&rel, CELL_W, CELL_H) {
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
pub fn backdrop_source<'a>(art: &mut Art, grid: &Grid, zone: &str, zones: &'a IndexMap<String, ZoneDef>) -> (Option<String>, &'a str) {
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
    let bottom = lerp(NIGHT, terrain_color(ground), 0.4);
    for k in 0..BACKDROP.h {
        let t = k as f32 / (BACKDROP.h - 1) as f32;
        p.fill(Rect::new(BACKDROP.x, BACKDROP.y + k, BACKDROP.w, 1), lerp(NIGHT, bottom, t));
    }
}

/// Pass 2: every cell's ground tile (a standing cell with prop art the ground under it, without art the flat tile),
/// the lips of sunken cells, the ground props and the grid lines at `grid_alpha`.
pub fn draw_ground(p: &mut Painter, art: &mut Art, grid: &Grid, grid_alpha: u8) {
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            let r = stage_cell(c);
            let t = grid.terrain_id(c);
            let shown = if standing_art(art, grid, c).is_some() { ground_under(grid, c) } else { t };
            draw_tile(p, art, shown, c, r);
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
            let row = stage_of(t);
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

/// The front face's body under a column (3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceBody {
    Wall,
    Water,
    Lava,
    Earth,
}

/// 3.4: column `x`'s front face: the ground its top row lightens (row 8's shown ground) and its body.
pub fn face_column(grid: &Grid, x: i32) -> (&'static str, FaceBody) {
    let c = V2::new(x, GRID_H - 1);
    let t = grid.terrain_id(c);
    let body = match t {
        "wall" => FaceBody::Wall,
        "shallow_water" | "deep_water" => FaceBody::Water,
        "lava" => FaceBody::Lava,
        _ => FaceBody::Earth,
    };
    (shown_ground(grid, c), body)
}

/// Pass 6: the front face (per column: the top row the ground's colour lightened 25 %, the body wall art, water,
/// lava or earth with flecks), then the drop shadow.
pub fn draw_face(p: &mut Painter, art: &mut Art, grid: &Grid) {
    for x in 0..GRID_W {
        let r = Rect::new(FRONT_FACE.x + x * CELL_W, FRONT_FACE.y, CELL_W, FACE_H);
        let (top, body) = face_column(grid, x);
        let flecks = |p: &mut Painter, col: Rgba| {
            let mut fx = 0;
            while fx < CELL_W {
                let bx = x * CELL_W + fx;
                p.fill(Rect::new(r.x + fx, r.y + 2 + (bx * 7).rem_euclid(6), 2, 1), col);
                fx += 7;
            }
        };
        match body {
            FaceBody::Wall => match face_art(art, x, GRID_H).and_then(|rel| art.image(&rel, CELL_W, FACE_H)) {
                Some(img) => p.image(img, r),
                None => p.fill(r, scaled(terrain_color("wall"), 0.75)),
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
        p.fill(Rect::new(r.x, r.y, r.w, 1), lighten(terrain_color(top), 0.25));
    }
    for k in 0..DROP_SHADOW.h {
        p.fill(Rect::new(DROP_SHADOW.x, DROP_SHADOW.y + k, DROP_SHADOW.w, 1), rgba(0, 0, 0, (120 - 15 * k).max(0) as u8));
    }
}

/// The feet pass: block `c` (3.3): its cap (the stage tile lifted `BLOCK_LIFT`), its face (the wall face art, else the
/// cap's top rows at 75 %) with its top line and the shade on the cell south, the cap's highlight and side lines.
pub fn draw_block(p: &mut Painter, art: &mut Art, grid: &Grid, c: V2) {
    let r = stage_cell(c);
    let cap = Rect::new(r.x, r.y - BLOCK_LIFT, CELL_W, CELL_H);
    let t = grid.terrain_id(c);
    draw_tile(p, art, t, c, cap);
    if block_face(grid, c) {
        let face = Rect::new(r.x, r.y + CELL_H - FACE_H, CELL_W, FACE_H);
        let drawn = match face_art(art, c.x, c.y).and_then(|rel| art.image(&rel, CELL_W, FACE_H)) {
            Some(img) => {
                p.image(img, face);
                true
            }
            None => false,
        };
        if !drawn {
            match darkened_cap(art, t, c) {
                Some(img) => p.image(&img, face),
                None => p.fill(face, scaled(terrain_color(t), 0.75)),
            }
        }
        p.fill(Rect::new(face.x, face.y, face.w, 1), rgb(24, 24, 30));
        if c.y + 1 < GRID_H {
            p.fill(Rect::new(r.x, r.y + r.h, r.w, 3), rgba(0, 0, 0, 70));
        }
    }
    if !is_block(grid, V2::new(c.x, c.y - 1)) {
        p.fill(Rect::new(cap.x, cap.y, cap.w, 1), rgb(170, 172, 186));
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
fn darkened_cap(art: &mut Art, t: &str, c: V2) -> Option<Image> {
    let img = match tile_art(art, t, c.x, c.y) {
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
            match stage_of(grid.terrain_id(c)).kind {
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

/// The order `battle.rs` runs the Stage's passes in.
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
mod tests {
    use super::*;
    use aegis_core::assets::Layers;
    use emberwake_battle::grid::TerrainTable;
    use serde_json::json;
    use std::path::{Path, PathBuf};

    fn zone(id: &str) -> Grid {
        let mut g = Grid::new(false, Vec::new(), TerrainTable { rows: Vec::new() }, 0);
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

    #[test]
    fn the_stage_table_sorts_every_terrain() {
        for id in emberwake_battle::grid::TERRAIN_IDS {
            let row = stage_of(id);
            assert_eq!(row.depth > 0, row.kind == Kind::Sunken, "{id}: only sunken cells have a depth");
        }
        assert_eq!(stage_of("lava"), StageRow { kind: Kind::Sunken, depth: 5, prop: None });
        assert_eq!(stage_of("rubble").prop, Some("stones"));
        assert_eq!(stage_of("tree").kind, Kind::Standing);
        assert_eq!(stage_of("wall").kind, Kind::Block);
        assert_eq!(stage_of("no_such_terrain"), StageRow { kind: Kind::Ground, depth: 0, prop: None });
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
        assert_eq!(lips(&forest, V2::new(0, 4)).0, stage_of("dirt").depth + 1, "a 3 px band");
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

    fn scratch(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("emberwake_stage_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
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
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        assert_eq!(tile_art(&mut art, "grass", 2, 3), TileArt::Common("tilesets/common/grass/v0.png".into()));
        assert_eq!(tile_art(&mut art, "lava", 2, 3), TileArt::Colour, "neither: the colour");
        assert_eq!(face_art(&mut art, 2, 3), None, "the face falls back to the darkened cap");
        assert!(darkened_cap(&mut art, "wall", V2::new(2, 3)).is_some_and(|img| img.h == CELL_H / 2 && img.px[0] == 96), "the cap's top rows at 75 %");
        assert_eq!(prop_art(&mut art, "tree", 2, 3), None);
        let forest = zone("forest_clearing");
        assert!(board_items(&mut art, &forest).is_empty(), "no tree art: nothing stands");
        assert_eq!(tile_art(&mut art, "tree", 0, 0), TileArt::Common("tilesets/common/tree/v0.png".into()), "the flat tree tile");
        // with the stage art
        let full = scratch("full");
        png(&full, "tilesets/common_stage/grass/v0.png", 128, 96);
        png(&full, "tilesets/common_stage/wall_face/v0.png", 128, 48);
        png(&full, "props/common/tree/v0.png", 40, 52);
        let mut art = Art::layered(Layers::of(vec![full.clone(), root.clone()]));
        assert_eq!(tile_art(&mut art, "grass", 2, 3), TileArt::Stage("tilesets/common_stage/grass/v0.png".into()));
        assert_eq!(face_art(&mut art, 2, 3), Some("tilesets/common_stage/wall_face/v0.png".into()));
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

    #[test]
    fn a_unit_rect_spans_its_cell_and_its_body() {
        let cell = Rect::new(128, 106, 32, 24);
        let body = Rect::new(139, 89, 17, 36);
        assert_eq!(unit_rect(cell, body), Rect::new(128, 89, 32, 41));
        assert_eq!(unit_rect(cell, Rect::new(140, 110, 4, 4)), cell, "a body inside the cell");
    }
}
