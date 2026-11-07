//! U1.2-S4 (docs/sessions/U1_STATS.md feasibility 6, U2_CLASS.md feasibility 2) on a scratch catalog built here (no
//! shipped data): the kit's tier table (`tiers.json`) holds each tier's `tier_hp`, `tier_power` and AI skill cap with
//! the values of the old `tier_scales`; max HP is `round((60 + CONS x 8 + STR x 2) x tier_hp)` (the old hp scale);
//! attack power keeps `tier_power` as its base-stage multiplier (the old power scale, so hits do not move); derived
//! formulas read `tier_hp` and `tier_power`; a tier leaves the unit's stats alone (the 6..18 check reads the class row).

use rpgkit_rules::stats::roundi;
use rpgkit_data::Catalog;
use rpgkit_rules::derived::{self, InputKind};
use rpgkit_rules::stats::{tier_row, HERO_TIER, HP_PER_CONS, TIERS};
use rpgkit_rules::{build_unit, damage, Unit};
use serde_json::{json, Value};

/// The old `stats::tier_scales` (hp, power, AI cap), written out: the table keeps these values.
const OLD: [(&str, f64, f64, i32); 5] = [("hero", 1.0, 1.0, 0), ("minion", 0.4, 0.6, 2), ("standard", 0.7, 0.8, 0), ("elite", 1.25, 1.0, 0), ("boss", 1.5, 1.1, 0)];

fn catalog() -> Catalog {
    let unit = |tier: &str| json!({"id": format!("u_{tier}"), "name": tier, "class_id": "drill", "level": 4, "tier": tier, "side": "enemy"});
    let units: Vec<Value> = OLD.iter().map(|(t, ..)| unit(t)).collect();
    Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10}, "growth": {"STR": 2, "CONS": 2, "DEX": 1},
            "proficiency": {"sword": {"level": 1, "scaling": 1.5}}}],
        "units": units,
        "items": [{"id": "blade", "name": "Blade", "item_type": "WEAPON", "weapon_type": "sword", "base_power": 9}],
        "derived_stats": [{"id": "tier_mark", "name": "Tier Mark", "formula": "tier_hp * 100 + tier_power * 10", "min": 0, "max": 1000}]
    }))
}

fn unit(cat: &Catalog, tier: &str) -> Unit {
    build_unit(cat, &cat.units[&format!("u_{tier}")], 0)
}

fn stat(u: &Unit, s: &str) -> i32 {
    u.stats.stat(s, u.weapon_context().as_deref())
}

#[test]
fn the_table_keeps_the_old_tier_values() {
    let ids: Vec<&str> = TIERS.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, OLD.iter().map(|o| o.0).collect::<Vec<_>>());
    for (id, hp, power, cap) in OLD {
        let r = tier_row(id);
        assert_eq!((r.hp, r.power, r.skill_cap), (hp, power, cap), "{id}");
    }
    assert_eq!(tier_row("legend"), tier_row(HERO_TIER), "an unknown tier reads as hero");
    // the kit schema's unit tier options are the table's ids
    let schema: Value = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema.json")).unwrap()).unwrap();
    let tier = schema["content"]["units"].as_array().unwrap().iter().find(|f| f["key"] == "tier").expect("the unit tier field");
    let mut options: Vec<&str> = tier["options"].as_array().unwrap().iter().map(|o| o.as_str().unwrap()).collect();
    let mut table = ids.clone();
    options.sort();
    table.sort();
    assert_eq!(options, table);
}

#[test]
fn max_hp_is_the_hero_formula_times_tier_hp() {
    let cat = catalog();
    let hero = unit(&cat, HERO_TIER);
    for (tier, hp, ..) in OLD {
        let u = unit(&cat, tier);
        let full = 60 + stat(&u, "CONS") * HP_PER_CONS + stat(&u, "STR") * 2;
        assert_eq!(u.tier_hp, hp, "{tier}");
        assert_eq!(u.max_hp, roundi(full as f64 * hp), "{tier}: (60 + CONS x 8 + STR x 2) x tier_hp, the old hp scale");
        assert_eq!(u.max_hp, roundi(hero.max_hp as f64 * hp), "{tier}: the hero's max HP x tier_hp");
    }
}

#[test]
fn tier_power_is_the_old_power_scale_in_the_base_stage() {
    let cat = catalog();
    for (tier, _, power, _) in OLD {
        let mut u = unit(&cat, tier);
        u.inventory.equip(rpgkit_rules::gear::Equipment { item: cat.items["blade"].clone() }, rpgkit_rules::gear::Slot::MainHand);
        assert_eq!(u.tier_power, power, "{tier}");
        let h = damage::attack_power(&u, false, false);
        assert_eq!(h.tier_power, power);
        // the old rule: (weapon power + unit power) x power scale, rounded, at least 1
        assert_eq!(h.attack_power, roundi((h.weapon_power + h.unit_power) * power).max(1), "{tier}");
    }
}

#[test]
fn formulas_read_the_tier_factors() {
    let cat = catalog();
    for (tier, hp, power, _) in OLD {
        let u = unit(&cat, tier);
        let b = derived::breakdown(&cat, "classic", &u, "tier_mark").expect("the row");
        let kinds: Vec<(String, InputKind, f64)> = b.inputs.iter().map(|i| (i.name.clone(), i.kind, i.value)).collect();
        assert_eq!(kinds, [("tier_hp".to_string(), InputKind::Tier, hp), ("tier_power".to_string(), InputKind::Tier, power)], "{tier}");
        assert_eq!(b.value, roundi(hp * 100.0 + power * 10.0), "{tier}");
    }
}

#[test]
fn a_tier_leaves_the_stats_alone() {
    let cat = catalog();
    let hero = unit(&cat, HERO_TIER);
    for (tier, ..) in OLD {
        assert_eq!(unit(&cat, tier).stats.core, hero.stats.core, "{tier}: the class row's stats, whatever the tier");
    }
}
