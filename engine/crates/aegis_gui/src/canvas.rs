//! Software 2D drawing into an RGBA framebuffer at the logical resolution (integer-scaled to the window by pixels).

use crate::font;

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
}

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
}

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h, px: vec![0; (w * h * 4) as usize] }
    }

    pub fn clear(&mut self, c: Rgba) {
        for p in self.px.chunks_exact_mut(4) {
            p.copy_from_slice(&c);
        }
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Rgba) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h || c[3] == 0 {
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

    pub fn outline(&mut self, r: Rect, c: Rgba) {
        for x in r.x..r.x + r.w {
            self.blend(x, r.y, c);
            self.blend(x, r.y + r.h - 1, c);
        }
        for y in r.y + 1..r.y + r.h - 1 {
            self.blend(r.x, y, c);
            self.blend(r.x + r.w - 1, y, c);
        }
    }

    /// A panel: fill plus a 1 px border.
    pub fn panel(&mut self, r: Rect, fill: Rgba, border: Rgba) {
        self.fill(r, fill);
        self.outline(r, border);
    }

    pub fn image(&mut self, img: &Image, x: i32, y: i32) {
        for sy in 0..img.h {
            for sx in 0..img.w {
                let i = ((sy * img.w + sx) * 4) as usize;
                self.blend(x + sx, y + sy, [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]);
            }
        }
    }

    pub fn text(&mut self, x: i32, y: i32, s: &str, c: Rgba) -> i32 {
        let mut cx = x;
        for ch in s.chars() {
            let g = font::glyph(ch);
            for (row, bits) in g.iter().enumerate() {
                for col in 0..font::GLYPH_W {
                    if bits & (1 << (font::GLYPH_W - 1 - col)) != 0 {
                        self.blend(cx + col, y + row as i32, c);
                    }
                }
            }
            cx += font::ADVANCE;
        }
        cx
    }

    /// Text clipped to `max_w` pixels; overflow ends in "..".
    pub fn text_fit(&mut self, x: i32, y: i32, s: &str, max_w: i32, c: Rgba) {
        self.text(x, y, &fit(s, max_w), c);
    }

    pub fn text_right(&mut self, right: i32, y: i32, s: &str, c: Rgba) {
        self.text(right - font::text_width(s), y, s, c);
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

    pub fn circle(&mut self, cx: i32, cy: i32, r: i32, c: Rgba) {
        let (mut x, mut y, mut err) = (r, 0, 1 - r);
        while x >= y {
            for (a, b) in [(x, y), (y, x), (-y, x), (-x, y), (-x, -y), (-y, -x), (y, -x), (x, -y)] {
                self.blend(cx + a, cy + b, c);
            }
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }
}

/// `s` shortened to fit `max_w` pixels.
pub fn fit(s: &str, max_w: i32) -> String {
    let max_chars = (max_w / font::ADVANCE).max(0) as usize;
    let n = s.chars().count();
    if n <= max_chars {
        return s.to_string();
    }
    let keep = max_chars.saturating_sub(2);
    let mut out: String = s.chars().take(keep).collect();
    out.push_str("..");
    out
}
