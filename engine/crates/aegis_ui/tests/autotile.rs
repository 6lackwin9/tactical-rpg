//! LOOK1-E2: the tile masks (`aegis_ui::autotile`) and the Kit page's Tiles page: all 16 masks on a fixture, a lone
//! upper cell's four corners, a block's inside, the same list twice, a T shape's open edges; the page on fixture layers
//! (the sets it lists, pieces on their corners, mask numbers without them, the block rule), at the four text sizes.

use aegis_core::assets::Layers;
use aegis_ui::autotile::{block_edges, corner_mask, corner_pieces, BlockEdges};
use aegis_ui::canvas::{Canvas, Rect, Rgba};
use aegis_ui::painter::{Fonts, View};
use aegis_ui::specimens::{tiles_map_cell, KitAct, Specimens, PAGES, TILES_MAP, TILE_VIEWS, TILE_ZOOMS};
use aegis_ui::text_audit;
use aegis_ui::theme::{self, Theme};
use aegis_ui::ui::{by_id, Gfx, Kit, Ui, Widget};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const TEXT_SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];
/// The Kit page's rect as the gallery gives it.
const KIT: Rect = Rect { x: 112, y: 22, w: 522, h: 332 };
const COLS: i32 = 12;
const ROWS: i32 = 9;

/// A closure over rows of text: a cell is upper when it holds `#`; off-grid cells are not.
fn grid<'a>(rows: &'a [&'a str]) -> impl Fn(i32, i32) -> bool + 'a {
    move |x, y| y >= 0 && (y as usize) < rows.len() && x >= 0 && rows[y as usize].as_bytes().get(x as usize) == Some(&b'#')
}

#[test]
fn a_fixture_holds_all_16_masks() {
    // Two single cells and a 1-cell hole, two diagonal touches, and a solid and an empty stretch.
    let rows = ["..........", ".#.....###", "....#..#.#", "..#....###", ".........#", "#.#.....##", ".#......##"];
    let is_upper = grid(&rows);
    let found: BTreeSet<u8> = (0..=rows.len() as i32).flat_map(|cy| (0..=rows[0].len() as i32).map(move |cx| (cx, cy))).map(|(cx, cy)| corner_mask(&is_upper, cx, cy)).collect();
    assert_eq!(found, (0..=15).collect::<BTreeSet<u8>>());
    let pieces: BTreeSet<u8> = corner_pieces(rows[0].len() as i32, rows.len() as i32, &is_upper).into_iter().map(|p| p.2).collect();
    assert_eq!(pieces, (1..=14).collect::<BTreeSet<u8>>(), "corner_pieces holds 1-14 and neither 0 nor 15");
}

#[test]
fn the_bits_name_the_four_cells_around_the_corner() {
    for (cell, bit) in [((2, 4), 1), ((3, 4), 2), ((2, 5), 4), ((3, 5), 8)] {
        assert_eq!(corner_mask(|x, y| (x, y) == cell, 3, 5), bit, "cell {cell:?}");
    }
    assert_eq!(corner_mask(|x, y| x < 0 || y < 0, 0, 0), 1 | 2 | 4, "the closure answers off-grid cells");
}

#[test]
fn a_lone_upper_cell_gives_8_4_2_1_at_its_corners() {
    let pieces = corner_pieces(5, 5, |x, y| (x, y) == (2, 3));
    assert_eq!(pieces, vec![(2, 3, 8), (3, 3, 4), (2, 4, 2), (3, 4, 1)]);
}

#[test]
fn a_3_x_3_upper_block_has_15_inside_and_no_piece_there() {
    let is_upper = |x: i32, y: i32| (1..4).contains(&x) && (1..4).contains(&y);
    for inner in [(2, 2), (3, 2), (2, 3), (3, 3)] {
        assert_eq!(corner_mask(is_upper, inner.0, inner.1), 15);
    }
    let pieces = corner_pieces(5, 5, is_upper);
    assert!(pieces.iter().all(|p| ![(2, 2), (3, 2), (2, 3), (3, 3)].contains(&(p.0, p.1))), "{pieces:?}");
    assert_eq!(pieces.len(), 12, "the outline's corners: 4 x 4 less the 4 inside");
    assert!(pieces.iter().all(|p| (1..=14).contains(&p.2)));
}

#[test]
fn the_same_grid_gives_the_same_list_twice() {
    let rows = ["#..#.", ".##..", "..#.#"];
    assert_eq!(corner_pieces(5, 3, grid(&rows)), corner_pieces(5, 3, grid(&rows)));
}

#[test]
fn block_edges_on_a_t_shape() {
    // ###
    //  #
    let rows = ["###", ".#."];
    let is_block = grid(&rows);
    let e = |north, east, south, west| BlockEdges { north, east, south, west };
    assert_eq!(block_edges(&is_block, 0, 0), e(true, false, true, true), "the bar's west end");
    assert_eq!(block_edges(&is_block, 1, 0), e(true, false, false, false), "the join");
    assert_eq!(block_edges(&is_block, 2, 0), e(true, true, true, false), "the bar's east end");
    assert_eq!(block_edges(&is_block, 1, 1), e(false, true, true, true), "the stem");
}

// ---- the Tiles page ------------------------------------------------------------------------------------------------

/// A folder for fixture layers, unique per test.
fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aegis_autotile_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A one-colour opaque 4 x 4 PNG.
fn tile(path: &Path, c: [u8; 3]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let f = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), 4, 4);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let px: Vec<u8> = (0..16).flat_map(|_| [c[0], c[1], c[2], 255]).collect();
    enc.write_header().unwrap().write_image_data(&px).unwrap();
}

const UPPER: [u8; 3] = [10, 200, 10];
const LOWER: [u8; 3] = [10, 10, 200];
const CAP: [u8; 3] = [200, 10, 10];
const FACE: [u8; 3] = [200, 200, 10];
const PIECE: [u8; 3] = [200, 10, 200];

/// Set `bare` (tiles only) and set `paired` (tiles, a wall with its face, the pair's 14 pieces).
fn fixture_layers(name: &str) -> Layers {
    let root = sandbox(name);
    tile(&root.join("tilesets/bare/ground/v0.png"), UPPER);
    let paired = root.join("tilesets/paired");
    tile(&paired.join("ground/v0.png"), UPPER);
    tile(&paired.join("pool/v0.png"), LOWER);
    tile(&paired.join("wall/v0.png"), CAP);
    tile(&paired.join("wall_face/v0.png"), FACE);
    for m in 1..=14 {
        tile(&paired.join(format!("ground/over_pool/m{m:02}.png")), PIECE);
    }
    Layers::of(vec![root])
}

fn font() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

fn gfx(scale: f32) -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.fonts.scale = scale;
    g.fonts.audit = Some(Vec::new());
    g.canvas.clear(Theme::default().bg);
    g
}

/// The page drawn; its widgets and the text audit's report.
fn page(k: &Specimens, g: &mut Gfx) -> (Vec<Widget<KitAct>>, Value) {
    let w = {
        let mut ui: Ui<KitAct> = Ui::new(g, (-1, -1), true);
        k.draw(&mut ui, KIT, |a| a);
        ui.widgets
    };
    let boxes: Vec<_> = w.iter().map(|x| (x.id.clone(), x.rect.x as f32, x.rect.y as f32, x.rect.w as f32, x.rect.h as f32)).collect();
    let recs = g.fonts.audit.clone().unwrap_or_default();
    (w, text_audit::issues(&recs, 640.0, 360.0, &boxes))
}

fn clean(audit: &Value, what: &str) {
    for k in ["edge", "overlap", "spill"] {
        assert_eq!(audit["counts"][k], json!(0), "{what}: {k}: {audit}");
    }
}

fn map_rect(w: &[Widget<KitAct>]) -> Rect {
    w.iter().find(|x| x.id == "kit.tiles.map").expect("the map's stage").rect
}

fn px(c: &Canvas, x: i32, y: i32) -> Rgba {
    let i = ((y * c.w + x) * 4) as usize;
    [c.px[i], c.px[i + 1], c.px[i + 2], c.px[i + 3]]
}

fn is(c: &Canvas, x: i32, y: i32, want: [u8; 3]) -> bool {
    let p = px(c, x, y);
    [p[0], p[1], p[2]] == want
}

fn tiles_page(set: usize) -> Specimens {
    let mut k = Specimens::default();
    k.act(KitAct::Page(6));
    k.act(KitAct::TileSet(set));
    k
}

/// The fixture map has what the spec asks and all 16 masks, its rim taking the nearest cell's kind.
#[test]
fn the_page_map_holds_every_mask() {
    assert_eq!((TILES_MAP[0].len() as i32, TILES_MAP.len() as i32), (COLS, ROWS));
    assert!(TILES_MAP.iter().all(|r| r.len() as i32 == COLS));
    let is_upper = |x: i32, y: i32| tiles_map_cell(x, y) != b'~';
    let found: BTreeSet<u8> = (0..=ROWS).flat_map(|cy| (0..=COLS).map(move |cx| (cx, cy))).map(|(cx, cy)| corner_mask(is_upper, cx, cy)).collect();
    assert_eq!(found, (0..=15).collect::<BTreeSet<u8>>());
}

/// With pieces: a piece's colour on the corners that take one, the ground and the lower kind away from them.
#[test]
fn a_set_with_pieces_draws_them_on_their_corners() {
    theme::set_layers(fixture_layers("pieces"));
    let k = tiles_page(1);
    let mut g = gfx(1.0);
    let (w, audit) = page(&k, &mut g);
    clean(&audit, "pieces");
    let m = map_rect(&w);
    let (cw, ch) = (TILE_VIEWS[0].1, TILE_VIEWS[0].2);
    let is_upper = |x: i32, y: i32| tiles_map_cell(x, y) != b'~';
    let is_block = |x: i32, y: i32| tiles_map_cell(x, y) == b'#';
    for (cx, cy, _) in corner_pieces(COLS, ROWS, is_upper) {
        // A block drawn after the pieces may cover a corner; a piece on the map's rim is half off.
        if [(cx - 1, cy - 1), (cx, cy - 1), (cx - 1, cy), (cx, cy)].iter().any(|c| is_block(c.0, c.1)) || cx == 0 || cy == 0 {
            continue;
        }
        assert!(is(&g.canvas, m.x + cx * cw - 1, m.y + cy * ch - 1, PIECE), "piece at corner ({cx}, {cy})");
    }
    // A cell with no water around it shows the ground; a corner with water on all four sides, the lower kind.
    assert!(is(&g.canvas, m.x + cw * 6 + 1, m.y + ch * 5 + 8, UPPER), "ground away from water");
    let corners: BTreeSet<(i32, i32)> = corner_pieces(COLS, ROWS, is_upper).into_iter().map(|p| (p.0, p.1)).collect();
    assert!(!corners.contains(&(2, 2)), "(2, 2) is all water");
    assert!(is(&g.canvas, m.x + 2 * cw, m.y + 2 * ch, LOWER), "the lower kind inside the pond");
    let said = w.iter().find(|x| x.id == "kit.tiles.map").unwrap();
    assert!(said.text.contains("Square"), "{}", said.text);
}

/// Without pieces: no piece colour, and every mask the map holds written on it.
#[test]
fn a_set_without_pieces_shows_the_mask_numbers() {
    theme::set_layers(fixture_layers("bare"));
    let mut g = gfx(1.0);
    let (w, audit) = page(&tiles_page(0), &mut g);
    clean(&audit, "bare");
    let m = map_rect(&w);
    let texts: Vec<String> = g.fonts.audit.as_deref().unwrap_or(&[]).iter().map(|t| t.text.clone()).collect();
    let is_upper = |x: i32, y: i32| tiles_map_cell(x, y) != b'~';
    let shown: BTreeSet<u8> = corner_pieces(COLS, ROWS, is_upper).into_iter().map(|p| p.2).collect();
    assert_eq!(shown.len(), 14);
    for mask in shown {
        assert!(texts.iter().any(|t| *t == mask.to_string()), "mask {mask} is written on the map: {texts:?}");
    }
    assert!((m.y..m.y + m.h).all(|y| (m.x..m.x + m.w).all(|x| !is(&g.canvas, x, y, PIECE))), "no piece colour");
}

/// The block rule on the fixture's wall run: the face over a wall's lower half when its south is open, the cap alone
/// when the south is a wall; 1 px lines on the open north and sides.
#[test]
fn a_wall_takes_its_face_only_over_an_open_south() {
    theme::set_layers(fixture_layers("wall"));
    let mut k = tiles_page(1);
    k.act(KitAct::TileView(1));
    let mut g = gfx(1.0);
    let (w, _) = page(&k, &mut g);
    let m = map_rect(&w);
    let (cw, ch) = (TILE_VIEWS[1].1, TILE_VIEWS[1].2);
    // Row 1: `###` at columns 8-10 with a wall under column 10 only (x 10 is the run's corner).
    let at = |x: i32, y: i32, fx: i32, fy: i32| (m.x + x * cw + fx, m.y + y * ch + fy);
    let (x, y) = at(8, 1, cw / 2, ch - 2);
    assert!(is(&g.canvas, x, y, FACE), "the south of (8, 1) is open");
    let (x, y) = at(10, 1, cw / 2, ch - 2);
    assert!(is(&g.canvas, x, y, CAP), "the south of (10, 1) is a wall");
    let (x, y) = at(9, 1, cw / 2, 0);
    assert!(!is(&g.canvas, x, y, CAP), "a line on the open north");
    let (x, y) = at(9, 1, cw / 2, 1);
    assert!(is(&g.canvas, x, y, CAP), "the line is 1 px");
    let (x, y) = at(8, 1, 0, 2);
    assert!(!is(&g.canvas, x, y, CAP), "a line on the open west");
    let (x, y) = at(9, 1, 0, 2);
    assert!(is(&g.canvas, x, y, CAP), "no line between two walls");
}

/// A set with several pairs: the pair picker lists them sorted, each is selectable through its option and draws its
/// own lower kind and pieces; a set with one pair has no picker; a new set starts at its first pair.
#[test]
fn a_set_with_two_pairs_picks_each() {
    const LAVA: [u8; 3] = [240, 120, 10];
    const LAVA_PIECE: [u8; 3] = [120, 240, 10];
    let root = sandbox("two_pairs");
    let duo = root.join("tilesets/duo");
    tile(&duo.join("ground/v0.png"), UPPER);
    tile(&duo.join("lava/v0.png"), LAVA);
    tile(&duo.join("pool/v0.png"), LOWER);
    for m in 1..=14 {
        tile(&duo.join(format!("ground/over_lava/m{m:02}.png")), LAVA_PIECE);
        tile(&duo.join(format!("ground/over_pool/m{m:02}.png")), PIECE);
    }
    let solo = root.join("tilesets/solo");
    tile(&solo.join("ground/over_pool/m01.png"), PIECE);
    theme::set_layers(Layers::of(vec![root]));

    let picker = "kit.tiles.pair";
    let mut k = tiles_page(0);
    // At a corner with water all round the lower kind shows; at a corner taking a piece, the piece.
    let shows = |k: &Specimens, lower: [u8; 3], piece: [u8; 3]| {
        let mut g = gfx(1.0);
        let (w, audit) = page(k, &mut g);
        clean(&audit, "two pairs");
        let m = map_rect(&w);
        let (cw, ch) = (TILE_VIEWS[0].1, TILE_VIEWS[0].2);
        assert!(is(&g.canvas, m.x + 2 * cw, m.y + 2 * ch, lower), "the pair's lower kind");
        assert!(is(&g.canvas, m.x + 3 * cw - 1, m.y + ch - 1, piece), "the pair's piece at corner (3, 1)");
    };
    shows(&k, LAVA, LAVA_PIECE);
    let (w, _) = page(&k, &mut gfx(1.0));
    assert_eq!(w.iter().find(|x| x.id == picker).expect("a picker for two pairs").text, "ground over lava");

    // The list is open: each option picks its pair.
    for (i, (name, lower, piece)) in [("ground over lava", LAVA, LAVA_PIECE), ("ground over pool", LOWER, PIECE)].into_iter().enumerate() {
        let mut g = gfx(1.0);
        g.kit.open = Some(picker.to_string());
        let (w, _) = page(&k, &mut g);
        let option = w.iter().find(|x| x.id == format!("{picker}.opt.{i}")).expect("an option per pair");
        assert_eq!(option.text, name);
        k.act(option.action.clone().expect("an option acts"));
        assert_eq!(k.tiles.pair, i);
        shows(&k, lower, piece);
        let (w, _) = page(&k, &mut gfx(1.0));
        assert_eq!(w.iter().find(|x| x.id == picker).unwrap().text, name);
    }

    k.act(KitAct::TileSet(1));
    assert_eq!(k.tiles.pair, 0, "a new set starts at its first pair");
    let (w, _) = page(&k, &mut gfx(1.0));
    assert!(w.iter().all(|x| x.id != picker), "one pair: no picker");
}

/// Every view and zoom on every set: the page's ids, a clean audit at the four text sizes; the pan is live only when
/// the map is larger than its stage and stops at the map's end.
#[test]
fn every_view_and_zoom_draws_clean() {
    assert_eq!(PAGES[6], "Tiles");
    theme::set_layers(fixture_layers("views"));
    for set in 0..2 {
        for view in 0..TILE_VIEWS.len() {
            for zoom in 0..TILE_ZOOMS.len() {
                let mut k = tiles_page(set);
                k.act(KitAct::TileView(view));
                k.act(KitAct::TileZoom(zoom));
                for scale in TEXT_SIZES {
                    let mut g = gfx(scale);
                    let (w, audit) = page(&k, &mut g);
                    clean(&audit, &format!("set {set} view {view} zoom {zoom} at {scale}"));
                    let m = map_rect(&w);
                    assert!(w.iter().any(|x| x.id == "kit.tiles.set"));
                    let right = w.iter().find(|x| x.id == "kit.tiles.pan.right").unwrap();
                    let (cw, ch) = (TILE_VIEWS[view].1 * TILE_ZOOMS[zoom], TILE_VIEWS[view].2 * TILE_ZOOMS[zoom]);
                    assert_eq!(right.enabled, COLS * cw > m.w);
                    let down = w.iter().find(|x| x.id == "kit.tiles.pan.down").unwrap();
                    assert_eq!(down.enabled, ROWS * ch > m.h);
                    // The controls stay on the page.
                    let zoom_tab = w.iter().find(|x| x.id == format!("kit.tiles.zoom.{}", TILE_ZOOMS.len() - 1)).unwrap();
                    assert!(zoom_tab.rect.x + zoom_tab.rect.w <= KIT.x + KIT.w, "at {scale}");
                }
            }
        }
    }
    // The widest view pans to the map's end and no further.
    let mut k = tiles_page(1);
    k.act(KitAct::TileView(1));
    k.act(KitAct::TileZoom(2));
    let mut steps = 0;
    loop {
        let (w, _) = page(&k, &mut gfx(1.0));
        if !w.iter().find(|x| x.id == "kit.tiles.pan.right").unwrap().enabled {
            break;
        }
        k.act(by_id(&w, "kit.tiles.pan.right").unwrap());
        steps += 1;
        assert!(steps <= COLS, "the pan ends");
    }
    assert!(steps > 0);
    let (w, _) = page(&k, &mut gfx(1.0));
    assert!(w.iter().find(|x| x.id == "kit.tiles.pan.left").unwrap().enabled);
    k.act(KitAct::TileZoom(0));
    assert_eq!(k.tiles.pan, (0, 0), "a new zoom starts at the map's corner");
}

/// The picker lists the layers' sets; no layers: a disabled picker, flat colours and the numbers, nothing panics; a
/// new `set_layers` (as a reload does) is read again.
#[test]
fn the_picker_lists_the_sets_of_the_layers() {
    theme::set_layers(Layers::default());
    let k = tiles_page(0);
    let mut g = gfx(1.0);
    let (w, audit) = page(&k, &mut g);
    clean(&audit, "no layers");
    assert!(!w.iter().find(|x| x.id == "kit.tiles.set").unwrap().enabled);

    theme::set_layers(fixture_layers("picker"));
    let (w, _) = page(&k, &mut gfx(1.0));
    let picker = w.iter().find(|x| x.id == "kit.tiles.set").unwrap();
    assert!(picker.enabled, "the same page reads the new layers");
    assert_eq!(picker.text, "bare");
    let mut kk = k.clone();
    kk.act(KitAct::TileSet(1));
    let (w, _) = page(&kk, &mut gfx(1.0));
    assert_eq!(w.iter().find(|x| x.id == "kit.tiles.set").unwrap().text, "paired");
    kk.act(KitAct::TileSet(9));
    let (w, _) = page(&kk, &mut gfx(1.0));
    assert_eq!(w.iter().find(|x| x.id == "kit.tiles.set").unwrap().text, "paired", "past the list: the last set");
}
