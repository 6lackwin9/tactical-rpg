//! PK1-KITFIX (c) (FB-184 Option A): the catalog kind `dojo_presets` (the Dojo's quick loadouts as data: gear per slot,
//! pouch items and passives) on a scratch data folder built here (no shipped data). A valid row loads into the typed
//! catalog (gear in loadout slot order) and checks clean; each broken rule is one error, at the Forge's check and at
//! load (the row is skipped); the writer stores the defaults as no key and re-saves a written file byte for byte.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{self, dojo_preset};
use rpgkit_content::{source, store};
use rpgkit_data::{DOJO_PASSIVES_MAX, DOJO_POUCH_MAX, DOJO_PRESETS_KIND, LOADOUT_SLOTS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_dojo_presets_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().unwrap().clone()
}

/// A one-handed sword (`t_blade`), a consumable (`t_salt`), two passives and, when `rows`, the Dojo preset rows.
fn catalog(data: &Path, rows: Option<Value>) {
    let band = |lo: i64, hi: i64| json!({"min": lo, "max": hi});
    write_kind(
        data,
        "item_families",
        1,
        json!([
            {"id": "sword", "name": "Sword", "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"],
             "base": {"main_1h": {"power": 14, "range": 1, "block_cost": 1, "burden": 0, "mitigation": 0}},
             "bands": {"main_1h": {"power": band(10, 23), "range": band(1, 2), "block_cost": band(1, 3), "burden": band(0, 4), "mitigation": band(0, 0), "bonus_value": band(0, 4)}},
             "adjustable": ["power"], "modifier_kinds": ["adjust", "stat_bonus", "requirement", "effect"], "max_modifiers": 8, "max_tiers": 3, "damage_type": "slash"},
            {"id": "pouch", "name": "Pouch", "item_type": "CONSUMABLE"}
        ]),
    );
    write_kind(data, "items", 2, json!([
        {"id": "t_blade", "name": "Test Blade", "family": "sword", "hand": "main_1h"},
        {"id": "t_salt", "name": "Salt", "family": "pouch", "consumable_type": "HEALTH_POTION", "heal": 5}
    ]));
    write_kind(data, "passives", 1, json!([
        {"id": "p_meter", "name": "Meter Test", "effects": [{"type": "stat", "stat": "STR", "value": 1}]},
        {"id": "p_other", "name": "Other Test", "effects": [{"type": "stat", "stat": "DEX", "value": 1}]}
    ]));
    if let Some(rows) = rows {
        write_kind(data, DOJO_PRESETS_KIND, 1, rows);
    }
}

/// The report's lines about the Dojo presets (their rows, and load lines naming one).
fn ours(report: &check::Report) -> Vec<String> {
    report.iter().flat_map(|(k, list)| list.iter().filter(move |m| k.starts_with(DOJO_PRESETS_KIND) || m.contains("Dojo preset")).map(move |m| format!("{k}: {m}"))).collect()
}

fn duel() -> Value {
    json!({"id": "t_duel", "name": "Test Duel", "description": "A blade, a pouch and a passive", "gear": {"main_hand": "t_blade"}, "pouch": ["t_salt"], "passives": ["p_meter"]})
}

fn errors_of(v: &check::View, change: &Value) -> Vec<String> {
    let mut e = v.get(DOJO_PRESETS_KIND, "t_duel").clone();
    assert!(!e.is_empty(), "fixture t_duel");
    for (k, x) in change.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, DOJO_PRESETS_KIND, &e, "t_duel")
}

#[test]
fn the_kind_is_in_the_schema_and_the_typed_catalog() {
    let keys: Vec<String> = fields(source::schema(), "content", DOJO_PRESETS_KIND).iter().map(|f| json::s(json::dict(f), "key")).collect();
    assert_eq!(keys, ["id", "name", "description", "gear", "pouch", "passives", "origin"]);
    assert_eq!(source::DOJO_PRESETS_KIND, DOJO_PRESETS_KIND);
    let gear = fields(source::schema(), "content", DOJO_PRESETS_KIND).iter().map(json::dict).find(|f| json::s(f, "key") == "gear").unwrap().clone();
    for slot in LOADOUT_SLOTS {
        assert!(json::s(&gear, "help").contains(slot), "the gear help names {slot}");
    }
}

#[test]
fn a_row_loads_into_the_catalog_and_checks_clean() {
    let (_scratch, data) = scratch("clean");
    catalog(&data, Some(json!([duel(), {"id": "t_bare", "name": "Bare Hands"}])));
    let src = source::load(&data);
    let catalog_errors: Vec<&String> = src.errors.iter().filter(|e| e.contains("catalog")).collect();
    assert!(catalog_errors.is_empty(), "{catalog_errors:?}");
    let v = store::view(&data);
    let p = &v.typed.dojo_presets["t_duel"];
    assert_eq!((p.name.as_str(), p.description.as_str()), ("Test Duel", "A blade, a pouch and a passive"));
    assert_eq!(p.gear, vec![("main_hand".to_string(), "t_blade".to_string())]);
    assert_eq!((p.pouch.clone(), p.passives.clone()), (vec!["t_salt".to_string()], vec!["p_meter".to_string()]));
    let bare = &v.typed.dojo_presets["t_bare"];
    assert!(bare.gear.is_empty() && bare.pouch.is_empty() && bare.passives.is_empty(), "every list defaults empty");
    assert_eq!(v.typed.dojo_presets.keys().cloned().collect::<Vec<_>>(), ["t_duel", "t_bare"], "file order");
    let report = check::validate_all(&v);
    assert!(ours(&report).is_empty(), "{:?}", ours(&report));
    for id in ["t_duel", "t_bare"] {
        assert_eq!(check::validate_entry(&v, DOJO_PRESETS_KIND, v.get(DOJO_PRESETS_KIND, id), id), Vec::<String>::new(), "{id}");
    }
}

#[test]
fn no_file_is_an_empty_table() {
    let (_scratch, data) = scratch("none");
    catalog(&data, None);
    let v = store::view(&data);
    assert!(v.typed.dojo_presets.is_empty());
    assert!(source::load(&data).errors.iter().all(|e| !e.contains("Dojo preset")));
}

#[test]
fn each_broken_rule_is_one_error() {
    let (_scratch, data) = scratch("rules");
    catalog(&data, Some(json!([duel()])));
    let v = store::view(&data);
    let too_many_pouch: Vec<&str> = vec!["t_salt"; DOJO_POUCH_MAX + 1];
    let too_many_passives: Vec<String> = (0..=DOJO_PASSIVES_MAX).map(|i| if i % 2 == 0 { "p_meter".into() } else { "p_other".into() }).collect();
    let cases: Vec<(Value, &str)> = vec![
        (json!({}), ""),
        (json!({"gear": {}, "pouch": [], "passives": []}), ""),
        (json!({"gear": {"off_hand": "t_blade"}}), ""),
        (json!({"name": ""}), "Name must be 1-32 characters."),
        (json!({"name": "x".repeat(33)}), "Name must be 1-32 characters."),
        (json!({"gear": {"belt": "t_blade"}}), "Gear belt: unknown slot"),
        (json!({"gear": {"main_hand": "t_nothing"}}), "Gear main_hand: unknown item 't_nothing'."),
        (json!({"gear": {"body": "t_blade"}}), "Gear body: 't_blade' does not fit the body slot."),
        (json!({"gear": {"main_hand": "t_salt"}}), "Gear main_hand: 't_salt' does not fit the main_hand slot."),
        (json!({"gear": ["t_blade"]}), "Gear must be an object"),
        (json!({"pouch": ["t_nothing"]}), "Pouch: unknown item 't_nothing'."),
        (json!({"pouch": ["t_blade"]}), "Pouch: 't_blade' is not a consumable."),
        (json!({"pouch": too_many_pouch}), "Pouch holds up to 3 items"),
        (json!({"passives": ["p_nothing"]}), "Passives: unknown passive 'p_nothing'."),
        (json!({"passives": ["p_meter", "p_meter"]}), "Passives: 'p_meter' is listed twice."),
        (json!({"origin": "shipped"}), "Origin: 'shipped' is not one of system, user."),
    ];
    let mut failures = Vec::new();
    for (change, want) in &cases {
        let errors = errors_of(&v, change);
        let ok = if want.is_empty() { errors.is_empty() } else { errors.len() == 1 && errors[0].contains(want) };
        if !ok {
            failures.push(format!("{change}: want {want:?}, got {errors:?}"));
        }
    }
    // the passives' cap (the repeats are their own errors; only the count message is asked for here)
    let errors = errors_of(&v, &json!({"passives": too_many_passives}));
    assert!(errors.iter().any(|e| e.contains(&format!("Passives holds up to {DOJO_PASSIVES_MAX} passives"))), "{errors:?}");
    // a second row with the same id
    let mut twin = v.get(DOJO_PRESETS_KIND, "t_duel").clone();
    twin.insert("name".into(), json!("Twin"));
    let errors = check::validate_entry(&v, DOJO_PRESETS_KIND, &twin, "");
    assert!(errors.iter().any(|e| e == "ID 't_duel' is already used by another Dojo preset."), "{errors:?}");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The loader checks each row as the Forge does and skips a broken one (its error names the file and the row).
#[test]
fn the_loader_skips_a_broken_row() {
    let (_scratch, data) = scratch("skip");
    catalog(&data, Some(json!([duel(), {"id": "t_bad", "name": "Bad", "gear": {"main_hand": "t_nothing"}}])));
    let src = source::load(&data);
    assert!(src.errors.iter().any(|e| e.contains(&format!("{DOJO_PRESETS_KIND}.json: Dojo preset 't_bad': Gear main_hand: unknown item 't_nothing'."))), "{:?}", src.errors);
    let v = store::view(&data);
    assert!(v.typed.dojo_presets.contains_key("t_duel") && !v.typed.dojo_presets.contains_key("t_bad"));
    // the rows module checks the same against the loaded tables
    let rows = [dict(json!({"id": "t_bad", "name": "Bad", "origin": "system", "gear": {"main_hand": "t_nothing"}}))];
    assert_eq!(dojo_preset::check_rows(&rows, v.entries("items"), v.entries("passives")), vec![vec!["Gear main_hand: unknown item 't_nothing'.".to_string()]]);
}

/// The writer stores the schema's fields in schema order (as every generic kind: a shipped `system` origin as no key),
/// and a file it wrote re-saves byte for byte.
#[test]
fn the_writer_stores_the_schema_fields_and_resaves_byte_for_byte() {
    let stored = store::catalog_storage(DOJO_PRESETS_KIND, &dict(json!({"id": "t_bare", "name": "Bare"})), &[]).unwrap();
    assert_eq!(stored.keys().cloned().collect::<Vec<_>>(), ["id", "name", "description", "gear", "pouch", "passives"]);
    let user = store::catalog_storage(DOJO_PRESETS_KIND, &dict(json!({"id": "t_bare", "name": "Bare", "origin": "user"})), &[]).unwrap();
    assert_eq!(user.get("origin"), Some(&json!("user")));
    let (_scratch, data) = scratch("resave");
    catalog(&data, None);
    for row in [duel(), json!({"id": "t_bare", "name": "Bare Hands", "origin": "user"})] {
        let row = dict(row);
        let out = store::upsert(&data, None, DOJO_PRESETS_KIND, &row, None);
        assert!(out.is_ok(), "{:?}", out.err());
    }
    let path = data.join("catalog").join(format!("{DOJO_PRESETS_KIND}.json"));
    let written = fs::read_to_string(&path).unwrap();
    assert_eq!(store::resave_text(&data, None, DOJO_PRESETS_KIND).unwrap(), written, "a written file re-saves byte for byte");
    let v = store::view(&data);
    assert_eq!(v.typed.dojo_presets["t_duel"].passives, vec!["p_meter".to_string()], "the written row loads");
    assert!(ours(&check::validate_all(&v)).is_empty());
}
