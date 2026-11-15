//! GEAR-REWORK1 (PR5-51a, `docs/sessions/U3_GEAR_REWORK.md` 1, 5) on a scratch catalog built here (no shipped data; the
//! item numbers are test-only). The strict gear rule is gated on the class: a class that tags any proficiency entry
//! with a `role` (`ClassDef::proficiency_roles`) is strict (`Proficiency::is_strict`), one without roles keeps the soft
//! U3.2 wield penalty. Under the rule:
//!
//! - proficiency 0 and raw `required_stats` above the unscaled stat plus the requirement discount are a hard equip lock
//!   (`gear::wear_lock`); accessories need no proficiency (their requirements still lock);
//! - proficiency 1 / 2 / 3 scale weapon power and flat mitigation by `rpgkit_rules::stats::PROF_GEAR_MULT` (0.80 / 1.00 / 1.25).
//!
//! The two classes are identical but for the roles, so every strict number is derived from the soft unit's.

use indexmap::IndexMap;
use rpgkit_data::Catalog;
use rpgkit_rules::damage::attack_power;
use rpgkit_rules::gear::{wear_lock, Equipment, Slot, WearLock};
use rpgkit_rules::stats::{CharacterStats, Proficiency, PROF_GEAR_MULT};
use rpgkit_rules::{build_unit, Unit};
use serde_json::{json, Value};

/// The class rows' shared proficiency, with or without roles.
fn proficiency(roles: bool) -> Value {
    let entry = |level: i32, role: &str| if roles { json!({"level": level, "role": role}) } else { json!({"level": level}) };
    json!({
        "blade": entry(3, "primary"), "edge": entry(2, "secondary"), "club": entry(1, "minor"),
        "plate": entry(3, "main"), "mail": entry(2, "third"), "hide": entry(1, "minor")
    })
}

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let weapon = |id: &str, family: &str| json!({"id": id, "name": id, "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": family, "base_power": 20});
    let armour = |id: &str, family: &str| json!({"id": id, "name": id, "item_type": "ARMOR", "slot": "BODY", "weapon_type": family, "mitigation": 9});
    Catalog::from_value(&json!({
        "classes": [
            {"id": "soft", "name": "soft", "base_stats": stats, "growth": {}, "proficiency": proficiency(false), "skill_path": {}},
            {"id": "strict", "name": "strict", "base_stats": stats, "growth": {}, "proficiency": proficiency(true), "skill_path": {}}
        ],
        "units": [
            {"id": "soft", "name": "Soft", "class_id": "soft", "level": 1, "side": "player"},
            {"id": "strict", "name": "Strict", "class_id": "strict", "level": 1, "side": "player"}
        ],
        "items": [
            weapon("blade", "blade"), weapon("edge", "edge"), weapon("club", "club"), weapon("axe", "axe"),
            armour("plate", "plate"), armour("mail", "mail"), armour("hide", "hide"), armour("silk", "silk"),
            {"id": "heavy_blade", "name": "Heavy Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "blade", "base_power": 20, "required_stats": {"STR": 20}},
            {"id": "fair_blade", "name": "Fair Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "blade", "base_power": 20, "required_stats": {"STR": 18}},
            {"id": "ring", "name": "Ring", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "weapon_type": "accessory", "stat_bonuses": {"DEX": 1}},
            {"id": "relic", "name": "Relic", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "weapon_type": "accessory", "required_stats": {"KNOW": 14}}
        ]
    }))
}

fn bare(cat: &Catalog, id: &str) -> Unit {
    let mut u = build_unit(cat, &cat.units[id], 0);
    for slot in Slot::ALL {
        u.inventory.unequip(slot);
    }
    u.sync_from_stats_and_gear(cat);
    u
}

fn wearing(cat: &Catalog, id: &str, item: &str, slot: Slot) -> Unit {
    let mut u = bare(cat, id);
    u.inventory.equip(Equipment::new(&cat.items[item]), slot);
    u.sync_from_stats_and_gear(cat);
    u
}

fn stats(cat: &Catalog, class: &str) -> CharacterStats {
    CharacterStats::new(&cat.classes[class], 1, &IndexMap::new())
}

#[test]
fn roles_turn_the_strict_rule_on_and_an_alias_reads_as_its_role() {
    let cat = catalog();
    let (soft, strict) = (Proficiency::from_class(&cat.classes["soft"]), Proficiency::from_class(&cat.classes["strict"]));
    assert!(!soft.is_strict() && strict.is_strict());
    assert_eq!(cat.classes["strict"].proficiency_roles.get("blade").map(String::as_str), Some("main"), "primary is main");
    let mut loose = strict.clone();
    loose.unrestricted = true;
    assert!(!loose.is_strict(), "a classless unit (unrestricted) stays unrestricted");
    assert!(!Proficiency { strict: true, ..Proficiency::default() }.is_strict(), "an empty proficiency map stays unrestricted");
}

#[test]
fn proficiency_0_is_a_hard_lock_only_under_the_strict_rule() {
    let cat = catalog();
    let (soft, strict) = (stats(&cat, "soft"), stats(&cat, "strict"));
    for item in ["axe", "silk"] {
        let family = cat.items[item].weapon_type.clone();
        assert_eq!(wear_lock(&strict, &cat.items[item]), Some(WearLock::Untrained { gear: family }), "{item}");
        assert_eq!(wear_lock(&soft, &cat.items[item]), None, "{item}: the soft rule never locks");
    }
    for item in ["blade", "edge", "club", "plate", "mail", "hide", "ring"] {
        assert_eq!(wear_lock(&strict, &cat.items[item]), None, "{item}: trained (or an accessory)");
    }
}

#[test]
fn unmet_raw_requirements_lock_after_the_proficiency_discount() {
    let cat = catalog();
    let (soft, strict) = (stats(&cat, "soft"), stats(&cat, "strict"));
    let have = strict.requirement_base("STR") + strict.proficiency.requirement_discount("blade");
    let need = |id: &str| cat.items[id].required_stats["STR"];
    assert!(need("heavy_blade") > have && need("fair_blade") <= have, "the fixture straddles the discount");
    assert_eq!(wear_lock(&strict, &cat.items["heavy_blade"]), Some(WearLock::Unfit { stat: "STR".into(), need: need("heavy_blade"), have }));
    assert_eq!(wear_lock(&strict, &cat.items["fair_blade"]), None);
    assert_eq!(wear_lock(&soft, &cat.items["heavy_blade"]), None, "the soft rule never locks");
    // an accessory needs no proficiency but its requirement locks (no discount)
    let know = strict.requirement_base("KNOW");
    assert_eq!(wear_lock(&strict, &cat.items["relic"]), (cat.items["relic"].required_stats["KNOW"] > know).then(|| WearLock::Unfit { stat: "KNOW".into(), need: 14, have: know }));
}

#[test]
fn proficiency_1_2_3_scale_weapon_power_by_the_table() {
    let cat = catalog();
    for item in ["blade", "edge", "club"] {
        let (soft, strict) = (wearing(&cat, "soft", item, Slot::MainHand), wearing(&cat, "strict", item, Slot::MainHand));
        let level = strict.stats.proficiency.level_of(item);
        let mult = PROF_GEAR_MULT[level as usize];
        let (a, b) = (attack_power(&soft, false, false), attack_power(&strict, false, false));
        assert!((b.weapon_power - a.weapon_power * mult).abs() < 1e-9, "{item} (proficiency {level}): {} x {mult} = {}", a.weapon_power, b.weapon_power);
        assert_eq!(a.prof_mult, 0.0, "{item}: no factor without the rule");
        assert_eq!(b.prof_mult, if mult == 1.0 { 0.0 } else { mult }, "{item}: the factor is shown");
    }
    assert_eq!(PROF_GEAR_MULT[1..], [0.80, 1.00, 1.25], "the spec's table");
}

#[test]
fn proficiency_1_2_3_scale_flat_mitigation_by_the_table() {
    let cat = catalog();
    for item in ["plate", "mail", "hide"] {
        let (soft, strict) = (wearing(&cat, "soft", item, Slot::Body), wearing(&cat, "strict", item, Slot::Body));
        let mult = PROF_GEAR_MULT[strict.stats.proficiency.level_of(item) as usize];
        let soft_mit = soft.inventory.total_flat_mitigation(Some(&soft.stats));
        assert_eq!(strict.inventory.total_flat_mitigation(Some(&strict.stats)), (soft_mit as f64 * mult).round() as i32, "{item} x {mult}");
    }
    let ring = (wearing(&cat, "soft", "ring", Slot::Accessory1), wearing(&cat, "strict", "ring", Slot::Accessory1));
    assert_eq!(ring.0.stats.stat("DEX", None), ring.1.stats.stat("DEX", None), "an accessory is proficiency-free");
}

/// With the gate off (a class without roles, e.g. fixture `tl_drill`) the soft untrained penalty is retired (PR5-57):
/// an untrained item is worn without an untrained penalty, and no proficiency power bonus or scaling applies.
#[test]
fn with_the_gate_off_an_untrained_item_is_worn_without_soft_penalty() {
    let cat = catalog();
    let u = wearing(&cat, "soft", "axe", Slot::MainHand);
    let axe = u.inventory.main_weapon().unwrap();
    let pen = axe.wield_penalty(&u.stats);
    assert!(!pen.penalized && pen.factor() == 1.0, "soft untrained penalty retired");
    assert_eq!(axe.prof_mult(&u.stats), 1.0);
    let ap = attack_power(&u, false, false);
    assert_eq!(ap.prof_mult, 0.0);
    assert!((ap.weapon_power - ap.total_w_power as f64).abs() < 1e-9, "no untrained reduction");
}
