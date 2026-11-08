//! R4-1 `U2.3-TIERS` slice 1: the catalog kind `unit_templates` (scratch data): a missing file is an empty table; a
//! row's checks (role, id, origin, both share vectors as a class's: totalling 1, the level 1 stats 6-18, 0-3 gained per
//! level) and across rows (unique ids, one row per role); the loader keeps the rows that pass; the writer takes a valid
//! row, re-saves it byte for byte and refuses a bad one; the loaded row reaches `Catalog::unit_templates` by role.

use aegis_core::json::{self, Dict};
use rpgkit_content::check::unit_template::{options, NO_SHARES};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, UNIT_TEMPLATES_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_unit_templates_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn write_rows(data: &Path, entries: Value) {
    fs::write(data.join("catalog").join(format!("{UNIT_TEMPLATES_KIND}.json")), json!({"kind": UNIT_TEMPLATES_KIND, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

/// A valid row: the level 1 stats 15.5 / 9.3 / 6.2 / 15.5 / 9.3 / 6.2 of 62, rounded by largest remainder, all 6-18.
fn brute(id: &str, role: &str) -> Value {
    json!({"id": id, "name": "Brute", "role": role,
        "base_shares": {"STR": 0.25, "DEX": 0.15, "INT": 0.1, "CONS": 0.25, "PERC": 0.15, "KNOW": 0.1},
        "growth_shares": {"STR": 0.4, "DEX": 0.2, "CONS": 0.4}})
}

fn with(mut row: Value, key: &str, v: Value) -> Value {
    row[key] = v;
    row
}

#[test]
fn a_missing_file_is_an_empty_table() {
    let (_scratch, data) = scratch("missing");
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[UNIT_TEMPLATES_KIND]).is_empty());
    assert!(src.errors.iter().all(|e| !e.contains(&format!("{UNIT_TEMPLATES_KIND}.json"))), "{:?}", src.errors);
    assert!(Catalog::from_value(&src.catalog).unit_templates.is_empty());
    drop_scratch(&data);
}

#[test]
fn the_roles_are_the_ai_role_options() {
    let schema = source::schema();
    let role_of = |kind: &str| -> Vec<String> {
        let f = schema["content"][kind].as_array().unwrap().iter().find(|f| f["key"] == "ai_role").unwrap();
        f["options"].as_array().unwrap().iter().map(|o| o.as_str().unwrap().to_string()).filter(|o| !o.is_empty()).collect()
    };
    assert_eq!(options("role"), role_of("classes"), "the class's ai_role options");
    assert_eq!(options("role"), role_of("units"), "the unit's ai_role options without ''");
}

#[test]
fn each_mistake_has_its_text() {
    let (_scratch, data) = scratch("texts");
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, UNIT_TEMPLATES_KIND, &dict(e), "");
    assert_eq!(check(brute("brute", "AGGRESSOR")), Vec::<String>::new());
    let off = check(with(brute("brute", "AGGRESSOR"), "base_shares", json!({"STR": 0.5, "DEX": 0.15, "INT": 0.1, "CONS": 0.25})));
    assert!(off.contains(&"Base PERC 0 must be between 6 and 18.".to_string()), "a stat left out is 0, below a base row's floor: {off:?}");
    let sum = check(with(brute("brute", "AGGRESSOR"), "growth_shares", json!({"STR": 0.5, "CONS": 0.4})));
    assert_eq!(sum, ["Growth shares total 0.9000, a share vector must total 1."]);
    // 0.4 x 62 = 24.8: STR 25, above the 18 ceiling of a base row
    let heavy = json!({"STR": 0.4, "DEX": 0.12, "INT": 0.12, "CONS": 0.12, "PERC": 0.12, "KNOW": 0.12});
    let high = check(with(brute("brute", "AGGRESSOR"), "base_shares", heavy));
    assert!(high.iter().any(|e| e.starts_with("Base STR ") && e.ends_with("must be between 6 and 18.")), "{high:?}");
    let empty = check(json!({"id": "blank", "role": "SUPPORT"}));
    assert_eq!(empty, [NO_SHARES]);
    let bad_id = check(brute("B", "AGGRESSOR"));
    assert!(bad_id.iter().any(|e| e.starts_with("ID 'B'")), "{bad_id:?}");
    drop_scratch(&data);
}

#[test]
fn the_loader_keeps_the_rows_that_pass() {
    let (_scratch, data) = scratch("loader");
    write_rows(&data, json!([brute("brute", "AGGRESSOR"), brute("brute", "SNIPER"), brute("thug", "AGGRESSOR"), with(brute("lean", "SUPPORT"), "growth_shares", json!({"STR": 1.5}))]));
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[UNIT_TEMPLATES_KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(kept, ["brute"], "the refused rows are skipped");
    let found: Vec<&String> = src.errors.iter().filter(|e| e.contains("unit template '")).collect();
    assert!(found.iter().any(|e| e.ends_with("unit template 'brute': ID 'brute' is already used by another unit template.")), "{found:?}");
    assert!(found.iter().any(|e| e.ends_with("unit template 'thug': Role AGGRESSOR already has the unit template 'brute'.")), "{found:?}");
    assert!(found.iter().any(|e| e.contains("unit template 'lean': Growth shares STR must be a number from 0 to 1.")), "{found:?}");
    let cat = Catalog::from_value(&src.catalog);
    assert_eq!(cat.unit_templates.keys().collect::<Vec<_>>(), ["AGGRESSOR"], "keyed by role");
    assert_eq!(cat.unit_templates["AGGRESSOR"].shares.base["STR"], 0.25);
    let v = store::view(&data);
    assert_eq!(check::validate_entry(&v, UNIT_TEMPLATES_KIND, &dict(brute("brute", "AGGRESSOR")), "brute"), Vec::<String>::new(), "an edit of the row itself");
    assert_eq!(check::validate_entry(&v, UNIT_TEMPLATES_KIND, &dict(brute("thug", "AGGRESSOR")), ""), ["Role AGGRESSOR already has the unit template 'brute'."]);
    drop_scratch(&data);
}

#[test]
fn the_writer_takes_a_row_and_refuses_a_bad_one() {
    let (_scratch, data) = scratch("writer");
    let file = data.join("catalog").join(format!("{UNIT_TEMPLATES_KIND}.json"));
    let refused = store::upsert(&data, None, UNIT_TEMPLATES_KIND, &dict(with(brute("brute", "AGGRESSOR"), "growth_shares", json!({"STR": 0.5}))), None).err().expect("a bad row is refused");
    assert!(refused.iter().any(|e| e.ends_with("Growth shares total 0.5000, a share vector must total 1.")), "{refused:?}");
    assert!(!file.exists(), "a refused first row leaves no file");
    store::upsert(&data, None, UNIT_TEMPLATES_KIND, &dict(brute("brute", "AGGRESSOR")), None).expect("a valid row");
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, UNIT_TEMPLATES_KIND).unwrap(), text, "a re-save without changes is byte-identical");
    assert!(store::check(&data).iter().all(|p| !p.contains("unit template '")), "{:?}", store::check(&data));
    let cat = Catalog::from_value(&source::load(&data).catalog);
    assert_eq!(cat.unit_templates["AGGRESSOR"].id, "brute");
    store::remove(&data, None, UNIT_TEMPLATES_KIND, "brute").unwrap();
    assert!(Catalog::from_value(&source::load(&data).catalog).unit_templates.is_empty());
    drop_scratch(&data);
}
