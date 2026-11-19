//! WORLD-PRIM1 (ENGINE-9, WP2): the world camera (points both ways, zoom steps, bounds, follow) and the tile layer
//! (cells on view only, rects that meet, transition corners).

use aegis_ui::canvas::Rect;
use aegis_ui::tilemap::TileLayer;
use aegis_ui::world_camera::WorldCamera;

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01
}

#[test]
fn points_go_both_ways_at_every_zoom_and_the_view_stays_in_bounds() {
    let mut cam = WorldCamera::new(Rect::new(100, 50, 400, 200));
    for z in 0..cam.zooms.len() {
        cam.zoom = z;
        cam.center = (37.0, -12.5);
        for p in [(0.0, 0.0), (123.0, 45.0), (-8.0, 300.0)] {
            assert!(near(cam.to_world(cam.to_screen(p)), p), "zoom {z}: {p:?}");
        }
        assert!(near(cam.to_screen(cam.center), (300.0, 150.0)), "the centre is drawn mid-view");
    }
    // A 1000 x 100 world at zoom 1: x stays half a view from the edges, y (smaller than the view) is centred.
    let mut cam = WorldCamera::new(Rect::new(0, 0, 400, 200)).bounded((0.0, 0.0, 1000.0, 100.0));
    cam.snap((-50.0, 0.0));
    assert_eq!(cam.center, (200.0, 50.0));
    cam.snap((5000.0, 0.0));
    assert_eq!(cam.center, (800.0, 50.0));
    cam.zoom_by(5);
    assert_eq!(cam.zoom, cam.zooms.len() - 1, "the last step, no further");
    cam.snap((5000.0, 0.0));
    assert_eq!(cam.center.0, 1000.0 - 200.0 / cam.scale(), "half a (smaller) view from the edge at the new zoom");
    cam.zoom_by(-9);
    assert_eq!((cam.zoom, cam.center), (0, (600.0, 50.0)), "at half size the view is 800 wide: 400 from the right edge");
    let (vx, vy, vw, vh) = WorldCamera::new(Rect::new(0, 0, 400, 200)).visible();
    assert_eq!((vx, vy, vw, vh), (-200.0, -100.0, 400.0, 200.0));
}

#[test]
fn follow_eases_half_the_way_each_half_life() {
    let mut cam = WorldCamera::new(Rect::new(0, 0, 100, 100));
    cam.follow((100.0, 0.0), 200.0, 200.0);
    assert!(near(cam.center, (50.0, 0.0)));
    cam.follow((100.0, 0.0), 400.0, 200.0);
    assert!(near(cam.center, (87.5, 0.0)));
    cam.follow((10.0, 10.0), 16.0, 0.0);
    assert_eq!(cam.center, (10.0, 10.0), "no half life: at once");
}

#[test]
fn a_tile_layer_draws_only_the_cells_on_view_in_rects_that_meet() {
    let mut layer = TileLayer::new(50, 40, 16);
    for y in 0..40 {
        for x in 0..50 {
            layer.set(x, y, 1 + ((x + y) % 2) as u16);
        }
    }
    layer.set(99, 99, 7);
    assert_eq!((layer.get(99, 99), layer.get(-1, 0)), (0, 0), "off the layer: nothing set, 0 read");
    let mut cam = WorldCamera::new(Rect::new(0, 0, 160, 96)).bounded(layer.bounds());
    cam.zoom = 2; // double size: 80 x 48 world pixels on view, x 360..440 and y 296..344: tiles 22..=27 by 18..=21
    cam.snap((400.0, 320.0));
    let mut drawn = Vec::new();
    layer.draw(&cam, |x, y, _, r| drawn.push((x, y, r)));
    assert_eq!(drawn.len(), 6 * 4, "the cells cut by the view's edges too: {:?}", drawn.iter().map(|d| (d.0, d.1)).collect::<Vec<_>>());
    for (x, y, r) in &drawn {
        assert_eq!((r.w, r.h), (32, 32), "({x}, {y}): a tile at double size");
        if let Some((_, _, right)) = drawn.iter().find(|(x2, y2, _)| *x2 == x + 1 && y2 == y) {
            assert_eq!(r.x + r.w, right.x, "neighbours meet");
        }
    }
    assert_eq!(layer.cell_at(cam.to_world((80.0, 48.0))), Some((25, 20)), "the cell under the view's centre");
    assert_eq!(layer.cell_at((-1.0, 3.0)), None);
    // One raised cell gives the four corner pieces around it.
    let mut one = TileLayer::new(3, 3, 8);
    one.set(1, 1, 2);
    let pieces = one.corners(|v| v == 2);
    assert_eq!(pieces, [(1, 1, 8), (2, 1, 4), (1, 2, 2), (2, 2, 1)]);
}
