//! Combatant (build-time and derived parts) and ContentFactory.unit: a catalog unit entry -> a ready unit.

use crate::gear::{Equipment, Inventory, Slot, TIER_SOURCE};
use crate::grants::{self, Combine, Grant, Immunity, Modifier, SkillGrant, SkillSet, Target};
use crate::stats::{self, roundi, CharacterStats, StatModifier};
use aegis_core::value::to_int;
use rpgkit_data::{Catalog, ClassDef, Deck, Technique, UnitDef, UnitTemplate, NO_ITEM};
use indexmap::{IndexMap, IndexSet};
use serde_json::Value;

pub const BASE_AP: i32 = 10;
pub const GEAR_REACH_CAP: i32 = 1;
pub const GROUPS: [&str; 2] = ["spell", "heal"];
/// RULESET1 V4 (tasks 1.4): the widest foresight reveal range in tiles.
pub const FORESIGHT_RANGE_MAX: i32 = 8;
/// RULESET1 tasks 1.4: the largest `stagger_taken` multiplier.
pub const STAGGER_TAKEN_MULT_MAX: f64 = 2.0;
/// RULESET1 K1: the melee reach a reach weapon gives its wielder (rule `reach`; melee band 1 to this) when its family
/// has no `range_band` (PR5-28: the family's `range_band.max` wins, `Equipment::reach_max`).
pub const REACH_WEAPON_REACH: i32 = 2;
/// U3.2-S4 (FB-35): the source kind of the stat modifiers worn items' effects add (their `stat` effects and their
/// families' mastery passives'); every sync takes them all off before it applies the worn gear again. AUDIT-05: each
/// modifier names its item as `gear_effect:<grant source>` (`gear_effect_source`), so a breakdown can name the item.
pub const GEAR_EFFECT_SOURCE: &str = "gear_effect";

/// AUDIT-05: the stat modifier source of a worn item's effect granted under `source` (`gear:<item>`,
/// `gear:<item>:tier<n>`, `gear:<item>:mastery`): `gear_effect:<source>`.
pub fn gear_effect_source(source: &str) -> String {
    format!("{GEAR_EFFECT_SOURCE}:{source}")
}

/// AUDIT-05: a stat modifier source of the `GEAR_EFFECT_SOURCE` kind (the kind alone or `gear_effect:<source>`).
pub fn is_gear_effect_source(source: &str) -> bool {
    source.strip_prefix(GEAR_EFFECT_SOURCE).is_some_and(|r| r.is_empty() || r.starts_with(':'))
}

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

/// U4.2-S4 (FB-30): one hit-time modifier grant the unit holds (`skill_potency`, `stagger_taken`, `forced_move` tiles)
/// and its source (`passive:<id>`, a band's `passive:<source>`, `gear:<item>`, `gear:<item>:tier<n>`), in apply order.
/// The accumulators stay the numbers every hit reads (unchanged arithmetic, FB-26's combine rules); these name the
/// sources of the stage modifier the unit's evaluators give a hit (`potency_modifier`, `stagger_taken_modifier`,
/// `forced_tile_parts`), so the hit's trace shows them.
#[derive(Debug, Clone, PartialEq)]
pub struct HitSource {
    pub source: String,
    pub target: Target,
    pub value: f64,
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
    /// Adds one `forced_move` modifier of the typed view (U4.2-S1: an immunity, always or guarding, set by any row;
    /// `bonus_tiles` 0-2 a row, summed).
    fn add(&mut self, m: &Modifier) {
        let c = m.target.combine();
        match m.target {
            Target::ForcedImmune(Immunity::Always) => self.immune_always = c.fold_flag(self.immune_always, true),
            Target::ForcedImmune(Immunity::Guarding) => self.immune_guarding = c.fold_flag(self.immune_guarding, true),
            Target::ForcedTiles => self.bonus_tiles = c.fold_i(self.bonus_tiles, m.value as i32),
            _ => {}
        }
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
    /// PK1-S1 (3.1): the band's `heal_taken` percent.
    pub heal_taken: i32,
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

    /// CLASS-BASICS (user 2026-11-08): these terms for a unit of class `class_id` (in `cat`): the ruleset's basics, then
    /// the class's role basics (`ClassDef::basics`, each by its family), a repeat once. They take no deck slot (an
    /// open basic, `Unit::deck_basic`). An unknown class or one without basics: these terms unchanged.
    pub fn with_class(&self, cat: &Catalog, class_id: &str) -> Self {
        let mut out = self.clone();
        for id in cat.classes.get(class_id).map(|c| c.basics.as_slice()).unwrap_or_default() {
            let fam = cat.family_of(id).to_string();
            if !out.basics.contains(&fam) {
                out.basics.push(fam);
            }
        }
        out
    }

    /// `with_class` for the unit of catalog row `unit_id` (its `class_id`; an unknown row: unchanged).
    pub fn with_class_of(&self, cat: &Catalog, unit_id: &str) -> Self {
        self.with_class(cat, cat.units.get(unit_id).map(|e| e.class_id.as_str()).unwrap_or_default())
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
    /// R4-1 (user 2026-11-09): the tier's one factor (`stats::TierRow::mult`; the formula input `tier_mult`, which
    /// replaced `tier_hp` and `tier_power`): max HP and attack power's base-stage multiplier.
    pub tier_mult: f64,
    /// R4-1 (`U2.3-TIERS`, FB-78): the unit took a role template (`template_for`): its stats at level are the
    /// template's scaled by `tier_mult`, floor 6 (slice 2, FB-144; attack power does not scale them again), and its
    /// max HP is `CharacterStats::max_hp_scaled` (the tier scales the 60 only).
    /// False for every other unit (heroes, allies, a role without a template row): the old formula.
    pub tier_scaled: bool,
    pub max_hp: i32,
    pub hp: i32,
    /// U1.2-S3c: initiative (the old speed): action order; a fixed-stat row's `speed`.
    pub initiative: i32,
    /// U1.2-S3c: the move rate, tiles one move covers before haste and slow (`stats::move_rate_curve`, 1 to 4).
    pub move_rate: i32,
    pub attack_range: i32,
    pub max_ap: i32,
    pub ap: i32,
    pub stance: Stance,
    pub attack_boost: f64,
    pub guard: f64,
    pub power_override: i32,
    pub armor_override: i32,
    /// The party's carry overload penalty (`carry::Report::overload_penalty`); lowers initiative first.
    pub carry_penalty: i32,
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
    /// U3.2-S4 (FB-35, OFFHAND1): the skills worn items grant (their `skill` effects, their families' mastery
    /// passives'), rebuilt on every sync: owned, unlocked and always active while worn, no slot (`gear_skill`).
    pub gear_skills: Vec<String>,
    /// U3.2-S4 (U3 Level 2): crit chance points the worn gear's families give at proficiency 2 or more (their
    /// `level2_bonus` crit, each family once), rebuilt on every sync.
    pub gear_crit: f64,
    /// HOOKS1 8.3: the held guard a trigger set this round (None in classic and without the rule).
    pub held_guard: Option<HeldGuard>,
    /// HOOKS1 8.2: crit chance points the trigger effect `crit` added for the battle (0 in classic).
    pub trigger_crit: f64,
    /// PK1-S1 (3.1): the percent the held meter bands' `heal_taken` add to the healing this unit receives (0 = none;
    /// `World::heal` reads it).
    pub band_heal_taken: i32,
    /// PK1-KITFIX (b): the ratings a trigger's `rating_set` holds at a value (one per rating; empty = none, so every
    /// rating reads as before).
    pub rating_sets: Vec<crate::status::RatingSet>,
    /// HOOKS1 UM1: the passives' `meter` effects in apply order; read only under the rule unit_meters.
    pub passive_meters: Vec<HeldMeter>,
    /// U4.2-S4 (FB-30): the hit-time modifier grants of passives and meter bands, with their sources (`HitSource`); a
    /// band's go when the band is left.
    pub hit_sources: Vec<HitSource>,
    /// U4.2-S4: the worn items', rebuilt on every sync.
    pub gear_hit_sources: Vec<HitSource>,
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
    /// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): the passives' `forbid_hands` (0 = none; 2 = no two-handed item).
    pub forbid_hands: i32,
    /// PK2-S1: the passives' `pierce_melee` (melee and bare-handed hits pierce armour under the rule pierce).
    pub pierce_melee: bool,
    /// PK2-S1: a party member's permanent max HP offset (added to the derived max HP on every sync, at least 1 HP).
    pub max_hp_offset: i32,
    /// PK2-S1: a party member's unit track stages (track id -> stage; a track it is not on reads 0). None = the unit holds
    /// no track (a catalog unit, an enemy, a guest, or any unit while the catalog has no track row): a battle mark never
    /// advances it.
    pub tracks: Option<IndexMap<String, i32>>,
    /// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3): a party member's graft count (its body slots in use) once the catalog
    /// has a body slot row. None = the unit holds no body (a catalog unit, an enemy, a guest, or any unit while the
    /// catalog has no body slot row): no floor table applies and the ruleset's `channeled_actives` limits its marks.
    pub graft_count: Option<i32>,
    /// PK2-S2 (FB-167 Q6): a party member's floor offset for this battle, added to every floor a meter effect's
    /// `floors` table gives (0 = none).
    pub floor_offset: i32,
    /// PK2-S2: the skills its keystone grafts teach: open without an active slot, each taking one of the deck's keystone
    /// slots (`slotted`). PK2-S4: or the wielded family's keystone (`mastery_keystone`).
    pub keystone_skills: Vec<String>,
    /// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): a party member's weapon mastery levels (gear type -> 1-3). None = the
    /// unit holds no mastery (a catalog unit, an enemy, a guest, a member with no mastery level and no masterwork main
    /// hand): no technique opens.
    pub mastery: Option<IndexMap<String, i32>>,
    /// PK2-S4: the main-hand weapon is a masterwork instance (its own `technique` opens; weapon instances, PR5-13a).
    pub masterwork: bool,
    /// PK2-S4: the techniques open now: the main-hand weapon family's first n at mastery n, then a masterwork weapon's
    /// own; rebuilt on every sync. Owned and open without a slot, standard rounds only, greyed under the wrong grip.
    pub techniques: Vec<Technique>,
    /// PK2-S4: the wielded family's keystone the unit holds (mastery 3 and its forge rite; "" = none; set once at build
    /// from the saved main hand, not on a mid-battle swap, unlike `techniques`), also in
    /// `keystone_skills`; standard rounds only.
    pub mastery_keystone: String,
}

impl Unit {
    /// U1.2-S3c: `initiative` is a fixed-stat row's `speed`; its move rate is the move curve on it (as before the split).
    fn blank(name: &str, hp: i32, initiative: i32, range: i32, stats: CharacterStats) -> Self {
        Self {
            name: name.into(),
            catalog_id: String::new(),
            tier: "hero".into(),
            tier_mult: 1.0,
            tier_scaled: false,
            max_hp: hp,
            hp,
            initiative,
            move_rate: stats::move_rate_curve(initiative),
            attack_range: range,
            max_ap: BASE_AP,
            ap: BASE_AP,
            stance: Stance::Neutral,
            attack_boost: 1.0,
            guard: 0.0,
            power_override: 0,
            armor_override: -1,
            carry_penalty: 0,
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
            gear_skills: Vec::new(),
            gear_crit: 0.0,
            held_guard: None,
            trigger_crit: 0.0,
            band_heal_taken: 0,
            rating_sets: Vec::new(),
            passive_meters: Vec::new(),
            hit_sources: Vec::new(),
            gear_hit_sources: Vec::new(),
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
            forbid_hands: 0,
            pierce_melee: false,
            max_hp_offset: 0,
            tracks: None,
            graft_count: None,
            floor_offset: 0,
            keystone_skills: Vec::new(),
            mastery: None,
            masterwork: false,
            techniques: Vec::new(),
            mastery_keystone: String::new(),
        }
    }

    /// The ready weapon's type (the stat context of weapon-conditional modifiers); AUDIT-09: the off-hand backup's
    /// while it is the ready one of a 2H pair (`Inventory::active_weapon`).
    pub fn weapon_context(&self) -> Option<String> {
        self.inventory.active_weapon().map(|w| w.weapon_type().to_string())
    }

    pub fn min_attack_range(&self) -> i32 {
        if self.attack_range > 1 {
            2
        } else {
            1
        }
    }

    /// RULESET1 K1, K2: whether the unit is melee (`attack_range` at most 1, which the rule leaves as it is) with a
    /// ready weapon (AUDIT-09: `Inventory::active_weapon`) whose family has the engine rule `reach`. A gear fact: the
    /// battle reads it only under the rule `reach` (`rpgkit_battle::skill::melee_reach`).
    pub fn has_reach_weapon(&self) -> bool {
        self.attack_range <= 1 && self.inventory.active_weapon().is_some_and(|w| w.has_reach())
    }

    /// PR5-28 (U3 FB-36c): the melee reach of the unit's ready weapon under the rule `reach`: its family's
    /// `range_band.max`, else `REACH_WEAPON_REACH` (also without a ready weapon; read only when `has_reach_weapon`).
    pub fn weapon_reach(&self) -> i32 {
        self.inventory.active_weapon().map_or(REACH_WEAPON_REACH, |w| w.reach_max())
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

    /// F1.2-S1 (FB-58c): the U1 Initiative rating the within-block order reads (`priority * 1000 + initiative`); the one
    /// read point the burden change (FB-07) moves.
    pub fn initiative_rating(&self) -> i32 {
        self.initiative
    }

    pub fn has_status(&self, s: &str) -> bool {
        self.st.active.contains(s)
    }

    pub fn active_magnitude(&self, s: &str) -> i32 {
        if self.has_status(s) { self.st.magnitudes.get(s).copied().unwrap_or(0) } else { 0 }
    }

    /// W2-5 (C09): the summed `active_magnitude` of the active statuses passing `read` (a status-table predicate).
    pub fn active_magnitude_where(&self, read: impl Fn(&str) -> bool) -> i32 {
        self.st.active.iter().filter(|s| read(s)).map(|s| self.active_magnitude(s)).sum()
    }

    /// Whether an active status passes `read` (the predicate form of `has_status`).
    pub fn has_status_where(&self, read: impl Fn(&str) -> bool) -> bool {
        self.st.active.iter().any(|s| read(s))
    }

    /// PK1-KITFIX (b): the value `rating` is held at (a trigger's `rating_set`), None when nothing holds it.
    pub fn rating_set(&self, rating: crate::status::Rating) -> Option<i32> {
        self.rating_sets.iter().find(|r| r.rating == rating).map(|r| r.value)
    }

    /// PK1-KITFIX (b): holds `rating` at `value` for `rounds` more round starts (below 0: the rest of the battle); a set
    /// of the same rating replaces the one there.
    pub fn set_rating(&mut self, rating: crate::status::Rating, value: i32, rounds: i32) {
        self.rating_sets.retain(|r| r.rating != rating);
        self.rating_sets.push(crate::status::RatingSet { rating, value, rounds: if rounds < 0 { -1 } else { rounds } });
    }

    /// Move Rate +1 under a Move Rate buff (haste), -1 under a Move Rate debuff (slow), floor 1; W2-5 (C09): the
    /// statuses are the table's `modifier` rows with `rating` move_rate (one step each way however many are active).
    /// PK1-KITFIX (b): a held `move_rate` is the value instead (at least 1).
    pub fn move_per_block(&self) -> i32 {
        if let Some(v) = self.rating_set(crate::status::Rating::MoveRate) {
            return v.max(1);
        }
        let hasted = self.has_status_where(|s| crate::status::raises_rating(s, crate::status::Rating::MoveRate)) as i32;
        let slowed = self.has_status_where(|s| crate::status::lowers_rating(s, crate::status::Rating::MoveRate)) as i32;
        (self.move_rate + hasted - slowed).max(1)
    }

    /// The worn gear's burden after the training reduction; U3.2-S3 (FB-33): plus the wield terms'
    /// `untrained_burden_add` per item worn untrained (proficiency 0), which lowers initiative first like any burden.
    pub fn worn_burden(&self) -> i32 {
        self.inventory.gear().map(|e| (e.item.burden - self.stats.proficiency.burden_reduction(e.weapon_type()) - self.level2_burden(e)).max(0) + e.wield_penalty(&self.stats).burden_add).sum()
    }

    /// U3.2-S4 (U3 Level 2): the burden the item's family's `level2_bonus` takes off it at proficiency 2 or more.
    fn level2_burden(&self, e: &Equipment) -> i32 {
        if self.stats.proficiency.level_of(e.weapon_type()) >= 2 { e.item.level2_bonus.burden } else { 0 }
    }

    pub fn debuff_resistance(&self) -> f64 {
        self.stats.debuff_resistance(self.weapon_context().as_deref())
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Gear stat bonuses -> tier stat bonuses (read without tier modifiers) -> gear effects -> derived numbers.
    /// AUDIT-08/09: from the ready gear (`Inventory::ready_gear`; a stowed backup weapon adds only its burden) and the
    /// ready weapon's range and type; call it again after `Inventory::toggle_active_weapon`.
    pub fn sync_from_stats_and_gear(&mut self, cat: &Catalog) {
        self.stats.remove_modifier("gear");
        // U3.2-S4 (FB-35): what the worn items' effects added goes first (tier criteria never read it)
        self.stats.modifiers.retain(|m| !is_gear_effect_source(&m.source));
        let bonuses: Vec<(String, i32)> =
            self.inventory.ready_gear().flat_map(|e| e.item.stat_bonuses.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>()).collect();
        for (k, v) in bonuses {
            if v != 0 {
                self.stats.modifiers.push(StatModifier::permanent(&k, v, "gear", "always"));
            }
        }
        self.rebuild_gear_tier_stats();
        self.rebuild_gear_effects(cat);
        let burden = self.worn_burden();
        let (base_rng, ctx) = match self.inventory.active_weapon() {
            Some(w) => (w.item.base_range, Some(w.weapon_type().to_string())),
            None => (1, None),
        };
        let ctx = ctx.as_deref();
        // PK2-S1: a member's permanent max HP offset on top of the derived max
        self.max_hp = (if self.tier_scaled { self.stats.max_hp_scaled(self.tier_mult, ctx) } else { self.stats.max_hp(self.tier_mult, ctx) } + self.max_hp_offset).max(1);
        self.hp = self.hp.min(self.max_hp);
        self.initiative = (self.stats.initiative(burden, ctx) - self.carry_penalty).max(1);
        self.move_rate = self.stats.move_rate(burden + self.carry_penalty, ctx);
        self.attack_range = self.stats.effective_range(base_rng, ctx);
        self.refresh_path_skills(cat);
    }

    fn rebuild_gear_tier_stats(&mut self) {
        self.stats.remove_modifier(TIER_SOURCE);
        let mut totals: IndexMap<String, i32> = IndexMap::new();
        for eq in self.inventory.ready_gear() {
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
        self.gear_hit_sources.clear();
        self.gear_skills.clear();
        self.gear_crit = 0.0;
        // AUDIT-01: a 2H pair's stowed weapon grants nothing while it is not ready (`Inventory::ready_gear`)
        let item_fx: Vec<(String, Vec<Value>)> = self.inventory.ready_gear().map(|e| (format!("gear:{}", e.id()), e.item.effects.clone())).collect();
        for (src, fx) in item_fx {
            self.apply_effects(cat, &fx, &src, true);
        }
        let tier_fx: Vec<(String, Vec<Value>)> = self
            .inventory
            .ready_gear()
            .flat_map(|e| e.active_tier_effects(&self.stats).into_iter().map(|(n, fx)| (format!("gear:{}:tier{}", e.id(), n), fx)).collect::<Vec<_>>())
            .collect();
        for (src, fx) in tier_fx {
            self.apply_effects(cat, &fx, &src, true);
        }
        self.apply_family_bonuses(cat);
        self.techniques = self.open_techniques();
    }

    /// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): the techniques open now (`techniques`): none without mastery or a main-hand
    /// weapon; else the weapon family's first n techniques at mastery n (at most 3), then the weapon's own technique when
    /// it is a masterwork instance, each skill once.
    fn open_techniques(&self) -> Vec<Technique> {
        let (Some(levels), Some(main)) = (&self.mastery, self.inventory.get(Slot::MainHand)) else { return Vec::new() };
        if main.item.item_type != "WEAPON" {
            return Vec::new();
        }
        let level = levels.get(main.weapon_type()).copied().unwrap_or(0).clamp(0, rpgkit_data::mastery::MASTERY_MAX);
        let mut out: Vec<Technique> = main.item.family_techniques.iter().take(level as usize).cloned().collect();
        if let Some(own) = main.item.technique.as_ref().filter(|_| self.masterwork) {
            out.push(own.clone());
        }
        let mut seen: Vec<String> = Vec::new();
        out.retain(|t| {
            let fresh = !seen.contains(&t.skill);
            seen.push(t.skill.clone());
            fresh
        });
        out
    }

    /// PK2-S4: the open technique of skill `skill_id` (None: not a technique now).
    pub fn technique(&self, skill_id: &str) -> Option<&Technique> {
        self.techniques.iter().find(|t| t.skill == skill_id)
    }

    /// PK2-S4: whether `skill_id` is a technique open now or the unit's wielded-family keystone: owned and open without
    /// a slot, usable only in standard rounds (never in a Channel round).
    pub fn technique_skill(&self, skill_id: &str) -> bool {
        self.technique(skill_id).is_some() || (!self.mastery_keystone.is_empty() && self.mastery_keystone == skill_id)
    }

    /// PK2-S4: why technique `skill_id` is greyed now (None: it fits, or it is no technique): its grip tag against the
    /// main-hand weapon's grip (`Technique::grip_problem`).
    pub fn technique_problem(&self, skill_id: &str) -> Option<&'static str> {
        let t = self.technique(skill_id)?;
        t.grip_problem(self.inventory.get(Slot::MainHand).is_some_and(|e| e.item.two_handed))
    }

    /// U3.2-S4 (U3 Level 2 and Level 3): each worn gear family once, at the unit's proficiency in it: from level 2 its
    /// `level2_bonus` crit (the burden part is `worn_burden`'s), at level 3 its `level3_passive`'s effects, applied as the
    /// item's (`gear:<item>:mastery`; worn gear's grants only, rebuilt with them on every sync). AUDIT-01: a 2H pair's
    /// stowed weapon counts no family while it is not ready (`Inventory::ready_gear`).
    fn apply_family_bonuses(&mut self, cat: &Catalog) {
        let mut seen: Vec<String> = Vec::new();
        let worn: Vec<(String, String, rpgkit_data::FamilyBonus, String)> =
            self.inventory.ready_gear().map(|e| (e.id().to_string(), e.weapon_type().to_string(), e.item.level2_bonus, e.item.level3_passive.clone())).collect();
        for (item, family, level2, level3) in worn {
            if family.is_empty() || seen.contains(&family) {
                continue;
            }
            seen.push(family.clone());
            let level = self.stats.proficiency.level_of(&family);
            if level >= 2 && level2.crit != 0.0 {
                self.apply_grants(cat, &[Grant::Modifier(Modifier { target: Target::Crit, value: level2.crit })], &format!("gear:{item}:level2"), true);
            }
            if level >= 3 {
                if let Some(p) = cat.passives.get(&level3) {
                    let fx = p.effects.clone();
                    self.apply_effects(cat, &fx, &format!("gear:{item}:mastery"), true);
                }
            }
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

    /// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): `passive` granted mid-battle for the rest of it (a unit track's stage
    /// reached in battle): applied once (`apply_passive`), then the derived numbers re-read (move rate, initiative,
    /// range) keeping the battle's max HP and HP (a wounded cut stays). Whether it was newly applied.
    pub fn grant_passive_now(&mut self, cat: &Catalog, passive: &rpgkit_data::PassiveDef) -> bool {
        if self.passives.contains(&passive.id) {
            return false;
        }
        self.apply_passive(cat, passive);
        if !self.passives.contains(&passive.id) {
            return false;
        }
        let (max, hp) = (self.max_hp, self.hp);
        self.sync_from_stats_and_gear(cat);
        self.max_hp = max;
        self.hp = hp.min(max);
        true
    }

    /// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3): the floor a meter effect's `floors` table gives this unit: the entry of
    /// its graft count (a count past the table reads the last), plus its floor offset; None without a table, a body or a
    /// graft (no graft: not a holder of the table's floor).
    pub fn table_floor(&self, floors: &[i32]) -> Option<i32> {
        let n = usize::try_from(self.graft_count?).ok().filter(|n| *n > 0)?;
        floors.get(n.min(floors.len()).checked_sub(1)?).map(|f| f + self.floor_offset)
    }

    /// PK2-S2: how many deck actives the unit may mark channeled: one per graft for a unit with a body (its deck's
    /// actives bound it), else the ruleset's `channeled_actives` (`ruleset`).
    pub fn channeled_limit(&self, ruleset: i32) -> i32 {
        self.graft_count.unwrap_or(ruleset).max(0)
    }

    /// PK2-S2: whether a keystone graft teaches `skill_id` (open without an active slot).
    pub fn keystone_skill(&self, skill_id: &str) -> bool {
        self.keystone_skills.iter().any(|s| s == skill_id)
    }

    /// PK2-S1: whether the member grant `forbid_hands` refuses `item` (a two-handed item under `hands` 2).
    pub fn forbids(&self, item: &rpgkit_data::ItemDef) -> bool {
        forbids_hands(self.forbid_hands, item)
    }

    /// Passives.apply_effects (item effects and passives share the grammar): the typed grant view of `effects`
    /// (U4.2-S1, `grants::read`), applied by `apply_grants`.
    pub fn apply_effects(&mut self, cat: &Catalog, effects: &[Value], source: &str, gear: bool) {
        self.apply_grants(cat, &grants::read(effects), source, gear);
    }

    /// U4.2-S1: applies typed grants under `source` (`passive:<id>`, `gear:<item>` or `gear:<item>:tier<n>`; `gear`
    /// for a worn item's) to today's accumulators, each target by its combine rule, in grant order (the foresight
    /// source, the trigger keys and the meter order read it).
    pub fn apply_grants(&mut self, cat: &Catalog, list: &[Grant], source: &str, gear: bool) {
        let id = source.trim_start_matches("passive:").to_string();
        for g in list.iter().filter(|g| !gear || g.gear_ok()) {
            match g {
                Grant::Modifier(m) => self.apply_modifier(cat, m, &id, gear),
                Grant::Proficiency { gear: family, levels } => self.stats.proficiency.add_bonus(family, *levels),
                // U3.2-S4 (FB-35, OFFHAND1): a worn item's skill is held apart, rebuilt on every sync (no slot)
                Grant::GrantSkill(SkillGrant::Learn(skill)) if gear => {
                    if !self.gear_skills.contains(skill) {
                        self.gear_skills.push(skill.clone());
                    }
                }
                Grant::GrantSkill(SkillGrant::Learn(skill)) => self.learn_skill(cat, skill),
                Grant::GrantSkill(SkillGrant::Unlock { set, levels }) => {
                    let (map, key) = match set {
                        SkillSet::Family(f) => (&mut self.unlock_bonus_family, f),
                        SkillSet::Group(g) => (&mut self.unlock_bonus_group, g),
                    };
                    let cur = map.get(key).copied().unwrap_or(0);
                    map.insert(key.clone(), SkillGrant::UNLOCK_COMBINE.fold_i(cur, *levels));
                    self.refresh_path_skills(cat);
                }
                // HOOKS1 8.1: passives and worn items (the gear list is cleared on every sync); a tier effect counts as
                // its item's
                Grant::Trigger(row) => {
                    let from = if gear { format!("gear:{}", source.trim_start_matches("gear:").split(':').next().unwrap_or("")) } else { source.to_string() };
                    let list = if gear { &mut self.gear_triggers } else { &mut self.passive_triggers };
                    let key = format!("{source}#{}", list.len());
                    list.push(HeldTrigger { source: from, key, row: row.clone() });
                }
                // HOOKS1 UM1: passives only (HOOKS1-S gives items no meter effect)
                // PK2-S2: a `floors` table reads the member's graft count (plus its floor offset) over `floor`
                Grant::GrantPool { meter, start, floor, floors } => {
                    let floor = self.table_floor(floors).or(*floor);
                    self.passive_meters.push(HeldMeter { meter: meter.clone(), start: *start, floor });
                }
                // PK2-S1: member grants, passives only (`Grant::gear_ok`)
                Grant::ForbidHands { hands } => self.forbid_hands = self.forbid_hands.max(*hands),
                Grant::PierceMelee => self.pierce_melee = true,
            }
        }
    }

    /// One modifier grant (`id`: the source without `passive:`).
    fn apply_modifier(&mut self, cat: &Catalog, m: &Modifier, id: &str, gear: bool) {
        let c: Combine = m.target.combine();
        // U4.2-S4 (FB-30): a hit-time target keeps its source (a passive's or band's as its stat modifiers name it)
        if matches!(m.target, Target::SkillPotency(_) | Target::StaggerTaken { .. } | Target::ForcedTiles) {
            let held = HitSource { source: if gear { id.to_string() } else { format!("passive:{id}") }, target: m.target.clone(), value: m.value };
            if gear {
                self.gear_hit_sources.push(held);
            } else {
                self.hit_sources.push(held);
            }
        }
        let int = m.value as i32;
        match &m.target {
            Target::Stat { stat, tag, percent } => {
                // U3.2-S4 (FB-35): a worn item's stat effect under the gear source every sync takes off (AUDIT-05: naming
                // its item)
                let source = if gear { gear_effect_source(id) } else { format!("passive:{id}") };
                let mut sm = StatModifier::permanent(stat, int, &source, tag);
                // DERIVE1 4.1: the optional percent counts only on a derived id (base stats stay flat-only)
                if cat.derived_stats.contains_key(stat) {
                    sm.percent = *percent;
                }
                self.stats.modifiers.push(sm);
            }
            Target::Resist { status } => {
                let held = if gear { &mut self.gear_status_resist } else { &mut self.status_resist };
                let cur = held.get(status).copied().unwrap_or(0);
                held.insert(status.clone(), c.fold_i(cur, int));
            }
            Target::ApRegen => self.ap_regen_bonus = c.fold_i(self.ap_regen_bonus, int),
            Target::SkillPotency(set) => {
                let (held, key) = match set {
                    SkillSet::Group(g) => (if gear { &mut self.gear_potency_group } else { &mut self.potency_group }, g),
                    SkillSet::Family(f) => (if gear { &mut self.gear_potency_family } else { &mut self.potency_bonus }, f),
                };
                let slot = held.entry(key.clone()).or_insert(0.0);
                *slot = c.fold_f(*slot, m.value);
            }
            Target::StatusPotency => {
                let held = if gear { &mut self.gear_status_potency } else { &mut self.passive_status_potency };
                *held = c.fold_i(*held, int);
            }
            Target::SpellReach => {
                let held = if gear { &mut self.gear_spell_reach } else { &mut self.passive_spell_reach };
                *held = c.fold_i(*held, int);
            }
            Target::Foresight => {
                if self.foresight_source.is_empty() {
                    self.foresight_source = id.to_string();
                }
                self.foresight_range = c.fold_i(self.foresight_range, int);
            }
            // RULESET1 P6: passives only; the battle multiplies the rows that match an action's length
            Target::StaggerTaken { max_blocks } => self.stagger_taken.push(StaggerTaken { mult: m.value, max_blocks: *max_blocks }),
            // HOOKS1 FM5: passives and worn items (the gear set is cleared on every sync)
            Target::ForcedTiles | Target::ForcedImmune(_) => {
                if gear {
                    self.gear_forced.add(m);
                } else {
                    self.passive_forced.add(m);
                }
            }
            // HOOKS1 UM4: a band's crit, read by `apply_band` only; U3.2-S4: a worn family's Level-2 crit
            Target::Crit if gear => self.gear_crit = c.fold_f(self.gear_crit, m.value),
            Target::Crit => {}
            // PK1-S1 (3.1): a band's, read by `apply_band` only
            Target::HealTaken => {}
        }
    }

    /// HOOKS1 UM4: applies a meter band's `effects` (the passive grammar: stat, resist, stagger_taken, skill_potency,
    /// status_potency; `crit` as the trigger effect) under `source` and returns what they added (`remove_band` takes it
    /// off again).
    pub fn apply_band(&mut self, cat: &Catalog, effects: &[Value], source: &str) -> BandApplied {
        let resist = self.status_resist.clone();
        let (fam, group) = (self.potency_bonus.clone(), self.potency_group.clone());
        let (potency, stagger) = (self.passive_status_potency, self.stagger_taken.len());
        let view = grants::read(effects);
        let passive: Vec<Grant> = view.iter().filter(|g| g.band_ok()).cloned().collect();
        self.apply_grants(cat, &passive, source, false);
        let crit = view.iter().filter_map(Grant::band_crit).fold(0.0, |held, v| Target::Crit.combine().fold_f(held, v));
        self.trigger_crit += crit;
        let heal_taken = view.iter().filter_map(Grant::band_heal_taken).fold(0.0, |held, v| Target::HealTaken.combine().fold_f(held, v)) as i32;
        self.band_heal_taken += heal_taken;
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
            heal_taken,
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
        self.band_heal_taken -= a.heal_taken;
        // U4.2-S4: the band's hit-time sources go with it
        self.hit_sources.retain(|h| h.source != a.source);
    }

    /// HOOKS1 UM6: whether the battle gave this unit `skill_id` as a meter skill.
    pub fn meter_skill(&self, skill_id: &str) -> bool {
        self.meter_skills.iter().any(|s| s == skill_id)
    }

    /// U3.2-S4 (OFFHAND1, FB-35): whether a worn item grants `skill_id` now (owned, unlocked and open without a slot
    /// while worn, like a meter skill; gone when the item comes off).
    pub fn gear_skill(&self, skill_id: &str) -> bool {
        self.gear_skills.iter().any(|s| s == skill_id)
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
        // PK2-S2: a keystone graft's skill sits in the keystone slot, open without an active slot
        // PK2-S4: a technique is open without a slot while its weapon is wielded
        if self.meter_skill(skill_id) || self.gear_skill(skill_id) || self.keystone_skill(skill_id) || self.technique_skill(skill_id) {
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
            // PK2-S2: each keystone graft's skill takes one of the keystone slots
            "keystone" => DeckRule::take(&self.deck.keystone, (rule.slots.keystone - self.keystone_skills.len() as i32).max(0)),
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
            // U3.2-S4 (OFFHAND1): a family owned only through worn gear takes no slot (it is open while worn)
            // PK2-S2: nor does a keystone graft's (it sits in the keystone slot)
            // PK2-S4: nor does a technique (open while its weapon is wielded)
            let owned = cat.family_rows(&fam).iter().any(|s| self.owns_skill(cat, &s.id) && !self.gear_skill(&s.id) && !self.keystone_skill(&s.id) && !self.technique_skill(&s.id));
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
        // PK2-S4: a technique open now is owned
        if self.meter_skill(skill_id) || self.gear_skill(skill_id) || self.technique_skill(skill_id) {
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

    /// KNOW for unlocks: base + permanent flat modifiers that apply in every context. R4-1 slice 2 (FB-144): a
    /// templated unit's base is its unscaled KNOW (`CharacterStats::requirement_base`).
    pub fn unlock_know(&self) -> i32 {
        self.stats.requirement_base("KNOW")
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
/// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): whether a `forbid_hands` of `hands` (0 = none) refuses `item`: 2 refuses
/// every two-handed item (bows and arbalests included).
pub fn forbids_hands(hands: i32, item: &rpgkit_data::ItemDef) -> bool {
    hands >= 2 && item.two_handed
}

pub fn rank_numeral(rank: i32) -> &'static str {
    ["-", "I", "II", "III", "IV", "V"].get(rank.max(0) as usize).copied().unwrap_or("V+")
}

/// SkillDef.in_group for the passive groups: "spell" = an attack that needs a spell-source weapon (U3.2-S2: a family
/// with the engine rule `spell_source`, `SkillDef::spell_source`); "heal" = a support heal (HEALPCT1: `heal_pct` too).
pub fn skill_in_group(s: &rpgkit_data::SkillDef, group: &str) -> bool {
    match group {
        "spell" => s.kind == "ATTACK" && s.spell_source,
        "heal" => s.kind == "SUPPORT" && (s.heal_base > 0 || s.heal_mult > 0.0 || s.heal_pct > 0),
        _ => false,
    }
}

/// The side whose units are the player's (heroes and allies): never templated.
pub const PARTY_SIDE: &str = "party";

/// R4-1 (`U2.3-TIERS`, FB-78, `docs/sessions/U2_CLASS.md` item 2): the role template catalog unit `e` of class `class`
/// takes its reference stats from: a derived unit that is not the party's (`PARTY_SIDE`) and whose tier is not the hero
/// tier (an unknown tier reads as the hero's, `stats::tier_row`) takes the `unit_templates` row of its AI role (its own
/// `ai_role`, else its class's). None for every other unit, and when the table has no row for the role (the kind ships
/// empty until the asset lane fills it, so the goldens hold). Slice 2 (FB-144): the unit's `template` overrides the
/// role: `TEMPLATE_CLASS` = none (the class's shares), a template role or id = that row (None when no row matches).
pub fn template_for<'c>(cat: &'c Catalog, e: &UnitDef, class: &ClassDef) -> Option<&'c UnitTemplate> {
    if e.stat_mode != "derived" || e.side == PARTY_SIDE || stats::tier_row(&e.tier).id == stats::HERO_TIER {
        return None;
    }
    match e.template.as_str() {
        "" => cat.unit_templates.get(if e.ai_role.is_empty() { &class.ai_role } else { &e.ai_role }),
        rpgkit_data::TEMPLATE_CLASS => None,
        key => rpgkit_data::unit_template::lookup(&cat.unit_templates, key),
    }
}

/// R4-1 slice 2 (FB-144): the growth shares catalog unit `e` spends its free points by: its role template's
/// (`template_for`), else its class's; empty without a class.
pub fn spend_growth(cat: &Catalog, e: &UnitDef) -> IndexMap<String, f64> {
    let Some(class) = class_for(cat, e) else { return IndexMap::new() };
    template_for(cat, e, class).map(|t| t.over(class)).as_ref().unwrap_or(class).unit_shares().growth
}

/// AUDIT-02 (FB-143): catalog unit row `e` raised to level `level` (a stage-scaled foe, `foe_level`). A row with an
/// explicit `spend` keeps it and auto-spends the extra pool (`spend_pool(level) - spend_pool(base level)`, one point a
/// level) by its growth shares (`spend_growth`, `stats::auto_spend`), so the raise leaves no point unallocated; a row
/// without `spend` auto-spends its whole pool at the level it is built at anyway. A level at or below the base only
/// sets the level.
pub fn raise_level(cat: &Catalog, e: &mut UnitDef, level: i32) {
    let base = e.level;
    e.level = level;
    if e.spend.is_empty() || level <= base {
        return;
    }
    let extra = stats::spend_pool(level) - stats::spend_pool(base);
    for (stat, points) in stats::auto_spend(&spend_growth(cat, e), extra) {
        *e.spend.entry(stat).or_insert(0) += points;
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
/// slots. None = `build_unit` exactly (classic). The row alone through `build_from`.
pub fn build_unit_in(cat: &Catalog, e: &UnitDef, default_ap: i32, deck: Option<&DeckRule>) -> Unit {
    build_from(cat, UnitBuild::row(e, default_ap, deck)).unit
}

/// U2.2-S2 (FB-12): a character's tuning deltas (its five keys), applied after everything else.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tuning {
    pub hp_pct: i32,
    pub power: i32,
    pub armor: i32,
    /// The world tuning key `speed`: an initiative delta (U1.2-S3c; the move rate is left as built).
    pub initiative: i32,
    pub ap: i32,
}

/// U2.2-S2 (FB-12): what a unit wears over its row, the one override layer of every unit: the passive ids of its
/// sources in gather order (FB-28: character, faction, disciplines, named items; a member's other owned ids after),
/// applied once each; the named items it wears; its tuning. `path_cat`: the catalog whose class paths the unit takes
/// (HOOKS1 CN3: the world's overlay; None = the catalog passed to `build_from`; the two differ only in class paths, and
/// only the row's build reads a class).
#[derive(Debug, Clone, Default)]
pub struct Overrides {
    pub passives: Vec<String>,
    pub named_items: Vec<rpgkit_data::ItemDef>,
    pub tuning: Tuning,
    pub path_cat: Option<std::sync::Arc<Catalog>>,
}

/// U2.2-S2 (FB-12): a growth unit's saved state (a party member's; its row carries the saved level and spend): its deck
/// (empty = the row's), learned skills, worn gear, pouch, stored loadout (None = the class default) and chosen ranks.
#[derive(Debug, Clone, Default)]
pub struct Growth {
    pub deck: Deck,
    pub learned_skills: Vec<String>,
    pub equipment: Vec<(Slot, rpgkit_data::ItemDef)>,
    pub pouch: Vec<rpgkit_data::ItemDef>,
    pub loadout: Option<(Vec<String>, Vec<String>)>,
    pub ranks: IndexMap<String, i32>,
    /// A1.2-K3: the played ruleset's `pouch_stacks` (the battle pouch takes stacks; `Inventory::pouch_stacks`).
    pub pouch_stacks: bool,
    /// PK2-S1: the member's permanent max HP offset (`Unit::max_hp_offset`).
    pub max_hp_offset: i32,
    /// PK2-S1: the member's unit track stages (`Unit::tracks`; the passives come in `Overrides::passives`).
    pub tracks: IndexMap<String, i32>,
    /// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3): the `GRAFT` items in the member's body slots (slot order); their
    /// `body_effects` apply as a passive's (a meter included), each counts toward the graft count once the catalog has a
    /// body slot row (`Unit::graft_count`).
    pub body: Vec<rpgkit_data::ItemDef>,
    /// PK2-S2 (FB-167 Q6): the member's floor offset for the next battle (`Unit::floor_offset`).
    pub floor_offset: i32,
    /// PK2-S3 (`docs/sessions/PK2_GRAFTS.md` 5): gear type -> the proficiency levels the member trained (a recipe), on
    /// top of its class's and its drills' (`Proficiency::add_bonus`; the level still tops out at `PROF_MAX_LEVEL`).
    pub trained: IndexMap<String, i32>,
    /// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): gear type -> the member's weapon mastery level (1-3; `Unit::mastery`).
    pub mastery: IndexMap<String, i32>,
    /// PK2-S4: the member's main-hand weapon is a masterwork instance (`Unit::masterwork`; weapon instances, PR5-13a).
    pub masterwork: bool,
    /// PK2-S4: the member's forge rite is done (with mastery 3 the wielded family's keystone opens, `Unit::mastery_keystone`).
    pub forge_rite: bool,
}

/// U2.2-S2 (FB-12): everything one unit is built from. A static unit (an NPC, a named enemy, a Dojo or sim unit) is its
/// row at the row's fixed level plus `over`; a growth unit (a party member) is its row at its saved level and spend,
/// plus `over` and `growth`.
#[derive(Debug, Clone)]
pub struct UnitBuild<'a> {
    pub row: &'a UnitDef,
    /// The AP of a row without its own (0 = the Combatant default).
    pub default_ap: i32,
    /// The battle's deck rule (None = classic).
    pub deck: Option<&'a DeckRule>,
    /// U3.2-S3: the wield terms of the battle's ruleset (`Numbers::wield`; the default outside a battle).
    pub wield: crate::stats::WieldRule,
    pub over: Overrides,
    pub growth: Option<Growth>,
}

impl<'a> UnitBuild<'a> {
    /// The row alone (a catalog unit: no overrides, no saved state).
    pub fn row(row: &'a UnitDef, default_ap: i32, deck: Option<&'a DeckRule>) -> Self {
        Self { row, default_ap, deck, wield: crate::stats::WieldRule::default(), over: Overrides::default(), growth: None }
    }

    /// AUDIT-03: the build under a ruleset's wield terms (`Numbers::wield`).
    pub fn with_wield(self, wield: crate::stats::WieldRule) -> Self {
        Self { wield, ..self }
    }
}

/// What `build_from` gives back: the unit, a growth unit's saved pouch ids that found no battle-pouch slot, and why its
/// stored loadout was ignored (None = kept or none stored).
#[derive(Debug, Clone)]
pub struct BuiltUnit {
    pub unit: Unit,
    pub pouch_unloaded: Vec<String>,
    pub loadout_error: Option<String>,
}

/// U2.2-S2 (FB-12): the one unit builder; every unit (hero, NPC, named enemy, Dojo and sim unit) is built here. The
/// steps, in order: the row (`build_catalog_unit`); the deck (a static unit under the rule: its row deck's slotted
/// passives, then the open basics and active slots; a growth unit: its own deck, filled at the end); the named items
/// worn; the passives once each in gather order; a growth unit's learned skills, worn gear (the two-handed rule), pouch,
/// vitals, stored loadout, deck fill and ranks (a static unit re-derives its vitals when named items geared a derived
/// unit); the tuning last. `over.path_cat` replaces `cat` when set.
pub fn build_from(cat: &Catalog, b: UnitBuild) -> BuiltUnit {
    let cat: &Catalog = b.over.path_cat.as_deref().unwrap_or(cat);
    let e = b.row;
    let mut u = build_catalog_unit(cat, e, b.default_ap, b.wield);
    u.deck = e.deck.clone();
    match (&b.growth, b.deck) {
        (None, Some(rule)) => {
            // CLASS-BASICS: the class's role basics are open beside the ruleset's (no slot)
            u.deck_rule = Some(rule.with_class(cat, &e.class_id));
            if u.apply_deck_passives(cat) > 0 && e.stat_mode == "derived" && class_for(cat, e).is_some() {
                u.sync_from_stats_and_gear(cat);
                u.hp = u.max_hp;
            }
            u.apply_deck(cat);
        }
        (Some(g), Some(rule)) => {
            // RULESET1 9 D3-D5: the member's deck replaces its row's; the rule gates the passives below
            if !g.deck.is_empty() {
                u.deck = g.deck.clone();
            }
            u.deck_rule = Some(rule.with_class(cat, &e.class_id));
        }
        (_, None) => {}
    }
    let mut geared = false;
    for item in b.over.named_items.iter().filter(|i| i.is_equipment()) {
        u.inventory.equip(Equipment::new(item), Slot::parse(&item.slot).unwrap_or(Slot::MainHand));
        geared = true;
    }
    // PK2-S2: a member's body before its passives (a keystone graft takes a keystone slot from the slotted passives)
    let body: Vec<(String, Vec<Grant>)> = match &b.growth {
        Some(g) if !cat.body_slots.is_empty() => {
            u.graft_count = Some(g.body.len() as i32);
            u.floor_offset = g.floor_offset;
            let held: Vec<(String, Vec<Grant>, bool)> = g.body.iter().filter_map(|i| i.graft.as_ref().map(|s| (i.id.clone(), grants::read(&s.effects), s.keystone))).collect();
            for (_, list, _) in held.iter().filter(|(_, _, keystone)| *keystone) {
                for g in list {
                    if let Grant::GrantSkill(SkillGrant::Learn(skill)) = g {
                        u.keystone_skills.push(skill.clone());
                    }
                }
            }
            held.into_iter().map(|(id, list, _)| (id, list)).collect()
        }
        _ => Vec::new(),
    };
    // PK2-S4: a member's mastery, and the wielded family's keystone before its passives (it takes a keystone slot)
    if let Some(g) = &b.growth {
        // a member without mastery (and no masterwork) holds none, so its unit reads as before
        u.mastery = (!g.mastery.is_empty() || g.masterwork).then(|| g.mastery.clone());
        u.masterwork = g.masterwork;
        u.mastery_keystone = mastery_keystone(&u, g);
        if !u.mastery_keystone.is_empty() {
            u.keystone_skills.push(u.mastery_keystone.clone());
        }
    }
    for id in &b.over.passives {
        if let Some(p) = cat.passives.get(id) {
            u.apply_passive(cat, p);
        }
    }
    // PK2-S2: the body items' effects as a passive's (`body:<item>`; no member grant), after the passives
    for (id, list) in body {
        let list: Vec<Grant> = list.into_iter().filter(|g| !matches!(g, Grant::ForbidHands { .. } | Grant::PierceMelee)).collect();
        u.apply_grants(cat, &list, &format!("body:{id}"), false);
    }
    let (mut pouch_unloaded, mut loadout_error) = (Vec::new(), None);
    match &b.growth {
        Some(g) => {
            // PK2-S1: a member holds its tracks (empty: on none yet) once the catalog has a track row, and its max HP
            // offset
            u.tracks = (!cat.unit_tracks.is_empty()).then(|| g.tracks.clone());
            u.max_hp_offset = g.max_hp_offset;
            // PK2-S3: the trained proficiency levels, before the gear is read
            for (gear, levels) in &g.trained {
                u.stats.proficiency.add_bonus(gear, *levels);
            }
            for sk in &g.learned_skills {
                u.learn_skill_checked(cat, sk, false);
            }
            for (slot, item) in g.equipment.iter().filter(|(_, i)| i.is_equipment()) {
                u.inventory.unequip(*slot);
                u.inventory.equip(Equipment::new(item), *slot);
            }
            // The two-handed rule: a class default shield or focus is not held under a two-handed weapon.
            if let (Some(main), Some(off)) = (u.inventory.get(Slot::MainHand), u.inventory.get(Slot::OffHand)) {
                if !crate::gear::off_hand_ok(Some(&main.item), &off.item) {
                    u.inventory.unequip(Slot::OffHand);
                }
            }
            u.inventory.pouch.clear();
            u.inventory.pouch_stacks = g.pouch_stacks;
            for item in g.pouch.iter().filter(|i| i.item_type == "CONSUMABLE") {
                if !u.inventory.add_to_pouch(item) {
                    pouch_unloaded.push(item.id.clone());
                }
            }
            u.sync_from_stats_and_gear(cat);
            u.hp = u.max_hp;
            if let Some((class_fams, extra_fams)) = &g.loadout {
                if let Err(err) = u.set_loadout(cat, class_fams.clone(), extra_fams.clone()) {
                    loadout_error = Some(err);
                }
            }
            // RULESET1 9 D1, D2: the deck (else the loadout, auto) fills the active slots
            u.apply_deck(cat);
            // SKILLUI1 R4: the ranks chosen on the Skills tab (after gear, passives and the loadout: every unlock final)
            u.rank_choice = Some(g.ranks.clone());
        }
        None if geared && e.stat_mode == "derived" => {
            u.sync_from_stats_and_gear(cat);
            u.hp = u.max_hp;
        }
        None => {}
    }
    apply_tuning(&mut u, &b.over.tuning);
    BuiltUnit { unit: u, pouch_unloaded, loadout_error }
}

/// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5, the standard-path keystone): the keystone member `g` holds ("" = none): the
/// main-hand weapon family's keystone (the member's saved main hand, else its row's) at mastery 3 once its forge rite is
/// done, and only while no keystone graft holds the keystone (a unit holds one keystone, of either path).
fn mastery_keystone(u: &Unit, g: &Growth) -> String {
    if !g.forge_rite || !u.keystone_skills.is_empty() {
        return String::new();
    }
    let saved = g.equipment.iter().find(|(s, _)| *s == Slot::MainHand).map(|(_, i)| i);
    let Some(main) = saved.or_else(|| u.inventory.get(Slot::MainHand).map(|e| &e.item)) else { return String::new() };
    let level = g.mastery.get(&main.weapon_type).copied().unwrap_or(0);
    if main.item_type != "WEAPON" || level < rpgkit_data::mastery::MASTERY_MAX {
        return String::new();
    }
    main.family_keystone.clone()
}

/// WorldFactory.apply_character's tuning: max HP by `hp_pct` percent, then power, armour, initiative (key `speed`) and AP
/// deltas.
fn apply_tuning(u: &mut Unit, t: &Tuning) {
    if t.hp_pct != 0 {
        u.max_hp = (roundi(u.max_hp as f64 * (1.0 + t.hp_pct as f64 / 100.0))).max(1);
        u.hp = u.max_hp;
    }
    if t.power != 0 {
        u.power_override = (crate::damage::power_of(u, u.attack_range > 1, false) + t.power).max(1);
    }
    if t.armor != 0 {
        u.armor_override = (crate::damage::armor_rating(u) + t.armor).max(0);
    }
    if t.initiative != 0 {
        u.initiative = (u.initiative + t.initiative).max(1);
    }
    if t.ap != 0 {
        u.max_ap = (u.max_ap + t.ap).max(1);
        u.ap = u.max_ap;
    }
}

/// HOOKS1 DU1: the active slots catalog unit `e` fills with an automatic deck (its authored deck ignored) under `rule`:
/// the battle's own build (`build_unit_in`, `loadout_of`, `apply_deck`), so the content check reads the run time's list.
pub fn auto_actives(cat: &Catalog, e: &UnitDef, rule: &DeckRule) -> Vec<String> {
    let mut auto = e.clone();
    auto.deck = Deck::default();
    build_unit_in(cat, &auto, 1, Some(rule)).active_families()
}

fn build_catalog_unit(cat: &Catalog, e: &UnitDef, default_ap: i32, wield: crate::stats::WieldRule) -> Unit {
    let arch = class_for(cat, e);
    let derived = e.stat_mode == "derived" && arch.is_some();
    // KITIDS1: a unit without a class takes the stats of the catalog's first class at level 1 (a blank class when the
    // catalog has none); the kit names no class id. U2.2-S2 (FB-23): a classless unit is a content error, so shipped
    // data never reaches this; it stays for catalogs frozen before (the goldens' fixture catalog)
    let classless = || cat.classes.values().next().cloned().unwrap_or_else(|| ClassDef::from_obj(&Default::default()));
    let mut u = if derived {
        let arch = arch.unwrap();
        // R4-1: a templated unit reads the template's shares (the class keeps its gear, path and loadout), and its
        // stats at level (spend included) are scaled by its tier's factor `tier_mult`, floor 6 (slice 2, FB-144)
        let template = template_for(cat, e, arch);
        let basis = template.map(|t| t.over(arch));
        let src = basis.as_ref().unwrap_or(arch);
        let spend = if e.spend.is_empty() { stats::auto_spend(&src.unit_shares().growth, stats::spend_pool(e.level)) } else { e.spend.clone() };
        let mut cs = CharacterStats::new(src, e.level, &spend);
        if template.is_some() {
            cs.scale_core(stats::tier_row(&e.tier).mult);
        }
        let mut u = Unit::blank(&e.name, 1, 1, 1, cs);
        u.tier_scaled = template.is_some();
        u
    } else if let Some(arch) = arch {
        Unit::blank(&e.name, e.hp, e.initiative, e.attack_range, CharacterStats::new(arch, e.level, &IndexMap::new()))
    } else {
        let mut u = Unit::blank(&e.name, e.hp, e.initiative, e.attack_range, CharacterStats::new(&classless(), 1, &IndexMap::new()));
        // U3.2-S3 (FB-33): no class, no gear restriction (the borrowed class's training does not bind it)
        u.stats.proficiency.unrestricted = true;
        u
    };
    // U3.2-S3: the gear is read under the battle's wield terms from the first sync on
    u.stats.proficiency.wield = wield;
    equip(cat, &mut u, e, arch);
    if derived {
        let arch = arch.unwrap();
        u.tier = e.tier.clone();
        u.tier_mult = stats::tier_row(&u.tier).mult;
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

/// PR5-26 (audit B2): the item id slot `slot_key` of row `e` holds before any member gear: the row's own id, its
/// class's default for "" (`arch`), None for `NO_ITEM` or nothing.
fn slot_default(e: &UnitDef, arch: Option<&ClassDef>, slot_key: &str) -> Option<String> {
    let item_id = e.slots.get(slot_key)?;
    if item_id == NO_ITEM {
        return None;
    }
    let id = if item_id.is_empty() { arch.and_then(|a| a.loadout.get(slot_key).cloned()).unwrap_or_default() } else { item_id.clone() };
    (!id.is_empty()).then_some(id)
}

/// PR5-26 (audit B2): the item id slot `slot_key` of catalog row `e` gives a unit built from it (its own or its class's
/// default; None: empty), as `build_from` equips it before a member's own gear.
pub fn row_slot_item(cat: &Catalog, e: &UnitDef, slot_key: &str) -> Option<String> {
    slot_default(e, class_for(cat, e), slot_key)
}

/// The class's default gear with the unit's per-slot overrides, then the pouch.
fn equip(cat: &Catalog, u: &mut Unit, e: &UnitDef, arch: Option<&ClassDef>) {
    for slot_key in e.slots.keys() {
        let Some(id) = slot_default(e, arch, slot_key) else { continue };
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

    /// Combatant.take_damage: `mitigated` false applies the unit's guard first. Returns the damage dealt. A2.2-S1
    /// (FB-43): through the pipeline (`damage::take_damage`; floor 1 only when the guard applies, as before).
    pub fn take_damage(&mut self, amount: i32, mitigated: bool) -> i32 {
        let dmg = crate::damage::take_damage(amount, self.guard, mitigated);
        self.hp = (self.hp - dmg).max(0);
        dmg
    }

    /// Combatant.heal. A2.2-S1: through the pipeline (`damage::heal_gain`; capped at max HP, as before).
    pub fn heal(&mut self, amount: i32) {
        if self.is_alive() {
            self.hp += crate::damage::heal_gain(self.hp, self.max_hp, amount);
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

    /// `1 - capped magnitude / 100` per active Attack Power debuff (weaken), 1.0 with none; W2-5 (C09): the statuses
    /// are the table's `modifier` rows with `rating` attack_power and `kind` debuff.
    /// PK1-KITFIX (b): 1.0 while `attack_power` is held (`empower_factor` carries the held value).
    pub fn weaken_factor(&self) -> f64 {
        if self.rating_set(crate::status::Rating::AttackPower).is_some() {
            return 1.0;
        }
        let lowers = |s: &str| crate::status::lowers_rating(s, crate::status::Rating::AttackPower);
        self.st.active.iter().filter(|s| lowers(s)).fold(1.0, |f, s| f * (1.0 - self.capped_magnitude(s) as f64 / 100.0))
    }

    /// `1 + capped magnitude / 100` per active Attack Power buff (empower), 1.0 with none; W2-5 (C09): the statuses
    /// are the table's `modifier` rows with `rating` attack_power and `kind` buff.
    /// PK1-KITFIX (b): a held `attack_power` is its value / 100 instead (so weaken x empower is the held factor).
    pub fn empower_factor(&self) -> f64 {
        if let Some(v) = self.rating_set(crate::status::Rating::AttackPower) {
            return f64::from(v.max(0)) / 100.0;
        }
        let raises = |s: &str| crate::status::raises_rating(s, crate::status::Rating::AttackPower);
        self.st.active.iter().filter(|s| raises(s)).fold(1.0, |f, s| f * (1.0 + self.capped_magnitude(s) as f64 / 100.0))
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

    /// U4.2-S4 (FB-30): the sources of `potency_for_skill`, in its sum order (the family's passive adds, its worn ones,
    /// then per group of the skill the passive adds and the worn ones); each part is one grant's add (a 0 add, which
    /// changes nothing, names no part).
    pub fn potency_parts(&self, skill: &rpgkit_data::SkillDef) -> Vec<crate::pipeline::Part> {
        let fam = skill.family_id();
        let groups: Vec<&str> = GROUPS.iter().copied().filter(|g| skill_in_group(skill, g)).collect();
        let of = |list: &[HitSource], set: &dyn Fn(&SkillSet) -> bool| -> Vec<crate::pipeline::Part> {
            list.iter()
                .filter(|h| h.value != 0.0 && matches!(&h.target, Target::SkillPotency(x) if set(x)))
                .map(|h| crate::pipeline::Part { source: h.source.clone().into(), value: h.value })
                .collect()
        };
        let family = |x: &SkillSet| matches!(x, SkillSet::Family(f) if f == fam);
        let mut out = of(&self.hit_sources, &family);
        out.extend(of(&self.gear_hit_sources, &family));
        for g in groups {
            let group = |x: &SkillSet| matches!(x, SkillSet::Group(y) if y == g);
            out.extend(of(&self.hit_sources, &group));
            out.extend(of(&self.gear_hit_sources, &group));
        }
        out
    }

    /// U4.2-S4 (FB-30): the skill potency as the hit's mult-stage modifier `potency`: the value `potency_for_skill` folds
    /// (1 + the sum, FB-26), the parts its sources.
    pub fn potency_modifier(&self, skill: &rpgkit_data::SkillDef) -> crate::pipeline::Modifier {
        use crate::pipeline::{Modifier as Stage1, Op, Stage};
        Stage1::new("potency", Stage::Mult, Op::Mul(self.potency_for_skill(skill))).with_parts(self.potency_parts(skill))
    }

    /// U4.2-S4 (FB-30): the target's `stagger_taken` effects that match an action `blocks` long as the stagger hit's
    /// mult-stage modifier `passive`: their product (FB-26, as `poise::factors` reads it), the parts its sources.
    pub fn stagger_taken_modifier(&self, blocks: i32) -> crate::pipeline::Modifier {
        use crate::pipeline::{Modifier as Stage1, Op, Part, Stage};
        let product: f64 = self.stagger_taken.iter().filter(|e| e.matches(blocks)).map(|e| e.mult).product();
        let parts = self
            .hit_sources
            .iter()
            .filter(|h| matches!(h.target, Target::StaggerTaken { max_blocks } if StaggerTaken { mult: h.value, max_blocks }.matches(blocks)))
            .map(|h| Part { source: h.source.clone().into(), value: h.value })
            .collect();
        Stage1::new("passive", Stage::Mult, Op::Mul(product)).with_parts(parts)
    }

    /// U4.2-S4 (FB-30): the sources of `forced_traits().bonus_tiles` (passives first, then worn items; each a row's tiles;
    /// an immunity-only row, 0 tiles, names no part).
    pub fn forced_tile_parts(&self) -> Vec<crate::pipeline::Part> {
        self.hit_sources
            .iter()
            .chain(&self.gear_hit_sources)
            .filter(|h| h.target == Target::ForcedTiles && h.value != 0.0)
            .map(|h| crate::pipeline::Part { source: h.source.clone().into(), value: h.value })
            .collect()
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
    /// AUDIT-09: "main" is the ready weapon (`Inventory::active_weapon`: the off-hand backup while it is ready in a 2H
    /// pair); a 2H pair shares one ready slot, so its "off" hand is empty either way.
    pub fn wielded_type(&self, hand: &str) -> String {
        let main = self.inventory.main_weapon();
        let weapon = if hand == "off" {
            if main.is_some_and(|m| m.item.two_handed) {
                None
            } else {
                self.inventory.offhand_weapon()
            }
        } else {
            self.inventory.active_weapon()
        };
        weapon.map(|w| w.weapon_type().to_string()).unwrap_or_else(|| "unarmed".into())
    }

    pub fn skill_unlock_level(&self, cat: &Catalog, skill: &rpgkit_data::SkillDef) -> i32 {
        stats::MAX_LEVEL.min(self.family_unlock_level(cat, skill.family_id()) + skill.tier_offset())
    }

    /// The level gate of can_use: basics (under the deck rule the ruleset's basics, D1), engine skills, meter skills
    /// (HOOKS1 UM6) and hand-built units are never gated.
    pub fn tier_unlocked(&self, cat: &Catalog, skill: &rpgkit_data::SkillDef) -> bool {
        if self.path_levels.is_empty() || skill.system || skill.basic || self.meter_skill(&skill.id) || self.gear_skill(&skill.id) || self.technique_skill(&skill.id) || self.deck_basic(cat, &skill.id) == Some(true) {
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
