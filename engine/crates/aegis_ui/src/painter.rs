//! Two-layer drawing at the window's real resolution. Layout is in logical units (a 640x360 design canvas); `View`
//! maps them to physical pixels. The battlefield is pixel art scaled with nearest sampling; panels and text are drawn
//! smooth at full resolution (anti-aliased Open Sans, rounded panels), like the Godot HUD.

use crate::canvas::{Canvas, Image, Rect, Rgba};
use std::collections::HashMap;

pub const LW: f32 = 640.0;
pub const LH: f32 = 360.0;

/// Logical -> physical mapping (uniform scale, letterboxed and centred).
#[derive(Debug, Clone, Copy)]
pub struct View {
    pub s: f32,
    pub ox: f32,
    pub oy: f32,
}

impl View {
    pub fn fit(pw: u32, ph: u32) -> Self {
        let s = (pw as f32 / LW).min(ph as f32 / LH).max(0.25);
        Self { s, ox: ((pw as f32 - LW * s) / 2.0).floor(), oy: ((ph as f32 - LH * s) / 2.0).floor() }
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
}

/// Rasterized glyphs per (char, physical px size).
pub struct Fonts {
    font: fontdue::Font,
    cache: HashMap<(char, u32), (fontdue::Metrics, Vec<u8>)>,
    /// The text size factor (settings `text_scale`: 0.85 Small .. 1.3 Extra large); every text size is multiplied by it.
    pub scale: f32,
}

impl Fonts {
    pub fn new(ttf: &[u8]) -> Self {
        let font = fontdue::Font::from_bytes(ttf, fontdue::FontSettings::default()).expect("valid TTF");
        Self { font, cache: HashMap::new(), scale: 1.0 }
    }

    fn glyph(&mut self, c: char, px: u32) -> &(fontdue::Metrics, Vec<u8>) {
        let font = &self.font;
        self.cache.entry((c, px)).or_insert_with(|| font.rasterize(c, px as f32))
    }

    /// Physical advance width of `s` at `px`.
    pub fn width_px(&mut self, s: &str, px: u32) -> f32 {
        s.chars().map(|c| self.glyph(c, px).0.advance_width).sum()
    }
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
    fn px(&self, size: f32) -> u32 {
        (size * self.fonts.scale * self.v.s).round().max(6.0) as u32
    }

    /// The logical height of a line of text of `size` (at the current text size), for centering text in a box.
    pub fn line_h(&self, size: f32) -> f32 {
        size * 1.28 * self.fonts.scale
    }

    /// Logical width of `s` in a font of logical `size`.
    pub fn text_w(&mut self, s: &str, size: f32) -> f32 {
        let px = self.px(size);
        self.fonts.width_px(s, px) / self.v.s
    }

    pub fn fill(&mut self, r: Rect, col: Rgba) {
        let pr = self.v.rect(r);
        self.c.fill(pr, col);
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
    }

    /// Pixel-art image scaled into logical `r` with nearest sampling.
    pub fn image(&mut self, img: &Image, r: Rect) {
        let pr = self.v.rect(r);
        if pr.w <= 0 || pr.h <= 0 {
            return;
        }
        for y in 0..pr.h {
            let sy = y * img.h / pr.h;
            for x in 0..pr.w {
                let sx = x * img.w / pr.w;
                let i = ((sy * img.w + sx) * 4) as usize;
                self.c.blend(pr.x + x, pr.y + y, [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]);
            }
        }
    }

    /// Anti-aliased text; (x, y) is the logical top-left of the line box. Returns the logical end x.
    pub fn text(&mut self, x: f32, y: f32, s: &str, size: f32, col: Rgba) -> f32 {
        let px = self.px(size);
        let ascent = px as f32 * 0.95;
        let mut pen = self.v.ox + x * self.v.s;
        let base = self.v.oy + y * self.v.s + ascent;
        for ch in s.chars() {
            let (m, bitmap) = {
                let g = self.fonts.glyph(ch, px);
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
        (pen - self.v.ox) / self.v.s
    }

    /// Text shortened with "..." to fit `max_w` logical units.
    pub fn text_fit(&mut self, x: f32, y: f32, s: &str, size: f32, max_w: f32, col: Rgba) {
        let t = self.fit(s, size, max_w);
        self.text(x, y, &t, size, col);
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
        let w = self.text_w(s, size);
        self.text(cx - w / 2.0, y, s, size, col);
    }

    /// Word wrap to `max_w` logical units (long words are cut).
    pub fn wrap(&mut self, s: &str, size: f32, max_w: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut cur = String::new();
        for word in s.split(' ') {
            let candidate = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
            if self.text_w(&candidate, size) <= max_w || cur.is_empty() {
                cur = candidate;
                while self.text_w(&cur, size) > max_w && cur.chars().count() > 1 {
                    let mut head = cur.clone();
                    let mut tail = String::new();
                    while self.text_w(&head, size) > max_w && head.chars().count() > 1 {
                        tail.insert(0, head.pop().unwrap());
                    }
                    lines.push(head);
                    cur = tail;
                }
            } else {
                lines.push(std::mem::take(&mut cur));
                cur = word.to_string();
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        lines
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, col: Rgba, dash: f32) {
        let (a, b, c, d) = (self.v.x(x0), self.v.y(y0), self.v.x(x1), self.v.y(y1));
        let dash_px = (dash * self.v.s).round() as i32;
        let t = (self.v.s * 0.6).round().max(1.0) as i32;
        for o in 0..t {
            self.c.line(a + o, b, c + o, d, col, dash_px);
        }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, col: Rgba) {
        let (x, y, pr) = (self.v.ox + cx * self.v.s, self.v.oy + cy * self.v.s, r * self.v.s);
        let t = (self.v.s * 0.5).max(1.0);
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
