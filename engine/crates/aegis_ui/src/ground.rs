//! The ground composer (REGIONGROUND1-E1, `docs/specs/REGIONGROUND1_ENGINE.md`): a pure function from a cell grid to
//! one opaque image, then a cache and a blit. A layer is a texture and numbers: this module names no ground and no game,
//! and no rule, save, walk or click reads what it makes.
//!
//! Each layer is sampled by art pixel (never by cell, so no cell edge exists in it) and varied by a map-wide tone with
//! no period. Layers meet at a generated edge: a blurred cell mask pushed by shared noise and thresholded, with a worn
//! band, stray bits and small bites around it. Every test is a threshold on art pixels and colours never blend between
//! grounds, so an edge stays hard at art pixels.
//!
//! `compose` holds no RNG state: a noise value is a hash of the seed, the field and its lattice point. Float steps are
//! the IEEE operations only (no libm); the blurs are integers. So the same job gives the same bytes on every run and
//! platform, whatever the view's scale, the camera, the frame or the party.
//!
//! Over that composite, quiet and never in a texture, so they never repeat with it (REGIONGROUND1-E2): random dark patches,
//! placed patches and one-off marks. A random patch or mark that would reach a road, another ground or a blocked cell is
//! dropped whole, never clipped; a random patch has a size cap and fades as it grows, so large ones exist only where a map
//! places them.

use crate::canvas::{Canvas, Image, Rect, Rgba};
use crate::painter::Painter;
use anyhow::{anyhow, bail, Context as _, Result};
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::io::Write as _;
use std::path::Path;
use std::rc::Rc;

/// Art px per cell: a ground art px is then 1 logical px, always a whole number of physical px at a whole window scale,
/// and never finer than the units on it. The cap's limit is 32 (the sprites' own density).
pub const CELL_PX: i32 = 16;
/// A texture covers this many cells each way, so it is 64 x 64 square and 64 x 48 tilted.
pub const TEX_CELLS: i32 = 4;
/// The tilted form's art px per cell.
pub const TILTED_CELL: (i32, i32) = (16, 12);
/// The most art px per cell `compose` takes (the sprites' own density): past it a job composes nothing.
pub const CELL_MAX: i32 = 32;
/// The most art px a job composes (64 MB of RGBA; the largest shipped map is 368,640): past it a job composes nothing.
pub const ART_PX_MAX: i64 = 16_000_000;
/// The most layers a job holds (a rank is a byte): past it a job composes nothing.
pub const LAYERS_MAX: usize = 255;
/// A physical copy past this many bytes is not kept: the art image draws to what shows instead (`blit_scaled`).
pub const PHYSICAL_MAX: usize = 64_000_000;
/// How many art images are kept, the oldest dropped first.
pub const ART_MAX: usize = 4;
/// The largest random patch, in art px at `CELL_PX` x `CELL_PX` (about one cell); `patch_cap` scales it with the cell.
pub const PATCH_CAP: i32 = 260;
/// The most a `patches` or a placed patch's `strength` darkens; past it (or NaN) the layer or patch is ignored.
pub const PATCHES_MAX: f32 = 0.3;
/// The most tries per 100 cells; past it (or below 0) a layer has no marks.
pub const MARKS_PER_100_MAX: i32 = 10;
/// A mark's largest side in art px (2 x 2 cells); a larger mark is never placed.
pub const MARK_PX_MAX: i32 = 32;
/// The most marks a layer lists and the most placed patches a job holds: past either a job composes nothing. (The kit
/// allows 4 and 16.)
pub const MARKS_MAX: usize = 64;
pub const PLACED_MAX: usize = 64;
/// `o_g` past this means a patch or mark would reach another ground: about 9 art px of clearance.
const CLEARANCE: f32 = 0.05;
/// A mark's blend at full alpha, so it stays quiet.
const MARK_ALPHA: f32 = 0.6;

/// A one-off mark's art at the job's density, at most `MARK_PX_MAX` a side.
#[derive(Debug, Clone, Copy)]
pub struct GroundMark<'a> {
    pub img: &'a Image,
}

/// One ground of a job: a texture and numbers.
#[derive(Debug, Clone)]
pub struct GroundLayer<'a> {
    /// Paint order, low first; unique in a job.
    pub order: i32,
    /// Texture at `TEX_CELLS` x cell px; None: flat `colour`.
    pub tex: Option<&'a Image>,
    pub colour: Rgba,
    /// 0.0-0.3: darkening of the worn band on the ground below.
    pub wear: f32,
    /// Bits of this ground scattered just past its edge.
    pub strays: bool,
    /// 0.0-0.2: amplitude of the map-wide tone.
    pub tone: f32,
    /// 0.0-`PATCHES_MAX`: darkening of a random patch's core; 0 none.
    pub patches: f32,
    /// A rim on its edge.
    pub sunken: bool,
    /// Random one-off marks.
    pub marks: Vec<GroundMark<'a>>,
    /// Tries per 100 cells of this ground, 0-`MARKS_PER_100_MAX`.
    pub marks_per_100: i32,
}

impl<'a> GroundLayer<'a> {
    /// A flat layer with every effect off.
    pub fn flat(order: i32, colour: Rgba) -> Self {
        Self { order, tex: None, colour, wear: 0.0, strays: false, tone: 0.0, patches: 0.0, sunken: false, marks: Vec::new(), marks_per_100: 0 }
    }
}

/// A large patch a map asks for, in cells. Its rect and a ring of one cell (cut off at the map's edge) must be one ground
/// and free of blocked cells, its strength 0-`PATCHES_MAX`: `compose` ignores one that is not (the kit's G8 refuses it
/// first).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedPatch {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub strength: f32,
}

/// All `compose` reads.
#[derive(Debug, Clone)]
pub struct GroundJob<'a> {
    pub seed: u64,
    pub cols: i32,
    pub rows: i32,
    /// Art px per cell: (`CELL_PX`, `CELL_PX`), or `TILTED_CELL`.
    pub cell: (i32, i32),
    /// Layer index per cell, row by row.
    pub cells: &'a [u8],
    /// Cells a patch or mark may not reach; one per cell (fewer composes nothing).
    pub blocked: &'a [bool],
    /// A texture's offset comes from its layer's place in this list, so keep the game's order: reordering the list moves
    /// the textures (paint order comes from `order` alone).
    pub layers: &'a [GroundLayer<'a>],
    pub placed: &'a [PlacedPatch],
}

/// A composed image's identity: the map, a hash of all the job holds, and the cell size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroundKey {
    pub map: String,
    pub hash: u64,
    pub cell: (i32, i32),
}

impl GroundKey {
    /// `map`'s key for this job. The hash covers the seed, the size, the cells, every layer's numbers and every pixel of
    /// each texture and mark: real content, so a retouched texture is a new key and a copy of a texture at another
    /// address is the same key. That is about 16 KB a texture: call this when the job changes and keep the key.
    pub fn of(map: &str, job: &GroundJob) -> Self {
        let mut h = Fnv::new();
        h.u64(job.seed);
        for v in [job.cols, job.rows, job.cell.0, job.cell.1] {
            h.i32(v);
        }
        h.bytes(job.cells);
        for b in job.blocked {
            h.bytes(&[*b as u8]);
        }
        for l in job.layers {
            h.i32(l.order);
            h.picture(l.tex);
            h.bytes(&l.colour);
            for v in [l.wear, l.tone, l.patches] {
                h.u64(v.to_bits() as u64);
            }
            h.bytes(&[l.strays as u8, l.sunken as u8]);
            h.i32(l.marks_per_100);
            for m in &l.marks {
                h.picture(Some(m.img));
            }
        }
        for p in job.placed {
            for v in [p.x, p.y, p.w, p.h] {
                h.i32(v);
            }
            h.u64(p.strength.to_bits() as u64);
        }
        Self { map: map.to_string(), hash: h.0, cell: job.cell }
    }
}

struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
    fn bytes(&mut self, b: &[u8]) {
        for v in b {
            self.0 = (self.0 ^ *v as u64).wrapping_mul(0x100000001b3);
        }
    }
    fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.bytes(&v.to_le_bytes());
    }
    fn picture(&mut self, img: Option<&Image>) {
        match img {
            Some(i) => {
                self.i32(i.w);
                self.i32(i.h);
                self.bytes(&i.px);
            }
            None => self.i32(-1),
        }
    }
}

/// The hash every noise value and texture offset comes from: a pure function of its input, no RNG state.
pub fn splitmix64(x: u64) -> u64 {
    let z = x.wrapping_add(0x9e3779b97f4a7c15);
    let z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// A lattice point's value in 0..1.
fn lattice(seed: u64, ch: u64, lx: i32, ly: i32) -> f32 {
    let at = seed ^ ch ^ ((lx as i64 as u64) << 20) ^ ((ly as i64 as u64) << 40);
    (splitmix64(at) >> 40) as f32 * (1.0 / 16_777_216.0)
}

/// Catmull-Rom weights of the 4 lattice points around each of a step's positions.
fn catmull_rom(step: i32) -> Vec<[f32; 4]> {
    (0..step)
        .map(|i| {
            let t = i as f32 / step as f32;
            let (t2, t3) = (t * t, t * t * t);
            [-0.5 * t + t2 - 0.5 * t3, 1.0 - 2.5 * t2 + 1.5 * t3, 0.5 * t + 2.0 * t2 - 1.5 * t3, -0.5 * t2 + 0.5 * t3]
        })
        .collect()
}

/// The x and y lattice steps of a square-form step at this cell size: x scales with `cell.0 / 16`, y is the x step
/// rounded at the cell's aspect (half up: 10 gives 10 and 8 at 16 x 12).
fn steps(step: i32, cell: (i32, i32)) -> (i32, i32) {
    let sx = (step * cell.0 / CELL_PX).max(1);
    let sy = (2 * sx * cell.1 + cell.0) / (2 * cell.0);
    (sx, sy.max(1))
}

/// Adds `weight` x `N(step, ch)` to `out`: value noise on a lattice of `step` art px, Catmull-Rom over the 4 x 4 lattice
/// points, clamped to 0..1. Separable: the lattice rows collapse once per pixel row.
fn add_noise(out: &mut [f32], w: i32, h: i32, seed: u64, ch: u64, step: (i32, i32), weight: f32) {
    let (sx, sy) = step;
    let (nx, ny) = ((w / sx + 4) as usize, (h / sy + 4) as usize);
    let mut lat = Vec::with_capacity(nx * ny);
    for ly in 0..ny as i32 {
        for lx in 0..nx as i32 {
            lat.push(lattice(seed, ch, lx - 1, ly - 1));
        }
    }
    let (wx, wy) = (catmull_rom(sx), catmull_rom(sy));
    let mut col = vec![0.0f32; nx];
    for ay in 0..h {
        let base = (ay / sy) as usize * nx;
        let k = wy[(ay % sy) as usize];
        for (c, v) in col.iter_mut().enumerate() {
            *v = k[0] * lat[base + c] + k[1] * lat[base + nx + c] + k[2] * lat[base + 2 * nx + c] + k[3] * lat[base + 3 * nx + c];
        }
        let row = &mut out[(ay * w) as usize..((ay + 1) * w) as usize];
        for (ax, o) in row.iter_mut().enumerate() {
            let lx = ax / sx as usize;
            let t = wx[ax % sx as usize];
            let v = t[0] * col[lx] + t[1] * col[lx + 1] + t[2] * col[lx + 2] + t[3] * col[lx + 3];
            *o += weight * v.clamp(0.0, 1.0);
        }
    }
}

/// One centred box pass of width `2 r + 1` over `src`, a running sum in integers; only `r..len - r` is written.
fn box_pass(src: &[i32], dst: &mut [i32], r: usize) {
    if src.len() < 2 * r + 1 {
        return;
    }
    let mut sum: i32 = src[..2 * r + 1].iter().sum();
    dst[r] = sum;
    for i in r + 1..src.len() - r {
        sum += src[i + r] - src[i - r - 1];
        dst[i] = sum;
    }
}

/// The odd box widths at this cell size: 11 x, and the odd width nearest its aspect's y (11 and 9 at 16 x 12).
fn box_widths(cell: (i32, i32)) -> (i32, i32) {
    let wx = (11 * cell.0 / CELL_PX) | 1;
    let wy = (wx * cell.1 / cell.0 / 2) * 2 + 1;
    (wx, wy)
}

/// `bl_k`: a layer's "this ground or one above it" mask as 0..1 per art pixel, limited to its bounding box plus the
/// blur's reach and a margin (0 outside it).
struct Field {
    x0: i32,
    x1: i32,
    y0: i32,
    y1: i32,
    v: Vec<f32>,
}

impl Field {
    /// Three box passes each way over the upscaled mask, off-map cells clamped to the nearest edge cell. A cell row's
    /// pixels are alike, so the x passes run once per cell row and the y passes become a weight per cell row at each
    /// pixel row: the same integers as running every pass over every pixel.
    fn blur(mask: impl Fn(i32, i32) -> bool, cols: i32, rows: i32, cell: (i32, i32), w: i32, h: i32) -> Option<Field> {
        let (cw, ch) = cell;
        let (mut cx0, mut cx1, mut cy0, mut cy1) = (cols, -1, rows, -1);
        for y in 0..rows {
            for x in 0..cols {
                if mask(x, y) {
                    (cx0, cx1, cy0, cy1) = (cx0.min(x), cx1.max(x), cy0.min(y), cy1.max(y));
                }
            }
        }
        if cx1 < 0 {
            return None;
        }
        let (wx, wy) = box_widths(cell);
        let (rx, ry) = ((wx / 2) as usize, (wy / 2) as usize);
        let (reach_x, reach_y) = ((3 * rx) as i32, (3 * ry) as i32);
        let (x0, x1) = ((cx0 * cw - reach_x - 3).max(0), ((cx1 + 1) * cw + reach_x + 3).min(w));
        let (y0, y1) = ((cy0 * ch - reach_y - 3).max(0), ((cy1 + 1) * ch + reach_y + 3).min(h));
        let (fw, fh) = ((x1 - x0) as usize, (y1 - y0) as usize);

        // The x passes, one row per cell row.
        let len = fw + 6 * rx;
        let mut across: Vec<Option<Vec<i32>>> = vec![None; rows as usize];
        let (mut a, mut b) = (vec![0i32; len], vec![0i32; len]);
        for r in 0..rows {
            if !(0..cols).any(|c| mask(c, r)) {
                continue;
            }
            for (i, v) in a.iter_mut().enumerate() {
                let px = x0 - reach_x + i as i32;
                *v = mask(px.div_euclid(cw).clamp(0, cols - 1), r) as i32;
            }
            for _ in 0..3 {
                box_pass(&a, &mut b, rx);
                std::mem::swap(&mut a, &mut b);
            }
            across[r as usize] = Some(a[3 * rx..3 * rx + fw].to_vec());
        }

        // The y kernel: a box of `wy` three times over.
        let mut kernel = vec![1i64];
        for _ in 0..3 {
            let mut next = vec![0i64; kernel.len() + wy as usize - 1];
            for (i, k) in kernel.iter().enumerate() {
                for n in &mut next[i..i + wy as usize] {
                    *n += *k;
                }
            }
            kernel = next;
        }
        let den = ((wx as i64).pow(3) * (wy as i64).pow(3)) as f32;

        let mut v = vec![0.0f32; fw * fh];
        let mut weights = vec![0i64; rows as usize];
        let mut sum = vec![0i64; fw];
        for ay in y0..y1 {
            weights.fill(0);
            for (j, k) in kernel.iter().enumerate() {
                let py = ay + j as i32 - 3 * ry as i32;
                weights[py.div_euclid(ch).clamp(0, rows - 1) as usize] += *k;
            }
            sum.fill(0);
            for (r, wt) in weights.iter().enumerate() {
                if *wt == 0 {
                    continue;
                }
                if let Some(row) = &across[r] {
                    for (s, h) in sum.iter_mut().zip(row) {
                        *s += *wt * *h as i64;
                    }
                }
            }
            let out = &mut v[(ay - y0) as usize * fw..(ay - y0 + 1) as usize * fw];
            for (o, s) in out.iter_mut().zip(&sum) {
                *o = *s as f32 / den;
            }
        }
        Some(Field { x0, x1, y0, y1, v })
    }

    #[inline]
    fn at(&self, ax: i32, ay: i32) -> Option<f32> {
        if ax < self.x0 || ax >= self.x1 || ay < self.y0 || ay >= self.y1 {
            return None;
        }
        Some(self.v[(ay - self.y0) as usize * (self.x1 - self.x0) as usize + (ax - self.x0) as usize])
    }
}

/// A layer's colour by art pixel: its texture at an offset, or flat, by the tone. The tables save a division per pixel.
struct Paint<'a> {
    tex: Option<&'a [u8]>,
    xs: Vec<u32>,
    ys: Vec<u32>,
    colour: [f32; 3],
    tone: f32,
}

impl<'a> Paint<'a> {
    fn new(layer: &GroundLayer<'a>, index: usize, seed: u64, w: i32, h: i32) -> Self {
        let colour = [layer.colour[0] as f32, layer.colour[1] as f32, layer.colour[2] as f32];
        let tex = layer.tex.filter(|t| t.w > 0 && t.h > 0 && t.px.len() == (t.w * t.h * 4) as usize);
        let (mut xs, mut ys) = (Vec::new(), Vec::new());
        if let Some(t) = tex {
            let off = splitmix64(seed ^ index as u64);
            let (ox, oy) = ((off % t.w as u64) as i32, ((off >> 32) % t.h as u64) as i32);
            xs = (0..w).map(|x| ((x + ox) % t.w * 4) as u32).collect();
            ys = (0..h).map(|y| ((y + oy) % t.h * t.w * 4) as u32).collect();
        }
        Self { tex: tex.map(|t| t.px.as_slice()), xs, ys, colour, tone: layer.tone }
    }

    #[inline]
    fn at(&self, ax: usize, ay: usize, t: f32) -> [f32; 3] {
        let k = 1.0 + self.tone * t;
        match self.tex {
            Some(px) => {
                let i = (self.ys[ay] + self.xs[ax]) as usize;
                [px[i] as f32 * k, px[i + 1] as f32 * k, px[i + 2] as f32 * k]
            }
            None => [self.colour[0] * k, self.colour[1] * k, self.colour[2] * k],
        }
    }
}

#[inline]
fn put(out: &mut [u8], i: usize, c: [f32; 3]) {
    // `as u8` saturates; adding .5 rounds, and a whole colour at a tone of 0 stays exactly itself.
    out[i * 4] = (c[0] + 0.5) as u8;
    out[i * 4 + 1] = (c[1] + 0.5) as u8;
    out[i * 4 + 2] = (c[2] + 0.5) as u8;
    out[i * 4 + 3] = 255;
}

#[inline]
fn times(c: [f32; 3], k: f32) -> [f32; 3] {
    [c[0] * k, c[1] * k, c[2] * k]
}

/// The art size of a job `compose` can make: every part positive, a cell of at most `CELL_MAX`, at most `LAYERS_MAX`
/// layers, `ART_PX_MAX` art px, `PLACED_MAX` placed patches and `MARKS_MAX` marks a layer, and cells and blocked flags that
/// fill the grid. None composes nothing.
fn art_size(job: &GroundJob) -> Option<(i32, i32)> {
    let (cw, ch) = job.cell;
    if job.cells.is_empty() || job.layers.is_empty() || job.layers.len() > LAYERS_MAX || job.cols <= 0 || job.rows <= 0 || !(1..=CELL_MAX).contains(&cw) || !(1..=CELL_MAX).contains(&ch) {
        return None;
    }
    let (w, h) = (job.cols.checked_mul(cw)?, job.rows.checked_mul(ch)?);
    let n = job.cols as i64 * job.rows as i64;
    if n > job.cells.len() as i64 || n > job.blocked.len() as i64 || w as i64 * h as i64 > ART_PX_MAX {
        return None;
    }
    if job.placed.len() > PLACED_MAX || job.layers.iter().any(|l| l.marks.len() > MARKS_MAX) {
        return None;
    }
    Some((w, h))
}

/// The whole map's ground: `cols x cell.0` by `rows x cell.1` art px, opaque, a pure function of the job. A job with
/// empty cells, cells or blocked flags that do not fill the grid, no layers, or a size past the limits (`CELL_MAX`,
/// `LAYERS_MAX`, `ART_PX_MAX`, `PLACED_MAX`, `MARKS_MAX`) composes nothing: a 0 x 0 image.
pub fn compose(job: &GroundJob) -> Image {
    let Some((w, h)) = art_size(job) else {
        return Image { w: 0, h: 0, px: Vec::new() };
    };
    let mut b = grounds(job, w, h);
    dress(job, &mut b);
    Image { w, h, px: b.px }
}

/// What section 2 of the spec makes and the dressing reads.
struct Base {
    w: i32,
    h: i32,
    px: Vec<u8>,
    /// The rank of each art pixel's top ground.
    top: Vec<u8>,
    /// Layer index by rank.
    by_rank: Vec<usize>,
    /// Rank by cell.
    cell_rank: Vec<u8>,
}

/// The grounds alone: textures, tone, edges, worn bands, strays, bites and sunken rims.
fn grounds(job: &GroundJob, w: i32, h: i32) -> Base {
    let (cw, ch) = job.cell;
    let n = job.cols as i64 * job.rows as i64;
    let (uw, uh) = (w as usize, h as usize);
    let cell = job.cell;
    let tilted = cw != ch;
    let seed = job.seed;

    // A rank is a layer's place in paint order; a cell's index past the layers counts as the lowest ground.
    let mut by_rank: Vec<usize> = (0..job.layers.len()).collect();
    by_rank.sort_by_key(|i| (job.layers[*i].order, *i));
    let mut rank_of = vec![0u8; job.layers.len()];
    for (rank, i) in by_rank.iter().enumerate() {
        rank_of[*i] = rank as u8;
    }
    let cell_rank: Vec<u8> = job.cells[..n as usize].iter().map(|c| rank_of.get(*c as usize).copied().unwrap_or(0)).collect();
    let ranks = by_rank.len();
    let at = |rank: u8, x: i32, y: i32| cell_rank[(y * job.cols + x) as usize] >= rank;

    let fields: Vec<Option<Field>> = (0..ranks).map(|k| if k == 0 { None } else { Field::blur(|x, y| at(k as u8, x, y), job.cols, job.rows, cell, w, h) }).collect();
    let paints: Vec<Paint> = by_rank.iter().map(|i| Paint::new(&job.layers[*i], *i, seed, w, h)).collect();
    let wears: Vec<f32> = by_rank.iter().map(|i| job.layers[*i].wear).collect();
    let strays: Vec<bool> = by_rank.iter().map(|i| job.layers[*i].strays).collect();

    // The noise, only what this job's layers read (a field no pixel reads cannot change a pixel).
    let any_upper = fields.iter().any(|f| f.is_some());
    let any_strays = fields.iter().zip(&strays).any(|(f, s)| f.is_some() && *s);
    let any_tone = by_rank.iter().any(|i| job.layers[*i].tone != 0.0);
    let field = || vec![0.0f32; uw * uh];
    let mut edge = Vec::new();
    let mut bite = Vec::new();
    if any_upper {
        edge = field();
        add_noise(&mut edge, w, h, seed, 1, steps(10, cell), 0.6);
        add_noise(&mut edge, w, h, seed, 2, steps(4, cell), 0.4);
        bite = field();
        add_noise(&mut bite, w, h, seed, 4, steps(3, cell), 1.0);
    }
    let mut stray_noise = Vec::new();
    if any_strays {
        stray_noise = field();
        add_noise(&mut stray_noise, w, h, seed, 3, steps(3, cell), 1.0);
    }
    let mut tone = Vec::new();
    if any_tone {
        tone = field();
        add_noise(&mut tone, w, h, seed, 7, steps(48, cell), 0.7);
        add_noise(&mut tone, w, h, seed, 8, steps(20, cell), 0.3);
    }

    // A rank's mask holds every rank above it (a lower ground runs under the upper ones), so where two grounds meet a
    // middle one's mask reads the same though none of its cells is near. Its worn band, strays and bites read its own
    // cells' share instead, or a third ground would show between the two: the masks' difference, exactly 0 when no
    // cell of it is in reach. On the top rank and on two layers it is the mask itself. The band tests the share against
    // its threshold, with the same push, so a ground's wear fades in with its share rather than switching on at the
    // blur's reach in a straight line.
    let blur_at = |k: usize, ax: i32, ay: i32| fields.get(k).and_then(|f| f.as_ref()).and_then(|f| f.at(ax, ay)).unwrap_or(0.0);
    let own = |k: usize, ax: i32, ay: i32| blur_at(k, ax, ay) - blur_at(k + 1, ax, ay);

    let mut px = vec![0u8; uw * uh * 4];
    let mut top = vec![0u8; uw * uh];
    for ay in 0..h {
        let row = ay as usize * uw;
        for ax in 0..w {
            let i = row + ax as usize;
            let t = if any_tone { tone[i] - 0.5 } else { 0.0 };
            let (mut g, mut vg) = (0usize, 0.0f32);
            let (mut wear, mut stray) = (0.0f32, None);
            let push = if any_upper { (edge[i] - 0.5) * 0.62 } else { 0.0 };
            for k in (1..ranks).rev() {
                let Some(bl) = fields[k].as_ref().and_then(|f| f.at(ax, ay)) else { continue };
                let v = bl + push;
                if v > 0.5 {
                    (g, vg) = (k, v);
                    break;
                }
                if wears[k] > wear && own(k, ax, ay) + push > 0.34 {
                    wear = wears[k];
                }
                if stray.is_none() && strays[k] && stray_noise[i] > 0.82 {
                    let mine = own(k, ax, ay);
                    if mine > 0.16 && mine < 0.42 {
                        stray = Some(k);
                    }
                }
            }
            top[i] = g as u8;
            let (ux, uy) = (ax as usize, ay as usize);
            let mut c = times(paints[g].at(ux, uy, t), 1.0 - wear);
            if let Some(k) = stray {
                c = times(paints[k].at(ux, uy, t), 0.92);
            }
            if g > 0 && vg < 0.56 && bite[i] > 0.7 {
                let mut under = (0usize, 1.0 - blur_at(1, ax, ay));
                for k in 1..g {
                    let mine = own(k, ax, ay);
                    if mine > under.1 {
                        under = (k, mine);
                    }
                }
                c = times(paints[under.0].at(ux, uy, t), 0.95);
            }
            put(&mut px, i, c);
        }
    }

    // The sunken rims: a pixel of a sunken ground takes its colour darkened on the side of a neighbour.
    let sunken: Vec<bool> = by_rank.iter().map(|i| job.layers[*i].sunken).collect();
    if sunken.iter().any(|s| *s) {
        let other = |ax: i32, ay: i32, g: u8| ax >= 0 && ay >= 0 && ax < w && ay < h && top[(ay * w + ax) as usize] != g;
        for ay in 0..h {
            for ax in 0..w {
                let i = (ay * w + ax) as usize;
                let g = top[i];
                if !sunken[g as usize] {
                    continue;
                }
                let k = if other(ax, ay - 1, g) {
                    0.55
                } else if (tilted && (other(ax, ay - 2, g) || other(ax, ay - 3, g))) || other(ax - 1, ay, g) || other(ax + 1, ay, g) || other(ax, ay + 1, g) {
                    0.7
                } else {
                    continue;
                };
                let t = if any_tone { tone[i] - 0.5 } else { 0.0 };
                put(&mut px, i, times(paints[g as usize].at(ax as usize, ay as usize, t), k));
            }
        }
    }
    Base { w, h, px, top, by_rank, cell_rank }
}

/// A layer's `patches`, 0 when it is off or past `PATCHES_MAX`.
fn patch_strength(layer: &GroundLayer) -> f32 {
    if layer.patches > 0.0 && layer.patches <= PATCHES_MAX {
        layer.patches
    } else {
        0.0
    }
}

/// The patch noise `q`: `.7 N(14, 5) + .3 N(5, 6)`.
fn patch_noise(job: &GroundJob, w: i32, h: i32) -> Vec<f32> {
    let mut q = vec![0.0f32; w as usize * h as usize];
    add_noise(&mut q, w, h, job.seed, 5, steps(14, job.cell), 0.7);
    add_noise(&mut q, w, h, job.seed, 6, steps(5, job.cell), 0.3);
    q
}

/// `o_g` of each rank `wanted` says: the blurred mask of cells that are not that ground or are blocked. None also means no
/// such cell, so 0 everywhere.
fn other_fields(job: &GroundJob, b: &Base, wanted: impl Fn(usize) -> bool) -> Vec<Option<Field>> {
    let (w, h, cols) = (b.w, b.h, job.cols);
    (0..b.by_rank.len())
        .map(|g| {
            if !wanted(g) {
                return None;
            }
            let other = |x: i32, y: i32| {
                let i = (y * cols + x) as usize;
                b.cell_rank[i] as usize != g || job.blocked[i]
            };
            Field::blur(other, cols, job.rows, job.cell, w, h)
        })
        .collect()
}

#[inline]
fn other_at(field: &Option<Field>, ax: i32, ay: i32) -> f32 {
    field.as_ref().and_then(|f| f.at(ax, ay)).unwrap_or(0.0)
}

/// The largest random patch at this cell size: `PATCH_CAP` by the cell's area (260 art px square, 195 tilted, the same
/// share of a cell).
pub fn patch_cap(cell: (i32, i32)) -> i32 {
    PATCH_CAP * cell.0 * cell.1 / (CELL_PX * CELL_PX)
}

/// A candidate for a random patch: one 4-connected group of art pixels of one ground where the patch noise is high.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchGroup {
    /// The ground, as an index into the job's layers.
    pub layer: usize,
    /// Its art pixels, all of them, whether kept or not.
    pub px: Vec<(i32, i32)>,
    /// Kept: drawn whole. Otherwise too large or too near another ground: not one pixel is drawn.
    pub kept: bool,
}

/// What `compose` decides about random patches, for tests and tools.
pub fn patch_groups(job: &GroundJob) -> Vec<PatchGroup> {
    let Some((w, h)) = art_size(job) else {
        return Vec::new();
    };
    let b = grounds(job, w, h);
    let strengths: Vec<f32> = b.by_rank.iter().map(|i| patch_strength(&job.layers[*i])).collect();
    if strengths.iter().all(|s| *s == 0.0) {
        return Vec::new();
    }
    let q = patch_noise(job, w, h);
    let others = other_fields(job, &b, |g| strengths[g] > 0.0);
    find_groups(job, &b, &q, &strengths, &others)
}

/// Section 3.1 steps 1 to 3: the candidates of every ground with patches, each whole, kept or dropped.
fn find_groups(job: &GroundJob, b: &Base, q: &[f32], strengths: &[f32], others: &[Option<Field>]) -> Vec<PatchGroup> {
    const SHAPE: f32 = 0.70;
    let cap = patch_cap(job.cell) as usize;
    let (w, h) = (b.w, b.h);
    let mut seen = vec![false; b.top.len()];
    let mut groups = Vec::new();
    let mut stack: Vec<(i32, i32)> = Vec::new();
    for ay in 0..h {
        for ax in 0..w {
            let i = (ay * w + ax) as usize;
            let g = b.top[i];
            if seen[i] || q[i] <= SHAPE || strengths[g as usize] == 0.0 {
                continue;
            }
            seen[i] = true;
            stack.push((ax, ay));
            let mut px = Vec::new();
            while let Some((x, y)) = stack.pop() {
                px.push((x, y));
                for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let n = (ny * w + nx) as usize;
                    if !seen[n] && b.top[n] == g && q[n] > SHAPE {
                        seen[n] = true;
                        stack.push((nx, ny));
                    }
                }
            }
            let other = &others[g as usize];
            let kept = px.len() <= cap && px.iter().all(|(x, y)| other_at(other, *x, *y) <= CLEARANCE);
            groups.push(PatchGroup { layer: b.by_rank[g as usize], px, kept });
        }
    }
    groups
}

/// Is this placed patch one `compose` draws? Inside the grid, a strength in range, and its rect and a ring of one cell (cut
/// off at the map's edge) all one ground and free of blocked cells.
fn placed_fits(p: &PlacedPatch, cols: i32, rows: i32, cells: &[u8], blocked: &[bool]) -> bool {
    let (x, y, pw, ph) = (p.x as i64, p.y as i64, p.w as i64, p.h as i64);
    if x < 0 || y < 0 || pw < 1 || ph < 1 || x + pw > cols as i64 || y + ph > rows as i64 || !(0.0..=PATCHES_MAX).contains(&p.strength) {
        return false;
    }
    let (cols, rows) = (cols as i64, rows as i64);
    let ground = cells[(y * cols + x) as usize];
    for ty in (y - 1).max(0)..(y + ph + 1).min(rows) {
        for tx in (x - 1).max(0)..(x + pw + 1).min(cols) {
            let i = (ty * cols + tx) as usize;
            if cells[i] != ground || blocked[i] {
                return false;
            }
        }
    }
    true
}

#[inline]
fn dim(px: &mut [u8], i: usize, k: f32) {
    for c in &mut px[i * 4..i * 4 + 3] {
        *c = (*c as f32 * k + 0.5) as u8;
    }
}

/// One pixel of a mark that shows: x, y, its colour, its alpha 0-1.
type MarkPixel = (i32, i32, [f32; 3], f32);

/// A mark's pixels that show; None for art `compose` cannot place.
fn mark_pixels(img: &Image) -> Option<Vec<MarkPixel>> {
    if img.w < 1 || img.h < 1 || img.w > MARK_PX_MAX || img.h > MARK_PX_MAX || img.px.len() != (img.w * img.h * 4) as usize {
        return None;
    }
    let mut out = Vec::new();
    for y in 0..img.h {
        for x in 0..img.w {
            let p = &img.px[((y * img.w + x) * 4) as usize..((y * img.w + x + 1) * 4) as usize];
            if p[3] > 0 {
                out.push((x, y, [p[0] as f32, p[1] as f32, p[2] as f32], p[3] as f32 / 255.0));
            }
        }
    }
    Some(out)
}

/// Section 3 of the spec, on the composite: random patches, placed patches, marks. A job that asks for none reads no noise
/// and changes no pixel.
fn dress(job: &GroundJob, b: &mut Base) {
    let (w, h) = (b.w, b.h);
    let strengths: Vec<f32> = b.by_rank.iter().map(|i| patch_strength(&job.layers[*i])).collect();
    let placed: Vec<&PlacedPatch> = job.placed.iter().filter(|p| placed_fits(p, job.cols, job.rows, &b.cell_rank, job.blocked)).collect();
    let tries: Vec<i64> = (0..b.by_rank.len())
        .map(|rank| {
            let l = &job.layers[b.by_rank[rank]];
            if l.marks.is_empty() || !(0..=MARKS_PER_100_MAX).contains(&l.marks_per_100) {
                return 0;
            }
            l.marks_per_100 as i64 * b.cell_rank.iter().filter(|r| **r as usize == rank).count() as i64 / 100
        })
        .collect();
    let any_patches = strengths.iter().any(|s| *s > 0.0);
    let any_marks = tries.iter().any(|t| *t > 0);
    if !any_patches && placed.is_empty() && !any_marks {
        return;
    }
    let q = if any_patches || !placed.is_empty() { patch_noise(job, w, h) } else { Vec::new() };
    let others = other_fields(job, b, |g| strengths[g] > 0.0 || tries[g] > 0);
    let mut patched = vec![false; b.top.len()];

    // Random patches: the larger, the fainter.
    if any_patches {
        let cap = patch_cap(job.cell) as f32;
        let mut rank_of = vec![0usize; job.layers.len()];
        for (rank, i) in b.by_rank.iter().enumerate() {
            rank_of[*i] = rank;
        }
        for group in find_groups(job, b, &q, &strengths, &others) {
            if !group.kept {
                continue;
            }
            let strong = strengths[rank_of[group.layer]] * (1.0 - 0.45 * group.px.len() as f32 / cap);
            for (x, y) in group.px {
                let i = (y * w + x) as usize;
                dim(&mut b.px, i, if q[i] > 0.74 { 1.0 - strong } else { 1.0 - 0.45 * strong });
                patched[i] = true;
            }
        }
    }

    // Placed patches: an ellipse in the rect, its edge pushed by the patch noise. Near the rect's side the outward push
    // gives way, so the shape closes before the side rather than being cut straight by it.
    for p in placed {
        let (cw, ch) = job.cell;
        let (x0, y0, x1, y1) = (p.x * cw, p.y * ch, (p.x + p.w) * cw, (p.y + p.h) * ch);
        let (rx, ry) = ((x1 - x0) as f32 * 0.5, (y1 - y0) as f32 * 0.5);
        let (cx, cy) = (x0 as f32 + rx, y0 as f32 + ry);
        for ay in y0..y1 {
            let dy = (ay as f32 + 0.5 - cy) / ry;
            for ax in x0..x1 {
                let dx = (ax as f32 + 0.5 - cx) / rx;
                let e = 1.0 - dx * dx - dy * dy;
                let i = (ay * w + ax) as usize;
                let inside = 1.0 - dx.abs().max(dy.abs());
                let shape = e + ((q[i] - 0.5) * 0.6).min(inside * 2.0);
                if shape > 0.0 {
                    dim(&mut b.px, i, if shape > 0.18 { 1.0 - p.strength } else { 1.0 - 0.45 * p.strength });
                    patched[i] = true;
                }
            }
        }
    }

    // Marks, each try on its own hash; dropped whole when any pixel would fall on another ground, near one, on a patch or
    // on an earlier mark.
    if any_marks {
        let mut used = patched;
        for (g, count) in tries.iter().enumerate() {
            let layer = b.by_rank[g];
            let marks: Vec<Option<Vec<MarkPixel>>> = job.layers[layer].marks.iter().map(|m| mark_pixels(m.img)).collect();
            for t in 0..*count {
                // The layer's index goes high, so one ground's try i is not another's try i + 1.
                let hash = splitmix64(job.seed ^ ((layer as u64) << 32) ^ t as u64);
                let which = ((hash >> 48) & 0xff) as usize % marks.len();
                let Some(art) = &marks[which] else { continue };
                let img = job.layers[layer].marks[which].img;
                if img.w > w || img.h > h || art.is_empty() {
                    continue;
                }
                let left = ((hash & 0xff_ffff) % (w - img.w + 1) as u64) as i32;
                let up = (((hash >> 24) & 0xff_ffff) % (h - img.h + 1) as u64) as i32;
                let flip = hash >> 63 == 1;
                let place = |x: i32, y: i32| (left + if flip { img.w - 1 - x } else { x }, up + y);
                let clear = art.iter().all(|(x, y, _, _)| {
                    let (ax, ay) = place(*x, *y);
                    let i = (ay * w + ax) as usize;
                    b.top[i] as usize == g && !used[i] && other_at(&others[g], ax, ay) <= CLEARANCE
                });
                if !clear {
                    continue;
                }
                for (x, y, c, alpha) in art {
                    let (ax, ay) = place(*x, *y);
                    let i = (ay * w + ax) as usize;
                    used[i] = true;
                    let a = alpha * MARK_ALPHA;
                    for (d, s) in b.px[i * 4..i * 4 + 3].iter_mut().zip(c) {
                        *d = (*d as f32 + (*s - *d as f32) * a + 0.5) as u8;
                    }
                }
            }
        }
    }
}

/// A soft blob, `2 rx + 1` by `2 ry + 1` px, its centre the middle pixel: `col`'s colour at alpha `alpha x (1 - d^2)^2`,
/// d the distance over the radii; 0 at and past them. (`col`'s own alpha is not read.)
pub fn soft_ellipse(rx: i32, ry: i32, col: Rgba, alpha: u8) -> Image {
    if rx <= 0 || ry <= 0 {
        return Image { w: 0, h: 0, px: Vec::new() };
    }
    let (w, h) = (2 * rx + 1, 2 * ry + 1);
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    let (r2x, r2y) = ((rx * rx) as f32, (ry * ry) as f32);
    for y in -ry..=ry {
        for x in -rx..=rx {
            let d2 = (x * x) as f32 / r2x + (y * y) as f32 / r2y;
            let a = if d2 >= 1.0 {
                0
            } else {
                let f = 1.0 - d2;
                (alpha as f32 * f * f + 0.5) as u8
            };
            px.extend_from_slice(&[col[0], col[1], col[2], a]);
        }
    }
    Image { w, h, px }
}

/// A seamless two-tone texture (each pixel `a` or `b` by a hash, half and half): the fixture the Gallery, the tests and
/// the timing use, so none of them needs a file.
pub fn two_tone(w: i32, h: i32, a: Rgba, b: Rgba, seed: u64) -> Image {
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let c = if splitmix64(seed ^ ((y as u64) << 16) ^ x as u64) & 1 == 0 { a } else { b };
            px.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    Image { w, h, px }
}

/// An art image and the one physical copy.
struct Physical {
    key: GroundKey,
    size: (i32, i32),
    px: Vec<u8>,
}

thread_local! {
    static ART: RefCell<Vec<(GroundKey, Rc<Image>)>> = const { RefCell::new(Vec::new()) };
    static COPY: RefCell<Option<Physical>> = const { RefCell::new(None) };
    static COMPOSES: Cell<u64> = const { Cell::new(0) };
    static EXPANDS: Cell<u64> = const { Cell::new(0) };
}

/// Cache misses on this thread that composed; tests and timings read it.
pub fn ground_composes() -> u64 {
    COMPOSES.with(|n| n.get())
}

/// Physical copies made on this thread.
pub fn ground_expands() -> u64 {
    EXPANDS.with(|n| n.get())
}

/// The bytes the cache holds on this thread now: the art images and the physical copy.
pub fn ground_held_bytes() -> usize {
    let art: usize = ART.with(|c| c.borrow().iter().map(|(_, i)| i.px.len()).sum());
    art + COPY.with(|c| c.borrow().as_ref().map_or(0, |p| p.px.len()))
}

/// The art image expanded to `w` x `h` physical px, nearest, the sampling `Painter::image` uses (whole pixels at a whole
/// scale). A row repeated at the scale is copied, so most of the cost is one pass over the art image.
pub fn expand(art: &Image, w: i32, h: i32) -> Vec<u8> {
    if art.w <= 0 || art.h <= 0 || w <= 0 || h <= 0 {
        return Vec::new();
    }
    let stride = w as usize * 4;
    let mut px = vec![0u8; stride * h as usize];
    let xs: Vec<usize> = (0..w as i64).map(|x| (x * art.w as i64 / w as i64) as usize * 4).collect();
    let mut last = None;
    for y in 0..h as usize {
        let sy = (y as i64 * art.h as i64 / h as i64) as usize;
        let start = y * stride;
        if last == Some(sy) {
            px.copy_within(start - stride..start, start);
            continue;
        }
        last = Some(sy);
        let src = &art.px[sy * art.w as usize * 4..(sy + 1) * art.w as usize * 4];
        for (d, x) in px[start..start + stride].as_chunks_mut::<4>().0.iter_mut().zip(&xs) {
            d.copy_from_slice(&src[*x..*x + 4]);
        }
    }
    px
}

/// Copies opaque rows (`stride` px wide) to `pr` on `c`, inside the canvas, its clip and its bounds.
fn blit(c: &mut Canvas, pr: Rect, stride: i32, px: &[u8]) {
    let mut v = pr.intersect(Rect::new(0, 0, c.w, c.h));
    for limit in [c.clip, c.bounds].into_iter().flatten() {
        v = v.intersect(limit);
    }
    if v.w <= 0 || v.h <= 0 {
        return;
    }
    let n = (v.w * 4) as usize;
    for y in v.y..v.y + v.h {
        let from = (((y - pr.y) * stride + v.x - pr.x) * 4) as usize;
        let to = ((y * c.w + v.x) * 4) as usize;
        c.px[to..to + n].copy_from_slice(&px[from..from + n]);
    }
}

/// `art` on `pr` by nearest sampling (`Painter::image`'s), for an opaque image: only what shows inside the canvas, its clip
/// and its bounds, a row repeated at the scale copied. So a map past `PHYSICAL_MAX` costs what shows, not its whole size.
fn blit_scaled(c: &mut Canvas, pr: Rect, art: &Image) {
    let mut v = pr.intersect(Rect::new(0, 0, c.w, c.h));
    for limit in [c.clip, c.bounds].into_iter().flatten() {
        v = v.intersect(limit);
    }
    if v.w <= 0 || v.h <= 0 {
        return;
    }
    let n = (v.w * 4) as usize;
    let xs: Vec<usize> = (v.x..v.x + v.w).map(|x| ((x - pr.x) as i64 * art.w as i64 / pr.w as i64) as usize * 4).collect();
    let mut last = None;
    for y in v.y..v.y + v.h {
        let sy = ((y - pr.y) as i64 * art.h as i64 / pr.h as i64) as usize;
        let to = ((y * c.w + v.x) * 4) as usize;
        if let Some((was, from)) = last.filter(|(was, _)| *was == sy) {
            c.px.copy_within(from..from + n, to);
            last = Some((was, from));
            continue;
        }
        last = Some((sy, to));
        let src = &art.px[sy * art.w as usize * 4..(sy + 1) * art.w as usize * 4];
        for (d, x) in c.px[to..to + n].as_chunks_mut::<4>().0.iter_mut().zip(&xs) {
            d.copy_from_slice(&src[*x..*x + 4]);
        }
    }
}

/// The ground on `r`: the art image of `key` from the cache (`job` composes it on a miss), its physical copy from the
/// cache (made on a miss, the old one freed first), the copy's rows blitted inside the clip. So a frame, a step or a
/// pan costs a copy of what shows; a resize one new copy; another map one compose unless it is among the last 4. A
/// copy past `PHYSICAL_MAX` draws the art image straight to what shows instead (the same pixels as `Painter::image`).
///
/// The copy's size follows the logical size and the scale, not the rect's rounded edges, so a pan at a fractional
/// window scale (where those edges differ by a pixel from place to place) keeps its copy; at a whole scale it is
/// exactly the rect.
///
/// `key` comes from `GroundKey::of`, which hashes every texture pixel: make it when the job changes and pass the kept
/// key each frame, never call `of` each frame. There is one physical copy per thread, so two draws at different sizes
/// in a frame make a new copy each, every frame: draw one size at a time.
pub fn draw(p: &mut Painter, key: GroundKey, r: Rect, job: impl FnOnce() -> Image) {
    let pr = p.v.rect(r);
    if pr.w <= 0 || pr.h <= 0 {
        return;
    }
    let held = ART.with(|cache| {
        let mut cache = cache.borrow_mut();
        let at = cache.iter().position(|e| e.0 == key)?;
        let entry = cache.remove(at);
        let img = entry.1.clone();
        cache.push(entry);
        Some(img)
    });
    let art = match held {
        Some(img) => img,
        None => {
            COMPOSES.with(|n| n.set(n.get() + 1));
            let img = Rc::new(job());
            ART.with(|cache| {
                let mut cache = cache.borrow_mut();
                cache.push((key.clone(), img.clone()));
                if cache.len() > ART_MAX {
                    cache.remove(0);
                }
            });
            img
        }
    };
    if art.w <= 0 || art.h <= 0 {
        return;
    }
    let size = (((r.w as f32 * p.v.s).ceil() as i32).max(pr.w), ((r.h as f32 * p.v.s).ceil() as i32).max(pr.h));
    if size.0 as usize * size.1 as usize * 4 > PHYSICAL_MAX {
        COPY.with(|copy| *copy.borrow_mut() = None);
        blit_scaled(p.c, pr, &art);
        return;
    }
    COPY.with(|copy| {
        let mut copy = copy.borrow_mut();
        if !copy.as_ref().is_some_and(|c| c.key == key && c.size == size) {
            *copy = None;
            EXPANDS.with(|n| n.set(n.get() + 1));
            *copy = Some(Physical { key, size, px: expand(&art, size.0, size.1) });
        }
        if let Some(c) = copy.as_ref() {
            blit(p.c, pr, size.0, &c.px);
        }
    });
}

/// What `compose_file` made.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComposedFile {
    pub w: i32,
    pub h: i32,
    pub compose_ms: f64,
}

/// The tools' way in (`ground_compose`, `docs/specs/REGIONGROUND1_ENGINE.md` section 6): composes a JSON job to a PNG, so
/// a review board's picture is the game's. Job fields (others are refused):
///
/// - `seed` (default 1), `cell` as whole `[w, h]`, each 1 to `CELL_MAX` (default `[16, 16]`);
/// - `cells`: rows of one legend char per cell;
/// - `legend`: char to layer, in the game's layer order (a texture's offset comes from its place here, so another order
///   moves the textures): `colour` whole `[r, g, b]`, each 0 to 255, and/or `tex` (a PNG path, relative to the job file;
///   another size is box-reduced to `TEX_CELLS` x cell), and optional `order` (default its place in the legend; unique),
///   `wear`, `strays`, `tone`, `sunken`, `patches` (0 to `PATCHES_MAX`), `marks` (PNG paths at the job's density, at most
///   `MARKS_MAX`, each at most `MARK_PX_MAX` a side), `marks_per_100` (whole, 0 to 10);
/// - `blocked` (optional): rows, `#` a blocked cell; `placed` (optional, at most `PLACED_MAX`): `{x, y, w, h, strength}` in
///   cells, strength 0 to `PATCHES_MAX`, the rect and a ring of one cell on one ground and free of blocked cells.
pub fn compose_file(job_path: &Path, out_path: &Path) -> Result<ComposedFile> {
    let text = std::fs::read_to_string(job_path).with_context(|| format!("reading {}", job_path.display()))?;
    let job: Value = serde_json::from_str(&text).with_context(|| format!("parsing {}", job_path.display()))?;
    let dir = job_path.parent().unwrap_or(Path::new("."));
    let loaded = load_job(&job, dir)?;
    let t0 = aegis_core::clock::Instant::now();
    let composed = compose_loaded(&loaded);
    let compose_ms = t0.elapsed().as_secs_f64() * 1000.0;
    if composed.w <= 0 || composed.h <= 0 {
        bail!("the job is too large: at most {ART_PX_MAX} art px");
    }
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(out_path).with_context(|| format!("writing {}", out_path.display()))?;
    let mut out = std::io::BufWriter::new(file);
    let mut enc = png::Encoder::new(&mut out, composed.w as u32, composed.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header()?;
    writer.write_image_data(&composed.px)?;
    writer.finish()?;
    out.flush().with_context(|| format!("writing {}", out_path.display()))?;
    Ok(ComposedFile { w: composed.w, h: composed.h, compose_ms })
}

/// A job file as read: everything `GroundJob` borrows.
struct LoadedJob {
    seed: u64,
    cols: i32,
    rows: i32,
    cell: (i32, i32),
    cells: Vec<u8>,
    blocked: Vec<bool>,
    layers: Vec<LoadedLayer>,
    placed: Vec<PlacedPatch>,
}

struct LoadedLayer {
    order: i32,
    tex: Option<Image>,
    colour: Rgba,
    wear: f32,
    strays: bool,
    tone: f32,
    patches: f32,
    sunken: bool,
    marks: Vec<Image>,
    marks_per_100: i32,
}

fn compose_loaded(job: &LoadedJob) -> Image {
    let layers: Vec<GroundLayer> = job
        .layers
        .iter()
        .map(|l| GroundLayer {
            order: l.order,
            tex: l.tex.as_ref(),
            colour: l.colour,
            wear: l.wear,
            strays: l.strays,
            tone: l.tone,
            patches: l.patches,
            sunken: l.sunken,
            marks: l.marks.iter().map(|img| GroundMark { img }).collect(),
            marks_per_100: l.marks_per_100,
        })
        .collect();
    compose(&GroundJob { seed: job.seed, cols: job.cols, rows: job.rows, cell: job.cell, cells: &job.cells, blocked: &job.blocked, layers: &layers, placed: &job.placed })
}

fn number(v: &Value, what: &str) -> Result<f64> {
    v.as_f64().ok_or_else(|| anyhow!("{what} must be a number"))
}

/// A whole number in `lo..=hi`; a fraction or anything past the range is refused, never cut.
fn whole(v: &Value, what: &str, lo: i64, hi: i64) -> Result<i32> {
    match v.as_i64() {
        Some(n) if (lo..=hi).contains(&n) => Ok(n as i32),
        _ => bail!("{what} must be a whole number from {lo} to {hi}, not {v}"),
    }
}

fn rows_of(v: &Value, what: &str) -> Result<Vec<String>> {
    let rows = v.as_array().ok_or_else(|| anyhow!("{what} must be an array of rows"))?;
    rows.iter().map(|r| r.as_str().map(str::to_string).ok_or_else(|| anyhow!("{what}: a row is not a string"))).collect()
}

fn known_keys(v: &serde_json::Map<String, Value>, known: &[&str], what: &str) -> Result<()> {
    if let Some(k) = v.keys().find(|k| !known.contains(&k.as_str())) {
        bail!("{what}: unknown field {k}");
    }
    Ok(())
}

fn load_job(job: &Value, dir: &Path) -> Result<LoadedJob> {
    let root = job.as_object().ok_or_else(|| anyhow!("the job must be an object"))?;
    known_keys(root, &["seed", "cell", "cells", "legend", "blocked", "placed"], "job")?;
    let seed = match root.get("seed") {
        Some(v) => v.as_u64().ok_or_else(|| anyhow!("seed must be a whole number"))?,
        None => 1,
    };
    let cell = match root.get("cell") {
        Some(v) => {
            let a = v.as_array().filter(|a| a.len() == 2).ok_or_else(|| anyhow!("cell must be [w, h]"))?;
            (whole(&a[0], "cell", 1, CELL_MAX as i64)?, whole(&a[1], "cell", 1, CELL_MAX as i64)?)
        }
        None => (CELL_PX, CELL_PX),
    };
    let rows = rows_of(root.get("cells").ok_or_else(|| anyhow!("no cells"))?, "cells")?;
    let cols = rows.first().map_or(0, |r| r.chars().count());
    if cols == 0 {
        bail!("cells is empty");
    }
    let legend = root.get("legend").and_then(Value::as_object).ok_or_else(|| anyhow!("no legend"))?;
    let chars: Vec<char> = legend.keys().map(|k| k.chars().next().filter(|_| k.chars().count() == 1).ok_or_else(|| anyhow!("legend key {k} is not one char"))).collect::<Result<_>>()?;
    if chars.len() > LAYERS_MAX {
        bail!("at most {LAYERS_MAX} layers");
    }
    let mut cells = Vec::with_capacity(cols * rows.len());
    for (y, row) in rows.iter().enumerate() {
        if row.chars().count() != cols {
            bail!("cells row {y} is not {cols} wide");
        }
        for c in row.chars() {
            cells.push(chars.iter().position(|l| *l == c).ok_or_else(|| anyhow!("cells row {y}: {c} is not in the legend"))? as u8);
        }
    }
    let mut blocked = vec![false; cells.len()];
    if let Some(v) = root.get("blocked") {
        let b = rows_of(v, "blocked")?;
        if b.len() != rows.len() || b.iter().any(|r| r.chars().count() != cols) {
            bail!("blocked must match cells in size");
        }
        for (y, row) in b.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                blocked[y * cols + x] = c == '#';
            }
        }
    }
    let load = |rel: &str| -> Result<Image> {
        let p = dir.join(rel);
        crate::art::load_png(&p).ok_or_else(|| anyhow!("cannot read {}", p.display()))
    };
    let mut layers = Vec::new();
    for (i, (k, v)) in legend.iter().enumerate() {
        let o = v.as_object().ok_or_else(|| anyhow!("legend {k} must be an object"))?;
        known_keys(o, &["order", "tex", "colour", "wear", "strays", "tone", "patches", "sunken", "marks", "marks_per_100"], &format!("legend {k}"))?;
        let f = |name: &str| -> Result<f32> { o.get(name).map_or(Ok(0.0), |v| number(v, name).map(|n| n as f32)) };
        let flag = |name: &str| -> Result<bool> { o.get(name).map_or(Ok(false), |v| v.as_bool().ok_or_else(|| anyhow!("{name} must be true or false"))) };
        let tex = match o.get("tex") {
            Some(v) => {
                let img = load(v.as_str().ok_or_else(|| anyhow!("legend {k}: tex must be a path"))?)?;
                let want = (TEX_CELLS * cell.0, TEX_CELLS * cell.1);
                Some(if (img.w, img.h) == want { img } else { img.downsample(want.0, want.1) })
            }
            None => None,
        };
        let colour = match o.get("colour") {
            Some(v) => {
                let a = v.as_array().filter(|a| a.len() == 3).ok_or_else(|| anyhow!("legend {k}: colour must be [r, g, b]"))?;
                let part = |i: usize| whole(&a[i], "colour", 0, 255).map(|n| n as u8);
                [part(0)?, part(1)?, part(2)?, 255]
            }
            None if tex.is_some() => [0, 0, 0, 255],
            None => bail!("legend {k}: needs tex or colour"),
        };
        let mut marks = Vec::new();
        if let Some(v) = o.get("marks") {
            let list = v.as_array().ok_or_else(|| anyhow!("legend {k}: marks must be paths"))?;
            if list.len() > MARKS_MAX {
                bail!("legend {k}: at most {MARKS_MAX} marks");
            }
            for m in list {
                let img = load(m.as_str().ok_or_else(|| anyhow!("legend {k}: marks must be paths"))?)?;
                if img.w > MARK_PX_MAX || img.h > MARK_PX_MAX {
                    bail!("legend {k}: a mark is {} x {}, at most {MARK_PX_MAX} a side at the job's density", img.w, img.h);
                }
                marks.push(img);
            }
        }
        layers.push(LoadedLayer {
            order: o.get("order").map_or(Ok(i as i32), |v| whole(v, "order", i32::MIN as i64, i32::MAX as i64))?,
            tex,
            colour,
            wear: f("wear")?,
            strays: flag("strays")?,
            tone: f("tone")?,
            patches: match f("patches")? {
                n if (0.0..=PATCHES_MAX).contains(&n) => n,
                n => bail!("legend {k}: patches must be 0 to {PATCHES_MAX}, not {n}"),
            },
            sunken: flag("sunken")?,
            marks,
            marks_per_100: o.get("marks_per_100").map_or(Ok(0), |v| whole(v, "marks_per_100", 0, 10))?,
        });
    }
    let mut orders: Vec<i32> = layers.iter().map(|l| l.order).collect();
    orders.sort_unstable();
    if let Some(same) = orders.windows(2).find(|w| w[0] == w[1]) {
        bail!("duplicate order {} in the legend", same[0]);
    }
    let grid = cells;
    let mut placed = Vec::new();
    if let Some(v) = root.get("placed") {
        let list = v.as_array().ok_or_else(|| anyhow!("placed must be an array"))?;
        if list.len() > PLACED_MAX {
            bail!("at most {PLACED_MAX} placed patches");
        }
        for p in list {
            let part = |name: &str| p.get(name).ok_or_else(|| anyhow!("placed: no {name}"));
            let cells = |name: &str| part(name).and_then(|v| whole(v, name, 0, i32::MAX as i64));
            let patch = PlacedPatch { x: cells("x")?, y: cells("y")?, w: cells("w")?, h: cells("h")?, strength: number(part("strength")?, "strength")? as f32 };
            if !placed_fits(&patch, cols as i32, rows.len() as i32, &grid, &blocked) {
                bail!("placed {patch:?}: needs a strength of 0 to {PATCHES_MAX}, and its rect and a ring of one cell inside the map, on one ground and free of blocked cells");
            }
            placed.push(patch);
        }
    }
    Ok(LoadedJob { seed, cols: cols as i32, rows: rows.len() as i32, cell, cells: grid, blocked, layers, placed })
}
