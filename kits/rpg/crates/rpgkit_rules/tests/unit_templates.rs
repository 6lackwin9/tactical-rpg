//! R4-1 `U2.3-TIERS` slice 1 (FB-78, docs/sessions/U2_CLASS.md item 2) on a scratch catalog built here (no shipped
//! data): a non-hero derived unit (not the party's, tier other than hero) takes the `unit_templates` row of its AI role
//! (its own `ai_role`, else its class's) for its reference stats, scales its stats at level by its tier's `tier_hp`
//! (`round(stat x tier_hp)`) and takes `round(60 x tier_hp) + CONS x 8 + STR x 2` as max HP; heroes, allies, fixed-stat
//! units and roles without a row keep their class's stats and the old formula; a class change never moves a templated
//! unit. Every expectation is derived from the rules (`unit_budget`, `auto_spend`, the tier table).

use rpgkit_data::{Catalog, ClassShares, UnitDef};
use rpgkit_rules::stats::{auto_spend, roundi, spend_pool, tier_row, HERO_TIER, HP_PER_CONS};
use rpgkit_rules::unit::template_for;
use rpgkit_rules::{build_unit, unit_budget, Unit};
use indexmap::IndexMap;
use serde_json::{json, Value};

/// The flat base of the max HP formulas (60).
const HP_BASE: i32 = 60;

const LEVEL: i32 = 5;
const FOE_TIERS: [&str; 4] = ["minion", "standard", "elite", "boss"];

fn class_row(base: Value) -> Value {
    json!({"id": "drill", "name": "Drill", "ai_role": "AGGRESSOR", "base_stats": base, "growth": {"STR": 2, "CONS": 2, "DEX": 1},
        "proficiency": {"sword": {"level": 1, "scaling": 1.5}}})
}

fn drill_base() -> Value {
    json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10})
}

/// The AGGRESSOR template: heavy on STR and CONS (shares of the unit budget, each vector totalling 1).
fn aggressor() -> Value {
    json!({"id": "brute", "name": "Brute", "role": "AGGRESSOR",
        "base_shares": {"STR": 0.25, "DEX": 0.15, "INT": 0.1, "CONS": 0.25, "PERC": 0.15, "KNOW": 0.1},
        "growth_shares": {"STR": 0.4, "DEX": 0.2, "CONS": 0.4}})
}

/// The SNIPER template: heavy on DEX and PERC.
fn sniper() -> Value {
    json!({"id": "marksman", "name": "Marksman", "role": "SNIPER",
        "base_shares": {"STR": 0.1, "DEX": 0.25, "INT": 0.1, "CONS": 0.15, "PERC": 0.25, "KNOW": 0.15},
        "growth_shares": {"DEX": 0.4, "PERC": 0.4, "CONS": 0.2}})
}

fn unit_row(id: &str, side: &str, tier: &str, role: &str) -> Value {
    json!({"id": id, "name": id, "class_id": "drill", "level": LEVEL, "tier": tier, "side": side, "ai_role": role})
}

fn units() -> Vec<Value> {
    let mut rows: Vec<Value> = FOE_TIERS.iter().map(|t| unit_row(&format!("foe_{t}"), "enemy", t, "")).collect();
    rows.push(unit_row("foe_hero", "enemy", HERO_TIER, ""));
    rows.push(unit_row("ally_minion", "party", "minion", ""));
    rows.push(unit_row("hero", "party", HERO_TIER, ""));
    rows.push(unit_row("neutral_elite", "neutral", "elite", ""));
    rows.push(unit_row("sniper_minion", "enemy", "minion", "SNIPER"));
    rows.push(unit_row("support_minion", "enemy", "minion", "SUPPORT"));
    rows.push(json!({"id": "dummy", "name": "Dummy", "class_id": "drill", "level": LEVEL, "tier": "minion", "side": "enemy", "stat_mode": "fixed", "hp": 50, "speed": 5}));
    rows
}

fn catalog(base: Value, templates: Vec<Value>) -> Catalog {
    Catalog::from_value(&json!({"classes": [class_row(base)], "units": units(), "unit_templates": templates}))
}

fn row<'c>(cat: &'c Catalog, id: &str) -> &'c UnitDef {
    &cat.units[id]
}

fn unit(cat: &Catalog, id: &str) -> Unit {
    build_unit(cat, row(cat, id), 0)
}

fn stat(u: &Unit, s: &str) -> i32 {
    u.stats.stat(s, u.weapon_context().as_deref())
}

fn shares_of(t: &Value) -> ClassShares {
    let map = |k: &str| t[k].as_object().unwrap().iter().map(|(s, v)| (s.clone(), v.as_f64().unwrap())).collect();
    ClassShares { base: map("base_shares"), growth: map("growth_shares") }
}

/// The rule: the template's stats at the level plus its free spend (by its growth shares), each times `factor`.
fn expected_core(t: &Value, level: i32, factor: f64) -> IndexMap<String, i32> {
    let shares = shares_of(t);
    let mut core = unit_budget::stats_at(&shares, level);
    for (s, pts) in auto_spend(&shares.growth, spend_pool(level)) {
        *core.get_mut(&s).unwrap() += pts;
    }
    core.into_iter().map(|(s, v)| (s, roundi(v as f64 * factor))).collect()
}

#[test]
fn with_no_template_row_every_unit_keeps_its_class_and_the_old_formula() {
    let plain = catalog(drill_base(), vec![]);
    let hero = unit(&plain, "hero");
    for t in FOE_TIERS {
        let u = unit(&plain, &format!("foe_{t}"));
        assert!(template_for(&plain, row(&plain, &format!("foe_{t}")), &plain.classes["drill"]).is_none(), "{t}");
        assert!(!u.tier_scaled, "{t}");
        assert_eq!(u.stats.core, hero.stats.core, "{t}: the class's stats");
        let old = roundi((HP_BASE + stat(&u, "CONS") * HP_PER_CONS + stat(&u, "STR") * 2) as f64 * tier_row(t).hp);
        assert_eq!(u.max_hp, old, "{t}: (60 + CONS x 8 + STR x 2) x tier_hp");
    }
    // a row for another role leaves an AGGRESSOR unit as it was
    let other = catalog(drill_base(), vec![sniper()]);
    for t in FOE_TIERS {
        let id = format!("foe_{t}");
        let (a, b) = (unit(&plain, &id), unit(&other, &id));
        assert_eq!((a.stats.core, a.max_hp, a.tier_scaled), (b.stats.core, b.max_hp, b.tier_scaled), "{t}");
    }
}

#[test]
fn a_non_hero_derived_unit_takes_its_role_template_scaled_by_tier_hp() {
    let cat = catalog(drill_base(), vec![aggressor(), sniper()]);
    let brute = aggressor();
    for t in FOE_TIERS {
        let id = format!("foe_{t}");
        let found = template_for(&cat, row(&cat, &id), &cat.classes["drill"]).map(|x| x.id.as_str());
        assert_eq!(found, Some("brute"), "{t}: the class's role (the row's ai_role is empty)");
        let u = unit(&cat, &id);
        assert!(u.tier_scaled, "{t}");
        assert_eq!(u.stats.core, expected_core(&brute, LEVEL, tier_row(t).hp), "{t}: round(template stat at level x tier_hp)");
    }
    // a neutral non-hero unit is not the party's: templated too
    assert_eq!(unit(&cat, "neutral_elite").stats.core, expected_core(&brute, LEVEL, tier_row("elite").hp));
    // the unit's own ai_role picks the row; a role without a row keeps the class
    assert_eq!(unit(&cat, "sniper_minion").stats.core, expected_core(&sniper(), LEVEL, tier_row("minion").hp), "ai_role SNIPER");
    let support = unit(&cat, "support_minion");
    assert!(!support.tier_scaled, "no SUPPORT row");
    assert_eq!(support.stats.core, unit(&cat, "hero").stats.core, "no SUPPORT row: the class's stats, unscaled");
}

#[test]
fn heroes_allies_and_fixed_units_never_take_a_template() {
    let cat = catalog(drill_base(), vec![aggressor(), sniper()]);
    let plain = catalog(drill_base(), vec![]);
    for id in ["hero", "ally_minion", "foe_hero", "dummy"] {
        assert!(template_for(&cat, row(&cat, id), &cat.classes["drill"]).is_none(), "{id}");
        let (u, before) = (unit(&cat, id), unit(&plain, id));
        assert!(!u.tier_scaled, "{id}");
        assert_eq!((u.stats.core, u.max_hp), (before.stats.core, before.max_hp), "{id}: as with no template table");
    }
}

#[test]
fn a_templated_unit_takes_the_base_only_hp_formula() {
    let cat = catalog(drill_base(), vec![aggressor()]);
    for t in FOE_TIERS {
        let u = unit(&cat, &format!("foe_{t}"));
        let hp = tier_row(t).hp;
        assert_eq!(u.max_hp, roundi(HP_BASE as f64 * hp) + stat(&u, "CONS") * HP_PER_CONS + stat(&u, "STR") * 2, "{t}: round(60 x tier_hp) + CONS x 8 + STR x 2");
        assert_eq!(u.hp, u.max_hp, "{t}: starts full");
    }
}

#[test]
fn a_class_change_never_moves_a_templated_unit() {
    let shifted = json!({"STR": 18, "DEX": 6, "INT": 6, "CONS": 18, "PERC": 7, "KNOW": 7});
    let (a, b) = (catalog(drill_base(), vec![aggressor()]), catalog(shifted, vec![aggressor()]));
    assert_ne!(unit(&a, "hero").stats.core, unit(&b, "hero").stats.core, "the class change moves the hero");
    for t in FOE_TIERS {
        let id = format!("foe_{t}");
        let (x, y) = (unit(&a, &id), unit(&b, &id));
        assert_eq!((x.stats.core, x.max_hp), (y.stats.core, y.max_hp), "{t}: delinked from the class row");
    }
}
