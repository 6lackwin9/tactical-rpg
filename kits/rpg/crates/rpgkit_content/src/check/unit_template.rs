//! Role templates (R4-1 `U2.3-TIERS`, FB-78; engine-only, no Godot original): the checks of one row of the kind
//! `unit_templates` (the id, name and origin, a role of the schema's `ai_role` options, both share vectors as a class's
//! shares are checked: 0-1 each, totalling 1, and the stats they give 6-18 per stat at level 1 and 0-3 gained per stat at
//! every level) and across rows (unique ids, one row per role). The 6-18 range applies to the row (the reference stats),
//! never to a tier-scaled unit. The loader keeps the rows that pass, in file order.

use super::View;
use crate::source::fields;
use aegis_core::json::{self, gstr, Dict};

/// The catalog kind (file data/catalog/unit_templates.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::UNIT_TEMPLATES_KIND;
/// The id's length range and the name's limit.
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 24;
pub const NAME_MAX: usize = 32;
/// The refusal of a row without both share vectors.
pub const NO_SHARES: &str = "A unit template needs base shares and growth shares.";

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// The schema options of a field of the kind (role, origin).
pub fn options(key: &str) -> Vec<String> {
    fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// The problems of each row (same order as the rows), on its own and against the others (unique ids and roles).
pub fn check_rows(rows: &[Dict]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = rows.iter().map(row_errors).collect();
    for (i, found) in out.iter_mut().enumerate() {
        let id = json::s(&rows[i], "id");
        if !id.is_empty() && rows[..i].iter().any(|r| json::s(r, "id") == id) {
            found.push(format!("ID '{id}' is already used by another unit template."));
        }
        let role = json::s(&rows[i], "role");
        if let Some(other) = rows[..i].iter().find(|r| json::s(r, "role") == role) {
            found.push(format!("Role {role} already has the unit template '{}'.", json::s(other, "id")));
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
    if n > NAME_MAX {
        out.push(format!("Name is {n} characters (max {NAME_MAX})."));
    }
    for (key, label) in [("role", "Role"), ("origin", "Origin")] {
        let value = json::s(e, key);
        let allowed = options(key);
        if !allowed.contains(&value) {
            out.push(format!("{label}: '{value}' is not one of {}.", allowed.join(", ")));
        }
    }
    let given = |key: &str| json::get(e, key).as_object().is_some_and(|m| !m.is_empty());
    if given("base_shares") && given("growth_shares") {
        out.extend(super::unit::validate_shares(e));
    } else {
        out.push(NO_SHARES.into());
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
