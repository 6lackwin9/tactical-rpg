//! Main menu, New game, Load / Save slots and Settings (scripts/ui/menus: main_menu_model, new_game_model,
//! save_slots_model, settings_model). Placeholder logic as in Godot; file import / export of saves is not ported yet.

use aegis_ui::audio::Bus;
use aegis_ui::canvas::{rgba, Rect};
use crate::music;
use crate::screens::{default_settings, enter_game, show_errors, title, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Glyph, Ui, Widget, BODY, DIM, GOLD, H, INFO, PANEL_HI, SMALL, TEXT, W};
use aegis_ui::screen::{Settings, FPS_CAPS, TEXT_SIZES, WINDOW_MODES, WINDOW_SIZES};
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

#[derive(Default)]
pub struct MainMenu {
    w: Vec<Widget<MenuAct>>,
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
                    cx.ask("Quit without saving?", |cx| cx.quit = true);
                } else {
                    cx.quit = true;
                }
            }
        }
    }
}

impl Screen<GameCx> for MainMenu {
    fn id(&self) -> &'static str {
        "main_menu"
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<MenuAct> = Ui::new(g, mouse, live);
        title(&mut ui, &cx.manifest.title, 78);
        ui.p.text_center(W as f32 / 2.0, 104.0, &cx.manifest.subtitle, BODY + 1.0, DIM);
        let mut entries: Vec<(&str, &str, MenuAct)> = Vec::new();
        if save::newest_slot(&cx.data, &cx.save_dir) >= 0 {
            entries.push(("menu.continue", "Continue", MenuAct::Continue));
        }
        entries.extend([
            ("menu.new_game", "New game", MenuAct::NewGame),
            ("menu.load", "Load game", MenuAct::Load),
            ("menu.settings", "Settings", MenuAct::Settings),
            ("menu.tools", "Tools (Forge, Dojo, Test Arena)", MenuAct::Tools),
            ("menu.quit", "Quit", MenuAct::Quit),
        ]);
        for (i, (id, label, a)) in entries.iter().enumerate() {
            let r = Rect::new(240, 139 + i as i32 * 24, 160, 18);
            ui.button_full(id, r, label, *a, true, false, BODY + 1.0, Vec::new());
        }
        ui.p.text_right(616.0, 309.0, "Version dev · Aegis engine", SMALL, DIM);
        self.w = ui.widgets;
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
                let cls = d.entry("classes", &s(e, "class_id"));
                let name = if shown.is_empty() { s_or(e, "name", unit_id) } else { shown };
                (name, s_or(cls, "name", &s(e, "class_id")), json::i(e, "level", 1))
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
                    cx.ask(q, move |cx| start_world(cx, &world));
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
            let lines = vec![(s_or(m, "name", &w), TEXT), (s(m, "tagline"), TEXT), (party_line(&Self::party_of(cx, &w)), TEXT)];
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
                } else if cx.has_game() && cx.is_dirty() {
                    let what = Self::describe(cx, slot);
                    let name = if slot == save::AUTOSAVE_SLOT { "the autosave".to_string() } else { format!("slot {slot}") };
                    cx.ask(format!("Load {name} ({what})? Unsaved progress in the current game is lost."), move |cx| load_slot(cx, slot));
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
                    cx.ask(format!("Overwrite slot {slot}{}?", if what.is_empty() { String::new() } else { format!(": {what}") }), move |cx| save_now(cx, slot));
                } else {
                    save_now(cx, slot);
                }
            }
            SlotAct::Delete(slot) => {
                if save::is_locked(&cx.save_dir, slot) {
                    return show_errors(cx, vec!["Unlock the save to delete it.".into()]);
                }
                let what = Self::describe(cx, slot);
                cx.ask(format!("Delete the save in slot {slot}{}?", if what.is_empty() { String::new() } else { format!(" ({what})") }), move |cx| {
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

    /// Why the running game can't be saved now ("" when it can).
    fn no_save(cx: &Cx) -> &'static str {
        if !cx.has_game() {
            "There is no game to save."
        } else if cx.busy() {
            "You can't save during a battle, a conversation or while a shop is open."
        } else {
            ""
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
            ui.p.panel(r, if hover { PANEL_HI } else { rgba(26, 36, 58, 250) }, if hover { GOLD } else { aegis_ui::ui::BORDER }, 2.0);
            let text_w = (r.w - 8 - if row.delete { 40 } else { 0 }) as f32;
            ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 3.0, &label, BODY, text_w, if ok { TEXT } else { DIM });
            ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 16.0, &status, BODY, text_w, if ok { INFO } else { DIM });
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
    Apply,
    Defaults,
    Close,
}

pub struct SettingsScreen {
    tab: usize,
    draft: Value,
    w: Vec<Widget<SetAct>>,
}

const TABS: [&str; 4] = ["Sound", "Display", "Gameplay", "Controls"];
/// The display choices are the engine's (aegis_ui::screen).
const CHOICES: [(&str, &[&str]); 3] = [("window_mode", &WINDOW_MODES), ("window_size", &WINDOW_SIZES), ("fps_cap", &FPS_CAPS)];
/// PLAYUI1-M: the region map's marker names (value, label); the first is the default.
pub const MAP_NAMES: [(&str, &str); 2] = [("hover", "On hover"), ("always", "Always")];
const CONTROLS: [(&str, &[(&str, &str)]); 3] = [
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
    ("Maps", &[("Move the party", "Arrow keys, W A S D"), ("Interact with the marker you face", "E"), ("Game menu", "Escape"), ("Choose a dialogue line", "1-9"), ("Party, journal, stash, help, event log", "P, J, I, ?, L")]),
    ("Battle", &[("Confirm the round", "Enter"), ("Step / next round", "Space"), ("Autoplay", "A"), ("Undo the hero's last step", "Z, right click"), ("Combat log", "L"), ("Screenshot", "F12")]),
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
                let (lo, hi) = if k.ends_with("speed") || k.ends_with("scale") { (0.5, 2.0) } else { (0.0, 100.0) };
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
            SetAct::Pick(k, i) => {
                if let Some(o) = choices(k).get(i) {
                    self.draft[k] = json!(o);
                }
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
        ui.p.fill(Rect::new(0, 0, W, H), rgba(0, 0, 0, 140));
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
        ui.p.panel(body, rgba(24, 26, 32, 255), rgba(24, 26, 32, 255), 1.0);
        let (x, mut y) = (body.x + 5, body.y + 6);
        let d = self.draft.clone();
        let pct = |k: &str| d[k].as_f64().unwrap_or(0.0);
        match self.tab {
            0 => {
                ui.checkbox("settings.mute", x + 3, y, "Mute", d["mute"].as_bool().unwrap_or(false), SetAct::Toggle("mute"), true);
                y += 20;
                for (k, label) in [("master", "Master volume"), ("music", "Music volume"), ("effects", "Effects volume"), ("interface", "Interface volume")] {
                    ui.p.text(x as f32, y as f32 + 2.0, label, BODY, TEXT);
                    let key = static_key(k);
                    ui.slider(&format!("settings.{k}"), Rect::new(x + 125, y, 200, 13), (pct(k) / VOLUME_STEP).round() as usize, 21, &format!("{}%", pct(k)), |i| SetAct::Set(key, i as f64 * VOLUME_STEP), true);
                    y += 18;
                }
            }
            1 => {
                for (k, label) in [("window_mode", "Window mode"), ("window_size", "Window size")] {
                    ui.p.text(x as f32, y as f32 + 3.0, label, BODY, TEXT);
                    choice(&mut ui, static_key(k), Rect::new(x + 125, y, 150, 16), d[k].as_str().unwrap_or(""));
                    y += 22;
                }
                ui.p.text(x as f32, y as f32, "In use", BODY, TEXT);
                ui.p.text(x as f32 + 125.0, y as f32, &format!("{} {}", d["window_mode"].as_str().unwrap_or(""), d["window_size"].as_str().unwrap_or("")), BODY, DIM);
                y += 18;
                ui.checkbox("settings.vsync", x + 3, y, "Vertical sync", d["vsync"].as_bool().unwrap_or(true), SetAct::Toggle("vsync"), true);
                y += 20;
                ui.p.text(x as f32, y as f32 + 3.0, "Frame rate cap", BODY, TEXT);
                choice(&mut ui, "fps_cap", Rect::new(x + 125, y, 150, 16), d["fps_cap"].as_str().unwrap_or(""));
                y += 24;
                ui.p.text(x as f32, y as f32 + 3.0, "Text size", BODY, TEXT);
                // The engine's four levels (Godot TEXT_LEVELS); a saved value off the levels shows the nearest one.
                let ts = pct("text_scale");
                let labels: Vec<String> = TEXT_SIZES.iter().map(|t| t.0.to_string()).collect();
                let at = TEXT_SIZES.iter().enumerate().min_by(|a, b| (a.1 .1 - ts).abs().total_cmp(&(b.1 .1 - ts).abs())).map(|(i, _)| i).unwrap_or(1);
                ui.select("settings.text_scale", Rect::new(x + 125, y, 150, 16), &labels, at, |i| SetAct::Set("text_scale", TEXT_SIZES[i].1), true, Vec::new());
            }
            2 => {
                ui.p.text(x as f32, y as f32 + 2.0, "Animation speed", BODY, TEXT);
                let a = pct("anim_speed");
                ui.slider("settings.anim_speed", Rect::new(x + 125, y, 200, 13), ((a - 0.5) / SCALE_STEP).round() as usize, 7, &format!("{a:.2}x"), |i| SetAct::Set("anim_speed", 0.5 + i as f64 * SCALE_STEP), true);
                ui.checkbox("settings.skip_anim", x + 300, y + 1, "Skip animations", d["skip_anim"].as_bool().unwrap_or(false), SetAct::Toggle("skip_anim"), true);
                y += 22;
                ui.checkbox("settings.show_tells", x + 3, y, "Show enemy tells", d["show_tells"].as_bool().unwrap_or(true), SetAct::Toggle("show_tells"), true);
                y += 18;
                ui.checkbox("settings.warn_missing", x + 3, y, "Warn about missing actions", d["warn_missing"].as_bool().unwrap_or(true), SetAct::Toggle("warn_missing"), true);
                y += 20;
                ui.p.text(x as f32, y as f32 + 3.0, "Map names", BODY, TEXT);
                let labels: Vec<String> = MAP_NAMES.iter().map(|m| m.1.to_string()).collect();
                let at = MAP_NAMES.iter().position(|m| d["map_names"].as_str() == Some(m.0)).unwrap_or(0);
                let t = tip("Map names", "On hover: a marker's name shows while you point at it, its legend row, or face it. Always: every person and exit is named.");
                ui.select("settings.map_names", Rect::new(x + 125, y, 150, 16), &labels, at, |i| SetAct::Pick("map_names", i), true, t);
            }
            _ => {
                for (head, rows) in CONTROLS {
                    ui.p.text(x as f32, y as f32, head, BODY + 2.0, GOLD);
                    y += 14;
                    for (what, keys) in rows {
                        ui.p.text(x as f32, y as f32, what, BODY, TEXT);
                        ui.p.text(x as f32 + 195.0, y as f32, keys, BODY, DIM);
                        y += 11;
                    }
                    y += 3;
                }
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

/// Volume sliders move in steps of 5 (0-100); speed and text-size sliders in quarters (0.5-2.0).
const VOLUME_STEP: f64 = 5.0;
const SCALE_STEP: f64 = 0.25;

fn choices(k: &str) -> &'static [&'static str] {
    CHOICES.iter().find(|c| c.0 == k).map(|c| c.1).unwrap_or(&[])
}

/// A settings choice (CHOICES) as a select with its option list.
fn choice(ui: &mut Ui<SetAct>, k: &'static str, r: Rect, current: &str) {
    let opts: Vec<String> = choices(k).iter().map(|o| o.to_string()).collect();
    let at = opts.iter().position(|o| o == current).unwrap_or(0);
    ui.select(&format!("settings.{k}"), r, &opts, at, |i| SetAct::Pick(k, i), true, Vec::new());
}

fn static_key(k: &str) -> &'static str {
    ["master", "music", "effects", "interface", "window_mode", "window_size", "fps_cap"].into_iter().find(|x| *x == k).unwrap_or("master")
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
        let clean = gs.to_value();
        let app = GameCx {
            data: Arc::new(data),
            manifest: Default::default(),
            game: Some(Game { gs, slot: -1, clean, busy: false, sample: false }),
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
}
