//! Typed content catalog (classes, units, skills, items, passives, encounters, terrain).
//!
//! Source today: the resolved catalog the Godot exporter writes (the game's `parity/out/catalog.json`), i.e. exactly the
//! entries Godot's `ContentCatalog.entries(kind)` returns after item-family resolution and schema normalisation.
//! Defaults below are the `ContentSchema` field defaults, so hand-written partial entries normalise the same way.

use aegis_core::value::*;
use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde_json::Value;
use std::path::Path;

pub const STATS: [&str; 6] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"];
pub const LOADOUT_SLOTS: [&str; 6] = ["main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2"];
/// Unit slot value meaning "leave this slot empty" (no class fallback).
pub const NO_ITEM: &str = "none";

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
    pub base_stats: IndexMap<String, i32>,
    pub growth: IndexMap<String, i32>,
    pub proficiency: IndexMap<String, ProfEntry>,
    /// Family head id -> unlock level, in entry order.
    pub skill_path: IndexMap<String, i32>,
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
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", "New Class"),
            art: string(o, "art", "warrior"),
            ai_role: string(o, "ai_role", "AGGRESSOR"),
            base_stats: int_map(o, "base_stats"),
            growth: int_map(o, "growth"),
            proficiency,
            skill_path: int_map(o, "skill_path"),
            loadout,
            pouch: strings(o, "pouch"),
        }
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
    pub tier: String,
    pub spend: IndexMap<String, i32>,
    pub stat_mode: String,
    pub hp: i32,
    pub speed: i32,
    pub attack_range: i32,
    pub ap: i32,
    pub power: i32,
    pub armor: i32,
    pub skill_cap: i32,
    pub extra_skills: Vec<String>,
    pub loadout: Vec<String>,
    pub ai_role: String,
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
            tier: string(o, "tier", "hero"),
            spend: int_map(o, "spend"),
            stat_mode: string(o, "stat_mode", "derived"),
            hp: int(o, "hp", 50),
            speed: int(o, "speed", 12),
            attack_range: int(o, "attack_range", 1),
            ap: int(o, "ap", 0),
            power: int(o, "power", 0),
            armor: int(o, "armor", -1),
            skill_cap: int(o, "skill_cap", 0),
            extra_skills: strings(o, "extra_skills"),
            loadout: strings(o, "loadout"),
            ai_role: string(o, "ai_role", ""),
            target_pref: string(o, "target_pref", "ROLE"),
            flee_ratio: float(o, "flee_ratio", 0.25),
            defend_below: float(o, "defend_below", 0.4),
            prefer_level: int(o, "prefer_level", 0),
            status_resist: int_map(o, "status_resist"),
            reckless: boolean(o, "reckless", false),
            heal_below: float(o, "heal_below", 0.7),
            slots: LOADOUT_SLOTS.iter().map(|s| (s.to_string(), string(o, s, ""))).collect(),
            pouch: strings(o, "pouch"),
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
    pub min_proficiency: i32,
    pub reach_min: i32,
    pub reach_max: i32,
    pub area: String,
    pub power_mult: f64,
    pub weapon_hand: String,
    pub dash_tiles: i32,
    pub status_effect: String,
    pub status_potency: i32,
    pub status_magnitude: i32,
    pub status_rounds: i32,
    pub target_mode: String,
    pub heal_stat: String,
    pub heal_base: i32,
    pub heal_mult: f64,
    pub grant_status: String,
    pub extra_grants: Vec<Value>,
    pub cleanse_statuses: Vec<String>,
    pub mitigation: f64,
    pub boost: f64,
    pub boost_rounds: i32,
    pub jump_tiles: i32,
    pub recover_ap: i32,
    pub ends_in_stance: String,
    pub disrupt: String,
    pub exploit: Vec<Value>,
    pub exploit_consume: bool,
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
            min_proficiency: int(o, "min_proficiency", 0),
            reach_min: int(o, "reach_min", 0),
            reach_max: int(o, "reach_max", 0),
            area: string(o, "area", "NONE"),
            power_mult: float(o, "power_mult", 1.0),
            weapon_hand: string(o, "weapon_hand", "main"),
            dash_tiles: int(o, "dash_tiles", 0),
            status_effect: string(o, "status_effect", ""),
            status_potency: int(o, "status_potency", 0),
            status_magnitude: int(o, "status_magnitude", 0),
            status_rounds: int(o, "status_rounds", 1),
            target_mode: string(o, "target_mode", "ALLY_OR_SELF"),
            heal_stat: string(o, "heal_stat", ""),
            heal_base: int(o, "heal_base", 0),
            heal_mult: float(o, "heal_mult", 0.0),
            grant_status: string(o, "grant_status", ""),
            extra_grants: array(o, "extra_grants").to_vec(),
            cleanse_statuses: strings(o, "cleanse_statuses"),
            mitigation: float(o, "mitigation", 0.0),
            boost: float(o, "boost", 0.0),
            boost_rounds: int(o, "boost_rounds", 0),
            jump_tiles: int(o, "jump_tiles", 0),
            recover_ap: int(o, "recover_ap", 0),
            ends_in_stance: string(o, "ends_in_stance", ""),
            disrupt: string(o, "disrupt", "NONE"),
            exploit: array(o, "exploit").to_vec(),
            exploit_consume: boolean(o, "exploit_consume", false),
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
    pub block_cost: i32,
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
    pub buff_stat: String,
    pub buff_pct: f64,
    pub duration: i32,
    pub use_blocks: i32,
    pub use_reach: i32,
    /// Carry weight (PartyCarry; Item.weight, 1 when unset).
    pub weight: f64,
    pub raw: Obj,
}

impl ItemDef {
    pub fn from_obj(o: &Obj) -> Self {
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
            block_cost: int(o, "block_cost", 1),
            burden: int(o, "burden", 0),
            mitigation: int(o, "mitigation", 0),
            required_stats: int_map(o, "required_stats"),
            stat_bonuses: int_map(o, "stat_bonuses"),
            effects: array(o, "effects").to_vec(),
            threshold_tiers: array(o, "threshold_tiers").iter().filter_map(|v| v.as_object().cloned()).collect(),
            consumable_type: string(o, "consumable_type", "HEALTH_POTION"),
            heal: int(o, "heal", 0),
            damage: int(o, "damage", 0),
            buff_stat: string(o, "buff_stat", ""),
            buff_pct: float(o, "buff_pct", 0.0),
            duration: int(o, "duration", 0),
            use_blocks: int(o, "use_blocks", 1),
            use_reach: int(o, "use_reach", -1),
            weight: float(o, "weight", 1.0),
            raw: o.clone(),
        }
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
    skill_index: IndexMap<String, usize>,
    tier_one: IndexMap<String, usize>,
}

impl Catalog {
    /// Loads a resolved catalog: `{kind: [entry, ...]}` as written by the parity exporter.
    pub fn load_resolved(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let root: Value = serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        Ok(Self::from_value(&root))
    }

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
            let p = PassiveDef { id: string(&o, "id", ""), name: string(&o, "name", ""), effects: array(&o, "effects").to_vec() };
            cat.passives.insert(p.id.clone(), p);
        }
        for o in rows("encounters") {
            cat.encounters.insert(string(&o, "id", ""), o);
        }
        for o in rows("terrain") {
            cat.terrain.insert(string(&o, "id", ""), o);
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
