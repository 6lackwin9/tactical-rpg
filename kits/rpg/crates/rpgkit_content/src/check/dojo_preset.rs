//! Dojo presets (PK1-KITFIX (c), FB-184 Option A; engine-only, no Godot original): the checks of one row of the kind
//! `dojo_presets` (the id, name and origin; `gear` an object of loadout slots to known equipment that fits the slot;
//! up to `DOJO_POUCH_MAX` known consumables in `pouch`; up to `DOJO_PASSIVES_MAX` known passives, each once) and across
//! rows (unique ids). The loader keeps the rows that pass, in file order.

use super::View;
use crate::source::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{ItemDef, DOJO_PASSIVES_MAX, DOJO_POUCH_MAX, LOADOUT_SLOTS};
use serde_json::Value;

/// The catalog kind (file data/catalog/dojo_presets.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::DOJO_PRESETS_KIND;
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

/// The problems of each row (same order as the rows), on its own against `items` and `passives` (the loaded rows) and
/// against the others (unique ids).
pub fn check_rows(rows: &[Dict], items: &[Dict], passives: &[Dict]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = rows.iter().map(|r| row_errors(r, items, passives)).collect();
    for (i, found) in out.iter_mut().enumerate() {
        let id = json::s(&rows[i], "id");
        if !id.is_empty() && rows[..i].iter().any(|r| json::s(r, "id") == id) {
            found.push(format!("ID '{id}' is already used by another Dojo preset."));
        }
    }
    out
}

/// The own problems of one (normalized) row against the item rows `items` and the passive rows `passives`.
pub fn row_errors(e: &Dict, items: &[Dict], passives: &[Dict]) -> Vec<String> {
    let mut out = Vec::new();
    let id = json::s(e, "id");
    if !json::id_ok(&id) || !(ID_MIN..=ID_MAX).contains(&chars(&id)) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _."));
    }
    let n = chars(&json::s(e, "name"));
    if n == 0 || n > NAME_MAX {
        out.push(format!("Name must be 1-{NAME_MAX} characters."));
    }
    let origin = json::s(e, "origin");
    let allowed = options("origin");
    if !allowed.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", allowed.join(", ")));
    }
    let item = |iid: &str| items.iter().find(|r| json::s(r, "id") == iid).map(ItemDef::from_obj);
    match json::get(e, "gear") {
        Value::Null => {}
        Value::Object(g) => {
            for (slot, raw) in g {
                let at = format!("Gear {slot}");
                if !LOADOUT_SLOTS.contains(&slot.as_str()) {
                    out.push(format!("{at}: unknown slot (use {}).", LOADOUT_SLOTS.join(", ")));
                    continue;
                }
                let iid = gstr(raw);
                match item(&iid) {
                    None => out.push(format!("{at}: unknown item '{iid}'.")),
                    Some(def) if !rpgkit_rules::gear::item_fits_slot(&def, slot) => out.push(format!("{at}: '{iid}' does not fit the {slot} slot.")),
                    Some(_) => {}
                }
            }
        }
        _ => out.push(format!("Gear must be an object {{slot: item id}} (slots {}).", LOADOUT_SLOTS.join(", "))),
    }
    let pouch = json::aget(e, "pouch");
    if pouch.len() > DOJO_POUCH_MAX {
        out.push(format!("Pouch holds up to {DOJO_POUCH_MAX} items (has {}).", pouch.len()));
    }
    for raw in pouch {
        let iid = gstr(raw);
        match item(&iid) {
            None => out.push(format!("Pouch: unknown item '{iid}'.")),
            Some(def) if def.item_type != "CONSUMABLE" => out.push(format!("Pouch: '{iid}' is not a consumable.")),
            Some(_) => {}
        }
    }
    let held = json::aget(e, "passives");
    if held.len() > DOJO_PASSIVES_MAX {
        out.push(format!("Passives holds up to {DOJO_PASSIVES_MAX} passives (has {}).", held.len()));
    }
    for (i, raw) in held.iter().enumerate() {
        let pid = gstr(raw);
        if !passives.iter().any(|p| json::s(p, "id") == pid) {
            out.push(format!("Passives: unknown passive '{pid}'."));
        } else if held[..i].iter().any(|p| gstr(p) == pid) {
            out.push(format!("Passives: '{pid}' is listed twice."));
        }
    }
    out
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): its own problems and those
/// against the loaded rows (its old row taken out); the entry is checked last.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    check_rows(&rows, v.entries("items"), v.entries("passives")).pop().unwrap_or_default()
}
