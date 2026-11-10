//! Equipment (tiers, wield penalty, mitigation) and Inventory (scripts/core/inventory/).

use crate::stats::{roundi, CharacterStats, Proficiency};
pub use crate::stats::WieldRule;
use aegis_core::value::{self, Obj};
use rpgkit_data::{ItemDef, ALWAYS_RANGED_RULE, FOCUS_RULE, SHIELD_RULE};
use indexmap::IndexMap;

/// StatModifier source of every gear tier stat bonus.
pub const TIER_SOURCE: &str = "gear_tier";
/// The stat-deficit ramp (FB-33: kept as built): weapon power and armour lose this share per missing requirement point
/// (after the requirement discount) ...
pub const WIELD_DEFICIT_STEP: f64 = 0.15;
/// ... up to this share.
pub const WIELD_DEFICIT_CAP: f64 = 0.60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Slot {
    MainHand,
    OffHand,
    Head,
    Body,
    Accessory1,
    Accessory2,
}

impl Slot {
    pub const ALL: [Slot; 6] = [Slot::MainHand, Slot::OffHand, Slot::Head, Slot::Body, Slot::Accessory1, Slot::Accessory2];

    /// From a loadout key (`main_hand`) or an engine name (`MAIN_HAND`).
    pub fn parse(s: &str) -> Option<Slot> {
        match s.to_uppercase().as_str() {
            "MAIN_HAND" => Some(Slot::MainHand),
            "OFF_HAND" => Some(Slot::OffHand),
            "HEAD" => Some(Slot::Head),
            "BODY" => Some(Slot::Body),
            "ACCESSORY_1" => Some(Slot::Accessory1),
            "ACCESSORY_2" => Some(Slot::Accessory2),
            _ => None,
        }
    }

    pub fn engine_name(self) -> &'static str {
        ["MAIN_HAND", "OFF_HAND", "HEAD", "BODY", "ACCESSORY_1", "ACCESSORY_2"][self as usize]
    }
}

#[derive(Debug, Clone)]
pub struct Equipment {
    pub item: ItemDef,
}

/// U3.2-S3 (U3 soft wield, FB-33): what wearing an item costs its wearer. `ratio` and `deficit` are the stat-deficit
/// ramp (`WIELD_DEFICIT_STEP` per missing point, capped at `WIELD_DEFICIT_CAP`); `untrained` is proficiency 0 in its gear
/// type (`trained_for` false), which multiplies by `untrained_mult` and adds `burden_add`; `penalized` is either.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WieldPenalty {
    pub penalized: bool,
    pub ratio: f64,
    pub deficit: i32,
    pub untrained: bool,
    /// The wield terms' `untrained_power_mult` while `untrained` (unused otherwise).
    pub untrained_mult: f64,
    /// The wield terms' `untrained_burden_add` while `untrained` (0 otherwise).
    pub burden_add: i32,
}

impl WieldPenalty {
    /// The factor on the item's weapon power and armour: `(1 - ratio) x untrained_mult`, the two stacking
    /// multiplicatively; 1.0 without a penalty.
    pub fn factor(&self) -> f64 {
        (1.0 - self.ratio) * if self.untrained { self.untrained_mult } else { 1.0 }
    }
}

impl Equipment {
    pub fn new(item: &ItemDef) -> Self {
        Self { item: item.clone() }
    }

    pub fn id(&self) -> &str {
        &self.item.id
    }

    pub fn weapon_type(&self) -> &str {
        &self.item.weapon_type
    }

    pub fn is_weapon(&self) -> bool {
        self.item.item_type == "WEAPON"
    }

    /// U3.2-S2: the item's family has the engine rule `shield`.
    pub fn is_shield(&self) -> bool {
        self.item.has_rule(SHIELD_RULE)
    }

    /// Equipment.is_ranged_weapon; U3.2-S2: a family with the engine rule `always_ranged`, or a base range above 1.
    pub fn is_ranged_weapon(&self) -> bool {
        self.is_weapon() && (self.item.has_rule(ALWAYS_RANGED_RULE) || self.item.base_range > 1)
    }

    pub fn is_melee_weapon(&self) -> bool {
        self.is_weapon() && !self.is_ranged_weapon()
    }

    /// RULESET1 K1: a melee weapon whose family has the engine rule `reach` (a gear fact; the battle reads it only
    /// under the rule `reach`).
    pub fn has_reach(&self) -> bool {
        self.is_melee_weapon() && self.item.reach_rule
    }

    /// PR5-28 (U3 FB-36c): the far edge of a reach weapon's melee band: its family's `range_band.max`, else
    /// `REACH_WEAPON_REACH` (a family without a `range_band`, or a catalog frozen before it).
    pub fn reach_max(&self) -> i32 {
        if self.item.family_reach_max > 0 {
            self.item.family_reach_max
        } else {
            crate::unit::REACH_WEAPON_REACH
        }
    }

    fn discount(&self, prof: &Proficiency) -> i32 {
        if self.item.weapon_type.is_empty() {
            0
        } else {
            prof.requirement_discount(&self.item.weapon_type)
        }
    }

    pub fn meets_requirements(&self, stats: &CharacterStats) -> bool {
        let discount = self.discount(&stats.proficiency);
        // R4-1 slice 2 (FB-144): a templated unit's requirements read its unscaled stats (`requirement_stat`)
        self.item.required_stats.iter().all(|(s, req)| stats.requirement_stat(s, Some(&self.item.weapon_type)) + discount >= *req)
    }

    /// U3.2-S3 (FB-33): the stat-deficit ramp (the largest missing point count after the requirement discount) and the
    /// proficiency 0 penalty of the wield terms (`stats.proficiency.wield`), stacking (`WieldPenalty::factor`).
    pub fn wield_penalty(&self, stats: &CharacterStats) -> WieldPenalty {
        let mut pen = WieldPenalty::default();
        if !self.item.required_stats.is_empty() && !self.meets_requirements(stats) {
            let discount = self.discount(&stats.proficiency);
            let mut deficit = 0;
            for (s, req) in &self.item.required_stats {
                let eff = stats.requirement_stat(s, Some(&self.item.weapon_type)) + discount;
                if eff < *req {
                    deficit = deficit.max(req - eff);
                }
            }
            (pen.ratio, pen.deficit) = ((deficit as f64 * WIELD_DEFICIT_STEP).min(WIELD_DEFICIT_CAP), deficit);
        }
        if !trained_for(&stats.proficiency, &self.item) {
            let w = stats.proficiency.wield;
            (pen.untrained, pen.untrained_mult, pen.burden_add) = (true, w.untrained_power_mult, w.untrained_burden_add);
        }
        pen.penalized = pen.deficit > 0 || pen.untrained;
        pen
    }

    fn tier_when(tier: &Obj) -> (String, String, String, i32) {
        // (type, stat, gear, value/level)
        if let Some(when) = value::obj(tier, "when").filter(|w| !w.is_empty()) {
            let kind = value::string(when, "type", "");
            let n = if kind == "proficiency_at_least" { value::int(when, "level", 0) } else { value::int(when, "value", 0) };
            return (kind, value::string(when, "stat", "STR"), value::string(when, "gear", ""), n);
        }
        ("stat_at_least".into(), value::string(tier, "stat", "STR"), String::new(), value::int(tier, "threshold", 0))
    }

    fn tier_bonus(tier: &Obj) -> Obj {
        match value::obj(tier, "bonus").filter(|b| !b.is_empty()) {
            Some(b) => b.clone(),
            None => {
                let mut o = Obj::new();
                o.insert("power".into(), value::int(tier, "bonus_power", 0).into());
                o
            }
        }
    }

    /// Tier criteria are read on stats WITHOUT tier stat bonuses (FORGE1_ITEMS 4.3).
    pub fn tier_met(&self, tier: &Obj, stats: &CharacterStats) -> bool {
        let (kind, stat, gear, n) = Self::tier_when(tier);
        match kind.as_str() {
            "stat_at_least" => stats.effective(&stat, Some(&self.item.weapon_type), TIER_SOURCE) + self.discount(&stats.proficiency) >= n,
            "proficiency_at_least" => {
                let g = if gear.is_empty() { self.item.weapon_type.clone() } else { gear };
                stats.proficiency.level_of(&g) >= n
            }
            "unit_level_at_least" => stats.level >= n,
            _ => false,
        }
    }

    pub fn active_tiers<'a>(&'a self, stats: &'a CharacterStats) -> impl Iterator<Item = (usize, &'a Obj)> + 'a {
        self.item.threshold_tiers.iter().enumerate().filter(move |(_, t)| self.tier_met(t, stats))
    }

    pub fn tier_power_boost(&self, stats: &CharacterStats) -> i32 {
        self.active_tiers(stats).map(|(_, t)| value::int(t, "bonus_power", 0)).sum()
    }

    /// The item's armour: its mitigation, plus its unlocked tiers' with `stats`; U3.2-S3 (FB-33): times the wearer's
    /// wield penalty factor (stat deficit and proficiency 0), rounded.
    pub fn effective_mitigation(&self, stats: Option<&CharacterStats>) -> i32 {
        match stats {
            None => self.item.mitigation,
            Some(st) => {
                let raw = self.item.mitigation + self.active_tiers(st).map(|(_, t)| value::int(&Self::tier_bonus(t), "mitigation", 0)).sum::<i32>();
                let pen = self.wield_penalty(st);
                if pen.penalized && raw != 0 {
                    roundi(raw as f64 * pen.factor())
                } else {
                    raw
                }
            }
        }
    }

    pub fn tier_stat_bonuses(&self, stats: &CharacterStats) -> IndexMap<String, i32> {
        let mut out: IndexMap<String, i32> = IndexMap::new();
        for (_, t) in self.active_tiers(stats) {
            let bonus = Self::tier_bonus(t);
            if let Some(b) = value::obj(&bonus, "stat_bonuses") {
                for (k, v) in b {
                    *out.entry(k.to_uppercase()).or_insert(0) += value::to_int(v, 0);
                }
            }
        }
        out
    }

    /// Effects of each unlocked tier, with its 1-based position.
    pub fn active_tier_effects(&self, stats: &CharacterStats) -> Vec<(usize, Vec<serde_json::Value>)> {
        self.active_tiers(stats)
            .map(|(i, t)| (i + 1, value::array(&Self::tier_bonus(t), "effects").to_vec()))
            .filter(|(_, fx)| !fx.is_empty())
            .collect()
    }
}

/// ContentFactory.item_fits_slot with worn gear kept: equipment whose slot is `slot` (a slot key like "main_hand"),
/// either accessory slot for an accessory, and a one-handed main-hand weapon in the off hand.
pub fn item_fits_slot(item: &ItemDef, slot: &str) -> bool {
    let up = slot.to_uppercase();
    item.is_equipment() && (item.slot == up || (up.starts_with("ACCESSORY") && item.slot.starts_with("ACCESSORY")) || (up == "OFF_HAND" && item.slot == "MAIN_HAND" && !item.two_handed && item.item_type == "WEAPON"))
}

/// The two-handed rule (decision "Two-handed weapon and the off hand (user 2026-10-20)", the Godot gear dojo):
/// under a two-handed main weapon the off hand holds only a weapon that is not a focus (Equipment.is_focus,
/// conflicts_with_main; U3.2-S2: a family with the engine rule `focus`).
pub fn off_hand_ok(main: Option<&ItemDef>, off: &ItemDef) -> bool {
    !main.is_some_and(|m| m.two_handed) || (off.item_type == "WEAPON" && !off.has_rule(FOCUS_RULE))
}

/// ClassArchetype.can_equip_gear, U3.2-S3 (FB-33, FB-34): no longer a gate (any unit may wear any item) but whether the
/// item is worn trained, free of the proficiency 0 penalty: accessories always; gear with no weapon type, or a class with
/// no proficiency list (or a unit with no class, `Proficiency::unrestricted`), too; otherwise training (level 1) in the
/// gear type. UNITDRILL1: `prof` is the class's proficiency plus the wearer's drills (`Proficiency::with_drills`), read
/// through `level_of` as `min_proficiency` reads it.
pub fn trained_for(prof: &Proficiency, item: &ItemDef) -> bool {
    item.item_type == "ACCESSORY" || item.slot.starts_with("ACCESSORY") || item.weapon_type.is_empty() || prof.entries.is_empty() || prof.unrestricted || prof.level_of(&item.weapon_type) >= 1
}

/// A1.2-K3 (INTENT2-K3): whether one more copy of `id` (stack limit `limit`) fits a pouch holding `ids` (repeated ids)
/// with `slots` slots: under `stacks` (ruleset `pouch_stacks`) at most `slots` distinct ids, each at most `limit` copies;
/// otherwise (classic) at most `slots` copies in all.
pub fn pouch_fits<'a>(ids: impl Iterator<Item = &'a str>, slots: usize, stacks: bool, id: &str, limit: i32) -> bool {
    let ids: Vec<&str> = ids.collect();
    if !stacks {
        return ids.len() < slots;
    }
    let held = ids.iter().filter(|x| **x == id).count() as i32;
    if held > 0 {
        return held < limit;
    }
    let mut distinct: Vec<&str> = Vec::new();
    for x in ids {
        if !distinct.contains(&x) {
            distinct.push(x);
        }
    }
    distinct.len() < slots && limit > 0
}

#[derive(Debug, Clone, Default)]
pub struct Inventory {
    /// Equipped gear in equip order (Godot iterates its slot dictionary in insertion order).
    pub equipped: Vec<(Slot, Equipment)>,
    pub pouch: Vec<ItemDef>,
    pub max_pouch_slots: usize,
    /// The off-hand weapon is the ready one (Inventory.active_weapon_slot == OFF_HAND); only meaningful while a
    /// two-handed main weapon is paired with an off-hand weapon (`needs_weapon_switch`).
    pub off_active: bool,
    /// A1.2-K3 (INTENT2-K3, ruleset `pouch_stacks`): the pouch holds `max_pouch_slots` distinct items, each a stack of at
    /// most its `stack_limit` copies; off (classic): `max_pouch_slots` copies, any mix.
    pub pouch_stacks: bool,
}

impl Inventory {
    pub fn new() -> Self {
        Self { equipped: Vec::new(), pouch: Vec::new(), max_pouch_slots: 3, off_active: false, pouch_stacks: false }
    }

    pub fn equip(&mut self, eq: Equipment, slot: Slot) {
        match self.equipped.iter_mut().find(|(s, _)| *s == slot) {
            Some(entry) => entry.1 = eq,
            None => self.equipped.push((slot, eq)),
        }
        self.normalize_active_weapon();
    }

    /// Removes and returns what `slot` holds (the slot leaves the equip order).
    pub fn unequip(&mut self, slot: Slot) -> Option<Equipment> {
        let i = self.equipped.iter().position(|(s, _)| *s == slot)?;
        let eq = self.equipped.remove(i).1;
        self.normalize_active_weapon();
        Some(eq)
    }

    /// Inventory._normalize_active_weapon: without a 2H pair the main hand is the ready one.
    fn normalize_active_weapon(&mut self) {
        if !self.needs_weapon_switch() {
            self.off_active = false;
        }
    }

    pub fn get(&self, slot: Slot) -> Option<&Equipment> {
        self.equipped.iter().find(|(s, _)| *s == slot).map(|(_, e)| e)
    }

    /// Inventory.needs_weapon_switch: a two-handed main weapon paired with an off-hand weapon share one ready slot.
    pub fn needs_weapon_switch(&self) -> bool {
        self.main_weapon().is_some_and(|m| m.item.two_handed) && self.offhand_weapon().is_some()
    }

    /// Inventory.is_offhand_active.
    pub fn is_offhand_active(&self) -> bool {
        self.needs_weapon_switch() && self.off_active
    }

    /// Inventory.get_active_weapon: the ready weapon (the off-hand one while it is active).
    pub fn active_weapon(&self) -> Option<&Equipment> {
        if self.is_offhand_active() { self.offhand_weapon() } else { self.main_weapon() }
    }

    /// Inventory.toggle_active_weapon: swaps the ready weapon; false (and the main hand ready) when nothing switches.
    pub fn toggle_active_weapon(&mut self) -> bool {
        if !self.needs_weapon_switch() {
            self.off_active = false;
            return false;
        }
        self.off_active = !self.off_active;
        true
    }

    pub fn main_weapon(&self) -> Option<&Equipment> {
        self.get(Slot::MainHand)
    }

    pub fn offhand_weapon(&self) -> Option<&Equipment> {
        self.get(Slot::OffHand).filter(|e| e.item.item_type == "WEAPON")
    }

    pub fn shield(&self) -> Option<&Equipment> {
        self.get(Slot::OffHand).filter(|e| e.is_shield())
    }

    pub fn gear(&self) -> impl Iterator<Item = &Equipment> {
        self.equipped.iter().map(|(_, e)| e)
    }

    /// AUDIT-01: the hand of a 2H main + off-hand weapon pair that is not ready (`needs_weapon_switch`): the main hand
    /// while the off-hand weapon is active, else the off hand. None without a pair.
    pub fn stowed_slot(&self) -> Option<Slot> {
        if !self.needs_weapon_switch() {
            return None;
        }
        Some(if self.off_active { Slot::MainHand } else { Slot::OffHand })
    }

    /// AUDIT-01: worn gear without the stowed weapon of a 2H pair (`stowed_slot`): the gear whose effects, tier
    /// effects and family bonuses count (a backup weapon grants none of them while it is not ready).
    pub fn ready_gear(&self) -> impl Iterator<Item = &Equipment> {
        let stowed = self.stowed_slot();
        self.equipped.iter().filter(move |(s, _)| Some(*s) != stowed).map(|(_, e)| e)
    }

    /// AUDIT-08: the ready gear's armour (`ready_gear`; a stowed backup weapon adds none).
    pub fn total_flat_mitigation(&self, stats: Option<&CharacterStats>) -> i32 {
        self.ready_gear().map(|e| e.effective_mitigation(stats)).sum()
    }

    pub fn add_to_pouch(&mut self, item: &ItemDef) -> bool {
        if !pouch_fits(self.pouch.iter().map(|p| p.id.as_str()), self.max_pouch_slots, self.pouch_stacks, &item.id, item.stack_limit()) {
            return false;
        }
        self.pouch.push(item.clone());
        true
    }
}
