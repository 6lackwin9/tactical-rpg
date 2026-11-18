//! SHELL-SPLIT1 (ENGINE-3): the shell's settings, moved from `shell.rs` (behaviour unchanged; moved items are `pub(super)`).
//! Settings: the window options a game declares, and applying changed settings to the open window.

use super::*;

/// Holds the saved window size to what the manifest declared (`declared` as written, `offered` as `read_window_options`
/// read it). A manifest with no such key says nothing, so it overrides nothing the game gave `keep_window_size` itself.
pub(super) fn hold_declared_sizes(settings: &mut Settings, declared: &Value, offered: &[String]) {
    if !declared.is_null() {
        settings.keep_window_size(&offered.iter().map(String::as_str).collect::<Vec<_>>());
    }
}

/// What a game's options say of its canvas and windows (LAYOUT720-E5): the policy, the sizes it offers and a notice for
/// each value refused.
pub fn read_window_options(canvas: &Value, window_sizes: &Value) -> (CanvasPolicy, Vec<String>, Vec<String>) {
    let mut notices = Vec::new();
    let policy = CanvasPolicy::read(canvas).unwrap_or_else(|e| {
        notices.push(format!("game.json: {e}; using the default canvas."));
        CanvasPolicy::default()
    });
    let sizes = read_window_sizes(window_sizes).unwrap_or_else(|e| {
        notices.push(format!("game.json: {e}; using the default window sizes."));
        WINDOW_SIZES.map(String::from).to_vec()
    });
    (policy, sizes, notices)
}

/// The settings' text size.
pub(super) fn text_scale_of(v: &Value) -> f32 {
    v["text_scale"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0) as f32
}

impl<G: Context> App<G> {
    /// Applies the engine's settings (DisplaySettings) when they changed since the last time: the window's show
    /// (DISPLAY2: borderless fullscreen over the monitor, a frameless centred window, or a framed one, the last two at the
    /// chosen size capped to the usable area), vsync, the frame-rate cap and the text size. The window is sized only when
    /// its show or the size setting changed, so a window the player resized keeps its size.
    pub(super) fn apply_settings(&mut self) {
        let (Some(shell), Some(window)) = (self.shell.as_mut(), self.window.as_ref()) else { return };
        let v = shell.cx.settings.v.clone();
        if self.applied.as_ref() == Some(&v) {
            return;
        }
        // The web (WEBREL1-E2): the page owns the canvas and its size, so the window size and borderless settings do not
        // apply; Fullscreen (DISPLAY1 rule 8) asks the browser, here inside the click that applied it (browsers allow
        // fullscreen only during a user gesture).
        #[cfg(target_arch = "wasm32")]
        {
            let want = WindowShow::of(&v) == WindowShow::Fullscreen;
            let asked = self.applied.as_ref().map(|a| WindowShow::of(a) == WindowShow::Fullscreen);
            if let WebFullscreen::Ask(on) = WebFullscreen::step(want, asked, window.fullscreen().is_some()) {
                window.set_fullscreen(on.then_some(Fullscreen::Borderless(None)));
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let show = WindowShow::of(&v);
        #[cfg(not(target_arch = "wasm32"))]
        if self.applied.as_ref().is_none_or(|a| WindowShow::of(a) != show || a["window_size"] != v["window_size"]) {
            if show == WindowShow::Fullscreen {
                if window.fullscreen().is_none() {
                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
            } else {
                window.set_fullscreen(None);
                window.set_decorations(show == WindowShow::Windowed);
                if let Some(note) = size_and_place(window, &v["window_size"], show) {
                    shell.cx.app.note(note);
                }
            }
        }
        if let Some(p) = self.pixels.as_mut() {
            p.enable_vsync(v["vsync"].as_bool().unwrap_or(true));
        }
        self.frame_every = v["fps_cap"].as_str().and_then(|c| c.parse::<u64>().ok()).filter(|n| *n > 0).map(|n| Duration::from_micros(1_000_000 / n));
        shell.gfx.fonts.scale = text_scale_of(&v);
        self.applied = Some(v);
    }
}
