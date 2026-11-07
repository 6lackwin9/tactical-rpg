//! CAMPAIGN1 F5: keyed saves (an act-start save `act<n>_<run>`) in the slot envelope, beside the slots but outside
//! the slot range: read back as written, never overwritten by a slot save, never listed as a slot, a bad key refused.

use aegis_save::{
    check_key, delete, exists, exists_keyed, key_path, migrate, newest_slot, open, path_for, peek, peek_keyed, read, read_keyed, set_locked, write,
    write_keyed, Format, Peek, KEY_MAX,
};
use aegis_core::json;
use serde_json::{json, Value};
use std::path::Path;

const FMT: Format = Format { tag: "keyed_test", version: 2, first_slot: 0, last_slot: 5, keys: &["a", "act"] };
const KEY: &str = "act1_ab12cd34";

fn temp_dir(name: &str) -> aegis_core::scratch::Scratch {
    let dir = aegis_core::scratch::Scratch::vacant(&format!("save_keyed_{name}"));
    dir
}

fn files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir).map(|r| r.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect()).unwrap_or_default();
    names.sort();
    names
}

/// The slots a load menu lists: every slot in the range whose header reads.
fn listed_slots(dir: &Path) -> Vec<i64> {
    (FMT.first_slot..=FMT.last_slot).filter(|s| matches!(peek(&FMT, dir, *s), Peek::Found(_))).collect()
}

#[test]
fn a_keyed_save_reads_back_as_the_same_envelope_a_slot_would() {
    let dir = temp_dir("roundtrip");
    let state = json!({"a": 7, "act": 1});
    assert!(write_keyed(&FMT, &dir, KEY, 1234, || Ok(state.clone())).is_empty());
    assert!(write(&FMT, &dir, 1, 1234, || Ok(state.clone())).is_empty());
    assert!(exists_keyed(&dir, KEY) && key_path(&dir, KEY).is_file());
    let keyed = open(&FMT, &read_keyed(&dir, KEY).unwrap()).unwrap();
    let slot = open(&FMT, &read(&dir, 1).unwrap()).unwrap();
    assert_eq!(keyed, slot, "same envelope (format, version, saved_at, game_state)");
    assert_eq!(json::s(&keyed, "format"), FMT.tag);
    assert_eq!(json::i(&keyed, "version", 0), FMT.version);
    assert_eq!(json::i(&keyed, "saved_at", 0), 1234);
    assert_eq!(Value::Object(json::dget(&keyed, "game_state").clone()), state);
    let Peek::Found(header) = peek_keyed(&FMT, &dir, KEY) else { panic!("the keyed save peeks as found") };
    assert_eq!(header, keyed);
    let (migrated, warnings) = migrate(&FMT, &keyed, |_, s, _| Some(s.clone())).unwrap();
    assert!(warnings.is_empty() && migrated == keyed, "a current keyed save migrates to itself");
    // A later write under the same key replaces it.
    assert!(write_keyed(&FMT, &dir, KEY, 2000, || Ok(json!({"a": 8}))).is_empty());
    assert_eq!(json::i(&open(&FMT, &read_keyed(&dir, KEY).unwrap()).unwrap(), "saved_at", 0), 2000);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn keyed_saves_are_not_slots_and_slot_saves_never_touch_them() {
    let dir = temp_dir("not_slots");
    assert!(write_keyed(&FMT, &dir, KEY, 9999, || Ok(json!({"act": 1}))).is_empty());
    // A key that reads like a slot number is still a keyed save, not that slot.
    assert!(write_keyed(&FMT, &dir, "1", 9999, || Ok(json!({"act": 2}))).is_empty());
    assert!(listed_slots(&dir).is_empty(), "no slot lists a keyed save");
    assert!(!exists(&dir, 1) && read(&dir, 1).is_err());
    let newest = |dir: &Path| newest_slot(&FMT, |s| if let Peek::Found(e) = peek(&FMT, dir, s) { Some(json::i(&e, "saved_at", 0)) } else { None });
    assert_eq!(newest(&dir), -1, "the newest-save pick sees no slot");
    let before = std::fs::read_to_string(key_path(&dir, KEY)).unwrap();
    for slot in FMT.first_slot..=FMT.last_slot {
        assert!(write(&FMT, &dir, slot, 10 + slot, || Ok(json!({"a": slot}))).is_empty());
    }
    assert_eq!(listed_slots(&dir), (FMT.first_slot..=FMT.last_slot).collect::<Vec<_>>());
    assert_eq!(newest(&dir), FMT.last_slot, "the newer keyed save does not count as the newest slot");
    set_locked(&FMT, &dir, 2, true).unwrap();
    assert!(delete(&dir, 3).is_ok());
    assert_eq!(std::fs::read_to_string(key_path(&dir, KEY)).unwrap(), before, "slot writes, locks and deletes leave it as it was");
    assert_eq!(json::i(json::dget(&open(&FMT, &read_keyed(&dir, "1").unwrap()).unwrap(), "game_state"), "act", 0), 2);
    let slot_files: Vec<String> = (FMT.first_slot..=FMT.last_slot).filter(|s| *s != 3).map(|s| path_for(&dir, s).file_name().unwrap().to_string_lossy().into_owned()).collect();
    let mut expected = slot_files;
    expected.extend(["key_1.json".to_string(), format!("key_{KEY}.json"), "locks.json".to_string()]);
    expected.sort();
    assert_eq!(files(&dir), expected, "the keyed files sit beside the slots under their own names");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_bad_key_is_refused_and_writes_nothing() {
    let dir = temp_dir("bad_key");
    let long = "a".repeat(KEY_MAX + 1);
    for key in ["", "Act1", "act 1", "../act1", "act1.json", "act/1", "akt\u{e9}", long.as_str()] {
        let problem = check_key(key).unwrap_err();
        assert_eq!(write_keyed(&FMT, &dir, key, 1, || Ok(json!({"a": 1}))), vec![problem.clone()], "{key:?}");
        assert_eq!(read_keyed(&dir, key).unwrap_err(), vec![problem], "{key:?}");
        assert!(!exists_keyed(&dir, key) && matches!(peek_keyed(&FMT, &dir, key), Peek::Missing), "{key:?}");
    }
    assert!(files(&dir).is_empty(), "no file and no folder for a refused key");
    assert!(check_key(&"a".repeat(KEY_MAX)).is_ok() && check_key("act3_run-0").is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_failed_or_corrupt_keyed_save_is_reported() {
    let dir = temp_dir("missing");
    assert_eq!(read_keyed(&dir, KEY).unwrap_err(), vec![format!("No save '{KEY}'.")]);
    assert!(matches!(peek_keyed(&FMT, &dir, KEY), Peek::Missing));
    assert_eq!(write_keyed(&FMT, &dir, KEY, 1, || Err(vec!["bad state".into()])), vec!["bad state".to_string()]);
    assert!(!exists_keyed(&dir, KEY), "a refused state writes nothing");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(key_path(&dir, KEY), "{").unwrap();
    assert!(matches!(peek_keyed(&FMT, &dir, KEY), Peek::Unreadable));
    assert_eq!(open(&FMT, &read_keyed(&dir, KEY).unwrap()).unwrap_err(), vec!["Save file is corrupt or truncated.".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}
