//! StatRules, StatModifier, Proficiency and CharacterStats (scripts/core/specs/).

use rpgkit_data::{Catalog, ClassDef, ProfEntry, STATS};
use indexmap::IndexMap;

/// The level 1 unit budget (`unit_budget::BASE_BUDGET`).
pub const CLASS_BUDGET: i32 = rpgkit_data::unit_budget::BASE_BUDGET;
pub const SPEND_BASE: i32 = 4;
pub const FREE_PER_LEVEL: i32 = 1;
pub const MAX_LEVEL: i32 = 10;
pub const MAX_CLASS_SLOTS: usize = 5;
pub const MAX_EXTRA_SLOTS: usize = 3;
pub const TIER_UNLOCK_STEP: i32 = 2;
pub const MAX_UNLOCK_REDUCTION: i32 = 2;
pub const ADVANCED_UNLOCK_FLOOR: i32 = 3;
pub const KNOW_UNLOCK_BASE: i32 = 12;
pub const KNOW_UNLOCK_STEP: i32 = 4;
pub const MINION_SKILL_CAP: i32 = 2;
pub const HP_PER_CONS: i32 = 8;
pub const RANGE_PERC_THRESHOLD: i32 = 18;

/// Godot's `roundf` / `roundi`: half away from zero (same as `f64::round`).
pub fn roundi(x: f64) -> i32 {
    x.round() as i32
}

/// StatRules.TIERS: (hp scale, power scale, default AI skill cap).
pub fn tier_scales(tier: &str) -> (f64, f64, i32) {
    match tier {
        "minion" => (0.4, 0.6, MINION_SKILL_CAP),
        "standard" => (0.7, 0.8, 0),
        "elite" => (1.25, 1.0, 0),
        "boss" => (1.5, 1.1, 0),
        _ => (1.0, 1.0, 0),
    }
}

pub fn spend_pool(level: i32) -> i32 {
    SPEND_BASE + (level.clamp(1, MAX_LEVEL) - 1) * FREE_PER_LEVEL
}

/// Unlock level of a family that is not on the unit's class path: 2 x AP level - 1.
pub fn default_unlock(ap_level: i32) -> i32 {
    (2 * ap_level - 1).max(1)
}

pub fn know_unlock_reduction(know: i32) -> i32 {
    ((know - KNOW_UNLOCK_BASE) as f64 / KNOW_UNLOCK_STEP as f64).floor().clamp(0.0, MAX_UNLOCK_REDUCTION as f64) as i32
}

/// `points` spread over the stats in proportion to the class growth weights, heaviest first (the default spend).
pub fn auto_spend(growth: &IndexMap<String, i32>, points: i32) -> IndexMap<String, i32> {
    let mut spend: IndexMap<String, i32> = IndexMap::new();
    if points <= 0 {
        return spend;
    }
    let w = |s: &str| growth.get(s).copied().unwrap_or(0);
    let mut order: Vec<&str> = STATS.iter().copied().filter(|s| w(s) > 0).collect();
    if order.is_empty() {
        order = STATS.to_vec();
    }
    let pos = |s: &str| STATS.iter().position(|x| *x == s).unwrap_or(0);
    order.sort_by(|a, b| w(b).cmp(&w(a)).then(pos(a).cmp(&pos(b))));
    let weights: Vec<i32> = order.iter().map(|s| w(s).max(1)).collect();
    let weight_sum: i32 = weights.iter().sum();
    let mut given = 0;
    for (i, s) in order.iter().enumerate() {
        let share = ((points * weights[i]) as f64 / weight_sum as f64).floor() as i32;
        spend.insert(s.to_string(), share);
        given += share;
    }
    let mut i = 0;
    while given < points {
        *spend.entry(order[i % order.len()].to_string()).or_insert(0) += 1;
        given += 1;
        i += 1;
    }
    STATS.iter().filter_map(|s| spend.get(*s).filter(|v| **v > 0).map(|v| (s.to_string(), *v))).collect()
}

#[derive(Debug, Clone)]
pub struct StatModifier {
    pub stat: String,
    pub flat: i32,
    pub percent: f64,
    pub source: String,
    /// "" / "always" = every context; "key:value" = only when the context has that pair.
    pub context: String,
    /// -1 = permanent.
    pub duration: i32,
}

impl StatModifier {
    pub fn permanent(stat: &str, flat: i32, source: &str, context: &str) -> Self {
        Self { stat: stat.to_uppercase(), flat, percent: 0.0, source: source.into(), context: context.into(), duration: -1 }
    }

    pub fn matches(&self, weapon: Option<&str>) -> bool {
        if self.context.is_empty() || self.context == "always" {
            return true;
        }
        if let Some((key, val)) = self.context.split_once(':') {
            if val.contains(':') {
                return false;
            }
            return key == "weapon" && weapon.is_some_and(|w| w.to_lowercase() == val.to_lowercase());
        }
        false
    }
}

pub const PROF_MAX_LEVEL: i32 = 3;
pub const DEFAULT_SCALING: f64 = 1.5;
pub const PASSIVE_ONLY_SCALING: f64 = 1.25;

#[derive(Debug, Clone, Default)]
pub struct Proficiency {
    pub entries: IndexMap<String, ProfEntry>,
    pub bonus_levels: IndexMap<String, i32>,
}

impl Proficiency {
    pub fn from_class(class: &ClassDef) -> Self {
        let entries = class
            .proficiency
            .iter()
            .map(|(k, e)| (k.to_lowercase(), ProfEntry { level: e.level.clamp(0, PROF_MAX_LEVEL), scaling: e.scaling.clamp(1.0, 2.0) }))
            .collect();
        Self { entries, bonus_levels: IndexMap::new() }
    }

    pub fn add_bonus(&mut self, gear: &str, levels: i32) {
        *self.bonus_levels.entry(gear.to_lowercase()).or_insert(0) += levels;
    }

    /// UNITDRILL1: this proficiency plus the drills among `passive_ids`, the always-on passives (`slot` "") with a
    /// `proficiency` grant, read through the typed grant view as a built unit reads them (U4.2-S1, FB-27:
    /// `grants::read`, `Unit::apply_grants`). Unknown ids and slottable passives add nothing: a slottable passive trains
    /// only while slotted, which is a battle's deck, not the gear rule's.
    pub fn with_drills<'a>(mut self, cat: &Catalog, passive_ids: impl IntoIterator<Item = &'a String>) -> Self {
        for passive in passive_ids.into_iter().filter_map(|id| cat.passives.get(id)).filter(|p| p.slot.is_empty()) {
            for grant in crate::grants::read(&passive.effects) {
                if let Some((gear, levels)) = grant.proficiency() {
                    self.add_bonus(gear, levels);
                }
            }
        }
        self
    }

    pub fn base_level(&self, gear: &str) -> i32 {
        self.entries.get(&gear.to_lowercase()).map(|e| e.level).unwrap_or(0)
    }

    pub fn level_of(&self, gear: &str) -> i32 {
        let key = gear.to_lowercase();
        if key.is_empty() {
            return 0;
        }
        (self.base_level(&key) + self.bonus_levels.get(&key).copied().unwrap_or(0)).clamp(0, PROF_MAX_LEVEL)
    }

    pub fn scaling_of(&self, gear: &str) -> f64 {
        let key = gear.to_lowercase();
        match self.entries.get(&key) {
            Some(e) => e.scaling,
            None if self.level_of(&key) > 0 => PASSIVE_ONLY_SCALING,
            None => DEFAULT_SCALING,
        }
    }

    pub fn requirement_discount(&self, gear: &str) -> i32 {
        self.level_of(gear) * 2
    }

    /// Proficiency.can_use: no gear type, or at least level 1 in it.
    pub fn can_use(&self, gear: &str) -> bool {
        gear.is_empty() || self.level_of(gear) >= 1
    }

    pub fn power_bonus(&self, gear: &str) -> i32 {
        self.level_of(gear)
    }

    pub fn crit_bonus(&self, gear: &str) -> f64 {
        (self.level_of(gear) - 1).max(0) as f64
    }

    pub fn burden_reduction(&self, gear: &str) -> i32 {
        (self.level_of(gear) - 1).max(0)
    }
}

#[derive(Debug, Clone)]
pub struct CharacterStats {
    pub level: i32,
    pub unallocated: i32,
    pub core: IndexMap<String, i32>,
    pub proficiency: Proficiency,
    pub modifiers: Vec<StatModifier>,
}

impl CharacterStats {
    /// The class stats at the level (base and growth, `ClassDef::stats_at`; U2.2 unit budget) + `spend`.
    pub fn new(class: &ClassDef, level: i32, spend: &IndexMap<String, i32>) -> Self {
        let target = level.clamp(1, MAX_LEVEL);
        let mut core = class.stats_at(target);
        let mut spent = 0;
        for s in STATS {
            let pts = spend.get(s).copied().unwrap_or(0).max(0);
            *core.get_mut(s).unwrap() += pts;
            spent += pts;
        }
        Self { level: target, unallocated: (spend_pool(target) - spent).max(0), core, proficiency: Proficiency::from_class(class), modifiers: Vec::new() }
    }

    pub fn base(&self, stat: &str) -> i32 {
        self.core.get(&stat.to_uppercase()).copied().unwrap_or(10)
    }

    /// Base + matching modifiers (flat then percent), minimum 1. `exclude_source` leaves one source out.
    pub fn effective(&self, stat: &str, weapon: Option<&str>, exclude_source: &str) -> i32 {
        let clean = stat.to_uppercase();
        let mut flat = 0;
        let mut pct = 0.0;
        for m in &self.modifiers {
            if !exclude_source.is_empty() && m.source == exclude_source {
                continue;
            }
            if m.stat.to_uppercase() == clean && m.matches(weapon) {
                flat += m.flat;
                pct += m.percent;
            }
        }
        roundi((self.base(&clean) + flat) as f64 * (1.0 + pct)).max(1)
    }

    pub fn stat(&self, stat: &str, weapon: Option<&str>) -> i32 {
        self.effective(stat, weapon, "")
    }

    pub fn max_hp(&self, weapon: Option<&str>) -> i32 {
        60 + self.stat("CONS", weapon) * HP_PER_CONS + self.stat("STR", weapon) * 2
    }

    pub fn initiative_speed(&self, burden: i32, weapon: Option<&str>) -> i32 {
        (8 + self.stat("DEX", weapon) - burden).max(1)
    }

    pub fn mitigation_rating(&self, gear: i32, weapon: Option<&str>) -> i32 {
        gear + roundi(self.stat("STR", weapon) as f64 * 0.25)
    }

    pub fn effective_range(&self, base_range: i32, weapon: Option<&str>) -> i32 {
        let bonus = (self.stat("PERC", weapon) >= RANGE_PERC_THRESHOLD && base_range > 1) as i32;
        (base_range + bonus).max(1)
    }

    pub fn crit_rate(&self, weapon: Option<&str>) -> f64 {
        crate::damage::CRIT_BASE + (self.stat("DEX", weapon) - crate::damage::CRIT_DEX_FLOOR).max(0) as f64 * crate::damage::CRIT_PER_DEX
    }

    pub fn debuff_resistance(&self, weapon: Option<&str>) -> f64 {
        self.stat("INT", weapon) as f64 * 2.5
    }

    pub fn remove_modifier(&mut self, source: &str) {
        self.modifiers.retain(|m| m.source != source);
    }
}
