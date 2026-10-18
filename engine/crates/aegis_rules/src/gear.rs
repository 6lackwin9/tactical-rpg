//! Equipment (tiers, wield penalty, mitigation) and Inventory (scripts/core/inventory/).

use crate::stats::{CharacterStats, Proficiency};
use aegis_data::value::{self, Obj};
use aegis_data::ItemDef;
use indexmap::IndexMap;

/// StatModifier source of every gear tier stat bonus.
pub const TIER_SOURCE: &str = "gear_tier";
const ALWAYS_RANGED_TYPES: [&str; 3] = ["bow", "crossbow", "sling"];

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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WieldPenalty {
    pub penalized: bool,
    pub ratio: f64,
    pub extra_block_cost: i32,
    pub deficit: i32,
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

    pub fn is_shield(&self) -> bool {
        self.item.item_type == "SHIELD" || self.item.off_hand_category == "shield"
    }

    pub fn is_ranged_weapon(&self) -> bool {
        self.is_weapon() && (ALWAYS_RANGED_TYPES.contains(&self.item.weapon_type.to_lowercase().as_str()) || self.item.base_range > 1)
    }

    pub fn is_melee_weapon(&self) -> bool {
        self.is_weapon() && !self.is_ranged_weapon()
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
        self.item.required_stats.iter().all(|(s, req)| stats.stat(s, Some(&self.item.weapon_type)) + discount >= *req)
    }

    pub fn wield_penalty(&self, stats: &CharacterStats) -> WieldPenalty {
        if self.item.required_stats.is_empty() || self.meets_requirements(stats) {
            return WieldPenalty::default();
        }
        let discount = self.discount(&stats.proficiency);
        let mut deficit = 0;
        for (s, req) in &self.item.required_stats {
            let eff = stats.stat(s, Some(&self.item.weapon_type)) + discount;
            if eff < *req {
                deficit = deficit.max(req - eff);
            }
        }
        WieldPenalty { penalized: true, ratio: (deficit as f64 * 0.15).min(0.60), extra_block_cost: (deficit >= 4) as i32, deficit }
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

    pub fn effective_mitigation(&self, stats: Option<&CharacterStats>) -> i32 {
        match stats {
            None => self.item.mitigation,
            Some(st) => self.item.mitigation + self.active_tiers(st).map(|(_, t)| value::int(&Self::tier_bonus(t), "mitigation", 0)).sum::<i32>(),
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

    pub fn effective_block_cost(&self, stats: &CharacterStats) -> i32 {
        let mut cost = self.item.block_cost + self.wield_penalty(stats).extra_block_cost;
        if !self.item.weapon_type.is_empty() {
            cost -= stats.proficiency.block_cost_reduction(&self.item.weapon_type);
        }
        cost.max(1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Inventory {
    /// Equipped gear in equip order (Godot iterates its slot dictionary in insertion order).
    pub equipped: Vec<(Slot, Equipment)>,
    pub pouch: Vec<ItemDef>,
    pub max_pouch_slots: usize,
}

impl Inventory {
    pub fn new() -> Self {
        Self { equipped: Vec::new(), pouch: Vec::new(), max_pouch_slots: 3 }
    }

    pub fn equip(&mut self, eq: Equipment, slot: Slot) {
        match self.equipped.iter_mut().find(|(s, _)| *s == slot) {
            Some(entry) => entry.1 = eq,
            None => self.equipped.push((slot, eq)),
        }
    }

    pub fn get(&self, slot: Slot) -> Option<&Equipment> {
        self.equipped.iter().find(|(s, _)| *s == slot).map(|(_, e)| e)
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

    pub fn total_flat_mitigation(&self, stats: Option<&CharacterStats>) -> i32 {
        self.gear().map(|e| e.effective_mitigation(stats)).sum()
    }

    pub fn add_to_pouch(&mut self, item: &ItemDef) -> bool {
        if self.pouch.len() >= self.max_pouch_slots {
            return false;
        }
        self.pouch.push(item.clone());
        true
    }
}
