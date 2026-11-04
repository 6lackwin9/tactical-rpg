//! Combatant (build-time and derived parts) and ContentFactory.unit: a catalog unit entry -> a ready unit.

use crate::gear::{Equipment, Inventory, Slot, TIER_SOURCE};
use crate::stats::{self, roundi, CharacterStats, StatModifier};
use aegis_core::value::{self, to_int, to_str};
use rpgkit_data::{Catalog, ClassDef, Deck, UnitDef, NO_ITEM};
use indexmap::{IndexMap, IndexSet};
use serde_json::Value;

pub const BASE_AP: i32 = 10;
pub const GEAR_REACH_CAP: i32 = 1;
pub const GROUPS: [&str; 2] = ["spell", "heal"];
/// RULESET1 V4 (tasks 1.4): the widest foresight reveal range in tiles.
pub const FORESIGHT_RANGE_MAX: i32 = 8;
/// RULESET1 tasks 1.4: the largest `stagger_taken` multiplier.
pub const STAGGER_TAKEN_MULT_MAX: f64 = 2.0;
/// RULESET1 K1: the melee reach a reach weapon gives its wielder (rule `reach`; melee band 1 to this).
pub const REACH_WEAPON_REACH: i32 = 2;

/// RULESET1 P6, tasks 1.4: a passive `stagger_taken` effect: stagger from an action of at most `max_blocks` blocks
/// (0 = any action) is multiplied by `mult` (a Snap-proof trait: mult 0, max_blocks 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StaggerTaken {
    pub mult: f64,
    pub max_blocks: i32,
}

impl StaggerTaken {
    /// Whether the effect applies to stagger from an action `blocks` long.
    pub fn matches(&self, blocks: i32) -> bool {
        self.max_blocks <= 0 || blocks <= self.max_blocks
    }
}

/// HOOKS1 FM5 (rule `forced_move`; engine-only): what a unit's passive and worn-item `forced_move` effects add up to:
/// it holds firm against every forced move (`immune` always) or while guarding (`immune` guarding), and its own pushes
/// and pulls move `bonus_tiles` more tiles. Read only under the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ForcedTraits {
    pub immune_always: bool,
    pub immune_guarding: bool,
    pub bonus_tiles: i32,
}

impl ForcedTraits {
    /// Adds one `forced_move` effect row (`immune` "", always or guarding; `bonus_tiles` 0-2, summed).
    fn add(&mut self, e: &value::Obj) {
        match value::string(e, "immune", "").as_str() {
            "always" => self.immune_always = true,
            "guarding" => self.immune_guarding = true,
            _ => {}
        }
        self.bonus_tiles += value::int(e, "bonus_tiles", 0).clamp(0, 2);
    }

    /// Both sets together.
    pub fn with(self, o: ForcedTraits) -> ForcedTraits {
        ForcedTraits { immune_always: self.immune_always || o.immune_always, immune_guarding: self.immune_guarding || o.immune_guarding, bonus_tiles: self.bonus_tiles + o.bonus_tiles }
    }
}

/// HOOKS1 8.1 (rule `triggers`; engine-only): one trigger a unit holds from a passive or a worn item: where it comes
/// from (`passive:<id>` or `gear:<item id>`, the battle names it from the catalog), its key for the `limit` counts
/// (unique on the unit) and its row. Read only under the rule.
#[derive(Debug, Clone, PartialEq)]
pub struct HeldTrigger {
    pub source: String,
    pub key: String,
    pub row: rpgkit_data::TriggerRow,
}

/// HOOKS1 8.3 (rule `triggers`): a held guard set by a trigger's `guard` effect: its mitigation and the last block of
/// the round it lasts through (a break, `end_guard` and the round's end end it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeldGuard {
    pub mitigation: f64,
    pub last_block: i32,
}

/// HOOKS1 UM1 (rule `unit_meters`; engine-only): a meter a passive's `meter` effect gives: the `unit_meters` row,
/// the value it starts at (-1 = the row's `start`) and the lowest value it takes (None = the row's `min`). Read only
/// under the rule.
#[derive(Debug, Clone, PartialEq)]
pub struct HeldMeter {
    pub meter: String,
    pub start: i32,
    pub floor: Option<i32>,
}

/// HOOKS1 UM4 (rule `unit_meters`; engine-only): what a meter band's `effects` added to a unit while the meter is in
/// that band, so leaving the band takes exactly that off: the stat modifiers' source, the resists, potencies, status
/// potency and crit points it added, and the `stagger_taken` rows it pushed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BandApplied {
    pub source: String,
    pub resist: Vec<(String, i32)>,
    /// The resist keys the band created (none before it): they go again when it is left (HOOKS1-UI).
    pub resist_new: Vec<String>,
    pub potency_family: Vec<(String, f64)>,
    pub potency_group: Vec<(String, f64)>,
    pub status_potency: i32,
    pub stagger: Vec<StaggerTaken>,
    pub crit: f64,
}

/// The source of the stat modifiers `apply_effects` adds for `source` (`passive:<id>`).
fn band_stat_source(source: &str) -> String {
    format!("passive:{}", source.trim_start_matches("passive:"))
}

/// RULESET1 9 D1-D4 (rule `deck`; engine-only, no Godot original): the deck terms a unit is built and fights under:
/// the ruleset's `basics` (skill families open to every unit, no slot, never removed; every other basic is closed) and
/// its `deck_slots`. Classic never makes one, so a classic unit keeps today's loadout, basics and passives.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeckRule {
    pub basics: Vec<String>,
    pub slots: crate::ruleset::DeckSlots,
}

impl DeckRule {
    /// The deck terms of `rs`: Some only while it has the rule `deck`.
    pub fn of(rs: &crate::ruleset::Ruleset) -> Option<Self> {
        rs.has(crate::ruleset::DECK).then(|| Self { basics: rs.numbers.basics.clone(), slots: rs.numbers.deck_slots })
    }

    /// The first `n` of `ids`: the slots a deck list fills.
    fn take(ids: &[String], n: i32) -> &[String] {
        &ids[..ids.len().min(n.max(0) as usize)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stance {
    #[default]
    Neutral,
    Offensive,
    Defensive,
    Tactical,
}

impl Stance {
    pub fn from_index(i: i32) -> Stance {
        match i {
            1 => Stance::Offensive,
            2 => Stance::Defensive,
            3 => Stance::Tactical,
            _ => Stance::Neutral,
        }
    }

    /// Offensive > Tactical > Defensive > Offensive; Neutral beats nothing.
    pub fn beats(self, other: Stance) -> bool {
        matches!(
            (self, other),
            (Stance::Offensive, Stance::Tactical) | (Stance::Tactical, Stance::Defensive) | (Stance::Defensive, Stance::Offensive)
        )
    }

    pub fn matchup(self, defender: Stance) -> f64 {
        if self.beats(defender) {
            1.25
        } else if defender.beats(self) {
            0.75
        } else {
            1.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub name: String,
    pub catalog_id: String,
    pub tier: String,
    pub hp_scale: f64,
    pub power_scale: f64,
    pub max_hp: i32,
    pub hp: i32,
    pub speed: i32,
    pub attack_range: i32,
    pub max_ap: i32,
    pub ap: i32,
    pub stance: Stance,
    pub attack_boost: f64,
    pub guard: f64,
    pub power_override: i32,
    pub armor_override: i32,
    pub carry_speed_penalty: i32,
    pub stats: CharacterStats,
    pub inventory: Inventory,
    pub known_skills: Vec<String>,
    pub granted_skills: Vec<String>,
    pub active_class_families: Vec<String>,
    pub active_extra_families: Vec<String>,
    /// Class path steps (level, skill); empty = hand-built unit, no level gate.
    pub path_levels: Vec<(i32, String)>,
    pub path_unlock: IndexMap<String, i32>,
    pub unlock_bonus_family: IndexMap<String, i32>,
    pub unlock_bonus_group: IndexMap<String, i32>,
    pub status_resist: IndexMap<String, i32>,
    pub ap_regen_bonus: i32,
    pub potency_bonus: IndexMap<String, f64>,
    pub potency_group: IndexMap<String, f64>,
    pub gear_potency_family: IndexMap<String, f64>,
    pub gear_potency_group: IndexMap<String, f64>,
    pub gear_status_resist: IndexMap<String, i32>,
    pub gear_status_potency: i32,
    pub gear_spell_reach: i32,
    pub passive_status_potency: i32,
    pub passive_spell_reach: i32,
    pub foresight_source: String,
    /// RULESET1 V4: the widest `range` of the unit's foresight passives (0 = today's rule only); read only under the
    /// rule `plan_badges`.
    pub foresight_range: i32,
    /// RULESET1 P2 (rule poise): the unit row's `poise`: -1 = the ruleset's derived Max Poise, 0 = no poise bar (never
    /// breaks), above 0 = a fixed Max Poise. Units built from member state keep -1.
    pub poise: i32,
    /// RULESET1 P6: the unit's passive `stagger_taken` effects in apply order (read only under the rule poise).
    pub stagger_taken: Vec<StaggerTaken>,
    /// HOOKS1 FM5: the passives' `forced_move` effects, and the worn items' (rebuilt on every gear sync); read only under
    /// the rule forced_move (`forced_traits`).
    pub passive_forced: ForcedTraits,
    pub gear_forced: ForcedTraits,
    /// HOOKS1 8.1: the passives' `trigger` effects in apply order, and the worn items' (rebuilt on every gear sync); read
    /// only under the rule triggers (`held_triggers`).
    pub passive_triggers: Vec<HeldTrigger>,
    pub gear_triggers: Vec<HeldTrigger>,
    /// HOOKS1 8.3: the held guard a trigger set this round (None in classic and without the rule).
    pub held_guard: Option<HeldGuard>,
    /// HOOKS1 8.2: crit chance points the trigger effect `crit` added for the battle (0 in classic).
    pub trigger_crit: f64,
    /// HOOKS1 UM1: the passives' `meter` effects in apply order; read only under the rule unit_meters.
    pub passive_meters: Vec<HeldMeter>,
    /// HOOKS1 UM6: the meter skills the battle gave this unit (open without a slot, like a basic; empty outside a
    /// battle under the rule unit_meters).
    pub meter_skills: Vec<String>,
    /// Passive ids applied (Passives.apply: each once).
    pub passives: Vec<String>,
    /// Combat state (empty for a freshly built unit).
    pub st: StatusState,
    pub is_withdrawn: bool,
    /// Target lock (a unit index in the battle).
    pub active_target: Option<usize>,
    pub attack_boost_rounds: i32,
    pub stance_hold_duration: i32,
    /// Attackers this unit already countered this round (RiposteRules).
    pub riposted: Vec<usize>,
    pub ap_cost_modifier: i32,
    pub loadout_locked: bool,
    /// SKILLUI1: the rank chosen per family on the Skills tab. `None` = the rule is off (catalog units, enemies,
    /// guests, sims: every unlocked tier stays usable); `Some` = a hero built from member state, who uses only the
    /// effective rank of each slotted family (`effective_rank`).
    pub rank_choice: Option<IndexMap<String, i32>>,
    /// RULESET1 9 D5: the unit's deck (its catalog row's; a party member's saved deck replaces it when not empty). Read
    /// only under `deck_rule`.
    pub deck: Deck,
    /// RULESET1 9 D1-D4: the deck terms the unit is built and fights under (None = classic: today's loadout, basics and
    /// passives).
    pub deck_rule: Option<DeckRule>,
}

impl Unit {
    fn blank(name: &str, hp: i32, speed: i32, range: i32, stats: CharacterStats) -> Self {
        Self {
            name: name.into(),
            catalog_id: String::new(),
            tier: "hero".into(),
            hp_scale: 1.0,
            power_scale: 1.0,
            max_hp: hp,
            hp,
            speed,
            attack_range: range,
            max_ap: BASE_AP,
            ap: BASE_AP,
            stance: Stance::Neutral,
            attack_boost: 1.0,
            guard: 0.0,
            power_override: 0,
            armor_override: -1,
            carry_speed_penalty: 0,
            stats,
            inventory: Inventory::new(),
            known_skills: Vec::new(),
            granted_skills: Vec::new(),
            active_class_families: Vec::new(),
            active_extra_families: Vec::new(),
            path_levels: Vec::new(),
            path_unlock: IndexMap::new(),
            unlock_bonus_family: IndexMap::new(),
            unlock_bonus_group: IndexMap::new(),
            status_resist: IndexMap::new(),
            ap_regen_bonus: 0,
            potency_bonus: IndexMap::new(),
            potency_group: IndexMap::new(),
            gear_potency_family: IndexMap::new(),
            gear_potency_group: IndexMap::new(),
            gear_status_resist: IndexMap::new(),
            gear_status_potency: 0,
            gear_spell_reach: 0,
            passive_status_potency: 0,
            passive_spell_reach: 0,
            foresight_source: String::new(),
            foresight_range: 0,
            poise: -1,
            stagger_taken: Vec::new(),
            passive_forced: ForcedTraits::default(),
            gear_forced: ForcedTraits::default(),
            passive_triggers: Vec::new(),
            gear_triggers: Vec::new(),
            held_guard: None,
            trigger_crit: 0.0,
            passive_meters: Vec::new(),
            meter_skills: Vec::new(),
            passives: Vec::new(),
            st: StatusState::default(),
            is_withdrawn: false,
            active_target: None,
            attack_boost_rounds: 0,
            stance_hold_duration: 0,
            riposted: Vec::new(),
            ap_cost_modifier: 0,
            loadout_locked: false,
            rank_choice: None,
            deck: Deck::default(),
            deck_rule: None,
        }
    }

    /// The main weapon's type (the stat context of weapon-conditional modifiers).
    pub fn weapon_context(&self) -> Option<String> {
        self.inventory.main_weapon().map(|w| w.weapon_type().to_string())
    }

    pub fn min_attack_range(&self) -> i32 {
        if self.attack_range > 1 {
            2
        } else {
            1
        }
    }

    /// RULESET1 K1, K2: whether the unit is melee (`attack_range` at most 1, which the rule leaves as it is) with a
    /// main weapon whose family has the engine rule `reach`. A gear fact: the battle reads it only under the rule
    /// `reach` (`rpgkit_battle::skill::melee_reach`).
    pub fn has_reach_weapon(&self) -> bool {
        self.attack_range <= 1 && self.inventory.main_weapon().is_some_and(|w| w.has_reach())
    }

    /// HOOKS1 FM5: the unit's passive and worn-item `forced_move` effects together (a battle reads them only under the
    /// rule forced_move).
    pub fn forced_traits(&self) -> ForcedTraits {
        self.passive_forced.with(self.gear_forced)
    }

    /// HOOKS1 8.1: the triggers the unit holds, passives first then worn items, each in apply order (a battle reads them
    /// only under the rule triggers).
    pub fn held_triggers(&self) -> impl Iterator<Item = &HeldTrigger> {
        self.passive_triggers.iter().chain(&self.gear_triggers)
    }

    pub fn has_status(&self, s: &str) -> bool {
        self.st.active.contains(s)
    }

    pub fn active_magnitude(&self, s: &str) -> i32 {
        if self.has_status(s) { self.st.magnitudes.get(s).copied().unwrap_or(0) } else { 0 }
    }

    pub fn move_per_block(&self) -> i32 {
        let base = ((self.speed + 2) as f64 / 5.0).floor().clamp(1.0, 4.0) as i32;
        let hasted = self.has_status("haste") as i32;
        let slowed = self.has_status("slow") as i32;
        (base + hasted - slowed).max(1)
    }

    pub fn worn_burden(&self) -> i32 {
        self.inventory.gear().map(|e| (e.item.burden - self.stats.proficiency.burden_reduction(e.weapon_type())).max(0)).sum()
    }

    pub fn debuff_resistance(&self) -> f64 {
        self.stats.debuff_resistance(self.weapon_context().as_deref())
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Gear stat bonuses -> tier stat bonuses (read without tier modifiers) -> gear effects -> derived numbers.
    pub fn sync_from_stats_and_gear(&mut self, cat: &Catalog) {
        self.stats.remove_modifier("gear");
        let bonuses: Vec<(String, i32)> =
            self.inventory.gear().flat_map(|e| e.item.stat_bonuses.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>()).collect();
        for (k, v) in bonuses {
            if v != 0 {
                self.stats.modifiers.push(StatModifier::permanent(&k, v, "gear", "always"));
            }
        }
        self.rebuild_gear_tier_stats();
        self.rebuild_gear_effects(cat);
        let burden = self.worn_burden();
        let (base_rng, ctx) = match self.inventory.main_weapon() {
            Some(w) => (w.item.base_range, Some(w.weapon_type().to_string())),
            None => (1, None),
        };
        let ctx = ctx.as_deref();
        self.max_hp = roundi(self.stats.max_hp(ctx) as f64 * self.hp_scale).max(1);
        self.hp = self.hp.min(self.max_hp);
        self.speed = (self.stats.initiative_speed(burden, ctx) - self.carry_speed_penalty).max(1);
        self.attack_range = self.stats.effective_range(base_rng, ctx);
        self.refresh_path_skills(cat);
    }

    fn rebuild_gear_tier_stats(&mut self) {
        self.stats.remove_modifier(TIER_SOURCE);
        let mut totals: IndexMap<String, i32> = IndexMap::new();
        for eq in self.inventory.gear() {
            for (k, v) in eq.tier_stat_bonuses(&self.stats) {
                *totals.entry(k).or_insert(0) += v;
            }
        }
        for (k, v) in totals {
            if v != 0 {
                self.stats.modifiers.push(StatModifier::permanent(&k, v, TIER_SOURCE, "always"));
            }
        }
    }

    fn rebuild_gear_effects(&mut self, cat: &Catalog) {
        self.gear_potency_family.clear();
        self.gear_potency_group.clear();
        self.gear_status_resist.clear();
        self.gear_status_potency = 0;
        self.gear_spell_reach = 0;
        self.gear_forced = ForcedTraits::default();
        self.gear_triggers.clear();
        let item_fx: Vec<(String, Vec<Value>)> = self.inventory.gear().map(|e| (format!("gear:{}", e.id()), e.item.effects.clone())).collect();
        for (src, fx) in item_fx {
            self.apply_effects(cat, &fx, &src, true);
        }
        let tier_fx: Vec<(String, Vec<Value>)> = self
            .inventory
            .gear()
            .flat_map(|e| e.active_tier_effects(&self.stats).into_iter().map(|(n, fx)| (format!("gear:{}:tier{}", e.id(), n), fx)).collect::<Vec<_>>())
            .collect();
        for (src, fx) in tier_fx {
            self.apply_effects(cat, &fx, &src, true);
        }
    }

    /// Passives.apply: once per id, then the path refresh (a KNOW bonus can open an advanced family earlier). RULESET1 9
    /// D3: under the deck rule a slottable passive applies only while slotted (`slotted`).
    pub fn apply_passive(&mut self, cat: &Catalog, passive: &rpgkit_data::PassiveDef) {
        if passive.id.is_empty() || self.passives.contains(&passive.id) || !self.slotted(passive) {
            return;
        }
        self.passives.push(passive.id.clone());
        self.apply_effects(cat, &passive.effects, &format!("passive:{}", passive.id), false);
        self.refresh_path_skills(cat);
    }

    /// Passives.apply_effects (item effects and passives share the grammar).
    pub fn apply_effects(&mut self, cat: &Catalog, effects: &[Value], source: &str, gear: bool) {
        let id = source.trim_start_matches("passive:").to_string();
        for raw in effects {
            let Some(e) = raw.as_object() else { continue };
            match value::string(e, "type", "").as_str() {
                "stat" => {
                    let mut tag = "always".to_string();
                    if let Some(ctx) = value::obj(e, "context") {
                        for (k, v) in ctx {
                            tag = format!("{}:{}", k, to_str(v));
                        }
                    }
                    let stat = value::string(e, "stat", "");
                    let mut m = StatModifier::permanent(&stat, value::int(e, "value", 0), &format!("passive:{id}"), &tag);
                    // DERIVE1 4.1: the optional percent counts only on a derived id (base stats stay flat-only)
                    if cat.derived_stats.contains_key(&stat) {
                        m.percent = value::float(e, "percent", 0.0);
                    }
                    self.stats.modifiers.push(m);
                }
                "proficiency" => self.stats.proficiency.add_effect(e),
                "resist" => {
                    let status = value::string(e, "status", "");
                    let target = if gear { &mut self.gear_status_resist } else { &mut self.status_resist };
                    let cur = target.get(&status).copied().unwrap_or(0);
                    target.insert(status, (cur + value::int(e, "value", 0)).min(100));
                }
                "ap_regen" => self.ap_regen_bonus += value::int(e, "value", 0),
                "skill" => self.learn_skill(cat, &value::string(e, "skill", "")),
                "skill_potency" => {
                    let v = value::float(e, "value", 0.0);
                    let group = value::string(e, "group", "");
                    if !group.is_empty() {
                        let g = if gear { &mut self.gear_potency_group } else { &mut self.potency_group };
                        *g.entry(group).or_insert(0.0) += v;
                    } else {
                        let fam = value::string(e, "family", "");
                        let f = if gear { &mut self.gear_potency_family } else { &mut self.potency_bonus };
                        *f.entry(fam).or_insert(0.0) += v;
                    }
                }
                "status_potency" => {
                    if gear {
                        self.gear_status_potency += value::int(e, "value", 0)
                    } else {
                        self.passive_status_potency += value::int(e, "value", 0)
                    }
                }
                "spell_reach" => {
                    if gear {
                        self.gear_spell_reach += value::int(e, "value", 0)
                    } else {
                        self.passive_spell_reach += value::int(e, "value", 0)
                    }
                }
                "unlock" if !gear => {
                    let levels = value::int(e, "levels", 0).clamp(0, 2);
                    let group = value::string(e, "group", "");
                    let (map, key) = if group.is_empty() {
                        (&mut self.unlock_bonus_family, value::string(e, "family", ""))
                    } else {
                        (&mut self.unlock_bonus_group, group)
                    };
                    let cur = map.get(&key).copied().unwrap_or(0);
                    map.insert(key, cur.max(levels));
                    self.refresh_path_skills(cat);
                }
                "foresight" if !gear => {
                    if self.foresight_source.is_empty() {
                        self.foresight_source = id.clone();
                    }
                    self.foresight_range = self.foresight_range.max(value::int(e, "range", 0).clamp(0, FORESIGHT_RANGE_MAX));
                }
                // RULESET1 P6: passives only (gear effects are rebuilt on every sync and would pile up)
                "stagger_taken" if !gear => {
                    let mult = value::float(e, "mult", 1.0).clamp(0.0, STAGGER_TAKEN_MULT_MAX);
                    self.stagger_taken.push(StaggerTaken { mult, max_blocks: value::int(e, "max_blocks", 0).max(0) });
                }
                // HOOKS1 FM5: passives and worn items (the gear set is cleared on every sync)
                "forced_move" => {
                    if gear {
                        self.gear_forced.add(e);
                    } else {
                        self.passive_forced.add(e);
                    }
                }
                // HOOKS1 8.1: passives and worn items (the gear list is cleared on every sync); a tier effect counts as
                // its item's
                "trigger" => {
                    let from = if gear { format!("gear:{}", source.trim_start_matches("gear:").split(':').next().unwrap_or("")) } else { source.to_string() };
                    let list = if gear { &mut self.gear_triggers } else { &mut self.passive_triggers };
                    let key = format!("{source}#{}", list.len());
                    list.push(HeldTrigger { source: from, key, row: rpgkit_data::TriggerRow::from_obj(e) });
                }
                // HOOKS1 UM1: passives only (HOOKS1-S gives items no meter effect)
                "meter" if !gear => {
                    let floor = e.get("floor").filter(|v| v.is_number()).map(|v| to_int(v, 0));
                    self.passive_meters.push(HeldMeter { meter: value::string(e, "meter", ""), start: value::int(e, "start", -1), floor });
                }
                _ => {}
            }
        }
    }

    /// HOOKS1 UM4: applies a meter band's `effects` (the passive grammar: stat, resist, stagger_taken, skill_potency,
    /// status_potency; `crit` as the trigger effect) under `source` and returns what they added (`remove_band` takes it
    /// off again).
    pub fn apply_band(&mut self, cat: &Catalog, effects: &[Value], source: &str) -> BandApplied {
        let resist = self.status_resist.clone();
        let (fam, group) = (self.potency_bonus.clone(), self.potency_group.clone());
        let (potency, stagger) = (self.passive_status_potency, self.stagger_taken.len());
        let mut crit = 0.0;
        let kinds = ["stat", "resist", "stagger_taken", "skill_potency", "status_potency"];
        let passive: Vec<Value> = effects.iter().filter(|e| e.as_object().is_some_and(|o| kinds.contains(&value::string(o, "type", "").as_str()))).cloned().collect();
        self.apply_effects(cat, &passive, source, false);
        for e in effects.iter().filter_map(Value::as_object).filter(|o| value::string(o, "type", "") == "crit") {
            crit += value::float(e, "value", 0.0);
        }
        self.trigger_crit += crit;
        let diff_i = |now: &IndexMap<String, i32>, was: &IndexMap<String, i32>| -> Vec<(String, i32)> {
            now.iter().filter_map(|(k, v)| Some((k.clone(), v - was.get(k).copied().unwrap_or(0))).filter(|(_, d)| *d != 0)).collect()
        };
        let diff_f = |now: &IndexMap<String, f64>, was: &IndexMap<String, f64>| -> Vec<(String, f64)> {
            now.iter().filter_map(|(k, v)| Some((k.clone(), v - was.get(k).copied().unwrap_or(0.0))).filter(|(_, d)| *d != 0.0)).collect()
        };
        BandApplied {
            source: band_stat_source(source),
            resist: diff_i(&self.status_resist, &resist),
            resist_new: self.status_resist.keys().filter(|k| !resist.contains_key(*k)).cloned().collect(),
            potency_family: diff_f(&self.potency_bonus, &fam),
            potency_group: diff_f(&self.potency_group, &group),
            status_potency: self.passive_status_potency - potency,
            stagger: self.stagger_taken[stagger..].to_vec(),
            crit,
        }
    }

    /// HOOKS1 UM4: takes off what `apply_band` added (a key the band created goes when it returns to nothing).
    pub fn remove_band(&mut self, a: &BandApplied) {
        self.stats.remove_modifier(&a.source);
        for (k, d) in &a.resist {
            let v = self.status_resist.get(k).copied().unwrap_or(0) - d;
            if v == 0 && a.resist_new.contains(k) {
                // HOOKS1-UI (the HOOKS1-M review): a key the band created goes with it (no stale 0 entry)
                self.status_resist.shift_remove(k);
            } else {
                self.status_resist.insert(k.clone(), v);
            }
        }
        for (map, list) in [(&mut self.potency_bonus, &a.potency_family), (&mut self.potency_group, &a.potency_group)] {
            for (k, d) in list {
                let v = map.get(k).copied().unwrap_or(0.0) - d;
                if v.abs() < 1e-9 {
                    map.shift_remove(k);
                } else {
                    map.insert(k.clone(), v);
                }
            }
        }
        self.passive_status_potency -= a.status_potency;
        for st in &a.stagger {
            if let Some(i) = self.stagger_taken.iter().rposition(|x| x == st) {
                self.stagger_taken.remove(i);
            }
        }
        self.trigger_crit -= a.crit;
    }

    /// HOOKS1 UM6: whether the battle gave this unit `skill_id` as a meter skill.
    pub fn meter_skill(&self, skill_id: &str) -> bool {
        self.meter_skills.iter().any(|s| s == skill_id)
    }

    // ---- Skills: ownership, unlocks (SK2), loadout ---------------------------------------------------------------

    pub fn active_families(&self) -> Vec<String> {
        self.active_class_families.iter().chain(&self.active_extra_families).cloned().collect()
    }

    /// RULESET1 9 D1 (under the deck rule only, else None): Some(true) for a skill of the ruleset's `basics` (open to
    /// every unit, no slot), Some(false) for any other basic row (closed: `guard`, `switch_target`), None for the rest
    /// (today's rules).
    pub fn deck_basic(&self, cat: &Catalog, skill_id: &str) -> Option<bool> {
        let rule = self.deck_rule.as_ref()?;
        let s = cat.skill(skill_id)?;
        if rule.basics.iter().any(|b| b == s.family_id()) {
            Some(true)
        } else if s.basic {
            Some(false)
        } else {
            None
        }
    }

    /// Catalog.is_always_active, with the deck rule's open and closed basics (D1).
    /// HOOKS1 UM6: a meter skill the battle gave is open without a slot.
    pub fn always_active(&self, cat: &Catalog, skill_id: &str) -> bool {
        if self.meter_skill(skill_id) {
            return true;
        }
        self.deck_basic(cat, skill_id).unwrap_or_else(|| cat.is_always_active(skill_id))
    }

    /// RULESET1 9 D3: whether `passive` applies: always without the deck rule and for a source-granted passive (`slot`
    /// ""); under the rule a `passive` or `keystone` one only while the deck names it within that slot kind's count.
    pub fn slotted(&self, passive: &rpgkit_data::PassiveDef) -> bool {
        let Some(rule) = &self.deck_rule else { return true };
        let list = match passive.slot.as_str() {
            "" => return true,
            "passive" => DeckRule::take(&self.deck.passive, rule.slots.passive),
            "keystone" => DeckRule::take(&self.deck.keystone, rule.slots.keystone),
            _ => return false,
        };
        list.contains(&passive.id)
    }

    /// RULESET1 9 D4: a catalog unit owns the slottable passives its deck names; applies those that are slotted (D3)
    /// and returns how many it applied. Nothing without the deck rule.
    pub fn apply_deck_passives(&mut self, cat: &Catalog) -> usize {
        if self.deck_rule.is_none() {
            return 0;
        }
        let named: Vec<String> = self.deck.passive.iter().chain(&self.deck.keystone).cloned().collect();
        let mut applied = 0;
        for id in named {
            let Some(p) = cat.passives.get(&id) else { continue };
            if !p.slot.is_empty() && !self.passives.contains(&p.id) && self.slotted(p) {
                self.apply_passive(cat, p);
                applied += 1;
            }
        }
        applied
    }

    /// RULESET1 9 D1, D2: the active slots under the deck rule: the deck's `active` families the unit owns (no basic, no
    /// repeat), else (auto) its loadout's families (today's default: the class path's first, then the extras), the
    /// first `deck_slots.active` of them (a row without an active count, which the content check refuses, limits
    /// nothing); the class and extra split does not apply. A unit with neither (a hand-built unit that owns every
    /// skill) keeps every family. Nothing without the deck rule.
    pub fn apply_deck(&mut self, cat: &Catalog) {
        let Some(rule) = self.deck_rule.clone() else { return };
        let wanted = if self.deck.active.is_empty() { self.active_families() } else { self.deck.active.clone() };
        let mut actives: Vec<String> = Vec::new();
        for fam in wanted {
            let basic = cat.is_basic_family(&fam) || rule.basics.contains(&fam);
            let owned = cat.family_rows(&fam).iter().any(|s| self.owns_skill(cat, &s.id));
            if !basic && owned && !actives.contains(&fam) {
                actives.push(fam);
            }
        }
        if actives.is_empty() {
            return;
        }
        let n = if rule.slots.active > 0 { rule.slots.active } else { actives.len() as i32 };
        self.active_class_families = DeckRule::take(&actives, n).to_vec();
        self.active_extra_families = Vec::new();
    }

    /// HOOKS1 UM6: a meter skill the battle gave is owned (before the deck rule's closed basics).
    pub fn owns_skill(&self, cat: &Catalog, skill_id: &str) -> bool {
        if self.meter_skill(skill_id) {
            return true;
        }
        if let Some(open) = self.deck_basic(cat, skill_id) {
            return open;
        }
        if self.known_skills.iter().any(|s| s == skill_id) || self.granted_skills.iter().any(|s| s == skill_id) || cat.is_system(skill_id) {
            return true;
        }
        if !self.path_levels.is_empty() {
            let fam = cat.family_of(skill_id);
            if self.known_skills.iter().any(|id| cat.family_of(id) == fam) {
                return true;
            }
        }
        self.known_skills.is_empty() && !cat.is_learned_only(skill_id)
    }

    pub fn knows_skill(&self, cat: &Catalog, skill_id: &str) -> bool {
        if !self.owns_skill(cat, skill_id) {
            return false;
        }
        let active = self.active_families();
        active.is_empty() || self.always_active(cat, skill_id) || active.iter().any(|f| f == cat.family_of(skill_id))
    }

    pub fn owned_families(&self, cat: &Catalog) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for id in self.known_skills.iter().chain(&self.granted_skills) {
            let fam = cat.family_of(id).to_string();
            if !out.contains(&fam) && !cat.is_always_active(id) {
                out.push(fam);
            }
        }
        out
    }

    /// Sets the active loadout; on any error nothing changes (returns the error).
    pub fn set_loadout(&mut self, cat: &Catalog, class_families: Vec<String>, extra_families: Vec<String>) -> Result<(), String> {
        if class_families.len() > stats::MAX_CLASS_SLOTS || extra_families.len() > stats::MAX_EXTRA_SLOTS {
            return Err("too many families".into());
        }
        let owned = self.owned_families(cat);
        let mut seen: Vec<&String> = Vec::new();
        for fam in class_families.iter().chain(&extra_families) {
            if seen.contains(&fam) {
                return Err(format!("family '{fam}' listed twice"));
            }
            seen.push(fam);
            if cat.is_basic_family(fam) {
                return Err(format!("'{fam}' is a basic family"));
            }
            if !owned.contains(fam) {
                return Err(format!("does not own family '{fam}'"));
            }
        }
        self.active_class_families = class_families;
        self.active_extra_families = extra_families;
        Ok(())
    }

    pub fn set_class_path(&mut self, cat: &Catalog, steps: Vec<(i32, String)>) {
        self.path_unlock.clear();
        for (lvl, skill) in &steps {
            let fam = cat.family_of(skill).to_string();
            if self.path_unlock.get(&fam).is_none_or(|cur| lvl < cur) {
                self.path_unlock.insert(fam, *lvl);
            }
        }
        self.path_levels = steps;
        self.refresh_path_skills(cat);
    }

    /// KNOW for unlocks: base + permanent flat modifiers that apply in every context.
    pub fn unlock_know(&self) -> i32 {
        self.stats.base("KNOW")
            + self.stats.modifiers.iter().filter(|m| m.stat == "KNOW" && m.duration < 0 && m.matches(None)).map(|m| m.flat).sum::<i32>()
    }

    pub fn unlock_reduction(&self, cat: &Catalog, family: &str) -> i32 {
        let Some(t1) = cat.tier_one(family) else { return 0 };
        if !self.path_unlock.contains_key(family) {
            return 0;
        }
        let mut passive_r = self.unlock_bonus_family.get(family).copied().unwrap_or(0);
        for g in GROUPS {
            if skill_in_group(t1, g) {
                passive_r = passive_r.max(self.unlock_bonus_group.get(g).copied().unwrap_or(0));
            }
        }
        let know_r = if t1.grade != "novice" { stats::know_unlock_reduction(self.unlock_know()) } else { 0 };
        stats::MAX_UNLOCK_REDUCTION.min(passive_r.max(know_r))
    }

    pub fn family_unlock_level(&self, cat: &Catalog, family: &str) -> i32 {
        let t1 = cat.tier_one(family);
        match self.path_unlock.get(family) {
            None => stats::default_unlock(t1.map(|s| s.level).unwrap_or(1)),
            Some(&base) => {
                let floor = if t1.is_some_and(|s| s.grade != "novice") { stats::ADVANCED_UNLOCK_FLOOR } else { 1 };
                base.min(floor.max(base - self.unlock_reduction(cat, family)))
            }
        }
    }

    pub fn refresh_path_skills(&mut self, cat: &Catalog) {
        if self.path_levels.is_empty() {
            return;
        }
        let mut owned: Vec<String> = Vec::new();
        for (_, id) in &self.path_levels {
            if self.stats.level >= self.family_unlock_level(cat, cat.family_of(id)) && !owned.contains(id) {
                owned.push(id.clone());
            }
        }
        for id in &self.known_skills {
            if !owned.contains(id) && !self.path_levels.iter().any(|(_, s)| s == id) {
                owned.push(id.clone());
            }
        }
        self.known_skills = owned;
    }

    pub fn learn_skill(&mut self, cat: &Catalog, skill_id: &str) {
        self.learn_skill_checked(cat, skill_id, true);
    }

    /// Combatant.learn_skill(id, check_requires): a saved hero keeps what it learned (no requires check).
    pub fn learn_skill_checked(&mut self, cat: &Catalog, skill_id: &str, check_requires: bool) {
        let fam = cat.family_of(skill_id).to_string();
        if let Some(head) = cat.tier_one(&fam).filter(|_| check_requires) {
            if head.requires.iter().any(|r| !self.owns_skill(cat, r)) {
                return;
            }
        }
        if !self.granted_skills.iter().any(|s| s == skill_id) {
            self.granted_skills.push(skill_id.to_string());
            let active = self.active_families();
            // RULESET1 9 D2: under the deck rule only the deck fills the active slots
            if self.deck_rule.is_none() && !active.is_empty() && !cat.is_always_active(skill_id) && !active.contains(&fam) && self.active_extra_families.len() < stats::MAX_EXTRA_SLOTS {
                self.active_extra_families.push(fam);
            }
        }
    }
}

/// A skill rank as the game writes it: I to V (SKILLUI1 rank texts and refusals).
pub fn rank_numeral(rank: i32) -> &'static str {
    ["-", "I", "II", "III", "IV", "V"].get(rank.max(0) as usize).copied().unwrap_or("V+")
}

/// SkillDef.in_group for the passive groups: "spell" = an attack that needs a staff or wand; "heal" = a support heal.
pub fn skill_in_group(s: &rpgkit_data::SkillDef, group: &str) -> bool {
    match group {
        "spell" => s.kind == "ATTACK" && s.weapon_types.iter().any(|w| w == "staff" || w == "wand"),
        "heal" => s.kind == "SUPPORT" && (s.heal_base > 0 || s.heal_mult > 0.0),
        _ => false,
    }
}

fn class_for<'a>(cat: &'a Catalog, e: &UnitDef) -> Option<&'a ClassDef> {
    if e.class_id.is_empty() {
        None
    } else {
        cat.classes.get(&e.class_id)
    }
}

/// ContentFactory.unit: a fresh unit for catalog entry `e`. `default_ap` 0 = the Combatant default.
pub fn build_unit(cat: &Catalog, e: &UnitDef, default_ap: i32) -> Unit {
    build_unit_in(cat, e, default_ap, None)
}

/// `build_unit` under the deck rule (RULESET1 9 D1-D4) when `deck` is Some: the deck's slotted passives apply (D4,
/// then the vitals of a derived unit are re-derived), the open basics replace the catalog's, the deck fills the active
/// slots. None = `build_unit` exactly (classic).
pub fn build_unit_in(cat: &Catalog, e: &UnitDef, default_ap: i32, deck: Option<&DeckRule>) -> Unit {
    let mut u = build_catalog_unit(cat, e, default_ap);
    u.deck = e.deck.clone();
    if let Some(rule) = deck {
        u.deck_rule = Some(rule.clone());
        if u.apply_deck_passives(cat) > 0 && e.stat_mode == "derived" && class_for(cat, e).is_some() {
            u.sync_from_stats_and_gear(cat);
            u.hp = u.max_hp;
        }
        u.apply_deck(cat);
    }
    u
}

/// HOOKS1 DU1: the active slots catalog unit `e` fills with an automatic deck (its authored deck ignored) under `rule`:
/// the battle's own build (`build_unit_in`, `loadout_of`, `apply_deck`), so the content check reads the run time's list.
pub fn auto_actives(cat: &Catalog, e: &UnitDef, rule: &DeckRule) -> Vec<String> {
    let mut auto = e.clone();
    auto.deck = Deck::default();
    build_unit_in(cat, &auto, 1, Some(rule)).active_families()
}

fn build_catalog_unit(cat: &Catalog, e: &UnitDef, default_ap: i32) -> Unit {
    let arch = class_for(cat, e);
    let derived = e.stat_mode == "derived" && arch.is_some();
    let warrior = || cat.classes.get("warrior").cloned().unwrap_or_else(|| ClassDef::from_obj(&Default::default()));
    let mut u = if derived {
        let arch = arch.unwrap();
        let spend = if e.spend.is_empty() { stats::auto_spend(&arch.growth, stats::spend_pool(e.level)) } else { e.spend.clone() };
        Unit::blank(&e.name, 1, 1, 1, CharacterStats::new(arch, e.level, &spend))
    } else if let Some(arch) = arch {
        Unit::blank(&e.name, e.hp, e.speed, e.attack_range, CharacterStats::new(arch, e.level, &IndexMap::new()))
    } else {
        Unit::blank(&e.name, e.hp, e.speed, e.attack_range, CharacterStats::new(&warrior(), 1, &IndexMap::new()))
    };
    equip(cat, &mut u, e, arch);
    if derived {
        let arch = arch.unwrap();
        u.tier = e.tier.clone();
        let (hp, power, _) = stats::tier_scales(&u.tier);
        u.hp_scale = hp;
        u.power_scale = power;
        u.sync_from_stats_and_gear(cat);
        u.hp = u.max_hp;
        let mut extras: Vec<String> = Vec::new();
        for x in &e.extra_skills {
            if !extras.contains(x) {
                extras.push(x.clone());
            }
        }
        u.known_skills = extras;
        u.set_class_path(cat, arch.path_steps());
        for b in cat.basic_ids() {
            if !u.known_skills.contains(&b) {
                u.known_skills.push(b);
            }
        }
        let (class_fams, extra_fams) = loadout_of(cat, e, arch, &u.known_skills);
        let _ = u.set_loadout(cat, class_fams, extra_fams);
    }
    let ap = if e.ap > 0 { e.ap } else { default_ap };
    if ap > 0 {
        u.max_ap = ap;
        u.ap = ap;
    }
    u.power_override = e.power;
    u.armor_override = e.armor;
    u.poise = e.poise;
    for (k, v) in &e.status_resist {
        u.status_resist.insert(k.clone(), *v);
    }
    u.catalog_id = e.id.clone();
    u
}

pub fn families_of(cat: &Catalog, ids: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for id in ids {
        let fam = cat.family_of(id).to_string();
        if !cat.is_always_active(id) && !out.contains(&fam) {
            out.push(fam);
        }
    }
    out
}

/// ContentFactory.loadout_of: (class families, extra families).
pub fn loadout_of(cat: &Catalog, e: &UnitDef, arch: &ClassDef, owned_ids: &[String]) -> (Vec<String>, Vec<String>) {
    let path_all: Vec<String> = arch.path_steps().into_iter().map(|(_, s)| s).collect();
    let path_families = families_of(cat, &path_all);
    let mut class_out = Vec::new();
    let mut extra_out = Vec::new();
    if !e.loadout.is_empty() {
        for fam in &e.loadout {
            if path_families.contains(fam) {
                class_out.push(fam.clone());
            } else {
                extra_out.push(fam.clone());
            }
        }
        return (class_out, extra_out);
    }
    let owned_path: Vec<String> = if owned_ids.is_empty() {
        arch.path_steps().into_iter().filter(|(l, _)| *l <= e.level).map(|(_, s)| s).collect()
    } else {
        path_all.iter().filter(|id| owned_ids.contains(id)).cloned().collect()
    };
    for fam in families_of(cat, &owned_path) {
        if class_out.len() < stats::MAX_CLASS_SLOTS {
            class_out.push(fam);
        }
    }
    for fam in families_of(cat, &e.extra_skills) {
        if !class_out.contains(&fam) && !path_families.contains(&fam) && extra_out.len() < stats::MAX_EXTRA_SLOTS {
            extra_out.push(fam);
        }
    }
    (class_out, extra_out)
}

/// The class's default gear with the unit's per-slot overrides, then the pouch.
fn equip(cat: &Catalog, u: &mut Unit, e: &UnitDef, arch: Option<&ClassDef>) {
    for (slot_key, item_id) in &e.slots {
        if item_id == NO_ITEM {
            continue;
        }
        let id = if item_id.is_empty() { arch.and_then(|a| a.loadout.get(slot_key).cloned()).unwrap_or_default() } else { item_id.clone() };
        let Some(item) = cat.items.get(&id) else { continue };
        if item.is_equipment() {
            u.inventory.equip(Equipment::new(item), Slot::parse(slot_key).unwrap_or(Slot::MainHand));
        }
    }
    if e.pouch.iter().any(|p| p == NO_ITEM) {
        return;
    }
    let pouch = if e.pouch.is_empty() { arch.map(|a| a.pouch.clone()).unwrap_or_default() } else { e.pouch.clone() };
    for id in pouch {
        if let Some(item) = cat.items.get(&id).filter(|i| i.item_type == "CONSUMABLE") {
            u.inventory.add_to_pouch(item);
        }
    }
}

/// An int from a JSON value (helper for tests and parity tools).
pub fn json_int(v: &Value) -> i32 {
    to_int(v, 0)
}

/// Statuses of one unit: active this round, and queued for next round. Every map keeps insertion order like a
/// Godot Dictionary (removal keeps the order of the rest), because round roll-over and planning iterate them.
#[derive(Debug, Clone, Default)]
pub struct StatusState {
    pub active: IndexSet<String>,
    pub rounds_left: IndexMap<String, i32>,
    pub magnitudes: IndexMap<String, i32>,
    pub sources: IndexMap<String, String>,
    pub pending: IndexSet<String>,
    pub pending_rounds: IndexMap<String, i32>,
    pub pending_magnitudes: IndexMap<String, i32>,
    pub pending_sources: IndexMap<String, String>,
    /// RULESET1_INTENT IN6 (timeline, rule round_intent): status -> the round intent an intent-bound instance (active or
    /// queued) was applied under; it ends when the unit's intent changes. Classic never fills it.
    pub intents: IndexMap<String, String>,
    /// EMPOWER1: status -> the magnitude a meter band holds. Filled only while a band holds the status.
    pub band_held: IndexMap<String, i32>,
    /// EMPOWER1: the skill or trigger grant beneath a band's merged instance (the higher magnitude applies), put back
    /// when the band lets go. Filled only while a band holds the status.
    pub band_under: IndexMap<String, GrantUnder>,
}

/// EMPOWER1: a grant's own part of an active instance a band also holds.
#[derive(Debug, Clone, Default)]
pub struct GrantUnder {
    pub rounds: i32,
    pub magnitude: i32,
    pub source: String,
}

impl StatusState {
    pub fn has_any(&self, s: &str) -> bool {
        self.active.contains(s) || self.pending.contains(s)
    }
}

impl Unit {
    pub fn is_active(&self) -> bool {
        self.is_alive() && !self.is_withdrawn
    }

    /// Combatant.take_damage: `mitigated` false applies the unit's guard first. Returns the damage dealt.
    pub fn take_damage(&mut self, amount: i32, mitigated: bool) -> i32 {
        let mut dmg = amount;
        if !mitigated && self.guard > 0.0 {
            dmg = roundi(amount as f64 * (1.0 - self.guard)).max(1);
        }
        self.hp = (self.hp - dmg).max(0);
        dmg
    }

    pub fn heal(&mut self, amount: i32) {
        if self.is_alive() {
            self.hp = self.max_hp.min(self.hp + amount);
        }
    }

    pub fn withdraw(&mut self) {
        self.is_withdrawn = true;
    }

    pub fn set_stance(&mut self, s: Stance) {
        if self.stance != s {
            self.stance = s;
            self.stance_hold_duration = 0;
            self.remove_status("riposte");
        }
    }

    /// Removes `status` entirely (active and pending).
    pub fn remove_status(&mut self, status: &str) {
        let st = &mut self.st;
        st.active.shift_remove(status);
        st.rounds_left.shift_remove(status);
        st.pending.shift_remove(status);
        st.pending_rounds.shift_remove(status);
        st.magnitudes.shift_remove(status);
        st.pending_magnitudes.shift_remove(status);
        st.sources.shift_remove(status);
        st.pending_sources.shift_remove(status);
        st.intents.shift_remove(status);
        st.band_under.shift_remove(status);
    }

    fn capped_magnitude(&self, status: &str) -> i32 {
        let cap = crate::status::cap_of(status);
        self.st.magnitudes.get(status).copied().unwrap_or(0).clamp(0, if cap > 0 { cap } else { 100 })
    }

    pub fn weaken_factor(&self) -> f64 {
        if !self.has_status("weaken") {
            return 1.0;
        }
        1.0 - self.capped_magnitude("weaken") as f64 / 100.0
    }

    pub fn empower_factor(&self) -> f64 {
        if !self.has_status("empower") {
            return 1.0;
        }
        1.0 + self.capped_magnitude("empower") as f64 / 100.0
    }

    pub fn potency_for(&self, family: &str) -> f64 {
        1.0 + self.potency_bonus.get(family).copied().unwrap_or(0.0) + self.gear_potency_family.get(family).copied().unwrap_or(0.0)
    }

    pub fn potency_for_skill(&self, skill: &rpgkit_data::SkillDef) -> f64 {
        let mut total = self.potency_for(skill.family_id());
        for g in GROUPS {
            if skill_in_group(skill, g) {
                total += self.potency_group.get(g).copied().unwrap_or(0.0) + self.gear_potency_group.get(g).copied().unwrap_or(0.0);
            }
        }
        total
    }

    /// SkillDef.power_for: the skill multiplier x potency x weaken x empower.
    pub fn power_for(&self, skill: &rpgkit_data::SkillDef) -> f64 {
        skill.power_mult * self.potency_for_skill(skill) * self.weaken_factor() * self.empower_factor()
    }

    pub fn status_potency_bonus(&self) -> i32 {
        self.gear_status_potency + self.passive_status_potency
    }

    pub fn spell_reach_bonus(&self) -> i32 {
        self.gear_spell_reach.min(GEAR_REACH_CAP) + self.passive_spell_reach
    }

    pub fn resist_for(&self, status: &str) -> i32 {
        (self.status_resist.get(status).copied().unwrap_or(0) + self.gear_status_resist.get(status).copied().unwrap_or(0)).min(100)
    }

    /// The weapon type that counts for skill gating in `hand` ("main", "off"): "unarmed" when that hand is empty.
    pub fn wielded_type(&self, hand: &str) -> String {
        let main = self.inventory.main_weapon();
        let weapon = if hand == "off" {
            if main.is_some_and(|m| m.item.two_handed) {
                None
            } else {
                self.inventory.offhand_weapon()
            }
        } else {
            main
        };
        weapon.map(|w| w.weapon_type().to_string()).unwrap_or_else(|| "unarmed".into())
    }

    pub fn skill_unlock_level(&self, cat: &Catalog, skill: &rpgkit_data::SkillDef) -> i32 {
        stats::MAX_LEVEL.min(self.family_unlock_level(cat, skill.family_id()) + skill.tier_offset())
    }

    /// The level gate of can_use: basics (under the deck rule the ruleset's basics, D1), engine skills, meter skills
    /// (HOOKS1 UM6) and hand-built units are never gated.
    pub fn tier_unlocked(&self, cat: &Catalog, skill: &rpgkit_data::SkillDef) -> bool {
        if self.path_levels.is_empty() || skill.system || skill.basic || self.meter_skill(&skill.id) || self.deck_basic(cat, &skill.id) == Some(true) {
            return true;
        }
        self.stats.level >= self.skill_unlock_level(cat, skill)
    }

    /// SKILLUI1 R1: the tiers of `family` this unit owns and has unlocked, ascending (a class hero owns whole
    /// ladders, so 1..top; a learned row gives only its own tier).
    pub fn selectable_ranks(&self, cat: &Catalog, family: &str) -> Vec<i32> {
        let mut out: Vec<i32> = cat.family_rows(family).into_iter().filter(|s| self.owns_skill(cat, &s.id) && self.tier_unlocked(cat, s)).map(|s| s.tier).collect();
        out.dedup();
        out
    }

    /// SKILLUI1 R2: the rank the unit uses: no entry, the highest selectable rank; an entry `c`, the highest
    /// selectable rank <= `c`, else the lowest; `None` when nothing is selectable (the family is locked).
    pub fn effective_rank(&self, cat: &Catalog, family: &str) -> Option<i32> {
        let ranks = self.selectable_ranks(cat, family);
        let chosen = self.rank_choice.as_ref().and_then(|m| m.get(family).copied());
        match chosen {
            None => ranks.last().copied(),
            Some(c) => ranks.iter().rev().find(|&&r| r <= c).or(ranks.first()).copied(),
        }
    }

    /// SKILLUI1 R3: the rank rule of `can_use`: off without a rank choice, for basics, engine and always-active
    /// skills and for families outside a set loadout (they fail `knows_skill` anyway); otherwise, with a loadout or
    /// without one (every owned family is then active), only the family's effective rank is usable.
    pub fn rank_allowed(&self, cat: &Catalog, skill: &rpgkit_data::SkillDef) -> bool {
        if self.rank_choice.is_none() || skill.system || skill.basic || self.always_active(cat, &skill.id) {
            return true;
        }
        let fam = skill.family_id();
        let active = self.active_families();
        if !active.is_empty() && !active.iter().any(|f| f == fam) {
            return true;
        }
        self.effective_rank(cat, fam) == Some(skill.tier)
    }

    /// Known, unlocked, at the chosen rank (SKILLUI1 R3), and trained for (min_proficiency in the hand's weapon type).
    pub fn can_use(&self, cat: &Catalog, skill: &rpgkit_data::SkillDef) -> bool {
        if !self.knows_skill(cat, &skill.id) || !self.tier_unlocked(cat, skill) || !self.rank_allowed(cat, skill) {
            return false;
        }
        if self.known_skills.is_empty() || skill.min_proficiency <= 0 {
            return true;
        }
        let hands: &[&str] = match skill.weapon_hand.to_lowercase().as_str() {
            "off" => &["off"],
            "both" => &["main", "off"],
            _ => &["main"],
        };
        for hand in hands {
            let t = self.wielded_type(hand);
            if !skill.weapon_types.is_empty() && !skill.weapon_types.contains(&t) {
                return false;
            }
            let prof_key = if t == "unarmed" { "fist" } else { t.as_str() };
            if self.stats.proficiency.level_of(prof_key) < skill.min_proficiency {
                return false;
            }
        }
        true
    }

    pub fn apply_attack_boost(&mut self, multiplier: f64, rounds: i32) {
        self.attack_boost = multiplier;
        self.attack_boost_rounds = rounds;
    }

    /// Combatant.tick_round_buffs: clears riposte memory and counts the Focus boost down.
    pub fn tick_round_buffs(&mut self) {
        self.riposted.clear();
        if self.attack_boost_rounds > 0 {
            self.attack_boost_rounds -= 1;
            if self.attack_boost_rounds <= 0 {
                self.attack_boost = 1.0;
            }
        }
    }
}
