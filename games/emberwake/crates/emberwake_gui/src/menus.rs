//! Main menu, New game, Load / Save slots and Settings (scripts/ui/menus: main_menu_model, new_game_model,
//! save_slots_model, settings_model). Placeholder logic as in Godot; file import / export of saves is not ported yet.

use aegis_layout::{frac, resolve, Anchor, Insets, Stack};
use aegis_ui::theme::th;
use aegis_ui::audio::Bus;
use aegis_ui::canvas::{rgba, Rect};
use crate::music;
use crate::portraits::{Faces, Who};
use crate::screens::{default_settings, enter_game, show_errors, title, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::painter::{Grid, Typeface};
use aegis_ui::panes::{self, MenuItem, MenuStyle};
use emberwake_battle::portrait::Look;
use aegis_ui::ui::{tip, Gfx, Glyph, Ui, Widget, BODY, H, W};
use crate::screens::WINDOW_SIZES;
use aegis_ui::screen::{Settings, WindowShow, FPS_CAPS, TEXT_SIZES, TEXT_SPEEDS};
use aegis_ui::vignette::{TextSpeed, REDUCE_MOTION, TEXT_SPEED};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or};
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
    Tools,
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

/// What Continue's peek shows of the newest save.
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
}

impl MainMenu {
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
            MenuAct::NewGame => cx.nav.push(Nav::Push(Box::new(NewGameScreen::new(cx)))),
            MenuAct::Load => cx.nav.push(Nav::Push(Box::new(Slots::new(false)))),
            MenuAct::Settings => cx.nav.push(Nav::Push(Box::new(SettingsScreen::new(cx)))),
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
            MenuItem::new("tools", "Tools (Forge, Dojo, Test Arena)", None, MenuAct::Tools),
            MenuItem::new("quit", "Quit", None, MenuAct::Quit),
        ]);
        // The column widens leftward at the larger text sizes so its longest label is never cut (only the labels take
        // the mouse: `MenuStyle::Text`).
        let widest = items.iter().map(|i| ui.p.text_w(&i.label, m.title)).fold(0.0, f32::max);
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
    /// The peek's portraits breathe on the menu's clock.
    fn tick(&mut self, _cx: &mut Cx) {
        self.faces.tick();
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
    pub fn new(cx: &Cx) -> Self {
        Self { selected: new_game::playable_worlds(&cx.data).first().cloned().unwrap_or_default(), w: Vec::new() }
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
        }
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
    fn id(&self) -> &'static str {
        "new_game"
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<NgAct> = Ui::new(g, mouse, live);
        title(&mut ui, "New game", 26);
        let mut y = 54;
        for w in new_game::playable_worlds(&cx.data) {
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
        Self { saving, w: Vec::new() }
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
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<SlotAct> = Ui::new(g, mouse, live);
        title(&mut ui, if self.saving { "Save game" } else { "Load game" }, 26);
        for slot in save::AUTOSAVE_SLOT..=save::LAST_SLOT {
            let r = Rect::new(70, 54 + slot as i32 * 36, 500, 31);
            let head = save::peek(&cx.data, &cx.save_dir, slot);
            let exists = json::b(&head, "exists", false);
            let ok = json::b(&head, "ok", false);
            let label = if slot == 0 { "Autosave".to_string() } else { format!("Slot {slot}") };
            let status = if ok {
                let world = s(&head, "world_id");
                let party: Vec<String> = json::aget(&head, "party")
                    .iter()
                    .map(|m| {
                        let m = json::dict(m);
                        format!("{} Lv {}", s_or(cx.data.entry("units", &s(m, "base_unit")), "name", &s(m, "base_unit")), json::i(m, "level", 0))
                    })
                    .collect();
                let at = json::i(&head, "saved_at", 0);
                format!("{}, {} · {} · {}", cx.world_name(&world), cx.location_name(&world, &s(&head, "location_id")), party.join(", "), if at > 0 { datetime(at) } else { String::new() })
            } else if exists {
                format!("Can't load ({})", s(&head, "error"))
            } else {
                "Empty".into()
            };
            // The whole row is the click (PLAYUI1 section 1); lock and delete are icons in its top-right corner.
            let locked = save::is_locked(&cx.save_dir, slot);
            let row = slot_row(self.saving, slot, exists, ok, locked, Self::no_save(cx));
            let hover = row.action.is_some() && ui.hover(r);
            ui.p.panel(r, if hover { th().card_hot } else { th().game("menu_row", rgba(26, 36, 58, 250)) }, if hover { th().gold } else { th().edge }, 2.0);
            let text_w = (r.w - 8 - if row.delete { 40 } else { 0 }) as f32;
            ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 3.0, &label, BODY, text_w, if ok { th().text } else { th().dim });
            ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 16.0, &status, BODY, text_w, if ok { th().info } else { th().dim });
            let enabled = row.action.is_some();
            ui.push(format!("slot.{slot}"), "card", format!("{label}: {status}"), r, enabled, false, row.action, tip(&label, &row.why));
            let del = Rect::new(r.x + r.w - 4 - 16, r.y + 4, 16, 16);
            let px = ui.icon_px(del);
            if row.lock {
                let lr = Rect::new(del.x - 18, del.y, 16, 16);
                let (label, card, glyph, file) = if locked {
                    ("Unlock", "Unlock", Glyph::Lock, "ui/lock.png")
                } else {
                    ("Lock", "Lock (protect from overwrite and delete)", Glyph::Unlock, "ui/unlock.png")
                };
                let img = cx.art.image(file, px, px).cloned();
                ui.icon_button(&format!("slot.{slot}.lock"), lr, label, img.as_ref(), glyph, SlotAct::Lock(slot, !locked), true, locked, tip(card, ""));
            }
            if row.delete {
                let img = cx.art.image("ui/delete.png", px, px).cloned();
                let card = if locked { tip("Delete", "Unlock the save to delete it.") } else { tip("Delete", "") };
                ui.icon_button(&format!("slot.{slot}.delete"), del, "Delete", img.as_ref(), Glyph::Delete, SlotAct::Delete(slot), !locked, false, card);
            }
        }
        ui.button("slots.back", Rect::new(70, 292, 70, 18), "Back", SlotAct::Back, true);
        self.w = ui.widgets;
    }
    widget_plumbing!();
}

// ---- Settings ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum SetAct {
    Tab(usize),
    Toggle(&'static str),
    /// A slider value (volumes 0-100, speeds and scales 0.5-2.0).
    Set(&'static str, f64),
    /// A choice from CHOICES by index.
    Pick(&'static str, usize),
    /// RULESET1-U slice 6: a battle overlay group switched (the settings key `battle_overlays`).
    Overlay(&'static str),
    Apply,
    Defaults,
    Close,
}

pub struct SettingsScreen {
    tab: usize,
    draft: Value,
    w: Vec<Widget<SetAct>>,
}

const TABS: [&str; 5] = ["Sound", "Display", "Gameplay", "Battle", "Controls"];
/// The display choices: the game's window sizes, the engine's frame-rate caps (aegis_ui::screen).
const CHOICES: [(&str, &[&str]); 2] = [("window_size", &WINDOW_SIZES), ("fps_cap", &FPS_CAPS)];
/// PLAYUI1-M: the region map's marker names (value, label); the first is the default.
pub const MAP_NAMES: [(&str, &str); 2] = [("hover", "On hover"), ("always", "Always")];
pub(crate) const CONTROLS: [(&str, &[(&str, &str)]); 3] = [
    (
        "Menus",
        &[
            ("Confirm / press the focused button", "Enter, Space"),
            ("Back / close the top panel", "Escape"),
            ("Focus the next / previous control", "Tab, Shift+Tab"),
            ("Move focus", "Arrow keys (not on the map)"),
            ("Step a focused slider", "Left, Right"),
            ("Back / close (mouse)", "Right click"),
        ],
    ),
    ("Maps", &[("Move the party", "Arrow keys, W A S D"), ("Run", "Hold Shift"), ("Interact with the marker you face", "E"), ("Game menu", "Escape"), ("Choose a dialogue line", "1-9"), ("Party, journal, stash, help, event log", "P, J, I, ?, L")]),
    ("Battle", &[("Confirm the round", "Enter"), ("Step / next round", "Space"), ("Autoplay", "A"), ("Undo the hero's last step", "Z, right click"), ("Select party member 1-4", "1-4"), ("Skills of the selected hero (timeline)", "Shift+1-9"), ("Accept the selected hero's auto-fill (timeline)", "F"), ("Support an ally while only the auto-fill shows (timeline)", "Ctrl+click"), ("Target self (timeline)", "S"), ("Inspect a unit (timeline)", "Right click"), ("Clear inspect, deselect (timeline)", "Escape"), ("Full timeline view, open / close (timeline)", "T, Escape"), ("Playback speed: 1x, 0.5x, block by block (timeline)", "P"), ("Next block (timeline)", "N"), ("Combat log", "L"), ("Outline every unit", "Hold Alt"), ("Every bar, arrow and hazard mark (timeline)", "Hold Alt"), ("Screenshot", "F12")]),
];

/// A volume slider's sample and bus (scripts/audio/volume_preview.gd: the master sample plays on the interface bus).
fn preview_of(key: &str) -> Option<(Bus, &'static str)> {
    match key {
        "master" => Some((Bus::Ui, "sfx/ui_confirm.wav")),
        "music" => Some((Bus::Music, "sfx/ui_confirm.wav")),
        "effects" => Some((Bus::Sfx, "sfx/hit.wav")),
        "interface" => Some((Bus::Ui, "sfx/ui_click.wav")),
        _ => None,
    }
}

impl SettingsScreen {
    pub fn new(cx: &Cx) -> Self {
        Self { tab: 0, draft: cx.settings.v.clone(), w: Vec::new() }
    }

    fn act(&mut self, cx: &mut Cx, a: SetAct) {
        match a {
            SetAct::Tab(t) => self.tab = t,
            SetAct::Toggle(k) => self.draft[k] = Value::Bool(!self.draft[k].as_bool().unwrap_or(false)),
            SetAct::Set(k, v) => {
                let (lo, hi) = if k.ends_with("speed") || k.ends_with("scale") {
                    (0.5, 2.0)
                } else if k == crate::mapview::WALK_STEP_KEY {
                    (crate::mapview::WALK_STEP_MS.0, crate::mapview::WALK_STEP_MS.1)
                } else {
                    (0.0, 100.0)
                };
                let v = v.clamp(lo, hi);
                self.draft[k] = if lo == 0.0 { json!(v.round() as i64) } else { json!((v * 100.0).round() / 100.0) };
                if let Some((bus, sample)) = preview_of(k) {
                    cx.audio.set_levels(&self.draft);
                    cx.audio.preview(bus, sample);
                }
            }
            SetAct::Pick("map_names", i) => {
                if let Some((v, _)) = MAP_NAMES.get(i) {
                    self.draft["map_names"] = json!(v);
                }
            }
            SetAct::Pick("map_zoom", i) => {
                if let Some(v) = crate::mapview::zoom_steps().nth(i) {
                    self.draft["map_zoom"] = json!(v);
                }
            }
            SetAct::Pick("see_through", i) => {
                if let Some((v, _)) = crate::stage::SEE_THROUGH.get(i) {
                    self.draft["see_through"] = json!(v);
                }
            }
            SetAct::Pick(TEXT_SPEED, i) => {
                if let Some(v) = TEXT_SPEEDS.get(i) {
                    self.draft[TEXT_SPEED] = json!(v);
                }
            }
            SetAct::Pick(crate::battle::PLAYBACK_SPEED, i) => {
                if let Some((v, _, _)) = crate::battle::PLAYBACK_SPEEDS.get(i) {
                    self.draft[crate::battle::PLAYBACK_SPEED] = json!(v);
                }
            }
            SetAct::Pick(k, i) => {
                if let Some(o) = choices(k).get(i) {
                    self.draft[k] = json!(o);
                }
            }
            SetAct::Overlay(k) => {
                let on = self.draft["battle_overlays"][k].as_bool().unwrap_or_else(|| crate::battle::overlay_default(k));
                if !self.draft["battle_overlays"].is_object() {
                    self.draft["battle_overlays"] = json!({});
                }
                self.draft["battle_overlays"][k] = Value::Bool(!on);
            }
            SetAct::Apply => {
                cx.settings.v = self.draft.clone();
                if let Err(e) = cx.settings.save() {
                    show_errors(cx, vec![format!("Settings could not be saved: {e}")]);
                }
            }
            SetAct::Defaults => self.draft = Settings::defaults(default_settings()),
            SetAct::Close => cx.nav.push(Nav::Pop),
        }
    }
}

impl Screen<GameCx> for SettingsScreen {
    fn id(&self) -> &'static str {
        "settings"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        // The draft levels play while the popup is open; the shell puts the saved ones back each frame.
        cx.audio.set_levels(&self.draft);
        let mut ui: Ui<SetAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("settings_veil", rgba(0, 0, 0, 140)));
        let r = Rect::new(95, 25, 450, 310);
        ui.frame(r, "Settings", false);
        ui.close_x("settings.close", Rect::new(r.x + r.w - 28, r.y + 9, 19, 18), SetAct::Close);
        let mut tx = r.x + 12;
        for (i, t) in TABS.iter().enumerate() {
            // At least 44 wide; wider when the label needs it (larger text sizes).
            let tw = 44.max(ui.p.text_w(t, BODY).ceil() as i32 + 10);
            ui.tab(&format!("settings.tab.{}", t.to_lowercase()), Rect::new(tx, r.y + 36, tw, 16), t, SetAct::Tab(i), self.tab == i);
            tx += tw + 2;
        }
        let body = Rect::new(r.x + 9, r.y + 55, r.w - 18, 222);
        ui.p.panel(body, th().game("settings_body", rgba(24, 26, 32, 255)), th().game("settings_body", rgba(24, 26, 32, 255)), 1.0);
        let (x, mut y) = (body.x + 5, body.y + 6);
        let d = self.draft.clone();
        let pct = |k: &str| d[k].as_f64().unwrap_or(0.0);
        match self.tab {
            0 => {
                ui.checkbox("settings.mute", x + 3, y, "Mute", d["mute"].as_bool().unwrap_or(false), SetAct::Toggle("mute"), true);
                y += 20;
                for (k, label) in [("master", "Master volume"), ("music", "Music volume"), ("effects", "Effects volume"), ("interface", "Interface volume")] {
                    ui.p.text(x as f32, y as f32 + 2.0, label, BODY, th().text);
                    let key = static_key(k);
                    ui.slider(&format!("settings.{k}"), Rect::new(x + 125, y, 200, 13), (pct(k) / VOLUME_STEP).round() as usize, 21, &format!("{}%", pct(k)), |i| SetAct::Set(key, i as f64 * VOLUME_STEP), true);
                    y += 18;
                }
            }
            1 => {
                let show = WindowShow::of(&d);
                ui.checkbox("settings.fullscreen", x + 3, y, "Fullscreen", show == WindowShow::Fullscreen, SetAct::Toggle("fullscreen"), true);
                y += 18;
                ui.checkbox("settings.borderless", x + 3, y, "Borderless", d["borderless"].as_bool().unwrap_or(false), SetAct::Toggle("borderless"), true);
                y += 20;
                ui.p.text(x as f32, y as f32 + 3.0, "Resolution", BODY, th().text);
                choice(&mut ui, "window_size", Rect::new(x + 125, y, 150, 16), d["window_size"].as_str().unwrap_or(""), show != WindowShow::Fullscreen);
                y += 22;
                ui.p.text(x as f32, y as f32, "In use", BODY, th().text);
                let size = d["window_size"].as_str().unwrap_or("");
                let in_use = match show {
                    WindowShow::Fullscreen => "Fullscreen".to_string(),
                    WindowShow::Borderless => format!("Borderless {size}"),
                    WindowShow::Windowed => format!("Windowed {size}"),
                };
                ui.p.text(x as f32 + 125.0, y as f32, &in_use, BODY, th().dim);
                y += 18;
                ui.checkbox("settings.vsync", x + 3, y, "Vertical sync", d["vsync"].as_bool().unwrap_or(true), SetAct::Toggle("vsync"), true);
                y += 20;
                ui.p.text(x as f32, y as f32 + 3.0, "Frame rate cap", BODY, th().text);
                choice(&mut ui, "fps_cap", Rect::new(x + 125, y, 150, 16), d["fps_cap"].as_str().unwrap_or(""), true);
                y += 24;
                ui.p.text(x as f32, y as f32 + 3.0, "Text size", BODY, th().text);
                // The engine's four levels (Godot TEXT_LEVELS); a saved value off the levels shows the nearest one.
                let ts = pct("text_scale");
                let labels: Vec<String> = TEXT_SIZES.iter().map(|t| t.0.to_string()).collect();
                let at = TEXT_SIZES.iter().enumerate().min_by(|a, b| (a.1 .1 - ts).abs().total_cmp(&(b.1 .1 - ts).abs())).map(|(i, _)| i).unwrap_or(1);
                ui.select("settings.text_scale", Rect::new(x + 125, y, 150, 16), &labels, at, |i| SetAct::Set("text_scale", TEXT_SIZES[i].1), true, Vec::new());
            }
            2 => {
                ui.p.text(x as f32, y as f32 + 2.0, "Animation speed", BODY, th().text);
                let a = pct("anim_speed");
                ui.slider("settings.anim_speed", Rect::new(x + 125, y, 200, 13), ((a - 0.5) / SCALE_STEP).round() as usize, 7, &format!("{a:.2}x"), |i| SetAct::Set("anim_speed", 0.5 + i as f64 * SCALE_STEP), true);
                ui.checkbox("settings.skip_anim", x + 300, y + 1, "Skip animations", d["skip_anim"].as_bool().unwrap_or(false), SetAct::Toggle("skip_anim"), true);
                y += 22;
                ui.checkbox("settings.show_tells", x + 3, y, "Show enemy tells", d["show_tells"].as_bool().unwrap_or(true), SetAct::Toggle("show_tells"), true);
                y += 18;
                ui.checkbox("settings.warn_missing", x + 3, y, "Warn about missing actions", d["warn_missing"].as_bool().unwrap_or(true), SetAct::Toggle("warn_missing"), true);
                y += 18;
                // INPUT1 GH7: the planner fills a hero's free blocks after its picks (timeline battles)
                ui.checkbox("settings.auto_fill", x + 3, y, "Auto-fill the rest", d["auto_fill"].as_bool().unwrap_or(true), SetAct::Toggle("auto_fill"), true);
                if let Some(wd) = ui.widgets.last_mut() {
                    wd.tip = tip("Auto-fill the rest", "After your picks the planner fills the round's free blocks with follow-ups (ghost blocks you can replace). Off: the free blocks Rest.");
                }
                y += 20;
                ui.p.text(x as f32, y as f32 + 3.0, "Map names", BODY, th().text);
                let labels: Vec<String> = MAP_NAMES.iter().map(|m| m.1.to_string()).collect();
                let at = MAP_NAMES.iter().position(|m| d["map_names"].as_str() == Some(m.0)).unwrap_or(0);
                let t = tip("Map names", "On hover: a marker's name shows while you point at it, its legend row, or face it. Always: every person and exit is named.");
                ui.select("settings.map_names", Rect::new(x + 125, y, 150, 16), &labels, at, |i| SetAct::Pick("map_names", i), true, t);
                y += 22;
                // REGIONVIEW1: the region map's zoom
                ui.p.text(x as f32, y as f32 + 3.0, "Map zoom", BODY, th().text);
                let labels: Vec<String> = crate::mapview::zoom_steps().map(|z| format!("{z}x")).collect();
                let at = (crate::mapview::zoom_of(&d["map_zoom"]) - crate::mapview::ZOOM_QUARTERS.0) as usize;
                ui.select("settings.map_zoom", Rect::new(x + 125, y, 150, 16), &labels, at, |i| SetAct::Pick("map_zoom", i), true, tip("Map zoom", crate::mapview::ZOOM_TIP));
                y += 22;
                // STAGE1-O: when units hidden behind others or cover are outlined in battle (the Flat view is a Scene
                // Runner debug setting since STAGE1-C, not a player choice).
                ui.p.text(x as f32, y as f32 + 3.0, "Units behind", BODY, th().text);
                let labels: Vec<String> = crate::stage::SEE_THROUGH.iter().map(|v| v.1.to_string()).collect();
                let at = crate::stage::SEE_THROUGH.iter().position(|v| d["see_through"].as_str() == Some(v.0)).unwrap_or(0);
                ui.select("settings.see_through", Rect::new(x + 125, y, 150, 16), &labels, at, |i| SetAct::Pick("see_through", i), true, tip("Units behind", crate::stage::SEE_THROUGH_TIP));
                y += 22;
                // STORYUI1 4.8: the story cards' typewriter speed and the shared reduce-motion switch (engine keys)
                ui.p.text(x as f32, y as f32 + 3.0, "Text speed", BODY, th().text);
                let labels: Vec<String> = TEXT_SPEEDS.iter().map(|v| TextSpeed::parse(v).label().to_string()).collect();
                let at = TEXT_SPEEDS.iter().position(|v| d[TEXT_SPEED].as_str() == Some(*v)).unwrap_or(1);
                let t = tip("Text speed", "How fast story captions type out: Slow, Normal, Fast, or Instant (the whole text at once). A click shows the rest at any speed.");
                ui.select("settings.text_speed", Rect::new(x + 125, y, 150, 16), &labels, at, |i| SetAct::Pick(TEXT_SPEED, i), true, t);
                y += 22;
                ui.checkbox("settings.reduce_motion", x + 3, y, "Reduce motion", d[REDUCE_MOTION].as_bool().unwrap_or(false), SetAct::Toggle(REDUCE_MOTION), true);
            }
            3 => {
                // RULESET1-U slice 6 (preference "Battle overlays and the clarity pass", user 2026-10-30): every
                // overlay group of a timeline battle, on by default; the clarity pass (VISCLAR1) decides later what
                // the game view keeps.
                ui.p.text(x as f32, y as f32, "Battle overlays (timeline battles)", BODY + 1.0, th().gold);
                // INPUT1 PB3: the playback speed (the battle's chips and key P set it too) at the heading row's end
                let speeds = &crate::battle::PLAYBACK_SPEEDS;
                let labels: Vec<String> = speeds.iter().map(|v| v.1.to_string()).collect();
                let at = speeds.iter().position(|v| d[crate::battle::PLAYBACK_SPEED].as_str() == Some(v.0)).unwrap_or(0);
                let sr = Rect::new(body.x + body.w - 5 - 64, y - 2, 64, 14);
                let label = "Playback speed";
                let lw = ui.p.text_w(label, BODY);
                ui.p.text(sr.x as f32 - 4.0 - lw, y as f32, label, BODY, th().text);
                let t = tip("Playback speed", "How a timeline battle plays its round: 1x; 0.5x (every walk, clip, float and pause twice as long); Block (it waits after each block until N or Next block). P cycles it in battle.");
                ui.select("settings.playback_speed", sr, &labels, at, |i| SetAct::Pick(crate::battle::PLAYBACK_SPEED, i), true, t);
                y += 14;
                let col_w = (body.w - 10) / 2;
                // VISCLAR1-A, INPUT1-PB, VISCLAR1-B: 12 rows of two at a 14 px pitch, so the debug row fits under them
                let groups = &crate::battle::OVERLAY_GROUPS;
                for (k, (id, label, help, default)) in groups.iter().enumerate() {
                    let (cx_, cy_) = (x + 3 + (k as i32 % 2) * col_w, y + (k as i32 / 2) * OVERLAY_PITCH);
                    let on = d["battle_overlays"][*id].as_bool().unwrap_or(*default);
                    ui.checkbox(&format!("settings.overlay.{id}"), cx_, cy_, label, on, SetAct::Overlay(id), true);
                    if let Some(wd) = ui.widgets.last_mut() {
                        wd.tip = tip(label, help);
                    }
                }
                // VISCLAR1 VC4, VC5: the debug-build-only Party foresight off (never in a release build), a row under the
                // groups: the heading, the checkbox in the second column
                #[cfg(debug_assertions)]
                {
                    let y = y + groups.len().div_ceil(2) as i32 * OVERLAY_PITCH + 2;
                    ui.p.text(x as f32, y as f32, "Debug", BODY + 1.0, th().gold);
                    // VIEWKEYS1: the walk's base step time (a release build uses the default), beside the heading
                    let (least, most, _) = crate::mapview::WALK_STEP_MS;
                    let base = crate::mapview::WalkStep::of(&json!({crate::mapview::WALK_STEP_KEY: d[crate::mapview::WALK_STEP_KEY]}), false).base;
                    ui.p.text(x as f32 + 36.0, y as f32, "Walk step", BODY, th().text);
                    let steps = ((most - least) / WALK_STEP_STEP) as usize + 1;
                    ui.slider("settings.debug.walk_step", Rect::new(x + 84, y, col_w - 92, 13), ((base - least) / WALK_STEP_STEP).round() as usize, steps, &format!("{base:.0} ms"), |i| SetAct::Set(crate::mapview::WALK_STEP_KEY, least + i as f64 * WALK_STEP_STEP), true);
                    let at = x + 3 + col_w;
                    ui.checkbox("settings.debug.party_foresight_off", at, y, "Party foresight off", d[crate::battle::FORESIGHT_OFF].as_bool().unwrap_or(false), SetAct::Toggle(crate::battle::FORESIGHT_OFF), true);
                    if let Some(wd) = ui.widgets.last_mut() {
                        wd.tip = tip("Party foresight off", "Debug: every enemy plan a party foresight passive reveals (Hawk Eye) is drawn as unrevealed. Enrage and the battle's plan reveal still show. Display only.");
                    }
                }
            }
            _ => {
                // The list scrolls inside the panel (the wheel over it) once it outgrows it.
                let content: i32 = CONTROLS.iter().map(|(_, rows)| 14 + rows.len() as i32 * 11 + 3).sum::<i32>() + 6;
                let off = ui.scroll_begin("settings.controls", body, content);
                y -= off;
                for (head, rows) in CONTROLS {
                    ui.p.text(x as f32, y as f32, head, BODY + 2.0, th().gold);
                    y += 14;
                    for (what, keys) in rows {
                        ui.p.text(x as f32, y as f32, what, BODY, th().text);
                        ui.p.text(x as f32 + 195.0, y as f32, keys, BODY, th().dim);
                        y += 11;
                    }
                    y += 3;
                }
                ui.scroll_end();
            }
        }
        ui.button("settings.defaults", Rect::new(r.x + r.w - 160, r.y + r.h - 25, 80, 17), "Restore defaults", SetAct::Defaults, true);
        ui.button("settings.apply", Rect::new(r.x + r.w - 75, r.y + r.h - 25, 66, 17), "Apply", SetAct::Apply, true);
        self.w = ui.widgets;
    }
    fn state(&self, _cx: &Cx) -> Value {
        json!({"tab": TABS[self.tab], "draft": self.draft})
    }
    widget_plumbing!();
}

/// VISCLAR1-A, INPUT1-PB: the row pitch of the Battle tab's overlay checkboxes (25 groups in 13 rows, then the debug
/// row).
const OVERLAY_PITCH: i32 = 14;

/// Volume sliders move in steps of 5 (0-100); speed and text-size sliders in quarters (0.5-2.0).
const VOLUME_STEP: f64 = 5.0;
const SCALE_STEP: f64 = 0.25;
/// The debug Walk step slider's step, ms.
#[cfg(debug_assertions)]
const WALK_STEP_STEP: f64 = 10.0;

fn choices(k: &str) -> &'static [&'static str] {
    CHOICES.iter().find(|c| c.0 == k).map(|c| c.1).unwrap_or(&[])
}

/// A settings choice (CHOICES) as a select with its option list.
fn choice(ui: &mut Ui<SetAct>, k: &'static str, r: Rect, current: &str, enabled: bool) {
    let opts: Vec<String> = choices(k).iter().map(|o| o.to_string()).collect();
    let at = opts.iter().position(|o| o == current).unwrap_or(0);
    ui.select(&format!("settings.{k}"), r, &opts, at, |i| SetAct::Pick(k, i), enabled, Vec::new());
}

fn static_key(k: &str) -> &'static str {
    ["master", "music", "effects", "interface"].into_iter().find(|x| *x == k).unwrap_or("master")
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

    /// PLAYUI1 section 1 acceptance over the screen's widgets (the agent link's ids): `slot.2` saves an empty slot at
    /// once and asks on a filled one; `slot.2.lock` locks it, after which `slot.2` and its delete take no click and
    /// their cards name the lock; the autosave has no lock; the load screen loads a locked save; unlock, delete.
    #[test]
    fn the_save_screen_rows_and_icons_follow_the_slot_rules() {
        let dir = std::env::temp_dir().join(format!("emberwake_slots_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut cx = cx(dir.clone());
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

        // The icons sit 4 px from the row's top and right edges, right to left: delete, lock.
        let (row, del, lock) = (get(&w, "slot.2").unwrap(), get(&w, "slot.2.delete").unwrap(), get(&w, "slot.2.lock").unwrap());
        assert_eq!((n(del, 0) + n(del, 2), n(del, 1)), (n(row, 0) + n(row, 2) - 4, n(row, 1) + 4));
        assert_eq!((n(del, 2), n(del, 3)), (16, 16));
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

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("emberwake_m1_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
        let w = menu_frame(&mut m, &mut cx, &mut g, OVER_FIRST);
        let p = get(&w, "menu.peek").expect("hovered: the peek");
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
        cx.app.manifest.title = "Fixture".into();
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.clone()]));
        let mut m = MainMenu::default();
        let w = menu_frame(&mut m, &mut cx, &mut g, (-1, -1));
        assert_eq!(get(&w, "menu.backdrop").unwrap()["text"], json!(MENU_ART));
        assert_eq!(get(&w, "menu.backdrop").unwrap()["kind"], json!("picture"));
        assert!(get(&w, "menu.crest").is_none(), "no crest file, no crest");
        assert!(get(&w, "menu.logo").is_none(), "no title picture");
        std::fs::copy(&source, root.join(TITLE_ART)).unwrap();
        std::fs::create_dir_all(root.join("ui")).unwrap();
        std::fs::copy(&source, root.join(CREST_ART)).unwrap();
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.clone()]));
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
            let (mut cx, mut g) = (cx(dir.clone()), gfx());
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.clone()]));
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
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.clone()]));
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.clone()]));
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
        cx.app.manifest.title = "Fixture title".into();
        cx.app.manifest.subtitle = "Fixture subtitle".into();
        let mut audited = |cx: &mut Cx| {
            cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.clone()]));
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
        let mut cx = cx(dir.clone());
        let m = aegis_content::manifest::Manifest::read(&root.join(aegis_content::manifest::FILE)).unwrap();
        cx.app.manifest = m;
        cx.art = aegis_ui::art::Art::new(Some(root.clone()));
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
        let (mut cx, mut g) = (cx(dir.clone()), menu_gfx());
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
        let dir = std::env::temp_dir().join(format!("emberwake_askno_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut cx = cx(dir.clone());
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
}
