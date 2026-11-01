//! The game's term rows (TERMS1-L, docs/specs/TERMS1.md 5.1; engine-only, no Godot original): the checks of one row of
//! the kind `terms` (a `game`-scope register id, the name, short form, word and tip lengths, origin) and across rows
//! (unique ids). A `world` term (the currency, the upkeep) is refused with where it is named instead (the world's purse
//! block, decision "TERMS1 answers" Q-T1). The loader keeps the rows that pass, in file order.

use super::View;
use crate::source::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::terms::{self, TermScope};

/// The catalog kind (file data/catalog/terms.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::TERMS_KIND;
/// 5.1: the text lengths (the name 1-NAME_MAX, the others 0-max).
pub const NAME_MAX: usize = 24;
pub const SHORT_MAX: usize = 16;
pub const WORD_MAX: usize = 24;
pub const TIP_MAX: usize = 160;
/// The refusal of a row without a name.
pub const NO_NAME: &str = "A term needs a name.";

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// The schema options of a field of the kind (origin).
fn options(key: &str) -> Vec<String> {
    fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// 5.1: the refusal of id `id`: "" for a `game`-scope register id.
pub fn id_problem(id: &str) -> String {
    match terms::def(id).map(|t| t.scope) {
        Some(TermScope::Game) => String::new(),
        Some(TermScope::World) => format!("'{id}' is named per world: set the world's purse block."),
        None => format!("Unknown term '{id}'. Terms: {}.", terms::ids(TermScope::Game).join(", ")),
    }
}

/// The problems of each row (same order as the rows), on its own and against the others (unique ids).
pub fn check_rows(rows: &[Dict]) -> Vec<Vec<String>> {
    let ids: Vec<String> = rows.iter().map(|r| json::s(r, "id")).collect();
    let mut out: Vec<Vec<String>> = rows.iter().map(row_errors).collect();
    for (i, found) in out.iter_mut().enumerate() {
        if !ids[i].is_empty() && ids[..i].contains(&ids[i]) {
            found.push(format!("ID '{}' is already used by another term.", ids[i]));
        }
    }
    out
}

/// The own problems of one (normalized) row.
pub fn row_errors(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let id = json::s(e, "id");
    let why = id_problem(&id);
    if !why.is_empty() {
        out.push(why);
    }
    let name = json::s(e, "name");
    if name.trim().is_empty() {
        out.push(NO_NAME.into());
    }
    for (key, label, max) in [("name", "Name", NAME_MAX), ("short", "Short", SHORT_MAX), ("word", "Word", WORD_MAX), ("tip", "Tip", TIP_MAX)] {
        let n = chars(&json::s(e, key));
        if n > max {
            out.push(format!("{label} is {n} characters (max {max})."));
        }
    }
    let origin = json::s(e, "origin");
    let origins = options("origin");
    if !origins.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", origins.join(", ")));
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
