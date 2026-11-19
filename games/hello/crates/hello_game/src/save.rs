//! One save slot through the engine's save framework (`aegis_save`): the walk in the versioned envelope.

use crate::walk::Walk;
use std::path::Path;

pub const FORMAT: aegis_save::Format = aegis_save::Format { tag: "hello_save", version: 1, first_slot: 0, last_slot: 0, keys: &["walk"] };
/// The only slot.
pub const SLOT: i64 = 0;

/// Writes the walk; the problems, empty when written.
pub fn write(dir: &Path, walk: Walk) -> Vec<String> {
    aegis_save::write(&FORMAT, dir, SLOT, 0, || Ok(serde_json::json!({"walk": walk.to_value()})))
}

/// Whether the slot holds a save.
pub fn exists(dir: &Path) -> bool {
    aegis_save::exists(dir, SLOT)
}

/// The saved walk, or why there is none.
pub fn read(dir: &Path) -> Result<Walk, Vec<String>> {
    let text = aegis_save::read(dir, SLOT)?;
    let envelope = aegis_save::open(&FORMAT, &text)?;
    let (data, _warnings) = aegis_save::migrate(&FORMAT, &envelope, |_, _, _| None)?;
    let state = data.get("game_state").and_then(|s| s.get("walk")).cloned().unwrap_or_default();
    Walk::from_value(&state).map_err(|e| vec![format!("The save's walk is broken: {e}.")])
}
