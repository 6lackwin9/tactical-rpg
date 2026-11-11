//! Body slots (PK2-S2, `docs/sessions/PK2_GRAFTS.md` section 3; engine-only, no Godot original): the checks of one row
//! of the kind `body_slots` (the id, name, role and origin) and across rows (unique ids, at most `BODY_SLOTS_MAX`
//! rows). The loader keeps the rows that pass, in file order.

use super::View;
use crate::source::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{BODY_ROLES, BODY_SLOTS_MAX};

/// The catalog kind (file data/catalog/body_slots.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::BODY_SLOTS_KIND;
/// The id's length range and the name's limit.
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 24;
pub const NAME_MAX: usize = 32;

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// The schema options of a field of the kind (origin).
fn options(key: &str) -> Vec<String> {
    fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// The problems of each row (same order as the rows), on its own and against the others (unique ids, the row count).
pub fn check_rows(rows: &[Dict]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = rows.iter().map(row_errors).collect();
    for (i, found) in out.iter_mut().enumerate() {
        let id = json::s(&rows[i], "id");
        if !id.is_empty() && rows[..i].iter().any(|r| json::s(r, "id") == id) {
            found.push(format!("ID '{id}' is already used by another body slot."));
        }
        if i >= BODY_SLOTS_MAX {
            found.push(format!("A catalog holds at most {BODY_SLOTS_MAX} body slots."));
        }
    }
    out
}

/// The own problems of one (normalized) row.
pub fn row_errors(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let id = json::s(e, "id");
    if !json::id_ok(&id) || !(ID_MIN..=ID_MAX).contains(&chars(&id)) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _."));
    }
    let n = chars(&json::s(e, "name"));
    if n == 0 {
        out.push("A body slot needs a name.".into());
    } else if n > NAME_MAX {
        out.push(format!("Name is {n} characters (max {NAME_MAX})."));
    }
    let role = json::s(e, "role");
    if !BODY_ROLES.contains(&role.as_str()) {
        out.push(format!("Role: '{role}' is not one of {}.", BODY_ROLES.join(", ")));
    }
    let origin = json::s(e, "origin");
    let allowed = options("origin");
    if !allowed.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", allowed.join(", ")));
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
