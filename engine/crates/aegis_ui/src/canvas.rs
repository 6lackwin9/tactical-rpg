//! Software 2D drawing into an RGBA framebuffer at the logical resolution (integer-scaled to the window by pixels).

pub type Rgba = [u8; 4];

pub const fn rgb(r: u8, g: u8, b: u8) -> Rgba {
    [r, g, b, 255]
}

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    [r, g, b, a]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
    pub fn inset(&self, d: i32) -> Rect {
        Rect::new(self.x + d, self.y + d, (self.w - 2 * d).max(0), (self.h - 2 * d).max(0))
    }
    /// The overlap of two rects (zero size when they do not meet).
    pub fn intersect(&self, o: Rect) -> Rect {
        let (x0, y0) = (self.x.max(o.x), self.y.max(o.y));
        let (x1, y1) = ((self.x + self.w).min(o.x + o.w), (self.y + self.h).min(o.y + o.h));
        Rect::new(x0, y0, (x1 - x0).max(0), (y1 - y0).max(0))
    }
}

/// The alpha from which a pixel counts as part of a sprite's silhouette (`Image::outline`).
pub const OUTLINE_ALPHA: u8 = 128;

/// An RGBA image (sprites, tiles).
#[derive(Debug, Clone)]
pub struct Image {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Image {
    /// Box-filter downsample to `w` x `h` (alpha-weighted colour), for placeholder art drawn at pixel scale.
    pub fn downsample(&self, w: i32, h: i32) -> Image {
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let (x0, x1) = (x * self.w / w, ((x + 1) * self.w / w).max(x * self.w / w + 1));
                let (y0, y1) = (y * self.h / h, ((y + 1) * self.h / h).max(y * self.h / h + 1));
                let (mut r, mut g, mut b, mut a, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
                for sy in y0..y1.min(self.h) {
                    for sx in x0..x1.min(self.w) {
                        let i = ((sy * self.w + sx) * 4) as usize;
                        let al = self.px[i + 3] as u32;
                        r += self.px[i] as u32 * al;
                        g += self.px[i + 1] as u32 * al;
                        b += self.px[i + 2] as u32 * al;
                        a += al;
                        n += 1;
                    }
                }
                let o = ((y * w + x) * 4) as usize;
                if a > 0 {
                    px[o] = (r / a) as u8;
                    px[o + 1] = (g / a) as u8;
                    px[o + 2] = (b / a) as u8;
                }
                px[o + 3] = (a / n.max(1)) as u8;
            }
        }
        Image { w, h, px }
    }

    /// The `w` x `h` part at (`x`, `y`) (outside the image: transparent).
    pub fn crop(&self, x: i32, y: i32, w: i32, h: i32) -> Image {
        let mut px = vec![0u8; (w.max(0) * h.max(0) * 4) as usize];
        for yy in 0..h {
            for xx in 0..w {
                let (sx, sy) = (x + xx, y + yy);
                if sx < 0 || sy < 0 || sx >= self.w || sy >= self.h {
                    continue;
                }
                let (i, o) = (((sy * self.w + sx) * 4) as usize, ((yy * w + xx) * 4) as usize);
                px[o..o + 4].copy_from_slice(&self.px[i..i + 4]);
            }
        }
        Image { w, h, px }
    }

    /// Each pixel `k` x `k` (nearest neighbour), for pixel art drawn larger.
    pub fn scale_nearest(&self, k: i32) -> Image {
        let k = k.max(1);
        let (w, h) = (self.w * k, self.h * k);
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let (i, o) = ((((y / k) * self.w + x / k) * 4) as usize, ((y * w + x) * 4) as usize);
                px[o..o + 4].copy_from_slice(&self.px[i..i + 4]);
            }
        }
        Image { w, h, px }
    }

    /// Every pixel of exactly a `from` colour (any alpha) takes its `to` colour (ASSETORG2 palettes).
    pub fn swap_colours(&mut self, pairs: &[([u8; 3], [u8; 3])]) {
        for p in self.px.chunks_exact_mut(4) {
            if let Some((_, to)) = pairs.iter().find(|(from, _)| p[..3] == from[..]) {
                p[..3].copy_from_slice(to);
            }
        }
    }

    /// The silhouette outline of `self` (SPRITE1-L): an image one pixel larger on every side ((w + 2) x (h + 2), drawn
    /// one pixel up and left of `self`) holding `col` on every pixel that is not solid (alpha below `OUTLINE_ALPHA`)
    /// but touches a solid one of `self`, sides or corners; the rest is transparent.
    pub fn outline(&self, col: Rgba) -> Image {
        let (w, h) = (self.w + 2, self.h + 2);
        let solid = |x: i32, y: i32| x >= 0 && y >= 0 && x < self.w && y < self.h && self.px[((y * self.w + x) * 4 + 3) as usize] >= OUTLINE_ALPHA;
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                // (x, y) of the outline is (x - 1, y - 1) of self.
                let (sx, sy) = (x - 1, y - 1);
                if solid(sx, sy) {
                    continue;
                }
                let touches = (-1..=1).any(|dy| (-1..=1).any(|dx| (dx != 0 || dy != 0) && solid(sx + dx, sy + dy)));
                if touches {
                    let o = ((y * w + x) * 4) as usize;
                    px[o..o + 4].copy_from_slice(&col);
                }
            }
        }
        Image { w, h, px }
    }

    /// `self` on a transparent `size` x `size` square: centred across, standing on the bottom edge.
    pub fn on_square(&self, size: i32) -> Image {
        let mut px = vec![0u8; (size * size * 4) as usize];
        let (ox, oy) = ((size - self.w) / 2, size - self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                let (dx, dy) = (ox + x, oy + y);
                if dx < 0 || dy < 0 || dx >= size || dy >= size {
                    continue;
                }
                let (i, o) = (((y * self.w + x) * 4) as usize, ((dy * size + dx) * 4) as usize);
                px[o..o + 4].copy_from_slice(&self.px[i..i + 4]);
            }
        }
        Image { w: size, h: size, px }
    }
}

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
    /// Pixels outside this rect are not drawn (a window's body, UIKIT3); None draws everywhere.
    pub clip: Option<Rect>,
    /// An outer limit the drawing code cannot lift (a preview's box, UIGALLERY1): `clip` is set and reset freely by
    /// the screen being drawn, this one only by its owner.
    pub bounds: Option<Rect>,
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h, px: vec![0; (w * h * 4) as usize], clip: None, bounds: None }
    }

    /// Fills the whole canvas (only `bounds` when set: a screen drawn in a preview clears its box).
    pub fn clear(&mut self, c: Rgba) {
        if let Some(b) = self.bounds {
            return self.fill(b, [c[0], c[1], c[2], 255]);
        }
        for p in self.px.chunks_exact_mut(4) {
            p.copy_from_slice(&c);
        }
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Rgba) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h || c[3] == 0 || self.clip.is_some_and(|r| !r.contains(x, y)) || self.bounds.is_some_and(|r| !r.contains(x, y)) {
            return;
        }
        let i = ((y * self.w + x) * 4) as usize;
        if c[3] == 255 {
            self.px[i..i + 4].copy_from_slice(&c);
            return;
        }
        let a = c[3] as u32;
        for k in 0..3 {
            self.px[i + k] = ((c[k] as u32 * a + self.px[i + k] as u32 * (255 - a)) / 255) as u8;
        }
        self.px[i + 3] = 255;
    }

    pub fn fill(&mut self, r: Rect, c: Rgba) {
        for y in r.y.max(0)..(r.y + r.h).min(self.h) {
            for x in r.x.max(0)..(r.x + r.w).min(self.w) {
                self.blend(x, y, c);
            }
        }
    }

    /// Bresenham line; `dash` > 0 draws dash-on / dash-off segments.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgba, dash: i32) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err, mut n) = (x0, y0, dx + dy, 0);
        loop {
            if dash <= 0 || (n / dash) % 2 == 0 {
                self.blend(x, y, c);
            }
            n += 1;
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}

