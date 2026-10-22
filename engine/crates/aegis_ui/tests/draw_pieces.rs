//! PROPS1-E draw pieces: the feet-sorted draw list (`aegis_ui::depth::DepthList`), mirrored images
//! (`Image::mirrored`, `Art::image_mirrored`), a PNG's own size (`Art::size_of`) and the variant pick over a chain of
//! folders (`Art::varied`, `Art::pick`; `Art::tile_in` wraps it). Expectations follow from the sort keys and the
//! position formula (x x 7 + y x 13) mod n of the spec (PROPS1 3.2, 4.1).

use aegis_core::assets::Layers;
use aegis_ui::art::Art;
use aegis_ui::canvas::Image;
use aegis_ui::depth::DepthList;
use std::path::{Path, PathBuf};

fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(px).unwrap();
}

/// A 1 x 1 PNG whose red channel is `red`.
fn dot(path: &Path, red: u8) {
    write_png(path, 1, 1, &[red, 0, 0, 255]);
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_draw_pieces_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
}

#[test]
fn the_depth_list_sorts_by_feet_then_group_then_x_then_push_order() {
    let mut list = DepthList::new();
    list.push(48.0, 2, 128.0, "party north");
    list.push(144.0, 0, 64.0, "hall");
    list.push(144.0, 1, 80.0, "npc at the hall's feet");
    list.push(144.0, 1, 32.0, "npc further west");
    list.push(160.0, 2, 0.0, "party south");
    list.push(144.0, 1, 32.0, "second npc on the same spot");
    list.push(-8.0, 0, 500.0, "prop above the map");
    assert_eq!(list.len(), 7);
    let order = list.into_sorted();
    assert_eq!(order, vec!["prop above the map", "party north", "hall", "npc further west", "second npc on the same spot", "npc at the hall's feet", "party south"]);
}

#[test]
fn equal_keys_keep_push_order() {
    let mut list = DepthList::new();
    for n in 0..10 {
        list.push(16.0, 1, 4.0, n);
    }
    assert_eq!(list.into_sorted(), (0..10).collect::<Vec<_>>());
    assert!(DepthList::<u8>::default().is_empty());
}

#[test]
fn mirroring_swaps_columns_and_twice_gives_the_original() {
    // 3 x 2: each pixel's red channel is 10 x row + column.
    let px: Vec<u8> = (0..2).flat_map(|y| (0..3).flat_map(move |x| [10 * y + x, 1, 2, 255])).collect();
    let img = Image { w: 3, h: 2, px };
    let m = img.mirrored();
    let reds = |i: &Image| i.px.chunks_exact(4).map(|p| p[0]).collect::<Vec<_>>();
    assert_eq!((m.w, m.h), (3, 2));
    assert_eq!(reds(&m), vec![2, 1, 0, 12, 11, 10]);
    assert_eq!(m.mirrored().px, img.px);
}

#[test]
fn art_mirrors_a_file_and_reads_its_own_size() {
    let root = scratch("mirror");
    write_png(&root.join("props/common/post/v0.png"), 2, 3, &[[9, 0, 0, 255], [0, 0, 0, 0]].repeat(3).concat());
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    assert_eq!(art.size_of("props/common/post/v0.png"), Some((2, 3)));
    assert_eq!(art.size_of("props/common/none/v0.png"), None);
    let plain = art.image("props/common/post/v0.png", 2, 3).unwrap().clone();
    let flipped = art.image_mirrored("props/common/post/v0.png", 2, 3).unwrap().clone();
    assert_eq!((plain.px[0], plain.px[7], flipped.px[0], flipped.px[7]), (9, 0, 0, 255));
    assert_eq!(flipped.mirrored().px, plain.px);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_variant_comes_from_the_first_folder_with_files() {
    // Two layers: the first holds world_a/tent v0..v1 and nothing of crate; the second common/tent v0..v2, common/crate v0..v3.
    let (game, kit) = (scratch("varied_game"), scratch("varied_kit"));
    for v in 0..2 {
        dot(&game.join(format!("props/world_a/tent/v{v}.png")), 100 + v);
    }
    for v in 0..3 {
        dot(&kit.join(format!("props/common/tent/v{v}.png")), 50 + v);
    }
    for v in 0..4 {
        dot(&kit.join(format!("props/common/crate/v{v}.png")), v);
        dot(&kit.join(format!("tilesets/common/crate/v{v}.png")), v);
    }
    let mut art = Art::layered(Layers::of(vec![game.clone(), kit.clone()]));
    let red = |art: &mut Art, dirs: &[&str], v: Option<u32>, x: i32, y: i32| art.varied(dirs, v, x, y, 1, 1).map(|i| i.px[0]);
    let tent = ["props/world_a/tent", "props/common/tent"];
    assert_eq!(red(&mut art, &tent, Some(1), 0, 0), Some(101), "the world's own folder first, its v1");
    assert_eq!(red(&mut art, &tent, Some(2), 0, 0), Some(100), "past the world's two files: v0");
    assert_eq!(red(&mut art, &tent, Some(15), 0, 0), Some(100));
    let crate_dirs = ["props/world_a/crate", "props/common/crate"];
    assert_eq!(red(&mut art, &crate_dirs, Some(3), 0, 0), Some(3), "no world files: common's v3");
    for (x, y) in [(0, 0), (1, 0), (0, 1), (3, 5), (-2, 7)] {
        let want = (x * 7 + y * 13i32).rem_euclid(4) as u8;
        assert_eq!(red(&mut art, &crate_dirs, None, x, y), Some(want), "position pick at ({x}, {y})");
        let tile = art.tile_in(&["world_a", "common"], "crate", x, y, 1).map(|i| i.px[0]);
        assert_eq!(tile, Some(want), "tile_in picks the same file");
    }
    assert_eq!(art.pick(&crate_dirs, None, 1, 0).as_deref(), Some("props/common/crate/v3.png"));
    assert!(art.varied(&["props/world_a/lamp", "props/common/lamp"], None, 0, 0, 1, 1).is_none());
    let _ = std::fs::remove_dir_all(&game);
    let _ = std::fs::remove_dir_all(&kit);
}
