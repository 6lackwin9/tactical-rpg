//! DIORAMA5 R1 (PR7-77): the battle board's ground through the region map's ground composer (`aegis_ui::ground`,
//! REGIONGROUND1-E1/E2): each ground terrain on the board is one layer whose texture is its `sheet.png` (a whole number
//! of 64 x 48 cells, sampled by art pixel so no cell edge shows), the layers meeting at the composer's worn edges, with a
//! board-wide tone and quiet patches. A board composes only when one of its grounds has a sheet; a terrain without one
//! gives its first tile as its texture. Over the composite, accent decals (`<terrain>/decal/v*`) on cells an integer hash
//! picks, more often beside a block, a cover prop or a ground prop. Presentation only: no rule reads any of it.
//!
//! The job is built when the board, the sets, the zone look or the art change (one key per thread, compared each frame by
//! a cheap fingerprint), never per frame; the composer's own cache keeps the picture.

use super::{at, ground_under, is_block, put, stage_cell, stage_image, stage_of, tile_shown, Kind, ZoneLook, BOARD, CELL_H, CELL_W, SET_ART_PX};
use aegis_ui::art::{cell_hash, Art};
use aegis_ui::canvas::{Image, Rgba};
use aegis_ui::ground::{self as composer, GroundJob, GroundKey, GroundLayer, PlacedPatch};
use aegis_ui::painter::Painter;
use rpgkit_battle::grid::{Grid, V2, GRID_H, GRID_W};
use rpgkit_content::art::battle_stage_dirs;
use std::cell::RefCell;
use std::rc::Rc;

/// The composer's art cell on the stage: the stage cell at the set's art px (64 x 48).
pub const ART_CELL: (i32, i32) = (CELL_W * SET_ART_PX, CELL_H * SET_ART_PX);
/// The decal share of a ground cell (`stage.fx.json` `decal_density` overrides it) and beside a block or a prop.
pub const DECAL_DENSITY: f32 = 0.10;
pub const DECAL_NEAR: f32 = 0.35;
/// The layer numbers of a ground and of a sunken terrain on the board (the region map's look, quieter).
const WEAR: f32 = 0.12;
const TONE: f32 = 0.06;
/// R6: the board's patches are a few coherent blobs 2 to 4 cells across (placed patches the board's seed picks), not
/// the composer's random per-cell patches: at most this many, this dark.
pub const BLOBS_MAX: usize = 3;
pub const BLOB_STRENGTH: f32 = 0.14;
/// The candidate rects tried for blobs.
const BLOB_TRIES: u64 = 48;
/// The hash stream of the decal pick's "whether" (its "which" is `Art::pick_hashed`'s stream 0).
const DECAL_WHETHER: u64 = 1;

/// The folders terrain `id`'s art is read from in this zone, first first: its override folder, the zone set's, the world
/// set's and `common_stage`. R5: a zone set without its own `rubble` reads its `floor` for rubble (before the world's).
pub fn terrain_dirs(art: &mut Art, id: &str, set: &str, look: &ZoneLook) -> Vec<String> {
    let mut dirs = Vec::new();
    if let Some((f, _)) = look.terrain(id) {
        dirs.push(f.to_string());
    }
    if let Some(z) = look.zone_set(set) {
        dirs.push(format!("tilesets/{z}/{id}"));
        if id == super::RUBBLE && !art.has(&format!("tilesets/{z}/{id}/v0.png")) {
            dirs.push(format!("tilesets/{z}/floor"));
        }
    }
    dirs.extend(battle_stage_dirs(id, set));
    dirs
}

/// R1: terrain `id`'s ground sheet in this zone: `<dir>/sheet.png` of the first of `terrain_dirs` that has one.
pub fn sheet_of(art: &mut Art, id: &str, set: &str, look: &ZoneLook) -> Option<String> {
    terrain_dirs(art, id, set, look).into_iter().map(|d| format!("{d}/sheet.png")).find(|rel| art.has(rel))
}

/// R1: the decal folder of terrain `id` (`<dir>/decal` of the first of `terrain_dirs` holding a `v0`).
fn decal_dir(art: &mut Art, id: &str, set: &str, look: &ZoneLook) -> Option<String> {
    terrain_dirs(art, id, set, look).into_iter().map(|d| format!("{d}/decal")).find(|d| art.has(&format!("{d}/v0.png")))
}

/// What a cell of the board is to the composer: the ground it shows (a block shows the ground under it) and whether a
/// patch may reach it.
fn board_cells<'g>(art: &mut Art, grid: &'g Grid) -> Vec<(&'g str, bool)> {
    let mut out = Vec::with_capacity((GRID_W * GRID_H) as usize);
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            if is_block(grid, c) {
                out.push((ground_under(grid, c), true));
            } else {
                let t = tile_shown(art, grid, c);
                out.push((t, stage_of(grid, grid.terrain_id(c)).prop.is_some()));
            }
        }
    }
    out
}

/// One board's job, owning what the composer borrows.
struct Picture {
    cells: Vec<u8>,
    blocked: Vec<bool>,
    /// Per layer: order, texture, flat colour, sunken.
    layers: Vec<(i32, Option<Image>, Rgba, bool)>,
    placed: Vec<PlacedPatch>,
    seed: u64,
}

impl Picture {
    fn ground_layers(&self) -> Vec<GroundLayer<'_>> {
        self.layers
            .iter()
            .map(|(order, tex, colour, sunken)| {
                let mut l = GroundLayer::flat(*order, *colour);
                l.tex = tex.as_ref();
                l.tone = TONE;
                if *sunken {
                    // the stage draws a sunken cell's lips on its cell edges, so its edge keeps to them
                    l.clamp_water = true;
                } else {
                    l.wear = WEAR;
                    l.strays = true;
                    l.contact = true;
                }
                l
            })
            .collect()
    }

    fn job<'a>(&'a self, layers: &'a [GroundLayer<'a>]) -> GroundJob<'a> {
        GroundJob { seed: self.seed, cols: GRID_W, rows: GRID_H, cell: ART_CELL, cells: &self.cells, blocked: &self.blocked, layers, placed: &self.placed }
    }
}

/// `img` multiplied by `tint` per channel.
fn tinted_image(img: &Image, tint: Option<Rgba>) -> Image {
    let mut out = img.clone();
    if let Some(t) = tint {
        for px in out.px.chunks_exact_mut(4) {
            for i in 0..3 {
                px[i] = ((px[i] as u32 * t[i] as u32 + 127) / 255) as u8;
            }
        }
    }
    out
}

/// R1: the job of `grid` in this zone, or None when no ground on it has a sheet (the board draws its tiles as before).
fn picture(art: &mut Art, grid: &Grid, set: &str, look: &ZoneLook, seed: u64) -> Option<Picture> {
    let cells = board_cells(art, grid);
    let mut ids: Vec<&str> = Vec::new();
    for (t, _) in &cells {
        if !ids.contains(t) {
            ids.push(t);
        }
    }
    if !ids.iter().any(|t| sheet_of(art, t, set, look).is_some()) {
        return None;
    }
    // the base is the commonest ground, lowest; the others paint over it, most cells first; sunken terrains under all
    let count = |t: &str| cells.iter().filter(|(c, _)| *c == t).count();
    let mut ranked = ids.clone();
    ranked.sort_by_key(|t| (std::cmp::Reverse(stage_of(grid, t).depth), std::cmp::Reverse(count(t))));
    let mut layers = Vec::with_capacity(ids.len());
    for t in &ids {
        let order = ranked.iter().position(|r| r == t).unwrap_or(0) as i32;
        let tint = look.tint_of(t);
        let tex = match sheet_of(art, t, set, look) {
            Some(rel) => art.size_of(&rel).and_then(|(w, h)| art.image(&rel, w, h).cloned()),
            None => {
                let dirs = terrain_dirs(art, t, set, look);
                art.pick(&dirs.iter().map(String::as_str).collect::<Vec<_>>(), Some(0), 0, 0).and_then(|rel| art.image(&rel, ART_CELL.0, ART_CELL.1).cloned())
            }
        };
        let colour = super::tinted(super::terrain_color(grid, t), tint);
        layers.push((order, tex.map(|i| tinted_image(&i, tint)), colour, stage_of(grid, t).kind == Kind::Sunken));
    }
    let index = |t: &str| ids.iter().position(|i| *i == t).unwrap_or(0) as u8;
    let (cells, blocked): (Vec<u8>, Vec<bool>) = cells.iter().map(|(t, b)| (index(t), *b)).unzip();
    let sunken: Vec<bool> = layers.iter().map(|l| l.3).collect();
    let placed = blobs(&cells, &blocked, &sunken, seed);
    Some(Picture { cells, blocked, layers, placed, seed })
}

/// R6: up to `BLOBS_MAX` placed patches 2 to 4 cells wide and 2 to 3 high, from candidates the board's seed picks: each
/// on one non-sunken ground with a ring of one cell (cut at the board's edge) of that ground, free of blocked cells and
/// apart from the others (the composer's own rule for a placed patch).
pub fn blobs(cells: &[u8], blocked: &[bool], sunken: &[bool], seed: u64) -> Vec<PlacedPatch> {
    let at = |x: i32, y: i32| (y * GRID_W + x) as usize;
    let mut out: Vec<PlacedPatch> = Vec::new();
    for i in 0..BLOB_TRIES {
        if out.len() >= BLOBS_MAX {
            break;
        }
        let h = cell_hash(i as i32, (seed >> 32) as i32 ^ seed as i32, 7);
        let (w, hh) = (2 + (h % 3) as i32, 2 + ((h >> 8) % 2) as i32);
        let (x, y) = (((h >> 16) % (GRID_W - w + 1) as u64) as i32, ((h >> 32) % (GRID_H - hh + 1) as u64) as i32);
        let ground = cells[at(x, y)];
        if sunken.get(ground as usize).copied().unwrap_or(true) {
            continue;
        }
        let (x0, y0, x1, y1) = ((x - 1).max(0), (y - 1).max(0), (x + w).min(GRID_W - 1), (y + hh).min(GRID_H - 1));
        let clean = (y0..=y1).all(|cy| (x0..=x1).all(|cx| cells[at(cx, cy)] == ground && !blocked[at(cx, cy)]));
        let apart = out.iter().all(|o| x + w + 1 <= o.x || o.x + o.w + 1 <= x || y + hh + 1 <= o.y || o.y + o.h + 1 <= y);
        if clean && apart {
            out.push(PlacedPatch { x, y, w, h: hh, strength: BLOB_STRENGTH });
        }
    }
    out
}

/// What the kept job was made from (compared each frame).
#[derive(PartialEq)]
struct Seen {
    terrain: Vec<String>,
    set: String,
    look: ZoneLook,
    art: u64,
}

thread_local! {
    static KEPT: RefCell<Option<(Seen, Option<Rc<(Picture, GroundKey)>>)>> = const { RefCell::new(None) };
}

/// The board's seed: its terrain and sets (so a board keeps its look between frames and fights).
fn seed_of(terrain: &[String], set: &str, look: &ZoneLook) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in terrain.iter().flat_map(|t| t.bytes().chain([0])).chain(set.bytes()).chain(look.set.as_deref().unwrap_or("").bytes()) {
        h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// R1: draws the composed ground over the board and returns true, or false when no ground on it has a sheet.
pub fn draw(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str, look: &ZoneLook) -> bool {
    let terrain: Vec<String> = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).map(|c| grid.terrain_id(c).to_string()).collect();
    let seen = Seen { terrain, set: set.to_string(), look: look.clone(), art: art.generation() };
    let kept = KEPT.with(|k| k.borrow().as_ref().filter(|(s, _)| *s == seen).map(|(_, j)| j.clone()));
    let job = match kept {
        Some(j) => j,
        None => {
            let seed = seed_of(&seen.terrain, set, look);
            let j = picture(art, grid, set, look, seed).map(|pic| {
                let layers = pic.ground_layers();
                let key = GroundKey::of("battle_stage", &pic.job(&layers));
                drop(layers);
                Rc::new((pic, key))
            });
            KEPT.with(|k| *k.borrow_mut() = Some((seen, j.clone())));
            j
        }
    };
    let Some(job) = job else { return false };
    let (pic, key) = &*job;
    composer::draw(p, key.clone(), BOARD, || composer::compose(&pic.job(&pic.ground_layers())));
    true
}

/// ENGINE-52 (COMPOSE-PAR1): the composed ground `draw` would show for `grid` in this zone, uncached (None: the board
/// draws its tiles); for tests and timings.
pub fn compose_board(art: &mut Art, grid: &Grid, set: &str, look: &ZoneLook) -> Option<Image> {
    let terrain: Vec<String> = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).map(|c| grid.terrain_id(c).to_string()).collect();
    let pic = picture(art, grid, set, look, seed_of(&terrain, set, look))?;
    let layers = pic.ground_layers();
    Some(composer::compose(&pic.job(&layers)))
}

/// R1: whether a cell beside `c` (eight around) holds a block, a cover prop or a prop, where wear gathers.
fn near_things(grid: &Grid, c: V2) -> bool {
    (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))).filter(|d| *d != (0, 0)).any(|(dx, dy)| {
        let n = V2::new(c.x + dx, c.y + dy);
        at(grid, n).is_some_and(|t| stage_of(grid, t).kind == Kind::Block || stage_of(grid, t).prop.is_some())
    })
}

/// R1: whether cell (x, y) takes a decal at `density` (near things: `DECAL_NEAR`), by the integer hash of the cell.
pub fn decal_on(x: i32, y: i32, density: f32, near: bool) -> bool {
    let share = if near { DECAL_NEAR.max(density) } else { density };
    (cell_hash(x, y, DECAL_WHETHER) % 10_000) < (share.clamp(0.0, 1.0) * 10_000.0) as u64
}

/// R1: the accent decals over the ground cells (not blocks, sunken cells or standing props): one of the terrain's
/// `decal/v*` on the cells `decal_on` picks, tinted as the terrain.
pub fn draw_decals(p: &mut Painter, art: &mut Art, grid: &Grid, set: &str, look: &ZoneLook) {
    let density = look.decal_density.unwrap_or(DECAL_DENSITY);
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            let t = grid.terrain_id(c);
            if stage_of(grid, t).kind != Kind::Ground || !decal_on(x, y, density, near_things(grid, c)) {
                continue;
            }
            let Some(dir) = decal_dir(art, t, set, look) else { continue };
            let Some(rel) = art.pick_hashed(&[&dir], x, y) else { continue };
            let tint = look.tint_of(t);
            if let Some(img) = stage_image(art, &rel, (set, look), CELL_W, CELL_H) {
                put(p, img, stage_cell(c), tint);
            }
        }
    }
}
