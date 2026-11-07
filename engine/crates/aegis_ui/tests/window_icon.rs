//! ICON1: the window icon from the manifest's path: the size bound, no path, a missing file and a bad file.

use aegis_core::scratch::Scratch;
use aegis_core::assets::Layers;
use aegis_ui::canvas::Image;
use aegis_ui::icon::{fit, window_icons, ICON_MAX, WINDOWS_SMALL};

fn image(w: i32, h: i32) -> Image {
    Image { w, h, px: [200, 150, 40, 255].repeat((w * h) as usize) }
}

fn scratch(name: &str) -> Scratch {
    let dir = Scratch::vacant(&format!("window_icon_{name}"));
    std::fs::create_dir_all(dir.join("ui")).unwrap();
    dir
}

fn write_png(path: &std::path::Path, w: u32, h: u32) {
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&[200, 150, 40, 255].repeat((w * h) as usize)).unwrap();
}

#[test]
fn a_large_image_is_reduced_within_the_bound_with_its_shape() {
    let square = fit(&image(1000, 1000), ICON_MAX);
    assert_eq!((square.w, square.h), (ICON_MAX, ICON_MAX));
    assert_eq!(square.px.len(), (ICON_MAX * ICON_MAX * 4) as usize);
    let wide = fit(&image(1000, 500), ICON_MAX);
    assert_eq!((wide.w, wide.h), (ICON_MAX, ICON_MAX / 2));
    let thin = fit(&image(2000, 1), WINDOWS_SMALL);
    assert_eq!((thin.w, thin.h), (WINDOWS_SMALL, 1), "a side never reaches 0");
}

#[test]
fn a_small_image_is_not_enlarged() {
    let small = fit(&image(20, 10), ICON_MAX);
    assert_eq!((small.w, small.h), (20, 10));
}

#[test]
fn averaging_keeps_a_flat_colour() {
    let out = fit(&image(1000, 1000), WINDOWS_SMALL);
    assert!(out.px.chunks_exact(4).all(|p| p == [200, 150, 40, 255]), "neither averaging nor sharpening moves a flat colour");
}

#[test]
fn no_path_means_no_icon_and_no_notice() {
    assert!(matches!(window_icons(&Layers::default(), ""), Ok(None)));
}

#[test]
fn a_named_file_no_layer_holds_is_a_notice() {
    let dir = scratch("missing");
    let err = window_icons(&Layers::of(vec![dir.to_path_buf()]), "ui/none.png").err().expect("a notice");
    assert!(err.contains("ui/none.png"), "{err}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_bad_file_is_a_notice_not_a_crash() {
    let dir = scratch("bad");
    std::fs::write(dir.join("ui/bad.png"), b"not a png at all").unwrap();
    std::fs::write(dir.join("ui/empty.png"), b"").unwrap();
    let layers = Layers::of(vec![dir.to_path_buf()]);
    for rel in ["ui/bad.png", "ui/empty.png"] {
        assert!(window_icons(&layers, rel).is_err(), "{rel}");
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_good_file_is_found_over_the_layers() {
    let (first, second) = (scratch("first"), scratch("second"));
    write_png(&second.join("ui/mark.png"), 300, 300);
    let layers = Layers::of(vec![first.to_path_buf(), second.to_path_buf()]);
    assert!(matches!(window_icons(&layers, "ui/mark.png"), Ok(Some(_))));
    let _ = std::fs::remove_dir_all(first);
    let _ = std::fs::remove_dir_all(second);
}
