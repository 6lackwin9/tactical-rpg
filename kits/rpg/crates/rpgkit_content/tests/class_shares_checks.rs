//! U2.2-S1 content side: a class row may give its stats as the unit budget's two share vectors (`base_shares`,
//! `growth_shares`) instead of `base_stats` and `growth`. Each vector must total 1 with every share 0-1 on a stat; the
//! two forms cannot mix; and the stats the shares give keep the integer row's limits. Each mistake adds its own error
//! to the errors the integer row already has. Scratch data folders only.

use aegis_core::json::Dict;
use rpgkit_content::{check, store};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

const BASE: [(&str, i64); 6] = [("STR", 13), ("DEX", 10), ("INT", 6), ("CONS", 13), ("PERC", 10), ("KNOW", 10)];
const GROWTH: [(&str, i64); 3] = [("STR", 2), ("CONS", 2), ("DEX", 1)];

fn ints(pairs: &[(&str, i64)]) -> Value {
    pairs.iter().map(|(k, v)| (k.to_string(), json!(v))).collect()
}

fn fractions(pairs: &[(&str, i64)], total: i64) -> Value {
    pairs.iter().map(|(k, v)| (k.to_string(), json!(*v as f64 / total as f64))).collect()
}

/// The errors of `row` (the class `drill`) that the integer row does not have.
fn added(v: &check::View, plain: &Dict, row: Value) -> Vec<String> {
    let base = check::validate_entry(v, "classes", plain, "drill");
    let mut e = plain.clone();
    e.shift_remove("base_stats");
    e.shift_remove("growth");
    for (k, x) in row.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, "classes", &e, "drill").into_iter().filter(|x| !base.contains(x)).collect()
}

/// A share row saved through the writer (`store::upsert`) keeps both vectors (to the writer's 15 digits), reads back as its integer row
/// (the typed catalog's view and stats), re-saves byte for byte, and a vector that does not total 1 is refused.
#[test]
fn a_share_row_goes_through_the_writer_and_back() {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_class_shares_store");
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    // the scratch game's one body (a class must name one): a copy of the kit's dummy
    let kit_body = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/units/dummy/base");
    let body = t.join("assets/units/dummy/base");
    fs::create_dir_all(&body).unwrap();
    for f in ["clips.json", "sheet.png"] {
        fs::copy(kit_body.join(f), body.join(f)).unwrap();
    }
    let prof = json!({"sword": {"level": 1, "scaling": 1.5}});
    write_kind(&data, "classes", json!([{"id": "plain", "name": "Plain", "art": "dummy", "base_stats": ints(&BASE), "growth": ints(&GROWTH), "proficiency": prof}]));
    let row = json!({"id": "drill", "name": "Drill", "art": "dummy", "base_shares": fractions(&BASE, 62), "growth_shares": fractions(&GROWTH, 5), "proficiency": prof});
    let entry: Dict = row.as_object().unwrap().clone();
    store::upsert(&data, None, "classes", &entry, None).unwrap_or_else(|e| panic!("a share row saves: {e:?}"));

    let file = data.join("catalog/classes.json");
    let text = fs::read_to_string(&file).unwrap();
    let saved: Value = serde_json::from_str(&text).unwrap();
    let stored = saved["entries"].as_array().unwrap().iter().find(|e| e["id"] == "drill").unwrap();
    // the writer prints floats to 15 significant digits: the shares come back that close, which the unit budget's
    // billionth-of-a-point quotas absorb (the stats below stay identical)
    for key in ["base_shares", "growth_shares"] {
        let (got, want) = (stored[key].as_object().unwrap(), row[key].as_object().unwrap());
        assert_eq!(got.keys().collect::<Vec<_>>(), want.keys().collect::<Vec<_>>(), "{key}: same stats, same order");
        for (k, w) in want {
            assert!((got[k].as_f64().unwrap() - w.as_f64().unwrap()).abs() < 1e-12, "{key} {k}: {} near {w}", got[k]);
        }
    }
    // U2.2-S3: the shares are schema fields (`float_stats`) written after `growth`; a share row stores no base_stats or
    // growth key (the computed view), an integer row no share key
    assert!(stored.get("base_stats").is_none() && stored.get("growth").is_none(), "no stats beside the shares: {stored}");
    let keys: Vec<&str> = stored.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(&keys[..6], ["id", "name", "description", "art", "ai_role", "base_shares"], "{keys:?}");
    assert_eq!(keys[6], "growth_shares");
    let plain_stored = saved["entries"].as_array().unwrap().iter().find(|e| e["id"] == "plain").unwrap();
    assert!(plain_stored.get("base_shares").is_none() && plain_stored.get("growth_shares").is_none(), "{plain_stored}");

    let v = store::view(&data);
    let (drill, plain) = (&v.typed.classes["drill"], &v.typed.classes["plain"]);
    assert!(drill.shares.is_some());
    assert_eq!(drill.base_stats, plain.base_stats);
    assert_eq!(drill.growth.iter().collect::<Vec<_>>(), plain.growth.iter().collect::<Vec<_>>());
    for level in 1..=10 {
        assert_eq!(drill.stats_at(level), plain.stats_at(level), "level {level}");
    }
    assert_eq!(check::validate_entry(&v, "classes", v.get("classes", "drill"), "drill"), check::validate_entry(&v, "classes", v.get("classes", "plain"), "plain"));
    assert_eq!(store::resave_text(&data, None, "classes").unwrap(), text, "re-saves byte for byte");

    let mut bad = entry.clone();
    bad.insert("id".into(), json!("lopsided"));
    bad.insert("growth_shares".into(), json!({"STR": 0.5}));
    let refused = store::upsert(&data, None, "classes", &bad, None).err().expect("a vector that does not total 1 is refused");
    assert!(refused.iter().any(|e| e.contains("Growth shares total")), "{refused:?}");
}

#[test]
fn share_rows_are_checked_like_integer_rows() {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_class_shares");
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    write_kind(&data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": ints(&BASE), "growth": ints(&GROWTH)}]));
    let v = store::view(&data);
    let plain: Dict = v.get("classes", "drill").clone();
    let good = || json!({"base_shares": fractions(&BASE, 62), "growth_shares": fractions(&GROWTH, 5)});
    let none: Vec<String> = Vec::new();
    assert_eq!(added(&v, &plain, good()), none, "the integer row as shares");

    // a vector that does not total 1
    let mut short = good();
    short["base_shares"]["STR"] = json!(13.0 / 62.0 - 0.1);
    let errors = added(&v, &plain, short);
    assert!(errors.iter().any(|e| e.starts_with("Base shares total ") && e.ends_with("a share vector must total 1.")), "{errors:?}");
    let mut long = good();
    long["growth_shares"]["INT"] = json!(0.2);
    let errors = added(&v, &plain, long);
    assert!(errors.iter().any(|e| e.starts_with("Growth shares total ")), "{errors:?}");

    // a share out of 0-1, a key that is not a stat
    let mut over = good();
    over["growth_shares"] = json!({"STR": 1.5, "DEX": -0.5});
    let errors = added(&v, &plain, over);
    assert!(errors.contains(&"Growth shares STR must be a number from 0 to 1.".to_string()), "{errors:?}");
    let mut stray = good();
    stray["growth_shares"] = json!({"STR": 0.4, "CONS": 0.4, "LUCK": 0.2});
    assert_eq!(added(&v, &plain, stray), ["Growth shares: 'LUCK' is not a stat."]);

    // the two forms do not mix
    let mut both = good();
    both["base_stats"] = ints(&BASE);
    assert_eq!(added(&v, &plain, both), ["A class given as shares cannot also set base_stats."]);
    let errors = added(&v, &plain, json!({"base_stats": ints(&BASE), "growth": ints(&GROWTH), "growth_shares": fractions(&GROWTH, 5)}));
    assert_eq!(errors, ["Growth shares need base shares (a class gives both vectors or neither)."]);

    // the stats the shares give keep the limits: a base stat above 18, a level gaining 4 in one stat
    let mut tall = good();
    tall["base_shares"] = json!({"STR": 20.0 / 62.0, "DEX": 10.0 / 62.0, "INT": 6.0 / 62.0, "CONS": 6.0 / 62.0, "PERC": 10.0 / 62.0, "KNOW": 10.0 / 62.0});
    assert_eq!(added(&v, &plain, tall), ["Base STR 20 must be between 6 and 18."]);
    let mut steep = good();
    steep["growth_shares"] = json!({"STR": 0.8, "CONS": 0.2});
    assert_eq!(added(&v, &plain, steep), ["Growth STR must be 0-3 per level."]);
}
