//! U3.2-S2: the family table drives the gear rules with no change to this game's results. On the goldens' frozen family
//! table (`emberwake_sim/tests/fixtures/sim_data.json`): the gear and weapon tokens the content check derives from the
//! table are the pre-family lists (as sets; the table lists them in its own order; LAYER-DOWN-2: the kit schema's
//! tokens, no code list), and every off-hand category is one of the check's off-hand families. GEAR-REWORK1 (PR5-51b,
//! `docs/sessions/U3_GEAR_REWORK.md` 2-5): the shipped table is the reworked one, so its tokens are its own families
//! (the spec's retired gear tokens gone, armour by weight, the seven `off_only` off-hand families, each its own
//! off-hand category). Every shipped item of a pre-family gear token and every skill reads the same gear behaviour from
//! its family's engine rules as from the pre-family reading (the legacy rules of its gear token).

use aegis_core::json::{self, Dict};
use emberwake_content::check::{item, skill};
use emberwake_content::source;
use emberwake_data::{ItemDef, SkillDef, ITEM_FAMILY_RULES, ITEM_REACH_RULE, REACH_ENGINE_RULE, SKILL_SPELL_SOURCE};
use emberwake_rules::gear::{off_hand_ok, Equipment};
use emberwake_rules::unit::skill_in_group;
use serde_json::Value;
use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn dicts(v: &Value) -> Vec<Dict> {
    v.as_array().map(|a| a.iter().filter_map(|e| e.as_object().cloned()).collect()).unwrap_or_default()
}

fn sorted(mut list: Vec<String>) -> Vec<String> {
    list.sort();
    list
}

fn sorted_consts(list: &[&str]) -> Vec<String> {
    sorted(list.iter().map(|s| s.to_string()).collect())
}

/// The shipped family table and the goldens' frozen one.
fn tables() -> Vec<(&'static str, Vec<Dict>)> {
    let shipped = source::load(&crate_dir().join("../../data"));
    let text = std::fs::read_to_string(crate_dir().join("../emberwake_sim/tests/fixtures/sim_data.json")).expect("the frozen game data");
    let frozen: Value = serde_json::from_str(&text).expect("sim_data.json");
    vec![("shipped", dicts(&shipped.game["item_families"])), ("frozen", dicts(&frozen["game"]["item_families"]))]
}

#[test]
fn the_derived_token_lists_are_the_pre_family_lists() {
    // LAYER-DOWN-2: the pre-family lists are the kit schema's gear tokens (`skill::schema_gear_tokens`) and the legacy
    // family rules' focus tokens; the kit code lists them no more
    let pre_family = skill::gear_tokens(&[]);
    let pre_weapons = skill::weapon_tokens(&[]);
    let (name, families) = tables().remove(1);
    assert!(!families.is_empty(), "{name}: a family table");
    assert_eq!(sorted(skill::gear_tokens(&families)), sorted(pre_family.clone()), "{name}: the gear tokens");
    assert_eq!(sorted(skill::weapon_tokens(&families)), sorted(pre_weapons.clone()), "{name}: the weapon tokens");
    // the messages list the table's tokens in the pre-family order (checks_golden)
    assert_eq!(skill::listed_gear_tokens(&families), pre_family, "{name}: the listed order");
    let off: Vec<String> = families.iter().map(|f| json::s(f, "off_hand_category")).filter(|c| !c.is_empty()).collect();
    assert_eq!(sorted(off), sorted_consts(&item::OFF_CATEGORIES), "{name}: the off-hand categories");
    assert_eq!(sorted(item::focus_tokens(&families)), sorted(item::focus_tokens(&[])), "{name}: the focus types");
    // without a family table the check reads the schema's list (the hand gear without the armour)
    assert_eq!(skill::gear_tokens(&[]), skill::schema_gear_tokens());
    assert_eq!(pre_weapons.len() + 3, pre_family.len());
}

/// The gear tokens the spec retires (U3.3 6.2: `axe`, `mace`, `wand`; U3.3 5: the armour families become weights).
const RETIRED_TOKENS: [&str; 6] = ["axe", "mace", "wand", "plate", "leather", "cloth"];
/// U3.3 4: the seven `off_only` off-hand families.
const OFF_ONLY_FAMILIES: [&str; 7] = ["shield", "buckler", "braces", "ammo", "cloak", "orb", "tome"];
/// U3.3 5: the armour weights.
const ARMOR_WEIGHTS: [&str; 3] = ["heavy", "medium", "light"];

/// GEAR-REWORK1 (PR5-51b): the shipped table's derived tokens are its own families: the gear tokens are its WEAPON,
/// SHIELD and ARMOR family ids, the weapon tokens the same without the armour (one family per weight), the messages
/// list the schema's surviving tokens first; none of the retired tokens is left; the off-hand families are the seven
/// `off_only` ones, each its own off-hand category (the closed `OFF_CATEGORIES` list holds only for a family without a
/// hand category, so it stays the pre-family list).
#[test]
fn the_shipped_tokens_are_the_reworked_families() {
    let (name, families) = tables().remove(0);
    assert_eq!(name, "shipped");
    let ids_of = |types: &[&str]| -> Vec<String> { families.iter().filter(|f| types.contains(&json::s_or(f, "item_type", "WEAPON").as_str())).map(|f| json::s(f, "id")).collect() };
    let gear = skill::gear_tokens(&families);
    assert_eq!(gear, ids_of(&["WEAPON", "SHIELD", "ARMOR"]), "the gear tokens are the gear families in table order");
    assert_eq!(skill::weapon_tokens(&families), ids_of(&["WEAPON", "SHIELD"]), "the weapon tokens are the hand families");
    let armor: Vec<Dict> = families.iter().filter(|f| json::s(f, "item_type") == "ARMOR").cloned().collect();
    assert_eq!(sorted(armor.iter().map(|f| json::s(f, emberwake_data::FAMILY_ARMOR_WEIGHT)).collect()), sorted_consts(&ARMOR_WEIGHTS), "one armour family per weight");
    for t in RETIRED_TOKENS {
        assert!(!gear.iter().any(|g| g == t), "{t} is retired");
    }
    let schema = skill::schema_gear_tokens();
    let mut listed: Vec<String> = schema.iter().filter(|g| gear.contains(g)).cloned().collect();
    listed.extend(gear.iter().filter(|g| !schema.contains(g)).cloned());
    assert_eq!(skill::listed_gear_tokens(&families), listed, "the schema's surviving tokens first, then the table's");
    let off: Vec<&Dict> = families.iter().filter(|f| json::sget(f, "hands") == ["off"]).collect();
    assert_eq!(sorted(off.iter().map(|f| json::s(f, "id")).collect()), sorted_consts(&OFF_ONLY_FAMILIES), "the off-hand families");
    for f in off {
        let id = json::s(f, "id");
        assert_eq!(json::s(f, emberwake_data::FAMILY_SLOT_RULE), emberwake_data::SLOT_OFF_ONLY, "{id}: off_only");
        assert_eq!(json::s(f, "off_hand_category"), id, "{id}: its own off-hand category");
    }
    assert!(item::focus_tokens(&families).iter().all(|t| OFF_ONLY_FAMILIES.contains(&t.as_str())), "a focus is an off-hand family");
}

/// The pre-family row of a resolved item: no family rules (so the legacy reading), the old loader's reach key.
fn pre_family(row: &Dict) -> ItemDef {
    let mut old = row.clone();
    let reach = json::sget(row, ITEM_FAMILY_RULES).iter().any(|r| r == REACH_ENGINE_RULE);
    old.shift_remove(ITEM_FAMILY_RULES);
    if reach {
        old.insert(ITEM_REACH_RULE.into(), Value::Bool(true));
    }
    ItemDef::from_obj(&old)
}

#[test]
fn shipped_gear_keeps_its_behaviour() {
    let data = crate_dir().join("../../data");
    let src = source::load(&data);
    let items = dicts(&src.catalog["items"]);
    assert!(!items.is_empty());
    // a two-handed main weapon, to ask the focus rule of each off-hand item
    let two = items.iter().map(ItemDef::from_obj).find(|i| i.two_handed).expect("a shipped two-handed weapon");
    // GEAR-REWORK1 (PR5-51b): the pre-family reading exists only for the pre-family gear tokens (the schema's); an item
    // of a reworked family (buckler, braces, ammo, cloak, the armour weights, ...) has no pre-family behaviour to keep
    let pre_tokens = skill::gear_tokens(&[]);
    let mut compared = 0;
    for row in items.iter().filter(|r| pre_tokens.contains(&json::s(r, "weapon_type"))) {
        compared += 1;
        let (new, old) = (ItemDef::from_obj(row), pre_family(row));
        let (n, o) = (Equipment::new(&new), Equipment::new(&old));
        let id = &new.id;
        assert_eq!(n.is_ranged_weapon(), o.is_ranged_weapon(), "{id}: ranged");
        assert_eq!(n.has_reach(), o.has_reach(), "{id}: reach");
        assert_eq!(n.is_shield(), old.item_type == "SHIELD" || old.off_hand_category == "shield", "{id}: shield");
        assert_eq!(off_hand_ok(Some(&two), &new), off_hand_ok(Some(&two), &old), "{id}: under a two-handed main");
    }
    assert!(compared > 0, "some shipped items keep a pre-family gear token");
    for row in dicts(&src.catalog["skills"]) {
        assert!(row.contains_key(SKILL_SPELL_SOURCE), "the loader marks every skill");
        let new = SkillDef::from_obj(&row);
        // PR5-57 added `spell_source` to the reworked `sprayer` family (`LEGACY_FAMILY_RULES` pre-dates `sprayer`).
        if new.weapon_types.iter().any(|w| w == "sprayer") {
            if new.kind == "ATTACK" {
                assert!(skill_in_group(&new, "spell"), "{}: sprayer attacks count in group spell", new.id);
            }
            continue;
        }
        let mut old = row.clone();
        old.shift_remove(SKILL_SPELL_SOURCE);
        assert_eq!(skill_in_group(&new, "spell"), skill_in_group(&SkillDef::from_obj(&old), "spell"), "{}: the group spell", new.id);
    }
}
