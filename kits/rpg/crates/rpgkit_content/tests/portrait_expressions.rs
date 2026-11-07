//! PORTRAITFX1-EX's catalog kind `portrait_expressions` (PORTRAITFX1.md 16; scratch data): a missing file is an empty
//! table; each error text, one per mistake; the loader keeps the rows that pass and reports the rest; the writer stores
//! only the fields that differ from the schema defaults, re-saves byte for byte and refuses a bad row; a row that draws
//! nothing warns; the loaded rows reach `Catalog::portrait_expressions` with the schema defaults. PORTRAITFX2-M (5.1,
//! FL3): the motion fields' defaults are the plain breath, each motion mistake has its text, a motion-only row draws
//! something, offsets past 3 px warn, and the writer stores only the motion fields that differ.

use aegis_core::json::{self, Dict};
use rpgkit_content::check::expression_fx::{
    self, ANCHOR, BOB_MS, BOB_PX_MAX, BREATH_AMP, BREATH_IN, BREATH_JITTER, BREATH_MS, DRAWS_NOTHING, ENTER_MS, MARK_SIZE, MOTION_PX, OFFSETS_CLAMPED, PULSE_MS, REST_MS_MAX, SETTLE_MS_MAX, SHAKE_MS, SHAKE_PX_MAX, SHIVER_GAP_MS, SHIVER_MS, SHIVER_PX_MAX, STRENGTH,
    TINT_BREATH, TINT_MS,
};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, PORTRAIT_EXPRESSIONS_KIND as KIND, PORTRAIT_MARKS, PORTRAIT_MARK_BOXES, PORTRAIT_MOTION_KEYS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// A scratch data folder `<tmp>/<tag>/data` with empty catalog and worlds folders.
fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_expression_fx_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

/// The schema default of `key` in the kind's field table.
fn default_of(key: &str) -> Value {
    let f = source::schema()["content"][KIND].as_array().unwrap().iter().find(|f| f["key"] == key).unwrap_or_else(|| panic!("a field {key}"));
    f["default"].clone()
}

#[test]
fn a_missing_file_is_an_empty_table() {
    let (_scratch, data) = scratch("missing");
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
    let (_scratch, data) = scratch("texts");
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, KIND, &dict(e), "");
    let ok = Vec::<String>::new();
    assert_eq!(check(json!({"id": "angry", "tint": "#e11d48", "mark": "vein", "mark_pulse_ms": 900})), ok, "an angry row");
    assert_eq!(check(json!({"id": "hurt", "tint": "#e11d48", "tint_ms": 240, "shake_px": 2, "shake_ms": 220})), ok, "a hurt row");
    assert_eq!(check(json!({"id": "afraid", "mark": "sweat", "mark_box": "eyes", "mark_x": ANCHOR.0, "mark_y": ANCHOR.1})), ok, "the anchor's ends");
    // PORTRAITFX2 FL2: the id is the game's own face name; `neutral` takes motion fields only
    let (lo, hi) = (expression_fx::ID_MIN, expression_fx::ID_MAX);
    assert_eq!(check(json!({"id": ""})), [format!("ID is required: the face's name ({lo}-{hi} characters of a-z, 0-9 and _, a letter first).")]);
    assert_eq!(check(json!({"id": "smug", "tint": "#e11d48"})), ok, "a game face");
    let long = "s".repeat(hi + 1);
    for bad in ["9lives", "Smug", "s", long.as_str(), "smug-face"] {
        assert_eq!(check(json!({"id": bad, "tint": "#e11d48"})), [format!("ID '{bad}' must be {lo}-{hi} characters of a-z, 0-9 and _, a letter first.")], "{bad}");
    }
    assert_eq!(check(json!({"id": "s".repeat(hi), "tint": "#e11d48"})), ok, "{hi} characters");
    assert_eq!(check(json!({"id": "neutral", "breath_ms": 4000})), ok, "a neutral row with motion");
    for plain in [json!({"tint": "#e11d48"}), json!({"mark": "vein"}), json!({"shake_px": 1, "shake_ms": 200})] {
        let mut row = dict(json!({"id": "neutral"}));
        row.extend(dict(plain.clone()));
        assert_eq!(check::validate_entry(&v, KIND, &row, ""), [expression_fx::NEUTRAL_MOTION_ONLY], "{plain}");
    }
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
    let (_scratch, data) = scratch("loader");
    let entries = json!([{"id": "angry", "tint": "#e11d48"}, {"id": "neutral", "tint": "#e11d48"}, {"id": "angry", "mark": "vein"}, {"id": "afraid"}]);
    fs::write(data.join("catalog").join(format!("{KIND}.json")), json!({"kind": KIND, "version": 1, "entries": entries}).to_string()).unwrap();
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(kept, ["angry", "afraid"], "the refused rows are skipped");
    let found: Vec<&String> = src.errors.iter().filter(|e| e.contains("expression effect '")).collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].ends_with(&format!("expression effect 'neutral': {}", expression_fx::NEUTRAL_MOTION_ONLY)), "{found:?}");
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
    let (_scratch, data) = scratch("writer");
    let file = data.join("catalog").join(format!("{KIND}.json"));
    let refused = store::upsert(&data, None, KIND, &dict(json!({"id": "neutral", "tint": "#e11d48"})), None).err().expect("a neutral row is refused");
    assert!(refused.iter().any(|e| e.contains(expression_fx::NEUTRAL_MOTION_ONLY)), "{refused:?}");
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

// ---- PORTRAITFX2-M: the motion fields (PORTRAITFX2.md 5.1 and FL3) -----------------------------------------------

/// The 6.1 motion rows of the first three faces, as the asset lane enters them (with their PORTRAITFX1 effects).
fn motion_rows() -> [Value; 3] {
    [
        json!({"id": "hurt", "tint": "#e11d48", "tint_strength": 0.8, "tint_ms": 240, "shake_px": 2, "shake_ms": 220, "breath_amp": 1.5, "breath_ms": 1700, "breath_in": 0.4, "breath_jitter": 0.35, "enter_stretch": -2, "enter_ms": 280, "blink": false}),
        json!({"id": "angry", "tint": "#e11d48", "tint_strength": 0.35, "mark": "vein", "breath_amp": 4.0, "breath_ms": 2100, "breath_in": 0.4, "breath_jitter": 0.1, "lean_px": 1, "tint_breath": 0.5}),
        json!({"id": "afraid", "mark": "sweat", "breath_amp": 1.2, "breath_ms": 1300, "breath_jitter": 0.2, "shiver_px": 1, "shiver_ms": 280, "shiver_gap_ms": 1400}),
    ]
}

#[test]
fn the_motion_fields_default_to_the_plain_breath() {
    let row = rpgkit_data::PortraitExpression::from_obj(&Dict::new());
    let typed = json!({
        "breath_amp": row.breath_amp, "breath_ms": row.breath_ms, "breath_in": row.breath_in, "breath_jitter": row.breath_jitter, "lean_px": row.lean_px,
        "bob_px": row.bob_px, "bob_ms": row.bob_ms, "enter_dy": row.enter_dy, "enter_stretch": row.enter_stretch, "enter_ms": row.enter_ms,
        "rest_dy": row.rest_dy, "rest_ms": row.rest_ms, "settle_ms": row.settle_ms, "shiver_px": row.shiver_px, "shiver_ms": row.shiver_ms,
        "shiver_gap_ms": row.shiver_gap_ms, "blink": row.blink, "tint_breath": row.tint_breath,
    });
    for key in PORTRAIT_MOTION_KEYS {
        assert_eq!(typed[key], default_of(key), "the typed reader's {key} is the schema's");
    }
    let schema: Vec<String> = source::schema()["content"][KIND].as_array().unwrap().iter().map(|f| f["key"].as_str().unwrap().to_string()).filter(|k| PORTRAIT_MOTION_KEYS.contains(&k.as_str())).collect();
    assert_eq!(schema, PORTRAIT_MOTION_KEYS, "the motion fields in schema order");
    // the breath's depth at the reference side is the PORTRAITFX1 breath, 0.012 of the side
    assert!((row.breath_amp - 0.012 * 240.0).abs() < 1e-9 && row.breath_ms == 3600 && row.breath_in == 0.5 && row.blink);
}

#[test]
fn each_motion_mistake_has_its_text() {
    let (_scratch, data) = scratch("motion_texts");
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, KIND, &dict(e), "");
    let ok = Vec::<String>::new();
    for row in motion_rows() {
        assert_eq!(check(row.clone()), ok, "{row}");
    }
    let base = |extra: Value| {
        let mut e = dict(json!({"id": "angry", "tint": "#e11d48"}));
        e.extend(dict(extra));
        Value::Object(e)
    };
    assert_eq!(check(base(json!({"breath_amp": 4.5}))), [format!("Breath amp must be {:?}-{:?} (is 4.5).", BREATH_AMP.0, BREATH_AMP.1)]);
    assert_eq!(check(base(json!({"breath_ms": 500}))), [format!("Breath ms must be {}-{} (is 500).", BREATH_MS.0, BREATH_MS.1)]);
    assert_eq!(check(base(json!({"breath_in": 0.9}))), [format!("Breath in must be {:?}-{:?} (is 0.9).", BREATH_IN.0, BREATH_IN.1)]);
    assert_eq!(check(base(json!({"breath_jitter": 0.6}))), [format!("Breath jitter must be {:?}-{:?} (is 0.6).", BREATH_JITTER.0, BREATH_JITTER.1)]);
    assert_eq!(check(base(json!({"lean_px": -4}))), [format!("Lean px must be {} to {} (is -4).", -MOTION_PX, MOTION_PX)]);
    assert_eq!(check(base(json!({"lean_px": 1, "breath_amp": 0.0}))), ["Lean px needs a breath (Breath amp above 0)."]);
    assert_eq!(check(base(json!({"bob_px": 4, "bob_ms": 1000}))), [format!("Bob px must be 0-{BOB_PX_MAX} (is 4).")]);
    assert_eq!(check(base(json!({"bob_px": 1}))), [format!("Bob ms must be {}-{} with a bob (is 0).", BOB_MS.0, BOB_MS.1)]);
    assert_eq!(check(base(json!({"bob_ms": 1000}))), ["Bob ms must be 0 without a bob (is 1000)."]);
    assert_eq!(check(base(json!({"enter_dy": 4, "enter_ms": 300}))), [format!("Enter dy must be {} to {} (is 4).", -MOTION_PX, MOTION_PX)]);
    assert_eq!(check(base(json!({"enter_stretch": -4, "enter_ms": 300}))), [format!("Enter stretch must be {} to {} (is -4).", -MOTION_PX, MOTION_PX)]);
    assert_eq!(check(base(json!({"enter_dy": -2}))), [format!("Enter ms must be {}-{} with an entry move (is 0).", ENTER_MS.0, ENTER_MS.1)]);
    assert_eq!(check(base(json!({"enter_stretch": -2, "enter_ms": 2000}))), [format!("Enter ms must be {}-{} with an entry move (is 2000).", ENTER_MS.0, ENTER_MS.1)]);
    assert_eq!(check(base(json!({"enter_ms": 300}))), ["Enter ms must be 0 without an entry move (is 300)."]);
    assert_eq!(check(base(json!({"rest_dy": 5}))), [format!("Rest dy must be {} to {} (is 5).", -MOTION_PX, MOTION_PX)]);
    assert_eq!(check(base(json!({"rest_dy": 1, "rest_ms": 4000}))), [format!("Rest ms must be 0-{REST_MS_MAX} with a rest (is 4000).")]);
    assert_eq!(check(base(json!({"rest_ms": 400}))), ["Rest ms must be 0 without a rest (is 400)."]);
    assert_eq!(check(base(json!({"settle_ms": 5000}))), [format!("Settle ms must be 0-{SETTLE_MS_MAX} (is 5000).")]);
    assert_eq!(check(base(json!({"shiver_px": 3, "shiver_ms": 280, "shiver_gap_ms": 1400}))), [format!("Shiver px must be 0-{SHIVER_PX_MAX} (is 3).")]);
    assert_eq!(
        check(base(json!({"shiver_px": 1}))),
        [format!("Shiver ms must be {}-{} with a shiver (is 0).", SHIVER_MS.0, SHIVER_MS.1), format!("Shiver gap ms must be {}-{} with a shiver (is 0).", SHIVER_GAP_MS.0, SHIVER_GAP_MS.1)]
    );
    assert_eq!(check(base(json!({"shiver_ms": 280, "shiver_gap_ms": 1400}))), ["Shiver ms must be 0 without a shiver (is 280).", "Shiver gap ms must be 0 without a shiver (is 1400)."]);
    assert_eq!(check(base(json!({"tint_breath": 1.5}))), [format!("Tint breath must be {:?}-{:?} (is 1.5).", TINT_BREATH.0, TINT_BREATH.1)]);
    assert_eq!(check(json!({"id": "angry", "mark": "vein", "tint_breath": 0.5})), ["Tint breath needs a tint."]);
    assert_eq!(check(base(json!({"tint_breath": 0.5, "breath_amp": 0.0}))), ["Tint breath needs a breath (Breath amp above 0)."]);
    drop_scratch(&data);
}

#[test]
fn a_motion_row_draws_something_and_large_offsets_warn() {
    // the warnings read a raw row as normalize would (a missing key is its default)
    let row = dict;
    assert_eq!(expression_fx::warnings(&row(json!({"id": "afraid"}))), [DRAWS_NOTHING], "no tint, mark, shake or motion");
    assert_eq!(expression_fx::warnings(&row(json!({"id": "afraid", "breath_amp": 2.88, "blink": true}))), [DRAWS_NOTHING], "the defaults written out draw nothing new");
    assert!(expression_fx::warnings(&row(json!({"id": "afraid", "breath_ms": 1300}))).is_empty(), "a motion field is something to draw");
    assert!(expression_fx::warnings(&row(json!({"id": "afraid", "blink": false}))).is_empty());
    for r in motion_rows() {
        assert!(expression_fx::warnings(&row(r.clone())).is_empty(), "{r}");
    }
    let wide = json!({"id": "angry", "tint": "#e11d48", "lean_px": 2, "rest_dy": -1, "rest_ms": 400, "shiver_px": 1, "shiver_ms": 280, "shiver_gap_ms": 1400});
    assert_eq!(expression_fx::warnings(&row(wide)), [OFFSETS_CLAMPED], "2 + 1 + 1 px past 3");
    let at_three = json!({"id": "angry", "tint": "#e11d48", "lean_px": 1, "bob_px": 1, "bob_ms": 1100, "enter_dy": -1, "enter_ms": 300});
    assert!(expression_fx::warnings(&row(at_three)).is_empty(), "3 px is within the limit");
}

#[test]
fn the_writer_stores_only_the_motion_fields_that_differ() {
    let (_scratch, data) = scratch("motion_writer");
    let file = data.join("catalog").join(format!("{KIND}.json"));
    // today's rows, with every motion field written at its default: none is stored
    let mut plain = dict(json!({"id": "angry", "tint": "#e11d48", "tint_strength": 0.35}));
    for key in PORTRAIT_MOTION_KEYS {
        plain.insert(key.to_string(), default_of(key));
    }
    store::upsert(&data, None, KIND, &plain, None).expect("an angry row at the default motion");
    let stored = |data: &Path| -> Vec<Dict> {
        let v: Value = serde_json::from_str(&fs::read_to_string(data.join("catalog").join(format!("{KIND}.json"))).unwrap()).unwrap();
        json::arr(&v["entries"]).iter().map(|e| json::dict(e).clone()).collect()
    };
    let keys = |d: &Dict| d.keys().cloned().collect::<Vec<_>>();
    assert_eq!(keys(&stored(&data)[0]), ["id", "tint", "tint_strength"], "the defaults are left out");
    // the 6.1 rows: their motion keys stored in schema order, re-saved byte for byte, read back as written
    for r in motion_rows() {
        store::upsert(&data, None, KIND, &dict(r), None).expect("a 6.1 row");
    }
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, KIND).unwrap(), text, "a re-save without changes is byte-identical");
    let rows = stored(&data);
    let hurt = rows.iter().find(|e| json::s(e, "id") == "hurt").unwrap();
    assert_eq!(keys(hurt), ["id", "tint", "tint_strength", "tint_ms", "shake_px", "shake_ms", "breath_amp", "breath_ms", "breath_in", "breath_jitter", "enter_stretch", "enter_ms", "blink"]);
    let cat = Catalog::from_value(&source::load(&data).catalog);
    let afraid = cat.portrait_expressions.iter().find(|r| r.id == "afraid").unwrap();
    assert_eq!((afraid.shiver_px, afraid.shiver_ms, afraid.shiver_gap_ms, afraid.breath_ms, afraid.blink), (1, 280, 1400, 1300, true));
    assert!((afraid.breath_amp - 1.2).abs() < 1e-4 && (afraid.breath_in - 0.5).abs() < 1e-9 && (afraid.breath_jitter - 0.2).abs() < 1e-4);
    let refused = store::upsert(&data, None, KIND, &dict(json!({"id": "hurt", "tint": "#e11d48", "enter_dy": -2})), None).err().expect("an entry with no length is refused");
    assert!(refused.iter().any(|e| e.contains("Enter ms must be")), "{refused:?}");
    assert!(store::check(&data).iter().all(|p| !p.contains("expression effect '")), "{:?}", store::check(&data));
    drop_scratch(&data);
}
