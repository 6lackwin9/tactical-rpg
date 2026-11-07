//! U2.2-S3 (FB-44): the `float_stats` field type, a stat -> number map kept to full float precision. `normalize`
//! keeps each number as a float (not rounded like `stats`, not snapped like `float`) and leaves zeros and non-numbers
//! out; `type_errors` wants an object of numbers; the writer prints the floats to 15 significant digits and a file
//! read back and written again is byte-identical.

use aegis_content::file::{self, stringify};
use aegis_content::schema::{normalize, type_errors};
use aegis_core::json::Dict;
use serde_json::{json, Value};

fn fields() -> Vec<Value> {
    vec![
        json!({"key": "id", "type": "id", "default": ""}),
        json!({"key": "shares", "label": "Shares", "type": "float_stats", "default": {}, "min": 0, "max": 1}),
        json!({"key": "points", "label": "Points", "type": "stats", "default": {}}),
    ]
}

fn dict(v: Value) -> Dict {
    v.as_object().unwrap().clone()
}

#[test]
fn normalize_keeps_full_precision_and_drops_zeros_and_non_numbers() {
    let third = 1.0 / 3.0;
    let row = normalize(&fields(), &dict(json!({"id": "a", "shares": {"STR": third, "DEX": 0.0, "INT": "x", "CONS": 1, "PERC": -0.25}, "points": {"STR": 2.6}})), false);
    let shares = row["shares"].as_object().unwrap();
    assert_eq!(shares.keys().map(String::as_str).collect::<Vec<_>>(), ["STR", "CONS", "PERC"], "zeros and non-numbers left out, order kept");
    assert_eq!(shares["STR"].as_f64(), Some(third), "no rounding or snapping");
    assert!(shares["CONS"].is_f64() && shares["CONS"].as_f64() == Some(1.0), "a whole number becomes a float");
    assert_eq!(shares["PERC"].as_f64(), Some(-0.25), "out-of-range values are kept for the game's checks (min/max are hints)");
    assert_eq!(row["points"], json!({"STR": 3}), "the integer `stats` type still rounds");
    let empty = normalize(&fields(), &dict(json!({"id": "b"})), false);
    assert_eq!(empty["shares"], json!({}), "the default");
    let not_object = normalize(&fields(), &dict(json!({"id": "c", "shares": [0.5, 0.5]})), false);
    assert_eq!(not_object["shares"], json!({}));
}

#[test]
fn type_errors_want_an_object_of_numbers() {
    assert!(type_errors(&fields(), &dict(json!({"shares": {"STR": 0.5, "DEX": 1}}))).is_empty());
    assert_eq!(type_errors(&fields(), &dict(json!({"shares": {"STR": "half"}}))), [r#"Shares: {"STR":"half"} is not an object of numbers."#]);
    assert_eq!(type_errors(&fields(), &dict(json!({"shares": 0.5}))), ["Shares: 0.5 is not an object of numbers."]);
}

#[test]
fn the_writer_prints_15_significant_digits_and_round_trips() {
    let shares: Value = [("STR", 13.0 / 62.0), ("DEX", 0.2), ("INT", 1.0), ("CONS", 2.0 / 3.0)].iter().map(|(k, v)| (k.to_string(), json!(v))).collect();
    let row = normalize(&fields(), &dict(json!({"id": "a", "shares": shares})), false);
    let text = stringify(&Value::Object(row.clone()));
    for want in ["\"STR\": 0.209677419354839", "\"DEX\": 0.2", "\"INT\": 1.0", "\"CONS\": 0.666666666666667"] {
        assert!(text.contains(want), "{want} in {text}");
    }
    let t = aegis_core::scratch::Scratch::new("aegis_float_stats");
    let path = t.join("row.json");
    std::fs::write(&path, &text).unwrap();
    let mut errors = Vec::new();
    let back = file::read(&path, &mut errors).unwrap_or_else(|| panic!("{errors:?}"));
    let again = normalize(&fields(), back.as_object().unwrap(), false);
    assert_eq!(stringify(&Value::Object(again.clone())), text, "read back and written again: byte-identical");
    for (k, v) in row["shares"].as_object().unwrap() {
        assert!((again["shares"][k].as_f64().unwrap() - v.as_f64().unwrap()).abs() < 1e-14, "{k} within 15 digits");
    }
}
