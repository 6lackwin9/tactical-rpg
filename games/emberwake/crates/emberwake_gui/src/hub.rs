//! The game menu over the map (hub_model.gd), the quest journal (journal_model.gd) and the battle results popup
//! (battle_results_model.gd). Read-only views over the game state, except the hub's doors (save, load, settings,
//! party, quit to the main menu).

use aegis_ui::pacing::{FrameNeed, SLOW_LIFE};
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgba, Rect};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_ui::audio::Bus;
use aegis_core::clock::Instant;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::state::{GameState, UnitState};
use emberwake_world::{mastery, progression, quest, roster, save};
use serde_json::{json, Value};
use crate::portraits::{Faces, Who};
use emberwake_battle::portrait::Look;

pub fn member_name(cx: &Cx, m: &UnitState) -> String {
    progression::display_name(&cx.data, m)
}

/// The member's class name in the world being played (HOOKS1 CN2: the world's `classes` overlay, else the class row's).
pub fn class_name(cx: &Cx, m: &UnitState) -> String {
    cx.data.class_name(&cx.gs().world_id, &m.class_id(&cx.data))
}

/// ROSTER1-U: member `m`'s HP as it takes the field in the played world, and whether it is wounded: (hp, max, wounded).
/// W2: a wounded member's max is cut by the act's row (`roster::field_unit`, the battle's own build); any other member
/// shows its stored HP (`progression::current_hp`), as before.
pub fn field_hp(cx: &Cx, m: &UnitState) -> (i64, i64, bool) {
    let world = cx.gs().world_id.clone();
    let wounded = m.has_condition(roster::WOUNDED);
    if wounded {
        if let Some(f) = roster::field_unit(&cx.data, cx.gs(), m, &world) {
            return (f.unit.hp as i64, f.unit.max_hp as i64, true);
        }
    }
    let (hp, max) = progression::current_hp(&cx.data, m, &world);
    (hp, max, wounded)
}

/// ROSTER1-U, BK1-BK2: the bunk chip ("Bunks 5/6"), only in a world with a `roster` block (None: today's screens).
pub fn bunks_text(gs: &GameState) -> Option<String> {
    gs.party.roster.has_block().then(|| format!("Bunks {}/{}", roster::bunks_used(&gs.party), roster::bunk_cap(gs)))
}

/// The hover text of a wounded mark (W2, W3): the field max against the stored max.
pub fn wounded_tip(cx: &Cx, m: &UnitState) -> String {
    let (_, full) = progression::current_hp(&cx.data, m, &cx.gs().world_id);
    let (_, field, _) = field_hp(cx, m);
    format!("Fights with {field} of {full} max HP until the party rests.")
}

/// PORTRAITFX1-R (6.2 row 2, surface `roster`): party member `m`'s portrait and the look its state rows read (the HP
/// its save carries, its ROSTER1 conditions: a wounded member desaturates). Keyed `member:<id>` (two members never
/// share a portrait's holds, whichever character they play); the chain and the initial are the member's
/// (`Who::of_member`). PK3 3.2 (PR7-11): a procedural member whose recruit template names `art` shows that file
/// (`roster::portrait_of`) before the chain, under the member's name.
pub fn member_portrait(d: &emberwake_world::Data, gs: &GameState, m: &UnitState) -> (Who, Look) {
    let mut who = Who::of_member(d, m);
    let art = emberwake_world::roster::portrait_of(d, m);
    if !art.is_empty() {
        let name = std::mem::take(&mut who.name);
        who = Who { name, ..Who::file(&art) };
    }
    who.key = format!("member:{}", m.id);
    (who, crate::portraits::look_of_member(d, gs, m))
}

/// The hub party list's portrait side (PORTRAITFX1 6.2: 20 px left of each name) and the indent of the row's lines.
pub const ROW_PORTRAIT: i32 = 20;
pub const ROW_INDENT: f32 = 22.0;

/// JournalModel.reward_lines: XP, items, passives, skills, gold.
pub fn reward_lines(cx: &Cx, rewards: &Dict) -> Vec<String> {
    let d = &cx.data;
    let mut out = Vec::new();
    let xp = json::i(rewards, "xp", 0);
    if xp > 0 {
        out.push(format!("{xp} XP"));
    }
    for (id, n) in json::ints(json::get(rewards, "items")) {
        out.push(format!("{} x{n}", s_or(d.entry("items", &id), "name", &format!("[{id}]"))));
    }
    for id in json::sget(rewards, "passives") {
        out.push(format!("Passive: {}", s_or(d.entry("passives", &id), "name", &format!("[{id}]"))));
    }
    for id in json::sget(rewards, "skills") {
        out.push(format!("Skill: {}", s_or(d.entry("skills", &id), "name", &format!("[{id}]"))));
    }
    let gold = json::i(rewards, "gold", 0);
    if gold > 0 {
        // TERMS1 TU3: the currency's word in the game's world ("12 gold", "12 Scrap")
        out.push(format!("{gold} {}", crate::hud::term_word(cx, emberwake_world::terms::CURRENCY)));
    }
    out
}

const GROUPS: [(&str, &str); 4] = [("ready", "Ready to turn in"), ("active", "Active"), ("blocked", "On hold"), ("completed", "Completed")];

/// Whether the journal has anything to show (the Hub's and the HUD bar's Journal buttons).
pub fn has_quests(cx: &Cx) -> bool {
    GROUPS.iter().any(|(st, _)| !quest::quests_with(&cx.data, cx.gs(), st).is_empty())
}

// ---- Hub -----------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HubAct {
    Party,
    Journal,
    Save,
    Load,
    Settings,
    Quit,
    Return,
}

/// The hub popup's frame.
pub const HUB_RECT: Rect = Rect::new(95, 25, 450, 310);
/// MENURAIL1 (PR7-30, LOOK1 M2): the left rail of the game menu (x inset from the frame, width, top, button height and
/// step) and the gap between the rail and the party box.
const RAIL_INSET: i32 = 10;
const RAIL_W: i32 = 130;
const RAIL_TOP: i32 = 80;
const RAIL_BTN_H: i32 = 20;
const RAIL_STEP: i32 = 26;
const RAIL_GAP: i32 = 12;

#[derive(Default)]
pub struct Hub {
    w: Vec<Widget<HubAct>>,
    /// PORTRAITFX1-R: the party list's portraits (surface `roster`), on the menu's own clock.
    faces: Faces,
}

impl Hub {
    fn reason(cx: &Cx, a: HubAct) -> String {
        match a {
            HubAct::Party if cx.busy() => "Party changes wait until the battle, conversation or shop is over.".into(),
            HubAct::Save if cx.busy() => "You can't save during a battle, a conversation or while a shop is open.".into(),
            HubAct::Journal if !has_quests(cx) => "No quests yet.".into(),
            HubAct::Load if cx.busy() => "Finish the fight, conversation or shop first.".into(),
            HubAct::Load if save::newest_slot(&cx.data, &cx.save_dir) < 0 => "There are no saves to load yet.".into(),
            HubAct::Quit if cx.busy() => "Finish the fight, conversation or shop first.".into(),
            _ => String::new(),
        }
    }

    fn act(&mut self, cx: &mut Cx, a: HubAct) {
        let why = Self::reason(cx, a);
        if !why.is_empty() {
            show_errors(cx, vec![why]);
            return;
        }
        match a {
            HubAct::Party => cx.nav.push(Nav::Push(Box::new(crate::party::Character::new(cx, "", "stats")))),
            HubAct::Journal => cx.nav.push(Nav::Push(Box::new(Journal::default()))),
            HubAct::Save => cx.nav.push(Nav::Push(Box::new(crate::menus::Slots::new(true)))),
            HubAct::Load => cx.nav.push(Nav::Push(Box::new(crate::menus::Slots::new(false)))),
            HubAct::Settings => cx.nav.push(Nav::Push(Box::new(crate::menus::SettingsScreen::in_game(cx)))),
            HubAct::Return => cx.nav.push(Nav::Pop),
            HubAct::Quit => quit_to_title(cx),
        }
    }
}

/// Back to the title (the hub's Quit to main menu, the in-game Settings' Return to title): not while a fight,
/// conversation or shop is open; it always asks first (No is the default), unsaved progress with the warning.
pub(crate) fn quit_to_title(cx: &mut Cx) {
    if cx.busy() {
        show_errors(cx, vec!["Finish the fight, conversation or shop first.".into()]);
        return;
    }
    let go = |cx: &mut Cx| {
        cx.end_game();
        cx.nav.push(Nav::Reset(Box::new(crate::menus::MainMenu::default())));
    };
    // PR7-41 (PT-35): always asked (a clean game asks too, without the warning)
    if cx.is_dirty() {
        cx.ask_no("Quit to the main menu without saving? Progress since your last save is lost.", go);
    } else {
        cx.ask_no("Quit to the main menu? Your game is saved.", go);
    }
}

impl Screen<GameCx> for Hub {
    /// At rest only the portrait moves, and slowly.
    fn frame_need(&self, _cx: &Cx) -> FrameNeed {
        FrameNeed::Within(SLOW_LIFE)
    }
    fn id(&self) -> &'static str {
        "hub"
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        crate::viewkeys::window_key(cx, k, crate::viewkeys::Shown::Notice);
    }
    fn overlay(&self) -> bool {
        true
    }
    /// STYLE-PASS1B (`PR7-48` (2)): a panel-shell screen; a panel opened over it takes its place until it closes.
    fn panel(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<HubAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("popup_veil", rgba(0, 0, 0, 110)));
        self.faces.begin();
        let gs = cx.gs().clone();
        let fr = HUB_RECT;
        ui.frame(fr, &cx.world_name(&gs.world_id), true);
        // MENURAIL1: the place over the party box
        let box_mid = fr.x + RAIL_INSET + RAIL_W + RAIL_GAP + (fr.w - 2 * RAIL_INSET - RAIL_W - RAIL_GAP) / 2;
        ui.p.text_center(box_mid as f32, 62.0, &cx.location_name(&gs.world_id, &gs.location_id), BODY + 1.0, th().dim);
        let active: Vec<String> = gs.party.active_members().iter().map(|m| m.id.clone()).collect();
        let mut rows: Vec<&UnitState> = gs.party.members.iter().filter(|m| active.contains(&m.id)).collect();
        rows.extend(gs.party.members.iter().filter(|m| !active.contains(&m.id)));
        // ROSTER1-U: the HP a member takes the field with (a wounded one's cut max, W2) and its mark; the box grows with
        // the widest line and takes a second column past four members (bunks above the party max, BK1); PORTRAITFX1-R: a
        // 20 px portrait at the row's left, its three lines indented 22 px (the column grows by the indent, so the name
        // fits 22 px shorter than the column and the class and HP lines keep their width)
        let step = 13f32.max(ui.p.line_h(BODY + 1.0).ceil());
        let mut row_h = (step * 3.0 + 3.0) as i32;
        let lines: Vec<(String, (i64, i64, bool))> = rows.iter().map(|m| (format!("{}  Lv {}", class_name(cx, m), m.level(&cx.data)), field_hp(cx, m))).collect();
        let mark = "Wounded";
        let gap = 6.0;
        let mut col_w = 100f32;
        for (class_line, (hp, max, wounded)) in &lines {
            let hp_w = ui.p.text_w(&format!("HP {hp} / {max}"), BODY + 1.0) + if *wounded { gap + ui.p.text_w(mark, BODY + 1.0) } else { 0.0 };
            col_w = col_w.max(ui.p.text_w(class_line, BODY + 1.0).max(hp_w) + 6.0);
        }
        let col_w = (col_w + ROW_INDENT).ceil() as i32;
        // PR7-26: a full roster at any text size keeps the menu below it inside the frame: past four members a second
        // column, and more columns (as many as the frame's width holds) while the box is taller than the room left
        // above the menu (three button rows and the detail line, over the frame's bottom edge); when the widest
        // columns are still too tall the rows go compact (name and HP; the class line moves to the portrait's card)
        // MENURAIL1: the party box takes the frame right of the rail, above the detail line at the frame's foot
        let lh = ui.p.line_h(BODY + 1.0).ceil() as i32;
        let bunks_h = if bunks_text(&gs).is_some() { step as i32 + 3 } else { 0 };
        let detail_y = fr.y + fr.h - 8 - lh;
        let area = Rect::new(fr.x + RAIL_INSET + RAIL_W + RAIL_GAP, RAIL_TOP, fr.w - 2 * RAIL_INSET - RAIL_W - RAIL_GAP, 0);
        let room = detail_y - 6 - bunks_h - RAIL_TOP;
        let widest = (area.w / col_w.max(1)).max(1) as usize;
        let fits = |cols: usize, row_h: i32| 4 + rows.len().div_ceil(cols) as i32 * row_h <= room;
        let mut cols = if rows.len() > 4 { 2 } else { 1 };
        let compact = !fits(widest.max(cols), row_h);
        if compact {
            row_h = (step * 2.0 + 3.0) as i32;
        }
        while cols < widest.min(rows.len().max(1)) && !fits(cols, row_h) {
            cols += 1;
        }
        let per_col = rows.len().div_ceil(cols).max(1);
        let hp_y = if compact { step } else { step * 2.0 };
        let bh = 4 + per_col as i32 * row_h;
        let bx = Rect::new(area.x + (area.w - cols as i32 * col_w) / 2, RAIL_TOP, cols as i32 * col_w, bh);
        crate::hud::hud_panel(&mut ui, bx, th().card, th().edge, 2.0);
        for (i, (m, (class_line, (hp, max, wounded)))) in rows.iter().zip(&lines).enumerate() {
            let x = (bx.x + (i / per_col) as i32 * col_w) as f32 + 3.0;
            let y = bx.y as f32 + 3.0 + (i % per_col) as f32 * row_h as f32;
            let on = active.contains(&m.id);
            let (who, look) = member_portrait(&cx.data, &gs, m);
            let pr = Rect::new(x as i32, y as i32, ROW_PORTRAIT, ROW_PORTRAIT);
            self.faces.draw(&mut ui.p, cx, &who, &look, "roster", pr);
            let card = if compact { tip(&member_name(cx, m), class_line) } else { Vec::new() };
            ui.push(format!("hub.portrait.{}", m.id), "portrait", member_name(cx, m), pr, true, false, None, card);
            let x = x + ROW_INDENT;
            ui.p.text_fit(x, y, &member_name(cx, m), BODY + 1.0, col_w as f32 - 6.0 - ROW_INDENT, if on { th().info } else { th().dim });
            if !compact {
                ui.p.text(x, y + step, class_line, BODY + 1.0, th().text);
            }
            let hp_line = format!("HP {hp} / {max}");
            ui.p.text(x, y + hp_y, &hp_line, BODY + 1.0, th().text);
            if *wounded {
                let mx = x + ui.p.text_w(&hp_line, BODY + 1.0) + gap;
                ui.p.text(mx, y + hp_y, mark, BODY + 1.0, th().foe);
                let r = Rect::new(mx as i32 - 1, (y + hp_y) as i32 - 1, ui.p.text_w(mark, BODY + 1.0) as i32 + 2, step as i32);
                ui.push(format!("hub.wounded.{}", m.id), "row", mark, r, true, false, None, tip(mark, &wounded_tip(cx, m)));
            }
        }
        if let Some(bunks) = bunks_text(&gs) {
            ui.p.text_center((bx.x + bx.w / 2) as f32, (bx.y + bh) as f32 + 3.0, &bunks, BODY + 1.0, th().dim);
        }
        let pts: i64 = gs.party.members.iter().map(|m| m.unspent(&cx.data).max(0)).sum();
        let entries = [
            (HubAct::Return, "hub.return", "Resume".to_string(), "Back to the map."),
            (HubAct::Party, "hub.party", if pts > 0 { format!("Party ({pts} point{} to spend)", if pts == 1 { "" } else { "s" }) } else { "Party".into() }, "Roster, level up, skills and gear."),
            (HubAct::Journal, "hub.journal", "Journal".into(), "Quests ready, active, on hold and completed."),
            (HubAct::Save, "hub.save", "Save game".into(), "Save to one of the manual slots."),
            (HubAct::Load, "hub.load", "Load game".into(), "Load a save; unsaved progress here is lost (you are asked first)."),
            (HubAct::Settings, "hub.settings", "Settings".into(), "Sound, display, gameplay and controls."),
            (HubAct::Quit, "hub.quit", "Quit to main menu".into(), "Back to the title (you are asked first; unsaved progress is lost)."),
        ];
        // MENURAIL1: the rail down the frame's left, Resume first
        let mut detail = "Back to the map.".to_string();
        for (i, (a, id, label, desc)) in entries.iter().enumerate() {
            let r = Rect::new(fr.x + RAIL_INSET, RAIL_TOP + i as i32 * RAIL_STEP, RAIL_W, RAIL_BTN_H);
            let why = Self::reason(cx, *a);
            ui.button_full(id, r, label, *a, true, false, BODY + 1.0, if why.is_empty() { tip(label, desc) } else { vec![(label.clone(), th().gold), (why.clone(), th().foe)] });
            if ui.hover(r) {
                detail = if why.is_empty() { desc.to_string() } else { why };
            }
        }
        ui.p.text_center((fr.x + fr.w / 2) as f32, detail_y as f32, &detail, BODY + 1.0, th().dim);
        self.w = ui.widgets;
    }
    fn resume(&mut self, _cx: &mut Cx) {}
    /// PORTRAITFX1 4.5: the portraits' clock moves by the frame's wall time (at most 100 ms a frame).
    fn tick(&mut self, _cx: &mut Cx) {
        self.faces.tick();
    }
    /// The agent link (4.11): the party list's portraits.
    fn state(&self, _cx: &Cx) -> Value {
        json!({"portraits": self.faces.state()})
    }
    widget_plumbing!();
}

// ---- Journal -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum JAct {
    Select(String),
    Back,
}

#[derive(Default)]
pub struct Journal {
    selected: String,
    w: Vec<Widget<JAct>>,
}

impl Journal {
    fn act(&mut self, cx: &mut Cx, a: JAct) {
        match a {
            JAct::Select(q) => self.selected = q,
            JAct::Back => cx.nav.push(Nav::Pop),
        }
    }

    fn shown(cx: &Cx) -> Vec<(String, String, Vec<String>)> {
        GROUPS
            .iter()
            .filter_map(|(st, label)| {
                let ids = quest::quests_with(&cx.data, cx.gs(), st);
                (!ids.is_empty()).then(|| (st.to_string(), label.to_string(), ids))
            })
            .collect()
    }
}

impl Screen<GameCx> for Journal {
    /// Nothing here moves by itself.
    fn frame_need(&self, _cx: &Cx) -> FrameNeed {
        FrameNeed::Idle
    }
    /// Up / Down pick the quest (the list's own keys).
    fn arrow_focus(&self) -> bool {
        false
    }
    fn id(&self) -> &'static str {
        "journal"
    }
    fn overlay(&self) -> bool {
        true
    }
    /// STYLE-PASS1B (`PR7-48` (2)): a panel-shell screen; a panel opened over it takes its place until it closes.
    fn panel(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let mut ui: Ui<JAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("popup_veil", rgba(0, 0, 0, 110)));
        let fr = Rect::new(16, 16, 608, 328);
        // PR7-76 (PT-62): the panel is opaque (the theme's panel fill is see-through; nothing of the map shows through)
        let fill = th().panel;
        ui.p.fill(Rect::new(fr.x + 2, fr.y + 2, fr.w - 4, fr.h - 4), [fill[0], fill[1], fill[2], 255]);
        ui.frame_in("ornate", fr, "Journal", false);
        let groups = Self::shown(cx);
        let all: Vec<String> = groups.iter().flat_map(|g| g.2.clone()).collect();
        if !all.contains(&self.selected) {
            self.selected = all.first().cloned().unwrap_or_default();
        }
        let mut y = 50;
        for (_, label, ids) in &groups {
            ui.p.text(25.0, y as f32, label, BODY + 1.0, th().info);
            y += 13;
            for q in ids {
                let name = s_or(&quest::resolved(&d, &gs, q), "name", q);
                ui.button_full(&format!("journal.quest.{}", q.replace(':', ".")), Rect::new(25, y, 130, 18), &format!("> {name}"), JAct::Select(q.clone()), true, *q == self.selected, BODY + 1.0, Vec::new());
                y += 22;
            }
        }
        if all.is_empty() {
            ui.p.text(25.0, 50.0, "No quests yet.", BODY + 1.0, th().dim);
        }
        if !self.selected.is_empty() {
            let q = quest::resolved(&d, &gs, &self.selected);
            let st = quest::status(&d, &gs, &self.selected);
            let (x, w) = (221, 390);
            let mut y = 50;
            ui.p.text(x as f32, y as f32, &s_or(&q, "name", &self.selected), BODY + 1.0, th().gold);
            y += 15;
            y = ui.para(x, y, w, &s(&q, "description"), BODY + 1.0, th().text) + 4;
            if st != "blocked" {
                let char_name = |r: &str| if r.is_empty() { String::new() } else { s_or(d.resolve(&gs.world_id, "characters", r), "name", &format!("[{r}]")) };
                let giver = char_name(&s(&q, "giver"));
                if !giver.is_empty() {
                    ui.p.text(x as f32, y as f32, &format!("From {giver}"), BODY + 1.0, th().dim);
                    y += 15;
                }
                let total = json::aget(&q, "objectives").len() as i64;
                let done = if st == "completed" { total } else { total.min(json::i(json::dict(gs.quests.get(&self.selected).unwrap_or(&Value::Null)), "objective", 0)) };
                ui.p.text(x as f32, y as f32, &format!("Progress {done} / {total}"), BODY + 1.0, th().dim);
                y += 15;
                let obj = s(&quest::current_objective(&d, &gs, &self.selected), "text");
                if !obj.is_empty() {
                    y = ui.para(x, y, w, &format!("Now: {obj}"), BODY + 1.0, th().good);
                }
                if st == "ready" {
                    let to = char_name(&s(&q, "turn_in"));
                    ui.p.text(x as f32, y as f32, &if to.is_empty() { "Ready to turn in.".to_string() } else { format!("Turn in to {to}.") }, BODY + 1.0, th().good);
                    y += 15;
                }
                let rewards = reward_lines(cx, json::dget(&q, "rewards"));
                if !rewards.is_empty() {
                    y = ui.para(x, y, w, &format!("Rewards: {}", rewards.join(", ")), BODY + 1.0, th().text);
                }
                if st == "active" || st == "ready" {
                    let offers: Vec<String> = quest::offers(&d, &gs, &self.selected).iter().map(|l| s_or(d.resolve(&gs.world_id, "loot", l), "name", &format!("[{l}]"))).collect();
                    let picks = quest::picks_allowed(&d, &gs, &self.selected);
                    if picks > 0 && !offers.is_empty() {
                        ui.para(x, y, w, &format!("Choose {picks} of: {}", offers.join(", ")), BODY + 1.0, th().text);
                    }
                }
            } else {
                ui.p.text(x as f32, y as f32, "On hold.", BODY + 1.0, th().dim);
            }
        }
        ui.button("journal.back", Rect::new(25, 318, 130, 18), "Back", JAct::Back, true);
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        let all: Vec<String> = Self::shown(cx).into_iter().flat_map(|g| g.2).collect();
        if all.is_empty() {
            return;
        }
        let at = all.iter().position(|q| *q == self.selected).unwrap_or(0) as i64;
        let step = match k {
            Key::Up => -1,
            Key::Down => 1,
            _ => return,
        };
        self.selected = all[(at + step).clamp(0, all.len() as i64 - 1) as usize].clone();
    }
    fn state(&self, _cx: &Cx) -> Value {
        json!({"selected": self.selected})
    }
    widget_plumbing!();
}

// ---- Battle results ------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum ResAct {
    LevelUp,
    Continue,
}

pub struct Results {
    award: Dict,
    advanced: Vec<String>,
    outcome: String,
    gold: i64,
    /// REWARD1: what the defeated enemies paid (`battle::apply`'s `rewards`).
    rewards: Dict,
    /// ENCOUNTER2 W4: one line per hero who stayed or stood (`battle::apply`'s `stay_lines`).
    stay_lines: Vec<String>,
    /// ROSTER1 3.2 (`battle::apply`'s `wounded` and `departed`): the members newly wounded and those who left
    /// (member, life).
    wounded: Vec<String>,
    departed: Vec<(String, String)>,
    /// INTENT2-U (A1.2-K3 "Stacks & Auto-Fill"): the pouch slots the post-battle auto-fill left short (`battle::apply`'s
    /// `stash_short`: "Stash short: Ash 1 of 3 Fire Flask").
    stash_short: Vec<String>,
    /// PK1-S1 (`battle::apply`'s `marks`): the marks the battle recorded on members (member, mark, value).
    marks: Vec<(String, String, i64)>,
    /// PR5-26 (audit B8; PK2-S1 `battle::apply`'s `tracks` and `stashed`): the unit tracks the marks moved (member,
    /// track, from, to) and the worn items a member grant sent to the stash (member, item).
    tracks: Vec<(String, String, i64, i64)>,
    stashed: Vec<(String, String)>,
    /// PR5-26 review F2 (`battle::apply`'s `cleared`): the slots left empty because their default is refused (member,
    /// slot).
    cleared: Vec<(String, String)>,
    /// PR7-3 (review N3; `battle::apply`'s `remains`, `mastery` and `wear`): what fallen members left in the stash
    /// (member, item), the weapon mastery gained by use, and the battle's wear on the weapon instances (the rows as the
    /// kit wrote them: member, slot, uid, item, worn, durability, dulled).
    remains: Vec<(String, String)>,
    mastery: Vec<mastery::Gain>,
    wear: Vec<Dict>,
    /// TRANSITIONS1 T2 (`over_battle`): the results stand over the battle they settled (the dimmed field under them,
    /// sliding in from `opened`; Continue fades to black, then closes both); Reduce motion (`plain`) drops the slide and
    /// shortens the fade; `before` is the hub's view as the fight started when a win arrives at the hub (`Arrival`).
    over: bool,
    plain: bool,
    before: Option<crate::transitions::Snapshot>,
    opened: Instant,
    leaving: Option<(Instant, ResAct)>,
    /// PR7-37 (PT-02, PT-24): the fight ended at its round limit (it reads "Out of time"), and the lines on what it
    /// leaves open (`open_jobs_lines`).
    timed_out: bool,
    next_lines: Vec<String>,
    /// PR7-83 (PT-73): an escape that got only some of the heroes it needed out (it reads "Partly escaped").
    partly: bool,
    /// OBJUI1 (OBJ1 3, MORALE1 3): the objective line ("Objective met: ..." or "The objective failed.") and, after a
    /// win by a break, the break lines (`obj_view::result_lines`; empty outside a battle).
    objective: Vec<String>,
    /// PR7-32 (BREPORT1-U): the kit's battle report of the fight (None outside a game battle: no report section).
    report: Option<emberwake_battle::report::Report>,
    w: Vec<Widget<ResAct>>,
}

/// PR7-37 (PT-02): the outcome label of a fight that ran out of rounds (whatever its `timeout_outcome` settles it as).
pub const OUT_OF_TIME: &str = "Out of time";
/// PR7-83 (PT-73): the head of an escape that got only some of the heroes it needed out.
pub const PARTLY_ESCAPED: &str = "Partly escaped";
/// ENRAGE2 EN9: the head of a random fight whose foes slipped away at its round cap.
pub const SLIPPED_AWAY: &str = "They slipped away";
/// ENRAGE2 EN9: its note.
pub const SLIPPED_NOTE: &str = "The foes got away before the fight was won. No rewards; the party keeps its wounds.";

/// PR7-37 (PT-02, PT-24): the results' and the battle banner's label of an outcome: "Out of time" at the round limit,
/// else Victory, Defeat or Fled.
pub fn outcome_head(outcome: &str, timed_out: bool) -> &str {
    match outcome {
        "won" => "Victory",
        emberwake_world::battle::ESCAPED => SLIPPED_AWAY,
        _ if timed_out => OUT_OF_TIME,
        "lost" => "Defeat",
        "fled" => "Fled",
        o => o,
    }
}

/// PR7-37 (PT-02, PT-24): the note of a fight that ran out of rounds, by whether a foe was defeated (`reward_text`'s
/// tally): no one fled, the round limit ended it.
pub fn timeout_note(rewards: &Dict) -> &'static str {
    if defeated_of(rewards) > 0 {
        "The round limit ran out before the fight was won. The party keeps the rewards of the foes it defeated."
    } else {
        "The round limit ran out before the fight was won. No foe was defeated, so no rewards."
    }
}

/// PR7-37 (PT-02, PT-24): what a fight not won leaves open: one line per job whose current step is to win it
/// (`quest::open_on_fight`): it can be tried again.
pub fn open_jobs_lines(cx: &Cx, encounter_ref: &str) -> Vec<String> {
    let (d, gs) = (&cx.data, cx.gs());
    quest::open_on_fight(d, gs, encounter_ref)
        .iter()
        .map(|q| format!("{} is still open: rest, then try the fight again.", s_or(&quest::resolved(d, gs, q), "name", q)))
        .collect()
}

/// REWARD1: the foes the rewards count as defeated (slain, surrendered, fled and withdrawn).
fn defeated_of(rewards: &Dict) -> i64 {
    let d = json::dget(rewards, "defeated");
    ["slain", "surrendered", "fled", "withdrawn"].iter().map(|k| json::i(d, k, 0)).sum()
}

/// How a party line of the results reads (its colour).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartyLine {
    Departed,
    Keepsake,
    Wounded,
}

/// ROSTER1-U: the results' party lines after the stay-behind lines (ROSTER1_TASKS 4, W4): one per departure ("Ash was
/// captured.", "Ash died."), then its keepsake when one was paid ("Keepsake: Ash's Locket, to the stash." KS1, KS2;
/// the departed record keeps the item), then one line for the newly wounded. Names come from the departed records
/// (the member has left `members`) or the members.
pub fn party_lines(d: &emberwake_world::Data, gs: &GameState, wounded: &[String], departed: &[(String, String)]) -> Vec<(String, PartyLine)> {
    let mut out = Vec::new();
    for (member, life) in departed {
        let rec = gs.party.departed.iter().rev().find(|x| x.member.id == *member);
        let name = rec.map_or_else(|| member.clone(), |x| progression::display_name(d, &x.member));
        let text = match life.as_str() {
            x if x == roster::CAPTURED => format!("{name} was captured."),
            x if x == roster::DEAD => format!("{name} died."),
            _ => format!("{name} left the party."),
        };
        out.push((text, PartyLine::Departed));
        if let Some(item) = rec.map(|x| x.keepsake.clone()).filter(|k| !k.is_empty()) {
            out.push((format!("Keepsake: {}, to the stash.", d.item_name(&item)), PartyLine::Keepsake));
        }
    }
    let names: Vec<String> = wounded.iter().map(|id| gs.party.member(id).map_or_else(|| id.clone(), |m| progression::display_name(d, m))).collect();
    if !names.is_empty() {
        out.push((format!("Wounded until the next rest: {}.", names.join(", ")), PartyLine::Wounded));
    }
    out
}

/// PK1-S1: the results' mark lines, one per marked member in first-mark order ("Ash is marked: calcify +1."); names
/// from the members (or the departed records), the mark ids as the kit records them.
pub fn mark_lines(d: &emberwake_world::Data, gs: &GameState, marks: &[(String, String, i64)]) -> Vec<String> {
    let mut order: Vec<&str> = Vec::new();
    for (m, _, _) in marks {
        if !order.contains(&m.as_str()) {
            order.push(m);
        }
    }
    order
        .into_iter()
        .map(|member| {
            let departed = || gs.party.departed.iter().rev().find(|x| x.member.id == member).map(|x| progression::display_name(d, &x.member));
            let name = gs.party.member(member).map(|m| progression::display_name(d, m)).or_else(departed).unwrap_or_else(|| member.to_string());
            let list: Vec<String> = marks.iter().filter(|(m, _, _)| m == member).map(|(_, k, v)| format!("{k} {}{v}", if *v > 0 { "+" } else { "" })).collect();
            format!("{name} is marked: {}.", list.join(", "))
        })
        .collect()
}

/// PR5-26 (audit B8; PK2-S1): the settle's lines: each track move ("Mara's Track rises to stage 2 (Two)." or "falls
/// to") and each worn item a member grant sent to the stash ("Mara can't wield Great Staff now: it went to the
/// stash."), then (review F2) each slot left empty because its default is refused ("Mara's main hand is empty: Mara
/// can't wield Iron Billhook now."). A member who left reads by its departed record.
pub fn settle_lines(d: &emberwake_world::Data, gs: &GameState, tracks: &[(String, String, i64, i64)], stashed: &[(String, String)], cleared: &[(String, String)]) -> Vec<String> {
    let name = |member: &str| {
        let departed = || gs.party.departed.iter().rev().find(|x| x.member.id == member).map(|x| progression::display_name(d, &x.member));
        gs.party.member(member).map(|m| progression::display_name(d, m)).or_else(departed).unwrap_or_else(|| member.to_string())
    };
    let mut out: Vec<String> = tracks
        .iter()
        .map(|(member, track, from, to)| {
            let row = d.cat.unit_tracks.get(track);
            let track_name = row.map(|t| t.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| track.clone());
            let stage = row.and_then(|t| t.stage(i32::try_from(*to).unwrap_or(0))).map(|st| format!(" ({})", st.name)).unwrap_or_default();
            let moves = if to > from { "rises to" } else { "falls to" };
            format!("{}'s {track_name} {moves} stage {to}{stage}.", name(member))
        })
        .collect();
    out.extend(stashed.iter().map(|(member, item)| format!("{} can't wield {} now: it went to the stash.", name(member), d.item_name(item))));
    // review F2: a slot left empty, its default refused ("Mara's main hand is empty: she can't wield Iron Billhook now.")
    out.extend(cleared.iter().map(|(member, slot)| {
        let default = gs.party.member(member).and_then(|m| m.default_item(d, &gs.world_id, slot)).map(|i| d.item_name(&i));
        let why = default.map(|i| format!(": {} can't wield {i} now", name(member))).unwrap_or_default();
        format!("{}'s {} is empty{why}.", name(member), slot.replace('_', " "))
    }));
    out
}

/// How a graft-era line of the results reads (its colour).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraftLine {
    Remains,
    Mastery,
    Wear,
    Dulled,
}

/// The text of the dulled penalty: the weapon power a dulled weapon loses (`DULLED_POWER_MULT`), "-50% weapon power".
pub fn dulled_penalty() -> String {
    format!("-{:.0}% weapon power", (1.0 - emberwake_rules::gear::DULLED_POWER_MULT) * 100.0)
}

/// PR7-3 (review N3): the results' graft-era lines after the settle lines: each remains a fallen member left ("Remains:
/// Bone Graft from Ash, to the stash."), each mastery gain by use (the kit's `mastery::note`: "Ash: sword mastery +10
/// (level 1)."), then each worn weapon ("Ash's Iron Sword wore 3: 7 left.") and, when it ended at 0, its dulled line
/// ("Ash's Iron Sword is dulled: -50% weapon power until repaired."). Nothing for a result without the keys.
pub fn graft_lines(d: &emberwake_world::Data, gs: &GameState, remains: &[(String, String)], gains: &[mastery::Gain], wear: &[Dict]) -> Vec<(String, GraftLine)> {
    let name = |member: &str| {
        let departed = || gs.party.departed.iter().rev().find(|x| x.member.id == member).map(|x| progression::display_name(d, &x.member));
        gs.party.member(member).map(|m| progression::display_name(d, m)).or_else(departed).unwrap_or_else(|| member.to_string())
    };
    let mut out: Vec<(String, GraftLine)> = remains.iter().map(|(member, item)| (format!("Remains: {} from {}, to the stash.", d.item_name(item), name(member)), GraftLine::Remains)).collect();
    out.extend(gains.iter().map(|g| (mastery::note(d, gs, g), GraftLine::Mastery)));
    for x in wear {
        let (who, item) = (name(&s(x, "member")), d.item_name(&s(x, "item")));
        out.push((format!("{who}'s {item} wore {}: {} left.", json::i(x, "worn", 0), json::i(x, "durability", 0)), GraftLine::Wear));
        if json::b(x, "dulled", false) {
            out.push((format!("{who}'s {item} is dulled: {} until repaired.", dulled_penalty()), GraftLine::Dulled));
        }
    }
    out
}

/// REWARD1 section 7: the tally line ("Foes defeated: 3 of 5 (2 slain, 1 surrendered)."), the outcome's note, and the
/// reward lines (share gold, objective bonus, drops merged by item, key loot, missed key loot); `name(kind, id)` gives
/// an item's, a loot entry's or a character's name; `currency` is the currency's short form in the battle's world
/// (TERMS1 TU2: "Gold", "Scrap").
pub fn reward_text(rewards: &Dict, outcome: &str, currency: &str, name: &dyn Fn(&str, &str) -> String) -> (String, String, Vec<String>) {
    let foes = json::i(rewards, "foes", 0);
    let d = json::dget(rewards, "defeated");
    let (slain, surr, fled) = (json::i(d, "slain", 0), json::i(d, "surrendered", 0), json::i(d, "fled", 0));
    // PR6-9: a retreat's withdrawn foe (the key only when there is one)
    let withdrawn = json::i(d, "withdrawn", 0);
    let defeated = slain + surr + fled + withdrawn;
    let tally = if foes == 0 {
        String::new()
    } else {
        let parts: Vec<String> = [(slain, "slain"), (surr, "surrendered"), (fled, "fled"), (withdrawn, "withdrawn")].iter().filter(|(n, _)| *n > 0).map(|(n, w)| format!("{n} {w}")).collect();
        if defeated == slain {
            format!("Foes defeated: {defeated} of {foes}.")
        } else {
            format!("Foes defeated: {defeated} of {foes} ({}).", parts.join(", "))
        }
    };
    let note = match (outcome, defeated > 0) {
        ("lost", true) => "The party was beaten, but keeps the rewards of the foes it defeated.",
        ("lost", false) => "The party was beaten before it defeated a foe. No rewards.",
        ("fled", true) => "The party got away with the rewards of the foes it defeated.",
        ("fled", false) => "The party got away. No foe was defeated, so no rewards.",
        _ => "",
    };
    let mut lines = Vec::new();
    // PR7-41 (PT-25): a fight that counts the captives it frees says how many got out (the kit's `survivors`)
    if let Some(sv) = rewards.get("survivors").and_then(Value::as_object) {
        let (lived, of) = (json::i(sv, "lived", 0), json::i(sv, "of", 0));
        lines.push(if lived == 0 { format!("Captives saved: none of {of}.") } else { format!("Captives saved: {lived} of {of}.") });
    }
    let gold = json::i(rewards, "gold", 0);
    if gold > 0 {
        lines.push(format!("{currency} +{gold}"));
    }
    let objective = json::dget(rewards, "objective");
    let mut bonus = Vec::new();
    if json::i(objective, "gold", 0) > 0 {
        bonus.push(format!("{currency} +{}", json::i(objective, "gold", 0)));
    }
    if !json::s(objective, "loot").is_empty() {
        bonus.push(name("loot", &json::s(objective, "loot")));
    }
    if !bonus.is_empty() {
        lines.push(format!("Objective bonus: {}", bonus.join(", ")));
    }
    let merged = merged_drops(rewards);
    if !merged.is_empty() {
        let parts: Vec<String> = merged.iter().map(|(item, n)| if *n > 1 { format!("{} x{n}", name("items", item)) } else { name("items", item) }).collect();
        lines.push(format!("{DROPPED}{}", parts.join(", ")));
    }
    let keys: Vec<String> = json::sget(rewards, "key_loot").iter().map(|k| name("loot", k)).collect();
    if !keys.is_empty() {
        lines.push(format!("Key loot: {}", keys.join(", ")));
    }
    for x in json::aget(rewards, "key_missed").iter().filter_map(Value::as_object) {
        lines.push(format!("Missed: {} (slay {} and win)", name("loot", &json::s(x, "loot")), name("characters", &json::s(x, "kill"))));
    }
    (tally, note.to_string(), lines)
}

/// PR7-32 (ART-18): the results panel's width, margin to the window, padding, row heights (a 32 px portrait, or the
/// compact row when the heroes would not fit), a number column's width, the XP bar's width, a drop tile's side, a
/// button's height.
const RES_W: i32 = 560;
const RES_MARGIN: i32 = 10;
const RES_PAD: i32 = 12;
const RES_ROW_H: i32 = 34;
const RES_ROW_COMPACT: i32 = 20;
const RES_COL_W: i32 = 52;
const RES_XP_W: i32 = 90;
const DROP_TILE: i32 = 16;
const RES_BUTTON_H: i32 = 18;
/// The rewards line the drop tiles replace (`reward_text`).
const DROPPED: &str = "Dropped: ";

/// A report column's header (BREPORT1 rule 5).
pub fn column_label(id: &str) -> &'static str {
    match id {
        "dealt" => "Damage",
        "taken" => "Taken",
        "healed" => "Healed",
        "effects" => "Effects",
        "combos" => "Combos",
        _ => "",
    }
}

/// The drops of `rewards`, one per item with its count, in drop order (as `reward_text` merges them).
pub fn merged_drops(rewards: &Dict) -> Vec<(String, i64)> {
    let mut merged: Vec<(String, i64)> = Vec::new();
    for x in json::aget(rewards, "drops").iter().filter_map(Value::as_object) {
        let (item, n) = (json::s(x, "item"), json::i(x, "count", 1));
        match merged.iter_mut().find(|m| m.0 == item) {
            Some(m) => m.1 += n,
            None => merged.push((item, n)),
        }
    }
    merged
}

impl Results {
    pub fn new(res: &Dict, outcome: &str, _fight: &Dict) -> Self {
        Self {
            award: json::dget(res, "xp").clone(),
            advanced: json::sget(res, "advanced"),
            outcome: outcome.to_string(),
            gold: json::i(res, "gold", 0),
            rewards: json::dget(res, "rewards").clone(),
            stay_lines: json::sget(res, "stay_lines"),
            wounded: json::sget(res, "wounded"),
            departed: json::aget(res, "departed").iter().filter_map(Value::as_object).map(|x| (s(x, "member"), s(x, "life"))).collect(),
            stash_short: json::sget(res, "stash_short"),
            marks: json::aget(res, "marks").iter().filter_map(Value::as_object).map(|x| (s(x, "member"), s(x, "mark"), json::i(x, "value", 0))).collect(),
            tracks: json::aget(res, "tracks").iter().filter_map(Value::as_object).map(|x| (s(x, "member"), s(x, "track"), json::i(x, "from", 0), json::i(x, "to", 0))).collect(),
            stashed: json::aget(res, "stashed").iter().filter_map(Value::as_object).map(|x| (s(x, "member"), s(x, "item"))).collect(),
            cleared: json::aget(res, "cleared").iter().filter_map(Value::as_object).map(|x| (s(x, "member"), s(x, "slot"))).collect(),
            remains: json::aget(res, "remains").iter().filter_map(Value::as_object).map(|x| (s(x, "member"), s(x, "item"))).collect(),
            mastery: json::aget(res, "mastery").iter().filter_map(Value::as_object).map(|x| mastery::Gain { member: s(x, "member"), family: s(x, "family"), from: json::i(x, "from", 0), to: json::i(x, "to", 0) }).collect(),
            wear: json::aget(res, "wear").iter().filter_map(Value::as_object).cloned().collect(),
            over: false,
            plain: false,
            before: None,
            opened: Instant::now(),
            leaving: None,
            timed_out: false,
            partly: false,
            next_lines: Vec::new(),
            objective: Vec::new(),
            report: None,
            w: Vec::new(),
        }
    }

    /// PR7-32: the fight's battle report (the per-hero table and the best moment).
    pub fn report(mut self, report: Option<emberwake_battle::report::Report>) -> Self {
        self.report = report;
        self
    }

    /// OBJUI1: the objective line and the break lines under it (`obj_view::result_lines`).
    pub fn objective(mut self, lines: Vec<String>) -> Self {
        self.objective = lines;
        self
    }

    /// PR7-37 (PT-02, PT-24): the fight ran out of rounds (`timed_out`), and the lines on what it leaves open.
    pub fn after_fight(mut self, timed_out: bool, next_lines: Vec<String>) -> Self {
        self.timed_out = timed_out;
        self.next_lines = next_lines;
        self
    }

    /// PR7-83 (PT-73): the fight was an escape that got only some of the heroes it needed out.
    pub fn partly_escaped(mut self, partly: bool) -> Self {
        self.partly = partly;
        self
    }

    /// The results' head (`outcome_head`, or "Partly escaped" for a partial escape).
    fn head(&self) -> &str {
        if self.partly && self.outcome == "fled" {
            PARTLY_ESCAPED
        } else {
            outcome_head(&self.outcome, self.timed_out)
        }
    }

    /// TRANSITIONS1 T2: the results over the battle that settled (`plain`: Reduce motion), with the hub's view as the
    /// fight started when a win arrives at the hub (None: no arrival).
    pub fn over_battle(mut self, plain: bool, before: Option<crate::transitions::Snapshot>) -> Self {
        self.over = true;
        self.plain = plain;
        self.before = before;
        self.opened = Instant::now();
        self
    }

    /// T2 step 3: how far Continue's fade has gone (None: not leaving).
    fn leave_ms(&self, now: Instant) -> Option<u64> {
        self.leaving.map(|(t, _)| now.saturating_duration_since(t).as_millis() as u64)
    }

    fn act(&mut self, cx: &mut Cx, a: ResAct) {
        // PR7-32 (PT-13): Level up now opens the character sheet over the results, so its Close comes back to them
        if let ResAct::LevelUp = a {
            if self.leaving.is_none() {
                self.level_up(cx);
            }
            return;
        }
        // TRANSITIONS1 T2 step 3: over a battle, Continue fades to black with the victory stinger first (a press during
        // the fade ends it); Level up now goes at once
        if self.over {
            if self.leaving.is_none() {
                self.leaving = Some((Instant::now(), a));
                if let ResAct::Continue = a {
                    if self.outcome == "won" {
                        cx.audio.play(Bus::Sfx, crate::transitions::VICTORY_STINGER);
                    }
                    return;
                }
            }
            self.leave(cx);
            return;
        }
        self.close(cx, a);
    }

    /// T2 steps 3-4: the fade is over: the results and the battle under them close, and a win's arrival waits for the
    /// hub (the region map shows it; the story stage drops it when a vignette plays next).
    fn leave(&mut self, cx: &mut Cx) {
        let Some((_, a)) = self.leaving else { return };
        cx.nav.push(Nav::Pop);
        if let Some(before) = self.before.take() {
            let arrival = crate::transitions::arrival(&cx.data, cx.gs(), &before);
            if let Some(g) = cx.game.as_mut() {
                g.arrival = Some(arrival);
            }
        }
        self.close(cx, a);
    }

    fn close(&mut self, cx: &mut Cx, _a: ResAct) {
        cx.nav.push(Nav::Pop);
    }

    /// PT-13: the character sheet's level-up tab on the first awarded member with points to spend, over the results.
    fn level_up(&mut self, cx: &mut Cx) {
        let first = cx.gs().party.members.iter().find(|m| self.award.contains_key(&m.id) && m.unspent(&cx.data) > 0).map(|m| m.id.clone()).unwrap_or_default();
        cx.nav.push(Nav::Push(Box::new(crate::party::Character::new(cx, &first, "level_up"))));
    }

    /// PR7-32: the table's rows: each party member with an award or a report row (party order), then the report's other
    /// rows (guests); each with its report row when there is one.
    fn heroes<'a>(&'a self, gs: &'a GameState) -> Vec<(Option<&'a UnitState>, Option<&'a emberwake_battle::report::Row>)> {
        let rows: &[emberwake_battle::report::Row] = self.report.as_ref().map_or(&[], |r| r.rows.as_slice());
        let mut out: Vec<(Option<&UnitState>, Option<&emberwake_battle::report::Row>)> = Vec::new();
        for m in &gs.party.members {
            let row = rows.iter().find(|r| !r.guest && r.key == m.id);
            if self.award.contains_key(&m.id) || row.is_some() {
                out.push((Some(m), row));
            }
        }
        for r in rows {
            if !out.iter().any(|(_, x)| x.is_some_and(|x| std::ptr::eq(x, r))) {
                out.push((None, Some(r)));
            }
        }
        out
    }
}

impl Screen<GameCx> for Results {
    /// Nothing here moves by itself, but the slide over the battle and Continue's fade (TRANSITIONS1 T2).
    fn frame_need(&self, _cx: &Cx) -> FrameNeed {
        let sliding = !self.plain && self.opened.elapsed().as_millis() as u64 <= crate::transitions::RESULTS_SLIDE_MS;
        if self.over && (sliding || self.leaving.is_some()) {
            FrameNeed::Full
        } else {
            FrameNeed::Idle
        }
    }
    fn tick(&mut self, cx: &mut Cx) {
        if self.leave_ms(Instant::now()).is_some_and(|ms| ms >= crate::transitions::out_ms(self.plain)) {
            self.leave(cx);
        }
    }
    fn id(&self) -> &'static str {
        "battle_results"
    }
    fn overlay(&self) -> bool {
        true
    }
    /// STYLE-PASS1B (`PR7-48` (2)): a panel-shell screen; a panel opened over it takes its place until it closes.
    fn panel(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let mut ui: Ui<ResAct> = Ui::new(g, mouse, live);
        // TRANSITIONS1 T2 step 2: over the battle the field shows dimmed under the results, which slide in
        let now = Instant::now();
        let shift = if self.over { crate::transitions::results_shift(now.saturating_duration_since(self.opened).as_millis() as u64, self.plain) } else { 0 };
        if self.over {
            ui.p.fill(Rect::new(0, 0, W, H), th().game("result_field_veil", rgba(10, 14, 24, 200)));
        } else {
            ui.p.fill(Rect::new(0, 0, W, H), th().game("result_veil", rgba(10, 14, 24, 235)));
        }
        let head = self.head().to_string();
        let world = gs.world_id.clone();
        let name = |kind: &str, id: &str| -> String {
            match kind {
                "items" => d.item_name(id),
                _ => s_or(d.resolve(&world, kind, id), "name", id),
            }
        };
        let currency = emberwake_world::terms::short(&d, &world, emberwake_world::terms::CURRENCY);
        let (tally, mut note, reward_lines) = reward_text(&self.rewards, &self.outcome, &currency, &name);
        if self.outcome == emberwake_world::battle::ESCAPED {
            note = SLIPPED_NOTE.to_string();
        } else if self.timed_out {
            note = timeout_note(&self.rewards).to_string();
        }
        // the text lines above the table, in order, with their inks
        let mut lines: Vec<(String, aegis_ui::canvas::Rgba)> = Vec::new();
        for (text, col) in [(&tally, th().text), (&note, th().dim)] {
            if !text.is_empty() {
                lines.push((text.clone(), col));
            }
        }
        // OBJUI1 (OBJ1 3, MORALE1 3): the objective line (gold when met), then a break win's lines
        for (k, text) in self.objective.iter().enumerate() {
            let col = if k > 0 { th().info } else if text == emberwake_battle::obj_view::FAILED_LINE { th().dim } else { th().gold };
            lines.push((text.clone(), col));
        }
        // PR7-37 (PT-24): what happens next; ENCOUNTER2 W4: who stayed behind
        lines.extend(self.next_lines.iter().map(|t| (t.clone(), th().gold)));
        lines.extend(self.stay_lines.iter().map(|t| (t.clone(), th().info)));
        // ROSTER1-U: who left (and the keepsake they left), who came home wounded
        for (text, kind) in party_lines(&d, &gs, &self.wounded, &self.departed) {
            let col = match kind {
                PartyLine::Departed => th().foe,
                PartyLine::Keepsake => th().gold,
                PartyLine::Wounded => th().dim,
            };
            lines.push((text, col));
        }
        // INTENT2-U, PK1-S1, PR5-26, PR7-3: stash short, marks, tracks and stashed items, remains, mastery and wear
        lines.extend(self.stash_short.iter().map(|t| (t.clone(), th().dim)));
        lines.extend(mark_lines(&d, &gs, &self.marks).into_iter().chain(settle_lines(&d, &gs, &self.tracks, &self.stashed, &self.cleared)).map(|t| (t, th().foe)));
        for (text, kind) in graft_lines(&d, &gs, &self.remains, &self.mastery, &self.wear) {
            let col = match kind {
                GraftLine::Remains => th().gold,
                GraftLine::Mastery => th().good,
                GraftLine::Wear => th().dim,
                GraftLine::Dulled => th().foe,
            };
            lines.push((text, col));
        }
        // the rewards: the drops as I2 tiles (ART-18), the other lines as text
        let drops = merged_drops(&self.rewards);
        let mut other_rewards: Vec<String> = reward_lines.into_iter().filter(|l| !l.starts_with(DROPPED)).collect();
        if !self.advanced.is_empty() {
            let names: Vec<String> = self.advanced.iter().map(|q| s_or(&quest::resolved(&d, &gs, q), "name", q)).collect();
            other_rewards.push(format!("Quests advanced: {}", names.join(", ")));
        }
        if other_rewards.is_empty() && drops.is_empty() {
            other_rewards.push("No other rewards.".into());
        }
        let heroes = self.heroes(&gs);
        let prompt = heroes.iter().any(|(m, _)| m.is_some_and(|m| self.award.get(&m.id).and_then(Value::as_object).is_some_and(|a| json::i(a, "new_level", 0) > json::i(a, "old_level", 0)) && m.unspent(&d) > 0));
        let columns: Vec<&str> = self.report.as_ref().map(|r| r.columns.iter().copied().filter(|c| *c != "name").collect()).unwrap_or_default();
        let best = self.report.as_ref().map(|r| r.best.clone()).unwrap_or_default();
        // ART-18: the panel is sized to its content (the rows compact when six or more heroes would not fit)
        let line = (ui.p.line_h(BODY + 1.0) + 3.0).ceil() as i32;
        let small = ui.p.line_h(SMALL).ceil() as i32 + 2;
        let head_h = (ui.p.line_h(aegis_ui::ui::HEAD + 4.0)).ceil() as i32 + 6;
        let fixed = RES_PAD + head_h + lines.len() as i32 * line + small + if best.is_empty() { 0 } else { line + 2 } + small + if drops.is_empty() { 0 } else { DROP_TILE + 4 } + other_rewards.len() as i32 * line + 8 + RES_BUTTON_H + RES_PAD;
        let room = H - 2 * RES_MARGIN;
        let mut row_h = RES_ROW_H;
        if fixed + heroes.len() as i32 * row_h > room {
            row_h = RES_ROW_COMPACT;
        }
        let ph = (fixed + heroes.len() as i32 * row_h).min(room);
        let panel = Rect::new((W - RES_W) / 2, RES_MARGIN + (room - ph) / 2, RES_W, ph);
        // PT-11: the slide-in stops short of the window's edge, so the panel is never cut by it
        let shift = shift.min(W - 4 - (panel.x + panel.w)).max(0);
        ui.p.shift(shift, 0);
        // F2 (ART-02, ART-18): the ornate tier over the panel's fill
        ui.p.fill(panel, th().game("result_panel", rgba(10, 11, 16, 235)));
        ui.frame_body("ornate", panel);
        let x0 = panel.x + RES_PAD;
        let mut y = panel.y + RES_PAD;
        ui.p.text_face(aegis_ui::painter::Typeface::Display, x0 as f32, y as f32, &head, aegis_ui::ui::HEAD + 4.0, th().gold);
        y += head_h;
        for (text, col) in &lines {
            ui.p.text_fit(x0 as f32, y as f32, text, BODY + 1.0, (panel.w - 2 * RES_PAD) as f32, *col);
            y += line;
        }
        // the table's header: the report's columns over their numbers
        let num_x = |k: usize| panel.x + panel.w - RES_PAD - (columns.len() - k) as i32 * RES_COL_W;
        ui.p.text(x0 as f32, y as f32, "Party", SMALL, th().dim);
        for (k, c) in columns.iter().enumerate() {
            ui.p.text_right((num_x(k) + RES_COL_W) as f32, y as f32, column_label(c), SMALL, th().dim);
        }
        y += small;
        let face = row_h - 2;
        for (i, (m, row)) in heroes.iter().enumerate() {
            let r = Rect::new(x0, y, panel.w - 2 * RES_PAD, row_h);
            if i % 2 == 1 {
                ui.p.fill(r, rgba(255, 255, 255, 8));
            }
            // a 32 px portrait (ART-18), the member's art or a well with the initial
            let pr = Rect::new(x0, y + 1, face, face);
            let label = m.map(|m| member_name(cx, m)).or_else(|| row.map(|r| r.name.clone())).unwrap_or_default();
            let px = ui.icon_px(pr);
            // PT-59: a guest's row (no member) by its catalog unit's portrait
            let who = m.map(|m| Who::of_member(&d, m)).or_else(|| row.filter(|r| !r.unit.is_empty()).map(|r| Who::of_catalog(&d, &r.unit, &r.name)));
            let img = who.and_then(|w| cx.art.first(&w.chain(""), px, px).cloned());
            let initial: String = label.chars().take(1).collect();
            aegis_ui::panes::icon_tile(&mut ui, pr, img.as_ref(), &initial);
            let tx = pr.x + face + 6;
            let state = row.map(|r| r.state).filter(|st| *st != "standing").map(|st| format!(" ({st})")).unwrap_or_default();
            let guest = if row.is_some_and(|r| r.guest) { " (guest)" } else { "" };
            ui.p.text_fit(tx as f32, (y + 2) as f32, &format!("{label}{guest}{state}"), BODY + 1.0, (num_x(0) - tx - 6) as f32, th().text);
            if let Some((m, a)) = m.and_then(|m| self.award.get(&m.id).and_then(Value::as_object).map(|a| (m, a))) {
                let level = m.level(&d);
                let base = progression::xp_for_level(&d, level);
                let span = progression::xp_for_level(&d, level + 1) - base;
                let fill = if span <= 0 { 1.0 } else { ((m.total_xp - base) as f32 / span as f32).clamp(0.0, 1.0) };
                let (old, new) = (json::i(a, "old_level", 0), json::i(a, "new_level", 0));
                let xp_line = if new > old { format!("+{} XP  Level {old} to {new}", json::i(a, "xp", 0)) } else { format!("+{} XP  Level {level}", json::i(a, "xp", 0)) };
                let ly = y + 2 + ui.p.line_h(BODY + 1.0).ceil() as i32;
                if row_h >= RES_ROW_H {
                    ui.p.text(tx as f32, ly as f32, &xp_line, SMALL, if new > old { th().gold } else { th().dim });
                }
                // the XP bar: amber on basalt (ART-18)
                let bar = Rect::new(tx, if row_h >= RES_ROW_H { y + row_h - 7 } else { ly }, RES_XP_W, 4);
                ui.p.fill(bar, th().game("xp_track", aegis_ui::canvas::rgb(30, 28, 27)));
                ui.p.fill(Rect::new(bar.x, bar.y, (RES_XP_W as f32 * fill) as i32, bar.h), crate::tokens::accent());
                let to_next = format!("{} to next", progression::xp_to_next(&d, m.total_xp));
                ui.push(format!("results.xp.{}", m.id), "label", xp_line.clone(), Rect::new(bar.x, bar.y - 2, bar.w, bar.h + 4), true, false, None, tip(&xp_line, &to_next));
            }
            if let Some(row) = row {
                let tips = [("dealt", emberwake_battle::report::dealt_tip(row)), ("taken", emberwake_battle::report::taken_tip(row)), ("healed", emberwake_battle::report::healed_tip(row)), ("effects", emberwake_battle::report::effects_tip(row)), ("combos", emberwake_battle::report::combos_tip(row))];
                for (k, c) in columns.iter().enumerate() {
                    let n = row.number(c);
                    let text = if n == 0 { "-".to_string() } else { n.to_string() };
                    let cr = Rect::new(num_x(k), y + 2, RES_COL_W, ui.p.line_h(BODY + 1.0).ceil() as i32);
                    ui.p.text_right((cr.x + cr.w) as f32, cr.y as f32, &text, BODY + 1.0, if n == 0 { th().dim } else { th().text });
                    let about = tips.iter().find(|(id, _)| id == c).map(|(_, t)| t.clone()).unwrap_or_default();
                    let card = if about.is_empty() { Vec::new() } else { tip(&format!("{label}: {} {text}", column_label(c)), &about) };
                    ui.push(format!("results.report.{i}.{c}"), "label", text, cr, true, false, None, card);
                }
            }
            y += row_h;
        }
        // rule 6: the best-moment line
        if !best.is_empty() {
            y += 2;
            ui.p.text_fit(x0 as f32, y as f32, &format!("Best moment: {best}"), BODY + 1.0, (panel.w - 2 * RES_PAD) as f32, th().gold);
            ui.push("results.best", "label", format!("Best moment: {best}"), Rect::new(x0, y, panel.w - 2 * RES_PAD, line), true, false, None, Vec::new());
            y += line;
        }
        ui.p.text(x0 as f32, y as f32, "Rewards", SMALL, th().dim);
        y += small;
        if !drops.is_empty() {
            let mut x = x0;
            for (item, n) in &drops {
                let tr = Rect::new(x, y, DROP_TILE, DROP_TILE);
                let px = ui.icon_px(tr);
                let img = cx.art.first(&crate::tokens::item_icons(&d, item), px, px).cloned();
                aegis_ui::panes::icon_tile(&mut ui, tr, img.as_ref(), "");
                let label = if *n > 1 { format!("{} x{n}", d.item_name(item)) } else { d.item_name(item) };
                let end = ui.p.text(x as f32 + DROP_TILE as f32 + 3.0, y as f32 + (DROP_TILE as f32 - ui.p.line_h(BODY + 1.0)) / 2.0, &label, BODY + 1.0, th().text);
                ui.push(format!("results.drop.{item}"), "label", label.clone(), Rect::new(x, y, end.ceil() as i32 - x, DROP_TILE), true, false, None, tip(&label, &d.item_name(item)));
                x = end.ceil() as i32 + 10;
            }
            y += DROP_TILE + 4;
        }
        for l in other_rewards {
            ui.p.text_fit(x0 as f32, y as f32, &l, BODY + 1.0, (panel.w - 2 * RES_PAD) as f32, if l.starts_with("No other") || l.starts_with("Missed") { th().dim } else { th().text });
            y += line;
        }
        let by = panel.y + panel.h - RES_PAD - RES_BUTTON_H;
        if prompt {
            ui.button("results.level_up", Rect::new(x0, by, 90, RES_BUTTON_H), "Level up now", ResAct::LevelUp, true);
        }
        ui.button_full("results.continue", Rect::new(panel.x + panel.w - RES_PAD - 76, by, 76, RES_BUTTON_H), "Continue", ResAct::Continue, true, true, BODY + 1.0, Vec::new());
        let _ = y;
        ui.p.shift(-shift, 0);
        // T2 step 3: Continue's fade to black
        if let Some(ms) = self.leave_ms(now) {
            crate::transitions::draw_fade(&mut ui.p, crate::transitions::out_alpha(ms, self.plain) as f32 / 255.0);
        }
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        if matches!(k, Key::Enter | Key::Space) {
            self.act(cx, ResAct::Continue);
        }
    }
    fn state(&self, _cx: &Cx) -> Value {
        let mut out = json!({"outcome": self.outcome, "head": self.head(), "next": self.next_lines, "xp": self.award, "advanced": self.advanced, "gold": self.gold, "rewards": self.rewards, "stay_lines": self.stay_lines, "wounded": self.wounded, "departed": self.departed, "stash_short": self.stash_short});
        // TRANSITIONS1 T2: only over a battle (the state of the other results reads as before)
        if self.over {
            out["over_battle"] = json!({"plain": self.plain, "arrives": self.before.is_some(), "leaving": self.leave_ms(Instant::now())});
        }
        // OBJUI1: only after a battle (the state of the other results reads as before)
        if !self.objective.is_empty() {
            out["objective"] = json!(self.objective);
        }
        // PR7-32: the battle report, only after a game battle
        if let Some(r) = &self.report {
            out["report"] = r.to_json();
        }
        // PK1-S1: only when the battle left marks (the state of a battle without them reads as before)
        if !self.marks.is_empty() {
            out["marks"] = json!(self.marks.iter().map(|(m, k, v)| json!({"member": m, "mark": k, "value": v})).collect::<Vec<_>>());
        }
        // PR5-26 (PK2-S1): only when the settle moved a track or stashed an item
        if !self.tracks.is_empty() {
            out["tracks"] = json!(self.tracks.iter().map(|(m, t, f, to)| json!({"member": m, "track": t, "from": f, "to": to})).collect::<Vec<_>>());
        }
        if !self.stashed.is_empty() {
            out["stashed"] = json!(self.stashed.iter().map(|(m, i)| json!({"member": m, "item": i})).collect::<Vec<_>>());
        }
        if !self.cleared.is_empty() {
            out["cleared"] = json!(self.cleared.iter().map(|(m, slot)| json!({"member": m, "slot": slot})).collect::<Vec<_>>());
        }
        // PR7-3 (review N3): only when the settle left remains, gained mastery or wore a weapon
        if !self.remains.is_empty() {
            out["remains"] = json!(self.remains.iter().map(|(m, i)| json!({"member": m, "item": i})).collect::<Vec<_>>());
        }
        if !self.mastery.is_empty() {
            out["mastery"] = json!(self.mastery.iter().map(|g| json!({"member": g.member, "family": g.family, "from": g.from, "to": g.to, "level": g.level()})).collect::<Vec<_>>());
        }
        if !self.wear.is_empty() {
            out["wear"] = json!(self.wear);
        }
        out
    }
    widget_plumbing!();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PR7-76 (PT-62): the Journal's panel is opaque under the game's theme (whose panel colour is see-through): drawn
    /// over a white and over a black screen, a spot inside it (away from its texts) reads the same.
    #[test]
    fn pt62_the_journal_panel_is_opaque() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        cx.art = aegis_ui::art::Art::new(Some(root.to_path_buf()));
        let theme = aegis_ui::theme::load(cx.art.layers()).expect("the game's theme");
        assert!(theme.panel[3] < 255, "the game's panel colour lets the screen through (what the Journal must not)");
        aegis_ui::theme::set_theme(theme);
        let spot = |cx: &mut Cx, under: [u8; 4]| {
            let mut g = crate::hud::hud_tests::gfx();
            for px in g.canvas.px.chunks_mut(4) {
                px.copy_from_slice(&under);
            }
            let mut j = Journal::default();
            j.draw(cx, &mut g, (-500, -500), true);
            let (x, y) = (g.view.x(560.0) as usize, g.view.y(300.0) as usize);
            let at = (y * g.canvas.w as usize + x) * 4;
            g.canvas.px[at..at + 3].to_vec()
        };
        let (light, dark) = (spot(&mut cx, [255, 255, 255, 255]), spot(&mut cx, [0, 0, 0, 255]));
        aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
        assert_eq!(light, dark, "nothing shows through");
    }

    fn name(kind: &str, id: &str) -> String {
        match (kind, id) {
            ("items", "wpn_shortbow") => "Shortbow".into(),
            ("items", "con_heal_potion") => "Healing Potion".into(),
            ("loot", "ashlands:warchiefs_axe") => "Warchief's Axe".into(),
            ("loot", "ashlands:keep_ring") => "Keep Ring".into(),
            ("characters", "ash_warchief") => "Horde Warchief".into(),
            _ => id.into(),
        }
    }

    fn rewards(slain: i64, surr: i64, fled: i64, extra: Value) -> Dict {
        let mut r = json::obj([("foes", json!(5)), ("defeated", json!({"slain": slain, "surrendered": surr, "fled": fled})), ("gold", json!(0))]);
        for (k, v) in extra.as_object().unwrap() {
            r.insert(k.clone(), v.clone());
        }
        r
    }

    /// PR7-37 (PT-02): a fight that ran out of rounds heads "Out of time", not "Defeat", and its note says the
    /// round limit ended it, with or without the rewards of foes already down; a win stays a win.
    #[test]
    fn a_timed_out_fight_reads_out_of_time() {
        assert_eq!(outcome_head("lost", true), OUT_OF_TIME);
        assert_eq!(outcome_head("escaped", true), SLIPPED_AWAY, "ENRAGE2 EN9: a random fight's foes slipped away");
        assert_eq!(outcome_head("", true), OUT_OF_TIME);
        assert_eq!(outcome_head("lost", false), "Defeat");
        assert_eq!(outcome_head("won", true), "Victory");
        assert!(timeout_note(&rewards(0, 0, 0, json!({}))).contains("No foe was defeated"));
        assert!(timeout_note(&rewards(1, 0, 0, json!({}))).contains("keeps the rewards"));
    }

    /// PR7-83 (PT-73): a partial escape heads "Partly escaped"; a whole flight stays "Fled", and the flag reads only on a
    /// flight.
    #[test]
    fn a_partial_escape_reads_partly_escaped() {
        let res = json::obj([("ok", json!(true))]);
        assert_eq!(Results::new(&res, "fled", &Dict::new()).partly_escaped(true).head(), PARTLY_ESCAPED);
        assert_eq!(Results::new(&res, "fled", &Dict::new()).head(), "Fled");
        assert_eq!(Results::new(&res, "won", &Dict::new()).partly_escaped(true).head(), "Victory");
    }

    /// REWARD1 section 7: the tally line, each outcome's note, and the reward lines in order.
    #[test]
    fn the_results_say_what_the_defeated_foes_paid() {
        let (tally, note, lines) = reward_text(&rewards(2, 1, 0, json!({})), "lost", "Gold", &name);
        assert_eq!(tally, "Foes defeated: 3 of 5 (2 slain, 1 surrendered).");
        assert_eq!(note, "The party was beaten, but keeps the rewards of the foes it defeated.");
        assert!(lines.is_empty());
        assert_eq!(reward_text(&rewards(5, 0, 0, json!({})), "won", "Gold", &name).0, "Foes defeated: 5 of 5.");
        assert_eq!(reward_text(&rewards(0, 0, 1, json!({})), "fled", "Gold", &name).0, "Foes defeated: 1 of 5 (1 fled).");
        assert_eq!(reward_text(&rewards(0, 0, 0, json!({})), "lost", "Gold", &name).1, "The party was beaten before it defeated a foe. No rewards.");
        assert_eq!(reward_text(&rewards(1, 0, 0, json!({})), "fled", "Gold", &name).1, "The party got away with the rewards of the foes it defeated.");
        assert_eq!(reward_text(&rewards(0, 0, 0, json!({})), "fled", "Gold", &name).1, "The party got away. No foe was defeated, so no rewards.");
        assert_eq!(reward_text(&rewards(5, 0, 0, json!({})), "won", "Gold", &name).1, "");
        assert_eq!(reward_text(&json::obj([("foes", json!(0))]), "won", "Gold", &name).0, "", "no roster, no tally line");
        let full = rewards(5, 0, 0, json!({
            "gold": 12,
            "objective": {"gold": 10, "loot": "ashlands:keep_ring"},
            "drops": [{"item": "wpn_shortbow", "count": 1}, {"item": "con_heal_potion", "count": 1}, {"item": "con_heal_potion", "count": 1}],
            "key_loot": ["ashlands:warchiefs_axe"],
            "key_missed": [{"loot": "ashlands:warchiefs_axe", "kill": "ash_warchief"}],
        }));
        assert_eq!(reward_text(&full, "won", "Gold", &name).2, [
            "Gold +12",
            "Objective bonus: Gold +10, Keep Ring",
            "Dropped: Shortbow, Healing Potion x2",
            "Key loot: Warchief's Axe",
            "Missed: Warchief's Axe (slay Horde Warchief and win)",
        ]);
    }

    /// ENCOUNTER2 W4: the popup keeps the runner's `stay_lines` (one per stayed or stood hero); none without them.
    #[test]
    fn the_results_keep_the_stay_behind_lines() {
        let res = json::obj([("ok", json!(true)), ("stay_lines", json!(["Ash held the winch."]))]);
        assert_eq!(Results::new(&res, "won", &Dict::new()).stay_lines, ["Ash held the winch."]);
        assert!(Results::new(&json::obj([("ok", json!(true))]), "won", &Dict::new()).stay_lines.is_empty());
    }

    use crate::hud::hud_tests::{frame, game_cx, gfx};
    use emberwake_world::state::RosterCaps;

    /// INTENT2-U: the results draw one line per pouch slot the post-battle auto-fill left short (`stash_short`, the
    /// kit's "Stash short: <name> <cur> of <packed> <item>"), after the party lines; none without the key.
    #[test]
    fn the_results_draw_the_stash_short_lines() {
        let mut cx = game_cx();
        let line = "Stash short: Ash 1 of 3 Fire Flask";
        let res = json::obj([("ok", json!(true)), ("stay_lines", json!(["Held."])), ("stash_short", json!([line]))]);
        let mut r = Results::new(&res, "won", &Dict::new());
        let t = texts(&frame(&mut r, &mut cx, &mut gfx(), 1.0));
        let at = |s: &str| t.iter().position(|x| x == s).unwrap_or_else(|| panic!("{s} in {t:?}"));
        assert!(at("Held.") < at(line), "after the stay-behind lines");
        assert_eq!(r.state(&cx)["stash_short"], json!([line]));
        assert_clean(&mut r, &mut cx, "results with a stash short line");
        let none = Results::new(&json::obj([("ok", json!(true))]), "won", &Dict::new());
        assert!(none.stash_short.is_empty());
    }

    /// PK1-S1: the results draw one mark line per marked member (`battle::apply`'s `marks`) after the stash lines, and
    /// the state lists them; no key, no line and no state key.
    #[test]
    fn the_results_draw_the_battle_marks() {
        let mut cx = game_cx();
        let member = cx.gs().party.members[0].id.clone();
        let name = progression::display_name(&cx.data, &cx.gs().party.members[0]);
        let marks = json!([{"member": member, "mark": "calcify", "value": 1}, {"member": member, "mark": "scar", "value": 2}]);
        let res = json::obj([("ok", json!(true)), ("stash_short", json!(["Stash short: x"])), ("marks", marks.clone())]);
        let mut r = Results::new(&res, "won", &Dict::new());
        let t = texts(&frame(&mut r, &mut cx, &mut gfx(), 1.0));
        let line = format!("{name} is marked: calcify +1, scar +2.");
        let at = |s: &str| t.iter().position(|x| x == s).unwrap_or_else(|| panic!("{s} in {t:?}"));
        assert!(at("Stash short: x") < at(&line), "after the stash lines");
        assert_eq!(r.state(&cx)["marks"], marks);
        assert_clean(&mut r, &mut cx, "results with marks");
        let none = Results::new(&json::obj([("ok", json!(true))]), "won", &Dict::new());
        assert!(none.state(&cx).get("marks").is_none());
    }

    /// PR5-26 (audit B8; PK2-S1): the results draw the settle's track moves, stashed items and (review F2) emptied
    /// slots (`battle::apply`'s `tracks`, `stashed` and `cleared`) after the mark lines, and the state lists them; no keys, no lines and no state keys. A
    /// track id the catalog lacks reads as itself (test-only ids, no shipped content).
    #[test]
    fn the_results_draw_the_track_moves_and_stashed_items() {
        let mut cx = game_cx();
        let member = cx.gs().party.members[0].id.clone();
        let name = progression::display_name(&cx.data, &cx.gs().party.members[0]);
        let item = cx.data.cat.items.keys().next().cloned().expect("an item");
        let item_name = cx.data.item_name(&item);
        let marks = json!([{"member": member, "mark": "t_mark", "value": 1}]);
        let tracks = json!([{"member": member, "track": "t_track", "from": 1, "to": 2}, {"member": member, "track": "t_other", "from": 3, "to": 2}]);
        let stashed = json!([{"member": member, "item": item}]);
        let cleared = json!([{"member": member, "slot": "main_hand"}]);
        let res = json::obj([("ok", json!(true)), ("marks", marks), ("tracks", tracks.clone()), ("stashed", stashed.clone()), ("cleared", cleared.clone())]);
        let mut r = Results::new(&res, "won", &Dict::new());
        let t = texts(&frame(&mut r, &mut cx, &mut gfx(), 1.0));
        let at = |s: &str| t.iter().position(|x| x == s).unwrap_or_else(|| panic!("{s} in {t:?}"));
        let (up, down) = (format!("{name}'s t_track rises to stage 2."), format!("{name}'s t_other falls to stage 2."));
        let gone = format!("{name} can't wield {item_name} now: it went to the stash.");
        // review F2: the emptied slot names the default the member can't wield (rule-derived: its played world's)
        let default = cx.gs().party.members[0].default_item(&cx.data, &cx.gs().world_id, "main_hand").map(|i| cx.data.item_name(&i));
        let empty = format!("{name}'s main hand is empty{}.", default.map(|i| format!(": {name} can't wield {i} now")).unwrap_or_default());
        assert!(at(&format!("{name} is marked: t_mark +1.")) < at(&up) && at(&up) < at(&down) && at(&down) < at(&gone) && at(&gone) < at(&empty), "{t:?}");
        assert_eq!((r.state(&cx)["tracks"].clone(), r.state(&cx)["stashed"].clone(), r.state(&cx)["cleared"].clone()), (tracks, stashed, cleared));
        assert_clean(&mut r, &mut cx, "results with track moves");
        let none = Results::new(&json::obj([("ok", json!(true))]), "won", &Dict::new());
        assert!(none.state(&cx).get("tracks").is_none() && none.state(&cx).get("stashed").is_none() && none.state(&cx).get("cleared").is_none());
    }

    /// PR7-3 (review N3): the results draw the settle's remains, mastery gains and weapon wear (`battle::apply`'s
    /// `remains`, `mastery` and `wear`) after the track lines, a dulled weapon with its penalty line, and the state
    /// lists them; a worn weapon that is not dulled has no dulled line; no keys, no lines and no state keys. The mastery
    /// line is the kit's own note; the penalty comes from `DULLED_POWER_MULT` (test-only family and uids).
    #[test]
    fn the_results_draw_remains_mastery_and_wear() {
        let mut cx = game_cx();
        let member = cx.gs().party.members[0].id.clone();
        let name = progression::display_name(&cx.data, &cx.gs().party.members[0]);
        let mut items = cx.data.cat.items.keys().cloned();
        let (bone, blade, club) = (items.next().expect("an item"), items.next().expect("a second item"), items.next().expect("a third item"));
        let (bone_name, blade_name, club_name) = (cx.data.item_name(&bone), cx.data.item_name(&blade), cx.data.item_name(&club));
        let tracks = json!([{"member": member, "track": "t_track", "from": 1, "to": 2}]);
        let remains = json!([{"member": member, "item": bone}]);
        let gain = mastery::Gain { member: member.clone(), family: "t_family".into(), from: 0, to: 10 };
        let mastery_rows = json!([{"member": member, "family": "t_family", "from": 0, "to": 10, "level": gain.level()}]);
        let wear = json!([
            {"member": member, "slot": "main_hand", "uid": "t_w1", "item": blade, "worn": 3, "durability": 0, "dulled": true},
            {"member": member, "slot": "off_hand", "uid": "t_w2", "item": club, "worn": 2, "durability": 5, "dulled": false}
        ]);
        let res = json::obj([("ok", json!(true)), ("tracks", tracks), ("remains", remains.clone()), ("mastery", mastery_rows.clone()), ("wear", wear.clone())]);
        let mut r = Results::new(&res, "won", &Dict::new());
        let t = texts(&frame(&mut r, &mut cx, &mut gfx(), 1.0));
        let at = |s: &str| t.iter().position(|x| x == s).unwrap_or_else(|| panic!("{s} in {t:?}"));
        let track = format!("{name}'s t_track rises to stage 2.");
        let rem = format!("Remains: {bone_name} from {name}, to the stash.");
        let mast = mastery::note(&cx.data, cx.gs(), &gain);
        let worn = format!("{name}'s {blade_name} wore 3: 0 left.");
        let penalty = format!("-{:.0}% weapon power", (1.0 - emberwake_rules::gear::DULLED_POWER_MULT) * 100.0);
        let dulled = format!("{name}'s {blade_name} is dulled: {penalty} until repaired.");
        let other = format!("{name}'s {club_name} wore 2: 5 left.");
        assert!(at(&track) < at(&rem) && at(&rem) < at(&mast) && at(&mast) < at(&worn) && at(&worn) < at(&dulled) && at(&dulled) < at(&other), "{t:?}");
        assert!(!t.iter().any(|x| x.contains(&club_name) && x.contains("dulled")), "a weapon with durability left is not dulled: {t:?}");
        let st = r.state(&cx);
        assert_eq!((st["remains"].clone(), st["mastery"].clone(), st["wear"].clone()), (remains, mastery_rows, wear));
        assert_clean(&mut r, &mut cx, "results with remains, mastery and wear");
        let mut none = Results::new(&json::obj([("ok", json!(true))]), "won", &Dict::new());
        let t = texts(&frame(&mut none, &mut cx, &mut gfx(), 1.0));
        assert!(!t.iter().any(|x| x.starts_with("Remains:") || x.contains(" mastery +") || x.contains(" wore ") || x.contains("dulled")), "{t:?}");
        assert!(["remains", "mastery", "wear"].iter().all(|k| none.state(&cx).get(*k).is_none()));
    }

    /// A report row (test-only numbers) for `key` (a guest when `key` is empty).
    fn report_row(key: &str, name: &str, dealt: i32, healed: i32) -> emberwake_battle::report::Row {
        emberwake_battle::report::Row {
            key: key.into(), name: name.into(), guest: key.is_empty(), unit: String::new(), state: "standing", dealt, dealt_dot: 0, ff_dealt: 0, taken: 7,
            taken_dot: 0, healed, regen: 0, effects: 0, combos: 0, kills: 1, status_counts: Default::default(), combo_counts: Default::default(),
        }
    }

    /// PR7-32 (BREPORTUI1): the results show the kit's battle report: a row per awarded member then the guests, each
    /// column's number as a widget with its tip (a 0 reads "-"), the best-moment line and the report in the state; drops
    /// draw as tiles in place of the "Dropped:" line; without a report there is no table number and no best line.
    #[test]
    fn the_results_show_the_battle_report_and_drop_tiles() {
        let mut cx = party_of(2);
        let (a, b) = (cx.gs().party.members[0].clone(), cx.gs().party.members[1].clone());
        let item = cx.data.cat.items.keys().next().expect("an item").clone();
        let award = json!({a.id.clone(): {"xp": 12, "old_level": 1, "new_level": 1}});
        let rewards = json!({"drops": [{"item": item, "count": 1}, {"item": item, "count": 2}]});
        let res = json::obj([("ok", json!(true)), ("xp", award), ("rewards", rewards)]);
        let report = emberwake_battle::report::Report {
            outcome: "won".into(), rounds: 3, rows: vec![report_row(&b.id, "B", 40, 0), report_row(&a.id, "A", 25, 9), report_row("", "Ash", 5, 0)],
            columns: vec!["dealt", "taken", "healed"], best: "A's Cut hit a foe for 30.".into(),
        };
        let mut r = Results::new(&res, "won", &Dict::new()).report(Some(report));
        let t = texts(&frame(&mut r, &mut cx, &mut gfx(), 1.0));
        let w = r.widgets();
        let get = |id: &str| w.iter().find(|x| x["id"] == id).unwrap_or_else(|| panic!("{id} in {w:?}")).clone();
        // party order (a, then b), then the guest
        assert_eq!(get("results.report.0.dealt")["text"], "25");
        assert_eq!(get("results.report.1.dealt")["text"], "40");
        assert_eq!(get("results.report.2.dealt")["text"], "5");
        assert_eq!(get("results.report.1.healed")["text"], "-", "a 0 reads as a dash");
        assert!(get("results.report.0.dealt")["tip"].to_string().contains("Damage 25"), "{}", get("results.report.0.dealt"));
        assert!(t.iter().any(|x| x == "Ash (guest)"), "{t:?}");
        assert!(["Damage", "Taken", "Healed"].iter().all(|h| t.iter().any(|x| x == h)), "{t:?}");
        assert_eq!(get("results.best")["text"], "Best moment: A's Cut hit a foe for 30.");
        assert_eq!(get(&format!("results.drop.{item}"))["text"], format!("{} x3", cx.data.item_name(&item)));
        assert!(!t.iter().any(|x| x.starts_with(DROPPED)), "the tiles replace the Dropped line: {t:?}");
        let st = r.state(&cx);
        assert_eq!((st["report"]["version"].clone(), st["report"]["rows"][0]["name"].clone()), (json!(1), json!("B")), "{st}");
        assert_clean(&mut r, &mut cx, "results with a report");
        let mut none = Results::new(&res, "won", &Dict::new());
        frame(&mut none, &mut cx, &mut gfx(), 1.0);
        assert!(!none.widgets().iter().any(|x| x["id"].as_str().is_some_and(|id| id.starts_with("results.report.") || id == "results.best")));
        assert!(none.state(&cx).get("report").is_none_or(Value::is_null));
    }

    /// PT-59 (PR7-75): a guest's results row shows its catalog unit's portrait (the initial only without one).
    #[test]
    fn a_guest_row_shows_its_portrait() {
        let mut cx = party_of(1);
        cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let units: Vec<String> = cx.data.cat.units.keys().cloned().collect();
        let unit = units.into_iter().find(|u| {
            let who = Who::of_catalog(&cx.data, u, "Guest");
            cx.art.first(&who.chain(""), 32, 32).is_some()
        });
        let unit = unit.expect("a unit with a portrait");
        let a = cx.gs().party.members[0].clone();
        let res = json::obj([("ok", json!(true)), ("xp", json!({a.id.clone(): {"xp": 1, "old_level": 1, "new_level": 1}}))]);
        let shot = |cx: &mut Cx, unit: &str| {
            let mut row = report_row("", "Guest", 5, 0);
            row.unit = unit.into();
            let report = emberwake_battle::report::Report { outcome: "won".into(), rounds: 1, rows: vec![report_row(&a.id, "A", 1, 0), row], columns: vec!["dealt"], best: String::new() };
            let mut r = Results::new(&res, "won", &Dict::new()).report(Some(report));
            let mut g = gfx();
            frame(&mut r, cx, &mut g, 1.0);
            let cell = r.widgets().into_iter().find(|w| w["id"] == "results.report.1.dealt").expect("the guest's row");
            (g.canvas.px.clone(), cell["rect"][1].as_i64().unwrap_or(0) as i32)
        };
        let (with, y) = shot(&mut cx, &unit);
        let (without, _) = shot(&mut cx, "");
        // the row's portrait well, left of its name, differs from the initial-only well
        let w = 640usize;
        let differs = (y..y + 20).any(|py| (0..320).any(|px| {
            let i = (py as usize * w + px) * 4;
            with[i..i + 3] != without[i..i + 3]
        }));
        assert!(differs, "the guest's portrait draws in place of its initial");
    }

    /// PR7-32 (PT-13): Level up now opens the character sheet over the results without closing them (so its Close
    /// comes back to the report), even over a battle; Continue closes them.
    #[test]
    fn level_up_now_keeps_the_results_under_the_character_sheet() {
        let mut cx = party_of(1);
        let id = cx.gs().party.members[0].id.clone();
        let res = json::obj([("ok", json!(true)), ("xp", json!({id: {"xp": 500, "old_level": 1, "new_level": 2}}))]);
        for over in [false, true] {
            let mut r = Results::new(&res, "won", &Dict::new());
            if over {
                r = r.over_battle(true, None);
            }
            cx.nav.clear();
            r.act(&mut cx, ResAct::LevelUp);
            assert_eq!(cx.nav.len(), 1, "over {over}: only the sheet opens");
            let Some(Nav::Push(sheet)) = cx.nav.pop() else { panic!("over {over}: the sheet opens") };
            assert_eq!(sheet.id(), "character");
            assert!(r.leaving.is_none(), "over {over}: the results stay");
        }
        cx.nav.clear();
        Results::new(&res, "won", &Dict::new()).act(&mut cx, ResAct::Continue);
        assert!(matches!(cx.nav.pop(), Some(Nav::Pop)));
    }

    /// PR7-32 (PT-11): mid-slide over a battle the panel and its Continue stay inside the window.
    #[test]
    fn the_sliding_results_stay_inside_the_window() {
        let mut cx = party_of(1);
        let id = cx.gs().party.members[0].id.clone();
        let res = json::obj([("ok", json!(true)), ("xp", json!({id: {"xp": 500, "old_level": 1, "new_level": 2}}))]);
        let mut r = Results::new(&res, "won", &Dict::new()).over_battle(false, None);
        for scale in [1.0, 1.3] {
            r.opened = Instant::now();
            frame(&mut r, &mut cx, &mut gfx(), scale);
            for b in ["results.continue", "results.level_up"] {
                let w = r.widgets().into_iter().find(|w| w["id"] == b).unwrap_or_else(|| panic!("{b}"));
                let (x, _, ww, _) = crate::hud::hud_tests::rect(&w);
                assert!(x >= 0 && x + ww <= W as i64, "{b} at {scale}: {x}+{ww}");
            }
        }
    }

    /// A game whose party has at least `n` members (copies of the first under new ids, test-only members).
    fn party_of(n: usize) -> Cx {
        let mut cx = game_cx();
        let party = &mut cx.gs_mut().party;
        assert!(!party.members.is_empty());
        let mut i = 0;
        while party.members.len() < n {
            let mut m = party.members[0].clone();
            m.id = format!("test_member_{i}");
            i += 1;
            party.members.push(m);
        }
        cx
    }

    /// The played world gets a `roster` block with `bunks` (BK1: a scratch block on the party's caps, no data edit).
    fn with_bunks(cx: &mut Cx, bunks: i64) {
        let max = cx.gs().party.roster.party_max;
        cx.gs_mut().party.roster = RosterCaps { party_max: max, manifest: emberwake_data::RosterManifest::from_obj(json!({"bunks": bunks}).as_object().unwrap()) };
    }

    fn texts(recs: &[aegis_ui::painter::TextRec]) -> Vec<String> {
        recs.iter().map(|r| r.text.clone()).collect()
    }

    /// TEXTFIT1: no text leaves the canvas, overlaps another or runs out of its control, at every text size.
    fn assert_clean(screen: &mut dyn Screen<GameCx>, cx: &mut Cx, what: &str) {
        let mut g = gfx();
        for (size, scale) in aegis_ui::screen::TEXT_SIZES {
            let recs = frame(screen, cx, &mut g, scale as f32);
            let boxes: Vec<(String, f32, f32, f32, f32)> = screen.widgets().into_iter().map(|w| {
                let (x, y, ww, h) = crate::hud::hud_tests::rect(&w);
                (w["id"].as_str().unwrap_or("").to_string(), x as f32, y as f32, ww as f32, h as f32)
            }).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            for k in ["edge", "overlap", "spill"] {
                assert_eq!(v["counts"][k], json!(0), "{what} at {size}: {v}");
            }
        }
    }

    /// ROSTER1-U on the game menu: a classic world shows no bunk chip and no wounded mark; with a roster block the
    /// chip reads "Bunks used/cap" (BK1, BK2) and a wounded member shows the HP it takes the field with (W2: the cut max,
    /// the default percent without a rule row) beside its mark; past four members the box takes two columns; the
    /// menu reads clean at every text size.
    #[test]
    fn the_game_menu_marks_the_wounded_and_counts_the_bunks() {
        let mut cx = party_of(1);
        let mut hub = Hub::default();
        let recs = frame(&mut hub, &mut cx, &mut gfx(), 1.0);
        assert!(!texts(&recs).iter().any(|t| t.starts_with("Bunks") || t == "Wounded"), "classic: no chip, no mark");
        assert!(!hub.w.iter().any(|w| w.id.starts_with("hub.wounded.")));
        assert_clean(&mut hub, &mut cx, "classic menu");

        let mut cx = party_of(6);
        with_bunks(&mut cx, 6);
        let id = cx.gs().party.members[0].id.clone();
        let world = cx.gs().world_id.clone();
        let (_, full) = progression::current_hp(&cx.data, &cx.gs().party.members[0], &world);
        cx.gs_mut().party.member_mut(&id).unwrap().add_condition(roster::WOUNDED);
        let cut = roster::wounded_max(full, emberwake_data::WOUNDED_PCT_DEFAULT);
        let recs = frame(&mut hub, &mut cx, &mut gfx(), 1.0);
        let t = texts(&recs);
        assert!(t.contains(&format!("Bunks {}/6", cx.gs().party.members.len())), "{t:?}");
        assert!(t.contains(&format!("HP {cut} / {cut}")), "the wounded member at full HP shows the cut max: {t:?}");
        assert_eq!(t.iter().filter(|x| *x == "Wounded").count(), 1);
        let mark = hub.w.iter().find(|w| w.id == format!("hub.wounded.{id}")).expect("the mark's hover");
        assert!(mark.tip.iter().any(|(l, _)| l.contains(&format!("{cut} of {full} max HP"))), "{:?}", mark.tip);
        assert_clean(&mut hub, &mut cx, "six members, bunks 6, one wounded");
    }

    /// STYLE-PASS1B (`PR7-48` (2), never stacked): the game menu's Party opens the Character window in the menu's place:
    /// over the map only the Character window draws, the menu stays in the stack and Back shows it again. Settings takes
    /// its place the same way.
    #[test]
    fn the_game_menu_then_party_shows_one_panel_and_back_returns_to_the_menu() {
        let mut cx = party_of(2);
        let mut stack: Vec<Box<dyn Screen<GameCx>>> = vec![Box::new(crate::menus::MainMenu::default()), Box::new(Hub::default())];
        assert_eq!(aegis_ui::screen::drawn_screens(&stack), [0, 1]);
        for (act, id) in [(HubAct::Party, "character"), (HubAct::Settings, "settings")] {
            cx.nav.clear();
            Hub::default().act(&mut cx, act);
            let Some(Nav::Push(opened)) = cx.nav.pop() else { panic!("{id}: a screen opens") };
            assert_eq!((opened.id(), opened.overlay(), opened.panel()), (id, true, true));
            stack.push(opened);
            assert_eq!(aegis_ui::screen::drawn_screens(&stack), [0, 2], "{id}: one panel over the map");
            stack.pop();
            assert_eq!(aegis_ui::screen::drawn_screens(&stack), [0, 1], "{id}: Back returns to the game menu");
        }
    }

    /// PR7-70 (PANELS2): Save and Load from the game menu open the slots as a panel that draws alone, and Back shows the
    /// game menu again; the forge, the dojo and the services screen (a shop off the map) are panels too, so a panel
    /// over the game menu's place never draws the menu under them.
    #[test]
    fn the_game_menu_then_slots_shows_only_the_slots_and_back_returns_to_the_menu() {
        let mut cx = party_of(2);
        let mut stack: Vec<Box<dyn Screen<GameCx>>> = vec![Box::new(crate::menus::MainMenu::default()), Box::new(Hub::default())];
        // Load asks first (unsaved progress), so its slots are pushed as its Yes does
        for (act, id) in [(Some(HubAct::Save), "save_slots"), (None, "load_slots")] {
            cx.nav.clear();
            let opened: Box<dyn Screen<GameCx>> = match act {
                Some(a) => {
                    Hub::default().act(&mut cx, a);
                    let Some(Nav::Push(opened)) = cx.nav.pop() else { panic!("{id}: a screen opens") };
                    opened
                }
                None => Box::new(crate::menus::Slots::new(false)),
            };
            assert_eq!((opened.id(), opened.panel()), (id, true));
            stack.push(opened);
            assert_eq!(aegis_ui::screen::drawn_screens(&stack), [2], "{id}: only the slots");
            stack.pop();
            assert_eq!(aegis_ui::screen::drawn_screens(&stack), [0, 1], "{id}: Back returns to the game menu");
        }
        let others: Vec<Box<dyn Screen<GameCx>>> = vec![Box::new(crate::forge::Forge::content()), Box::new(crate::dojo::Dojo::new(&cx)), Box::new(crate::talk::ServiceScreen::default())];
        for other in others {
            let id = other.id();
            assert!(other.panel() && !other.overlay(), "{id}: a panel that fills the screen");
            stack.push(other);
            assert_eq!(aegis_ui::screen::drawn_screens(&stack), [2], "{id}: drawn alone");
            stack.pop();
        }
    }

    /// ROSTER1-U on the results: after the stay-behind lines, a line per departure (its life), the keepsake its record
    /// kept (KS1, KS2), then the newly wounded; none without the keys; the popup reads clean at every text size.
    #[test]
    fn the_results_name_the_departed_their_keepsakes_and_the_wounded() {
        let mut cx = party_of(3);
        let d = cx.data.clone();
        let (gone, hurt) = (cx.gs().party.members[0].clone(), cx.gs().party.members[1].clone());
        let (gone_name, hurt_name) = (progression::display_name(&d, &gone), progression::display_name(&d, &hurt));
        roster::depart(&d, cx.gs_mut(), &gone.id, roster::CAPTURED, "stayed", "").expect("departs");
        let item = d.cat.items.keys().next().cloned().expect("a catalog item");
        cx.gs_mut().party.departed.last_mut().unwrap().keepsake = item.clone();
        cx.gs_mut().party.member_mut(&hurt.id).unwrap().add_condition(roster::WOUNDED);
        let res = json::obj([
            ("ok", json!(true)),
            ("stay_lines", json!(["Held."])),
            ("departed", json!([{"member": gone.id, "life": roster::CAPTURED}])),
            ("wounded", json!([hurt.id])),
        ]);
        let mut r = Results::new(&res, "won", &Dict::new());
        assert_eq!(party_lines(&d, cx.gs(), &r.wounded, &r.departed), [
            (format!("{gone_name} was captured."), PartyLine::Departed),
            (format!("Keepsake: {}, to the stash.", d.item_name(&item)), PartyLine::Keepsake),
            (format!("Wounded until the next rest: {hurt_name}."), PartyLine::Wounded),
        ]);
        let t = texts(&frame(&mut r, &mut cx, &mut gfx(), 1.0));
        let at = |s: &str| t.iter().position(|x| x == s).unwrap_or_else(|| panic!("{s} in {t:?}"));
        assert!(at("Held.") < at(&format!("{gone_name} was captured.")), "the party lines follow the stay-behind lines");
        assert_eq!(r.state(&cx)["departed"], json!([[gone.id, roster::CAPTURED]]));
        assert_clean(&mut r, &mut cx, "results with party lines");
        assert!(party_lines(&d, cx.gs(), &[], &[]).is_empty(), "no keys, no lines");
        cx.gs_mut().party.departed.last_mut().unwrap().keepsake.clear();
        assert_eq!(party_lines(&d, cx.gs(), &[], &[(gone.id.clone(), roster::DEAD.into())]), [(format!("{gone_name} died."), PartyLine::Departed)], "no keepsake paid, no keepsake line");
    }
}

/// PORTRAITFX1-R (docs/specs/PORTRAITFX1.md 6.2 row 2, PORTRAITFX1_TASKS 1.6 R1-R3): the party list's portraits.
#[cfg(test)]
mod portrait_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};
    use crate::portraits::tests::{roster_cx, row};
    use aegis_ui::fx::Clock;
    use rpgkit_ui::portrait_fx::EASE_MS;
    use aegis_ui::screen::TEXT_SIZES;
    use emberwake_world::state::RosterCaps;

    /// Whether text `r` and rect `b` share any area.
    fn meets(r: &aegis_ui::painter::TextRec, b: Rect) -> bool {
        r.x < (b.x + b.w) as f32 && r.x + r.w > b.x as f32 && r.y < (b.y + b.h) as f32 && r.y + r.h > b.y as f32
    }

    /// The centre pixel's channel spread (max - min): smaller is greyer.
    fn spread(hub: &Hub, key: &str) -> i32 {
        let img = hub.faces.get(key).and_then(|f| f.composed()).expect("composed");
        let i = ((img.h / 2 * img.w + img.w / 2) * 4) as usize;
        let c = &img.px[i..i + 3];
        *c.iter().max().unwrap() as i32 - *c.iter().min().unwrap() as i32
    }

    /// R1-R3: one or six members (two columns, the bunk chip); every row draws a 20 px portrait inside the menu frame at
    /// the row's left, overlapping no widget and no text; the name starts 22 px right of it; the wounded member's
    /// portrait holds the `wounded` row's desaturate 0.35 (its centre pixel greyer than an unwounded copy's), the others
    /// none; the menu's `state` lists every row's portrait; TEXTFIT1 clean at every text size.
    #[test]
    fn r1_r3_the_party_list_rows_draw_member_portraits() {
        let rows = vec![row(json!({"id": "wounded", "trigger": "condition", "condition": "wounded", "effect": "desaturate", "intensity": 0.35}))];
        for n in [1, 6] {
            let mut cx = game_cx();
            {
                let party = &mut cx.gs_mut().party;
                let mut i = 0;
                while party.members.len() < n {
                    let mut m = party.members[0].clone();
                    m.id = format!("test_member_{i}");
                    i += 1;
                    party.members.push(m);
                }
            }
            let mut cx = roster_cx(cx, &format!("hub_r{n}"), rows.clone(), [200, 80, 40]);
            let max = cx.gs().party.roster.party_max;
            cx.gs_mut().party.roster = RosterCaps { party_max: max, manifest: emberwake_data::RosterManifest::from_obj(json!({"bunks": 6}).as_object().unwrap()) };
            let members: Vec<UnitState> = cx.gs().party.members.clone();
            let wounded = members[0].id.clone();
            cx.gs_mut().party.member_mut(&wounded).unwrap().add_condition(roster::WOUNDED);
            let fr = Rect::new(95, 25, 450, 310);
            let mut hub = Hub::default();
            let mut g = gfx();
            for (size, scale) in TEXT_SIZES {
                let mut recs = Vec::new();
                for t in [1000, 1000 + EASE_MS] {
                    hub.faces.clock = Clock { ms: t as f64, hold: 0.0 };
                    recs = frame(&mut hub, &mut cx, &mut g, scale as f32);
                }
                for m in &members {
                    let pid = format!("hub.portrait.{}", m.id);
                    let pr = hub.w.iter().find(|w| w.id == pid).unwrap_or_else(|| panic!("{size}: {pid}")).rect;
                    assert_eq!((pr.w, pr.h), (ROW_PORTRAIT, ROW_PORTRAIT), "{size}: 20 px");
                    assert!(pr.x >= fr.x && pr.y >= fr.y && pr.x + pr.w <= fr.x + fr.w && pr.y + pr.h <= fr.y + fr.h, "{size}: {pid} {pr:?} inside the frame");
                    for w in hub.w.iter().filter(|w| w.id != pid) {
                        let x = pr.intersect(w.rect);
                        assert!(x.w <= 0 || x.h <= 0, "{size}: {pid} {pr:?} overlaps {} {:?}", w.id, w.rect);
                    }
                    assert!(!recs.iter().any(|r| !r.clipped && meets(r, pr)), "{size}: no text on {pid}");
                    let name = member_name(&cx, m);
                    let at = recs.iter().find(|r| r.cut.as_deref().unwrap_or(&r.text) == name && (r.y - pr.y as f32).abs() < 0.5 && (r.x - (pr.x as f32 + ROW_INDENT)).abs() < 0.5);
                    assert!(at.is_some(), "{size}: {}'s name starts 22 px right of its portrait", m.id);
                    let fx = hub.faces.get(&format!("member:{}", m.id)).expect("kept");
                    let desat = fx.level("desaturate", hub.faces.now());
                    if m.id == wounded {
                        assert!(desat.is_some_and(|l| (l.i - 0.35).abs() < 1e-3), "{size}: wounded: desaturate 0.35");
                    } else {
                        assert!(desat.is_none(), "{size}: {} is not wounded", m.id);
                    }
                }
                if n > 1 {
                    assert!(spread(&hub, &format!("member:{wounded}")) < spread(&hub, &format!("member:{}", members[1].id)), "{size}: the wounded portrait is greyer");
                }
                assert_eq!(hub.state(&cx)["portraits"].as_array().map(Vec::len), Some(n), "{size}: every row's portrait in the state");
                let boxes: Vec<(String, f32, f32, f32, f32)> = hub.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                for k in ["edge", "overlap", "spill"] {
                    assert_eq!(v["counts"][k], json!(0), "{n} members at {size}: {v}");
                }
            }
        }
    }
}
