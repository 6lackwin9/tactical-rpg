//! WORLD-PRIM1 (ENGINE-9, WP2): a camera over a game world drawn in a screen area. The world is in world pixels (a
//! tile `n` world pixels wide sits at `n * col`); the camera holds the world point at the area's centre, a zoom from a
//! fixed list of steps, and the world's bounds it keeps the view inside. `follow` eases the centre toward a target
//! (half the way every `half_life_ms`), `snap` jumps; `to_screen` and `to_world` convert points, `visible` names the
//! world rect on view (what a tile layer draws). Not the screen shake (`fx::Camera`): a shake offsets what this draws.

use crate::canvas::Rect;

/// The zoom steps a camera offers by default: half, whole, double and triple size.
pub const ZOOMS: [f32; 4] = [0.5, 1.0, 2.0, 3.0];

#[derive(Debug, Clone, PartialEq)]
pub struct WorldCamera {
    /// The world point at the centre of `view`.
    pub center: (f32, f32),
    /// The zoom steps, smallest first, and the one in use.
    pub zooms: Vec<f32>,
    pub zoom: usize,
    /// The world rect the view stays inside (x, y, w, h); None: anywhere.
    pub bounds: Option<(f32, f32, f32, f32)>,
    /// The screen area the world is drawn in.
    pub view: Rect,
}

impl WorldCamera {
    /// A camera over `view` at zoom 1 (the default steps), centred on the world origin, with no bounds.
    pub fn new(view: Rect) -> WorldCamera {
        WorldCamera { center: (0.0, 0.0), zooms: ZOOMS.to_vec(), zoom: 1, bounds: None, view }
    }

    /// The camera kept inside world rect `bounds` (x, y, w, h).
    pub fn bounded(mut self, bounds: (f32, f32, f32, f32)) -> WorldCamera {
        self.bounds = Some(bounds);
        self.clamp();
        self
    }

    /// Screen pixels per world pixel.
    pub fn scale(&self) -> f32 {
        self.zooms.get(self.zoom).copied().unwrap_or(1.0).max(0.01)
    }

    /// Moves `steps` zoom steps (up: larger), within the list; the view is clamped again.
    pub fn zoom_by(&mut self, steps: i32) {
        let last = self.zooms.len().saturating_sub(1) as i32;
        self.zoom = (self.zoom as i32 + steps).clamp(0, last) as usize;
        self.clamp();
    }

    /// The centre on `target` at once (clamped).
    pub fn snap(&mut self, target: (f32, f32)) {
        self.center = target;
        self.clamp();
    }

    /// The centre eased toward `target` over `dt_ms`: half the way left every `half_life_ms` (0: at once), clamped.
    pub fn follow(&mut self, target: (f32, f32), dt_ms: f32, half_life_ms: f32) {
        let k = if half_life_ms <= 0.0 { 1.0 } else { 1.0 - 0.5f32.powf(dt_ms.max(0.0) / half_life_ms) };
        self.center = (self.center.0 + (target.0 - self.center.0) * k, self.center.1 + (target.1 - self.center.1) * k);
        self.clamp();
    }

    /// The centre moved so the view stays inside the bounds; on an axis where the world is smaller than the view, the
    /// world is centred.
    pub fn clamp(&mut self) {
        let Some((bx, by, bw, bh)) = self.bounds else { return };
        let s = self.scale();
        let axis = |c: f32, lo: f32, len: f32, half: f32| if len <= 2.0 * half { lo + len / 2.0 } else { c.clamp(lo + half, lo + len - half) };
        self.center = (axis(self.center.0, bx, bw, self.view.w as f32 / 2.0 / s), axis(self.center.1, by, bh, self.view.h as f32 / 2.0 / s));
    }

    /// Where world point `p` is drawn on the screen.
    pub fn to_screen(&self, p: (f32, f32)) -> (f32, f32) {
        let s = self.scale();
        (self.view.x as f32 + self.view.w as f32 / 2.0 + (p.0 - self.center.0) * s, self.view.y as f32 + self.view.h as f32 / 2.0 + (p.1 - self.center.1) * s)
    }

    /// The world point under screen point `p`.
    pub fn to_world(&self, p: (f32, f32)) -> (f32, f32) {
        let s = self.scale();
        (self.center.0 + (p.0 - self.view.x as f32 - self.view.w as f32 / 2.0) / s, self.center.1 + (p.1 - self.view.y as f32 - self.view.h as f32 / 2.0) / s)
    }

    /// The screen rect of world rect (x, y, w, h), rounded to whole pixels edge by edge (neighbours share their edges).
    pub fn rect_on_screen(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        let (x0, y0) = self.to_screen((x, y));
        let (x1, y1) = self.to_screen((x + w, y + h));
        let (x0, y0, x1, y1) = (x0.round() as i32, y0.round() as i32, x1.round() as i32, y1.round() as i32);
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }

    /// The world rect on view (x, y, w, h).
    pub fn visible(&self) -> (f32, f32, f32, f32) {
        let s = self.scale();
        let (w, h) = (self.view.w as f32 / s, self.view.h as f32 / s);
        (self.center.0 - w / 2.0, self.center.1 - h / 2.0, w, h)
    }
}
