//! U1.2-S4, R4-1 (user 2026-11-09: one factor per tier) on a scratch catalog built here (no shipped data): the kit's
//! tier table (`tiers.json`) holds each tier's `mult` (the formula input `tier_mult`, which replaced `tier_hp` and
//! `tier_power`) and AI skill cap, each factor inside the user's range for its tier; a unit without a role template
//! has max HP `round((60 + CONS x 8 + STR x 2) x tier_mult)` and attack power's base-stage multiplier `tier_mult`;
//! derived formulas read `tier_mult` (and its deprecated aliases `tier_hp`, `tier_power`, the same value); a tier leaves
//! an untemplated unit's stats alone (the 6..18 check reads the class row).

use rpgkit_data::Catalog;
use rpgkit_rules::derived::{self, InputKind};
use rpgkit_rules::stats::{roundi, tier_row, HERO_TIER, HP_PER_CONS, TIERS};
use rpgkit_rules::{build_unit, damage, Unit};
use serde_json::{json, Value};

/// The user's ranges per tier (2026-11-09), inclusive; the hero is 1.
const RANGES: [(&str, f64, f64); 5] = [("hero", 1.0, 1.0), ("minion", 0.4, 0.6), ("standard", 0.8, 1.0), ("elite", 1.25, 1.5), ("boss", 2.0, 3.0)];

fn catalog() -> Catalog {
    let unit = |tier: &str| json!({"id": format!("u_{tier}"), "name": tier, "class_id": "drill", "level": 4, "tier": tier, "side": "enemy"});
    let units: Vec<Value> = RANGES.iter().map(|(t, ..)| unit(t)).collect();
    Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10}, "growth": {"STR": 2, "CONS": 2, "DEX": 1},
            "proficiency": {"sword": {"level": 1, "scaling": 1.5}}}],
        "units": units,
        "items": [{"id": "blade", "name": "Blade", "item_type": "WEAPON", "weapon_type": "sword", "base_power": 9}],
        "derived_stats": [{"id": "tier_mark", "name": "Tier Mark", "formula": "tier_mult * 100 + tier_hp * 10 + tier_power", "min": 0, "max": 1000}]
    }))
}

fn unit(cat: &Catalog, tier: &str) -> Unit {
    build_unit(cat, &cat.units[&format!("u_{tier}")], 0)
}

fn stat(u: &Unit, s: &str) -> i32 {
    u.stats.stat(s, u.weapon_context().as_deref())
}

#[test]
fn every_tier_factor_is_inside_its_range() {
    let ids: Vec<&str> = TIERS.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, RANGES.iter().map(|o| o.0).collect::<Vec<_>>());
    for (id, lo, hi) in RANGES {
        let m = tier_row(id).mult;
        assert!((lo..=hi).contains(&m), "{id}: {m} outside {lo}..{hi}");
    }
    assert_eq!(tier_row("minion").skill_cap, 2, "the minion AI cap is kept");
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
fn max_hp_is_the_hero_formula_times_tier_mult() {
    let cat = catalog();
    for (tier, ..) in RANGES {
        let u = unit(&cat, tier);
        let m = tier_row(tier).mult;
        let full = 60 + stat(&u, "CONS") * HP_PER_CONS + stat(&u, "STR") * 2;
        assert_eq!(u.tier_mult, m, "{tier}");
        assert!(!u.tier_scaled, "{tier}: no template table");
        assert_eq!(u.max_hp, roundi(full as f64 * m), "{tier}: (60 + CONS x 8 + STR x 2) x tier_mult");
    }
}

#[test]
fn tier_mult_is_the_base_stage_multiplier() {
    let cat = catalog();
    for (tier, ..) in RANGES {
        let m = tier_row(tier).mult;
        let mut u = unit(&cat, tier);
        u.inventory.equip(rpgkit_rules::gear::Equipment { item: cat.items["blade"].clone() }, rpgkit_rules::gear::Slot::MainHand);
        let h = damage::attack_power(&u, false, false);
        assert_eq!(h.tier_mult, m);
        assert_eq!(h.attack_power, roundi((h.weapon_power + h.unit_power) * m).max(1), "{tier}: (weapon + unit) x tier_mult");
    }
}

#[test]
fn formulas_read_the_tier_factor_and_its_aliases() {
    let cat = catalog();
    for (tier, ..) in RANGES {
        let m = tier_row(tier).mult;
        let u = unit(&cat, tier);
        let b = derived::breakdown(&cat, "classic", &u, "tier_mark").expect("the row");
        let kinds: Vec<(String, InputKind, f64)> = b.inputs.iter().map(|i| (i.name.clone(), i.kind, i.value)).collect();
        let want: Vec<(String, InputKind, f64)> = ["tier_mult", "tier_hp", "tier_power"].iter().map(|n| (n.to_string(), InputKind::Tier, m)).collect();
        assert_eq!(kinds, want, "{tier}");
        assert_eq!(b.value, roundi(m * 111.0).min(1000), "{tier}");
    }
}

#[test]
fn a_tier_leaves_the_stats_alone() {
    let cat = catalog();
    let hero = unit(&cat, HERO_TIER);
    for (tier, ..) in RANGES {
        assert_eq!(unit(&cat, tier).stats.core, hero.stats.core, "{tier}: the class row's stats, whatever the tier");
    }
}
