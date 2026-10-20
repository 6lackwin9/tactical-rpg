//! A screen drawn into a rect of another (UIGALLERY1: the UI Gallery's screen and panel entries). The view is mapped
//! to the rect while the screen draws, so its own layout, fonts and art scale with it, and the canvas is bounded to the
//! rect (`Canvas::bounds`, which the screen's own clipping cannot lift). A whole screen is shrunk into the rect; a
//! panel (a region of the screen) is enlarged into it. A preview is drawn not live (no hover, no drag) and its widgets
//! stay the screen's own (the gallery opens the screen itself to try it).

use crate::canvas::Rect;
use crate::painter::{View, LH, LW};
use crate::ui::Gfx;

/// The view that maps the logical region `src` of a screen into `r` (logical units of `outer`), keeping its aspect
/// and centred.
pub fn region_view(outer: View, r: Rect, src: Rect) -> View {
    let k = (r.w as f32 / src.w.max(1) as f32).min(r.h as f32 / src.h.max(1) as f32).max(0.05);
    let (w, h) = (src.w as f32 * k, src.h as f32 * k);
    View {
        s: outer.s * k,
        ox: outer.ox + (r.x as f32 + (r.w as f32 - w) / 2.0 - src.x as f32 * k) * outer.s,
        oy: outer.oy + (r.y as f32 + (r.h as f32 - h) / 2.0 - src.y as f32 * k) * outer.s,
    }
}

/// The view that maps the whole logical canvas into `r`.
pub fn view_in(outer: View, r: Rect) -> View {
    region_view(outer, r, Rect::new(0, 0, LW as i32, LH as i32))
}

/// Runs `draw` (a whole screen's drawing, e.g. `screen.draw(cx, g, (-1, -1), false)`) with the region `src` of the
/// screen shown in `r`, then puts the view back.
pub fn region_in(g: &mut Gfx, r: Rect, src: Rect, draw: impl FnOnce(&mut Gfx)) {
    let outer = g.view;
    let (old_bounds, old_clip) = (g.canvas.bounds, g.canvas.clip);
    g.view = region_view(outer, r, src);
    let shown = region_view(outer, r, src).rect(src).intersect(outer.rect(r));
    g.canvas.bounds = Some(old_bounds.map_or(shown, |b| b.intersect(shown)));
    draw(g);
    g.view = outer;
    g.canvas.bounds = old_bounds;
    g.canvas.clip = old_clip;
}

/// The whole screen shrunk into `r`.
pub fn screen_in(g: &mut Gfx, r: Rect, draw: impl FnOnce(&mut Gfx)) {
    region_in(g, r, Rect::new(0, 0, LW as i32, LH as i32), draw);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_whole_canvas_lands_inside_the_rect_at_its_aspect() {
        let outer = View { s: 2.0, ox: 0.0, oy: 0.0 };
        let v = view_in(outer, Rect::new(100, 50, 320, 300));
        assert_eq!(v.s, 1.0, "half the width, so half the scale");
        assert_eq!((v.x(0.0), v.y(0.0)), (200, 100 + 2 * (300 - 180) / 2), "left edge on the rect, centred down");
        assert_eq!((v.x(LW), v.y(LH)), (200 + 640, 100 + 120 + 360));
    }

    #[test]
    fn a_region_is_enlarged_into_the_rect_and_nothing_draws_outside_it() {
        let outer = View { s: 1.0, ox: 0.0, oy: 0.0 };
        let v = region_view(outer, Rect::new(0, 0, 200, 100), Rect::new(500, 300, 100, 50));
        assert_eq!(v.s, 2.0);
        assert_eq!((v.x(500.0), v.y(300.0), v.x(600.0), v.y(350.0)), (0, 0, 200, 100), "the region's corners on the rect's");
        let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        let mut g = Gfx { canvas: crate::canvas::Canvas::new(640, 360), view: outer, fonts: crate::painter::Fonts::new(&font), kit: Default::default() };
        region_in(&mut g, Rect::new(10, 10, 100, 50), Rect::new(0, 0, 640, 320), |g| {
            g.canvas.clip = None; // the screen's own clipping cannot lift the bound
            g.canvas.fill(Rect::new(0, 0, 640, 360), [255, 255, 255, 255]);
        });
        let lit = |x: i32, y: i32| g.canvas.px[((y * 640 + x) * 4) as usize] == 255;
        assert!(lit(20, 20) && !lit(5, 5) && !lit(200, 100), "inside the box only");
        assert_eq!((g.view, g.canvas.bounds), (outer, None), "the view and bounds come back");
        g.canvas.bounds = Some(Rect::new(0, 0, 4, 4));
        g.canvas.clear([9, 9, 9, 255]);
        assert!(g.canvas.px[0] == 9 && g.canvas.px[((300 * 640 + 300) * 4) as usize] == 0, "a bounded clear fills the bounds only");
    }
}
