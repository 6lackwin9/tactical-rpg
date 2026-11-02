//! The game menu over the map (hub_model.gd), the quest journal (journal_model.gd) and the battle results popup
//! (battle_results_model.gd). Read-only views over the game state, except the hub's doors (save, load, settings,
//! party, quit to the main menu).

use aegis_ui::theme::th;
use aegis_ui::canvas::{rgba, Rect};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::state::{GameState, UnitState};
use emberwake_world::{progression, quest, roster, save};
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
/// (`Who::of_member`).
pub fn member_portrait(d: &emberwake_world::Data, gs: &GameState, m: &UnitState) -> (Who, Look) {
    let mut who = Who::of_member(d, m);
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
            HubAct::Settings => cx.nav.push(Nav::Push(Box::new(crate::menus::SettingsScreen::new(cx)))),
            HubAct::Return => cx.nav.push(Nav::Pop),
            HubAct::Quit => {
                let go = |cx: &mut Cx| {
                    cx.end_game();
                    cx.nav.push(Nav::Reset(Box::new(crate::menus::MainMenu::default())));
                };
                if cx.is_dirty() {
                    cx.ask_no("Quit to the main menu without saving? Progress since your last save is lost.", go);
                } else {
                    go(cx);
                }
            }
        }
    }
}

impl Screen<GameCx> for Hub {
    fn id(&self) -> &'static str {
        "hub"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<HubAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("popup_veil", rgba(0, 0, 0, 110)));
        self.faces.begin();
        let gs = cx.gs().clone();
        let fr = Rect::new(95, 25, 450, 310);
        ui.frame(fr, &cx.world_name(&gs.world_id), true);
        ui.p.text_center(320.0, 62.0, &cx.location_name(&gs.world_id, &gs.location_id), BODY + 1.0, th().dim);
        let active: Vec<String> = gs.party.active_members().iter().map(|m| m.id.clone()).collect();
        let mut rows: Vec<&UnitState> = gs.party.members.iter().filter(|m| active.contains(&m.id)).collect();
        rows.extend(gs.party.members.iter().filter(|m| !active.contains(&m.id)));
        // ROSTER1-U: the HP a member takes the field with (a wounded one's cut max, W2) and its mark; the box grows with
        // the widest line and takes a second column past four members (bunks above the party max, BK1); PORTRAITFX1-R: a
        // 20 px portrait at the row's left, its three lines indented 22 px (the column grows by the indent, so the name
        // fits 22 px shorter than the column and the class and HP lines keep their width)
        let step = 13f32.max(ui.p.line_h(BODY + 1.0).ceil());
        let row_h = (step * 3.0 + 3.0) as i32;
        let lines: Vec<(String, (i64, i64, bool))> = rows.iter().map(|m| (format!("{}  Lv {}", class_name(cx, m), m.level(&cx.data)), field_hp(cx, m))).collect();
        let mark = "Wounded";
        let gap = 6.0;
        let mut col_w = 100f32;
        for (class_line, (hp, max, wounded)) in &lines {
            let hp_w = ui.p.text_w(&format!("HP {hp} / {max}"), BODY + 1.0) + if *wounded { gap + ui.p.text_w(mark, BODY + 1.0) } else { 0.0 };
            col_w = col_w.max(ui.p.text_w(class_line, BODY + 1.0).max(hp_w) + 6.0);
        }
        let col_w = (col_w + ROW_INDENT).ceil() as i32;
        let cols = if rows.len() > 4 { 2 } else { 1 };
        let per_col = rows.len().div_ceil(cols);
        let bh = 4 + per_col as i32 * row_h;
        let bx = Rect::new(320 - cols as i32 * col_w / 2, 80, cols as i32 * col_w, bh);
        ui.p.panel(bx, th().card, th().edge, 2.0);
        for (i, (m, (class_line, (hp, max, wounded)))) in rows.iter().zip(&lines).enumerate() {
            let x = (bx.x + (i / per_col) as i32 * col_w) as f32 + 3.0;
            let y = bx.y as f32 + 3.0 + (i % per_col) as f32 * row_h as f32;
            let on = active.contains(&m.id);
            let (who, look) = member_portrait(&cx.data, &gs, m);
            let pr = Rect::new(x as i32, y as i32, ROW_PORTRAIT, ROW_PORTRAIT);
            self.faces.draw(&mut ui.p, cx, &who, &look, "roster", pr);
            ui.push(format!("hub.portrait.{}", m.id), "portrait", member_name(cx, m), pr, true, false, None, Vec::new());
            let x = x + ROW_INDENT;
            ui.p.text_fit(x, y, &member_name(cx, m), BODY + 1.0, col_w as f32 - 6.0 - ROW_INDENT, if on { th().info } else { th().dim });
            ui.p.text(x, y + step, class_line, BODY + 1.0, th().text);
            let hp_line = format!("HP {hp} / {max}");
            ui.p.text(x, y + step * 2.0, &hp_line, BODY + 1.0, th().text);
            if *wounded {
                let mx = x + ui.p.text_w(&hp_line, BODY + 1.0) + gap;
                ui.p.text(mx, y + step * 2.0, mark, BODY + 1.0, th().foe);
                let r = Rect::new(mx as i32 - 1, (y + step * 2.0) as i32 - 1, ui.p.text_w(mark, BODY + 1.0) as i32 + 2, step as i32);
                ui.push(format!("hub.wounded.{}", m.id), "row", mark, r, true, false, None, tip(mark, &wounded_tip(cx, m)));
            }
        }
        let mut below = bx.y + bh;
        if let Some(bunks) = bunks_text(&gs) {
            ui.p.text_center(320.0, below as f32 + 3.0, &bunks, BODY + 1.0, th().dim);
            below += step as i32 + 3;
        }
        let pts: i64 = gs.party.members.iter().map(|m| m.unspent(&cx.data).max(0)).sum();
        let entries = [
            (HubAct::Party, "hub.party", if pts > 0 { format!("Party ({pts} point{} to spend)", if pts == 1 { "" } else { "s" }) } else { "Party".into() }, "Roster, level up, skills and gear."),
            (HubAct::Journal, "hub.journal", "Journal".into(), "Quests ready, active, on hold and completed."),
            (HubAct::Save, "hub.save", "Save game".into(), "Save to one of the manual slots."),
            (HubAct::Load, "hub.load", "Load game".into(), "Load a save; unsaved progress here is lost (you are asked first)."),
            (HubAct::Settings, "hub.settings", "Settings".into(), "Sound, display, gameplay and controls."),
            (HubAct::Quit, "hub.quit", "Quit to main menu".into(), "Back to the title; unsaved progress is lost (you are asked first)."),
        ];
        let top = below + 10;
        let mut detail = "Roster, level up, skills and gear.".to_string();
        for (i, (a, id, label, desc)) in entries.iter().enumerate() {
            let r = Rect::new(if i % 2 == 0 { 187 } else { 323 }, top + (i as i32 / 2) * 24, 130, 18);
            let why = Self::reason(cx, *a);
            ui.button_full(id, r, label, *a, true, false, BODY + 1.0, if why.is_empty() { tip(label, desc) } else { vec![(label.clone(), th().gold), (why.clone(), th().foe)] });
            if ui.hover(r) {
                detail = if why.is_empty() { desc.to_string() } else { why };
            }
        }
        ui.p.text_center(320.0, (top + 76) as f32, &detail, BODY + 1.0, th().dim);
        ui.button("hub.return", Rect::new(110, 303, 75, 18), "Return to map", HubAct::Return, true);
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
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let mut ui: Ui<JAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("popup_veil", rgba(0, 0, 0, 110)));
        let fr = Rect::new(16, 16, 608, 328);
        ui.frame(fr, "Journal", false);
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
    w: Vec<Widget<ResAct>>,
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

/// REWARD1 section 7: the tally line ("Foes defeated: 3 of 5 (2 slain, 1 surrendered)."), the outcome's note, and the
/// reward lines (share gold, objective bonus, drops merged by item, key loot, missed key loot); `name(kind, id)` gives
/// an item's, a loot entry's or a character's name; `currency` is the currency's short form in the battle's world
/// (TERMS1 TU2: "Gold", "Scrap").
pub fn reward_text(rewards: &Dict, outcome: &str, currency: &str, name: &dyn Fn(&str, &str) -> String) -> (String, String, Vec<String>) {
    let foes = json::i(rewards, "foes", 0);
    let d = json::dget(rewards, "defeated");
    let (slain, surr, fled) = (json::i(d, "slain", 0), json::i(d, "surrendered", 0), json::i(d, "fled", 0));
    let defeated = slain + surr + fled;
    let tally = if foes == 0 {
        String::new()
    } else {
        let parts: Vec<String> = [(slain, "slain"), (surr, "surrendered"), (fled, "fled")].iter().filter(|(n, _)| *n > 0).map(|(n, w)| format!("{n} {w}")).collect();
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
    let mut merged: Vec<(String, i64)> = Vec::new();
    for x in json::aget(rewards, "drops").iter().filter_map(Value::as_object) {
        let (item, n) = (json::s(x, "item"), json::i(x, "count", 1));
        match merged.iter_mut().find(|m| m.0 == item) {
            Some(m) => m.1 += n,
            None => merged.push((item, n)),
        }
    }
    if !merged.is_empty() {
        let parts: Vec<String> = merged.iter().map(|(item, n)| if *n > 1 { format!("{} x{n}", name("items", item)) } else { name("items", item) }).collect();
        lines.push(format!("Dropped: {}", parts.join(", ")));
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
            w: Vec::new(),
        }
    }

    fn act(&mut self, cx: &mut Cx, a: ResAct) {
        cx.nav.push(Nav::Pop);
        if let ResAct::LevelUp = a {
            let first = cx.gs().party.members.iter().find(|m| self.award.contains_key(&m.id) && m.unspent(&cx.data) > 0).map(|m| m.id.clone()).unwrap_or_default();
            cx.nav.push(Nav::Push(Box::new(crate::party::Character::new(cx, &first, "level_up"))));
        }
    }
}

impl Screen<GameCx> for Results {
    fn id(&self) -> &'static str {
        "battle_results"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let mut ui: Ui<ResAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("result_veil", rgba(10, 14, 24, 235)));
        let head = match self.outcome.as_str() {
            "won" => "Victory",
            "lost" => "Defeat",
            "fled" => "Fled",
            o => o,
        };
        ui.p.text(24.0, 24.0, head, aegis_ui::ui::HEAD + 4.0, th().gold);
        let world = gs.world_id.clone();
        let name = |kind: &str, id: &str| -> String {
            match kind {
                "items" => d.item_name(id),
                _ => s_or(d.resolve(&world, kind, id), "name", id),
            }
        };
        let currency = emberwake_world::terms::short(&d, &world, emberwake_world::terms::CURRENCY);
        let (tally, note, reward_lines) = reward_text(&self.rewards, &self.outcome, &currency, &name);
        let line = (ui.p.line_h(BODY + 1.0) + 3.0).ceil() as i32;
        let mut y = 50;
        for (text, col) in [(&tally, th().text), (&note, th().dim)] {
            if !text.is_empty() {
                ui.p.text(24.0, y as f32, text, BODY + 1.0, col);
                y += line;
            }
        }
        // ENCOUNTER2 W4: who stayed behind ("Kaelen held the winch.")
        for text in &self.stay_lines {
            ui.p.text(24.0, y as f32, text, BODY + 1.0, th().info);
            y += line;
        }
        // ROSTER1-U: who left (and the keepsake they left), who came home wounded
        for (text, kind) in party_lines(&d, &gs, &self.wounded, &self.departed) {
            let col = match kind {
                PartyLine::Departed => th().foe,
                PartyLine::Keepsake => th().gold,
                PartyLine::Wounded => th().dim,
            };
            ui.p.text(24.0, y as f32, &text, BODY + 1.0, col);
            y += line;
        }
        let mut prompt = false;
        for m in &gs.party.members {
            let Some(a) = self.award.get(&m.id).and_then(Value::as_object) else { continue };
            let level = m.level(&d);
            let base = progression::xp_for_level(&d, level);
            let span = progression::xp_for_level(&d, level + 1) - base;
            let fill = if span <= 0 { 1.0 } else { ((m.total_xp - base) as f32 / span as f32).clamp(0.0, 1.0) };
            let (old, new) = (json::i(a, "old_level", 0), json::i(a, "new_level", 0));
            ui.p.text(24.0, y as f32, &member_name(cx, m), BODY + 1.0, th().info);
            ui.p.text(130.0, y as f32, &format!("+{} XP", json::i(a, "xp", 0)), BODY + 1.0, th().text);
            ui.p.text(190.0, y as f32, &if new > old { format!("Level {old} -> {new}!") } else { format!("Level {level}") }, BODY + 1.0, if new > old { th().good } else { th().text });
            let bar = Rect::new(270, y + 3, 150, 5);
            ui.p.panel(bar, th().track, th().track, 2.0);
            ui.p.panel(Rect::new(bar.x, bar.y, (150.0 * fill) as i32, 5), th().gold, th().gold, 2.0);
            ui.p.text(426.0, y as f32, &format!("{} to next", progression::xp_to_next(&d, m.total_xp)), SMALL, th().dim);
            if new > old && m.unspent(&d) > 0 {
                prompt = true;
            }
            y += 16;
        }
        ui.p.text(24.0, (y + 4) as f32, "Rewards", SMALL, th().info);
        y += 16;
        let mut lines = reward_lines;
        if !self.advanced.is_empty() {
            let names: Vec<String> = self.advanced.iter().map(|q| s_or(&quest::resolved(&d, &gs, q), "name", q)).collect();
            lines.push(format!("Quests advanced: {}", names.join(", ")));
        }
        if lines.is_empty() {
            lines.push("No other rewards.".into());
        }
        for l in lines {
            ui.p.text(24.0, y as f32, &l, BODY + 1.0, if l.starts_with("No other") || l.starts_with("Missed") { th().dim } else { th().text });
            y += line;
        }
        if prompt {
            ui.button("results.level_up", Rect::new(24, y + 10, 90, 18), "Level up now", ResAct::LevelUp, true);
        }
        ui.button_full("results.continue", Rect::new(540, 300, 76, 18), "Continue", ResAct::Continue, true, true, BODY + 1.0, Vec::new());
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut Cx, k: Key) {
        if matches!(k, Key::Enter | Key::Space) {
            self.act(cx, ResAct::Continue);
        }
    }
    fn state(&self, _cx: &Cx) -> Value {
        json!({"outcome": self.outcome, "xp": self.award, "advanced": self.advanced, "gold": self.gold, "rewards": self.rewards, "stay_lines": self.stay_lines, "wounded": self.wounded, "departed": self.departed})
    }
    widget_plumbing!();
}

#[cfg(test)]
mod tests {
    use super::*;

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
    use aegis_ui::portrait_fx::EASE_MS;
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
