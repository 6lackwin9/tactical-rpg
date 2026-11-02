//! Two-layer drawing at the window's real resolution. Layout is in logical units; `View` maps them to physical pixels,
//! and a screen's `Grid` says which units it writes (LAYOUT720-E1: the old 640x360 design canvas, or the grid `GRID`).
//! The battlefield is pixel art scaled with nearest sampling; panels and text are drawn
//! smooth at full resolution (anti-aliased Open Sans, rounded panels), like the Godot HUD. LOOK1-E1: three faces
//! (`Typeface`: body, display, mono; the theme's fonts, `Fonts::sync`) and nine-slice frames (`Painter::nine`).

use crate::canvas::{Canvas, Image, Rect, Rgba};
use crate::theme::{Frame, Nine, NineFill};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

pub const LW: f32 = 640.0;
pub const LH: f32 = 360.0;

/// The layout grid new screens are written on, in units (LAYOUT720-E1): set here and nowhere else. Its width must be a
/// whole multiple of the old canvas, so that the old mapping stays whole.
pub const GRID: (i32, i32) = (1280, 720);

/// Grid units per old unit.
const UNITS: i32 = GRID.0 / LW as i32;
const _: () = assert!(GRID.0 % LW as i32 == 0 && GRID.1 == LH as i32 * UNITS, "the grid is a whole multiple of the old canvas");

/// Which units a screen writes its rects in (`Screen::grid`), and so how they reach the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Grid {
    /// The wrapper, every screen's default: old numbers filling the window, exactly the view `View::fit` gives.
    #[default]
    Old,
    /// Old numbers one to one on the grid: the whole picture smaller, in the top-left of the window.
    Quadrant,
    /// Grid units filling the window, for a screen written relative to its canvas.
    Native,
}

impl Grid {
    /// Units per old unit, `View.g`.
    pub fn g(self) -> i32 {
        match self {
            Grid::Native => UNITS,
            Grid::Old | Grid::Quadrant => 1,
        }
    }

    /// How many times smaller the scale is than the wrapper's.
    fn shrink(self) -> i32 {
        match self {
            Grid::Old => 1,
            Grid::Quadrant | Grid::Native => UNITS,
        }
    }

    /// The whole window in this screen's units: what a preview of the screen shows. Under `Quadrant` it is larger
    /// than the canvas the screen draws on; the rest stays blank.
    pub fn extent(self) -> (i32, i32) {
        (LW as i32 * self.shrink(), LH as i32 * self.shrink())
    }

    /// The name the agent link reports.
    pub fn name(self) -> &'static str {
        match self {
            Grid::Old => "old",
            Grid::Quadrant => "quadrant",
            Grid::Native => "native",
        }
    }
}

/// Logical -> physical mapping (uniform scale, letterboxed and centred).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub s: f32,
    pub ox: f32,
    pub oy: f32,
    /// Units per old unit: 1 on an unmoved screen, `GRID.0 / LW` on a native one. What art, thin strokes and sidecar
    /// numbers multiply by, as those stay in old units for good.
    pub g: i32,
}

impl View {
    pub fn fit(pw: u32, ph: u32) -> Self {
        let s = (pw as f32 / LW).min(ph as f32 / LH).max(0.25);
        Self { s, ox: ((pw as f32 - LW * s) / 2.0).floor(), oy: ((ph as f32 - LH * s) / 2.0).floor(), g: 1 }
    }
    /// The view a screen on `grid` draws with, `self` being the whole window's (`View::fit`). `Grid::Old` gives `self`
    /// exactly.
    pub fn for_grid(self, grid: Grid) -> Self {
        Self { s: self.s / grid.shrink() as f32, g: grid.g(), ..self }
    }
    /// `n` old units in this view's units.
    pub fn u(&self, n: i32) -> i32 {
        n * self.g
    }
    /// Physical px per old unit: for what stays sized in old units, such as a stroke or a frame's art.
    pub fn old_scale(&self) -> f32 {
        self.s * self.g as f32
    }
    /// The canvas a screen draws on, in this view's units.
    pub fn canvas(&self) -> (i32, i32) {
        (LW as i32 * self.g, LH as i32 * self.g)
    }
    pub fn x(&self, x: f32) -> i32 {
        (self.ox + x * self.s).round() as i32
    }
    pub fn y(&self, y: f32) -> i32 {
        (self.oy + y * self.s).round() as i32
    }
    /// Physical rect of a logical rect (edges rounded independently, so neighbours share edges without gaps).
    pub fn rect(&self, r: Rect) -> Rect {
        let (x0, y0) = (self.x(r.x as f32), self.y(r.y as f32));
        let (x1, y1) = (self.x((r.x + r.w) as f32), self.y((r.y + r.h) as f32));
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }
    /// Physical point -> logical point.
    pub fn to_logical(&self, px: f32, py: f32) -> (i32, i32) {
        (((px - self.ox) / self.s).floor() as i32, ((py - self.oy) / self.s).floor() as i32)
    }
    /// The physical middle of a logical point, which `to_logical` reads back as that point.
    pub fn to_physical(&self, x: i32, y: i32) -> (f32, f32) {
        (self.ox + (x as f32 + 0.5) * self.s, self.oy + (y as f32 + 0.5) * self.s)
    }
    /// The content rect of `r` in a frame, `r` and the result in this view's units (`nine_content_rect` for a view on
    /// any grid; the sidecar's `pad` and corners stay in old units).
    pub fn nine_content(&self, nine: &Nine, r: Rect) -> Rect {
        content_rect(nine, r, self.s, self.g)
    }
}

/// The three faces a theme names (LOOK1-E1, spec LOOK1 3.3): `Body` for every text as before, `Display` for frame
/// titles and headers, `Mono` for badge numbers. Without a theme file all three are the body font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Typeface {
    Body,
    Display,
    Mono,
}

impl Typeface {
    pub const ALL: [Typeface; 3] = [Typeface::Body, Typeface::Display, Typeface::Mono];

    /// The face's key in `ui/theme.json` `fonts`.
    pub fn name(self) -> &'static str {
        match self {
            Typeface::Body => "body",
            Typeface::Display => "display",
            Typeface::Mono => "mono",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// One face's font, its size factor and its glyph cache; `src` holds the bytes it was built from (None: the base
/// font), so a cache is reused only for those very bytes (`Arc::ptr_eq` on a held `Arc`, never a stale address).
struct FaceCache {
    font: fontdue::Font,
    scale: f32,
    src: Option<Arc<Vec<u8>>>,
    cache: HashMap<(char, u32), (fontdue::Metrics, Vec<u8>)>,
}

/// Rasterized glyphs per (face, char, physical px size).
pub struct Fonts {
    /// The font `Fonts::new` was given (the shell's UI font): every face without a font file of its own uses it.
    base: fontdue::Font,
    faces: [FaceCache; 3],
    /// The text size factor (settings `text_scale`: 0.85 Small .. 1.3 Extra large); every text size is multiplied by it.
    pub scale: f32,
    /// TEXTFIT1 text audit: while Some, every text drawn is recorded here (the shell's `text_audit` turns it on for
    /// one frame and reports what is cut, off the canvas or overlapping).
    pub audit: Option<Vec<TextRec>>,
    /// TEXTFIT2: while on, the texts drawn are transient (a damage float rising over the board) and the audit records
    /// them as such, so `text_audit::issues` ignores them; set through `Painter::transient`.
    pub transient: bool,
}

/// One text drawn during an audited frame, in logical units.
#[derive(Debug, Clone, PartialEq)]
pub struct TextRec {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub text: String,
    /// The text lies partly or wholly outside the clip in force (a scroll area): not a layout problem.
    pub clipped: bool,
    /// The full text when `text_fit` had to shorten it.
    pub cut: Option<String>,
    /// A near-opaque fill or panel drawn later covers the text's first letter or most of it (a dialog over the
    /// screen): hidden, so not a layout problem.
    pub covered: bool,
    /// TEXTFIT2: drawn inside `Painter::transient` (a float that moves and fades on its own): never a layout problem.
    pub transient: bool,
}

/// The fill alpha from which a fill or panel hides the audited text beneath it.
pub const COVER_ALPHA: u8 = 200;

impl Fonts {
    /// Fonts with `ttf` as every face (body, display and mono alike, each at size factor 1).
    pub fn new(ttf: &[u8]) -> Self {
        let font = fontdue::Font::from_bytes(ttf, fontdue::FontSettings::default()).expect("valid TTF");
        let face = || FaceCache { font: font.clone(), scale: 1.0, src: None, cache: HashMap::new() };
        Self { faces: [face(), face(), face()], base: font.clone(), scale: 1.0, audit: None, transient: false }
    }

    /// Sets `face` to the font in `ttf` (None: the base font) at size factor `scale`; the glyph cache is kept while the
    /// bytes stay the same `Arc` (`Arc::ptr_eq`). A TTF that does not parse leaves the face as it was (the theme
    /// loader has checked it).
    pub fn set_face(&mut self, face: Typeface, ttf: Option<&Arc<Vec<u8>>>, scale: f32) {
        let base = &self.base;
        let f = &mut self.faces[face.index()];
        f.scale = scale;
        let same = match (&f.src, ttf) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if same {
            return;
        }
        let font = match ttf {
            None => Some(base.clone()),
            Some(b) => fontdue::Font::from_bytes(b.as_slice(), fontdue::FontSettings::default()).ok(),
        };
        if let Some(font) = font {
            *f = FaceCache { font, scale, src: ttf.cloned(), cache: HashMap::new() };
        }
    }

    /// The faces of `theme` (`set_face` for each; cheap when nothing changed).
    pub fn sync(&mut self, theme: &crate::theme::Theme) {
        for face in Typeface::ALL {
            let spec = theme.face(face);
            self.set_face(face, spec.bytes.as_ref(), spec.scale);
        }
    }

    /// The size factor of `face` (`ui/theme.json` `fonts.<face>.scale`).
    pub fn face_scale(&self, face: Typeface) -> f32 {
        self.faces[face.index()].scale
    }

    fn glyph(&mut self, face: Typeface, c: char, px: u32) -> &(fontdue::Metrics, Vec<u8>) {
        let f = &mut self.faces[face.index()];
        let font = &f.font;
        f.cache.entry((c, px)).or_insert_with(|| font.rasterize(c, px as f32))
    }

    /// Physical advance width of `s` at `px` in the body face.
    pub fn width_px(&mut self, s: &str, px: u32) -> f32 {
        self.width_px_face(Typeface::Body, s, px)
    }

    /// Physical advance width of `s` at `px` in `face`.
    pub fn width_px_face(&mut self, face: Typeface, s: &str, px: u32) -> f32 {
        s.chars().map(|c| self.glyph(face, c, px).0.advance_width).sum()
    }
}

/// How many averaged frames are kept (a screen shows a few sizes), the oldest dropped first: a window drag cannot pile
/// them up.
const REDUCED_MAX: usize = 8;

/// What a composed frame's pixels depend on: the picture, the physical size, the physical px per art px (two views can
/// give one physical size) and the sidecar.
type ReducedKey = ((usize, i32, i32, u64), (i32, i32), u32, Nine);

thread_local! {
    static REDUCED: RefCell<Vec<(ReducedKey, Vec<u8>)>> = const { RefCell::new(Vec::new()) };
    static REDUCTIONS: Cell<u64> = const { Cell::new(0) };
}

/// (averagings so far on this thread, averaged frames held now), for tests.
pub fn nine_reductions() -> (u64, usize) {
    (REDUCTIONS.with(|n| n.get()), REDUCED.with(|c| c.borrow().len()))
}

/// Hands `draw` the frame composed once for a frame, size, view scale and sidecar.
fn reduced_nine(img: &Image, nine: &Nine, k: f32, pr: Rect, compose: impl FnOnce() -> Vec<u8>, draw: impl FnOnce(&[u8])) {
    let key = (crate::panes::picture_id(img), (pr.w, pr.h), k.to_bits(), nine.clone());
    REDUCED.with(|cache| {
        let mut cache = cache.borrow_mut();
        let entry = match cache.iter().position(|e| e.0 == key) {
            Some(at) => cache.remove(at),
            None => {
                REDUCTIONS.with(|n| n.set(n.get() + 1));
                (key, compose())
            }
        };
        draw(&entry.1);
        cache.push(entry);
        if cache.len() > REDUCED_MAX {
            cache.remove(0);
        }
    });
}

/// The nine cells averaged to their spans, RGBA row by row at `pr`'s size: a stretched cell to its span, a tiled one to
/// its source size at `k` and repeated.
fn compose_reduced(img: &Image, nine: &Nine, k: f32, pr: Rect, cols: &[(i32, i32, i32, i32); 3], rows: &[(i32, i32, i32, i32); 3]) -> Vec<u8> {
    let mut out = vec![0u8; (pr.w * pr.h * 4) as usize];
    for (j, &(sy, src_h, dy, dst_h)) in rows.iter().enumerate() {
        for (i, &(sx, src_w, dx, dst_w)) in cols.iter().enumerate() {
            let (mid_col, mid_row) = (i == 1, j == 1);
            let fill = if mid_col && mid_row { nine.center } else if mid_col || mid_row { nine.edges } else { NineFill::Stretch };
            if fill == NineFill::None || src_w <= 0 || src_h <= 0 || dst_w <= 0 || dst_h <= 0 {
                continue;
            }
            let tile = fill == NineFill::Tile;
            let mut cell = fit_cell(img, (sx, sy, src_w, src_h), (dst_w, dst_h), tile && mid_col, tile && mid_row, k);
            if !mid_col && !mid_row {
                sharpen(&mut cell, nine.sharpen);
            }
            for y in 0..dst_h {
                let (from, to) = ((y * dst_w * 4) as usize, (((dy - pr.y + y) * pr.w + dx - pr.x) * 4) as usize);
                let n = (dst_w * 4) as usize;
                out[to..to + n].copy_from_slice(&cell.px[from..from + n]);
            }
        }
    }
    out
}

/// `src` (x, y, w, h) of `img` averaged to `dst`: stretched, or tiled along an axis at its source size at `k`.
fn fit_cell(img: &Image, src: (i32, i32, i32, i32), dst: (i32, i32), tile_x: bool, tile_y: bool, k: f32) -> Image {
    let tw = if tile_x { ((src.2 as f32 * k).round() as i32).max(1) } else { dst.0 };
    let th = if tile_y { ((src.3 as f32 * k).round() as i32).max(1) } else { dst.1 };
    let cell = img.crop(src.0, src.1, src.2, src.3).downsample(tw, th);
    if (tw, th) == dst {
        return cell;
    }
    let mut px = vec![0u8; (dst.0 * dst.1 * 4) as usize];
    for y in 0..dst.1 {
        for x in 0..dst.0 {
            let (from, to) = (((y % th * tw + x % tw) * 4) as usize, ((y * dst.0 + x) * 4) as usize);
            px[to..to + 4].copy_from_slice(&cell.px[from..from + 4]);
        }
    }
    Image { w: dst.0, h: dst.1, px }
}

/// An unsharp mask of strength `amount` on the colours (a 3 x 3 blur; the alpha stays).
pub(crate) fn sharpen(img: &mut Image, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    let src = img.px.clone();
    for y in 0..img.h {
        for x in 0..img.w {
            let at = ((y * img.w + x) * 4) as usize;
            for c in 0..3 {
                let (mut sum, mut weight) = (0.0, 0.0);
                for (dy, wy) in [(-1, 1.0), (0, 2.0), (1, 1.0)] {
                    for (dx, wx) in [(-1, 1.0), (0, 2.0), (1, 1.0)] {
                        let (nx, ny) = ((x + dx).clamp(0, img.w - 1), (y + dy).clamp(0, img.h - 1));
                        sum += src[((ny * img.w + nx) * 4) as usize + c] as f32 * wx * wy;
                        weight += wx * wy;
                    }
                }
                let v = src[at + c] as f32;
                img.px[at + c] = (v + amount * (v - sum / weight)).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// `layer` over the RGBA buffer `out` (`w` wide) at (x, y), what falls outside dropped.
fn over(out: &mut [u8], w: i32, layer: &Image, x: i32, y: i32) {
    let h = out.len() as i32 / 4 / w;
    for ly in 0..layer.h {
        for lx in 0..layer.w {
            let (ox, oy) = (x + lx, y + ly);
            if ox < 0 || oy < 0 || ox >= w || oy >= h {
                continue;
            }
            let (from, to) = (((ly * layer.w + lx) * 4) as usize, ((oy * w + ox) * 4) as usize);
            let (a, under) = (layer.px[from + 3] as f32 / 255.0, out[to + 3] as f32 / 255.0);
            let total = a + under * (1.0 - a);
            if a <= 0.0 || total <= 0.0 {
                continue;
            }
            for c in 0..3 {
                let mixed = (layer.px[from + c] as f32 * a + out[to + c] as f32 * under * (1.0 - a)) / total;
                out[to + c] = mixed.round() as u8;
            }
            out[to + 3] = (total * 255.0).round() as u8;
        }
    }
}

/// How far a straddling frame can draw past the panel: half its edge bands at `k`, or its corners past their anchor at
/// `p`, both in physical px per art px.
fn straddle_over(nine: &Nine, k: f32, p: f32) -> i32 {
    let most = nine.slice.into_iter().max().unwrap_or(0) as f32;
    let anchor = nine.corner_anchor.map_or(most / 2.0, |[x, y]| x.max(y) as f32);
    let line = nine.line_width.map_or(0.0, |w| w * k * nine.scale.max(1) as f32 / 2.0);
    (most * k / 2.0).max(anchor * p).max(line).ceil() as i32 + 2
}

/// The art px a corner `side` art px long reaches once its tip is sized (`tip`, `tip_size`).
fn sized_corner(nine: &Nine, side: i32) -> i32 {
    match nine.tip {
        Some([from, _]) if nine.tip_size != 1.0 => from + (((side - from).max(0) as f32 * nine.tip_size).round() as i32).max(1),
        _ => side,
    }
}

/// The top-left form of a corner with its two heads resized from their bases on the arms (`tip`, `tip_size`).
fn sized_tips(q: Image, nine: &Nine) -> Image {
    let Some([from, half]) = nine.tip.filter(|_| nine.tip_size != 1.0) else { return q };
    let [ax, ay] = nine.corner_anchor.unwrap_or([nine.slice[0] / 2, nine.slice[1] / 2]);
    let (w, h) = (sized_corner(nine, q.w), sized_corner(nine, q.h));
    let wide = 2 * half + 1;
    let across = ((wide as f32 * nine.tip_size).round() as i32).max(1);
    let (head_x, head_y) = (q.crop(from, ay - half, q.w - from, wide), q.crop(ax - half, from, wide, q.h - from));
    let mut rest = q.clone();
    for y in 0..q.h {
        for x in 0..q.w {
            if (x >= from && (y - ay).abs() <= half) || (y >= from && (x - ax).abs() <= half) {
                rest.px[((y * q.w + x) * 4 + 3) as usize] = 0;
            }
        }
    }
    let mut out = vec![0u8; (w * h * 4) as usize];
    over(&mut out, w, &rest, 0, 0);
    over(&mut out, w, &head_x.downsample((w - from).max(1), across), from, ay - across / 2);
    over(&mut out, w, &head_y.downsample(across, (h - from).max(1)), ax - across / 2, from);
    Image { w, h, px: out }
}

/// The plain line a straddling frame's art has on its top band: its width in logical px and its colour. None: no line in
/// the art (a see-through band), or edges that are not a line (tiled art, a frame inside its rect).
pub fn nine_art_line(img: &Image, nine: &Nine) -> Option<(f32, Rgba)> {
    let [sl, st, sr, _] = nine.slice;
    if !nine.straddle || nine.edges != NineFill::Stretch || sl + sr >= img.w || st > img.h {
        return None;
    }
    let x = sl + (img.w - sl - sr) / 2;
    let drawn: Vec<i32> = (0..st).filter(|y| img.px[((y * img.w + x) * 4 + 3) as usize] > 0).collect();
    let (a, b) = (*drawn.first()?, *drawn.last()?);
    let at = (((a + b) / 2 * img.w + x) * 4) as usize;
    Some(((b - a + 1) as f32 / nine.scale.max(1) as f32, [img.px[at], img.px[at + 1], img.px[at + 2], img.px[at + 3]]))
}

/// The physical px per art px a straddling frame's corners draw at, one scale in both axes: the frame's own `k`, so they
/// follow the window and keep one size on any panel; on a panel too small for two corners, as large as fits so they do not
/// overlap (a px is left for rounding each corner's size and anchor apart).
fn corner_scale(nine: &Nine, k: f32, pr: Rect) -> f32 {
    let [ax, ay] = nine.corner_anchor.unwrap_or([nine.slice[0] / 2, nine.slice[1] / 2]);
    let room = |edge: i32, slice: i32, anchor: i32| (edge - 2).max(1) as f32 / 2.0 / (sized_corner(nine, slice) - anchor).max(1) as f32;
    (k * nine.corner_size).min(room(pr.w, nine.slice[0].min(nine.slice[2]), ax)).min(room(pr.h, nine.slice[1].min(nine.slice[3]), ay))
}

/// The content rect of logical `r` in a frame at view scale `view`, on an unmoved screen (on any grid: `View::nine_content`):
/// inside the sidecar's `pad` and, with `corner_inner`, past the corners' inward reach at the size they draw on this panel.
/// Frames without it: `Nine::content`.
pub fn nine_content_rect(nine: &Nine, r: Rect, view: f32) -> Rect {
    content_rect(nine, r, view, 1)
}

/// `nine_content_rect` with `s` physical px per unit and `g` units per old unit: worked out in old units, then multiplied.
fn content_rect(nine: &Nine, r: Rect, s: f32, g: i32) -> Rect {
    let [l, t, rr, b] = nine.pad.map(|p| p * g);
    let (l, t, rr, b) = match nine.corner_inner {
        None => (l, t, rr, b),
        Some([ix, iy]) => {
            let old = s * g as f32;
            let physical = Rect::new(0, 0, (r.w as f32 * s).round() as i32, (r.h as f32 * s).round() as i32);
            let p = corner_scale(nine, old / nine.scale.max(1) as f32, physical);
            let (dx, dy) = ((ix as f32 * p / old).ceil() as i32 * g, (iy as f32 * p / old).ceil() as i32 * g);
            (l.max(dx), t.max(dy), rr.max(dx), b.max(dy))
        }
    };
    Rect::new(r.x + l, r.y + t, (r.w - l - rr).max(0), (r.h - t - b).max(0))
}

/// `img` flipped along the axes asked.
fn mirrored(img: &Image, mx: bool, my: bool) -> Image {
    let mut px = vec![0u8; img.px.len()];
    for y in 0..img.h {
        for x in 0..img.w {
            let (fx, fy) = (if mx { img.w - 1 - x } else { x }, if my { img.h - 1 - y } else { y });
            let (from, to) = (((y * img.w + x) * 4) as usize, ((fy * img.w + fx) * 4) as usize);
            px[to..to + 4].copy_from_slice(&img.px[from..from + 4]);
        }
    }
    Image { w: img.w, h: img.h, px }
}

/// A straddling frame: the centre over exactly the panel, the edge bands centred on its boundary, the corners at `p`
/// physical px per art px with their anchor (`corner_anchor`, else their middle) on its corner points, over the rest, in a
/// buffer `reach` px larger than `pr` on every side. Plain bands run on past the corner points by the half width of what they
/// draw, so the lines close under an open corner; tiled bands run between the corners. Each corner is averaged from its outer corner, so mirrored art reduces to
/// mirrored pixels at any ratio.
fn compose_straddle(img: &Image, nine: &Nine, k: f32, p: f32, pr: Rect, reach: i32) -> Vec<u8> {
    let (bw, bh) = (pr.w + 2 * reach, pr.h + 2 * reach);
    let mut out = vec![0u8; (bw * bh * 4) as usize];
    let [sl, st, sr, sb] = nine.slice;
    let (sw, sh) = (img.w - sl - sr, img.h - st - sb);
    if nine.center != NineFill::None {
        let tile = nine.center == NineFill::Tile;
        over(&mut out, bw, &fit_cell(img, (sl, st, sw, sh), (pr.w, pr.h), tile, tile, k), reach, reach);
    }
    let thick = |s: i32| ((s as f32 * k).round() as i32).max(1);
    // The half width of the top band's drawn rows, in physical px.
    let drawn: Vec<i32> = (0..st).filter(|y| img.px[((y * img.w + sl) * 4 + 3) as usize] > 0).collect();
    let run = drawn.first().zip(drawn.last()).map_or(0, |(a, b)| (((b - a + 1) as f32 * k / 2.0).ceil()) as i32);
    let tile = nine.edges == NineFill::Tile;
    // A tiled band is art like the corners it joins, so it takes their `sharpen`; a stretched one is a plain line.
    let band = |src: (i32, i32, i32, i32), dst: (i32, i32), tile_x: bool, tile_y: bool| {
        let mut cell = fit_cell(img, src, dst, tile_x, tile_y, k);
        if tile {
            sharpen(&mut cell, nine.sharpen);
        }
        cell
    };
    let [ax, ay] = nine.corner_anchor.unwrap_or([sl / 2, st / 2]);
    let (fx, fy) = ((ax as f32 * p).round() as i32, (ay as f32 * p).round() as i32);
    let (kw, kh) = (sl.min(sr), st.min(sb));
    let (cw, ch) = (((sized_corner(nine, kw) as f32 * p).round() as i32).max(1), ((sized_corner(nine, kh) as f32 * p).round() as i32).max(1));
    // Where a band starts, from the corner point: a tiled band starts where its corner ends, since a soft row drawn twice under
    // the corner would read as a step there; a plain line closes under the corner.
    let (from_x, from_y) = if tile { (cw - fx, ch - fy) } else { (-run, -run) };
    let (lw, lh) = ((pr.w - 2 * from_x).max(0), (pr.h - 2 * from_y).max(0));
    let (tt, tb, tl, tr) = (thick(st), thick(sb), thick(sl), thick(sr));
    if !tile && (nine.line_width.is_some() || nine.line_color.is_some()) {
        // The sidecar's line for the art's: the art's own width and colour wherever the sidecar does not say.
        let art = nine_art_line(img, nine);
        let wide = nine.line_width.or(art.map(|a| a.0)).unwrap_or(0.0);
        let thick = ((wide * k * nine.scale.max(1) as f32).round() as i32).max(1);
        let colour = nine.line_color.as_ref().map(|c| c.rgba).or(art.map(|a| a.1)).unwrap_or([0, 0, 0, 0]);
        let solid = |w: i32, h: i32| Image { w, h, px: colour.repeat((w * h) as usize) };
        let lap = thick - thick / 2;
        let (long, tall) = (pr.w + 2 * lap, pr.h + 2 * lap);
        over(&mut out, bw, &solid(long, thick), reach - lap, reach - thick / 2);
        over(&mut out, bw, &solid(long, thick), reach - lap, reach + pr.h - lap);
        over(&mut out, bw, &solid(thick, tall), reach - thick / 2, reach - lap);
        over(&mut out, bw, &solid(thick, tall), reach + pr.w - lap, reach - lap);
    } else {
        over(&mut out, bw, &band((sl, 0, sw, st), (lw, tt), tile, false), reach + from_x, reach - tt / 2);
        over(&mut out, bw, &band((sl, img.h - sb, sw, sb), (lw, tb), tile, false), reach + from_x, reach + pr.h - (tb - tb / 2));
        over(&mut out, bw, &band((0, st, sl, sh), (tl, lh), false, tile), reach - tl / 2, reach + from_y);
        over(&mut out, bw, &band((img.w - sr, st, sr, sh), (tr, lh), false, tile), reach + pr.w - (tr - tr / 2), reach + from_y);
    }
    for (mx, my) in [(false, false), (true, false), (false, true), (true, true)] {
        let (sx, sy) = (if mx { img.w - kw } else { 0 }, if my { img.h - kh } else { 0 });
        let mut corner = mirrored(&sized_tips(mirrored(&img.crop(sx, sy, kw, kh), mx, my), nine).downsample(cw, ch), mx, my);
        sharpen(&mut corner, nine.sharpen);
        let x = if mx { reach + pr.w + fx - corner.w } else { reach - fx };
        let y = if my { reach + pr.h + fy - corner.h } else { reach - fy };
        over(&mut out, bw, &corner, x, y);
    }
    out
}

pub struct Painter<'a> {
    pub c: &'a mut Canvas,
    pub v: View,
    pub fonts: &'a mut Fonts,
}

fn mix(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], ((c[3] as f32) * a.clamp(0.0, 1.0)).round() as u8]
}

impl<'a> Painter<'a> {
    /// The physical glyph size of logical `size` in `face` (the text size and the face's size factor applied).
    fn px(&self, face: Typeface, size: f32) -> u32 {
        (size * self.fonts.scale * self.fonts.face_scale(face) * self.v.s).round().max(6.0) as u32
    }

    /// The logical height of a line of text of `size` (at the current text size), for centering text in a box.
    pub fn line_h(&self, size: f32) -> f32 {
        self.line_h_face(Typeface::Body, size)
    }

    /// `line_h` in `face` (its size factor applied).
    pub fn line_h_face(&self, face: Typeface, size: f32) -> f32 {
        size * 1.28 * self.fonts.scale * self.fonts.face_scale(face)
    }

    /// Logical width of `s` in a font of logical `size`.
    pub fn text_w(&mut self, s: &str, size: f32) -> f32 {
        self.text_w_face(Typeface::Body, s, size)
    }

    /// `text_w` in `face`.
    pub fn text_w_face(&mut self, face: Typeface, s: &str, size: f32) -> f32 {
        let px = self.px(face, size);
        self.fonts.width_px_face(face, s, px) / self.v.s
    }

    pub fn fill(&mut self, r: Rect, col: Rgba) {
        let pr = self.v.rect(r);
        self.c.fill(pr, col);
        self.cover(r, col[3]);
    }

    /// TEXTFIT1: marks the audited texts so far that `r` (filled at `alpha`) hides.
    pub(crate) fn cover(&mut self, r: Rect, alpha: u8) {
        let Some(a) = self.fonts.audit.as_mut().filter(|_| alpha >= COVER_ALPHA) else { return };
        let (x0, y0, x1, y1) = (r.x as f32, r.y as f32, (r.x + r.w) as f32, (r.y + r.h) as f32);
        for t in a.iter_mut().filter(|t| !t.covered) {
            let mid = t.y + t.h / 2.0;
            if mid < y0 || mid > y1 {
                continue;
            }
            let shared = (t.x + t.w).min(x1) - t.x.max(x0);
            t.covered = (t.x + 1.0 >= x0 && t.x + 1.0 <= x1) || shared > t.w / 2.0;
        }
    }

    /// TEXTFIT2: runs `f` with its texts marked transient (floats that rise and fade over the board): they draw as
    /// usual, but the text audit ignores them. The flag is restored after `f`.
    pub fn transient<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let was = std::mem::replace(&mut self.fonts.transient, true);
        let out = f(self);
        self.fonts.transient = was;
        out
    }

    /// Draws only inside the logical rect `r` from now on (None: everywhere).
    pub fn clip(&mut self, r: Option<Rect>) {
        self.c.clip = r.map(|r| self.v.rect(r));
    }

    /// FX1-E: moves everything drawn after it by whole logical px (adds `d x scale` to the view offset, a camera
    /// shake's world layer); `shift(-dx, -dy)` undoes it. A clip set before the shift stays where it was (it is held in
    /// physical px), so a shaken board stays inside its panel; widget rects and hit tests never move. The text audit keeps the
    /// unshifted places (a shake is a few px for a moment).
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.v.ox += dx as f32 * self.v.s;
        self.v.oy += dy as f32 * self.v.s;
    }

    /// Rounded panel with a 1-physical-pixel border; corner radius in logical units.
    pub fn panel(&mut self, r: Rect, fill: Rgba, border: Rgba, radius: f32) {
        let pr = self.v.rect(r);
        let rad = (radius * self.v.s).max(0.0);
        let (hw, hh) = (pr.w as f32 / 2.0, pr.h as f32 / 2.0);
        let rad = rad.min(hw).min(hh);
        let (cx, cy) = (pr.x as f32 + hw, pr.y as f32 + hh);
        for y in pr.y..pr.y + pr.h {
            for x in pr.x..pr.x + pr.w {
                // Rounded-box signed distance (negative inside).
                let qx = (x as f32 + 0.5 - cx).abs() - (hw - rad);
                let qy = (y as f32 + 0.5 - cy).abs() - (hh - rad);
                let d = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - rad;
                let cover = (0.5 - d).clamp(0.0, 1.0);
                if cover <= 0.0 {
                    continue;
                }
                let inner = (-0.5 - d).clamp(0.0, 1.0);
                if inner > 0.0 {
                    self.c.blend(x, y, mix(fill, inner));
                }
                let ring = cover - inner;
                if ring > 0.0 {
                    self.c.blend(x, y, mix(border, ring.max(0.0)));
                }
            }
        }
        self.cover(r, fill[3]);
    }

    /// Pixel-art image scaled into logical `r` with nearest sampling.
    pub fn image(&mut self, img: &Image, r: Rect) {
        self.image_alpha(img, r, 255);
    }

    /// STATUSFX1-E: `image` with each pixel's alpha scaled by `a / 255` (rounded; 255 draws exactly as `image`, 0
    /// draws nothing).
    pub fn image_alpha(&mut self, img: &Image, r: Rect, a: u8) {
        let pr = self.v.rect(r);
        if pr.w <= 0 || pr.h <= 0 || a == 0 {
            return;
        }
        for y in 0..pr.h {
            let sy = y * img.h / pr.h;
            for x in 0..pr.w {
                let sx = x * img.w / pr.w;
                let i = ((sy * img.w + sx) * 4) as usize;
                let alpha = ((img.px[i + 3] as u32 * a as u32 + 127) / 255) as u8;
                self.c.blend(pr.x + x, pr.y + y, [img.px[i], img.px[i + 1], img.px[i + 2], alpha]);
            }
        }
    }

    /// Anti-aliased text; (x, y) is the logical top-left of the line box. Returns the logical end x.
    pub fn text(&mut self, x: f32, y: f32, s: &str, size: f32, col: Rgba) -> f32 {
        self.text_face(Typeface::Body, x, y, s, size, col)
    }

    /// `text` in `face` (LOOK1-E1: frame titles and headers in the display face, badge numbers in the mono face).
    pub fn text_face(&mut self, face: Typeface, x: f32, y: f32, s: &str, size: f32, col: Rgba) -> f32 {
        let px = self.px(face, size);
        let ascent = px as f32 * 0.95;
        let mut pen = self.v.ox + x * self.v.s;
        let base = self.v.oy + y * self.v.s + ascent;
        for ch in s.chars() {
            let (m, bitmap) = {
                let g = self.fonts.glyph(face, ch, px);
                (g.0, g.1.clone())
            };
            let gx = (pen + m.xmin as f32).round() as i32;
            let gy = (base - m.height as f32 - m.ymin as f32).round() as i32;
            for row in 0..m.height {
                for colx in 0..m.width {
                    let a = bitmap[row * m.width + colx];
                    if a > 0 {
                        self.c.blend(gx + colx as i32, gy + row as i32, mix(col, a as f32 / 255.0));
                    }
                }
            }
            pen += m.advance_width;
        }
        let end = (pen - self.v.ox) / self.v.s;
        if self.fonts.audit.is_some() && !s.trim().is_empty() {
            let h = self.line_h_face(face, size);
            let clipped = self.c.clip.is_some_and(|c| {
                let (cx0, cy0) = ((c.x as f32 - self.v.ox) / self.v.s, (c.y as f32 - self.v.oy) / self.v.s);
                let (cx1, cy1) = (cx0 + c.w as f32 / self.v.s, cy0 + c.h as f32 / self.v.s);
                x < cx0 - 0.5 || y < cy0 - 0.5 || end > cx1 + 0.5 || y + h > cy1 + 0.5
            });
            if let Some(a) = self.fonts.audit.as_mut() {
                a.push(TextRec { x, y, w: end - x, h, text: s.to_string(), clipped, cut: None, covered: false, transient: self.fonts.transient });
            }
        }
        end
    }

    /// Text shortened with "..." to fit `max_w` logical units.
    pub fn text_fit(&mut self, x: f32, y: f32, s: &str, size: f32, max_w: f32, col: Rgba) {
        let t = self.fit(s, size, max_w);
        self.text(x, y, &t, size, col);
        if t != s {
            self.mark_cut(s);
        }
    }

    /// TEXTFIT1: the text just drawn was `full` shortened (for the audit; drawing that shortens text with `fit`
    /// itself calls this after drawing).
    pub fn mark_cut(&mut self, full: &str) {
        if let Some(last) = self.fonts.audit.as_mut().and_then(|a| a.last_mut()) {
            last.cut = Some(full.to_string());
        }
    }

    pub fn fit(&mut self, s: &str, size: f32, max_w: f32) -> String {
        if self.text_w(s, size) <= max_w {
            return s.to_string();
        }
        let mut out: String = s.to_string();
        while !out.is_empty() && self.text_w(&format!("{out}..."), size) > max_w {
            out.pop();
        }
        format!("{}...", out.trim_end())
    }

    pub fn text_right(&mut self, right: f32, y: f32, s: &str, size: f32, col: Rgba) {
        let w = self.text_w(s, size);
        self.text(right - w, y, s, size, col);
    }

    pub fn text_center(&mut self, cx: f32, y: f32, s: &str, size: f32, col: Rgba) {
        self.text_center_face(Typeface::Body, cx, y, s, size, col);
    }

    /// `text_center` in `face`.
    pub fn text_center_face(&mut self, face: Typeface, cx: f32, y: f32, s: &str, size: f32, col: Rgba) {
        let w = self.text_w_face(face, s, size);
        self.text_face(face, cx - w / 2.0, y, s, size, col);
    }

    /// LOOK1-E1 (spec LOOK1 3.5): the nine-slice frame `img` cut by `nine` into logical `r`. The corners are drawn at
    /// `slice / scale` logical px (nearest sampling), the edges and the centre stretched or tiled between them;
    /// `center: none` leaves the centre undrawn (the caller fills it with the role's colour first). A rect smaller
    /// than two corners shrinks the corners to half the rect each way, so they never overlap. With `nine.reduce` a frame
    /// whose art is finer than the window (`scale` above the view's scale) is averaged down instead of sampled, each
    /// cell apart so the slices stay exact, and cached per frame and physical size (`reduced_nine`); at a view scale of
    /// `scale` or more it samples as any frame. `nine.straddle` centres the frame on `r`'s boundary (`compose_straddle`),
    /// drawing past `r`, under the canvas's clip and bounds, and only draws: `r` stays the panel's.
    pub fn nine(&mut self, img: &Image, nine: &Nine, r: Rect) {
        let pr = self.v.rect(r);
        if pr.w <= 0 || pr.h <= 0 || img.w <= 0 || img.h <= 0 {
            return;
        }
        let [sl, st, sr, sb] = nine.slice;
        // Physical px per art px: a sidecar's `scale` is art px per old unit on every grid.
        let k = self.v.old_scale() / nine.scale.max(1) as f32;
        if nine.straddle {
            let p = corner_scale(nine, k, pr);
            let reach = straddle_over(nine, k, p);
            reduced_nine(img, nine, k, pr, || compose_straddle(img, nine, k, p, pr, reach), |px| {
                let bw = pr.w + 2 * reach;
                for y in 0..pr.h + 2 * reach {
                    for x in 0..bw {
                        let i = ((y * bw + x) * 4) as usize;
                        self.c.blend(pr.x - reach + x, pr.y - reach + y, [px[i], px[i + 1], px[i + 2], px[i + 3]]);
                    }
                }
            });
            if nine.center != NineFill::None {
                self.cover(r, 255);
            }
            return;
        }
        let side = |a: i32, b: i32, room: i32| -> (i32, i32) {
            let (pa, pb) = (((a as f32) * k * nine.corner_size).round() as i32, ((b as f32) * k * nine.corner_size).round() as i32);
            if pa + pb > room {
                (room / 2, room - room / 2)
            } else {
                (pa, pb)
            }
        };
        let (dl, dr) = side(sl, sr, pr.w);
        let (dt, db) = side(st, sb, pr.h);
        let (sw, sh) = (img.w - sl - sr, img.h - st - sb);
        let (dw, dh) = (pr.w - dl - dr, pr.h - dt - db);
        // Source spans (x, w) and destination spans (x, w) of the three columns and rows.
        let cols = [(0, sl, pr.x, dl), (sl, sw, pr.x + dl, dw), (img.w - sr, sr, pr.x + pr.w - dr, dr)];
        let rows = [(0, st, pr.y, dt), (st, sh, pr.y + dt, dh), (img.h - sb, sb, pr.y + pr.h - db, db)];
        if nine.reduce && k < 1.0 {
            reduced_nine(img, nine, k, pr, || compose_reduced(img, nine, k, pr, &cols, &rows), |px| {
                for y in 0..pr.h {
                    for x in 0..pr.w {
                        let i = ((y * pr.w + x) * 4) as usize;
                        self.c.blend(pr.x + x, pr.y + y, [px[i], px[i + 1], px[i + 2], px[i + 3]]);
                    }
                }
            });
            if nine.center != NineFill::None {
                self.cover(r, 255);
            }
            return;
        }
        for (j, &(sy, src_h, dy, dst_h)) in rows.iter().enumerate() {
            for (i, &(sx, src_w, dx, dst_w)) in cols.iter().enumerate() {
                let (mid_col, mid_row) = (i == 1, j == 1);
                let fill = if mid_col && mid_row { nine.center } else if mid_col || mid_row { nine.edges } else { NineFill::Stretch };
                if fill == NineFill::None || src_w <= 0 || src_h <= 0 || dst_w <= 0 || dst_h <= 0 {
                    continue;
                }
                // Stretch maps the destination span onto the whole source span (a shrunk corner is sampled down);
                // tile repeats the source at its own size along the middle column or row.
                let tile_x = fill == NineFill::Tile && mid_col;
                let tile_y = fill == NineFill::Tile && mid_row;
                for y in 0..dst_h {
                    let oy = if tile_y { ((y as f32 / k) as i32).rem_euclid(src_h) } else { y * src_h / dst_h };
                    for x in 0..dst_w {
                        let ox = if tile_x { ((x as f32 / k) as i32).rem_euclid(src_w) } else { x * src_w / dst_w };
                        let at = (((sy + oy) * img.w + sx + ox) * 4) as usize;
                        self.c.blend(dx + x, dy + y, [img.px[at], img.px[at + 1], img.px[at + 2], img.px[at + 3]]);
                    }
                }
            }
        }
        if nine.center != NineFill::None {
            self.cover(r, 255);
        }
    }

    /// LOOK1-E1: a themed panel: `frame` (a frame role's file, `Theme::frame`) cut nine ways over `r`, its centre filled
    /// with `fill` first when the sidecar says `center: none`; without a frame the vector `panel(r, fill, edge,
    /// radius)` exactly as before.
    pub fn skin(&mut self, frame: Option<&Frame>, r: Rect, fill: Rgba, edge: Rgba, radius: f32) {
        let Some(f) = frame else { return self.panel(r, fill, edge, radius) };
        if f.nine.center == NineFill::None && f.nine.straddle {
            self.fill(r, fill);
        } else if f.nine.center == NineFill::None {
            let k = f.nine.scale.max(1);
            let [l, t, rr, b] = f.nine.slice.map(|s| self.v.u(s / k));
            self.fill(Rect::new(r.x + l, r.y + t, (r.w - l - rr).max(0), (r.h - t - b).max(0)), fill);
        }
        self.nine(&f.img, &f.nine, r);
    }

    /// Word wrap to `max_w` logical units (a word wider than a line breaks across lines, wherever it falls).
    pub fn wrap(&mut self, s: &str, size: f32, max_w: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut cur = String::new();
        for word in s.split(' ') {
            let candidate = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
            if self.text_w(&candidate, size) <= max_w || cur.is_empty() {
                cur = candidate;
            } else {
                lines.push(std::mem::take(&mut cur));
                cur = word.to_string();
            }
            while self.text_w(&cur, size) > max_w && cur.chars().count() > 1 {
                let mut head = cur.clone();
                let mut tail = String::new();
                while self.text_w(&head, size) > max_w && head.chars().count() > 1 {
                    tail.insert(0, head.pop().unwrap());
                }
                lines.push(head);
                cur = tail;
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        lines
    }

    /// A line from (x0, y0) to (x1, y1) with an arrowhead of `head` logical units at the end (`dash` > 0 dashes the
    /// shaft, the head stays solid).
    #[allow(clippy::too_many_arguments)]
    pub fn arrow(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, col: Rgba, dash: f32, head: f32) {
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.5 {
            return;
        }
        let (ux, uy) = (dx / len, dy / len);
        self.line(x0, y0, x1, y1, col, dash);
        for side in [-1.0f32, 1.0] {
            // The head's two strokes at +-30 degrees back from the tip.
            let (c, s) = (0.866f32, 0.5 * side);
            let (bx, by) = (-(ux * c - uy * s), -(uy * c + ux * s));
            self.line(x1, y1, x1 + bx * head, y1 + by * head, col, 0.0);
        }
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, col: Rgba, dash: f32) {
        let (a, b, c, d) = (self.v.x(x0), self.v.y(y0), self.v.x(x1), self.v.y(y1));
        let dash_px = (dash * self.v.s).round() as i32;
        let t = (self.v.old_scale() * 0.6).round().max(1.0) as i32;
        for o in 0..t {
            self.c.line(a + o, b, c + o, d, col, dash_px);
        }
    }

    /// A filled, anti-aliased ellipse centred on logical (`cx`, `cy`) with radii `rx` and `ry` (a unit's foot shadow).
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, col: Rgba) {
        let (x, y, prx, pry) = (self.v.ox + cx * self.v.s, self.v.oy + cy * self.v.s, (rx * self.v.s).max(0.5), (ry * self.v.s).max(0.5));
        let (x0, y0, x1, y1) = ((x - prx - 1.0).floor() as i32, (y - pry - 1.0).floor() as i32, (x + prx + 1.0).ceil() as i32, (y + pry + 1.0).ceil() as i32);
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (ux, uy) = (px as f32 + 0.5 - x, py as f32 + 0.5 - y);
                let k = ((ux / prx).powi(2) + (uy / pry).powi(2)).sqrt();
                // Distance to the edge in physical pixels: the implicit function over its gradient's length.
                let grad = ((ux / (prx * prx)).powi(2) + (uy / (pry * pry)).powi(2)).sqrt() / k.max(1e-6);
                let d = if k <= 1e-6 { -prx.min(pry) } else { (k - 1.0) / grad.max(1e-6) };
                let cover = (0.5 - d).clamp(0.0, 1.0);
                if cover > 0.0 {
                    self.c.blend(px, py, mix(col, cover));
                }
            }
        }
    }

    /// A 1 px (old unit) anti-aliased ellipse outline centred on logical (`cx`, `cy`) with radii `rx` and `ry`, no fill
    /// (STAGE1-E: a unit's foot ring). Only pixels within half the stroke plus half a pixel of the edge are touched.
    pub fn ellipse_ring(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, col: Rgba) {
        let (x, y, prx, pry) = (self.v.ox + cx * self.v.s, self.v.oy + cy * self.v.s, (rx * self.v.s).max(0.5), (ry * self.v.s).max(0.5));
        let t = self.v.old_scale().max(1.0);
        let pad = t / 2.0 + 1.0;
        let (x0, y0, x1, y1) = ((x - prx - pad).floor() as i32, (y - pry - pad).floor() as i32, (x + prx + pad).ceil() as i32, (y + pry + pad).ceil() as i32);
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (ux, uy) = (px as f32 + 0.5 - x, py as f32 + 0.5 - y);
                let k = ((ux / prx).powi(2) + (uy / pry).powi(2)).sqrt();
                // Distance to the edge in physical pixels, measured as `ellipse` does.
                let grad = ((ux / (prx * prx)).powi(2) + (uy / (pry * pry)).powi(2)).sqrt() / k.max(1e-6);
                let d = if k <= 1e-6 { prx.min(pry) } else { ((k - 1.0) / grad.max(1e-6)).abs() };
                let cover = (t / 2.0 + 0.5 - d).clamp(0.0, 1.0);
                if cover > 0.0 {
                    self.c.blend(px, py, mix(col, cover));
                }
            }
        }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, col: Rgba) {
        let (x, y, pr) = (self.v.ox + cx * self.v.s, self.v.oy + cy * self.v.s, r * self.v.s);
        let t = (self.v.old_scale() * 0.5).max(1.0);
        let (x0, y0, x1, y1) = ((x - pr - 2.0) as i32, (y - pr - 2.0) as i32, (x + pr + 2.0) as i32, (y + pr + 2.0) as i32);
        for py in y0..=y1 {
            for px in x0..=x1 {
                let d = ((px as f32 + 0.5 - x).powi(2) + (py as f32 + 0.5 - y).powi(2)).sqrt();
                let cover = (t / 2.0 + 0.5 - (d - pr).abs()).clamp(0.0, 1.0);
                if cover > 0.0 {
                    self.c.blend(px, py, mix(col, cover));
                }
            }
        }
    }
}

/// A run of text in one style (`parse_markup`).
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub col: Rgba,
    pub bold: bool,
}

fn hex_colour(hex: &str) -> Option<Rgba> {
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let ch = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([ch(0)?, ch(2)?, ch(4)?, 255])
}

fn push_span(out: &mut Vec<Span>, cur: &mut String, col: Rgba, bold: bool) {
    if !cur.is_empty() {
        out.push(Span { text: std::mem::take(cur), col, bold });
    }
}

/// Markup into spans: `[color=#rrggbb]..[/color]` and `[b]..[/b]`, nested; a tag it does not know (or a malformed one)
/// stays as text, and `\n` stays in the text. `col` is the colour outside any tag.
pub fn parse_markup(s: &str, col: Rgba) -> Vec<Span> {
    let (mut out, mut cur, mut colours, mut bold) = (Vec::new(), String::new(), vec![col], 0usize);
    let mut rest = s;
    while let Some(ch) = rest.chars().next() {
        if ch == '[' {
            if let Some(end) = rest.find(']') {
                let tag = &rest[1..end];
                let top = *colours.last().unwrap();
                let handled = if let Some(c) = tag.strip_prefix("color=#").and_then(hex_colour) {
                    push_span(&mut out, &mut cur, top, bold > 0);
                    colours.push(c);
                    true
                } else if tag == "/color" && colours.len() > 1 {
                    push_span(&mut out, &mut cur, top, bold > 0);
                    colours.pop();
                    true
                } else if tag == "b" || (tag == "/b" && bold > 0) {
                    push_span(&mut out, &mut cur, top, bold > 0);
                    if tag == "b" {
                        bold += 1;
                    } else {
                        bold -= 1;
                    }
                    true
                } else {
                    false
                };
                if handled {
                    rest = &rest[end + 1..];
                    continue;
                }
            }
        }
        cur.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    push_span(&mut out, &mut cur, *colours.last().unwrap(), bold > 0);
    out
}

/// The text of markup without its tags.
pub fn strip_markup(s: &str) -> String {
    parse_markup(s, [0, 0, 0, 255]).into_iter().map(|sp| sp.text).collect()
}

impl Painter<'_> {
    /// Markup laid out in lines of at most `max_w` (words wrap; `\n` breaks a line).
    pub fn rich_lines(&mut self, markup: &str, size: f32, max_w: f32, col: Rgba) -> Vec<Vec<Span>> {
        let mut lines: Vec<Vec<Span>> = vec![Vec::new()];
        let mut width = 0.0;
        for sp in parse_markup(markup, col) {
            // Words keep their trailing space; a newline is its own token.
            let mut tokens: Vec<String> = Vec::new();
            for (i, part) in sp.text.split('\n').enumerate() {
                if i > 0 {
                    tokens.push("\n".into());
                }
                tokens.extend(part.split_inclusive(' ').map(String::from));
            }
            for tok in tokens {
                if tok == "\n" {
                    lines.push(Vec::new());
                    width = 0.0;
                    continue;
                }
                let w = self.text_w(&tok, size);
                if width > 0.0 && width + w > max_w {
                    lines.push(Vec::new());
                    width = 0.0;
                }
                let tok = if width == 0.0 { tok.trim_start().to_string() } else { tok };
                width += self.text_w(&tok, size);
                let line = lines.last_mut().unwrap();
                match line.last_mut() {
                    Some(last) if last.col == sp.col && last.bold == sp.bold => last.text.push_str(&tok),
                    _ => line.push(Span { text: tok, col: sp.col, bold: sp.bold }),
                }
            }
        }
        lines
    }

    /// Draws markup at (x, y) wrapped to `max_w`; bold is drawn twice a pixel apart. Returns the height drawn.
    pub fn rich(&mut self, x: f32, y: f32, max_w: f32, markup: &str, size: f32, col: Rgba) -> f32 {
        let lh = self.line_h(size);
        let lines = self.rich_lines(markup, size, max_w, col);
        for (i, line) in lines.iter().enumerate() {
            let mut pen = x;
            let ly = y + i as f32 * lh;
            for sp in line {
                if sp.bold {
                    self.text(pen + 1.0 / self.v.s, ly, &sp.text, size, sp.col);
                }
                pen = self.text(pen, ly, &sp.text, size, sp.col);
            }
        }
        lines.len() as f32 * lh
    }

    /// The height `rich` would draw.
    pub fn rich_height(&mut self, markup: &str, size: f32, max_w: f32) -> f32 {
        self.rich_lines(markup, size, max_w, [0, 0, 0, 255]).len() as f32 * self.line_h(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Rgba = [255, 255, 255, 255];

    #[test]
    fn markup_becomes_styled_spans_and_unknown_tags_stay_text() {
        let s = parse_markup("a [color=#ff0000]red [b]bold[/b][/color] [x] z", W);
        assert_eq!(s.iter().map(|sp| (sp.text.as_str(), sp.col, sp.bold)).collect::<Vec<_>>(), [
            ("a ", W, false),
            ("red ", [255, 0, 0, 255], false),
            ("bold", [255, 0, 0, 255], true),
            (" [x] z", W, false),
        ]);
        assert_eq!(strip_markup("[b]RESULT: 7[/b] [color=#zzzzzz]x"), "RESULT: 7 [color=#zzzzzz]x");
        assert_eq!(parse_markup("[/color][/b]x", W), [Span { text: "[/color][/b]x".into(), col: W, bold: false }], "closing tags without an opener are text");
    }

    #[test]
    fn rich_text_wraps_to_the_width_and_breaks_on_newlines() {
        let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        let mut canvas = crate::canvas::Canvas::new(640, 360);
        let mut fonts = Fonts::new(&font);
        let mut p = Painter { c: &mut canvas, v: View::fit(640, 360), fonts: &mut fonts };
        let text = "one two three four five six seven eight nine ten\n[b]last[/b]";
        let wide = p.rich_lines(text, 7.0, 600.0, W);
        assert_eq!(wide.len(), 2);
        assert!(wide[1][0].bold);
        let narrow = p.rich_lines(text, 7.0, 40.0, W);
        assert!(narrow.len() > 3);
        assert!(narrow.iter().all(|l| l.first().is_none_or(|s| !s.text.starts_with(' '))), "no leading spaces");
        assert_eq!(p.rich_height(text, 7.0, 600.0), 2.0 * p.line_h(7.0));
    }

    /// TEXTFIT1: a word wider than the line breaks across lines after another word too (raw JSON in the forges).
    #[test]
    fn wrap_breaks_a_long_word_wherever_it_falls() {
        let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        let mut canvas = crate::canvas::Canvas::new(640, 360);
        let mut fonts = Fonts::new(&font);
        let mut p = Painter { c: &mut canvas, v: View::fit(640, 360), fonts: &mut fonts };
        let long = "{\"id\":\"signpost\",\"kind\":\"exit\",\"anchor\":\"road\",\"to\":\"world_map\"}";
        for s in [long.to_string(), format!("[ {long}"), format!("short words {long} end")] {
            let lines = p.wrap(&s, 7.0, 80.0);
            assert!(lines.len() > 2, "{lines:?}");
            assert!(lines.iter().all(|l| p.text_w(l, 7.0) <= 80.0), "every line fits: {lines:?}");
            assert_eq!(lines.concat().replace(' ', ""), s.replace(' ', ""), "nothing is lost");
        }
    }
    /// TEXTFIT1: a near-opaque panel drawn after a text hides it from the audit; a see-through one, a panel drawn
    /// before the text (a button behind its label) or one past the text does not.
    #[test]
    fn a_later_opaque_panel_covers_the_audited_text_beneath_it() {
        let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        let mut canvas = crate::canvas::Canvas::new(640, 360);
        let mut fonts = Fonts::new(&font);
        fonts.audit = Some(Vec::new());
        let mut p = Painter { c: &mut canvas, v: View::fit(640, 360), fonts: &mut fonts };
        p.panel(Rect::new(8, 98, 120, 14), [20, 20, 20, 255], W, 2.0);
        for (i, s) in ["Under the dialog", "Under the dim", "Beside the dialog", "On its button"].iter().enumerate() {
            p.text(if i == 2 { 400.0 } else { 10.0 }, 10.0 + 30.0 * i as f32, s, 8.0, W);
        }
        p.fill(Rect::new(0, 0, 200, 20), [20, 20, 20, 240]);
        p.fill(Rect::new(0, 30, 200, 20), [0, 0, 0, 120]);
        p.panel(Rect::new(0, 60, 200, 20), [20, 20, 20, 255], W, 4.0);
        let covered: Vec<bool> = p.fonts.audit.as_ref().unwrap().iter().map(|t| t.covered).collect();
        assert_eq!(covered, [true, false, false, false]);
    }
}
