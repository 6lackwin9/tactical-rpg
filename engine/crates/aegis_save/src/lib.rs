//! Engine save framework, ported from SaveGame and SaveMigrations (scripts/game_state/): one versioned JSON envelope
//! per slot ({format, version, saved_at, game_state}). Writes go to a temp file then rename; reads parse, check the
//! format tag and run the migration runner, which brings an older envelope up to the current version one step at a
//! time. The game supplies its `Format` (tag, version, slots, state keys) and the steps; the engine knows nothing of
//! the state's shape (docs/specs/SPLIT1.md section 2).

use aegis_core::json::{self, int, Dict};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// What a game's saves look like to the engine.
pub struct Format {
    /// The envelope's `format` tag.
    pub tag: &'static str,
    /// The current version; older envelopes migrate up to it, newer ones are refused.
    pub version: i64,
    /// The first slot (the autosave) and the last manual slot.
    pub first_slot: i64,
    pub last_slot: i64,
    /// The game_state keys kept after migration; any other key is dropped with a warning.
    pub keys: &'static [&'static str],
}

pub fn path_for(dir: &Path, slot: i64) -> PathBuf {
    dir.join(format!("slot_{slot}.json"))
}

pub fn exists(dir: &Path, slot: i64) -> bool {
    path_for(dir, slot).is_file()
}

pub fn delete(dir: &Path, slot: i64) -> bool {
    exists(dir, slot) && std::fs::remove_file(path_for(dir, slot)).is_ok()
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Problem strings; empty = written. `state` gives the game_state value or the game's own problems (checked after
/// the slot). `saved_at` > 0 keeps a given save time.
pub fn write(fmt: &Format, dir: &Path, slot: i64, saved_at: i64, state: impl FnOnce() -> Result<Value, Vec<String>>) -> Vec<String> {
    if !(fmt.first_slot..=fmt.last_slot).contains(&slot) {
        return vec![format!("Save slot must be {}-{}.", fmt.first_slot, fmt.last_slot)];
    }
    let state = match state() {
        Ok(state) => state,
        Err(errors) => return errors,
    };
    let envelope = json::obj([
        ("format", fmt.tag.into()),
        ("version", int(fmt.version)),
        ("saved_at", int(if saved_at > 0 { saved_at } else { now() })),
        ("game_state", state),
    ]);
    if std::fs::create_dir_all(dir).is_err() && !dir.is_dir() {
        return vec![format!("Can't create the save folder '{}'.", dir.display())];
    }
    let final_path = path_for(dir, slot);
    let mut tmp = final_path.with_extension("json.fc_tmp");
    let mut n = 1;
    while tmp.exists() {
        tmp = final_path.with_extension(format!("json.fc_tmp{n}"));
        n += 1;
    }
    let mut buf = Vec::new();
    let pretty = serde_json::ser::PrettyFormatter::with_indent(b"\t");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, pretty);
    if serde::Serialize::serialize(&json::sorted(&Value::Object(envelope)), &mut ser).is_err() || std::fs::write(&tmp, &buf).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return vec![format!("Can't write '{}'.", final_path.display())];
    }
    if std::fs::rename(&tmp, &final_path).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return vec!["Can't move the save into place (the previous save was kept).".into()];
    }
    Vec::new()
}

/// A slot's file text, or the refusal when the slot is empty.
pub fn read(dir: &Path, slot: i64) -> Result<String, Vec<String>> {
    if !exists(dir, slot) {
        return Err(vec![format!("No save in slot {slot}.")]);
    }
    Ok(std::fs::read_to_string(path_for(dir, slot)).unwrap_or_default())
}

/// A save file's text as an envelope: parse and format check (not migrated).
pub fn open(fmt: &Format, text: &str) -> Result<Dict, Vec<String>> {
    let Ok(Value::Object(envelope)) = serde_json::from_str::<Value>(text) else {
        return Err(vec!["Save file is corrupt or truncated.".into()]);
    };
    if json::s(&envelope, "format") != fmt.tag {
        return Err(vec![format!("Not a save file (format '{}').", json::s(&envelope, "format"))]);
    }
    Ok(envelope)
}

/// Brings an envelope to `fmt.version`: (envelope, warnings) or the refusal. `step(version, state, warnings)` turns a
/// game_state of `version` into one of `version + 1` (None: no such step) and may add warnings.
pub fn migrate(
    fmt: &Format,
    envelope: &Dict,
    mut step: impl FnMut(i64, &Dict, &mut Vec<String>) -> Option<Dict>,
) -> Result<(Dict, Vec<String>), Vec<String>> {
    let mut warnings = Vec::new();
    let mut data = envelope.clone();
    let raw_version = json::get(&data, "version");
    if !raw_version.is_number() {
        return Err(vec!["save has no version".into()]);
    }
    let mut version = json::to_int(raw_version);
    if version > fmt.version {
        return Err(vec!["save is from a newer version".into()]);
    }
    if version < 1 {
        return Err(vec![format!("save version {version} is invalid")]);
    }
    let mut state = json::dget(&data, "game_state").clone();
    while version < fmt.version {
        let Some(next) = step(version, &state, &mut warnings) else {
            return Err(vec![format!("no migration from save version {version}")]);
        };
        state = next;
        version += 1;
    }
    let keys: Vec<String> = state.keys().cloned().collect();
    for k in keys {
        if !fmt.keys.contains(&k.as_str()) {
            warnings.push(format!("dropped unknown game_state key '{k}'"));
            state.shift_remove(&k);
        }
    }
    data.insert("game_state".into(), Value::Object(state));
    data.insert("version".into(), int(version));
    Ok((data, warnings))
}

/// A slot read for its header, without migrating.
pub enum Peek {
    Missing,
    /// Not JSON, another format, or no valid version.
    Unreadable,
    /// The envelope as written (its version may be newer than the game's).
    Found(Dict),
}

pub fn peek(fmt: &Format, dir: &Path, slot: i64) -> Peek {
    if !exists(dir, slot) {
        return Peek::Missing;
    }
    let text = std::fs::read_to_string(path_for(dir, slot)).unwrap_or_default();
    let envelope = match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(e)) if json::s(&e, "format") == fmt.tag => e,
        _ => return Peek::Unreadable,
    };
    let version = json::get(&envelope, "version");
    if !version.is_number() || json::to_int(version) < 1 {
        return Peek::Unreadable;
    }
    Peek::Found(envelope)
}

/// The slot with the newest saved_at among those `saved_at(slot)` reports as readable; -1 when none.
pub fn newest_slot(fmt: &Format, saved_at: impl Fn(i64) -> Option<i64>) -> i64 {
    let (mut best, mut best_at) = (-1, -1);
    for slot in fmt.first_slot..=fmt.last_slot {
        if let Some(at) = saved_at(slot) {
            if at > best_at {
                best = slot;
                best_at = at;
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    const FMT: Format = Format { tag: "test_save", version: 3, first_slot: 0, last_slot: 2, keys: &["a", "b"] };

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aegis_save_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn write_read_peek_and_newest() {
        let dir = temp_dir("slots");
        assert_eq!(write(&FMT, &dir, 3, 1, || Ok(Value::Null)), vec!["Save slot must be 0-2.".to_string()]);
        assert_eq!(write(&FMT, &dir, 9, 1, || Err(vec!["bad state".into()])), vec!["Save slot must be 0-2.".to_string()]);
        assert_eq!(write(&FMT, &dir, 1, 1, || Err(vec!["bad state".into()])), vec!["bad state".to_string()]);
        assert!(!exists(&dir, 1));
        assert!(write(&FMT, &dir, 1, 100, || Ok(json::obj([("a", int(1))]).into())).is_empty());
        assert!(write(&FMT, &dir, 2, 200, || Ok(json::obj([("a", int(2))]).into())).is_empty());
        let envelope = open(&FMT, &read(&dir, 1).unwrap()).unwrap();
        assert_eq!(json::i(&envelope, "saved_at", 0), 100);
        assert_eq!(json::i(json::dget(&envelope, "game_state"), "a", 0), 1);
        assert!(matches!(peek(&FMT, &dir, 0), Peek::Missing));
        assert!(matches!(peek(&FMT, &dir, 2), Peek::Found(_)));
        std::fs::write(path_for(&dir, 0), "{\"format\": \"other\", \"version\": 1}").unwrap();
        assert!(matches!(peek(&FMT, &dir, 0), Peek::Unreadable));
        assert_eq!(open(&FMT, "{\"format\": \"other\"}").unwrap_err(), vec!["Not a save file (format 'other').".to_string()]);
        assert_eq!(open(&FMT, "{").unwrap_err(), vec!["Save file is corrupt or truncated.".to_string()]);
        assert_eq!(newest_slot(&FMT, |s| if let Peek::Found(e) = peek(&FMT, &dir, s) { Some(json::i(&e, "saved_at", 0)) } else { None }), 2);
        assert!(delete(&dir, 2) && !exists(&dir, 2));
        assert_eq!(read(&dir, 2).unwrap_err(), vec!["No save in slot 2.".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn migrate_runs_each_step_then_drops_unknown_keys() {
        let envelope = |v: Value| json::obj([("format", "test_save".into()), ("version", v), ("game_state", json::obj([("a", int(0)), ("z", int(0))]).into())]);
        let mut seen = Vec::new();
        let (data, warnings) = migrate(&FMT, &envelope(int(1)), |version, state, warnings| {
            seen.push(version);
            warnings.push(format!("step {version}"));
            let mut next = state.clone();
            next.insert("b".into(), int(version));
            Some(next)
        })
        .unwrap();
        assert_eq!(seen, vec![1, 2]);
        assert_eq!(warnings, vec!["step 1", "step 2", "dropped unknown game_state key 'z'"]);
        assert_eq!(json::i(&data, "version", 0), 3);
        assert_eq!(json::i(json::dget(&data, "game_state"), "b", 0), 2);
        let refuse = |v: Value| migrate(&FMT, &envelope(v), |_, s, _| Some(s.clone())).unwrap_err();
        assert_eq!(refuse(Value::Null), vec!["save has no version".to_string()]);
        assert_eq!(refuse(int(4)), vec!["save is from a newer version".to_string()]);
        assert_eq!(refuse(int(0)), vec!["save version 0 is invalid".to_string()]);
        let gap = migrate(&FMT, &envelope(int(1)), |v, s, _| (v < 2).then(|| s.clone())).unwrap_err();
        assert_eq!(gap, vec!["no migration from save version 2".to_string()]);
    }
}
