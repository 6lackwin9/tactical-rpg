//! The game menu over the map (hub_model.gd), the quest journal (journal_model.gd) and the battle results popup
//! (battle_results_model.gd). Read-only views over the game state, except the hub's doors (save, load, settings,
//! party, quit to the main menu).

use aegis_ui::canvas::{rgba, Rect};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, CARD, DIM, FOE, GOLD, GOOD, H, INFO, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::state::UnitState;
use emberwake_world::{progression, quest, save};
use serde_json::{json, Value};

pub fn member_name(cx: &Cx, m: &UnitState) -> String {
    progression::display_name(&cx.data, m)
}

/// The member's class name in the world being played (HOOKS1 CN2: the world's `classes` overlay, else the class row's).
pub fn class_name(cx: &Cx, m: &UnitState) -> String {
    cx.data.class_name(&cx.gs().world_id, &m.class_id(&cx.data))
}

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
        out.push(format!("{gold} gold"));
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
        ui.p.fill(Rect::new(0, 0, W, H), rgba(0, 0, 0, 110));
        let gs = cx.gs().clone();
        let fr = Rect::new(95, 25, 450, 310);
        ui.frame(fr, &cx.world_name(&gs.world_id), true);
        ui.p.text_center(320.0, 62.0, &cx.location_name(&gs.world_id, &gs.location_id), BODY + 1.0, DIM);
        let active: Vec<String> = gs.party.active_members().iter().map(|m| m.id.clone()).collect();
        let mut rows: Vec<&UnitState> = gs.party.members.iter().filter(|m| active.contains(&m.id)).collect();
        rows.extend(gs.party.members.iter().filter(|m| !active.contains(&m.id)));
        let bh = 4 + rows.len() as i32 * 42;
        let bx = Rect::new(270, 80, 100, bh);
        ui.p.panel(bx, CARD, BORDER, 2.0);
        for (i, m) in rows.iter().enumerate() {
            let (hp, max) = progression::current_hp(&cx.data, m, &gs.world_id);
            let y = bx.y as f32 + 3.0 + i as f32 * 42.0;
            let on = active.contains(&m.id);
            ui.p.text_fit(bx.x as f32 + 3.0, y, &member_name(cx, m), BODY + 1.0, 94.0, if on { INFO } else { DIM });
            ui.p.text(bx.x as f32 + 3.0, y + 13.0, &format!("{}  Lv {}", class_name(cx, m), m.level(&cx.data)), BODY + 1.0, TEXT);
            ui.p.text(bx.x as f32 + 3.0, y + 26.0, &format!("HP {hp} / {max}"), BODY + 1.0, TEXT);
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
        let top = bx.y + bh + 10;
        let mut detail = "Roster, level up, skills and gear.".to_string();
        for (i, (a, id, label, desc)) in entries.iter().enumerate() {
            let r = Rect::new(if i % 2 == 0 { 187 } else { 323 }, top + (i as i32 / 2) * 24, 130, 18);
            let why = Self::reason(cx, *a);
            ui.button_full(id, r, label, *a, true, false, BODY + 1.0, if why.is_empty() { tip(label, desc) } else { vec![(label.clone(), GOLD), (why.clone(), FOE)] });
            if ui.hover(r) {
                detail = if why.is_empty() { desc.to_string() } else { why };
            }
        }
        ui.p.text_center(320.0, (top + 76) as f32, &detail, BODY + 1.0, DIM);
        ui.button("hub.return", Rect::new(110, 303, 75, 18), "Return to map", HubAct::Return, true);
        self.w = ui.widgets;
    }
    fn resume(&mut self, _cx: &mut Cx) {}
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
        ui.p.fill(Rect::new(0, 0, W, H), rgba(0, 0, 0, 110));
        let fr = Rect::new(16, 16, 608, 328);
        ui.frame(fr, "Journal", false);
        let groups = Self::shown(cx);
        let all: Vec<String> = groups.iter().flat_map(|g| g.2.clone()).collect();
        if !all.contains(&self.selected) {
            self.selected = all.first().cloned().unwrap_or_default();
        }
        let mut y = 50;
        for (_, label, ids) in &groups {
            ui.p.text(25.0, y as f32, label, BODY + 1.0, INFO);
            y += 13;
            for q in ids {
                let name = s_or(&quest::resolved(&d, &gs, q), "name", q);
                ui.button_full(&format!("journal.quest.{}", q.replace(':', ".")), Rect::new(25, y, 130, 18), &format!("> {name}"), JAct::Select(q.clone()), true, *q == self.selected, BODY + 1.0, Vec::new());
                y += 22;
            }
        }
        if all.is_empty() {
            ui.p.text(25.0, 50.0, "No quests yet.", BODY + 1.0, DIM);
        }
        if !self.selected.is_empty() {
            let q = quest::resolved(&d, &gs, &self.selected);
            let st = quest::status(&d, &gs, &self.selected);
            let (x, w) = (221, 390);
            let mut y = 50;
            ui.p.text(x as f32, y as f32, &s_or(&q, "name", &self.selected), BODY + 1.0, GOLD);
            y += 15;
            y = ui.para(x, y, w, &s(&q, "description"), BODY + 1.0, TEXT) + 4;
            if st != "blocked" {
                let char_name = |r: &str| if r.is_empty() { String::new() } else { s_or(d.resolve(&gs.world_id, "characters", r), "name", &format!("[{r}]")) };
                let giver = char_name(&s(&q, "giver"));
                if !giver.is_empty() {
                    ui.p.text(x as f32, y as f32, &format!("From {giver}"), BODY + 1.0, DIM);
                    y += 15;
                }
                let total = json::aget(&q, "objectives").len() as i64;
                let done = if st == "completed" { total } else { total.min(json::i(json::dict(gs.quests.get(&self.selected).unwrap_or(&Value::Null)), "objective", 0)) };
                ui.p.text(x as f32, y as f32, &format!("Progress {done} / {total}"), BODY + 1.0, DIM);
                y += 15;
                let obj = s(&quest::current_objective(&d, &gs, &self.selected), "text");
                if !obj.is_empty() {
                    y = ui.para(x, y, w, &format!("Now: {obj}"), BODY + 1.0, GOOD);
                }
                if st == "ready" {
                    let to = char_name(&s(&q, "turn_in"));
                    ui.p.text(x as f32, y as f32, &if to.is_empty() { "Ready to turn in.".to_string() } else { format!("Turn in to {to}.") }, BODY + 1.0, GOOD);
                    y += 15;
                }
                let rewards = reward_lines(cx, json::dget(&q, "rewards"));
                if !rewards.is_empty() {
                    y = ui.para(x, y, w, &format!("Rewards: {}", rewards.join(", ")), BODY + 1.0, TEXT);
                }
                if st == "active" || st == "ready" {
                    let offers: Vec<String> = quest::offers(&d, &gs, &self.selected).iter().map(|l| s_or(d.resolve(&gs.world_id, "loot", l), "name", &format!("[{l}]"))).collect();
                    let picks = quest::picks_allowed(&d, &gs, &self.selected);
                    if picks > 0 && !offers.is_empty() {
                        ui.para(x, y, w, &format!("Choose {picks} of: {}", offers.join(", ")), BODY + 1.0, TEXT);
                    }
                }
            } else {
                ui.p.text(x as f32, y as f32, "On hold.", BODY + 1.0, DIM);
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
    w: Vec<Widget<ResAct>>,
}

/// REWARD1 section 7: the tally line ("Foes defeated: 3 of 5 (2 slain, 1 surrendered)."), the outcome's note, and the
/// reward lines (share gold, objective bonus, drops merged by item, key loot, missed key loot); `name(kind, id)` gives
/// an item's, a loot entry's or a character's name.
pub fn reward_text(rewards: &Dict, outcome: &str, name: &dyn Fn(&str, &str) -> String) -> (String, String, Vec<String>) {
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
        lines.push(format!("Gold +{gold}"));
    }
    let objective = json::dget(rewards, "objective");
    let mut bonus = Vec::new();
    if json::i(objective, "gold", 0) > 0 {
        bonus.push(format!("Gold +{}", json::i(objective, "gold", 0)));
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
        ui.p.fill(Rect::new(0, 0, W, H), rgba(10, 14, 24, 235));
        let head = match self.outcome.as_str() {
            "won" => "Victory",
            "lost" => "Defeat",
            "fled" => "Fled",
            o => o,
        };
        ui.p.text(24.0, 24.0, head, aegis_ui::ui::HEAD + 4.0, GOLD);
        let world = gs.world_id.clone();
        let name = |kind: &str, id: &str| -> String {
            match kind {
                "items" => d.item_name(id),
                _ => s_or(d.resolve(&world, kind, id), "name", id),
            }
        };
        let (tally, note, reward_lines) = reward_text(&self.rewards, &self.outcome, &name);
        let line = (ui.p.line_h(BODY + 1.0) + 3.0).ceil() as i32;
        let mut y = 50;
        for (text, col) in [(&tally, TEXT), (&note, DIM)] {
            if !text.is_empty() {
                ui.p.text(24.0, y as f32, text, BODY + 1.0, col);
                y += line;
            }
        }
        let mut prompt = false;
        for m in &gs.party.members {
            let Some(a) = self.award.get(&m.id).and_then(Value::as_object) else { continue };
            let level = m.level(&d);
            let base = progression::xp_for_level(&d, level);
            let span = progression::xp_for_level(&d, level + 1) - base;
            let fill = if span <= 0 { 1.0 } else { ((m.total_xp - base) as f32 / span as f32).clamp(0.0, 1.0) };
            let (old, new) = (json::i(a, "old_level", 0), json::i(a, "new_level", 0));
            ui.p.text(24.0, y as f32, &member_name(cx, m), BODY + 1.0, INFO);
            ui.p.text(130.0, y as f32, &format!("+{} XP", json::i(a, "xp", 0)), BODY + 1.0, TEXT);
            ui.p.text(190.0, y as f32, &if new > old { format!("Level {old} -> {new}!") } else { format!("Level {level}") }, BODY + 1.0, if new > old { GOOD } else { TEXT });
            let bar = Rect::new(270, y + 3, 150, 5);
            ui.p.panel(bar, rgba(40, 46, 60, 255), rgba(40, 46, 60, 255), 2.0);
            ui.p.panel(Rect::new(bar.x, bar.y, (150.0 * fill) as i32, 5), GOLD, GOLD, 2.0);
            ui.p.text(426.0, y as f32, &format!("{} to next", progression::xp_to_next(&d, m.total_xp)), SMALL, DIM);
            if new > old && m.unspent(&d) > 0 {
                prompt = true;
            }
            y += 16;
        }
        ui.p.text(24.0, (y + 4) as f32, "Rewards", SMALL, INFO);
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
            ui.p.text(24.0, y as f32, &l, BODY + 1.0, if l.starts_with("No other") || l.starts_with("Missed") { DIM } else { TEXT });
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
        json!({"outcome": self.outcome, "xp": self.award, "advanced": self.advanced, "gold": self.gold, "rewards": self.rewards})
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
        let (tally, note, lines) = reward_text(&rewards(2, 1, 0, json!({})), "lost", &name);
        assert_eq!(tally, "Foes defeated: 3 of 5 (2 slain, 1 surrendered).");
        assert_eq!(note, "The party was beaten, but keeps the rewards of the foes it defeated.");
        assert!(lines.is_empty());
        assert_eq!(reward_text(&rewards(5, 0, 0, json!({})), "won", &name).0, "Foes defeated: 5 of 5.");
        assert_eq!(reward_text(&rewards(0, 0, 1, json!({})), "fled", &name).0, "Foes defeated: 1 of 5 (1 fled).");
        assert_eq!(reward_text(&rewards(0, 0, 0, json!({})), "lost", &name).1, "The party was beaten before it defeated a foe. No rewards.");
        assert_eq!(reward_text(&rewards(1, 0, 0, json!({})), "fled", &name).1, "The party got away with the rewards of the foes it defeated.");
        assert_eq!(reward_text(&rewards(0, 0, 0, json!({})), "fled", &name).1, "The party got away. No foe was defeated, so no rewards.");
        assert_eq!(reward_text(&rewards(5, 0, 0, json!({})), "won", &name).1, "");
        assert_eq!(reward_text(&json::obj([("foes", json!(0))]), "won", &name).0, "", "no roster, no tally line");
        let full = rewards(5, 0, 0, json!({
            "gold": 12,
            "objective": {"gold": 10, "loot": "ashlands:keep_ring"},
            "drops": [{"item": "wpn_shortbow", "count": 1}, {"item": "con_heal_potion", "count": 1}, {"item": "con_heal_potion", "count": 1}],
            "key_loot": ["ashlands:warchiefs_axe"],
            "key_missed": [{"loot": "ashlands:warchiefs_axe", "kill": "ash_warchief"}],
        }));
        assert_eq!(reward_text(&full, "won", &name).2, [
            "Gold +12",
            "Objective bonus: Gold +10, Keep Ring",
            "Dropped: Shortbow, Healing Potion x2",
            "Key loot: Warchief's Axe",
            "Missed: Warchief's Axe (slay Horde Warchief and win)",
        ]);
    }
}
