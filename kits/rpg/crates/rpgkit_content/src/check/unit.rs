//! Class and unit rows (ClassUnitValidator errors and warnings, StatRules.validate_class / validate_spend).

use super::{capitalize, View, LOADOUT_SLOTS, NO_ITEM};
use aegis_content::world::Catalogs;
use aegis_core::fmt::fixed;
use aegis_core::json::{self, Dict};
use rpgkit_data::{ClassDef, Deck, ItemDef, SkillDef, STATS};
use rpgkit_rules::{ruleset, unit_budget};
use rpgkit_rules::stats::{spend_pool, Proficiency, CLASS_BUDGET, DEFAULT_SCALING, MAX_CLASS_SLOTS, MAX_EXTRA_SLOTS, MAX_LEVEL, PROF_MAX_LEVEL};
use indexmap::IndexMap;
use serde_json::Value;

const STAT_FLOOR: i64 = 6;
const STAT_CEILING: i64 = 18;
const GROWTH_TOTAL: i64 = unit_budget::GROWTH_BUDGET as i64;
const GROWTH_STAT_MAX: i64 = 3;
/// StatRules.ADVANCED_MIN_LEVEL: an advanced family may not sit on a class path below this level.
const ADVANCED_MIN_LEVEL: i64 = 5;
const MIN_SCALING: f64 = 1.0;
const MAX_SCALING: f64 = 2.0;
/// Proficiency.WEAPON_TYPES (RULESET1-S adds `polearm` and `scourge`, CAST1-S `censer`, `bone_saw` and `sprayer`); U3.2-S2:
/// read only without a family table (`skill::weapon_tokens`).
pub const WEAPON_TYPES: [&str; 18] = ["sword", "axe", "mace", "hammer", "fist", "dagger", "bow", "crossbow", "staff", "wand", "polearm", "scourge", "censer", "bone_saw", "sprayer", "orb", "tome", "shield"];
pub const DUPLICATE_ACCESSORY_ERROR: &str = "The same accessory cannot fill both accessory slots.";
/// UNITDRILL1: the unit field listing the items of its own gear it wears with no class or drill training. U3.2-S3
/// (FB-34): retired (untrained gear is a soft wield penalty, never refused); nothing reads it, an empty list stores as no
/// key, and the asset lane drops the last one.
pub const WEARS_UNTRAINED: &str = "wears_untrained";
/// W.2-S3 (FB-139): the unit field giving the levels an enemy gains per world stage (0 = none, stored as no key).
pub const GROWTH_PER_STAGE: &str = "growth_per_stage";
/// The top of `growth_per_stage` (the schema field's `max`).
pub const GROWTH_PER_STAGE_MAX: i64 = 10;
/// U4.2-S3 (FB-29): the character field naming its temperament passive.
pub const TEMPERAMENT: &str = "temperament";

type Ints = IndexMap<String, i64>;

/// StatRules.validate_class: every base stat in range, the class budget, the growth per level.
fn validate_class(base: &Ints, growth: &Ints) -> Vec<String> {
    let mut errors = validate_base(base);
    let growth_total: i64 = growth.values().sum();
    if growth_total != GROWTH_TOTAL {
        errors.push(format!("Growth totals {growth_total}, every class gains exactly {GROWTH_TOTAL} per level."));
    }
    for s in STATS {
        if !(0..=GROWTH_STAT_MAX).contains(&growth.get(s).copied().unwrap_or(0)) {
            errors.push(format!("Growth {s} must be 0-{GROWTH_STAT_MAX} per level."));
        }
    }
    errors
}

/// The base stats half of `validate_class`: every stat in range and the class budget.
fn validate_base(base: &Ints) -> Vec<String> {
    let mut errors = Vec::new();
    let mut total = 0;
    for s in STATS {
        let v = base.get(s).copied().unwrap_or(STAT_FLOOR);
        total += v;
        if !(STAT_FLOOR..=STAT_CEILING).contains(&v) {
            errors.push(format!("Base {s} {v} must be between {STAT_FLOOR} and {STAT_CEILING}."));
        }
    }
    if total != CLASS_BUDGET as i64 {
        errors.push(format!("Base stats total {total}, every class must total {CLASS_BUDGET} (every stat set)."));
    }
    errors
}

/// U2.2-S1: a class row given as shares (`base_shares`, `growth_shares`): both vectors sum to 1, each share is 0-1,
/// no `base_stats` or `growth` beside them, and the stats the unit budget gives keep the integer row's limits (base
/// 6-18 at level 1, 0-3 gained per stat at every level).
pub(crate) fn validate_shares(e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    for (key, label) in [("base_shares", "Base shares"), ("growth_shares", "Growth shares")] {
        let raw = json::get(e, key).as_object().cloned().unwrap_or_default();
        let shares: unit_budget::Shares = raw.iter().map(|(k, v)| (k.clone(), v.as_f64().unwrap_or(-1.0))).collect();
        for (k, v) in &shares {
            if !STATS.contains(&k.as_str()) {
                errors.push(format!("{label}: '{k}' is not a stat."));
            } else if !(0.0..=1.0).contains(v) {
                errors.push(format!("{label} {k} must be a number from 0 to 1."));
            }
        }
        if !unit_budget::sums_to_one(&shares) {
            errors.push(format!("{label} total {}, a share vector must total 1.", fixed(shares.values().sum::<f64>(), 4)));
        }
    }
    for key in ["base_stats", "growth"] {
        if json::get(e, key).as_object().is_some_and(|m| !m.is_empty()) {
            errors.push(format!("A class given as shares cannot also set {key}."));
        }
    }
    if !errors.is_empty() {
        return errors;
    }
    let shares = ClassDef::from_obj(e).unit_shares();
    let base: Ints = unit_budget::stats_at(&shares, 1).into_iter().map(|(k, v)| (k, v as i64)).collect();
    errors.extend(validate_base(&base));
    for level in 2..=MAX_LEVEL {
        for (s, gain) in unit_budget::gain_at(&shares, level) {
            let msg = format!("Growth {s} must be 0-{GROWTH_STAT_MAX} per level.");
            if !(0..=GROWTH_STAT_MAX).contains(&(gain as i64)) && !errors.contains(&msg) {
                errors.push(msg);
            }
        }
    }
    errors
}

/// StatRules.validate_spend: empty (follow the class) or exactly the pool of `level`.
fn validate_spend(spend: &Ints, level: i64) -> Vec<String> {
    let mut errors = Vec::new();
    if spend.is_empty() {
        return errors;
    }
    let pool = spend_pool(level.clamp(i32::MIN as i64, i32::MAX as i64) as i32) as i64;
    for (k, v) in spend {
        if !STATS.contains(&k.as_str()) {
            errors.push(format!("Spend: unknown stat '{k}'."));
        } else if *v < 0 {
            errors.push(format!("Spend {k} can't be negative."));
        }
    }
    let total: i64 = spend.values().sum();
    if total != pool {
        errors.push(format!("Spend totals {total}, a level {level} unit spends exactly {pool} (leave it empty to follow the class)."));
    }
    errors
}

/// Equipment.can_equip_in_slot without worn gear kept (the loadout gate; plain dual wield off): a shield or an
/// off-hand weapon in the off hand only, other weapons in the main hand only, an accessory in either accessory slot,
/// other gear in its own slot.
fn fits_slot(item: &ItemDef, slot: &str) -> bool {
    if !item.is_equipment() {
        return false;
    }
    let target = slot.to_uppercase();
    // U3.2-S2: a shield is an item whose family has the engine rule `shield` (`Equipment::is_shield`)
    if item.has_rule(rpgkit_data::SHIELD_RULE) {
        return target == "OFF_HAND";
    }
    if item.item_type == "WEAPON" {
        if item.two_handed {
            return target == "MAIN_HAND";
        }
        if !item.off_hand_category.is_empty() {
            return target == "OFF_HAND";
        }
        return target == "MAIN_HAND";
    }
    if item.slot.starts_with("ACCESSORY") {
        return target.starts_with("ACCESSORY");
    }
    target == item.slot
}

/// WorldFactory.character_passives: its own, its faction's, its disciplines' and its named items', once each. U4.2-S3
/// (FB-29, U4 "Character (drills + temperament)"): its `temperament` comes right after its own passives.
pub fn character_passives(worlds: &Catalogs, character: &Dict, w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut add = |raw: &Value| {
        for id in json::strings(raw) {
            if !out.contains(&id) {
                out.push(id);
            }
        }
    };
    add(json::get(character, "passives"));
    let temperament = json::s(character, TEMPERAMENT);
    if !temperament.is_empty() {
        add(&Value::from(vec![temperament]));
    }
    add(json::get(worlds.resolve(w, "factions", &json::s(character, "faction")), "grants"));
    for r in json::aget(character, "disciplines") {
        add(json::get(worlds.resolve(w, "disciplines", &json::gstr(r)), "passives"));
    }
    for r in json::aget(character, "named_items") {
        add(json::get(worlds.resolve(w, "items", &json::gstr(r)), "grants"));
    }
    out
}

/// ClassUnitValidator._loadout_errors: every item fits its slot; no two-handed weapon with an off-hand item; no
/// accessory twice. U3.2-S3 (FB-34): training is not checked (an untrained item is worn under the wield penalty).
fn loadout_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    for slot in LOADOUT_SLOTS {
        let item_id = json::s(e, slot);
        if item_id.is_empty() {
            continue;
        }
        let item = v.get("items", &item_id);
        if item.is_empty() {
            continue;
        }
        let name = json::s_or(item, "name", &item_id);
        if !fits_slot(&ItemDef::from_obj(item), slot) {
            errors.push(format!("{name} can't be equipped in {}.", capitalize(slot)));
        }
    }
    let main = v.get("items", &json::s(e, "main_hand"));
    let off = v.get("items", &json::s(e, "off_hand"));
    if json::b(main, "two_handed", false) && !off.is_empty() {
        errors.push("A two-handed weapon can't be paired with an off-hand item.".into());
    }
    let (a1, a2) = (json::s(e, "accessory_1"), json::s(e, "accessory_2"));
    if !a1.is_empty() && a1 != NO_ITEM && a1 == a2 {
        errors.push(DUPLICATE_ACCESSORY_ERROR.into());
    }
    errors
}

/// ClassUnitValidator.path_errors: the class skill path {family head: unlock level}.
fn path_errors(v: &View, raw: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(path) = raw.as_object() else {
        errors.push("The skill path must be a dictionary {skill family: unlock level} (a list [{level, skill}] is the old shape).".into());
        return errors;
    };
    for (skill_id, raw_level) in path {
        if !json::is_int(raw_level) {
            errors.push(format!("Skill path: the level of '{skill_id}' must be a whole number 1-{MAX_LEVEL}."));
            continue;
        }
        let level = json::to_int(raw_level);
        let skill = v.get("skills", skill_id);
        if !(1..=MAX_LEVEL as i64).contains(&level) {
            errors.push(format!("Skill path: level {level} of '{skill_id}' is outside 1-{MAX_LEVEL}."));
        }
        if skill.is_empty() {
            errors.push(format!("Skill path: unknown skill '{skill_id}'."));
        } else if json::b(skill, "system", false) {
            errors.push(format!("Skill path: '{skill_id}' is an engine skill (always known)."));
        } else if json::b(skill, "basic", false) {
            errors.push(format!("Skill path: '{skill_id}' is a basic skill; every unit owns the basics from level 1, so they are not valid in a class path."));
        } else {
            if json::i(skill, "tier", 1) != 1 {
                errors.push(format!("Skill path: '{skill_id}' is a higher tier; list the tier 1 skill of the family (tiers open with it)."));
            }
            if json::s_or(skill, "grade", "novice") != "novice" && level < ADVANCED_MIN_LEVEL {
                errors.push(format!("Skill path: {skill_id} is an advanced skill and needs unit level {ADVANCED_MIN_LEVEL} or higher (listed at {level})."));
            }
            for required in json::sget(skill, "requires") {
                match path.get(&required).map(json::to_int) {
                    None => errors.push(format!("Skill path: {skill_id} needs {required} on the same path.")),
                    Some(at) if at > level => errors.push(format!("Skill path: {skill_id} (level {level}) needs {required} first, but {required} is only listed at level {at}.")),
                    Some(_) => {}
                }
            }
        }
    }
    errors
}

/// ClassUnitValidator.class_errors of a normalized class row.
pub fn class_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = if json::get(e, "base_shares").as_object().is_some_and(|m| !m.is_empty()) {
        validate_shares(e)
    } else {
        let mut list = validate_class(&json::ints(json::get(e, "base_stats")), &json::ints(json::get(e, "growth")));
        if json::get(e, "growth_shares").as_object().is_some_and(|m| !m.is_empty()) {
            list.push("Growth shares need base shares (a class gives both vectors or neither).".into());
        }
        list
    };
    // a class always names its body (A1; "" inherits only on units and characters)
    let art = json::s(e, "art");
    if art.is_empty() && !v.assets.roots.is_empty() {
        errors.push("Art '' isn't a body (units/<class>/<variant>/ in the game, kit or engine).".into());
    }
    errors.extend(super::assets::body_errors(v, &art));
    match json::get(e, "proficiency").as_object().filter(|p| !p.is_empty()) {
        None => errors.push("A class needs a gear proficiency dictionary.".into()),
        Some(prof) => {
            let mut weapons = 0;
            let (gear_types, weapon_types) = (super::skill::gear_tokens(v.entries("item_families")), super::skill::weapon_tokens(v.entries("item_families")));
            for (gear, raw) in prof {
                let entry = json::dict(raw);
                if !gear_types.contains(gear) {
                    errors.push(format!("Proficiency: unknown gear type '{gear}'."));
                }
                let level = json::i(entry, "level", 0);
                if !entry.contains_key("level") || !(1..=PROF_MAX_LEVEL as i64).contains(&level) {
                    errors.push(format!("Proficiency {gear}: level must be 1-{PROF_MAX_LEVEL}."));
                }
                let scaling = entry.get("scaling").map_or(DEFAULT_SCALING, |s| json::to_float_or(s, -1.0));
                if !(MIN_SCALING..=MAX_SCALING).contains(&scaling) {
                    errors.push(format!("Proficiency {gear}: scaling must be {}-{}.", fixed(MIN_SCALING, 1), fixed(MAX_SCALING, 1)));
                }
                if weapon_types.contains(gear) {
                    weapons += 1;
                }
            }
            if weapons == 0 {
                errors.push("Proficiency needs at least one weapon type.".into());
            }
        }
    }
    errors.extend(path_errors(v, json::get(e, "skill_path")));
    errors.extend(class_basics_errors(v, e));
    errors.extend(loadout_errors(v, e));
    errors
}

/// CLASS-BASICS (user 2026-11-08): a class's `basics` are skill ids, each a tier 1 skill (an unknown id, an engine skill
/// and a repeat are the schema's reference, filter and list checks); open without a deck slot under the rule `deck`.
fn class_basics_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    for id in json::sget(e, "basics") {
        let skill = v.get("skills", &id);
        if skill.is_empty() {
            continue;
        }
        if json::i(skill, "tier", 1) != 1 {
            errors.push(format!("Class basics: '{id}' is a higher tier; list the tier 1 skill of the family."));
        }
    }
    errors
}

/// ClassUnitValidator.unit_errors of a normalized unit row.
pub fn unit_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let fixed_mode = json::s(e, "stat_mode") == "fixed";
    let class_id = json::s(e, "class_id");
    let arch = v.typed.classes.get(&class_id).filter(|_| !class_id.is_empty());
    if !fixed_mode && arch.is_none() {
        errors.push("Class-derived units need a class (or set the stat model to fixed).".into());
    } else if arch.is_none() {
        // U2.2-S2 (FB-23): no silent fallback to the catalog's first class
        errors.push("Fixed units need a class too (their base stats are the class's at their level).".into());
    }
    if !fixed_mode {
        errors.extend(validate_spend(&json::ints(json::get(e, "spend")), json::i(e, "level", 1)));
    } else if json::get(e, "spend").as_object().is_some_and(|s| !s.is_empty()) {
        errors.push("Fixed units don't spend points.".into());
    }
    errors.extend(loadout_errors(v, e));
    if let Some(a) = arch.filter(|_| !fixed_mode) {
        errors.extend(skill_loadout_errors(v, e, a));
    }
    errors.extend(requires_errors(v, e, arch));
    errors.extend(template_errors(v, e));
    errors.extend(deck_errors(v, e));
    errors.extend(default_intent_errors(v, e));
    // HOOKS1 AT2: the unit's AI tactics
    errors.extend(super::hooks::tactics_errors(json::get(e, "tactics"), "Tactics"));
    // ROSTER1 RK1: a party unit's id is a start-party member's id, never a roster read word
    let id = json::s(e, "id");
    if json::s(e, "side") == "party" && rpgkit_data::ROSTER_WORDS.contains(&id.as_str()) {
        errors.push(format!("Id '{id}' is a roster read word ({}); pick another.", rpgkit_data::ROSTER_WORDS.join(", ")));
    }
    errors
}

/// R4-1 slice 2 (`U2.3-TIERS`, FB-144): a unit's `template` is "" (by its AI role), `class` (its class's shares) or
/// the role or id of a `unit_templates` row.
pub fn template_errors(v: &View, e: &Dict) -> Vec<String> {
    let key = json::s(e, rpgkit_data::TEMPLATE_FIELD);
    if key.is_empty() || key == rpgkit_data::TEMPLATE_CLASS || rpgkit_data::unit_template::lookup(&v.typed.unit_templates, &key).is_some() {
        return Vec::new();
    }
    vec![format!("Stat template '{key}' is not a unit template role or id (leave it empty for the AI role's, or '{}' for the class's).", rpgkit_data::TEMPLATE_CLASS)]
}

/// W.2-S3 (FB-139): a unit's `growth_per_stage` as written (`raw`, before the schema clamps it): absent, or a whole
/// number 0-`GROWTH_PER_STAGE_MAX`.
pub fn growth_per_stage_errors(raw: &Dict) -> Vec<String> {
    match raw.get(GROWTH_PER_STAGE) {
        None => Vec::new(),
        Some(v) if json::is_int(v) && (0..=GROWTH_PER_STAGE_MAX).contains(&json::to_int(v)) => Vec::new(),
        Some(_) => vec![format!("Growth per stage must be a whole number 0-{GROWTH_PER_STAGE_MAX}.")],
    }
}

/// The skill families a unit row owns (its class path up to its level and its extra skills; always-active skills left
/// out), as the deck checks read them (RULESET1 9 D1; HOOKS1 DU2).
pub(crate) fn owned_families(v: &View, e: &Dict) -> Vec<String> {
    let class_id = json::s(e, "class_id");
    let arch = v.typed.classes.get(&class_id).filter(|_| !class_id.is_empty());
    let mut owned_ids: Vec<String> = arch.map(|a| a.path_steps().into_iter().filter(|(l, _)| *l as i64 <= json::i(e, "level", 1)).map(|(_, s)| s).collect()).unwrap_or_default();
    owned_ids.extend(json::sget(e, "extra_skills"));
    families_of(v, &owned_ids)
}

/// RULESET1 TQ23 (a): a unit's `default_intent` (the schema's enum: "" or one of the four intents) is one a ruleset
/// row lists among its intents; without ruleset rows only the enum applies.
pub(crate) fn default_intent_errors(v: &View, e: &Dict) -> Vec<String> {
    let intent = json::s(e, "default_intent");
    let rows = v.entries(ruleset::KIND);
    if intent.is_empty() || !ruleset::DEFAULT_INTENTS.contains(&intent.as_str()) || rows.is_empty() || rows.iter().any(|r| json::sget(r, "intents").contains(&intent)) {
        return Vec::new();
    }
    let lists: Vec<String> = rows.iter().map(|r| format!("{}: {}", json::s(r, "id"), json::sget(r, "intents").join(", "))).collect();
    vec![format!("Default intent: no ruleset lists '{intent}' among its intents ({}).", lists.join("; "))]
}

/// The ids of a deck list (`key` absent = none); a value that is not a list of strings is reported.
fn deck_list(deck: &Dict, key: &str, errors: &mut Vec<String>) -> Vec<String> {
    match deck.get(key) {
        None => Vec::new(),
        Some(Value::Array(a)) if a.iter().all(Value::is_string) => a.iter().map(json::gstr).collect(),
        Some(_) => {
            errors.push(format!("Deck {key} must be a list of ids."));
            Vec::new()
        }
    }
}

/// Pushes "lists 'x' twice" for every repeat of `ids` against `seen` (which grows).
fn repeats(label: &str, ids: &[String], seen: &mut Vec<String>, errors: &mut Vec<String>) {
    for id in ids {
        if seen.contains(id) {
            errors.push(format!("{label} lists '{id}' twice."));
        }
        seen.push(id.clone());
    }
}

/// RULESET1 9 D1-D5 (RULESET1_TASKS 1.5; rule deck, read only under timeline): a unit's `deck` is an object with the
/// keys active, passive, keystone and order (empty or missing = auto). Actives are skill families the unit owns (class
/// path at its level or extra skills), never a basic (a `basic` skill, a ruleset's basics or the unit's class basics,
/// CLASS-BASICS); passive and keystone hold
/// passives of that `slot`; the counts fit the `deck_slots` of every ruleset with the deck rule; order maps an intent
/// to families of that intent among the actives and the basics. Nothing repeats.
fn deck_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(deck) = json::get(e, "deck").as_object() else {
        errors.push("Deck must be an object {active, passive, keystone, order}.".into());
        return errors;
    };
    for k in deck.keys().filter(|k| !Deck::KEYS.contains(&k.as_str())) {
        errors.push(format!("Deck: '{k}' is not one of {}.", Deck::KEYS.join(", ")));
    }
    let deck_rows: Vec<&Dict> = v.entries(ruleset::KIND).iter().filter(|r| json::sget(r, "rules").iter().any(|x| x == ruleset::DECK)).collect();
    let mut basics: Vec<String> = Vec::new();
    for b in deck_rows.iter().flat_map(|r| json::sget(r, "basics")) {
        if !basics.contains(&b) {
            basics.push(b);
        }
    }
    // CLASS-BASICS: the unit's class basics are basics too (open, no slot)
    if !deck_rows.is_empty() {
        basics = rpgkit_rules::DeckRule { basics, ..Default::default() }.with_class(&v.typed, &json::s(e, "class_id")).basics;
    }
    let active = deck_list(deck, "active", &mut errors);
    repeats("Deck active", &active, &mut Vec::new(), &mut errors);
    let owned = owned_families(v, e);
    for fam in &active {
        if v.typed.tier_one(fam).is_none() {
            errors.push(format!("Deck active: unknown skill family '{fam}'."));
        } else if v.typed.is_basic_family(fam) || basics.contains(fam) || v.typed.is_system(fam) {
            errors.push(format!("Deck active: '{fam}' is a basic (open to every unit, no slot)."));
        } else if !owned.contains(fam) {
            errors.push(format!("Deck active: the unit does not own the skill family '{fam}'."));
        }
    }
    let mut slotted: Vec<(&str, Vec<String>)> = vec![("active", active.clone())];
    let mut seen_passives: Vec<String> = Vec::new();
    for slot in ["passive", "keystone"] {
        let ids = deck_list(deck, slot, &mut errors);
        repeats(&format!("Deck {slot}"), &ids, &mut seen_passives, &mut errors);
        for id in &ids {
            let p = v.get("passives", id);
            if p.is_empty() {
                errors.push(format!("Deck {slot}: unknown passive '{id}'."));
            } else if json::s(p, "slot") != slot {
                let has = json::s(p, "slot");
                let has = if has.is_empty() { "no slot (granted by its source, always on)".to_string() } else { format!("the slot {has}") };
                errors.push(format!("Deck {slot}: '{id}' has {has}, not the slot {slot}."));
            }
        }
        slotted.push((slot, ids));
    }
    for r in &deck_rows {
        let slots = json::dget(r, "deck_slots");
        for (slot, ids) in &slotted {
            let max = json::i(slots, slot, 0);
            if ids.len() as i64 > max {
                errors.push(format!("Deck: {} {slot} entries (the {} ruleset's deck slots allow {max}).", ids.len(), json::s_or(r, "name", &json::s(r, "id"))));
            }
        }
    }
    match deck.get("order") {
        None => {}
        Some(Value::Object(order)) => {
            let mut seen: Vec<String> = Vec::new();
            // A4.2-K1b (IR15): under field_intent the order reads the derived rows and may name `field`
            let field = super::ruleset::deck_plays_field(v);
            let names = super::ruleset::order_intents(field);
            for (intent, fams) in order {
                if !names.contains(&intent.as_str()) {
                    errors.push(format!("Deck order: '{intent}' is not an intent (one of {}).", names.join(", ")));
                    continue;
                }
                let Some(list) = fams.as_array().filter(|a| a.iter().all(Value::is_string)) else {
                    errors.push(format!("Deck order {intent} must be a list of skill families."));
                    continue;
                };
                let list: Vec<String> = list.iter().map(json::gstr).collect();
                // IR15: under field_intent a skill may stand in each of its rows' lists (once per list)
                if field {
                    seen.clear();
                }
                repeats("Deck order", &list, &mut seen, &mut errors);
                for fam in &list {
                    if !active.contains(fam) && !basics.contains(fam) {
                        errors.push(format!("Deck order {intent}: '{fam}' is neither an active of the deck nor a basic."));
                        continue;
                    }
                    let head = v.get("skills", fam);
                    let rows = if head.is_empty() { Vec::new() } else { super::ruleset::intent_rows(head, field) };
                    if !rows.contains(intent) {
                        let of = super::ruleset::rows_label(&rows);
                        errors.push(format!("Deck order {intent}: '{fam}' is of intent {}.", if of.is_empty() { "none (unknown family)" } else { &of }));
                    }
                }
            }
        }
        Some(_) => errors.push("Deck order must be an object {intent: [skill families]}.".into()),
    }
    let field = super::ruleset::deck_plays_field(v);
    let intent_of = |fam: &str| {
        let head = v.get("skills", fam);
        if head.is_empty() { Vec::new() } else { super::ruleset::intent_rows(head, field) }
    };
    follow_up_errors(deck.get("follow_ups"), &active, &basics, &intent_of, &mut errors);
    errors
}

/// The display form of an intent id ("assault" -> "Assault"; "" -> "no intent").
fn intent_label(id: &str) -> String {
    if id.is_empty() { "no intent".into() } else { capitalize(id) }
}

/// The rows as the follow-up checks name them ("Assault", "Field or Assault"; "no intent" for none).
fn rows_text(rows: &[String]) -> String {
    if rows.is_empty() { intent_label("") } else { rows.iter().map(|r| intent_label(r)).collect::<Vec<_>>().join(" or ") }
}

/// INPUT1 FL1, FL5: a deck's `follow_ups` maps an opener (a family of `active` or the basics whose intent is not
/// neutral) to families of `active` or the basics with the opener's intent, none twice in a list (the opener may stand
/// in its own list).
fn follow_up_errors(raw: Option<&Value>, active: &[String], basics: &[String], intent_of: &dyn Fn(&str) -> Vec<String>, errors: &mut Vec<String>) {
    let lists = match raw {
        None => return,
        Some(Value::Object(lists)) => lists,
        Some(_) => {
            errors.push("Deck follow_ups must be an object {opener family: [skill families]}.".into());
            return;
        }
    };
    let in_deck = |fam: &str| active.iter().chain(basics).any(|f| f == fam);
    for (key, fams) in lists {
        let Some(list) = fams.as_array().filter(|a| a.iter().all(Value::is_string)) else {
            errors.push(format!("Follow-ups for '{key}' must be a list of skill families."));
            continue;
        };
        // A4.2-K1b (IR15): an opener's rows (one or two); a follow-up shares one of them
        let of = intent_of(key);
        if !in_deck(key) || of.is_empty() || of.iter().any(|i| i == ruleset::NEUTRAL) {
            errors.push(format!("Follow-ups for '{key}': not an opener in the deck."));
            continue;
        }
        let mut seen: Vec<String> = Vec::new();
        for fam in list.iter().map(json::gstr) {
            if seen.contains(&fam) {
                errors.push(format!("Follow-ups after '{key}': '{fam}' twice."));
                continue;
            }
            seen.push(fam.clone());
            if !in_deck(&fam) {
                errors.push(format!("Follow-ups after '{key}': '{fam}' is not in the deck."));
                continue;
            }
            let has = intent_of(&fam);
            if !has.iter().any(|i| of.contains(i)) {
                errors.push(format!("Follow-ups after '{key}': '{fam}' is {}, not {}.", rows_text(&has), rows_text(&of)));
            }
        }
    }
}

/// ContentFactory.families_of: the families of skill ids, always-active ones left out, first-seen order.
fn families_of(v: &View, ids: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for id in ids {
        let fam = v.typed.family_of(id).to_string();
        if !v.typed.is_always_active(id) && !out.contains(&fam) {
            out.push(fam);
        }
    }
    out
}

/// ClassUnitValidator._skill_loadout_errors: an explicit unit loadout lists owned, non-basic families once each,
/// within the class and extra slot counts (ContentFactory.loadout_of splits a listed loadout by the class path).
fn skill_loadout_errors(v: &View, e: &Dict, a: &ClassDef) -> Vec<String> {
    let mut errors = Vec::new();
    let listed = json::sget(e, "loadout");
    if listed.is_empty() {
        return errors;
    }
    let level = json::i(e, "level", 1);
    let steps = a.path_steps();
    let path_all: Vec<String> = steps.iter().map(|(_, s)| s.clone()).collect();
    let mut owned_ids: Vec<String> = steps.iter().filter(|(l, _)| *l as i64 <= level).map(|(_, s)| s.clone()).collect();
    owned_ids.extend(json::sget(e, "extra_skills"));
    let owned = families_of(v, &owned_ids);
    let mut seen: Vec<&String> = Vec::new();
    for fam in &listed {
        if seen.contains(&fam) {
            errors.push(format!("Loadout: family '{fam}' is listed twice."));
        }
        seen.push(fam);
        if !v.typed.skills.iter().any(|s| s.family_id() == fam) {
            errors.push(format!("Loadout: unknown skill family '{fam}'."));
        } else if v.typed.is_basic_family(fam) {
            errors.push(format!("Loadout: '{fam}' is a basic family (always active, no slot)."));
        } else if !owned.contains(fam) {
            errors.push(format!("Loadout: the unit does not own the skill family '{fam}'."));
        }
    }
    let path_families = families_of(v, &path_all);
    let class_count = listed.iter().filter(|f| path_families.contains(f)).count();
    let extra_count = listed.len() - class_count;
    if class_count > MAX_CLASS_SLOTS {
        errors.push(format!("Loadout: {class_count} class families (max {MAX_CLASS_SLOTS})."));
    }
    if extra_count > MAX_EXTRA_SLOTS {
        errors.push(format!("Loadout: {extra_count} extra families (max {MAX_EXTRA_SLOTS})."));
    }
    errors
}

/// ClassUnitValidator.class_warnings: the gear budget, the path (more families than class slots, none at level 1,
/// tiers past the level cap, weapon gates the class cannot reach, a class basic also in the path) and the role
/// mismatches.
/// CLASS-BASICS (FB-123): a class basic of class row `e` is open without a slot, so listing it in skill path `path` (the
/// row's own, or L3 (FB-138) a world overlay's: `hooks::classes_warnings`) also spends a path slot.
pub fn basics_in_path_warnings(e: &Dict, path: &Dict) -> Vec<String> {
    json::sget(e, "basics")
        .iter()
        .filter(|id| path.contains_key(id.as_str()))
        .map(|id| format!("Class basic '{id}' is also in the skill path, where it takes a path slot (an auto loadout fills slots in path order)."))
        .collect()
}

pub fn class_warnings(v: &View, e: &Dict) -> Vec<String> {
    let class = ClassDef::from_obj(e);
    let mut out = rpgkit_rules::budget::check_class(&class, &v.typed.items);
    let path = json::dget(e, "skill_path");
    let families: Vec<String> = path.keys().filter(|k| !v.typed.is_basic_family(k)).cloned().collect();
    if families.len() > MAX_CLASS_SLOTS {
        out.push(format!("The path lists {} families; only the first {MAX_CLASS_SLOTS} (by unlock level, then position) are active by default.", families.len()));
    }
    out.extend(basics_in_path_warnings(e, path));
    if !path.is_empty() && !path.values().any(|l| json::is_int(l) && json::to_int(l) == 1) {
        out.push("No path family unlocks at level 1: a level 1 unit of this class has no class skill in its loadout.".into());
    }
    let prof = Proficiency::from_class(&class);
    for fam in &families {
        let base_level = json::to_int_or(&path[fam], 1);
        let mut tiers: Vec<&SkillDef> = v.typed.skills.iter().filter(|s| s.family_id() == fam).collect();
        tiers.sort_by_key(|s| s.tier);
        for row in tiers {
            let opens = base_level + row.tier_offset() as i64;
            if row.tier > 1 && opens > MAX_LEVEL as i64 {
                out.push(format!("{} unlocks at level {opens}, beyond the level cap {MAX_LEVEL} (capped).", row.name));
            }
        }
        if let Some(head) = v.typed.tier_one(fam).filter(|h| !h.weapon_types.is_empty() && !gate_reachable(&h.weapon_types, &prof)) {
            let unarmed = if head.weapon_types.iter().any(|t| t == UNARMED_TOKEN) { " ('unarmed' needs fist)" } else { "" };
            out.push(format!("{} needs {}, but the class trains none of them{unarmed}.", head.name, head.weapon_types.join("/")));
        }
    }
    out.extend(role_warnings(v, &json::s(e, "ai_role"), path_has_support(v, &families), &json::s(e, "main_hand"), false));
    out
}

/// ClassUnitValidator.unit_warnings: the role mismatches for the unit's own role or its class's (derived units only).
pub fn unit_warnings(v: &View, e: &Dict) -> Vec<String> {
    let class_id = json::s(e, "class_id");
    let Some(arch) = v.typed.classes.get(&class_id).filter(|_| !class_id.is_empty() && json::s_or(e, "stat_mode", "derived") != "fixed") else {
        return Vec::new();
    };
    let mut role = json::s(e, "ai_role");
    if role.is_empty() {
        role = arch.ai_role.clone();
    }
    let mut families: Vec<String> = arch.path_steps().into_iter().map(|(_, s)| s).collect();
    families.extend(json::sget(e, "extra_skills").iter().map(|x| v.typed.family_of(x).to_string()));
    let mut main = json::s(e, "main_hand");
    if main.is_empty() {
        main = arch.loadout.get("main_hand").cloned().unwrap_or_default();
    } else if main == NO_ITEM {
        main.clear();
    }
    role_warnings(v, &role, path_has_support(v, &families), &main, json::b(e, "reckless", false))
}

fn role_warnings(v: &View, role: &str, has_support_skill: bool, main_hand: &str, reckless: bool) -> Vec<String> {
    let mut out = Vec::new();
    if role == "SUPPORT" && !has_support_skill {
        out.push("Role SUPPORT without a SUPPORT skill: the unit has nothing to heal or buff with.".into());
    }
    if role == "SNIPER" {
        let item = v.get("items", main_hand);
        let reach = if item.is_empty() { 1 } else { json::i(item, "base_range", 1) };
        if reach < 2 {
            out.push(format!("Role SNIPER with a melee main hand (range {reach}): a sniper keeps its distance and needs range 2+."));
        }
    }
    if role == "SUPPORT" && reckless {
        out.push("A reckless SUPPORT never holds back to heal: reckless charges the nearest foe.".into());
    }
    out
}

fn path_has_support(v: &View, families: &[String]) -> bool {
    families.iter().any(|f| v.typed.tier_one(f).is_some_and(|h| h.kind == "SUPPORT"))
}

/// SkillDef.UNARMED_TOKEN: the weapon gate of bare hands, trained as the fist proficiency.
const UNARMED_TOKEN: &str = "unarmed";

/// A weapon gate is reachable when the class trains one of its gear types.
fn gate_reachable(tokens: &[String], prof: &Proficiency) -> bool {
    tokens.iter().any(|t| prof.can_use(if t == UNARMED_TOKEN { "fist" } else { t }))
}

/// ClassUnitValidator._unit_requires_errors: a skill in `extra_skills` or `loadout` needs every family it requires on
/// the class path (any level) or among the extra skills.
fn requires_errors(v: &View, e: &Dict, arch: Option<&ClassDef>) -> Vec<String> {
    let mut errors = Vec::new();
    let extras = json::sget(e, "extra_skills");
    let path: Vec<String> = arch.map(|a| a.path_steps().into_iter().map(|(_, s)| s).collect()).unwrap_or_default();
    let owned: Vec<String> = path.iter().chain(&extras).map(|id| super::skill::head_of(v.get("skills", id))).collect();
    let mut checked = extras.clone();
    checked.extend(json::sget(e, "loadout"));
    for id in &checked {
        for required in json::sget(v.get("skills", id), "requires") {
            if !owned.contains(&required) {
                errors.push(format!("Unit skill '{id}' needs '{required}' (on the class path or in the extra skills)."));
            }
        }
    }
    errors
}
