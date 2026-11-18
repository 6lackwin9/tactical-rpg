//! Siege rules of the world checks (PR6-9, `docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 5; engine-only, no Godot
//! original): a `sieges` row's rooms (`ROOMS_RANGE` rows `{encounter, name, facility}`, the encounter required and
//! resolving, the facility resolving when set) and its win and loss effect lists (campaign effect lists: no effect that
//! opens a screen). The field table holds `restocks` in range.

use super::campaign::unknown_keys;
use super::world::ref_errors;
use super::View;
use aegis_core::json::{self, Dict};
use rpgkit_data::facility::FACILITIES_KIND;
use rpgkit_data::siege::{OUTCOME_EFFECTS_MAX, ROOMS_RANGE, ROOM_KEYS};
use serde_json::Value;

/// The problems of a `sieges` row in world `w` beyond its field table.
pub fn siege_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    match json::get(e, "rooms").as_array() {
        None => errors.push("Sieges: rooms must be a list of room rows.".into()),
        Some(rooms) => {
            if !(ROOMS_RANGE.0..=ROOMS_RANGE.1).contains(&rooms.len()) {
                errors.push(format!("Sieges: {}..{} rooms.", ROOMS_RANGE.0, ROOMS_RANGE.1));
            }
            for (i, raw) in rooms.iter().enumerate() {
                let label = format!("Sieges room {}", i + 1);
                let Some(room) = raw.as_object() else {
                    errors.push(format!("{label}: must be an object {{{}}}.", ROOM_KEYS.join(", ")));
                    continue;
                };
                errors.extend(unknown_keys(room, &ROOM_KEYS, &label));
                let enc = json::s(room, "encounter");
                if enc.is_empty() {
                    errors.push(format!("{label}: choose the encounter."));
                } else {
                    errors.extend(ref_errors(v, w, &format!("{label} encounter"), "encounters", &enc));
                }
                let fac = json::s(room, "facility");
                if !fac.is_empty() {
                    errors.extend(ref_errors(v, w, &format!("{label} facility"), FACILITIES_KIND, &fac));
                }
            }
        }
    }
    for key in ["win_effects", "loss_effects"] {
        let raw = json::get(e, key);
        if raw.as_array().is_some_and(|l| l.len() > OUTCOME_EFFECTS_MAX) {
            errors.push(format!("Sieges: at most {OUTCOME_EFFECTS_MAX} {key}."));
        }
        if !matches!(raw, Value::Null) {
            errors.extend(super::campaign::effects_errors(v, w, raw, &format!("Sieges {key}")));
        }
    }
    errors
}
