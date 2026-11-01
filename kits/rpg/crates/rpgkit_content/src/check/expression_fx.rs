//! Expression-effect rows (PORTRAITFX1-EX, docs/specs/PORTRAITFX1.md section 16; engine-only, no Godot original): the
//! checks of one row of the kind `portrait_expressions` (the expression id, the tint and its strength and length, the
//! mark and its colour, box, anchor, size and pulse, the shake, origin) and across rows (unique ids). The loader keeps
//! the rows that pass, in file order. A row that draws nothing is a warning.

use super::View;
use crate::source::fields;
use aegis_core::assets::hex_colour;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{EXPRESSIONS, PORTRAIT_MARKS, PORTRAIT_MARK_BOXES};

/// The catalog kind (file data/catalog/portrait_expressions.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::PORTRAIT_EXPRESSIONS_KIND;
/// The plain face: it takes no effect row.
pub const NEUTRAL: &str = "neutral";
/// The ranges: tint strength, a timed tint's ms, the anchor (in units of its box), the mark's width (a share of the
/// portrait's side), a pulse period, the shake px (the jolt ceiling) and ms.
pub const STRENGTH: (f64, f64) = (0.05, 1.0);
pub const TINT_MS: (i64, i64) = (60, 3000);
pub const ANCHOR: (f64, f64) = (-1.0, 2.0);
pub const MARK_SIZE: (f64, f64) = (0.04, 0.3);
pub const PULSE_MS: (i64, i64) = (300, 4000);
pub const SHAKE_PX_MAX: i64 = 4;
pub const SHAKE_MS: (i64, i64) = (60, 600);
/// The warning of a row with no tint, no mark and no shake.
pub const DRAWS_NOTHING: &str = "The row draws nothing: set a tint, a mark or a shake.";

/// The schema options of a field of the kind (origin).
fn options(key: &str) -> Vec<String> {
    fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// The problems of each row (same order as the rows), on its own and against the others (unique ids).
pub fn check_rows(rows: &[Dict]) -> Vec<Vec<String>> {
    let ids: Vec<String> = rows.iter().map(|r| json::s(r, "id")).collect();
    let mut out: Vec<Vec<String>> = rows.iter().map(row_errors).collect();
    for (i, found) in out.iter_mut().enumerate() {
        if !ids[i].is_empty() && ids[..i].contains(&ids[i]) {
            found.push(format!("ID '{}' is already used by another expression effect.", ids[i]));
        }
    }
    out
}

/// `x` within `(lo, hi)`, else the message "`label` must be lo-hi (is x)."
fn float_in(e: &Dict, key: &str, label: &str, (lo, hi): (f64, f64), default: f64, out: &mut Vec<String>) {
    let x = json::f(e, key, default);
    if !(lo..=hi).contains(&x) {
        out.push(format!("{label} must be {lo:?}-{hi:?} (is {x:?})."));
    }
}

/// A colour field: "" or `#rrggbb`.
fn colour(e: &Dict, key: &str, label: &str, out: &mut Vec<String>) -> bool {
    let c = json::s(e, key);
    if !c.is_empty() && hex_colour(&c).is_none() {
        out.push(format!("{label} '{c}' must be \"\" or #rrggbb."));
    }
    !c.is_empty()
}

/// The own problems of one (normalized) row.
pub fn row_errors(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let id = json::s(e, "id");
    let named: Vec<&str> = EXPRESSIONS.iter().copied().filter(|x| *x != NEUTRAL).collect();
    if id.is_empty() {
        out.push(format!("ID is required: one of {}.", named.join(", ")));
    } else if id == NEUTRAL {
        out.push(format!("'{NEUTRAL}' is the plain face: it takes no effect row (one of {}).", named.join(", ")));
    } else if !named.contains(&id.as_str()) {
        out.push(format!("ID '{id}' is not an expression (one of {}).", named.join(", ")));
    }
    // the tint
    let tinted = colour(e, "tint", "Tint", &mut out);
    float_in(e, "tint_strength", "Tint strength", STRENGTH, 0.3, &mut out);
    let tint_ms = json::i(e, "tint_ms", 0);
    if tint_ms != 0 && !(TINT_MS.0..=TINT_MS.1).contains(&tint_ms) {
        out.push(format!("Tint ms must be 0 (it holds) or {}-{} (is {tint_ms}).", TINT_MS.0, TINT_MS.1));
    }
    if tint_ms != 0 && !tinted {
        out.push("Tint ms is for a row with a tint.".into());
    }
    // the mark
    let mark = json::s(e, "mark");
    if !mark.is_empty() && !PORTRAIT_MARKS.contains(&mark.as_str()) {
        out.push(format!("Mark: '{mark}' is not one of {}.", PORTRAIT_MARKS.join(", ")));
    }
    if colour(e, "mark_color", "Mark colour", &mut out) && mark.is_empty() {
        out.push("Mark colour is for a row with a mark.".into());
    }
    let mbox = json::s(e, "mark_box");
    if !PORTRAIT_MARK_BOXES.contains(&mbox.as_str()) {
        out.push(format!("Mark box: '{mbox}' is not one of {}.", PORTRAIT_MARK_BOXES.join(", ")));
    }
    float_in(e, "mark_x", "Mark x", ANCHOR, 0.85, &mut out);
    float_in(e, "mark_y", "Mark y", ANCHOR, 0.15, &mut out);
    float_in(e, "mark_size", "Mark size", MARK_SIZE, 0.12, &mut out);
    let pulse = json::i(e, "mark_pulse_ms", 0);
    if pulse != 0 && !(PULSE_MS.0..=PULSE_MS.1).contains(&pulse) {
        out.push(format!("Mark pulse ms must be 0 (steady) or {}-{} (is {pulse}).", PULSE_MS.0, PULSE_MS.1));
    }
    if pulse != 0 && mark.is_empty() {
        out.push("Mark pulse ms is for a row with a mark.".into());
    }
    // the shake
    let (px, ms) = (json::i(e, "shake_px", 0), json::i(e, "shake_ms", 0));
    if !(0..=SHAKE_PX_MAX).contains(&px) {
        out.push(format!("Shake px must be 0-{SHAKE_PX_MAX} (is {px})."));
    }
    match (px > 0, ms) {
        (true, ms) if !(SHAKE_MS.0..=SHAKE_MS.1).contains(&ms) => out.push(format!("Shake ms must be {}-{} with a shake (is {ms}).", SHAKE_MS.0, SHAKE_MS.1)),
        (false, ms) if ms != 0 => out.push(format!("Shake ms must be 0 without a shake (is {ms}).")),
        _ => {}
    }
    let origin = json::s(e, "origin");
    let origins = options("origin");
    if !origins.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", origins.join(", ")));
    }
    out
}

/// The soft problems of one row: it draws nothing.
pub fn warnings(e: &Dict) -> Vec<String> {
    if json::s(e, "tint").is_empty() && json::s(e, "mark").is_empty() && json::i(e, "shake_px", 0) == 0 {
        vec![DRAWS_NOTHING.into()]
    } else {
        Vec::new()
    }
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): its own problems and those
/// against the loaded rows (its old row taken out); the entry is checked last.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    check_rows(&rows).pop().unwrap_or_default()
}
