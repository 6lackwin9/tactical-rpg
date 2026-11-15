//! Equipment (tiers, wield penalty, mitigation) and Inventory (scripts/core/inventory/).

use crate::stats::{roundi, CharacterStats, Proficiency};
pub use crate::stats::WieldRule;
use aegis_core::value::{self, Obj};
use rpgkit_data::{ItemDef, ALWAYS_RANGED_RULE, SHIELD_RULE};
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

/// PK2-S5b: the share of its weapon power a dulled weapon swings at (`rpgkit_data::DULLED_POWER_MULT`; kept in the data
/// crate beside the durability bound, so the `LOCKED` table and the rules contract stay as they were).
pub use rpgkit_data::DULLED_POWER_MULT;

/// PK2-S5b: one wielded weapon instance's wear in a battle: its durability as the battle began (`start`, more than 0
/// for a weapon that wears; an instance of an item that never wears is not tracked) and what the battle spent from it
/// so far (`spent`, counted past the start: the settle clamps the instance at 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Wear {
    pub start: i32,
    pub spent: i32,
}

impl Wear {
    /// The durability left now (0 or more).
    pub fn left(&self) -> i32 {
        (self.start - self.spent).max(0)
    }

    /// Whether the weapon is dulled now (no durability left).
    pub fn dulled(&self) -> bool {
        self.left() == 0
    }
}

#[derive(Debug, Clone)]
pub struct Equipment {
    pub item: ItemDef,
}

/// U3.2-S3 (U3 soft wield, FB-33; updated PR5-57): what wearing an item below `required_stats` costs its wearer on
/// non-strict paths or previews. `ratio` and `deficit` are the stat-deficit ramp (`WIELD_DEFICIT_STEP` per missing point,
/// capped at `WIELD_DEFICIT_CAP`); `penalized` is `deficit > 0`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WieldPenalty {
    pub penalized: bool,
    pub ratio: f64,
    pub deficit: i32,
}

impl WieldPenalty {
    /// The factor on the item's weapon power and armour: `1 - ratio`; 1.0 without a penalty.
    pub fn factor(&self) -> f64 {
        1.0 - self.ratio
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

    /// U3.2-S3 (FB-33; updated PR5-57): the stat-deficit ramp (the largest missing point count after the requirement
    /// discount); `untrained` soft fallback retired in PR5-57.
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
        pen.penalized = pen.deficit > 0;
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
    /// wield penalty factor (stat deficit and proficiency 0), rounded. GEAR-REWORK1 (PR5-51a): times the strict rule's
    /// proficiency factor in its gear type too (`Proficiency::gear_mult`: 0.80 / 1.00 / 1.25; 1.0 without the rule).
    pub fn effective_mitigation(&self, stats: Option<&CharacterStats>) -> i32 {
        match stats {
            None => self.item.mitigation,
            Some(st) => {
                let raw = self.item.mitigation + self.active_tiers(st).map(|(_, t)| value::int(&Self::tier_bonus(t), "mitigation", 0)).sum::<i32>();
                let pen = self.wield_penalty(st);
                let mult = self.prof_mult(st);
                if (pen.penalized || mult != 1.0) && raw != 0 {
                    roundi(raw as f64 * if pen.penalized { pen.factor() } else { 1.0 } * mult)
                } else {
                    raw
                }
            }
        }
    }

    /// GEAR-REWORK1 (PR5-51a): the strict rule's proficiency factor on this item's weapon power and flat mitigation
    /// (`Proficiency::gear_mult` in its gear type; accessories and gear without a type 1.0).
    pub fn prof_mult(&self, stats: &CharacterStats) -> f64 {
        if is_accessory(&self.item) {
            1.0
        } else {
            stats.proficiency.gear_mult(&self.item.weapon_type)
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
/// GEAR-REWORK1 (PR5-51a pass 2, U3.3 2): an item whose family names its hand category (`ItemDef::slot_rule`) goes where
/// the category says: `1h_light` in either hand, `1h_heavy` and the two-handed ones in the main hand only, `off_only` in
/// the off hand only; an item without one keeps the rule above.
pub fn item_fits_slot(item: &ItemDef, slot: &str) -> bool {
    let up = slot.to_uppercase();
    if !item.slot_rule.is_empty() && (up == "MAIN_HAND" || up == "OFF_HAND") {
        return item.is_equipment() && hands_of_rule(&item.slot_rule).contains(&up.as_str());
    }
    item.is_equipment() && (item.slot == up || (up.starts_with("ACCESSORY") && item.slot.starts_with("ACCESSORY")) || (up == "OFF_HAND" && item.slot == "MAIN_HAND" && !item.two_handed && item.item_type == "WEAPON"))
}

/// GEAR-REWORK1 (U3.3 2): the slots a hand category may fill (an unknown one: none).
pub fn hands_of_rule(rule: &str) -> &'static [&'static str] {
    match rule {
        rpgkit_data::SLOT_1H_LIGHT => &["MAIN_HAND", "OFF_HAND"],
        rpgkit_data::SLOT_1H_HEAVY | rpgkit_data::SLOT_2H_SOLO | rpgkit_data::SLOT_2H_RIG => &["MAIN_HAND"],
        rpgkit_data::SLOT_OFF_ONLY => &["OFF_HAND"],
        _ => &[],
    }
}

/// The two-handed rule. GEAR-REWORK1 (PR5-51a, `docs/sessions/U3_GEAR_REWORK.md` 1): `Switch Weapon` is retired, so a
/// two-handed main weapon leaves the off hand empty (the U3.2 2H main + stowed off-hand weapon pair is gone). Pass 2
/// (U3.3 2): except for an off-hand item of a family its family's `pairs_with` lists (a `2h_rig` bow and its ammo).
pub fn off_hand_ok(main: Option<&ItemDef>, off: &ItemDef) -> bool {
    match main {
        Some(m) if m.two_handed => !off.weapon_type.is_empty() && m.pairs_with.iter().any(|p| *p == off.weapon_type),
        _ => true,
    }
}

/// GEAR-REWORK1 (PR5-51a pass 2, U3.3 2-4): what an off-hand item does on the battle's Off-Hand button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OffHandUse {
    /// A `1h_light` weapon: the off-hand jab, reaching 1 to `reach` (the weapon's own reach, `light_reach`).
    Jab { reach: i32 },
    /// The family's `off_hand_skill` (granted while worn, `Unit::gear_skills`); `reload` when the family has an
    /// ammunition rule (`empower_next`: the skill refills its meter, open only at 0).
    Skill { skill: String, reload: bool },
}

/// GEAR-REWORK1 (U3.3 2): how far a `1h_light` weapon's off-hand jab reaches: its family's `range_band.max`
/// (`family_reach_max`) or its own base range, whichever is larger, at least 1 (thrown knives reach 2). None for any
/// other item (the jab then stays at distance 1).
pub fn light_reach(item: &ItemDef) -> Option<i32> {
    (item.slot_rule == rpgkit_data::SLOT_1H_LIGHT && item.item_type == "WEAPON").then(|| item.family_reach_max.max(item.base_range).max(1))
}

/// GEAR-REWORK1 (U3.3 4): the Off-Hand button's use of off-hand item `off` (None: it has none, as every item from
/// before the rework: the button then stays hidden).
pub fn off_hand_use(off: &ItemDef) -> Option<OffHandUse> {
    if !off.off_hand_skill.is_empty() {
        return Some(OffHandUse::Skill { skill: off.off_hand_skill.clone(), reload: off.empower_next.is_some() });
    }
    light_reach(off).map(|reach| OffHandUse::Jab { reach })
}

/// GEAR-REWORK1 (PR5-51a pass 3, U3.3 2 / 4): the Throw skill main-hand weapon `main` gains beside off-hand item `off`:
/// the off-hand ammunition rule's `throw_skill` when `main` is a `1h_light` weapon (a knife beside a throwing bandolier);
/// None otherwise (every item from before the rework).
pub fn light_throw<'a>(main: Option<&ItemDef>, off: &'a ItemDef) -> Option<&'a str> {
    let skill = off.empower_next.as_ref().map(|e| e.throw_skill.as_str()).filter(|s| !s.is_empty())?;
    main.and_then(light_reach).map(|_| skill)
}

/// ClassArchetype.can_equip_gear, U3.2-S3 (FB-33, FB-34): no longer a gate (any unit may wear any item) but whether the
/// item is worn trained, free of the proficiency 0 penalty: accessories always; gear with no weapon type, or a class with
/// no proficiency list (or a unit with no class, `Proficiency::unrestricted`), too; otherwise training (level 1) in the
/// gear type. UNITDRILL1: `prof` is the class's proficiency plus the wearer's drills (`Proficiency::with_drills`), read
/// through `level_of` as `min_proficiency` reads it. GEAR-REWORK1 (PR5-51a): under the strict gear rule an untrained
/// item is a hard equip lock instead (`wear_lock`).
pub fn trained_for(prof: &Proficiency, item: &ItemDef) -> bool {
    is_accessory(item) || item.weapon_type.is_empty() || prof.entries.is_empty() || prof.unrestricted || prof.level_of(&item.weapon_type) >= 1
}

/// GEAR-REWORK1 (U3.3 5): an accessory (item type `ACCESSORY` or an accessory slot): proficiency-free.
pub fn is_accessory(item: &ItemDef) -> bool {
    item.item_type == "ACCESSORY" || item.slot.starts_with("ACCESSORY")
}

/// GEAR-REWORK1 (PR5-51a, U3.3 1): why the strict gear rule refuses an item (`wear_lock`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WearLock {
    /// Proficiency 0 in the item's gear type.
    Untrained { gear: String },
    /// A raw `required_stats` entry above the wearer's unscaled stat plus the requirement discount: (stat, needed,
    /// had after the discount).
    Unfit { stat: String, need: i32, have: i32 },
}

impl WearLock {
    /// The refusal line ("{who} can't wear {item}: ...").
    pub fn text(&self, who: &str, item: &str) -> String {
        match self {
            WearLock::Untrained { gear } => format!("{who} can't wear {item}: untrained in {gear} (proficiency 0)."),
            WearLock::Unfit { stat, need, have } => format!("{who} can't wear {item}: needs {stat} {need} (has {have} with the proficiency discount)."),
        }
    }
}

/// GEAR-REWORK1 (PR5-51a, U3.3 1): the strict rule's hard equip lock (None: the item may be worn; always None when
/// `stats.proficiency` is not strict, `Proficiency::is_strict`: a class without roles keeps the soft wield penalty). Under
/// it, a non-accessory with a gear type at proficiency 0 is refused (`Untrained`), and any item (accessories included,
/// U3.3 5) whose raw `required_stats` exceed the wearer's unscaled stat (`requirement_base`, no gear bonuses) plus the
/// proficiency discount (`requirement_discount`) is refused (`Unfit`, the largest shortfall first in row order).
pub fn wear_lock(stats: &CharacterStats, item: &ItemDef) -> Option<WearLock> {
    let prof = &stats.proficiency;
    if !prof.is_strict() {
        return None;
    }
    if !trained_for(prof, item) {
        return Some(WearLock::Untrained { gear: item.weapon_type.clone() });
    }
    let discount = if item.weapon_type.is_empty() || is_accessory(item) { 0 } else { prof.requirement_discount(&item.weapon_type) };
    let mut worst: Option<WearLock> = None;
    let mut gap = 0;
    for (stat, need) in &item.required_stats {
        let have = stats.requirement_base(stat) + discount;
        if need - have > gap {
            gap = need - have;
            worst = Some(WearLock::Unfit { stat: stat.to_uppercase(), need: *need, have });
        }
    }
    worst
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
    /// A1.2-K3 (INTENT2-K3, ruleset `pouch_stacks`): the pouch holds `max_pouch_slots` distinct items, each a stack of at
    /// most its `stack_limit` copies; off (classic): `max_pouch_slots` copies, any mix.
    pub pouch_stacks: bool,
}

impl Inventory {
    pub fn new() -> Self {
        Self { equipped: Vec::new(), pouch: Vec::new(), max_pouch_slots: 3, pouch_stacks: false }
    }

    pub fn equip(&mut self, eq: Equipment, slot: Slot) {
        match self.equipped.iter_mut().find(|(s, _)| *s == slot) {
            Some(entry) => entry.1 = eq,
            None => self.equipped.push((slot, eq)),
        }
    }

    /// Removes and returns what `slot` holds (the slot leaves the equip order).
    pub fn unequip(&mut self, slot: Slot) -> Option<Equipment> {
        let i = self.equipped.iter().position(|(s, _)| *s == slot)?;
        Some(self.equipped.remove(i).1)
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

    /// The worn gear's armour. GEAR-REWORK1 (PR5-51a): every worn item counts (`Switch Weapon` and its stowed backup
    /// weapon, AUDIT-01/08, are retired).
    pub fn total_flat_mitigation(&self, stats: Option<&CharacterStats>) -> i32 {
        self.gear().map(|e| e.effective_mitigation(stats)).sum()
    }

    pub fn add_to_pouch(&mut self, item: &ItemDef) -> bool {
        if !pouch_fits(self.pouch.iter().map(|p| p.id.as_str()), self.max_pouch_slots, self.pouch_stacks, &item.id, item.stack_limit()) {
            return false;
        }
        self.pouch.push(item.clone());
        true
    }
}
