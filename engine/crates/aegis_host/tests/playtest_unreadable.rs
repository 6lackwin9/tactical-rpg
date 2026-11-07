//! CIPANES1: an unreadable recording's `reason` is what the recordings screen draws, so it holds no path and no operating
//! system wording (a `\` in a path once made the drawn frame differ between Windows and Linux); `detail` keeps the full
//! cause and is what `Recording::load` answers.

use aegis_host::playtest::{self, Recording, Unreadable};
use std::path::{Path, PathBuf};

/// A scratch folder holding `pt.json` with `bytes`.
fn folder(tag: &str, bytes: &[u8]) -> aegis_core::scratch::Scratch {
    let dir = aegis_core::scratch::Scratch::new(&format!("unreadable_{tag}"));
    std::fs::write(dir.join("pt.json"), bytes).unwrap();
    dir
}

/// What `list` says of the only file in `dir`.
fn listed(dir: &Path) -> (PathBuf, Unreadable) {
    let mut all = playtest::list(dir);
    assert_eq!(all.len(), 1);
    let (path, rec) = all.remove(0);
    (path, rec.err().expect("the file is refused"))
}

/// The same bad file gives the same reason from two folders; the detail names the path and `load` answers it.
fn holds(tag: &str, bytes: &[u8], reason: &str) {
    let (a, b) = (folder(&format!("{tag}_a"), bytes), folder(&format!("{tag}_deeper_b"), bytes));
    let ((path_a, seen_a), (_, seen_b)) = (listed(&a), listed(&b));
    assert_eq!(seen_a.reason, reason);
    assert_eq!(seen_a.reason, seen_b.reason);
    assert!(seen_a.detail.contains(&path_a.display().to_string()), "{}", seen_a.detail);
    assert_eq!(Recording::load(&path_a).err().as_ref(), Some(&seen_a.detail), "load answers the detail");
    let _ = std::fs::remove_dir_all(a);
    let _ = std::fs::remove_dir_all(b);
}

#[test]
fn a_file_that_is_not_json_is_refused_in_words_that_name_no_path() {
    let parse = serde_json::from_slice::<serde_json::Value>(b"not a recording\n").unwrap_err();
    holds("json", b"not a recording\n", &format!("Not valid JSON: {parse}"));
}

/// Bytes that are not text fail in `read_to_string`: the operating system's words stay in the detail.
#[test]
fn a_file_that_cannot_be_read_is_refused_without_the_operating_systems_words() {
    holds("bytes", &[0xff, 0xfe, 0x00], "The file cannot be read.");
}

/// JSON the engine refuses keeps the engine's own sentence on both sides.
#[test]
fn a_refused_recording_shows_the_engines_own_sentence() {
    let dir = folder("refused", b"{\"format\": \"other\"}");
    let (_, seen) = listed(&dir);
    let refused = Recording::from_json(&serde_json::json!({"format": "other"})).err().unwrap();
    assert_eq!(seen, Unreadable { reason: refused.clone(), detail: refused });
    let _ = std::fs::remove_dir_all(dir);
}
