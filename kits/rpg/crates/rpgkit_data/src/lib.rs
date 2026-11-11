//! Typed content catalog (classes, units, skills, items, passives, encounters, terrain, and the engine-only derived
//! stats of DERIVE1 and ruleset rows of RULESET1), and the campaign vocabulary of CAMPAIGN1 (kinds, effect kinds, read
//! keys, the manifest blocks with their defaults) and the story vocabulary of STORYUI1 (kinds, card and step keys,
//! story effects, limits, the step reader), and the portrait cue rows and vocabulary of PORTRAITFX1 (triggers, events,
//! effects, surfaces, expressions), and the combat hook rows and vocabulary of HOOKS1 (the hazards, unit meters and
//! zones kinds; forced moves, hazard placements, trigger rows, riders and meter costs on skills; events, effects,
//! tactics), and the term register of TERMS1 (`terms`: neutral term ids with today's texts as defaults).
//!
//! Source: the resolved catalog the game's loader builds (`{kind: [entry, ...]}` after item-family resolution and
//! schema normalisation; `Catalog::from_value`). Defaults below are the `ContentSchema` field defaults, so
//! hand-written partial entries normalise the same way.

use aegis_core::formula::Formula;
use aegis_core::value::*;
use indexmap::IndexMap;
use serde_json::Value;

pub mod dojo_preset;
pub mod terms;
pub mod unit_budget;
pub mod unit_template;
pub mod unit_track;
pub use dojo_preset::{DojoPresetDef, DOJO_PASSIVES_MAX, DOJO_POUCH_MAX, DOJO_PRESETS_KIND};
pub use terms::{Term, TermDef, TermRow, TermScope, TERMS_KIND};
pub use unit_budget::ClassShares;
pub use unit_template::{UnitTemplate, TEMPLATE_CLASS, TEMPLATE_FIELD, UNIT_TEMPLATES_KIND};
pub use unit_track::{TrackStage, UnitTrackDef, TRACK_DEPARTURES, TRACK_STAGES_MAX, TRACK_STAGE_KEYS, UNIT_TRACKS_KIND};

pub const STATS: [&str; 6] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"];
pub const LOADOUT_SLOTS: [&str; 6] = ["main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2"];
/// Unit slot value meaning "leave this slot empty" (no class fallback).
pub const NO_ITEM: &str = "none";
/// AUDIT-01 (FB-143): the tier of a unit row without `tier` (the schema default). Every reader of a unit row's tier
/// (`UnitDef`, the world roster's stage level, XP weight and morale tier, the loot check, the sim) goes through
/// `unit_tier`, so a tierless row reads the same tier everywhere.
pub const DEFAULT_TIER: &str = "hero";

/// AUDIT-01: unit row `o`'s tier, `DEFAULT_TIER` when the row has none.
pub fn unit_tier(o: &Obj) -> String {
    string(o, "tier", DEFAULT_TIER)
}

/// INPUT1 MR1-MR2 (rule round_intent): the spot a Maneuver attack with a `position` dashes to by its target and hits
/// from: `flank` (beside it) or `behind`. The skill field's "" is no position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Flank,
    Behind,
}

impl Position {
    /// The field values (`""` is none).
    pub const IDS: [&'static str; 2] = ["flank", "behind"];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "flank" => Some(Position::Flank),
            "behind" => Some(Position::Behind),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Position::Flank => "flank",
            Position::Behind => "behind",
        }
    }
}

/// A3.2-K2 (INTENT2 IR9, A3.1): which units an area shape hits. The skill field `affects`; its `""` takes the row's
/// default (`rpgkit_battle::skill::affects_by`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Affects {
    /// Every unit on the shape's tiles but the user (the classic area, "hits allies too").
    All,
    /// The user's foes only.
    Enemies,
    /// The user's friends only (never the user).
    Allies,
}

impl Affects {
    /// The field values (`""` is the row's default).
    pub const IDS: [&'static str; 3] = ["all", "enemies", "allies"];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "all" => Some(Affects::All),
            "enemies" => Some(Affects::Enemies),
            "allies" => Some(Affects::Allies),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Affects::All => "all",
            Affects::Enemies => "enemies",
            Affects::Allies => "allies",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfEntry {
    pub level: i32,
    pub scaling: f64,
}

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub id: String,
    pub name: String,
    pub art: String,
    pub ai_role: String,
    /// Level 1 stats. On a row given as shares (`base_shares`), the computed view `unit_budget::stats_at(level 1)`.
    pub base_stats: IndexMap<String, i32>,
    /// Points per level. On a row given as shares, the computed view of level 2's gain (`unit_budget::gain_at`).
    pub growth: IndexMap<String, i32>,
    /// U2.2-S1: the row's own `base_shares` and `growth_shares` (None on a row given as `base_stats` and `growth`).
    pub shares: Option<ClassShares>,
    pub proficiency: IndexMap<String, ProfEntry>,
    /// Family head id -> unlock level, in entry order.
    pub skill_path: IndexMap<String, i32>,
    /// CLASS-BASICS (user 2026-11-08): the class's role basics, skill ids open to every unit of the class without a deck
    /// slot under the rule `deck` (after the ruleset's basics; `DeckRule::with_class`). Empty = none.
    pub basics: Vec<String>,
    /// Slot key (`main_hand` ...) -> item id; empty slots left out.
    pub loadout: IndexMap<String, String>,
    pub pouch: Vec<String>,
}

impl ClassDef {
    pub fn from_obj(o: &Obj) -> Self {
        let proficiency = obj(o, "proficiency")
            .map(|m| {
                m.iter()
                    .map(|(k, v)| {
                        let e = v.as_object().cloned().unwrap_or_default();
                        (k.clone(), ProfEntry { level: int(&e, "level", 1), scaling: float(&e, "scaling", 1.5) })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let loadout = LOADOUT_SLOTS
            .iter()
            .filter_map(|s| {
                let id = string(o, s, "");
                (!id.is_empty()).then(|| (s.to_string(), id))
            })
            .collect();
        let id = string(o, "id", "");
        // U2.2-S1: a row with base_shares is read as shares and base_stats/growth become a computed view
        let shares = obj(o, "base_shares").filter(|m| !m.is_empty()).map(|base| ClassShares {
            base: float_map(base),
            growth: obj(o, "growth_shares").map(float_map).unwrap_or_default(),
        });
        let (base_stats, growth) = match &shares {
            Some(sh) => (unit_budget::stats_at(sh, 1), unit_budget::gain_at(sh, 2)),
            None => (int_map(o, "base_stats"), int_map(o, "growth")),
        };
        Self {
            // KITIDS1: without an art key the class wears the body named like itself (the kit names no class)
            art: string(o, "art", &id),
            id,
            name: string(o, "name", "New Class"),
            ai_role: string(o, "ai_role", "AGGRESSOR"),
            base_stats,
            growth,
            shares,
            proficiency,
            skill_path: int_map(o, "skill_path"),
            basics: strings(o, "basics"),
            loadout,
            pouch: strings(o, "pouch"),
        }
    }

    /// The class's share vectors: its own (a share row), or today's integer row read as shares (base / 62, growth / 5).
    pub fn unit_shares(&self) -> ClassShares {
        self.shares.clone().unwrap_or_else(|| ClassShares {
            base: unit_budget::shares_of(&self.base_stats, unit_budget::BASE_BUDGET),
            growth: unit_budget::shares_of(&self.growth, unit_budget::GROWTH_BUDGET),
        })
    }

    /// The class stats at `level` before the free spend: a share row by the unit budget's running total; an integer
    /// row as base + growth per level gained (a missing base stat is 10), as before U2.2.
    pub fn stats_at(&self, level: i32) -> IndexMap<String, i32> {
        if let Some(sh) = &self.shares {
            return unit_budget::stats_at(sh, level);
        }
        let gained = level.max(1) - 1;
        STATS
            .iter()
            .map(|s| {
                let v = self.base_stats.get(*s).copied().unwrap_or(10) + self.growth.get(*s).copied().unwrap_or(0) * gained;
                (s.to_string(), v)
            })
            .collect()
    }

    /// The path as ordered steps (level, family): unlock level ascending, then entry position.
    pub fn path_steps(&self) -> Vec<(i32, String)> {
        let mut rows: Vec<(i32, usize, String)> =
            self.skill_path.iter().enumerate().map(|(i, (k, &lvl))| (lvl, i, k.clone())).collect();
        rows.sort_by_key(|r| (r.0, r.1));
        rows.into_iter().map(|(l, _, s)| (l, s)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct UnitDef {
    pub id: String,
    pub name: String,
    /// The body key (ASSETORG2: `<class>` or `<class>/<variant>`; "" = the class's).
    pub art: String,
    /// The palette the body wears (ASSETORG2; "" = none).
    pub palette: String,
    pub side: String,
    pub class_id: String,
    pub level: i32,
    /// W.2-S3 (FB-139): the levels an enemy gains per world stage (`rpgkit_world::battle::foe_level`; 0 = none).
    pub growth_per_stage: i32,
    pub tier: String,
    pub spend: IndexMap<String, i32>,
    pub stat_mode: String,
    pub hp: i32,
    /// A fixed-stat row's initiative (the row key `speed`, kept; U1.2-S3c).
    pub initiative: i32,
    pub attack_range: i32,
    pub ap: i32,
    pub power: i32,
    pub armor: i32,
    pub skill_cap: i32,
    pub extra_skills: Vec<String>,
    pub loadout: Vec<String>,
    pub ai_role: String,
    /// R4-1 slice 2 (`U2.3-TIERS`, FB-144): the role template the unit takes (`rpgkit_rules::unit::template_for`):
    /// "" = by its AI role, `TEMPLATE_CLASS` = none (it keeps its class's shares), else a template role or id.
    pub template: String,
    pub target_pref: String,
    pub flee_ratio: f64,
    pub defend_below: f64,
    pub prefer_level: i32,
    pub status_resist: IndexMap<String, i32>,
    pub reckless: bool,
    pub heal_below: f64,
    /// Slot key -> item id as written ("" = the class's item, `NO_ITEM` = empty).
    pub slots: IndexMap<String, String>,
    pub pouch: Vec<String>,
    /// RULESET1 9 D5 (timeline, rule deck): the slotted actives, passives, keystones and per-intent order (empty = auto).
    pub deck: Deck,
    /// RULESET1 5 P2 (rule poise): -1 = the derived Max Poise, 0 = no poise bar, above 0 = a fixed Max Poise.
    pub poise: i32,
    /// RULESET1 TQ23 (a) (rule round_intent): the round intent the unit starts a battle in ("" = none).
    pub default_intent: String,
    /// HOOKS1 AT2 (rule ai_tactics): the unit's AI tactics (TACTICS ids, in order).
    pub tactics: Vec<String>,
    /// LAYER-DOWN-2: the unit's part in the gear Dojo (`UNIT_DOJO_KEY`: `UNIT_DOJO_HERO`, `UNIT_DOJO_DUMMY`, "" = none).
    pub dojo: String,
}

/// LAYER-DOWN-2: the unit row key naming the gear Dojo's cast (the heroes in catalog order, the first dummy).
pub const UNIT_DOJO_KEY: &str = "dojo";
/// LAYER-DOWN-2: a Dojo hero (`UNIT_DOJO_KEY`).
pub const UNIT_DOJO_HERO: &str = "hero";
/// LAYER-DOWN-2: the Dojo's training dummy (`UNIT_DOJO_KEY`).
pub const UNIT_DOJO_DUMMY: &str = "dummy";

/// A unit's or party member's deck (RULESET1 9 D5, field `deck`): skill families in the active slots, passive ids in
/// the passive and keystone slots, and per round intent the families in the order the planner fills them. Missing or
/// empty keys mean auto.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Deck {
    pub active: Vec<String>,
    pub passive: Vec<String>,
    pub keystone: Vec<String>,
    /// Round intent -> skill families, in the order written.
    pub order: IndexMap<String, Vec<String>>,
    /// INPUT1 FL1 (`follow_ups`): opener family -> the families the planner's fill tries first after it, in the
    /// player's priority (written order kept; an empty list is no list).
    pub follow_ups: IndexMap<String, Vec<String>>,
    /// PK1-S2a (`channeled`, PK1 4.2, rule channel): the actives whose channeled version opens in a Channel round (written
    /// order; at most the ruleset's `channeled_actives` of them are read).
    pub channeled: Vec<String>,
}

impl Deck {
    /// The deck keys, in their stored order.
    pub const KEYS: [&'static str; 6] = ["active", "passive", "keystone", "order", "follow_ups", "channeled"];

    /// From a `deck` value (anything but an object is the empty deck; lists keep their strings; an empty
    /// `follow_ups` list is dropped, FL1).
    pub fn from_value(v: &Value) -> Self {
        let Some(o) = v.as_object() else { return Self::default() };
        let order = obj(o, "order").map(|m| m.keys().map(|k| (k.clone(), strings(m, k))).collect()).unwrap_or_default();
        let follow_ups = obj(o, "follow_ups")
            .map(|m| m.keys().map(|k| (k.clone(), strings(m, k))).filter(|(_, l)| !l.is_empty()).collect())
            .unwrap_or_default();
        Self { active: strings(o, "active"), passive: strings(o, "passive"), keystone: strings(o, "keystone"), order, follow_ups, channeled: strings(o, "channeled") }
    }

    /// True when every key is empty (the auto deck).
    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
            && self.passive.is_empty()
            && self.keystone.is_empty()
            && self.order.values().all(Vec::is_empty)
            && self.follow_ups.values().all(Vec::is_empty)
            && self.channeled.is_empty()
    }

    /// INPUT1 FL1: the follow-up list set for opener family `fam` (None: no list, the fill reads the order).
    pub fn follow_ups_of(&self, fam: &str) -> Option<&Vec<String>> {
        self.follow_ups.get(fam).filter(|l| !l.is_empty())
    }
}

impl UnitDef {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", "New Unit"),
            art: string(o, "art", ""),
            palette: string(o, "palette", ""),
            side: string(o, "side", "enemy"),
            class_id: string(o, "class_id", ""),
            level: int(o, "level", 1),
            growth_per_stage: int(o, "growth_per_stage", 0),
            tier: unit_tier(o),
            spend: int_map(o, "spend"),
            stat_mode: string(o, "stat_mode", "derived"),
            hp: int(o, "hp", 50),
            initiative: int(o, "speed", 12),
            attack_range: int(o, "attack_range", 1),
            ap: int(o, "ap", 0),
            power: int(o, "power", 0),
            armor: int(o, "armor", -1),
            skill_cap: int(o, "skill_cap", 0),
            extra_skills: strings(o, "extra_skills"),
            loadout: strings(o, "loadout"),
            ai_role: string(o, "ai_role", ""),
            template: string(o, "template", ""),
            target_pref: string(o, "target_pref", "ROLE"),
            flee_ratio: float(o, "flee_ratio", 0.25),
            defend_below: float(o, "defend_below", 0.4),
            prefer_level: int(o, "prefer_level", 0),
            status_resist: int_map(o, "status_resist"),
            reckless: boolean(o, "reckless", false),
            heal_below: float(o, "heal_below", 0.7),
            slots: LOADOUT_SLOTS.iter().map(|s| (s.to_string(), string(o, s, ""))).collect(),
            pouch: strings(o, "pouch"),
            deck: Deck::from_value(o.get("deck").unwrap_or(&Value::Null)),
            poise: int(o, "poise", -1),
            default_intent: string(o, "default_intent", ""),
            tactics: strings(o, "tactics"),
            dojo: string(o, UNIT_DOJO_KEY, ""),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub level: i32,
    pub blocks: i32,
    pub system: bool,
    pub basic: bool,
    pub learned_only: bool,
    pub grade: String,
    pub requires: Vec<String>,
    pub stance: String,
    pub unlock_offset: i32,
    pub weapon_types: Vec<String>,
    /// U3.2-S2: one of `weapon_types` is a family with the engine rule `spell_source` (the resolved row's
    /// `SKILL_SPELL_SOURCE` key; a row without it reads `LEGACY_FAMILY_RULES`).
    pub spell_source: bool,
    pub min_proficiency: i32,
    pub reach_min: i32,
    pub reach_max: i32,
    pub area: String,
    /// A3.2-K2 (IR9): the units the area hits as written; None (`""`) = the row's default.
    pub affects: Option<Affects>,
    pub power_mult: f64,
    pub weapon_hand: String,
    pub dash_tiles: i32,
    /// INPUT1 MR1 (rule round_intent): the position a Maneuver attack dashes to (its `reach_max` is how far) and hits
    /// from; None = it hits from where it stands.
    pub position: Option<Position>,
    pub status_effect: String,
    pub status_potency: i32,
    pub status_magnitude: i32,
    pub status_rounds: i32,
    pub target_mode: String,
    pub heal_stat: String,
    pub heal_base: i32,
    pub heal_mult: f64,
    /// HEALPCT1 (FB-177): a heal of this percent of the target's max HP, added after the heal formula (0 = none).
    pub heal_pct: i32,
    pub grant_status: String,
    pub extra_grants: Vec<Value>,
    pub cleanse_statuses: Vec<String>,
    /// DEFEND: the guard fraction (0 = none); S1.2-S4: a row without the key takes the ruleset default for its
    /// `guard_rounds` (`rpgkit_battle::skill::guard_mitigation`).
    pub mitigation: f64,
    /// S1.2-S4 (DEFEND): the rounds the guard lasts (1 = this round, from the wind-up start to the round's end); 2 or
    /// more queues the `guard` status for the later rounds.
    pub guard_rounds: i32,
    pub boost: f64,
    pub boost_rounds: i32,
    pub jump_tiles: i32,
    pub recover_ap: i32,
    pub ends_in_stance: String,
    pub disrupt: String,
    pub exploit: Vec<Value>,
    pub exploit_consume: bool,
    /// RULESET1_INTENT IN4 (rule round_intent): the skill's round intent as written ("" = by kind: `rpgkit_rules::ruleset::kind_intent`).
    pub round_intent: String,
    /// IN6: what the skill grants its own user is intent-bound.
    pub intent_buff: bool,
    /// RULESET1 T10 (rule ap_cost): the AP base instead of the level; -1 = unset.
    pub ap_cost: i32,
    /// RULESET1 T1 (rule enemy_moves): blocks an enemy-side unit spends after the impact (0-2).
    pub recovery_blocks: i32,
    /// RULESET1 P4 (rule poise): poise taken from each unit the hit damages.
    pub stagger: i32,
    /// P11: poise the user gains while the action winds up.
    pub windup_poise: i32,
    /// P12: the user's poise cannot drop below 1 while this action winds up.
    pub unbreakable: bool,
    /// P14: a hit landing in a windup breaks the target.
    pub sure_break: bool,
    /// RULESET1 A2 (rule pierce): the hit skips the armour term.
    pub armor_piercing: bool,
    /// HOOKS1 FM1 (rule forced_move): the push or pull of each unit the skill damaged (None = no forced move).
    pub forced_move: Option<ForcedMove>,
    /// HOOKS1 GH2 (rule ground_hazards): the hazard the skill leaves (None = none).
    pub hazard: Option<HazardPlace>,
    /// HOOKS1 8.1 (rule triggers): the triggers held while the skill's command runs (start to impact).
    pub triggers: Vec<TriggerRow>,
    /// HOOKS1 RD1 (rule triggers): the riders run per unit the skill landed on.
    pub riders: Vec<Rider>,
    /// HOOKS1 UM2 (rule unit_meters): the meter changes paid at the command's start.
    pub meters: Vec<MeterDelta>,
    /// HOOKS1 HC1 (rule hp_cost): HP paid at the command's start (0 = none).
    pub hp_cost: i32,
    /// HOOKS1 IG1 (rule ignore_guard): the hits skip the guard term and P6's guard factor.
    pub ignore_guard: bool,
    /// PK1-S2a (rule hp_cost): HP paid at the command's start as a percent of max HP (0 = none; beside `hp_cost`).
    pub hp_cost_pct: i32,
    /// PK1-S2a (rule unit_meters): the meter value the skill needs at its start (None = no gate).
    pub requires_meter: Option<MeterGate>,
    /// PK1-S2a (rule channel): the skill's channeled version block (the default: a derived version for a single-intent skill).
    pub channel: ChannelSpec,
    /// PK2-S1 (PK1 3.7, FB-174 (6)): a SUPPORT skill's `unmark` `{mark}`, the consumable's effect as a skill: the mark id
    /// whose latest entry on the target this battle it removes ("" = none).
    pub unmark: String,
    /// PK2-S1: how many times a unit may use the skill in one battle (0 = no limit).
    pub uses_per_battle: i32,
    /// Family head id; "" when the skill heads its own family (ContentFactory drops `family == id`).
    pub family: String,
    pub tier: i32,
    pub raw: Obj,
}

impl SkillDef {
    pub fn from_obj(o: &Obj) -> Self {
        let id = string(o, "id", "");
        let family = string(o, "family", "");
        Self {
            family: if family == id { String::new() } else { family },
            id,
            name: string(o, "name", ""),
            kind: string(o, "kind", "ATTACK"),
            level: int(o, "level", 1),
            blocks: int(o, "blocks", 1),
            system: boolean(o, "system", false),
            basic: boolean(o, "basic", false),
            learned_only: boolean(o, "learned_only", false),
            grade: string(o, "grade", "novice"),
            requires: strings(o, "requires"),
            stance: string(o, "stance", "AUTO"),
            unlock_offset: int(o, "unlock_offset", -1),
            weapon_types: strings(o, "weapon_types"),
            spell_source: match o.get(SKILL_SPELL_SOURCE) {
                Some(v) => v.as_bool().unwrap_or(false),
                None => strings(o, "weapon_types").iter().any(|w| legacy_rules_of(w).any(|r| r == SPELL_SOURCE_RULE)),
            },
            min_proficiency: int(o, "min_proficiency", 0),
            reach_min: int(o, "reach_min", 0),
            reach_max: int(o, "reach_max", 0),
            area: string(o, "area", "NONE"),
            affects: Affects::parse(&string(o, "affects", "")),
            power_mult: float(o, "power_mult", 1.0),
            weapon_hand: string(o, "weapon_hand", "main"),
            dash_tiles: int(o, "dash_tiles", 0),
            position: Position::parse(&string(o, "position", "")),
            status_effect: string(o, "status_effect", ""),
            status_potency: int(o, "status_potency", 0),
            status_magnitude: int(o, "status_magnitude", 0),
            status_rounds: int(o, "status_rounds", 1),
            target_mode: string(o, "target_mode", "ALLY_OR_SELF"),
            heal_stat: string(o, "heal_stat", ""),
            heal_base: int(o, "heal_base", 0),
            heal_mult: float(o, "heal_mult", 0.0),
            heal_pct: int(o, "heal_pct", 0),
            grant_status: string(o, "grant_status", ""),
            extra_grants: array(o, "extra_grants").to_vec(),
            cleanse_statuses: strings(o, "cleanse_statuses"),
            mitigation: float(o, "mitigation", 0.0),
            guard_rounds: int(o, "guard_rounds", 1),
            boost: float(o, "boost", 0.0),
            boost_rounds: int(o, "boost_rounds", 0),
            jump_tiles: int(o, "jump_tiles", 0),
            recover_ap: int(o, "recover_ap", 0),
            ends_in_stance: string(o, "ends_in_stance", ""),
            disrupt: string(o, "disrupt", "NONE"),
            exploit: array(o, "exploit").to_vec(),
            exploit_consume: boolean(o, "exploit_consume", false),
            round_intent: string(o, "round_intent", ""),
            intent_buff: boolean(o, "intent_buff", false),
            ap_cost: int(o, "ap_cost", -1),
            recovery_blocks: int(o, "recovery_blocks", 0),
            stagger: int(o, "stagger", 0),
            windup_poise: int(o, "windup_poise", 0),
            unbreakable: boolean(o, "unbreakable", false),
            sure_break: boolean(o, "sure_break", false),
            armor_piercing: boolean(o, "armor_piercing", false),
            forced_move: obj(o, "forced_move").and_then(ForcedMove::from_obj),
            hazard: obj(o, "hazard").and_then(HazardPlace::from_obj),
            triggers: objs(o, "triggers").iter().map(TriggerRow::from_obj).collect(),
            riders: objs(o, "riders").iter().map(Rider::from_obj).collect(),
            meters: objs(o, "meters").iter().map(MeterDelta::from_obj).collect(),
            hp_cost: int(o, "hp_cost", 0),
            ignore_guard: boolean(o, "ignore_guard", false),
            hp_cost_pct: int(o, "hp_cost_pct", 0),
            requires_meter: obj(o, "requires_meter").and_then(MeterGate::from_obj),
            channel: obj(o, "channel").map(ChannelSpec::from_obj).unwrap_or_default(),
            unmark: obj(o, "unmark").map(|u| string(u, "mark", "")).unwrap_or_default(),
            uses_per_battle: int(o, "uses_per_battle", 0),
            tier: int(o, "tier", 1),
            raw: o.clone(),
        }
    }

    /// The family this skill belongs to (its own id when it stands alone).
    pub fn family_id(&self) -> &str {
        if self.family.is_empty() {
            &self.id
        } else {
            &self.family
        }
    }

    /// Levels after the family unlock at which this tier opens (SK2).
    pub fn tier_offset(&self) -> i32 {
        if self.unlock_offset >= 0 {
            self.unlock_offset
        } else {
            2 * (self.tier - 1).max(0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub item_type: String,
    pub slot: String,
    pub weapon_type: String,
    pub base_power: i32,
    pub base_range: i32,
    pub two_handed: bool,
    pub off_hand_category: String,
    pub scaling_stat: String,
    pub burden: i32,
    pub mitigation: i32,
    pub required_stats: IndexMap<String, i32>,
    pub stat_bonuses: IndexMap<String, i32>,
    pub effects: Vec<Value>,
    /// Resolved tier rows (legacy keys `stat`/`threshold`/`bonus_power`, extended rows add `when`/`bonus`).
    pub threshold_tiers: Vec<Obj>,
    pub consumable_type: String,
    pub heal: i32,
    pub damage: i32,
    pub buff_pct: f64,
    pub duration: i32,
    pub use_blocks: i32,
    pub use_reach: i32,
    /// Carry weight (PartyCarry; Item.weight, 1 when unset).
    pub weight: f64,
    /// RULESET1 K1: the item's family has the engine rule `reach` (`family_rules` holds it); the battle reads it only
    /// under the rule `reach`.
    pub reach_rule: bool,
    /// U3.2-S2: the engine rules of the item's family (the resolved row's `ITEM_FAMILY_RULES` key; a row without it
    /// reads the pre-family rules, `family_rules_of`). The gear rules read these, never a family id.
    pub family_rules: Vec<String>,
    /// U3.2-S4 (U3 "Level 2 (Adept)"): the family's Level-2 bonus (the resolved row's `ITEM_FAMILY_LEVEL2` key; the
    /// default: none).
    pub level2_bonus: FamilyBonus,
    /// U3.2-S4 (U3 "Level 3 (Master)"): the family's mastery passive id (the resolved row's `ITEM_FAMILY_LEVEL3` key; ""
    /// = none).
    pub level3_passive: String,
    /// PR5-28 (U3 FB-36c): the family's `range_band.max` (the resolved row's `ITEM_FAMILY_REACH_MAX` key; 0 = the family
    /// has no `range_band`, the reach rule then reads its fallback `rpgkit_rules::unit::REACH_WEAPON_REACH`).
    pub family_reach_max: i32,
    /// HOOKS1 GH2 (rule ground_hazards): the hazard a consumable leaves at use (None = none).
    pub hazard: Option<HazardPlace>,
    /// A1.2-K3 (INTENT2 section 6, INTENT2-K3): a thrown consumable's one authored area shape (`NONE` = single target;
    /// `PLUS`, `BURST`), the same in its Field and Assault rows.
    pub area: String,
    /// A1.2-K3 (A3.1 section 1, IR9): which units the area hits (None = the row's default, as a skill's `affects`).
    pub affects: Option<Affects>,
    /// The most copies of the item one stack holds (`max_stack`, 1 when unset).
    pub max_stack: i32,
    /// PK1-S1 (3.7): a consumable's `unmark` `{mark}`: the mark id whose latest entry on the target this battle it
    /// removes ("" = none).
    pub unmark: String,
    pub raw: Obj,
}

/// A1.2-K3: the `area` of a single-target consumable (the schema default; the writer stores no key for it).
pub const ITEM_AREA_NONE: &str = "NONE";
/// A1.2-K3 (INTENT2 section 6): the area shapes a consumable may author (a bomb is PLUS or BURST).
pub const ITEM_AREAS: [&str; 3] = [ITEM_AREA_NONE, "PLUS", "BURST"];
/// INTENT2-K3: the most copies a pouch stack holds whatever an item's `max_stack` (decision "Material stacks").
pub const POUCH_STACK_CAP: i32 = 5;

/// RULESET1 K1: the resolved item row's key that carried its family's engine rule `reach` before U3.2-S2 (true only
/// for those families); read now only on a row without `ITEM_FAMILY_RULES`, and the writer leaves it out of the stored
/// form.
pub const ITEM_REACH_RULE: &str = "reach_rule";
/// U3.2-S2: the resolved item row's key that carries its family's `engine_rules` (written by the loader on every row it
/// resolves against a family; the writer leaves it out of the stored form).
pub const ITEM_FAMILY_RULES: &str = "family_rules";
/// U3.2-S4: the resolved item row's key that carries its family's `level2_bonus` (written by the loader; the writer
/// leaves it out of the stored form).
pub const ITEM_FAMILY_LEVEL2: &str = "family_level2_bonus";
/// U3.2-S4: the resolved item row's key that carries its family's `level3_passive` (as `ITEM_FAMILY_LEVEL2`).
pub const ITEM_FAMILY_LEVEL3: &str = "family_level3_passive";
/// PR5-28 (U3 FB-36c): the resolved item row's key that carries its family's `range_band.max` (written by the loader only
/// when the family has a `range_band`; the writer leaves it out of the stored form).
pub const ITEM_FAMILY_REACH_MAX: &str = "family_reach_max";

/// U3.2-S4 (U3 "Level 2 (Adept)": "+crit or -burden from the family row"): a family's Level-2 bonus, `{"crit": n}` or
/// `{"burden": n}` (`{}` = none, the default). `crit` adds crit chance points while an item of the family is worn at
/// proficiency 2 or more; `burden` takes that much off each such item's burden (to 0 at least). Both sit on top of the
/// built per-level crit and burden of `Proficiency` (`crit_bonus`, `burden_reduction`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FamilyBonus {
    pub crit: f64,
    pub burden: i32,
}

impl FamilyBonus {
    pub fn from_obj(o: &Obj) -> Self {
        Self { crit: float(o, "crit", 0.0), burden: int(o, "burden", 0) }
    }

    pub fn is_empty(&self) -> bool {
        self.crit == 0.0 && self.burden == 0
    }
}

/// U3.2-S2: the resolved skill row's key that says whether one of its `weapon_types` is a family with the engine rule
/// `spell_source` (written by the loader on every skill row when the catalog has a family table).
pub const SKILL_SPELL_SOURCE: &str = "spell_source";
/// RULESET1 K1: the family engine rule that gives a melee weapon its reach.
pub const REACH_ENGINE_RULE: &str = "reach";
/// U3.2-S2: the family engine rule that makes a weapon ranged whatever its base range.
pub const ALWAYS_RANGED_RULE: &str = "always_ranged";
/// U3.2-S2: the family engine rule of an off-hand focus (never held under a two-handed main weapon).
pub const FOCUS_RULE: &str = "focus";
/// U3.2-S2: the family engine rule of a shield (the off-hand item that blocks).
pub const SHIELD_RULE: &str = "shield";
/// U3.2-S2: the family engine rule of a spell-source weapon (an attack skill needing one is in the passive group
/// "spell").
pub const SPELL_SOURCE_RULE: &str = "spell_source";
/// U3.2-S2: the engine rules of the gear families the kit named in code before the family table drove them (Godot's
/// Equipment and SkillDef lists). Read only for a row the loader did not resolve against a family table: a catalog
/// frozen before U3.2-S2 (the golden fixtures) or a hand-built row; shipped data reads its family's `engine_rules`.
pub const LEGACY_FAMILY_RULES: [(&str, &str); 8] = [
    ("bow", ALWAYS_RANGED_RULE),
    ("crossbow", ALWAYS_RANGED_RULE),
    ("sling", ALWAYS_RANGED_RULE),
    ("staff", SPELL_SOURCE_RULE),
    ("wand", SPELL_SOURCE_RULE),
    ("orb", FOCUS_RULE),
    ("tome", FOCUS_RULE),
    ("shield", SHIELD_RULE),
];

/// U3.2-S2: `LEGACY_FAMILY_RULES` of gear token `gear`.
pub fn legacy_rules_of(gear: &str) -> impl Iterator<Item = &'static str> + '_ {
    LEGACY_FAMILY_RULES.iter().filter(move |(f, _)| *f == gear).map(|(_, r)| *r)
}

/// U3.2-S2: an item row's family rules: the resolved `ITEM_FAMILY_RULES` key, else the pre-family reading (the
/// legacy rules of its gear token, `shield` for item type SHIELD, `reach` for the old `ITEM_REACH_RULE` key).
fn family_rules_of(o: &Obj) -> Vec<String> {
    if o.contains_key(ITEM_FAMILY_RULES) {
        return strings(o, ITEM_FAMILY_RULES);
    }
    let mut out: Vec<String> = legacy_rules_of(&string(o, "weapon_type", "")).map(String::from).collect();
    let mut add = |rule: &str| {
        if !out.iter().any(|r| r == rule) {
            out.push(rule.to_string());
        }
    };
    if string(o, "item_type", "MATERIAL") == "SHIELD" {
        add(SHIELD_RULE);
    }
    if boolean(o, ITEM_REACH_RULE, false) {
        add(REACH_ENGINE_RULE);
    }
    out
}

impl ItemDef {
    pub fn from_obj(o: &Obj) -> Self {
        let family_rules = family_rules_of(o);
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", ""),
            item_type: string(o, "item_type", "MATERIAL"),
            slot: string(o, "slot", "MAIN_HAND"),
            weapon_type: string(o, "weapon_type", ""),
            base_power: int(o, "base_power", 0),
            base_range: int(o, "base_range", 1),
            two_handed: boolean(o, "two_handed", false),
            off_hand_category: string(o, "off_hand_category", ""),
            scaling_stat: string(o, "scaling_stat", "STR"),
            burden: int(o, "burden", 0),
            mitigation: int(o, "mitigation", 0),
            required_stats: int_map(o, "required_stats"),
            stat_bonuses: int_map(o, "stat_bonuses"),
            effects: array(o, "effects").to_vec(),
            threshold_tiers: array(o, "threshold_tiers").iter().filter_map(|v| v.as_object().cloned()).collect(),
            consumable_type: string(o, "consumable_type", "HEALTH_POTION"),
            heal: int(o, "heal", 0),
            damage: int(o, "damage", 0),
            buff_pct: float(o, "buff_pct", 0.0),
            duration: int(o, "duration", 0),
            use_blocks: int(o, "use_blocks", 1),
            use_reach: int(o, "use_reach", -1),
            weight: float(o, "weight", 1.0),
            reach_rule: family_rules.iter().any(|r| r == REACH_ENGINE_RULE),
            family_rules,
            level2_bonus: obj(o, ITEM_FAMILY_LEVEL2).map(FamilyBonus::from_obj).unwrap_or_default(),
            level3_passive: string(o, ITEM_FAMILY_LEVEL3, ""),
            family_reach_max: int(o, ITEM_FAMILY_REACH_MAX, 0),
            hazard: obj(o, "hazard").and_then(HazardPlace::from_obj),
            area: string(o, "area", ITEM_AREA_NONE),
            affects: Affects::parse(&string(o, "affects", "")),
            max_stack: int(o, "max_stack", 1),
            unmark: obj(o, "unmark").map(|u| string(u, "mark", "")).unwrap_or_default(),
            raw: o.clone(),
        }
    }

    /// U3.2-S2: whether the item's family has engine rule `rule`.
    pub fn has_rule(&self, rule: &str) -> bool {
        self.family_rules.iter().any(|r| r == rule)
    }

    pub fn is_equipment(&self) -> bool {
        matches!(self.item_type.as_str(), "WEAPON" | "SHIELD" | "ARMOR" | "ACCESSORY")
    }

    pub fn is_consumable(&self) -> bool {
        self.item_type == "CONSUMABLE"
    }

    /// Consumable.TYPE_REACH: the battle reach of the item's type, -1 when the type is not offered in battle
    /// (an unknown type reads as HEALTH_POTION, like ContentFactory's enum fallback).
    pub fn type_reach(&self) -> i32 {
        match self.consumable_type.as_str() {
            "FOCUS_INCENSE" => 0,
            "THROWABLE_BOMB" => 3,
            "STAMINA_BREW" | "ANTIDOTE" => -1,
            _ => 1,
        }
    }

    /// Consumable.reach: its own use_reach, else the type default (0 = self only).
    pub fn battle_reach(&self) -> i32 {
        if self.use_reach >= 0 {
            self.use_reach
        } else {
            self.type_reach()
        }
    }

    /// ItemCommand.offered: a consumable whose type is offered in battle.
    pub fn offered_in_battle(&self) -> bool {
        self.is_consumable() && self.type_reach() >= 0 && self.battle_reach() >= 0
    }

    /// A1.2-K3: whether the item hits an area shape (`area` other than `NONE`).
    pub fn is_area(&self) -> bool {
        !self.area.is_empty() && self.area != ITEM_AREA_NONE
    }

    /// INTENT2-K3: the copies one pouch stack of the item holds under `pouch_stacks`: `min(5, max_stack)`, at least 1.
    pub fn stack_limit(&self) -> i32 {
        self.max_stack.clamp(1, POUCH_STACK_CAP)
    }

    /// ItemCommand.aims_at_foes: damage with no heal or boost.
    pub fn aims_at_foes(&self) -> bool {
        self.damage > 0 && self.heal <= 0 && self.buff_pct <= 0.0
    }
}

#[derive(Debug, Clone)]
pub struct PassiveDef {
    pub id: String,
    pub name: String,
    pub effects: Vec<Value>,
    /// RULESET1 9 D3 (rule deck): "" = source-granted, always on; "passive" or "keystone" = applies only while slotted.
    pub slot: String,
}

/// The formula input name of the unit level (DERIVE1 4.1; reserved: no row id or constant takes it).
pub const LEVEL_INPUT: &str = "level";
/// RULESET1 P1 (tasks 1.7): the formula input of the summed `effective_mitigation` of the unit's worn gear (reserved
/// like `level`: no row id or constant takes it).
pub const GEAR_MITIGATION_INPUT: &str = "gear_mitigation";
/// R4-1 (user 2026-11-09): the formula input of the unit's tier factor, its row's `mult` in the kit's tier table;
/// reserved like `level`.
pub const TIER_MULT_INPUT: &str = "tier_mult";

/// A derived stat's change while a status is active on the unit (DERIVE1 4.1 `status_modifiers`).
#[derive(Debug, Clone, PartialEq)]
pub struct StatusModifier {
    pub status: String,
    pub flat: i32,
    pub percent: f64,
}

/// A derived-stat row (DERIVE1 4.1, kind `derived_stats`): a formula over the base stats, `level`, other rows and the
/// row's constants, caps, the rulesets it exists in and the screens that list it. Engine-only (no Godot original).
#[derive(Debug, Clone)]
pub struct DerivedStatDef {
    pub id: String,
    pub name: String,
    /// Table header ("" = the name).
    pub label: String,
    pub description: String,
    pub formula: Formula,
    pub constants: IndexMap<String, f64>,
    pub min: i32,
    pub max: i32,
    pub status_modifiers: Vec<StatusModifier>,
    /// Empty = every ruleset.
    pub rulesets: Vec<String>,
    pub show_on: Vec<String>,
    pub order: i32,
    pub suffix: String,
}

impl DerivedStatDef {
    /// From a loaded (checked) row; None when the formula does not parse (the loader reports and skips such rows).
    pub fn from_obj(o: &Obj) -> Option<Self> {
        let formula = Formula::parse(&string(o, "formula", "0")).ok()?;
        let constants = obj(o, "constants").map(|m| m.iter().map(|(k, v)| (k.clone(), to_float(v, 0.0))).collect()).unwrap_or_default();
        let status_modifiers = array(o, "status_modifiers")
            .iter()
            .filter_map(Value::as_object)
            .map(|m| StatusModifier { status: string(m, "status", ""), flat: int(m, "flat", 0), percent: float(m, "percent", 0.0) })
            .collect();
        Some(Self {
            id: string(o, "id", ""),
            name: string(o, "name", "New Stat"),
            label: string(o, "label", ""),
            description: string(o, "description", ""),
            formula,
            constants,
            min: int(o, "min", 0),
            max: int(o, "max", 100),
            status_modifiers,
            rulesets: strings(o, "rulesets"),
            show_on: if o.contains_key("show_on") { strings(o, "show_on") } else { vec!["party".into(), "dojo".into()] },
            order: int(o, "order", 0),
            suffix: string(o, "suffix", ""),
        })
    }

    /// The table header: the label, or the name when it has none.
    pub fn header(&self) -> &str {
        if self.label.is_empty() {
            &self.name
        } else {
            &self.label
        }
    }

    /// Whether the row exists in `ruleset` (no rulesets listed = every one).
    pub fn in_ruleset(&self, ruleset: &str) -> bool {
        self.rulesets.is_empty() || self.rulesets.iter().any(|r| r == ruleset)
    }

    /// Whether `screen` (party, dojo, battle) lists the row.
    pub fn shows_on(&self, screen: &str) -> bool {
        self.show_on.iter().any(|s| s == screen)
    }
}

/// The whole catalog, every kind in file order. Entries the engine does not type yet stay raw.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub classes: IndexMap<String, ClassDef>,
    pub units: IndexMap<String, UnitDef>,
    pub skills: Vec<SkillDef>,
    pub items: IndexMap<String, ItemDef>,
    pub passives: IndexMap<String, PassiveDef>,
    pub encounters: IndexMap<String, Obj>,
    pub terrain: IndexMap<String, Obj>,
    /// Derived-stat rows in the loader's order (topological: a row after the rows its formula names).
    pub derived_stats: IndexMap<String, DerivedStatDef>,
    /// Ruleset rows (RULESET1-R, kind `rulesets`; engine-only), raw: `rpgkit_rules::ruleset::Ruleset` reads them.
    pub rulesets: IndexMap<String, Obj>,
    /// Portrait cue rows (PORTRAITFX1 5.1, kind `portrait_cues`; engine-only) in file order.
    pub portrait_cues: Vec<PortraitCue>,
    /// Expression-effect rows (PORTRAITFX1-EX, kind `portrait_expressions`; engine-only) in file order.
    pub portrait_expressions: Vec<PortraitExpression>,
    /// Ground hazard rows (HOOKS1 GH1, kind `hazards`; engine-only).
    pub hazards: IndexMap<String, HazardDef>,
    /// Unit meter rows (HOOKS1 UM1, kind `unit_meters`; engine-only).
    pub unit_meters: IndexMap<String, UnitMeterDef>,
    /// Data zone rows (HOOKS1 ZN1, kind `zones`; engine-only; the 8 code zones are not rows).
    pub zones: IndexMap<String, ZoneDef>,
    /// The game's term rows (TERMS1 5.1, kind `terms`; engine-only; `game`-scope register ids only).
    pub terms: IndexMap<String, TermRow>,
    /// Ground rows of a region map's picture (REGIONGROUND1 KT 1.1, kind `grounds`; engine-only; presentation only: no
    /// rule reads them).
    pub grounds: IndexMap<String, GroundDef>,
    /// R4-1 (`U2.3-TIERS`, FB-78): the role templates (kind `unit_templates`; engine-only), keyed by role (the first
    /// row of a role wins; the content check refuses a second). No row for a role = its units keep their class's stats.
    pub unit_templates: IndexMap<String, UnitTemplate>,
    /// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): the unit tracks (kind `unit_tracks`; engine-only), keyed by id. No row =
    /// no member holds a track.
    pub unit_tracks: IndexMap<String, UnitTrackDef>,
    /// PK1-KITFIX (c): the Dojo's data presets (kind `dojo_presets`; engine-only), keyed by id in file order. No row =
    /// the Dojo lists its built-in presets only.
    pub dojo_presets: IndexMap<String, DojoPresetDef>,
    skill_index: IndexMap<String, usize>,
    tier_one: IndexMap<String, usize>,
}

impl Catalog {
    /// PORTRAITFX2 FL1: the game's faces: `neutral`, then the `portrait_expressions` row ids in file order
    /// (`faces_of`; no rows: `DEFAULT_FACES`).
    pub fn faces(&self) -> Vec<String> {
        faces_of(self.portrait_expressions.iter().map(|r| r.id.as_str()))
    }

    /// Builds the catalog from a resolved `{kind: [entry, ...]}` value.
    pub fn from_value(root: &Value) -> Self {
        let rows = |kind: &str| -> Vec<Obj> {
            root.get(kind).and_then(Value::as_array).map(|a| a.iter().filter_map(|v| v.as_object().cloned()).collect()).unwrap_or_default()
        };
        let mut cat = Catalog::default();
        for o in rows("classes") {
            let c = ClassDef::from_obj(&o);
            cat.classes.insert(c.id.clone(), c);
        }
        for o in rows("units") {
            let u = UnitDef::from_obj(&o);
            cat.units.insert(u.id.clone(), u);
        }
        for o in rows("items") {
            let i = ItemDef::from_obj(&o);
            cat.items.insert(i.id.clone(), i);
        }
        for o in rows("passives") {
            let p = PassiveDef { id: string(&o, "id", ""), name: string(&o, "name", ""), effects: array(&o, "effects").to_vec(), slot: string(&o, "slot", "") };
            cat.passives.insert(p.id.clone(), p);
        }
        for o in rows("encounters") {
            cat.encounters.insert(string(&o, "id", ""), o);
        }
        for o in rows("terrain") {
            cat.terrain.insert(string(&o, "id", ""), o);
        }
        for o in rows("rulesets") {
            cat.rulesets.insert(string(&o, "id", ""), o);
        }
        for d in rows("derived_stats").iter().filter_map(DerivedStatDef::from_obj) {
            cat.derived_stats.insert(d.id.clone(), d);
        }
        cat.portrait_cues = rows(PORTRAIT_CUES_KIND).iter().map(PortraitCue::from_obj).collect();
        cat.portrait_expressions = rows(PORTRAIT_EXPRESSIONS_KIND).iter().map(PortraitExpression::from_obj).collect();
        cat.hazards = rows(HAZARDS_KIND).iter().map(HazardDef::from_obj).map(|h| (h.id.clone(), h)).collect();
        cat.unit_meters = rows(UNIT_METERS_KIND).iter().map(UnitMeterDef::from_obj).map(|m| (m.id.clone(), m)).collect();
        cat.zones = rows(ZONES_KIND).iter().map(ZoneDef::from_obj).map(|z| (z.id.clone(), z)).collect();
        cat.terms = rows(TERMS_KIND).iter().map(TermRow::from_obj).map(|t| (t.id.clone(), t)).collect();
        cat.grounds = rows(GROUNDS_KIND).iter().map(GroundDef::from_obj).map(|g| (g.id.clone(), g)).collect();
        for t in rows(UNIT_TEMPLATES_KIND).iter().map(UnitTemplate::from_obj) {
            cat.unit_templates.entry(t.role.clone()).or_insert(t);
        }
        cat.unit_tracks = rows(UNIT_TRACKS_KIND).iter().map(UnitTrackDef::from_obj).map(|t| (t.id.clone(), t)).collect();
        for p in rows(DOJO_PRESETS_KIND).iter().map(DojoPresetDef::from_obj) {
            cat.dojo_presets.entry(p.id.clone()).or_insert(p);
        }
        cat.skills = rows("skills").iter().map(SkillDef::from_obj).collect();
        cat.index_skills();
        cat
    }

    /// Appends skills after the authored ones (a battle's synthetic item skills) and re-indexes.
    pub fn add_skills(&mut self, extra: Vec<SkillDef>) {
        self.skills.extend(extra);
        self.index_skills();
    }

    /// SkillCatalog._build: first row per id, lowest tier per family (ties keep file order).
    fn index_skills(&mut self) {
        self.skill_index.clear();
        self.tier_one.clear();
        for (i, s) in self.skills.iter().enumerate() {
            self.skill_index.entry(s.id.clone()).or_insert(i);
            let fam = s.family_id().to_string();
            match self.tier_one.get(&fam) {
                Some(&low) if self.skills[low].tier <= s.tier => {}
                _ => {
                    self.tier_one.insert(fam, i);
                }
            }
        }
    }

    /// TERMS1 TM3, TM4: term `id` inside a battle: the game's `terms` row, else the default (a `world` term, which the
    /// catalog cannot name, is its default; with a world use `rpgkit_world::terms::term`).
    pub fn term(&self, id: &str) -> Term {
        match self.terms.get(id) {
            Some(row) => row.term(),
            None => Term::default_of(id),
        }
    }

    /// PK2-S1: the departure causes a save may record: `DEPART_CAUSES`, then each unit track row's `cause` (the game's
    /// word, named in its data), each once.
    pub fn depart_causes(&self) -> Vec<String> {
        let mut out: Vec<String> = DEPART_CAUSES.iter().map(|c| c.to_string()).collect();
        for t in self.unit_tracks.values() {
            if !t.cause.is_empty() && !out.contains(&t.cause) {
                out.push(t.cause.clone());
            }
        }
        out
    }

    /// `cause` is one of `depart_causes`.
    pub fn depart_cause_ok(&self, cause: &str) -> bool {
        DEPART_CAUSES.contains(&cause) || self.unit_tracks.values().any(|t| !t.cause.is_empty() && t.cause == cause)
    }

    pub fn skill(&self, id: &str) -> Option<&SkillDef> {
        self.skill_index.get(id).map(|&i| &self.skills[i])
    }

    pub fn tier_one(&self, family: &str) -> Option<&SkillDef> {
        self.tier_one.get(family).map(|&i| &self.skills[i])
    }

    pub fn family_of<'a>(&'a self, skill_id: &'a str) -> &'a str {
        self.skill(skill_id).map(SkillDef::family_id).unwrap_or(skill_id)
    }

    /// The family's rows sorted by tier (SKILLUI1: the ranks a Skills tab slot steps over).
    pub fn family_rows(&self, family: &str) -> Vec<&SkillDef> {
        let mut rows: Vec<&SkillDef> = self.skills.iter().filter(|s| s.family_id() == family).collect();
        rows.sort_by_key(|s| s.tier);
        rows
    }

    /// Basic, non-system skills in catalog order (every unit owns them from level 1).
    pub fn basic_ids(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for s in &self.skills {
            if s.basic && !s.system && !out.contains(&s.id) {
                out.push(s.id.clone());
            }
        }
        out
    }

    pub fn is_system(&self, skill_id: &str) -> bool {
        self.skill(skill_id).is_some_and(|s| s.system)
    }

    pub fn is_learned_only(&self, skill_id: &str) -> bool {
        self.skill(skill_id).is_some_and(|s| s.learned_only)
    }

    pub fn is_basic_family(&self, family: &str) -> bool {
        self.skills.iter().any(|s| s.family_id() == family && s.basic)
    }

    /// Basic or engine skill: always passes the active-loadout filter.
    pub fn is_always_active(&self, skill_id: &str) -> bool {
        self.skill(skill_id).is_some_and(|s| s.system || self.is_basic_family(s.family_id()))
    }
}

// ---- Campaign (CAMPAIGN1 11.1-11.4, kit hook; engine-only, no Godot original) ---------------------------------------

/// CAMPAIGN1 11.2: the world kinds of a campaign (files `meters.json`, `checkpoints.json`, `triggers.json`).
pub const CAMPAIGN_KINDS: [&str; 3] = ["meters", "checkpoints", "triggers"];
/// M1: a gauge holds a value in `min..max`; a countdown holds a due time on the travel clock.
pub const METER_KINDS: [&str; 2] = ["gauge", "countdown"];
/// M2: the HUD tone of a band.
pub const METER_TONES: [&str; 4] = ["calm", "warn", "danger", "fatal"];
/// M5: the crossing direction of a meter rule (`up` the gauge default, `down` the countdown default).
pub const RULE_DIRS: [&str; 2] = ["up", "down"];
/// M3: what moves a gauge.
pub const INPUT_ON: [&str; 3] = ["battle", "day", "leg"];
/// A settled battle's outcomes (meter inputs, triggers).
pub const OUTCOMES: [&str; 3] = ["won", "lost", "fled"];
/// T1: the cause a trigger is checked after.
pub const TRIGGER_ON: [&str; 5] = ["any", "day", "battle", "enter", "act"];
/// F1: what an `end` effect ends the run as.
pub const END_KINDS: [&str; 2] = ["fail", "bad_end"];
/// R1: where the party may rest.
pub const REST_WHERE: [&str; 3] = ["nodes", "rest_places", "none"];
/// R4: what a lost battle does.
pub const DEFEAT_MODES: [&str; 2] = ["wake", "retreat"];
/// 11.3: the campaign effect kinds and their keys besides `effect` (dialogue choices, meter rules, checkpoints and
/// triggers take them).
pub const CAMPAIGN_EFFECTS: [(&str, &[&str]); 9] = [
    ("meter_add", &["meter", "delta"]),
    ("meter_set", &["meter", "value"]),
    ("meter_show", &["meter", "visible"]),
    ("clock_start", &["meter", "days"]),
    ("clock_set", &["meter", "days", "label"]),
    ("clock_stop", &["meter"]),
    ("set_act", &["act"]),
    ("rest", &[]),
    ("end", &["kind", "card", "text"]),
];
/// 11.4: flag-key prefixes the campaign answers (read-only: `set_flag` and `add_flag` refuse them in every world).
pub const RESERVED_PREFIXES: [&str; 3] = ["meter.", "campaign.", "roster."];
/// 11.4: the read keys besides `meter.<id>`, `meter.<id>.band` and `meter.<id>.running`.
pub const CAMPAIGN_READ_KEYS: [&str; 3] = ["campaign.act", "campaign.day", "roster.living"];
/// 11.4: the read-key suffixes after `meter.<id>`.
pub const METER_READ_SUFFIXES: [&str; 2] = ["band", "running"];

/// Whether `key` starts with a reserved prefix (11.4).
pub fn reserved_key(key: &str) -> bool {
    RESERVED_PREFIXES.iter().any(|p| key.starts_with(p))
}

/// The meter id a read key names (`meter.<id>`, `meter.<id>.band`, `meter.<id>.running`), else None.
pub fn meter_of_read_key(key: &str) -> Option<&str> {
    let rest = key.strip_prefix("meter.")?;
    match rest.split_once('.') {
        None => Some(rest),
        Some((id, suffix)) if METER_READ_SUFFIXES.contains(&suffix) => Some(id),
        Some(_) => None,
    }
}

/// The manifest's `campaign` block (11.1): `on` once the block holds anything (X1).
#[derive(Debug, Clone, PartialEq)]
pub struct CampaignManifest {
    pub on: bool,
    pub start_act: i32,
    pub act_saves: bool,
}

impl CampaignManifest {
    pub fn from_obj(o: &Obj) -> Self {
        Self { on: !o.is_empty(), start_act: int(o, "start_act", 0), act_saves: boolean(o, "act_saves", true) }
    }
}

/// One purse counter's names (P1): the HUD name, its short form, icon and tip.
#[derive(Debug, Clone, PartialEq)]
pub struct PurseCounter {
    pub name: String,
    pub short: String,
    pub icon: String,
    pub tip: String,
}

impl PurseCounter {
    fn from_obj(o: &Obj, name: &str, icon: &str, tip: &str) -> Self {
        let name = string(o, "name", name);
        Self { short: string(o, "short", &name), icon: string(o, "icon", icon), tip: string(o, "tip", tip), name }
    }
}

/// The manifest's `purse` block (P1-P3): the currency's and the upkeep's names, the upkeep spent per day and what
/// zero upkeep does (`road_pct` multiplies a leg's road-fight chance, `start_ap` changes each party unit's battle AP).
#[derive(Debug, Clone, PartialEq)]
pub struct Purse {
    pub currency: PurseCounter,
    pub upkeep: PurseCounter,
    pub per_day: i32,
    pub road_pct: i32,
    pub start_ap: i32,
}

/// The purse defaults: today's HUD names and tips, x1.5 road fights at zero supplies (`travel.rs` STARVED_PCT).
pub const CURRENCY_NAME: &str = "Gold";
pub const CURRENCY_ICON: &str = "ui/hud/gold.png";
pub const CURRENCY_TIP: &str = "Earned from fights, chests and quests; spent in shops.";
pub const UPKEEP_NAME: &str = "Supplies";
pub const UPKEEP_ICON: &str = "ui/hud/supplies.png";
pub const UPKEEP_TIP: &str = "Food for the road: each day of travel eats some, and so does making camp.";

impl Purse {
    pub fn from_obj(o: &Obj) -> Self {
        let empty = Obj::new();
        let up = obj(o, "upkeep").unwrap_or(&empty);
        let zero = obj(up, "at_zero").unwrap_or(&empty);
        Self {
            currency: PurseCounter::from_obj(obj(o, "currency").unwrap_or(&empty), CURRENCY_NAME, CURRENCY_ICON, CURRENCY_TIP),
            upkeep: PurseCounter::from_obj(up, UPKEEP_NAME, UPKEEP_ICON, UPKEEP_TIP),
            per_day: int(up, "per_day", 1),
            road_pct: int(zero, "road_pct", 150),
            start_ap: int(zero, "start_ap", 0),
        }
    }
}

/// The manifest's `rest` block (R1); the defaults are today's Camp and free rest-place heal.
#[derive(Debug, Clone, PartialEq)]
pub struct RestRule {
    pub days: i32,
    pub cost: i32,
    /// One of REST_WHERE.
    pub place: String,
    pub on_enter: bool,
    pub needs_upkeep: bool,
}

impl RestRule {
    pub fn from_obj(o: &Obj) -> Self {
        Self { days: int(o, "days", 0), cost: int(o, "cost", 3), place: string(o, "where", "nodes"), on_enter: boolean(o, "on_enter", true), needs_upkeep: boolean(o, "needs_upkeep", true) }
    }
}

/// The manifest's `defeat` block (R4); the default is today's wake.
#[derive(Debug, Clone, PartialEq)]
pub struct DefeatRule {
    /// One of DEFEAT_MODES.
    pub mode: String,
    pub to: String,
    pub days: i32,
}

impl DefeatRule {
    pub fn from_obj(o: &Obj) -> Self {
        Self { mode: string(o, "mode", "wake"), to: string(o, "to", ""), days: int(o, "days", 0) }
    }
}

/// CAMPAIGN1-TM: the keys of the manifest's `travel_modes` block and of one mode row.
pub const TRAVEL_MODES_KEYS: [&str; 2] = ["default", "modes"];
pub const TRAVEL_MODE_KEYS: [&str; 7] = ["id", "name", "tip", "days", "road_pct", "sure_when", "road_encounters"];
/// CAMPAIGN1-TM: the most modes a world names.
pub const TRAVEL_MODES_MAX: usize = 4;

/// CAMPAIGN1-TM: one travel mode (a row of the manifest's `travel_modes.modes`), read with its defaults: `days` more
/// per leg (they pass and cost upkeep as the route's own), `road_pct` times the leg's road-fight chance (100 =
/// unchanged), `sure_when` flag conditions (read keys included) under which every leg's road fight is sure (empty =
/// never) and `road_encounters` ([ref, weight] pairs) added to the leg's road pool.
#[derive(Debug, Clone, PartialEq)]
pub struct TravelMode {
    pub id: String,
    pub name: String,
    pub tip: String,
    pub days: i32,
    pub road_pct: i32,
    pub sure_when: Vec<Value>,
    pub road_encounters: Vec<Value>,
}

impl TravelMode {
    pub fn from_obj(o: &Obj) -> Self {
        let id = string(o, "id", "");
        Self {
            name: string(o, "name", &id),
            tip: string(o, "tip", ""),
            days: int(o, "days", 0),
            road_pct: int(o, "road_pct", 100),
            sure_when: array(o, "sure_when").to_vec(),
            road_encounters: array(o, "road_encounters").to_vec(),
            id,
        }
    }

    /// CAMPAIGN1-TM2: the days this mode adds to every leg (`days`, never below 0).
    pub fn leg_days(&self) -> i64 {
        (self.days as i64).max(0)
    }
}

/// CAMPAIGN1-TM: the manifest's `travel_modes` block: the modes in data order (rows without an id are left out) and
/// the default's id (`default`, else the first mode). No modes = travel as before.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TravelModes {
    pub modes: Vec<TravelMode>,
    pub default: String,
}

impl TravelModes {
    pub fn from_obj(o: &Obj) -> Self {
        let modes: Vec<TravelMode> = objs(o, "modes").iter().map(TravelMode::from_obj).filter(|m| !m.id.is_empty()).collect();
        let named = string(o, "default", "");
        let default = if modes.iter().any(|m| m.id == named) { named } else { modes.first().map(|m| m.id.clone()).unwrap_or_default() };
        Self { modes, default }
    }

    pub fn is_empty(&self) -> bool {
        self.modes.is_empty()
    }

    pub fn get(&self, id: &str) -> Option<&TravelMode> {
        self.modes.iter().find(|m| m.id == id)
    }
}

// ---- Roster (ROSTER1 3-6, 9, 11, kit hook; engine-only, no Godot original) ------------------------------------------

/// 3.1: the lives a departed member ends with (`party.departed[].life`).
pub const DEPART_LIVES: [&str; 5] = ["captured", "dead", "retired", "dismissed", "lost"];
/// KS1, 11.1: the lives that pay a keepsake (`keepsakes.on`, a rule row's `keepsakes_on`).
pub const KEEPSAKE_LIVES: [&str; 3] = ["captured", "dead", "lost"];
/// 11.4: why a member departed (`party.departed[].cause`).
/// PK2-S1: a unit track row's `cause` is a known cause too (`Catalog::depart_cause_ok`); the game names it in its data.
pub const DEPART_CAUSES: [&str; 7] = ["stayed", "stood", "downed", "bled_out", "lost_battle", "story", "roster"];
/// L4: the HP of members who come home (`carry` = HEAL1's carry-over, `full` = back to full).
pub const HP_AFTER: [&str; 2] = ["carry", "full"];
/// L3: the settle value that keeps today's HEAL1 result.
pub const SETTLE_CLASSIC: &str = "classic";
/// 3.2: the values of `downed_won` and `standing_lost`.
pub const SETTLE_HURT: [&str; 2] = ["classic", "wounded"];
/// 3.2: the values of `downed_left` and `downed_lost`.
pub const SETTLE_LOSS: [&str; 4] = ["classic", "wounded", "captured", "dead"];
/// BK2: where a member lives (`bunk` the default; saved only when not `bunk`).
pub const HOUSING: [&str; 3] = ["bunk", "crypt", "crypt_only"];
/// BK5: the named join levels of the `recruit` effect's `level` (else a whole number 1..RECRUIT_LEVEL_MAX).
pub const RECRUIT_LEVELS: [&str; 2] = ["highest", "average"];
pub const RECRUIT_LEVEL_MAX: i64 = 20;
/// 11.1: the most bunks, bunk cap and crypt places a manifest names.
pub const BUNKS_MAX: i32 = 12;
/// 11.1: the most rule rows of `roster.rules`.
pub const ROSTER_RULES_MAX: usize = 4;
/// 11.2: the longest bleed-out (rounds; 0 = off).
pub const BLEED_OUT_ROUNDS_MAX: i32 = 6;
/// W2, 11.2: a wounded member's max HP cut (percent of its unwounded max).
pub const WOUNDED_PCT_DEFAULT: i32 = 20;
pub const WOUNDED_PCT_MIN: i32 = 1;
pub const WOUNDED_PCT_MAX: i32 = 90;
/// BK6, 11.3: an encounter's `party_min` range top (and at most the world's party max).
pub const PARTY_MIN_MAX: i64 = 4;
/// KS3, 11.1: the procedural keepsake's display name (`keepsakes.name`).
pub const KEEPSAKE_NAME_DEFAULT: &str = "{name}'s {item}";
pub const KEEPSAKE_NAME_MAX: usize = 40;
pub const KEEPSAKE_NAME_TOKENS: [&str; 2] = ["{name}", "{item}"];
/// RK1: the words after `roster.` that are not member ids (a member id equal to one is refused).
pub const ROSTER_WORDS: [&str; 6] = ["living", "bunks", "bunks_free", "pool", "open", "departed"];
/// RK1: the read-key prefix the roster answers.
pub const ROSTER_PREFIX: &str = "roster.";

/// RK1: what a `roster.` read key asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterRead<'a> {
    /// `roster.living`: members housed in bunks or crypts.
    Living,
    /// `roster.bunks`: the bunk cap.
    Bunks,
    /// `roster.bunks_free`: the cap less the bunks in use.
    BunksFree,
    /// `roster.pool`: pool entries.
    Pool,
    /// `roster.open`: the open flag holds.
    Open,
    /// `roster.departed.<life>`: departed members of that life.
    Departed(&'a str),
    /// `roster.<member id>`: that member's state (`ready` or a departed life; "" when unknown).
    Member(&'a str),
}

/// RK1: the question a `roster.` key asks (None: not a roster read key, or malformed).
pub fn roster_read_key(key: &str) -> Option<RosterRead<'_>> {
    let rest = key.strip_prefix(ROSTER_PREFIX)?;
    let id_like = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    match rest {
        "living" => Some(RosterRead::Living),
        "bunks" => Some(RosterRead::Bunks),
        "bunks_free" => Some(RosterRead::BunksFree),
        "pool" => Some(RosterRead::Pool),
        "open" => Some(RosterRead::Open),
        _ => match rest.strip_prefix("departed.") {
            Some(life) => DEPART_LIVES.contains(&life).then_some(RosterRead::Departed(life)),
            None => (id_like(rest) && !ROSTER_WORDS.contains(&rest)).then_some(RosterRead::Member(rest)),
        },
    }
}

/// 11.2: one rule row of `roster.rules` (the act's lifecycle rules), read with its defaults (every one today's rule).
#[derive(Debug, Clone, PartialEq)]
pub struct RosterRule {
    /// The acts the row covers (empty = every act; only the last row).
    pub acts: Vec<i32>,
    pub bleed_out_rounds: i32,
    /// One of HP_AFTER.
    pub hp_after: String,
    /// One of SETTLE_HURT.
    pub downed_won: String,
    /// One of SETTLE_LOSS.
    pub downed_left: String,
    /// One of SETTLE_LOSS.
    pub downed_lost: String,
    /// One of SETTLE_HURT.
    pub standing_lost: String,
    pub wounded_max_hp_pct: i32,
    /// Overrides the manifest's `keepsakes.on` for this row when set.
    pub keepsakes_on: Option<Vec<String>>,
}

impl RosterRule {
    pub fn from_obj(o: &Obj) -> Self {
        let settle = |key: &str| string(o, key, SETTLE_CLASSIC);
        Self {
            acts: array(o, "acts").iter().map(|a| to_int(a, 0)).collect(),
            bleed_out_rounds: int(o, "bleed_out_rounds", 0),
            hp_after: string(o, "hp_after", HP_AFTER[0]),
            downed_won: settle("downed_won"),
            downed_left: settle("downed_left"),
            downed_lost: settle("downed_lost"),
            standing_lost: settle("standing_lost"),
            wounded_max_hp_pct: int(o, "wounded_max_hp_pct", WOUNDED_PCT_DEFAULT),
            keepsakes_on: o.get("keepsakes_on").map(|_| strings(o, "keepsakes_on")),
        }
    }

    /// L6: whether the row covers `act`.
    pub fn covers(&self, act: i32) -> bool {
        self.acts.is_empty() || self.acts.contains(&act)
    }
}

/// 11.1: the manifest's `roster` block, read with its defaults ({} = today's rules: bunks = party max, no rule row,
/// no crypts, no open flag, no keepsakes).
#[derive(Debug, Clone, PartialEq)]
pub struct RosterManifest {
    /// 0 = the party max (BK1).
    pub bunks: i32,
    /// An int flag added to the bunks (clamped at 0).
    pub bunks_flag: String,
    pub bunks_max: i32,
    pub crypts: i32,
    /// "" = always open.
    pub open_flag: String,
    pub rules: Vec<RosterRule>,
    /// KS1: the lives that pay a keepsake (a rule row's `keepsakes_on` overrides).
    pub keepsakes_on: Vec<String>,
    /// KS3: class id -> keepsake template item id.
    pub keepsakes_by_class: IndexMap<String, String>,
    /// KS3: the procedural keepsake's name, tokens `{name}`, `{item}`.
    pub keepsake_name: String,
}

impl RosterManifest {
    pub fn from_obj(o: &Obj) -> Self {
        let empty = Obj::new();
        let ks = obj(o, "keepsakes").unwrap_or(&empty);
        let by_class = obj(ks, "by_class").map(|m| m.iter().map(|(k, v)| (k.clone(), to_str(v))).collect()).unwrap_or_default();
        Self {
            bunks: int(o, "bunks", 0),
            bunks_flag: string(o, "bunks_flag", ""),
            bunks_max: int(o, "bunks_max", BUNKS_MAX),
            crypts: int(o, "crypts", 0),
            open_flag: string(o, "open_flag", ""),
            rules: objs(o, "rules").iter().map(RosterRule::from_obj).collect(),
            keepsakes_on: strings(ks, "on"),
            keepsakes_by_class: by_class,
            keepsake_name: string(ks, "name", KEEPSAKE_NAME_DEFAULT),
        }
    }

    /// L6: the rule row of `act` (the first row whose `acts` is empty or holds it); None = classic.
    pub fn rule(&self, act: i32) -> Option<&RosterRule> {
        self.rules.iter().find(|r| r.covers(act))
    }

    /// KS1: the lives that pay a keepsake in `act` (the row's `keepsakes_on` when set, else `keepsakes.on`).
    pub fn keepsake_lives(&self, act: i32) -> &[String] {
        self.rule(act).and_then(|r| r.keepsakes_on.as_deref()).unwrap_or(&self.keepsakes_on)
    }

    /// BK1: the bunk cap: `bunks` (0 = `party_max`) plus the bunks flag's value (clamped at 0), at most `bunks_max`.
    pub fn bunk_cap(&self, party_max: i32, flag_value: i32) -> i32 {
        let base = if self.bunks == 0 { party_max } else { self.bunks };
        (base + flag_value.max(0)).min(self.bunks_max.max(base))
    }
}

// ---- Story (STORYUI1 4.1-4.8, kit hook; engine-only, no Godot original) ---------------------------------------------

/// STORYUI1 4.1, 4.3, 4.4: the world kinds of the story layer (files `vignettes.json`, `sequences.json`, `boards.json`).
pub const STORY_KINDS: [&str; 3] = ["vignettes", "sequences", "boards"];
/// 4.2: a card's style.
pub const CARD_STYLES: [&str; 4] = ["image", "text", "title", "line"];
/// 4.2: the keys a card takes (any other key is refused); PORTRAITFX1 7: a `line` card's `expression`; FX1 5.1: a
/// card's camera `shake`.
pub const CARD_KEYS: [&str; 16] = ["style", "image", "top", "bottom", "ash", "title", "subtitle", "text", "speaker", "fade_in_ms", "fade_out_ms", "hold_ms", "cue", "when", "expression", "shake"];
/// FX1 5.1: the camera shake presets a card's `shake` and a phase's shake effect may name: the kit's copy of the
/// engine's `aegis_ui::fx::SHAKE_PRESETS` (the content crates do not depend on `aegis_ui`; a GUI test keeps them equal).
pub const SHAKE_PRESETS: [&str; 5] = ["jolt", "heavy", "toll", "tremor", "quake"];
/// FX1 5.1: the phase effect that asks for a camera shake (`{"type": "shake", "preset"}`, at most one per phase; no
/// rule reads it, the GUI does).
pub const SHAKE_EFFECT: &str = "shake";
pub const SHAKE_EFFECT_KEYS: [&str; 2] = ["type", "preset"];
/// 4.1: a vignette holds 1..CARDS_MAX cards.
pub const CARDS_MAX: usize = 24;
/// 4.2: the longest card title, subtitle and text (characters).
pub const CARD_TITLE_MAX: usize = 48;
pub const CARD_SUBTITLE_MAX: usize = 80;
pub const CARD_TEXT_MAX: usize = 360;
/// 4.2: the card timings (ms): fades 0..FADE_MS_MAX, holds 0..HOLD_MS_MAX (0 waits for the player).
pub const FADE_MS_MAX: i64 = 3000;
pub const HOLD_MS_MAX: i64 = 20000;
pub const FADE_IN_MS: i64 = 600;
pub const FADE_OUT_MS: i64 = 400;
/// 4.1: the default frame (letterbox and fade) colour, also a card backdrop's default top.
pub const FRAME_COLOUR: &str = "#0a0b10";
/// 4.1: a vignette's `music` that means silence ("" keeps the music playing; anything else is a track, STORYMUSIC1: or
/// a music key).
pub const MUSIC_NONE: &str = "none";
/// 4.1: a vignette's music track ends with this (a path under the asset layers' `audio/`).
pub const MUSIC_EXT: &str = ".ogg";
/// 4.1, 4.2: the asset folder music tracks and card cues sit under.
pub const AUDIO_DIR: &str = "audio/";
/// STORYMUSIC1: the folder under `AUDIO_DIR` a music key's track sits in (`music/<key>.ogg`).
pub const MUSIC_DIR: &str = "music/";
/// W3: the token kinds of a card text, `{flag:KEY}` (the flag's value as text) and `{member:KEY}` (the name of the
/// member or character whose id the flag holds).
pub const TOKEN_KINDS: [&str; 2] = ["flag", "member"];
/// 4.3: a sequence holds 1..STEPS_MAX steps; an effects step 1..STEP_EFFECTS_MAX effects.
pub const STEPS_MAX: usize = 40;
pub const STEP_EFFECTS_MAX: usize = 8;
/// 4.3: the step kinds.
pub const STEP_KINDS: [&str; 5] = ["vignette", "fight", "effects", "enter", "end"];
/// 4.3: the step kinds that may only be the last step.
pub const LAST_STEP_KINDS: [&str; 2] = ["enter", "end"];
/// 4.3, Q5: what a lost or fled story fight does (`abort` the default).
pub const FIGHT_OUTCOMES: [&str; 3] = ["retry", "abort", "continue"];
/// 4.3, Q7: where an `end` step goes (`map` the default).
pub const END_TO: [&str; 2] = ["map", "menu"];
/// 4.7: the story effect kinds and their keys besides `effect`, all three in dialogue choices; meter rules,
/// checkpoints and triggers take `vignette` and `sequence`; a sequence's effects step takes `vignette` only.
pub const STORY_EFFECTS: [(&str, &[&str]); 3] = [("vignette", &["vignette"]), ("sequence", &["sequence"]), ("open_board", &["board"])];
/// T1.2-S2 (FB-54b, C25): the choice-time dialogue roll, `{"effect": "roll", "stat", "dc", "key", "who"}`: on choosing,
/// one draw `1..dialogue_roll_die` from the save's `dialogue` roll stream plus the stat (the active party's best, or
/// the active member `who`'s) against `dc`; flag `key` is set true (success) or false. A choice effect only.
pub const ROLL_EFFECT: &str = "roll";
pub const ROLL_EFFECT_KEYS: [&str; 4] = ["stat", "dc", "key", "who"];
/// T1.2-S2: the world manifest's `dialogue_roll_die` (-1: the nearest parent world's): the default and the range.
pub const DIALOGUE_ROLL_DIE: i64 = 20;
pub const DIALOGUE_ROLL_DIE_MIN: i64 = 6;
pub const DIALOGUE_ROLL_DIE_MAX: i64 = 100;
/// T1.2-S3 (FB-54c): the U1 standing ratings a dialogue stat check reads by name besides `STATS` and the derived-stat
/// rows of the world's ruleset: Initiative, Move Rate and Armor Rating, as the member stands out of battle.
pub const STANDING_RATINGS: [&str; 3] = ["initiative", "move_rate", "armor"];
/// 4.3: the effects a sequence's `effects` step never takes (they open a screen or start a sequence).
pub const SEQUENCE_BANNED_EFFECTS: [&str; 5] = ["fight", "turn_in", "open_shop", "open_board", "sequence"];
/// 4.5: a board quest's section.
pub const TRACKS: [&str; 2] = ["main", "side"];
/// 4.5: the longest Closed line of a board quest; 4.4: the longest board intro.
pub const CLOSED_TEXT_MAX: usize = 80;
pub const BOARD_INTRO_MAX: usize = 160;
/// 4.7: the service line id of the board choice.
pub const BOARD_SERVICE: &str = "board";

/// The keys a step of `kind` takes (`kind` included; an unknown kind takes only `kind`). STORYMUSIC1: every kind takes
/// an optional `music` (as a vignette's: "" keeps the music playing).
pub fn step_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "vignette" => &["kind", "vignette", "music"],
        "fight" => &["kind", "encounter", "on_lost", "on_fled", "music"],
        "effects" => &["kind", "effects", "music"],
        "enter" => &["kind", "location", "anchor", "music"],
        "end" => &["kind", "to", "music"],
        _ => &["kind"],
    }
}

/// STORYMUSIC1: what a vignette's or a sequence step's `music` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicAsk<'a> {
    /// "": the music playing goes on.
    Keep,
    /// `MUSIC_NONE`.
    Silence,
    /// A track under the asset layers' `AUDIO_DIR`, ending `MUSIC_EXT` (`music/town.ogg`).
    Track(&'a str),
    /// A music key: one of the game's music keys (its manifest's `music_keys`: its music contexts and story keys) or a
    /// track's key (`music/<key>.ogg`); the game resolves it through its music chain.
    Key(&'a str),
}

/// STORYMUSIC1: the ask of a `music` value.
pub fn music_ask(music: &str) -> MusicAsk<'_> {
    match music {
        "" => MusicAsk::Keep,
        MUSIC_NONE => MusicAsk::Silence,
        m if m.ends_with(MUSIC_EXT) => MusicAsk::Track(m),
        m => MusicAsk::Key(m),
    }
}

/// W3: a card token's kind and key (`flag:act` -> ("flag", "act")), None for a token that is not `kind:key`.
pub fn split_token(token: &str) -> Option<(&str, &str)> {
    token.split_once(':')
}

/// B9: the flag the first open of board `board` sets (true).
pub fn board_opened_flag(board: &str) -> String {
    format!("board.{board}.opened")
}

/// One step of a story sequence (4.3), read with its defaults.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Vignette { vignette: String },
    /// `on_lost` and `on_fled`: one of FIGHT_OUTCOMES.
    Fight { encounter: String, on_lost: String, on_fled: String },
    Effects { effects: Vec<Value> },
    /// `anchor` "" = the location's entry.
    Enter { location: String, anchor: String },
    /// `to`: one of END_TO.
    End { to: String },
}

impl Step {
    /// The step `o` holds; None for an unknown kind.
    pub fn from_obj(o: &Obj) -> Option<Self> {
        Some(match string(o, "kind", "").as_str() {
            "vignette" => Step::Vignette { vignette: string(o, "vignette", "") },
            "fight" => Step::Fight { encounter: string(o, "encounter", ""), on_lost: string(o, "on_lost", "abort"), on_fled: string(o, "on_fled", "abort") },
            "effects" => Step::Effects { effects: array(o, "effects").to_vec() },
            "enter" => Step::Enter { location: string(o, "location", ""), anchor: string(o, "anchor", "") },
            "end" => Step::End { to: string(o, "to", "map") },
            _ => return None,
        })
    }
}

// ---- Portrait cues (PORTRAITFX1 5.1, 5.4, kit hook; engine-only, no Godot original) ---------------------------------

/// 5.1: the catalog kind of the cue rows (file `data/catalog/portrait_cues.json`, read beside the catalog kinds).
pub const PORTRAIT_CUES_KIND: &str = "portrait_cues";
/// 5.1: what a row answers: `event` rows react to a battle step event; the others are state rows that hold while
/// their match is true (PORTRAITFX2 5.3: `unspent` while a party member has unspent points).
pub const PORTRAIT_TRIGGERS: [&str; 7] = ["event", "status", "condition", "hp", "derived", "down", "unspent"];
/// 5.1: the trigger reserved for unit-scope meters (refused until CAMPAIGN1-DOC defines them).
pub const PORTRAIT_METER_TRIGGER: &str = "meter";
/// 5.1, 5.3: the battle events an `event` row matches (PORTRAITFX2 5.3: the battle's start, an ambush, a round's
/// start (classic), a skill's wind-up, an ally going down, the player's win).
pub const PORTRAIT_EVENTS: [&str; 11] = ["hit", "crit", "guard_hit", "heal", "break", "battle_start", "ambush", "round_start", "skill", "ally_down", "victory"];
/// PORTRAITFX2 5.3: the event whose rows read `min_level` (the skill's level).
pub const PORTRAIT_SKILL_EVENT: &str = "skill";
/// 5.1: the kit's copy of the engine's portrait effect names (`aegis_ui::portrait_fx::EFFECTS`, same order; the kit
/// crates do not depend on the UI, so a game test keeps the two equal).
pub const PORTRAIT_EFFECTS: [&str; 12] = ["flash", "jolt", "desaturate", "tint", "fade", "still", "glow", "veins", "eye_glow", "ash", "smoke", "face"];
/// 5.1: the jolt presets (`aegis_ui::portrait_fx::PORTRAIT_JOLTS`).
pub const PORTRAIT_JOLTS: [&str; 2] = ["hit", "crit"];
/// 5.1: the effects whose colour a row may set.
pub const PORTRAIT_COLOR_EFFECTS: [&str; 7] = ["flash", "tint", "glow", "veins", "eye_glow", "ash", "smoke"];
/// 5.1, 6.2: the screens that draw portraits (a row's `surfaces`; none listed = all).
pub const PORTRAIT_SURFACES: [&str; 4] = ["dialogue", "roster", "panel", "timeline"];
/// PORTRAITFX2 FL1: the plain face (`face_chain`'s fallback frame; "" draws it too).
pub const NEUTRAL: &str = "neutral";
/// PORTRAITFX2 FL1: the faces of a game without `portrait_expressions` rows (other than `neutral`'s): the names of a
/// portrait's face frames (`portraits/<folder>/faces/<id>/<face>.png`), a `face` row's `expression`, and a dialogue
/// node's or `line` card's `expression` ("" = neutral). A game's own list is `Catalog::faces` (`faces_of`).
pub const DEFAULT_FACES: [&str; 4] = [NEUTRAL, "hurt", "angry", "afraid"];

/// PORTRAITFX2 FL1: the face list of the `portrait_expressions` row ids `ids` (file order): `neutral`, then each id
/// once (a `neutral` row adds nothing); with no id but `neutral`'s, `DEFAULT_FACES`.
pub fn faces_of<'a>(ids: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut out = vec![NEUTRAL.to_string()];
    for id in ids {
        if !id.is_empty() && !out.iter().any(|f| f == id) {
            out.push(id.to_string());
        }
    }
    if out.len() == 1 {
        return DEFAULT_FACES.iter().map(|f| f.to_string()).collect();
    }
    out
}
/// ROSTER1 W1: the member conditions the kit knows (a cue row naming another condition warns).
pub const MEMBER_CONDITIONS: [&str; 1] = ["wounded"];

/// A portrait cue row (PORTRAITFX1 5.1, kind `portrait_cues`): a trigger and its match, the effect it fires or holds
/// and the effect's numbers, and where it applies. Read from a loaded (checked) row; the defaults are the schema's.
#[derive(Debug, Clone, PartialEq)]
pub struct PortraitCue {
    pub id: String,
    /// One of PORTRAIT_TRIGGERS.
    pub trigger: String,
    /// `event` rows: one of PORTRAIT_EVENTS.
    pub event: String,
    /// `status` rows: a status id.
    pub status: String,
    /// `condition` rows: a member condition id.
    pub condition: String,
    /// `derived` rows: a derived stat id.
    pub stat: String,
    /// `hp`: HP percent of the max (strictly below); `derived`: an absolute value (strictly below); 0 = not used.
    pub below: i32,
    /// `derived`: the drop in percent from the value without statuses (at least); 0 = not used.
    pub drop_pct: i32,
    /// One of PORTRAIT_EFFECTS.
    pub effect: String,
    pub intensity: f64,
    /// Event rows: how long the effect runs (0 = the jolt preset's own); state rows: 0.
    pub ms: i32,
    /// `jolt`: one of PORTRAIT_JOLTS.
    pub preset: String,
    /// "" (the effect's default) or `#rrggbb`.
    pub color: String,
    /// `face`: a face of the game (`Catalog::faces`).
    pub expression: String,
    /// `face`: the highest wins.
    pub priority: i32,
    /// None listed = every surface.
    pub surfaces: Vec<String>,
    /// None listed = every ruleset.
    pub rulesets: Vec<String>,
    /// PORTRAITFX2 5.3: `skill` event rows fire for a skill of at least this level (0 = any skill); else 0.
    pub min_level: i32,
}

impl PortraitCue {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            trigger: string(o, "trigger", "event"),
            event: string(o, "event", ""),
            status: string(o, "status", ""),
            condition: string(o, "condition", ""),
            stat: string(o, "stat", ""),
            below: int(o, "below", 0),
            drop_pct: int(o, "drop_pct", 0),
            effect: string(o, "effect", "flash"),
            intensity: float(o, "intensity", 1.0),
            ms: int(o, "ms", 0),
            preset: string(o, "preset", ""),
            color: string(o, "color", ""),
            expression: string(o, "expression", ""),
            priority: int(o, "priority", 50),
            surfaces: strings(o, "surfaces"),
            rulesets: strings(o, "rulesets"),
            min_level: int(o, "min_level", 0),
        }
    }

    /// An `event` row (fired by a step event); every other trigger holds a state.
    pub fn is_event(&self) -> bool {
        self.trigger == "event"
    }

    /// Whether the row applies on `surface` (none listed = every surface).
    pub fn on_surface(&self, surface: &str) -> bool {
        self.surfaces.is_empty() || self.surfaces.iter().any(|s| s == surface)
    }

    /// Whether the row applies under `ruleset` (none listed = every ruleset).
    pub fn in_ruleset(&self, ruleset: &str) -> bool {
        self.rulesets.is_empty() || self.rulesets.iter().any(|r| r == ruleset)
    }
}

// ---- Expression effects (PORTRAITFX1-EX, kit hook; engine-only, no Godot original) ---------------------------------

/// PORTRAITFX1-EX: the catalog kind of the expression-effect rows (file `data/catalog/portrait_expressions.json`,
/// read beside the catalog kinds); a row's id is the face it draws for (PORTRAITFX2 FL1: the rows name the game's
/// faces; a `neutral` row carries only the plain face's motion).
pub const PORTRAIT_EXPRESSIONS_KIND: &str = "portrait_expressions";
/// The kit's copy of the engine's mark names (`aegis_ui::portrait_fx::MARKS`; a game test keeps the two equal).
pub const PORTRAIT_MARKS: [&str; 2] = ["vein", "sweat"];
/// The boxes a mark's anchor is relative to (`aegis_ui::portrait_fx::MARK_BOXES`).
pub const PORTRAIT_MARK_BOXES: [&str; 2] = ["face", "eyes"];

/// An expression-effect row (PORTRAITFX1-EX, kind `portrait_expressions`): what a portrait draws while a `face` cue
/// holds the expression `id`: a tint over the sidecar's face box, a code-drawn mark anchored to the face or eye box, a
/// short shake. Read from a loaded (checked) row; the defaults are the schema's.
#[derive(Debug, Clone, PartialEq)]
pub struct PortraitExpression {
    /// The face (PORTRAITFX2 FL1; `neutral`: motion fields only).
    pub id: String,
    /// "" (no tint) or `#rrggbb`, its strength 0.05-1.0, its length (0 holds; above 0 fades out over that many ms).
    pub tint: String,
    pub tint_strength: f64,
    pub tint_ms: i32,
    /// "" (no mark) or one of PORTRAIT_MARKS, its colour ("" the mark's default), its box (PORTRAIT_MARK_BOXES), its
    /// anchor in units of the box, its width as a share of the portrait's side, its pulse period (0 steady).
    pub mark: String,
    pub mark_color: String,
    pub mark_box: String,
    pub mark_x: f64,
    pub mark_y: f64,
    pub mark_size: f64,
    pub mark_pulse_ms: i32,
    /// A shake of `shake_px` logical px over `shake_ms` from the expression's start (0: none).
    pub shake_px: i32,
    pub shake_ms: i32,
    /// PORTRAITFX2-M: the face's motion profile (px at a 240 px portrait; the defaults are the plain breath): the
    /// breath's depth, length, share breathing in and jitter; a lean with it; a bob; an entry move (dy, stretch, ms);
    /// a held rest offset reached over `rest_ms`; the settle after the entry; shiver bursts (px, ms, gap); whether it
    /// blinks; the tint's pulse with the breath.
    pub breath_amp: f64,
    pub breath_ms: i32,
    pub breath_in: f64,
    pub breath_jitter: f64,
    pub lean_px: i32,
    pub bob_px: i32,
    pub bob_ms: i32,
    pub enter_dy: i32,
    pub enter_stretch: i32,
    pub enter_ms: i32,
    pub rest_dy: i32,
    pub rest_ms: i32,
    pub settle_ms: i32,
    pub shiver_px: i32,
    pub shiver_ms: i32,
    pub shiver_gap_ms: i32,
    pub blink: bool,
    pub tint_breath: f64,
}

/// PORTRAITFX2-M: the motion fields of a `portrait_expressions` row, in schema order.
pub const PORTRAIT_MOTION_KEYS: [&str; 18] = [
    "breath_amp", "breath_ms", "breath_in", "breath_jitter", "lean_px", "bob_px", "bob_ms", "enter_dy", "enter_stretch", "enter_ms", "rest_dy", "rest_ms", "settle_ms", "shiver_px", "shiver_ms", "shiver_gap_ms", "blink", "tint_breath",
];

impl PortraitExpression {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            tint: string(o, "tint", ""),
            tint_strength: float(o, "tint_strength", 0.3),
            tint_ms: int(o, "tint_ms", 0),
            mark: string(o, "mark", ""),
            mark_color: string(o, "mark_color", ""),
            mark_box: string(o, "mark_box", "face"),
            mark_x: float(o, "mark_x", 0.85),
            mark_y: float(o, "mark_y", 0.15),
            mark_size: float(o, "mark_size", 0.12),
            mark_pulse_ms: int(o, "mark_pulse_ms", 0),
            shake_px: int(o, "shake_px", 0),
            shake_ms: int(o, "shake_ms", 0),
            breath_amp: float(o, "breath_amp", 2.88),
            breath_ms: int(o, "breath_ms", 3600),
            breath_in: float(o, "breath_in", 0.5),
            breath_jitter: float(o, "breath_jitter", 0.0),
            lean_px: int(o, "lean_px", 0),
            bob_px: int(o, "bob_px", 0),
            bob_ms: int(o, "bob_ms", 0),
            enter_dy: int(o, "enter_dy", 0),
            enter_stretch: int(o, "enter_stretch", 0),
            enter_ms: int(o, "enter_ms", 0),
            rest_dy: int(o, "rest_dy", 0),
            rest_ms: int(o, "rest_ms", 0),
            settle_ms: int(o, "settle_ms", 0),
            shiver_px: int(o, "shiver_px", 0),
            shiver_ms: int(o, "shiver_ms", 0),
            shiver_gap_ms: int(o, "shiver_gap_ms", 0),
            blink: boolean(o, "blink", true),
            tint_breath: float(o, "tint_breath", 0.0),
        }
    }
}

// ---- Combat hooks (HOOKS1_TASKS 1.1-1.6, kit hook; engine-only, no Godot original) ---------------------------------

/// GH1: the catalog kind of the ground hazard rows (file `data/catalog/hazards.json`).
pub const HAZARDS_KIND: &str = "hazards";
/// UM1: the catalog kind of the unit meter rows (file `data/catalog/unit_meters.json`).
pub const UNIT_METERS_KIND: &str = "unit_meters";
/// ZN1: the catalog kind of the data zone rows (file `data/catalog/zones.json`).
pub const ZONES_KIND: &str = "zones";

/// FM1: a forced move's kind, origin and condition.
pub const FORCED_KINDS: [&str; 2] = ["push", "pull"];
pub const FORCED_FROM: [&str; 2] = ["source", "center"];
pub const FORCED_WHEN: [&str; 3] = ["always", "moved", "windup"];
/// FM1: the keys of a skill's `forced_move`.
pub const FORCED_MOVE_KEYS: [&str; 8] = ["kind", "tiles", "from", "when", "stop_moves", "collide_mult", "collide_stagger", "collide_break"];
/// FM5: a passive or item `forced_move` effect's `immune` values ("" = none).
pub const FORCED_IMMUNE: [&str; 3] = ["", "always", "guarding"];

/// GH2: the shapes a placed hazard covers (SKILL = the skill's own area cells; the others as `area`, NONE one tile).
/// A3.2-K2: RING and ARC as the skill shapes (W2-4 retired the legacy SWEEP alias).
pub const HAZARD_SHAPES: [&str; 7] = ["SKILL", "NONE", "PLUS", "BURST", "LINE", "RING", "ARC"];
/// GH2: where a hazard is placed.
pub const HAZARD_AT: [&str; 2] = ["target", "self"];
/// GH2: the keys of a skill's or item's `hazard`.
pub const HAZARD_PLACE_KEYS: [&str; 4] = ["id", "shape", "at", "rounds"];
/// GH1: when a hazard row acts, and whom it affects (relative to the placer).
pub const HAZARD_ON: [&str; 3] = ["enter", "round_end", "both"];
pub const HAZARD_AFFECTS: [&str; 3] = ["all", "foes", "friends"];

/// 8.1: the events a trigger answers.
pub const TRIGGER_EVENTS: [&str; 15] = [
    "hit_taken", "guarded_hit", "ally_hit", "hit_dealt", "kill", "ally_downed", "broken", "break_dealt", "evaded", "heal_given", "cleanse_given", "hp_below", "guard_start", "round_start", "round_end",
];
/// 1.1: the events a skill's own `triggers` answer (while its command runs).
pub const SKILL_TRIGGER_EVENTS: [&str; 5] = ["hit_taken", "guarded_hit", "ally_hit", "evaded", "broken"];
/// 8.2: the events whose `when` takes `range` (a friend within range).
pub const ALLY_EVENTS: [&str; 2] = ["ally_hit", "ally_downed"];
/// 8.1: how often a trigger fires (`round` the default).
pub const TRIGGER_LIMITS: [&str; 3] = ["round", "battle", "none"];
/// 1.3: the keys of a trigger row (a passive or item effect adds `type`; a rider takes `when` and `effects` only).
pub const TRIGGER_KEYS: [&str; 4] = ["on", "when", "effects", "limit"];
pub const RIDER_KEYS: [&str; 2] = ["when", "effects"];
/// 8.2: the condition keys of a trigger's or rider's `when`.
pub const WHEN_KEYS: [&str; 10] = ["pct", "range", "ranged", "melee", "skills", "other_hp_below", "other_broken", "other_status", "intent", "meter"];
/// 8.2: the trigger effects and their keys besides `type` (`to`: self or other, where the effect takes it). ROLLTABLE1:
/// `roll_table` picks one of its `rows` by weight on the battle RNG and runs that row's effects. PK1-S1: `ap` takes
/// `set` (with `rounds`) instead of `value`, `damage` takes `pct_of_hit` instead of `value`, and `strike_nearest` and
/// `mark` join.
pub const TRIGGER_EFFECTS: [(&str, &[&str]); 18] = [
    ("ap", &["value", "set", "rounds", "to"]),
    ("grant", &["status", "rounds", "magnitude", "until", "to"]),
    ("inflict", &["status", "potency", "rounds", "magnitude", "to"]),
    ("guard", &["mitigation", "blocks", "to"]),
    ("redirect", &["pct"]),
    ("move", &["kind", "tiles", "to"]),
    ("meter", &["meter", "delta", "to"]),
    ("crit", &["value", "to"]),
    ("resist", &["status", "value", "to"]),
    ("stagger", &["value", "to"]),
    ("damage", &["value", "pct_of_hit", "to"]),
    ("reveal", &["rounds"]),
    ("end_guard", &["to"]),
    ("stop_moves", &["to"]),
    (ROLL_TABLE, &["rows"]),
    (STRIKE_NEAREST, &["power", "then_status", "then_rounds"]),
    (MARK, &["mark", "value", "to"]),
    (RATING_SET, &["rating", "value", "rounds", "to"]),
];
/// PK1-KITFIX (b, FB-184 A): a rating held at a value `{rating, value, rounds}` (rounds 1-5 or -1 the rest of the
/// battle): while it holds, the rating is the value whatever the statuses raising or lowering it add (a later set of
/// the same rating replaces it).
pub const RATING_SET: &str = "rating_set";
/// PK1-KITFIX (b): the ratings a `rating_set` names (the status table's `rating` words; `guard` is not one: a guard
/// comes from the Defend command, held guards and the guard status, so it has no single value to hold) and each one's
/// value range: `armor` the Armor Rating (points), `attack_power` the statuses' attack factor (a percent: 100 = as
/// without them), `move_rate` the tiles a Walk moves per block.
pub const RATING_SET_RANGES: [(&str, (i64, i64)); 3] = [("armor", (0, 200)), ("attack_power", (0, 300)), ("move_rate", (1, 9))];
/// PK1-S1 (docs/sessions/PK1_CORRUPTION.md 3.3): a weapon strike resolved at once on the nearest other unit of any side
/// (no AP, no plan), then an optional status on the striker for its next rounds.
pub const STRIKE_NEAREST: &str = "strike_nearest";
/// PK1-S1 (3.5): a mark `{mark, value}` the battle records on a unit for the world settle (`HeroEnd::marks`).
pub const MARK: &str = "mark";
/// PK1-S1 (3.7): a consumable's `unmark` `{mark}` removes the target's latest mark of that id this battle; its keys.
pub const UNMARK_KEYS: [&str; 1] = ["mark"];
/// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): the member grants of the passive grammar (passives only; gear, bands and
/// meters may not carry them): `forbid_hands` `{hands: 2}` (equipping refuses items held in that many hands; a worn one
/// goes to the stash at the settle) and `pierce_melee` `{}` (melee and bare-handed hits pierce armour, rule pierce).
pub const FORBID_HANDS: &str = "forbid_hands";
pub const PIERCE_MELEE: &str = "pierce_melee";
pub const MEMBER_GRANTS: [&str; 2] = [FORBID_HANDS, PIERCE_MELEE];
/// PK2-S1: the `hands` a `forbid_hands` grant may name (two-handed items only so far).
pub const FORBID_HANDS_VALUES: [i64; 1] = [2];
/// PK1-S1 (3.2): the events whose trigger knows the hit it answers (a `damage` effect's `pct_of_hit` reads it; a rider
/// reads the hit it rode on).
pub const HIT_EVENTS: [&str; 4] = ["hit_taken", "guarded_hit", "hit_dealt", "kill"];
/// ROLLTABLE1 (ACTS23 PK1 hook a): the weighted random-table trigger effect, the keys of one of its `rows`, and a row's
/// weight range (no `roll_table` inside a row).
pub const ROLL_TABLE: &str = "roll_table";
pub const ROLL_ROW_KEYS: [&str; 2] = ["weight", "effects"];
pub const ROLL_WEIGHT: (i64, i64) = (1, 1000);
/// 8.2: whom an effect acts on.
pub const EFFECT_TO: [&str; 2] = ["self", "other"];
/// 8.2: a `grant` effect's `until` ("" = its rounds).
pub const GRANT_UNTIL: [&str; 2] = ["", "guard_end"];
/// UM5: the effect a meter rule adds to the trigger effects, and its keys besides `type`.
pub const METER_SET: &str = "meter_set";
pub const METER_SET_KEYS: [&str; 2] = ["meter", "value"];

/// AT1: the unit `target_pref` values (HOOKS1 adds SOFTEST and TOUGHEST, read under the rule ai_tactics).
pub const TARGET_PREFS: [&str; 5] = ["ROLE", "NEAREST", "WEAKEST", "SOFTEST", "TOUGHEST"];
/// AT2: the AI tactics a unit or a phase `ai` effect may list.
pub const TACTICS: [&str; 3] = ["brace_after_hit", "hold_line", "flush_cover"];

/// UM1: how a meter is shown, and the most pips a `pips` meter draws.
pub const METER_SHOW: [&str; 3] = ["pips", "bar", "hidden"];
pub const METER_PIPS_MAX: i32 = 10;
/// UM4: the passive effect types a meter band's `effects` take (`crit` as the trigger effect). PK1-S1 (3.1): the
/// band-only `heal_taken` (a percent added to the healing the holder receives; no passive or item takes it).
pub const BAND_EFFECTS: [&str; 7] = ["stat", "resist", "stagger_taken", "skill_potency", "status_potency", "crit", HEAL_TAKEN];
pub const HEAL_TAKEN: &str = "heal_taken";

/// AH2: the directions of an arena hazard's push.
pub const ARENA_DIRS: [&str; 4] = ["east", "west", "north", "south"];
/// AH1: the keys of an arena hazard row and of its `effect`.
pub const ARENA_HAZARD_KEYS: [&str; 9] = ["id", "name", "cells", "columns", "rows", "rounds", "blocks", "effect", "line"];
/// PK1-S1 (3.6): `meter` `{meter, delta}` changes that unit meter of each struck holder (the ground hazard row's `meter`
/// field too).
pub const ARENA_EFFECT_KEYS: [&str; 9] = ["damage", "damage_pct", "stagger", "status", "potency", "magnitude", "status_rounds", "push", "meter"];
/// PK1-S1 (3.6): the keys of a hazard's `meter` (arena effect and ground hazard row).
pub const HAZARD_METER_KEYS: [&str; 2] = ["meter", "delta"];

/// DU1: the phase effect that changes a unit's deck mid-battle (rule deck), and its keys.
pub const DECK_EFFECT: &str = "deck";
pub const DECK_EFFECT_KEYS: [&str; 4] = ["type", "unit", "add", "order"];

/// CN1: the keys of a world manifest's `classes` entry.
pub const CLASS_OVERLAY_KEYS: [&str; 3] = ["name", "description", "skill_path"];

/// The objects of the list `key` (entries that are not objects are left out).
fn objs(o: &Obj, key: &str) -> Vec<Obj> {
    array(o, key).iter().filter_map(|v| v.as_object().cloned()).collect()
}

/// The numeric values of an object as floats, in its key order (other values left out).
pub(crate) fn float_map(m: &Obj) -> IndexMap<String, f64> {
    m.iter().filter_map(|(k, v)| v.as_f64().map(|x| (k.clone(), x))).collect()
}

/// FM1: a skill's forced move (`forced_move`), read with its defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct ForcedMove {
    /// One of FORCED_KINDS.
    pub kind: String,
    pub tiles: i32,
    /// One of FORCED_FROM.
    pub from: String,
    /// One of FORCED_WHEN.
    pub when: String,
    pub stop_moves: bool,
    pub collide_mult: f64,
    pub collide_stagger: i32,
    pub collide_break: bool,
}

impl ForcedMove {
    /// None for an empty object (no forced move).
    pub fn from_obj(o: &Obj) -> Option<Self> {
        if o.is_empty() {
            return None;
        }
        Some(Self {
            kind: string(o, "kind", "push"),
            tiles: int(o, "tiles", 1),
            from: string(o, "from", "source"),
            when: string(o, "when", "always"),
            stop_moves: boolean(o, "stop_moves", false),
            collide_mult: float(o, "collide_mult", 0.0),
            collide_stagger: int(o, "collide_stagger", 0),
            collide_break: boolean(o, "collide_break", false),
        })
    }
}

/// GH2: where a skill or item places a hazard (`hazard`), read with its defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct HazardPlace {
    /// A `hazards` row id.
    pub id: String,
    /// One of HAZARD_SHAPES.
    pub shape: String,
    /// One of HAZARD_AT.
    pub at: String,
    /// 0 = the row's `duration`.
    pub rounds: i32,
}

impl HazardPlace {
    /// None for an empty object (no hazard).
    pub fn from_obj(o: &Obj) -> Option<Self> {
        if o.is_empty() {
            return None;
        }
        Some(Self { id: string(o, "id", ""), shape: string(o, "shape", "NONE"), at: string(o, "at", "target"), rounds: int(o, "rounds", 0) })
    }
}

/// 1.3: a trigger row (a skill's `triggers`, a passive or item `trigger` effect): the event, the conditions and the
/// effects (raw; HOOKS1-E reads them), how often it fires.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerRow {
    /// One of TRIGGER_EVENTS.
    pub on: String,
    pub when: Obj,
    pub effects: Vec<Obj>,
    /// One of TRIGGER_LIMITS.
    pub limit: String,
}

impl TriggerRow {
    pub fn from_obj(o: &Obj) -> Self {
        Self { on: string(o, "on", ""), when: obj(o, "when").cloned().unwrap_or_default(), effects: objs(o, "effects"), limit: string(o, "limit", "round") }
    }
}

/// RD1: a skill's rider: conditions and effects run per unit the skill landed on (`to` defaults to that unit).
#[derive(Debug, Clone, PartialEq)]
pub struct Rider {
    pub when: Obj,
    pub effects: Vec<Obj>,
}

impl Rider {
    pub fn from_obj(o: &Obj) -> Self {
        Self { when: obj(o, "when").cloned().unwrap_or_default(), effects: objs(o, "effects") }
    }
}

/// UM2: a skill's meter change (`meters` entry).
#[derive(Debug, Clone, PartialEq)]
pub struct MeterDelta {
    pub meter: String,
    pub delta: i32,
}

impl MeterDelta {
    pub fn from_obj(o: &Obj) -> Self {
        Self { meter: string(o, "meter", ""), delta: int(o, "delta", 0) }
    }

    /// PK1-S1 (3.6): a hazard's `meter` object (None when absent, empty, or without a meter id or a delta).
    pub fn of_hazard(o: &Obj) -> Option<Self> {
        obj(o, "meter").map(Self::from_obj).filter(|m| !m.meter.is_empty() && m.delta != 0)
    }
}

/// PK1-S2a (PK1 4.3 item 5, rule unit_meters): a skill's `requires_meter`: it starts only while the unit holds `meter`
/// at `min` or more (Vent's Strain 50).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeterGate {
    pub meter: String,
    pub min: i32,
}

impl MeterGate {
    /// The object's keys.
    pub const KEYS: [&'static str; 2] = ["meter", "min"];

    /// None for an empty object or one without a meter id.
    pub fn from_obj(o: &Obj) -> Option<Self> {
        Some(Self { meter: string(o, "meter", ""), min: int(o, "min", 0) }).filter(|g| !g.meter.is_empty())
    }
}

/// PK1-S2a (PK1 4.1, rule channel): a skill's `channel` block: `none` (no channeled version), `only` (channeled only: open
/// only in a Channel round), `cost` (replaces the derived channel meter cost), `power` (replaces `channel_power`) and
/// `effects` (trigger effects run as a rider on each unit the channeled version lands on).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChannelSpec {
    pub none: bool,
    pub only: bool,
    pub cost: Option<i32>,
    pub power: Option<f64>,
    pub effects: Vec<Obj>,
}

impl ChannelSpec {
    /// The block's keys, in their stored order.
    pub const KEYS: [&'static str; 5] = ["none", "only", "cost", "effects", "power"];

    pub fn from_obj(o: &Obj) -> Self {
        Self {
            none: boolean(o, "none", false),
            only: boolean(o, "only", false),
            cost: o.contains_key("cost").then(|| int(o, "cost", 0)),
            power: o.contains_key("power").then(|| float(o, "power", 1.0)),
            effects: objs(o, "effects"),
        }
    }
}

/// GH1: a ground hazard row (kind `hazards`): a timed tile overlay with terrain-like numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct HazardDef {
    pub id: String,
    pub name: String,
    pub description: String,
    /// One of HAZARD_ON.
    pub on: String,
    /// A debuff id ("" = none) and its roll.
    pub status: String,
    pub potency: i32,
    pub magnitude: i32,
    pub status_rounds: i32,
    /// Flat damage, through guard, not armour.
    pub damage: i32,
    pub cover_guard: f64,
    /// Added to the terrain's move cost.
    pub move_cost: i32,
    /// One of HAZARD_AFFECTS.
    pub affects: String,
    pub trap: bool,
    pub hidden: bool,
    pub stop_moves: bool,
    /// Rounds it lasts when the placement sets none.
    pub duration: i32,
    /// `#rrggbb`.
    pub tint: String,
    pub glyph: String,
    /// PK1-S1 (3.6): the unit meter change it makes on each holder of that meter it acts on (None = none).
    pub meter: Option<MeterDelta>,
}

impl HazardDef {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", "New Hazard"),
            description: string(o, "description", ""),
            on: string(o, "on", "both"),
            status: string(o, "status", ""),
            potency: int(o, "potency", 100),
            magnitude: int(o, "magnitude", 0),
            status_rounds: int(o, "status_rounds", 1),
            damage: int(o, "damage", 0),
            cover_guard: float(o, "cover_guard", 0.0),
            move_cost: int(o, "move_cost", 0),
            affects: string(o, "affects", "all"),
            trap: boolean(o, "trap", false),
            hidden: boolean(o, "hidden", false),
            stop_moves: boolean(o, "stop_moves", false),
            duration: int(o, "duration", 2),
            tint: string(o, "tint", "#808080"),
            glyph: string(o, "glyph", "*"),
            meter: MeterDelta::of_hazard(o),
        }
    }
}

/// U1.2-S5: the largest change one `meter` effect makes to a meter by default (a row's `delta_max`).
pub const METER_DELTA_MAX: i32 = 10;
/// U1.2-S5: the largest `delta_max` a meter row may set.
pub const METER_DELTA_CAP: i32 = 100;

/// UM1: a unit meter row (kind `unit_meters`): its range and start, how it shows, its decay, bands and rules (raw;
/// HOOKS1-M reads them) and the meter skills it opens. U1.2-S5 (a pool from data, U1 "Adding a derived value without a
/// logic change"): `max_stat` names a derived-stat row whose value is each holder's max (clamped to min..max; "" = the
/// row's max), `start_full` starts each battle at that max, `delta_max` bounds one `meter` effect's change, and
/// `triggers` are trigger rows every holder holds while it holds the meter.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitMeterDef {
    pub id: String,
    pub name: String,
    pub short: String,
    pub description: String,
    pub min: i32,
    pub max: i32,
    pub start: i32,
    /// One of METER_SHOW.
    pub show: String,
    pub decay: Vec<Obj>,
    pub bands: Vec<Obj>,
    pub rules: Vec<Obj>,
    pub skills: Vec<String>,
    pub max_stat: String,
    pub start_full: bool,
    pub delta_max: i32,
    pub triggers: Vec<TriggerRow>,
}

impl UnitMeterDef {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", "New Meter"),
            short: string(o, "short", ""),
            description: string(o, "description", ""),
            min: int(o, "min", 0),
            max: int(o, "max", 10),
            start: int(o, "start", 0),
            show: string(o, "show", "pips"),
            decay: objs(o, "decay"),
            bands: objs(o, "bands"),
            rules: objs(o, "rules"),
            skills: strings(o, "skills"),
            max_stat: string(o, "max_stat", ""),
            start_full: boolean(o, "start_full", false),
            delta_max: int(o, "delta_max", METER_DELTA_MAX),
            triggers: objs(o, "triggers").iter().map(TriggerRow::from_obj).collect(),
        }
    }
}

/// REGIONGROUND1 KT 1.1: the catalog kind of the ground rows (file `data/catalog/grounds.json`).
pub const GROUNDS_KIND: &str = "grounds";
/// KT 1.1: a ground's `kind` that is sunken and belongs on `~` cells.
pub const GROUND_WATER: &str = "water";

/// KT 1.1: a ground row (kind `grounds`): what a layer of a region map's picture is. Presentation only: its art comes
/// from the location's tile sets (`art::ground_dirs`), its numbers go to the engine's ground composer.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundDef {
    pub id: String,
    pub name: String,
    pub order: i32,
    pub kind: String,
    pub colour: String,
    pub wear: f64,
    pub strays: bool,
    pub tone: f64,
    pub patches: f64,
    pub marks: Vec<String>,
    pub marks_per_100: i32,
}

impl GroundDef {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", ""),
            order: int(o, "order", -1),
            kind: string(o, "kind", "land"),
            colour: string(o, "colour", ""),
            wear: float(o, "wear", 0.1),
            strays: boolean(o, "strays", false),
            tone: float(o, "tone", 0.13),
            patches: float(o, "patches", 0.0),
            marks: strings(o, "marks"),
            marks_per_100: int(o, "marks_per_100", 0),
        }
    }

    /// A water ground: sunken (a rim on its edge), and only on `~` cells (G7).
    pub fn is_water(&self) -> bool {
        self.kind == GROUND_WATER
    }
}

/// ZN1: a data zone row (kind `zones`): its layout (empty = 9 rows of floor), backdrop asset key ("" =
/// `backgrounds/battle/<id>.png`) and the terrain of its fallback gradient.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rows: Vec<String>,
    pub backdrop: String,
    pub ground: String,
}

impl ZoneDef {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", "New Zone"),
            description: string(o, "description", ""),
            rows: strings(o, "rows"),
            backdrop: string(o, "backdrop", ""),
            ground: string(o, "ground", "floor"),
        }
    }
}
