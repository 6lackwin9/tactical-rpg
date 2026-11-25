//! GEAR-REWORK1 (PR5-51a, `docs/sessions/U3_GEAR_REWORK.md` 1) on a scratch catalog built here (no shipped data; the
//! item numbers are test-only): `Switch Weapon` and the AUDIT-01/08/09 stowed backup weapon are retired. A two-handed
//! main weapon leaves the off hand empty (`gear::off_hand_ok`, a build drops what the off hand held), every worn item
//! counts (stat and tier bonuses, armour, burden), and a main-hand hit always swings the main hand.
//!
//! Expectations are derived from units that wear one weapon at a time (no hard-coded derived numbers).

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

/// GEAR-REWORK1: under a two-handed main weapon the off hand holds nothing: the hand rule refuses the sling there and a
/// build that names both drops the sling; a one-handed main weapon pairs freely.
#[test]
fn a_two_handed_main_weapon_leaves_the_off_hand_empty() {
    use rpgkit_rules::gear::off_hand_ok;
    use rpgkit_rules::{build_from, Growth, UnitBuild};
    let cat = catalog();
    let (pike, sling) = (&cat.items["pike"], &cat.items["sling"]);
    assert!(!off_hand_ok(Some(pike), sling), "no off-hand weapon under a 2H main");
    assert!(off_hand_ok(Some(sling), pike) && off_hand_ok(None, sling), "a 1H main or none pairs freely");
    let growth = Growth { equipment: vec![(Slot::MainHand, pike.clone()), (Slot::OffHand, sling.clone())], ..Growth::default() };
    let built = build_from(&cat, UnitBuild { growth: Some(growth), ..UnitBuild::row(&cat.units["ranger"], 0, None) }).unit;
    let solo = wearing(&cat, &[("pike", Slot::MainHand)]);
    assert!(built.inventory.get(Slot::OffHand).is_none(), "the build drops the sling");
    assert_eq!((stat_line(&built), mitigation(&built), built.worn_burden()), (stat_line(&solo), mitigation(&solo), solo.worn_burden()));
}

/// GEAR-REWORK1: no stowed weapon: an off-hand weapon counts as worn in full (its bonuses, tier bonuses, armour and
/// burden are the sling-in-the-main-hand unit's), and nothing lends the main hand its type, range or reach.
#[test]
fn a_worn_off_hand_weapon_counts_in_full() {
    let cat = catalog();
    let sling = wearing(&cat, &[("sling", Slot::MainHand)]);
    let none = bare(&cat);
    let u = wearing(&cat, &[("sling", Slot::OffHand)]);
    assert_eq!((stat_line(&u), mitigation(&u), u.worn_burden()), (stat_line(&sling), mitigation(&sling), sling.worn_burden()));
    assert_eq!((u.weapon_context(), u.attack_range, u.wielded_type("main")), (none.weapon_context(), none.attack_range, none.wielded_type("main")));
    assert_eq!(u.wielded_type("off"), "sling");
}

/// PR5-26 (audit B7), GEAR-REWORK1: a main-hand hit swings the main hand's weapon (`damage::weapon_for(u, false)`), an
/// off-hand hit the off hand's; nothing switches.
#[test]
fn a_main_hand_hit_swings_the_main_hand() {
    use rpgkit_rules::damage::weapon_for;
    let cat = catalog();
    let u = wearing(&cat, &[("sling", Slot::MainHand), ("pike", Slot::OffHand)]);
    let id = |u: &Unit, off: bool| weapon_for(u, off).map(|e| e.item.id.clone());
    assert_eq!((id(&u, false).as_deref(), id(&u, true).as_deref()), (Some("sling"), Some("pike")));
    assert_eq!(u.wielded_type("main"), "sling");
}
