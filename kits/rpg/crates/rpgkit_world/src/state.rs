//! GameState, PartyState and UnitState (scripts/game_state/game_state.gd, scripts/progression/party_state.gd,
//! unit_state.gd): the saved snapshot. `to_value` writes Godot's to_dict key order; `from_value` reads like
//! from_dict (damaged blobs are kept as load problems that `validate` reports, never coerced silently).

use crate::data::{Data, PARTY_MAX};
use crate::progression;
use aegis_core::json::{self, int, strs, Dict};
use rpgkit_data::{Deck, RosterManifest, DEPART_LIVES, MEMBER_CONDITIONS, NO_ITEM, STATS};
use indexmap::IndexMap;

// The gear rules live in the rules crate (the battle crate's gear dojo uses them too).
pub use rpgkit_rules::gear::{item_fits_slot, off_hand_ok, trained_for};
use serde_json::Value;

pub use rpgkit_content::source::PurseCaps;
/// The default purse gold cap (E2.2-S1: a game's `purse_gold_cap` replaces it; read `PartyState::purse`).
pub const GOLD_MAX: i64 = PurseCaps::PURSE_GOLD_CAP;
/// The default supply cap (E2.2-S1: a game's `supplies_max` replaces it; read `PartyState::purse`).
pub const SUPPLIES_MAX: i64 = PurseCaps::SUPPLIES_MAX;
pub const SUPPLIES_START: i64 = rpgkit_content::source::SUPPLIES_START;
pub const SEED_MAX: i64 = 2_147_483_647;
pub const POUCH_MAX: usize = 3;
pub const NO_POSITION: (i64, i64) = (-1, -1);
pub const HOME_WORLD_KEY: &str = "home_world";
pub const EXPLORE_KEYS: [&str; 5] = ["steps", "next_at", "loc_steps", "last_safe", "last_rest"];
/// MAPNAV1-U: the optional `explore` key naming the member who leads on the maps (`GameState::explore_leader`); absent
/// = the first standing active member (a save without it loads as before).
pub const LEADER_KEY: &str = "leader";
pub const AUTO_IDS: [&str; 6] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy"];
pub const DEFAULT_TACTIC: &str = "balanced";
/// A1.2-K3 (INTENT2-K3): the copies one pouch stack of catalog item `item` holds: `min(5, max_stack)`, at least 1
/// (`ItemDef::stack_limit`; 1 for an unknown item).
pub fn stack_limit(d: &Data, item: &str) -> i64 {
    d.cat.items.get(item).map_or(1, |i| i.stack_limit() as i64)
}

/// A1.2-K3 (INTENT2-K3): the saved member key of the pouch's packed targets (`UnitState::pouch_packed`).
pub const POUCH_PACKED_KEY: &str = "pouch_packed";
/// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): the saved member key of its unit track stages (`UnitState::tracks`) and of
/// its permanent max HP offset (`UnitState::max_hp_offset`); each written only when set (no save VERSION bump).
pub const TRACKS_KEY: &str = "tracks";
pub const MAX_HP_OFFSET_KEY: &str = "max_hp_offset";
/// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3): the saved member keys of its body slots (`UnitState::body`: slot -> graft
/// item id), its class rite (`UnitState::class_rite`) and its floor offset for the next battle
/// (`UnitState::floor_offset`); each written only when set (no save VERSION bump).
/// `body` is the graft body slots, not `equipment.body` (the armour slot).
pub const BODY_KEY: &str = "body";
pub const CLASS_RITE_KEY: &str = "class_rite";
pub const FLOOR_OFFSET_KEY: &str = "floor_offset";
/// PK2-S3: the member key of the trained proficiency levels (`UnitState::trained`).
pub const TRAINED_KEY: &str = "trained";
/// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): the member keys of its weapon mastery XP (`UnitState::mastery_xp`) and its
/// forge rite (`UnitState::forge_rite`); each written only when set (no save VERSION bump).
pub const MASTERY_XP_KEY: &str = "mastery_xp";
pub const FORGE_RITE_KEY: &str = "forge_rite";
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
    /// RULESET1 9 D5 (read only under the rule `deck`): the member's actives, passives, keystones and per-intent order;
    /// empty = auto (the base unit's deck). Saved as `deck` only when not empty (the unreleased save v13 carries it:
    /// a save without it reads the empty deck).
    pub deck: Deck,
    /// The saved `deck`'s type problems (not saved; `validate` reports them, so a damaged deck is refused, never
    /// coerced).
    pub deck_problems: Vec<String>,
    /// ROSTER1 W1, 11.4: the member's conditions (`rpgkit_data::MEMBER_CONDITIONS`: `wounded`), each held once. Saved
    /// as `conditions` only when not empty (the unreleased save v13 carries it: a save without it reads none).
    pub conditions: Vec<String>,
    /// A1.2-K3 (INTENT2-K3, INTENT2-U): item id -> how many copies auto-fill tops that pouch slot up to (`1..min(5,
    /// max_stack)`; a pouch id without an entry reads its current count). A key with no copy left keeps its slot. Saved
    /// as `pouch_packed` only when not empty (no save bump: a save without it reads the current pouch).
    pub pouch_packed: IndexMap<String, i64>,
    /// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): unit track id -> the member's stage on it (1..the row's stages; a track
    /// without an entry reads 0). Saved as `tracks` only when a stage is above 0 (no save bump: a save without it reads
    /// every track at 0).
    pub tracks: IndexMap<String, i64>,
    /// PK2-S1: the permanent max HP offset (bone-scraping, excision; negative lowers it), added to the built max HP
    /// (at least 1). Saved as `max_hp_offset` only when not 0.
    pub max_hp_offset: i64,
    /// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3): body slot id -> the `GRAFT` item installed there (a second slot set,
    /// separate from `equipment`; the graft count is its length). Saved as `body` only when not empty.
    pub body: IndexMap<String, String>,
    /// PK2-S2: the member did its class rite, which opens the class body slots (class pieces only). Saved as
    /// `class_rite` only when true (`set_class_rite`).
    pub class_rite: bool,
    /// PK2-S2 (FB-167 Q6): added to the floors a body meter's `floors` table gives in the member's next battle; the
    /// settle clears it. Saved as `floor_offset` only when not 0.
    pub floor_offset: i64,
    /// PK2-S3 (`docs/sessions/PK2_GRAFTS.md` 5): gear type -> the proficiency levels the member trained (a recipe's
    /// `train_proficiency`), on top of its class's and its drills'. Saved as `trained` only when not empty.
    pub trained: IndexMap<String, i64>,
    /// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): gear type -> the member's weapon mastery XP in that family (a bar toward
    /// levels 1-3, `rpgkit_data::mastery::level_of`). Saved as `mastery_xp` only when not empty.
    pub mastery_xp: IndexMap<String, i64>,
    /// PK2-S4: the member's forge rite is done (with mastery 3 it opens the wielded family's keystone). Saved as
    /// `forge_rite` only when true (`set_forge_rite`).
    pub forge_rite: bool,
}

/// RULESET1 9 D5: a saved `deck` value: (the deck, its type problems). Absent = the empty deck; anything but an object
/// of string lists (and an `order` object of string lists) is a problem.
fn deck_from(raw: Option<&Value>) -> (Deck, Vec<String>) {
    let mut problems = Vec::new();
    let Some(raw) = raw else { return (Deck::default(), problems) };
    let Some(o) = raw.as_object() else {
        problems.push(format!("deck must be an object {{{}}}.", Deck::KEYS.join(", ")));
        return (Deck::default(), problems);
    };
    let texts = |v: &Value| v.as_array().filter(|a| a.iter().all(Value::is_string)).map(|a| a.iter().map(json::gstr).collect::<Vec<String>>());
    let mut deck = Deck::default();
    for (k, v) in o {
        match k.as_str() {
            // PK1-S2a: `channeled` (the actives marked channeled) is a list of text too (a save without it reads none)
            "active" | "passive" | "keystone" | "channeled" => {
                let slot = match k.as_str() {
                    "active" => &mut deck.active,
                    "passive" => &mut deck.passive,
                    "channeled" => &mut deck.channeled,
                    _ => &mut deck.keystone,
                };
                match texts(v) {
                    Some(list) => *slot = list,
                    None => problems.push(format!("deck.{k} must be a list of text.")),
                }
            }
            "order" => match v.as_object() {
                Some(order) => {
                    for (intent, fams) in order {
                        match texts(fams) {
                            // an empty list is the auto order, as an absent one
                            Some(list) if list.is_empty() => {}
                            Some(list) => {
                                deck.order.insert(intent.clone(), list);
                            }
                            None => problems.push(format!("deck.order.{intent} must be a list of text.")),
                        }
                    }
                }
                None => problems.push("deck.order must be an object {intent: [skill families]}.".into()),
            },
            // INPUT1 FL4: the opener's follow-up lists (a save without them reads none; an empty list is no list)
            "follow_ups" => match v.as_object() {
                Some(lists) => {
                    for (opener, fams) in lists {
                        match texts(fams) {
                            Some(list) if list.is_empty() => {}
                            Some(list) => {
                                deck.follow_ups.insert(opener.clone(), list);
                            }
                            None => problems.push(format!("deck.follow_ups.{opener} must be a list of text.")),
                        }
                    }
                }
                None => problems.push("deck.follow_ups must be an object {opener family: [skill families]}.".into()),
            },
            _ => problems.push(format!("deck: unknown key '{k}'.")),
        }
    }
    (deck, problems)
}

/// The saved form of a deck: its non-empty keys in `Deck::KEYS` order (order and follow_ups: their non-empty lists;
/// INPUT1 FL4: a deck without follow-up lists writes as before).
fn deck_value(deck: &Deck) -> Value {
    let mut out = Dict::new();
    for (k, list) in [("active", &deck.active), ("passive", &deck.passive), ("keystone", &deck.keystone)] {
        if !list.is_empty() {
            out.insert(k.into(), strs(list));
        }
    }
    let order: Dict = deck.order.iter().filter(|(_, l)| !l.is_empty()).map(|(k, l)| (k.clone(), strs(l))).collect();
    if !order.is_empty() {
        out.insert("order".into(), Value::Object(order));
    }
    let follow_ups: Dict = deck.follow_ups.iter().filter(|(_, l)| !l.is_empty()).map(|(k, l)| (k.clone(), strs(l))).collect();
    if !follow_ups.is_empty() {
        out.insert("follow_ups".into(), Value::Object(follow_ups));
    }
    // PK1-S2a: a deck without channeled marks writes as before
    if !deck.channeled.is_empty() {
        out.insert("channeled".into(), strs(&deck.channeled));
    }
    Value::Object(out)
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

    /// UnitState._gear_errors for one item: it fits `slot` (ContentFactory.item_fits_slot, worn gear kept). U3.2-S3
    /// (FB-34): training is no longer a gate (any unit may wear any item; an untrained one fights under the wield
    /// penalty, `trains`), so `wears_untrained` is retired.
    pub fn can_wear(&self, d: &Data, item: &str, slot: &str) -> Result<(), String> {
        let def = d.cat.items.get(item).ok_or_else(|| format!("unknown item '{item}'."))?;
        if !item_fits_slot(def, slot) {
            return Err(format!("{} can't be equipped in {}.", def.name, slot.replace('_', " ")));
        }
        // PK2-S1: a member grant `forbid_hands` (a unit track's stage) refuses two-handed items
        if rpgkit_rules::unit::forbids_hands(progression::forbid_hands(d, self), def) {
            return Err(format!("{} is two-handed: {} can't wield two-handed items now.", def.name, progression::display_name(d, self)));
        }
        Ok(())
    }

    /// PR5-26 (audit B2): the item slot `slot` gives the member without its own gear: its base unit's row in world
    /// `world` (the row's item or its class's default, as a build equips it; None: empty).
    pub fn default_item(&self, d: &Data, world: &str, slot: &str) -> Option<String> {
        let cat = d.cat_in(world);
        cat.units.get(&self.base_unit).and_then(|row| rpgkit_rules::unit::row_slot_item(cat, row, slot))
    }

    /// PR5-26 (audit B2): whether the member's grant `forbid_hands` refuses the default item of `slot` (`default_item`),
    /// so a slot left to its default must hold `NO_ITEM` instead.
    pub fn forbids_default(&self, d: &Data, world: &str, slot: &str) -> bool {
        let hands = progression::forbid_hands(d, self);
        hands > 0 && self.default_item(d, world, slot).and_then(|id| d.cat_in(world).items.get(&id)).is_some_and(|i| rpgkit_rules::unit::forbids_hands(hands, i))
    }

    /// U3.2-S3 (FB-34, FB-63 (2)): whether the member wears `item` trained: its class's training or its drill passives'
    /// (UNITDRILL1), the passives re-gathered as a build reads them (`progression::member_passives`), not the saved list.
    /// A class with no proficiency list trains everything (`gear::trained_for`).
    pub fn trains(&self, d: &Data, item: &str) -> bool {
        let Some(def) = d.cat.items.get(item) else { return true };
        let class = d.cat.classes.get(&self.class_id(d));
        class.is_none_or(|c| trained_for(&self.proficiency(d, c), def))
    }

    /// PK2-S3: the member's proficiency out of battle: its class's, its drill passives' (`with_drills`) and its trained
    /// levels (`trained`), as a build reads them.
    pub fn proficiency(&self, d: &Data, class: &rpgkit_data::ClassDef) -> rpgkit_rules::stats::Proficiency {
        let mut p = rpgkit_rules::stats::Proficiency::from_class(class).with_drills(&d.cat, &progression::member_passives(d, self));
        for (gear, levels) in &self.trained {
            p.add_bonus(gear, (*levels).clamp(0, i64::from(rpgkit_rules::stats::PROF_MAX_LEVEL)) as i32);
        }
        p
    }

    pub fn class_id(&self, d: &Data) -> String {
        json::s(d.entry("units", &self.base_unit), "class_id")
    }

    /// Non-basic families owned at its level in world `world`: the class path there (HOOKS1 CN3; decision "HOOKS1-UI
    /// slice 1 answers": the world being played, "" = the base catalog's), the base unit's extras and learned skills.
    pub fn owned_families(&self, d: &Data, world: &str) -> Vec<String> {
        self.owned_in(d, &[world.to_string()])
    }

    /// The families it owns in some world (the base catalog's path or any world's overlay path): what a saved loadout,
    /// deck or rank may name (a save is played in more than one world; a build in a world where a slotted family is not
    /// owned drops it as an unowned loadout entry does).
    pub fn owned_families_anywhere(&self, d: &Data) -> Vec<String> {
        let mut worlds = vec![String::new()];
        worlds.extend(d.world_ids());
        self.owned_in(d, &worlds)
    }

    fn owned_in(&self, d: &Data, worlds: &[String]) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        let lvl = self.level(d) as i32;
        for w in worlds {
            if let Some(arch) = d.cat_in(w).classes.get(&self.class_id(d)) {
                for (_, s) in arch.path_steps().into_iter().filter(|(l, _)| *l <= lvl) {
                    if !ids.contains(&s) {
                        ids.push(s);
                    }
                }
            }
        }
        ids.extend(json::sget(d.entry("units", &self.base_unit), "extra_skills"));
        ids.extend(self.learned_skills.iter().cloned());
        // PK2-S2: a graft's skill is owned while it sits in the body (a deck active may name it)
        ids.extend(self.body_skills(d));
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
        let mut out = json::obj([
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
        ]);
        // RULESET1 9 D5: only a deck in use is saved (a classic member's save is unchanged)
        if !self.deck.is_empty() {
            out.insert("deck".into(), deck_value(&self.deck));
        }
        // ROSTER1 11.4: only a member with a condition saves `conditions`
        if !self.conditions.is_empty() {
            out.insert("conditions".into(), strs(&self.conditions));
        }
        // A1.2-K3: only a member with a packed target saves `pouch_packed`
        if !self.pouch_packed.is_empty() {
            out.insert(POUCH_PACKED_KEY.into(), Value::Object(self.pouch_packed.iter().map(|(k, v)| (k.clone(), int(*v))).collect()));
        }
        // PK2-S1: only a member on a track saves `tracks`, only an offset saves `max_hp_offset`
        let tracks: Dict = self.tracks.iter().filter(|(_, v)| **v != 0).map(|(k, v)| (k.clone(), int(*v))).collect();
        if !tracks.is_empty() {
            out.insert(TRACKS_KEY.into(), Value::Object(tracks));
        }
        if self.max_hp_offset != 0 {
            out.insert(MAX_HP_OFFSET_KEY.into(), int(self.max_hp_offset));
        }
        // PK2-S2: only a grafted member saves `body`, only a rite `class_rite`, only an offset `floor_offset`
        if !self.body.is_empty() {
            out.insert(BODY_KEY.into(), Value::Object(self.body.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect()));
        }
        if self.class_rite {
            out.insert(CLASS_RITE_KEY.into(), Value::Bool(true));
        }
        if self.floor_offset != 0 {
            out.insert(FLOOR_OFFSET_KEY.into(), int(self.floor_offset));
        }
        // PK2-S3: only a trained member saves `trained`
        if !self.trained.is_empty() {
            out.insert(TRAINED_KEY.into(), Value::Object(self.trained.iter().map(|(k, v)| (k.clone(), int(*v))).collect()));
        }
        // PK2-S4: only a member with mastery XP saves `mastery_xp`, only a rite `forge_rite`
        if !self.mastery_xp.is_empty() {
            out.insert(MASTERY_XP_KEY.into(), Value::Object(self.mastery_xp.iter().map(|(k, v)| (k.clone(), int(*v))).collect()));
        }
        if self.forge_rite {
            out.insert(FORGE_RITE_KEY.into(), Value::Bool(true));
        }
        Value::Object(out)
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
        let (deck, deck_problems) = deck_from(d.get("deck"));
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
            deck,
            deck_problems,
            conditions: json::sget(d, "conditions"),
            pouch_packed: d.get(POUCH_PACKED_KEY).filter(|v| v.is_object()).map(json::ints).unwrap_or_default(),
            tracks: d.get(TRACKS_KEY).filter(|v| v.is_object()).map(json::ints).unwrap_or_default(),
            max_hp_offset: json::i(d, MAX_HP_OFFSET_KEY, 0),
            body: json::dget(d, BODY_KEY).iter().map(|(k, v)| (k.clone(), json::gstr(v))).collect(),
            class_rite: json::b(d, CLASS_RITE_KEY, false),
            floor_offset: json::i(d, FLOOR_OFFSET_KEY, 0),
            trained: d.get(TRAINED_KEY).filter(|v| v.is_object()).map(json::ints).unwrap_or_default(),
            mastery_xp: d.get(MASTERY_XP_KEY).filter(|v| v.is_object()).map(json::ints).unwrap_or_default(),
            forge_rite: json::b(d, FORGE_RITE_KEY, false),
        }
    }

    /// PK2-S4: the member's mastery level in gear type `family` (0-3, from its XP).
    pub fn mastery_level(&self, family: &str) -> i32 {
        rpgkit_data::mastery::level_of(self.mastery_xp.get(family).copied().unwrap_or(0))
    }

    /// PK2-S4: sets the forge rite (with mastery 3 it opens the wielded family's keystone); the rite's source (a quest, a
    /// recipe) is the game's.
    pub fn set_forge_rite(&mut self, done: bool) {
        self.forge_rite = done;
    }

    /// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3): the graft count, the body slots in use.
    pub fn graft_count(&self) -> usize {
        self.body.len()
    }

    /// PK2-S2: the installed grafts in the catalog's body slot order (a slot or item the data no longer has is skipped).
    pub fn body_items(&self, d: &Data) -> Vec<rpgkit_data::ItemDef> {
        d.cat.body_slots.keys().filter_map(|slot| self.body.get(slot)).filter_map(|id| d.cat.items.get(id)).filter(|i| i.is_graft()).cloned().collect()
    }

    /// PK2-S2: the skills its grafts teach (their `body_effects`' `skill` grants), each once.
    pub fn body_skills(&self, d: &Data) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for g in self.body_items(d).iter().filter_map(|i| i.graft.as_ref()) {
            for grant in rpgkit_rules::grants::read(&g.effects) {
                if let rpgkit_rules::grants::Grant::GrantSkill(rpgkit_rules::grants::SkillGrant::Learn(skill)) = grant {
                    if !out.contains(&skill) {
                        out.push(skill);
                    }
                }
            }
        }
        out
    }

    /// PK2-S2: sets the class rite (it opens the class body slots); the rite's source (a quest, a recipe) is the game's.
    pub fn set_class_rite(&mut self, done: bool) {
        self.class_rite = done;
    }

    /// PK2-S2: why graft `item` may not sit in body slot `slot` of this member (None: it fits): a known slot, a known
    /// `GRAFT` item, the slot's role (a class slot takes class pieces only, once the class rite is done; a monster slot
    /// monster pieces), a class piece's class.
    pub fn graft_misfit(&self, d: &Data, slot: &str, item: &str) -> Option<String> {
        let Some(row) = d.cat.body_slots.get(slot) else { return Some(format!("unknown body slot '{slot}'.")) };
        let Some(def) = d.cat.items.get(item) else { return Some(format!("unknown item '{item}' in body slot {slot}.")) };
        let Some(g) = &def.graft else { return Some(format!("{} in body slot {slot} is not a graft.", def.name)) };
        if g.role != row.role {
            return Some(format!("body slot {slot} takes {} grafts; {} is a {} graft.", row.role, def.name, g.role));
        }
        if row.is_class() && !self.class_rite {
            return Some(format!("body slot {slot} opens with the member's class rite."));
        }
        if !g.class_id.is_empty() && g.class_id != self.class_id(d) {
            return Some(format!("{} is a piece of class '{}'.", def.name, g.class_id));
        }
        None
    }

    /// PK2-S2: whether the installed graft in `slot` is a keystone graft.
    fn keystone_at(&self, d: &Data, slot: &str) -> bool {
        self.body.get(slot).and_then(|i| d.cat.items.get(i)).and_then(|i| i.graft.as_ref()).is_some_and(|g| g.keystone)
    }

    /// PK2-S2: the body rules of a saved member: every entry fits (`graft_misfit`) and at most one keystone graft.
    pub fn body_errors(&self, d: &Data, at: &str) -> Vec<String> {
        let mut errors: Vec<String> = self.body.iter().filter_map(|(slot, item)| self.graft_misfit(d, slot, item)).map(|e| format!("{at}: {e}")).collect();
        let keystones = self.body.keys().filter(|slot| self.keystone_at(d, slot)).count();
        if keystones > 1 {
            errors.push(format!("{at}: body holds {keystones} keystone grafts (one per member)."));
        }
        errors
    }

    /// PK2-S2: a loaded member's body entries the data no longer allows (`graft_misfit`, a second keystone graft) are
    /// dropped, each with a load warning (the graft is gone; never a refusal). Returns the warnings.
    pub fn sanitize_body(&mut self, d: &Data) -> Vec<String> {
        let mut warnings = Vec::new();
        let mut kept: IndexMap<String, String> = IndexMap::new();
        let mut keystone = false;
        for (slot, item) in &self.body {
            let problem = match self.graft_misfit(d, slot, item) {
                Some(p) => Some(p),
                None if self.keystone_at(d, slot) && keystone => Some(format!("a second keystone graft in body slot {slot}.")),
                None => None,
            };
            match problem {
                Some(p) => warnings.push(format!("Unit '{}': {p} The graft is dropped.", self.id)),
                None => {
                    keystone |= self.keystone_at(d, slot);
                    kept.insert(slot.clone(), item.clone());
                }
            }
        }
        self.body = kept;
        warnings
    }

    /// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3, "The surgery process", "Replacing a graft"): installs graft `item` in
    /// body slot `slot` (no cost here: a recipe pays it): it must fit (`graft_misfit`), a keystone graft is one per
    /// member (one in the same slot it replaces), and a top-tier graft (a keystone graft too, `GraftSpec::top_tier`)
    /// needs `GRAFT_TOP_TIER_NEEDS` grafts first (the slot it overwrites counts). An occupied slot is overwritten: Ok(the
    /// replaced item id, destroyed: no refund); the count and so the floor stay. Err (nothing changes) with the refusal.
    pub fn install_graft(&mut self, d: &Data, slot: &str, item: &str) -> Result<Option<String>, String> {
        if let Some(e) = self.graft_misfit(d, slot, item) {
            return Err(e);
        }
        let g = d.cat.items.get(item).and_then(|i| i.graft.clone()).unwrap_or_default();
        if g.keystone && self.body.keys().any(|s| s != slot && self.keystone_at(d, s)) {
            return Err("the member already holds a keystone graft (one per member).".into());
        }
        let held = self.graft_count();
        if g.top_tier() && held < rpgkit_data::GRAFT_TOP_TIER_NEEDS {
            return Err(format!("a tier {} graft needs {} grafts first (the member holds {held}).", rpgkit_data::GRAFT_TIERS.1, rpgkit_data::GRAFT_TOP_TIER_NEEDS));
        }
        Ok(self.body.insert(slot.to_string(), item.to_string()))
    }

    /// PK2-S2 (excision): empties body slot `slot`; the removed item id (destroyed: no refund), None when it was empty.
    /// The permanent max HP cost is the recipe's (`max_hp_offset`).
    pub fn remove_graft(&mut self, slot: &str) -> Option<String> {
        self.body.shift_remove(slot)
    }

    /// PK2-S1: the member's stage on unit track `track` (0 = not on it).
    pub fn track_stage(&self, track: &str) -> i64 {
        self.tracks.get(track).copied().unwrap_or(0)
    }

    /// PK2-S1: a loaded member's tracks the data no longer allows: a track id with no row is dropped, a stage past its
    /// row's top is held at the top (each with a load warning; never a refusal). Returns the warnings.
    pub fn sanitize_tracks(&mut self, d: &Data) -> Vec<String> {
        let mut warnings = Vec::new();
        let name = self.id.clone();
        self.tracks.retain(|track, stage| match d.cat.unit_tracks.get(track) {
            None => {
                warnings.push(format!("Unit '{name}': unit track '{track}' is gone; its stage {stage} is dropped."));
                false
            }
            Some(row) => {
                let top = i64::from(row.top());
                if *stage > top {
                    warnings.push(format!("Unit '{name}': unit track '{track}' stage {stage} is held at its top stage {top}."));
                    *stage = top;
                }
                true
            }
        });
        warnings
    }

    /// ROSTER1 W1: whether the member carries condition `id`.
    pub fn has_condition(&self, id: &str) -> bool {
        self.conditions.iter().any(|c| c == id)
    }

    /// ROSTER1 W1 (RQ5 a): adds condition `id` once (a second application keeps one). Whether it was added.
    pub fn add_condition(&mut self, id: &str) -> bool {
        if self.has_condition(id) {
            return false;
        }
        self.conditions.push(id.to_string());
        true
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
        // PK2-S1: a stage is never below 0 (an unknown track or a stage past the row's top is dropped or held at load,
        // `sanitize_tracks`)
        for (track, stage) in &self.tracks {
            if *stage < 0 {
                errors.push(format!("{at}: track '{track}' stage must be 0 or more."));
            }
        }
        // PK2-S3: a trained entry is 1..the top proficiency level
        for (gear, levels) in &self.trained {
            if !(1..=i64::from(rpgkit_rules::stats::PROF_MAX_LEVEL)).contains(levels) {
                errors.push(format!("{at}: trained {gear} must be 1-{}.", rpgkit_rules::stats::PROF_MAX_LEVEL));
            }
        }
        // PK2-S4: mastery XP is never below 0
        for (gear, xp) in &self.mastery_xp {
            if *xp < 0 {
                errors.push(format!("{at}: mastery_xp {gear} must be 0 or more."));
            }
        }
        for (i, c) in self.conditions.iter().enumerate() {
            if !MEMBER_CONDITIONS.contains(&c.as_str()) {
                errors.push(format!("{at}: unknown condition '{c}' (known: {}).", MEMBER_CONDITIONS.join(", ")));
            } else if self.conditions[..i].contains(c) {
                errors.push(format!("{at}: condition '{c}' is listed twice."));
            }
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
        // PK2-S2: the body slots (an entry the data no longer allows is dropped at load, `sanitize_body`)
        errors.extend(self.body_errors(d, &at));
        errors.extend(self.deck_problems.iter().map(|p| format!("{at}: {p}")));
        errors.extend(self.deck_errors(d, &at));
        for (slot, item) in &self.equipment {
            if !LOADOUT_SLOTS.contains(&slot.as_str()) {
                errors.push(format!("{at}: unknown equipment slot '{slot}'."));
            } else if item == NO_ITEM {
                // PR5-26 (audit B2): a slot left empty in place of its default (`stash_forbidden`)
                continue;
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
        errors.extend(self.pouch_errors(d, &at));
        errors
    }

    /// The pouch rules of a saved member. A1.2-K3 (INTENT2-K3): a pouch is either the classic form (at most POUCH_MAX
    /// copies) or the stack form (`pouch_stacks`: at most POUCH_MAX distinct items, each at most its stack limit); every
    /// id is a consumable; a packed target is `1..min(5, max_stack)`.
    pub fn pouch_errors(&self, d: &Data, at: &str) -> Vec<String> {
        let mut errors = Vec::new();
        let stacked = self.pouch_slots().len() <= POUCH_MAX && self.pouch_slots().iter().all(|id| self.pouch_count(id) <= stack_limit(d, id));
        if self.pouch.len() > POUCH_MAX && !stacked {
            errors.push(format!("{at}: Pouch holds at most {POUCH_MAX} items (or {POUCH_MAX} stacks, each at most its stack limit)."));
        }
        for item in &self.pouch {
            if json::s(d.entry("items", item), "item_type") != "CONSUMABLE" {
                errors.push(format!("{at}: Pouch: '{item}' is not a consumable."));
            }
        }
        for (item, n) in &self.pouch_packed {
            if json::s(d.entry("items", item), "item_type") != "CONSUMABLE" {
                errors.push(format!("{at}: Pouch packed: '{item}' is not a consumable."));
            } else if *n < 1 || *n > stack_limit(d, item) {
                errors.push(format!("{at}: Pouch packed: '{item}' packs 1-{}, not {n}.", stack_limit(d, item)));
            }
        }
        errors
    }

    /// A1.2-K3: copies of `item` in the saved pouch.
    pub fn pouch_count(&self, item: &str) -> i64 {
        self.pouch.iter().filter(|p| *p == item).count() as i64
    }

    /// A1.2-K3 (INTENT2-K3, INTENT2-U): the pouch slots in order: the slots with a packed target in `pouch_packed`'s key
    /// order (the order the player set: `move_pouch_slot`), then the distinct pouch ids without one, in pouch order.
    pub fn pouch_slots(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for id in self.pouch_packed.keys().chain(self.pouch.iter()) {
            if !out.contains(id) {
                out.push(id.clone());
            }
        }
        out
    }

    /// A1.2-K3 (INTENT2-K3): how many copies auto-fill tops slot `item` up to: its `pouch_packed`, else its current count.
    pub fn packed(&self, item: &str) -> i64 {
        self.pouch_packed.get(item).copied().unwrap_or_else(|| self.pouch_count(item))
    }

    /// A1.2-K3 (INTENT2-K3, INTENT2-U): assigns `item` to a pouch slot: its packed target defaults to `min(5,
    /// max_stack)` even when the stash holds fewer. Refused for a non-consumable, an item already slotted (a second slot
    /// of the same item is not allowed) or a full pouch (POUCH_MAX slots).
    pub fn assign_pouch(&mut self, d: &Data, item: &str) -> Result<(), String> {
        if json::s(d.entry("items", item), "item_type") != "CONSUMABLE" {
            return Err(format!("'{item}' is not a consumable."));
        }
        let slots = self.pouch_slots();
        if slots.iter().any(|s| s == item) {
            return Err(format!("'{item}' already has a pouch slot."));
        }
        if slots.len() >= POUCH_MAX {
            return Err(format!("The pouch holds at most {POUCH_MAX} stacks."));
        }
        self.pouch_packed.insert(item.to_string(), stack_limit(d, item));
        Ok(())
    }

    /// A1.2-K3 (INTENT2-U): sets slot `item`'s packed target to `n` (`1..min(5, max_stack)`; refused outside it or for
    /// an item with no slot).
    pub fn set_packed(&mut self, d: &Data, item: &str, n: i64) -> Result<(), String> {
        if !self.pouch_slots().iter().any(|s| s == item) {
            return Err(format!("'{item}' has no pouch slot."));
        }
        let limit = stack_limit(d, item);
        if n < 1 || n > limit {
            return Err(format!("Pack 1-{limit} of '{item}'."));
        }
        self.pouch_packed.insert(item.to_string(), n);
        Ok(())
    }

    /// INTENT2-U: empties pouch slot `i` (`pouch_slots` order): its copies go to `stash`, its packed target goes.
    /// Returns the item. Refused for no such slot.
    pub fn take_pouch_slot(&mut self, i: usize, stash: &mut IndexMap<String, i64>) -> Result<String, String> {
        let item = self.pouch_slots().get(i).cloned().ok_or("That pouch slot is empty.")?;
        let n = self.pouch_count(&item);
        if n > 0 {
            *stash.entry(item.clone()).or_insert(0) += n;
        }
        self.pouch.retain(|p| *p != item);
        self.pouch_packed.shift_remove(&item);
        Ok(item)
    }

    /// INTENT2-U: moves pouch slot `i` to place `j` (`pouch_slots` order, `j` clamped to the last), copies and packed
    /// target with it. Every slot then has a packed target (one without takes its current count in `1..stack limit`, as
    /// at combat start), so the order holds. Refused for `i == j` or no slot `i`.
    pub fn move_pouch_slot(&mut self, d: &Data, i: usize, j: usize) -> Result<(), String> {
        let mut slots = self.pouch_slots();
        if i == j || i >= slots.len() {
            return Err("Nothing to move.".into());
        }
        let item = slots.remove(i);
        slots.insert(j.min(slots.len()), item);
        let mut pouch = Vec::new();
        let mut packed = IndexMap::new();
        for s in &slots {
            pouch.extend(self.pouch.iter().filter(|p| *p == s).cloned());
            packed.insert(s.clone(), self.pouch_packed.get(s).copied().unwrap_or_else(|| self.pouch_count(s).clamp(1, stack_limit(d, s))));
        }
        self.pouch = pouch;
        self.pouch_packed = packed;
        Ok(())
    }

    /// INTENT2-U: `set_packed`, then the copies above the new target go to `stash`.
    pub fn pack_pouch(&mut self, d: &Data, item: &str, n: i64, stash: &mut IndexMap<String, i64>) -> Result<(), String> {
        self.set_packed(d, item, n)?;
        while self.pouch_count(item) > n {
            let Some(k) = self.pouch.iter().rposition(|p| p == item) else { break };
            *stash.entry(self.pouch.remove(k)).or_insert(0) += 1;
        }
        Ok(())
    }

    /// RULESET1 9 D1-D5 for a member's saved deck (the content check's unit deck rules, ownership from the member):
    /// actives are owned families, never a basic (a `basic` skill, a deck ruleset's `basics` or its class's `basics`,
    /// CLASS-BASICS); passive and keystone
    /// hold known passives of that `slot` the member owns (its `passives`, D4); the counts fit the `deck_slots` of
    /// every ruleset with the deck rule; order maps an intent to families of that intent among the actives and the
    /// basics. Nothing repeats.
    pub fn deck_errors(&self, d: &Data, at: &str) -> Vec<String> {
        self.deck_pass(d, at, false).0
    }

    /// R4-2 M1 (FB-138): drops the saved deck entries the current data disallows (`deck_errors`' rules: an entry with
    /// a problem is not kept, a list past a deck ruleset's slots is cut to them, an order or follow-up list is read
    /// against the entries kept) and returns one load warning per problem. A valid deck (or one with shape problems,
    /// `deck_problems`, which stay errors) is untouched; a deck still invalid after the drop is reset to the automatic
    /// deck with a warning.
    pub fn sanitize_deck(&mut self, d: &Data) -> Vec<String> {
        let at = format!("Unit '{}'", self.id);
        if self.deck.is_empty() || !self.deck_problems.is_empty() || d.entry("units", &self.base_unit).is_empty() || self.deck_errors(d, &at).is_empty() {
            return Vec::new();
        }
        let (dropped, kept) = self.deck_pass(d, &at, true);
        self.deck = kept;
        let mut warnings: Vec<String> = dropped.into_iter().map(|e| format!("{e} Dropped from the saved deck (the data no longer allows it).")).collect();
        if !self.deck_errors(d, &at).is_empty() {
            self.deck = Deck::default();
            warnings.push(format!("{at}: the saved deck no longer fits the data; it is reset to the automatic deck."));
        }
        warnings
    }

    /// The deck rules over the saved deck: (problems, the deck of the entries without one). `clean` false reads every
    /// list against the deck as saved (`deck_errors`); true reads the order and follow-ups against the kept entries and
    /// cuts a list past a ruleset's slots (`sanitize_deck`).
    fn deck_pass(&self, d: &Data, at: &str, clean: bool) -> (Vec<String>, Deck) {
        let mut errors = Vec::new();
        if self.deck.is_empty() {
            return (errors, self.deck.clone());
        }
        let rules: Vec<rpgkit_rules::ruleset::Ruleset> =
            d.cat.rulesets.values().map(rpgkit_rules::ruleset::Ruleset::from_row).filter(|r| r.has(rpgkit_rules::ruleset::DECK)).collect();
        let mut basics: Vec<String> = Vec::new();
        for b in rules.iter().flat_map(|r| r.numbers.basics.iter()) {
            if !basics.contains(b) {
                basics.push(b.clone());
            }
        }
        // CLASS-BASICS: the member's class basics are basics too (open, no slot)
        if !rules.is_empty() {
            basics = rpgkit_rules::DeckRule { basics, ..Default::default() }.with_class(&d.cat, &self.class_id(d)).basics;
        }
        // each repeat is a problem; the flags mark the repeats (the first one stands)
        let twice = |label: &str, ids: &[String], seen: &mut Vec<String>, errors: &mut Vec<String>| -> Vec<bool> {
            let mut flags = Vec::new();
            for id in ids {
                let again = seen.contains(id);
                if again {
                    errors.push(format!("{at}: deck {label} lists '{id}' twice."));
                }
                flags.push(again);
                seen.push(id.clone());
            }
            flags
        };
        let keep = |ids: &[String], bad: &[bool]| -> Vec<String> { ids.iter().zip(bad).filter(|(_, b)| !**b).map(|(id, _)| id.clone()).collect() };
        let mut kept = Deck::default();
        let owned = self.owned_families_anywhere(d);
        // U3.2-S3 (FB-63 (2)): the passives it owns as a build gathers them (character sources now, then saved ids)
        let owned_passives = progression::member_passives(d, self);
        let mut bad = twice("active", &self.deck.active, &mut Vec::new(), &mut errors);
        for (i, fam) in self.deck.active.iter().enumerate() {
            let before = errors.len();
            if d.cat.tier_one(fam).is_none() {
                errors.push(format!("{at}: deck active: unknown skill family '{fam}'."));
            } else if d.cat.is_basic_family(fam) || basics.contains(fam) || d.cat.is_system(fam) {
                errors.push(format!("{at}: deck active: '{fam}' is a basic (open to every unit, no slot)."));
            } else if !owned.contains(fam) {
                errors.push(format!("{at}: deck active: family '{fam}' is not owned."));
            }
            bad[i] |= errors.len() > before;
        }
        kept.active = keep(&self.deck.active, &bad);
        let mut seen_passives = Vec::new();
        for (slot, ids) in [("passive", &self.deck.passive), ("keystone", &self.deck.keystone)] {
            let mut bad = twice(slot, ids, &mut seen_passives, &mut errors);
            for (i, id) in ids.iter().enumerate() {
                let before = errors.len();
                match d.cat.passives.get(id) {
                    None => errors.push(format!("{at}: deck {slot}: unknown passive '{id}'.")),
                    Some(p) if p.slot != slot => errors.push(format!("{at}: deck {slot}: '{id}' does not take the {slot} slot.")),
                    Some(_) if !owned_passives.contains(id) => errors.push(format!("{at}: deck {slot}: passive '{id}' is not owned.")),
                    Some(_) => {}
                }
                bad[i] |= errors.len() > before;
            }
            *(if slot == "passive" { &mut kept.passive } else { &mut kept.keystone }) = keep(ids, &bad);
        }
        for r in &rules {
            let s = r.numbers.deck_slots;
            let counted = if clean { &kept } else { &self.deck };
            let lens = [counted.active.len(), counted.passive.len(), counted.keystone.len()];
            for ((slot, max), n) in [("active", s.active), ("passive", s.passive), ("keystone", s.keystone)].into_iter().zip(lens) {
                if n as i64 > max as i64 {
                    errors.push(format!("{at}: deck: {n} {slot} entries (the {} ruleset's deck slots allow {max}).", r.name));
                    if clean {
                        let list = match slot {
                            "active" => &mut kept.active,
                            "passive" => &mut kept.passive,
                            _ => &mut kept.keystone,
                        };
                        list.truncate(usize::try_from(max).unwrap_or(0));
                    }
                }
            }
        }
        let actives = if clean { kept.active.clone() } else { self.deck.active.clone() };
        // A4.2-K1b (INTENT2 IR15, IR21): under a deck ruleset with `field_intent` the order reads the derived rows
        // (`intent::allowed`) and may name `field`; else the legacy single intent
        let field_rules = rules.iter().find(|r| r.has(rpgkit_rules::ruleset::FIELD_INTENT));
        let rows_of = |fam: &str| -> Vec<String> {
            match d.cat.tier_one(fam) {
                None => Vec::new(),
                Some(s) => match field_rules {
                    Some(r) => rpgkit_rules::intent::allowed(r, &rpgkit_rules::intent::IntentInput::of_def(s), &s.round_intent).into_iter().map(String::from).collect(),
                    None => vec![rpgkit_battle::round_intent::skill_intent(s).to_string()],
                },
            }
        };
        let mut names: Vec<&str> = rpgkit_rules::ruleset::INTENTS.to_vec();
        if field_rules.is_some() {
            names.push(rpgkit_rules::intent::FIELD);
        }
        let mut seen = Vec::new();
        for (intent, list) in &self.deck.order {
            if !names.contains(&intent.as_str()) {
                errors.push(format!("{at}: deck order: '{intent}' is not an intent (one of {}).", names.join(", ")));
                continue;
            }
            // IR15: under field_intent a skill may stand in each of its rows' lists (once per list)
            if field_rules.is_some() {
                seen.clear();
            }
            let mut bad = twice("order", list, &mut seen, &mut errors);
            for (i, fam) in list.iter().enumerate() {
                if !actives.contains(fam) && !basics.contains(fam) {
                    errors.push(format!("{at}: deck order {intent}: '{fam}' is neither an active of the deck nor a basic."));
                    bad[i] = true;
                    continue;
                }
                let rows = rows_of(fam);
                if !rows.contains(intent) {
                    let of = rows.join(" or ");
                    errors.push(format!("{at}: deck order {intent}: '{fam}' is of intent {}.", if of.is_empty() { "none (unknown family)" } else { &of }));
                    bad[i] = true;
                }
            }
            let list_kept = keep(list, &bad);
            if !list_kept.is_empty() || list.is_empty() {
                kept.order.insert(intent.clone(), list_kept);
            }
        }
        // INPUT1 FL5: each list follows an opener of the deck (an active or a basic, not neutral) with families of the
        // deck of the opener's intent, none twice (the opener may stand in its own list)
        let intent_of = |fam: &str| rows_of(fam);
        let in_deck = |fam: &str| actives.iter().chain(&basics).any(|f| f == fam);
        let label = |id: &str| match rpgkit_battle::round_intent::RoundIntent::parse(id) {
            Some(i) => i.name().to_string(),
            None if id.is_empty() => "no intent".into(),
            None => id.to_string(),
        };
        for (key, list) in &self.deck.follow_ups {
            let of = intent_of(key);
            if !in_deck(key) || of.is_empty() || of.iter().any(|i| i == rpgkit_rules::ruleset::NEUTRAL) {
                errors.push(format!("{at}: Follow-ups for '{key}': not an opener in the deck."));
                continue;
            }
            let mut seen: Vec<&String> = Vec::new();
            let mut list_kept = Vec::new();
            for fam in list {
                if seen.contains(&fam) {
                    errors.push(format!("{at}: Follow-ups after '{key}': '{fam}' twice."));
                    continue;
                }
                seen.push(fam);
                if !in_deck(fam) {
                    errors.push(format!("{at}: Follow-ups after '{key}': '{fam}' is not in the deck."));
                    continue;
                }
                let has = intent_of(fam);
                if !has.iter().any(|i| of.contains(i)) {
                    let text = |rows: &[String]| if rows.is_empty() { label("") } else { rows.iter().map(|r| label(r)).collect::<Vec<_>>().join(" or ") };
                    errors.push(format!("{at}: Follow-ups after '{key}': '{fam}' is {}, not {}.", text(&has), text(&of)));
                    continue;
                }
                list_kept.push(fam.clone());
            }
            if !list_kept.is_empty() {
                kept.follow_ups.insert(key.clone(), list_kept);
            }
        }
        // PK1-S2a (PK1 4.2, rule channel): each channeled mark is an active of the deck with a channeled version (IR21 single
        // intent, no channel none or only), none twice, within every channel ruleset's channeled actives
        // PR5-26 (audit B3): an automatic deck (no actives listed) marks the actives each channel ruleset's deck rule
        // fills (the battle's own list, truncated to its active slots); with no such ruleset, the families it owns
        let autos: Vec<(String, Vec<String>)> = if actives.is_empty() && !self.deck.channeled.is_empty() {
            // the member as it fights: its deck (the marks keep it non-empty, so it replaces the base unit's) with no
            // actives listed, built without the validation that called this
            let mut work = self.clone();
            work.deck.active.clear();
            // PR5-26 review F3: in the member's own world (its class path there; a save holds no played world)
            let world = aegis_content::split_qid(&self.lore_ref).map(|(w, _)| w).unwrap_or_default();
            rules
                .iter()
                .filter(|r| r.has(rpgkit_rules::ruleset::CHANNEL))
                .filter_map(|r| {
                    let rule = rpgkit_rules::DeckRule::of(r)?;
                    Some((r.name.clone(), progression::build_unchecked(d, &work, &world, false, Some(&rule))?.unit.active_families()))
                })
                .collect()
        } else {
            Vec::new()
        };
        let mut bad = twice("channeled", &self.deck.channeled, &mut Vec::new(), &mut errors);
        for (i, fam) in self.deck.channeled.iter().enumerate() {
            let before = errors.len();
            let in_deck = if !actives.is_empty() {
                actives.contains(fam)
            } else if autos.is_empty() {
                owned.contains(fam)
            } else {
                autos.iter().all(|(_, auto)| auto.contains(fam))
            };
            if !in_deck {
                errors.push(format!("{at}: deck channeled: '{fam}' is not an active of the deck."));
            } else if d.cat.tier_one(fam).is_some_and(|s| s.channel.none || s.channel.only || !rpgkit_rules::intent::single_intent(&rpgkit_rules::intent::IntentInput::of_def(s))) {
                errors.push(format!("{at}: deck channeled: '{fam}' has no channeled version."));
            }
            bad[i] |= errors.len() > before;
        }
        kept.channeled = keep(&self.deck.channeled, &bad);
        // PK2-S2: with body slots in the catalog a member marks one per graft, whatever the ruleset
        if !d.cat.body_slots.is_empty() {
            let max = self.graft_count();
            let n = if clean { kept.channeled.len() } else { self.deck.channeled.len() };
            if n > max {
                errors.push(format!("{at}: deck: {n} channeled entries (one per graft; the member holds {max})."));
                if clean {
                    kept.channeled.truncate(max);
                }
            }
            return (errors, kept);
        }
        let channel_rows = d.cat.rulesets.values().map(rpgkit_rules::ruleset::Ruleset::from_row).filter(|r| r.has(rpgkit_rules::ruleset::CHANNEL));
        for r in channel_rows {
            let max = r.numbers.channeled_actives.max(0);
            let n = if clean { kept.channeled.len() } else { self.deck.channeled.len() };
            if n as i64 > i64::from(max) {
                errors.push(format!("{at}: deck: {n} channeled entries (the {} ruleset's channeled actives allow {max}).", r.name));
                if clean {
                    kept.channeled.truncate(usize::try_from(max).unwrap_or(0));
                }
            }
        }
        (errors, kept)
    }

    /// PR4-2 (AUDIT-08): drops the saved loadout and gear entries the current data disallows and returns one load
    /// warning per dropped entry (as `sanitize_deck` does for the deck): a skill-loadout family that is unknown, not
    /// owned, listed twice or past the slot caps; a `loadout_ranks` entry `loadout_errors` refuses (a basic, unowned or
    /// unknown family, a rank the family lacks); worn gear in an unknown slot or of an unknown item (its item family
    /// left the data: nothing to keep), or that the member can no longer wear (`can_wear`, an off-hand held with a
    /// two-handed main): a known item goes to `stash`. A member whose base unit or class is gone is left to `validate`;
    /// a valid member is untouched.
    pub fn sanitize_loadout(&mut self, d: &Data, stash: &mut IndexMap<String, i64>) -> Vec<String> {
        let at = format!("Unit '{}'", self.id);
        let entry = d.entry("units", &self.base_unit);
        if entry.is_empty() || !d.cat.classes.contains_key(&self.class_id(d)) {
            return Vec::new();
        }
        let mut warnings = Vec::new();
        let dropped = |problem: String, what: &str| format!("{at}: {problem} Dropped from the saved {what} (the data no longer allows it).");
        if !self.loadout_errors(d, &at).is_empty() {
            let owned = self.owned_families_anywhere(d);
            let mut seen: Vec<String> = Vec::new();
            for (list, cap) in [(&mut self.loadout_class, rpgkit_rules::stats::MAX_CLASS_SLOTS), (&mut self.loadout_extra, rpgkit_rules::stats::MAX_EXTRA_SLOTS)] {
                let mut kept: Vec<String> = Vec::new();
                for fam in list.iter() {
                    let problem = if d.cat.tier_one(fam).is_none() {
                        format!("loadout family '{fam}' is unknown.")
                    } else if !owned.contains(fam) {
                        format!("loadout family '{fam}' is not owned.")
                    } else if seen.contains(fam) {
                        format!("loadout lists family '{fam}' twice.")
                    } else if kept.len() >= cap {
                        format!("loadout family '{fam}' is past the {cap} slots.")
                    } else {
                        seen.push(fam.clone());
                        kept.push(fam.clone());
                        continue;
                    };
                    warnings.push(dropped(problem, "loadout"));
                }
                *list = kept;
            }
            let ranks = std::mem::take(&mut self.loadout_ranks);
            for (fam, rank) in ranks {
                let problem = if d.cat.is_basic_family(&fam) {
                    format!("rank set for family '{fam}', which is a basic family.")
                } else if !owned.contains(&fam) {
                    format!("rank set for family '{fam}' that is not owned.")
                } else if !d.cat.family_rows(&fam).iter().any(|s| s.tier as i64 == rank) {
                    format!("family '{fam}' has no rank {rank}.")
                } else {
                    self.loadout_ranks.insert(fam, rank);
                    continue;
                };
                warnings.push(dropped(problem, "loadout ranks"));
            }
        }
        let to_stash = |item: &str, stash: &mut IndexMap<String, i64>| *stash.entry(item.to_string()).or_insert(0) += 1;
        let worn = std::mem::take(&mut self.equipment);
        for (slot, item) in worn {
            let problem = if !LOADOUT_SLOTS.contains(&slot.as_str()) {
                format!("unknown equipment slot '{slot}'.")
            } else if item == NO_ITEM {
                // PR5-26 (audit B2): an emptied slot stays empty
                self.equipment.insert(slot, item);
                continue;
            } else if !d.has("items", &item) {
                format!("unknown item '{item}' in {slot}.")
            } else if let Err(e) = self.can_wear(d, &item, &slot) {
                e
            } else {
                self.equipment.insert(slot, item);
                continue;
            };
            if d.has("items", &item) {
                to_stash(&item, stash);
                warnings.push(format!("{at}: {problem} Moved from the saved gear to the stash (the data no longer allows it)."));
            } else {
                warnings.push(dropped(problem, "gear"));
            }
        }
        let held = |slot: &str, eq: &IndexMap<String, String>| eq.get(slot).and_then(|i| d.cat.items.get(i)).cloned();
        if let (Some(main), Some(off)) = (held("main_hand", &self.equipment), held("off_hand", &self.equipment)) {
            if !off_hand_ok(Some(&main), &off) {
                if let Some(item) = self.equipment.shift_remove("off_hand") {
                    to_stash(&item, stash);
                }
                warnings.push(format!("{at}: {} can't be held with the two-handed {}. Moved from the saved gear to the stash (the data no longer allows it).", off.name, main.name));
            }
        }
        warnings
    }

    pub fn loadout_errors(&self, d: &Data, at: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if self.loadout_class.len() > rpgkit_rules::stats::MAX_CLASS_SLOTS {
            errors.push(format!("{at}: at most {} class families in the loadout.", rpgkit_rules::stats::MAX_CLASS_SLOTS));
        }
        if self.loadout_extra.len() > rpgkit_rules::stats::MAX_EXTRA_SLOTS {
            errors.push(format!("{at}: at most {} extra families in the loadout.", rpgkit_rules::stats::MAX_EXTRA_SLOTS));
        }
        let owned = self.owned_families_anywhere(d);
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

/// ROSTER1 L1, L2, 11.4: a member who left the party (`party.departed[]`): its whole state at departure (choices and
/// gear record; the gear itself went to the stash), how and why it left, the encounter ("" outside a battle), the day
/// (`gs.time`) and the keepsake item paid for it ("" = none).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Departed {
    pub member: UnitState,
    /// One of `rpgkit_data::DEPART_LIVES`.
    pub life: String,
    /// One of `rpgkit_data::DEPART_CAUSES` or a unit track row's `cause` (`Catalog::depart_causes`).
    pub cause: String,
    pub encounter: String,
    pub day: i64,
    pub keepsake: String,
}

impl Departed {
    pub fn to_value(&self) -> Value {
        Value::Object(json::obj([
            ("member", self.member.to_value()),
            ("life", Value::String(self.life.clone())),
            ("cause", Value::String(self.cause.clone())),
            ("encounter", Value::String(self.encounter.clone())),
            ("day", int(self.day)),
            ("keepsake", Value::String(self.keepsake.clone())),
        ]))
    }

    pub fn from_dict(d: &Dict) -> Self {
        Self {
            member: UnitState::from_dict(json::dget(d, "member")),
            life: json::s(d, "life"),
            cause: json::s(d, "cause"),
            encounter: json::s(d, "encounter"),
            day: json::i(d, "day", 0),
            keepsake: json::s(d, "keepsake"),
        }
    }
}

/// ROSTER1 BK7: the played world's resolved roster settings, kept on the party like `max_active` (never saved; set at
/// create, load and a world change) so the `roster.` read keys need no `Data`.
#[derive(Debug, Clone, PartialEq)]
pub struct RosterCaps {
    /// The world's party max (`bunks` 0 means this many bunks, BK1).
    pub party_max: i64,
    /// The world's `roster` block (11.1; {} = today's rules).
    pub manifest: RosterManifest,
}

impl Default for RosterCaps {
    fn default() -> Self {
        Self { party_max: PARTY_MAX, manifest: RosterManifest::from_obj(&Dict::new()) }
    }
}

impl RosterCaps {
    /// BK1: the bunk cap with the bunks flag at `flag_value`.
    pub fn bunk_cap(&self, flag_value: i64) -> i64 {
        self.manifest.bunk_cap(self.party_max as i32, flag_value.clamp(0, i32::MAX as i64) as i32) as i64
    }

    /// BK3: the world sets its own bunks (`bunks` above 0; else the RECRUIT1 "party is full" wording).
    pub fn has_bunks(&self) -> bool {
        self.manifest.bunks > 0
    }

    /// The world has a `roster` block (not {}: today's rules, where a Godot save may hold more members than the party
    /// max and loads without a word).
    pub fn has_block(&self) -> bool {
        self.manifest != RosterManifest::from_obj(&Dict::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartyState {
    pub members: Vec<UnitState>,
    pub active_ids: Vec<String>,
    /// The world's cap on active members (never saved).
    pub max_active: i64,
    /// ROSTER1 BK7: the world's roster settings (never saved).
    pub roster: RosterCaps,
    /// E2.2-S1 (FB-53a): the game's purse caps (never saved; set at create and load from `Data::purse`).
    pub purse: PurseCaps,
    pub stash: IndexMap<String, i64>,
    pub gold: i64,
    pub supplies: i64,
    /// ROSTER1 L1: the members who left (captured, dead, retired, dismissed, lost), departure order. Saved as
    /// `departed` only when not empty (the unreleased save v13 carries it: a save without it reads none).
    pub departed: Vec<Departed>,
}

impl Default for PartyState {
    fn default() -> Self {
        Self { members: Vec::new(), active_ids: Vec::new(), max_active: PARTY_MAX, roster: RosterCaps::default(), purse: PurseCaps::default(), stash: IndexMap::new(), gold: 0, supplies: 0, departed: Vec::new() }
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
        self.gold = (self.gold + amount.max(0)).clamp(0, self.purse.purse_gold_cap);
        self.gold - before
    }

    pub fn add_supplies(&mut self, n: i64) -> i64 {
        let before = self.supplies;
        self.supplies = (self.supplies + n.max(0)).clamp(0, self.purse.supplies_max);
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

    /// E2.2-S1 (FB-53a): takes the game's purse caps, lowering gold and supplies above them to them (a save made under
    /// higher caps loads, never fails the check); a damaged value (below 0) is left for `validate`.
    pub fn apply_purse(&mut self, caps: PurseCaps) {
        self.purse = caps;
        self.gold = self.gold.min(caps.purse_gold_cap);
        self.supplies = self.supplies.min(caps.supplies_max);
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
        let mut out = json::obj([
            ("members", Value::Array(self.members.iter().map(UnitState::to_value).collect())),
            ("active_ids", strs(&self.active_ids)),
            ("stash", Value::Object(stash)),
            ("gold", int(self.gold)),
            ("supplies", int(self.supplies)),
        ]);
        // ROSTER1 11.4: only a party someone left saves `departed`
        if !self.departed.is_empty() {
            out.insert("departed".into(), Value::Array(self.departed.iter().map(Departed::to_value).collect()));
        }
        Value::Object(out)
    }

    /// ROSTER1 L1: the departed record of member `id`.
    pub fn departed_of(&self, id: &str) -> Option<&Departed> {
        self.departed.iter().find(|x| x.member.id == id)
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
            roster: RosterCaps::default(),
            purse: PurseCaps::default(),
            stash: d.get("stash").map(json::ints).unwrap_or_default(),
            gold: whole("gold"),
            supplies: whole("supplies"),
            departed: json::aget(d, "departed").iter().map(|x| Departed::from_dict(json::dict(x))).collect(),
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
        // ROSTER1 11.4: a departed member's id is never another's; its life and cause are known (its state is a
        // record, not rebuilt, so its build is not checked)
        for x in &self.departed {
            let id = &x.member.id;
            if !json::id_ok(id) {
                errors.push(format!("Departed member id '{id}' must be non-empty [a-z0-9_]."));
            } else if seen.contains(&id) {
                errors.push(format!("Departed member id '{id}' is also a member or departed twice."));
            }
            seen.push(id);
            if !DEPART_LIVES.contains(&x.life.as_str()) {
                errors.push(format!("Departed '{id}': unknown life '{}' (known: {}).", x.life, DEPART_LIVES.join(", ")));
            }
            if !d.cat.depart_cause_ok(&x.cause) {
                errors.push(format!("Departed '{id}': unknown cause '{}' (known: {}).", x.cause, d.cat.depart_causes().join(", ")));
            }
            if x.day < 0 {
                errors.push(format!("Departed '{id}': day must be 0 or more."));
            }
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
        let PurseCaps { supplies_max, purse_gold_cap } = self.purse;
        if self.gold < 0 || self.gold > purse_gold_cap {
            errors.push(format!("gold must be 0..{purse_gold_cap}."));
        }
        if self.supplies < 0 || self.supplies > supplies_max {
            errors.push(format!("supplies must be 0..{supplies_max}."));
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
    /// STORYUI1 4.8: the running sequence, the vignette queue and the seen list; saved as the key `story` only when
    /// not empty.
    pub story: crate::story::Story,
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
            story: crate::story::Story::default(),
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
        g.party.roster = crate::roster::caps(d, world_id);
        g.party.apply_purse(d.purse);
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

    /// A flag condition's value: a read key (`meter.`, `campaign.`; CAMPAIGN1 11.4) from the campaign, a `roster.` key
    /// (ROSTER1 RK1) from the party, any other key from the flags.
    pub fn read_flag(&self, key: &str) -> Option<Value> {
        if key.starts_with(rpgkit_data::ROSTER_PREFIX) {
            crate::roster::read_key(self, key)
        } else if rpgkit_data::reserved_key(key) {
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

    /// MAPNAV1-U: the members who can lead on the maps, roster order: the active members `standing` keeps (the caller
    /// says who is downed).
    pub fn leader_choices(&self, standing: impl Fn(&UnitState) -> bool) -> Vec<&UnitState> {
        self.party.active_members().into_iter().filter(|m| standing(m)).collect()
    }

    /// MAPNAV1-U: the member who leads on the maps: the saved `explore.leader` while that member can lead
    /// (`leader_choices`), else the first who can, else the first active member (None without one).
    pub fn explore_leader(&self, standing: impl Fn(&UnitState) -> bool) -> Option<&UnitState> {
        let choices = self.leader_choices(standing);
        let saved = json::s(&self.explore, LEADER_KEY);
        choices.iter().find(|m| m.id == saved).or(choices.first()).copied().or_else(|| self.party.active_members().first().copied())
    }

    /// MAPNAV1-U: makes member `id` the leader (saved as `explore.leader`); false (nothing changes) when `id` is not one
    /// of the `leader_choices`.
    pub fn set_explore_leader(&mut self, id: &str, standing: impl Fn(&UnitState) -> bool) -> bool {
        if !self.leader_choices(standing).iter().any(|m| m.id == id) {
            return false;
        }
        self.explore.insert(LEADER_KEY.into(), Value::String(id.into()));
        true
    }

    /// MAPNAV1-U: passes the lead `by` places along the `leader_choices` (1 the next, -1 the one before, wrapping);
    /// the new leader's id, None when no one can lead.
    pub fn cycle_explore_leader(&mut self, by: i64, standing: impl Fn(&UnitState) -> bool) -> Option<String> {
        let ids: Vec<String> = self.leader_choices(&standing).iter().map(|m| m.id.clone()).collect();
        if ids.is_empty() {
            return None;
        }
        let now = self.explore_leader(&standing).map(|m| m.id.clone()).unwrap_or_default();
        let at = ids.iter().position(|i| *i == now).unwrap_or(0) as i64;
        let next = ids[(at + by).rem_euclid(ids.len() as i64) as usize].clone();
        self.set_explore_leader(&next, &standing);
        Some(next)
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
        if !self.story.is_empty() {
            out.insert("story".into(), self.story.to_value());
        }
        Value::Object(out)
    }

    pub fn from_dict(data: &Data, d: &Dict) -> Self {
        let mut g = Self { world_id: json::s(d, "world_id"), location_id: json::s(d, "location_id"), ..Default::default() };
        g.party = PartyState::from_dict(json::dget(d, "party"));
        g.party.apply_cap(data.party_max(&g.world_id));
        g.party.roster = crate::roster::caps(data, &g.world_id);
        g.party.apply_purse(data.purse);
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
        // STORYUI1 4.8: an absent block reads empty (older saves, worlds without a story)
        if let Some(raw) = d.get("story") {
            g.story = crate::story::Story::from_value(raw, &mut g.load_problems);
        }
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
        errors.extend(self.story.validate());
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
    let has_leader = e.contains_key(LEADER_KEY);
    let mut ok = e.len() == EXPLORE_KEYS.len() + has_home as usize + has_leader as usize;
    ok = ok && (!has_home || e.get(HOME_WORLD_KEY).and_then(Value::as_str).is_some_and(json::id_ok));
    ok = ok && (!has_leader || e.get(LEADER_KEY).and_then(Value::as_str).is_some_and(json::id_ok));
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
