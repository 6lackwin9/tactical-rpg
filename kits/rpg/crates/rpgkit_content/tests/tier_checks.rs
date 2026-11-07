//! U1.2-S4 (U2_CLASS.md feasibility 2: "apply the 6..18 stat check to the class base row, not tier-scaled minions"):
//! the base stat range is checked on the class row; a unit of any tier is not checked against it (a tier scales HP
//! and power, never the stats), so a minion of a valid class carries no stat-range error.

use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

#[test]
fn the_stat_range_is_checked_on_the_class_row_only() {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_tier_checks");
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    let base = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let high = json!({"STR": 19, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 3});
    let growth = json!({"STR": 2, "CONS": 2, "DEX": 1});
    write_kind(&data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": base, "growth": growth}, {"id": "brute", "name": "Brute", "base_stats": high, "growth": growth}]));
    let units: Vec<Value> = ["hero", "minion", "standard", "elite", "boss"].iter().map(|t| json!({"id": format!("u_{t}"), "name": t, "class_id": "drill", "tier": t, "level": 3})).collect();
    write_kind(&data, "units", Value::Array(units));
    let v = store::view(&data);
    let range = |e: &[String]| e.iter().filter(|x| x.starts_with("Base ") && x.contains("must be between")).cloned().collect::<Vec<_>>();
    assert!(range(&check::validate_entry(&v, "classes", v.get("classes", "drill"), "drill")).is_empty());
    assert_eq!(range(&check::validate_entry(&v, "classes", v.get("classes", "brute"), "brute")), ["Base STR 19 must be between 6 and 18.", "Base KNOW 3 must be between 6 and 18."]);
    for tier in ["hero", "minion", "standard", "elite", "boss"] {
        let id = format!("u_{tier}");
        assert!(range(&check::validate_entry(&v, "units", v.get("units", &id), &id)).is_empty(), "{tier}");
    }
}
