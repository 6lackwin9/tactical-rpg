//! The Character popup (party_model.gd with stats_model, level_up_model, loadout_model, gear_model): one member at a
//! time (Q / E cycle), tabs Stats, Level up, Skills and Gear. Drafts live here until Confirm writes them to the game
//! state (level-up points are permanent and asked about first); switching member or tab drops a draft after asking.

use aegis_ui::canvas::{rgba, Rect};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, DIM, GOLD, H, INFO, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s_or};
use aegis_data::STATS;
use aegis_rules::damage;
use aegis_rules::Unit;
use aegis_world::state::{UnitState, LOADOUT_SLOTS, POUCH_MAX};
use aegis_world::{progression, Data};
use indexmap::IndexMap;
use serde_json::{json, Value};

const TABS: [(&str, &str); 4] = [("stats", "Stats"), ("level_up", "Level up"), ("skills", "Skills"), ("gear", "Gear")];

#[derive(Debug, Clone)]
pub enum PAct {
    Member(i64),
    Tab(&'static str),
    Close,
    ToggleActive,
    Add(&'static str),
    Remove(&'static str),
    Auto,
    Revert,
    ConfirmLevel,
    Slot(String),
    Unslot(bool, usize),
    UseDefault,
    ResetSkills,
    ConfirmSkills,
    Unequip(String),
    Equip(String),
    PouchAdd(String),
    PouchRemove(usize),
    ResetGear,
    ConfirmGear,
}

pub struct Character {
    member: String,
    tab: &'static str,
    draft: IndexMap<String, i64>,
    class_slots: Vec<String>,
    extra_slots: Vec<String>,
    using_default: bool,
    skills_loaded: bool,
    gear: Option<(UnitState, IndexMap<String, i64>)>,
    pending_switch: Option<(Option<String>, Option<&'static str>)>,
    /// First family row shown on the Skills tab (the wheel scrolls).
    scroll: usize,
    detail: String,
    w: Vec<Widget<PAct>>,
}

/// PartyPreview.derived: [(label, value)] of a member built without auto-spend.
fn derived(d: &Data, m: &UnitState) -> Vec<(&'static str, i64)> {
    let Some(b) = progression::build_with(d, m, false) else { return Vec::new() };
    let u: &Unit = &b.unit;
    vec![
        ("Max HP", u.max_hp as i64),
        ("Speed", u.speed as i64),
        ("Tiles per move", u.move_per_block() as i64),
        ("Crit %", damage::crit_chance(u).round() as i64),
        ("Melee power", damage::attack_power(u, false, false).attack_power as i64),
        ("Ranged power", damage::attack_power(u, true, false).attack_power as i64),
        ("Armor", damage::armor_rating(u) as i64),
        ("Debuff resist %", u.debuff_resistance().round() as i64),
        ("AP", u.max_ap as i64),
    ]
}

fn fits(item: &aegis_data::ItemDef, slot: &str) -> bool {
    let up = slot.to_uppercase();
    item.is_equipment() && (item.slot == up || (up.starts_with("ACCESSORY") && item.slot.starts_with("ACCESSORY")) || (up == "OFF_HAND" && item.slot == "MAIN_HAND" && !item.two_handed && item.item_type == "WEAPON"))
}

impl Character {
    pub fn new(cx: &Cx, member: &str, tab: &str) -> Self {
        let gs = cx.gs();
        let first = gs.party.members.first().map(|m| m.id.clone()).unwrap_or_default();
        let member = if gs.party.member(member).is_some() { member.to_string() } else { first };
        let tab = TABS.iter().find(|t| t.0 == tab).map(|t| t.0).unwrap_or("stats");
        Self { member, tab, draft: IndexMap::new(), class_slots: Vec::new(), extra_slots: Vec::new(), using_default: true, skills_loaded: false, gear: None, pending_switch: None, scroll: 0, detail: String::new(), w: Vec::new() }
    }

    fn m<'a>(&self, cx: &'a Cx) -> &'a UnitState {
        cx.gs().party.member(&self.member).expect("member")
    }

    fn has_draft(&self, cx: &Cx) -> bool {
        match self.tab {
            "level_up" => self.draft.values().sum::<i64>() > 0,
            "skills" => {
                let m = self.m(cx);
                let (c, e) = if self.using_default { (Vec::new(), Vec::new()) } else { (self.class_slots.clone(), self.extra_slots.clone()) };
                self.skills_loaded && (c != m.loadout_class || e != m.loadout_extra)
            }
            "gear" => self.gear.as_ref().is_some_and(|(w, st)| w != self.m(cx) || *st != cx.gs().party.stash),
            _ => false,
        }
    }

    fn drop_drafts(&mut self) {
        self.draft.clear();
        self.skills_loaded = false;
        self.gear = None;
    }

    fn load_skills(&mut self, cx: &Cx) {
        if self.skills_loaded {
            return;
        }
        let m = self.m(cx).clone();
        self.using_default = m.loadout_class.is_empty() && m.loadout_extra.is_empty();
        if self.using_default {
            self.fill_default(cx);
        } else {
            self.class_slots = m.loadout_class.clone();
            self.extra_slots = m.loadout_extra.clone();
        }
        self.skills_loaded = true;
    }

    fn fill_default(&mut self, cx: &Cx) {
        let mut work = self.m(cx).clone();
        work.loadout_class.clear();
        work.loadout_extra.clear();
        if let Some(b) = progression::build_with(&cx.data, &work, false) {
            self.class_slots = b.unit.active_class_families.clone();
            self.extra_slots = b.unit.active_extra_families.clone();
        }
    }

    fn load_gear(&mut self, cx: &Cx) {
        if self.gear.is_none() {
            self.gear = Some((self.m(cx).clone(), cx.gs().party.stash.clone()));
        }
    }

    /// A member or tab switch; with a draft open it asks first (router.ask(DISCARD_QUESTION)) and applies on Yes.
    fn switch(&mut self, cx: &mut Cx, member: Option<String>, tab: Option<&'static str>) {
        if self.has_draft(cx) {
            self.pending_switch = Some((member, tab));
            cx.ask_tag("Discard the changes on this tab?", "character.discard");
        } else {
            self.apply_switch(member, tab);
        }
    }

    fn apply_switch(&mut self, member: Option<String>, tab: Option<&'static str>) {
        self.drop_drafts();
        if let Some(m) = member {
            self.member = m;
        }
        if let Some(t) = tab {
            self.tab = t;
        }
    }

    /// Answers to this screen's questions (taken on the frame after the question closes).
    fn take_answers(&mut self, cx: &mut Cx) {
        if cx.take_answer("character.discard") {
            if let Some((m, t)) = self.pending_switch.take() {
                self.apply_switch(m, t);
            }
        }
        if cx.take_answer("character.close") {
            cx.nav.push(Nav::Pop);
        }
        if cx.take_answer("character.spend") {
            let d = cx.data.clone();
            let draft: serde_json::Map<String, Value> = self.draft.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
            let id = self.member.clone();
            if let Some(m) = cx.gs_mut().party.member_mut(&id) {
                let errors = progression::confirm(&d, m, &draft);
                if errors.is_empty() {
                    self.draft.clear();
                } else {
                    show_errors(cx, errors);
                }
            }
        }
    }

    fn act(&mut self, cx: &mut Cx, a: PAct) {
        let d = cx.data.clone();
        match a {
            PAct::Member(step) => {
                let ids: Vec<String> = cx.gs().party.members.iter().map(|m| m.id.clone()).collect();
                if ids.len() < 2 {
                    return;
                }
                let at = ids.iter().position(|i| *i == self.member).unwrap_or(0) as i64;
                let next = ids[(at + step).rem_euclid(ids.len() as i64) as usize].clone();
                self.switch(cx, Some(next), None);
            }
            PAct::Tab(t) => {
                if t != self.tab {
                    self.switch(cx, None, Some(t));
                }
            }
            PAct::Close => {
                if self.has_draft(cx) {
                    cx.ask_tag("Discard the changes on this tab?", "character.close");
                } else {
                    cx.nav.push(Nav::Pop);
                }
            }
            PAct::ToggleActive => {
                let id = self.member.clone();
                let party = &mut cx.gs_mut().party;
                if party.active_ids.is_empty() {
                    party.active_ids = party.members.iter().take(party.max_active as usize).map(|m| m.id.clone()).collect();
                }
                if let Some(i) = party.active_ids.iter().position(|x| *x == id) {
                    if party.active_ids.len() <= 1 {
                        show_errors(cx, vec!["Keep at least one active member.".into()]);
                    } else {
                        party.active_ids.remove(i);
                    }
                } else if party.active_ids.len() as i64 >= party.max_active {
                    let cap = party.max_active;
                    show_errors(cx, vec![format!("The active party holds at most {cap}.")]);
                } else {
                    party.active_ids.push(id);
                }
            }
            PAct::Add(stat) => {
                let left = self.m(cx).unspent(&d) - self.draft.values().sum::<i64>();
                if left > 0 {
                    *self.draft.entry(stat.into()).or_insert(0) += 1;
                }
            }
            PAct::Remove(stat) => {
                if let Some(v) = self.draft.get_mut(stat) {
                    *v -= 1;
                    if *v <= 0 {
                        self.draft.shift_remove(stat);
                    }
                }
            }
            PAct::Auto => {
                let m = self.m(cx);
                let left = m.unspent(&d) - self.draft.values().sum::<i64>();
                if let (Some(arch), true) = (d.cat.classes.get(&m.class_id(&d)), left > 0) {
                    for (k, v) in aegis_rules::stats::auto_spend(&arch.growth, left as i32) {
                        if v > 0 {
                            *self.draft.entry(k).or_insert(0) += v as i64;
                        }
                    }
                }
            }
            PAct::Revert => self.draft.clear(),
            PAct::ConfirmLevel => {
                if self.draft.values().sum::<i64>() > 0 {
                    cx.ask_tag("Spend these points? This is permanent.", "character.spend");
                }
            }
            PAct::Slot(fam) => {
                self.load_skills(cx);
                self.using_default = false;
                let path: Vec<String> = {
                    let m = self.m(cx);
                    let arch = d.cat.classes.get(&m.class_id(&d));
                    arch.map(|a| aegis_rules::unit::families_of(&d.cat, &a.path_steps().into_iter().map(|(_, s)| s).collect::<Vec<_>>())).unwrap_or_default()
                };
                self.class_slots.retain(|f| *f != fam);
                self.extra_slots.retain(|f| *f != fam);
                if path.contains(&fam) {
                    self.class_slots.push(fam);
                } else {
                    self.extra_slots.push(fam);
                }
            }
            PAct::Unslot(class, i) => {
                self.load_skills(cx);
                self.using_default = false;
                let list = if class { &mut self.class_slots } else { &mut self.extra_slots };
                if i < list.len() {
                    list.remove(i);
                }
            }
            PAct::UseDefault => {
                self.using_default = true;
                self.fill_default(cx);
            }
            PAct::ResetSkills => self.skills_loaded = false,
            PAct::ConfirmSkills => {
                let mut work = self.m(cx).clone();
                if self.using_default {
                    work.loadout_class.clear();
                    work.loadout_extra.clear();
                } else {
                    work.loadout_class = self.class_slots.clone();
                    work.loadout_extra = self.extra_slots.clone();
                }
                let errors = work.loadout_errors(&d, &format!("Unit '{}'", work.id));
                if errors.is_empty() {
                    let id = self.member.clone();
                    if let Some(m) = cx.gs_mut().party.member_mut(&id) {
                        m.loadout_class = work.loadout_class;
                        m.loadout_extra = work.loadout_extra;
                    }
                    self.skills_loaded = false;
                } else {
                    show_errors(cx, errors);
                }
            }
            PAct::Unequip(slot) => {
                self.load_gear(cx);
                let (work, stash) = self.gear.as_mut().unwrap();
                if let Some(item) = work.equipment.shift_remove(&slot) {
                    *stash.entry(item).or_insert(0) += 1;
                } else {
                    show_errors(cx, vec![format!("The {} slot already holds the class default.", slot.replace('_', " "))]);
                }
            }
            PAct::Equip(item) => {
                self.load_gear(cx);
                let Some(def) = d.cat.items.get(&item).cloned() else { return };
                let (work, stash) = self.gear.as_mut().unwrap();
                let Some(slot) = LOADOUT_SLOTS.iter().find(|s| fits(&def, s) && !work.equipment.contains_key(**s)).or_else(|| LOADOUT_SLOTS.iter().find(|s| fits(&def, s))) else {
                    show_errors(cx, vec![format!("{} can't be worn.", def.name)]);
                    return;
                };
                if let Some(old) = work.equipment.insert(slot.to_string(), item.clone()) {
                    *stash.entry(old).or_insert(0) += 1;
                }
                take_one(stash, &item);
            }
            PAct::PouchAdd(item) => {
                self.load_gear(cx);
                let (work, stash) = self.gear.as_mut().unwrap();
                if work.pouch.len() >= POUCH_MAX {
                    show_errors(cx, vec![format!("The pouch holds at most {POUCH_MAX} items.")]);
                    return;
                }
                work.pouch.push(item.clone());
                take_one(stash, &item);
            }
            PAct::PouchRemove(i) => {
                self.load_gear(cx);
                let (work, stash) = self.gear.as_mut().unwrap();
                if i < work.pouch.len() {
                    let item = work.pouch.remove(i);
                    *stash.entry(item).or_insert(0) += 1;
                }
            }
            PAct::ResetGear => self.gear = None,
            PAct::ConfirmGear => {
                let Some((work, stash)) = self.gear.clone() else { return };
                let errors = work.validate(&d);
                if !errors.is_empty() {
                    show_errors(cx, errors);
                    return;
                }
                let id = self.member.clone();
                let gs = cx.gs_mut();
                if let Some(m) = gs.party.member_mut(&id) {
                    m.equipment = work.equipment;
                    m.pouch = work.pouch;
                }
                gs.party.stash = stash;
                self.gear = None;
            }
        }
    }
}

fn take_one(stash: &mut IndexMap<String, i64>, item: &str) {
    let left = stash.get(item).copied().unwrap_or(0) - 1;
    if left <= 0 {
        stash.shift_remove(item);
    } else {
        stash.insert(item.to_string(), left);
    }
}

impl Screen<GameCx> for Character {
    fn id(&self) -> &'static str {
        "character"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.take_answers(cx);
        let d = cx.data.clone();
        if cx.gs().party.member(&self.member).is_none() {
            self.member = cx.gs().party.members.first().map(|m| m.id.clone()).unwrap_or_default();
        }
        let m = self.m(cx).clone();
        let mut ui: Ui<PAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgba(0, 0, 0, 110));
        let fr = Rect::new(16, 16, 608, 328);
        ui.frame(fr, "Character", true);
        ui.close_x("character.close", Rect::new(fr.x + fr.w - 26, fr.y + 8, 19, 18), PAct::Close);
        let unspent = m.unspent(&d);
        let pick = format!("{}  Lv {}{}", crate::hub::member_name(cx, &m), m.level(&d), if unspent > 0 { format!("  +{unspent}") } else { String::new() });
        let pw = ui.p.text_w(&pick, BODY + 1.0) as i32 + 12;
        ui.p.text(320.0 - pw as f32 / 2.0 - 10.0, 57.0, "Q", BODY + 1.0, DIM);
        ui.button_full("character.member", Rect::new(320 - pw / 2, 53, pw, 17), &pick, PAct::Member(1), true, true, BODY + 1.0, tip("Next member (E)", "Q / E cycle the party."));
        ui.p.text(320.0 + pw as f32 / 2.0 + 4.0, 57.0, "E", BODY + 1.0, DIM);
        for (i, (id, label)) in TABS.iter().enumerate() {
            let label = if *id == "level_up" { format!("{label} ({unspent})") } else { label.to_string() };
            ui.tab(&format!("character.tab.{id}"), Rect::new(174 + i as i32 * 74, 78, 70, 18), &label, PAct::Tab(id), self.tab == *id);
        }
        self.detail.clear();
        match self.tab {
            "stats" => self.draw_stats(&mut ui, cx, &m),
            "level_up" => self.draw_level(&mut ui, cx, &m),
            "skills" => self.draw_skills(&mut ui, cx, &m),
            _ => self.draw_gear(&mut ui, cx, &m),
        }
        let dbox = Rect::new(25, 275, 590, 31);
        ui.p.panel(dbox, rgba(20, 28, 46, 255), BORDER, 2.0);
        let hovered = ui.widgets.iter().rev().find(|w| w.rect.contains(mouse.0, mouse.1) && w.tip.len() > 1).map(|w| w.tip[1].0.clone());
        let text = hovered.unwrap_or_else(|| if self.detail.is_empty() { "Move to a row to read what it does.".into() } else { self.detail.clone() });
        ui.para(30, 279, 580, &text, BODY, TEXT);
        self.w = ui.widgets;
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        match k {
            Key::Char('q') => self.act(cx, PAct::Member(-1)),
            Key::Char('e') => self.act(cx, PAct::Member(1)),
            _ => {}
        }
    }

    fn back(&mut self, cx: &mut Cx) {
        self.act(cx, PAct::Close);
    }

    fn wheel(&mut self, _cx: &mut Cx, lines: i32) {
        self.scroll = (self.scroll as i64 - lines as i64).max(0) as usize;
    }

    fn state(&self, cx: &Cx) -> Value {
        json!({"member": self.member, "tab": self.tab, "draft": self.draft.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<String, Value>>(), "has_draft": self.has_draft(cx)})
    }

    widget_plumbing!();
}

impl Character {
    fn draw_stats(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        let d = &cx.data;
        let gs = cx.gs();
        let lvl = m.level(d);
        ui.p.text(68.0, 104.0, &format!("Level {lvl}  ·  XP {} ({} to next)  ·  Gold {}", m.total_xp, progression::xp_to_next(d, m.total_xp), gs.party.gold), BODY + 1.0, TEXT);
        let active = gs.party.active_members().iter().any(|a| a.id == m.id);
        let n = gs.party.active_members().len();
        ui.checkbox("character.active", 72, 122, &format!("In party ({n}/{})", gs.party.max_active), active, PAct::ToggleActive, true);
        ui.p.text(155.0, 122.0, "Keep at least one active member.", BODY + 1.0, DIM);
        let Some(arch) = d.cat.classes.get(&m.class_id(d)) else { return };
        let mut spend: IndexMap<String, i32> = m.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
        if spend.is_empty() {
            spend.insert(STATS[0].into(), 0);
        }
        let cs = aegis_rules::stats::CharacterStats::new(arch, lvl as i32, &spend);
        let cols = [(68.0, "Stat"), (112.0, "Base"), (148.0, "Growth"), (190.0, "Spent"), (226.0, "Total")];
        for (x, h) in cols {
            ui.p.text(x, 143.0, h, BODY + 1.0, DIM);
        }
        for (i, st) in STATS.iter().enumerate() {
            let y = 158.0 + i as f32 * 15.0;
            let base = arch.base_stats.get(*st).copied().unwrap_or(0);
            let sp = m.spend.get(*st).copied().unwrap_or(0) as i32;
            let total = cs.base(st);
            ui.p.text(68.0, y, st, BODY + 1.0, TEXT);
            for (x, v) in [(126.0, base), (168.0, total - base - sp), (204.0, sp), (234.0, total)] {
                ui.p.text_right(x, y, &v.to_string(), BODY + 1.0, TEXT);
            }
        }
        for (i, (label, v)) in derived(d, m).iter().enumerate() {
            let y = 143.0 + i as f32 * 15.0;
            ui.p.text(250.0, y, label, BODY + 1.0, DIM);
            ui.p.text(325.0, y, &v.to_string(), BODY + 1.0, TEXT);
        }
        let names: Vec<String> = m.passives.iter().map(|p| s_or(d.entry("passives", p), "name", p)).collect();
        ui.p.text_fit(68.0, 258.0, &format!("Passives: {}", if names.is_empty() { "none".into() } else { names.join(", ") }), BODY + 1.0, 540.0, DIM);
    }

    fn draw_level(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        let d = &cx.data;
        let drafted: i64 = self.draft.values().sum();
        let left = m.unspent(d) - drafted;
        ui.p.text(70.0, 104.0, &format!("Points left: {left}   (confirmed points are permanent)"), BODY + 1.0, DIM);
        let Some(arch) = d.cat.classes.get(&m.class_id(d)) else { return };
        let mut spend: IndexMap<String, i32> = m.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
        spend.entry(STATS[0].into()).or_insert(0);
        let cs = aegis_rules::stats::CharacterStats::new(arch, m.level(d) as i32, &spend);
        for (x, h) in [(70.0, "Stat"), (105.0, "Now"), (132.0, "Added"), (165.0, "New")] {
            ui.p.text(x, 122.0, h, BODY + 1.0, DIM);
        }
        for (i, st) in STATS.iter().enumerate() {
            let y = 136 + i as i32 * 21;
            let now = cs.base(st) as i64;
            let add = self.draft.get(*st).copied().unwrap_or(0);
            ui.p.text(70.0, y as f32 + 4.0, st, BODY + 1.0, TEXT);
            ui.p.text_right(125.0, y as f32 + 4.0, &now.to_string(), BODY + 1.0, TEXT);
            ui.p.text_right(158.0, y as f32 + 4.0, &add.to_string(), BODY + 1.0, if add > 0 { GOLD } else { DIM });
            ui.p.text_right(187.0, y as f32 + 4.0, &(now + add).to_string(), BODY + 1.0, TEXT);
            ui.button(&format!("level.{st}.minus"), Rect::new(195, y, 23, 17), "-", PAct::Remove(st), add > 0);
            ui.button(&format!("level.{st}.plus"), Rect::new(226, y, 23, 17), "+", PAct::Add(st), left > 0);
        }
        ui.button("level.auto", Rect::new(266, 121, 69, 17), "Auto", PAct::Auto, left > 0);
        ui.button("level.revert", Rect::new(266, 145, 69, 17), "Revert", PAct::Revert, drafted > 0);
        let mut work = m.clone();
        for (k, v) in &self.draft {
            *work.spend.entry(k.clone()).or_insert(0) += v;
        }
        let (before, after) = (derived(d, m), derived(d, &work));
        for (i, (label, v)) in after.iter().enumerate() {
            let y = 122.0 + i as f32 * 15.0;
            let old = before.get(i).map(|b| b.1).unwrap_or(*v);
            ui.p.text(352.0, y, label, BODY + 1.0, DIM);
            ui.p.text(427.0, y, &if old != *v { format!("{old} -> {v}") } else { v.to_string() }, BODY + 1.0, if old != *v { GOLD } else { TEXT });
        }
        ui.button("level.confirm", Rect::new(500, 312, 70, 18), "Confirm", PAct::ConfirmLevel, drafted > 0);
    }

    fn draw_skills(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        self.load_skills(cx);
        let d = &cx.data;
        let cat = &d.cat;
        let Some(b) = progression::build_with(d, m, false) else { return };
        let u = &b.unit;
        ui.p.text(68.0, 104.0, if self.using_default { "Default loadout (picked by the class)" } else { "Custom loadout" }, BODY + 1.0, DIM);
        let fam_name = |f: &str| cat.tier_one(f).map(|s| s.name.clone()).unwrap_or_else(|| f.to_string());
        let mut y = 118;
        for (class, list, limit) in [(true, self.class_slots.clone(), aegis_rules::stats::MAX_CLASS_SLOTS), (false, self.extra_slots.clone(), aegis_rules::stats::MAX_EXTRA_SLOTS)] {
            if !class && list.is_empty() && u.owned_families(cat).iter().all(|f| self.class_slots.contains(f)) {
                continue;
            }
            ui.p.text(68.0, y as f32, &format!("{} slots {} / {limit}", if class { "Class" } else { "Extra" }, list.len()), BODY + 1.0, INFO);
            y += 13;
            let mut x = 68;
            for (i, f) in list.iter().enumerate() {
                let label = format!("{}  x", fam_name(f));
                let w = ui.p.text_w(&label, BODY) as i32 + 12;
                ui.button_tip(&format!("skills.chip.{f}"), Rect::new(x, y, w, 16), &label, PAct::Unslot(class, i), true, tip(&fam_name(f), "Click to take it out of the loadout."));
                x += w + 3;
            }
            y += 21;
        }
        let owned: Vec<String> = u.owned_families(cat).into_iter().filter(|f| !cat.is_basic_family(f)).collect();
        let rows = ((262 - y) / 22).max(1) as usize;
        self.scroll = self.scroll.min(owned.len().saturating_sub(rows));
        if self.scroll > 0 || owned.len() > rows {
            ui.p.text_right(604.0, y as f32 - 12.0, &format!("{}-{} of {} (wheel scrolls)", self.scroll + 1, (self.scroll + rows).min(owned.len()), owned.len()), SMALL, DIM);
        }
        for f in owned.iter().skip(self.scroll).take(rows) {
            let mut tiers: Vec<&aegis_data::SkillDef> = cat.skills.iter().filter(|s| cat.family_of(&s.id) == f).collect();
            tiers.sort_by_key(|s| s.tier);
            let active = tiers.iter().filter(|s| u.tier_unlocked(cat, s)).last().copied();
            let slotted = self.class_slots.contains(f) || self.extra_slots.contains(f);
            ui.button(&format!("skills.slot.{f}"), Rect::new(68, y, 60, 16), if slotted { "Slotted" } else { "Slot" }, PAct::Slot(f.clone()), !slotted);
            let grade: String = tiers.first().map(|s| { let mut c = s.grade.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).unwrap_or_default();
            let rank = active.map(|s| s.tier).unwrap_or(0);
            let desc = active.map(|s| json::s(&s.raw, "description")).unwrap_or_default();
            ui.p.text(134.0, y as f32 + 1.0, &format!("{}  [{}]  Rank {}", fam_name(f), grade, roman(rank)), BODY, TEXT);
            ui.p.text_fit(134.0, y as f32 + 10.0, &desc, SMALL, 470.0, DIM);
            ui.push(format!("skills.family.{f}"), "row", fam_name(f), Rect::new(134, y, 470, 20), true, slotted, None, tip(&fam_name(f), &desc));
            y += 22;
        }
        ui.button("skills.reset", Rect::new(68, 312, 70, 18), "Reset", PAct::ResetSkills, true);
        ui.button("skills.default", Rect::new(144, 312, 70, 18), "Use default", PAct::UseDefault, !self.using_default);
        ui.button("skills.confirm", Rect::new(498, 312, 70, 18), "Confirm", PAct::ConfirmSkills, self.has_draft(cx));
    }

    fn draw_gear(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        self.load_gear(cx);
        let d = &cx.data;
        let (work, stash) = self.gear.clone().unwrap();
        let built = progression::build_with(d, &work, false);
        let mut y = 104;
        for slot in LOADOUT_SLOTS {
            let explicit = work.equipment.get(slot).cloned();
            let shown = explicit.clone().or_else(|| {
                let s = aegis_rules::gear::Slot::parse(slot)?;
                built.as_ref().and_then(|b| b.unit.inventory.get(s).map(|e| e.item.id.clone()))
            });
            let name = shown.map(|i| d.item_name(&i)).unwrap_or_else(|| "empty".into());
            let label = format!("{}: {name}{}", title_case(slot), if explicit.is_some() { "" } else { "" });
            let act = explicit.as_ref().map(|_| PAct::Unequip(slot.to_string()));
            ui.row(&format!("gear.slot.{slot}"), Rect::new(64, y, 220, 14), &label, if explicit.is_some() { TEXT } else { DIM }, act, false, tip(&label, if explicit.is_some() { "Click to take it off (the class default comes back)." } else { "Class default gear." }));
            y += 15;
        }
        y += 6;
        ui.p.text(68.0, y as f32, "Battle pouch", BODY + 1.0, INFO);
        y += 14;
        for i in 0..POUCH_MAX {
            let label = match work.pouch.get(i) {
                Some(item) => format!("{}. {}", i + 1, d.item_name(item)),
                None => format!("{}. Empty", i + 1),
            };
            let act = work.pouch.get(i).map(|_| PAct::PouchRemove(i));
            ui.row(&format!("gear.pouch.{i}"), Rect::new(64, y, 220, 14), &label, TEXT, act, false, Vec::new());
            y += 15;
        }
        ui.p.text(294.0, 104.0, "Stash", BODY + 1.0, INFO);
        let mut sy = 118;
        let mut ids: Vec<&String> = stash.keys().collect();
        ids.sort();
        if ids.is_empty() {
            ui.p.text(294.0, sy as f32, "Empty", BODY + 1.0, DIM);
        }
        for id in ids {
            let Some(def) = d.cat.items.get(id) else { continue };
            let n = stash[id];
            let (act, help) = if def.is_equipment() {
                (Some(PAct::Equip(id.clone())), "Click to wear it.")
            } else if def.item_type == "CONSUMABLE" {
                (Some(PAct::PouchAdd(id.clone())), "Click to put one in the battle pouch.")
            } else {
                (None, "")
            };
            ui.row(&format!("gear.stash.{id}"), Rect::new(290, sy, 300, 14), &format!("{} x{n}", def.name), TEXT, act, false, tip(&def.name, help));
            sy += 15;
            if sy > 262 {
                break;
            }
        }
        let dirty = &work != m || stash != cx.gs().party.stash;
        ui.button("gear.reset", Rect::new(68, 312, 70, 18), "Reset", PAct::ResetGear, dirty);
        ui.button("gear.confirm", Rect::new(498, 312, 70, 18), "Confirm", PAct::ConfirmGear, dirty);
    }
}

fn title_case(slot: &str) -> String {
    slot.split('_').map(|w| {
        let mut c = w.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    }).collect::<Vec<_>>().join(" ")
}

fn roman(n: i32) -> &'static str {
    ["-", "I", "II", "III", "IV", "V"].get(n.max(0) as usize).copied().unwrap_or("V+")
}
