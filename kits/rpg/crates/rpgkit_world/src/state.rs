//! GameState, PartyState and UnitState (scripts/game_state/game_state.gd, scripts/progression/party_state.gd,
//! unit_state.gd): the saved snapshot. `to_value` writes Godot's to_dict key order; `from_value` reads like
//! from_dict (damaged blobs are kept as load problems that `validate` reports, never coerced silently).

use crate::data::{Data, PARTY_MAX};
use crate::progression;
use aegis_core::json::{self, int, strs, Dict};
use rpgkit_data::STATS;
use indexmap::IndexMap;

// The gear rules live in the rules crate (the battle crate's gear dojo uses them too).
pub use rpgkit_rules::gear::{class_can_equip, item_fits_slot, off_hand_ok};
use serde_json::Value;

pub const GOLD_MAX: i64 = 9_999_999;
pub const SUPPLIES_MAX: i64 = 99;
pub const SUPPLIES_START: i64 = 10;
pub const SEED_MAX: i64 = 2_147_483_647;
pub const POUCH_MAX: usize = 3;
pub const NO_POSITION: (i64, i64) = (-1, -1);
pub const HOME_WORLD_KEY: &str = "home_world";
pub const EXPLORE_KEYS: [&str; 5] = ["steps", "next_at", "loc_steps", "last_safe", "last_rest"];
pub const AUTO_IDS: [&str; 6] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy"];
pub const DEFAULT_TACTIC: &str = "balanced";
pub const LOADOUT_SLOTS: [&str; 6] = ["main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2"];

pub fn is_tactic(id: &str) -> bool {
    AUTO_IDS.contains(&id) || id == "custom"
}

// ---- UnitState -----------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UnitState {
    pub id: String,
    pub base_unit: String,
    pub lore_ref: String,
    pub total_xp: i64,
    pub spend: IndexMap<String, i64>,
    pub learned_skills: Vec<String>,
    pub passives: Vec<String>,
    pub loadout_class: Vec<String>,
    pub loadout_extra: Vec<String>,
    /// SKILLUI1: family -> the rank chosen on the Skills tab; a family without an entry uses its highest unlocked
    /// rank. Saved as `loadout.ranks`, only when non-empty.
    pub loadout_ranks: IndexMap<String, i64>,
    pub equipment: IndexMap<String, String>,
    pub carried: Vec<String>,
    pub pouch: Vec<String>,
    pub hp_lost: i64,
}

impl UnitState {
    pub fn create(id: &str, base_unit: &str) -> Self {
        Self { id: id.into(), base_unit: base_unit.into(), ..Default::default() }
    }

    /// A state of `base_unit` at exactly the XP threshold of `level`.
    pub fn at_level(d: &Data, base_unit: &str, level: i64, id: &str) -> Self {
        let mut s = Self::create(if id.is_empty() { base_unit } else { id }, base_unit);
        s.total_xp = progression::xp_for_level(d, level);
        s
    }

    pub fn level(&self, d: &Data) -> i64 {
        progression::level_for(d, self.total_xp)
    }

    pub fn spent(&self) -> i64 {
        self.spend.values().sum()
    }

    pub fn unspent(&self, d: &Data) -> i64 {
        rpgkit_rules::stats::spend_pool(self.level(d) as i32) as i64 - self.spent()
    }

    /// UnitState._gear_errors for one item: it fits `slot` (ContentFactory.item_fits_slot, worn gear kept) and the
    /// class has training in its weapon or gear type (Proficiency.can_use on the class's own proficiency).
    pub fn can_wear(&self, d: &Data, item: &str, slot: &str) -> Result<(), String> {
        let def = d.cat.items.get(item).ok_or_else(|| format!("unknown item '{item}'."))?;
        if !item_fits_slot(def, slot) {
            return Err(format!("{} can't be equipped in {}.", def.name, slot.replace('_', " ")));
        }
        let class = d.cat.classes.get(&self.class_id(d));
        if !class.is_some_and(|c| class_can_equip(&rpgkit_rules::stats::Proficiency::from_class(c), def)) {
            return Err(format!("{} has no training in {} ({}).", class.map(|c| c.name.as_str()).unwrap_or("This class"), def.weapon_type, def.name));
        }
        Ok(())
    }

    pub fn class_id(&self, d: &Data) -> String {
        json::s(d.entry("units", &self.base_unit), "class_id")
    }

    /// Non-basic families owned at its level: class path, the base unit's extras and learned skills.
    pub fn owned_families(&self, d: &Data) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        if let Some(arch) = d.cat.classes.get(&self.class_id(d)) {
            let lvl = self.level(d) as i32;
            ids.extend(arch.path_steps().into_iter().filter(|(l, _)| *l <= lvl).map(|(_, s)| s));
        }
        ids.extend(json::sget(d.entry("units", &self.base_unit), "extra_skills"));
        ids.extend(self.learned_skills.iter().cloned());
        rpgkit_rules::unit::families_of(&d.cat, &ids)
    }

    pub fn to_value(&self) -> Value {
        let mut clean = Dict::new();
        for stat in STATS {
            let v = self.spend.get(stat).copied().unwrap_or(0);
            if v > 0 {
                clean.insert(stat.into(), int(v));
            }
        }
        let mut equipment = Dict::new();
        for (k, v) in &self.equipment {
            equipment.insert(k.clone(), Value::String(v.clone()));
        }
        let mut loadout = json::obj([("class", strs(&self.loadout_class)), ("extra", strs(&self.loadout_extra))]);
        if !self.loadout_ranks.is_empty() {
            loadout.insert("ranks".into(), Value::Object(self.loadout_ranks.iter().map(|(k, v)| (k.clone(), int(*v))).collect()));
        }
        Value::Object(json::obj([
            ("id", Value::String(self.id.clone())),
            ("base_unit", Value::String(self.base_unit.clone())),
            ("lore_ref", Value::String(self.lore_ref.clone())),
            ("total_xp", int(self.total_xp)),
            ("spend", Value::Object(clean)),
            ("learned_skills", strs(&self.learned_skills)),
            ("passives", strs(&self.passives)),
            ("loadout", Value::Object(loadout)),
            ("equipment", Value::Object(equipment)),
            ("carried", strs(&self.carried)),
            ("pouch", strs(&self.pouch)),
            ("hp_lost", int(self.hp_lost)),
        ]))
    }

    fn hp_lost_of(raw: Option<&Value>) -> i64 {
        match raw {
            None | Some(Value::Null) => 0,
            Some(v) if json::is_int(v) => json::to_int(v),
            Some(Value::Number(n)) => {
                let x = n.as_f64().unwrap_or(0.5);
                if x.is_finite() && x == x.floor() {
                    x as i64
                } else {
                    -1
                }
            }
            _ => -1,
        }
    }

    pub fn from_dict(d: &Dict) -> Self {
        let lo = json::dget(d, "loadout");
        let mut equipment = IndexMap::new();
        for (k, v) in json::dget(d, "equipment") {
            equipment.insert(k.clone(), json::gstr(v));
        }
        Self {
            id: json::s(d, "id"),
            base_unit: json::s(d, "base_unit"),
            lore_ref: json::s(d, "lore_ref"),
            total_xp: json::i(d, "total_xp", 0),
            spend: d.get("spend").map(json::ints).unwrap_or_default(),
            learned_skills: json::sget(d, "learned_skills"),
            passives: json::sget(d, "passives"),
            loadout_class: json::sget(lo, "class"),
            loadout_extra: json::sget(lo, "extra"),
            loadout_ranks: lo.get("ranks").filter(|v| v.is_object()).map(json::ints).unwrap_or_default(),
            equipment,
            carried: json::sget(d, "carried"),
            pouch: json::sget(d, "pouch"),
            hp_lost: Self::hp_lost_of(d.get("hp_lost")),
        }
    }

    pub fn validate(&self, d: &Data) -> Vec<String> {
        let mut errors = Vec::new();
        let at = format!("Unit '{}'", self.id);
        if !json::id_ok(&self.id) {
            errors.push(format!("Unit id '{}' must be non-empty [a-z0-9_].", self.id));
        }
        let entry = d.entry("units", &self.base_unit);
        if entry.is_empty() {
            errors.push(format!("{at}: unknown base unit '{}'.", self.base_unit));
            return errors;
        }
        if json::s(entry, "side") != "party" {
            errors.push(format!("{at}: base unit '{}' is not a party unit.", self.base_unit));
        }
        if !d.cat.classes.contains_key(&self.class_id(d)) || json::s_or(entry, "stat_mode", "derived") != "derived" {
            errors.push(format!("{at}: base unit '{}' must be a class-derived unit.", self.base_unit));
            return errors;
        }
        if self.total_xp < 0 {
            errors.push(format!("{at}: total_xp can't be negative."));
        }
        if self.hp_lost < 0 {
            errors.push(format!("{at}: hp_lost must be 0 or more."));
        }
        for (k, v) in &self.spend {
            if !STATS.contains(&k.as_str()) {
                errors.push(format!("{at}: spend has unknown stat '{k}'."));
            } else if *v < 0 {
                errors.push(format!("{at}: spend {k} can't be negative."));
            }
        }
        let pool = rpgkit_rules::stats::spend_pool(self.level(d) as i32) as i64;
        if self.spent() > pool {
            errors.push(format!("{at}: spend totals {}, above the level {} pool of {pool}.", self.spent(), self.level(d)));
        }
        for sk in &self.learned_skills {
            if !d.has("skills", sk) {
                errors.push(format!("{at}: unknown learned skill '{sk}'."));
            }
        }
        for p in &self.passives {
            if !d.has("passives", p) {
                errors.push(format!("{at}: unknown passive '{p}'."));
            }
        }
        errors.extend(self.loadout_errors(d, &at));
        for (slot, item) in &self.equipment {
            if !LOADOUT_SLOTS.contains(&slot.as_str()) {
                errors.push(format!("{at}: unknown equipment slot '{slot}'."));
            } else if !d.has("items", item) {
                errors.push(format!("{at}: unknown item '{item}' in {slot}."));
            } else if let Err(e) = self.can_wear(d, item, slot) {
                errors.push(format!("{at}: {e}"));
            }
        }
        let worn = |slot: &str| self.equipment.get(slot).and_then(|i| d.cat.items.get(i));
        if let (Some(main), Some(off)) = (worn("main_hand"), worn("off_hand")) {
            if !off_hand_ok(Some(main), off) {
                errors.push(format!("{at}: {} can't be held with the two-handed {}.", off.name, main.name));
            }
        }
        for item in &self.carried {
            if !d.has("items", item) {
                errors.push(format!("{at}: unknown carried item '{item}'."));
            }
        }
        if self.pouch.len() > POUCH_MAX {
            errors.push(format!("{at}: Pouch holds at most {POUCH_MAX} items."));
        }
        for item in &self.pouch {
            if json::s(d.entry("items", item), "item_type") != "CONSUMABLE" {
                errors.push(format!("{at}: Pouch: '{item}' is not a consumable."));
            }
        }
        errors
    }

    pub fn loadout_errors(&self, d: &Data, at: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if self.loadout_class.len() > rpgkit_rules::stats::MAX_CLASS_SLOTS {
            errors.push(format!("{at}: at most {} class families in the loadout.", rpgkit_rules::stats::MAX_CLASS_SLOTS));
        }
        if self.loadout_extra.len() > rpgkit_rules::stats::MAX_EXTRA_SLOTS {
            errors.push(format!("{at}: at most {} extra families in the loadout.", rpgkit_rules::stats::MAX_EXTRA_SLOTS));
        }
        let owned = self.owned_families(d);
        let mut seen: Vec<&String> = Vec::new();
        for fam in self.loadout_class.iter().chain(&self.loadout_extra) {
            if seen.contains(&fam) {
                errors.push(format!("{at}: loadout lists family '{fam}' twice."));
            }
            seen.push(fam);
            if !owned.contains(fam) {
                errors.push(format!("{at}: loadout family '{fam}' is not owned."));
            }
        }
        // SKILLUI1 V1 and V2: a rank entry names an owned, non-basic family and one of its tiers.
        for (fam, rank) in &self.loadout_ranks {
            if d.cat.is_basic_family(fam) {
                errors.push(format!("{at}: rank set for family '{fam}', which is a basic family (basics have one rank)."));
            } else if !owned.contains(fam) {
                errors.push(format!("{at}: rank set for family '{fam}' that is not owned."));
            } else if !d.cat.family_rows(fam).iter().any(|s| s.tier as i64 == *rank) {
                errors.push(format!("{at}: family '{fam}' has no rank {rank}."));
            }
        }
        errors
    }
}

// ---- PartyState ----------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct PartyState {
    pub members: Vec<UnitState>,
    pub active_ids: Vec<String>,
    /// The world's cap on active members (never saved).
    pub max_active: i64,
    pub stash: IndexMap<String, i64>,
    pub gold: i64,
    pub supplies: i64,
}

impl Default for PartyState {
    fn default() -> Self {
        Self { members: Vec::new(), active_ids: Vec::new(), max_active: PARTY_MAX, stash: IndexMap::new(), gold: 0, supplies: 0 }
    }
}

impl PartyState {
    pub fn member(&self, id: &str) -> Option<&UnitState> {
        self.members.iter().find(|m| m.id == id)
    }

    pub fn member_mut(&mut self, id: &str) -> Option<&mut UnitState> {
        self.members.iter_mut().find(|m| m.id == id)
    }

    /// Indexes of the members that take the field, roster order.
    pub fn active_indexes(&self) -> Vec<usize> {
        let cap = self.max_active.max(0) as usize;
        let mut out = Vec::new();
        for (i, m) in self.members.iter().enumerate() {
            if out.len() < cap && (self.active_ids.is_empty() || self.active_ids.contains(&m.id)) {
                out.push(i);
            }
        }
        out
    }

    pub fn active_members(&self) -> Vec<&UnitState> {
        self.active_indexes().into_iter().map(|i| &self.members[i]).collect()
    }

    pub fn add_to_stash(&mut self, item: &str, count: i64) {
        *self.stash.entry(item.to_string()).or_insert(0) += count;
    }

    pub fn remove_from_stash(&mut self, item: &str, count: i64) -> bool {
        let held = self.stash.get(item).copied().unwrap_or(0);
        if count < 1 || held < count {
            return false;
        }
        if held == count {
            self.stash.shift_remove(item);
        } else {
            self.stash.insert(item.to_string(), held - count);
        }
        true
    }

    pub fn add_gold(&mut self, amount: i64) -> i64 {
        let before = self.gold;
        self.gold = (self.gold + amount.max(0)).clamp(0, GOLD_MAX);
        self.gold - before
    }

    pub fn add_supplies(&mut self, n: i64) -> i64 {
        let before = self.supplies;
        self.supplies = (self.supplies + n.max(0)).clamp(0, SUPPLIES_MAX);
        (self.supplies - before).max(0)
    }

    pub fn supplies_cost(have: i64, n: i64) -> i64 {
        n.max(0).min(have.max(0))
    }

    pub fn spend_supplies(&mut self, n: i64) -> i64 {
        let spent = Self::supplies_cost(self.supplies, n);
        self.supplies -= spent;
        spent
    }

    pub fn apply_cap(&mut self, cap: i64) {
        self.max_active = cap.clamp(1, PARTY_MAX);
        self.active_ids.truncate(self.max_active as usize);
    }

    pub fn to_value(&self) -> Value {
        let mut stash = Dict::new();
        for (k, v) in &self.stash {
            stash.insert(k.clone(), int(*v));
        }
        Value::Object(json::obj([
            ("members", Value::Array(self.members.iter().map(UnitState::to_value).collect())),
            ("active_ids", strs(&self.active_ids)),
            ("stash", Value::Object(stash)),
            ("gold", int(self.gold)),
            ("supplies", int(self.supplies)),
        ]))
    }

    pub fn from_dict(d: &Dict) -> Self {
        let whole = |key: &str| {
            let raw = json::normalize(d.get(key).unwrap_or(&int(0)));
            if json::is_int(&raw) {
                json::to_int(&raw)
            } else {
                -1
            }
        };
        Self {
            members: json::aget(d, "members").iter().map(|m| UnitState::from_dict(json::dict(m))).collect(),
            active_ids: json::sget(d, "active_ids"),
            max_active: PARTY_MAX,
            stash: d.get("stash").map(json::ints).unwrap_or_default(),
            gold: whole("gold"),
            supplies: whole("supplies"),
        }
    }

    pub fn validate(&self, d: &Data) -> Vec<String> {
        let mut errors = Vec::new();
        if self.members.is_empty() {
            errors.push("A party needs at least one member.".into());
        }
        let mut seen: Vec<&String> = Vec::new();
        for m in &self.members {
            if seen.contains(&&m.id) {
                errors.push(format!("Duplicate member id '{}'.", m.id));
            }
            seen.push(&m.id);
            errors.extend(m.validate(d));
        }
        if self.active_ids.len() as i64 > self.max_active {
            errors.push(format!("At most {} active members (got {}).", self.max_active, self.active_ids.len()));
        }
        let mut listed: Vec<&String> = Vec::new();
        for a in &self.active_ids {
            if !seen.contains(&a) {
                errors.push(format!("Active id '{a}' is not a member."));
            }
            if listed.contains(&a) {
                errors.push(format!("Active id '{a}' is listed twice."));
            }
            listed.push(a);
        }
        for (k, v) in &self.stash {
            if !d.has("items", k) {
                errors.push(format!("Stash item '{k}' does not exist."));
            } else if *v < 1 {
                errors.push(format!("Stash count of '{k}' must be >= 1."));
            }
        }
        if self.gold < 0 || self.gold > GOLD_MAX {
            errors.push(format!("gold must be 0..{GOLD_MAX}."));
        }
        if self.supplies < 0 || self.supplies > SUPPLIES_MAX {
            errors.push(format!("supplies must be 0..{SUPPLIES_MAX}."));
        }
        errors
    }
}

// ---- GameState -----------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    pub world_id: String,
    pub location_id: String,
    pub party: PartyState,
    pub flags: Dict,
    pub quests: Dict,
    pub loot_claimed: Vec<String>,
    pub position: (i64, i64),
    pub explore: Dict,
    pub rng_seed: i64,
    pub rolls: Dict,
    pub shops: Dict,
    pub time: i64,
    pub autoplay: Dict,
    pub purses: Dict,
    /// REWARD1: {encounter qid: [roster key `ref#n`, or "objective"]} whose `once` purse share or objective reward
    /// was paid.
    pub reward_paid: Dict,
    /// CAMPAIGN1 11.5: the campaign state, Some only in a world whose lineage sets `campaign` (X1); saved as the key
    /// `campaign` only then (X3).
    pub campaign: Option<crate::campaign::Campaign>,
    /// Raw type problems from from_dict (not saved; validate reports them).
    pub load_problems: Vec<String>,
}

pub fn default_explore() -> Dict {
    json::obj([
        ("steps", int(0)),
        ("next_at", int(16)),
        ("loc_steps", int(0)),
        ("last_safe", Value::String(String::new())),
        ("last_rest", Value::String(String::new())),
    ])
}

pub fn default_autoplay() -> Dict {
    json::obj([("on", Value::Bool(false)), ("party", Value::String(DEFAULT_TACTIC.into())), ("heroes", Value::Object(Dict::new()))])
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            world_id: String::new(),
            location_id: String::new(),
            party: PartyState::default(),
            flags: Dict::new(),
            quests: Dict::new(),
            loot_claimed: Vec::new(),
            position: NO_POSITION,
            explore: default_explore(),
            rng_seed: 0,
            rolls: Dict::new(),
            shops: Dict::new(),
            time: 0,
            autoplay: default_autoplay(),
            purses: Dict::new(),
            reward_paid: Dict::new(),
            campaign: None,
            load_problems: Vec::new(),
        }
    }
}

pub fn flag_key_ok(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

pub fn flag_value_ok(v: &Value) -> bool {
    matches!(v, Value::Bool(_) | Value::String(_)) || json::is_int(v)
}

/// QuestRules.blob_key_ok: exactly one ':' between two ids.
pub fn qid_ok(key: &str) -> bool {
    let parts: Vec<&str> = key.split(':').collect();
    parts.len() == 2 && json::id_ok(parts[0]) && json::id_ok(parts[1])
}

impl GameState {
    pub fn create(d: &Data, world_id: &str, party: PartyState) -> Self {
        let mut g = Self { world_id: world_id.into(), party, ..Default::default() };
        g.party.apply_cap(d.party_max(world_id));
        g.campaign = crate::campaign::Campaign::fresh(d, world_id, g.rng_seed);
        g
    }

    pub fn set_flag(&mut self, key: &str, value: Value) -> String {
        if !flag_key_ok(key) {
            return format!("Flag key '{key}' must be non-empty [a-z0-9_.].");
        }
        if rpgkit_data::reserved_key(key) {
            return crate::campaign::reserved_text(key);
        }
        if !flag_value_ok(&value) {
            return format!("Flag '{key}' must be a bool, int or String.");
        }
        self.flags.insert(key.into(), value);
        String::new()
    }

    /// `gs.get_flag(key, fallback)`.
    pub fn flag(&self, key: &str) -> Option<&Value> {
        self.flags.get(key)
    }

    /// A flag condition's value: a read key (`meter.`, `campaign.`, `roster.`; CAMPAIGN1 11.4) from the campaign,
    /// any other key from the flags.
    pub fn read_flag(&self, key: &str) -> Option<Value> {
        if rpgkit_data::reserved_key(key) {
            crate::campaign::read_key(self, key)
        } else {
            self.flags.get(key).cloned()
        }
    }

    pub fn flag_true(&self, key: &str) -> bool {
        self.flags.get(key) == Some(&Value::Bool(true))
    }

    pub fn clear_flag(&mut self, key: &str) {
        self.flags.shift_remove(key);
    }

    pub fn explore_int(&self, key: &str) -> i64 {
        json::i(&self.explore, key, 0)
    }

    pub fn to_value(&self) -> Value {
        let mut loot = self.loot_claimed.clone();
        loot.sort();
        let mut out = json::obj([
            ("world_id", Value::String(self.world_id.clone())),
            ("location_id", Value::String(self.location_id.clone())),
            ("party", self.party.to_value()),
            ("flags", Value::Object(self.flags.clone())),
            ("quests", Value::Object(self.quests.clone())),
            ("loot_claimed", strs(&loot)),
            ("position", Value::Array(vec![int(self.position.0), int(self.position.1)])),
            ("explore", Value::Object(self.explore.clone())),
            ("rng_seed", int(self.rng_seed)),
            ("rolls", Value::Object(self.rolls.clone())),
            ("shops", Value::Object(self.shops.clone())),
            ("time", int(self.time)),
            ("autoplay", Value::Object(self.autoplay.clone())),
            ("purses", Value::Object(self.purses.clone())),
            ("reward_paid", Value::Object(self.reward_paid.clone())),
        ]);
        if let Some(c) = &self.campaign {
            out.insert("campaign".into(), c.to_value());
        }
        Value::Object(out)
    }

    pub fn from_dict(data: &Data, d: &Dict) -> Self {
        let mut g = Self { world_id: json::s(d, "world_id"), location_id: json::s(d, "location_id"), ..Default::default() };
        g.party = PartyState::from_dict(json::dget(d, "party"));
        g.party.apply_cap(data.party_max(&g.world_id));
        for (k, v) in json::dict(&json::normalize(d.get("flags").unwrap_or(&Value::Null))) {
            g.flags.insert(k.clone(), v.clone());
        }
        let raw_quests = d.get("quests").cloned().unwrap_or(Value::Object(Dict::new()));
        if !raw_quests.is_object() {
            g.load_problems.push(format!("quests must be an object (the save has {}).", type_name(&raw_quests)));
        }
        g.quests = json::dict(&json::normalize(&raw_quests)).clone();
        let raw_loot = d.get("loot_claimed").cloned().unwrap_or(Value::Array(Vec::new()));
        if !raw_loot.is_array() {
            g.load_problems.push(format!("loot_claimed must be a list (the save has {}).", type_name(&raw_loot)));
        } else {
            for x in json::arr(&raw_loot) {
                if !x.is_string() {
                    g.load_problems.push(format!("loot_claimed: '{}' is not a String.", json::gstr(x)));
                }
            }
        }
        g.loot_claimed = json::strings(&raw_loot);
        let pos = json::normalize(d.get("position").unwrap_or(&Value::Array(vec![int(-1), int(-1)])));
        let p = json::arr(&pos);
        if p.len() != 2 || !json::is_int(&p[0]) || !json::is_int(&p[1]) {
            g.load_problems.push("position must be [x, y].".into());
        } else {
            g.position = (json::to_int(&p[0]), json::to_int(&p[1]));
        }
        let mut take_dict = |key: &str, problem: &str, slot: &mut Dict| match d.get(key).map(json::normalize) {
            None => {}
            Some(Value::Object(o)) => *slot = o,
            Some(_) => g.load_problems.push(problem.into()),
        };
        let mut explore = default_explore();
        let mut rolls = Dict::new();
        let mut shops = Dict::new();
        let mut autoplay = default_autoplay();
        let mut purses = Dict::new();
        let mut reward_paid = Dict::new();
        take_dict("explore", "explore must be {steps, next_at, loc_steps, last_safe, last_rest}.", &mut explore);
        take_dict("rolls", "rolls must be an object {stream: count}.", &mut rolls);
        take_dict("shops", "shops must be an object {shop qid: {item id: sold count}}.", &mut shops);
        take_dict("autoplay", "autoplay must be {on, party, heroes}.", &mut autoplay);
        take_dict("purses", "purses must be an object {encounter qid: wins}.", &mut purses);
        take_dict("reward_paid", "reward_paid must map encounter ids to lists of keys.", &mut reward_paid);
        g.explore = explore;
        g.rolls = rolls;
        g.shops = shops;
        g.autoplay = autoplay;
        g.purses = purses;
        g.reward_paid = reward_paid;
        let whole = |key: &str, problem: &str, g: &mut GameState| -> i64 {
            let raw = json::normalize(d.get(key).unwrap_or(&int(0)));
            if json::is_int(&raw) {
                json::to_int(&raw)
            } else {
                g.load_problems.push(problem.into());
                0
            }
        };
        g.rng_seed = whole("rng_seed", "rng_seed must be a whole number.", &mut g);
        g.time = whole("time", "time must be a whole number >= 0.", &mut g);
        // CAMPAIGN1 11.5: an absent block reads as a fresh one in a campaign world (older saves), none elsewhere
        g.campaign = match d.get("campaign") {
            Some(raw) => crate::campaign::Campaign::from_value(data, &g.world_id, g.rng_seed, g.time, raw, &mut g.load_problems),
            None => crate::campaign::Campaign::fresh(data, &g.world_id, g.rng_seed).map(|mut c| {
                c.sync(g.time);
                c
            }),
        };
        g
    }

    pub fn validate(&self, d: &Data) -> Vec<String> {
        let mut errors = self.load_problems.clone();
        if self.world_id.is_empty() {
            errors.push("world_id can't be empty.".into());
        }
        errors.extend(self.party.validate(d));
        for (k, v) in &self.flags {
            if !flag_key_ok(k) {
                errors.push(format!("Bad flag key '{k}'."));
            } else if !flag_value_ok(v) {
                errors.push(format!("Flag '{k}' must be a bool, int or String."));
            }
        }
        errors.extend(quest_blob_errors(&self.quests));
        let mut seen: Vec<&String> = Vec::new();
        for id in &self.loot_claimed {
            if !qid_ok(id) {
                errors.push(format!("loot_claimed: '{id}' must be a qualified id world:id."));
            } else if seen.contains(&id) {
                errors.push(format!("loot_claimed lists '{id}' twice."));
            }
            seen.push(id);
        }
        if !self.location_id.is_empty() && !qid_ok(&self.location_id) {
            errors.push(format!("location_id '{}' must be a qualified id world:id (or empty).", self.location_id));
        }
        if self.position.0 < -1 || self.position.1 < -1 {
            errors.push("position must be [x, y].".into());
        }
        errors.extend(explore_errors(&self.explore));
        if self.rng_seed < 0 || self.rng_seed > SEED_MAX {
            errors.push(format!("rng_seed must be 0..{SEED_MAX}."));
        }
        for (k, v) in &self.rolls {
            if !json::id_ok(k) || !json::is_int(v) || json::to_int(v) < 0 {
                errors.push(format!("rolls: '{k}' must be a stream id with a count >= 0."));
            }
        }
        for (k, v) in &self.shops {
            if !qid_ok(k) {
                errors.push(format!("shops: '{k}' must be a qualified shop id world:id."));
            }
            let Some(sold) = v.as_object() else {
                errors.push(format!("shops.{k} must be an object {{item id: sold count}}."));
                continue;
            };
            for (item, n) in sold {
                if !json::id_ok(item) || !json::is_int(n) || json::to_int(n) < 1 {
                    errors.push(format!("shops.{k}: '{item}' must be an item id with a sold count >= 1."));
                }
            }
        }
        if self.time < 0 {
            errors.push("time must be a whole number >= 0.".into());
        }
        errors.extend(autoplay_errors(&self.autoplay));
        for (k, v) in &self.purses {
            if !qid_ok(k) || !json::is_int(v) || json::to_int(v) < 0 {
                errors.push(format!("purses: '{k}' must be a qualified encounter id world:id with a win count >= 0."));
            }
        }
        if self.reward_paid.values().any(|v| !v.as_array().is_some_and(|a| a.iter().all(Value::is_string))) {
            errors.push("reward_paid must map encounter ids to lists of keys.".into());
        }
        if let Some(c) = &self.campaign {
            errors.extend(c.validate());
        }
        errors
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "Nil",
        Value::Bool(_) => "bool",
        Value::Number(n) if n.is_f64() => "float",
        Value::Number(_) => "int",
        Value::String(_) => "String",
        Value::Array(_) => "Array",
        Value::Object(_) => "Dictionary",
    }
}

pub fn quest_blob_errors(blob: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    for (key, value) in blob {
        if !qid_ok(key) {
            errors.push(format!("quests: key '{key}' must be a qualified id world:id."));
        }
        let Some(d) = value.as_object() else {
            errors.push(format!("quests.{key} must be an object {{state, objective}}."));
            continue;
        };
        for k in d.keys() {
            if k != "state" && k != "objective" {
                errors.push(format!("quests.{key}: unknown key '{k}'."));
            }
        }
        for k in ["state", "objective"] {
            if !d.contains_key(k) {
                errors.push(format!("quests.{key}: missing '{k}'."));
            }
        }
        if let Some(st) = d.get("state") {
            if !matches!(st.as_str(), Some("active" | "ready" | "completed")) {
                errors.push(format!("quests.{key}: state '{}' is not one of active, ready, completed.", json::gstr(st)));
            }
        }
        if let Some(o) = d.get("objective") {
            if !json::is_int(o) || json::to_int(o) < 0 || json::to_int(o) > crate::quest::MAX_OBJECTIVES {
                errors.push(format!("quests.{key}: objective must be a whole number 0..{}.", crate::quest::MAX_OBJECTIVES));
            }
        }
    }
    errors
}

pub fn explore_errors(e: &Dict) -> Vec<String> {
    let has_home = e.contains_key(HOME_WORLD_KEY);
    let mut ok = e.len() == EXPLORE_KEYS.len() + has_home as usize;
    ok = ok && (!has_home || e.get(HOME_WORLD_KEY).and_then(Value::as_str).is_some_and(json::id_ok));
    ok = ok && EXPLORE_KEYS.iter().all(|k| e.contains_key(*k));
    if ok {
        let int_ge = |k: &str, lo: i64| json::is_int(&e[k]) && json::to_int(&e[k]) >= lo;
        ok = int_ge("steps", 0) && int_ge("loc_steps", 0) && int_ge("next_at", 1) && json::to_int(&e["next_at"]) <= 99;
        let place = |k: &str| e[k].as_str().is_some_and(|s| s.is_empty() || qid_ok(s));
        ok = ok && place("last_safe") && place("last_rest");
    }
    if ok {
        Vec::new()
    } else {
        vec!["explore must be {steps, next_at, loc_steps, last_safe, last_rest}.".into()]
    }
}

pub fn autoplay_errors(a: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if a.len() != 3 || !["on", "party", "heroes"].iter().all(|k| a.contains_key(*k)) {
        errors.push("autoplay must be {on, party, heroes}.".into());
        return errors;
    }
    if !a["on"].is_boolean() {
        errors.push("autoplay.on must be true or false.".into());
    }
    if !a["party"].as_str().is_some_and(|p| AUTO_IDS.contains(&p)) {
        errors.push(format!("autoplay.party '{}' must be one of {}.", json::gstr(&a["party"]), AUTO_IDS.join(", ")));
    }
    let Some(heroes) = a["heroes"].as_object() else {
        errors.push("autoplay.heroes must be an object {hero id: tactic}.".into());
        return errors;
    };
    for (k, v) in heroes {
        if k.is_empty() || !v.as_str().is_some_and(is_tactic) {
            errors.push(format!("autoplay.heroes: '{k}' must map a hero id to a tactic ({}, custom).", AUTO_IDS.join(", ")));
        }
    }
    errors
}
