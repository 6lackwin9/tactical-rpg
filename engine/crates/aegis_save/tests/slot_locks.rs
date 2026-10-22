//! PLAYUI1 section 1 rule 5: slot locks in `<save dir>/locks.json`; `write` and `delete` refuse a locked slot, the
//! autosave can't be locked, a lock counts only while its save exists, and the locks are read back from disk.

use aegis_save::{delete, exists, is_locked, set_locked, write, Format, LOCKS_FILE};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const FMT: Format = Format { tag: "lock_test", version: 1, first_slot: 0, last_slot: 3, keys: &["a"] };

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aegis_save_locks_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn save(dir: &Path, slot: i64) -> Vec<String> {
    write(&FMT, dir, slot, 1, || Ok(json!({"a": slot})))
}

fn lock_file(dir: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(dir.join(LOCKS_FILE)).unwrap()).unwrap()
}

#[test]
fn a_locked_slot_refuses_write_and_delete_until_unlocked() {
    let dir = temp_dir("refuse");
    assert!(save(&dir, 2).is_empty());
    assert!(!is_locked(&dir, 2), "no lock file: nothing is locked");
    set_locked(&FMT, &dir, 2, true).unwrap();
    assert!(is_locked(&dir, 2));
    assert_eq!(save(&dir, 2), vec!["Slot 2 is locked.".to_string()]);
    assert_eq!(delete(&dir, 2), Err("Slot 2 is locked.".to_string()));
    assert!(exists(&dir, 2), "the refused delete kept the save");
    assert!(save(&dir, 1).is_empty(), "another slot still saves");
    set_locked(&FMT, &dir, 2, false).unwrap();
    assert!(!is_locked(&dir, 2));
    assert!(save(&dir, 2).is_empty(), "unlocked: the slot saves again");
    assert!(delete(&dir, 2).is_ok() && !exists(&dir, 2));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn locks_live_on_disk_sorted_and_unique() {
    let dir = temp_dir("disk");
    for slot in [3, 1, 2] {
        assert!(save(&dir, slot).is_empty());
    }
    for slot in [3, 1, 3] {
        set_locked(&FMT, &dir, slot, true).unwrap();
    }
    assert_eq!(lock_file(&dir), json!({"locked": [1, 3]}), "sorted, unique");
    // A restart reads the file again: nothing is cached between calls.
    assert_eq!((1..=3).map(|s| is_locked(&dir, s)).collect::<Vec<_>>(), vec![true, false, true]);
    set_locked(&FMT, &dir, 1, false).unwrap();
    assert_eq!(lock_file(&dir), json!({"locked": [3]}));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_autosave_and_empty_slots_can_not_be_locked() {
    let dir = temp_dir("refuse_lock");
    assert!(save(&dir, 0).is_empty());
    assert_eq!(set_locked(&FMT, &dir, 0, true), Err("The autosave can't be locked.".to_string()));
    assert_eq!(set_locked(&FMT, &dir, 2, true), Err("No save in slot 2.".to_string()));
    assert_eq!(set_locked(&FMT, &dir, 4, true), Err("Save slot must be 1-3.".to_string()));
    assert!(set_locked(&FMT, &dir, 2, false).is_ok(), "unlocking an unlocked slot is fine");
    assert!(!dir.join(LOCKS_FILE).exists(), "nothing to keep: no lock file is written");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_stale_or_unreadable_lock_counts_for_nothing() {
    let dir = temp_dir("stale");
    assert!(save(&dir, 1).is_empty() && save(&dir, 2).is_empty());
    set_locked(&FMT, &dir, 1, true).unwrap();
    set_locked(&FMT, &dir, 2, true).unwrap();
    // The save of slot 1 goes outside the engine (a player deleting the file): its entry no longer locks the slot.
    std::fs::remove_file(aegis_save::path_for(&dir, 1)).unwrap();
    assert!(!is_locked(&dir, 1));
    assert_eq!(lock_file(&dir), json!({"locked": [1, 2]}), "kept until the next lock file write");
    set_locked(&FMT, &dir, 3, false).unwrap();
    assert_eq!(lock_file(&dir), json!({"locked": [2]}), "the next set_locked drops the stale entry");
    set_locked(&FMT, &dir, 2, true).unwrap();
    std::fs::remove_file(aegis_save::path_for(&dir, 2)).unwrap();
    assert!(save(&dir, 2).is_empty(), "a new save in a slot with a stale entry is written");
    assert!(!is_locked(&dir, 2), "and the stale lock does not come back with it");
    assert!(!dir.join(LOCKS_FILE).exists() || lock_file(&dir) == json!({"locked": []}));
    set_locked(&FMT, &dir, 2, true).unwrap();
    std::fs::write(dir.join(LOCKS_FILE), "not json").unwrap();
    assert!(!is_locked(&dir, 2), "an unreadable lock file locks nothing");
    assert!(save(&dir, 2).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
