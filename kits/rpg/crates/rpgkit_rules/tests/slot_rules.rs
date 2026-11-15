//! GEAR-REWORK1 (PR5-51a pass 2, `docs/sessions/U3_GEAR_REWORK.md` 2-4) on a scratch catalog built here (no shipped
//! data; resolved item rows carry their family's keys as the loader writes them). Expectations come from the rules:
//!
//! - a family's hand category (`slot_rule`) fixes the hands its items fill: `1h_light` either, `1h_heavy` and the
//!   two-handed ones the main hand, `off_only` the off hand; an item without one keeps the pre-rework slot rule;
//! - a two-handed main weapon leaves the off hand empty unless its family `pairs_with` the off-hand item's family;
//! - a `1h_light` off-hand weapon's jab reaches as the weapon does; the Off-Hand button's use reads the family fields;
//! - an off-hand item's `power_stat_override` replaces a fitting main-hand weapon's scaling stat only when higher.

use rpgkit_data::{Catalog, ItemDef};
use rpgkit_rules::damage::attack_power;
use rpgkit_rules::gear::{item_fits_slot, light_reach, off_hand_ok, off_hand_use, Equipment, OffHandUse, Slot};
use rpgkit_rules::{build_unit, Unit};
use serde_json::{json, Value};

const SLOTS: [&str; 2] = ["MAIN_HAND", "OFF_HAND"];

fn row(id: &str, family: &str, extra: Value) -> Value {
    let mut r = json!({"id": id, "name": id, "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": family, "base_power": 20});
    for (k, v) in extra.as_object().unwrap() {
        r[k] = v.clone();
    }
    r
}

fn item(id: &str, family: &str, extra: Value) -> ItemDef {
    ItemDef::from_obj(row(id, family, extra).as_object().unwrap())
}

fn rule(r: &str) -> Value {
    json!({"family_slot_rule": r})
}

fn bow() -> ItemDef {
    item("bow", "bow", json!({"two_handed": true, "family_slot_rule": "2h_rig", "family_pairs_with": ["ammo", "cloak"], "family_rules": ["always_ranged"]}))
}

fn off(id: &str, family: &str, extra: Value) -> ItemDef {
    let mut e = json!({"slot": "OFF_HAND", "family_slot_rule": "off_only", "off_hand_category": family});
    for (k, v) in extra.as_object().unwrap() {
        e[k] = v.clone();
    }
    item(id, family, e)
}

#[test]
fn each_hand_category_fills_its_own_hands() {
    let cases: [(&str, bool, &str, [bool; 2]); 5] = [("2h_solo", true, "MAIN_HAND", [true, false]), ("2h_rig", true, "MAIN_HAND", [true, false]), ("1h_heavy", false, "MAIN_HAND", [true, false]), ("1h_light", false, "MAIN_HAND", [true, true]), ("off_only", false, "OFF_HAND", [false, true])];
    for (r, two, slot, fits) in cases {
        let i = item(r, "fam", json!({"family_slot_rule": r, "two_handed": two, "slot": slot}));
        for (k, s) in SLOTS.iter().enumerate() {
            assert_eq!(item_fits_slot(&i, s), fits[k], "{r} in {s}");
        }
        assert!(!item_fits_slot(&i, "HEAD"), "{r} is no helm");
    }
}

#[test]
fn without_a_hand_category_the_pre_rework_slot_rule_holds() {
    let one = item("sword", "sword", json!({}));
    let two = item("maul", "hammer", json!({"two_handed": true}));
    let focus = item("orb", "orb", json!({"slot": "OFF_HAND", "off_hand_category": "orb"}));
    assert!(item_fits_slot(&one, "MAIN_HAND") && item_fits_slot(&one, "OFF_HAND"), "a one-handed main weapon in either hand");
    assert!(item_fits_slot(&two, "MAIN_HAND") && !item_fits_slot(&two, "OFF_HAND"));
    assert!(!item_fits_slot(&focus, "MAIN_HAND") && item_fits_slot(&focus, "OFF_HAND"));
    assert!(!off_hand_ok(Some(&two), &focus), "a two-handed main weapon keeps the off hand empty");
    assert!(off_hand_ok(Some(&one), &focus) && off_hand_ok(None, &focus));
    assert_eq!(light_reach(&one), None);
    assert_eq!(off_hand_use(&one), None, "no button for gear from before the rework");
    assert_eq!(off_hand_use(&focus), None);
}

#[test]
fn a_rig_weapon_shares_the_off_hand_only_with_the_families_it_pairs_with() {
    let (ammo, cloak, shield) = (off("quiver", "ammo", json!({})), off("mantle", "cloak", json!({})), off("buckler", "shield", json!({})));
    let solo = item("maul", "hammer", json!({"two_handed": true, "family_slot_rule": "2h_solo"}));
    let knife = item("knife", "knife", rule("1h_light"));
    assert!(off_hand_ok(Some(&bow()), &ammo) && off_hand_ok(Some(&bow()), &cloak), "2h_rig + pairs_with");
    assert!(!off_hand_ok(Some(&bow()), &shield), "not listed in pairs_with");
    assert!(!off_hand_ok(Some(&bow()), &knife), "a weapon is not listed either");
    assert!(!off_hand_ok(Some(&solo), &ammo), "2h_solo pairs with nothing");
    let heavy = item("saber", "sword", rule("1h_heavy"));
    for o in [&ammo, &cloak, &shield, &knife] {
        assert!(off_hand_ok(Some(&heavy), o), "a one-handed main pairs freely ({})", o.id);
    }
}

#[test]
fn a_light_off_hand_weapon_jabs_at_its_own_reach_and_off_hand_items_name_their_skill() {
    let knife = item("knife", "knife", json!({"family_slot_rule": "1h_light", "family_reach_max": 2}));
    let dagger = item("dagger", "dagger", rule("1h_light"));
    let heavy = item("saber", "sword", json!({"family_slot_rule": "1h_heavy", "family_reach_max": 2}));
    assert_eq!(light_reach(&knife), Some(2), "the family's range_band max");
    assert_eq!(light_reach(&dagger), Some(1), "no band: distance 1");
    assert_eq!(light_reach(&heavy), None, "1h_heavy is main hand only");
    assert_eq!(off_hand_use(&knife), Some(OffHandUse::Jab { reach: 2 }));
    let cloak = off("mantle", "cloak", json!({"family_off_hand_skill": "t_fade"}));
    let quiver = off("quiver", "ammo", json!({"family_off_hand_skill": "t_reload", "family_empower_next": {"meter": "t_ammo"}}));
    assert_eq!(off_hand_use(&cloak), Some(OffHandUse::Skill { skill: "t_fade".into(), reload: false }));
    assert_eq!(off_hand_use(&quiver), Some(OffHandUse::Skill { skill: "t_reload".into(), reload: true }));
    let e = quiver.empower_next.as_ref().expect("the rule");
    assert_eq!((e.cost, e.power_mult), (rpgkit_data::EmpowerNext::DEFAULT_COST, rpgkit_data::EmpowerNext::DEFAULT_POWER_MULT), "the defaults");
}

fn catalog() -> Catalog {
    let sage = json!({"STR": 10, "DEX": 10, "INT": 16, "CONS": 12, "PERC": 9, "KNOW": 10});
    let brute = json!({"STR": 16, "DEX": 10, "INT": 8, "CONS": 12, "PERC": 9, "KNOW": 10});
    Catalog::from_value(&json!({
        "classes": [
            {"id": "sage", "name": "sage", "base_stats": sage, "growth": {}, "skill_path": {}},
            {"id": "brute", "name": "brute", "base_stats": brute, "growth": {}, "skill_path": {}}
        ],
        "units": [
            {"id": "sage", "name": "Sage", "class_id": "sage", "level": 1, "side": "player"},
            {"id": "brute", "name": "Brute", "class_id": "brute", "level": 1, "side": "player"}
        ],
        "items": [
            row("saw", "bone_saw", json!({"family_slot_rule": "1h_heavy", "scaling_stat": "STR"})),
            row("wand", "wand", json!({"family_slot_rule": "1h_heavy", "scaling_stat": "INT"})),
            row("focus", "orb", json!({"slot": "OFF_HAND", "family_slot_rule": "off_only", "off_hand_category": "orb", "family_power_stat_override": "INT", "base_power": 0})),
            row("plain", "orb", json!({"slot": "OFF_HAND", "family_slot_rule": "off_only", "off_hand_category": "orb", "base_power": 0}))
        ]
    }))
}

fn armed(cat: &Catalog, unit: &str, main: &str, off: Option<&str>) -> Unit {
    let mut u = build_unit(cat, &cat.units[unit], 0);
    for slot in Slot::ALL {
        u.inventory.unequip(slot);
    }
    u.inventory.equip(Equipment::new(&cat.items[main]), Slot::MainHand);
    if let Some(o) = off {
        u.inventory.equip(Equipment::new(&cat.items[o]), Slot::OffHand);
    }
    u.sync_from_stats_and_gear(cat);
    u
}

#[test]
fn an_orb_lends_int_to_a_weapon_swing_only_when_higher() {
    let cat = catalog();
    let bare = attack_power(&armed(&cat, "sage", "saw", None), false, false);
    let lent = attack_power(&armed(&cat, "sage", "saw", Some("focus")), false, false);
    assert_eq!(bare.stat_name, "STR");
    assert!(!bare.stat_override);
    let u = armed(&cat, "sage", "saw", Some("focus"));
    assert!(u.stats.stat("INT", None) > u.stats.stat("STR", None), "the fixture: the sage's INT is higher");
    assert!(lent.stat_override && lent.stat_name == "INT", "{lent:?}");
    assert_eq!(lent.stat_val, u.stats.stat("INT", Some("bone_saw")));
    assert!(lent.attack_power > bare.attack_power, "INT {} over STR {}", lent.stat_val, bare.stat_val);
    // only when higher
    let brute = attack_power(&armed(&cat, "brute", "saw", Some("focus")), false, false);
    assert!(!brute.stat_override && brute.stat_name == "STR", "STR is higher: kept");
    // an orb without the field lends nothing; an INT weapon has nothing to replace
    let plain = attack_power(&armed(&cat, "sage", "saw", Some("plain")), false, false);
    assert!(!plain.stat_override && plain.attack_power == bare.attack_power);
    let wand = attack_power(&armed(&cat, "sage", "wand", Some("focus")), false, false);
    assert!(!wand.stat_override && wand.stat_name == "INT");
    // only on a fitting weapon swing: not the off-hand swing, not a melee weapon fired at range
    assert!(!attack_power(&u, false, true).stat_override, "the off-hand swing");
    assert!(!attack_power(&u, true, false).stat_override, "an unfit swing has no weapon power");
}
