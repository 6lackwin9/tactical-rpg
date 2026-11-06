//! U2.2-S1 proof (FB-10): every class, shipped (`data/catalog/classes.json`) and frozen (the timeline golden's
//! `timeline_kit.json`), gives identical stats at levels 1-10 when read as the unit budget's two share vectors (base /
//! 62, growth / 5), both as computed shares and as a row rewritten in the share form; the row's computed
//! base_stats/growth view equals its integer fields (growth in its key order, which the sim's `primary_stat` reads).
//! Expectations come from each row's own integer fields (base + growth per level gained), not from shipped numbers.

use emberwake_data::{ClassDef, STATS};
use emberwake_rules::stats::{CharacterStats, MAX_LEVEL};
use emberwake_rules::unit_budget::{self, BASE_BUDGET, GROWTH_BUDGET};
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

#[test]
fn every_class_at_every_level_is_unchanged_by_the_unit_budget() {
    let none = Default::default();
    for (name, r) in all_classes() {
        let ints = ClassDef::from_obj(&r);
        let shares = share_row(&r);
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
