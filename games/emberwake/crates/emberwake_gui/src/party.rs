//! The Character popup (party_model.gd with stats_model, level_up_model, loadout_model, gear_model): one member at a
//! time (Q / E cycle), tabs Stats, Level up, Skills and Gear. Drafts live here until Confirm writes them to the game
//! state (level-up points are permanent and asked about first); switching member or tab drops a draft after asking.

use aegis_ui::canvas::{rgba, Rect};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, DIM, FOE, GOLD, GOOD, H, INFO, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s_or};
use emberwake_data::STATS;
use emberwake_rules::damage;
use emberwake_rules::Unit;
use emberwake_world::state::{off_hand_ok, UnitState, LOADOUT_SLOTS, POUCH_MAX};
use emberwake_world::{progression, Data};
use indexmap::IndexMap;
use serde_json::{json, Value};

const TABS: [(&str, &str); 4] = [("stats", "Stats"), ("level_up", "Level up"), ("skills", "Skills"), ("gear", "Gear")];

/// A Skills tab draft: class slots, extra slots, the chosen rank per slotted family.
type SkillDraft = (Vec<String>, Vec<String>, IndexMap<String, i64>);

/// SKILLUI1 R2 on the draft: no entry, the highest selectable rank; an entry `c`, the highest selectable rank <= `c`,
/// else the lowest; 0 when nothing is selectable.
fn draft_rank(selectable: &[i32], chosen: Option<i32>) -> i32 {
    match chosen {
        None => selectable.last().copied().unwrap_or(0),
        Some(c) => selectable.iter().rev().find(|&&r| r <= c).or(selectable.first()).copied().unwrap_or(0),
    }
}

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
    /// SKILLUI1: step a filled slot (class?, index) to the previous (-1) or next (+1) selectable rank.
    RankStep(bool, usize, i32),
    ClearAll,
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
    /// SKILLUI1: the draft's chosen rank per slotted family (family -> rank).
    ranks: IndexMap<String, i64>,
    using_default: bool,
    /// The mouse at the last draw (a right click unslots the slot under it).
    mouse: (i32, i32),
    skills_loaded: bool,
    gear: Option<(UnitState, IndexMap<String, i64>)>,
    pending_switch: Option<(Option<String>, Option<&'static str>)>,
    /// First family row shown on the Skills tab (the wheel scrolls).
    scroll: usize,
    /// First stash row shown on the Gear tab (the wheel scrolls).
    gear_scroll: usize,
    detail: String,
    w: Vec<Widget<PAct>>,
}

/// PartyPreview.derived: [(label, value)] of a member built without auto-spend.
fn derived(d: &Data, m: &UnitState) -> Vec<(&'static str, i64)> {
    progression::build_with(d, m, false).map(|b| derived_of(&b.unit)).unwrap_or_default()
}

/// The derived values of a built unit.
fn derived_of(u: &Unit) -> Vec<(&'static str, i64)> {
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

/// Table columns at the current text size: each header keeps its position at normal size unless the previous header
/// would run into it (then it moves right by `gap`); (x, right edge), numbers are right-aligned to the right edge.
fn columns(ui: &mut Ui<PAct>, cols: &[(f32, &str)], gap: f32) -> Vec<(f32, f32)> {
    let mut out: Vec<(f32, f32)> = Vec::new();
    for &(x, h) in cols {
        let x = out.last().map_or(x, |p| x.max(p.1 + gap));
        out.push((x, x + ui.p.text_w(h, BODY + 1.0)));
    }
    out
}

/// What a core stat does, for the detail pane (numbers from the rules: StatRules, DamageCalculator).
fn stat_desc(st: &str) -> String {
    use emberwake_rules::damage::{CRIT_DEX_FLOOR, CRIT_PER_DEX};
    use emberwake_rules::stats::{HP_PER_CONS, KNOW_UNLOCK_BASE, KNOW_UNLOCK_STEP, MAX_UNLOCK_REDUCTION, RANGE_PERC_THRESHOLD};
    match st {
        "STR" => "Strength: power with STR weapons, +2 max HP per point, +1 armor per 4 points; unarmed hits use STR + DEX.".into(),
        "DEX" => format!("Dexterity: speed (8 + DEX - gear burden), crit chance +{CRIT_PER_DEX}% per point above {CRIT_DEX_FLOOR}, power with DEX weapons and of shots without a ranged weapon."),
        "INT" => "Intelligence: debuff resistance (2.5% per point) and power with INT weapons.".into(),
        "CONS" => format!("Constitution: +{HP_PER_CONS} max HP per point."),
        "PERC" => format!("Perception: at {RANGE_PERC_THRESHOLD} or more, ranged weapons reach 1 tile further; power with PERC weapons."),
        "KNOW" => format!("Knowledge: from {KNOW_UNLOCK_BASE}, every {KNOW_UNLOCK_STEP} points open skill families one level earlier (up to {MAX_UNLOCK_REDUCTION} levels)."),
        _ => String::new(),
    }
}

/// What a derived value (`derived` labels) means, for the detail pane.
fn derived_desc(label: &str) -> String {
    use emberwake_rules::damage::{ARMOR_K, CRIT_BASE, CRIT_DEX_FLOOR, CRIT_PER_DEX};
    use emberwake_rules::stats::HP_PER_CONS;
    match label {
        "Max HP" => format!("Health: 60 + {HP_PER_CONS} per CONS + 2 per STR."),
        "Speed" => "Initiative: 8 + DEX - the burden of worn gear; faster units act earlier and move further.".into(),
        "Tiles per move" => "Tiles one move covers: (speed + 2) / 5, from 1 to 4; haste adds 1, slow takes 1.".into(),
        "Crit %" => format!("Critical hit chance: {CRIT_BASE}% + {CRIT_PER_DEX}% per DEX above {CRIT_DEX_FLOOR}, plus the weapon proficiency bonus."),
        "Melee power" => "Melee hit power: the weapon's power (with tier and proficiency bonuses) + its scaling stat x your proficiency scaling.".into(),
        "Ranged power" => "Ranged hit power, the same way with a ranged weapon; without one, DEX alone.".into(),
        "Armor" => format!("Armor rating: gear mitigation + STR / 4; it cuts damage taken by rating / (rating + {ARMOR_K})."),
        "Debuff resist %" => "Chance to shake off a debuff: 2.5% per INT, plus the unit's own resistances.".into(),
        "AP" => "Action points each round; a skill costs its level in AP.".into(),
        _ => String::new(),
    }
}

fn fits(item: &emberwake_data::ItemDef, slot: &str) -> bool {
    emberwake_world::state::item_fits_slot(item, slot)
}

impl Character {
    pub fn new(cx: &Cx, member: &str, tab: &str) -> Self {
        let gs = cx.gs();
        let first = gs.party.members.first().map(|m| m.id.clone()).unwrap_or_default();
        let member = if gs.party.member(member).is_some() { member.to_string() } else { first };
        let tab = TABS.iter().find(|t| t.0 == tab).map(|t| t.0).unwrap_or("stats");
        Self { member, tab, draft: IndexMap::new(), class_slots: Vec::new(), extra_slots: Vec::new(), ranks: IndexMap::new(), using_default: true, mouse: (0, 0), skills_loaded: false, gear: None, pending_switch: None, scroll: 0, gear_scroll: 0, detail: String::new(), w: Vec::new() }
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
                self.skills_loaded && (c != m.loadout_class || e != m.loadout_extra || self.ranks != m.loadout_ranks)
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
        self.ranks = m.loadout_ranks.clone();
        self.skills_loaded = true;
    }

    /// The member's class path families (class slots take only these, SKILLUI1 Q3 A).
    fn path_families(&self, cx: &Cx) -> Vec<String> {
        let d = &cx.data;
        let m = self.m(cx);
        d.cat.classes.get(&m.class_id(d)).map(|a| emberwake_rules::unit::families_of(&d.cat, &a.path_steps().into_iter().map(|(_, s)| s).collect::<Vec<_>>())).unwrap_or_default()
    }

    /// The highest rank of `family` the member has unlocked (with confirmed gear): what a newly slotted skill stores.
    fn top_rank(&self, cx: &Cx, family: &str) -> Option<i64> {
        progression::build_with(&cx.data, self.m(cx), false).and_then(|b| b.unit.selectable_ranks(&cx.data.cat, family).last().map(|&r| r as i64))
    }

    /// The draft after dropping payload `from` (`skill.<f>` from the list, `slot.<g>.<i>` from a slot) on target `to`
    /// (`slot.<g>.<i>` or `list`), or why it can't be done (SKILLUI1 section 4 drop table).
    fn skills_after_drop(&self, cx: &Cx, from: &str, to: &str) -> Result<SkillDraft, String> {
        let path = self.path_families(cx);
        let (mut class, mut extra, mut ranks) = (self.class_slots.clone(), self.extra_slots.clone(), self.ranks.clone());
        let slot_of = |key: &str| -> Option<(bool, usize)> {
            let rest = key.strip_prefix("slot.")?;
            let (g, i) = rest.split_once('.')?;
            Some((g == "class", i.parse().ok()?))
        };
        let fits = |fam: &str, class_group: bool| !class_group || path.iter().any(|p| p == fam);
        if let Some(fam) = from.strip_prefix("skill.") {
            let (to_class, i) = slot_of(to).ok_or("Drop a skill on a slot.")?;
            // A family already slotted moves from its own slot (the slot-to-slot rules: swap or move, never a shifted
            // index), whether it was dragged from the list or from its slot.
            if let Some(j) = class.iter().position(|f| f == fam) {
                return self.skills_after_drop(cx, &format!("slot.class.{j}"), to);
            }
            if let Some(j) = extra.iter().position(|f| f == fam) {
                return self.skills_after_drop(cx, &format!("slot.extra.{j}"), to);
            }
            if !fits(fam, to_class) {
                return Err("Class slots take class path skills only.".into());
            }
            let list = if to_class { &mut class } else { &mut extra };
            match list.get(i).cloned() {
                Some(old) => {
                    list[i] = fam.to_string();
                    ranks.shift_remove(&old);
                }
                None => {
                    let cap = if to_class { emberwake_rules::stats::MAX_CLASS_SLOTS } else { emberwake_rules::stats::MAX_EXTRA_SLOTS };
                    if list.len() >= cap {
                        return Err("That group is full.".into());
                    }
                    list.push(fam.to_string());
                }
            }
            // the user's Q2 answer: a newly slotted skill stores its highest unlocked rank now
            if let Some(r) = self.top_rank(cx, fam) {
                ranks.insert(fam.to_string(), r);
            }
            return Ok((class, extra, ranks));
        }
        let (from_class, i) = slot_of(from).ok_or("That can't go there.")?;
        let fam = (if from_class { &class } else { &extra }).get(i).cloned().ok_or("That slot is empty.")?;
        if to == "list" {
            (if from_class { &mut class } else { &mut extra }).remove(i);
            ranks.shift_remove(&fam);
            return Ok((class, extra, ranks));
        }
        let (to_class, j) = slot_of(to).ok_or("That can't go there.")?;
        if (from_class, i) == (to_class, j) {
            return Err("Same slot.".into());
        }
        let other = (if to_class { &class } else { &extra }).get(j).cloned();
        if !fits(&fam, to_class) || other.as_ref().is_some_and(|o| !fits(o, from_class)) {
            return Err("Class slots take class path skills only.".into());
        }
        match other {
            Some(o) => {
                (if from_class { &mut class } else { &mut extra })[i] = o;
                (if to_class { &mut class } else { &mut extra })[j] = fam;
            }
            None => {
                let cap = if to_class { emberwake_rules::stats::MAX_CLASS_SLOTS } else { emberwake_rules::stats::MAX_EXTRA_SLOTS };
                (if from_class { &mut class } else { &mut extra }).remove(i);
                let list = if to_class { &mut class } else { &mut extra };
                if list.len() >= cap {
                    return Err("That group is full.".into());
                }
                list.push(fam);
            }
        }
        Ok((class, extra, ranks))
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
                    for (k, v) in emberwake_rules::stats::auto_spend(&arch.growth, left as i32) {
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
                // SKILLUI1 click path: the first free slot of the family's group (a class path family may use a free
                // extra slot when the class slots are full, Q3 A); a full group says how to make room.
                self.load_skills(cx);
                if self.class_slots.contains(&fam) || self.extra_slots.contains(&fam) {
                    return;
                }
                let on_path = self.path_families(cx).contains(&fam);
                let class_free = self.class_slots.len() < emberwake_rules::stats::MAX_CLASS_SLOTS;
                let extra_free = self.extra_slots.len() < emberwake_rules::stats::MAX_EXTRA_SLOTS;
                let to_class = if on_path && class_free {
                    true
                } else if extra_free {
                    false
                } else {
                    let group = if on_path { "Class" } else { "Extra" };
                    cx.notice.push(format!("{group} slots are full: drag onto a slot to replace one, or right click a slot to take it out."));
                    return;
                };
                self.using_default = false;
                if to_class {
                    self.class_slots.push(fam.clone());
                } else {
                    self.extra_slots.push(fam.clone());
                }
                // the user's Q2 answer: a newly slotted skill stores its highest unlocked rank now
                if let Some(r) = self.top_rank(cx, &fam) {
                    self.ranks.insert(fam, r);
                }
            }
            PAct::Unslot(class, i) => {
                self.load_skills(cx);
                self.using_default = false;
                let list = if class { &mut self.class_slots } else { &mut self.extra_slots };
                if i < list.len() {
                    let fam = list.remove(i);
                    self.ranks.shift_remove(&fam);
                }
            }
            PAct::RankStep(class, i, step) => {
                self.load_skills(cx);
                let Some(fam) = (if class { &self.class_slots } else { &self.extra_slots }).get(i).cloned() else { return };
                let Some(b) = progression::build_with(&d, self.m(cx), false) else { return };
                let ranks = b.unit.selectable_ranks(&d.cat, &fam);
                let now = draft_rank(&ranks, self.ranks.get(&fam).map(|&r| r as i32));
                let at = ranks.iter().position(|&r| r == now).unwrap_or(ranks.len().saturating_sub(1)) as i32;
                if let Some(&next) = ranks.get((at + step).max(0) as usize).filter(|_| at + step >= 0) {
                    self.ranks.insert(fam, next as i64);
                }
            }
            PAct::ClearAll => {
                self.load_skills(cx);
                self.using_default = false;
                self.class_slots.clear();
                self.extra_slots.clear();
                self.ranks.clear();
            }
            PAct::UseDefault => {
                self.using_default = true;
                self.fill_default(cx);
                let keep: Vec<String> = self.class_slots.iter().chain(&self.extra_slots).cloned().collect();
                self.ranks.retain(|f, _| keep.contains(f));
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
                // ranks for the slotted families only; every slotted family gets an entry (the user's Q2 answer)
                let slotted: Vec<String> = self.class_slots.iter().chain(&self.extra_slots).cloned().collect();
                work.loadout_ranks = self.ranks.iter().filter(|(f, _)| slotted.contains(f)).map(|(f, r)| (f.clone(), *r)).collect();
                let errors = work.loadout_errors(&d, &format!("Unit '{}'", work.id));
                if errors.is_empty() {
                    progression::pin_ranks(&d, &mut work);
                    let id = self.member.clone();
                    if let Some(m) = cx.gs_mut().party.member_mut(&id) {
                        m.loadout_class = work.loadout_class;
                        m.loadout_extra = work.loadout_extra;
                        m.loadout_ranks = work.loadout_ranks;
                    }
                    self.skills_loaded = false;
                } else {
                    show_errors(cx, errors);
                }
            }
            PAct::Unequip(slot) => self.gear_move(cx, |d, work, stash| take_off(d, work, stash, &slot)),
            PAct::Equip(item) => self.gear_move(cx, |d, work, stash| equip(d, work, stash, &item, None)),
            PAct::PouchAdd(item) => self.gear_move(cx, |d, work, stash| pouch_put(d, work, stash, &item, None)),
            PAct::PouchRemove(i) => self.gear_move(cx, |_, work, stash| pouch_take(work, stash, i)),
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

impl Character {
    /// One gear move on the draft (loaded first); a refused move shows its reason.
    fn gear_move(&mut self, cx: &mut Cx, mv: impl FnOnce(&Data, &mut UnitState, &mut Stash) -> Result<(), String>) {
        self.load_gear(cx);
        let d = cx.data.clone();
        let (work, stash) = self.gear.as_mut().unwrap();
        if let Err(e) = mv(&d, work, stash) {
            show_errors(cx, vec![e]);
        }
    }

    /// The draft after the drop of payload `from` on target `to` (GEARUI1 drag and drop), or why it can't be done.
    fn after_drop(&self, d: &Data, from: &str, to: &str) -> Result<(UnitState, Stash), String> {
        let (mut work, mut stash) = self.gear.clone().ok_or("No gear draft.")?;
        drop_move(d, &mut work, &mut stash, from, to)?;
        Ok((work, stash))
    }
}

/// The party stash: item id -> count.
type Stash = IndexMap<String, i64>;

fn take_one(stash: &mut Stash, item: &str) {
    let left = stash.get(item).copied().unwrap_or(0) - 1;
    if left <= 0 {
        stash.shift_remove(item);
    } else {
        stash.insert(item.to_string(), left);
    }
}

fn give_one(stash: &mut Stash, item: String) {
    *stash.entry(item).or_insert(0) += 1;
}

// ---- Gear moves on a draft (GearModel equip / unequip / pouch): clicks, drops and the stat preview share them ----

/// Wears `item` from the stash in `slot`, or (None) the first free slot it fits, else the first it fits; what the slot
/// held goes back to the stash.
fn equip(d: &Data, work: &mut UnitState, stash: &mut Stash, item: &str, slot: Option<&str>) -> Result<(), String> {
    let def = d.cat.items.get(item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if stash.get(item).copied().unwrap_or(0) < 1 {
        return Err(format!("There is no {} in the stash.", def.name));
    }
    let slot = match slot {
        Some(s) if fits(def, s) => s,
        Some(s) => return Err(format!("{} doesn't go in the {} slot.", def.name, title_case(s).to_lowercase())),
        None => *LOADOUT_SLOTS.iter().find(|s| fits(def, s) && !work.equipment.contains_key(**s)).or_else(|| LOADOUT_SLOTS.iter().find(|s| fits(def, s))).ok_or_else(|| format!("{} can't be worn.", def.name))?,
    };
    work.can_wear(d, item, slot)?;
    if slot == "off_hand" {
        if let Some(main) = main_weapon(d, work).filter(|m| !off_hand_ok(Some(m), def)) {
            return Err(format!("{} needs both hands: the off hand can hold only a weapon with it.", main.name));
        }
    }
    if let Some(old) = work.equipment.insert(slot.to_string(), item.to_string()) {
        give_one(stash, old);
    }
    take_one(stash, item);
    if slot == "main_hand" && def.two_handed {
        // A shield or focus goes back to the stash (the Godot dojo's "stowed: ... is 2-Handed").
        if let Some(off) = work.equipment.get("off_hand").cloned().filter(|o| d.cat.items.get(o).is_some_and(|o| !off_hand_ok(Some(def), o))) {
            work.equipment.shift_remove("off_hand");
            give_one(stash, off);
        }
    }
    Ok(())
}

/// The weapon the main hand holds: the worn one, else the class default the build gives.
fn main_weapon(d: &Data, work: &UnitState) -> Option<emberwake_data::ItemDef> {
    match work.equipment.get("main_hand") {
        Some(id) => d.cat.items.get(id).cloned(),
        None => progression::build_with(d, work, false).and_then(|b| b.unit.inventory.get(emberwake_rules::gear::Slot::MainHand).map(|e| e.item.clone())),
    }
}

/// Takes off what `slot` holds; when the main hand's class default that comes back is two-handed, a shield or focus
/// in the off hand goes to the stash too (the two-handed rule).
fn take_off(d: &Data, work: &mut UnitState, stash: &mut Stash, slot: &str) -> Result<(), String> {
    unequip(work, stash, slot)?;
    if slot == "main_hand" {
        let off = work.equipment.get("off_hand").and_then(|o| d.cat.items.get(o)).cloned();
        if let (Some(main), Some(off)) = (main_weapon(d, work), off) {
            if !off_hand_ok(Some(&main), &off) {
                unequip(work, stash, "off_hand")?;
            }
        }
    }
    Ok(())
}

/// Takes off what `slot` holds (the class default comes back).
fn unequip(work: &mut UnitState, stash: &mut Stash, slot: &str) -> Result<(), String> {
    let item = work.equipment.shift_remove(slot).ok_or_else(|| format!("The {} slot already holds the class default.", slot.replace('_', " ")))?;
    give_one(stash, item);
    Ok(())
}

/// Moves the item worn in `from` to `to` (it must fit there); what `to` held moves to `from` when it fits, else to the
/// stash.
fn move_slot(d: &Data, work: &mut UnitState, stash: &mut Stash, from: &str, to: &str) -> Result<(), String> {
    let item = work.equipment.get(from).cloned().ok_or("That slot holds the class default.")?;
    let def = d.cat.items.get(&item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if from == to || !fits(def, to) {
        return Err(format!("{} doesn't go in the {} slot.", def.name, title_case(to).to_lowercase()));
    }
    if to == "off_hand" {
        let mut rest = work.clone();
        rest.equipment.shift_remove(from);
        if let Some(main) = main_weapon(d, &rest).filter(|m| !off_hand_ok(Some(m), def)) {
            return Err(format!("{} needs both hands: the off hand can hold only a weapon with it.", main.name));
        }
    }
    work.equipment.shift_remove(from);
    if let Some(old) = work.equipment.insert(to.to_string(), item) {
        if d.cat.items.get(&old).is_some_and(|o| fits(o, from)) {
            work.equipment.insert(from.to_string(), old);
        } else {
            give_one(stash, old);
        }
    }
    Ok(())
}

/// Puts a consumable from the stash in the pouch: in place of pouch slot `at` when it is filled (that item goes back),
/// else at the end while there is room.
fn pouch_put(d: &Data, work: &mut UnitState, stash: &mut Stash, item: &str, at: Option<usize>) -> Result<(), String> {
    let def = d.cat.items.get(item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if def.item_type != "CONSUMABLE" {
        return Err(format!("{} can't go in the battle pouch.", def.name));
    }
    if !def.offered_in_battle() {
        return Err(format!("{} can't be used in battle.", def.name));
    }
    if stash.get(item).copied().unwrap_or(0) < 1 {
        return Err(format!("There is no {} in the stash.", def.name));
    }
    match at.filter(|&i| i < work.pouch.len()) {
        Some(i) => {
            let old = std::mem::replace(&mut work.pouch[i], item.to_string());
            give_one(stash, old);
        }
        None if work.pouch.len() >= POUCH_MAX => return Err(format!("The pouch holds at most {POUCH_MAX} items.")),
        None => work.pouch.push(item.to_string()),
    }
    take_one(stash, item);
    Ok(())
}

fn pouch_take(work: &mut UnitState, stash: &mut Stash, i: usize) -> Result<(), String> {
    if i >= work.pouch.len() {
        return Err("That pouch slot is empty.".into());
    }
    give_one(stash, work.pouch.remove(i));
    Ok(())
}

/// A drop: payload `stash.<item>`, `equip.<slot>` or `pouch.<i>` on target `slot.<slot>`, `pouch.<i>` or `stash`.
fn drop_move(d: &Data, work: &mut UnitState, stash: &mut Stash, from: &str, to: &str) -> Result<(), String> {
    let (kind, what) = from.split_once('.').unwrap_or((from, ""));
    let (target, at) = to.split_once('.').unwrap_or((to, ""));
    let index = |s: &str| s.parse::<usize>().map_err(|_| format!("Bad pouch slot '{s}'."));
    match (kind, target) {
        ("stash", "slot") => equip(d, work, stash, what, Some(at)),
        ("stash", "pouch") => pouch_put(d, work, stash, what, Some(index(at)?)),
        ("equip", "stash") => take_off(d, work, stash, what),
        ("equip", "slot") => move_slot(d, work, stash, what, at),
        ("pouch", "stash") => pouch_take(work, stash, index(what)?),
        ("pouch", "pouch") => {
            let (i, j) = (index(what)?, index(at)?);
            if i == j || i >= work.pouch.len() {
                return Err("Nothing to move.".into());
            }
            let item = work.pouch.remove(i);
            let j = j.min(work.pouch.len());
            work.pouch.insert(j, item);
            Ok(())
        }
        _ => Err("That can't go there.".into()),
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
        self.mouse = mouse;
        match self.tab {
            "stats" => self.draw_stats(&mut ui, cx, &m),
            "level_up" => self.draw_level(&mut ui, cx, &m),
            "skills" => self.draw_skills(&mut ui, cx, &m),
            _ => self.draw_gear(&mut ui, cx, &m),
        }
        let dbox = Rect::new(25, 275, 590, 31);
        ui.p.panel(dbox, rgba(20, 28, 46, 255), BORDER, 2.0);
        // while dragging on the Skills tab the pane says where the skill can go (SKILLUI1), not the hovered card
        let dragging_skill = self.tab == "skills" && ui.dragging().is_some() && !self.detail.is_empty();
        let hovered = ui.widgets.iter().rev().find(|w| !dragging_skill && w.rect.contains(mouse.0, mouse.1) && w.tip.len() > 1).map(|w| w.tip[1].0.clone());
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

    /// Esc always closes (the right-click unslot of `back` is for the mouse only, SKILLUI1 Q4).
    fn escape(&mut self, cx: &mut Cx) {
        self.act(cx, PAct::Close);
    }

    /// A right click: on the Skills tab over a filled slot it takes the skill out (SKILLUI1 Q4); anywhere else it closes.
    fn back(&mut self, cx: &mut Cx) {
        if self.tab == "skills" {
            let (mx, my) = self.mouse;
            let over = self.w.iter().rev().find(|w| w.rect.contains(mx, my) && (w.id.starts_with("skills.class.") || w.id.starts_with("skills.extra.")));
            if let Some(w) = over {
                let class = w.id.starts_with("skills.class.");
                let i: usize = w.id.rsplit('.').next().and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
                if i < (if class { self.class_slots.len() } else { self.extra_slots.len() }) {
                    self.act(cx, PAct::Unslot(class, i));
                }
                return;
            }
        }
        self.act(cx, PAct::Close);
    }

    fn wheel(&mut self, _cx: &mut Cx, lines: i32) {
        let at = if self.tab == "gear" { &mut self.gear_scroll } else { &mut self.scroll };
        *at = (*at as i64 - lines as i64).max(0) as usize;
    }

    /// GEARUI1 and SKILLUI1: a drop is valid when the move it makes is (the same rules as the clicks).
    fn can_drop(&self, cx: &Cx, from: &str, to: &str) -> bool {
        match self.tab {
            "gear" => self.after_drop(&cx.data, from, to).is_ok(),
            "skills" => self.skills_after_drop(cx, from, to).is_ok(),
            _ => false,
        }
    }

    fn accept_drop(&mut self, cx: &mut Cx, from: &str, to: &str) {
        if self.tab == "skills" {
            self.load_skills(cx);
            match self.skills_after_drop(cx, from, to) {
                Ok((class, extra, ranks)) => {
                    (self.class_slots, self.extra_slots, self.ranks) = (class, extra, ranks);
                    self.using_default = false;
                }
                Err(e) => show_errors(cx, vec![e]),
            }
            return;
        }
        self.load_gear(cx);
        match self.after_drop(&cx.data, from, to) {
            Ok(draft) => self.gear = Some(draft),
            Err(e) => show_errors(cx, vec![e]),
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        let mut out = json!({"member": self.member, "tab": self.tab, "draft": self.draft.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<String, Value>>(), "has_draft": self.has_draft(cx)});
        if self.tab == "skills" {
            out["class"] = json!(self.class_slots);
            out["extra"] = json!(self.extra_slots);
            out["ranks"] = Value::Object(self.ranks.iter().map(|(k, v)| (k.clone(), json!(v))).collect());
            out["using_default"] = json!(self.using_default);
        }
        out
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
        let cs = emberwake_rules::stats::CharacterStats::new(arch, lvl as i32, &spend);
        let cols = columns(ui, &[(68.0, "Stat"), (112.0, "Base"), (148.0, "Growth"), (190.0, "Spent"), (226.0, "Total")], 10.0);
        for (&(x, _), h) in cols.iter().zip(["Stat", "Base", "Growth", "Spent", "Total"]) {
            ui.p.text(x, 143.0, h, BODY + 1.0, DIM);
        }
        let table_right = cols[4].1;
        for (i, st) in STATS.iter().enumerate() {
            let y = 158.0 + i as f32 * 15.0;
            let base = arch.base_stats.get(*st).copied().unwrap_or(0);
            let sp = m.spend.get(*st).copied().unwrap_or(0) as i32;
            let total = cs.base(st);
            ui.p.text(68.0, y, st, BODY + 1.0, TEXT);
            for (c, v) in cols[1..].iter().zip([base, total - base - sp, sp, total]) {
                ui.p.text_right(c.1, y, &v.to_string(), BODY + 1.0, TEXT);
            }
            ui.push(format!("stats.{st}"), "row", format!("{st} {total}"), Rect::new(66, y as i32 - 2, (table_right - 62.0) as i32, 14), true, false, None, tip(st, &stat_desc(st)));
        }
        let rows = derived(d, m);
        let label_x = 250f32.max(table_right + 6.0);
        let value_x = 325f32.max(label_x + rows.iter().map(|r| ui.p.text_w(r.0, BODY + 1.0)).fold(0.0, f32::max) + 8.0);
        for (i, (label, v)) in rows.iter().enumerate() {
            let y = 143.0 + i as f32 * 15.0;
            ui.p.text(label_x, y, label, BODY + 1.0, DIM);
            ui.p.text(value_x, y, &v.to_string(), BODY + 1.0, TEXT);
            ui.push(format!("stats.derived.{label}"), "row", format!("{label} {v}"), Rect::new(label_x as i32 - 2, y as i32 - 2, (value_x - label_x) as i32 + 26, 14), true, false, None, tip(label, &derived_desc(label)));
        }
        let names: Vec<String> = m.passives.iter().map(|p| s_or(d.entry("passives", p), "name", p)).collect();
        let line = format!("Passives: {}", if names.is_empty() { "none".into() } else { names.join(", ") });
        ui.p.text_fit(68.0, 258.0, &line, BODY + 1.0, label_x - 78.0, DIM);
        let about: Vec<String> = m.passives.iter().map(|p| format!("{}: {}", s_or(d.entry("passives", p), "name", p), json::s(d.entry("passives", p), "description"))).collect();
        let line_h = (ui.p.line_h(BODY + 1.0) + 2.0).ceil().max(13.0) as i32;
        ui.push("stats.passives", "row", line, Rect::new(66, 256, (label_x - 74.0) as i32, line_h), true, false, None, tip("Passives", &if about.is_empty() { "None yet: passives come from the hero's background, faction, disciplines and named items.".into() } else { about.join(" ") }));
    }

    fn draw_level(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        let d = &cx.data;
        let drafted: i64 = self.draft.values().sum();
        let left = m.unspent(d) - drafted;
        ui.p.text(70.0, 104.0, &format!("Points left: {left}   (confirmed points are permanent)"), BODY + 1.0, DIM);
        let Some(arch) = d.cat.classes.get(&m.class_id(d)) else { return };
        let mut spend: IndexMap<String, i32> = m.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
        spend.entry(STATS[0].into()).or_insert(0);
        let cs = emberwake_rules::stats::CharacterStats::new(arch, m.level(d) as i32, &spend);
        let cols = columns(ui, &[(70.0, "Stat"), (105.0, "Now"), (132.0, "Added"), (165.0, "New")], 8.0);
        for (&(x, _), h) in cols.iter().zip(["Stat", "Now", "Added", "New"]) {
            ui.p.text(x, 122.0, h, BODY + 1.0, DIM);
        }
        let minus_x = 195.max(cols[3].1 as i32 + 8);
        let auto_x = 266.max(minus_x + 71);
        for (i, st) in STATS.iter().enumerate() {
            let y = 136 + i as i32 * 21;
            let now = cs.base(st) as i64;
            let add = self.draft.get(*st).copied().unwrap_or(0);
            ui.p.text(70.0, y as f32 + 4.0, st, BODY + 1.0, TEXT);
            ui.p.text_right(cols[1].1, y as f32 + 4.0, &now.to_string(), BODY + 1.0, TEXT);
            ui.p.text_right(cols[2].1, y as f32 + 4.0, &add.to_string(), BODY + 1.0, if add > 0 { GOLD } else { DIM });
            ui.p.text_right(cols[3].1, y as f32 + 4.0, &(now + add).to_string(), BODY + 1.0, TEXT);
            ui.push(format!("level.{st}"), "row", format!("{st} {now} + {add}"), Rect::new(68, y, minus_x - 73, 17), true, false, None, tip(st, &stat_desc(st)));
            ui.button_tip(&format!("level.{st}.minus"), Rect::new(minus_x, y, 23, 17), "-", PAct::Remove(st), add > 0, tip(&format!("-1 {st}"), "Take back a drafted point. Nothing is spent until you Confirm."));
            ui.button_tip(&format!("level.{st}.plus"), Rect::new(minus_x + 31, y, 23, 17), "+", PAct::Add(st), left > 0, tip(&format!("+1 {st}"), &stat_desc(st)));
        }
        ui.button_tip("level.auto", Rect::new(auto_x, 121, 69, 17), "Auto", PAct::Auto, left > 0, tip("Auto", "Spend the remaining points by the class growth weights."));
        ui.button_tip("level.revert", Rect::new(auto_x, 145, 69, 17), "Revert", PAct::Revert, drafted > 0, tip("Revert", "Clear the drafted points. Nothing is spent until you Confirm."));
        let mut work = m.clone();
        for (k, v) in &self.draft {
            *work.spend.entry(k.clone()).or_insert(0) += v;
        }
        let (before, after) = (derived(d, m), derived(d, &work));
        let label_x = 352f32.max((auto_x + 86) as f32);
        let value_x = 427f32.max(label_x + after.iter().map(|r| ui.p.text_w(r.0, BODY + 1.0)).fold(0.0, f32::max) + 8.0);
        for (i, (label, v)) in after.iter().enumerate() {
            let y = 122.0 + i as f32 * 15.0;
            let old = before.get(i).map(|b| b.1).unwrap_or(*v);
            ui.p.text(label_x, y, label, BODY + 1.0, DIM);
            ui.p.text(value_x, y, &if old != *v { format!("{old} -> {v}") } else { v.to_string() }, BODY + 1.0, if old != *v { GOLD } else { TEXT });
            ui.push(format!("level.derived.{label}"), "row", format!("{label} {v}"), Rect::new(label_x as i32 - 2, y as i32 - 2, (value_x - label_x) as i32 + 55, 14), true, false, None, tip(label, &derived_desc(label)));
        }
        ui.button("level.confirm", Rect::new(500, 312, 70, 18), "Confirm", PAct::ConfirmLevel, drafted > 0);
    }

    fn draw_skills(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        self.load_skills(cx);
        let d = &cx.data;
        let cat = &d.cat;
        let Some(b) = progression::build_with(d, m, false) else { return };
        let u = &b.unit;
        let fam_name = |f: &str| cat.tier_one(f).map(|s| s.name.clone()).unwrap_or_else(|| f.to_string());
        let row_of = |f: &str, rank: i32| cat.family_rows(f).into_iter().find(|s| s.tier == rank);
        let path = self.path_families(cx);
        let max_class = emberwake_rules::stats::MAX_CLASS_SLOTS;
        let max_extra = emberwake_rules::stats::MAX_EXTRA_SLOTS;
        // Left: the fixed slots, five class and three extra (SKILLUI1 section 4).
        ui.p.text(64.0, 104.0, &format!("Class slots {} / {max_class}", self.class_slots.len()), BODY + 1.0, INFO);
        ui.p.text(150.0, 104.0, if self.using_default { "Default loadout" } else { "Custom loadout" }, BODY, DIM);
        ui.p.text(64.0, 205.0, &format!("Extra slots {} / {max_extra}", self.extra_slots.len()), BODY + 1.0, INFO);
        for (class, top_y, count) in [(true, 116, max_class), (false, 216, max_extra)] {
            let list = if class { self.class_slots.clone() } else { self.extra_slots.clone() };
            let g = if class { "class" } else { "extra" };
            for i in 0..count {
                let r = Rect::new(64, top_y + i as i32 * 17, 236, 15);
                let id = format!("skills.{g}.{i}");
                let Some(f) = list.get(i) else {
                    ui.p.panel(r, rgba(20, 28, 46, 120), rgba(58, 78, 116, 120), 2.0);
                    ui.p.text(r.x as f32 + 4.0, r.y as f32 + 3.0, "Empty", BODY, DIM);
                    let label = if class { "class" } else { "extra" };
                    ui.push(id, "row", "Empty", r, true, false, None, tip(&format!("Empty {label} slot"), "Drag a skill here, or click a skill in the list."));
                    ui.drop_target(format!("slot.{g}.{i}"));
                    continue;
                };
                let ranks = u.selectable_ranks(cat, f);
                let top = ranks.last().copied().unwrap_or(0);
                let chosen = self.ranks.get(f).map(|&r| r as i32);
                let rank = draft_rank(&ranks, chosen);
                let at = ranks.iter().position(|&x| x == rank).unwrap_or(0);
                let row = row_of(f, rank);
                let ap = row.map(|s| emberwake_battle::skill::ap_cost(s, Some(u))).unwrap_or(0);
                let desc = row.map(|s| json::s(&s.raw, "description")).unwrap_or_default();
                ui.p.panel(r, rgba(26, 34, 54, 255), BORDER, 2.0);
                ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 3.0, &fam_name(f), BODY, 112.0, TEXT);
                let card = vec![(fam_name(f), GOLD), (desc.clone(), TEXT), ("Drag to another slot or back to the list. Right click takes it out.".into(), DIM)];
                ui.push(id, "row", fam_name(f), r, true, false, None, card);
                ui.draggable(format!("slot.{g}.{i}"));
                ui.drop_target(format!("slot.{g}.{i}"));
                // < Rank >: step over the unlocked ranks; GOLD when kept below the highest unlocked rank
                let lines = emberwake_world::skill_text::rank_lines(cat, u, f);
                let step_tip = |to: Option<i32>| -> aegis_ui::ui::Tip {
                    let Some(t) = to else { return Vec::new() };
                    let line = lines.iter().find(|l| l.tier == t).map(|l| l.text.clone()).unwrap_or_default();
                    let to_ap = lines.iter().find(|l| l.tier == t).map(|l| l.ap).unwrap_or(ap);
                    let cost = match to_ap.cmp(&ap) {
                        std::cmp::Ordering::Less => format!("Cheaper: {to_ap} AP (now {ap})"),
                        std::cmp::Ordering::Greater => format!("Stronger: {to_ap} AP (now {ap})"),
                        std::cmp::Ordering::Equal => format!("Same cost: {ap} AP"),
                    };
                    vec![(format!("Rank {}", roman(t)), GOLD), (line, TEXT), (cost, DIM)]
                };
                let down = at.checked_sub(1).and_then(|k| ranks.get(k).copied());
                let up = ranks.get(at + 1).copied();
                ui.button_full(&format!("skills.rank.{g}.{i}.down"), Rect::new(r.x + 120, r.y + 1, 13, 13), "<", PAct::RankStep(class, i, -1), down.is_some(), false, BODY, step_tip(down));
                let mut num_tip = vec![(format!("{}: Rank {} of {}", fam_name(f), roman(rank), roman(top)), GOLD)];
                num_tip.extend(lines.iter().map(|l| (format!("{}: {}{}", roman(l.tier), l.text, if l.tier == rank { " (chosen)" } else { "" }), if l.unlocked { TEXT } else { DIM })));
                num_tip.push((if chosen.is_some() { "Stays at this rank when new ranks unlock; change it with < and >." } else { "Follows the top rank until you confirm; from then on it stays put." }.into(), DIM));
                ui.p.text(r.x as f32 + 136.0, r.y as f32 + 3.0, roman(rank), BODY, if rank < top { GOLD } else { TEXT });
                ui.push(format!("skills.rank.{g}.{i}"), "row", roman(rank), Rect::new(r.x + 134, r.y, 22, 15), true, false, None, num_tip);
                ui.button_full(&format!("skills.rank.{g}.{i}.up"), Rect::new(r.x + 157, r.y + 1, 13, 13), ">", PAct::RankStep(class, i, 1), up.is_some(), false, BODY, step_tip(up));
                ui.p.text(r.x as f32 + 176.0, r.y as f32 + 3.0, &format!("{ap} AP"), BODY, DIM);
            }
        }
        // Right: the owned skills; the whole column takes slots dropped back (recorded under the rows).
        let list_area = Rect::new(310, 114, 294, 156);
        ui.p.text(310.0, 104.0, "Owned skills (drag onto a slot or click)", BODY + 1.0, INFO);
        ui.push("skills.list", "panel", "Owned skills", list_area, true, false, None, tip("Owned skills", "Drop a slot here to take the skill out."));
        ui.drop_target("list");
        let owned: Vec<String> = u.owned_families(cat).into_iter().filter(|f| !cat.is_basic_family(f)).collect();
        const ROWS: usize = 6;
        self.scroll = self.scroll.min(owned.len().saturating_sub(ROWS));
        if self.scroll > 0 || owned.len() > ROWS {
            ui.p.text_right(604.0, 104.0, &format!("{}-{} of {}", self.scroll + 1, (self.scroll + ROWS).min(owned.len()), owned.len()), SMALL, DIM);
        }
        for (k, f) in owned.iter().skip(self.scroll).take(ROWS).enumerate() {
            let y = 118 + k as i32 * 22;
            let top = u.selectable_ranks(cat, f).last().copied().unwrap_or(0);
            let row = row_of(f, top);
            let slotted = self.class_slots.contains(f) || self.extra_slots.contains(f);
            let on_path = path.contains(f);
            let full = !slotted && self.extra_slots.len() >= max_extra && (!on_path || self.class_slots.len() >= max_class);
            ui.button(&format!("skills.slot.{f}"), Rect::new(310, y, 50, 16), if slotted { "Slotted" } else if full { "Full" } else { "Slot" }, PAct::Slot(f.clone()), !slotted && !full);
            let grade: String = cat.family_rows(f).first().map(|s| { let mut c = s.grade.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).unwrap_or_default();
            let desc = row.map(|s| json::s(&s.raw, "description")).unwrap_or_default();
            ui.p.text_fit(366.0, y as f32 + 1.0, &format!("{}  [{}]  Rank {}", fam_name(f), grade, roman(top)), BODY, 238.0, TEXT);
            ui.p.text_fit(366.0, y as f32 + 10.0, &desc, SMALL, 238.0, DIM);
            ui.push(format!("skills.family.{f}"), "row", fam_name(f), Rect::new(366, y, 238, 20), true, slotted, Some(PAct::Slot(f.clone())), tip(&fam_name(f), &desc));
            ui.draggable(format!("skill.{f}"));
        }
        // While a skill is dragged the pane says where it can go.
        if let Some(from) = ui.dragging().map(String::from) {
            let (mx, my) = ui.mouse;
            let target = ui.widgets.iter().rev().find(|x| x.drop.is_some() && x.rect.contains(mx, my)).and_then(|x| x.drop.clone());
            self.detail = match target.as_deref().and_then(|t| t.strip_prefix("slot.")).and_then(|t| t.split_once('.')) {
                Some((g, i)) if self.skills_after_drop(cx, &from, &format!("slot.{g}.{i}")).is_ok() => {
                    let list = if g == "class" { &self.class_slots } else { &self.extra_slots };
                    match list.get(i.parse::<usize>().unwrap_or(usize::MAX)) {
                        Some(old) if from.strip_prefix("skill.").is_some_and(|f| !self.class_slots.iter().chain(&self.extra_slots).any(|x| x == f)) => format!("Replaces {}.", fam_name(old)),
                        Some(old) => format!("Swaps with {}.", fam_name(old)),
                        None => "Drop to slot it here.".into(),
                    }
                }
                _ => "Drop on a class slot (class path skills) or an extra slot; drop on the list to take it out.".into(),
            };
        }
        ui.button("skills.reset", Rect::new(68, 312, 70, 18), "Reset", PAct::ResetSkills, true);
        ui.button("skills.default", Rect::new(144, 312, 70, 18), "Use default", PAct::UseDefault, !self.using_default);
        ui.button_tip("skills.clear", Rect::new(220, 312, 70, 18), "Clear all", PAct::ClearAll, !self.class_slots.is_empty() || !self.extra_slots.is_empty(), tip("Clear all", "Empties every class and extra slot in the draft (an empty loadout means the class default)."));
        ui.button("skills.confirm", Rect::new(498, 312, 70, 18), "Confirm", PAct::ConfirmSkills, self.has_draft(cx));
    }

    /// GEARUI1: three columns like the Godot gear dojo: the hero's stats with this gear (and the change a hovered or
    /// dragged item would make), the equipped slots with the battle pouch under them, the party stash. Clicks and drag
    /// and drop both ways (stash to a slot or the pouch equips, a slot or pouch item to the stash takes it off).
    fn draw_gear(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        self.load_gear(cx);
        let d = &cx.data;
        let (work, stash) = self.gear.clone().unwrap();
        let built = progression::build_with(d, &work, false);
        let head = BODY + 1.0;
        // Middle: the equipped slots, then the pouch.
        let (mx, mw) = (243, 185);
        ui.p.text(mx as f32 + 2.0, 102.0, "Equipped", head, INFO);
        for (i, slot) in LOADOUT_SLOTS.iter().enumerate() {
            let y = 116 + i as i32 * 15;
            let explicit = work.equipment.get(*slot).cloned();
            let shown = explicit.clone().or_else(|| {
                let s = emberwake_rules::gear::Slot::parse(slot)?;
                built.as_ref().and_then(|b| b.unit.inventory.get(s).map(|e| e.item.id.clone()))
            });
            let name = shown.as_ref().map(|i| d.item_name(i)).unwrap_or_else(|| "empty".into());
            let label = format!("{}: {name}", title_case(slot));
            let card = match (&explicit, &shown) {
                (Some(item), _) => item_tip(d, item, "Click or drag it to the stash to take it off (the class default comes back); drag it onto another slot it fits to move it."),
                (None, Some(item)) => item_tip(d, item, "Class default gear: drag an item from the stash onto this slot to replace it."),
                (None, None) => tip(&label, "Empty: drag an item from the stash onto this slot."),
            };
            ui.row(&format!("gear.slot.{slot}"), Rect::new(mx, y, mw, 14), &label, if explicit.is_some() { TEXT } else { DIM }, explicit.as_ref().map(|_| PAct::Unequip(slot.to_string())), false, card);
            if explicit.is_some() {
                ui.draggable(format!("equip.{slot}"));
            }
            ui.drop_target(format!("slot.{slot}"));
        }
        ui.p.text(mx as f32 + 2.0, 210.0, "Battle pouch", head, INFO);
        for i in 0..POUCH_MAX {
            let y = 224 + i as i32 * 15;
            let item = work.pouch.get(i);
            let label = match item {
                Some(item) => format!("{}. {}", i + 1, d.item_name(item)),
                None => format!("{}. Empty", i + 1),
            };
            let card = match item {
                Some(item) => item_tip(d, item, "Click or drag it to the stash to take it out; drag it onto another pouch slot to reorder."),
                None => tip(&label, "Drag a consumable from the stash here; the hero can use it in battle."),
            };
            ui.row(&format!("gear.pouch.{i}"), Rect::new(mx, y, mw, 14), &label, if item.is_some() { TEXT } else { DIM }, item.map(|_| PAct::PouchRemove(i)), false, card);
            if item.is_some() {
                ui.draggable(format!("pouch.{i}"));
            }
            ui.drop_target(format!("pouch.{i}"));
        }
        // Right: the party stash (the whole column takes items dropped back).
        let (sx, sw) = (436, 179);
        ui.p.text(sx as f32 + 2.0, 102.0, "Party stash", head, INFO);
        let area = Rect::new(sx, 114, sw, 156);
        ui.p.panel(area, rgba(20, 28, 46, 160), BORDER, 2.0);
        ui.push("gear.stash", "panel", "Party stash", area, true, false, None, tip("Party stash", "Drop a worn or pouch item here to take it off."));
        ui.drop_target("stash");
        let mut ids: Vec<&String> = stash.keys().filter(|id| d.cat.items.contains_key(*id)).collect();
        ids.sort_by_key(|id| d.item_name(id));
        const ROWS: usize = 10;
        self.gear_scroll = self.gear_scroll.min(ids.len().saturating_sub(ROWS));
        if ids.is_empty() {
            ui.p.text(sx as f32 + 4.0, 118.0, "Empty", head, DIM);
        }
        for (row, id) in ids.iter().skip(self.gear_scroll).take(ROWS).enumerate() {
            let def = &d.cat.items[*id];
            let (act, help) = if def.is_equipment() {
                (Some(PAct::Equip((*id).clone())), "Click to wear it, or drag it onto a slot.")
            } else if def.item_type == "CONSUMABLE" {
                (Some(PAct::PouchAdd((*id).clone())), "Click to put one in the battle pouch, or drag it onto a pouch slot.")
            } else {
                (None, "Not wearable.")
            };
            let movable = act.is_some();
            ui.row(&format!("gear.stash.{id}"), Rect::new(sx + 2, 116 + row as i32 * 15, sw - 4, 14), &format!("{} x{}", def.name, stash[*id]), TEXT, act, false, item_tip(d, id, help));
            if movable {
                ui.draggable(format!("stash.{id}"));
            }
        }
        if ids.len() > ROWS {
            let shown = format!("{}-{} of {} (wheel)", self.gear_scroll + 1, (self.gear_scroll + ROWS).min(ids.len()), ids.len());
            ui.p.text_right((sx + sw - 4) as f32, 262.0, &shown, SMALL, DIM);
        }
        // Left: the stats with this gear, and the change of the dragged item over a target, else of the hovered item.
        let preview = self.gear_preview(ui, d, &work, &stash);
        let now = sheet(d, &work);
        let then = preview.as_ref().map(|p| sheet(d, p));
        let (lx, lw) = (25, 210);
        ui.p.text(lx as f32 + 3.0, 102.0, &format!("{}  Lv {}", d.cat.classes.get(&work.class_id(d)).map(|c| c.name.clone()).unwrap_or_default(), work.level(d)), head, INFO);
        for (i, (label, v)) in now.iter().enumerate() {
            let core = i < STATS.len();
            let (x, y, w) = if core { (lx + 3 + (i as i32 % 2) * 104, 117 + (i as i32 / 2) * 12, 100) } else { (lx + 3, 158 + (i - STATS.len()) as i32 * 12, lw - 6) };
            ui.p.text(x as f32, y as f32, label, BODY, DIM);
            let right = (x + w - 26) as f32;
            ui.p.text_right(right, y as f32, &v.to_string(), BODY, TEXT);
            let delta = then.as_ref().and_then(|t| t.get(i)).map(|t| t.1 - v).unwrap_or(0);
            if delta != 0 {
                ui.p.text(right + 3.0, y as f32, &format!("{delta:+}"), BODY, if delta > 0 { GOOD } else { FOE });
            }
            let desc = if core { stat_desc(label) } else { derived_desc(label) };
            ui.push(format!("gear.stat.{label}"), "row", format!("{label} {v}"), Rect::new(x - 2, y - 1, w, 12), true, false, None, tip(label, &desc));
        }
        let dirty = &work != m || stash != cx.gs().party.stash;
        ui.button("gear.reset", Rect::new(68, 312, 70, 18), "Reset", PAct::ResetGear, dirty);
        ui.button("gear.confirm", Rect::new(498, 312, 70, 18), "Confirm", PAct::ConfirmGear, dirty);
    }
}

impl Character {
    /// The draft a hovered or dragged item would make (UIR-3B stat change): a drag over a target it can drop on, else
    /// a hovered stash item worn in its first slot, else a hovered worn item taken off; None otherwise.
    fn gear_preview(&self, ui: &Ui<PAct>, d: &Data, work: &UnitState, stash: &Stash) -> Option<UnitState> {
        let (mut w, mut st) = (work.clone(), stash.clone());
        let mouse = ui.mouse;
        if let Some(from) = ui.dragging() {
            let to = ui.widgets.iter().rev().find(|x| x.drop.is_some() && x.rect.contains(mouse.0, mouse.1))?.drop.clone()?;
            return drop_move(d, &mut w, &mut st, from, &to).ok().map(|_| w);
        }
        let hovered = ui.widgets.iter().rev().find(|x| x.action.is_some() && ui.hover(x.rect))?;
        let ok = if let Some(item) = hovered.id.strip_prefix("gear.stash.") {
            d.cat.items.get(item).is_some_and(|i| i.is_equipment()) && equip(d, &mut w, &mut st, item, None).is_ok()
        } else if let Some(slot) = hovered.id.strip_prefix("gear.slot.") {
            take_off(d, &mut w, &mut st, slot).is_ok()
        } else {
            false
        };
        ok.then_some(w)
    }
}

/// The Gear tab's stat list: the core stats with gear, then the derived values (`derived`).
fn sheet(d: &Data, m: &UnitState) -> Vec<(String, i64)> {
    let Some(b) = progression::build_with(d, m, false) else { return Vec::new() };
    let mut out: Vec<(String, i64)> = STATS.iter().map(|st| (st.to_string(), b.unit.stats.stat(st, None) as i64)).collect();
    out.extend(derived_of(&b.unit).into_iter().map(|(l, v)| (l.to_string(), v)));
    out
}

/// An item's hover card: its name, then its description (tokens filled), its numbers and what a click or drag does.
fn item_tip(d: &Data, id: &str, help: &str) -> aegis_ui::ui::Tip {
    let Some(def) = d.cat.items.get(id) else { return tip(id, help) };
    let desc = d.item_description(id);
    let numbers = item_numbers(def);
    let body: Vec<&str> = [desc.as_str(), numbers.as_str(), help].into_iter().filter(|s| !s.is_empty()).collect();
    tip(&def.name, &body.join(" "))
}

/// An item's numbers in one sentence: power and range, mitigation, burden, bonuses, requirements, what it heals.
fn item_numbers(def: &emberwake_data::ItemDef) -> String {
    let mut parts: Vec<String> = Vec::new();
    if def.item_type == "WEAPON" {
        parts.push(format!("Power {}, range {}{}", def.base_power, def.base_range, if def.two_handed { ", two-handed" } else { "" }));
    }
    if def.mitigation != 0 {
        parts.push(format!("mitigation {}", def.mitigation));
    }
    if def.burden != 0 {
        parts.push(format!("burden {}", def.burden));
    }
    parts.extend(def.stat_bonuses.iter().map(|(s, v)| format!("{v:+} {s}")));
    if !def.required_stats.is_empty() {
        parts.push(format!("needs {}", def.required_stats.iter().map(|(s, v)| format!("{v} {s}")).collect::<Vec<_>>().join(", ")));
    }
    if def.heal > 0 {
        parts.push(format!("heals {}", def.heal));
    }
    if def.damage > 0 {
        parts.push(format!("{} damage", def.damage));
    }
    if parts.is_empty() {
        return String::new();
    }
    let line = parts.join("; ");
    let mut c = line.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str() + ".").unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Data, UnitState, Stash) {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let d = Data::open(&dir).expect("data").0;
        let gs = emberwake_world::new_game::create_for_world(&d, "ashlands", 1).expect("new game");
        let m = gs.party.members[0].clone();
        let mut stash = Stash::new();
        for (id, def) in &d.cat.items {
            if def.is_equipment() || def.item_type == "CONSUMABLE" {
                stash.insert(id.clone(), 2);
            }
        }
        (d, m, stash)
    }

    fn first(d: &Data, slot: &str) -> String {
        let mut ids: Vec<&String> = d.cat.items.iter().filter(|(_, i)| fits(i, slot)).map(|(id, _)| id).collect();
        ids.sort();
        ids[0].clone()
    }

    /// The first item (by id) the member's class can wear in `slot`.
    fn wearable(d: &Data, m: &UnitState, slot: &str) -> String {
        let mut ids: Vec<&String> = d.cat.items.keys().filter(|id| m.can_wear(d, id, slot).is_ok()).collect();
        ids.sort();
        ids[0].clone()
    }

    fn consumables(d: &Data) -> Vec<String> {
        let mut ids: Vec<String> = d.cat.items.iter().filter(|(_, i)| i.item_type == "CONSUMABLE").map(|(id, _)| id.clone()).collect();
        ids.sort();
        ids
    }

    #[test]
    fn a_stash_item_dropped_on_a_slot_it_fits_is_worn_and_the_old_one_returns() {
        let (d, mut m, mut st) = setup();
        let (a, sword) = (first(&d, "accessory_1"), first(&d, "main_hand"));
        drop_move(&d, &mut m, &mut st, &format!("stash.{a}"), "slot.accessory_1").unwrap();
        assert_eq!(m.equipment.get("accessory_1"), Some(&a));
        assert_eq!(st[&a], 1);
        drop_move(&d, &mut m, &mut st, &format!("stash.{a}"), "slot.accessory_1").unwrap();
        assert_eq!(st[&a], 1, "the replaced copy came back");
        assert!(drop_move(&d, &mut m, &mut st, &format!("stash.{sword}"), "slot.body").is_err(), "a weapon is not body armour");
    }

    #[test]
    fn a_worn_item_dropped_on_the_stash_comes_off() {
        let (d, mut m, mut st) = setup();
        let a = first(&d, "accessory_1");
        let before = st[&a];
        drop_move(&d, &mut m, &mut st, &format!("stash.{a}"), "slot.accessory_1").unwrap();
        drop_move(&d, &mut m, &mut st, "equip.accessory_1", "stash").unwrap();
        assert!(!m.equipment.contains_key("accessory_1"));
        assert_eq!(st[&a], before);
        assert!(drop_move(&d, &mut m, &mut st, "equip.accessory_1", "stash").is_err(), "a class default can't come off");
    }

    #[test]
    fn a_worn_item_moves_to_another_slot_it_fits() {
        let (d, mut m, mut st) = setup();
        let a = first(&d, "accessory_1");
        drop_move(&d, &mut m, &mut st, &format!("stash.{a}"), "slot.accessory_1").unwrap();
        drop_move(&d, &mut m, &mut st, "equip.accessory_1", "slot.accessory_2").unwrap();
        assert_eq!((m.equipment.get("accessory_1"), m.equipment.get("accessory_2")), (None, Some(&a)));
        assert!(drop_move(&d, &mut m, &mut st, "equip.accessory_2", "slot.body").is_err());
    }

    #[test]
    fn the_pouch_takes_consumables_replaces_reorders_and_stops_at_its_size() {
        let (d, mut m, mut st) = setup();
        m.pouch.clear();
        let c = consumables(&d);
        let wear = first(&d, "main_hand");
        assert!(drop_move(&d, &mut m, &mut st, &format!("stash.{wear}"), "pouch.0").is_err(), "gear is not a consumable");
        st.insert(c[0].clone(), 10);
        for _ in 0..POUCH_MAX {
            pouch_put(&d, &mut m, &mut st, &c[0], None).unwrap();
        }
        assert!(pouch_put(&d, &mut m, &mut st, &c[0], None).is_err(), "a full pouch");
        let other = c.last().unwrap().clone();
        let n = st[&c[0]];
        drop_move(&d, &mut m, &mut st, &format!("stash.{other}"), "pouch.1").unwrap();
        assert_eq!(m.pouch[1], other);
        assert_eq!(st[&c[0]], n + 1, "the replaced one came back");
        drop_move(&d, &mut m, &mut st, "pouch.1", "pouch.0").unwrap();
        assert_eq!(m.pouch[0], other);
        drop_move(&d, &mut m, &mut st, "pouch.0", "stash").unwrap();
        assert_eq!(m.pouch.len(), POUCH_MAX - 1);
    }

    /// UnitState._gear_errors: a class without training in a weapon type can't wear it (clicks, drops and Confirm).
    #[test]
    fn a_class_without_training_cannot_wear_the_weapon() {
        let (d, mut m, mut st) = setup();
        let class = d.cat.classes.get(&m.class_id(&d)).unwrap().clone();
        let prof = emberwake_rules::stats::Proficiency::from_class(&class);
        let mut untrained: Vec<&String> = d.cat.items.iter().filter(|(_, i)| fits(i, "main_hand") && !prof.can_use(&i.weapon_type)).map(|(id, _)| id).collect();
        untrained.sort();
        let item = untrained.first().expect("a weapon this class has no training in").to_string();
        let err = drop_move(&d, &mut m, &mut st, &format!("stash.{item}"), "slot.main_hand").unwrap_err();
        assert!(err.contains("no training"), "{err}");
        m.equipment.insert("main_hand".into(), item);
        assert!(m.validate(&d).iter().any(|e| e.contains("no training")), "Confirm refuses it too");
    }

    /// The two-handed rule (decision "Two-handed weapon and the off hand (user 2026-10-20)").
    #[test]
    fn a_two_handed_weapon_and_a_shield_exclude_each_other() {
        let (d, mut m, mut st) = setup();
        let pick = |m: &UnitState, slot: &str, two: bool, kind: &str| {
            let mut ids: Vec<&String> = d.cat.items.iter().filter(|(id, i)| i.two_handed == two && i.item_type == kind && m.can_wear(&d, id, slot).is_ok()).map(|(id, _)| id).collect();
            ids.sort();
            ids.first().map(|s| s.to_string())
        };
        let (Some(great), Some(shield)) = (pick(&m, "main_hand", true, "WEAPON"), pick(&m, "off_hand", false, "SHIELD")) else { panic!("a 2H weapon and a shield this class can wear") };
        drop_move(&d, &mut m, &mut st, &format!("stash.{shield}"), "slot.off_hand").unwrap();
        let shields = st[&shield];
        drop_move(&d, &mut m, &mut st, &format!("stash.{great}"), "slot.main_hand").unwrap();
        assert!(!m.equipment.contains_key("off_hand"), "the shield is stowed");
        assert_eq!(st[&shield], shields + 1);
        let err = drop_move(&d, &mut m, &mut st, &format!("stash.{shield}"), "slot.off_hand").unwrap_err();
        assert!(err.contains("both hands"), "{err}");
        let built = progression::build_with(&d, &m, false).unwrap();
        assert!(built.unit.inventory.get(emberwake_rules::gear::Slot::OffHand).is_none_or(|o| off_hand_ok(Some(&d.cat.items[&great]), &o.item)), "no default shield under the 2H weapon");
        m.equipment.insert("off_hand".into(), shield);
        assert!(m.validate(&d).iter().any(|e| e.contains("two-handed")), "validation refuses the pair");
    }

    /// The focus half of the rule: an orb or tome is not held under a two-handed staff (Equipment.is_focus).
    #[test]
    fn a_two_handed_weapon_and_a_focus_exclude_each_other() {
        let (d, _, mut st) = setup();
        let base = d.cat.units.values().filter(|u| u.class_id == "mage" && u.side == "party").map(|u| u.id.clone()).min().expect("a party mage");
        let mut m = UnitState::create("test_mage", &base);
        let wear = |m: &UnitState, slot: &str, f: &dyn Fn(&emberwake_data::ItemDef) -> bool| {
            let mut ids: Vec<&String> = d.cat.items.iter().filter(|(id, i)| f(i) && m.can_wear(&d, id, slot).is_ok()).map(|(id, _)| id).collect();
            ids.sort();
            ids.first().map(|s| s.to_string())
        };
        let staff = wear(&m, "main_hand", &|i| i.two_handed && i.item_type == "WEAPON").expect("a 2H weapon a mage can wear");
        let focus = wear(&m, "off_hand", &|i| matches!(i.weapon_type.as_str(), "orb" | "tome")).expect("a focus");
        drop_move(&d, &mut m, &mut st, &format!("stash.{focus}"), "slot.off_hand").unwrap();
        drop_move(&d, &mut m, &mut st, &format!("stash.{staff}"), "slot.main_hand").unwrap();
        assert!(!m.equipment.contains_key("off_hand"), "the focus is stowed");
        assert!(drop_move(&d, &mut m, &mut st, &format!("stash.{focus}"), "slot.off_hand").unwrap_err().contains("both hands"));
        let built = progression::build_with(&d, &m, false).unwrap();
        assert!(built.unit.inventory.get(emberwake_rules::gear::Slot::OffHand).is_none(), "no default orb under the staff");
        m.equipment.insert("off_hand".into(), focus);
        assert!(m.validate(&d).iter().any(|e| e.contains("two-handed")));
    }

    #[test]
    fn wearing_an_item_changes_the_stat_sheet_the_preview_shows() {
        let (d, m, mut st) = setup();
        let before = sheet(&d, &m);
        let mut changed = false;
        for slot in ["main_hand", "body", "accessory_1"] {
            let mut w = m.clone();
            let item = wearable(&d, &m, slot);
            equip(&d, &mut w, &mut st, &item, Some(slot)).unwrap();
            changed |= sheet(&d, &w) != before;
        }
        assert!(changed);
        assert_eq!(before.len(), STATS.len() + derived(&d, &m).len());
    }
}
