//! PORTRAITFX1-EX's catalog kind `portrait_expressions` (PORTRAITFX1.md 16; scratch data): a missing file is an empty
//! table; each error text, one per mistake; the loader keeps the rows that pass and reports the rest; the writer stores
//! only the fields that differ from the schema defaults, re-saves byte for byte and refuses a bad row; a row that draws
//! nothing warns; the loaded rows reach `Catalog::portrait_expressions` with the schema defaults.

use aegis_core::json::{self, Dict};
use rpgkit_content::check::expression_fx::{self, ANCHOR, DRAWS_NOTHING, MARK_SIZE, PULSE_MS, SHAKE_MS, SHAKE_PX_MAX, STRENGTH, TINT_MS};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, EXPRESSIONS, PORTRAIT_EXPRESSIONS_KIND as KIND, PORTRAIT_MARKS, PORTRAIT_MARK_BOXES};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// A scratch data folder `<tmp>/<tag>/data` with empty catalog and worlds folders.
fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_expression_fx_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

/// The expressions a row may name (every one but the plain face).
fn named() -> String {
    EXPRESSIONS.iter().filter(|e| **e != expression_fx::NEUTRAL).copied().collect::<Vec<_>>().join(", ")
}

/// The schema default of `key` in the kind's field table.
fn default_of(key: &str) -> Value {
    let f = source::schema()["content"][KIND].as_array().unwrap().iter().find(|f| f["key"] == key).unwrap_or_else(|| panic!("a field {key}"));
    f["default"].clone()
}

#[test]
fn a_missing_file_is_an_empty_table() {
    let data = scratch("missing");
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[KIND]).is_empty(), "the kind loads as no rows");
    assert!(src.errors.iter().all(|e| !e.contains(&format!("{KIND}.json"))), "{:?}", src.errors);
    assert!(Catalog::from_value(&src.catalog).portrait_expressions.is_empty());
    drop_scratch(&data);
}

#[test]
fn the_schema_options_are_the_kit_lists() {
    let options = |key: &str| -> Vec<String> {
        let f = source::schema()["content"][KIND].as_array().unwrap().iter().find(|f| f["key"] == key).unwrap().clone();
        f["options"].as_array().unwrap().iter().map(|o| o.as_str().unwrap().to_string()).collect()
    };
    assert_eq!(options("mark"), std::iter::once("").chain(PORTRAIT_MARKS).collect::<Vec<_>>());
    assert_eq!(options("mark_box"), PORTRAIT_MARK_BOXES);
    // the typed reader's defaults are the schema's
    let row = rpgkit_data::PortraitExpression::from_obj(&Dict::new());
    assert_eq!(json!(row.tint_strength), default_of("tint_strength"));
    assert_eq!(json!(row.mark_box), default_of("mark_box"));
    assert_eq!((json!(row.mark_x), json!(row.mark_y), json!(row.mark_size)), (default_of("mark_x"), default_of("mark_y"), default_of("mark_size")));
    assert_eq!((json!(row.tint_ms), json!(row.mark_pulse_ms), json!(row.shake_px), json!(row.shake_ms)), (default_of("tint_ms"), default_of("mark_pulse_ms"), default_of("shake_px"), default_of("shake_ms")));
}

#[test]
fn each_mistake_has_its_text() {
    let data = scratch("texts");
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, KIND, &dict(e), "");
    let ok = Vec::<String>::new();
    assert_eq!(check(json!({"id": "angry", "tint": "#e11d48", "mark": "vein", "mark_pulse_ms": 900})), ok, "an angry row");
    assert_eq!(check(json!({"id": "hurt", "tint": "#e11d48", "tint_ms": 240, "shake_px": 2, "shake_ms": 220})), ok, "a hurt row");
    assert_eq!(check(json!({"id": "afraid", "mark": "sweat", "mark_box": "eyes", "mark_x": ANCHOR.0, "mark_y": ANCHOR.1})), ok, "the anchor's ends");
    assert_eq!(check(json!({"id": ""})), [format!("ID is required: one of {}.", named())]);
    assert_eq!(check(json!({"id": "neutral", "tint": "#e11d48"})), [format!("'neutral' is the plain face: it takes no effect row (one of {}).", named())]);
    assert_eq!(check(json!({"id": "smug", "tint": "#e11d48"})), [format!("ID 'smug' is not an expression (one of {}).", named())]);
    assert_eq!(check(json!({"id": "angry", "tint": "red"})), ["Tint 'red' must be \"\" or #rrggbb."]);
    assert_eq!(check(json!({"id": "angry", "tint": "#e11d48", "tint_strength": 1.5})), [format!("Tint strength must be {:?}-{:?} (is 1.5).", STRENGTH.0, STRENGTH.1)]);
    assert_eq!(check(json!({"id": "angry", "tint": "#e11d48", "tint_ms": 10})), [format!("Tint ms must be 0 (it holds) or {}-{} (is 10).", TINT_MS.0, TINT_MS.1)]);
    assert_eq!(check(json!({"id": "angry", "mark": "vein", "tint_ms": 100})), ["Tint ms is for a row with a tint."]);
    assert_eq!(check(json!({"id": "angry", "mark": "star"})), [format!("Mark: 'star' is not one of {}.", PORTRAIT_MARKS.join(", "))]);
    assert_eq!(check(json!({"id": "angry", "tint": "#e11d48", "mark_color": "#ffffff"})), ["Mark colour is for a row with a mark."]);
    assert_eq!(check(json!({"id": "angry", "mark": "vein", "mark_box": "nose"})), [format!("Mark box: 'nose' is not one of {}.", PORTRAIT_MARK_BOXES.join(", "))]);
    assert_eq!(check(json!({"id": "angry", "mark": "vein", "mark_x": 2.5})), [format!("Mark x must be {:?}-{:?} (is 2.5).", ANCHOR.0, ANCHOR.1)]);
    assert_eq!(check(json!({"id": "angry", "mark": "vein", "mark_size": 0.5})), [format!("Mark size must be {:?}-{:?} (is 0.5).", MARK_SIZE.0, MARK_SIZE.1)]);
    assert_eq!(check(json!({"id": "angry", "mark": "vein", "mark_pulse_ms": 100})), [format!("Mark pulse ms must be 0 (steady) or {}-{} (is 100).", PULSE_MS.0, PULSE_MS.1)]);
    assert_eq!(check(json!({"id": "angry", "tint": "#e11d48", "mark_pulse_ms": 900})), ["Mark pulse ms is for a row with a mark."]);
    assert_eq!(check(json!({"id": "hurt", "shake_px": SHAKE_PX_MAX + 1, "shake_ms": 200})), [format!("Shake px must be 0-{SHAKE_PX_MAX} (is {}).", SHAKE_PX_MAX + 1)]);
    assert_eq!(check(json!({"id": "hurt", "shake_px": 2})), [format!("Shake ms must be {}-{} with a shake (is 0).", SHAKE_MS.0, SHAKE_MS.1)]);
    assert_eq!(check(json!({"id": "hurt", "tint": "#e11d48", "shake_ms": 200})), ["Shake ms must be 0 without a shake (is 200)."]);
    drop_scratch(&data);
}

#[test]
fn the_loader_keeps_the_rows_that_pass_and_warns_on_an_empty_row() {
    let data = scratch("loader");
    let entries = json!([{"id": "angry", "tint": "#e11d48"}, {"id": "neutral", "tint": "#e11d48"}, {"id": "angry", "mark": "vein"}, {"id": "afraid"}]);
    fs::write(data.join("catalog").join(format!("{KIND}.json")), json!({"kind": KIND, "version": 1, "entries": entries}).to_string()).unwrap();
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(kept, ["angry", "afraid"], "the refused rows are skipped");
    let found: Vec<&String> = src.errors.iter().filter(|e| e.contains("expression effect '")).collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].contains("expression effect 'neutral': 'neutral' is the plain face"), "{found:?}");
    assert!(found[1].ends_with("expression effect 'angry': ID 'angry' is already used by another expression effect."), "{found:?}");
    let v = store::view(&data);
    let warnings = check::warnings_all(&v);
    assert_eq!(warnings.get(&format!("{KIND}:afraid")).cloned().unwrap_or_default(), [DRAWS_NOTHING]);
    assert!(!warnings.contains_key(&format!("{KIND}:angry")));
    assert_eq!(check::validate_entry(&v, KIND, &dict(json!({"id": "angry", "mark": "vein"})), "angry"), Vec::<String>::new(), "an edit of the row itself");
    drop_scratch(&data);
}

#[test]
fn the_writer_stores_what_differs_and_resaves_byte_for_byte() {
    let data = scratch("writer");
    let file = data.join("catalog").join(format!("{KIND}.json"));
    let refused = store::upsert(&data, None, KIND, &dict(json!({"id": "neutral", "tint": "#e11d48"})), None).err().expect("a neutral row is refused");
    assert!(refused.iter().any(|e| e.contains("'neutral' is the plain face")), "{refused:?}");
    assert!(!file.exists(), "a refused first row leaves no file");
    let angry = json!({"id": "angry", "tint": "#e11d48", "tint_strength": 0.35, "mark": "vein", "mark_color": "#e11d48", "mark_pulse_ms": 900});
    store::upsert(&data, None, KIND, &dict(angry.clone()), None).expect("an angry row");
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, KIND).unwrap(), text, "a re-save without changes is byte-identical");
    let stored = json::dict(&json::arr(&serde_json::from_str::<Value>(&text).unwrap()["entries"])[0]).clone();
    let keys: Vec<&str> = stored.keys().map(String::as_str).collect();
    assert_eq!(keys, ["id", "tint", "tint_strength", "mark", "mark_color", "mark_pulse_ms"], "only the keys that differ from the defaults (in schema order)");
    let cat = Catalog::from_value(&source::load(&data).catalog);
    let row = &cat.portrait_expressions[0];
    assert_eq!((row.id.as_str(), row.tint.as_str(), row.mark.as_str(), row.mark_pulse_ms), ("angry", "#e11d48", "vein", 900));
    assert!((row.tint_strength - 0.35).abs() < 1e-4, "normalize snaps floats to 0.0001: {}", row.tint_strength);
    assert!((row.mark_x - default_of("mark_x").as_f64().unwrap()).abs() < 1e-4 && json!(row.mark_box) == default_of("mark_box"), "the rest at the defaults");
    assert!(store::check(&data).iter().all(|p| !p.contains("expression effect '")), "{:?}", store::check(&data));
    store::remove(&data, None, KIND, "angry").unwrap();
    assert!(Catalog::from_value(&source::load(&data).catalog).portrait_expressions.is_empty());
    drop_scratch(&data);
}
