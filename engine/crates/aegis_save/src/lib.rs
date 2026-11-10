//! Engine save framework, ported from SaveGame and SaveMigrations (scripts/game_state/): one versioned JSON envelope
//! per slot ({format, version, saved_at, game_state}). Writes go to a temp file then rename; reads parse, check the
//! format tag and run the migration runner, which brings an older envelope up to the current version one step at a
//! time. The game supplies its `Format` (tag, version, slots, state keys) and the steps; the engine knows nothing of
//! the state's shape (docs/specs/SPLIT1.md section 2). Engine-only addition (PLAYUI1 section 1): slot locks in a
//! `locks.json` beside the saves; `write` and `delete` refuse a locked slot. Engine-only addition (CAMPAIGN1 F5):
//! keyed saves (`key_<key>.json`, for example an act-start save) beside the slots, in the same envelope; they are
//! outside the slot range, so slot saves never overwrite them and slot listings never show them. Every user file goes
//! through the user store (`store`, WEBREL1-E1b): the disk on native, `localStorage` on the web.

pub mod store;

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

/// The engine's user folder: %APPDATA%/Aegis (the working folder when APPDATA is unset).
#[cfg(not(target_arch = "wasm32"))]
pub fn app_dir() -> PathBuf {
    std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(".")).join("Aegis")
}

/// The engine's user folder on the web: `aegis`, the first part of every `localStorage` key (`store::key_for`).
#[cfg(target_arch = "wasm32")]
pub fn app_dir() -> PathBuf {
    PathBuf::from(store::WEB_ROOT)
}

/// A game's own user folder (its saves and settings): %APPDATA%/Aegis/<game id>.
pub fn user_dir(game_id: &str) -> PathBuf {
    app_dir().join(game_id)
}

/// A game's first start since saves moved under the game id (SPLIT1-8): when its user folder does not exist yet,
/// copies the engine's shared folder into it (%APPDATA%/Aegis/saves/* and settings.json); the old files stay as they
/// are. The copied paths (relative to the shared folder), or the problem.
pub fn adopt_shared(game_id: &str) -> Result<Vec<String>, String> {
    let (shared, own) = (app_dir(), user_dir(game_id));
    let st = store::current();
    if st.exists(&own) {
        return Ok(Vec::new());
    }
    let mut copied = Vec::new();
    let mut copy = |rel: &Path| -> Result<(), String> {
        if let Some(dir) = own.join(rel).parent() {
            st.create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        st.copy(&shared.join(rel), &own.join(rel)).map_err(|e| format!("{}: {e}", shared.join(rel).display()))?;
        copied.push(rel.display().to_string());
        Ok(())
    };
    if let Ok(entries) = st.list_dir(&shared.join("saves")) {
        for p in entries.into_iter().filter(|p| st.is_file(p)) {
            copy(&Path::new("saves").join(p.file_name().unwrap_or_default()))?;
        }
    }
    if st.is_file(&shared.join("settings.json")) {
        copy(Path::new("settings.json"))?;
    }
    st.create_dir_all(&own).map_err(|e| format!("{}: {e}", own.display()))?;
    Ok(copied)
}

pub fn path_for(dir: &Path, slot: i64) -> PathBuf {
    dir.join(format!("slot_{slot}.json"))
}

pub fn exists(dir: &Path, slot: i64) -> bool {
    store::current().is_file(&path_for(dir, slot))
}

/// Deletes a slot's save; refused while the slot is locked (`set_locked`).
pub fn delete(dir: &Path, slot: i64) -> Result<(), String> {
    if !exists(dir, slot) {
        return Err(format!("No save in slot {slot}."));
    }
    if is_locked(dir, slot) {
        return Err(locked_text(slot));
    }
    store::current().remove_file(&path_for(dir, slot)).map_err(|_| format!("Slot {slot} could not be deleted."))
}

// ---- Slot locks (PLAYUI1 section 1): a sidecar file beside the saves, so a lock never rewrites a save ----------

/// The lock file in a save folder: `{"locked": [<slot>, ...]}` (sorted, unique, manual slots only).
pub const LOCKS_FILE: &str = "locks.json";

fn locked_text(slot: i64) -> String {
    format!("Slot {slot} is locked.")
}

/// The slots listed in `<dir>/locks.json`; a missing or unreadable file lists none.
fn listed_locks(dir: &Path) -> Vec<i64> {
    let text = store::current().read_to_string(&dir.join(LOCKS_FILE)).unwrap_or_default();
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(o)) => json::aget(&o, "locked").iter().filter(|v| v.is_number()).map(json::to_int).collect(),
        _ => Vec::new(),
    }
}

/// Whether `slot` is locked: listed in the lock file while its save file exists (a stale entry counts for nothing).
pub fn is_locked(dir: &Path, slot: i64) -> bool {
    exists(dir, slot) && listed_locks(dir).contains(&slot)
}

/// Locks or unlocks a manual slot's save. Locking needs a save in the slot; the autosave (`fmt.first_slot`) is never
/// locked. Every write of the lock file drops stale entries (slots without a save, slots outside the manual range);
/// `write` into an empty slot drops that slot's stale entry too.
pub fn set_locked(fmt: &Format, dir: &Path, slot: i64, on: bool) -> Result<(), String> {
    if slot == fmt.first_slot {
        return Err("The autosave can't be locked.".into());
    }
    if !(fmt.first_slot + 1..=fmt.last_slot).contains(&slot) {
        return Err(format!("Save slot must be {}-{}.", fmt.first_slot + 1, fmt.last_slot));
    }
    if on && !exists(dir, slot) {
        return Err(format!("No save in slot {slot}."));
    }
    let mut locked: Vec<i64> = listed_locks(dir).into_iter().filter(|s| *s != slot && (fmt.first_slot + 1..=fmt.last_slot).contains(s) && exists(dir, *s)).collect();
    if on {
        locked.push(slot);
    }
    locked.sort_unstable();
    locked.dedup();
    let st = store::current();
    if locked.is_empty() && !st.exists(&dir.join(LOCKS_FILE)) {
        return Ok(());
    }
    let text = serde_json::to_string_pretty(&serde_json::json!({ "locked": locked })).unwrap_or_default();
    let (path, tmp) = (dir.join(LOCKS_FILE), dir.join(format!("{LOCKS_FILE}.fc_tmp")));
    if st.write(&tmp, text.as_bytes()).is_err() || st.rename(&tmp, &path).is_err() {
        let _ = st.remove_file(&tmp);
        return Err(format!("Can't write '{}'.", path.display()));
    }
    Ok(())
}

fn now() -> i64 {
    aegis_core::clock::SystemTime::now().duration_since(aegis_core::clock::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Problem strings; empty = written. `state` gives the game_state value or the game's own problems (checked after
/// the slot). `saved_at` > 0 keeps a given save time.
pub fn write(fmt: &Format, dir: &Path, slot: i64, saved_at: i64, state: impl FnOnce() -> Result<Value, Vec<String>>) -> Vec<String> {
    if !(fmt.first_slot..=fmt.last_slot).contains(&slot) {
        return vec![format!("Save slot must be {}-{}.", fmt.first_slot, fmt.last_slot)];
    }
    // The autosave is never locked, even by a hand-edited lock file.
    if slot != fmt.first_slot && is_locked(dir, slot) {
        return vec![locked_text(slot)];
    }
    let state = match state() {
        Ok(state) => state,
        Err(errors) => return errors,
    };
    // A stale lock (its save went outside the engine) must not lock the new save: drop it first.
    if !exists(dir, slot) && slot != fmt.first_slot && listed_locks(dir).contains(&slot) {
        let _ = set_locked(fmt, dir, slot, false);
    }
    write_envelope(fmt, dir, &path_for(dir, slot), saved_at, state)
}

/// Writes `state` in the envelope to `final_path` through a temp file and a rename; problem strings, empty = written.
fn write_envelope(fmt: &Format, dir: &Path, final_path: &Path, saved_at: i64, state: Value) -> Vec<String> {
    let envelope = json::obj([
        ("format", fmt.tag.into()),
        ("version", int(fmt.version)),
        ("saved_at", int(if saved_at > 0 { saved_at } else { now() })),
        ("game_state", state),
    ]);
    let st = store::current();
    if st.create_dir_all(dir).is_err() && !st.is_dir(dir) {
        return vec![format!("Can't create the save folder '{}'.", dir.display())];
    }
    let mut tmp = final_path.with_extension("json.fc_tmp");
    let mut n = 1;
    while st.exists(&tmp) {
        tmp = final_path.with_extension(format!("json.fc_tmp{n}"));
        n += 1;
    }
    let mut buf = Vec::new();
    let pretty = serde_json::ser::PrettyFormatter::with_indent(b"\t");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, pretty);
    if serde::Serialize::serialize(&json::sorted(&Value::Object(envelope)), &mut ser).is_err() || st.write(&tmp, &buf).is_err() {
        let _ = st.remove_file(&tmp);
        return vec![format!("Can't write '{}'.", final_path.display())];
    }
    if st.rename(&tmp, final_path).is_err() {
        let _ = st.remove_file(&tmp);
        return vec!["Can't move the save into place (the previous save was kept).".into()];
    }
    Vec::new()
}

/// A slot's file text, or the refusal when the slot is empty.
pub fn read(dir: &Path, slot: i64) -> Result<String, Vec<String>> {
    if !exists(dir, slot) {
        return Err(vec![format!("No save in slot {slot}.")]);
    }
    Ok(store::current().read_to_string(&path_for(dir, slot)).unwrap_or_default())
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
    peek_file(fmt, &path_for(dir, slot))
}

fn peek_file(fmt: &Format, path: &Path) -> Peek {
    let st = store::current();
    if !st.is_file(path) {
        return Peek::Missing;
    }
    let text = st.read_to_string(path).unwrap_or_default();
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

// ---- Keyed saves (CAMPAIGN1 F5): a save named by a key instead of a slot number (an act-start save) -------------

/// The longest save key.
pub const KEY_MAX: usize = 64;

/// Whether `key` names a keyed save: 1-`KEY_MAX` characters, each a-z, 0-9, '_' or '-' (so the file name is safe on
/// every platform and never reads as a slot). The refusal text otherwise.
pub fn check_key(key: &str) -> Result<(), String> {
    let ok = !key.is_empty() && key.len() <= KEY_MAX && key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-');
    if ok {
        Ok(())
    } else {
        Err(format!("Save key '{key}' must be 1-{KEY_MAX} characters a-z, 0-9, '_' or '-'."))
    }
}

/// A keyed save's file: `<dir>/key_<key>.json` (the key is not checked here; the keyed calls check it).
pub fn key_path(dir: &Path, key: &str) -> PathBuf {
    dir.join(format!("key_{key}.json"))
}

/// Whether a keyed save exists; a bad key has none.
pub fn exists_keyed(dir: &Path, key: &str) -> bool {
    check_key(key).is_ok() && store::current().is_file(&key_path(dir, key))
}

/// `write` for a keyed save: the same envelope and atomic write, no slot range and no locks; a later write with the
/// same key replaces it. Problem strings; empty = written.
pub fn write_keyed(fmt: &Format, dir: &Path, key: &str, saved_at: i64, state: impl FnOnce() -> Result<Value, Vec<String>>) -> Vec<String> {
    if let Err(problem) = check_key(key) {
        return vec![problem];
    }
    match state() {
        Ok(state) => write_envelope(fmt, dir, &key_path(dir, key), saved_at, state),
        Err(errors) => errors,
    }
}

/// A keyed save's file text (for `open` and `migrate`, like `read`), or the refusal.
pub fn read_keyed(dir: &Path, key: &str) -> Result<String, Vec<String>> {
    check_key(key).map_err(|problem| vec![problem])?;
    if !exists_keyed(dir, key) {
        return Err(vec![format!("No save '{key}'.")]);
    }
    Ok(store::current().read_to_string(&key_path(dir, key)).unwrap_or_default())
}

/// `peek` for a keyed save; a bad key is `Missing`.
pub fn peek_keyed(fmt: &Format, dir: &Path, key: &str) -> Peek {
    if check_key(key).is_err() {
        return Peek::Missing;
    }
    peek_file(fmt, &key_path(dir, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FMT: Format = Format { tag: "test_save", version: 3, first_slot: 0, last_slot: 2, keys: &["a", "b"] };

    fn temp_dir(name: &str) -> aegis_core::scratch::Scratch {
        let dir = aegis_core::scratch::Scratch::vacant(&format!("save_{name}"));
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
        assert!(delete(&dir, 2).is_ok() && !exists(&dir, 2));
        assert_eq!(delete(&dir, 2), Err("No save in slot 2.".to_string()));
        assert_eq!(read(&dir, 2).unwrap_err(), vec!["No save in slot 2.".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn adopt_shared_copies_the_shared_folder_once() {
        let base = temp_dir("adopt");
        std::fs::create_dir_all(base.join("Aegis/saves")).unwrap();
        std::fs::write(base.join("Aegis/saves/slot_1.json"), "{}").unwrap();
        std::fs::write(base.join("Aegis/settings.json"), "{\"mute\": true}").unwrap();
        std::env::set_var("APPDATA", base.path());
        let mut copied = adopt_shared("demo").unwrap();
        copied.sort();
        assert_eq!(copied, vec![Path::new("saves").join("slot_1.json").display().to_string(), "settings.json".to_string()]);
        assert!(base.join("Aegis/demo/saves/slot_1.json").is_file() && base.join("Aegis/saves/slot_1.json").is_file());
        std::fs::write(base.join("Aegis/saves/slot_2.json"), "{}").unwrap();
        assert!(adopt_shared("demo").unwrap().is_empty());
        assert!(!base.join("Aegis/demo/saves/slot_2.json").exists());
        assert!(adopt_shared("other").unwrap().len() == 3 && user_dir("other").is_dir());
        let _ = std::fs::remove_dir_all(&base);
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
