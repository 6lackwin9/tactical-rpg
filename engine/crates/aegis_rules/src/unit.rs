//! Combatant (build-time and derived parts) and ContentFactory.unit: a catalog unit entry -> a ready unit.

use crate::gear::{Equipment, Inventory, Slot, TIER_SOURCE};
use crate::stats::{self, roundi, CharacterStats, StatModifier};
use aegis_data::value::{self, to_int, to_str};
use aegis_data::{Catalog, ClassDef, UnitDef, NO_ITEM};
use indexmap::{IndexMap, IndexSet};
use serde_json::Value;

pub const BASE_AP: i32 = 10;
pub const GEAR_REACH_CAP: i32 = 1;
pub const GROUPS: [&str; 2] = ["spell", "heal"];

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
            passives: Vec::new(),
            st: StatusState::default(),
            is_withdrawn: false,
            active_target: None,
            attack_boost_rounds: 0,
            stance_hold_duration: 0,
            riposted: Vec::new(),
            ap_cost_modifier: 0,
            loadout_locked: false,
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

    /// Passives.apply: once per id, then the path refresh (a KNOW bonus can open an advanced family earlier).
    pub fn apply_passive(&mut self, cat: &Catalog, passive: &aegis_data::PassiveDef) {
        if passive.id.is_empty() || self.passives.contains(&passive.id) {
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
                    self.stats.modifiers.push(StatModifier::permanent(&stat, value::int(e, "value", 0), &format!("passive:{id}"), &tag));
                }
                "proficiency" => self.stats.proficiency.add_bonus(&value::string(e, "gear", ""), value::int(e, "levels", 0)),
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
                "foresight" if !gear && self.foresight_source.is_empty() => self.foresight_source = id.clone(),
                _ => {}
            }
        }
    }

    // ---- Skills: ownership, unlocks (SK2), loadout ---------------------------------------------------------------

    pub fn active_families(&self) -> Vec<String> {
        self.active_class_families.iter().chain(&self.active_extra_families).cloned().collect()
    }

    pub fn owns_skill(&self, cat: &Catalog, skill_id: &str) -> bool {
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
        active.is_empty() || cat.is_always_active(skill_id) || active.iter().any(|f| f == cat.family_of(skill_id))
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
            if !active.is_empty() && !cat.is_always_active(skill_id) && !active.contains(&fam) && self.active_extra_families.len() < stats::MAX_EXTRA_SLOTS {
                self.active_extra_families.push(fam);
            }
        }
    }
}

/// SkillDef.in_group for the passive groups: "spell" = an attack that needs a staff or wand; "heal" = a support heal.
pub fn skill_in_group(s: &aegis_data::SkillDef, group: &str) -> bool {
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

    pub fn potency_for_skill(&self, skill: &aegis_data::SkillDef) -> f64 {
        let mut total = self.potency_for(skill.family_id());
        for g in GROUPS {
            if skill_in_group(skill, g) {
                total += self.potency_group.get(g).copied().unwrap_or(0.0) + self.gear_potency_group.get(g).copied().unwrap_or(0.0);
            }
        }
        total
    }

    /// SkillDef.power_for: the skill multiplier x potency x weaken x empower.
    pub fn power_for(&self, skill: &aegis_data::SkillDef) -> f64 {
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

    pub fn skill_unlock_level(&self, cat: &Catalog, skill: &aegis_data::SkillDef) -> i32 {
        stats::MAX_LEVEL.min(self.family_unlock_level(cat, skill.family_id()) + skill.tier_offset())
    }

    /// The level gate of can_use: basics, engine skills and hand-built units are never gated.
    pub fn tier_unlocked(&self, cat: &Catalog, skill: &aegis_data::SkillDef) -> bool {
        if self.path_levels.is_empty() || skill.system || skill.basic {
            return true;
        }
        self.stats.level >= self.skill_unlock_level(cat, skill)
    }

    /// Known, unlocked, and trained for (min_proficiency in the hand's weapon type).
    pub fn can_use(&self, cat: &Catalog, skill: &aegis_data::SkillDef) -> bool {
        if !self.knows_skill(cat, &skill.id) || !self.tier_unlocked(cat, skill) {
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
