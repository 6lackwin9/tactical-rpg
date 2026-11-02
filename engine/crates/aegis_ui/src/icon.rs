//! The window's icon (ICON1): the game's manifest names a PNG (`game.json` `icon`), found over the asset layers when the
//! window opens and averaged down to the sizes the desktop shows. Nothing named means no icon; a named file that is
//! missing or unreadable is a start notice, never a crash. A game's `.exe` icon is its own build's business.

use crate::art::load_png;
use crate::canvas::Image;
use crate::painter::sharpen;
use aegis_core::assets::Layers;
use winit::window::Icon;

/// The largest side an icon is reduced to.
pub const ICON_MAX: i32 = 256;
/// The sides Windows gets (its title bar's small icon and the taskbar's big one, which it scales itself and scales
/// badly from far above its own size). Other systems get one icon, `ICON_MAX`.
pub const WINDOWS_SMALL: i32 = 32;
pub const WINDOWS_BIG: i32 = 64;
/// An unsharp mask after averaging: thin lines of detailed art survive the reduction.
const SHARPEN: f32 = 0.8;

/// The icons for a new window. `taskbar` is used on Windows only.
pub struct WindowIcons {
    pub window: Icon,
    pub taskbar: Icon,
}

/// `img` averaged so its longest side is at most `max` (the shape kept, each side at least 1, a smaller image unchanged).
pub fn fit(img: &Image, max: i32) -> Image {
    let long = img.w.max(img.h);
    if long <= max {
        return Image { w: img.w, h: img.h, px: img.px.clone() };
    }
    let mut out = img.downsample((img.w * max / long).max(1), (img.h * max / long).max(1));
    sharpen(&mut out, SHARPEN);
    out
}

/// The PNG `rel` over the layers, as icons. `rel` empty: Ok(None). A missing or unreadable file: the notice to show.
pub fn window_icons(layers: &Layers, rel: &str) -> Result<Option<WindowIcons>, String> {
    if rel.is_empty() {
        return Ok(None);
    }
    let Some(path) = layers.find(rel) else { return Err(format!("Window icon '{rel}' (game.json) is in no asset layer: no icon.")) };
    let Some(img) = load_png(&path).filter(|i| i.w > 0 && i.h > 0) else { return Err(format!("Window icon {}: not a readable PNG: no icon.", path.display())) };
    let icon = |max: i32| {
        let small = fit(&img, max);
        Icon::from_rgba(small.px, small.w as u32, small.h as u32).map_err(|e| format!("Window icon {}: {e}: no icon.", path.display()))
    };
    Ok(Some(WindowIcons { window: icon(if cfg!(windows) { WINDOWS_SMALL } else { ICON_MAX })?, taskbar: icon(WINDOWS_BIG)? }))
}
