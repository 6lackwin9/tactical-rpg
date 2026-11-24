//! ENGINE-61 (FEEL-CLOSE1): the row blend (`Canvas::blend_rgba`) puts the same pixels as a `blend` of each pixel.

use aegis_ui::canvas::{Canvas, Rect};

/// Every alpha over a busy canvas, partly off its edge and under a clip: the row pass and the pixel pass agree byte
/// for byte (alpha 0 leaves the canvas pixel, alpha included, as `blend` does).
#[test]
fn the_row_blend_puts_the_pixels_a_blend_of_each_does() {
    let (w, h) = (37, 23);
    let base = || {
        let mut c = Canvas::new(w, h);
        for (i, p) in c.px.chunks_exact_mut(4).enumerate() {
            p.copy_from_slice(&[(i * 7) as u8, (i * 13) as u8, (i * 29) as u8, (i * 3) as u8]);
        }
        c
    };
    let (sw, sh) = (20, 16);
    let src: Vec<u8> = (0..sw * sh).flat_map(|i| [(i * 11) as u8, (i * 5) as u8, (i * 17) as u8, ((i * 37) % 256) as u8]).collect();
    for (x, y, clip) in [(3, 2, None), (-4, -3, None), (25, 12, None), (5, 4, Some(Rect::new(8, 6, 9, 5)))] {
        let (mut rows, mut each) = (base(), base());
        rows.clip = clip;
        each.clip = clip;
        rows.blend_rgba(x, y, sw, sh, &src);
        for j in 0..sh {
            for i in 0..sw {
                let k = ((j * sw + i) * 4) as usize;
                each.blend(x + i, y + j, [src[k], src[k + 1], src[k + 2], src[k + 3]]);
            }
        }
        assert!(rows.px == each.px, "at ({x}, {y}) clip {clip:?}");
    }
}
