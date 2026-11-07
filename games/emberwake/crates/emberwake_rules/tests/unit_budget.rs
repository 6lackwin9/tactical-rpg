//! U2.2-S1 proof (FB-10): every class, shipped (`data/catalog/classes.json`) and frozen (the timeline golden's
//! `timeline_kit.json`), gives identical stats at levels 1-10 when read as the unit budget's two share vectors (base /
//! 62, growth / 5), both as computed shares and as a row rewritten in the share form; the row's computed
//! base_stats/growth view equals its integer fields (growth in its key order, which the sim's `primary_stat` reads).
//! Expectations come from each row's own integer fields (base + growth per level gained), not from shipped numbers.
//! U2.2-S3: a row already stored as shares (after the asset lane's re-save) gives its integer form from its computed
//! view, so the same proof holds before and after the re-save; `auto_spend` reads the growth shares (FB-46) and spends
//! exactly as the integer rule did on the growth row, at every level's pool and beyond.

use emberwake_data::{ClassDef, STATS};
use emberwake_rules::stats::{auto_spend, spend_pool, CharacterStats, MAX_LEVEL};
use emberwake_rules::unit_budget::{self, BASE_BUDGET, GROWTH_BUDGET};
use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rows(path: PathBuf, frozen: bool) -> Vec<Map<String, Value>> {
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing {}", path.display()));
    let v: Value = serde_json::from_str(&text).unwrap();
    let list = if frozen { &v["classes"] } else { &v["entries"] };
    list.as_array().unwrap().iter().map(|r| r.as_object().unwrap().clone()).collect()
}

fn all_classes() -> Vec<(String, Map<String, Value>)> {
    let mut out = Vec::new();
    for r in rows(crate_dir().join("../../data/catalog/classes.json"), false) {
        out.push((format!("shipped {}", r["id"]), r));
    }
    for r in rows(crate_dir().join("../emberwake_battle/tests/fixtures/timeline_kit.json"), true) {
        out.push((format!("frozen {}", r["id"]), r));
    }
    assert!(out.len() >= 2, "classes found");
    out
}

/// The row with base_stats/growth replaced by base_shares/growth_shares, through JSON text as a data file would be.
fn share_row(r: &Map<String, Value>) -> ClassDef {
    let as_shares = |key: &str, budget: i32| -> Value {
        r[key].as_object().unwrap().iter().map(|(k, v)| (k.clone(), json!(v.as_f64().unwrap() / budget as f64))).collect()
    };
    let mut row = r.clone();
    row.insert("base_shares".into(), as_shares("base_stats", BASE_BUDGET));
    row.insert("growth_shares".into(), as_shares("growth", GROWTH_BUDGET));
    row.remove("base_stats");
    row.remove("growth");
    let text = Value::Object(row).to_string();
    ClassDef::from_obj(serde_json::from_str::<Value>(&text).unwrap().as_object().unwrap())
}

/// The row with base_shares/growth_shares replaced by its computed base_stats/growth view (a share row's integer form).
fn int_row(r: &Map<String, Value>, c: &ClassDef) -> ClassDef {
    let ints = |m: &IndexMap<String, i32>| -> Value { m.iter().map(|(k, v)| (k.clone(), json!(v))).collect() };
    let mut row = r.clone();
    row.remove("base_shares");
    row.remove("growth_shares");
    row.insert("base_stats".into(), ints(&c.base_stats));
    row.insert("growth".into(), ints(&c.growth));
    ClassDef::from_obj(&row)
}

/// The class in both forms: (integer row, share row). A row stored as integers is rewritten as shares; a row stored as
/// shares is rewritten as its computed view, which the tests then hold to the integer rule.
fn both_forms(r: &Map<String, Value>) -> (ClassDef, ClassDef) {
    let c = ClassDef::from_obj(r);
    if c.shares.is_some() {
        (int_row(r, &c), c)
    } else {
        (c, share_row(r))
    }
}

/// The free spend before U2.2-S3 (FB-46), on whole growth points: every stat with growth gets the floor of its part,
/// heaviest first (ties in stat order); the points left go one each in that order.
fn integer_auto_spend(growth: &IndexMap<String, i32>, points: i32) -> IndexMap<String, i32> {
    let mut spend: IndexMap<String, i32> = IndexMap::new();
    if points <= 0 {
        return spend;
    }
    let w = |s: &str| growth.get(s).copied().unwrap_or(0);
    let mut order: Vec<&str> = STATS.iter().copied().filter(|s| w(s) > 0).collect();
    if order.is_empty() {
        order = STATS.to_vec();
    }
    let pos = |s: &str| STATS.iter().position(|x| *x == s).unwrap_or(0);
    order.sort_by(|a, b| w(b).cmp(&w(a)).then(pos(a).cmp(&pos(b))));
    let weights: Vec<i32> = order.iter().map(|s| w(s).max(1)).collect();
    let weight_sum: i32 = weights.iter().sum();
    let mut given = 0;
    for (i, s) in order.iter().enumerate() {
        let share = ((points * weights[i]) as f64 / weight_sum as f64).floor() as i32;
        spend.insert(s.to_string(), share);
        given += share;
    }
    let mut i = 0;
    while given < points {
        *spend.entry(order[i % order.len()].to_string()).or_insert(0) += 1;
        given += 1;
        i += 1;
    }
    STATS.iter().filter_map(|s| spend.get(*s).filter(|v| **v > 0).map(|v| (s.to_string(), *v))).collect()
}

/// `shares` as the data writer stores them (15 significant digits), read back.
fn written(shares: &unit_budget::Shares) -> unit_budget::Shares {
    shares.iter().map(|(k, v)| (k.clone(), format!("{v:.14e}").parse::<f64>().unwrap())).collect()
}

#[test]
fn every_class_at_every_level_is_unchanged_by_the_unit_budget() {
    let none = Default::default();
    for (name, r) in all_classes() {
        let (ints, shares) = both_forms(&r);
        assert_eq!(shares.base_stats, ints.base_stats, "{name}: base view");
        assert_eq!(shares.growth.iter().collect::<Vec<_>>(), ints.growth.iter().collect::<Vec<_>>(), "{name}: growth view");
        for level in 1..=MAX_LEVEL {
            // today's rule, from the row's own integers
            let want: Vec<(String, i32)> = STATS
                .iter()
                .map(|s| (s.to_string(), ints.base_stats.get(*s).copied().unwrap_or(10) + ints.growth.get(*s).copied().unwrap_or(0) * (level - 1)))
                .collect();
            let got = |c: &ClassDef| CharacterStats::new(c, level, &none).core.into_iter().collect::<Vec<_>>();
            assert_eq!(got(&ints), want, "{name} level {level}: integer row");
            assert_eq!(got(&shares), want, "{name} level {level}: share row");
            let computed: Vec<_> = unit_budget::stats_at(&ints.unit_shares(), level).into_iter().collect();
            assert_eq!(computed, want, "{name} level {level}: computed shares");
        }
    }
}

#[test]
fn auto_spend_reads_the_growth_shares_and_spends_as_the_integer_rule_did() {
    for (name, r) in all_classes() {
        let (ints, shares) = both_forms(&r);
        let pools: Vec<i32> = (1..=MAX_LEVEL).map(spend_pool).chain(0..=3 * spend_pool(MAX_LEVEL)).collect();
        for points in pools {
            let want = integer_auto_spend(&ints.growth, points);
            assert_eq!(want.values().sum::<i32>(), points.max(0), "{name} {points}: the rule spends every point");
            for (form, c) in [("integer row", &ints), ("share row", &shares)] {
                let growth = c.unit_shares().growth;
                assert_eq!(auto_spend(&growth, points), want, "{name} {points} points: {form}");
                assert_eq!(auto_spend(&written(&growth), points), want, "{name} {points} points: {form} at the writer's 15 digits");
            }
        }
    }
}
