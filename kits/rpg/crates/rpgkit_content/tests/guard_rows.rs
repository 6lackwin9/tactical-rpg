//! S1.2-S4 (docs/sessions/S1_STATUS.md section 1, multi-round guard): `guard_rounds` loads into the typed catalog (1
//! when absent) and belongs to DEFEND skills; AUDIT-07 registered it in the kit schema (an int 1-5, one value per
//! family), so the checks and the writer read it from there. Scratch data folders only.

use aegis_core::json::Dict;
use rpgkit_content::check;
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn scratch() -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_guard_rows");
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let skills = json!([
        {"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.2, "guard_rounds": 3, "description": "Braces.", "ranks": []},
        {"id": "t_guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3, "description": "Guards.", "ranks": []},
        {"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "description": "Jabs.", "ranks": []},
    ]);
    fs::write(data.join("catalog/skills.json"), json!({"kind": "skills", "version": 2, "entries": skills}).to_string()).unwrap();
    (t, data)
}

fn guard_errors(v: &check::View, id: &str, change: Value) -> Vec<String> {
    let mut e: Dict = v.get("skills", id).clone();
    for (k, x) in change.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, "skills", &e, id).into_iter().filter(|m| m.contains("Guard rounds")).collect()
}

#[test]
fn guard_rounds_loads_and_is_checked_on_defend_rows() {
    let (_t, data) = scratch();
    let src = source::load(&data);
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let rounds = |id: &str| cat.skills.iter().find(|s| s.id == id).map(|s| s.guard_rounds);
    assert_eq!([rounds("t_brace"), rounds("t_guard"), rounds("t_jab")], [Some(3), Some(1), Some(1)], "the loader keeps the key; 1 when absent");
    let v = store::view(&data);
    assert!(guard_errors(&v, "t_brace", json!({})).is_empty());
    assert!(guard_errors(&v, "t_guard", json!({"guard_rounds": 5})).is_empty());
    // AUDIT-07: the schema entry's range clamps on normalize (as every int field's), so no row error past it
    assert!(guard_errors(&v, "t_guard", json!({"guard_rounds": 6})).is_empty());
    assert_eq!(guard_errors(&v, "t_jab", json!({"guard_rounds": 2})), ["Guard rounds belongs to DEFEND skills, not ATTACK."]);
}

/// AUDIT-07: `guard_rounds` is a schema field (S1.2-S4): an int 1-5, default 1, shown for DEFEND skills in the guard
/// section, one value for the whole family (not per rank: the loader's rank keys leave it out); a row out of range loads
/// clamped; the writer stores it as no key at its default (a row written before the entry re-saves byte for byte) and
/// at its schema place otherwise.
#[test]
fn guard_rounds_is_a_schema_field_stored_only_off_its_default() {
    let f: Dict = source::schema()["content"]["skills"].as_array().unwrap().iter().filter_map(Value::as_object).find(|f| f["key"] == "guard_rounds").cloned().expect("the schema entry");
    assert_eq!((f["type"].clone(), f["default"].clone(), f["min"].clone(), f["max"].clone()), (json!("int"), json!(1), json!(1), json!(5)));
    assert_eq!((f["when"].clone(), f["section"].clone()), (json!({"key": "kind", "in": ["DEFEND"]}), json!(["guard"])));
    assert!(!f.get("rank").and_then(Value::as_bool).unwrap_or(false), "one value per family");
    let (t, data) = scratch();
    let wide = json!([{"id": "t_wall", "name": "Wall", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.2, "guard_rounds": 9, "description": "Walls.", "ranks": []}]);
    fs::write(data.join("catalog/skills.json"), json!({"kind": "skills", "version": 2, "entries": wide}).to_string()).unwrap();
    let cat = rpgkit_data::Catalog::from_value(&source::load(&data).catalog);
    assert_eq!(cat.skills.iter().find(|s| s.id == "t_wall").map(|s| s.guard_rounds), Some(5), "clamped to the schema max");
    drop(t);
    let row = |extra: Value| {
        let mut e = json!({"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.2, "description": "Braces."});
        e.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        store::catalog_storage("skills", e.as_object().unwrap(), &[]).unwrap()
    };
    assert!(!row(json!({})).contains_key("guard_rounds") && !row(json!({"guard_rounds": 1})).contains_key("guard_rounds"), "no key at the default");
    let stored = row(json!({"guard_rounds": 3}));
    let keys: Vec<&String> = stored.keys().collect();
    let at = |k: &str| keys.iter().position(|x| *x == k).unwrap_or_else(|| panic!("{k} stored"));
    assert_eq!(stored["guard_rounds"], json!(3));
    assert_eq!(at("guard_rounds"), at("mitigation") + 1, "stored at its schema place, after mitigation: {keys:?}");
}

/// AUDIT-11 (S1.2-S4, FB-85): the loader tells an authored `mitigation` from the schema default: a DEFEND entry that
/// authors none (neither the entry nor the rank's modifier) loads without the key, so the battle takes the ruleset's
/// `guard_common_mitigation` (1-round) or `guard_multi_round_mitigation` (multi-round) for it; an authored value (the
/// schema default's value included) and a rank modifier's value stay; non-DEFEND rows keep the normalised key; the
/// writer leaves an unauthored key out (`an_authored_default_mitigation_survives_a_resave` covers the authored one).
#[test]
fn an_unauthored_mitigation_loads_as_the_ruleset_default() {
    let (_t, data) = scratch();
    let default = source::schema()["content"]["skills"].as_array().unwrap().iter().find(|f| f["key"] == "mitigation").and_then(|f| f["default"].as_f64()).expect("the schema default");
    let skills = json!([
        {"id": "t_stand", "name": "Stand", "kind": "DEFEND", "level": 1, "blocks": 1, "description": "Stands.", "ranks": []},
        {"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 2, "guard_rounds": 3, "description": "Braces.", "ranks": []},
        {"id": "t_plain", "name": "Plain", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": default, "description": "Plain.", "ranks": []},
        {"id": "t_climb", "name": "Climb", "kind": "DEFEND", "level": 1, "blocks": 1, "description": "Climbs.", "ranks": [{"mitigation": 0.4}, {"blocks": 2}]},
        {"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "description": "Jabs.", "ranks": []},
    ]);
    fs::write(data.join("catalog/skills.json"), json!({"kind": "skills", "version": 2, "entries": skills}).to_string()).unwrap();
    let src = source::load(&data);
    let rows: Vec<&Dict> = src.catalog["skills"].as_array().unwrap().iter().filter_map(Value::as_object).collect();
    let has = |id: &str| rows.iter().find(|r| r["id"] == id).unwrap_or_else(|| panic!("{id}")).contains_key("mitigation");
    assert_eq!(
        ["t_stand", "t_brace", "t_plain", "t_climb", "t_climb_2", "t_climb_3", "t_jab"].map(has),
        [false, false, true, false, true, false, true],
        "authored keys stay (entry or the rank's own modifier); rank 3 inherits no rank 2 modifier"
    );
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let n = rpgkit_rules::ruleset::Numbers::default();
    let fraction = |id: &str| rpgkit_battle::skill::guard_mitigation(cat.skills.iter().find(|s| s.id == id).unwrap(), &n);
    let later = |id: &str| rpgkit_battle::skill::guard_later_mitigation(cat.skills.iter().find(|s| s.id == id).unwrap(), &n);
    assert_eq!(
        [fraction("t_stand"), fraction("t_brace"), fraction("t_plain"), fraction("t_climb"), fraction("t_climb_2")],
        [n.guard_common_mitigation, n.guard_multi_round_mitigation, default, n.guard_common_mitigation, 0.4]
    );
    assert_eq!([later("t_brace"), later("t_plain")], [n.guard_multi_round_mitigation, default], "Round 2+: the ruleset's unless authored");
    let entry = skills[0].as_object().unwrap();
    assert!(!store::catalog_storage("skills", entry, &[]).unwrap().contains_key("mitigation"), "no key when none was authored");
}

/// AUDIT-11 addition (FB-94): the writer keeps a DEFEND row's authored `mitigation` even at the schema default (and a
/// rank modifier's even at rank 1's value), so an authored default saves with the key and loads back as itself, not as
/// the ruleset's guard fraction; an unauthored row still saves and loads without it; non-DEFEND rows are unchanged.
#[test]
fn an_authored_default_mitigation_survives_a_resave() {
    let (_t, data) = scratch();
    let default = source::schema()["content"]["skills"].as_array().unwrap().iter().find(|f| f["key"] == "mitigation").and_then(|f| f["default"].as_f64()).expect("the schema default");
    let skills = json!([
        {"id": "t_stand", "name": "Stand", "kind": "DEFEND", "level": 1, "blocks": 1, "description": "Stands.", "ranks": []},
        {"id": "t_plain", "name": "Plain", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": default, "description": "Plain.", "ranks": []},
        {"id": "t_step", "name": "Step", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.3, "description": "Steps.", "ranks": [{"mitigation": 0.3}]},
        {"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "mitigation": default, "description": "Jabs.", "ranks": []},
    ]);
    fs::write(data.join("catalog/skills.json"), json!({"kind": "skills", "version": 2, "entries": skills}).to_string()).unwrap();
    let saved = store::resave_text(&data, None, "skills").unwrap();
    fs::write(data.join("catalog/skills.json"), &saved).unwrap();
    let doc: Value = serde_json::from_str(&saved).unwrap();
    let stored = |id: &str| doc["entries"].as_array().unwrap().iter().find(|e| e["id"] == id).unwrap_or_else(|| panic!("{id}")).clone();
    assert_eq!(stored("t_plain")["mitigation"], json!(default), "the authored default is saved");
    assert!(stored("t_stand").get("mitigation").is_none(), "an unauthored row saves no key");
    assert!(stored("t_jab").get("mitigation").is_none(), "non-DEFEND rows: no key at the default");
    assert_eq!(stored("t_step")["ranks"][0]["mitigation"], json!(0.3), "a rank modifier's authored value stays");
    let cat = rpgkit_data::Catalog::from_value(&source::load(&data).catalog);
    let n = rpgkit_rules::ruleset::Numbers::default();
    let fraction = |id: &str| rpgkit_battle::skill::guard_mitigation(cat.skills.iter().find(|s| s.id == id).unwrap(), &n);
    assert_eq!([fraction("t_plain"), fraction("t_stand"), fraction("t_step_2")], [default, n.guard_common_mitigation, 0.3]);
    assert_eq!(store::resave_text(&data, None, "skills").unwrap(), saved, "the saved form re-saves byte for byte");
}
