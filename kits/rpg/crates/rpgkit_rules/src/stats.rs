//! StatRules, StatModifier, Proficiency and CharacterStats (scripts/core/specs/).

use rpgkit_data::unit_budget::Shares;
use rpgkit_data::{Catalog, ClassDef, ProfEntry, STATS};
use indexmap::IndexMap;
use serde_json::Value;
use std::sync::LazyLock;

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
pub const HP_PER_CONS: i32 = 8;
pub const RANGE_PERC_THRESHOLD: i32 = 18;

/// Godot's `roundf` / `roundi`: half away from zero (same as `f64::round`).
pub fn roundi(x: f64) -> i32 {
    x.round() as i32
}

/// U1.2-S4 (U1_STATS feasibility 6, U2_CLASS feasibility 2): one row of the kit's tier table `tiers.json` (beside this
/// file, embedded at build time; StatRules.TIERS before): `hp` is the formula input `tier_hp` (it multiplies the
/// whole max HP for now; the `60 x tier_hp + stats` form is held, FB-78), `power` is `tier_power` (attack power's base-stage multiplier), `skill_cap` the default AI skill cap
/// (0 = none).
#[derive(Debug, Clone, PartialEq)]
pub struct TierRow {
    pub id: String,
    pub hp: f64,
    pub power: f64,
    pub skill_cap: i32,
}

/// The tier a unit has unless its row names one (and the row an unknown id reads as).
pub const HERO_TIER: &str = "hero";

const TIERS_TEXT: &str = include_str!("tiers.json");

/// The tier table's rows in file order.
pub static TIERS: LazyLock<Vec<TierRow>> = LazyLock::new(|| {
    let rows: Value = serde_json::from_str(TIERS_TEXT).expect("tiers.json: valid JSON");
    let rows = rows.as_array().expect("tiers.json: an array of rows");
    rows.iter()
        .enumerate()
        .map(|(i, r)| {
            let num = |k: &str| r.get(k).and_then(Value::as_f64).unwrap_or_else(|| panic!("tiers.json row {i}: `{k}` is not a number"));
            let id = r.get("id").and_then(Value::as_str).unwrap_or_else(|| panic!("tiers.json row {i}: `id` is not a string"));
            TierRow { id: id.to_string(), hp: num("hp"), power: num("power"), skill_cap: num("skill_cap") as i32 }
        })
        .collect()
});

/// The row of `tier`; an id the table lacks reads as the hero row (factors 1, no cap), as before.
pub fn tier_row(tier: &str) -> &'static TierRow {
    TIERS.iter().find(|r| r.id == tier).or_else(|| TIERS.iter().find(|r| r.id == HERO_TIER)).expect("tiers.json: a hero row")
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

/// Shares are weighed in billionths (`unit_budget`'s quota unit), so float noise in a stored share (the writer's 15
/// digits) never changes an order or a floor.
const SHARE_TICKS: f64 = 1_000_000_000.0;

/// `points` spread over the stats in proportion to the class growth shares (`ClassDef::unit_shares().growth`: a share
/// row's `growth_shares`, an integer row's growth / 5; U2.2-S3, FB-46), heaviest first (the default spend). Each stat
/// with a share gets the floor of its proportional part, the points left go one each in that order; with no share at
/// all every stat weighs the same.
pub fn auto_spend(growth: &Shares, points: i32) -> IndexMap<String, i32> {
    let mut spend: IndexMap<String, i32> = IndexMap::new();
    if points <= 0 {
        return spend;
    }
    let w = |s: &str| growth.get(s).map(|x| (x * SHARE_TICKS).round() as i64).unwrap_or(0);
    let mut order: Vec<&str> = STATS.iter().copied().filter(|s| w(s) > 0).collect();
    if order.is_empty() {
        order = STATS.to_vec();
    }
    let pos = |s: &str| STATS.iter().position(|x| *x == s).unwrap_or(0);
    order.sort_by(|a, b| w(b).cmp(&w(a)).then(pos(a).cmp(&pos(b))));
    let weights: Vec<i64> = order.iter().map(|s| w(s).max(1)).collect();
    let weight_sum: i64 = weights.iter().sum();
    let mut given = 0;
    for (i, s) in order.iter().enumerate() {
        let share = (points as i64 * weights[i]).div_euclid(weight_sum) as i32;
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
/// U3.2-S3 (U3 "Fixed vs parameter", FB-33): the default of the ruleset number `untrained_power_mult`, the factor on an
/// untrained item's weapon power and armour (proficiency 0 in its gear type).
pub const UNTRAINED_POWER_MULT: f64 = 0.75;
/// U3.2-S3: the default of `untrained_burden_add`, the burden an untrained item adds.
pub const UNTRAINED_BURDEN_ADD: i32 = 1;
/// U3.2-S3: the default of `req_discount_per_level`, the stat requirement discount per proficiency level.
pub const REQ_DISCOUNT_PER_LEVEL: i32 = 2;

/// U3.2-S3 (FB-33, FB-34): the wield terms of the battle's ruleset (its row-only numbers `untrained_power_mult`,
/// `untrained_burden_add`, `req_discount_per_level`; `Numbers::wield`). `Default` is the spec's defaults, so a unit
/// built outside a battle (the party screen, the content check) reads them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WieldRule {
    pub untrained_power_mult: f64,
    pub untrained_burden_add: i32,
    pub req_discount_per_level: i32,
}

impl Default for WieldRule {
    fn default() -> Self {
        Self { untrained_power_mult: UNTRAINED_POWER_MULT, untrained_burden_add: UNTRAINED_BURDEN_ADD, req_discount_per_level: REQ_DISCOUNT_PER_LEVEL }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Proficiency {
    pub entries: IndexMap<String, ProfEntry>,
    pub bonus_levels: IndexMap<String, i32>,
    /// U3.2-S3: the wield terms its gear is read under (the ruleset's; `UnitBuild::wield`).
    pub wield: WieldRule,
    /// U3.2-S3 (FB-33): no gear restriction although `entries` is set: a unit with no class, which borrows the first
    /// class's stats (`build_catalog_unit`'s fallback), wears everything trained, like a class with no proficiency list.
    pub unrestricted: bool,
}

impl Proficiency {
    pub fn from_class(class: &ClassDef) -> Self {
        let entries = class
            .proficiency
            .iter()
            .map(|(k, e)| (k.to_lowercase(), ProfEntry { level: e.level.clamp(0, PROF_MAX_LEVEL), scaling: e.scaling.clamp(1.0, 2.0) }))
            .collect();
        Self { entries, bonus_levels: IndexMap::new(), wield: WieldRule::default(), unrestricted: false }
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

    /// U3.2-S3: `req_discount_per_level` (the wield terms') per level in `gear`.
    pub fn requirement_discount(&self, gear: &str) -> i32 {
        self.level_of(gear) * self.wield.req_discount_per_level
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

/// U1.2-S3c: the move curve (the one speed used): tiles one move covers, `floor((value + 2) / 5)` from 1 to 4.
pub fn move_rate_curve(value: i32) -> i32 {
    ((value + 2) as f64 / 5.0).floor().clamp(1.0, 4.0) as i32
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

    /// `round((60 + CONS x 8 + STR x 2) x tier_hp)` (U1.2-S4: `tier_hp` from the tier table; the old hp scale). The
    /// spec's base-only form, `60 x tier_hp + CONS x 8 + STR x 2`, waits for the user's confirmation as its own change.
    pub fn max_hp(&self, tier_hp: f64, weapon: Option<&str>) -> i32 {
        roundi((60 + self.stat("CONS", weapon) * HP_PER_CONS + self.stat("STR", weapon) * 2) as f64 * tier_hp)
    }

    /// U1.2-S3c: initiative, `8 + DEX - burden`, at least 1 (the old speed formula; the unit takes carry overload off
    /// it as well).
    pub fn initiative(&self, burden: i32, weapon: Option<&str>) -> i32 {
        (8 + self.stat("DEX", weapon) - burden).max(1)
    }

    /// U1.2-S3c (U1_STATS feasibility 4): the move rate, `move_rate_curve(8 + DEX - excess)`. `load` (worn burden plus
    /// carry overload) lowers initiative first; only the part that would take initiative below its floor of 1,
    /// `excess = max(0, 1 - (8 + DEX - load))`, lowers the move rate.
    pub fn move_rate(&self, load: i32, weapon: Option<&str>) -> i32 {
        let free = 8 + self.stat("DEX", weapon);
        let excess = (1 - (free - load)).max(0);
        move_rate_curve(free - excess)
    }

    pub fn mitigation_rating(&self, gear: i32, weapon: Option<&str>) -> i32 {
        gear + roundi(self.stat("STR", weapon) as f64 * 0.25)
    }

    pub fn effective_range(&self, base_range: i32, weapon: Option<&str>) -> i32 {
        let bonus = (self.stat("PERC", weapon) >= RANGE_PERC_THRESHOLD && base_range > 1) as i32;
        (base_range + bonus).max(1)
    }

    /// Crit chance before the weapon proficiency bonus: base + PERC above the floor (U1.2-S3, crit from PERC).
    pub fn crit_rate(&self, weapon: Option<&str>) -> f64 {
        crate::damage::CRIT_BASE + (self.stat("PERC", weapon) - crate::damage::CRIT_PERC_FLOOR).max(0) as f64 * crate::damage::CRIT_PER_PERC
    }

    pub fn debuff_resistance(&self, weapon: Option<&str>) -> f64 {
        self.stat("INT", weapon) as f64 * 2.5
    }

    pub fn remove_modifier(&mut self, source: &str) {
        self.modifiers.retain(|m| m.source != source);
    }
}
