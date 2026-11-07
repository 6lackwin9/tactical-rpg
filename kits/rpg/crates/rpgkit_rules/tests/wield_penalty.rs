//! U3.2-S3 (docs/sessions/U3_GEAR.md "Proficiency (0-3) and soft wield penalty", feasibility FB-33 and FB-34) on a scratch
//! catalog built here (no shipped data). Any unit wears any item; what it costs:
//!
//! - the stat-deficit ramp, kept as built and extended to armour: `WIELD_DEFICIT_STEP` of the item's weapon power and
//!   armour per missing requirement point (the largest, after the requirement discount `level x req_discount_per_level`),
//!   capped at `WIELD_DEFICIT_CAP`;
//! - proficiency 0 in the item's gear type: weapon power and armour x `untrained_power_mult`, plus `untrained_burden_add`
//!   burden (which lowers initiative like any burden);
//! - the two stack multiplicatively; trained levels keep their +level weapon power;
//! - accessories, gear with no type, a class with no proficiency list and a unit with no class wear free; unarmed power is
//!   as before (0 unless fist proficiency 2 or more, then the unarmed base plus the level); skill gates stay hard.
//!
//! The three terms are ruleset numbers (row-only), defaults from the spec table.

use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, UNARMED_BASE_POWER};
use rpgkit_rules::gear::{trained_for, Equipment, Slot, WieldRule, WIELD_DEFICIT_CAP, WIELD_DEFICIT_STEP};
use rpgkit_rules::ruleset::{self, Numbers};
use rpgkit_rules::stats::{roundi, REQ_DISCOUNT_PER_LEVEL, UNTRAINED_BURDEN_ADD, UNTRAINED_POWER_MULT};
use rpgkit_rules::{build_from, build_unit, Unit, UnitBuild};
use serde_json::json;

const POWER: i32 = 20;
const MITIGATION: i32 = 10;

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let unit = |id: &str, class: &str| json!({"id": id, "name": id, "class_id": class, "level": 1, "side": "player"});
    Catalog::from_value(&json!({
        "classes": [
            {"id": "drill", "name": "Drill", "base_stats": stats, "growth": {}, "proficiency": {"sword": {"level": 1}, "leather": {"level": 1}}},
            {"id": "axeman", "name": "Axeman", "base_stats": stats, "growth": {}, "proficiency": {"axe": {"level": 1}}},
            {"id": "free", "name": "Free", "base_stats": stats, "growth": {}},
            {"id": "brawler", "name": "Brawler", "base_stats": stats, "growth": {}, "proficiency": {"sword": {"level": 1}, "fist": {"level": 2}}},
            {"id": "novice", "name": "Novice", "base_stats": stats, "growth": {}, "proficiency": {"sword": {"level": 1}, "fist": {"level": 1}}}
        ],
        "units": [
            unit("probe", "drill"), unit("axer", "axeman"), unit("loose", "free"), unit("boxer", "brawler"), unit("rookie", "novice"),
            {"id": "nobody", "name": "Nobody", "class_id": "", "stat_mode": "fixed", "hp": 50, "speed": 8, "attack_range": 1, "side": "enemy", "main_hand": "axe", "body": "plate"}
        ],
        "items": [
            {"id": "sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": POWER},
            {"id": "axe", "name": "Axe", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "axe", "base_power": POWER},
            {"id": "club", "name": "Club", "item_type": "WEAPON", "slot": "MAIN_HAND", "base_power": POWER},
            {"id": "jerkin", "name": "Jerkin", "item_type": "ARMOR", "slot": "BODY", "weapon_type": "leather", "mitigation": MITIGATION, "burden": 1},
            {"id": "plate", "name": "Plate", "item_type": "ARMOR", "slot": "BODY", "weapon_type": "plate", "mitigation": MITIGATION, "burden": 1},
            {"id": "ring", "name": "Ring", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "weapon_type": "plate", "mitigation": 4}
        ],
        "skills": [{"id": "chop", "name": "Chop", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "min_proficiency": 1, "weapon_types": ["axe"]}]
    }))
}

/// Unit `unit` built under `wield`, wearing `items` (item id, slot, requirement over the wearer's stat + discount), synced.
fn wearing(cat: &Catalog, unit: &str, wield: WieldRule, items: &[(&str, Slot, Option<(&str, i32)>)]) -> Unit {
    let mut u = build_from(cat, UnitBuild { wield, ..UnitBuild::row(&cat.units[unit], 0, None) }).unit;
    for (id, slot, req) in items {
        let mut item = cat.items[*id].clone();
        if let Some((stat, over)) = req {
            let discount = u.stats.proficiency.requirement_discount(&item.weapon_type);
            item.required_stats.insert(stat.to_string(), u.stats.stat(stat, Some(&item.weapon_type)) + discount + over);
        }
        u.inventory.equip(Equipment::new(&item), *slot);
    }
    u.sync_from_stats_and_gear(cat);
    u
}

fn worn(u: &Unit, slot: Slot) -> &Equipment {
    u.inventory.get(slot).expect("worn")
}

fn weapon_power(u: &Unit) -> f64 {
    damage::attack_power(u, false, false).weapon_power
}

fn armour(u: &Unit, slot: Slot) -> i32 {
    worn(u, slot).effective_mitigation(Some(&u.stats))
}

fn default() -> WieldRule {
    WieldRule::default()
}

#[test]
fn the_defaults_are_the_spec_table_and_the_ruleset_reads_them() {
    let w = default();
    assert_eq!((w.untrained_power_mult, w.untrained_burden_add, w.req_discount_per_level), (0.75, 1, 2), "U3 'Fixed vs parameter'");
    assert_eq!((UNTRAINED_POWER_MULT, UNTRAINED_BURDEN_ADD, REQ_DISCOUNT_PER_LEVEL), (0.75, 1, 2));
    assert_eq!((WIELD_DEFICIT_STEP, WIELD_DEFICIT_CAP), (0.15, 0.60), "FB-33: the ramp kept as built");
    assert_eq!(Numbers::default().wield(), w);
    assert_eq!(ruleset::Ruleset::classic().numbers.wield(), w);
    let row = json!({"id": "t", "rules": [], "untrained_power_mult": 0.5, "untrained_burden_add": 2, "req_discount_per_level": 0});
    let n = Numbers::from_row(row.as_object().unwrap());
    assert_eq!(n.wield(), WieldRule { untrained_power_mult: 0.5, untrained_burden_add: 2, req_discount_per_level: 0 });
    assert_eq!([n.field("untrained_power_mult"), n.field("untrained_burden_add"), n.field("req_discount_per_level")], [Some("0.5".into()), Some("2".into()), Some("0".into())]);
    for (key, range) in [("untrained_power_mult", (0.25, 1.0)), ("untrained_burden_add", (0.0, 4.0)), ("req_discount_per_level", (0.0, 4.0))] {
        let rule = ruleset::field_rule_of(key).unwrap_or_else(|| panic!("{key}: a field rule"));
        assert_eq!((rule.range, rule.needs.is_empty()), (Some(range), true), "{key}: row-only, read whatever the rules");
        assert!(ruleset::PROMOTED.iter().any(|(_, k)| *k == key), "{key}: promoted");
    }
}

#[test]
fn a_trained_item_meeting_its_requirements_wears_free_with_its_training_power() {
    let cat = catalog();
    let u = wearing(&cat, "probe", default(), &[("sword", Slot::MainHand, None), ("jerkin", Slot::Body, None)]);
    for slot in [Slot::MainHand, Slot::Body] {
        let p = worn(&u, slot).wield_penalty(&u.stats);
        assert!(!p.penalized && p.factor() == 1.0, "{slot:?}: {p:?}");
    }
    let training = u.stats.proficiency.power_bonus("sword");
    assert_eq!(training, 1, "level 1 keeps +level power");
    assert_eq!(weapon_power(&u), (POWER + training) as f64);
    assert_eq!(armour(&u, Slot::Body), MITIGATION);
    assert_eq!(u.worn_burden(), 1, "the jerkin's own burden");
}

#[test]
fn the_stat_deficit_ramp_is_kept_on_power_and_reaches_armour() {
    let cat = catalog();
    for over in 0..=6 {
        let u = wearing(&cat, "probe", default(), &[("sword", Slot::MainHand, Some(("STR", over))), ("jerkin", Slot::Body, Some(("CONS", over)))]);
        let ratio = (over as f64 * WIELD_DEFICIT_STEP).min(WIELD_DEFICIT_CAP);
        let p = worn(&u, Slot::MainHand).wield_penalty(&u.stats);
        assert_eq!((p.deficit, p.ratio, p.untrained), (over, ratio, false), "over {over}");
        assert_eq!(weapon_power(&u), (POWER + 1) as f64 * (1.0 - ratio), "over {over}");
        assert_eq!(armour(&u, Slot::Body), roundi(MITIGATION as f64 * (1.0 - ratio)), "over {over}");
        assert_eq!(u.worn_burden(), 1, "a stat deficit adds no burden (over {over})");
    }
}

#[test]
fn proficiency_0_multiplies_power_and_armour_and_adds_burden_that_lowers_initiative() {
    let cat = catalog();
    let w = default();
    let u = wearing(&cat, "probe", w, &[("axe", Slot::MainHand, None), ("plate", Slot::Body, None)]);
    for slot in [Slot::MainHand, Slot::Body] {
        let p = worn(&u, slot).wield_penalty(&u.stats);
        assert!(p.penalized && p.untrained && p.deficit == 0, "{slot:?}: {p:?}");
        assert_eq!((p.untrained_mult, p.burden_add, p.factor()), (w.untrained_power_mult, w.untrained_burden_add, w.untrained_power_mult));
    }
    assert_eq!(u.stats.proficiency.power_bonus("axe"), 0, "no training power at level 0");
    assert_eq!(weapon_power(&u), POWER as f64 * w.untrained_power_mult);
    assert_eq!(armour(&u, Slot::Body), roundi(MITIGATION as f64 * w.untrained_power_mult));
    // the same pieces trained (the axeman's axe; the jerkin is the plate's twin in a trained type)
    let trained = wearing(&cat, "probe", w, &[("sword", Slot::MainHand, None), ("jerkin", Slot::Body, None)]);
    assert_eq!(u.worn_burden(), trained.worn_burden() + 2 * w.untrained_burden_add, "+1 burden per untrained item");
    assert_eq!(u.initiative, trained.initiative - 2 * w.untrained_burden_add, "the burden lowers initiative first");
    assert_eq!(u.move_rate, trained.move_rate, "the move rate only past initiative's floor");
}

#[test]
fn the_deficit_and_proficiency_0_stack_multiplicatively() {
    let cat = catalog();
    let w = default();
    for over in [1, 2, 5] {
        let u = wearing(&cat, "probe", w, &[("axe", Slot::MainHand, Some(("STR", over))), ("plate", Slot::Body, Some(("CONS", over)))]);
        let ratio = (over as f64 * WIELD_DEFICIT_STEP).min(WIELD_DEFICIT_CAP);
        let p = worn(&u, Slot::MainHand).wield_penalty(&u.stats);
        assert!(p.untrained && p.deficit == over, "{p:?}");
        assert_eq!(p.factor(), (1.0 - ratio) * w.untrained_power_mult);
        assert_eq!(weapon_power(&u), POWER as f64 * ((1.0 - ratio) * w.untrained_power_mult), "over {over}");
        assert_eq!(armour(&u, Slot::Body), roundi(MITIGATION as f64 * ((1.0 - ratio) * w.untrained_power_mult)), "over {over}");
    }
}

#[test]
fn the_ruleset_numbers_set_the_terms() {
    let cat = catalog();
    let w = WieldRule { untrained_power_mult: 0.5, untrained_burden_add: 2, req_discount_per_level: 0 };
    let u = wearing(&cat, "probe", w, &[("axe", Slot::MainHand, None), ("plate", Slot::Body, None)]);
    assert_eq!(u.stats.proficiency.wield, w, "the build carries the terms");
    assert_eq!(weapon_power(&u), POWER as f64 * 0.5);
    assert_eq!(armour(&u, Slot::Body), roundi(MITIGATION as f64 * 0.5));
    assert_eq!(u.worn_burden(), 1 + 2 + 2, "the plate's burden plus 2 per untrained item");
    // no requirement discount: a sword needing STR + 1 over the bare stat is 1 short (2 under the default terms is none)
    let mut sword = cat.items["sword"].clone();
    let need = u.stats.stat("STR", Some("sword")) + 1;
    sword.required_stats.insert("STR".into(), need);
    let eq = Equipment::new(&sword);
    assert_eq!(eq.wield_penalty(&u.stats).deficit, 1);
    let free = wearing(&cat, "probe", default(), &[]);
    assert_eq!(eq.wield_penalty(&free.stats).deficit, 0, "the default discount covers it");
}

#[test]
fn accessories_untyped_gear_classes_without_a_list_and_classless_units_wear_free() {
    let cat = catalog();
    let u = wearing(&cat, "probe", default(), &[("ring", Slot::Accessory1, None), ("club", Slot::MainHand, None)]);
    assert!(trained_for(&u.stats.proficiency, &cat.items["ring"]), "an accessory, whatever its type");
    assert!(trained_for(&u.stats.proficiency, &cat.items["club"]), "gear with no type");
    assert!(!worn(&u, Slot::Accessory1).wield_penalty(&u.stats).penalized && !worn(&u, Slot::MainHand).wield_penalty(&u.stats).penalized);
    let loose = wearing(&cat, "loose", default(), &[("axe", Slot::MainHand, None), ("plate", Slot::Body, None)]);
    assert!(loose.stats.proficiency.entries.is_empty());
    assert!(!worn(&loose, Slot::MainHand).wield_penalty(&loose.stats).untrained && !worn(&loose, Slot::Body).wield_penalty(&loose.stats).untrained, "no proficiency list, no restriction");
    // a unit with no class borrows the first class's stats, not its training rule
    let nobody = build_unit(&cat, &cat.units["nobody"], 0);
    assert!(nobody.stats.proficiency.unrestricted);
    for slot in [Slot::MainHand, Slot::Body] {
        assert!(!worn(&nobody, slot).wield_penalty(&nobody.stats).untrained, "{slot:?}");
    }
}

#[test]
fn unarmed_power_is_as_before() {
    let cat = catalog();
    for (unit, fists) in [("probe", 0), ("rookie", 1), ("boxer", 2)] {
        let u = wearing(&cat, unit, default(), &[]);
        assert_eq!(u.stats.proficiency.level_of("fist"), fists);
        let want = if fists >= damage::TRAINED_FIST_MIN_LEVEL { (UNARMED_BASE_POWER + fists) as f64 } else { 0.0 };
        assert_eq!(weapon_power(&u), want, "{unit}: fist {fists}");
    }
}

#[test]
fn skill_gates_stay_hard() {
    let cat = catalog();
    let chop = cat.skill("chop").expect("the skill").clone();
    let mut untrained = wearing(&cat, "probe", default(), &[("axe", Slot::MainHand, None)]);
    let mut trained = wearing(&cat, "axer", default(), &[("axe", Slot::MainHand, None)]);
    for u in [&mut untrained, &mut trained] {
        u.known_skills = vec!["chop".into()];
    }
    assert!(trained.can_use(&cat, &chop), "the axeman's training opens the skill");
    assert!(!untrained.can_use(&cat, &chop), "worn untrained, the skill's min_proficiency still refuses it");
}
