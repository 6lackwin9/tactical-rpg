//! The web build's display (WEBREL1-E2, DISPLAY1 rule 8): the page canvas carries no size of the desktop window's
//! (so the page's CSS sizes it and the game scales to the window and to fullscreen), the Display rows each platform
//! offers, and how the web's session-only Fullscreen asks the browser or follows it. The shell's browser calls are in
//! `shell.rs` (`apply_settings`, `follow_fullscreen`), over these rules.

use aegis_ui::painter::View;
use aegis_ui::screen::{display_keys, Settings, WebFullscreen};
use aegis_ui::shell::web_attributes_of;
use serde_json::json;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::window::Window;

/// The desktop's window attributes (the shell's `resumed` sets a size and a minimum) lose every size on the web, so
/// winit writes no inline CSS width, height or minimum onto the canvas.
#[test]
fn the_web_canvas_gets_no_size_of_its_own() {
    let desktop = Window::default_attributes().with_inner_size(PhysicalSize::new(1280, 720)).with_min_inner_size(PhysicalSize::new(640, 360)).with_max_inner_size(LogicalSize::new(4000.0, 3000.0));
    let web = web_attributes_of(desktop.clone());
    assert!(desktop.inner_size.is_some() && desktop.min_inner_size.is_some(), "the desktop keeps its sizes");
    assert_eq!((web.inner_size, web.min_inner_size, web.max_inner_size), (None, None, None));
    assert_eq!(web.title, desktop.title, "nothing else changes");
}

/// The canvas filling a bigger page or screen scales the whole-window view up as a resized desktop window does
/// (`View::fit` on the buffer, which is the CSS size times the device pixel ratio): a 2560 x 1440 page at ratio 1
/// and a 1280 x 720 page at ratio 2 both draw the 1280 x 720 layout at scale 2.
#[test]
fn a_bigger_canvas_scales_the_game_up() {
    let base = View::fit(1280, 720);
    let full = View::fit(2560, 1440);
    assert_eq!(full.s, base.s * 2.0);
    assert_eq!(full.to_logical(2559.0, 1439.0), base.to_logical(1279.5, 719.5), "the far corner is the same point of the layout");
}

/// The desktop offers every DISPLAY1 row; the web only Fullscreen and Text size, each an engine settings key.
#[test]
fn each_platform_offers_its_display_rows() {
    let desktop = display_keys(false);
    let web = display_keys(true);
    assert_eq!(desktop, ["fullscreen", "borderless", "window_size", "vsync", "fps_cap", "text_scale"]);
    assert_eq!(web, ["fullscreen", "text_scale"]);
    assert!(web.iter().all(|k| desktop.contains(k)), "the web's rows are desktop rows");
    let defaults = Settings::engine_defaults();
    assert!(desktop.iter().all(|k| defaults.get(*k).is_some()), "every row is an engine key with a default");
}

/// A changed setting asks the browser only when the page is not there yet; an unchanged one follows the page (Esc
/// leaves fullscreen without the game); a match does nothing.
#[test]
fn web_fullscreen_asks_on_a_change_and_follows_the_page() {
    // the first apply after start (the setting is forced off for the session): nothing to ask on a windowed page
    assert_eq!(WebFullscreen::step(false, None, false), WebFullscreen::Keep);
    // Apply with Fullscreen on
    assert_eq!(WebFullscreen::step(true, Some(false), false), WebFullscreen::Ask(true));
    // the browser entered it: nothing more
    assert_eq!(WebFullscreen::step(true, Some(true), true), WebFullscreen::Keep);
    // Esc left it: the setting follows
    assert_eq!(WebFullscreen::step(true, Some(true), false), WebFullscreen::Follow(false));
    // Apply with Fullscreen off while fullscreen
    assert_eq!(WebFullscreen::step(false, Some(true), true), WebFullscreen::Ask(false));
    // a fullscreen the game did not ask for (the browser's own) is followed, not undone
    assert_eq!(WebFullscreen::step(false, Some(false), true), WebFullscreen::Follow(true));
    // the setting changed to what the page already shows: no request
    assert_eq!(WebFullscreen::step(true, Some(false), true), WebFullscreen::Keep);
}

/// The key the web's checkbox shares with the desktop is off by default (the web also turns it off at every start).
#[test]
fn fullscreen_defaults_off() {
    assert_eq!(Settings::engine_defaults()["fullscreen"], json!(false));
}

/// WEBFIX2 (d): the drawing buffer is the canvas's size while it fits the GPU's largest texture; past it the buffer
/// shrinks evenly (the aspect kept to a px) until its longer side is that limit; no limit known (0) keeps the size.
#[test]
fn the_buffer_is_capped_at_the_gpus_largest_texture() {
    use aegis_ui::shell::capped_buffer;
    for (w, h) in [(1280, 720), (2560, 1440), (8192, 8192), (1, 1)] {
        assert_eq!(capped_buffer(w, h, 8192), (w, h), "{w} x {h} fits");
        assert_eq!(capped_buffer(w, h, 0), (w, h), "{w} x {h} with no limit known");
    }
    assert_eq!(capped_buffer(0, 0, 8192), (1, 1), "never an empty buffer");
    // a 2560 x 1440 CSS page at a pixel ratio of 4, and a tall one
    for (w, h, max) in [(10240, 5760, 8192), (5760, 10240, 8192), (16384, 9216, 4096), (20000, 30, 2048)] {
        let (bw, bh) = capped_buffer(w, h, max);
        assert_eq!(bw.max(bh), max, "{w} x {h} under {max}: the longer side is the limit");
        assert!(bw <= max && bh <= max && bw >= 1 && bh >= 1);
        let (aspect, kept) = (w as f64 / h as f64, bw as f64 / bh as f64);
        let short = bw.min(bh) as f64;
        assert!((aspect - kept).abs() / aspect <= 1.0 / short + 1e-9, "{w} x {h} -> {bw} x {bh}: the aspect is kept");
    }
}
