//! LOOK1-ME: the engine's menu panes (spec LOOK1 section 6), each a standalone piece a screen composes and the UI
//! Gallery's Kit page shows on its own (the Panes page): `backdrop` (a full-screen picture, cover fit, with a vignette
//! and a veil, composed once and blitted after; the screen's L0), `menu_column` (large menu buttons with hotkeys and
//! one primary, or text rows on a picture: `MenuStyle`, under a `menu_title` block), `tab_rail` (a vertical tab rail
//! and its content area, settings style), `slot_card` (a save
//! or load card: thumbnail, title and lines, a portrait row, a timestamp, corner icons) and `ask_dialog` (the themed
//! yes / no question in the `ornate` or `panel` frame). They draw only through the theme in force (`Ui::t`: colour
//! roles, faces, layout values, frame roles), take every text and picture from the caller (they name no game), record
//! their widgets for the agent link and the keyboard focus like every widget (ids `<id>.<key>` or `<id>.<i>`), and lay
//! the rects they place themselves on the theme's grid (`layout.grid`; the fixed spec offsets, e.g. the slot card's
//! 112 x 63 thumbnail at `padding`, are drawn as given).

use crate::canvas::{Canvas, Image, Rect, Rgba};
use crate::painter::{Painter, Typeface};
use crate::screen::Key;
use crate::ui::{tip, Glyph, Tip, Ui, BODY, HEAD, SMALL, TITLE};
use std::cell::{Cell, RefCell};

/// The ask dialog's least size at the Normal text size (logical px; the spec's 600 x 300 at the 1280 x 720
/// reference window); it grows with the text size and the question.
pub const ASK_MIN_W: i32 = 300;
pub const ASK_MIN_H: i32 = 150;
/// The tab rail's width (settings: 120) and its rows (18 high, one every 20).
pub const RAIL_W: i32 = 120;
pub const TAB_H: i32 = 18;
pub const TAB_STEP: i32 = 20;
/// The slot card's thumbnail (16:9), portrait size and corner icon hit box (a 12 px icon inside).
pub const THUMB_W: i32 = 112;
pub const THUMB_H: i32 = 63;
pub const PORTRAIT: i32 = 20;
pub const CORNER: i32 = 16;

/// `v` rounded to the nearest step of the theme's grid (`layout.grid`).
pub fn snap(v: i32) -> i32 {
    let g = crate::theme::th().layout.grid.max(1);
    (v + g / 2).div_euclid(g) * g
}

/// `v` rounded up to the theme's grid (sizes that must hold their content).
pub fn snap_up(v: i32) -> i32 {
    let g = crate::theme::th().layout.grid.max(1);
    (v + g - 1).div_euclid(g) * g
}

/// `v` rounded down to the theme's grid (sizes that must stay inside their room).
pub fn snap_down(v: i32) -> i32 {
    let g = crate::theme::th().layout.grid.max(1);
    v.div_euclid(g) * g
}

/// `s` shortened with "..." to fit `max_w` in `face` (`Painter::fit` for any face).
fn fit_face(p: &mut Painter, face: Typeface, s: &str, size: f32, max_w: f32) -> String {
    if p.text_w_face(face, s, size) <= max_w {
        return s.to_string();
    }
    let mut out = s.to_string();
    while !out.is_empty() && p.text_w_face(face, &format!("{out}..."), size) > max_w {
        out.pop();
    }
    format!("{}...", out.trim_end())
}

/// Draws `s` in `face` fitted to `max_w` (marked cut for the text audit when shortened); returns whether it was cut.
#[allow(clippy::too_many_arguments)]
fn text_cut(p: &mut Painter, face: Typeface, x: f32, y: f32, s: &str, size: f32, max_w: f32, col: Rgba) -> bool {
    let t = fit_face(p, face, s, size, max_w.max(0.0));
    p.text_face(face, x, y, &t, size, col);
    if t != s {
        p.mark_cut(s);
    }
    t != s
}

// ---- Backdrop ----------------------------------------------------------------------------------------------------

/// The part of an `iw` x `ih` image a cover fit shows in `r` (image px): the image scaled by max(r.w / iw, r.h / ih)
/// so it fills `r`, centred, the overflow cropped.
pub fn cover_crop(iw: i32, ih: i32, r: Rect) -> Rect {
    if iw <= 0 || ih <= 0 || r.w <= 0 || r.h <= 0 {
        return Rect::new(0, 0, iw.max(0), ih.max(0));
    }
    let k = (r.w as f32 / iw as f32).max(r.h as f32 / ih as f32);
    let (sw, sh) = (((r.w as f32 / k).round() as i32).clamp(1, iw), ((r.h as f32 / k).round() as i32).clamp(1, ih));
    Rect::new((iw - sw) / 2, (ih - sh) / 2, sw, sh)
}

/// `img` cover-fitted into logical `r` (`cover_crop`, nearest sampling), then a vignette: `bg` toward the corners
/// up to `vignette` x 255 alpha (0-1; the square of the distance from the centre, 1 at a corner).
fn cover_image(p: &mut Painter, img: Option<&Image>, r: Rect, vignette: f32, bg: Rgba) {
    let pr = p.v.rect(r);
    cover_pixels(p.c, img, r, pr, vignette, bg);
}

/// `cover_image` onto canvas `c` at physical `pr`.
fn cover_pixels(c: &mut Canvas, img: Option<&Image>, r: Rect, pr: Rect, vignette: f32, bg: Rgba) {
    if pr.w <= 0 || pr.h <= 0 {
        return;
    }
    let crop = img.map(|i| (i, cover_crop(i.w, i.h, r)));
    let vig = (vignette.clamp(0.0, 1.0) * 255.0).round();
    let (hw, hh) = (pr.w as f32 / 2.0, pr.h as f32 / 2.0);
    for y in 0..pr.h {
        let dy = (y as f32 + 0.5 - hh) / hh;
        for x in 0..pr.w {
            let (dx, at) = ((x as f32 + 0.5 - hw) / hw, (pr.x + x, pr.y + y));
            match crop {
                Some((img, part)) => {
                    let (sx, sy) = (part.x + x * part.w / pr.w, part.y + y * part.h / pr.h);
                    let i = ((sy * img.w + sx) * 4) as usize;
                    c.blend(at.0, at.1, [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]);
                }
                None => c.blend(at.0, at.1, [bg[0], bg[1], bg[2], 255]),
            }
            if vig > 0.0 {
                let d = ((dx * dx + dy * dy) / 2.0).min(1.0);
                let a = (vig * d).round() as u8;
                c.blend(at.0, at.1, [bg[0], bg[1], bg[2], a]);
            }
        }
    }
}

/// The backdrop (L0): `img` over the whole of `r` with a cover fit (scale max(r.w / w, r.h / h), centred crop,
/// nearest sampling; None: the `bg` colour), a vignette darkening toward the corners with `bg` up to `vignette` x 255
/// alpha (0-1), then `bg` at `veil` alpha over all (the spec's 50 % veil for menus is 128). Composed once and blitted
/// on later frames (it would cost tens of ms a frame at full window size), keyed on the picture, the physical size, the
/// veil, the vignette and `bg`, so a resize or a theme change composes again; the blit honours `Canvas::clip` and
/// `bounds`. A picture with see-through pixels shows what lies under it, so it is drawn live every frame.
pub fn backdrop(p: &mut Painter, img: Option<&Image>, r: Rect, veil: u8, vignette: f32) {
    let bg = crate::theme::th().bg;
    let pr = p.v.rect(r);
    if pr.w <= 0 || pr.h <= 0 {
        return;
    }
    let key = ComposedKey { picture: img.map(picture_id), logical: (r.w, r.h), size: (pr.w, pr.h), veil, vignette: vignette.to_bits(), bg };
    let opaque = COMPOSED.with(|cache| {
        let mut cache = cache.borrow_mut();
        let entry = match cache.iter().position(|e| e.key == key) {
            Some(at) => cache.remove(at),
            None => {
                COMPOSES.with(|n| n.set(n.get() + 1));
                Composed { key, px: compose(img, r, pr, veil, vignette, bg) }
            }
        };
        if let Some(px) = &entry.px {
            blit(p.c, pr, px);
        }
        let opaque = entry.px.is_some();
        cache.push(entry);
        if cache.len() > COMPOSED_MAX {
            cache.remove(0);
        }
        opaque
    });
    if opaque {
        p.cover(r, veil);
    } else {
        backdrop_live(p, img, r, veil, vignette);
    }
}

/// `backdrop` drawn pixel by pixel, without the composed copy (a see-through picture; the reference the blit matches).
pub fn backdrop_live(p: &mut Painter, img: Option<&Image>, r: Rect, veil: u8, vignette: f32) {
    let bg = crate::theme::th().bg;
    cover_image(p, img, r, vignette, bg);
    if veil > 0 {
        p.fill(r, [bg[0], bg[1], bg[2], veil]);
    }
}

/// How many composed backdrops are kept (a screen has one; the Gallery's previews a few), the oldest dropped first.
/// The cost: each is a full window of RGBA held until it is pushed out, never freed otherwise: 3.7 MB at 1280 x 720,
/// 8.3 MB at 1920 x 1080, 33 MB at 3840 x 2160, so at most four times that (a resize leaves the old sizes in until then).
const COMPOSED_MAX: usize = 4;

/// What a composed backdrop depends on. Its place is not part of it: the blit puts it anywhere.
#[derive(Clone, Copy, PartialEq)]
struct ComposedKey {
    picture: Option<(usize, i32, i32, u64)>,
    logical: (i32, i32),
    size: (i32, i32),
    veil: u8,
    vignette: u32,
    bg: Rgba,
}

/// A composed backdrop's pixels (None: its picture is see-through, `backdrop_live` draws it).
struct Composed {
    key: ComposedKey,
    px: Option<Vec<u8>>,
}

thread_local! {
    static COMPOSED: RefCell<Vec<Composed>> = const { RefCell::new(Vec::new()) };
    static COMPOSES: Cell<u64> = const { Cell::new(0) };
}

/// How many times this thread has composed a backdrop (a cache miss); tests and timings read it.
pub fn backdrop_composes() -> u64 {
    COMPOSES.with(|n| n.get())
}

/// A picture's identity: where it lives, its size and a hash of up to 256 pixels across it, so another image that lands
/// on a freed one's address does not pass for it (an edit to pixels the hash does not sample would).
pub(crate) fn picture_id(img: &Image) -> (usize, i32, i32, u64) {
    let n = img.px.len() / 4;
    let step = (n / 256).max(1);
    let mut h: u64 = 0xcbf29ce484222325;
    for i in (0..n).step_by(step) {
        h = (h ^ u32::from_le_bytes([img.px[i * 4], img.px[i * 4 + 1], img.px[i * 4 + 2], img.px[i * 4 + 3]]) as u64).wrapping_mul(0x100000001b3);
    }
    (img.px.as_ptr() as usize, img.w, img.h, h)
}

/// The backdrop's pixels at physical size `pr` (row by row, opaque), the same blends as `backdrop_live`; None when the
/// part of the picture it shows has see-through pixels.
fn compose(img: Option<&Image>, r: Rect, pr: Rect, veil: u8, vignette: f32, bg: Rgba) -> Option<Vec<u8>> {
    if let Some(i) = img {
        let c = cover_crop(i.w, i.h, r);
        for y in c.y..c.y + c.h {
            let row = ((y * i.w + c.x) * 4) as usize..((y * i.w + c.x + c.w) * 4) as usize;
            if i.px[row].as_chunks::<4>().0.iter().any(|p| p[3] != 255) {
                return None;
            }
        }
    }
    let mut c = Canvas::new(pr.w, pr.h);
    let whole = Rect::new(0, 0, pr.w, pr.h);
    cover_pixels(&mut c, img, r, whole, vignette, bg);
    if veil > 0 {
        c.fill(whole, [bg[0], bg[1], bg[2], veil]);
    }
    Some(c.px)
}

/// Copies opaque pixels (`pr.w` wide) to `pr` on `c`, inside the canvas, its clip and its bounds.
fn blit(c: &mut Canvas, pr: Rect, px: &[u8]) {
    let mut v = pr.intersect(Rect::new(0, 0, c.w, c.h));
    for limit in [c.clip, c.bounds].into_iter().flatten() {
        v = v.intersect(limit);
    }
    if v.w <= 0 || v.h <= 0 {
        return;
    }
    let n = (v.w * 4) as usize;
    for y in v.y..v.y + v.h {
        let from = (((y - pr.y) * pr.w + v.x - pr.x) * 4) as usize;
        let to = ((y * c.w + v.x) * 4) as usize;
        c.px[to..to + n].copy_from_slice(&px[from..from + n]);
    }
}

/// `backdrop` recorded for the agent link: a widget `id` of kind `picture` (no action, no focus) whose text is
/// `name` (the picture's path, as the screen knows it).
pub fn backdrop_pane<A: Clone>(ui: &mut Ui<A>, id: &str, img: Option<&Image>, name: &str, r: Rect, veil: u8, vignette: f32) {
    backdrop(&mut ui.p, img, r, veil, vignette);
    ui.push(id, "picture", name, r, true, false, None, Vec::new());
}

// ---- A pane button (menu items, the ask's answers) ----------------------------------------------------------------

/// A button of the panes: the `button` frame roles as `Ui::button_full` picks them, the label (`left`: at the
/// padding, else centred) and an optional hotkey right-aligned in `dim`. `primary` draws the screen's one primary
/// action: the `focus` border (and the `layout.focus_glow` ring) even while idle.
#[allow(clippy::too_many_arguments)]
fn pane_button<A: Clone>(ui: &mut Ui<A>, id: &str, r: Rect, label: &str, hotkey: Option<char>, action: A, enabled: bool, primary: bool, size: f32, left: bool, t: Tip) {
    let th = ui.t.clone();
    let hover = enabled && ui.hover(r);
    let fill = if hover { th.card_hot } else { th.card };
    let border = if enabled && (hover || primary) { th.focus } else if enabled { th.edge_live } else { th.edge };
    let role = match (enabled, hover) {
        (false, _) => "button_off",
        (true, true) => "button_hot",
        (true, false) => "button",
    };
    let glow = th.layout.focus_glow;
    if glow > 0 && enabled && (hover || primary) {
        ui.p.panel(Rect::new(r.x - glow, r.y - glow, r.w + 2 * glow, r.h + 2 * glow), [0, 0, 0, 0], th.focus, th.layout.radius + glow as f32);
    }
    ui.p.skin(th.frame(role), r, fill, border, th.layout.radius);
    let pad = th.layout.padding as f32;
    let col = if enabled { th.ink_on(role) } else { th.dim };
    let mut room = r.w as f32 - 2.0 * pad;
    if let Some(k) = hotkey {
        let hk = k.to_uppercase().to_string();
        let lh = ui.p.line_h(SMALL);
        let w = ui.p.text_w(&hk, SMALL);
        ui.p.text((r.x + r.w) as f32 - pad - w, r.y as f32 + (r.h as f32 - lh) / 2.0, &hk, SMALL, th.dim);
        room -= w + th.layout.gutter as f32;
    }
    let lh = ui.p.line_h(size);
    let y = r.y as f32 + (r.h as f32 - lh) / 2.0;
    let shown = ui.p.fit(label, size, room.max(0.0));
    if left {
        ui.p.text(r.x as f32 + pad, y, &shown, size, col);
    } else {
        // Centred in the room left of the hotkey (the whole button without one).
        ui.p.text_center(r.x as f32 + pad + room.max(0.0) / 2.0, y, &shown, size, col);
    }
    if shown != label {
        ui.p.mark_cut(label);
    }
    // A cut label reads in full on hover (a caller's hover card already leads with it: `menu_column`).
    let t = if shown != label && t.is_empty() { tip(label, "") } else { t };
    ui.push(id, "button", label, r, enabled, false, Some(action), t);
}

/// The part of a `MenuStyle::Text` row that takes the mouse: its label's measured width (the room left of a hotkey at
/// most) plus `MENU_TEXT_MARGIN` each side, at the row's height.
fn text_hit<A: Clone>(ui: &mut Ui<A>, r: Rect, label: &str, hotkey: Option<char>, size: f32) -> Rect {
    let pad = ui.t.layout.padding;
    let mut room = r.w as f32 - 2.0 * pad as f32;
    if let Some(k) = hotkey {
        room -= ui.p.text_w(&k.to_uppercase().to_string(), SMALL) + ui.t.layout.gutter as f32;
    }
    let w = ui.p.text_w(label, size).min(room.max(0.0)).ceil() as i32;
    Rect::new(r.x + pad - MENU_TEXT_MARGIN, r.y, w + 2 * MENU_TEXT_MARGIN, r.h)
}

/// A row of `MenuStyle::Text` laid out in `r`, clicked in `hit` (`text_hit`): no panel and no marker, the label (left, at
/// the padding) and hotkey with a 1 px `bg` shadow. Its text is `text` at `MENU_TEXT_IDLE` alpha, `dim` when disabled and
/// `gold` when it is the active item: the one with the keyboard focus, which the mouse moves too (`focus::hover_target`),
/// so one item is lit at a time; the mouse lights a row by itself only while no focus shows.
#[allow(clippy::too_many_arguments)]
fn text_row<A: Clone>(ui: &mut Ui<A>, id: &str, r: Rect, hit: Rect, label: &str, hotkey: Option<char>, action: A, enabled: bool, size: f32, t: Tip) {
    let th = ui.t.clone();
    let lit = enabled && (ui.focused(id) || (!ui.focus_shown() && ui.hover(hit)));
    let mut col = if !enabled { th.dim } else if lit { th.gold } else { th.text };
    if enabled && !lit {
        col[3] = MENU_TEXT_IDLE;
    }
    let pad = th.layout.padding as f32;
    let mut room = r.w as f32 - 2.0 * pad;
    if let Some(k) = hotkey {
        let hk = k.to_uppercase().to_string();
        let (lh, w) = (ui.p.line_h(SMALL), ui.p.text_w(&hk, SMALL));
        let (hx, hy) = ((r.x + r.w) as f32 - pad - w, r.y as f32 + (r.h as f32 - lh) / 2.0);
        ui.p.text(hx + 1.0, hy + 1.0, &hk, SMALL, th.bg);
        ui.p.text(hx, hy, &hk, SMALL, th.dim);
        room -= w + th.layout.gutter as f32;
    }
    let y = r.y as f32 + (r.h as f32 - ui.p.line_h(size)) / 2.0;
    let shown = ui.p.fit(label, size, room.max(0.0));
    ui.p.text(r.x as f32 + pad + 1.0, y + 1.0, &shown, size, th.bg);
    ui.p.text(r.x as f32 + pad, y, &shown, size, col);
    if shown != label {
        ui.p.mark_cut(label);
    }
    let t = if shown != label && t.is_empty() { tip(label, "") } else { t };
    ui.push(id, "text_button", label, hit, enabled, false, Some(action), t);
}

// ---- MenuColumn ----------------------------------------------------------------------------------------------------

/// How a `menu_column` looks: `Framed`, buttons with the first enabled one the primary, or `Text`, rows of text only for
/// a menu that sits on a picture (`text_row`: only the active item stands out, by brighter text; the mouse takes only
/// the label, not the whole row).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MenuStyle {
    #[default]
    Framed,
    Text,
}

/// A `MenuStyle::Text` row's idle text alpha (the `text` role, dimmer than the active item's `gold`).
pub const MENU_TEXT_IDLE: u8 = 190;
/// How far past its label a `MenuStyle::Text` row takes the mouse, each side.
pub const MENU_TEXT_MARGIN: i32 = 3;

/// One item of a `menu_column`: its id key, label, hotkey (a letter typed on the screen, `menu_key`), action,
/// whether it can be used and its hover card text (on a disabled item, the reason it can't).
#[derive(Debug, Clone)]
pub struct MenuItem<A> {
    pub key: String,
    pub label: String,
    pub hotkey: Option<char>,
    pub action: A,
    pub enabled: bool,
    pub tip: String,
}

impl<A> MenuItem<A> {
    pub fn new(key: &str, label: &str, hotkey: Option<char>, action: A) -> Self {
        Self { key: key.into(), label: label.into(), hotkey, action, enabled: true, tip: String::new() }
    }

    /// The item disabled, `reason` on its hover card.
    pub fn off(mut self, reason: &str) -> Self {
        self.enabled = false;
        self.tip = reason.into();
        self
    }

    pub fn with_tip(mut self, t: &str) -> Self {
        self.tip = t.into();
        self
    }
}

/// The rect of item `i` of a column at (x, y), `w` x `h`, one every `step`.
pub fn menu_rect(x: i32, y: i32, w: i32, h: i32, step: i32, i: usize) -> Rect {
    Rect::new(x, y + i as i32 * step, w, h)
}

/// A menu column's title block (the display face): `title` in `HEAD` and `gold`, `subtitle` (may be empty) under it
/// in `dim`, both fitted to `w`; returns the y below it plus the gutter (where the column can start).
pub fn menu_title<A: Clone>(ui: &mut Ui<A>, x: i32, y: i32, w: i32, title: &str, subtitle: &str) -> i32 {
    let t = ui.t.clone();
    text_cut(&mut ui.p, Typeface::Display, x as f32, y as f32, title, HEAD, w as f32, t.gold);
    let mut yy = y as f32 + ui.p.line_h_face(Typeface::Display, HEAD);
    if !subtitle.is_empty() {
        text_cut(&mut ui.p, Typeface::Body, x as f32, yy, subtitle, BODY, w as f32, t.dim);
        yy += ui.p.line_h(BODY);
    }
    snap_up(yy.ceil() as i32 + t.layout.gutter)
}

/// A vertical list of large menu buttons (`TITLE` size, label left, hotkey right in `dim`) at (x, y), each `w` x `h`,
/// one every `step`; the first enabled item is the primary (`focus` border and `focus_glow`). Ids `<id>.<key>`, in
/// order top-down (the keyboard focus walks them so); a disabled item takes no click and shows its reason on hover.
/// Returns the item under the mouse (enabled or not).
#[allow(clippy::too_many_arguments)]
pub fn menu_column<A: Clone>(ui: &mut Ui<A>, id: &str, x: i32, y: i32, w: i32, h: i32, step: i32, items: &[MenuItem<A>]) -> Option<usize> {
    menu_column_styled(ui, id, x, y, w, h, step, items, MenuStyle::Framed)
}

/// `menu_column` in `style` (`MenuStyle::Text`: widgets of kind `text_button`, same ids and rows, each widget's rect the
/// label's own `text_hit`).
#[allow(clippy::too_many_arguments)]
pub fn menu_column_styled<A: Clone>(ui: &mut Ui<A>, id: &str, x: i32, y: i32, w: i32, h: i32, step: i32, items: &[MenuItem<A>], style: MenuStyle) -> Option<usize> {
    let primary = items.iter().position(|i| i.enabled);
    let mut hovered = None;
    for (i, it) in items.iter().enumerate() {
        let r = menu_rect(x, y, w, h, step, i);
        let hit = if style == MenuStyle::Text { text_hit(ui, r, &it.label, it.hotkey, TITLE) } else { r };
        if ui.hover(hit) {
            hovered = Some(i);
        }
        let t = if it.tip.is_empty() { Vec::new() } else { tip(&it.label, &it.tip) };
        let row = format!("{id}.{}", it.key);
        match style {
            MenuStyle::Framed => pane_button(ui, &row, r, &it.label, it.hotkey, it.action.clone(), it.enabled, primary == Some(i), TITLE, true, t),
            MenuStyle::Text => text_row(ui, &row, r, hit, &it.label, it.hotkey, it.action.clone(), it.enabled, TITLE, t),
        }
    }
    hovered
}

/// The action of the enabled item whose hotkey `k` types (letters match either case).
pub fn menu_key<A: Clone>(items: &[MenuItem<A>], k: Key) -> Option<A> {
    let Key::Char(c) = k else { return None };
    items.iter().find(|i| i.enabled && i.hotkey.is_some_and(|h| h.eq_ignore_ascii_case(&c))).map(|i| i.action.clone())
}

// ---- TabRail -------------------------------------------------------------------------------------------------------

/// The rects of a tab rail in `r`: the rail (`RAIL_W`, at most half of `r`), each tab's row (`TAB_H` high, one every
/// `TAB_STEP`; only the rows that fit, `tab_window` picks what they show) and the content area right of the rail past
/// the gutter.
pub fn tab_rail_rects(r: Rect, n: usize) -> (Vec<Rect>, Rect) {
    let t = crate::theme::th();
    let rail = RAIL_W.min(snap_down(r.w / 2));
    let rows = (0..n).map(|i| Rect::new(r.x, r.y + i as i32 * TAB_STEP, rail, TAB_H)).take_while(|row| row.y + row.h <= r.y + r.h).collect();
    let x = r.x + rail + t.layout.gutter;
    (rows, Rect::new(x, r.y, (r.x + r.w - x).max(0), r.h))
}

/// What a rail of `rows` rows shows of `n` tabs with tab `open` on: the first tab, how many, and whether the first and
/// last rows are the previous / next arrows. All tabs when they fit; past that the rows are a window that always holds
/// the open tab, under arrows when there are at least three rows (with fewer, `tab_key` still walks the tabs).
pub fn tab_window(rows: usize, n: usize, open: usize) -> (usize, usize, bool) {
    if n <= rows {
        return (0, n, false);
    }
    let arrows = rows >= 3;
    let count = if arrows { rows - 2 } else { rows };
    let first = open.min(n - 1).saturating_sub(count.saturating_sub(1) / 2).min(n - count);
    (first, count, arrows)
}

/// A vertical tab rail (settings style) in `r`: `tabs` down the left (`Ui::tab`, frame roles `tab` and `tab_on`; ids
/// `<id>.<i>`; `open` drawn on; a click sends `act(i)`), the content area right of it as an `inset`. Tabs past the
/// rail's height stay in reach: `tab_window` shows those around the open one, between `<id>.prev` and `<id>.next`
/// arrows that open the tab above / below (wrapping, as `tab_key`). Returns the content rect inside the inset, where
/// the screen draws the open tab's form.
pub fn tab_rail<A: Clone, S: AsRef<str>>(ui: &mut Ui<A>, id: &str, r: Rect, tabs: &[S], open: usize, act: impl Fn(usize) -> A) -> Rect {
    let (rows, content) = tab_rail_rects(r, tabs.len());
    let n = tabs.len();
    let (first, count, arrows) = tab_window(rows.len(), n, open);
    let mut row = rows.iter();
    if arrows {
        ui.tab(&format!("{id}.prev"), *row.next().unwrap(), "^", act((open + n - 1) % n), false);
    }
    for (i, tab) in tabs.iter().enumerate().skip(first).take(count) {
        ui.tab(&format!("{id}.{i}"), *row.next().unwrap(), tab.as_ref(), act(i), open == i);
    }
    if arrows {
        ui.tab(&format!("{id}.next"), *row.next().unwrap(), "v", act((open + 1) % n), false);
    }
    ui.inset(content, false)
}

/// The tab a key moves to on a rail of `n` tabs (PageUp / PageDown: the one above / below, wrapping).
pub fn tab_key(n: usize, open: usize, k: Key) -> Option<usize> {
    if n == 0 {
        return None;
    }
    match k {
        Key::PageUp => Some((open + n - 1) % n),
        Key::PageDown => Some((open + 1) % n),
        _ => None,
    }
}

// ---- SlotCard ------------------------------------------------------------------------------------------------------

/// What a slot card shows: the thumbnail (None: an `inset` with the title's initial), the title (the display face),
/// the lines under it (place, day and time; the first in `text`, the rest in `dim`), the portraits along the bottom
/// and the timestamp at the bottom right (may be empty).
#[derive(Debug, Clone, Default)]
pub struct SlotView<'a> {
    pub thumb: Option<&'a Image>,
    pub title: String,
    pub lines: Vec<String>,
    pub portraits: Vec<&'a Image>,
    pub stamp: String,
}

/// An icon at a slot card's top right (lock, delete): id key, label (the agent link's name and the hover title),
/// shape, action, whether it can be used (a disabled one says why on hover: `tip`) and whether it is on.
#[derive(Debug, Clone)]
pub struct Corner<A> {
    pub key: String,
    pub label: String,
    pub glyph: Glyph,
    pub action: A,
    pub enabled: bool,
    pub toggled: bool,
    pub tip: String,
}

/// The rects of a slot card in `r`: the thumbnail (112 x 63 at the padding, centred down; smaller cards keep 16:9),
/// the text's left x (past the thumbnail and the padding), the portrait row's top y and the corner icons' hit boxes
/// (right to left from the top right corner).
pub fn slot_rects(r: Rect, corners: usize) -> (Rect, i32, i32, Vec<Rect>) {
    let pad = crate::theme::th().layout.padding;
    let th_h = THUMB_H.min(r.h - 2 * pad).max(0);
    let th_w = if th_h == THUMB_H { THUMB_W } else { th_h * 16 / 9 };
    let thumb = Rect::new(r.x + pad, r.y + (r.h - th_h) / 2, th_w, th_h);
    let tx = thumb.x + thumb.w + pad;
    let py = r.y + r.h - pad - PORTRAIT;
    let icons = (0..corners).map(|i| Rect::new(r.x + r.w - 2 - CORNER * (corners - i) as i32, r.y + 2, CORNER, CORNER)).collect();
    (thumb, tx, py, icons)
}

/// A save / load card (frame roles `card`, `card_hot`, `card_on`): `view`'s thumbnail, title, lines, portrait row and
/// timestamp; `action` on a click (None: an empty card that only shows), `selected` drawn on, `corner` icons top right
/// (ids `<id>.<key>`, drawn over the card so they take their own clicks). Text that does not fit is cut, the full
/// text on the hover card.
#[allow(clippy::too_many_arguments)]
pub fn slot_card<A: Clone>(ui: &mut Ui<A>, id: &str, r: Rect, view: &SlotView, action: Option<A>, selected: bool, corner: &[Corner<A>]) {
    let t = ui.t.clone();
    let hover = action.is_some() && ui.hover(r);
    let fill = if selected { t.selected } else if hover { t.card_hot } else { t.card };
    let border = if selected { t.focus } else if hover { t.edge_hot } else { t.edge };
    let role = if selected { "card_on" } else if hover { "card_hot" } else { "card" };
    ui.p.skin(t.frame(role), r, fill, border, t.layout.radius);
    let (pad, gutter) = (t.layout.padding, t.layout.gutter);
    let (thumb, tx, py, icons) = slot_rects(r, corner.len());
    match view.thumb {
        Some(img) => cover_image(&mut ui.p, Some(img), thumb, 0.0, t.bg),
        None if thumb.w > 0 && thumb.h > 0 => {
            let inner = ui.inset(thumb, false);
            let initial: String = view.title.chars().find(|c| !c.is_whitespace()).map(|c| c.to_uppercase().collect()).unwrap_or_default();
            let lh = ui.p.line_h_face(Typeface::Display, HEAD);
            ui.p.text_center_face(Typeface::Display, inner.x as f32 + inner.w as f32 / 2.0, inner.y as f32 + (inner.h as f32 - lh) / 2.0, &initial, HEAD, t.gold);
        }
        None => {}
    }
    let ink = t.ink_on(role);
    let right = r.x + r.w - pad;
    let title_room = (icons.first().map_or(right, |i| i.x - 2) - tx) as f32;
    let mut cut = false;
    let mut y = r.y as f32 + pad as f32 - 2.0;
    cut |= text_cut(&mut ui.p, Typeface::Display, tx as f32, y, &view.title, TITLE, title_room, t.gold);
    y += ui.p.line_h_face(Typeface::Display, TITLE).ceil();
    let bottom = if view.portraits.is_empty() && view.stamp.is_empty() { (r.y + r.h - pad) as f32 } else { (py - 1) as f32 };
    let lh = ui.p.line_h(BODY);
    let mut shown = 0;
    for (i, line) in view.lines.iter().enumerate() {
        if y + lh > bottom {
            break;
        }
        cut |= text_cut(&mut ui.p, Typeface::Body, tx as f32, y, line, BODY, (right - tx) as f32, if i == 0 { ink } else { t.dim });
        y += lh.ceil();
        shown += 1;
    }
    let mut stamp_x = right;
    if !view.stamp.is_empty() {
        let (sh, sw) = (ui.p.line_h(SMALL), ui.p.text_w(&view.stamp, SMALL));
        let sw = sw.min((right - tx) as f32);
        stamp_x = right - sw.ceil() as i32;
        cut |= text_cut(&mut ui.p, Typeface::Body, stamp_x as f32, (r.y + r.h - pad) as f32 - sh, &view.stamp, SMALL, sw.max(1.0), t.dim);
    }
    let mut px = tx;
    for img in &view.portraits {
        if px + PORTRAIT > stamp_x - gutter {
            break;
        }
        ui.p.image(img, Rect::new(px, py, PORTRAIT, PORTRAIT));
        px += PORTRAIT + 2;
    }
    let mut full: Vec<String> = vec![view.title.clone()];
    full.extend(view.lines.iter().cloned());
    if !view.stamp.is_empty() {
        full.push(view.stamp.clone());
    }
    // The hover card holds the full text, a line each, when any of it was cut or a line did not fit.
    let tp: Tip = if cut || shown < view.lines.len() { full.iter().enumerate().map(|(i, l)| (l.clone(), if i == 0 { t.gold } else { t.text })).collect() } else { Vec::new() };
    ui.push(id, "card", full.join(" | "), r, true, selected, action, tp);
    for (c, ir) in corner.iter().zip(icons) {
        let tp = if c.tip.is_empty() { Vec::new() } else { tip(&c.label, &c.tip) };
        ui.icon_button(&format!("{id}.{}", c.key), ir, &c.label, None, c.glyph, c.action.clone(), c.enabled, c.toggled, tp);
    }
}

// ---- The themed ask -------------------------------------------------------------------------------------------------

/// What the ask dialog says: an optional title (the display face), the question, the two answers' labels, its frame
/// (`ornate`: the one-time cards' frame, else `panel`) and which answer is the default (`no_default`: No, for a
/// destructive question; the default is the primary).
#[derive(Debug, Clone)]
pub struct AskView<'a> {
    pub title: &'a str,
    pub question: &'a str,
    pub yes: &'a str,
    pub no: &'a str,
    pub ornate: bool,
    pub no_default: bool,
}

impl<'a> AskView<'a> {
    pub fn new(title: &'a str, question: &'a str) -> Self {
        Self { title, question, yes: "Yes", no: "No", ornate: false, no_default: false }
    }
}

/// The ask dialog's layout in `area` at the current text size: the dialog rect (centred, on the grid, at least
/// `ASK_MIN_W` x `ASK_MIN_H` times the text size, as tall as the question needs, inside the margins), the question's
/// lines, and the Yes and No rects (Yes left of No at the bottom right).
pub fn ask_layout(p: &mut Painter, area: Rect, v: &AskView) -> (Rect, Vec<String>, Rect, Rect) {
    let t = crate::theme::th();
    let (m, pad, gutter) = (t.layout.margin, t.layout.padding, t.layout.gutter);
    let s = p.fonts.scale.max(1.0);
    let w = snap_up((ASK_MIN_W as f32 * s).round() as i32).min(snap_down(area.w - 2 * m)).max(0);
    let role = if v.ornate { "ornate" } else { "panel" };
    let view = p.v.s;
    let inset = |r: Rect| t.frame(role).map_or(r.inset(pad), |f| crate::painter::nine_content_rect(&f.nine, r, view).inset(pad / 2));
    let least_h = (ASK_MIN_H as f32 * s).round() as i32;
    let inner_w = inset(Rect::new(0, 0, w, least_h)).w;
    let lines = p.wrap(v.question, BODY, inner_w as f32);
    let lh = p.line_h(BODY).ceil() as i32;
    let title_h = if v.title.is_empty() { 0 } else { p.line_h_face(Typeface::Display, HEAD).ceil() as i32 + gutter };
    let bh = snap_up(p.line_h(BODY).ceil() as i32 + 6);
    let bw = snap_up((p.text_w(v.yes, BODY).max(p.text_w(v.no, BODY)) + 24.0).ceil() as i32).max(48);
    let frame_pad = least_h - inset(Rect::new(0, 0, w, least_h)).h;
    let need = frame_pad + title_h + lines.len() as i32 * lh + gutter + bh;
    let h = snap_up(least_h.max(need)).min(snap_down(area.h - 2 * m)).max(0);
    let r = Rect::new(area.x + snap((area.w - w) / 2), area.y + snap((area.h - h) / 2), w, h);
    let inner = inset(r);
    let no = Rect::new(inner.x + inner.w - bw, inner.y + inner.h - bh, bw, bh);
    let yes = Rect::new(no.x - gutter - bw, no.y, bw, bh);
    (r, lines, yes, no)
}

/// The themed yes / no question over `area` (its scrim and a `backdrop` widget `<id>` keep the focus and clicks off
/// what lies under it): the `ornate` or `panel` frame, the title in the display face, the question wrapped, Yes then
/// No (`<id>.yes`, `<id>.no`; the default one the primary). A cut title (`<id>.title`) or question (`<id>.question`:
/// lines the height cap dropped, the last one left ending in "...") shows its full text on hover, and the text audit
/// sees it as cut. Returns the dialog's rect. Keys: `ask_key`.
pub fn ask_dialog<A: Clone>(ui: &mut Ui<A>, id: &str, area: Rect, v: &AskView, yes: A, no: A) -> Rect {
    let t = ui.t.clone();
    ui.p.fill(area, t.scrim);
    ui.push(id, "backdrop", v.question, area, true, false, None, Vec::new());
    let (r, lines, yr, nr) = ask_layout(&mut ui.p, area, v);
    let role = if v.ornate { "ornate" } else { "panel" };
    let (fill, edge, radius) = t.vector(role);
    ui.p.skin(t.frame(role), r, fill, edge, radius);
    let view = ui.p.v.s;
    let inner = t.frame(role).map_or(r.inset(t.layout.padding), |f| crate::painter::nine_content_rect(&f.nine, r, view).inset(t.layout.padding / 2));
    let mut y = inner.y as f32;
    if !v.title.is_empty() {
        let lh = ui.p.line_h_face(Typeface::Display, HEAD);
        let shown = fit_face(&mut ui.p, Typeface::Display, v.title, HEAD, inner.w as f32);
        ui.p.text_center_face(Typeface::Display, inner.x as f32 + inner.w as f32 / 2.0, y, &shown, HEAD, t.gold);
        if shown != v.title {
            ui.p.mark_cut(v.title);
            ui.push(format!("{id}.title"), "label", v.title, Rect::new(inner.x, y as i32, inner.w, lh.ceil() as i32), true, false, None, tip(v.title, ""));
        }
        y += lh.ceil() + t.layout.gutter as f32;
    }
    let (ink, lh) = (t.ink_on(role), ui.p.line_h(BODY).ceil());
    let top = y;
    let shown = lines.len().min((((yr.y as f32 - y) / lh).floor().max(0.0)) as usize);
    let dropped = shown < lines.len();
    for (i, l) in lines[..shown].iter().enumerate() {
        if dropped && i + 1 == shown {
            let last = fit_face(&mut ui.p, Typeface::Body, &format!("{}...", l.trim_end()), BODY, inner.w as f32);
            ui.p.text(inner.x as f32, y, &last, BODY, ink);
        } else {
            ui.p.text(inner.x as f32, y, l, BODY, ink);
        }
        y += lh;
    }
    if dropped {
        ui.p.mark_cut(v.question);
        ui.push(format!("{id}.question"), "label", v.question, Rect::new(inner.x, top as i32, inner.w, (y - top).max(lh) as i32), true, false, None, tip(v.question, ""));
    }
    pane_button(ui, &format!("{id}.yes"), yr, v.yes, None, yes, true, !v.no_default, BODY, false, Vec::new());
    pane_button(ui, &format!("{id}.no"), nr, v.no, None, no, true, v.no_default, BODY, false, Vec::new());
    r
}

/// The ask's keys, as the kit's question (`screen::Ask`) answers them: Enter the default (Yes, or No when
/// `no_default`), `y` Yes, `n` and Esc No; None for any other key.
pub fn ask_key(no_default: bool, k: Key) -> Option<bool> {
    match k {
        Key::Enter => Some(!no_default),
        Key::Char('y') | Key::Char('Y') => Some(true),
        Key::Char('n') | Key::Char('N') | Key::Esc => Some(false),
        _ => None,
    }
}
