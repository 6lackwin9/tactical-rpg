//! LAYOUT720-E5, `CanvasPolicy` (docs/specs/LAYOUT720.md 6.4, 14): how a game's canvas relates to the window, carried in the
//! view as a value, and the manifest's two keys. Old screens never see it, and `Fixed` at the grid's size is exactly the
//! view every native screen had before. Expectations come from the table of 6.4.

use aegis_layout::{fill, Rect};
use aegis_ui::painter::{CanvasPolicy, Grid, View, GRID};
use aegis_ui::screen::{read_window_sizes, Settings, WINDOW_SIDE_MAX, WINDOW_SIZES, WINDOW_SIZES_MAX};
use aegis_ui::shell::read_window_options;
use serde_json::{json, Value};

const FIT: CanvasPolicy = CanvasPolicy::Fit { w: GRID.0, h: GRID.1 };

/// Window sizes through every aspect, whole and fractional scales alike.
fn windows() -> Vec<(u32, u32)> {
    let mut all = vec![(1, 1), (640, 360), (1280, 720), (1366, 768), (1600, 900), (1920, 1080), (2560, 1440), (3840, 2160), (1920, 1200), (3440, 1440), (1280, 800), (800, 600), (1170, 2532), (2532, 1170)];
    for pw in (16..4000).step_by(173) {
        for ph in (9..2600).step_by(131) {
            all.push((pw, ph));
        }
    }
    all
}

#[test]
fn the_default_is_exactly_the_view_native_screens_always_had() {
    assert_eq!(CanvasPolicy::default(), CanvasPolicy::Fixed { w: GRID.0, h: GRID.1 });
    for (pw, ph) in windows() {
        let was = View::fit(pw, ph).for_grid(Grid::Native);
        assert_eq!(View::native(CanvasPolicy::default(), pw, ph, 1.0, 1.0), was, "{pw}x{ph}");
        assert_eq!(was.canvas(), GRID);
        assert_eq!(View::native(CanvasPolicy::default(), pw, ph, 3.0, 1.5), was, "{pw}x{ph}: Fixed reads no dpr or UI scale");
    }
}

#[test]
fn old_and_quadrant_screens_keep_the_old_canvas() {
    for (pw, ph) in windows() {
        let whole = View::fit(pw, ph);
        assert_eq!(whole.for_grid(Grid::Old), whole, "{pw}x{ph}");
        assert_eq!(whole.canvas(), (640, 360));
        assert_eq!(whole.for_grid(Grid::Quadrant).canvas(), (640, 360));
    }
}

#[test]
fn fixed_keeps_its_canvas_at_any_window_with_bars() {
    let policy = CanvasPolicy::Fixed { w: 1920, h: 1080 };
    for (pw, ph) in windows().into_iter().filter(|w| w.0 >= 320 && w.1 >= 180) {
        let v = View::native(policy, pw, ph, 1.0, 1.0);
        assert_eq!((v.canvas(), v.g), ((1920, 1080), 3), "{pw}x{ph}");
        assert_eq!(v.s, (pw as f32 / 1920.0).min(ph as f32 / 1080.0), "{pw}x{ph}");
        // As `View::fit` has always done, a side the scale came from can round to one pixel over.
        let drawn = v.rect(Rect::new(0, 0, 1920, 1080));
        assert!(drawn.x >= -1 && drawn.y >= -1 && drawn.x + drawn.w <= pw as i32 + 1 && drawn.y + drawn.h <= ph as i32 + 1, "{pw}x{ph}: {drawn:?}");
    }
}

#[test]
fn fit_and_dp_follow_the_table() {
    // (policy, window, dpr, ui scale, the canvas the table gives)
    let table = [
        (FIT, (1280, 720), 1.0, 1.0, (1280, 720)),
        (FIT, (1280, 800), 1.0, 1.0, (1280, 800)),
        (FIT, (1920, 1200), 1.0, 1.0, (1280, 800)),
        (FIT, (3440, 1440), 1.0, 1.0, (1720, 720)),
        (CanvasPolicy::Dp, (800, 600), 1.0, 1.0, (800, 600)),
        (CanvasPolicy::Dp, (2532, 1170), 3.0, 1.0, (844, 390)),
        (CanvasPolicy::Dp, (1170, 2532), 3.0, 1.0, (390, 844)),
        (CanvasPolicy::Dp, (800, 600), 2.0, 2.0, (200, 150)),
    ];
    for (policy, (pw, ph), dpr, ui, canvas) in table {
        let v = View::native(policy, pw, ph, dpr, ui);
        assert_eq!(v.canvas(), canvas, "{policy:?} {pw}x{ph}");
        assert_eq!(v.g, GRID.0 / 640, "{policy:?}: tokens stay in the grid's units");
    }
    // Under Fit the UI scale shrinks the canvas: the larger the scale, the fewer the units.
    let handheld = View::native(FIT, 1280, 800, 1.0, 1.5);
    assert_eq!((handheld.s, handheld.canvas()), (1.5, (1280 * 2 / 3, 800 * 2 / 3)));
}

#[test]
fn a_fit_canvas_is_whole_units_inside_the_window_and_at_least_its_size() {
    for (pw, ph) in windows().into_iter().filter(|w| w.0 >= 320 && w.1 >= 180) {
        let v = View::native(FIT, pw, ph, 1.0, 1.0);
        let (cw, ch) = v.canvas();
        assert!(cw >= GRID.0 && ch >= GRID.1, "{pw}x{ph}: {cw}x{ch}");
        // The tight side is the one the scale came from: exactly the grid, however the float falls.
        assert!(cw == GRID.0 || ch == GRID.1, "{pw}x{ph}: {cw}x{ch}");
        let drawn = v.rect(Rect::new(0, 0, cw, ch));
        assert!(drawn.x >= 0 && drawn.y >= 0 && drawn.x + drawn.w <= pw as i32 && drawn.y + drawn.h <= ph as i32, "{pw}x{ph}: {drawn:?}");
        assert!(pw as f32 - (cw as f32 + 1.0) * v.s < 1.0 && ph as f32 - (ch as f32 + 1.0) * v.s < 1.0, "{pw}x{ph}: less than a unit is left");
    }
}

#[test]
fn the_pointer_reads_back_through_any_canvas() {
    for (policy, dpr, ui) in [(FIT, 1.0, 1.0), (FIT, 1.0, 1.5), (CanvasPolicy::Dp, 3.0, 1.0), (CanvasPolicy::Dp, 1.0, 1.0)] {
        for (pw, ph) in [(1280, 800), (3440, 1440), (1170, 2532), (800, 600)] {
            let v = View::native(policy, pw, ph, dpr, ui);
            let (cw, ch) = v.canvas();
            for p in [(0, 0), (cw / 2, ch / 2), (cw - 1, ch - 1)] {
                let (x, y) = v.to_physical(p.0, p.1);
                assert_eq!(v.to_logical(x, y), p, "{policy:?} {pw}x{ph}");
            }
        }
    }
}

#[test]
fn neighbours_share_a_pixel_edge_on_every_canvas() {
    for (policy, dpr, ui) in [(FIT, 1.0, 1.0), (FIT, 1.0, 1.5), (CanvasPolicy::Dp, 3.0, 1.0)] {
        for (pw, ph) in [(1280, 800), (3440, 1440), (1366, 768), (1170, 2532)] {
            let v = View::native(policy, pw, ph, dpr, ui);
            let (cw, ch) = v.canvas();
            let cells = Rect::new(0, 0, cw, ch).split_x(&[fill(2), fill(1), fill(3)], 0);
            let px: Vec<Rect> = cells.iter().map(|c| v.rect(*c)).collect();
            assert!(px.windows(2).all(|p| p[0].x + p[0].w == p[1].x), "{policy:?} {pw}x{ph}: {px:?}");
        }
    }
}

#[test]
fn a_bad_dpr_or_ui_scale_counts_as_one() {
    for bad in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(View::native(CanvasPolicy::Dp, 800, 600, bad, 1.0).canvas(), (800, 600), "{bad}");
        assert_eq!(View::native(CanvasPolicy::Dp, 800, 600, 1.0, bad).canvas(), (800, 600), "{bad}");
        assert_eq!(View::native(FIT, 1280, 720, 1.0, bad).canvas(), GRID, "{bad}");
    }
}

#[test]
fn a_tiny_scale_is_held_so_the_canvas_stays_bounded() {
    let (pw, ph) = (WINDOW_SIDE_MAX, WINDOW_SIDE_MAX);
    for (policy, dpr, ui) in [(FIT, 1.0, 1e-6), (CanvasPolicy::Dp, 1e-6, 1e-6)] {
        let (cw, ch) = View::native(policy, pw, ph, dpr, ui).canvas();
        assert!(cw > 0 && ch > 0 && cw <= pw as i32 * 8 && ch <= ph as i32 * 8, "{policy:?}: {cw}x{ch}");
    }
    for (pw, ph) in [(1, 1), (1, 4000), (4000, 1)] {
        for policy in [CanvasPolicy::default(), FIT, CanvasPolicy::Dp] {
            let (cw, ch) = View::native(policy, pw, ph, 1.0, 1.0).canvas();
            assert!(cw >= 1 && ch >= 1, "{policy:?} {pw}x{ph}");
        }
    }
}

#[test]
fn the_manifests_canvas_reads_each_policy() {
    let good = [
        (Value::Null, CanvasPolicy::default()),
        (json!({}), CanvasPolicy::default()),
        (json!({"policy": "fixed"}), CanvasPolicy::default()),
        (json!({"policy": "fit"}), FIT),
        (json!({"policy": "dp"}), CanvasPolicy::Dp),
        (json!({"policy": "fit", "w": 1920, "h": 1200}), CanvasPolicy::Fit { w: 1920, h: 1200 }),
        (json!({"w": 640, "h": 360}), CanvasPolicy::Fixed { w: 640, h: 360 }),
        (json!({"w": 2560}), CanvasPolicy::Fixed { w: 2560, h: GRID.1 }),
    ];
    for (v, want) in good {
        assert_eq!(CanvasPolicy::read(&v), Ok(want), "{v}");
        assert_eq!(CanvasPolicy::read(&json!({"policy": want.name()})).map(|p| p.name()), Ok(want.name()), "the name reads back");
    }
}

#[test]
fn a_bad_canvas_is_refused_saying_what() {
    let max = i64::from(WINDOW_SIDE_MAX);
    let bad = [
        (json!(7), "object"),
        (json!("fixed"), "object"),
        (json!({"policy": "stretch"}), "policy"),
        (json!({"policy": 3}), "policy"),
        (json!({"policy": null}), "policy"),
        (json!({"size": 3}), "unknown key 'size'"),
        (json!({"w": 1000}), "multiple"),
        (json!({"w": 0}), "w must"),
        (json!({"w": -1280}), "w must"),
        (json!({"h": 0}), "h must"),
        (json!({"w": 1280.5}), "w must"),
        (json!({"w": "1280"}), "w must"),
        (json!({"h": max + 1}), "h must"),
        (json!({"w": (max + 640) / 640 * 640}), "w must"),
        (json!({"w": 2147483648i64}), "w must"),
        (json!({"policy": "dp", "w": 1280}), "dp takes no"),
    ];
    for (v, why) in bad {
        let e = CanvasPolicy::read(&v).expect_err(&v.to_string());
        assert!(e.contains(why), "{v}: {e}");
    }
}

#[test]
fn the_manifests_window_sizes_read_or_are_refused() {
    assert_eq!(read_window_sizes(&Value::Null).unwrap(), WINDOW_SIZES.to_vec(), "absent: today's list");
    assert_eq!(read_window_sizes(&json!(["1280x720", "1920x1080"])).unwrap(), ["1280x720", "1920x1080"]);
    let long: Vec<String> = (0..=WINDOW_SIZES_MAX).map(|i| format!("{}x720", 1280 + i)).collect();
    let bad = [json!("1280x720"), json!([]), json!(["1280"]), json!(["1280x0"]), json!(["0x720"]), json!([1280]), json!(["1280x720", "1280x720"]), json!([format!("{}x720", WINDOW_SIDE_MAX + 1)]), json!(["x720"]), json!(["1280x720x2"]), json!(long)];
    for v in bad {
        assert!(read_window_sizes(&v).is_err(), "{v}");
    }
    assert!(read_window_sizes(&json!(long[..WINDOW_SIZES_MAX])).is_ok(), "the limit itself is allowed");
}

#[test]
fn a_bad_value_is_a_notice_and_the_default() {
    let (policy, sizes, notices) = read_window_options(&Value::Null, &Value::Null);
    assert_eq!((policy, sizes, notices.len()), (CanvasPolicy::default(), WINDOW_SIZES.map(String::from).to_vec(), 0));
    let (policy, sizes, notices) = read_window_options(&json!({"policy": "fit"}), &json!(["1280x720"]));
    assert_eq!((policy, sizes, notices.len()), (FIT, vec!["1280x720".to_string()], 0));
    let (policy, sizes, notices) = read_window_options(&json!({"w": 7}), &json!(7));
    assert_eq!((policy, sizes), (CanvasPolicy::default(), WINDOW_SIZES.map(String::from).to_vec()));
    assert_eq!(notices.len(), 2, "{notices:?}");
    assert!(notices[0].contains("canvas") && notices[1].contains("window_sizes"), "{notices:?}");
    assert!(notices.iter().all(|n| n.contains("default")), "{notices:?}");
}

#[test]
fn settings_remember_the_sizes_a_game_offers() {
    let scratch = std::env::temp_dir().join(format!("aegis_canvas_policy_{}.json", std::process::id()));
    let mut s = Settings::load(scratch, json!({}));
    assert_eq!(s.window_sizes(), WINDOW_SIZES, "nothing said yet: the engine's list");
    s.v["window_size"] = json!("960x540");
    s.keep_window_size(&["1280x720", "1920x1080"]);
    assert_eq!(s.window_sizes(), ["1280x720", "1920x1080"]);
    assert_eq!(s.v["window_size"], json!("1280x720"), "the nearest listed, as ever");
    s.keep_window_size(&[]);
    assert_eq!(s.window_sizes(), ["1280x720", "1920x1080"], "an empty list changes nothing");
}
