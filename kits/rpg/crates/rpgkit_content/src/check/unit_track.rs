//! Unit tracks (PK2-S1, `docs/sessions/PK2_GRAFTS.md` section 2; engine-only, no Godot original): the checks of one row
//! of the kind `unit_tracks` (the id, name, origin and cause, the advancing mark id, 1-9 stages each with a name, a
//! slotless passive row or none, and a departure only on the last stage) and across rows (unique ids). The loader keeps
//! the rows that pass, in file order.

use super::View;
use crate::source::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{TRACK_DEPARTURES, TRACK_STAGES_MAX, TRACK_STAGE_KEYS};
use serde_json::Value;

/// The catalog kind (file data/catalog/unit_tracks.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::UNIT_TRACKS_KIND;
/// The id's length range, the name's and a stage name's limit, the mark id's limit.
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 24;
pub const NAME_MAX: usize = 32;
pub const MARK_MAX: usize = 24;
/// The departure cause id's limit (the game names the cause; a save's departure accepts it, `Catalog::track_causes`).
pub const CAUSE_MAX: usize = 24;

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// The schema options of a field of the kind (origin).
fn options(key: &str) -> Vec<String> {
    fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// The problems of each row (same order as the rows), on its own against `passives` (the passive rows) and against
/// the others (unique ids).
pub fn check_rows(rows: &[Dict], passives: &[Dict]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = rows.iter().map(|r| row_errors(r, passives)).collect();
    for (i, found) in out.iter_mut().enumerate() {
        let id = json::s(&rows[i], "id");
        if !id.is_empty() && rows[..i].iter().any(|r| json::s(r, "id") == id) {
            found.push(format!("ID '{id}' is already used by another unit track."));
        }
    }
    out
}

/// The own problems of one (normalized) row against the passive rows `passives`.
pub fn row_errors(e: &Dict, passives: &[Dict]) -> Vec<String> {
    let mut out = Vec::new();
    let id = json::s(e, "id");
    if !json::id_ok(&id) || !(ID_MIN..=ID_MAX).contains(&chars(&id)) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _."));
    }
    let n = chars(&json::s(e, "name"));
    // PR5-26 (audit B9): the results and the cards name the track, so it needs one
    if n == 0 {
        out.push("A unit track needs a name.".into());
    } else if n > NAME_MAX {
        out.push(format!("Name is {n} characters (max {NAME_MAX})."));
    }
    let mark = json::s(e, "mark");
    if !mark.is_empty() && (!json::id_ok(&mark) || chars(&mark) > MARK_MAX) {
        out.push(format!("Advancing mark '{mark}' must be an id of up to {MARK_MAX} characters (a-z, 0-9 and _)."));
    }
    let origin = json::s(e, "origin");
    let allowed = options("origin");
    if !allowed.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", allowed.join(", ")));
    }
    let cause = json::s(e, "cause");
    if !cause.is_empty() && (!json::id_ok(&cause) || chars(&cause) > CAUSE_MAX) {
        out.push(format!("Departure cause '{cause}' must be an id of up to {CAUSE_MAX} characters (a-z, 0-9 and _)."));
    }
    let Some(stages) = json::get(e, "stages").as_array() else {
        out.push("Stages must be a list.".into());
        return out;
    };
    let departs = stages.iter().any(|s| s.as_object().is_some_and(|o| !json::s(o, "departure").is_empty()));
    if departs && cause.is_empty() {
        out.push("A unit track with a departure stage needs a departure cause.".into());
    }
    if stages.is_empty() || stages.len() > TRACK_STAGES_MAX {
        out.push(format!("A unit track needs 1-{TRACK_STAGES_MAX} stages (has {}).", stages.len()));
    }
    for (i, raw) in stages.iter().enumerate() {
        out.extend(stage_errors(i, raw, i + 1 == stages.len(), passives));
    }
    out
}

/// The problems of stage `i` (0-based; `last`: the row's last stage).
fn stage_errors(i: usize, raw: &Value, last: bool, passives: &[Dict]) -> Vec<String> {
    let at = format!("Stage {}", i + 1);
    let Some(s) = raw.as_object() else { return vec![format!("{at} must be an object {{{}}}.", TRACK_STAGE_KEYS.join(", "))] };
    let mut out = Vec::new();
    for k in s.keys().filter(|k| !TRACK_STAGE_KEYS.contains(&k.as_str())) {
        out.push(format!("{at}: unknown key '{k}' (use {}).", TRACK_STAGE_KEYS.join(", ")));
    }
    let name = json::s(s, "name");
    if name.is_empty() || chars(&name) > NAME_MAX {
        out.push(format!("{at}: a name of 1-{NAME_MAX} characters."));
    }
    let passive = json::s(s, "passive");
    if !passive.is_empty() {
        match passives.iter().find(|p| json::s(p, "id") == passive) {
            None => out.push(format!("{at}: unknown passive '{passive}'.")),
            Some(p) if !json::s(p, "slot").is_empty() => out.push(format!("{at}: passive '{passive}' takes a deck slot; a stage passive is slotless.")),
            Some(_) => {}
        }
    }
    let departure = json::s(s, "departure");
    if !departure.is_empty() {
        if !TRACK_DEPARTURES.contains(&departure.as_str()) {
            out.push(format!("{at}: departure '{departure}' is not one of {}.", TRACK_DEPARTURES.join(", ")));
        } else if !last {
            out.push(format!("{at}: only the last stage may carry a departure."));
        }
    }
    out
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): its own problems and those
/// against the loaded rows (its old row taken out); the entry is checked last.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    check_rows(&rows, v.entries("passives")).pop().unwrap_or_default()
}
