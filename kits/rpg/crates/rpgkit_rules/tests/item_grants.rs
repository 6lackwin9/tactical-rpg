//! U3.2-S4 (docs/sessions/U3_GEAR.md: OFFHAND1, FB-35; Level 2 and Level 3) on a scratch catalog built here (no shipped
//! data; the family bonus values are test-only). What a worn item grants is rebuilt on every sync, so taking it off takes
//! the grant away:
//!
//! - a `skill` effect: the skill is owned, unlocked and open while worn with no slot (no loadout or deck slot; a deck
//!   that names its family does not slot it), and gone when the item comes off; the hand rule decides what is worn;
//! - a `stat` effect: a modifier under `GEAR_EFFECT_SOURCE`, never stacking over syncs;
//! - a `proficiency` effect on an item applies nothing (training stays the class's and the drills');
//! - the family's Level-2 bonus (crit points, or burden off each item) from proficiency 2, each family once; its Level-3
//!   passive from proficiency 3, applied as the item's effects and taken off with it.

use rpgkit_data::{Catalog, ItemDef, ITEM_FAMILY_LEVEL2, ITEM_FAMILY_LEVEL3};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::ruleset::DeckSlots;
use rpgkit_rules::unit::GEAR_EFFECT_SOURCE;
use rpgkit_rules::{build_from, build_unit, build_unit_in, damage, DeckRule, Growth, Unit, UnitBuild};
use serde_json::{json, Value};

const STR_BONUS: i32 = 3;
const CRIT: f64 = 4.0;
const BURDEN_OFF: i32 = 1;

fn skill(id: &str) -> Value {
    json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"})
}

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let class = |id: &str, prof: Value| json!({"id": id, "name": id, "base_stats": stats, "growth": {}, "proficiency": prof, "skill_path": {"slash": 1}});
    let skill_fx = |s: &str| json!([{"type": "skill", "skill": s}]);
    Catalog::from_value(&json!({
        "classes": [
            class("adept", json!({"sword": {"level": 2}, "leather": {"level": 2}, "staff": {"level": 1}, "orb": {"level": 1}, "dagger": {"level": 1}})),
            class("master", json!({"sword": {"level": 3}, "leather": {"level": 1}})),
            class("novice", json!({"sword": {"level": 1}, "leather": {"level": 1}}))
        ],
        "units": [
            {"id": "adept", "name": "Adept", "class_id": "adept", "level": 1, "side": "player"},
            {"id": "master", "name": "Master", "class_id": "master", "level": 1, "side": "player"},
            {"id": "novice", "name": "Novice", "class_id": "novice", "level": 1, "side": "player"}
        ],
        "skills": [skill("slash"), skill("flourish"), skill("parry_jab"), skill("orb_bolt")],
        "passives": [{"id": "sword_mastery", "name": "Sword Mastery", "slot": "", "effects": [{"type": "stat", "stat": "DEX", "value": 2}, {"type": "skill", "skill": "flourish"}]}],
        "items": [
            {"id": "plain_sword", "name": "Plain Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10},
            {"id": "show_sword", "name": "Show Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "effects": skill_fx("flourish")},
            {"id": "strong_belt", "name": "Strong Belt", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "effects": [{"type": "stat", "stat": "STR", "value": STR_BONUS}]},
            {"id": "drill_ring", "name": "Drill Ring", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "effects": [{"type": "proficiency", "gear": "axe", "levels": 2}]},
            {"id": "parry_dagger", "name": "Parry Dagger", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "dagger", "base_power": 6, "effects": skill_fx("parry_jab")},
            {"id": "great_staff", "name": "Great Staff", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "staff", "two_handed": true, "base_power": 20},
            {"id": "seer_orb", "name": "Seer Orb", "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": "orb", "off_hand_category": "orb", "family_rules": ["focus"], "effects": skill_fx("orb_bolt")},
            {"id": "jerkin", "name": "Jerkin", "item_type": "ARMOR", "slot": "BODY", "weapon_type": "leather", "mitigation": 5, "burden": 2},
            {"id": "jerkin_cap", "name": "Jerkin Cap", "item_type": "ARMOR", "slot": "HEAD", "weapon_type": "leather", "mitigation": 2, "burden": 1}
        ]
    }))
}

/// `item` with its family's (test-only) Level-2 bonus and Level-3 passive on the resolved row, as the loader writes them.
fn with_family(cat: &Catalog, item: &str, level2: Value, level3: &str) -> ItemDef {
    let mut raw = cat.items[item].raw.clone();
    raw.insert(ITEM_FAMILY_LEVEL2.into(), level2);
    raw.insert(ITEM_FAMILY_LEVEL3.into(), json!(level3));
    ItemDef::from_obj(&raw)
}

fn wear(cat: &Catalog, u: &mut Unit, item: &ItemDef, slot: Slot) {
    u.inventory.equip(Equipment::new(item), slot);
    u.sync_from_stats_and_gear(cat);
}

fn take_off(cat: &Catalog, u: &mut Unit, slot: Slot) {
    u.inventory.unequip(slot);
    u.sync_from_stats_and_gear(cat);
}

fn unit(cat: &Catalog, id: &str) -> Unit {
    let mut u = build_unit(cat, &cat.units[id], 0);
    for slot in Slot::ALL {
        u.inventory.unequip(slot);
    }
    u.sync_from_stats_and_gear(cat);
    u
}

#[test]
fn a_worn_skill_is_open_while_worn_with_no_slot_and_goes_with_the_item() {
    let cat = catalog();
    let mut u = unit(&cat, "novice");
    let flourish = cat.skill("flourish").unwrap().clone();
    assert!(!u.owns_skill(&cat, "flourish") && !u.can_use(&cat, &flourish), "not owned unworn");
    u.set_loadout(&cat, vec!["slash".into()], vec![]).unwrap();
    wear(&cat, &mut u, &cat.items["show_sword"].clone(), Slot::MainHand);
    assert_eq!(u.gear_skills, ["flourish"]);
    assert!(u.owns_skill(&cat, "flourish") && u.always_active(&cat, "flourish") && u.can_use(&cat, &flourish), "open while worn");
    assert_eq!(u.active_families(), ["slash"], "it takes no loadout slot");
    assert!(!u.owned_families(&cat).contains(&"flourish".to_string()), "nor can a loadout slot it");
    assert!(u.granted_skills.is_empty() && !u.known_skills.contains(&"flourish".to_string()), "nothing kept for good");
    for _ in 0..3 {
        u.sync_from_stats_and_gear(&cat);
    }
    assert_eq!(u.gear_skills, ["flourish"], "a sync rebuilds it once");
    take_off(&cat, &mut u, Slot::MainHand);
    assert!(u.gear_skills.is_empty() && !u.owns_skill(&cat, "flourish") && !u.can_use(&cat, &flourish), "gone with the item");
}

#[test]
fn under_the_deck_rule_a_worn_skill_takes_no_active_slot() {
    let cat = catalog();
    let rule = DeckRule { basics: Vec::new(), slots: DeckSlots { active: 1, passive: 0, keystone: 0 } };
    let mut u = build_unit_in(&cat, &cat.units["novice"], 0, Some(&rule));
    u.deck.active = vec!["flourish".into(), "slash".into()];
    wear(&cat, &mut u, &cat.items["show_sword"].clone(), Slot::MainHand);
    u.apply_deck(&cat);
    assert_eq!(u.active_families(), ["slash"], "the deck's worn-only family is skipped; the one slot holds slash");
    assert!(u.can_use(&cat, cat.skill("flourish").unwrap()), "open while worn");
    take_off(&cat, &mut u, Slot::MainHand);
    assert!(!u.can_use(&cat, cat.skill("flourish").unwrap()));
}

#[test]
fn a_worn_stat_effect_is_rebuilt_every_sync_and_taken_off() {
    let cat = catalog();
    let mut u = unit(&cat, "novice");
    let base = u.stats.stat("STR", None);
    wear(&cat, &mut u, &cat.items["strong_belt"].clone(), Slot::Accessory1);
    for _ in 0..3 {
        u.sync_from_stats_and_gear(&cat);
        assert_eq!(u.stats.stat("STR", None), base + STR_BONUS, "no stacking over syncs");
    }
    assert!(u.stats.modifiers.iter().any(|m| m.source == GEAR_EFFECT_SOURCE));
    take_off(&cat, &mut u, Slot::Accessory1);
    assert_eq!(u.stats.stat("STR", None), base);
    assert!(!u.stats.modifiers.iter().any(|m| m.source == GEAR_EFFECT_SOURCE));
}

#[test]
fn an_item_proficiency_effect_trains_nothing() {
    let cat = catalog();
    let mut u = unit(&cat, "novice");
    wear(&cat, &mut u, &cat.items["drill_ring"].clone(), Slot::Accessory1);
    assert_eq!(u.stats.proficiency.level_of("axe"), 0);
}

/// The built hand rule: a focus is not held under a two-handed main weapon, so its skill is not granted; worn alone it
/// is.
#[test]
fn the_hand_rule_decides_which_worn_skills_hold() {
    let cat = catalog();
    let (staff, orb) = (cat.items["great_staff"].clone(), cat.items["seer_orb"].clone());
    let growth = |equipment: Vec<(Slot, ItemDef)>| Growth { equipment, ..Growth::default() };
    let both = build_from(&cat, UnitBuild { growth: Some(growth(vec![(Slot::MainHand, staff.clone()), (Slot::OffHand, orb.clone())])), ..UnitBuild::row(&cat.units["adept"], 0, None) }).unit;
    assert!(both.inventory.get(Slot::OffHand).is_none() && !both.gear_skill("orb_bolt"), "the orb is not held under the staff");
    let alone = build_from(&cat, UnitBuild { growth: Some(growth(vec![(Slot::OffHand, orb)])), ..UnitBuild::row(&cat.units["adept"], 0, None) }).unit;
    assert!(alone.gear_skill("orb_bolt"), "held, it grants its skill");
    // an off-hand weapon held beside a two-handed main (the weapon switch) is worn: its skill holds whichever is ready
    let mut u = unit(&cat, "adept");
    u.inventory.equip(Equipment::new(&staff), Slot::MainHand);
    wear(&cat, &mut u, &cat.items["parry_dagger"].clone(), Slot::OffHand);
    assert!(u.inventory.needs_weapon_switch() && u.gear_skill("parry_jab"));
    u.inventory.toggle_active_weapon();
    u.sync_from_stats_and_gear(&cat);
    assert!(u.gear_skill("parry_jab"));
}

#[test]
fn the_level_2_crit_counts_from_proficiency_2_once_per_family() {
    let cat = catalog();
    let sword = with_family(&cat, "plain_sword", json!({"crit": CRIT}), "");
    let plain = with_family(&cat, "plain_sword", json!({}), "");
    for (who, level) in [("novice", 1), ("adept", 2), ("master", 3)] {
        let mut with = unit(&cat, who);
        wear(&cat, &mut with, &sword, Slot::MainHand);
        let mut without = unit(&cat, who);
        wear(&cat, &mut without, &plain, Slot::MainHand);
        let want = if level >= 2 { CRIT } else { 0.0 };
        assert_eq!(with.gear_crit, want, "{who}");
        assert_eq!(damage::crit_chance(&with), damage::crit_chance(&without) + want, "{who}");
    }
    let mut u = unit(&cat, "adept");
    wear(&cat, &mut u, &sword, Slot::MainHand);
    wear(&cat, &mut u, &sword, Slot::OffHand);
    assert_eq!(u.gear_crit, CRIT, "one family counts once");
    take_off(&cat, &mut u, Slot::MainHand);
    take_off(&cat, &mut u, Slot::OffHand);
    assert_eq!(u.gear_crit, 0.0, "taken off with the item");
}

#[test]
fn the_level_2_burden_comes_off_each_item_of_the_family_from_proficiency_2() {
    let cat = catalog();
    let (body, head) = (with_family(&cat, "jerkin", json!({"burden": BURDEN_OFF}), ""), with_family(&cat, "jerkin_cap", json!({"burden": BURDEN_OFF}), ""));
    let (plain_body, plain_head) = (cat.items["jerkin"].clone(), cat.items["jerkin_cap"].clone());
    for (who, level) in [("novice", 1), ("adept", 2)] {
        let mut with = unit(&cat, who);
        wear(&cat, &mut with, &body, Slot::Body);
        wear(&cat, &mut with, &head, Slot::Head);
        let mut without = unit(&cat, who);
        wear(&cat, &mut without, &plain_body, Slot::Body);
        wear(&cat, &mut without, &plain_head, Slot::Head);
        let r = without.stats.proficiency.burden_reduction("leather");
        let want = if level >= 2 { (2 - r - BURDEN_OFF).max(0) + (1 - r - BURDEN_OFF).max(0) } else { without.worn_burden() };
        assert_eq!(with.worn_burden(), want, "{who}: per item, to 0 at least");
    }
}

#[test]
fn the_level_3_passive_holds_at_proficiency_3_while_worn() {
    let cat = catalog();
    let sword = with_family(&cat, "plain_sword", json!({}), "sword_mastery");
    for (who, master) in [("adept", false), ("master", true)] {
        let mut u = unit(&cat, who);
        let dex = u.stats.stat("DEX", None);
        wear(&cat, &mut u, &sword, Slot::MainHand);
        assert_eq!(u.gear_skill("flourish"), master, "{who}");
        assert_eq!(u.stats.stat("DEX", None), dex + if master { 2 } else { 0 }, "{who}");
        assert!(!u.passives.contains(&"sword_mastery".to_string()), "applied as the item's, not kept as a passive");
        take_off(&cat, &mut u, Slot::MainHand);
        assert!(!u.gear_skill("flourish") && u.stats.stat("DEX", None) == dex, "{who}: taken off with the item");
    }
}

#[test]
fn a_row_without_family_bonuses_reads_none() {
    let cat = catalog();
    let item = &cat.items["plain_sword"];
    assert!(item.level2_bonus.is_empty() && item.level3_passive.is_empty());
    let sword = with_family(&cat, "plain_sword", json!({"burden": 2}), "sword_mastery");
    assert_eq!((sword.level2_bonus.crit, sword.level2_bonus.burden, sword.level3_passive.as_str()), (0.0, 2, "sword_mastery"));
}
