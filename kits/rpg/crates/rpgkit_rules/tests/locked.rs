//! BASERULES1-M (docs/specs/BASERULES1.md 4.1): the `LOCKED` table cannot go stale. Every numeric `pub const` of
//! `rpgkit_rules/src` (a number or an array of numbers) is either listed in `LOCKED` (stays code, with why) or in
//! `PROMOTED` (the default of a ruleset row field); every entry of both names a constant that exists, in its file.

use rpgkit_rules::ruleset::{Numbers, LOCKED, PROMOTED};
use std::path::PathBuf;

const NUMERIC: [&str; 12] = ["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize", "f32", "f64"];

/// The numeric `pub const`s of every source file: (file name, constant name).
fn numeric_consts() -> Vec<(String, String)> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&src).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|x| x == "rs")).collect();
    files.sort();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        for line in text.lines() {
            let Some(rest) = line.trim_start().strip_prefix("pub const ") else { continue };
            let Some((name, ty)) = rest.split_once(':') else { continue };
            let ty = ty.split('=').next().unwrap_or("").trim();
            let base = ty.trim_start_matches('[').split(';').next().unwrap_or("").trim();
            if NUMERIC.contains(&base) {
                out.push((file.clone(), name.trim().to_string()));
            }
        }
    }
    out
}

#[test]
fn every_numeric_pub_const_is_locked_or_promoted() {
    let found = numeric_consts();
    assert!(found.len() >= LOCKED.len(), "the scan finds the constants ({})", found.len());
    for (file, name) in &found {
        let locked = LOCKED.iter().any(|l| l.name == name && l.file == file);
        let promoted = PROMOTED.iter().any(|(p, _)| p == name);
        assert!(locked || promoted, "{file}: pub const {name} is neither in LOCKED nor in PROMOTED (rpgkit_rules::ruleset)");
    }
    for l in LOCKED.iter() {
        assert!(found.iter().any(|(f, n)| n == l.name && f == l.file), "LOCKED lists {}::{}, which is not a numeric pub const there", l.file, l.name);
        assert!(!l.system.is_empty() && !l.why.is_empty(), "{}: system and why", l.name);
        assert_eq!(LOCKED.iter().filter(|x| x.name == l.name).count(), 1, "{} once", l.name);
    }
    for (name, key) in PROMOTED.iter() {
        assert!(found.iter().any(|(_, n)| n == name), "PROMOTED lists {name}, which is not a numeric pub const");
        assert!(Numbers::default().field(key).is_some(), "{name} is the default of ruleset field {key}");
    }
}
