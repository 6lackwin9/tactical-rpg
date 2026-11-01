//! Expression-effect rows (PORTRAITFX1-EX, docs/specs/PORTRAITFX1.md section 16; engine-only, no Godot original): the
//! checks of one row of the kind `portrait_expressions` (the face id, the tint and its strength and length, the mark
//! and its colour, box, anchor, size and pulse, the shake, the motion, origin) and across rows (unique ids). The loader
//! keeps the rows that pass, in file order; their ids are the game's faces after `neutral` (PORTRAITFX2 FL1-FL2,
//! `rpgkit_data::faces_of`). A row that draws nothing is a warning.

use super::View;
use crate::source::fields;
use aegis_core::assets::hex_colour;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{PORTRAIT_MARKS, PORTRAIT_MARK_BOXES};

/// The catalog kind (file data/catalog/portrait_expressions.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::PORTRAIT_EXPRESSIONS_KIND;
/// The plain face: its row (PORTRAITFX2 FL2) carries motion fields only.
pub const NEUTRAL: &str = rpgkit_data::NEUTRAL;
/// PORTRAITFX2 FL2: a face id's length range (a letter first, then a-z, 0-9 or _).
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 24;
/// PORTRAITFX2 FL2: the refusal of a `neutral` row with a tint, a mark or a shake.
pub const NEUTRAL_MOTION_ONLY: &str = "neutral is the plain face: motion fields only (no tint, mark or shake).";
/// The ranges: tint strength, a timed tint's ms, the anchor (in units of its box), the mark's width (a share of the
/// portrait's side), a pulse period, the shake px (the jolt ceiling) and ms.
pub const STRENGTH: (f64, f64) = (0.05, 1.0);
pub const TINT_MS: (i64, i64) = (60, 3000);
pub const ANCHOR: (f64, f64) = (-1.0, 2.0);
pub const MARK_SIZE: (f64, f64) = (0.04, 0.3);
pub const PULSE_MS: (i64, i64) = (300, 4000);
pub const SHAKE_PX_MAX: i64 = 4;
pub const SHAKE_MS: (i64, i64) = (60, 600);
/// The warning of a row with no tint, no mark, no shake and every motion field at its default (PORTRAITFX2 FL3).
pub const DRAWS_NOTHING: &str = "The row draws nothing: set a tint, a mark, a shake or a motion field.";
/// PORTRAITFX2-M 5.1, the motion fields' ranges: the breath's depth (px at 240), length, share breathing in and
/// jitter; the lean, entry and rest px (each way); the bob px and ms; the entry ms; the rest and settle ms; the shiver px,
/// ms and gap; the tint's pulse share.
pub const BREATH_AMP: (f64, f64) = (0.0, 4.0);
pub const BREATH_MS: (i64, i64) = (800, 8000);
pub const BREATH_IN: (f64, f64) = (0.2, 0.8);
pub const BREATH_JITTER: (f64, f64) = (0.0, 0.5);
pub const MOTION_PX: i64 = 3;
pub const BOB_PX_MAX: i64 = 3;
pub const BOB_MS: (i64, i64) = (300, 3000);
pub const ENTER_MS: (i64, i64) = (60, 1200);
pub const REST_MS_MAX: i64 = 3000;
pub const SETTLE_MS_MAX: i64 = 4000;
pub const SHIVER_PX_MAX: i64 = 2;
pub const SHIVER_MS: (i64, i64) = (80, 1000);
pub const SHIVER_GAP_MS: (i64, i64) = (200, 8000);
pub const TINT_BREATH: (f64, f64) = (0.0, 1.0);
/// The warning of offsets that add up past `MOTION_PX` (the engine clamps each axis there).
pub const OFFSETS_CLAMPED: &str = "The offsets (lean, bob, entry, rest, shiver) add up past 3 px: clamped at 3 px.";

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
    if id.is_empty() {
        out.push(format!("ID is required: the face's name ({ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _, a letter first)."));
    } else if !id_ok(&id) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _, a letter first."));
    }
    // the tint
    let tinted = colour(e, "tint", "Tint", &mut out);
    if id == NEUTRAL && (tinted || !json::s(e, "mark").is_empty() || json::i(e, "shake_px", 0) != 0) {
        out.push(NEUTRAL_MOTION_ONLY.into());
    }
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
    motion_errors(e, tinted, &mut out);
    let origin = json::s(e, "origin");
    let origins = options("origin");
    if !origins.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", origins.join(", ")));
    }
    out
}

/// PORTRAITFX2 FL2: `[a-z][a-z0-9_]{ID_MIN - 1, ID_MAX - 1}`.
fn id_ok(id: &str) -> bool {
    id.chars().next().is_some_and(|c| c.is_ascii_lowercase()) && (ID_MIN..=ID_MAX).contains(&id.chars().count()) && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// An int field within `lo..=hi`, else "`label` must be lo-hi (is x)." ("lo to hi" when lo is negative).
fn int_in(e: &Dict, key: &str, label: &str, (lo, hi): (i64, i64), out: &mut Vec<String>) -> i64 {
    let x = json::i(e, key, 0);
    if !(lo..=hi).contains(&x) {
        let range = if lo < 0 { format!("{lo} to {hi}") } else { format!("{lo}-{hi}") };
        out.push(format!("{label} must be {range} (is {x})."));
    }
    x
}

/// A length that is `range` with its part (`with` true) and 0 without it: "`label` must be lo-hi with `what` (is x)."
/// or "`label` must be 0 without `what` (is x)."
fn ms_with(e: &Dict, key: &str, label: &str, with: bool, what: &str, (lo, hi): (i64, i64), out: &mut Vec<String>) {
    let ms = json::i(e, key, 0);
    if with && !(lo..=hi).contains(&ms) {
        out.push(format!("{label} must be {lo}-{hi} with {what} (is {ms})."));
    } else if !with && ms != 0 {
        out.push(format!("{label} must be 0 without {what} (is {ms})."));
    }
}

/// PORTRAITFX2-M 5.1: the motion fields' problems (`tinted`: the row has a tint).
fn motion_errors(e: &Dict, tinted: bool, out: &mut Vec<String>) {
    float_in(e, "breath_amp", "Breath amp", BREATH_AMP, 2.88, out);
    let breath = json::f(e, "breath_amp", 2.88) > 0.0;
    let ms = json::i(e, "breath_ms", 3600);
    if !(BREATH_MS.0..=BREATH_MS.1).contains(&ms) {
        out.push(format!("Breath ms must be {}-{} (is {ms}).", BREATH_MS.0, BREATH_MS.1));
    }
    float_in(e, "breath_in", "Breath in", BREATH_IN, 0.5, out);
    float_in(e, "breath_jitter", "Breath jitter", BREATH_JITTER, 0.0, out);
    if int_in(e, "lean_px", "Lean px", (-MOTION_PX, MOTION_PX), out) != 0 && !breath {
        out.push("Lean px needs a breath (Breath amp above 0).".into());
    }
    let bob = int_in(e, "bob_px", "Bob px", (0, BOB_PX_MAX), out) != 0;
    ms_with(e, "bob_ms", "Bob ms", bob, "a bob", BOB_MS, out);
    let dy = int_in(e, "enter_dy", "Enter dy", (-MOTION_PX, MOTION_PX), out);
    let st = int_in(e, "enter_stretch", "Enter stretch", (-MOTION_PX, MOTION_PX), out);
    ms_with(e, "enter_ms", "Enter ms", dy != 0 || st != 0, "an entry move", ENTER_MS, out);
    let rest = int_in(e, "rest_dy", "Rest dy", (-MOTION_PX, MOTION_PX), out) != 0;
    ms_with(e, "rest_ms", "Rest ms", rest, "a rest", (0, REST_MS_MAX), out);
    int_in(e, "settle_ms", "Settle ms", (0, SETTLE_MS_MAX), out);
    let shiver = int_in(e, "shiver_px", "Shiver px", (0, SHIVER_PX_MAX), out) != 0;
    ms_with(e, "shiver_ms", "Shiver ms", shiver, "a shiver", SHIVER_MS, out);
    ms_with(e, "shiver_gap_ms", "Shiver gap ms", shiver, "a shiver", SHIVER_GAP_MS, out);
    float_in(e, "tint_breath", "Tint breath", TINT_BREATH, 0.0, out);
    if json::f(e, "tint_breath", 0.0) > 0.0 {
        if !tinted {
            out.push("Tint breath needs a tint.".into());
        }
        if !breath {
            out.push("Tint breath needs a breath (Breath amp above 0).".into());
        }
    }
}

/// PORTRAITFX2 FL3: whether every motion field of the (normalized) row is at its schema default (floats compared as
/// normalize snaps them).
pub fn motion_at_defaults(e: &Dict) -> bool {
    let fields = fields("content", KIND);
    rpgkit_data::PORTRAIT_MOTION_KEYS.iter().all(|key| {
        let Some(f) = fields.iter().map(json::dict).find(|f| json::s(f, "key") == *key) else { return true };
        let d = json::get(f, "default");
        match (e.get(*key), d) {
            (None, _) => true,
            (Some(v), d) if v.is_number() && d.is_number() => (json::to_float_or(v, 0.0) - json::to_float_or(d, 0.0)).abs() < 1e-6,
            (Some(v), d) => v == d,
        }
    })
}

/// The soft problems of one row: it draws nothing (no tint, mark or shake and the default motion); its offsets add up
/// past 3 px (the engine clamps them).
pub fn warnings(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    if json::s(e, "tint").is_empty() && json::s(e, "mark").is_empty() && json::i(e, "shake_px", 0) == 0 && motion_at_defaults(e) {
        out.push(DRAWS_NOTHING.into());
    }
    let sum = ["lean_px", "bob_px", "enter_dy", "rest_dy", "shiver_px"].iter().map(|k| json::i(e, k, 0).abs()).sum::<i64>();
    if sum > MOTION_PX {
        out.push(OFFSETS_CLAMPED.into());
    }
    out
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): its own problems and those
/// against the loaded rows (its old row taken out); the entry is checked last.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    check_rows(&rows).pop().unwrap_or_default()
}
