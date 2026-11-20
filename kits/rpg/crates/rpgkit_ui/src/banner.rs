//! MENURAIL1 (PR7-30, LOOK1 M2): a framed card with its title on a banner across the card's top edge (decision
//! "LOOK1-A6 picks": the story card's banner, "Story card banner: crests as the ends"). The banner is a picture the
//! game names (its asset path); it is drawn at the window's physical size as three slices: the two ends kept at their
//! art aspect and the middle repeated to the title's length, so the ends never stretch. With no banner file the title
//! draws on the card's top edge over a plain bar in the frame's colours; the card itself is the theme's `ornate` frame
//! (the vector panel without one). The kit names no game: the caller passes the picture and the texts' colours.

use aegis_ui::art::Art;
use aegis_ui::canvas::{Image, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::ui::Ui;

/// The frame role of a banner card.
pub const CARD_ROLE: &str = "ornate";
/// The share of the banner picture's height each end takes across (the ends hold the finials or crests).
pub const END_SHARE: f32 = 0.78;
/// The banner's height over the title's line height.
pub const HEIGHT_OVER_LINE: f32 = 1.9;

/// The banner rect for a title `title_w` wide in lines `line_h` tall, centred on the point (`cx`, `top`) (the card's
/// top edge): the title plus an end and a margin each side.
pub fn banner_rect(cx: i32, top: i32, title_w: f32, line_h: f32) -> Rect {
    let h = (line_h * HEIGHT_OVER_LINE).ceil() as i32;
    let end = (h as f32 * END_SHARE).ceil() as i32;
    let w = title_w.ceil() as i32 + 2 * end + h / 2;
    Rect::new(cx - w / 2, top - h / 2, w, h)
}

/// The columns of a `w` wide banner from a source `sw` wide, `sh` tall at that height: for each output column the
/// source column (the left end as is, the middle repeated, the right end as is).
pub fn slice_columns(w: i32, sw: i32, sh: i32) -> Vec<i32> {
    let end = ((sh as f32 * END_SHARE).round() as i32).clamp(0, sw / 2);
    let mid = (sw - 2 * end).max(1);
    (0..w)
        .map(|x| {
            if x < end.min(w / 2) {
                x
            } else if x >= w - end.min(w - w / 2) {
                sw - (w - x)
            } else {
                end + (x - end) % mid
            }
        })
        .collect()
}

/// Banner picture `rel` drawn in `r` (logical) at the window's physical size; false (nothing drawn) without the file.
pub fn draw_banner(p: &mut Painter, art: &mut Art, rel: &str, r: Rect) -> bool {
    let Some((iw, ih)) = art.size_of(rel).filter(|s| s.0 > 0 && s.1 > 0) else { return false };
    let pr = p.v.rect(r);
    if pr.w <= 0 || pr.h <= 0 {
        return false;
    }
    let sw = ((iw as f32 * pr.h as f32 / ih as f32).round() as i32).max(1);
    let Some(src) = art.image(rel, sw, pr.h) else { return false };
    let cols = slice_columns(pr.w, src.w, src.h);
    let mut px = Vec::with_capacity((pr.w * pr.h * 4) as usize);
    for y in 0..pr.h {
        for &sx in &cols {
            let i = ((y * src.w + sx.clamp(0, src.w - 1)) * 4) as usize;
            px.extend_from_slice(&src.px[i..i + 4]);
        }
    }
    p.image(&Image { w: pr.w, h: pr.h, px }, r);
    true
}

/// A banner card: the `ornate` frame in `card`, the banner (`banner`, "" none) across its top edge holding `title`
/// in `size` and `title_col`; with no banner file a plain bar in `bar` and `edge`. Returns the card's body rect (below
/// the banner's lower half).
#[allow(clippy::too_many_arguments)]
pub fn banner_card<A: Clone>(ui: &mut Ui<A>, art: &mut Art, banner: &str, card: Rect, title: &str, size: f32, title_col: Rgba, bar: Rgba, edge: Rgba) -> Rect {
    let body = ui.frame_body(CARD_ROLE, card);
    let (tw, lh) = (ui.p.text_w(title, size), ui.p.line_h(size));
    let br = banner_rect(card.x + card.w / 2, card.y, tw, lh);
    if banner.is_empty() || !draw_banner(&mut ui.p, art, banner, br) {
        let plain = Rect::new(br.x + br.h / 2, br.y + br.h / 6, br.w - br.h, br.h - br.h / 3);
        ui.p.panel(plain, bar, edge, 0.0);
    }
    ui.p.text_center((br.x + br.w / 2) as f32, br.y as f32 + (br.h as f32 - lh) / 2.0, title, size, title_col);
    let below = br.y + br.h + 2;
    Rect::new(body.x, below.max(body.y), body.w, (body.y + body.h - below.max(body.y)).max(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ends keep their source columns, the middle repeats, and a banner narrower than two ends splits them.
    #[test]
    fn the_ends_stay_and_the_middle_repeats() {
        // source 100 x 10: ends 8 columns each (0.78 x 10), middle 84
        let cols = slice_columns(200, 100, 10);
        assert_eq!(&cols[..8], &[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(&cols[192..], &[92, 93, 94, 95, 96, 97, 98, 99]);
        assert_eq!(cols[8], 8);
        assert_eq!(cols[8 + 84], 8, "the middle wraps");
        assert!(cols[8..192].iter().all(|c| (8..92).contains(c)));
        let narrow = slice_columns(10, 100, 10);
        assert_eq!(narrow, vec![0, 1, 2, 3, 4, 95, 96, 97, 98, 99]);
    }

    /// The banner is centred on the edge point, as tall as the line by `HEIGHT_OVER_LINE`, and wider than the title by
    /// both ends.
    #[test]
    fn the_banner_sits_on_the_edge_around_the_title() {
        let r = banner_rect(320, 100, 120.0, 14.0);
        assert_eq!((r.x + r.w / 2, r.y + r.h / 2), (320, 100));
        assert_eq!(r.h, (14.0f32 * HEIGHT_OVER_LINE).ceil() as i32);
        assert!(r.w >= 120 + 2 * (r.h as f32 * END_SHARE) as i32);
    }
}
