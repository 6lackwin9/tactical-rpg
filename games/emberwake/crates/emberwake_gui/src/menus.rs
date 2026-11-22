//! Main menu, New game, Load / Save slots and Settings (scripts/ui/menus: main_menu_model, new_game_model,
//! save_slots_model, settings_model). Placeholder logic as in Godot; file import / export of saves is not ported yet.

use aegis_layout::{frac, resolve, Anchor, Insets, Stack};
use aegis_ui::theme::th;
use aegis_ui::audio::{Bus, MUTE_IN_BACKGROUND};
use aegis_ui::canvas::{rgba, Rect};
use crate::music;
use crate::portraits::{Faces, Who};
use crate::screens::{default_settings, enter_game, show_errors, title, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::pacing::FrameNeed;
use aegis_ui::painter::{Grid, Typeface};
use aegis_ui::panes::{self, MenuItem, MenuStyle};
use emberwake_battle::portrait::Look;
use aegis_ui::ui::{tip, Gfx, Glyph, Ui, Widget, BODY, H, W};
use crate::screens::WINDOW_SIZES;
use aegis_ui::screen::{display_keys, Settings, WindowShow, FPS_CAPS, ON_WEB, TEXT_SIZES, TEXT_SPEEDS};
use aegis_core::clock::{Duration, Instant};
use rpgkit_ui::vignette::{TextSpeed, REDUCE_MOTION, TEXT_SPEED};
use rpgkit_ui::portrait_overlays;
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_content::art as kart;
use emberwake_world::{new_game, progression, save};
use serde_json::{json, Value};

/// "2026-10-18 14:21" (UTC) from a unix time.
pub fn datetime(unix: i64) -> String {
    let days = unix.div_euclid(86400);
    let secs = unix.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}", secs / 3600, secs % 3600 / 60)
}

// ---- Main menu -----------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum MenuAct {
    Continue,
    NewGame,
    Load,
    Settings,
    #[cfg(feature = "dev_tools")]
    Tools,
    /// Desktop only: a web page cannot close its own tab (WEBREL1 6.2, `FB-157`), so the web menu has no Quit.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Quit,
}

/// The title backdrop (LOOK1 art A6) when present, else the MENU2 picture.
pub const TITLE_ART: &str = "backgrounds/menu/title.png";
pub const MENU_ART: &str = "backgrounds/menu/main_menu.png";
/// The crest behind the typeset logo (LOOK1 art A6), only when present.
pub const CREST_ART: &str = "ui/logo_crest.png";
/// The title as a picture, a high-resolution master: when present it replaces the typeset title, the subtitle and the
/// crest (the crest is inside the mark).
pub const LOGO_ART: &str = "ui/logo_title.png";
/// The picture's room, in shares of the canvas's right half (per mille; a half, because a share of the whole canvas cannot
/// land on the width the art was signed off at): its right edge, top and size. The picture sits right-aligned in it. On
/// the 1280 x 720 canvas: x 780 to 1250 from the top 26, the mockup's right edge and top.
const LOGO_ROOM_RIGHT: u16 = 954;
const LOGO_ROOM_TOP: u16 = 37;
const LOGO_ROOM_W: u16 = 735;
const LOGO_ROOM_H: u16 = 173;
/// The vignette over the backdrop (no veil: the title is the picture).
const MENU_VIGNETTE: f32 = 0.4;
/// The typeset logo's right edge and top, in shares of the canvas, and its sizes in old units (the display face).
const LOGO_RIGHT: u16 = 969;
const LOGO_TOP: u16 = 112;
const LOGO_SIZE: f32 = 28.0;
const LOGO_SUB_SIZE: f32 = 10.0;
/// The menu column at the Normal text size, in shares of the canvas: the strip right of it, its top and width. It widens
/// leftward for a label that needs it.
const MENU_RIGHT: u16 = 94;
const MENU_TOP: u16 = 417;
const MENU_W: u16 = 250;
/// A row's height and step, in old units.
const MENU_H: i32 = 18;
const MENU_STEP: i32 = 22;
/// Continue's peek at the Normal text size, in shares of the canvas; the gap left of the column and between its portraits,
/// in old units; party portraits shown.
const PEEK_W: u16 = 250;
const PEEK_H: u16 = 178;
const PEEK_GAP: i32 = 12;
const PEEK_FACE_GAP: i32 = 2;
const PEEK_PARTY: usize = 4;

/// The peek's second line: when the save stands (today the day: the game's day 1 is a new game, the clock plus one).
fn peek_when(gs: &emberwake_world::state::GameState) -> String {
    format!("Day {}", gs.time + 1)
}

/// PR7-36 (ART-08): the dimmed title backdrop (about 60 %) with an `ornate` main panel over `r`, for the screens the title
/// opens in place (slots, New game).
fn menu_panel<A: Clone>(ui: &mut Ui<A>, cx: &mut Cx, r: Rect) {
    let area = ui.screen();
    let picture = if cx.art.has(TITLE_ART) { TITLE_ART } else { MENU_ART };
    let img = cx.art.size_of(picture).and_then(|(w, h)| cx.art.image(picture, w, h));
    panes::backdrop(&mut ui.p, img, area, MENU_DIM, MENU_VIGNETTE);
    ui.p.fill(r, th().panel);
    ui.frame_body("ornate", r);
}

/// How far the title backdrop dims under a panel (about 60 %).
const MENU_DIM: u8 = 153;

/// What Continue's peek shows of the newest save.
#[derive(Clone)]
struct Peek {
    location: String,
    when: String,
    /// Member id, the name, its portrait and look; the active members first.
    party: Vec<(String, String, Who, Look)>,
}

impl Peek {
    /// Reads `slot`; None when it does not load (Continue says why when pressed).
    fn read(cx: &Cx, slot: i64) -> Option<Peek> {
        let gs = save::read(&cx.data, &cx.save_dir, slot).ok()?.state;
        let active: Vec<String> = gs.party.active_members().iter().map(|m| m.id.clone()).collect();
        let (on, off): (Vec<_>, Vec<_>) = gs.party.members.iter().partition(|m| active.contains(&m.id));
        let party = on.into_iter().chain(off).take(PEEK_PARTY).map(|m| {
            let (who, look) = crate::hub::member_portrait(&cx.data, &gs, m);
            (m.id.clone(), crate::hub::member_name(cx, m), who, look)
        });
        Some(Peek { location: cx.location_name(&gs.world_id, &gs.location_id), when: peek_when(&gs), party: party.collect() })
    }
}

#[derive(Default)]
pub struct MainMenu {
    w: Vec<Widget<MenuAct>>,
    /// The peek of this slot, read once when it first shows (a None peek: the slot did not load).
    peek: Option<(i64, Option<Peek>)>,
    /// The peek's portraits (surface `roster`), on the menu's own clock.
    faces: Faces,
    /// The newest readable save (-1: none), read once; it parses every slot. `resume` reads it again.
    newest: Option<i64>,
    /// The mouse or the keyboard has moved since the menu opened: the opening focus on Continue does not show the peek.
    moved: bool,
    /// TITLEFX1 (PR7-28): the title backdrop's fire layer, its spots read once; the picture the menu drew last.
    fire: Option<crate::title_fx::TitleFx>,
    picture: &'static str,
}

/// TITLEFX1: the fire layer's frame pacing (about 30 frames a second).
const FIRE_FRAME_MS: u64 = 33;

impl MainMenu {
    /// The title's fire shows (its spots read, the picture drawn is theirs, Reduce motion off).
    fn fire_on(&self, cx: &Cx) -> bool {
        self.fire.as_ref().is_some_and(|f| f.on(self.picture, cx.settings.reduce_motion()))
    }

    fn act(&mut self, cx: &mut Cx, a: MenuAct) {
        match a {
            MenuAct::Continue => {
                let slot = save::newest_slot(&cx.data, &cx.save_dir);
                if slot < 0 {
                    show_errors(cx, vec!["There is no save to continue.".into()]);
                } else {
                    load_slot(cx, slot);
                }
            }
            // PR7-37 (PT-01): a player's New game starts the game's one start world at once (the world picker, with the
            // test worlds, in a dev build or when the game offers more than one)
            MenuAct::NewGame => match new_game_worlds(&cx.data).as_slice() {
                [only] if !crate::DEV_BUILD => {
                    let only = only.clone();
                    start_new_game(cx, &only);
                }
                _ => cx.nav.push(Nav::Push(Box::new(NewGameScreen::new(cx)))),
            },
            MenuAct::Load => cx.nav.push(Nav::Push(Box::new(Slots::new(false)))),
            MenuAct::Settings => cx.nav.push(Nav::Push(Box::new(SettingsScreen::new(cx)))),
            #[cfg(feature = "dev_tools")]
            MenuAct::Tools => cx.nav.push(Nav::Push(Box::new(crate::runner::SceneRunner::new()))),
            MenuAct::Quit => {
                if cx.is_dirty() {
                    cx.ask_no("Quit without saving?", |cx| cx.quit = true);
                } else {
                    cx.quit = true;
                }
            }
        }
    }

    /// The logo (LOOK1 6): the title picture when there is one (`logo_picture`; nothing typeset, no crest), else the
    /// title in the display face in `gold` with a 1 old unit `bg` shadow, right-aligned, over the crest when there is one,
    /// the subtitle under it in `text`.
    fn logo(ui: &mut Ui<MenuAct>, cx: &mut Cx) {
        let (t, m, area) = (ui.t.clone(), ui.m(), ui.screen());
        if let Some(r) = Self::logo_picture(ui, cx, logo_room(area)) {
            ui.push("menu.logo", "picture", LOGO_ART, r, true, false, None, Vec::new());
            return;
        }
        let title_text = cx.manifest.title.clone();
        let (size, sub_size) = (m.uf(LOGO_SIZE), m.uf(LOGO_SUB_SIZE));
        let (w, h) = (ui.p.text_w_face(Typeface::Display, &title_text, size), ui.p.line_h_face(Typeface::Display, size));
        let at = logo_point(area);
        let (right, y) = (at.x as f32, at.y as f32);
        let x = right - w;
        if cx.art.has(CREST_ART) {
            let side = panes::snap_up_on((h * 2.0).ceil() as i32, m.grid);
            // Centred on the title, on a step of the art's unit: the crest's pixels stay where they were signed off.
            let (cx_at, cy_at) = (panes::snap_down_on((x + w / 2.0) as i32, m.u(1)), panes::snap_down_on((y + h / 2.0) as i32, m.u(1)));
            let r = Rect::new(cx_at - side / 2, cy_at - side / 2, side, side);
            let px = (side as f32 * ui.p.v.s).round() as i32;
            if let Some(img) = cx.art.image_fit(CREST_ART, px, px) {
                ui.image_in(img, r);
                ui.push("menu.crest", "picture", CREST_ART, r, true, false, None, Vec::new());
            }
        }
        let shadow = m.uf(1.0);
        ui.p.text_face(Typeface::Display, x + shadow, y + shadow, &title_text, size, t.bg);
        ui.p.text_face(Typeface::Display, x, y, &title_text, size, t.gold);
        let sub_w = ui.p.text_w_face(Typeface::Display, &cx.manifest.subtitle, sub_size);
        ui.p.text_face(Typeface::Display, right - sub_w, y + h, &cx.manifest.subtitle, sub_size, t.text);
    }

    /// The master at the exact physical size that fits `room` at this window, keeping its aspect (`Art::image_fit`:
    /// averaged down, and enlarged by nearest once the room is wider than the master), blended pixel for pixel: `Ui::image_in`
    /// takes a rect in units and would round its height and stretch it. Returns the rect it covers, None without the file.
    fn logo_picture(ui: &mut Ui<MenuAct>, cx: &mut Cx, room: Rect) -> Option<Rect> {
        let v = ui.p.v;
        let pr = logo_px(v, room, cx.art.size_of(LOGO_ART)?);
        let img = cx.art.image_fit(LOGO_ART, pr.w, pr.h)?;
        for y in 0..pr.h {
            for x in 0..pr.w {
                let i = ((y * pr.w + x) * 4) as usize;
                ui.p.c.blend(pr.x + x, pr.y + y, [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]);
            }
        }
        let (w, h) = ((pr.w as f32 / v.s).round() as i32, (pr.h as f32 / v.s).ceil() as i32);
        Some(room.place(Anchor::TopRight, w, h))
    }

    /// Continue's peek at the top right of `room`, what the cuts leave left of the column: an `ornate` panel with the save's
    /// location, `peek_when` and up to `PEEK_PARTY` portraits at the pane's portrait size; it grows with the text size.
    fn draw_peek(&mut self, ui: &mut Ui<MenuAct>, cx: &mut Cx, room: Rect, slot: i64) {
        if self.peek.as_ref().is_none_or(|(s, _)| *s != slot) {
            self.peek = Some((slot, Peek::read(cx, slot)));
        }
        let Some((_, Some(peek))) = &self.peek else { return };
        let (t, m, area) = (ui.t.clone(), ui.m(), ui.screen());
        let framed = t.frame("ornate").is_some();
        let inner_of = |body: Rect| if framed { body.inset(m.padding / 2) } else { body };
        let s = ui.p.fonts.scale.max(1.0);
        let lh = ui.p.line_h(m.body).ceil() as i32;
        let face = m.u(panes::PORTRAIT);
        let least = resolve(&[frac(PEEK_W)], area.w)[0];
        let w = panes::snap_up_on((least as f32 * s).ceil() as i32, m.grid);
        let least_h = resolve(&[frac(PEEK_H)], area.h)[0];
        let probe = room.place(Anchor::TopRight, w, least_h);
        let need = least_h - inner_of(ui.frame_content("ornate", probe)).h + 2 * lh + m.u(PEEK_FACE_GAP) + face;
        let r = room.place(Anchor::TopRight, w, panes::snap_up_on(least_h.max(need), m.grid));
        let mut inner = inner_of(ui.frame_body("ornate", r));
        let mut faces = inner.cut_bottom(face);
        let mut lines = Stack::new(inner, 0);
        let (place, when) = (lines.next(lh), lines.next(lh));
        let day = &peek.when;
        let cut = ui.p.fit(&peek.location, m.body, place.w as f32) != peek.location;
        ui.p.text_fit(place.x as f32, place.y as f32, &peek.location, m.body, place.w as f32, t.ink_on("ornate"));
        ui.p.text(when.x as f32, when.y as f32, day, m.body, t.dim);
        let hover = if cut { tip(&peek.location, day) } else { Vec::new() };
        ui.push("menu.peek", "panel", format!("{} | {day}", peek.location), r, true, false, None, hover);
        for (id, name, who, look) in &peek.party {
            let pr = faces.cut_left(face);
            faces.cut_left(m.u(PEEK_FACE_GAP));
            self.faces.draw(&mut ui.p, cx, who, look, "roster", pr);
            ui.push(format!("menu.peek.{id}"), "portrait", name.as_str(), pr, true, false, None, Vec::new());
        }
    }
}

/// The title picture's room on the canvas `area`.
fn logo_room(area: Rect) -> Rect {
    let right_half = area.place(Anchor::TopRight, frac(500), frac(1000));
    right_half.at(LOGO_ROOM_RIGHT, LOGO_ROOM_TOP, Anchor::TopRight, frac(LOGO_ROOM_W), frac(LOGO_ROOM_H))
}

/// Where the typeset logo's right edge and top meet on the canvas `area` (a rect of no size).
fn logo_point(area: Rect) -> Rect {
    area.at(LOGO_RIGHT, LOGO_TOP, Anchor::TopLeft, 0, 0)
}

/// The physical rect of a `size` picture in the logical `room` at view `v`: as large as fits, keeping the aspect, its right
/// edge and top on the room's.
fn logo_px(v: aegis_ui::painter::View, room: Rect, size: (i32, i32)) -> Rect {
    let pr = v.rect(room);
    let k = (pr.w as f32 / size.0.max(1) as f32).min(pr.h as f32 / size.1.max(1) as f32);
    pr.place(Anchor::TopRight, (size.0 as f32 * k).round() as i32, (size.1 as f32 * k).round() as i32)
}

/// The menu column `w` wide on the canvas `area`, and the room the cuts leave left of it past `gap` (the peek's).
fn cut_column(area: Rect, w: i32, gap: i32) -> (Rect, Rect) {
    let mut room = area;
    room.cut_top(frac(MENU_TOP));
    room.cut_right(frac(MENU_RIGHT));
    let column = room.cut_right(w);
    room.cut_right(gap);
    (column, room)
}

impl Screen<GameCx> for MainMenu {
    fn id(&self) -> &'static str {
        "main_menu"
    }
    /// Every position and size is calculated from the canvas (LAYOUT720-M0).
    fn grid(&self) -> Grid {
        Grid::Native
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        crate::viewkeys::window_key(cx, k, crate::viewkeys::Shown::Notice);
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        // The opening focus lands on Continue without showing the peek: it follows Continue as the active item once the
        // mouse has been on it or the keyboard has moved. A focus on another item rules (a mouse resting on Continue
        // after the keyboard moved away is not it).
        let on_continue = g.kit.focus.as_deref() == Some("menu.continue");
        let elsewhere = g.kit.focus_visible && g.kit.focus.is_some() && !on_continue;
        self.moved |= elsewhere;
        let focused = live && g.kit.focus_visible && on_continue;
        let mut ui: Ui<MenuAct> = Ui::new(g, mouse, live);
        let (m, area) = (ui.m(), ui.screen());
        self.faces.begin();
        let picture = if cx.art.has(TITLE_ART) { TITLE_ART } else { MENU_ART };
        let img = cx.art.size_of(picture).and_then(|(w, h)| cx.art.image(picture, w, h));
        panes::backdrop_pane(&mut ui, "menu.backdrop", img, picture, area, 0, MENU_VIGNETTE);
        // TITLEFX1 (PR7-28): lantern glows and rising embers at the data-placed spots, over the picture they belong to
        self.picture = picture;
        if self.fire.as_ref().is_none_or(|f| f.picture != picture) {
            self.fire = Some(crate::title_fx::TitleFx::load(cx.art.layers(), picture));
        }
        let fire = self.fire.as_mut().expect("loaded above");
        if fire.on(picture, cx.settings.reduce_motion()) {
            if let Some(size) = cx.art.size_of(picture) {
                fire.draw(&mut ui.p, area, size);
            }
        }
        Self::logo(&mut ui, cx);
        let newest = *self.newest.get_or_insert_with(|| save::newest_slot(&cx.data, &cx.save_dir));
        let mut items: Vec<MenuItem<MenuAct>> = Vec::new();
        if newest >= 0 {
            items.push(MenuItem::new("continue", "Continue", None, MenuAct::Continue));
        }
        items.extend([
            MenuItem::new("new_game", "New game", None, MenuAct::NewGame),
            MenuItem::new("load", "Load game", None, MenuAct::Load),
            MenuItem::new("settings", "Settings", None, MenuAct::Settings),
        ]);
        // WEBREL1-G1: the dev tools (the Scene Runner) only in a build with them (`crate::DEV_TOOLS`)
        // PR7-37 (PT-14): and only in a dev build (`crate::DEV_BUILD`); a player never sees it
        #[cfg(feature = "dev_tools")]
        if crate::DEV_BUILD {
            items.push(MenuItem::new("tools", "Tools (Forge, Dojo, Test Arena)", None, MenuAct::Tools));
        }
        // WEBREL1 6.2 (`FB-157`): no Quit on the web, where a page cannot close its own tab
        #[cfg(not(target_arch = "wasm32"))]
        items.push(MenuItem::new("quit", "Quit", None, MenuAct::Quit));
        // The column widens leftward at the larger text sizes so its longest label is never cut (only the labels take
        // the mouse: `MenuStyle::Text`).
        let widest = items.iter().map(|i| ui.p.text_w_face(Typeface::Display, &i.label, m.title)).fold(0.0, f32::max);
        let w = resolve(&[frac(MENU_W)], area.w)[0].max(panes::snap_up_on(widest.ceil() as i32 + 2 * m.padding, m.grid));
        let (column, peek_room) = cut_column(area, w, m.u(PEEK_GAP));
        let hovered = panes::menu_column_styled(&mut ui, "menu", column.x, column.y, column.w, m.u(MENU_H), m.u(MENU_STEP), &items, MenuStyle::Text);
        let over = newest >= 0 && hovered == Some(0) && !elsewhere;
        self.moved |= over;
        if newest >= 0 && (over || (focused && self.moved)) {
            self.draw_peek(&mut ui, cx, peek_room, newest);
        } else {
            self.peek = None;
        }
        let foot = area.pad(Insets::all(m.margin));
        let version_y = (foot.y + foot.h) as f32 - ui.p.line_h(m.small);
        let dim = ui.t.dim;
        ui.p.text_right((foot.x + foot.w) as f32, version_y, "Version dev · Aegis engine", m.small, dim);
        self.w = ui.widgets;
    }
    /// The peek's portraits breathe and the title's fire moves on the menu's clock.
    fn tick(&mut self, _cx: &mut Cx) {
        self.faces.tick();
        if let Some(f) = self.fire.as_mut() {
            f.tick();
        }
    }
    /// The picture, the logo and the entries stand still; the peek's portraits move, and the title's fire at about
    /// 30 frames a second while it shows (TITLEFX1: never under Reduce motion).
    fn frame_need(&self, cx: &Cx) -> FrameNeed {
        if self.peek.is_some() {
            FrameNeed::Full
        } else if self.fire_on(cx) {
            FrameNeed::Within(Duration::from_millis(FIRE_FRAME_MS))
        } else {
            FrameNeed::Idle
        }
    }
    /// The agent link's reading: the title's fire layer (on or off, its spots).
    fn state(&self, cx: &Cx) -> Value {
        json!({"title_fx": self.fire.as_ref().map(|f| f.state(self.fire_on(cx)))})
    }
    /// Back on the menu: the newest save may have changed, and the focus opens on Continue again.
    fn resume(&mut self, _cx: &mut Cx) {
        self.peek = None;
        self.newest = None;
        self.moved = false;
    }
    /// The menu starts with the first entry focused (the kit's focus: arrows, Tab, Enter).
    fn auto_focus(&self) -> bool {
        true
    }
    fn back(&mut self, _cx: &mut Cx) {}
    fn music(&self, cx: &Cx) -> Option<String> {
        music::track(cx, music::MENU, "", "")
    }
    widget_plumbing!();
}

/// Reads `slot` and enters the game (load warnings shown once); errors as a notice.
pub fn load_slot(cx: &mut Cx, slot: i64) {
    match save::read(&cx.data, &cx.save_dir, slot) {
        Ok(l) => {
            enter_game(cx, l.state, slot, l.warnings);
            cx.mark_saved(slot);
        }
        Err(errors) => show_errors(cx, errors),
    }
}

// ---- New game ------------------------------------------------------------------------------------------------

pub struct NewGameScreen {
    selected: String,
    w: Vec<Widget<NgAct>>,
}

#[derive(Debug, Clone)]
pub enum NgAct {
    Pick(String),
    Start,
    Back,
}

/// NewGameModel.party_line: "Party, all Lv 5: Bryn Hale (Warrior)".
fn party_line(party: &[(String, String, i64)]) -> String {
    if party.is_empty() {
        return "Party: none".into();
    }
    let shared = party.iter().all(|p| p.2 == party[0].2);
    let names: Vec<String> = party
        .iter()
        .map(|(name, cls, level)| {
            let mut bits = Vec::new();
            if !cls.is_empty() && cls != name {
                bits.push(cls.clone());
            }
            if !shared {
                bits.push(format!("Lv {level}"));
            }
            if bits.is_empty() {
                name.clone()
            } else {
                format!("{name} ({})", bits.join(", "))
            }
        })
        .collect();
    if shared {
        format!("Party, all Lv {}: {}", party[0].2, names.join(", "))
    } else {
        format!("Party: {}", names.join(", "))
    }
}

impl NewGameScreen {
    /// ACT1-HOOKS1 (FB-241 c): the manifest's first playable start world is picked first (`new_game::default_world`).
    pub fn new(cx: &Cx) -> Self {
        Self { selected: new_game::default_world(&cx.data), w: Vec::new() }
    }

    fn party_of(cx: &Cx, world: &str) -> Vec<(String, String, i64)> {
        let d = &cx.data;
        d.start_party(world)
            .iter()
            .map(|unit_id| {
                let (mut base, mut shown) = (unit_id.clone(), String::new());
                if !d.has("units", unit_id) {
                    if let Some(h) = new_game::hero_from_character(d, world, unit_id) {
                        shown = progression::display_name(d, &h);
                        base = h.base_unit;
                    }
                }
                let e = d.entry("units", &base);
                let name = if shown.is_empty() { s_or(e, "name", unit_id) } else { shown };
                // HOOKS1 CN2: the class name in the world being started
                (name, d.class_name(world, &s(e, "class_id")), json::i(e, "level", 1))
            })
            .collect()
    }

    fn act(&mut self, cx: &mut Cx, a: NgAct) {
        match a {
            NgAct::Pick(w) => self.selected = w,
            NgAct::Back => cx.nav.push(Nav::Pop),
            NgAct::Start => {
                if self.selected.is_empty() {
                    show_errors(cx, vec!["Choose a world first.".into()]);
                    return;
                }
                let world = self.selected.clone();
                start_new_game(cx, &world);
            }
        }
    }
}

/// PR7-37 (PT-01): the worlds New game offers: in a dev build every playable world (the test worlds too), else the
/// game's start worlds (`new_game::player_worlds`).
pub fn new_game_worlds(d: &emberwake_world::Data) -> Vec<String> {
    if crate::DEV_BUILD {
        new_game::playable_worlds(d)
    } else {
        new_game::player_worlds(d)
    }
}

/// A new game in `world`: asks first when it would replace the autosave.
fn start_new_game(cx: &mut Cx, world: &str) {
    let world = world.to_string();
    let head = save::peek(&cx.data, &cx.save_dir, save::AUTOSAVE_SLOT);
    if json::b(&head, "ok", false) {
        let at = json::i(&head, "saved_at", 0);
        let q = format!(
            "Starting a new game replaces your autosave ({}, {}, {}). Continue?",
            cx.world_name(&s(&head, "world_id")),
            cx.location_name(&s(&head, "world_id"), &s(&head, "location_id")),
            if at > 0 { datetime(at) } else { "unknown time".into() }
        );
        cx.ask_no(q, move |cx| start_world(cx, &world));
    } else {
        start_world(cx, &world);
    }
}

fn start_world(cx: &mut Cx, world: &str) {
    match new_game::create_for_world(&cx.data, world, cx.seed.unwrap_or(-1)) {
        Err(errors) => show_errors(cx, errors),
        Ok(gs) => {
            enter_game(cx, gs, -1, Vec::new());
            cx.autosave();
        }
    }
}

impl Screen<GameCx> for NewGameScreen {
    /// Nothing here moves by itself.
    fn frame_need(&self, _cx: &Cx) -> FrameNeed {
        FrameNeed::Idle
    }
    fn id(&self) -> &'static str {
        "new_game"
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<NgAct> = Ui::new(g, mouse, live);
        // PR7-36 (ART-08): a main panel over the dimmed title backdrop
        menu_panel(&mut ui, cx, Rect::new(56, 8, 528, 312));
        title(&mut ui, "New game", 26);
        let mut y = 54;
        for w in new_game_worlds(&cx.data) {
            let m = cx.data.manifest(&w);
            let lines = vec![(s_or(m, "name", &w), th().text), (s(m, "tagline"), th().text), (party_line(&Self::party_of(cx, &w)), th().text)];
            ui.card(&format!("world.{w}"), Rect::new(70, y, 500, 44), &lines, Some(NgAct::Pick(w.clone())), w == self.selected, true, Vec::new());
            y += 50;
        }
        ui.button("new_game.back", Rect::new(70, 292, 70, 18), "Back", NgAct::Back, true);
        ui.button("new_game.start", Rect::new(500, 292, 70, 18), "Start", NgAct::Start, !self.selected.is_empty());
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        if k == Key::Enter {
            self.act(cx, NgAct::Start);
        }
    }
    fn state(&self, _cx: &Cx) -> Value {
        json!({"selected": self.selected})
    }
    widget_plumbing!();
}

// ---- Load / Save slots ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotAct {
    Load(i64),
    Save(i64),
    Delete(i64),
    /// Lock (true) or unlock (false) a manual slot's save.
    Lock(i64, bool),
    Back,
}

pub struct Slots {
    saving: bool,
    w: Vec<Widget<SlotAct>>,
    /// PR7-36 (ART-08): each filled slot's lead and day, read once per save (keyed by slot and its `saved_at`).
    leads: std::collections::BTreeMap<(i64, i64), Option<Peek>>,
    faces: Faces,
}

/// What a slot row offers (PLAYUI1 section 1): the row's click (or why there is none, its hover card) and the icons in
/// its top-right corner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRow {
    pub action: Option<SlotAct>,
    /// The hover card's line: what a click does, or why the row takes no click.
    pub why: String,
    /// The lock icon shows (an existing manual save).
    pub lock: bool,
    /// The delete icon shows (any existing save).
    pub delete: bool,
}

/// The rules of a slot row: save mode saves into a manual slot that is not locked (`no_save` is why the running game
/// can't be saved now, "" when it can); load mode loads a readable save; the lock never shows on the autosave.
pub fn slot_row(saving: bool, slot: i64, exists: bool, ok: bool, locked: bool, no_save: &str) -> SlotRow {
    let manual = slot != save::AUTOSAVE_SLOT;
    let (action, why) = if saving {
        if !manual {
            (None, "The game writes the autosave.".to_string())
        } else if locked {
            (None, "Locked: unlock it to save over it.".to_string())
        } else if !no_save.is_empty() {
            (None, no_save.to_string())
        } else {
            (Some(SlotAct::Save(slot)), if exists { "Save over this save." } else { "Save here." }.to_string())
        }
    } else if ok {
        (Some(SlotAct::Load(slot)), "Load this save.".to_string())
    } else if exists {
        (None, "This save can't be loaded.".to_string())
    } else {
        (None, "Empty slot.".to_string())
    };
    SlotRow { action, why, lock: manual && exists, delete: exists }
}

impl Slots {
    pub fn new(saving: bool) -> Self {
        Self { saving, w: Vec::new(), leads: std::collections::BTreeMap::new(), faces: Faces::default() }
    }

    fn describe(cx: &Cx, slot: i64) -> String {
        let head = save::peek(&cx.data, &cx.save_dir, slot);
        if !json::b(&head, "ok", false) {
            return String::new();
        }
        let world = s(&head, "world_id");
        let mut parts = vec![cx.world_name(&world)];
        let where_ = cx.location_name(&world, &s(&head, "location_id"));
        if !where_.is_empty() {
            parts.push(where_);
        }
        let at = json::i(&head, "saved_at", 0);
        if at > 0 {
            parts.push(datetime(at));
        }
        parts.join(", ")
    }

    fn act(&mut self, cx: &mut Cx, a: SlotAct) {
        match a {
            SlotAct::Back => cx.nav.push(Nav::Pop),
            SlotAct::Load(slot) => {
                if cx.busy() {
                    show_errors(cx, vec!["Finish the fight, conversation or shop first.".into()]);
                } else if cx.has_game() && cx.is_dirty() && !crate::runover::is_over(cx) {
                    let what = Self::describe(cx, slot);
                    let name = if slot == save::AUTOSAVE_SLOT { "the autosave".to_string() } else { format!("slot {slot}") };
                    cx.ask_no(format!("Load {name} ({what})? Unsaved progress in the current game is lost."), move |cx| load_slot(cx, slot));
                } else {
                    load_slot(cx, slot);
                }
            }
            SlotAct::Save(slot) => {
                if !cx.has_game() {
                    show_errors(cx, vec!["There is no game to save.".into()]);
                } else if cx.busy() {
                    show_errors(cx, vec!["You can't save during a battle, a conversation or while a shop is open.".into()]);
                } else if save::is_locked(&cx.save_dir, slot) {
                    show_errors(cx, vec![format!("Slot {slot} is locked.")]);
                } else if save::exists(&cx.save_dir, slot) {
                    let what = Self::describe(cx, slot);
                    cx.ask_no(format!("Overwrite slot {slot}{}?", if what.is_empty() { String::new() } else { format!(": {what}") }), move |cx| save_now(cx, slot));
                } else {
                    save_now(cx, slot);
                }
            }
            SlotAct::Delete(slot) => {
                if save::is_locked(&cx.save_dir, slot) {
                    return show_errors(cx, vec!["Unlock the save to delete it.".into()]);
                }
                let what = Self::describe(cx, slot);
                cx.ask_no(format!("Delete the save in slot {slot}{}?", if what.is_empty() { String::new() } else { format!(" ({what})") }), move |cx| {
                    if let Err(e) = save::delete(&cx.save_dir, slot) {
                        show_errors(cx, vec![e]);
                    }
                });
            }
            SlotAct::Lock(slot, on) => {
                if let Err(e) = save::set_locked(&cx.save_dir, slot, on) {
                    show_errors(cx, vec![e]);
                }
            }
        }
    }

    /// Why the running game can't be saved now ("" when it can; CAMPAIGN1 F3: never while the run is over).
    fn no_save(cx: &Cx) -> &'static str {
        if !cx.has_game() {
            "There is no game to save."
        } else if cx.busy() {
            "You can't save during a battle, a conversation or while a shop is open."
        } else {
            emberwake_world::campaign::save_refusal(cx.gs())
        }
    }
}

/// A slot row's first line (its name and, for a readable save, the world, the place and the time) and its second (the
/// whole party, or why it can't load, or "Empty"); PR7-41 (PT-35). The day of the save's lead follows from `slot_when`.
pub(crate) fn slot_text(cx: &Cx, slot: i64, head: &Dict) -> (String, String) {
    let (exists, ok) = (json::b(head, "exists", false), json::b(head, "ok", false));
    let mut label = if slot == 0 { "Autosave".to_string() } else { format!("Slot {slot}") };
    // PR7-41 (PT-35): the place and time on the slot's line, the whole party on its own line (no cut names)
    let status = if ok {
        let world = s(head, "world_id");
        let party: Vec<String> = json::aget(head, "party")
            .iter()
            .map(|m| {
                let m = json::dict(m);
                // the member's character name in the saved world, else its base unit's
                let unit = s_or(cx.data.entry("units", &s(m, "base_unit")), "name", &s(m, "base_unit"));
                format!("{} Lv {}", s_or(cx.data.resolve(&world, "characters", &s(m, "id")), "name", &unit), json::i(m, "level", 0))
            })
            .collect();
        let at = json::i(head, "saved_at", 0);
        label = format!("{label} · {}{}", slot_place(cx, head), if at > 0 { format!(" · {}", datetime(at)) } else { String::new() });
        party.join(", ")
    } else if exists {
        format!("Can't load ({})", s(head, "error"))
    } else {
        "Empty".into()
    };
    (label, status)
}

/// A readable save's place: the world and the location ("The Ashlands, Warden Keep"), the world alone when the save
/// names no location (PR7-29, FB-369: never "The Ashlands, ").
pub(crate) fn slot_place(cx: &Cx, head: &Dict) -> String {
    let world = s(head, "world_id");
    let mut parts = vec![cx.world_name(&world)];
    let at = cx.location_name(&world, &s(head, "location_id"));
    if !at.is_empty() {
        parts.push(at);
    }
    parts.retain(|p| !p.is_empty());
    parts.join(", ")
}

/// PR7-29 (SLOTCARD1, LOOK1 M2): the slot cards' grid: six 290 x 84 cards in two columns, the autosave first.
pub const CARD_W: i32 = 290;
pub const CARD_H: i32 = 84;
const CARD_GAP: i32 = 8;
const CARDS_X: i32 = (W - 2 * CARD_W - CARD_GAP) / 2;
const CARDS_Y: i32 = 50;

/// Slot `slot`'s card rect (two columns, filled row by row).
pub fn card_rect(slot: i64) -> Rect {
    let (col, row) = (slot as i32 % 2, slot as i32 / 2);
    Rect::new(CARDS_X + col * (CARD_W + CARD_GAP), CARDS_Y + row * (CARD_H + CARD_GAP), CARD_W, CARD_H)
}

/// The day line of a readable save's lead (the Continue peek's), None when it does not load.
pub(crate) fn slot_when(cx: &Cx, slot: i64) -> Option<String> {
    Peek::read(cx, slot).map(|p| p.when)
}

impl Slots {
    /// Why the running game can't be saved now (`no_save`), for the view-model.
    pub(crate) fn no_save_now(cx: &Cx) -> &'static str {
        Self::no_save(cx)
    }
}

fn save_now(cx: &mut Cx, slot: i64) {
    if cx.game.as_ref().is_some_and(|g| g.sample) {
        return show_errors(cx, vec!["The UI Gallery's sample game is not saved.".into()]);
    }
    let errors = save::write(&cx.data, &cx.save_dir, cx.gs(), slot, 0);
    if errors.is_empty() {
        cx.mark_saved(slot);
    } else {
        show_errors(cx, errors);
    }
}

impl Screen<GameCx> for Slots {
    /// Nothing here moves by itself.
    fn frame_need(&self, _cx: &Cx) -> FrameNeed {
        FrameNeed::Idle
    }
    fn id(&self) -> &'static str {
        if self.saving {
            "save_slots"
        } else {
            "load_slots"
        }
    }
    fn overlay(&self) -> bool {
        false
    }
    /// PR7-70 (PANELS2, `FB-378` question 2): a main panel; it already fills the screen (not an overlay), so the game
    /// menu never draws under it, and a panel opened over it takes its place (`drawn_screens`).
    fn panel(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<SlotAct> = Ui::new(g, mouse, live);
        self.faces.begin();
        // PR7-36 (ART-08): a main panel over the dimmed title backdrop, as Settings sits over the menu
        menu_panel(&mut ui, cx, Rect::new(CARDS_X - 12, 6, W - 2 * CARDS_X + 24, H - 12));
        title(&mut ui, if self.saving { "Save game" } else { "Load game" }, 20);
        for slot in save::AUTOSAVE_SLOT..=save::LAST_SLOT {
            let r = card_rect(slot);
            let head = save::peek(&cx.data, &cx.save_dir, slot);
            let exists = json::b(&head, "exists", false);
            let ok = json::b(&head, "ok", false);
            let lead = if ok {
                let key = (slot, json::i(&head, "saved_at", 0));
                self.leads.retain(|k, _| k.0 != slot || *k == key);
                self.leads.entry(key).or_insert_with(|| Peek::read(cx, slot)).clone()
            } else {
                None
            };
            let (label, status) = slot_text(cx, slot, &head);
            let locked = save::is_locked(&cx.save_dir, slot);
            let row = slot_row(self.saving, slot, exists, ok, locked, Self::no_save(cx));
            // PR7-29 (SLOTCARD1): the location's picture (else the world's title picture), the name, the place and the
            // lead's day, the party's portraits and the time saved; one click acts as before (PLAYUI1), lock and
            // delete in the top-right corner
            let (thumb_r, _, _, _) = panes::slot_rects_on(&ui.m(), r, 2);
            let scale = ui.p.v.s;
            let (tw, tpx) = ((thumb_r.w as f32 * scale).round() as i32, (thumb_r.h as f32 * scale).round() as i32);
            // a slot with no readable save shows a plain well (no letter), as the mock's empty card
            let well = || {
                let c = crate::portraits::portrait_well();
                aegis_ui::canvas::Image { w: 1, h: 1, px: vec![c[0], c[1], c[2], 255] }
            };
            let thumb = if ok {
                let world = s(&head, "world_id");
                let files = [kart::location_background(&s(&head, "location_id")), kart::title_background(&world)];
                files.iter().find_map(|f| cx.art.image(f, tw.max(1), tpx.max(1)).cloned())
            } else {
                Some(well())
            };
            let face = ui.icon_px(Rect::new(0, 0, panes::PORTRAIT, panes::PORTRAIT));
            let faces: Vec<aegis_ui::canvas::Image> = lead.as_ref().map(|p| p.party.iter().filter_map(|(_, _, who, _)| cx.art.first(&who.chain(""), face, face).cloned()).collect()).unwrap_or_default();
            let name = if slot == save::AUTOSAVE_SLOT { "Autosave".to_string() } else { format!("Slot {slot}") };
            let lines = if ok {
                let mut l = vec![slot_place(cx, &head)];
                if let Some(p) = &lead {
                    l.push(p.when.clone());
                }
                l
            } else {
                vec![status.clone()]
            };
            let at = json::i(&head, "saved_at", 0);
            let view = panes::SlotView { thumb: thumb.as_ref(), title: name, lines, portraits: faces.iter().collect(), stamp: if ok && at > 0 { datetime(at) } else { String::new() } };
            let mut corner = Vec::new();
            if row.lock {
                let (label, glyph) = if locked { ("Unlock", Glyph::Lock) } else { ("Lock", Glyph::Unlock) };
                corner.push(panes::Corner { key: "lock".into(), label: label.into(), glyph, action: SlotAct::Lock(slot, !locked), enabled: true, toggled: locked, tip: String::new() });
            }
            if row.delete {
                corner.push(panes::Corner { key: "delete".into(), label: "Delete".into(), glyph: Glyph::Delete, action: SlotAct::Delete(slot), enabled: !locked, toggled: false, tip: if locked { "Unlock the save to delete it.".into() } else { String::new() } });
            }
            let id = format!("slot.{slot}");
            panes::slot_card(&mut ui, &id, r, &view, row.action, false, &corner);
            // the card's widget: off when a click does nothing, its text the row's two lines, its hover card what a click
            // does (or why it does nothing), as the rows before it said
            let full = match &lead {
                Some(p) => format!("{label} · {}", p.when),
                None => label.clone(),
            };
            if let Some(w) = ui.widgets.iter_mut().find(|w| w.id == format!("{id}.lock")) {
                w.tip = tip(if locked { "Unlock" } else { "Lock (protect from overwrite and delete)" }, "");
            }
            if let Some(w) = ui.widgets.iter_mut().find(|w| w.id == id) {
                w.enabled = row.action.is_some();
                w.text = format!("{full}: {status}");
                if w.tip.is_empty() {
                    w.tip = tip(&full, &row.why);
                }
            }
        }
        ui.button("slots.back", Rect::new(CARDS_X, CARDS_Y + 3 * (CARD_H + CARD_GAP), 70, 18), "Back", SlotAct::Back, true);
        self.w = ui.widgets;
    }
    /// The agent link's reading: the screen's view-model (UISPLIT1-H Stage A, `save_view`).
    fn state(&self, cx: &Cx) -> Value {
        json!({"saving": self.saving, "view": crate::save_view::view(cx, self.saving).to_json()})
    }
    widget_plumbing!();
}

// ---- Settings ------------------------------------------------------------------------------------------------
//
// SETTINGS-MOCK1 (PR7-21): the approved mock `tools/mock_ui/js/04_settings.js` (ART_UI_STUDIO screen 3, decision
// "Display settings DISPLAY1"): a tab rail (Display, Sound, Gameplay, Battle, Controls, each with a subtitle), one
// help line per row, every change written at once except Display Mode and Resolution, which wait for Apply and then
// a 15 s keep-or-revert confirm; closing with a staged display change asks first. Opened in game (the hub) it is the
// decluttered popup: smaller, no help lines or tab subtitles, and a Return to title button.

#[derive(Debug, Clone)]
pub enum SetAct {
    Tab(usize),
    Toggle(&'static str),
    /// A slider value (volumes 0-100, speeds and scales 0.5-2.0).
    Set(&'static str, f64),
    /// A choice row's option by index (`SettingsScreen::options`).
    Pick(&'static str, usize),
    /// A choice row's arrows: one option back (-1) or on (+1), wrapping round.
    Step(&'static str, i32),
    /// RULESET1-U slice 6: a battle overlay group switched (the settings key `battle_overlays`).
    Overlay(&'static str),
    /// Writes the staged Display Mode and Resolution and opens the keep-or-revert confirm.
    Apply,
    /// Drops the staged display change.
    Reset,
    /// The confirm's Keep changes.
    Keep,
    /// The confirm's Revert (also its X, Esc, right click and the timer running out).
    Revert,
    Defaults,
    Close,
    /// The popup's Return to title.
    Title,
    /// PR7-41 (PT-32): the battle Menu's Retreat (asks first).
    Retreat,
    /// The close guard's three answers.
    GuardStay,
    GuardDiscard,
    GuardApply,
    /// CONTROLS-UI1 (PR7-44): an action of the Controls tab's view (`controls_view`).
    View(aegis_ui::screen_layout::Act),
}

/// The settings keys Display Mode and Resolution stage until Apply (DISPLAY2's `fullscreen` and `borderless`, and
/// `window_size`).
pub const STAGED: [&str; 3] = ["fullscreen", "borderless", "window_size"];
/// How long the keep-or-revert confirm waits before it puts the previous display back.
pub const CONFIRM_SECS: u64 = 15;
/// The rail: each tab and its subtitle (the popup shows no subtitles).
pub const TABS: [(&str, &str); 5] = [("Display", "Mode, size & text"), ("Sound", "Volume & audio mix"), ("Gameplay", "Dialogue & map pacing"), ("Battle", "Timeline & speed"), ("Controls", "Keybindings")];
/// The desktop's Display Mode choices (Design 2026-11-18: windowed, borderless, fullscreen; the web keeps its
/// Fullscreen row) and each one's help line.
pub const DISPLAY_MODES: [(WindowShow, &str, &str); 3] = [
    (WindowShow::Windowed, "Windowed", "A framed desktop window at the chosen resolution"),
    (WindowShow::Borderless, "Borderless", "A frameless window at the resolution, centred"),
    (WindowShow::Fullscreen, "Fullscreen", "Covers the whole monitor; the resolution rests"),
];
/// The animation speeds the Battle tab steps through.
const ANIM_SPEEDS: [f64; 7] = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
/// PLAYUI1-M: the region map's marker names (value, label); the first is the default.
pub const MAP_NAMES: [(&str, &str); 2] = [("hover", "On hover"), ("always", "Always")];
/// A volume slider's sample and bus (scripts/audio/volume_preview.gd: the master sample plays on the interface bus).
fn preview_of(key: &str) -> Option<(Bus, &'static str)> {
    match key {
        "master" => Some((Bus::Ui, "sfx/ui_confirm.wav")),
        "music" => Some((Bus::Music, "sfx/ui_confirm.wav")),
        "effects" => Some((Bus::Sfx, "sfx/hit.wav")),
        "interface" => Some((Bus::Ui, "sfx/ui_click.wav")),
        "ambience" => Some((Bus::Ambience, "sfx/ui_confirm.wav")),
        _ => None,
    }
}

/// The staged keys of `v` in the form Display Mode writes them.
fn staged_of(v: &Value) -> Value {
    let show = WindowShow::of(v);
    json!({"fullscreen": show == WindowShow::Fullscreen, "borderless": show == WindowShow::Borderless, "window_size": v["window_size"]})
}

/// What the close guard leads to once answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Leave {
    Close,
    Title,
}

/// The keep-or-revert confirm after Apply: the display before it, and when it reverts by itself.
struct Confirm {
    before: Value,
    until: Instant,
}

pub struct SettingsScreen {
    tab: usize,
    /// The decluttered in-game popup (opened from the hub), else the full screen (the main menu).
    popup: bool,
    /// The web build's rows (DISPLAY1 rule 8, WEBFIX1): Fullscreen and Text size of the display keys.
    web: bool,
    /// The staged Display Mode and Resolution (`STAGED`).
    pending: Value,
    confirm: Option<Confirm>,
    guard: Option<Leave>,
    /// PR7-41 (PT-32): opened from a world battle whose fight allows a retreat: the footer offers Retreat.
    retreat: bool,
    /// CONTROLS-UI1 (PR7-44): the Controls tab's rebind in progress, and the layouts its view draws with.
    rebind: aegis_ui::controls::Rebind,
    layouts: aegis_ui::screen_layout::Layouts,
    w: Vec<Widget<SetAct>>,
}

/// PR7-41 (PT-32): the answer tag of the battle Menu's Retreat question (Yes: the battle screen sends `retreat`).
pub const RETREAT_ASK: &str = "battle.retreat";

/// The rows a tab lists, top to bottom.
#[derive(Clone, Copy)]
enum Row {
    /// Resolution and Display Mode with Apply (the desktop).
    Video,
    Choice(&'static str, &'static str, &'static str),
    Toggle(&'static str, &'static str, &'static str),
    Volume(&'static str, &'static str, &'static str),
    /// Animation speed with its Skip switch.
    Anim,
    /// The battle overlay groups (and the debug row in a debug build).
    Overlays,
    Controls,
    /// A section header over the rows after it (FB-308: the Battle tab's "Advanced").
    Heading(&'static str),
}

/// The row help lines' text size.
const HELP: f32 = BODY - 1.0;
/// A tab body's heading size.
const HEAD: f32 = BODY + 2.0;
/// The gap between rows.
const GAP: i32 = 3;

impl SettingsScreen {
    /// The full screen (the main menu, the gallery).
    pub fn new(cx: &Cx) -> Self {
        Self::open(cx, false)
    }

    /// The decluttered in-game popup (the hub's Settings).
    pub fn in_game(cx: &Cx) -> Self {
        Self::open(cx, true)
    }

    fn open(cx: &Cx, popup: bool) -> Self {
        let layouts = aegis_ui::screen_layout::Layouts::new(cx.art.layers().clone());
        Self { tab: 0, popup, web: ON_WEB, pending: staged_of(&cx.settings.v), confirm: None, guard: None, retreat: false, rebind: Default::default(), layouts, w: Vec::new() }
    }

    /// PR7-41 (PT-32): the in-game popup as a world battle's Menu: Retreat in the footer when `can_retreat` (the
    /// fight allows it now, `Session::can_retreat`).
    pub fn in_battle(cx: &Cx, can_retreat: bool) -> Self {
        let mut s = Self::open(cx, true);
        s.retreat = can_retreat;
        s
    }

    /// The web build's rows on any build (tests).
    #[cfg(test)]
    pub(crate) fn as_web(mut self) -> Self {
        self.web = true;
        self
    }

    /// A Display Mode or Resolution change waits for Apply.
    pub fn display_dirty(&self, cx: &Cx) -> bool {
        let now = &cx.settings.v;
        WindowShow::of(&self.pending) != WindowShow::of(now) || self.pending["window_size"] != now["window_size"]
    }

    /// Seconds left on the keep-or-revert confirm at `now` (None: it is not open).
    pub fn confirm_left(&self, now: Instant) -> Option<u64> {
        self.confirm.as_ref().map(|c| {
            let left = c.until.saturating_duration_since(now);
            left.as_secs() + u64::from(left.subsec_nanos() > 0)
        })
    }

    /// The confirm's timer at `now`: past its end the previous display comes back.
    pub fn tick_at(&mut self, cx: &mut Cx, now: Instant) {
        if self.confirm.as_ref().is_some_and(|c| now >= c.until) {
            self.act(cx, SetAct::Revert);
        }
    }

    fn save(cx: &mut Cx) {
        if let Err(e) = cx.settings.save() {
            show_errors(cx, vec![format!("Settings could not be saved: {e}")]);
        }
    }

    /// A change that applies at once: written and saved.
    fn commit(cx: &mut Cx, k: &str, v: Value) {
        cx.settings.v[k] = v;
        Self::save(cx);
    }

    /// A choice row's option labels and the current one.
    fn options(&self, cx: &Cx, k: &str) -> (Vec<String>, usize) {
        let d = &cx.settings.v;
        let labels = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        match k {
            "display_mode" => {
                let show = WindowShow::of(&self.pending);
                (DISPLAY_MODES.iter().map(|m| m.1.to_string()).collect(), DISPLAY_MODES.iter().position(|m| m.0 == show).unwrap_or(0))
            }
            "window_size" => (labels(&WINDOW_SIZES), WINDOW_SIZES.iter().position(|s| self.pending["window_size"].as_str() == Some(*s)).unwrap_or(0)),
            "fps_cap" => (labels(&FPS_CAPS), FPS_CAPS.iter().position(|s| d["fps_cap"].as_str() == Some(*s)).unwrap_or(0)),
            // The engine's four levels (Godot TEXT_LEVELS); a saved value off the levels shows the nearest one.
            "text_scale" => {
                let ts = d["text_scale"].as_f64().unwrap_or(1.0);
                let at = TEXT_SIZES.iter().enumerate().min_by(|a, b| (a.1 .1 - ts).abs().total_cmp(&(b.1 .1 - ts).abs())).map(|(i, _)| i).unwrap_or(1);
                (TEXT_SIZES.iter().map(|t| t.0.to_string()).collect(), at)
            }
            TEXT_SPEED => (TEXT_SPEEDS.iter().map(|v| TextSpeed::parse(v).label().to_string()).collect(), TEXT_SPEEDS.iter().position(|v| d[TEXT_SPEED].as_str() == Some(*v)).unwrap_or(1)),
            "map_names" => (MAP_NAMES.iter().map(|m| m.1.to_string()).collect(), MAP_NAMES.iter().position(|m| d["map_names"].as_str() == Some(m.0)).unwrap_or(0)),
            // REGIONVIEW1: the region map's zoom
            "map_zoom" => (crate::mapview::zoom_steps().map(|z| format!("{z}x")).collect(), (crate::mapview::zoom_of(&d["map_zoom"]) - crate::mapview::ZOOM_QUARTERS.0) as usize),
            "see_through" => (crate::stage::SEE_THROUGH.iter().map(|v| v.1.to_string()).collect(), crate::stage::SEE_THROUGH.iter().position(|v| d["see_through"].as_str() == Some(v.0)).unwrap_or(0)),
            crate::battle::PLAYBACK_SPEED => {
                let speeds = &crate::battle::PLAYBACK_SPEEDS;
                (speeds.iter().map(|v| v.1.to_string()).collect(), speeds.iter().position(|v| d[crate::battle::PLAYBACK_SPEED].as_str() == Some(v.0)).unwrap_or(0))
            }
            "anim_speed" => {
                let a = d["anim_speed"].as_f64().unwrap_or(1.0);
                let at = ANIM_SPEEDS.iter().enumerate().min_by(|x, y| (x.1 - a).abs().total_cmp(&(y.1 - a).abs())).map(|(i, _)| i).unwrap_or(2);
                (ANIM_SPEEDS.iter().map(|s| format!("{s}x")).collect(), at)
            }
            _ => (Vec::new(), 0),
        }
    }

    fn pick(&mut self, cx: &mut Cx, k: &'static str, i: usize) {
        match k {
            "display_mode" => {
                if let Some((show, _, _)) = DISPLAY_MODES.get(i) {
                    self.pending["fullscreen"] = json!(*show == WindowShow::Fullscreen);
                    self.pending["borderless"] = json!(*show == WindowShow::Borderless);
                }
            }
            "window_size" => {
                if let Some(v) = WINDOW_SIZES.get(i) {
                    self.pending["window_size"] = json!(v);
                }
            }
            "fps_cap" => {
                if let Some(v) = FPS_CAPS.get(i) {
                    Self::commit(cx, k, json!(v));
                }
            }
            "text_scale" => {
                if let Some(t) = TEXT_SIZES.get(i) {
                    Self::commit(cx, k, json!(t.1));
                }
            }
            TEXT_SPEED => {
                if let Some(v) = TEXT_SPEEDS.get(i) {
                    Self::commit(cx, k, json!(v));
                }
            }
            "map_names" => {
                if let Some((v, _)) = MAP_NAMES.get(i) {
                    Self::commit(cx, k, json!(v));
                }
            }
            "map_zoom" => {
                if let Some(v) = crate::mapview::zoom_steps().nth(i) {
                    Self::commit(cx, k, json!(v));
                }
            }
            "see_through" => {
                if let Some((v, _)) = crate::stage::SEE_THROUGH.get(i) {
                    Self::commit(cx, k, json!(v));
                }
            }
            crate::battle::PLAYBACK_SPEED => {
                if let Some((v, _, _)) = crate::battle::PLAYBACK_SPEEDS.get(i) {
                    Self::commit(cx, k, json!(v));
                }
            }
            "anim_speed" => {
                if let Some(v) = ANIM_SPEEDS.get(i) {
                    Self::commit(cx, k, json!(v));
                }
            }
            _ => {}
        }
    }

    /// Leaving (X, Esc, right click, Return to title): with a staged display change the guard asks first.
    fn leave(&mut self, cx: &mut Cx, to: Leave) {
        if self.display_dirty(cx) {
            self.guard = Some(to);
        } else {
            Self::finish(cx, to);
        }
    }

    fn finish(cx: &mut Cx, to: Leave) {
        match to {
            Leave::Close => cx.nav.push(Nav::Pop),
            Leave::Title => crate::hub::quit_to_title(cx),
        }
    }

    fn act(&mut self, cx: &mut Cx, a: SetAct) {
        match a {
            SetAct::Tab(t) => {
                self.tab = t.min(TABS.len() - 1);
                self.rebind.cancel();
            }
            SetAct::View(a) => {
                if crate::controls_view::act(&mut self.rebind, &mut cx.settings.v, &a) {
                    Self::save(cx);
                }
            }
            SetAct::Toggle(k) => {
                let on = !cx.settings.v[k].as_bool().unwrap_or(false);
                Self::commit(cx, k, Value::Bool(on));
                // the web's Fullscreen applies at once: nothing stays staged
                if STAGED.contains(&k) {
                    self.pending = staged_of(&cx.settings.v);
                }
            }
            SetAct::Set(k, v) => {
                let (lo, hi) = if k.ends_with("speed") || k.ends_with("scale") {
                    (0.5, 2.0)
                } else if k == crate::mapview::WALK_STEP_KEY {
                    (crate::mapview::WALK_STEP_MS.0, crate::mapview::WALK_STEP_MS.1)
                } else {
                    (0.0, 100.0)
                };
                let v = v.clamp(lo, hi);
                Self::commit(cx, k, if lo == 0.0 { json!(v.round() as i64) } else { json!((v * 100.0).round() / 100.0) });
                if let Some((bus, sample)) = preview_of(k) {
                    cx.audio.set_levels(&cx.settings.v);
                    cx.audio.preview(bus, sample);
                }
            }
            SetAct::Pick(k, i) => self.pick(cx, k, i),
            SetAct::Step(k, d) => {
                let (labels, at) = self.options(cx, k);
                let n = labels.len() as i32;
                if n > 0 {
                    self.pick(cx, k, (at as i32 + d).rem_euclid(n) as usize);
                }
            }
            SetAct::Overlay(k) => {
                let mut v = cx.settings.v["battle_overlays"].clone();
                let on = v[k].as_bool().unwrap_or_else(|| crate::battle::overlay_default(k));
                if !v.is_object() {
                    v = json!({});
                }
                v[k] = Value::Bool(!on);
                Self::commit(cx, "battle_overlays", v);
            }
            SetAct::Apply => {
                if self.display_dirty(cx) {
                    let before = staged_of(&cx.settings.v);
                    for k in STAGED {
                        cx.settings.v[k] = self.pending[k].clone();
                    }
                    Self::save(cx);
                    self.confirm = Some(Confirm { before, until: Instant::now() + Duration::from_secs(CONFIRM_SECS) });
                }
            }
            SetAct::Reset => self.pending = staged_of(&cx.settings.v),
            SetAct::Keep => self.confirm = None,
            SetAct::Revert => {
                if let Some(c) = self.confirm.take() {
                    for k in STAGED {
                        cx.settings.v[k] = c.before[k].clone();
                    }
                    Self::save(cx);
                    self.pending = staged_of(&cx.settings.v);
                }
            }
            SetAct::Defaults => {
                // Every row at once, but the display: its defaults are staged for Apply like any display change. The web
                // stages nothing (PR7-26): its Fullscreen is the browser's, so the display keys stay as they are.
                let mut d = Settings::defaults(default_settings());
                self.rebind.cancel();
                self.pending = staged_of(if self.web { &cx.settings.v } else { &d });
                for k in STAGED {
                    d[k] = cx.settings.v[k].clone();
                }
                cx.settings.v = d;
                Self::save(cx);
            }
            SetAct::Close => self.leave(cx, Leave::Close),
            SetAct::Title => self.leave(cx, Leave::Title),
            SetAct::Retreat => {
                cx.nav.push(Nav::Pop);
                cx.ask_tag_no("Retreat from this fight? The party falls back to where it came from. Foes already beaten still pay; the rest pay nothing, and the fight is not won.", RETREAT_ASK);
            }
            SetAct::GuardStay => self.guard = None,
            SetAct::GuardDiscard => {
                self.pending = staged_of(&cx.settings.v);
                if let Some(to) = self.guard.take() {
                    Self::finish(cx, to);
                }
            }
            SetAct::GuardApply => {
                self.guard = None;
                self.act(cx, SetAct::Apply);
            }
        }
    }

    /// The rows of the open tab.
    fn rows(&self, cx: &Cx) -> Vec<Row> {
        let shown = |k: &str| display_keys(self.web).contains(&k);
        match self.tab {
            0 => {
                let mut r = Vec::new();
                if shown("window_size") {
                    r.push(Row::Video);
                } else {
                    // DISPLAY1 rule 8: the browser's fullscreen, at once (the click is the gesture it needs)
                    r.push(Row::Toggle("fullscreen", "Fullscreen", "The browser's fullscreen, for this session"));
                }
                r.push(Row::Choice("text_scale", "Text Size", "Scale of dialogue, tooltips and readouts (85%-130%)"));
                // PR7-15 (b), FB-287 Option A: the Gameplay tab's Map zoom row also here, on the web too (the web has no
                // window size to make the map bigger)
                r.push(Row::Choice("map_zoom", "Map Camera Zoom", "Camera zoom on street and hub maps"));
                if shown("fps_cap") {
                    r.push(Row::Choice("fps_cap", "Frame Rate Cap", "The most frames drawn each second"));
                }
                if shown("vsync") {
                    r.push(Row::Toggle("vsync", "Vertical Sync (VSync)", "Prevents frame tearing"));
                }
                r.push(Row::Toggle(REDUCE_MOTION, "Reduce Motion", "No camera shake or drifting ash"));
                // ASSETORG4: only for a game whose manifest mounts the portrait overlays
                if cx.data.portrait_overlays {
                    r.push(Row::Toggle(portrait_overlays::SETTING, "Portrait Effects", "Glows, lines and haze over character portraits"));
                }
                r
            }
            1 => vec![
                Row::Volume("master", "Master Volume", "Overall game audio output"),
                Row::Volume("music", "Music Volume", "Orchestral and hurdy-gurdy ward score"),
                Row::Volume("effects", "Effects Volume", "Combat steel clashes, billhooks and chains"),
                Row::Volume("interface", "Interface Volume", "Menu chimes, card clicks and roll ticks"),
                // PR7-34 (AMBIENCE1): the settings mock's row
                Row::Volume("ambience", "Ward Ambience", "Canal rain, distant bells and furnace rumble"),
            ],
            // PR7-37 (PT-33): Map Camera Zoom on the Display tab only (PR7-15 had it here too)
            2 => vec![
                Row::Choice(TEXT_SPEED, "Text Speed", "How fast story captions and dialogue lines reveal"),
                Row::Choice("map_names", "Map Labels & Names", "Person and exit names on street and hub maps"),
            ],
            3 => vec![
                Row::Choice(crate::battle::PLAYBACK_SPEED, "Playback Speed", "How a timeline round plays: 1x, 0.5x or by block"),
                Row::Anim,
                Row::Toggle("auto_fill", "Auto-Fill Remaining Blocks", "Fill a hero's free blocks with follow-ups"),
                Row::Toggle(crate::battle::AUTO_CONFIRM, "Auto-Confirm Next Round", "Go on to the next round's planning by itself"),
                Row::Toggle("show_tells", "Show Enemy Tells", "Enemy intent arcs and targets"),
                Row::Toggle("warn_missing", "Warn Missing Actions", "Ask before confirming with actions unspent"),
                // FB-308 (Design, READY section 4): the rows past the mock's six sit under an Advanced header
                Row::Heading("Advanced"),
                Row::Choice("see_through", "Units Behind", "When units hidden behind others are outlined"),
                // DIORAMA3 R6: the board's grid lines (faint at rest, stronger while planning)
                Row::Toggle(crate::battle::BOARD_GRID, "Board Grid", "Faint tile lines on the battle board"),
                Row::Overlays,
            ],
            _ => vec![Row::Controls],
        }
    }

    /// The row height: the label and its help line (the popup: the label only).
    fn row_h(&self, ui: &Ui<SetAct>) -> i32 {
        let lh = ui.p.line_h(BODY);
        if self.popup {
            ((lh + 8.0).ceil() as i32).max(20)
        } else {
            ((lh + ui.p.line_h(HELP) + 7.0).ceil() as i32).max(26)
        }
    }

    fn row_height(&self, ui: &mut Ui<SetAct>, cx: &Cx, row: Row, w: i32) -> i32 {
        let rh = self.row_h(ui);
        match row {
            Row::Video => rh * 2 + 4,
            Row::Overlays => {
                let head = ui.p.line_h(BODY + 1.0).ceil() as i32 + 4;
                let groups = crate::battle::settings_overlay_groups().len().div_ceil(2) as i32 * OVERLAY_PITCH;
                head + groups + if crate::DEV_BUILD { OVERLAY_PITCH + 4 } else { 0 }
            }
            // CONTROLS-UI1: the default takes its whole height (the body scrolls it); a layout file places its own
            Row::Controls => aegis_ui::screen_layout::default_height(ui, &crate::controls_view::view(&cx.settings.v, &self.rebind), w),
            Row::Heading(_) => ui.p.line_h(HEAD).ceil() as i32 + 8,
            _ => rh,
        }
    }

    /// A row's card, its label and (the full screen) its help line, left of a control `ctrl_w` wide.
    fn label(&self, ui: &mut Ui<SetAct>, r: Rect, label: &str, help: &str, ctrl_w: i32, enabled: bool) {
        let t = th();
        ui.p.panel(r, t.card, t.edge, 2.0);
        let lh = ui.p.line_h(BODY);
        let room = (r.w - ctrl_w - 16) as f32;
        let col = if enabled { t.text } else { t.dim };
        if self.popup || help.is_empty() {
            ui.p.text_fit(r.x as f32 + 6.0, r.y as f32 + (r.h as f32 - lh) / 2.0, label, BODY, room, col);
        } else {
            let hh = ui.p.line_h(HELP);
            let y = r.y as f32 + (r.h as f32 - lh - hh) / 2.0;
            ui.p.text_fit(r.x as f32 + 6.0, y, label, BODY, room, col);
            ui.p.text_fit(r.x as f32 + 6.0, y + lh, help, HELP, room, t.dim);
        }
    }

    /// The width of choice row `k`'s control: the arrows and a list wide enough for its longest option.
    fn choice_w(&self, ui: &mut Ui<SetAct>, cx: &Cx, k: &str) -> i32 {
        let (labels, _) = self.options(cx, k);
        let widest = labels.iter().map(|l| ui.p.text_w(l, BODY).ceil() as i32).max().unwrap_or(0);
        (widest + 20 + 30).max(if self.popup { 110 } else { 124 })
    }

    /// The choice control: back arrow, the list, on arrow.
    fn choice(&self, ui: &mut Ui<SetAct>, cx: &Cx, k: &'static str, r: Rect, enabled: bool, t: aegis_ui::ui::Tip) {
        let (labels, at) = self.options(cx, k);
        let aw = 13;
        ui.button(&format!("settings.{k}.prev"), Rect::new(r.x, r.y, aw, r.h), "<", SetAct::Step(k, -1), enabled);
        ui.select(&format!("settings.{k}"), Rect::new(r.x + aw + 2, r.y, r.w - 2 * aw - 4, r.h), &labels, at, |i| SetAct::Pick(k, i), enabled, t);
        ui.button(&format!("settings.{k}.next"), Rect::new(r.x + r.w - aw, r.y, aw, r.h), ">", SetAct::Step(k, 1), enabled);
    }

    fn toggle(ui: &mut Ui<SetAct>, id: &str, r: Rect, text: &str, on: bool, a: SetAct, t: aegis_ui::ui::Tip) {
        ui.button_full(id, r, text, a, true, on, BODY, t);
    }

    fn draw_row(&self, ui: &mut Ui<SetAct>, cx: &Cx, row: Row, r: Rect) {
        let d = cx.settings.v.clone();
        let ch = 14;
        let cy = r.y + (self.row_h(ui) - ch) / 2;
        let on_w = |ui: &mut Ui<SetAct>| (ui.p.text_w("Off", BODY).ceil() as i32 + 16).max(40);
        match row {
            Row::Video => {
                let t = th();
                let dirty = self.display_dirty(cx);
                ui.p.panel(r, t.card, if dirty { t.gold } else { t.edge }, 2.0);
                let rh = self.row_h(ui);
                let cw = self.choice_w(ui, cx, "window_size").max(self.choice_w(ui, cx, "display_mode"));
                let side = (ui.p.text_w("In sync", HELP).ceil() as i32).max(ui.p.text_w("Apply", BODY).ceil() as i32 + 12).max(48);
                let inner = Rect::new(r.x, r.y, r.w - side - 10, r.h);
                let show = WindowShow::of(&self.pending);
                let res = Rect::new(inner.x, inner.y, inner.w, rh);
                self.label_bare(ui, res, "Resolution", "Window size, staged until you press Apply", cw);
                self.choice(ui, cx, "window_size", Rect::new(res.x + res.w - cw - 6, res.y + (rh - ch) / 2, cw, ch), show != WindowShow::Fullscreen, Vec::new());
                ui.p.fill(Rect::new(inner.x + 4, r.y + rh + 1, inner.w - 6, 1), t.edge);
                let mode = Rect::new(inner.x, inner.y + rh + 2, inner.w, rh);
                let help = DISPLAY_MODES.iter().find(|m| m.0 == show).map_or("", |m| m.2);
                self.label_bare(ui, mode, "Display Mode", help, cw);
                self.choice(ui, cx, "display_mode", Rect::new(mode.x + mode.w - cw - 6, mode.y + (rh - ch) / 2, cw, ch), true, tip("Display Mode", "Windowed, Borderless or Fullscreen. A change waits for Apply; then keep it or it reverts by itself."));
                let sx = r.x + r.w - side - 6;
                ui.p.fill(Rect::new(sx - 4, r.y + 4, 1, r.h - 8), t.edge);
                let bh = 16;
                let by = r.y + r.h / 2 - bh - 1;
                ui.button_full("settings.apply", Rect::new(sx, by, side, bh), "Apply", SetAct::Apply, true, dirty, BODY, tip("Apply", "Switch to the chosen Display Mode and Resolution; keep them within 15 s or they revert."));
                if dirty {
                    ui.button("settings.reset", Rect::new(sx, by + bh + 3, side, 14), "Reset", SetAct::Reset, true);
                } else {
                    let w = ui.p.text_w("In sync", HELP);
                    ui.p.text(sx as f32 + (side as f32 - w) / 2.0, (by + bh + 5) as f32, "In sync", HELP, t.dim);
                }
            }
            Row::Choice(k, label, help) => {
                let cw = self.choice_w(ui, cx, k);
                self.label(ui, r, label, help, cw, true);
                let t = match k {
                    "map_names" => tip("Map Labels & Names", "On hover: a marker's name shows while you point at it, its legend row, or face it. Always: every person and exit is named."),
                    "map_zoom" => tip("Map Camera Zoom", crate::mapview::ZOOM_TIP),
                    "see_through" => tip("Units Behind", crate::stage::SEE_THROUGH_TIP),
                    TEXT_SPEED => tip("Text Speed", "How fast story captions type out: Slow, Normal, Fast, or Instant (the whole text at once). A click shows the rest at any speed."),
                    crate::battle::PLAYBACK_SPEED => tip("Playback Speed", "How a timeline battle plays its round: 1x; 0.5x (every walk, clip, float and pause twice as long); Block (it waits after each block until N or Next block). P cycles it in battle."),
                    _ => Vec::new(),
                };
                self.choice(ui, cx, k, Rect::new(r.x + r.w - cw - 6, cy, cw, ch), true, t);
            }
            Row::Toggle(k, label, help) => {
                let w = on_w(ui);
                self.label(ui, r, label, help, w, true);
                let default = !matches!(k, "fullscreen") && k != REDUCE_MOTION;
                let on = if k == "fullscreen" { WindowShow::of(&d) == WindowShow::Fullscreen } else { d[k].as_bool().unwrap_or(default) };
                let t = match k {
                    "auto_fill" => tip("Auto-Fill Remaining Blocks", "After your picks the planner fills the round's free blocks with follow-ups (ghost blocks you can replace). Off: the free blocks Rest."),
                    crate::battle::AUTO_CONFIRM => tip("Auto-Confirm Next Round", "Timeline battles: after a round plays, the battle goes to the next round's planning by itself; your plan still waits for your Confirm. Off: press Next round (Space) to go on. A recording you watch always waits."),
                    portrait_overlays::SETTING => tip("Portrait Effects", "The glows, growing lines, specks and haze over character portraits in talk boxes, scenes, party cards and the timeline. Off: plain portraits; blinks, breathing and expressions stay."),
                    _ => Vec::new(),
                };
                let id = if k == portrait_overlays::SETTING { "settings.portrait_overlays".to_string() } else { format!("settings.{k}") };
                Self::toggle(ui, &id, Rect::new(r.x + r.w - w - 6, cy, w, ch), if on { "On" } else { "Off" }, on, SetAct::Toggle(k), t);
            }
            Row::Volume(k, label, help) => {
                let vw = if self.popup { 150 } else { 170 };
                self.label(ui, r, label, help, vw, true);
                let pct = d[k].as_f64().unwrap_or(0.0);
                let key = static_key(k);
                let x = r.x + r.w - vw - 6;
                ui.button(&format!("settings.{k}.down"), Rect::new(x, cy, 13, ch), "<", SetAct::Set(key, pct - VOLUME_STEP), true);
                ui.slider(&format!("settings.{k}"), Rect::new(x + 15, cy, vw - 30, ch), (pct / VOLUME_STEP).round() as usize, 21, &format!("{pct}%"), |i| SetAct::Set(key, i as f64 * VOLUME_STEP), true);
                ui.button(&format!("settings.{k}.up"), Rect::new(x + vw - 13, cy, 13, ch), ">", SetAct::Set(key, pct + VOLUME_STEP), true);
            }
            Row::Anim => {
                let skip = d["skip_anim"].as_bool().unwrap_or(false);
                let sw = (ui.p.text_w("Skip: Off", BODY).ceil() as i32 + 12).max(46);
                let cw = self.choice_w(ui, cx, "anim_speed");
                self.label(ui, r, "Animation Speed", "Unit movement stride and attack cadence", cw + sw + 4, true);
                let x = r.x + r.w - cw - sw - 10;
                self.choice(ui, cx, "anim_speed", Rect::new(x, cy, cw, ch), !skip, Vec::new());
                Self::toggle(ui, "settings.skip_anim", Rect::new(x + cw + 4, cy, sw, ch), if skip { "Skip: On" } else { "Skip: Off" }, skip, SetAct::Toggle("skip_anim"), Vec::new());
            }
            Row::Heading(label) => {
                let t = th();
                let lh = ui.p.line_h(HEAD).ceil() as i32;
                ui.p.text((r.x + 2) as f32, (r.y + 3) as f32, label, HEAD, t.gold);
                ui.p.fill(Rect::new(r.x + 2, r.y + 3 + lh + 2, r.w - 4, 1), t.edge);
                ui.push(format!("settings.heading.{}", label.to_lowercase()), "label", label, Rect::new(r.x, r.y, r.w, r.h), true, false, None, Vec::new());
            }
            Row::Overlays => {
                // RULESET1-U slice 6 (preference "Battle overlays and the clarity pass", user 2026-10-30): every overlay
                // group of a timeline battle; not in the mock, kept under its rows.
                let (x, mut y) = (r.x + 2, r.y + 2);
                ui.p.text(x as f32, y as f32, "Battle overlays (timeline battles)", BODY + 1.0, th().gold);
                y += ui.p.line_h(BODY + 1.0).ceil() as i32 + 2;
                let col_w = r.w / 2;
                let groups = crate::battle::settings_overlay_groups();
                for (k, (id, label, help, default)) in groups.iter().enumerate() {
                    let (gx, gy) = (x + 3 + (k as i32 % 2) * col_w, y + (k as i32 / 2) * OVERLAY_PITCH);
                    let on = d["battle_overlays"][*id].as_bool().unwrap_or(*default);
                    ui.checkbox(&format!("settings.overlay.{id}"), gx, gy, label, on, SetAct::Overlay(id), true);
                    if let Some(wd) = ui.widgets.last_mut() {
                        wd.tip = tip(label, help);
                    }
                }
                // VISCLAR1 VC4, VC5: the debug-build-only row (never in a release build); PR7-37 (PT-33): a dev build's
                // only (`crate::DEV_BUILD`)
                if crate::DEV_BUILD {
                    let y = y + groups.len().div_ceil(2) as i32 * OVERLAY_PITCH + 2;
                    ui.p.text(x as f32, y as f32, "Debug", BODY + 1.0, th().gold);
                    // VIEWKEYS1: the walk's base step time (a release build uses the default), beside the heading
                    let (least, most, _) = crate::mapview::WALK_STEP_MS;
                    let base = crate::mapview::WalkStep::of(&json!({crate::mapview::WALK_STEP_KEY: d[crate::mapview::WALK_STEP_KEY]}), false).base;
                    ui.p.text(x as f32 + 36.0, y as f32, "Walk step", BODY, th().text);
                    let steps = ((most - least) / WALK_STEP_STEP) as usize + 1;
                    ui.slider("settings.debug.walk_step", Rect::new(x + 84, y, col_w - 92, 13), ((base - least) / WALK_STEP_STEP).round() as usize, steps, &format!("{base:.0} ms"), |i| SetAct::Set(crate::mapview::WALK_STEP_KEY, least + i as f64 * WALK_STEP_STEP), true);
                    ui.checkbox("settings.debug.party_foresight_off", x + 3 + col_w, y, "Party foresight off", d[crate::battle::FORESIGHT_OFF].as_bool().unwrap_or(false), SetAct::Toggle(crate::battle::FORESIGHT_OFF), true);
                    if let Some(wd) = ui.widgets.last_mut() {
                        wd.tip = tip("Party foresight off", "Debug: every enemy plan a party foresight passive reveals (Hawk Eye) is drawn as unrevealed. Enrage and the battle's plan reveal still show. Display only.");
                    }
                }
            }
            // drawn by `draw` through its view (CONTROLS-UI1)
            Row::Controls => {}
        }
    }

    /// `label` without its own card (the video card's two halves).
    fn label_bare(&self, ui: &mut Ui<SetAct>, r: Rect, label: &str, help: &str, ctrl_w: i32) {
        let t = th();
        let lh = ui.p.line_h(BODY);
        let room = (r.w - ctrl_w - 16) as f32;
        if self.popup {
            ui.p.text_fit(r.x as f32 + 6.0, r.y as f32 + (r.h as f32 - lh) / 2.0, label, BODY, room, t.text);
        } else {
            let hh = ui.p.line_h(HELP);
            let y = r.y as f32 + (r.h as f32 - lh - hh) / 2.0;
            ui.p.text_fit(r.x as f32 + 6.0, y, label, BODY, room, t.text);
            // the mock: the Display Mode's line in gold, the Resolution's dim
            ui.p.text_fit(r.x as f32 + 6.0, y + lh, help, HELP, room, if label == "Display Mode" { t.gold } else { t.dim });
        }
    }

    /// The modal box of the confirm or the guard: a frame `w` wide around the title, the wrapped text and the buttons
    /// (id, label, action, primary) from the right.
    fn modal(ui: &mut Ui<SetAct>, title: &str, lines: &[(String, f32, aegis_ui::canvas::Rgba)], buttons: &[(&str, String, SetAct, bool)], close: SetAct) {
        let w = 300;
        let bh = 16;
        let mut text_h = 0.0;
        let mut wrapped = Vec::new();
        for (s, size, col) in lines {
            let ls = ui.p.wrap(s, *size, (w - 24) as f32);
            text_h += ls.len() as f32 * ui.p.line_h(*size) + 3.0;
            wrapped.push((ls, *size, *col));
        }
        let head = ui.p.line_h_face(Typeface::Display, ui.m().head).ceil() as i32;
        let h = 10 + head + 8 + text_h.ceil() as i32 + 8 + bh + 10;
        let r = Rect::new((W - w) / 2, (H - h) / 2, w, h);
        let t = th();
        ui.p.fill(Rect::new(0, 0, W, H), t.game("settings_veil", rgba(0, 0, 0, 140)));
        // the box hides the screen under it (the panel's fill lets a little through)
        let [pr, pg, pb, _] = t.panel;
        ui.p.fill(r, rgba(pr, pg, pb, 255));
        // the mock's reduced corner-only frame (theme role `ornate`)
        ui.frame_body("ornate", r);
        let head_size = ui.m().head;
        ui.p.text_face(Typeface::Display, r.x as f32 + 12.0, r.y as f32 + 8.0, title, head_size, t.gold);
        ui.close_x("settings.modal.close", Rect::new(r.x + r.w - 26, r.y + 7, 17, 16), close);
        let mut y = (r.y + 10 + head + 4) as f32;
        for (ls, size, col) in wrapped {
            for l in ls {
                ui.p.text((r.x + 12) as f32, y, &l, size, col);
                y += ui.p.line_h(size);
            }
            y += 3.0;
        }
        let mut x = r.x + r.w - 12;
        for (id, label, a, primary) in buttons {
            let bw = ui.p.text_w(label, BODY).ceil() as i32 + 14;
            x -= bw;
            ui.button_full(id, Rect::new(x, r.y + r.h - bh - 10, bw, bh), label, a.clone(), true, *primary, BODY, Vec::new());
            x -= 6;
        }
    }
}

impl Screen<GameCx> for SettingsScreen {
    /// Nothing here moves by itself but the confirm's countdown.
    fn frame_need(&self, _cx: &Cx) -> FrameNeed {
        if self.confirm.is_some() {
            FrameNeed::Within(Duration::from_millis(250))
        } else {
            FrameNeed::Idle
        }
    }
    fn id(&self) -> &'static str {
        "settings"
    }
    fn overlay(&self) -> bool {
        true
    }
    /// STYLE-PASS1B (`PR7-48` (2)): a panel-shell screen; a panel opened over it takes its place until it closes.
    fn panel(&self) -> bool {
        true
    }
    fn tick(&mut self, cx: &mut Cx) {
        self.tick_at(cx, Instant::now());
    }
    /// CONTROLS-UI1: while a rebind waits (or asks to swap), the raw key or pad button is its input.
    fn capture(&mut self, cx: &mut Cx, input: aegis_ui::controls::Input) -> bool {
        if self.tab != TABS.len() - 1 || !self.rebind.capturing() {
            return false;
        }
        if self.rebind.input(&crate::controls_view::ACTIONS, &mut cx.settings.v, input) == aegis_ui::controls::Outcome::Bound {
            Self::save(cx);
        }
        true
    }
    /// Right click and Esc: a rebind cancels, the confirm reverts, the guard stays, else the screen closes (asking first
    /// when a display change is staged).
    fn back(&mut self, cx: &mut Cx) {
        if self.rebind.capturing() {
            self.rebind.cancel();
            return;
        }
        let a = if self.confirm.is_some() {
            SetAct::Revert
        } else if self.guard.is_some() {
            SetAct::GuardStay
        } else {
            SetAct::Close
        };
        self.act(cx, a);
    }
    /// Enter answers the open box: Keep changes, or Apply display.
    fn key(&mut self, cx: &mut Cx, k: Key) {
        if k == Key::Enter {
            if self.confirm.is_some() {
                self.act(cx, SetAct::Keep);
            } else if self.guard.is_some() {
                self.act(cx, SetAct::GuardApply);
            }
        }
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let modal = self.confirm.is_some() || self.guard.is_some();
        let mut ui: Ui<SetAct> = Ui::new(g, if modal { (-500, -500) } else { mouse }, live);
        let t = th();
        ui.p.fill(Rect::new(0, 0, W, H), t.game("settings_veil", rgba(0, 0, 0, 140)));
        let popup = self.popup;
        let r = if popup { Rect::new(85, 50, 470, 260) } else { Rect::new(32, 18, 576, 324) };
        // the mock's full ornate frame (theme role `ornate_full`; the vector panel without its art)
        ui.frame_body("ornate_full", r);
        let m = ui.m();
        ui.p.text_face(Typeface::Display, r.x as f32 + 9.0, r.y as f32 + 6.0, "Settings", m.head, t.gold);
        let top = r.y + m.frame_title_h;
        ui.close_x("settings.close", Rect::new(r.x + r.w - 28, r.y + 9, 19, 18), SetAct::Close);
        // the footer: the note and Restore defaults (and the popup's Return to title)
        let fh = 16;
        let fy = r.y + r.h - 9 - fh;
        let mut bx = r.x + r.w - 10;
        let mut foot = vec![("settings.defaults", "Restore defaults", SetAct::Defaults)];
        if popup {
            foot.insert(0, ("settings.title", "Return to title", SetAct::Title));
        }
        if popup && self.retreat {
            foot.insert(0, ("settings.retreat", "Retreat", SetAct::Retreat));
        }
        for (id, label, a) in foot {
            let bw = ui.p.text_w(label, BODY).ceil() as i32 + 14;
            bx -= bw;
            ui.button(id, Rect::new(bx, fy, bw, fh), label, a, true);
            bx -= 6;
        }
        if !popup {
            let note = "Changes apply at once; Display Mode and Resolution wait for Apply";
            let room = (bx - r.x - 14) as f32;
            if ui.p.text_w(note, HELP) <= room {
                ui.p.text((r.x + 12) as f32, fy as f32 + (fh as f32 - ui.p.line_h(HELP)) / 2.0, note, HELP, t.dim);
            }
        }
        let area = Rect::new(r.x + 10, top + 2, r.w - 20, fy - 5 - (top + 2));
        // the rail
        let rail_w = if popup { 92 } else { 120 };
        let lh = ui.p.line_h(BODY + 1.0);
        let tab_h = if popup { ((lh + 8.0).ceil() as i32).max(20) } else { ((lh + ui.p.line_h(HELP) + 8.0).ceil() as i32).max(27) };
        for (i, (name, sub)) in TABS.iter().enumerate() {
            let tr = Rect::new(area.x, area.y + i as i32 * (tab_h + 4), rail_w, tab_h);
            let open = self.tab == i;
            let hover = ui.hover(tr);
            ui.p.panel(tr, if open { t.selected } else if hover { t.card_hot } else { t.card }, if open { t.focus } else if hover { t.edge_hot } else { t.edge }, 2.0);
            let col = if open { t.gold } else { t.text };
            let room = (rail_w - 20) as f32;
            if popup {
                ui.p.text_fit((tr.x + 7) as f32, tr.y as f32 + (tab_h as f32 - lh) / 2.0, name, BODY + 1.0, room, col);
            } else {
                let y = tr.y as f32 + (tab_h as f32 - lh - ui.p.line_h(HELP)) / 2.0;
                ui.p.text_fit((tr.x + 7) as f32, y, name, BODY + 1.0, room, col);
                let sub = if i == 0 && self.web { "Fullscreen & text" } else { sub };
                ui.p.text_fit((tr.x + 7) as f32, y + lh, sub, HELP, room, t.dim);
            }
            aegis_ui::ui::draw_glyph(&mut ui.p, Glyph::ChevronRight, Rect::new(tr.x + tr.w - 14, tr.y + (tab_h - 8) / 2, 8, 8), if open { crate::tokens::accent() } else { t.dim });
            ui.push(format!("settings.tab.{}", name.to_lowercase()), "tab", *name, tr, true, open, Some(SetAct::Tab(i)), Vec::new());
        }
        // the tab body: its heading (and the Display badge or Sound's Mute all), then its rows, scrolling when taller
        let outer = Rect::new(area.x + rail_w + 7, area.y, area.w - rail_w - 7, area.h);
        let fill = t.game("settings_body", rgba(24, 26, 32, 255));
        ui.p.panel(outer, fill, t.edge, 2.0);
        let body = outer.pad(Insets::all(6));
        let heads = ["Display & Text", "Audio & Soundscape", "Gameplay & Exploration", "Battle & Timeline Planner", "Controls & Keybindings"];
        let hh = (ui.p.line_h_face(Typeface::Display, HEAD).ceil() as i32).max(16);
        ui.p.text_face(Typeface::Display, body.x as f32, body.y as f32, heads[self.tab], HEAD, t.gold);
        let d = cx.settings.v.clone();
        if self.tab == 0 && !self.web {
            let size = d["window_size"].as_str().unwrap_or("");
            let badge = match WindowShow::of(&d) {
                WindowShow::Fullscreen => "Active: Fullscreen".to_string(),
                WindowShow::Borderless => format!("Active: Borderless · {size}"),
                WindowShow::Windowed => format!("Active: Windowed · {size}"),
            };
            let bw = ui.p.text_w(&badge, HELP).ceil() as i32 + 10;
            let br = Rect::new(body.x + body.w - bw, body.y + 1, bw, (ui.p.line_h(HELP).ceil() as i32 + 3).min(hh));
            ui.p.panel(br, t.selected, t.focus, 1.0);
            ui.p.text((br.x + 5) as f32, br.y as f32 + 1.5, &badge, HELP, t.text);
        }
        if self.tab == 1 {
            // the mock's header pair: Mute in Background (PR7-26; the engine's key, on by default) left of Mute all
            let mute = d["mute"].as_bool().unwrap_or(false);
            let label = if mute { "Mute all: On" } else { "Mute all: Off" };
            let bw = ui.p.text_w("Mute all: Off", BODY).ceil() as i32 + 14;
            Self::toggle(&mut ui, "settings.mute", Rect::new(body.x + body.w - bw, body.y, bw, 14), label, mute, SetAct::Toggle("mute"), Vec::new());
            let away = d[MUTE_IN_BACKGROUND].as_bool().unwrap_or(true);
            let label = if away { "Mute in Background: On" } else { "Mute in Background: Off" };
            let aw = ui.p.text_w("Mute in Background: Off", BODY).ceil() as i32 + 14;
            let help = tip("Mute in Background", "Silences every sound while the game's window is not in focus; the volumes stay as set.");
            Self::toggle(&mut ui, "settings.mute_in_background", Rect::new(body.x + body.w - bw - 6 - aw, body.y, aw, 14), label, away, SetAct::Toggle(MUTE_IN_BACKGROUND), help);
        }
        let view = Rect::new(body.x, body.y + hh + 3, body.w, body.h - hh - 3);
        let rows = self.rows(cx);
        let inner_w = view.w - 5;
        // CONTROLS-UI1: the Controls tab's view and its layout (None: the default); a layout's problems go to the notice
        let controls = matches!(rows.first(), Some(Row::Controls)).then(|| crate::controls_view::view(&cx.settings.v, &self.rebind));
        let layout = controls.as_ref().and_then(|v| self.layouts.get(v).cloned());
        cx.notice.extend(self.layouts.take_problems());
        let heights: Vec<i32> = rows.iter().map(|row| if layout.is_some() && matches!(row, Row::Controls) { view.h } else { self.row_height(&mut ui, cx, *row, inner_w) }).collect();
        let content = heights.iter().sum::<i32>() + GAP * (rows.len() as i32 - 1).max(0);
        let off = ui.scroll_begin("settings.body", view, content);
        let mut y = view.y - off;
        for (row, h) in rows.iter().zip(&heights) {
            let r = Rect::new(view.x, y, inner_w, *h);
            match (row, &controls) {
                (Row::Controls, Some(v)) => {
                    let art = &mut cx.art;
                    ui.nest(SetAct::View, |sub| aegis_ui::screen_layout::draw(sub, art, layout.as_ref(), v, r));
                }
                _ => self.draw_row(&mut ui, cx, *row, r),
            }
            y += h + GAP;
        }
        ui.scroll_end();
        // the keep-or-revert confirm and the close guard draw over the screen and take its input
        if modal {
            ui.widgets.clear();
            ui.mouse = mouse;
            let size = |v: &Value| v["window_size"].as_str().unwrap_or("").to_string();
            let named = |v: &Value| match WindowShow::of(v) {
                WindowShow::Fullscreen => "Fullscreen".to_string(),
                WindowShow::Borderless => format!("Borderless · {}", size(v)),
                WindowShow::Windowed => format!("Windowed · {}", size(v)),
            };
            if let Some(left) = self.confirm_left(Instant::now()) {
                let lines = [(format!("Display switched to {}.", named(&d)), BODY, t.text), (format!("Reverting to the previous display in {left} s."), BODY, t.foe)];
                Self::modal(&mut ui, "Keep Display Settings?", &lines, &[("settings.confirm.keep", "Keep changes [Enter]".into(), SetAct::Keep, true), ("settings.confirm.revert", "Revert".into(), SetAct::Revert, false)], SetAct::Revert);
            } else if self.guard.is_some() {
                let lines = [
                    (format!("You chose {} without pressing Apply.", named(&self.pending)), BODY, t.text),
                    ("All other settings apply at once. Apply or discard the display change before closing?".to_string(), HELP, t.dim),
                ];
                let buttons = [("settings.guard.apply", "Apply display [Enter]".to_string(), SetAct::GuardApply, true), ("settings.guard.discard", "Discard & close".to_string(), SetAct::GuardDiscard, false), ("settings.guard.stay", "Stay".to_string(), SetAct::GuardStay, false)];
                Self::modal(&mut ui, "Unapplied Display Changes?", &lines, &buttons, SetAct::GuardStay);
            }
        }
        self.w = ui.widgets;
    }
    fn state(&self, cx: &Cx) -> Value {
        json!({"tab": TABS[self.tab].0, "popup": self.popup, "settings": cx.settings.v, "pending": self.pending, "display_dirty": self.display_dirty(cx),
            "confirm_left": self.confirm_left(Instant::now()), "guard": self.guard.map(|g| format!("{g:?}").to_lowercase()), "rebind": crate::controls_view::state(&self.rebind)})
    }
    widget_plumbing!();
}

/// VISCLAR1-A, INPUT1-PB: the row pitch of the Battle tab's overlay checkboxes (25 groups in 13 rows, then the debug
/// row).
const OVERLAY_PITCH: i32 = 14;

/// Volume sliders move in steps of 5 (0-100).
const VOLUME_STEP: f64 = 5.0;
/// The debug Walk step slider's step, ms.
const WALK_STEP_STEP: f64 = 10.0;

fn static_key(k: &str) -> &'static str {
    ["master", "music", "effects", "interface", "ambience"].into_iter().find(|x| *x == k).unwrap_or("master")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::{Game, GameCx};
    use aegis_ui::canvas::Canvas;
    use aegis_ui::painter::{Fonts, View};
    use aegis_ui::ui::Kit;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    /// PLAYUI1 section 1 rules 1 and 3: which rows click and which icons show, by mode and slot state.
    #[test]
    fn a_slot_row_clicks_only_where_its_action_applies() {
        let row = |saving, slot, exists, ok, locked| slot_row(saving, slot, exists, ok, locked, "");
        assert_eq!(row(true, 0, true, true, false).action, None, "the game writes the autosave");
        assert_eq!(row(true, 0, true, true, false).why, "The game writes the autosave.");
        assert!(!row(true, 0, true, true, false).lock && row(true, 0, true, true, false).delete, "the autosave has delete, never a lock");
        assert_eq!(row(true, 2, false, false, false).action, Some(SlotAct::Save(2)));
        assert!(!row(true, 2, false, false, false).lock && !row(true, 2, false, false, false).delete, "an empty slot has no icons");
        assert_eq!(row(true, 2, true, true, false).action, Some(SlotAct::Save(2)), "a filled slot saves (the screen asks first)");
        assert_eq!(row(true, 2, true, true, true).action, None);
        assert_eq!(row(true, 2, true, true, true).why, "Locked: unlock it to save over it.");
        assert_eq!(slot_row(true, 2, false, false, false, "There is no game to save.").why, "There is no game to save.");
        assert_eq!(row(false, 2, true, true, true).action, Some(SlotAct::Load(2)), "a locked save still loads");
        assert_eq!(row(false, 0, true, true, false).action, Some(SlotAct::Load(0)));
        assert_eq!((row(false, 3, false, false, false).action, row(false, 3, false, false, false).why.as_str()), (None, "Empty slot."));
        let broken = row(false, 3, true, false, false);
        assert!(broken.action.is_none() && broken.lock && broken.delete, "an unreadable manual save can still be locked or deleted");
    }

    fn cx(save_dir: PathBuf) -> Cx {
        let data_dir = emberwake_world::Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let data = emberwake_world::Data::open(&data_dir).expect("data").0;
        let world = new_game::playable_worlds(&data).first().cloned().expect("a playable world");
        let gs = new_game::create_for_world(&data, &world, 1).expect("a new game");
        let app = GameCx {
            data: Arc::new(data),
            manifest: Default::default(),
            game: Some(Game::new(gs, -1, false)),
            save_dir,
            tuning: aegis_ui::tuning::TuningFile::new(None),
            log: Vec::new(),
            encounter: String::new(),
            seed: None,
            debug: false,
            data_dir,
        };
        aegis_ui::Cx {
            settings: Settings::load(std::env::temp_dir().join("emberwake_slots_no_settings.json"), default_settings()),
            art: aegis_ui::art::Art::new(None),
            audio: aegis_ui::audio::Audio::offline(None),
            nav: Vec::new(),
            notice: Vec::new(),
            shot_dir: std::env::temp_dir(),
            quit: false,
            answers: Vec::new(),
            playtest: aegis_ui::playtest::Playtests::off(),
            held: Default::default(),
            app,
        }
    }

    fn gfx() -> Gfx {
        let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        Gfx { canvas: Canvas::new(W, H), view: View::fit(W as u32, H as u32), fonts: Fonts::new(&font), kit: Kit::default() }
    }

    /// The screen's widget infos after a frame.
    fn frame(s: &mut Slots, cx: &mut Cx, g: &mut Gfx) -> Vec<Value> {
        s.draw(cx, g, (-1, -1), true);
        s.widgets()
    }

    fn get<'a>(w: &'a [Value], id: &str) -> Option<&'a Value> {
        w.iter().find(|x| x["id"] == id)
    }

    fn n(v: &Value, i: usize) -> i32 {
        v["rect"][i].as_i64().unwrap() as i32
    }

    fn centre(v: &Value) -> (i32, i32) {
        (n(v, 0) + n(v, 2) / 2, n(v, 1) + n(v, 3) / 2)
    }

    /// Runs the Yes of the question the last click asked.
    fn answer_yes(cx: &mut Cx, g: &mut Gfx) {
        let Some(Nav::Push(mut ask)) = cx.nav.pop() else { panic!("a question was asked") };
        ask.draw(cx, g, (-1, -1), true);
        ask.click_id(cx, "ask.yes").unwrap();
        cx.nav.clear();
    }

    /// UISPLIT1-H Stage A (`save_view`): the slot list's view says what the drawn rows say: a row per slot, on exactly
    /// when its card takes a click, its note the card's line, lock or unlock and delete as buttons where the icons show
    /// (delete off while locked); the agent link carries it; a view click maps to the screen's own act.
    #[test]
    fn the_slots_view_matches_the_drawn_rows() {
        use crate::save_view::{slot_act, view, BACK, DELETE, LOCK, OPEN, UNLOCK};
        use aegis_ui::screen_layout::{Act, Field};
        let dir = aegis_core::scratch::Scratch::vacant("emberwake_slots_view");
        let mut cx = cx(dir.to_path_buf());
        let mut g = gfx();
        assert!(save::write(&cx.data, &dir, cx.gs(), 2, 0).is_empty() && save::write(&cx.data, &dir, cx.gs(), 3, 0).is_empty());
        save::set_locked(&dir, 3, true).unwrap();
        for saving in [true, false] {
            let mut s = Slots::new(saving);
            let w = frame(&mut s, &mut cx, &mut g);
            let v = view(&cx, saving);
            assert_eq!(s.state(&cx)["view"], v.to_json(), "the agent link's view");
            let Some(Field::Rows(rows)) = v.get("slots") else { panic!("slots") };
            assert_eq!(rows.len() as i64, save::LAST_SLOT - save::AUTOSAVE_SLOT + 1);
            for r in rows {
                let card = get(&w, &format!("slot.{}", r.id)).unwrap();
                assert_eq!(card["enabled"].as_bool(), Some(r.enabled), "{saving} slot {}", r.id);
                assert_eq!(card["tip"][1].as_str(), Some(r.note.as_str()), "{saving} slot {}", r.id);
                let has = |a: &str| r.buttons.iter().find(|b| b.action == a);
                assert_eq!(get(&w, &format!("slot.{}.lock", r.id)).is_some(), has(LOCK).or(has(UNLOCK)).is_some(), "slot {}", r.id);
                assert_eq!(get(&w, &format!("slot.{}.delete", r.id)).is_some(), has(DELETE).is_some(), "slot {}", r.id);
            }
            let three = rows.iter().find(|r| r.id == "3").unwrap();
            assert!(three.buttons.iter().any(|b| b.action == UNLOCK) && three.buttons.iter().any(|b| b.action == DELETE && !b.enabled), "locked: unlock, delete off");
            assert!(!rows[0].detail.is_empty() || !json::b(&save::peek(&cx.data, &dir, 0), "ok", false), "a readable save shows its day");
            assert!(!rows.iter().find(|r| r.id == "2").unwrap().detail.is_empty(), "slot 2's day");
            let act = |a: &str, arg: Option<&str>| slot_act(saving, &Act { action: a.into(), arg: arg.map(str::to_string) });
            assert_eq!(act(OPEN, Some("2")), Some(if saving { SlotAct::Save(2) } else { SlotAct::Load(2) }));
            assert_eq!((act(LOCK, Some("2")), act(UNLOCK, Some("3")), act(DELETE, Some("2")), act(BACK, None)), (Some(SlotAct::Lock(2, true)), Some(SlotAct::Lock(3, false)), Some(SlotAct::Delete(2)), Some(SlotAct::Back)));
            assert_eq!((act(OPEN, None), act("nope", Some("1"))), (None, None));
        }
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_slots_fixture_has_the_view_shape() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, crate::save_view::SCREEN).expect("the fixture file").expect("it parses");
        let dir = aegis_core::scratch::Scratch::vacant("emberwake_slots_fixture");
        let cx = cx(dir.to_path_buf());
        let shape = |v: &aegis_ui::screen_layout::View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&crate::save_view::view(&cx, true)));
    }

    /// Writes the fixture (by hand: `cargo test -p emberwake_gui write_slots_fixture -- --ignored`): the save screen with
    /// a save in slot 2 and a locked one in slot 3.
    #[test]
    #[ignore]
    fn write_slots_fixture() {
        let dir = aegis_core::scratch::Scratch::vacant("emberwake_slots_write");
        let cx = cx(dir.to_path_buf());
        assert!(save::write(&cx.data, &dir, cx.gs(), 2, 0).is_empty() && save::write(&cx.data, &dir, cx.gs(), 3, 0).is_empty());
        save::set_locked(&dir, 3, true).unwrap();
        let v = crate::save_view::view(&cx, true).to_json();
        std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/slots.view.json"), serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
    }

    /// PR7-29 (FB-369): a save that names no location reads "World · time" with no comma and no empty part; one that
    /// names a location reads "World, Location".
    #[test]
    fn a_save_with_no_location_drops_the_comma() {
        let dir = aegis_core::scratch::Scratch::vacant("emberwake_slot_place");
        let cx = cx(dir.to_path_buf());
        let world = cx.gs().world_id.clone();
        let name = cx.world_name(&world);
        let head = |loc: &str| json::dict(&json!({"world_id": world, "location_id": loc})).clone();
        assert_eq!(slot_place(&cx, &head("")), name);
        let loc = cx.data.merged_entries(&world, "locations").first().map(|l| s(l, "id")).expect("a location");
        assert_eq!(slot_place(&cx, &head(&loc)), format!("{name}, {}", cx.location_name(&world, &loc)));
        let head = json::dict(&json!({"exists": true, "ok": true, "world_id": world, "location_id": "", "saved_at": 1_700_000_000, "party": []})).clone();
        let (label, _) = slot_text(&cx, 1, &head);
        assert!(label.starts_with(&format!("Slot 1 · {name} · ")) && !label.contains(", "), "{label}");
    }

    /// PLAYUI1 section 1 acceptance over the screen's widgets (the agent link's ids): `slot.2` saves an empty slot at
    /// once and asks on a filled one; `slot.2.lock` locks it, after which `slot.2` and its delete take no click and
    /// their cards name the lock; the autosave has no lock; the load screen loads a locked save; unlock, delete.
    #[test]
    fn the_save_screen_rows_and_icons_follow_the_slot_rules() {
        let dir = aegis_core::scratch::Scratch::vacant("emberwake_slots");
        let mut cx = cx(dir.to_path_buf());
        let mut g = gfx();
        let mut s = Slots::new(true);
        let w = frame(&mut s, &mut cx, &mut g);
        assert_eq!(get(&w, "slot.0").unwrap()["enabled"], false);
        assert_eq!(get(&w, "slot.0").unwrap()["tip"][1], "The game writes the autosave.");
        assert!(get(&w, "slot.2.lock").is_none() && get(&w, "slot.2.delete").is_none(), "an empty slot shows no icons");

        s.click_id(&mut cx, "slot.2").unwrap();
        assert!(cx.nav.is_empty() && save::exists(&dir, 2), "an empty slot saves at once");
        let w = frame(&mut s, &mut cx, &mut g);
        assert!(get(&w, "slot.2.lock").is_some() && get(&w, "slot.2.delete").is_some());
        s.click_id(&mut cx, "slot.2").unwrap();
        assert!(matches!(cx.nav.last(), Some(Nav::Push(_))), "a filled slot asks before overwriting");
        cx.nav.clear();

        // PR7-29 (SLOTCARD1): the card is slot 2's place in the two-column grid; its icons are the card's corner icons
        // (`panes::slot_rects`), right to left: delete, lock.
        let (row, del, lock) = (get(&w, "slot.2").unwrap(), get(&w, "slot.2.delete").unwrap(), get(&w, "slot.2.lock").unwrap());
        let r = card_rect(2);
        assert_eq!((n(row, 0), n(row, 1), n(row, 2), n(row, 3)), (r.x, r.y, CARD_W, CARD_H));
        let (_, _, _, icons) = panes::slot_rects(r, 2);
        assert_eq!((n(del, 0), n(del, 1), n(del, 2)), (icons[1].x, icons[1].y, icons[1].w));
        assert_eq!((n(lock, 0), n(lock, 1)), (icons[0].x, icons[0].y));
        assert!(n(lock, 0) + n(lock, 2) <= n(del, 0), "the lock is left of delete");

        s.click_id(&mut cx, "slot.2.lock").unwrap();
        assert!(save::is_locked(&dir, 2) && cx.notice.is_empty());
        let w = frame(&mut s, &mut cx, &mut g);
        let (row, lock, del) = (get(&w, "slot.2").unwrap(), get(&w, "slot.2.lock").unwrap(), get(&w, "slot.2.delete").unwrap());
        assert_eq!((row["enabled"].as_bool(), row["tip"][1].as_str()), (Some(false), Some("Locked: unlock it to save over it.")));
        assert_eq!((lock["active"].as_bool(), lock["tip"][0].as_str()), (Some(true), Some("Unlock")));
        assert_eq!((del["enabled"].as_bool(), del["tip"][1].as_str()), (Some(false), Some("Unlock the save to delete it.")));
        assert!(s.click_id(&mut cx, "slot.2").is_err(), "a locked slot takes no save click");
        assert!(!s.click(&mut cx, centre(del)), "a click on the disabled delete does nothing");
        assert!(cx.nav.is_empty() && save::exists(&dir, 2));

        // The autosave: delete, never a lock. The load screen loads a locked save.
        assert!(save::write(&cx.data, &dir, cx.gs(), save::AUTOSAVE_SLOT, 0).is_empty());
        let mut load = Slots::new(false);
        let w = frame(&mut load, &mut cx, &mut g);
        assert!(get(&w, "slot.0.lock").is_none() && get(&w, "slot.0.delete").is_some());
        assert_eq!(get(&w, "slot.1").unwrap()["tip"][1], "Empty slot.");
        assert_eq!(get(&w, "slot.2").unwrap()["enabled"], true, "a locked save loads");
        assert!(!load.click(&mut cx, centre(get(&w, "slot.2.delete").unwrap())), "the disabled delete does not load either");

        s.click_id(&mut cx, "slot.2.lock").unwrap();
        assert!(!save::is_locked(&dir, 2));
        let w = frame(&mut s, &mut cx, &mut g);
        assert_eq!(get(&w, "slot.2.lock").unwrap()["tip"][0], "Lock (protect from overwrite and delete)");
        s.click_id(&mut cx, "slot.2.delete").unwrap();
        assert!(save::exists(&dir, 2), "delete asks first");
        answer_yes(&mut cx, &mut g);
        assert!(!save::exists(&dir, 2) && cx.notice.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- LOOK1-M1: the main menu on the panes -------------------------------------------------------------------

    const MENU_IDS: [&str; 5] = ["menu.new_game", "menu.load", "menu.settings", "menu.tools", "menu.quit"];

    fn scratch(name: &str) -> aegis_core::scratch::Scratch {
        let dir = aegis_core::scratch::Scratch::vacant(&format!("emberwake_m1_{name}"));
        dir
    }

    /// A frame of the menu in the view the shell gives it, for the window `g.canvas` is.
    fn menu_frame(m: &mut MainMenu, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32)) -> Vec<Value> {
        g.view = View::fit(g.canvas.w as u32, g.canvas.h as u32).for_grid(m.grid());
        m.draw(cx, g, mouse, true);
        m.widgets()
    }

    /// A 1280 x 720 window, the menu's base.
    fn menu_gfx() -> Gfx {
        let mut g = gfx();
        g.canvas = Canvas::new(CANVAS.w, CANVAS.h);
        g
    }

    /// The canvas of a native screen under the fixed canvas.
    const CANVAS: Rect = Rect::new(0, 0, 1280, 720);
    /// A point on Continue's label, the first row with a save (and on New game's without one).
    const OVER_FIRST: (i32, i32) = (860, 310);

    /// The column at the Normal text size, on the base canvas.
    fn base_column() -> Rect {
        cut_column(CANVAS, resolve(&[frac(MENU_W)], CANVAS.w)[0], 0).0
    }

    /// LAYOUT720-E5: what a manifest file declares reaches the shell's reader as written, with no notice.
    #[test]
    fn a_manifest_files_canvas_and_window_sizes_read_clean() {
        let dir = scratch("menu_manifest");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("game.json");
        std::fs::write(&path, r#"{"id": "demo", "title": "Demo", "canvas": {"policy": "fit", "w": 1280, "h": 800}, "window_sizes": ["1280x800", "2560x1600"]}"#).unwrap();
        let m = aegis_content::manifest::Manifest::read(&path).unwrap();
        let (policy, sizes, notices) = aegis_ui::shell::read_window_options(&m.canvas, &m.window_sizes);
        assert_eq!(notices, Vec::<String>::new());
        assert_eq!((policy, sizes), (aegis_ui::painter::CanvasPolicy::Fit { w: 1280, h: 800 }, vec!["1280x800".to_string(), "2560x1600".to_string()]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// LAYOUT720-E5: the main menu must pass `desktop` at every text size, with and without its peek, on the default and the
    /// game's theme; every other profile only has to resolve, and says what it finds. `AEGIS_AUDIT_PRINT=1` prints the answers.
    #[test]
    fn the_main_menu_passes_the_desktop_profile_and_resolves_on_every_other() {
        use aegis_ui::layout_audit::{audit_screen, NO_MOUSE};
        let dir = scratch("menu_audit");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        cx.app.manifest.title = "Emberwake".into();
        cx.app.manifest.subtitle = "A Scarlet Lullaby".into();
        write_fixture(&mut cx, 12);
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let shipped = aegis_ui::theme::load(&aegis_core::assets::Layers::for_game(&game)).expect("the game's theme loads");
        cx.art = aegis_ui::art::Art::new(Some(game));
        let before = (g.canvas.w, g.canvas.h, g.view);
        for (name, t) in [("default", aegis_ui::theme::Theme::default()), ("game", shipped)] {
            aegis_ui::theme::set_theme(t);
            for (state, mouse) in [("idle", NO_MOUSE), ("peek", OVER_FIRST)] {
                let mut m = MainMenu::default();
                let v = audit_screen(&mut m, &mut cx, &mut g, aegis_ui::painter::CanvasPolicy::default(), mouse, None);
                if std::env::var_os("AEGIS_AUDIT_PRINT").is_some() {
                    for p in v["profiles"].as_array().unwrap() {
                        println!("{name} {state} {}: canvas {} classes {} counts {} text {} issues {}", p["profile"], p["canvas"], p["classes"], p["counts"], p["text_counts"], p["issues"]);
                    }
                }
                assert_eq!(v["pass"], json!(true), "{name} {state}: {v}");
                assert_eq!(v["profiles"].as_array().unwrap().len(), aegis_ui::layout_audit::PROFILES.len(), "{v}");
                assert_eq!((g.canvas.w, g.canvas.h, g.view), before, "left as it was found");
            }
            // The state above had its peek on the canvas it was written for, at the largest text size too.
            let mut m = MainMenu::default();
            let desktop = aegis_ui::layout_audit::profile_named("desktop").unwrap();
            let v = audit_screen(&mut m, &mut cx, &mut g, aegis_ui::painter::CanvasPolicy::default(), OVER_FIRST, Some(desktop));
            assert_eq!(v["pass"], json!(true), "{name}: {v}");
            assert!(get(&m.widgets(), "menu.peek").is_some(), "{name}: the peek was drawn and audited");
        }
        aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The picture signed off in old units (LOOK1-M1), doubled, each edge within a unit: column x 420 to 580 from the top
    /// 150, the peek's right edge 12 left of it, the logo's room x 390 to 625 from the top 13 and 62 high, the typeset logo
    /// at 620, 40. The shares are picked to land there exactly: the art's width is what T4 could not lose.
    #[test]
    fn the_main_menu_is_native_and_lands_on_the_signed_off_picture() {
        assert_eq!(MainMenu::default().grid(), Grid::Native);
        let (column, room) = cut_column(CANVAS, resolve(&[frac(MENU_W)], CANVAS.w)[0], 24);
        assert_eq!((column.x, column.y, column.w), (840, 300, 320), "the column is the old one doubled");
        assert_eq!((room.x + room.w, room.y), (816, 300), "the peek's room ends the old gap left of it");
        let logo = logo_room(CANVAS);
        assert_eq!((logo.x, logo.y, logo.w, logo.h), (780, 26, 470, 124), "the old room (and 62 high) doubled: the art lands where it did");
        let at = logo_point(CANVAS);
        assert_eq!((at.x, at.y), (1240, 80), "the typeset logo");
        assert_eq!((resolve(&[frac(PEEK_W)], CANVAS.w)[0], resolve(&[frac(PEEK_H)], CANVAS.h)[0]), (320, 128), "the peek's least size");
    }

    /// The layout is calculated from the canvas: on other canvases the column keeps its shares and its cuts' relations, and
    /// the room left of it ends one gap away, at the same top.
    #[test]
    fn the_menu_layout_follows_the_canvas() {
        for area in [CANVAS, Rect::new(0, 0, 1720, 720), Rect::new(0, 0, 1280, 800), Rect::new(0, 0, 800, 600), Rect::new(10, 20, 2560, 1440)] {
            let w = resolve(&[frac(MENU_W)], area.w)[0];
            let (column, room) = cut_column(area, w, 24);
            assert_eq!(area.x + area.w - (column.x + column.w), resolve(&[frac(MENU_RIGHT)], area.w)[0], "{area:?}: the strip right of it");
            assert_eq!((column.y - area.y, column.y + column.h), (resolve(&[frac(MENU_TOP)], area.h)[0], area.y + area.h), "{area:?}: from its top to the bottom");
            assert_eq!((room.x + room.w + 24, room.x, room.y, room.h), (column.x, area.x, column.y, column.h), "{area:?}: the room left of it");
            let logo = logo_room(area);
            assert!(logo.x >= area.x && logo.x + logo.w <= area.x + area.w && logo.y + logo.h <= column.y, "{area:?}: the logo's room stays above the column");
        }
    }

    /// `logo_px` keeps the room's right edge and top and the picture's aspect, at any scale.
    #[test]
    fn the_logo_keeps_its_aspect_on_the_room_edges() {
        for (ww, wh) in [(1280u32, 720u32), (1366, 768), (1920, 1080), (3840, 2160)] {
            let v = View::fit(ww, wh).for_grid(Grid::Native);
            let room = v.rect(logo_room(CANVAS));
            for size in [(1246, 325), (64, 64), (325, 1246)] {
                let pr = logo_px(v, logo_room(CANVAS), size);
                assert_eq!((pr.x + pr.w, pr.y), (room.x + room.w, room.y), "{ww}: {size:?}");
                assert!(pr.w == room.w || pr.h == room.h, "{ww}: {size:?}: as large as fits: {pr:?} in {room:?}");
                let (want_h, want_w) = (pr.w as f32 * size.1 as f32 / size.0 as f32, pr.h as f32 * size.0 as f32 / size.1 as f32);
                assert!((pr.h as f32 - want_h).abs() <= 1.0 || (pr.w as f32 - want_w).abs() <= 1.0, "{ww}: {size:?}: the aspect kept: {pr:?}");
            }
        }
    }

    fn buttons(w: &[Value]) -> Vec<String> {
        w.iter().filter(|x| x["kind"] == "text_button").map(|x| x["id"].as_str().unwrap().to_string()).collect()
    }

    /// Writes the running game with day `time` as the fixture save.
    fn write_fixture(cx: &mut Cx, time: i64) {
        cx.gs_mut().time = time;
        assert!(save::write(&cx.data, &cx.save_dir, cx.gs(), 2, 1000).is_empty());
    }

    /// The buttons keep their ids and actions, top-down in the focus order: Continue (only with a save), then the rest,
    /// at the brief's rects at the Normal text size; the opening focus is the first.
    #[test]
    fn the_main_menu_keeps_its_ids_and_actions() {
        let dir = scratch("ids");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        let mut m = MainMenu::default();
        assert_eq!(buttons(&menu_frame(&mut m, &mut cx, &mut g, (-1, -1))), MENU_IDS, "no save: no Continue");
        write_fixture(&mut cx, 3);
        m.resume(&mut cx);
        let w = menu_frame(&mut m, &mut cx, &mut g, (-1, -1));
        let mut want = vec!["menu.continue"];
        want.extend(MENU_IDS);
        assert_eq!(buttons(&w), want);
        for (i, id) in want.iter().enumerate() {
            let (r, m, column) = (get(&w, id).unwrap(), aegis_ui::theme::Metrics::of(&th(), g.view), base_column());
            assert_eq!((n(r, 0), n(r, 1), n(r, 3)), (column.x + m.padding - m.u(panes::MENU_TEXT_MARGIN), column.y + m.u(MENU_STEP) * i as i32, m.u(MENU_H)), "{id}: the label at the padding, less its margin");
            assert!(n(r, 2) < column.w, "{id}: the label's width, not the column's");
            assert_eq!(r["clickable"], json!(true), "{id}");
        }
        assert!(m.auto_focus());
        let ring: Vec<&str> = aegis_ui::focus::ring(&w, false).iter().map(|x| x["id"].as_str().unwrap()).collect();
        assert_eq!(ring, want, "the backdrop and the peek take no focus");
        cx.app.game.as_mut().unwrap().clean = cx.gs().to_value();
        for (id, screen) in [("menu.new_game", "new_game"), ("menu.settings", "settings"), ("menu.tools", "scene_runner")] {
            m.click_id(&mut cx, id).unwrap();
            let Some(Nav::Push(s)) = cx.nav.pop() else { panic!("{id} pushes a screen") };
            assert_eq!((s.id(), cx.nav.len()), (screen, 0), "{id}");
        }
        m.click_id(&mut cx, "menu.load").unwrap();
        let Some(Nav::Push(s)) = cx.nav.pop() else { panic!("Load pushes a screen") };
        assert_eq!(s.id(), Slots::new(false).id());
        m.click_id(&mut cx, "menu.quit").unwrap();
        assert!(cx.quit, "Quit with nothing unsaved");
        cx.gs_mut().time = 99;
        m.click_id(&mut cx, "menu.continue").unwrap();
        assert_eq!(cx.gs().time, 3, "Continue loads the newest save");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Continue's peek reads the fixture save (its location, day (the clock plus one) and at most four of its party, the
    /// active members first), at the brief's rect, while Continue is hovered or the keyboard moved onto it, and only then:
    /// the focus the menu opens with (or comes back with) does not show it.
    #[test]
    fn the_peek_reads_the_newest_save_while_continue_is_hovered_or_focused() {
        let dir = scratch("peek");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        let mut extra = cx.gs().party.members[0].clone();
        for i in 0..5 {
            extra.id = format!("m1_extra_{i}");
            cx.gs_mut().party.members.push(extra.clone());
        }
        write_fixture(&mut cx, 7);
        let gs = save::read(&cx.data, &dir, 2).expect("the fixture loads").state;
        // The game in play moves on: the peek shows the save, not the running game.
        cx.gs_mut().time = 50;
        let mut m = MainMenu::default();
        assert!(get(&menu_frame(&mut m, &mut cx, &mut g, (-1, -1)), "menu.peek").is_none(), "idle: no peek");
        assert_eq!(m.frame_need(&cx), FrameNeed::Idle, "FPSCAP1: nothing moves on the menu without a peek");
        let w = menu_frame(&mut m, &mut cx, &mut g, OVER_FIRST);
        let p = get(&w, "menu.peek").expect("hovered: the peek");
        assert_eq!(m.frame_need(&cx), FrameNeed::Full, "FPSCAP1: the peek's portraits move");
        let (gap, face, column) = (g.view.u(PEEK_GAP), g.view.u(panes::PORTRAIT), base_column());
        let (pw, ph) = (resolve(&[frac(PEEK_W)], CANVAS.w)[0], resolve(&[frac(PEEK_H)], CANVAS.h)[0]);
        assert_eq!((n(p, 0) + n(p, 2) + gap, n(p, 1), n(p, 2), n(p, 3)), (column.x, column.y, pw, ph), "one gap left of the column, at its top");
        assert_eq!(p["text"], json!(format!("{} | Day 8", cx.location_name(&gs.world_id, &gs.location_id))));
        let faces: Vec<&Value> = w.iter().filter(|x| x["kind"] == "portrait").collect();
        assert_eq!(faces.len(), 4, "at most four portraits");
        assert!(faces.iter().all(|f| (n(f, 2), n(f, 3)) == (face, face) && n(f, 1) + face <= n(p, 1) + n(p, 3) && n(f, 0) + face <= n(p, 0) + n(p, 2)), "the pane's portrait size, inside the panel");
        let active = gs.party.active_members();
        assert_eq!(faces[0]["id"], json!(format!("menu.peek.{}", active[0].id)), "the active members first");
        assert!(get(&menu_frame(&mut m, &mut cx, &mut g, centre(get(&w, "menu.new_game").unwrap())), "menu.peek").is_none(), "on another item: none");
        let label = get(&w, "menu.continue").unwrap();
        assert!(get(&menu_frame(&mut m, &mut cx, &mut g, (n(label, 0) + n(label, 2) + 10, centre(label).1)), "menu.peek").is_none(), "right of Continue's label: none");
        let mut m = MainMenu::default();
        let focus_on = |m: &mut MainMenu, cx: &mut Cx, g: &mut Gfx, id: &str, visible: bool| {
            g.kit.focus = Some(id.into());
            g.kit.focus_visible = visible;
            get(&menu_frame(m, cx, g, (-1, -1)), "menu.peek").is_some()
        };
        assert!(!focus_on(&mut m, &mut cx, &mut g, "menu.continue", true), "the opening focus: no peek");
        assert!(!focus_on(&mut m, &mut cx, &mut g, "menu.new_game", true), "moved off Continue");
        assert!(focus_on(&mut m, &mut cx, &mut g, "menu.continue", true), "the keyboard moved onto it: the peek");
        assert!(!focus_on(&mut m, &mut cx, &mut g, "menu.continue", false), "a hidden focus does not show it");
        assert!(focus_on(&mut m, &mut cx, &mut g, "menu.continue", true));
        m.draw(&mut cx, &mut g, (-1, -1), false);
        assert!(get(&m.widgets(), "menu.peek").is_none(), "under an overlay: none");
        m.resume(&mut cx);
        assert!(!focus_on(&mut m, &mut cx, &mut g, "menu.continue", true), "back on the menu the focus opens there again: no peek");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The pictures over a scratch asset layer: without the A6 files the MENU2 picture and no crest; with them the title
    /// backdrop and the crest behind the logo. The veil is 0: the centre of the picture stays as it is under the vignette's
    /// zero at the centre.
    #[test]
    fn the_title_art_is_used_when_present_else_the_menu_picture() {
        let dir = scratch("art");
        let root = scratch("art_layer");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/frames/button.png");
        std::fs::create_dir_all(root.join("backgrounds/menu")).unwrap();
        std::fs::copy(&source, root.join(MENU_ART)).unwrap();
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        cx.app.manifest.title = "Fixture".into();
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]));
        let mut m = MainMenu::default();
        let w = menu_frame(&mut m, &mut cx, &mut g, (-1, -1));
        assert_eq!(get(&w, "menu.backdrop").unwrap()["text"], json!(MENU_ART));
        assert_eq!(get(&w, "menu.backdrop").unwrap()["kind"], json!("picture"));
        assert!(get(&w, "menu.crest").is_none(), "no crest file, no crest");
        assert!(get(&w, "menu.logo").is_none(), "no title picture");
        std::fs::copy(&source, root.join(TITLE_ART)).unwrap();
        std::fs::create_dir_all(root.join("ui")).unwrap();
        std::fs::copy(&source, root.join(CREST_ART)).unwrap();
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]));
        let w = menu_frame(&mut m, &mut cx, &mut g, (-1, -1));
        assert_eq!(get(&w, "menu.backdrop").unwrap()["text"], json!(TITLE_ART));
        let crest = get(&w, "menu.crest").expect("the crest");
        assert_eq!(crest["text"], json!(CREST_ART));
        assert!(n(crest, 0) + n(crest, 2) <= CANVAS.w && n(crest, 0) >= 0 && n(crest, 1) >= 0, "on the canvas");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Down from the top visits every row in order and Up comes back, with and without a save, at the four text sizes
    /// (the labels differ in width, the arrows go by the column's left edge).
    #[test]
    fn the_arrow_keys_walk_every_main_menu_row() {
        use aegis_ui::focus::{step, Move};
        let dir = scratch("arrows");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tallow = aegis_ui::theme::load(&aegis_core::assets::Layers::for_game(&game)).expect("the game's theme loads");
        for with_save in [false, true] {
            if with_save {
                write_fixture(&mut cx, 3);
            }
            for t in [aegis_ui::theme::Theme::default(), tallow.clone()] {
                aegis_ui::theme::set_theme(t);
                for (size, scale) in TEXT_SIZES {
                    g.fonts.scale = scale as f32;
                    let w = menu_frame(&mut MainMenu::default(), &mut cx, &mut g, (-1, -1));
                    let rows = buttons(&w);
                    assert_eq!(rows.len(), if with_save { 6 } else { 5 });
                    for (i, id) in rows.iter().enumerate() {
                        let (down, up) = (step(&w, Some(id), Move::Dir(0, 1), false), step(&w, Some(id), Move::Dir(0, -1), false));
                        if let Some(next) = rows.get(i + 1) {
                            assert_eq!(down.as_ref(), Some(next), "Down from {id} at {size}");
                        }
                        if i > 0 {
                            assert_eq!(up.as_ref(), Some(&rows[i - 1]), "Up from {id} at {size}");
                        }
                    }
                }
            }
        }
        aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The real path of a hover at several window sizes (letterboxed too), with and without a save: a physical mouse
    /// position through `View::to_logical` and `Kit::mouse_moved`, the two steps the shell's mouse handler takes, then
    /// a frame: the peek's panel is really drawn on the canvas (its pixels change), on the canvas and left of the column.
    /// Moving off keeps Continue active and its peek; moving to another item drops it. No save: no Continue, nothing drawn.
    #[test]
    fn a_real_hover_draws_the_peek_at_every_window_size() {
        for (ww, wh) in [(960, 540), (1280, 720), (1366, 768), (1920, 1080), (2560, 1080), (1024, 768)] {
            let dir = scratch(&format!("hover_{ww}"));
            let (mut cx, mut g) = (cx(dir.to_path_buf()), gfx());
            g.canvas = Canvas::new(ww, wh);
            let view = View::fit(ww as u32, wh as u32).for_grid(Grid::Native);
            let physical = |w: &[Value], id: &str| {
                let r = get(w, id).unwrap();
                ((n(r, 0) + n(r, 2) / 2) as f32 * view.s + view.ox, (n(r, 1) + n(r, 3) / 2) as f32 * view.s + view.oy)
            };
            let mut m = MainMenu::default();
            let idle = menu_frame(&mut m, &mut cx, &mut g, (-500, -500));
            assert!(get(&idle, "menu.continue").is_none(), "{ww}: no save, no Continue");
            let at = physical(&idle, "menu.quit");
            let mouse = view.to_logical(at.0, at.1);
            g.kit.mouse_moved(&idle, mouse);
            assert!(get(&menu_frame(&mut m, &mut cx, &mut g, mouse), "menu.peek").is_none(), "{ww}: no save, no peek");
            write_fixture(&mut cx, 7);
            g.kit.focus = None;
            g.kit.focus_visible = false;
            m.resume(&mut cx);
            g.kit.focus = Some("menu.continue".into());
            g.kit.focus_visible = true;
            let idle = menu_frame(&mut m, &mut cx, &mut g, (-500, -500));
            assert!(get(&idle, "menu.peek").is_none(), "{ww}: the opening focus shows no peek");
            let before = g.canvas.px.clone();
            let at = physical(&idle, "menu.continue");
            let mouse = view.to_logical(at.0, at.1);
            g.kit.mouse_moved(&idle, mouse);
            let w = menu_frame(&mut m, &mut cx, &mut g, mouse);
            let p = get(&w, "menu.peek").unwrap_or_else(|| panic!("{ww}: hovering Continue draws the peek"));
            let (x, y, pw, ph) = (n(p, 0), n(p, 1), n(p, 2), n(p, 3));
            assert!(x >= 0 && y >= 0 && x + pw <= n(get(&w, "menu.continue").unwrap(), 0) && y + ph <= CANVAS.h, "{ww}: {p}");
            let pr = view.rect(Rect::new(x, y, pw, ph));
            let changed = (pr.y..pr.y + pr.h).flat_map(|py| (pr.x..pr.x + pr.w).map(move |px| ((py * ww + px) * 4) as usize)).filter(|i| g.canvas.px[*i..*i + 4] != before[*i..*i + 4]).count();
            assert!(changed * 2 > (pr.w * pr.h) as usize, "{ww}: the panel is on the canvas ({changed} px changed)");
            assert!(pr.x >= 0 && pr.y >= 0 && pr.x + pr.w <= ww && pr.y + pr.h <= wh, "{ww}: on the canvas {pr:?}");
            let off = (20, 680);
            g.kit.mouse_moved(&w, off);
            let w = menu_frame(&mut m, &mut cx, &mut g, off);
            assert!(get(&w, "menu.peek").is_some(), "{ww}: off the labels Continue stays active, and its peek");
            let at = physical(&w, "menu.load");
            let mouse = view.to_logical(at.0, at.1);
            g.kit.mouse_moved(&w, mouse);
            assert!(get(&menu_frame(&mut m, &mut cx, &mut g, mouse), "menu.peek").is_none(), "{ww}: another item is active");
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    /// A solid `w` x `h` PNG (RGBA), or with `stripes` alternating black and white columns.
    fn write_picture(path: &Path, w: u32, h: u32, stripes: bool) {
        let px: Vec<u8> = (0..w * h).flat_map(|i| if !stripes { [200, 30, 60, 255] } else if (i % w) % 2 == 0 { [0, 0, 0, 255] } else { [255, 255, 255, 255] }).collect();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
    }

    /// The master draws at the room's exact physical size at every window size: the same fraction of the window, its
    /// right edge and top on the room's, the aspect kept, and reduced by averaging (stripes come out grey, not sampled).
    #[test]
    fn the_title_picture_fills_its_room_at_every_window_size() {
        let (iw, ih) = (1246, 325);
        let (dir, root) = (scratch("logo_sizes"), scratch("logo_sizes_layer"));
        std::fs::create_dir_all(root.join("ui")).unwrap();
        write_picture(&root.join(LOGO_ART), iw as u32, ih as u32, false);
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]));
        let room = logo_room(CANVAS);
        for (ww, wh) in [(960, 540), (1280, 720), (1920, 1080), (3840, 2160)] {
            g.canvas = Canvas::new(ww, wh);
            let w = menu_frame(&mut MainMenu::default(), &mut cx, &mut g, (-1, -1));
            let phys = g.view.rect(room);
            let logo = get(&w, "menu.logo").expect("the title picture");
            assert_eq!((n(logo, 0) + n(logo, 2), n(logo, 1)), (room.x + room.w, room.y), "{ww}: on the room's right edge and top");
            assert!((n(logo, 2) - room.w).abs() <= 1 && n(logo, 3) <= room.h, "{ww}: in the room: {logo}");
            let mut hits = (ww, wh, -1, -1);
            for (i, p) in g.canvas.px.chunks(4).enumerate() {
                if p[..3] == [200, 30, 60] {
                    let (x, y) = (i as i32 % ww, i as i32 / ww);
                    hits = (hits.0.min(x), hits.1.min(y), hits.2.max(x), hits.3.max(y));
                }
            }
            let (pw, ph) = (hits.2 - hits.0 + 1, hits.3 - hits.1 + 1);
            assert_eq!(pw, phys.w, "{ww}: the room's whole width");
            assert_eq!(ph, (ih as f32 * phys.w as f32 / iw as f32).round() as i32, "{ww}: the aspect kept");
            assert_eq!(hits.2 + 1, phys.x + phys.w, "{ww}: right-aligned");
            assert_eq!(hits.1, phys.y, "{ww}: at the top");
            assert!((pw as f32 / ww as f32 - room.w as f32 / CANVAS.w as f32).abs() <= 1.0 / ww as f32, "{ww}: the same fraction");
        }
        write_picture(&root.join(LOGO_ART), iw as u32, ih as u32, true);
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]));
        g.canvas = Canvas::new(1280, 720);
        let _ = menu_frame(&mut MainMenu::default(), &mut cx, &mut g, (-1, -1));
        let pr = logo_px(g.view, room, (iw, ih));
        let i = (((pr.y + pr.h / 2) * 1280 + pr.x + pr.w / 2) * 4) as usize;
        let mid = &g.canvas.px[i..i + 3];
        assert!(mid.iter().all(|c| (100..156).contains(c)), "averaged to grey: {mid:?}");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Nothing in the frame's content meets the frame: at the four text sizes and window sizes, no text, portrait or button
    /// of the peek or the ask dialog falls on a pixel of the corners or the line (the file drawn with its centre see-through).
    #[test]
    fn the_ornate_frame_clears_the_peek_and_the_ask_dialog() {
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tallow = aegis_ui::theme::load(&aegis_core::assets::Layers::for_game(&game)).expect("the game's theme loads");
        aegis_ui::theme::set_theme(tallow.clone());
        let frame = tallow.frame("ornate").expect("the ornate frame");
        let mut bare = (*frame.img).clone();
        let (side, slice) = (bare.w, frame.nine.slice[0]);
        for (i, px) in bare.px.chunks_mut(4).enumerate() {
            let (x, y) = (i as i32 % side, i as i32 / side);
            if x >= slice && x < side - slice && y >= slice && y < side - slice {
                px.copy_from_slice(&[0, 0, 0, 0]);
            }
        }
        let dir = scratch("frame_clear");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        write_fixture(&mut cx, 3);
        for (ww, wh) in [(960, 540), (1280, 720), (1920, 1080), (3840, 2160)] {
            for (size, scale) in TEXT_SIZES {
                g.canvas = Canvas::new(ww, wh);
                g.view = View::fit(ww as u32, wh as u32);
                g.fonts.scale = scale as f32;
                // The frame's mask at `r`, then every rect that must stay off it.
                let check = |g: &mut Gfx, what: &str, r: Rect, rects: Vec<Rect>| {
                    let mut mask = Canvas::new(ww, wh);
                    mask.clear([0, 255, 0, 255]);
                    let view = g.view;
                    aegis_ui::painter::Painter { c: &mut mask, v: view, fonts: &mut g.fonts }.nine(&bare, &frame.nine, r);
                    for q in rects {
                        let pq = view.rect(q);
                        for y in pq.y..pq.y + pq.h {
                            for x in pq.x..pq.x + pq.w {
                                assert_eq!(mask.px[((y * ww + x) * 4 + 1) as usize], 255, "{ww} {size} {what}: {q:?} meets the frame at ({x}, {y}), panel {r:?}");
                            }
                        }
                    }
                };
                let rect_of = |v: &Value| Rect::new(n(v, 0), n(v, 1), n(v, 2), n(v, 3));
                let inside = |r: Rect, t: &aegis_ui::painter::TextRec| r.contains((t.x + t.w / 2.0) as i32, (t.y + t.h / 2.0) as i32);
                let text_rect = |t: &aegis_ui::painter::TextRec| Rect::new(t.x.floor() as i32, t.y.floor() as i32, t.w.ceil() as i32 + 1, t.h.ceil() as i32);
                // The peek.
                let mut m = MainMenu::default();
                let idle = menu_frame(&mut m, &mut cx, &mut g, (-1, -1));
                g.fonts.audit = Some(Vec::new());
                let w = menu_frame(&mut m, &mut cx, &mut g, centre(get(&idle, "menu.continue").unwrap()));
                let audit = g.fonts.audit.take().unwrap();
                let peek = rect_of(get(&w, "menu.peek").expect("the peek"));
                let mut rects: Vec<Rect> = audit.iter().filter(|t| inside(peek, t)).map(text_rect).collect();
                assert!(rects.len() >= 2, "{ww} {size}: the peek's texts");
                rects.extend(w.iter().filter(|x| x["kind"] == "text_button" || x["id"].as_str().is_some_and(|i| i.starts_with("menu.peek."))).map(rect_of));
                // The content rect clears the art by itself, before any padding, and the peek lays out in it.
                let padding = aegis_ui::theme::Metrics::of(&tallow, g.view).padding / 2;
                let content = g.view.nine_content(&frame.nine, peek);
                let inner = content.inset(padding);
                let face = rect_of(w.iter().find(|x| x["id"].as_str().is_some_and(|i| i.starts_with("menu.peek."))).expect("a portrait"));
                assert_eq!((face.x, face.y + face.h), (inner.x, inner.y + inner.h), "{ww} {size}: the peek's portraits sit in the content rect");
                assert!(audit.iter().filter(|t| inside(peek, t)).any(|t| t.x as i32 == inner.x), "{ww} {size}: the peek's text starts at the content rect");
                rects.push(content);
                check(&mut g, "peek", peek, rects);
                // The ask dialog: its own screen, drawn in old units over the menu.
                g.view = View::fit(ww as u32, wh as u32);
                g.fonts.audit = Some(Vec::new());
                let v = aegis_ui::panes::AskView { ornate: true, ..aegis_ui::panes::AskView::new("Leave this place?", "Anything not saved since the last save is lost. Enter answers the default, y and n answer, Esc is No.") };
                let area = Rect::new(0, 0, W, H);
                let mut ui: Ui<MenuAct> = Ui::new(&mut g, (-1, -1), true);
                let ask = panes::ask_dialog(&mut ui, "ask", area, &v, MenuAct::Quit, MenuAct::Quit);
                let (_, _, yes, no) = panes::ask_layout(&mut ui.p, area, &v);
                drop(ui);
                let mut shown = vec![yes, no];
                let audit = g.fonts.audit.take().unwrap();
                shown.extend(audit.iter().filter(|t| inside(ask, t)).map(text_rect));
                assert!(shown.len() >= 4, "{ww} {size}: the ask's title, question and buttons: {}", shown.len());
                let content = aegis_ui::painter::nine_content_rect(&frame.nine, ask, g.view.s);
                let inner = content.inset(tallow.layout.padding / 2);
                assert_eq!((no.x + no.w, no.y + no.h), (inner.x + inner.w, inner.y + inner.h), "{ww} {size}: the ask's buttons sit in the content rect");
                shown.push(content);
                check(&mut g, "ask", ask, shown);
            }
        }
        g.fonts.scale = 1.0;
        aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The picture's rect never meets a menu row or the peek, at any text size, with or without a save.
    #[test]
    fn the_title_picture_never_meets_a_menu_row() {
        let (dir, root) = (scratch("logo_rows"), scratch("logo_rows_layer"));
        std::fs::create_dir_all(root.join("ui")).unwrap();
        write_picture(&root.join(LOGO_ART), 1246, 325, false);
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]));
        for with_save in [false, true] {
            if with_save {
                write_fixture(&mut cx, 3);
            }
            for (size, scale) in TEXT_SIZES {
                g.fonts.scale = scale as f32;
                let mut m = MainMenu::default();
                let idle = menu_frame(&mut m, &mut cx, &mut g, (-1, -1));
                let w = if with_save { let at = centre(get(&idle, "menu.continue").unwrap()); menu_frame(&mut m, &mut cx, &mut g, at) } else { idle };
                assert_eq!(get(&w, "menu.peek").is_some(), with_save, "{size}: the peek shows with a save");
                let logo = get(&w, "menu.logo").expect("the title picture");
                let rect = |v: &Value| Rect::new(n(v, 0), n(v, 1), n(v, 2), n(v, 3));
                let others: Vec<&Value> = w.iter().filter(|x| x["kind"] == "text_button" || x["id"].as_str().is_some_and(|i| i.starts_with("menu.peek"))).collect();
                assert!(others.len() >= 5, "{size}: the rows");
                for o in others {
                    let meet = rect(logo).intersect(rect(o));
                    assert!(meet.w <= 0 || meet.h <= 0, "{size}: the logo meets {}", o["id"]);
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// `ui/logo_title.png` replaces the typeset title and subtitle, and the crest even when both files are there; it sits
    /// in the logo's room above the column, without it today's text.
    #[test]
    fn the_title_picture_replaces_the_typeset_logo() {
        let (dir, root) = (scratch("logo"), scratch("logo_layer"));
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/frames/button.png");
        std::fs::create_dir_all(root.join("ui")).unwrap();
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        cx.app.manifest.title = "Fixture title".into();
        cx.app.manifest.subtitle = "Fixture subtitle".into();
        let mut audited = |cx: &mut Cx| {
            cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]));
            g.fonts.audit = Some(Vec::new());
            let w = menu_frame(&mut MainMenu::default(), cx, &mut g, (-1, -1));
            let texts: Vec<String> = g.fonts.audit.take().unwrap().into_iter().map(|r| r.text).collect();
            (w, texts)
        };
        let (w, texts) = audited(&mut cx);
        assert!(get(&w, "menu.logo").is_none() && texts.iter().any(|t| t == "Fixture title") && texts.iter().any(|t| t == "Fixture subtitle"), "no picture: the text");
        std::fs::copy(&source, root.join(CREST_ART)).unwrap();
        std::fs::copy(&source, root.join(LOGO_ART)).unwrap();
        let (w, texts) = audited(&mut cx);
        let logo = get(&w, "menu.logo").expect("the title picture");
        assert_eq!((logo["kind"].as_str(), logo["text"].as_str()), (Some("picture"), Some(LOGO_ART)));
        assert!(!texts.iter().any(|t| t.starts_with("Fixture")), "nothing typeset: {texts:?}");
        assert!(get(&w, "menu.crest").is_none(), "the title picture wins over the crest");
        let (x, y, ww, h) = (n(logo, 0), n(logo, 1), n(logo, 2), n(logo, 3));
        let room = logo_room(CANVAS);
        assert_eq!((ww, h), (room.h, room.h), "the square fixture stays square, to its room's height");
        assert_eq!((x + ww, y), (room.x + room.w, room.y), "right-aligned at the room's top");
        assert!(x + ww <= CANVAS.w && y >= 0 && y + h <= base_column().y, "on the canvas, above the column");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A review render of the full frame at 1280 x 720: only with `AEGIS_REVIEW_DIR` set.
    #[test]
    fn review_render_of_the_full_frame() {
        let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tallow = aegis_ui::theme::load(&aegis_core::assets::Layers::for_game(&game)).expect("the game's theme loads");
        let frame = tallow.frame("ornate_full").expect("the full frame");
        let mut g = gfx();
        g.canvas = Canvas::new(1280, 720);
        g.canvas.clear([40, 44, 52, 255]);
        g.view = View::fit(1280, 720);
        let view = g.view;
        let mut p = aegis_ui::painter::Painter { c: &mut g.canvas, v: view, fonts: &mut g.fonts };
        for r in [Rect::new(35, 20, 160, 64), Rect::new(240, 20, 380, 95), Rect::new(35, 130, 100, 200), Rect::new(240, 150, 200, 160)] {
            p.nine(&frame.img, &frame.nine, r);
        }
        let file = std::fs::File::create(Path::new(&out).join("a8_full_frame_1280x720.png")).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 1280, 720);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
    }

    /// A review render of the ornate frame on a very wide, a small, a tall, a large and a too small panel, at 1280 x 720 and
    /// 1920 x 1080: only with `AEGIS_REVIEW_DIR` set.
    #[test]
    fn review_render_of_the_ornate_frame_on_other_panels() {
        let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tallow = aegis_ui::theme::load(&aegis_core::assets::Layers::for_game(&game)).expect("the game's theme loads");
        let frame = tallow.frame("ornate").expect("the ornate frame");
        let mut g = gfx();
        for (ww, wh) in [(1280, 720), (1920, 1080)] {
            g.canvas = Canvas::new(ww, wh);
            g.canvas.clear([40, 44, 52, 255]);
            g.view = View::fit(ww as u32, wh as u32);
            let view = g.view;
            let mut p = aegis_ui::painter::Painter { c: &mut g.canvas, v: view, fonts: &mut g.fonts };
            for r in [Rect::new(30, 40, 580, 60), Rect::new(60, 160, 100, 40), Rect::new(220, 150, 60, 180), Rect::new(340, 150, 260, 170), Rect::new(60, 240, 20, 16)] {
                p.nine(&frame.img, &frame.nine, r);
            }
            let file = std::fs::File::create(Path::new(&out).join(format!("ornate_other_panels_{ww}x{wh}.png"))).unwrap();
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), ww as u32, wh as u32);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
        }
    }

    /// Review renders of the menu at 1280 x 720 with the game's art and theme: only with `AEGIS_REVIEW_DIR` set.
    #[test]
    fn review_shots() {
        let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
        std::fs::create_dir_all(&out).unwrap();
        let dir = scratch("review");
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut cx = cx(dir.to_path_buf());
        let m = aegis_content::manifest::Manifest::read(&root.join(aegis_content::manifest::FILE)).unwrap();
        cx.app.manifest = m;
        cx.art = aegis_ui::art::Art::new(Some(root.to_path_buf()));
        aegis_ui::theme::set_theme(aegis_ui::theme::load(cx.art.layers()).expect("the game's theme"));
        let mut g = menu_gfx();
        let mut shot = |cx: &mut Cx, name: &str, save: bool, focus: &str, mouse: (i32, i32), scale: f32| {
            let _ = std::fs::remove_dir_all(&dir);
            if save {
                write_fixture(cx, 11);
            }
            g.fonts.scale = scale;
            g.kit.focus = Some(focus.to_string());
            g.kit.focus_visible = true;
            g.canvas.clear([0, 0, 0, 255]);
            g.fonts.audit = Some(Vec::new());
            let mut m = MainMenu::default();
            let w = menu_frame(&mut m, cx, &mut g, mouse);
            let recs = g.fonts.audit.take().unwrap();
            let boxes: Vec<(String, f32, f32, f32, f32)> = w.iter().filter(|x| x["kind"] != "picture").map(|x| (x["id"].as_str().unwrap().to_string(), n(x, 0) as f32, n(x, 1) as f32, n(x, 2) as f32, n(x, 3) as f32)).collect();
            println!("{name}: {} peek {}", aegis_ui::text_audit::issues_on(&recs, CANVAS.w as f32, CANVAS.h as f32, &boxes, 2)["counts"], get(&w, "menu.peek").is_some());
            let file = std::fs::File::create(format!("{out}/{name}.png")).unwrap();
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 1280, 720);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
        };
        let away = (40, 680);
        shot(&mut cx, "headless_idle", true, "menu.continue", away, 1.0);
        shot(&mut cx, "headless_hover_item", true, "menu.settings", (900, 442), 1.0);
        shot(&mut cx, "headless_peek_hover", true, "menu.continue", OVER_FIRST, 1.0);
        aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The fixture save's file as JSON, `f` applied, written back.
    fn doctor_save(dir: &Path, f: impl FnOnce(&mut Value)) {
        let path = dir.join("slot_2.json");
        let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        f(&mut v);
        std::fs::write(&path, v.to_string()).unwrap();
    }

    /// A save of a refused older version is no save to continue: no Continue, no peek.
    #[test]
    fn a_refused_older_save_gives_no_continue_and_no_peek() {
        let dir = scratch("older");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        write_fixture(&mut cx, 3);
        doctor_save(&dir, |v| v["version"] = json!(save::OLDEST - 1));
        let w = menu_frame(&mut MainMenu::default(), &mut cx, &mut g, OVER_FIRST);
        assert_eq!(buttons(&w), MENU_IDS);
        assert!(get(&w, "menu.peek").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A save that passes the slot header's check but fails to load: Continue stays (pressing it says why), no panel, and
    /// it is read once while the peek is wanted, not every frame (the mended file changes nothing until the hover ends).
    #[test]
    fn a_save_that_fails_validate_keeps_continue_without_a_peek_or_a_reread() {
        let dir = scratch("invalid");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        write_fixture(&mut cx, 3);
        let good = std::fs::read_to_string(dir.join("slot_2.json")).unwrap();
        doctor_save(&dir, |v| v["game_state"]["party"]["members"][0]["base_unit"] = json!("no_such_unit"));
        assert!(save::read(&cx.data, &dir, 2).is_err(), "the fixture fails validate");
        let mut m = MainMenu::default();
        let over = OVER_FIRST;
        let w = menu_frame(&mut m, &mut cx, &mut g, over);
        assert_eq!(buttons(&w)[0], "menu.continue", "the header reads");
        assert!(get(&w, "menu.peek").is_none(), "no panel");
        std::fs::write(dir.join("slot_2.json"), good).unwrap();
        assert!(get(&menu_frame(&mut m, &mut cx, &mut g, over), "menu.peek").is_none(), "the failed read is kept while it hovers");
        assert!(get(&menu_frame(&mut m, &mut cx, &mut g, (-1, -1)), "menu.peek").is_none());
        assert!(get(&menu_frame(&mut m, &mut cx, &mut g, over), "menu.peek").is_some(), "read again on the next hover");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `newest_slot` parses every slot, so the menu reads it once and again on `resume`.
    #[test]
    fn the_newest_slot_is_read_once_until_the_menu_resumes() {
        let dir = scratch("newest");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        let mut m = MainMenu::default();
        assert_eq!(buttons(&menu_frame(&mut m, &mut cx, &mut g, (-1, -1))), MENU_IDS);
        write_fixture(&mut cx, 3);
        assert_eq!(buttons(&menu_frame(&mut m, &mut cx, &mut g, (-1, -1))), MENU_IDS, "kept");
        m.resume(&mut cx);
        assert_eq!(buttons(&menu_frame(&mut m, &mut cx, &mut g, (-1, -1)))[0], "menu.continue", "read again");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The logo right-aligned on its point with the subtitle under it, and the menu clean in the text audit (nothing cut, past
    /// the canvas, overlapping or out of its control) at the four text sizes, with and without the peek, under the engine
    /// default and the game's own theme; the column and the peek grow leftward and stay on the canvas.
    #[test]
    fn the_main_menu_is_clean_at_the_four_text_sizes() {
        let dir = scratch("audit");
        let (mut cx, mut g) = (cx(dir.to_path_buf()), menu_gfx());
        cx.app.manifest.title = "Emberwake".into();
        cx.app.manifest.subtitle = "A Scarlet Lullaby".into();
        write_fixture(&mut cx, 12);
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tallow = aegis_ui::theme::load(&aegis_core::assets::Layers::for_game(&game)).expect("the game's theme loads");
        // The active item is shown by brighter text alone: it reads on the backdrop's colour and apart from the idle rows.
        let mut idle = tallow.bg;
        let a = aegis_ui::panes::MENU_TEXT_IDLE as u32;
        for (c, (t, b)) in idle.iter_mut().zip(tallow.text.iter().zip(&tallow.bg)).take(3) {
            *c = ((*t as u32 * a + *b as u32 * (255 - a)) / 255) as u8;
        }
        assert!(tallow.contrast(tallow.gold, tallow.bg) >= 4.5 && tallow.contrast(idle, tallow.bg) >= 4.5, "both read on the bg");
        assert!(tallow.contrast(tallow.gold, idle) >= 1.3, "the active item stands apart: {}", tallow.contrast(tallow.gold, idle));
        for (name, t) in [("default", aegis_ui::theme::Theme::default()), ("game", tallow)] {
            aegis_ui::theme::set_theme(t);
            for (size, scale) in TEXT_SIZES {
                for mouse in [(-1, -1), OVER_FIRST] {
                    let what = format!("{name} at {size}, mouse {mouse:?}");
                    let mut m = MainMenu::default();
                    g.fonts.scale = scale as f32;
                    g.fonts.audit = Some(Vec::new());
                    g.canvas.clear([0, 0, 0, 255]);
                    let w = menu_frame(&mut m, &mut cx, &mut g, mouse);
                    let recs = g.fonts.audit.take().unwrap();
                    let boxes: Vec<(String, f32, f32, f32, f32)> = w.iter().filter(|x| x["kind"] != "picture").map(|x| (x["id"].as_str().unwrap().to_string(), n(x, 0) as f32, n(x, 1) as f32, n(x, 2) as f32, n(x, 3) as f32)).collect();
                    let v = aegis_ui::text_audit::issues_on(&recs, CANVAS.w as f32, CANVAS.h as f32, &boxes, g.view.g);
                    for k in ["cut", "edge", "overlap", "spill"] {
                        assert_eq!(v["counts"][k], json!(0), "{what}: {k}: {v}");
                    }
                    let logo = recs.iter().rfind(|r| r.text == "Emberwake").expect("the logo");
                    let sub = recs.iter().find(|r| r.text == "A Scarlet Lullaby").expect("the subtitle");
                    let (at, metrics, column) = (logo_point(CANVAS), aegis_ui::theme::Metrics::of(&th(), g.view), base_column());
                    let (right, top) = (at.x as f32, at.y as f32);
                    assert!((logo.x + logo.w - right).abs() <= 2.0 && logo.y == top && (sub.x + sub.w - right).abs() <= 2.0 && sub.y >= logo.y + logo.h - 1.0, "{what}: {} {} {}", logo.x, logo.w, sub.y);
                    let tools = get(&w, "menu.tools").unwrap();
                    assert!(n(tools, 0) + n(tools, 2) <= column.x + column.w, "{what}: the column keeps its right edge");
                    if scale <= 1.0 {
                        assert_eq!(n(tools, 0), column.x + metrics.padding - metrics.u(panes::MENU_TEXT_MARGIN), "{what}");
                    }
                    if let Some(p) = get(&w, "menu.peek") {
                        assert!(n(p, 0) >= 0 && n(p, 0) + n(p, 2) <= n(tools, 0) && n(p, 1) + n(p, 3) <= CANVAS.h, "{what}");
                        assert_eq!(n(p, 2) % metrics.grid + n(p, 3) % metrics.grid, 0, "{what}: its size on the grid");
                    }
                }
            }
        }
        aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ASKNO2: the question the last action asked is the No-default form (it asks for the pointer on No); Enter
    /// answers No: the question closes and nothing else happens.
    fn enter_is_no(cx: &mut Cx, g: &mut Gfx, what: &str) {
        let Some(Nav::Push(mut ask)) = cx.nav.pop() else { panic!("{what}: a question was asked") };
        assert!(cx.nav.is_empty(), "{what}");
        ask.draw(cx, g, (-1, -1), true);
        assert_eq!((ask.id(), ask.state(cx)["default"].as_str(), ask.pointer_to()), ("ask", Some("no"), Some("ask.no")), "{what}");
        ask.key(cx, Key::Enter);
        assert!(matches!(cx.nav.as_slice(), [Nav::Pop]), "{what}: Enter only closes the question");
        cx.nav.clear();
    }

    /// ASKNO2: every question that loses progress or a save defaults to No (quit, quit to the main menu, load over
    /// unsaved progress, overwrite, delete, a new game over the autosave).
    #[test]
    fn the_risky_questions_answer_no_on_enter() {
        let dir = aegis_core::scratch::Scratch::vacant("emberwake_askno");
        let mut cx = cx(dir.to_path_buf());
        let mut g = gfx();
        cx.app.game.as_mut().unwrap().clean = Value::Null;
        assert!(cx.is_dirty(), "unsaved progress");

        MainMenu::default().act(&mut cx, MenuAct::Quit);
        enter_is_no(&mut cx, &mut g, "Quit without saving?");
        assert!(!cx.quit);

        let mut hub = crate::hub::Hub::default();
        hub.draw(&mut cx, &mut g, (-1, -1), true);
        hub.click_id(&mut cx, "hub.quit").unwrap();
        enter_is_no(&mut cx, &mut g, "Quit to the main menu without saving?");
        assert!(cx.has_game() && cx.is_dirty());

        assert!(save::write(&cx.data, &dir, cx.gs(), 2, 0).is_empty());
        let mut slots = Slots::new(true);
        slots.act(&mut cx, SlotAct::Save(2));
        enter_is_no(&mut cx, &mut g, "Overwrite slot 2?");
        assert!(cx.is_dirty(), "not saved");
        slots.act(&mut cx, SlotAct::Load(2));
        enter_is_no(&mut cx, &mut g, "Load slot 2?");
        assert!(cx.is_dirty(), "not loaded");
        slots.act(&mut cx, SlotAct::Delete(2));
        enter_is_no(&mut cx, &mut g, "Delete the save in slot 2?");
        assert!(save::exists(&dir, 2), "not deleted");

        assert!(save::write(&cx.data, &dir, cx.gs(), save::AUTOSAVE_SLOT, 0).is_empty());
        NewGameScreen::new(&cx).act(&mut cx, NgAct::Start);
        enter_is_no(&mut cx, &mut g, "Starting a new game replaces your autosave");
        assert!(cx.is_dirty(), "no new game");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ACT1-HOOKS1 (FB-241 c): New game picks the game manifest's first playable `start_worlds` entry (read from
    /// `game.json` beside the data folder), not the first playable world; with no list it picks the first playable world
    /// as before. Read from the shipped manifest, never named here.
    #[test]
    fn new_game_preselects_the_manifests_first_start_world() {
        let dir = scratch("new_game_start");
        let mut cx = cx(dir.to_path_buf());
        let manifest = aegis_content::manifest::Manifest::read(&cx.data_dir.parent().expect("the game folder").join(aegis_content::manifest::FILE)).expect("game.json");
        let playable = new_game::playable_worlds(&cx.data);
        let want = manifest.start_worlds.iter().find(|w| playable.contains(w)).cloned().expect("the manifest names a playable start world");
        assert_eq!(cx.data.start_worlds, manifest.start_worlds, "the data carries the manifest's list");
        assert_eq!(NewGameScreen::new(&cx).state(&cx)["selected"], json!(want), "the first playable start world");
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").start_worlds.clear();
        assert_eq!(NewGameScreen::new(&cx).state(&cx)["selected"], json!(playable[0]), "no list: the first playable world");
    }
}

/// SETTINGS-MOCK1 (PR7-21): the settings screen's audit: the tab rail, the staged display change, the keep-or-revert
/// timer, the close guard, the web's rows and the fit of every tab at the four text sizes, full screen and popup. With
/// `AEGIS_REVIEW_DIR` set it writes the review shots there.
#[cfg(test)]
mod settings_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use aegis_ui::canvas::Canvas;
    use aegis_ui::painter::View;

    fn saved(cx: &Cx, key: &str) -> Value {
        let text = std::fs::read_to_string(cx.shot_dir.join("settings.json")).expect("the settings file");
        serde_json::from_str::<Value>(&text).expect("json")[key].clone()
    }

    fn redraw(s: &mut SettingsScreen, cx: &mut Cx, g: &mut Gfx) -> Vec<Value> {
        s.draw(cx, g, (-500, -500), true);
        s.widgets()
    }

    fn has(w: &[Value], id: &str) -> bool {
        w.iter().any(|x| x["id"] == id)
    }

    /// Picks Display Mode `mode` by its arrows (staged only).
    fn stage_mode(s: &mut SettingsScreen, cx: &mut Cx, g: &mut Gfx, mode: WindowShow) {
        for _ in 0..DISPLAY_MODES.len() {
            if WindowShow::of(&s.state(cx)["pending"]) == mode {
                return;
            }
            redraw(s, cx, g);
            s.click_id(cx, "settings.display_mode.next").expect("the arrow");
        }
        assert_eq!(WindowShow::of(&s.state(cx)["pending"]), mode);
    }

    /// The rail lists the five tabs in the mock's order (Display first), each with its subtitle on the full screen and
    /// none in the popup; a click opens a tab, and every tab sits left of the body.
    #[test]
    fn the_rail_lists_the_five_tabs_with_subtitles_on_the_full_screen() {
        let (mut cx, mut g) = (game_cx(), gfx());
        for popup in [false, true] {
            let mut s = if popup { SettingsScreen::in_game(&cx) } else { SettingsScreen::new(&cx) };
            let recs = frame(&mut s, &mut cx, &mut g, 1.0);
            let w = s.widgets();
            let tabs: Vec<&Value> = w.iter().filter(|x| x["kind"] == "tab").collect();
            assert_eq!(tabs.iter().map(|t| t["text"].as_str().unwrap()).collect::<Vec<_>>(), TABS.iter().map(|t| t.0).collect::<Vec<_>>());
            let body = w.iter().find(|x| x["id"] == "settings.body").expect("the body");
            for t in &tabs {
                assert!(rect(t).0 + rect(t).2 < rect(body).0, "{t} left of the body");
            }
            for (name, sub) in TABS {
                assert_eq!(recs.iter().any(|r| r.text == sub), !popup, "{name}'s subtitle (popup {popup})");
            }
            for (i, (name, _)) in TABS.iter().enumerate() {
                s.click_id(&mut cx, &format!("settings.tab.{}", name.to_lowercase())).expect("the tab");
                assert_eq!(s.state(&cx)["tab"], json!(name));
                let w = redraw(&mut s, &mut cx, &mut g);
                let open: Vec<&Value> = w.iter().filter(|x| x["kind"] == "tab" && x["active"] == json!(true)).collect();
                assert_eq!(open.len(), 1);
                assert_eq!(open[0]["text"], json!(TABS[i].0));
            }
            // the popup is the decluttered one: smaller, with Return to title; the full screen has none
            assert_eq!(has(&s.widgets(), "settings.title"), popup);
        }
    }

    /// Display Mode and Resolution are staged until Apply (the card's Reset drops them); every other row writes and
    /// saves at once.
    #[test]
    fn display_changes_wait_for_apply_and_the_rest_apply_at_once() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut s = SettingsScreen::new(&cx);
        redraw(&mut s, &mut cx, &mut g);
        let before = (cx.settings.v["fullscreen"].clone(), cx.settings.v["borderless"].clone(), cx.settings.v["window_size"].clone());
        stage_mode(&mut s, &mut cx, &mut g, WindowShow::Borderless);
        redraw(&mut s, &mut cx, &mut g);
        s.click_id(&mut cx, "settings.window_size.next").expect("the arrow");
        assert!(s.display_dirty(&cx));
        assert_eq!((cx.settings.v["fullscreen"].clone(), cx.settings.v["borderless"].clone(), cx.settings.v["window_size"].clone()), before, "staged only");
        let w = redraw(&mut s, &mut cx, &mut g);
        assert!(has(&w, "settings.reset"), "Reset shows while a change is staged");
        assert_eq!(w.iter().find(|x| x["id"] == "settings.apply").unwrap()["active"], json!(true), "Apply is lit");
        s.click_id(&mut cx, "settings.reset").expect("reset");
        assert!(!s.display_dirty(&cx), "Reset drops the staged change");
        assert!(!has(&redraw(&mut s, &mut cx, &mut g), "settings.reset"));
        // Apply with nothing staged does nothing
        s.click_id(&mut cx, "settings.apply").expect("apply");
        assert_eq!(s.state(&cx)["confirm_left"], Value::Null);
        // the rest: at once, saved
        let vsync = cx.settings.v["vsync"].as_bool().unwrap();
        s.click_id(&mut cx, "settings.vsync").expect("vsync");
        assert_eq!(cx.settings.v["vsync"], json!(!vsync));
        assert_eq!(saved(&cx, "vsync"), json!(!vsync), "saved at once");
        s.click_id(&mut cx, "settings.text_scale.next").expect("text size");
        assert_eq!(saved(&cx, "text_scale"), json!(TEXT_SIZES[2].1), "Normal steps to Large");
        s.click_id(&mut cx, "settings.tab.sound").expect("sound");
        redraw(&mut s, &mut cx, &mut g);
        let master = cx.settings.v["master"].as_f64().unwrap();
        s.click_id(&mut cx, "settings.master.down").expect("the arrow");
        assert_eq!(saved(&cx, "master").as_f64(), Some(master - VOLUME_STEP));
        s.click_id(&mut cx, "settings.mute").expect("mute all");
        assert_eq!(saved(&cx, "mute"), json!(true));
        assert!(!s.display_dirty(&cx), "nothing on the Sound tab stages");
    }

    /// Apply writes the staged display and opens the 15 s confirm: before its end the change holds, at its end (or on
    /// Revert, Esc, the box's X) the previous display comes back; Keep (or Enter) commits it and no timer reverts it.
    #[test]
    fn the_confirm_reverts_when_its_time_runs_out_and_keep_commits() {
        let (mut cx, mut g) = (game_cx(), gfx());
        for how in ["timer", "revert", "esc", "x", "keep", "enter"] {
            let mut s = SettingsScreen::new(&cx);
            redraw(&mut s, &mut cx, &mut g);
            let prev = staged_of(&cx.settings.v);
            let want = if WindowShow::of(&prev) == WindowShow::Fullscreen { WindowShow::Windowed } else { WindowShow::Fullscreen };
            stage_mode(&mut s, &mut cx, &mut g, want);
            redraw(&mut s, &mut cx, &mut g);
            s.click_id(&mut cx, "settings.apply").expect("apply");
            let t0 = Instant::now();
            assert_eq!(WindowShow::of(&cx.settings.v), want, "{how}: applied");
            assert_eq!(WindowShow::of(&json!({"fullscreen": saved(&cx, "fullscreen"), "borderless": saved(&cx, "borderless")})), want, "{how}: saved");
            assert_eq!(s.confirm_left(t0), Some(CONFIRM_SECS), "{how}: the countdown starts at 15 s");
            assert_eq!(s.frame_need(&cx), FrameNeed::Within(Duration::from_millis(250)), "the countdown redraws");
            let w = redraw(&mut s, &mut cx, &mut g);
            assert!(has(&w, "settings.confirm.keep") && has(&w, "settings.confirm.revert"), "{how}: the confirm");
            assert!(!has(&w, "settings.tab.display") && !has(&w, "settings.apply"), "{how}: the box takes the input");
            s.tick_at(&mut cx, t0 + Duration::from_secs(CONFIRM_SECS - 1));
            assert_eq!(WindowShow::of(&cx.settings.v), want, "{how}: still on before the end");
            assert_eq!(s.confirm_left(t0 + Duration::from_secs(CONFIRM_SECS - 1)), Some(1));
            match how {
                "timer" => s.tick_at(&mut cx, t0 + Duration::from_secs(CONFIRM_SECS + 1)),
                "revert" => s.click_id(&mut cx, "settings.confirm.revert").expect("revert"),
                "esc" => s.back(&mut cx),
                "x" => s.click_id(&mut cx, "settings.modal.close").expect("the X"),
                "keep" => s.click_id(&mut cx, "settings.confirm.keep").expect("keep"),
                _ => s.key(&mut cx, Key::Enter),
            }
            let kept = matches!(how, "keep" | "enter");
            assert_eq!(s.confirm_left(t0), None, "{how}: the box closed");
            assert!(cx.nav.is_empty(), "{how}: the screen stays open");
            s.tick_at(&mut cx, t0 + Duration::from_secs(CONFIRM_SECS * 4));
            let now = WindowShow::of(&cx.settings.v);
            let file = WindowShow::of(&json!({"fullscreen": saved(&cx, "fullscreen"), "borderless": saved(&cx, "borderless")}));
            if kept {
                assert_eq!((now, file), (want, want), "{how}: kept, and no timer reverts it");
                // back to the start for the next case
                stage_mode(&mut s, &mut cx, &mut g, WindowShow::of(&prev));
                s.act(&mut cx, SetAct::Apply);
                s.act(&mut cx, SetAct::Keep);
            } else {
                assert_eq!((now, file), (WindowShow::of(&prev), WindowShow::of(&prev)), "{how}: the previous display is back");
                assert_eq!(cx.settings.v["window_size"], prev["window_size"]);
                assert!(!s.display_dirty(&cx), "{how}: nothing staged after a revert");
            }
            assert_eq!(s.frame_need(&cx), FrameNeed::Idle);
        }
    }

    /// Closing with a staged display change (X, Esc, right click, Return to title) asks first: Stay keeps the screen
    /// and the change, Discard & close drops it and goes on, Apply display applies it (the confirm) and stays. With
    /// nothing staged the screen closes at once.
    #[test]
    fn the_close_guard_asks_before_dropping_a_staged_display_change() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut s = SettingsScreen::in_game(&cx);
        redraw(&mut s, &mut cx, &mut g);
        s.click_id(&mut cx, "settings.close").expect("the X");
        assert!(matches!(cx.nav.pop(), Some(Nav::Pop)), "nothing staged: it closes");
        let applied = WindowShow::of(&cx.settings.v);
        let other = if applied == WindowShow::Borderless { WindowShow::Windowed } else { WindowShow::Borderless };
        for how in ["x", "esc", "title"] {
            stage_mode(&mut s, &mut cx, &mut g, other);
            redraw(&mut s, &mut cx, &mut g);
            match how {
                "x" => s.click_id(&mut cx, "settings.close").expect("the X"),
                "esc" => s.back(&mut cx),
                _ => s.click_id(&mut cx, "settings.title").expect("Return to title"),
            }
            assert!(cx.nav.is_empty(), "{how}: the guard holds the screen");
            assert_eq!(s.state(&cx)["guard"], json!(if how == "title" { "title" } else { "close" }));
            let w = redraw(&mut s, &mut cx, &mut g);
            for id in ["settings.guard.stay", "settings.guard.discard", "settings.guard.apply"] {
                assert!(has(&w, id), "{how}: {id}");
            }
            assert!(!has(&w, "settings.close"), "the box takes the input");
            // Stay (and Esc on the box) keep the screen and the staged change
            s.back(&mut cx);
            assert_eq!(s.state(&cx)["guard"], Value::Null);
            assert!(cx.nav.is_empty() && s.display_dirty(&cx), "{how}: Esc on the guard stays");
            s.act(&mut cx, if how == "title" { SetAct::Title } else { SetAct::Close });
            redraw(&mut s, &mut cx, &mut g);
            s.click_id(&mut cx, "settings.guard.stay").expect("stay");
            assert!(cx.nav.is_empty() && s.display_dirty(&cx));
        }
        // Discard & close: the staged change goes and the screen closes; the display never changed
        s.act(&mut cx, SetAct::Close);
        redraw(&mut s, &mut cx, &mut g);
        s.click_id(&mut cx, "settings.guard.discard").expect("discard");
        assert!(matches!(cx.nav.pop(), Some(Nav::Pop)));
        assert!(!s.display_dirty(&cx) && WindowShow::of(&cx.settings.v) == applied);
        // Return to title with Discard: back to the title (a clean game asks nothing)
        stage_mode(&mut s, &mut cx, &mut g, other);
        s.act(&mut cx, SetAct::Title);
        redraw(&mut s, &mut cx, &mut g);
        s.click_id(&mut cx, "settings.guard.discard").expect("discard");
        assert!(cx.nav.iter().any(|n| matches!(n, Nav::Reset(_))) || cx.nav.iter().any(|n| matches!(n, Nav::Push(_))), "to the title, or its unsaved-progress question");
        cx.nav.clear();
        // Apply display: the change applies, the confirm opens, the screen stays
        let mut s = SettingsScreen::new(&cx);
        stage_mode(&mut s, &mut cx, &mut g, other);
        s.act(&mut cx, SetAct::Close);
        redraw(&mut s, &mut cx, &mut g);
        s.key(&mut cx, Key::Enter);
        assert!(cx.nav.is_empty());
        assert_eq!(WindowShow::of(&cx.settings.v), other);
        assert_eq!(s.confirm_left(Instant::now()).map(|l| l <= CONFIRM_SECS), Some(true), "the confirm is open");
    }

    /// WEBFIX1, DISPLAY1 rule 8: the web's Display tab keeps Fullscreen (at once: the click is the browser's gesture) and
    /// Text size of the display rows; no Display Mode, Resolution, VSync or Frame rate cap.
    #[test]
    fn the_web_keeps_fullscreen_and_text_size_only() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut s = SettingsScreen::new(&cx).as_web();
        let w = redraw(&mut s, &mut cx, &mut g);
        assert!(has(&w, "settings.fullscreen") && has(&w, "settings.text_scale"));
        for id in ["settings.display_mode", "settings.window_size", "settings.vsync", "settings.fps_cap", "settings.apply"] {
            assert!(!has(&w, id), "{id} is not on the web");
        }
        let was = WindowShow::of(&cx.settings.v) == WindowShow::Fullscreen;
        s.click_id(&mut cx, "settings.fullscreen").expect("fullscreen");
        assert_eq!(WindowShow::of(&cx.settings.v) == WindowShow::Fullscreen, !was, "at once");
        assert!(!s.display_dirty(&cx) && s.confirm_left(Instant::now()).is_none(), "no staging, no confirm");
    }

    /// PR7-15 (b) (FB-287 Option A): Map Camera Zoom is on the Display tab, desktop and web, full screen and popup;
    /// PR7-37 (PT-33): on that tab only (no longer on Gameplay); a pick applies at once (no display staging).
    #[test]
    fn map_zoom_is_on_the_display_tab_only() {
        let (mut cx, mut g) = (game_cx(), gfx());
        for web in [false, true] {
            for popup in [false, true] {
                let base = if popup { SettingsScreen::in_game(&cx) } else { SettingsScreen::new(&cx) };
                let mut s = if web { base.as_web() } else { base };
                let w = redraw(&mut s, &mut cx, &mut g);
                assert!(has(&w, "settings.map_zoom"), "Display has Map zoom (web {web}, popup {popup})");
                s.click_id(&mut cx, "settings.tab.gameplay").expect("the Gameplay tab");
                let w = redraw(&mut s, &mut cx, &mut g);
                assert!(!has(&w, "settings.map_zoom"), "Gameplay has no Map zoom (web {web}, popup {popup})");
            }
        }
        let mut s = SettingsScreen::new(&cx);
        let w = redraw(&mut s, &mut cx, &mut g);
        let row = w.iter().find(|x| x["id"] == "settings.map_zoom").expect("the row").clone();
        let next = crate::mapview::zoom_steps().map(|z| format!("{z}x")).find(|z| Some(z.as_str()) != row["text"].as_str()).expect("another step");
        g.kit.open = Some("settings.map_zoom".to_string());
        redraw(&mut s, &mut cx, &mut g);
        let at = crate::mapview::zoom_steps().position(|z| format!("{z}x") == next).unwrap();
        s.click_id(&mut cx, &format!("settings.map_zoom.opt.{at}")).expect("the option");
        g.kit.open = None;
        assert!(!s.display_dirty(&cx) && s.confirm_left(Instant::now()).is_none(), "no staging, no confirm");
        let picked = cx.settings.v["map_zoom"].clone();
        assert_eq!(format!("{}x", picked.as_f64().unwrap()), next, "applied at once");
        let w = redraw(&mut s, &mut cx, &mut g);
        assert_eq!(w.iter().find(|x| x["id"] == "settings.map_zoom").expect("the row")["text"], json!(next), "the row shows the pick");
    }

    /// Restore defaults resets every row at once but stages the default display for Apply.
    #[test]
    fn restore_defaults_stages_the_default_display() {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["fullscreen"] = json!(true);
        cx.settings.v["show_tells"] = json!(false);
        let mut s = SettingsScreen::new(&cx);
        redraw(&mut s, &mut cx, &mut g);
        s.click_id(&mut cx, "settings.defaults").expect("defaults");
        let d = Settings::defaults(default_settings());
        assert_eq!(cx.settings.v["show_tells"], d["show_tells"], "at once");
        assert_eq!(WindowShow::of(&cx.settings.v), WindowShow::Fullscreen, "the display waits");
        assert_eq!(WindowShow::of(&s.state(&cx)["pending"]), WindowShow::of(&d), "staged");
        assert!(s.display_dirty(&cx));
    }

    /// PR7-26 (5): the Sound tab's Mute in Background is on by default and toggles the engine's key; with it on a
    /// lost focus silences the audio and a regained one restores it; off, the focus changes nothing; the saved
    /// volumes never change.
    #[test]
    fn mute_in_background_is_on_by_default_and_follows_the_focus() {
        let (mut cx, mut g) = (game_cx(), gfx());
        assert_eq!(Settings::defaults(default_settings())[MUTE_IN_BACKGROUND], json!(true), "on by default");
        let mut s = SettingsScreen::in_game(&cx);
        redraw(&mut s, &mut cx, &mut g);
        s.click_id(&mut cx, "settings.tab.sound").expect("sound");
        let w = redraw(&mut s, &mut cx, &mut g);
        assert!(has(&w, "settings.mute_in_background") && has(&w, "settings.mute"), "the header pair");
        assert_eq!(cx.settings.v[MUTE_IN_BACKGROUND], json!(true));
        let volumes = |cx: &Cx| ["master", "music", "effects", "interface"].map(|k| cx.settings.v[k].clone());
        let before = volumes(&cx);
        cx.audio.set_levels(&cx.settings.v);
        let full = cx.audio.level(Bus::Music);
        assert!(full > 0.0);
        cx.audio.set_focused(false, &cx.settings.v);
        assert_eq!(cx.audio.level(Bus::Music), 0.0, "on: a lost focus mutes");
        cx.audio.set_focused(true, &cx.settings.v);
        assert_eq!(cx.audio.level(Bus::Music), full, "the focus back: restored");
        s.click_id(&mut cx, "settings.mute_in_background").expect("the toggle");
        assert_eq!(cx.settings.v[MUTE_IN_BACKGROUND], json!(false), "toggled off");
        cx.audio.set_focused(false, &cx.settings.v);
        assert_eq!(cx.audio.level(Bus::Music), full, "off: a lost focus keeps the sound");
        cx.audio.set_focused(true, &cx.settings.v);
        assert_eq!(volumes(&cx), before, "the saved volumes never change");
    }

    /// PR7-34 (AMBIENCE1): the Sound tab's Ward Ambience slider sits below Interface Volume, full screen and popup; its
    /// arrow saves the `ambience` key at once and the ambience bus plays at the new level, the music's untouched.
    #[test]
    fn the_sound_tab_has_a_ward_ambience_slider_on_its_own_bus() {
        let (mut cx, mut g) = (game_cx(), gfx());
        assert_eq!(Settings::defaults(default_settings())["ambience"], json!(80), "a default beside the other volumes");
        for popup in [false, true] {
            let mut s = if popup { SettingsScreen::in_game(&cx) } else { SettingsScreen::new(&cx) };
            redraw(&mut s, &mut cx, &mut g);
            s.click_id(&mut cx, "settings.tab.sound").expect("sound");
            let w = redraw(&mut s, &mut cx, &mut g);
            let y = |id: &str| w.iter().find(|x| x["id"] == id).and_then(|x| x["rect"][1].as_i64()).unwrap_or_else(|| panic!("{id} drawn (popup {popup})"));
            assert!(y("settings.interface") < y("settings.ambience"), "popup {popup}: below Interface Volume");
            assert!(has(&w, "settings.ambience.down") && has(&w, "settings.ambience.up"));
        }
        let mut s = SettingsScreen::new(&cx);
        redraw(&mut s, &mut cx, &mut g);
        s.click_id(&mut cx, "settings.tab.sound").expect("sound");
        redraw(&mut s, &mut cx, &mut g);
        cx.audio.set_levels(&cx.settings.v);
        let (music, bed) = (cx.audio.level(Bus::Music), cx.audio.level(Bus::Ambience));
        let was = cx.settings.v["ambience"].as_f64().unwrap();
        s.click_id(&mut cx, "settings.ambience.down").expect("the arrow");
        assert_eq!(saved(&cx, "ambience").as_f64(), Some(was - VOLUME_STEP), "saved at once");
        cx.audio.set_levels(&cx.settings.v);
        assert!(cx.audio.level(Bus::Ambience) < bed, "the ambience bus follows");
        assert_eq!(cx.audio.level(Bus::Music), music, "the music does not");
    }

    /// FB-308 (PR7-26 row): the Battle tab's rows past the mock's six (Units Behind, the overlay groups, the debug row)
    /// sit under an "Advanced" header, below Warn Missing Actions and above Units Behind, full screen and popup.
    #[test]
    fn the_battle_tab_puts_units_behind_and_the_overlays_under_advanced() {
        let (mut cx, mut g) = (game_cx(), gfx());
        for popup in [false, true] {
            let mut s = if popup { SettingsScreen::in_game(&cx) } else { SettingsScreen::new(&cx) };
            redraw(&mut s, &mut cx, &mut g);
            s.click_id(&mut cx, "settings.tab.battle").expect("battle");
            let w = redraw(&mut s, &mut cx, &mut g);
            let y = |id: &str| w.iter().find(|x| x["id"] == id).and_then(|x| x["rect"][1].as_i64()).unwrap_or_else(|| panic!("{id} drawn (popup {popup})"));
            let head = y("settings.heading.advanced");
            assert!(y("settings.warn_missing") < head && head < y("settings.see_through"), "popup {popup}: Advanced sits between the mock's rows and Units Behind");
            let first_overlay = w.iter().find(|x| x["id"].as_str().is_some_and(|i| i.starts_with("settings.overlay."))).and_then(|x| x["rect"][1].as_i64()).expect("an overlay group");
            assert!(head < first_overlay, "popup {popup}: the overlay groups are under it");
        }
    }

    /// PR7-26: on the web Restore defaults leaves the display keys as they are: nothing staged, no close guard.
    #[test]
    fn restore_defaults_on_the_web_stages_no_display_change() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let d = Settings::defaults(default_settings());
        let other = !d["fullscreen"].as_bool().unwrap_or(false);
        cx.settings.v["fullscreen"] = json!(other);
        cx.settings.v["show_tells"] = json!(false);
        let mut s = SettingsScreen::in_game(&cx).as_web();
        redraw(&mut s, &mut cx, &mut g);
        s.click_id(&mut cx, "settings.defaults").expect("defaults");
        assert_eq!(cx.settings.v["show_tells"], d["show_tells"], "the other rows reset at once");
        assert_eq!(cx.settings.v["fullscreen"], json!(other), "the display keys are untouched");
        assert!(!s.display_dirty(&cx), "nothing staged");
        s.click_id(&mut cx, "settings.close").expect("close");
        assert!(matches!(cx.nav.last(), Some(Nav::Pop)) && s.state(&cx)["guard"].is_null(), "closes with no guard: {}", s.state(&cx));
    }

    /// CONTROLS-UI1 (PR7-44): the Controls tab lists every action with its key and pad button; a click on a row waits
    /// for the raw input (`capture`), a key rebinds and the file is saved; right click (and Esc) cancel; a clash asks and
    /// Swap trades the two; Reset to defaults drops the map; the map read back from the saved file is the one shown.
    #[test]
    fn the_controls_tab_rebinds_saves_and_resets() {
        use aegis_ui::controls::{Controls, Input};
        use crate::controls_view::{binding_text, ACTIONS};
        let (mut cx, mut g) = (game_cx(), gfx());
        for popup in [false, true] {
            let mut s = if popup { SettingsScreen::in_game(&cx) } else { SettingsScreen::new(&cx) };
            redraw(&mut s, &mut cx, &mut g);
            s.click_id(&mut cx, "settings.tab.controls").expect("the tab");
            let w = redraw(&mut s, &mut cx, &mut g);
            let row = |i: usize| format!("settings_controls.f_bindings.{i}");
            for (i, a) in ACTIONS.iter().enumerate() {
                let r = w.iter().find(|x| x["id"] == json!(row(i))).unwrap_or_else(|| panic!("{}'s row", a.id));
                assert!(r["text"].as_str().unwrap().contains(a.label), "{r}");
            }
            let at = |id: &str| ACTIONS.iter().position(|a| a.id == id).unwrap();
            // idle: a key goes on to the screen
            assert!(!s.capture(&mut cx, Input::Key(Key::Char('r'))));
            s.click_id(&mut cx, &row(at("journal"))).expect("the row");
            assert_eq!(s.state(&cx)["rebind"], json!({"step": "waiting", "action": "journal"}));
            assert!(s.capture(&mut cx, Input::Key(Key::Char('k'))));
            assert_eq!(saved(&cx, "controls")["keys"]["k"], json!("j"), "saved at once");
            // the file read again gives the same map and the row shows it
            let back = Settings::load(cx.shot_dir.join("settings.json"), default_settings());
            let c = Controls::from_settings(&back.v).0;
            assert_eq!(c, Controls::from_settings(&cx.settings.v).0);
            assert_eq!(c.key_for(Key::Char('j')), Some(Key::Char('k')));
            assert!(binding_text(&c, &ACTIONS[at("journal")]).starts_with("K · "));
            // right click cancels a waiting rebind and keeps the screen open
            s.click_id(&mut cx, &row(at("journal"))).expect("the row");
            s.back(&mut cx);
            assert_eq!(s.state(&cx)["rebind"], json!({"step": "idle"}));
            assert!(cx.nav.is_empty(), "the screen stays");
            // a clash asks; Swap trades
            s.click_id(&mut cx, &row(at("interact"))).expect("the row");
            assert!(s.capture(&mut cx, Input::Key(Key::Char('z'))));
            assert_eq!(s.state(&cx)["rebind"]["with"], json!("undo"));
            // the question fits at every text size (GUI audit)
            for &(_, scale) in TEXT_SIZES.iter() {
                let recs = frame(&mut s, &mut cx, &mut g, scale as f32);
                let boxes: Vec<(String, f32, f32, f32, f32)> = s.widgets().iter().filter(|w| w["kind"] != "scroll" && rect(w).2 > 0 && rect(w).3 > 0).map(|w| {
                    let r = rect(w);
                    (w["id"].as_str().unwrap_or("").to_string(), r.0 as f32, r.1 as f32, r.2 as f32, r.3 as f32)
                }).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "popup {popup} scale {scale}: {v}");
                assert!(recs.iter().any(|r| r.text.contains("taken")), "the asked row says so");
            }
            redraw(&mut s, &mut cx, &mut g);
            s.click_id(&mut cx, "settings_controls.swap").expect("Swap");
            let c = Controls::from_settings(&cx.settings.v).0;
            assert_eq!((c.key_for(Key::Char('e')), c.key_for(Key::Char('z'))), (Some(Key::Char('z')), Some(Key::Char('e'))));
            // Reset to defaults: the saved file has no map
            redraw(&mut s, &mut cx, &mut g);
            s.click_id(&mut cx, "settings_controls.reset").expect("Reset");
            assert_eq!(saved(&cx, "controls"), Value::Null);
            assert_eq!(Controls::from_settings(&cx.settings.v).0, Controls::default());
        }
    }

    /// Every tab, the confirm and the guard fit at the four text sizes, full screen and popup: no text spills out of
    /// its widget, is cut, runs off the canvas or collides with another.
    #[test]
    fn every_tab_and_box_fits_at_every_text_size() {
        let (mut cx, mut g) = (game_cx(), gfx());
        g.canvas = Canvas::new(1280, 720);
        g.view = View::fit(1280, 720);
        for popup in [false, true] {
            for &(_, scale) in TEXT_SIZES.iter() {
                for view in 0..TABS.len() + 2 {
                    let mut s = if popup { SettingsScreen::in_game(&cx) } else { SettingsScreen::new(&cx) };
                    s.act(&mut cx, SetAct::Tab(if view < TABS.len() { view } else { 0 }));
                    if view >= TABS.len() {
                        let other = if WindowShow::of(&cx.settings.v) == WindowShow::Borderless { WindowShow::Windowed } else { WindowShow::Borderless };
                        stage_mode(&mut s, &mut cx, &mut g, other);
                        s.act(&mut cx, if view == TABS.len() { SetAct::Apply } else { SetAct::Close });
                    }
                    let recs = frame(&mut s, &mut cx, &mut g, scale as f32);
                    // (a row scrolled out of the body keeps its widget, clipped to nothing: no holder for any text)
                    let boxes: Vec<(String, f32, f32, f32, f32)> = s.widgets().iter().filter(|w| w["kind"] != "scroll" && rect(w).2 > 0 && rect(w).3 > 0).map(|w| {
                        let r = rect(w);
                        (w["id"].as_str().unwrap_or("").to_string(), r.0 as f32, r.1 as f32, r.2 as f32, r.3 as f32)
                    }).collect();
                    let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                    assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "popup {popup} scale {scale} view {view}: {v}");
                    if view == TABS.len() {
                        s.act(&mut cx, SetAct::Revert);
                    }
                }
            }
        }
    }

    /// Review shots (only with `AEGIS_REVIEW_DIR` set): each tab at Normal and Extra large text, full screen and popup,
    /// the keep-or-revert confirm and the close guard, at 1280 x 720 with the game's theme.
    #[test]
    fn review_shots() {
        let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
        std::fs::create_dir_all(&out).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut cx = game_cx();
        cx.art = aegis_ui::art::Art::new(Some(root.to_path_buf()));
        aegis_ui::theme::set_theme(aegis_ui::theme::load(cx.art.layers()).expect("the game's theme"));
        let mut g = gfx();
        g.canvas = Canvas::new(1280, 720);
        g.view = View::fit(1280, 720);
        let shot = |s: &mut SettingsScreen, cx: &mut Cx, g: &mut Gfx, name: &str, scale: f32| {
            g.canvas.clear([22, 20, 26, 255]);
            frame(s, cx, g, scale);
            let file = std::fs::File::create(format!("{out}/{name}.png")).unwrap();
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 1280, 720);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
        };
        for (popup, kind) in [(false, "full"), (true, "popup")] {
            for (scale, size) in [(1.0, "normal"), (1.3, "xl")] {
                for (i, (name, _)) in TABS.iter().enumerate() {
                    let mut s = if popup { SettingsScreen::in_game(&cx) } else { SettingsScreen::new(&cx) };
                    s.act(&mut cx, SetAct::Tab(i));
                    shot(&mut s, &mut cx, &mut g, &format!("settings_{kind}_{}_{size}", name.to_lowercase()), scale);
                }
                let mut s = if popup { SettingsScreen::in_game(&cx) } else { SettingsScreen::new(&cx) };
                stage_mode(&mut s, &mut cx, &mut g, WindowShow::Borderless);
                s.act(&mut cx, SetAct::Step("window_size", 1));
                shot(&mut s, &mut cx, &mut g, &format!("settings_{kind}_display_staged_{size}"), scale);
                s.act(&mut cx, SetAct::Close);
                shot(&mut s, &mut cx, &mut g, &format!("settings_{kind}_close_guard_{size}"), scale);
                s.act(&mut cx, SetAct::GuardApply);
                shot(&mut s, &mut cx, &mut g, &format!("settings_{kind}_revert_confirm_{size}"), scale);
                s.act(&mut cx, SetAct::Revert);
            }
        }
        aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
    }
}
