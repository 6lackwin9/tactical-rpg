//! WEBREL1-E1b (`docs/specs/WEBREL1.md` 2.2): the web user store, natively through the store trait. The key of a user
//! file is `aegis/<game id>/<relative path>`; the save API runs unchanged on `Keys` (the web store over a key backend,
//! here the in-memory `MemKeys`) and writes the same bytes as on disk; a write past the quota is the save API's usual
//! problem (no panic) and keeps the previous save.

use aegis_save::store::{key_for, with_store, KeyBackend, Keys, MemKeys, Store, WEB_ROOT};
use aegis_save::{exists, is_locked, open, path_for, peek, read, set_locked, write, Format, Peek, LOCKS_FILE};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const FMT: Format = Format { tag: "web_store_test", version: 1, first_slot: 0, last_slot: 3, keys: &["a"] };
const GAME: &str = "demo";

/// The game's user folder as the web build names it (`app_dir()` is `WEB_ROOT` there).
fn web_dir() -> PathBuf {
    Path::new(WEB_ROOT).join(GAME)
}

fn mem(quota: usize) -> Rc<Keys<MemKeys>> {
    Rc::new(Keys::new(MemKeys::with_quota(quota)))
}

#[test]
fn a_user_file_maps_to_its_aegis_game_relative_path_key() {
    assert_eq!(key_for(&web_dir().join("slot_1.json")).as_deref(), Some("aegis/demo/slot_1.json"));
    assert_eq!(key_for(&web_dir().join("settings.json")).as_deref(), Some("aegis/demo/settings.json"));
    assert_eq!(key_for(&web_dir().join("saves").join(LOCKS_FILE)).as_deref(), Some("aegis/demo/saves/locks.json"));
    assert_eq!(key_for(&web_dir().join("./saves/../key_act1.json")).as_deref(), Some("aegis/demo/key_act1.json"), "normalised by component");
    assert_eq!(key_for(Path::new("elsewhere/demo/slot_1.json")), None, "outside the store");
    assert_eq!(key_for(Path::new(WEB_ROOT)), None, "the root itself is no file");
    assert_eq!(key_for(Path::new("aegisx/demo/a.json")), None, "a look-alike root");
}

#[test]
fn the_web_store_keeps_files_by_key_and_derives_folders() {
    let s = mem(0);
    let f = web_dir().join("saves/slot_1.json");
    assert!(!s.is_file(&f) && !s.is_dir(&web_dir()));
    s.create_dir_all(&web_dir().join("saves")).unwrap();
    s.write(&f, b"{}").unwrap();
    assert_eq!(s.backend.get("aegis/demo/saves/slot_1.json").as_deref(), Some("{}"));
    assert!(s.is_file(&f) && s.is_dir(&web_dir()) && s.is_dir(&web_dir().join("saves")) && !s.is_file(&web_dir().join("saves")));
    s.write(&web_dir().join("settings.json"), b"{\"mute\": true}").unwrap();
    assert_eq!(s.list_dir(&web_dir()).unwrap(), vec![web_dir().join("saves"), web_dir().join("settings.json")], "sorted, folders once");
    s.rename(&f, &web_dir().join("saves/slot_2.json")).unwrap();
    assert!(!s.is_file(&f) && s.read_to_string(&web_dir().join("saves/slot_2.json")).unwrap() == "{}");
    assert_eq!(s.remove_file(&f).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    assert_eq!(s.write(Path::new("/tmp/x.json"), b"{}").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied, "outside the store");
    assert_eq!(s.write(&web_dir().join("bin.dat"), &[0xff, 0xfe]).unwrap_err().kind(), std::io::ErrorKind::InvalidData, "text only");
}

#[test]
fn the_save_api_runs_on_the_web_store_and_writes_the_same_bytes_as_on_disk() {
    let state = json!({"a": [1, 2.5, "x"]});
    let disk = aegis_core::scratch::Scratch::vacant("web_store_disk");
    assert!(write(&FMT, &disk, 1, 1234, || Ok(state.clone())).is_empty());
    let on_disk = std::fs::read_to_string(path_for(&disk, 1)).unwrap();

    let s = mem(0);
    let dir = web_dir();
    with_store(s.clone(), || {
        assert!(!exists(&dir, 1));
        assert!(write(&FMT, &dir, 1, 1234, || Ok(state.clone())).is_empty());
        assert!(exists(&dir, 1));
        assert_eq!(read(&dir, 1).unwrap(), on_disk, "web saves are the same bytes");
        assert!(matches!(peek(&FMT, &dir, 1), Peek::Found(_)));
        assert!(open(&FMT, &read(&dir, 1).unwrap()).is_ok());
        set_locked(&FMT, &dir, 1, true).unwrap();
        assert!(is_locked(&dir, 1));
    });
    assert_eq!(s.backend.get("aegis/demo/slot_1.json").as_deref(), Some(on_disk.as_str()));
    assert!(s.backend.get("aegis/demo/locks.json").is_some());
    assert!(s.backend.keys().iter().all(|k| !k.contains("fc_tmp")), "no temp key is left behind");
    assert!(!exists(&dir, 1), "outside with_store the platform store is back");
}

#[test]
fn a_write_past_the_quota_is_a_save_problem_and_keeps_the_previous_save() {
    let small = json!({"a": 1});
    let big = json!({"a": "x".repeat(4000)});
    // room for the first save and its temp copy (a write goes to a temp key, then moves over the save)
    let s = mem(1000);
    let dir = web_dir();
    with_store(s.clone(), || {
        assert!(write(&FMT, &dir, 1, 1234, || Ok(small.clone())).is_empty());
        let kept = read(&dir, 1).unwrap();
        let problems = write(&FMT, &dir, 1, 1235, || Ok(big.clone()));
        assert_eq!(problems.len(), 1, "one problem, no panic: {problems:?}");
        assert!(problems[0].starts_with("Can't write"), "the save API's usual problem: {problems:?}");
        assert_eq!(read(&dir, 1).unwrap(), kept, "the previous save is kept");
    });
    assert!(s.backend.used() <= 1000);
    assert!(s.backend.keys().iter().all(|k| !k.contains("fc_tmp")), "the refused temp key is not left behind");
}

#[test]
fn the_quota_counts_keys_and_values_and_a_refused_set_changes_nothing() {
    let b = MemKeys::with_quota(20);
    assert!(b.set("aegis/a", "0123456789").is_ok(), "7 + 10 bytes");
    assert_eq!(b.used(), 17);
    assert!(b.set("aegis/a", "0123456789abcd").is_err(), "7 + 14 > 20");
    assert_eq!(b.get("aegis/a").as_deref(), Some("0123456789"));
    assert!(b.set("aegis/a", "0123456789abc").is_ok(), "replacing counts the new value only: 7 + 13");
    let s = Keys::new(MemKeys::with_quota(20));
    assert_eq!(s.write(&web_dir().join("big.json"), b"too long for twenty bytes").unwrap_err().kind(), std::io::ErrorKind::StorageFull);
}
