//! Items and item families (ItemValidator, ItemText.validate / warnings, ContentCatalog._prepare_item with
//! ItemModifiers.collapse / stored, ItemFamilies). An item is checked as the resolved row its family gives it; a flat
//! row (a get_entry copy, a legacy row) collapses back to the stored shape first, as the Forge's save does.

use super::{capitalize, simple, View};
use crate::source::{category, content, default_hand, hands_of, item_type_of, legacy_tier, render_item, resolve_item, tier_desc};
use aegis_core::fmt::fixed;
use aegis_core::json::{self, Dict};
use rpgkit_data::{ItemDef, STATS};
use rpgkit_rules::{budget, stats::MAX_LEVEL};
use serde_json::Value;

const EQUIP_TYPES: [&str; 4] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"];
const MODIFIABLE_TYPES: [&str; 4] = EQUIP_TYPES;
const GEAR_TYPES: [&str; 3] = ["WEAPON", "SHIELD", "ARMOR"];
const HAND_TYPES: [&str; 2] = ["WEAPON", "SHIELD"];
const SLOT_TYPES: [&str; 2] = ["ARMOR", "ACCESSORY"];
const HANDS: [&str; 3] = ["main_1h", "main_2h", "off"];
/// ItemFamilies.OFF_CATEGORIES: the off-hand families the family check allows (U3.2-S2: still a closed list; its
/// message prints it in this order, and the loader derives an off-hand family's category from its id).
pub const OFF_CATEGORIES: [&str; 3] = ["shield", "orb", "tome"];
const ARMOR_SLOTS: [&str; 2] = ["HEAD", "BODY"];
const ACCESSORY_SLOTS: [&str; 2] = ["ACCESSORY_1", "ACCESSORY_2"];
const SCALING_STATS: [&str; 3] = ["STR", "DEX", "INT"];
const NO_HAND_KEY: &str = "none";
const BONUS_BAND: &str = "bonus_value";
/// ItemFamilies.ENGINE_RULES; RULESET1-S adds `reach` (RULESET1 K1: a melee family whose wielder reaches 2 under the
/// timeline's reach rule).
const ENGINE_RULES: [&str; 5] = ["focus", "always_ranged", "shield", "spell_source", "reach"];
/// ItemFamilies.NUMBER_FIELDS with their flat keys and schema ranges.
const NUMBERS: [(&str, &str, i64, i64); 4] = [("power", "base_power", 0, 200), ("range", "base_range", 1, 6), ("burden", "burden", 0, 10), ("mitigation", "mitigation", 0, 100)];
/// ItemModifiers.KINDS: (kind, params, unique param, min, max).
const KINDS: [(&str, &[&str], &str, i64, i64); 4] = [("adjust", &["field", "delta"], "field", -100, 100), ("stat_bonus", &["stat", "value"], "stat", 1, 8), ("requirement", &["stat", "value"], "stat", 1, 30), ("effect", &["effect"], "effect", 0, 0)];
/// CAST1-S2: the top of a gear stat bonus naming a derived stat (the bottom stays 1). The derived_stats row's own
/// min/max cap the stat's value, not a bonus, so the row has no bound to read; 50 fits Max Poise +30 and +12.
const DERIVED_BONUS_MAX: i64 = 50;
const DERIVED_KEYS: [&str; 19] = [
    "item_type", "weapon_type", "two_handed", "off_hand_category", "base_power", "base_range", "burden", "mitigation", "required_stats", "stat_bonuses", "effects", "threshold_tiers", rpgkit_data::ITEM_FAMILY_RULES,
    rpgkit_data::ITEM_FAMILY_LEVEL2, rpgkit_data::ITEM_FAMILY_LEVEL3, rpgkit_data::ITEM_FAMILY_REACH_MAX, rpgkit_data::mastery::ITEM_FAMILY_TECHNIQUES,
    rpgkit_data::mastery::ITEM_FAMILY_KEYSTONE, rpgkit_data::ITEM_FAMILY_DURABILITY,
];
const FLAT_MARKERS: [&str; 8] = ["base_power", "base_range", "burden", "mitigation", "required_stats", "stat_bonuses", "effects", "threshold_tiers"];
const ENGINE_KEYS: [&str; 14] = ["item_type", "slot", "weapon_type", "base_power", "base_range", "two_handed", "off_hand_category", "scaling_stat", "burden", "mitigation", "required_stats", "stat_bonuses", "effects", "threshold_tiers"];
const WHEN_STAT: &str = "stat_at_least";
const WHEN_PROFICIENCY: &str = "proficiency_at_least";
const WHEN_LEVEL: &str = "unit_level_at_least";
const WHEN_TYPES: [&str; 3] = [WHEN_STAT, WHEN_PROFICIENCY, WHEN_LEVEL];
const BONUS_KEYS: [&str; 4] = ["power", "mitigation", "stat_bonuses", "effects"];
const TIER_KEYS: [&str; 4] = ["label", "when", "bonus", "desc"];
const TIER_LABEL_MAX: usize = 24;
const TIER_DESC_MAX: usize = 120;
const TIER_STAT_MIN: i64 = 1;
const TIER_STAT_MAX: i64 = 40;
const TIER_MAX_STATS: usize = 3;
const TIER_MAX_EFFECTS: usize = 2;
const PROF_MAX_LEVEL: i64 = 3;
const TEXT_MAX: usize = 240;

fn chars(s: &str) -> usize {
    s.chars().count()
}

fn kind_names() -> String {
    KINDS.iter().map(|k| k.0).collect::<Vec<_>>().join(", ")
}

fn number_fields_for(item_type: &str) -> Vec<&'static str> {
    match item_type {
        "WEAPON" => NUMBERS.iter().map(|n| n.0).collect(),
        "SHIELD" | "ARMOR" | "ACCESSORY" => vec!["burden", "mitigation"],
        _ => Vec::new(),
    }
}

fn number(field: &str) -> (&'static str, &'static str, i64, i64) {
    *NUMBERS.iter().find(|n| n.0 == field).expect("a number field")
}

/// A canonical copy for comparisons and tags (ItemModifiers.sorted_value): whole floats as ints; key order ignored.
fn canon(v: &Value) -> Value {
    json::normalize(v)
}

/// ItemModifiers.sorted_value as text (tags and criterion keys).
fn tag(v: &Value) -> String {
    serde_json::to_string(&json::sorted(&canon(v))).unwrap_or_default()
}

/// ItemModifiers.same_value: numbers approximately, everything else by value (key order, 1 vs 1.0 ignored).
fn same_value(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => {
            let tolerance = (x.abs() * 0.00001).max(0.00001);
            x == y || (x - y).abs() < tolerance
        }
        _ => canon(a) == canon(b),
    }
}

// ---- ItemFamilies ----

fn hand_key(family: &Dict, hand: &str) -> String {
    let hands = hands_of(family);
    if hands.is_empty() {
        NO_HAND_KEY.into()
    } else if hands.iter().any(|h| h == hand) {
        hand.into()
    } else {
        default_hand(family)
    }
}

/// ItemFamilies.base_of: the four base numbers of `hand` (a missing one is the schema minimum).
fn base_of(family: &Dict, hand: &str) -> Vec<(&'static str, i64)> {
    let row = json::dget(json::dget(family, "base"), &hand_key(family, hand));
    NUMBERS.iter().map(|(f, _, lo, _)| (*f, row.get(*f).map_or(*lo, |v| json::to_int_or(v, *lo)))).collect()
}

fn base_value(base: &[(&str, i64)], field: &str) -> i64 {
    base.iter().find(|(f, _)| *f == field).map_or(0, |(_, v)| *v)
}

fn band_of<'a>(family: &'a Dict, hand: &str, field: &str) -> &'a Dict {
    json::dget(json::dget(json::dget(family, "bands"), &hand_key(family, hand)), field)
}

/// ItemFamilies.family_id_for_row: a flat row's gear type, else its item type's catch-all family.
fn family_id_for_row(row: &Dict) -> String {
    let gear = json::s(row, "weapon_type");
    if !gear.is_empty() {
        return gear;
    }
    match json::s(row, "item_type").as_str() {
        "ACCESSORY" => "accessory",
        "CONSUMABLE" => "consumable",
        "MATERIAL" => "material",
        "QUEST" => "quest",
        _ => "",
    }
    .into()
}

// ---- ItemModifiers: stored <-> flat ----

fn modifiers_of(e: &Dict) -> Vec<Dict> {
    json::aget(e, "modifiers").iter().map(|m| json::dict(m).clone()).collect()
}

fn tiers_of(e: &Dict) -> Vec<Dict> {
    json::aget(e, "tiers").iter().map(|t| json::dict(t).clone()).collect()
}

fn hand_in(e: &Dict, family: &Dict) -> String {
    if hands_of(family).is_empty() {
        return String::new();
    }
    let hand = json::s(e, "hand");
    if hand.is_empty() {
        default_hand(family)
    } else {
        hand
    }
}

/// ItemModifiers.raw_numbers: the family base of the item's hand plus the adjust deltas, before any clamping.
fn raw_numbers(e: &Dict, family: &Dict) -> Vec<(&'static str, i64)> {
    let mut numbers = base_of(family, &hand_in(e, family));
    for m in modifiers_of(e).iter().filter(|m| json::s(m, "kind") == "adjust") {
        let field = json::s(m, "field");
        if let Some(slot) = numbers.iter_mut().find(|(f, _)| *f == field) {
            slot.1 += json::i(m, "delta", 0);
        }
    }
    numbers
}

fn is_stored_shape(e: &Dict) -> bool {
    !json::s(e, "family").is_empty() && !FLAT_MARKERS.iter().any(|k| e.contains_key(*k))
}

fn engine_equal(a: &Dict, b: &Dict) -> bool {
    ENGINE_KEYS.iter().all(|k| same_value(json::get(a, k), json::get(b, k)))
}

/// ItemModifiers.flat_hand: the hand a flat row occupies.
fn flat_hand(row: &Dict) -> &'static str {
    match json::s_or(row, "item_type", "WEAPON").as_str() {
        "SHIELD" => "off",
        "WEAPON" if json::b(row, "two_handed", false) => "main_2h",
        "WEAPON" if !json::s(row, "off_hand_category").is_empty() => "off",
        "WEAPON" => "main_1h",
        _ => "",
    }
}

/// ItemModifiers.label_of_desc: the text before ":".
fn label_of_desc(desc: &str) -> String {
    desc.split(':').next().unwrap_or("").trim().to_string()
}

/// ItemModifiers.tiers_from_legacy.
fn tiers_from_legacy(legacy: &[Value]) -> Vec<Value> {
    legacy
        .iter()
        .map(|raw| {
            let t = json::dict(raw);
            let mut when = json::dget(t, "when").clone();
            if when.is_empty() {
                when = json::obj([("type", Value::from(WHEN_STAT)), ("stat", Value::from(json::s_or(t, "stat", "STR"))), ("value", json::int(json::i(t, "threshold", 0)))]);
            }
            let mut bonus = json::dget(t, "bonus").clone();
            if bonus.is_empty() {
                bonus = json::obj([("power", json::int(json::i(t, "bonus_power", 0)))]);
            }
            let desc = json::s(t, "desc");
            let mut tier = json::obj([("label", Value::from(label_of_desc(&desc))), ("when", canon(&Value::Object(when))), ("bonus", canon(&Value::Object(bonus))), ("desc", Value::from(""))]);
            if tier_desc(&tier) != desc {
                tier.insert("desc".into(), Value::from(desc));
            }
            Value::Object(tier)
        })
        .collect()
}

/// ItemModifiers.tiers_for_row: the row's own tiers while they still resolve to its threshold tiers, else converted.
fn tiers_for_row(row: &Dict) -> Vec<Value> {
    let own = tiers_of(row);
    let legacy = json::aget(row, "threshold_tiers");
    let resolved = Value::Array(own.iter().map(|t| Value::Object(legacy_tier(t))).collect());
    if !own.is_empty() && same_value(&resolved, &Value::Array(legacy.to_vec())) {
        return json::arr(&canon(json::get(row, "tiers"))).to_vec();
    }
    tiers_from_legacy(legacy)
}

/// ItemModifiers.collapse: the stored entry of a flat row against `family`.
fn collapse(row: &Dict, family: &Dict) -> Dict {
    let mut out: Dict = row.iter().filter(|(k, _)| !DERIVED_KEYS.contains(&k.as_str())).map(|(k, v)| (k.clone(), canon(v))).collect();
    let t = item_type_of(family);
    out.insert("family".into(), Value::from(json::s(family, "id")));
    let mut hand = String::new();
    if !hands_of(family).is_empty() {
        hand = flat_hand(row).to_string();
        if category(family) == "main" && hand == "off" {
            hand = "main_1h".into();
        }
        if hand.is_empty() {
            hand = default_hand(family);
        }
        out.shift_remove("slot");
    }
    out.insert("hand".into(), Value::from(hand.clone()));
    if t != "WEAPON" {
        out.shift_remove("scaling_stat");
    }
    let mut mods: Vec<Value> = Vec::new();
    let base = base_of(family, &hand);
    for field in number_fields_for(&t) {
        let b = base_value(&base, field);
        let have = row.get(number(field).1).map_or(b, |v| json::to_int_or(v, b));
        if have != b {
            mods.push(Value::Object(json::obj([("kind", Value::from("adjust")), ("field", Value::from(field)), ("delta", json::int(have - b))])));
        }
    }
    if MODIFIABLE_TYPES.contains(&t.as_str()) {
        for (kind, key) in [("requirement", "required_stats"), ("stat_bonus", "stat_bonuses")] {
            for (stat, v) in json::dget(row, key) {
                mods.push(Value::Object(json::obj([("kind", Value::from(kind)), ("stat", Value::from(stat.clone())), ("value", json::int(json::to_int(v)))])));
            }
        }
        for fx in json::aget(row, "effects") {
            mods.push(Value::Object(json::obj([("kind", Value::from("effect")), ("effect", canon(&Value::Object(json::dict(fx).clone())))])));
        }
    }
    out.insert("modifiers".into(), Value::Array(mods));
    out.insert("tiers".into(), Value::Array(tiers_for_row(row)));
    if json::s(row, "damage_type") == json::s(family, "damage_type") {
        out.insert("damage_type".into(), Value::from(""));
    }
    out
}

/// ItemModifiers.tag_of: kind + its unique param.
fn tag_of(m: &Dict) -> String {
    let kind = json::s(m, "kind");
    let unique = KINDS.iter().find(|k| k.0 == kind).map_or("", |k| k.2);
    format!("{kind}:{}", tag(json::get(m, unique)))
}

/// ItemModifiers.stable_order: `fresh` in the order of `previous` (matched by tag), new tags after.
fn stable_order(fresh: Vec<Dict>, previous: &[Dict]) -> Vec<Value> {
    let mut used = vec![false; fresh.len()];
    let mut out = Vec::new();
    for old in previous {
        let t = tag_of(old);
        if let Some(i) = (0..fresh.len()).find(|&i| !used[i] && tag_of(&fresh[i]) == t) {
            used[i] = true;
            out.push(Value::Object(fresh[i].clone()));
        }
    }
    out.extend(fresh.into_iter().enumerate().filter(|(i, _)| !used[*i]).map(|(_, m)| Value::Object(m)));
    out
}

/// ItemModifiers.stored: a resolved row without its derived keys.
fn stored(row: &Dict) -> Dict {
    let has_hand = !json::s(row, "hand").is_empty();
    row.iter().filter(|(k, _)| !(DERIVED_KEYS.contains(&k.as_str()) || (k.as_str() == "slot" && has_hand) || (k.as_str() == "hand" && !has_hand))).map(|(k, v)| (k.clone(), canon(v))).collect()
}

fn family_matches(family: &Dict, flat: &Dict) -> bool {
    let t = item_type_of(family);
    let token = if GEAR_TYPES.contains(&t.as_str()) { json::s(family, "id") } else { String::new() };
    json::s(flat, "item_type") == t && json::s(flat, "weapon_type") == token
}

pub fn inherit(v: &View) -> bool {
    !v.entries("item_families").is_empty()
}

/// ContentCatalog.family_of: the family row an item names ({} when none).
pub fn family_of<'a>(v: &'a View, item: &Dict) -> &'a Dict {
    v.get("item_families", &json::s(item, "family"))
}

/// ContentCatalog._prepare_item: the resolved row an item save checks. A stored entry resolves as it is; a flat row
/// collapses against its family when its engine keys changed (keeping the stored modifier order), else keeps its own
/// lists.
pub fn prepare(v: &View, raw: &Dict, old: &Dict) -> Dict {
    if !inherit(v) {
        return content("items", raw);
    }
    let mut source = raw.clone();
    let mut family_id = json::s(raw, "family");
    if !is_stored_shape(raw) {
        let flat = content("items", raw);
        if old.is_empty() || json::s(old, "family").is_empty() || !engine_equal(&flat, old) {
            let own = v.get("item_families", &family_id);
            if own.is_empty() || !family_matches(own, &flat) {
                family_id = family_id_for_row(&flat);
            }
            let target = v.get("item_families", &family_id);
            source = if target.is_empty() { flat } else { collapse(&flat, target) };
            if !old.is_empty() && source.contains_key("modifiers") {
                let ordered = stable_order(modifiers_of(&source), &modifiers_of(old));
                source.insert("modifiers".into(), Value::Array(ordered));
            }
        } else {
            if family_id.is_empty() {
                family_id = json::s(old, "family");
            }
            source = stored(&flat);
        }
    }
    let family = v.get("item_families", &family_id);
    if family.is_empty() {
        let mut unresolved = content("items", &source);
        unresolved.insert("family".into(), Value::from(family_id));
        return unresolved;
    }
    content("items", &resolve_item(&source, family))
}

// ---- ItemValidator: items ----

/// ItemValidator.item_errors of a resolved row (`family` its family row, {} when unknown; `inherit` = families exist).
pub fn item_errors(v: &View, e: &Dict, family: &Dict, inherit: bool) -> Vec<String> {
    let mut out = Vec::new();
    out.extend(super::assets::icon_errors(v, e, family));
    // PK2-S2: a body piece's own fields (flat or resolved, the type its family gives)
    if json::s(e, "item_type") == rpgkit_data::GRAFT {
        out.extend(graft_problems(v, e));
    }
    // PK2-S4: a mastery book's family; a weapon's own (masterwork) technique
    if json::s(e, "item_type") == rpgkit_data::MASTERY_BOOK {
        out.extend(book_problems(v, e));
    }
    if let Some(t) = e.get(rpgkit_data::mastery::TECHNIQUE_KEY).filter(|t| t.as_object().is_none_or(|o| !o.is_empty())) {
        out.extend(technique_entry_errors(v, t, "Masterwork technique"));
    }
    // PK2-S5a: a weapon's own full durability
    out.extend(durability_problems(e));
    // PK2-S3: a theme (the salvage it gives) is an id
    let theme = json::s(e, "theme");
    if !theme.is_empty() && !json::id_ok(&theme) {
        out.push(format!("Theme '{theme}' must be an id (lowercase a-z, 0-9 and _)."));
    }
    if !inherit {
        out.extend(legacy_errors(v, e));
        return out;
    }
    if json::s(e, "family").is_empty() {
        out.push("Family is required (one of the item families).".into());
        return out;
    }
    if family.is_empty() {
        return out;
    }
    let t = item_type_of(family);
    let family_id = json::s(family, "id");
    let hands = hands_of(family);
    let hand = json::s(e, "hand");
    if !hands.is_empty() && !hands.contains(&hand) {
        out.push(format!("Hand '{hand}' is not allowed by family '{family_id}' (allowed: {}).", hands.join(", ")));
    }
    if t == "WEAPON" {
        let scaling = json::sget(family, "scaling_stats");
        let have = json::s(e, "scaling_stat");
        if !scaling.contains(&have) {
            out.push(format!("Scales with '{have}' is not in family '{family_id}' (allowed: {}).", scaling.join(", ")));
        }
    }
    if SLOT_TYPES.contains(&t.as_str()) {
        let slots = json::sget(family, "slots");
        let slot = json::s(e, "slot");
        if !slots.contains(&slot) {
            out.push(format!("Slot '{slot}' is not one of family '{family_id}' slots ({}).", slots.join(", ")));
        }
    }
    if GEAR_TYPES.contains(&t.as_str()) && json::s(e, "weapon_type") != family_id {
        out.push(format!("Gear type '{}' must equal the family id '{family_id}'.", json::s(e, "weapon_type")));
    }
    out.extend(modifier_problems(v, e, family));
    out.extend(tier_problems(v, e, family));
    out.extend(text_problems(&json::s(e, "description"), e));
    if t == "CONSUMABLE" {
        out.extend(consumable_problems(e));
        // HOOKS1 GH2: the hazard a consumable leaves (no SKILL shape: an item has no area of its own)
        if json::get(e, "hazard").as_object().is_none_or(|h| !h.is_empty()) {
            out.extend(super::hooks::hazard_place_errors(json::get(e, "hazard"), &super::hooks::Ctx::of(v), false));
        }
    }
    out
}

/// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3): a `GRAFT` item's keystone mark (a top-tier class piece only), class (a
/// class piece only) and `body_effects` (the passive grammar without a member grant); its tier (1-3), body role and a
/// known class are the schema's (normalised, enum- and ref-checked with every field).
fn graft_problems(v: &View, e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let role = json::s(e, "body_role");
    let class_piece = role == rpgkit_data::BODY_ROLE_CLASS;
    let keystone = json::b(e, "keystone", false);
    if keystone && !class_piece {
        out.push(format!("A keystone graft is a class piece (body role {}).", rpgkit_data::BODY_ROLE_CLASS));
    }
    // PK2 3: the keystone is a top-tier graft (so the 3-graft gate applies to it)
    let top = rpgkit_data::GRAFT_TIERS.1;
    if keystone && json::i(e, "tier", 0) != i64::from(top) {
        out.push(format!("A keystone graft is a tier {top} graft (it needs 3 grafts first, as every tier {top} graft)."));
    }
    // a known class is the schema's ref check
    if !json::s(e, "class_id").is_empty() && !class_piece {
        out.push(format!("Only a class piece (body role {}) names a class.", rpgkit_data::BODY_ROLE_CLASS));
    }
    let effects = json::get(e, rpgkit_data::BODY_EFFECTS_KEY);
    for message in simple::effect_errors(effects, &has_skill(v), &v.derived_ids(), true, false, &super::hooks::Ctx::of(v)) {
        out.push(format!("Body effects: {message}"));
    }
    for (i, fx) in effects.as_array().map(Vec::as_slice).unwrap_or_default().iter().enumerate() {
        let t = json::s(json::dict(fx), "type");
        if rpgkit_data::MEMBER_GRANTS.contains(&t.as_str()) {
            out.push(format!("Body effects: Effect {}: '{t}' is a passive's member grant; a graft can't carry it.", i + 1));
        }
    }
    out
}

/// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): a `MASTERY_BOOK` item names the main-hand weapon family it trains (its XP
/// range is the schema's).
fn book_problems(v: &View, e: &Dict) -> Vec<String> {
    let family = json::s(e, rpgkit_data::mastery::BOOK_FAMILY_KEY);
    if family.is_empty() {
        return vec!["A mastery book names the weapon family it trains (mastery_family).".into()];
    }
    let row = v.get("item_families", &family);
    if !v.entries("item_families").is_empty() && !rpgkit_data::mastery::main_hand_family(row) {
        return vec![format!("Mastery family '{family}' is not a main-hand weapon family (only those gain mastery).")];
    }
    Vec::new()
}

/// PK2-S4: one technique `{skill, grip}` (`label` names it in the messages): an object with a known skill and a grip tag
/// of `mastery::GRIPS`, no other key.
fn technique_entry_errors(v: &View, raw: &Value, label: &str) -> Vec<String> {
    use rpgkit_data::mastery::GRIPS;
    let Some(t) = raw.as_object() else { return vec![format!("{label} must be an object {{\"skill\", \"grip\"}}.")] };
    let mut out: Vec<String> = t.keys().filter(|k| !["skill", "grip"].contains(&k.as_str())).map(|k| format!("{label}: unknown key '{k}' (skill, grip).")).collect();
    let skill = json::s(t, "skill");
    if skill.is_empty() {
        out.push(format!("{label} names a skill."));
    } else if !v.has("skills", &skill) {
        out.push(format!("{label}: unknown skill '{skill}'."));
    }
    let grip = json::s(t, "grip");
    if !GRIPS.contains(&grip.as_str()) {
        out.push(format!("{label}: grip '{grip}' is not one of \"\", 1h, 2h."));
    }
    out
}

/// PK2-S5a (`docs/sessions/PK2_GRAFTS.md` 6-7): a full `durability` (an item family's for its weapons, a weapon item's
/// own) is a whole number 0 to `DURABILITY_MAX`, and only on a WEAPON row (only weapons become instances).
fn durability_problems(row: &Dict) -> Vec<String> {
    let Some(raw) = row.get(rpgkit_data::ITEM_DURABILITY) else { return Vec::new() };
    let n = json::normalize(raw);
    let mut out = Vec::new();
    if !json::is_int(&n) || !(0..=rpgkit_data::DURABILITY_MAX).contains(&json::to_int(&n)) {
        out.push(format!("Durability must be a whole number 0-{}.", rpgkit_data::DURABILITY_MAX));
    } else if json::to_int(&n) > 0 && json::s(row, "item_type") != "WEAPON" {
        out.push("Durability needs a WEAPON (only weapons become instances).".into());
    }
    out
}

/// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): a family's `techniques` (0-3 techniques, each skill once) and `keystone`
/// (a known skill, the schema's ref check) only on a main-hand WEAPON family (the families that gain mastery).
fn technique_problems(v: &View, f: &Dict) -> Vec<String> {
    use rpgkit_data::mastery as m;
    let mut out = Vec::new();
    let raw = json::get(f, m::TECHNIQUES_KEY);
    let list: &[Value] = match raw {
        Value::Array(a) => a,
        Value::Null => &[],
        _ => {
            out.push("Techniques must be a list of {\"skill\", \"grip\"}.".into());
            &[]
        }
    };
    let keystone = json::s(f, m::KEYSTONE_KEY);
    if (!list.is_empty() || !keystone.is_empty()) && !m::main_hand_family(f) {
        out.push("Techniques and a keystone need a main-hand WEAPON family (only those gain mastery).".into());
    }
    if list.len() > m::MASTERY_MAX as usize {
        out.push(format!("A family lists at most {} techniques (one per mastery level); it lists {}.", m::MASTERY_MAX, list.len()));
    }
    let mut seen: Vec<String> = Vec::new();
    for (i, t) in list.iter().enumerate() {
        out.extend(technique_entry_errors(v, t, &format!("Technique {}", i + 1)));
        let skill = json::s(json::dict(t), "skill");
        if !skill.is_empty() && seen.contains(&skill) {
            out.push(format!("Technique {}: '{skill}' is listed twice.", i + 1));
        }
        seen.push(skill);
    }
    out
}

fn has_skill<'a>(v: &'a View) -> impl Fn(&str) -> bool + 'a {
    move |id: &str| v.has("skills", id)
}

fn modifier_problems(v: &View, e: &Dict, family: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let t = item_type_of(family);
    let family_id = json::s(family, "id");
    let mods = modifiers_of(e);
    for (i, raw) in json::aget(e, "modifiers").iter().enumerate() {
        if !raw.is_object() {
            out.push(format!("Modifier {} is not an object.", i + 1));
        }
    }
    if mods.is_empty() {
        return out;
    }
    if !MODIFIABLE_TYPES.contains(&t.as_str()) {
        out.push(format!("{t} items take no modifiers (family '{family_id}')."));
        return out;
    }
    let cap = json::i(family, "max_modifiers", 0);
    if mods.len() as i64 > cap {
        out.push(format!("At most {cap} modifiers (family '{family_id}'); this item has {}.", mods.len()));
    }
    let allowed = json::sget(family, "modifier_kinds");
    let adjustable = json::sget(family, "adjustable");
    let numbers = number_fields_for(&t);
    let mut seen: Vec<String> = Vec::new();
    let skill_ok = has_skill(v);
    for (i, m) in mods.iter().enumerate() {
        let n = i + 1;
        let kind = json::s(m, "kind");
        let Some(&(_, params, unique, lo, hi)) = KINDS.iter().find(|k| k.0 == kind) else {
            out.push(format!("Modifier {n}: unknown kind '{kind}' (one of {}).", kind_names()));
            continue;
        };
        if !allowed.contains(&kind) {
            out.push(format!("Modifier {n}: kind '{kind}' is not allowed by family '{family_id}' (allowed: {}).", allowed.join(", ")));
        }
        for key in m.keys() {
            if key != "kind" && !params.contains(&key.as_str()) {
                out.push(format!("Modifier {n} ({kind}): unknown key '{key}' (keys: {}).", params.join(", ")));
            }
        }
        let mut unique_tag = kind.clone();
        // CAST1-S2: a stat bonus on a derived stat row (max_poise +30) takes the derived range, not the base 1-8
        let stat_name = json::s(m, "stat");
        let hi = if kind == "stat_bonus" && !STATS.contains(&stat_name.as_str()) && v.has(super::derived::KIND, &stat_name) { DERIVED_BONUS_MAX } else { hi };
        let int_problems = |out: &mut Vec<String>, value: &Value, key: &str, nonzero: bool| {
            let whole = value.as_f64().is_some_and(|f| f == f.floor());
            if !whole {
                out.push(format!("Modifier {n}: {key} must be a whole number."));
                return;
            }
            let x = json::to_int(value);
            if x < lo || x > hi {
                out.push(format!("Modifier {n}: {key} {x} is outside {lo} to {hi}."));
            }
            if nonzero && x == 0 {
                out.push(format!("Modifier {n}: a {key} of 0 changes nothing; remove it."));
            }
        };
        match kind.as_str() {
            "adjust" => {
                let field = json::s(m, "field");
                unique_tag = format!("{unique_tag}:{field}");
                if !numbers.contains(&field.as_str()) {
                    out.push(format!("Modifier {n}: '{field}' is not a number of {t} items ({}).", numbers.join(", ")));
                } else if !adjustable.contains(&field) {
                    out.push(format!("Modifier {n}: family '{family_id}' does not allow adjusting '{field}' (adjustable: {}).", adjustable.join(", ")));
                }
                int_problems(&mut out, json::get(m, "delta"), "delta", true);
            }
            "stat_bonus" | "requirement" => {
                let stat = json::s(m, "stat");
                unique_tag = format!("{unique_tag}:{stat}");
                // DERIVE1 4.1: a stat bonus may name a derived stat; requirements stay on the base stats
                if kind == "stat_bonus" && !v.is_stat(&stat) {
                    out.push(format!("Modifier {n}: unknown stat '{stat}' (one of {}).", v.stat_names()));
                } else if kind == "requirement" && !STATS.contains(&stat.as_str()) {
                    out.push(format!("Modifier {n}: unknown stat '{stat}' (one of {}).", STATS.join(", ")));
                }
                int_problems(&mut out, json::get(m, "value"), "value", false);
            }
            _ => {
                let fx = json::get(m, "effect");
                unique_tag = format!("{unique_tag}:{}", tag(fx));
                if !fx.is_object() {
                    out.push(format!("Modifier {n}: effect needs an effect object ({{\"type\": ...}})."));
                } else {
                    for message in simple::effect_errors(&Value::Array(vec![fx.clone()]), &skill_ok, &[], true, true, &super::hooks::Ctx::of(v)) {
                        out.push(format!("Modifier {n}: {message}"));
                    }
                }
            }
        }
        if seen.contains(&unique_tag) {
            out.push(format!("Modifier {n} repeats an earlier {kind} modifier (one per {unique})."));
        }
        seen.push(unique_tag);
    }
    let raw = raw_numbers(e, family);
    let base = base_of(family, &json::s(e, "hand"));
    for field in numbers {
        let (_, _, lo, hi) = number(field);
        let x = base_value(&raw, field);
        if x < lo || x > hi {
            out.push(format!("{} resolves to {x} (family base {} plus adjustments), outside {lo}-{hi}.", capitalize(field), base_value(&base, field)));
        }
    }
    out
}

fn range_problem(out: &mut Vec<String>, x: i64, lo: i64, hi: i64, n: usize, what: &str) {
    if x < lo || x > hi {
        out.push(format!("Tier {n}: {what} {x} is outside {lo}-{hi}."));
    }
}

fn unknown_keys(out: &mut Vec<String>, d: &Dict, allowed: &[&str], n: usize, what: &str) {
    for key in d.keys().filter(|k| !allowed.contains(&k.as_str())) {
        out.push(format!("Tier {n}: unknown {what} key '{key}'."));
    }
}

fn tier_problems(v: &View, e: &Dict, family: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let t = item_type_of(family);
    let tiers = tiers_of(e);
    for (i, raw) in json::aget(e, "tiers").iter().enumerate() {
        if !raw.is_object() {
            out.push(format!("Tier {} is not an object.", i + 1));
        }
    }
    if tiers.is_empty() {
        return out;
    }
    let cap = json::i(family, "max_tiers", 0);
    if tiers.len() as i64 > cap {
        out.push(format!("At most {cap} tiers (family '{}'); this item has {}.", json::s(family, "id"), tiers.len()));
    }
    let mut seen: Vec<String> = Vec::new();
    let mut last: Vec<(String, i64)> = Vec::new();
    for (i, tier) in tiers.iter().enumerate() {
        let n = i + 1;
        for key in tier.keys().filter(|k| !TIER_KEYS.contains(&k.as_str())) {
            out.push(format!("Tier {n}: unknown key '{key}' (keys: {}).", TIER_KEYS.join(", ")));
        }
        let label = chars(&json::s(tier, "label"));
        if label > TIER_LABEL_MAX {
            out.push(format!("Tier {n}: the label is {label} characters (max {TIER_LABEL_MAX})."));
        }
        let desc = chars(&json::s(tier, "desc"));
        if desc > TIER_DESC_MAX {
            out.push(format!("Tier {n}: the description is {desc} characters (max {TIER_DESC_MAX})."));
        }
        let when = json::dget(tier, "when");
        let (mut group, mut value) = (String::new(), 0);
        match json::s(when, "type").as_str() {
            WHEN_STAT => {
                let stat = json::s(when, "stat");
                if !STATS.contains(&stat.as_str()) {
                    out.push(format!("Tier {n}: unknown stat '{stat}'."));
                }
                value = json::i(when, "value", 0);
                range_problem(&mut out, value, TIER_STAT_MIN, TIER_STAT_MAX, n, "stat value");
                unknown_keys(&mut out, when, &["type", "stat", "value"], n, "criterion");
                group = format!("stat:{stat}");
            }
            WHEN_PROFICIENCY => {
                let gear = json::s_or(when, "gear", &json::s(family, "id"));
                // LAYER-DOWN-2: the family table's gear tokens (the schema's list without one), listed in the schema's order
                let tokens = super::skill::listed_gear_tokens(v.entries("item_families"));
                if !tokens.contains(&gear) {
                    out.push(format!("Tier {n}: '{gear}' is not a gear type ({}).", tokens.join(", ")));
                }
                value = json::i(when, "level", 0);
                range_problem(&mut out, value, 1, PROF_MAX_LEVEL, n, "proficiency level");
                unknown_keys(&mut out, when, &["type", "gear", "level"], n, "criterion");
                group = format!("proficiency:{gear}");
            }
            WHEN_LEVEL => {
                value = json::i(when, "value", 0);
                range_problem(&mut out, value, 1, MAX_LEVEL as i64, n, "unit level");
                unknown_keys(&mut out, when, &["type", "value"], n, "criterion");
                group = "level".into();
            }
            other => out.push(format!("Tier {n}: criterion type '{other}' is not one of {}.", WHEN_TYPES.join(", "))),
        }
        let key = tag(&Value::Object(when.clone()));
        if seen.contains(&key) {
            out.push(format!("Tier {n} repeats the criterion of an earlier tier."));
        }
        seen.push(key);
        if !group.is_empty() {
            match last.iter_mut().find(|(g, _)| *g == group) {
                Some(slot) => {
                    if value <= slot.1 {
                        out.push(format!("Tier {n}: tiers on the same criterion go in ascending order ({value} after {}).", slot.1));
                    }
                    slot.1 = value;
                }
                None => last.push((group, value)),
            }
        }
        out.extend(bonus_problems(v, json::dget(tier, "bonus"), &t, n));
    }
    out
}

fn bonus_problems(v: &View, bonus: &Dict, t: &str, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut any = false;
    for (k, raw) in bonus {
        match k.as_str() {
            "power" => {
                if t != "WEAPON" {
                    out.push(format!("Tier {n}: a power bonus is for weapons only."));
                }
                let p = json::to_int(raw);
                range_problem(&mut out, p, 0, 50, n, "power bonus");
                any = any || p > 0;
            }
            "mitigation" => {
                if t != "ARMOR" && t != "SHIELD" {
                    out.push(format!("Tier {n}: a mitigation bonus is for armor and shields only."));
                }
                let m = json::to_int(raw);
                range_problem(&mut out, m, 0, 20, n, "mitigation bonus");
                any = any || m > 0;
            }
            "stat_bonuses" => {
                let stats = json::dict(raw);
                if stats.len() > TIER_MAX_STATS {
                    out.push(format!("Tier {n}: at most {TIER_MAX_STATS} stat bonuses."));
                }
                for (stat, x) in stats {
                    // DERIVE1 4.1: a tier bonus may name a derived stat
                    if !v.is_stat(stat) {
                        out.push(format!("Tier {n}: unknown stat '{stat}'."));
                    }
                    range_problem(&mut out, json::to_int(x), 1, 5, n, &format!("{stat} bonus"));
                }
                any = any || !stats.is_empty();
            }
            "effects" => {
                let fx = json::arr(raw);
                if fx.len() > TIER_MAX_EFFECTS {
                    out.push(format!("Tier {n}: at most {TIER_MAX_EFFECTS} effects."));
                }
                for message in simple::effect_errors(&Value::Array(fx.to_vec()), &has_skill(v), &[], true, true, &super::hooks::Ctx::of(v)) {
                    out.push(format!("Tier {n}: {message}"));
                }
                any = any || !fx.is_empty();
            }
            _ => out.push(format!("Tier {n}: unknown bonus '{k}' (one of {}).", BONUS_KEYS.join(", "))),
        }
    }
    if !any {
        out.push(format!("Tier {n} needs at least one non-zero bonus."));
    }
    out
}

fn consumable_problems(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    // PK1-S1 (PK1_CORRUPTION 3.7): an `unmark` item is a use of its own
    let unmark = json::get(e, "unmark").as_object().is_some_and(|u| !u.is_empty());
    if json::i(e, "heal", 0) == 0 && json::i(e, "damage", 0) == 0 && json::f(e, "buff_pct", 0.0) == 0.0 && !unmark {
        out.push("A consumable should heal, damage or buff.".into());
    }
    out.extend(unmark_problems(e));
    // A1.2-K3 (INTENT2 section 6, IR9): only a damage item thrown at foes hits an area, and `affects` needs one
    let area = json::s(e, "area");
    let shaped = !area.is_empty() && area != rpgkit_data::ITEM_AREA_NONE;
    let foe_aimed = json::i(e, "damage", 0) > 0 && json::i(e, "heal", 0) <= 0 && json::f(e, "buff_pct", 0.0) <= 0.0;
    if shaped && !foe_aimed {
        out.push(format!("Area: only a damage item (no heal or buff) hits an area ({area})."));
    }
    if !shaped && !json::s(e, "affects").is_empty() {
        out.push("Affects: a single-target item has no area to affect.".into());
    }
    out
}

/// PK1-S1 (3.7): a consumable's `unmark` (`{}` or absent = none): an object `{mark}` with a mark id (as a trigger's
/// `mark`), on an item that does not also damage (it is used on allies).
fn unmark_problems(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let raw = json::get(e, "unmark");
    if raw.is_null() || raw.as_object().is_some_and(|o| o.is_empty()) {
        return out;
    }
    let Some(o) = raw.as_object() else {
        out.push("Unmark must be an object {\"mark\": id} ({} = none).".into());
        return out;
    };
    for k in o.keys().filter(|k| !rpgkit_data::UNMARK_KEYS.contains(&k.as_str())) {
        out.push(format!("Unmark: unknown key '{k}' (keys: {}).", rpgkit_data::UNMARK_KEYS.join(", ")));
    }
    let id = json::s(o, "mark");
    if id.is_empty() || id.chars().count() > super::hooks::MARK_ID_MAX || !aegis_content::schema::is_id(&id) {
        out.push(format!("Unmark: mark needs an id of 1-{} characters (a-z, 0-9, _, a letter first).", super::hooks::MARK_ID_MAX));
    }
    if json::i(e, "damage", 0) > 0 {
        out.push("Unmark: an unmark item is used on allies (no damage).".into());
    }
    out
}

/// LAYER-DOWN-2: the focus gear tokens: the families with the engine rule `focus`, in table order; a pre-family catalog
/// (no family table, `legacy_errors`) reads the legacy family rules' focus tokens (`rpgkit_data::LEGACY_FAMILY_RULES`).
pub fn focus_tokens(families: &[Dict]) -> Vec<String> {
    if families.is_empty() {
        return rpgkit_data::LEGACY_FAMILY_RULES.iter().filter(|(_, r)| *r == rpgkit_data::FOCUS_RULE).map(|(g, _)| g.to_string()).collect();
    }
    crate::source::families_with_rule(families, rpgkit_data::FOCUS_RULE)
}

/// ItemValidator._legacy_errors: the pre-family rules of a flat row (no item families in the catalog).
fn legacy_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let t = json::s(e, "item_type");
    let slot = json::s(e, "slot");
    let gear = json::s(e, "weapon_type");
    let off = json::s(e, "off_hand_category");
    let two = json::b(e, "two_handed", false);
    let equip = EQUIP_TYPES.contains(&t.as_str());
    let focus = focus_tokens(&[]);
    if t != "WEAPON" && focus.contains(&gear) && equip {
        errors.push("Orb and tome are weapon types (item type WEAPON).".into());
    }
    if focus.contains(&off) && gear != off {
        errors.push(format!("Off-hand category '{off}' needs the matching gear type '{off}'."));
    }
    if equip {
        errors.extend(simple::effect_errors(json::get(e, "effects"), &has_skill(v), &[], true, true, &super::hooks::Ctx::of(v)));
        match json::get(e, "threshold_tiers") {
            Value::Array(tiers) => {
                if tiers.iter().any(|t| !json::dict(t).contains_key("stat") || !json::dict(t).contains_key("threshold")) {
                    errors.push("Each threshold tier needs \"stat\" and \"threshold\".".into());
                }
            }
            _ => errors.push("Threshold tiers must be a list.".into()),
        }
    }
    match t.as_str() {
        "WEAPON" => {
            if slot != "MAIN_HAND" && slot != "OFF_HAND" {
                errors.push("Weapons go in MAIN_HAND or OFF_HAND.".into());
            }
            if two && slot != "MAIN_HAND" {
                errors.push("Two-handed weapons use the MAIN_HAND slot.".into());
            }
            if two && !off.is_empty() {
                errors.push("Hand conflict: a two-handed weapon (main_2h) can't also be an off-hand category weapon (off).".into());
            }
            if focus.contains(&gear) {
                if slot != "OFF_HAND" {
                    errors.push("Orb and tome weapons are off-hand only (use the OFF_HAND slot).".into());
                }
                if off != gear {
                    errors.push(format!("An orb or tome needs off-hand category '{gear}' (matching its gear type)."));
                }
                if two {
                    errors.push("Orb and tome weapons are never two-handed.".into());
                }
            }
        }
        "SHIELD" if slot != "OFF_HAND" => errors.push("Shields go in the OFF_HAND slot.".into()),
        "ARMOR" if slot != "HEAD" && slot != "BODY" => errors.push("Armor goes in HEAD or BODY.".into()),
        "ACCESSORY" if slot != "ACCESSORY_1" && slot != "ACCESSORY_2" => errors.push("Accessories go in ACCESSORY_1 or ACCESSORY_2.".into()),
        "CONSUMABLE" => errors.extend(consumable_problems(e)),
        _ => {}
    }
    errors
}

// ---- ItemText ----

/// ItemText.TOKEN_TYPES: token -> the item types it is valid for (None = every type).
const TEXT_TOKENS: [(&str, Option<&[&str]>); 11] = [
    ("name", None),
    ("family", None),
    ("power", Some(&["WEAPON"])),
    ("range", Some(&["WEAPON"])),
    ("hand", Some(&["WEAPON", "SHIELD"])),
    ("burden", Some(&EQUIP_TYPES)),
    ("mitigation", Some(&EQUIP_TYPES)),
    ("req", Some(&EQUIP_TYPES)),
    ("bon", Some(&EQUIP_TYPES)),
    ("tier.value", Some(&EQUIP_TYPES)),
    ("tier.power", Some(&["WEAPON"])),
];
const STAT_TOKENS: [&str; 2] = ["req", "bon"];
const NUMBER_TOKENS: [(&str, &str); 4] = [("power", "base_power"), ("range", "base_range"), ("burden", "burden"), ("mitigation", "mitigation")];

fn token_valid_for(token: &str, t: &str) -> bool {
    TEXT_TOKENS.iter().find(|(k, _)| *k == token).is_some_and(|(_, types)| types.is_none_or(|ts| ts.contains(&t)))
}

fn tokens_for(t: &str) -> Vec<String> {
    TEXT_TOKENS
        .iter()
        .map(|(k, _)| *k)
        .filter(|k| token_valid_for(k, t))
        .map(|k| {
            if STAT_TOKENS.contains(&k) {
                format!("{k}:STAT")
            } else if let Some(field) = k.strip_prefix("tier.") {
                format!("tier1.{field}")
            } else {
                k.to_string()
            }
        })
        .collect()
}

/// ItemText._split: "tier2.power" -> ("tier.power", 2); any other token -> (token, 0).
fn split(token: &str) -> (String, usize) {
    if let Some((digits, field)) = token.strip_prefix("tier").and_then(|r| r.split_once('.')) {
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) && (field == "value" || field == "power") {
            return (format!("tier.{field}"), digits.parse().unwrap_or(0));
        }
    }
    (token.to_string(), 0)
}

/// ItemText.token_value: the text of one token for a resolved row ("" when it has no value for this item).
fn token_value(token: &str, suffix: &str, row: &Dict) -> String {
    let (base, n) = split(token);
    match base.as_str() {
        "name" | "family" => json::s(row, &base),
        "hand" => match json::s(row, "hand").as_str() {
            "" => String::new(),
            "main_1h" => "main hand, one-handed".into(),
            "main_2h" => "main hand, two-handed".into(),
            "off" => "off hand".into(),
            _ => "-".into(),
        },
        "req" | "bon" => {
            let stats = json::dget(row, if base == "req" { "required_stats" } else { "stat_bonuses" });
            stats.get(suffix).map(|x| json::to_int(x).to_string()).unwrap_or_default()
        }
        "tier.value" | "tier.power" => {
            let tiers = tiers_of(row);
            if n < 1 || n > tiers.len() {
                return String::new();
            }
            let tier = &tiers[n - 1];
            if base == "tier.value" {
                let when = json::dget(tier, "when");
                return json::to_int(when.get("value").or_else(|| when.get("level")).unwrap_or(&json::int(0))).to_string();
            }
            let power = json::i(json::dget(tier, "bonus"), "power", 0);
            if power > 0 {
                power.to_string()
            } else {
                String::new()
            }
        }
        _ => NUMBER_TOKENS.iter().find(|(k, _)| *k == base).map(|(_, key)| json::i(row, key, 0).to_string()).unwrap_or_default(),
    }
}

/// ItemText.validate: the description template problems of a resolved row.
fn text_problems(template: &str, row: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let t = json::s_or(row, "item_type", "WEAPON");
    if chars(template) > TEXT_MAX {
        errors.push(format!("Description is {} characters (max {TEXT_MAX}).", chars(template)));
    }
    if !template.contains('{') && !template.contains('}') {
        return errors;
    }
    let (parts, parse_errors) = super::skill::parse(template);
    errors.extend(parse_errors);
    let valid = tokens_for(&t).join(", ");
    for part in &parts {
        let super::skill::Part::Token { token, suffix, raw } = part else { continue };
        let (base, _) = split(token);
        if !TEXT_TOKENS.iter().any(|(k, _)| *k == base) {
            errors.push(format!("Unknown token {{{raw}}}. Valid for {t}: {valid}."));
        } else if !token_valid_for(&base, &t) {
            errors.push(format!("Token {{{raw}}} does not apply to {t} items. Valid: {valid}."));
        } else if STAT_TOKENS.contains(&base.as_str()) && !STATS.contains(&suffix.as_str()) {
            errors.push(format!("Token {{{raw}}} needs a stat: {{{base}:STR}} (one of {}).", STATS.join(", ")));
        } else if !STAT_TOKENS.contains(&base.as_str()) && !suffix.is_empty() {
            errors.push(format!("Token {{{raw}}}: unknown suffix ':{suffix}'."));
        } else if token_value(token, suffix, row).is_empty() {
            errors.push(format!("Token {{{raw}}} has no value for this item (no such requirement, bonus or tier)."));
        }
    }
    if errors.is_empty() {
        let rendered = chars(&render_item(template, row));
        if rendered > TEXT_MAX {
            errors.push(format!("The rendered description is {rendered} characters (max {TEXT_MAX})."));
        }
    }
    errors
}

/// ItemText.warnings: literal numbers equal to a resolved number (suggests the token); descriptions without tokens
/// never warn.
fn text_warnings(template: &str, row: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    if !template.contains('{') {
        return out;
    }
    let t = json::s_or(row, "item_type", "WEAPON");
    let mut candidates: Vec<(String, String)> = Vec::new();
    let mut add = |digits: String, token: String| {
        if !candidates.iter().any(|(d, _)| *d == digits) {
            candidates.push((digits, token));
        }
    };
    for (token, key) in NUMBER_TOKENS {
        let value = json::i(row, key, 0);
        if token_valid_for(token, &t) && value > 1 {
            add(value.to_string(), token.to_string());
        }
    }
    for base in STAT_TOKENS {
        if !token_valid_for(base, &t) {
            continue;
        }
        for (stat, x) in json::dget(row, if base == "req" { "required_stats" } else { "stat_bonuses" }) {
            let x = json::to_int(x);
            if x > 1 {
                add(x.to_string(), format!("{base}:{stat}"));
            }
        }
    }
    for i in 0..tiers_of(row).len() {
        for field in ["value", "power"] {
            let tok = format!("tier{}.{field}", i + 1);
            let text = token_value(&tok, "", row);
            if token_valid_for(&format!("tier.{field}"), &t) && text.parse::<i64>().unwrap_or(0) > 1 {
                add(text, tok);
            }
        }
    }
    if candidates.is_empty() {
        return out;
    }
    let (parts, _) = super::skill::parse(template);
    for part in &parts {
        let super::skill::Part::Lit(lit) = part else { continue };
        let mut digits = String::new();
        for ch in lit.chars().chain(std::iter::once(' ')) {
            if ch.is_ascii_digit() {
                digits.push(ch);
            } else if !digits.is_empty() {
                if let Some((_, token)) = candidates.iter().find(|(d, _)| *d == digits) {
                    out.push(format!("Description has the literal number {digits}; it can drift from the item. Use {{{token}}}."));
                }
                digits.clear();
            }
        }
    }
    out
}

/// ItemValidator.item_warnings plus GearBudget.check_item (ContentCatalog.warnings_for "items").
pub fn item_warnings(v: &View, e: &Dict) -> Vec<String> {
    let def = ItemDef::from_obj(e);
    let mut out = budget::check_item(&def);
    let family = family_of(v, e);
    out.extend(text_warnings(&json::s(e, "description"), e));
    if json::s(e, "item_type") == "CONSUMABLE" && def.type_reach() < 0 {
        out.push(format!("Consumable type {} is not offered in battle (no battle use yet).", json::s(e, "consumable_type")));
    }
    if family.is_empty() || budget::is_special(&def) {
        return out;
    }
    let hand = json::s(e, "hand");
    for field in number_fields_for(&item_type_of(family)) {
        let band = band_of(family, &hand, field);
        if band.is_empty() {
            continue;
        }
        let x = json::i(e, number(field).1, 0);
        let (lo, hi) = (json::i(band, "min", x), json::i(band, "max", x));
        if x < lo || x > hi {
            out.push(format!("{} {x} is outside the family band {}-{} (soft until the balance pass).", capitalize(field), json::i(band, "min", 0), json::i(band, "max", 0)));
        }
    }
    let band = band_of(family, &hand, BONUS_BAND);
    if !band.is_empty() {
        let value: f64 = modifiers_of(e).iter().filter(|m| matches!(json::s(m, "kind").as_str(), "stat_bonus" | "effect")).fold(0.0, |total, m| total + budget::modifier_value(&Value::Object(m.clone())));
        if value < json::f(band, "min", value) - 0.0001 || value > json::f(band, "max", value) + 0.0001 {
            out.push(format!("Bonus value {} (stat bonuses and effects) is outside the family band {}-{} (soft).", fixed(value, 1), json::s(band, "min"), json::s(band, "max")));
        }
    }
    out
}

// ---- ItemValidator: families ----

/// ItemValidator.family_errors of a derived family row (`current_id`: the id it replaces). Every item of the family is
/// checked again against the new row.
pub fn family_errors(v: &View, f: &Dict, current_id: &str) -> Vec<String> {
    let mut out = Vec::new();
    let t = item_type_of(f);
    let id = json::s(f, "id");
    // LAYER-DOWN-2: with a family table a gear family's id is its gear token (a new family adds one; any id but the
    // reserved `unarmed`); a catalog without one keeps the schema's tokens (`skill::listed_gear_tokens`)
    let families = v.entries("item_families");
    let gear_types = super::skill::listed_gear_tokens(families);
    let named = if families.is_empty() { gear_types.contains(&id) } else { aegis_content::schema::is_id(&id) && id != super::skill::UNARMED_TOKEN };
    if GEAR_TYPES.contains(&t.as_str()) && !named {
        out.push(format!("A {t} family is named after its gear token (one of {}).", gear_types.join(", ")));
    }
    let hands = hands_of(f);
    if HAND_TYPES.contains(&t.as_str()) {
        let mut seen: Vec<&String> = Vec::new();
        for h in &hands {
            if !HANDS.contains(&h.as_str()) {
                out.push(format!("Unknown hand '{h}' (one of {}).", HANDS.join(", ")));
            }
            if seen.contains(&h) {
                out.push(format!("Hands list '{h}' twice."));
            }
            seen.push(h);
        }
        let cat = category(f);
        if hands.is_empty() {
            out.push(format!("{t} families need at least one hand."));
        } else if cat.is_empty() {
            out.push("One hand category per family: main hands (main_1h, main_2h; damage weapons) or exactly off (support items); mixing is an error.".into());
        }
        if t == "SHIELD" && cat != "off" {
            out.push("Shields are off-hand only (hands = off).".into());
        }
        if cat == "off" && !OFF_CATEGORIES.contains(&id.as_str()) {
            out.push(format!("Off-hand families are {} (their id is the off-hand category).", OFF_CATEGORIES.join(", ")));
        }
    }
    if t == "WEAPON" {
        let scaling = json::sget(f, "scaling_stats");
        if scaling.is_empty() {
            out.push("Weapon families need at least one scaling stat.".into());
        }
        for s in scaling.iter().filter(|s| !SCALING_STATS.contains(&s.as_str())) {
            out.push(format!("Weapons scale from {} only ('{s}').", SCALING_STATS.join(", ")));
        }
    }
    if SLOT_TYPES.contains(&t.as_str()) {
        let allowed: &[&str] = if t == "ARMOR" { &ARMOR_SLOTS } else { &ACCESSORY_SLOTS };
        let slots = json::sget(f, "slots");
        if slots.is_empty() {
            out.push(format!("{t} families list their slots ({}).", allowed.join(", ")));
        }
        for s in slots.iter().filter(|s| !allowed.contains(&s.as_str())) {
            out.push(format!("Slot '{s}' is not a {t} slot ({}).", allowed.join(", ")));
        }
    }
    out.extend(base_problems(f, &t));
    let numbers = number_fields_for(&t);
    for a in json::sget(f, "adjustable").iter().filter(|a| !numbers.contains(&a.as_str())) {
        out.push(format!("'{a}' is not a number of {t} items ({}).", numbers.join(", ")));
    }
    let kinds = json::sget(f, "modifier_kinds");
    for k in kinds.iter().filter(|k| !KINDS.iter().any(|x| x.0 == k.as_str())) {
        out.push(format!("Unknown modifier kind '{k}' (one of {}).", kind_names()));
    }
    if !kinds.is_empty() && !MODIFIABLE_TYPES.contains(&t.as_str()) {
        out.push(format!("{t} families take no modifiers."));
    }
    for rule in json::sget(f, "engine_rules") {
        if !ENGINE_RULES.contains(&rule.as_str()) {
            out.push(format!("Unknown engine rule '{rule}' (closed list: {}).", ENGINE_RULES.join(", ")));
        } else if rule == "focus" && (t != "WEAPON" || category(f) != "off") {
            out.push("focus needs a WEAPON family with hands = off.".into());
        } else if rule == "shield" && t != "SHIELD" {
            out.push("shield needs a SHIELD family.".into());
        } else if (rule == "always_ranged" || rule == "spell_source") && t != "WEAPON" {
            out.push(format!("{rule} needs a WEAPON family."));
        } else if rule == "reach" {
            out.extend(reach_rule_problems(f, &t));
        }
    }
    out.extend(mastery_problems(v, f, &t));
    out.extend(technique_problems(v, f));
    out.extend(durability_problems(f));
    out.extend(range_band_problems(f));
    let old_id = if current_id.is_empty() { id.clone() } else { current_id.to_string() };
    let users: Vec<&Dict> = v.entries("items").iter().filter(|i| json::s(i, "family") == old_id).collect();
    if old_id != id && !users.is_empty() {
        out.push(format!("The id of family '{old_id}' is read-only while items use it ({} items).", users.len()));
    }
    let mut failing = Vec::new();
    for item in users {
        let row = content("items", &resolve_item(&stored(item), f));
        if let Some(first) = item_errors(v, &row, f, true).first() {
            failing.push(format!("{} ({first})", json::s(item, "id")));
        }
    }
    if !failing.is_empty() {
        out.push(format!("This family change breaks {} item(s): {}.", failing.len(), failing.join("; ")));
    }
    out
}

/// U3.2-S4: the bounds of a family's Level-2 bonus (`level2_bonus`): crit chance points, burden taken off each item.
const LEVEL2_CRIT: (f64, f64) = (1.0, 10.0);
const LEVEL2_BURDEN: (i64, i64) = (1, 3);
/// U3.2-S4: the item types whose proficiency a unit trains (an accessory, a consumable or a material has no level).
const TRAINED_TYPES: [&str; 3] = ["WEAPON", "SHIELD", "ARMOR"];

/// U3.2-S4 (U3 Level 2 and Level 3): `level2_bonus` is `{}` or exactly one of `{"crit": 1-10}` (crit chance points) and
/// `{"burden": 1-3}` (whole); `level3_passive` is "" or an always-on passive (slot "") whose effects worn gear can apply
/// (`grants::Grant::gear_ok`: no unlock, foresight, stagger_taken, meter or proficiency); both only on a family whose
/// level a unit trains (WEAPON, SHIELD, ARMOR).
fn mastery_problems(v: &View, f: &Dict, t: &str) -> Vec<String> {
    let mut out = Vec::new();
    let raw = json::get(f, "level2_bonus");
    let bonus = raw.as_object();
    let level3 = json::s(f, "level3_passive");
    let set = bonus.is_some_and(|b| !b.is_empty()) || !level3.is_empty();
    if set && !TRAINED_TYPES.contains(&t) {
        out.push(format!("Level 2 and 3 bonuses need a family a unit trains ({}).", TRAINED_TYPES.join(", ")));
    }
    match bonus {
        None if !raw.is_null() => out.push("Level 2 bonus must be an object: {} or {\"crit\": n} or {\"burden\": n}.".into()),
        Some(b) if b.len() > 1 => out.push("Level 2 bonus is one of crit or burden, not both.".into()),
        Some(b) => {
            for (k, x) in b {
                match k.as_str() {
                    "crit" if !x.as_f64().is_some_and(|c| (LEVEL2_CRIT.0..=LEVEL2_CRIT.1).contains(&c)) => {
                        out.push(format!("Level 2 crit must be {}-{}.", LEVEL2_CRIT.0, LEVEL2_CRIT.1));
                    }
                    "burden" if !x.as_f64().is_some_and(|c| c.fract() == 0.0 && (LEVEL2_BURDEN.0 as f64..=LEVEL2_BURDEN.1 as f64).contains(&c)) => {
                        out.push(format!("Level 2 burden must be a whole number {}-{}.", LEVEL2_BURDEN.0, LEVEL2_BURDEN.1));
                    }
                    "crit" | "burden" => {}
                    _ => out.push(format!("Level 2 bonus: unknown key '{k}' (crit or burden).")),
                }
            }
        }
        None => {}
    }
    if !level3.is_empty() {
        match v.typed.passives.get(&level3) {
            None => out.push(format!("Level 3 passive: unknown passive '{level3}'.")),
            Some(p) if !p.slot.is_empty() => out.push(format!("Level 3 passive '{level3}' is slottable ({}); a mastery passive is always on (slot \"\").", p.slot)),
            Some(p) => {
                if rpgkit_rules::grants::read(&p.effects).iter().any(|g| !g.gear_ok()) {
                    out.push(format!("Level 3 passive '{level3}' has an effect worn gear can't apply (unlock, foresight, stagger_taken, meter or proficiency)."));
                }
                // applied as the item's, so only what an item may carry (no ap_regen, whose bonus no sync resets)
                for e in &p.effects {
                    let t = e.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if !super::simple::ITEM_TYPES.contains(&t.as_str()) {
                        out.push(format!("Level 3 passive '{level3}' has effect '{t}', which an item can't carry ({}).", super::simple::ITEM_TYPES.join(", ")));
                    }
                }
            }
        }
    }
    out
}

/// RULESET1 K1: the engine rule `reach` is for melee WEAPON families only: not with always_ranged, and every hand's
/// base range is 1 (the rule, not the range, gives the reach of 2).
fn reach_rule_problems(f: &Dict, t: &str) -> Vec<String> {
    let mut out = Vec::new();
    if t != "WEAPON" {
        out.push("reach needs a WEAPON family.".into());
        return out;
    }
    if json::sget(f, "engine_rules").iter().any(|r| r == "always_ranged") {
        out.push("reach is for melee families; it can't go with always_ranged.".into());
    }
    for (hand, row) in json::dget(f, "base") {
        let range = json::get(json::dict(row), "range");
        if !range.is_null() && json::to_int(range) != 1 {
            out.push(format!("reach needs base range 1 on every hand ({hand} has {}).", json::to_int(range)));
        }
    }
    out
}

/// PR5-28 (U3 FB-36c): the widest `range_band.max` (the top of a family's range number).
const RANGE_BAND_MAX: i64 = 6;

/// PR5-28 (U3 FB-36c): `range_band` is `{}` (none: the reach rule's fallback band edge 2) or `{"min": 1, "max": 2-6}`
/// (whole numbers), and only on a family with the engine rule `reach` (the one rule that reads it).
fn range_band_problems(f: &Dict) -> Vec<String> {
    let raw = json::get(f, "range_band");
    let Some(band) = raw.as_object() else {
        return if raw.is_null() { Vec::new() } else { vec!["Range band must be an object: {} or {\"min\": 1, \"max\": n}.".into()] };
    };
    if band.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if !json::sget(f, "engine_rules").iter().any(|r| r == "reach") {
        out.push("Range band needs the engine rule reach (the reach rule's melee band).".into());
    }
    if let Some(k) = band.keys().find(|k| *k != "min" && *k != "max") {
        out.push(format!("Range band: unknown key '{k}' (min, max)."));
    }
    let whole = |k: &str| band.get(k).and_then(Value::as_i64);
    if whole("min") != Some(1) {
        out.push("Range band min must be 1 (a melee band starts next to the target).".into());
    }
    if !whole("max").is_some_and(|m| (2..=RANGE_BAND_MAX).contains(&m)) {
        out.push(format!("Range band max must be a whole number 2-{RANGE_BAND_MAX}."));
    }
    out
}

/// ItemValidator._base_problems: `base` and `bands`, one row per hand ("none" without hands), numbers in range.
fn base_problems(f: &Dict, t: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut keys = hands_of(f);
    if keys.is_empty() {
        keys.push(NO_HAND_KEY.into());
    }
    let keys_text = keys.join(", ");
    if f.get("base").is_some_and(|b| !b.is_object()) {
        out.push("Base numbers are an object keyed by hand.".into());
    }
    let base = json::dget(f, "base");
    for key in base.keys().filter(|k| !keys.contains(k)) {
        out.push(format!("Base row '{key}' is not one of the family's hands ({keys_text})."));
    }
    for key in &keys {
        let Some(row) = base.get(key) else {
            if MODIFIABLE_TYPES.contains(&t) {
                out.push(format!("Base numbers for '{key}' are missing."));
            }
            continue;
        };
        for (field, x) in json::dict(row) {
            let Some(&(_, _, lo, hi)) = NUMBERS.iter().find(|n| n.0 == field.as_str()) else {
                out.push(format!("Base '{key}': unknown number '{field}' ({}).", NUMBERS.iter().map(|n| n.0).collect::<Vec<_>>().join(", ")));
                continue;
            };
            let x = json::to_int(x);
            if x < lo || x > hi {
                out.push(format!("Base '{key}' {field} {x} is outside {lo}-{hi}."));
            }
        }
    }
    for (key, row) in json::dget(f, "bands") {
        if !keys.contains(key) {
            out.push(format!("Band row '{key}' is not one of the family's hands ({keys_text})."));
            continue;
        }
        for (name, band) in json::dict(row) {
            let band = json::dict(band);
            let (lo, hi) = (json::f(band, "min", 0.0), json::f(band, "max", 0.0));
            if !band.contains_key("min") || !band.contains_key("max") {
                out.push(format!("Band '{key}' {name} needs min and max."));
            } else if lo > hi {
                out.push(format!("Band '{key}' {name}: min {} is above max {}.", json::s(band, "min"), json::s(band, "max")));
            }
            if name == BONUS_BAND {
                if lo < 0.0 {
                    out.push(format!("Band '{key}' bonus_value starts at 0 or above."));
                }
            } else if let Some(&(_, _, rlo, rhi)) = NUMBERS.iter().find(|n| n.0 == name.as_str()) {
                if lo < rlo as f64 || hi > rhi as f64 {
                    out.push(format!("Band '{key}' {name} {}-{} is outside the schema range {rlo}-{rhi}.", json::s(band, "min"), json::s(band, "max")));
                }
            } else {
                out.push(format!("Band '{key}': unknown number '{name}'."));
            }
        }
    }
    out
}

/// ItemValidator.family_warnings: base numbers outside the family's own bands, a 2H base power below the 1H one, a
/// focus power band above 10.
pub fn family_warnings(f: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let hands = hands_of(f);
    let mut keys = hands.clone();
    if keys.is_empty() {
        keys.push(NO_HAND_KEY.into());
    }
    for key in &keys {
        let base = base_of(f, key);
        for field in number_fields_for(&item_type_of(f)) {
            let band = band_of(f, key, field);
            if band.is_empty() {
                continue;
            }
            let x = base_value(&base, field);
            if x < json::i(band, "min", x) || x > json::i(band, "max", x) {
                out.push(format!("Base '{key}' {field} {x} is outside its own band {}-{}.", json::s(band, "min"), json::s(band, "max")));
            }
        }
    }
    if hands.iter().any(|h| h == "main_1h") && hands.iter().any(|h| h == "main_2h") {
        let one = base_value(&base_of(f, "main_1h"), "power");
        let two = base_value(&base_of(f, "main_2h"), "power");
        if two < one {
            out.push(format!("The main_2h base power {two} is below the main_1h base power {one}."));
        }
    }
    if json::sget(f, "engine_rules").iter().any(|r| r == "focus") {
        for key in &keys {
            let band = band_of(f, key, "power");
            if !band.is_empty() && json::i(band, "max", 0) > 10 {
                out.push(format!("A focus weapon is a support item: its power band max {} should stay at 10 or below.", json::s(band, "max")));
            }
        }
    }
    out
}

/// ContentCatalog._prepare("item_families"): ItemFamilies.derive of the normalized row.
pub fn prepare_family(raw: &Dict) -> Dict {
    crate::source::derive_family(&content("item_families", raw))
}
