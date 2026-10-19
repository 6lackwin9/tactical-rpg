//! Main menu, New game, Load / Save slots and Settings (scripts/ui/menus: main_menu_model, new_game_model,
//! save_slots_model, settings_model). Placeholder logic as in Godot; file import / export of saves is not ported yet.

use aegis_ui::canvas::{rgba, Rect};
use crate::screens::{default_settings, enter_game, show_errors, title, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{Gfx, Ui, Widget, BODY, DIM, GOLD, H, INFO, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or};
use aegis_world::{new_game, progression, save};
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
    focus: usize,
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
        title(&mut ui, "Tactical RPG", 78);
        ui.p.text_center(W as f32 / 2.0, 104.0, "A small campaign of tactical battles.", BODY + 1.0, DIM);
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
        self.focus = self.focus.min(entries.len() - 1);
        for (i, (id, label, a)) in entries.iter().enumerate() {
            let r = Rect::new(240, 139 + i as i32 * 24, 160, 18);
            ui.button_full(id, r, label, *a, true, i == self.focus, BODY + 1.0, Vec::new());
        }
        ui.p.text_right(616.0, 309.0, "Version dev · Aegis engine", SMALL, DIM);
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        let n = self.w.iter().filter(|w| w.kind == "button").count().max(1);
        match k {
            Key::Up => self.focus = (self.focus + n - 1) % n,
            Key::Down | Key::Tab => self.focus = (self.focus + 1) % n,
            Key::Enter | Key::Space => {
                if let Some(a) = self.w.get(self.focus).and_then(|w| w.action) {
                    self.act(cx, a);
                }
            }
            _ => {}
        }
    }
    fn back(&mut self, _cx: &mut Cx) {}
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

#[derive(Debug, Clone, Copy)]
pub enum SlotAct {
    Load(i64),
    Save(i64),
    Delete(i64),
    Back,
}

pub struct Slots {
    saving: bool,
    w: Vec<Widget<SlotAct>>,
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
                } else if save::exists(&cx.save_dir, slot) {
                    let what = Self::describe(cx, slot);
                    cx.ask(format!("Overwrite slot {slot}{}?", if what.is_empty() { String::new() } else { format!(": {what}") }), move |cx| save_now(cx, slot));
                } else {
                    save_now(cx, slot);
                }
            }
            SlotAct::Delete(slot) => {
                let what = Self::describe(cx, slot);
                cx.ask(format!("Delete the save in slot {slot}{}?", if what.is_empty() { String::new() } else { format!(" ({what})") }), move |cx| {
                    if !save::delete(&cx.save_dir, slot) {
                        show_errors(cx, vec![format!("Slot {slot} could not be deleted.")]);
                    }
                });
            }
        }
    }
}

fn save_now(cx: &mut Cx, slot: i64) {
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
            ui.p.panel(r, rgba(26, 36, 58, 250), aegis_ui::ui::BORDER, 2.0);
            ui.p.text(r.x as f32 + 4.0, r.y as f32 + 3.0, &label, BODY, if ok { TEXT } else { DIM });
            ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 16.0, &status, BODY, 340.0, if ok { INFO } else { DIM });
            ui.push(format!("slot.{slot}"), "card", format!("{label}: {status}"), r, true, false, None, Vec::new());
            let mut bx = r.x + r.w - 62;
            if exists {
                ui.button(&format!("slot.{slot}.delete"), Rect::new(bx, r.y + 7, 56, 17), "Delete", SlotAct::Delete(slot), true);
                bx -= 60;
            }
            if self.saving && slot != save::AUTOSAVE_SLOT {
                ui.button(&format!("slot.{slot}.save"), Rect::new(bx, r.y + 7, 56, 17), "Save here", SlotAct::Save(slot), cx.has_game() && !cx.busy());
            } else if !self.saving && ok {
                ui.button(&format!("slot.{slot}.load"), Rect::new(bx, r.y + 7, 56, 17), "Load", SlotAct::Load(slot), true);
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
    Step(&'static str, f64),
    Cycle(&'static str),
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
const CHOICES: [(&str, &[&str]); 3] =
    [("window_mode", &["Windowed", "Borderless", "Fullscreen"]), ("window_size", &["1280x720", "1600x900", "1920x1080"]), ("fps_cap", &["Unlimited", "30", "60", "120", "144"])];
const CONTROLS: [(&str, &[(&str, &str)]); 3] = [
    (
        "Menus",
        &[
            ("Confirm / press the focused button", "Enter, Space"),
            ("Back / close the top panel", "Escape"),
            ("Focus the next control", "Tab"),
            ("Move focus", "Up, Down"),
            ("Back / close (mouse)", "Right click"),
        ],
    ),
    ("Maps", &[("Move the party", "Arrow keys, W A S D"), ("Interact with the marker you face", "E"), ("Game menu", "Escape"), ("Choose a dialogue line", "1-9")]),
    ("Battle", &[("Confirm the round", "Enter"), ("Step / next round", "Space"), ("Autoplay", "A"), ("Undo the hero's last step", "Z, right click"), ("Combat log", "L"), ("Screenshot", "F12")]),
];

impl SettingsScreen {
    pub fn new(cx: &Cx) -> Self {
        Self { tab: 0, draft: cx.settings.v.clone(), w: Vec::new() }
    }

    fn act(&mut self, cx: &mut Cx, a: SetAct) {
        match a {
            SetAct::Tab(t) => self.tab = t,
            SetAct::Toggle(k) => self.draft[k] = Value::Bool(!self.draft[k].as_bool().unwrap_or(false)),
            SetAct::Step(k, d) => {
                let (lo, hi) = if k.ends_with("speed") || k.ends_with("scale") { (0.5, 2.0) } else { (0.0, 100.0) };
                let v = (self.draft[k].as_f64().unwrap_or(0.0) + d).clamp(lo, hi);
                self.draft[k] = if lo == 0.0 { json!(v.round() as i64) } else { json!((v * 100.0).round() / 100.0) };
            }
            SetAct::Cycle(k) => {
                let opts = CHOICES.iter().find(|c| c.0 == k).map(|c| c.1).unwrap_or(&[]);
                let cur = self.draft[k].as_str().unwrap_or("");
                let i = opts.iter().position(|o| *o == cur).map(|i| (i + 1) % opts.len()).unwrap_or(0);
                self.draft[k] = json!(opts[i]);
            }
            SetAct::Apply => {
                cx.settings.v = self.draft.clone();
                if let Err(e) = cx.settings.save() {
                    show_errors(cx, vec![format!("Settings could not be saved: {e}")]);
                }
            }
            SetAct::Defaults => self.draft = default_settings(),
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
    fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<SetAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgba(0, 0, 0, 140));
        let r = Rect::new(95, 25, 450, 310);
        ui.frame(r, "Settings", false);
        ui.close_x("settings.close", Rect::new(r.x + r.w - 28, r.y + 9, 19, 18), SetAct::Close);
        for (i, t) in TABS.iter().enumerate() {
            ui.tab(&format!("settings.tab.{}", t.to_lowercase()), Rect::new(r.x + 12 + i as i32 * 46, r.y + 36, 44, 16), t, SetAct::Tab(i), self.tab == i);
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
                    ui.stepper(&format!("settings.{k}"), Rect::new(x + 125, y, 200, 13), (pct(k) / 100.0) as f32, &format!("{}%", pct(k)), SetAct::Step(static_key(k), -10.0), SetAct::Step(static_key(k), 10.0), true);
                    y += 18;
                }
            }
            1 => {
                for (k, label) in [("window_mode", "Window mode"), ("window_size", "Window size")] {
                    ui.p.text(x as f32, y as f32 + 3.0, label, BODY, TEXT);
                    ui.dropdown(&format!("settings.{k}"), Rect::new(x + 125, y, 150, 16), d[k].as_str().unwrap_or(""), SetAct::Cycle(static_key(k)), true, Vec::new());
                    y += 22;
                }
                ui.p.text(x as f32, y as f32, "In use", BODY, TEXT);
                ui.p.text(x as f32 + 125.0, y as f32, &format!("{} {}", d["window_mode"].as_str().unwrap_or(""), d["window_size"].as_str().unwrap_or("")), BODY, DIM);
                y += 18;
                ui.checkbox("settings.vsync", x + 3, y, "Vertical sync", d["vsync"].as_bool().unwrap_or(true), SetAct::Toggle("vsync"), true);
                y += 20;
                ui.p.text(x as f32, y as f32 + 3.0, "Frame rate cap", BODY, TEXT);
                ui.dropdown("settings.fps_cap", Rect::new(x + 125, y, 150, 16), d["fps_cap"].as_str().unwrap_or(""), SetAct::Cycle("fps_cap"), true, Vec::new());
                y += 24;
                ui.p.text(x as f32, y as f32 + 2.0, "Text size", BODY, TEXT);
                let ts = pct("text_scale");
                ui.stepper("settings.text_scale", Rect::new(x + 125, y, 200, 13), ((ts - 0.5) / 1.5) as f32, &format!("{ts:.2}x"), SetAct::Step("text_scale", -0.25), SetAct::Step("text_scale", 0.25), true);
            }
            2 => {
                ui.p.text(x as f32, y as f32 + 2.0, "Animation speed", BODY, TEXT);
                let a = pct("anim_speed");
                ui.stepper("settings.anim_speed", Rect::new(x + 125, y, 200, 13), ((a - 0.5) / 1.5) as f32, &format!("{a:.2}x"), SetAct::Step("anim_speed", -0.25), SetAct::Step("anim_speed", 0.25), true);
                ui.checkbox("settings.skip_anim", x + 300, y + 1, "Skip animations", d["skip_anim"].as_bool().unwrap_or(false), SetAct::Toggle("skip_anim"), true);
                y += 22;
                ui.checkbox("settings.show_tells", x + 3, y, "Show enemy tells", d["show_tells"].as_bool().unwrap_or(true), SetAct::Toggle("show_tells"), true);
                y += 18;
                ui.checkbox("settings.warn_missing", x + 3, y, "Warn about missing actions", d["warn_missing"].as_bool().unwrap_or(true), SetAct::Toggle("warn_missing"), true);
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

fn static_key(k: &str) -> &'static str {
    ["master", "music", "effects", "interface", "window_mode", "window_size", "fps_cap"].into_iter().find(|x| *x == k).unwrap_or("master")
}
