//! U3.2-S2: the family table drives the gear rules with no change to this game's results. On the shipped data and on the
//! goldens' frozen family table (`emberwake_sim/tests/fixtures/sim_data.json`): the gear and weapon tokens the content
//! check derives from the table are the pre-family lists (as sets; the table lists them in its own order; LAYER-DOWN-2: the
//! kit schema's tokens, no code list), every
//! off-hand category is one of the check's off-hand families, and every shipped item and skill reads the same gear
//! behaviour from its family's engine rules as from the pre-family reading (the legacy rules of its gear token).

use aegis_core::json::Dict;
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
    for (name, families) in tables() {
        assert!(!families.is_empty(), "{name}: a family table");
        assert_eq!(sorted(skill::gear_tokens(&families)), sorted(pre_family.clone()), "{name}: the gear tokens");
        assert_eq!(sorted(skill::weapon_tokens(&families)), sorted(pre_weapons.clone()), "{name}: the weapon tokens");
        // the messages list the table's tokens in the pre-family order (checks_golden)
        assert_eq!(skill::listed_gear_tokens(&families), pre_family, "{name}: the listed order");
        let off: Vec<String> = families.iter().map(|f| aegis_core::json::s(f, "off_hand_category")).filter(|c| !c.is_empty()).collect();
        assert_eq!(sorted(off), sorted_consts(&item::OFF_CATEGORIES), "{name}: the off-hand categories");
        assert_eq!(sorted(item::focus_tokens(&families)), sorted(item::focus_tokens(&[])), "{name}: the focus types");
    }
    // without a family table the check reads the schema's list (the hand gear without the armour)
    assert_eq!(skill::gear_tokens(&[]), skill::schema_gear_tokens());
    assert_eq!(pre_weapons.len() + 3, pre_family.len());
}

/// The pre-family row of a resolved item: no family rules (so the legacy reading), the old loader's reach key.
fn pre_family(row: &Dict) -> ItemDef {
    let mut old = row.clone();
    let reach = aegis_core::json::sget(row, ITEM_FAMILY_RULES).iter().any(|r| r == REACH_ENGINE_RULE);
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
    for row in &items {
        let (new, old) = (ItemDef::from_obj(row), pre_family(row));
        let (n, o) = (Equipment::new(&new), Equipment::new(&old));
        let id = &new.id;
        assert_eq!(n.is_ranged_weapon(), o.is_ranged_weapon(), "{id}: ranged");
        assert_eq!(n.has_reach(), o.has_reach(), "{id}: reach");
        assert_eq!(n.is_shield(), old.item_type == "SHIELD" || old.off_hand_category == "shield", "{id}: shield");
        assert_eq!(off_hand_ok(Some(&two), &new), off_hand_ok(Some(&two), &old), "{id}: under a two-handed main");
    }
    for row in dicts(&src.catalog["skills"]) {
        assert!(row.contains_key(SKILL_SPELL_SOURCE), "the loader marks every skill");
        let new = SkillDef::from_obj(&row);
        let mut old = row.clone();
        old.shift_remove(SKILL_SPELL_SOURCE);
        assert_eq!(skill_in_group(&new, "spell"), skill_in_group(&SkillDef::from_obj(&old), "spell"), "{}: the group spell", new.id);
    }
}
