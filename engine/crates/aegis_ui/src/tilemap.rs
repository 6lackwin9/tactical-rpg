//! WORLD-PRIM1 (ENGINE-9, WP2): a tilemap layer. A `w` x `h` grid of tile values (0: none) drawn through a
//! `WorldCamera`, only the cells on view, each in the screen rect its tile covers (`tile` world pixels square); what a
//! value looks like is the caller's (`draw` hands each cell to a closure; `draw_images` takes an image per value).
//! `corners` gives the dual-grid transition pieces over the layer (`autotile::corner_pieces`, a value counting as
//! upper when the caller says so) and `draw_corners` draws them, each piece centred on its corner.

use crate::autotile::corner_pieces;
use crate::canvas::{Image, Rect};
use crate::painter::Painter;
use crate::world_camera::WorldCamera;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TileLayer {
    pub w: i32,
    pub h: i32,
    /// World pixels per tile side.
    pub tile: i32,
    /// Row by row; 0 is an empty cell.
    pub cells: Vec<u16>,
}

impl TileLayer {
    /// An empty `w` x `h` layer of `tile`-pixel tiles.
    pub fn new(w: i32, h: i32, tile: i32) -> TileLayer {
        TileLayer { w: w.max(0), h: h.max(0), tile: tile.max(1), cells: vec![0; (w.max(0) * h.max(0)) as usize] }
    }

    /// The value at (x, y); 0 off the layer.
    pub fn get(&self, x: i32, y: i32) -> u16 {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return 0;
        }
        self.cells[(y * self.w + x) as usize]
    }

    /// Sets (x, y) (nothing off the layer).
    pub fn set(&mut self, x: i32, y: i32, v: u16) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.cells[(y * self.w + x) as usize] = v;
        }
    }

    /// The layer's world rect (x, y, w, h), for `WorldCamera::bounded`.
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (0.0, 0.0, (self.w * self.tile) as f32, (self.h * self.tile) as f32)
    }

    /// The cell under world point `p`, if on the layer.
    pub fn cell_at(&self, p: (f32, f32)) -> Option<(i32, i32)> {
        let (x, y) = ((p.0 / self.tile as f32).floor() as i32, (p.1 / self.tile as f32).floor() as i32);
        (x >= 0 && y >= 0 && x < self.w && y < self.h).then_some((x, y))
    }

    /// The cells on view through `cam` (x0..x1, y0..y1, clipped to the layer).
    pub fn on_view(&self, cam: &WorldCamera) -> (std::ops::Range<i32>, std::ops::Range<i32>) {
        let (vx, vy, vw, vh) = cam.visible();
        let t = self.tile as f32;
        let x0 = ((vx / t).floor() as i32).clamp(0, self.w);
        let y0 = ((vy / t).floor() as i32).clamp(0, self.h);
        let x1 = (((vx + vw) / t).ceil() as i32).clamp(0, self.w);
        let y1 = (((vy + vh) / t).ceil() as i32).clamp(0, self.h);
        (x0..x1, y0..y1)
    }

    /// The screen rect of cell (x, y) through `cam`.
    pub fn cell_rect(&self, cam: &WorldCamera, x: i32, y: i32) -> Rect {
        let t = self.tile as f32;
        cam.rect_on_screen(x as f32 * t, y as f32 * t, t, t)
    }

    /// Every cell on view with a value, row by row: `cell(x, y, value, screen rect)`.
    pub fn draw(&self, cam: &WorldCamera, mut cell: impl FnMut(i32, i32, u16, Rect)) {
        let (xs, ys) = self.on_view(cam);
        for y in ys {
            for x in xs.clone() {
                let v = self.get(x, y);
                if v != 0 {
                    cell(x, y, v, self.cell_rect(cam, x, y));
                }
            }
        }
    }

    /// `draw` with an image per value (none: the cell is skipped).
    pub fn draw_images<'a>(&self, p: &mut Painter, cam: &WorldCamera, image: impl Fn(u16) -> Option<&'a Image>) {
        self.draw(cam, |_, _, v, r| {
            if let Some(img) = image(v) {
                p.image(img, r);
            }
        });
    }

    /// The transition pieces over the layer: (corner x, corner y, mask 1-14), `upper` saying which values count.
    pub fn corners(&self, upper: impl Fn(u16) -> bool) -> Vec<(i32, i32, u8)> {
        corner_pieces(self.w, self.h, |x, y| upper(self.get(x, y)))
    }

    /// The pieces of `corners` on view, each a tile square centred on its corner, with an image per mask.
    pub fn draw_corners<'a>(&self, p: &mut Painter, cam: &WorldCamera, pieces: &[(i32, i32, u8)], image: impl Fn(u8) -> Option<&'a Image>) {
        let (xs, ys) = self.on_view(cam);
        let t = self.tile as f32;
        for &(cx, cy, mask) in pieces.iter().filter(|(cx, cy, _)| (xs.start..=xs.end).contains(cx) && (ys.start..=ys.end).contains(cy)) {
            if let Some(img) = image(mask) {
                p.image(img, cam.rect_on_screen(cx as f32 * t - t / 2.0, cy as f32 * t - t / 2.0, t, t));
            }
        }
    }
}
