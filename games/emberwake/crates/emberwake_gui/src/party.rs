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
use emberwake_world::state::{UnitState, LOADOUT_SLOTS, POUCH_MAX};
use emberwake_world::{progression, Data};
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
        Self { member, tab, draft: IndexMap::new(), class_slots: Vec::new(), extra_slots: Vec::new(), using_default: true, skills_loaded: false, gear: None, pending_switch: None, scroll: 0, gear_scroll: 0, detail: String::new(), w: Vec::new() }
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
                self.load_skills(cx);
                self.using_default = false;
                let path: Vec<String> = {
                    let m = self.m(cx);
                    let arch = d.cat.classes.get(&m.class_id(&d));
                    arch.map(|a| emberwake_rules::unit::families_of(&d.cat, &a.path_steps().into_iter().map(|(_, s)| s).collect::<Vec<_>>())).unwrap_or_default()
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
            PAct::Unequip(slot) => self.gear_move(cx, |_, work, stash| unequip(work, stash, &slot)),
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
    if let Some(old) = work.equipment.insert(slot.to_string(), item.to_string()) {
        give_one(stash, old);
    }
    take_one(stash, item);
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
        ("equip", "stash") => unequip(work, stash, what),
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
        let at = if self.tab == "gear" { &mut self.gear_scroll } else { &mut self.scroll };
        *at = (*at as i64 - lines as i64).max(0) as usize;
    }

    /// GEARUI1: a drop is valid when the move it makes is (the same rules as the clicks).
    fn can_drop(&self, cx: &Cx, from: &str, to: &str) -> bool {
        self.tab == "gear" && self.after_drop(&cx.data, from, to).is_ok()
    }

    fn accept_drop(&mut self, cx: &mut Cx, from: &str, to: &str) {
        self.load_gear(cx);
        match self.after_drop(&cx.data, from, to) {
            Ok(draft) => self.gear = Some(draft),
            Err(e) => show_errors(cx, vec![e]),
        }
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
        ui.push("stats.passives", "row", line, Rect::new(66, 256, (label_x - 74.0) as i32, 13), true, false, None, tip("Passives", &if about.is_empty() { "None yet: passives come from the hero's background, faction, disciplines and named items.".into() } else { about.join(" ") }));
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
        ui.p.text(68.0, 104.0, if self.using_default { "Default loadout (picked by the class)" } else { "Custom loadout" }, BODY + 1.0, DIM);
        let fam_name = |f: &str| cat.tier_one(f).map(|s| s.name.clone()).unwrap_or_else(|| f.to_string());
        let mut y = 118;
        for (class, list, limit) in [(true, self.class_slots.clone(), emberwake_rules::stats::MAX_CLASS_SLOTS), (false, self.extra_slots.clone(), emberwake_rules::stats::MAX_EXTRA_SLOTS)] {
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
            let mut tiers: Vec<&emberwake_data::SkillDef> = cat.skills.iter().filter(|s| cat.family_of(&s.id) == f).collect();
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
            unequip(&mut w, &mut st, slot).is_ok()
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
