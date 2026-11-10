//! R4-1 `U2.3-TIERS` slices 1 and 2 (FB-78, FB-144, docs/sessions/U2_CLASS.md item 2) on a scratch catalog built here
//! (no shipped data): a non-hero derived unit (not the party's, tier other than hero) takes the `unit_templates` row of
//! its AI role (its own `ai_role`, else its class's; its `template` overrides: `class` = none, a role or id = that row)
//! for its reference stats, scales its stats at level by its tier's `tier_mult` with a floor of 6
//! (`max(6, round(stat x tier_mult))`) and takes `round(60 x tier_mult) + CONS x 8 + STR x 2` as max HP; heroes, allies,
//! fixed-stat units and roles without a row keep their class's stats and the old formula; a class change never moves a
//! templated unit. Gear requirements and the KNOW of skill unlocks read the unscaled stats; attack power does not scale
//! the (already scaled) stat by `tier_mult` again. Every expectation is derived from the rules (`unit_budget`,
//! `auto_spend`, the tier table, the damage pipeline's base stage).

use rpgkit_data::{Catalog, ClassShares, UnitDef};
use rpgkit_rules::stats::{auto_spend, roundi, spend_pool, tier_row, HERO_TIER, HP_PER_CONS};
use rpgkit_rules::damage::attack_power;
use rpgkit_rules::gear::Equipment;
use rpgkit_rules::unit::{spend_growth, template_for};
use rpgkit_rules::{build_unit, unit_budget, Unit};
use indexmap::IndexMap;
use serde_json::{json, Value};

/// The flat base of the max HP formulas (60).
const HP_BASE: i32 = 60;

/// The floor of a tier-scaled template stat (FB-144).
const STAT_FLOOR: i32 = 6;

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
    // slice 2: the `template` override and a sword wearer
    for (id, side, key) in [("as_class", "enemy", "class"), ("as_sniper", "enemy", "SNIPER"), ("as_marksman", "enemy", "marksman"), ("as_support", "enemy", "SUPPORT"), ("ally_sniper", "party", "SNIPER")] {
        let mut r = unit_row(id, side, "minion", "");
        r["template"] = json!(key);
        rows.push(r);
    }
    for t in FOE_TIERS {
        let mut r = unit_row(&format!("armed_{t}"), "enemy", t, "");
        r["main_hand"] = json!("blade");
        rows.push(r);
    }
    let mut hero = unit_row("armed_hero", "party", HERO_TIER, "");
    hero["main_hand"] = json!("blade");
    rows.push(hero);
    rows
}

fn catalog(base: Value, templates: Vec<Value>) -> Catalog {
    let blade = json!({"id": "blade", "name": "Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 20, "scaling_stat": "STR"});
    Catalog::from_value(&json!({"classes": [class_row(base)], "units": units(), "items": [blade], "unit_templates": templates}))
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

/// The template's stats at the level plus its free spend (by its growth shares), unscaled.
fn unscaled_core(t: &Value, level: i32) -> IndexMap<String, i32> {
    let shares = shares_of(t);
    let mut core = unit_budget::stats_at(&shares, level);
    for (s, pts) in auto_spend(&shares.growth, spend_pool(level)) {
        *core.get_mut(&s).unwrap() += pts;
    }
    core
}

/// The rule: the unscaled stats, each `max(6, round(stat x factor))`.
fn expected_core(t: &Value, level: i32, factor: f64) -> IndexMap<String, i32> {
    unscaled_core(t, level).into_iter().map(|(s, v)| (s, roundi(v as f64 * factor).max(STAT_FLOOR))).collect()
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
        let old = roundi((HP_BASE + stat(&u, "CONS") * HP_PER_CONS + stat(&u, "STR") * 2) as f64 * tier_row(t).mult);
        assert_eq!(u.max_hp, old, "{t}: (60 + CONS x 8 + STR x 2) x tier_mult");
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
fn a_non_hero_derived_unit_takes_its_role_template_scaled_by_tier_mult() {
    let cat = catalog(drill_base(), vec![aggressor(), sniper()]);
    let brute = aggressor();
    for t in FOE_TIERS {
        let id = format!("foe_{t}");
        let found = template_for(&cat, row(&cat, &id), &cat.classes["drill"]).map(|x| x.id.as_str());
        assert_eq!(found, Some("brute"), "{t}: the class's role (the row's ai_role is empty)");
        let u = unit(&cat, &id);
        assert!(u.tier_scaled, "{t}");
        assert_eq!(u.stats.core, expected_core(&brute, LEVEL, tier_row(t).mult), "{t}: max(6, round(template stat at level x tier_mult))");
        assert_eq!(u.stats.unscaled, Some(unscaled_core(&brute, LEVEL)), "{t}: the unscaled stats are kept");
    }
    // a neutral non-hero unit is not the party's: templated too
    assert_eq!(unit(&cat, "neutral_elite").stats.core, expected_core(&brute, LEVEL, tier_row("elite").mult));
    // the unit's own ai_role picks the row; a role without a row keeps the class
    assert_eq!(unit(&cat, "sniper_minion").stats.core, expected_core(&sniper(), LEVEL, tier_row("minion").mult), "ai_role SNIPER");
    let support = unit(&cat, "support_minion");
    assert!(!support.tier_scaled, "no SUPPORT row");
    assert_eq!(support.stats.core, unit(&cat, "hero").stats.core, "no SUPPORT row: the class's stats, unscaled");
    assert_eq!(support.stats.unscaled, None);
}

#[test]
fn a_scaled_stat_never_drops_below_the_floor() {
    let cat = catalog(drill_base(), vec![aggressor()]);
    let factor = tier_row("minion").mult;
    let raw = unscaled_core(&aggressor(), LEVEL);
    let floored: Vec<&String> = raw.iter().filter(|(_, &v)| roundi(v as f64 * factor) < STAT_FLOOR).map(|(s, _)| s).collect();
    assert!(!floored.is_empty(), "the fixture puts a minion stat under the floor: {raw:?} x {factor}");
    let u = unit(&cat, "foe_minion");
    for s in floored {
        assert_eq!(u.stats.core[s], STAT_FLOOR, "{s}: floored");
    }
    assert!(u.stats.core.values().all(|&v| v >= STAT_FLOOR));
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
        let hp = tier_row(t).mult;
        assert_eq!(u.max_hp, roundi(HP_BASE as f64 * hp) + stat(&u, "CONS") * HP_PER_CONS + stat(&u, "STR") * 2, "{t}: round(60 x tier_mult) + CONS x 8 + STR x 2");
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

/// The blade every `armed_*` unit wears, and its requirement discount for the unit.
fn blade(u: &Unit) -> (&Equipment, i32) {
    let w = u.inventory.active_weapon().expect("the blade");
    (w, u.stats.proficiency.requirement_discount(&w.item.weapon_type))
}

#[test]
fn gear_requirements_and_skill_unlocks_read_the_unscaled_stats() {
    let cat = catalog(drill_base(), vec![aggressor()]);
    let u = unit(&cat, "armed_minion");
    assert!(u.tier_scaled);
    let (w, discount) = blade(&u);
    let ctx = Some(w.item.weapon_type.as_str());
    let raw = unscaled_core(&aggressor(), LEVEL);
    assert_eq!(u.stats.requirement_base("STR"), raw["STR"]);
    assert!(u.stats.stat("STR", ctx) < u.stats.requirement_stat("STR", ctx), "the minion's scaled STR is under its unscaled STR");
    // a requirement exactly the unscaled STR (after the discount) is met, one more is one short
    let mut item = w.item.clone();
    item.required_stats.insert("STR".into(), raw["STR"] + discount);
    let met = Equipment::new(&item);
    assert!(met.meets_requirements(&u.stats) && met.wield_penalty(&u.stats).deficit == 0, "read on the unscaled STR");
    item.required_stats.insert("STR".into(), raw["STR"] + discount + 1);
    assert_eq!(Equipment::new(&item).wield_penalty(&u.stats).deficit, 1);
    // a gear bonus (a modifier, unscaled) counts on top of the unscaled stat
    let mut boosted = u.stats.clone();
    boosted.modifiers.push(rpgkit_rules::stats::StatModifier::permanent("STR", 1, "ring", ""));
    assert!(Equipment::new(&item).meets_requirements(&boosted), "unscaled STR + 1 from gear");
    // the KNOW of skill unlocks is the unscaled KNOW (plus permanent modifiers; the fixture has none on KNOW)
    for t in FOE_TIERS {
        let u = unit(&cat, &format!("armed_{t}"));
        assert_eq!(u.unlock_know(), raw["KNOW"], "{t}: unscaled KNOW");
    }
    // an untemplated unit reads its own stats, as before
    let hero = unit(&cat, "armed_hero");
    assert_eq!(hero.stats.requirement_stat("STR", ctx), hero.stats.stat("STR", ctx));
    assert_eq!(hero.unlock_know(), hero.stats.base("KNOW"));
}

#[test]
fn attack_power_does_not_scale_a_templated_stat_twice() {
    let cat = catalog(drill_base(), vec![aggressor()]);
    for t in FOE_TIERS {
        let u = unit(&cat, &format!("armed_{t}"));
        let ap = attack_power(&u, false, false);
        assert!(ap.weapon_fits && ap.tier_scaled, "{t}");
        assert_eq!(ap.stat_val, u.stats.stat("STR", Some("sword")), "{t}: the scaled stat");
        // the base stage: the weapon power times tier_mult, plus the unit (stat) power unscaled again
        assert_eq!(ap.attack_power, roundi(ap.weapon_power * u.tier_mult + ap.unit_power).max(1), "{t}");
        let twice = roundi((ap.weapon_power + ap.unit_power) * u.tier_mult).max(1);
        // they part by unit power x (tier_mult - 1); where that is a whole point or more, the rounding cannot hide it
        if (ap.unit_power * (u.tier_mult - 1.0)).abs() >= 1.0 {
            assert_ne!(ap.attack_power, twice, "{t}: not (weapon + unit) x tier_mult");
        }
    }
    // untemplated: the old base stage, (weapon + unit) x tier_mult
    let plain = catalog(drill_base(), vec![]);
    for t in FOE_TIERS {
        let u = unit(&plain, &format!("armed_{t}"));
        let ap = attack_power(&u, false, false);
        assert!(!ap.tier_scaled, "{t}");
        assert_eq!(ap.attack_power, roundi((ap.weapon_power + ap.unit_power) * u.tier_mult).max(1), "{t}");
    }
}

#[test]
fn the_template_field_overrides_the_role() {
    let cat = catalog(drill_base(), vec![aggressor(), sniper()]);
    let found = |id: &str| template_for(&cat, row(&cat, id), &cat.classes["drill"]).map(|x| x.id.clone());
    assert_eq!(found("as_class"), None, "class: the class's shares");
    let as_class = unit(&cat, "as_class");
    assert!(!as_class.tier_scaled);
    assert_eq!(as_class.stats.core, unit(&cat, "hero").stats.core, "class: the class's stats, unscaled");
    assert_eq!(found("as_sniper").as_deref(), Some("marksman"), "a role");
    assert_eq!(found("as_marksman").as_deref(), Some("marksman"), "a template id");
    assert_eq!(unit(&cat, "as_sniper").stats.core, expected_core(&sniper(), LEVEL, tier_row("minion").mult));
    assert_eq!(found("as_support"), None, "a role without a row: none");
    assert_eq!(found("ally_sniper"), None, "the party never takes a template");
}

#[test]
fn the_spend_growth_reads_the_template() {
    let cat = catalog(drill_base(), vec![aggressor(), sniper()]);
    let class_growth = cat.classes["drill"].unit_shares().growth;
    assert_eq!(spend_growth(&cat, row(&cat, "as_sniper")), shares_of(&sniper()).growth);
    assert_eq!(spend_growth(&cat, row(&cat, "foe_minion")), shares_of(&aggressor()).growth);
    for id in ["hero", "as_class", "ally_sniper", "as_support"] {
        assert_eq!(spend_growth(&cat, row(&cat, id)), class_growth, "{id}: the class's");
    }
}

/// PR4-2 single-scaling guard: a templated `elite` has its stats already scaled by `tier_mult`, so a derived formula's
/// tier input that shares a product with a base stat (or with a row reading one) reads 1 there, and scales the other
/// terms only; the same rows on the untemplated `elite` read as written (the stat scaled once, by the formula).
#[test]
fn a_derived_formula_never_scales_a_templated_stat_twice() {
    let rows = json!([
        {"id": "stout", "name": "Stout", "formula": "CONS * tier_mult", "min": 0, "max": 100000},
        {"id": "grit", "name": "Grit", "formula": "60 * tier_mult + CONS * 8", "min": 0, "max": 100000},
        {"id": "bulk", "name": "Bulk", "formula": "(60 + CONS * 8) * tier_mult", "min": 0, "max": 100000},
        {"id": "frame", "name": "Frame", "formula": "CONS * 2", "min": 0, "max": 100000},
        {"id": "framed", "name": "Framed", "formula": "frame * tier_mult + 10 * tier_mult", "min": 0, "max": 100000},
        {"id": "shadowed", "name": "Shadowed", "formula": "frame * tier_mult", "constants": {"frame": 30}, "min": 0, "max": 100000}
    ]);
    let with_rows = |templates: Vec<Value>| {
        let blade = json!({"id": "blade", "name": "Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 20});
        Catalog::from_value(&json!({"classes": [class_row(drill_base())], "units": units(), "items": [blade], "unit_templates": templates, "derived_stats": rows}))
    };
    let value = |cat: &Catalog, u: &Unit, id: &str| rpgkit_rules::derived::value(cat, "classic", u, id).expect("the row");
    let m = tier_row("elite").mult;
    assert!(m > 1.0, "the elite factor scales up");
    let cat = with_rows(vec![aggressor()]);
    let u = unit(&cat, "foe_elite");
    assert!(u.tier_scaled && u.tier_mult == m);
    let cons = stat(&u, "CONS");
    let unscaled = unscaled_core(&aggressor(), LEVEL)["CONS"];
    assert_eq!(cons, roundi(unscaled as f64 * m).max(STAT_FLOOR), "CONS carries tier_mult once already");
    assert_eq!(value(&cat, &u, "stout"), cons, "the stat is not scaled again");
    assert_eq!(value(&cat, &u, "grit"), roundi(HP_BASE as f64 * m) + cons * 8, "the constant term still scales");
    assert_eq!(value(&cat, &u, "bulk"), HP_BASE + cons * 8, "a product holding a stat is not scaled");
    assert_eq!(value(&cat, &u, "framed"), roundi(cons as f64 * 2.0 + 10.0 * m), "through another row: that row reads a stat");
    assert_eq!(value(&cat, &u, "shadowed"), roundi(30.0 * m), "a row constant named like a row is a constant: it scales");
    // untemplated: the formula as written
    let plain = with_rows(vec![]);
    let p = unit(&plain, "foe_elite");
    assert!(!p.tier_scaled);
    let pc = stat(&p, "CONS");
    assert_eq!(value(&plain, &p, "stout"), roundi(pc as f64 * m));
    assert_eq!(value(&plain, &p, "bulk"), roundi((HP_BASE + pc * 8) as f64 * m));
    assert_eq!(value(&plain, &p, "framed"), roundi(pc as f64 * 2.0 * m + 10.0 * m));
}
