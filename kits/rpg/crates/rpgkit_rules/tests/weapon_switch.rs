//! AUDIT-08 and AUDIT-09 (READY.md R1.1-1, FB-91) on a scratch catalog built here (no shipped data; the item numbers
//! are test-only). In a two-handed main + off-hand weapon pair (`Inventory::needs_weapon_switch`) only the ready weapon
//! counts after a sync:
//!
//! - AUDIT-08: the stowed weapon adds no flat `stat_bonuses`, no tier `stat_bonuses` and no armour
//!   (`Inventory::total_flat_mitigation`); its burden still counts;
//! - AUDIT-09: the weapon type (`Unit::weapon_context`, `Unit::wielded_type("main")`), the attack range and the reach
//!   gear fact (`Unit::has_reach_weapon`) are the ready weapon's, so switching to the backup uses its type and range.
//!
//! Expectations are derived from units that wear only the ready weapon (no hard-coded derived numbers).

use rpgkit_data::{Catalog, ItemDef};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::{build_unit, Unit};
use serde_json::json;

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let level1 = json!({"type": "unit_level_at_least", "value": 1});
    Catalog::from_value(&json!({
        "classes": [
            {"id": "ranger", "name": "ranger", "base_stats": stats, "growth": {}, "proficiency": {"spear": {"level": 1}, "sling": {"level": 1}}, "skill_path": {"jab": 1}}
        ],
        "units": [{"id": "ranger", "name": "Ranger", "class_id": "ranger", "level": 1, "side": "player"}],
        "skills": [{"id": "jab", "name": "jab", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"}],
        "items": [
            {"id": "pike", "name": "Pike", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "spear", "two_handed": true, "base_power": 14,
             "family_rules": ["reach"], "burden": 2, "mitigation": 3, "stat_bonuses": {"STR": 2},
             "threshold_tiers": [{"when": level1, "bonus": {"stat_bonuses": {"CONS": 1}, "mitigation": 1}}]},
            {"id": "sling", "name": "Sling", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sling", "base_power": 6, "base_range": 3,
             "burden": 1, "mitigation": 1, "stat_bonuses": {"DEX": 2},
             "threshold_tiers": [{"when": level1, "bonus": {"stat_bonuses": {"PERC": 1}, "mitigation": 2}}]}
        ]
    }))
}

fn bare(cat: &Catalog) -> Unit {
    let mut u = build_unit(cat, &cat.units["ranger"], 0);
    for slot in Slot::ALL {
        u.inventory.unequip(slot);
    }
    u.sync_from_stats_and_gear(cat);
    u
}

fn wearing(cat: &Catalog, items: &[(&str, Slot)]) -> Unit {
    let mut u = bare(cat);
    for (id, slot) in items {
        let item: &ItemDef = &cat.items[*id];
        u.inventory.equip(Equipment::new(item), *slot);
    }
    u.sync_from_stats_and_gear(cat);
    u
}

const STATS: [&str; 4] = ["STR", "DEX", "CONS", "PERC"];

fn stat_line(u: &Unit) -> Vec<i32> {
    STATS.iter().map(|s| u.stats.stat(s, None)).collect()
}

fn mitigation(u: &Unit) -> i32 {
    u.inventory.total_flat_mitigation(Some(&u.stats))
}

/// The pair `u` counts exactly what `solo` (wearing only the ready weapon) counts, plus the stowed weapon's burden.
fn assert_counts_only(u: &Unit, solo: &Unit, stowed: &Unit, label: &str) {
    assert_eq!(stat_line(u), stat_line(solo), "{label}: flat and tier stat bonuses are the ready weapon's");
    assert_eq!(mitigation(u), mitigation(solo), "{label}: armour is the ready weapon's");
    assert_eq!(u.weapon_context(), solo.weapon_context(), "{label}: weapon context");
    assert_eq!(u.wielded_type("main"), solo.wielded_type("main"), "{label}: wielded type");
    assert_eq!(u.wielded_type("off"), "unarmed", "{label}: a 2H pair shares one ready slot");
    assert_eq!(u.attack_range, solo.attack_range, "{label}: range");
    assert_eq!(u.has_reach_weapon(), solo.has_reach_weapon(), "{label}: reach");
    assert_eq!(u.worn_burden(), solo.worn_burden() + stowed.worn_burden(), "{label}: the stowed weapon's burden still counts");
}

/// The scratch items do add what the test checks for (so the pair assertions are not vacuous).
#[test]
fn each_weapon_alone_adds_its_bonuses_type_range_and_reach() {
    let cat = catalog();
    let none = bare(&cat);
    let pike = wearing(&cat, &[("pike", Slot::MainHand)]);
    let sling = wearing(&cat, &[("sling", Slot::MainHand)]);
    let (n, p, s) = (stat_line(&none), stat_line(&pike), stat_line(&sling));
    assert!(p[0] > n[0] && p[2] > n[2] && p[1] == n[1] && p[3] == n[3], "pike: flat STR and tier CONS");
    assert!(s[1] > n[1] && s[3] > n[3] && s[0] == n[0] && s[2] == n[2], "sling: flat DEX and tier PERC");
    assert!(mitigation(&pike) > cat.items["pike"].mitigation && mitigation(&sling) > cat.items["sling"].mitigation, "tier armour counts");
    assert!(pike.has_reach_weapon() && pike.attack_range == 1 && pike.wielded_type("main") == "spear");
    assert!(!sling.has_reach_weapon() && sling.attack_range > 1 && sling.wielded_type("main") == "sling");
}

/// AUDIT-08/09: the stowed weapon of a 2H pair grants no stat bonus, tier stat bonus or armour and lends no type,
/// range or reach; the switch (and a resync) moves all of it to the other hand and back; without a pair the off-hand
/// weapon counts as worn.
#[test]
fn a_stowed_backup_weapon_adds_only_its_burden_and_the_switch_moves_type_range_and_reach() {
    let cat = catalog();
    let pike = wearing(&cat, &[("pike", Slot::MainHand)]);
    let sling = wearing(&cat, &[("sling", Slot::MainHand)]);
    let mut u = wearing(&cat, &[("pike", Slot::MainHand), ("sling", Slot::OffHand)]);
    assert!(u.inventory.needs_weapon_switch() && !u.inventory.is_offhand_active());
    assert_counts_only(&u, &pike, &sling, "pike ready");
    assert!(u.inventory.toggle_active_weapon());
    u.sync_from_stats_and_gear(&cat);
    assert!(u.inventory.is_offhand_active());
    assert_counts_only(&u, &sling, &pike, "sling ready");
    assert!(u.inventory.toggle_active_weapon());
    u.sync_from_stats_and_gear(&cat);
    assert_counts_only(&u, &pike, &sling, "switched back");
    u.inventory.unequip(Slot::MainHand);
    u.sync_from_stats_and_gear(&cat);
    let off_only = wearing(&cat, &[("sling", Slot::OffHand)]);
    assert!(!u.inventory.needs_weapon_switch());
    assert_eq!(stat_line(&u), stat_line(&sling), "no pair: the off-hand sling's bonuses count");
    assert_eq!(mitigation(&u), mitigation(&sling));
    assert_eq!((stat_line(&u), mitigation(&u), u.worn_burden()), (stat_line(&off_only), mitigation(&off_only), off_only.worn_burden()));
}
