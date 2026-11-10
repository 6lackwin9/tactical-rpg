//! WEBREL1-E1b runtime acceptance (`docs/specs/WEBREL1.md` 2.1, 2.2, 6.1; `FB-147`): runtime code reads data, assets
//! and fonts only through `aegis_core::vfs` and user files only through `aegis_save`'s store. std's file system
//! compiles on `wasm32-unknown-unknown` and returns `Err` there at run time, so this lint refuses every `std::fs` path
//! (`use std::fs`, `std::fs::read`, `use std::{fs, ..}`, `std::fs::*`) and the no-argument std path probes
//! (`p.is_file()`, `p.is_dir()`, `p.exists()`, `p.metadata()`, `p.read_dir()`, ...) in the non-test, non-CLI `src`
//! of every engine, kit and game crate.
//!
//! What is not scanned (test-only and native-only items, `src/bin/`, comments, literals): `lint/mod.rs`; so native-only
//! code (`#[cfg(not(target_arch = "wasm32"))]`, for example the tuning hot reload or the F12 screenshot) may use
//! `std::fs`. CLI crates: `CLI_CRATES`. Allowed sites, each with its reason: `ALLOWED`. The clock twin is
//! `clock_lint.rs`.

mod lint;

use lint::{child_dir, lint_tree, root, src_dirs, Rule};
use std::path::Path;

/// Files that may use `std::fs`, each with its reason (keep it short).
const ALLOWED: &[(&str, &str)] = &[
    ("engine/crates/aegis_core/src/vfs.rs", "the read seam itself and its native-only dev files (std::fs on native)"),
    ("engine/crates/aegis_save/src/store.rs", "the native user store (Files: std::fs)"),
    ("engine/crates/aegis_core/src/scratch.rs", "test scratch folders (TESTCLEAN1): called by tests only, never at run time"),
];

/// Files whose std::fs *writes* are native tooling (`aegis content`'s writer: upsert, remove, set_world, game.json and
/// kit.json); their reads and probes still go through the seam, so only the `WRITE_FNS` calls are passed.
const WRITERS: &[(&str, &str)] = &[
    ("engine/crates/aegis_content/src/store.rs", "the content writer (aegis content upsert / remove / set_world)"),
    ("engine/crates/aegis_content/src/manifest.rs", "game.json / kit.json writers (aegis content game / kit set)"),
];

/// The std::fs calls a `WRITERS` file may make.
const WRITE_FNS: &[&str] = &["write", "create_dir_all", "remove_file", "remove_dir"];

/// Crates that are command-line tools only (never part of the web build), each with its reason.
const CLI_CRATES: &[&str] = &[
    // the `aegis` command line (content writer, sim, scenario shots); not a dependency of any game binary
    "aegis_cli",
];

/// No-argument std path methods that touch the file system.
const PROBES: &[&str] = &["is_file", "is_dir", "exists", "try_exists", "metadata", "symlink_metadata", "read_dir", "canonicalize", "read_link"];

/// Why `p` (a full path, `*` for a glob) is refused, if it is.
fn refused(p: &[&str]) -> Option<String> {
    match p {
        ["std", "fs", ..] => Some(format!("names {} (read through aegis_core::vfs; user files through aegis_save)", p.join("::"))),
        _ => None,
    }
}

const RULE: Rule = Rule { heads: &["std"], refused, probes: PROBES };

fn violations(text: &str) -> (Vec<(usize, String)>, Vec<String>) {
    lint::violations(text, &RULE)
}

/// Whether a lint line (`path:line: names std::fs::<fn> ...`) is a `WRITE_FNS` call in a `WRITERS` file.
fn writer_call(line: &str) -> bool {
    WRITERS.iter().any(|(p, _)| line.starts_with(&format!("{p}:"))) && WRITE_FNS.iter().any(|f| line.contains(&format!(": names std::fs::{f} (")))
}

fn lines_of(src: &str) -> Vec<usize> {
    violations(src).0.into_iter().map(|(l, _)| l).collect()
}

#[test]
fn runtime_code_touches_the_file_system_only_through_the_seams() {
    let root = root();
    for (p, _) in ALLOWED {
        assert!(root.join(p).is_file(), "an allowed site that no longer exists: {p} (drop it from ALLOWED)");
    }
    for (p, _) in WRITERS {
        assert!(root.join(p).is_file(), "a writer that no longer exists: {p} (drop it from WRITERS)");
    }
    let found: Vec<String> = lint_tree(&root, &RULE, ALLOWED, CLI_CRATES).into_iter().filter(|line| !writer_call(line)).collect();
    assert!(found.is_empty(), "std::fs outside aegis_core::vfs and the aegis_save store (WEBREL1-E1b; std's file system fails on the web):\n{}", found.join("\n"));
}

#[test]
fn the_lint_catches_every_spelling_of_std_fs_and_the_path_probes() {
    let cases = [
        "use std::fs;",
        "use std::fs::File;",
        "use std::fs::{self, File};",
        "use std::{fs, path::Path};",
        "use std::{io, fs::read_to_string};",
        "use std::fs::*;",
        "use ::std::fs;",
        "fn f() -> String { std::fs::read_to_string(\"a\").unwrap_or_default() }",
        "fn f(p: &Path) -> bool { p.is_file() }",
        "fn f(p: &Path) -> bool { p.join(\"x\").is_dir() }",
        "fn f(p: &Path) -> bool { p.exists() }",
        "fn f(p: &Path) -> u64 { p.metadata().map(|m| m.len()).unwrap_or(0) }",
        "fn f(p: &Path) { for e in p.read_dir().unwrap() { let _ = e; } }",
    ];
    for src in cases {
        assert!(!lines_of(src).is_empty(), "missed: {src}");
    }
    assert_eq!(lines_of("use std::path::Path;\n\nfn f(p: &Path) -> bool {\n    p.is_file()\n}"), vec![4], "the line of the probe");
}

#[test]
fn the_lint_passes_the_seams_native_only_code_comments_and_tests() {
    let clean = [
        "use std::path::{Path, PathBuf};",
        "use aegis_core::vfs;\nfn f(p: &Path) -> bool { vfs::is_file(p) }",
        "fn f(s: &dyn Store, p: &Path) -> bool { s.is_file(p) }",
        "// std::fs::read is a run-time Err on the web\nfn f() {}",
        "const S: &str = \"p.is_file()\";",
        "#[cfg(test)]\nmod tests {\n    use std::fs;\n    fn f() { let _ = fs::read(\"a\"); }\n}",
        "#[cfg(not(target_arch = \"wasm32\"))]\nfn shot(p: &Path) { let _ = std::fs::write(p, b\"\"); }",
        "#[cfg(not(target_arch = \"wasm32\"))]\nuse std::fs;",
        "fn f(v: &[u8]) -> bool { v.is_empty() }",
    ];
    for src in clean {
        assert!(lines_of(src).is_empty(), "flagged: {src}: {:?}", violations(src).0);
    }
    // the skip of a native-only item ends with it
    assert_eq!(lines_of("#[cfg(not(target_arch = \"wasm32\"))]\nfn f() { if true { } }\nuse std::fs;"), vec![3]);
    // a wasm-only item is runtime web code: scanned
    assert_eq!(lines_of("#[cfg(target_arch = \"wasm32\")]\nfn f() { let _ = std::fs::read(\"a\"); }"), vec![2]);
}

#[test]
fn a_native_only_module_file_and_the_cli_crates_are_skipped() {
    let (_, mods) = violations("mod board;\n#[cfg(not(target_arch = \"wasm32\"))]\nmod headless;\n#[cfg(test)]\nmod tests;\n");
    assert_eq!(mods, vec!["headless".to_string(), "tests".to_string()]);
    assert_eq!(child_dir(Path::new("src/shell.rs")), Path::new("src/shell"));
    let root = root();
    let dirs = src_dirs(&root, CLI_CRATES);
    assert!(dirs.iter().any(|d| d.ends_with("aegis_save/src")), "engine crates are walked");
    assert!(dirs.iter().any(|d| d.starts_with(root.join("kits"))), "kit crates are walked");
    assert!(dirs.iter().any(|d| d.starts_with(root.join("games"))), "game crates are walked");
    assert!(!dirs.iter().any(|d| CLI_CRATES.iter().any(|c| d.ends_with(format!("{c}/src")))), "CLI crates are not walked");
}

#[test]
fn a_content_writer_passes_only_its_writes() {
    let w = WRITERS[0].0;
    assert!(writer_call(&format!("{w}:12: names std::fs::write (read through aegis_core::vfs; user files through aegis_save)")));
    assert!(writer_call(&format!("{w}:12: names std::fs::create_dir_all (read through aegis_core::vfs; user files through aegis_save)")));
    assert!(!writer_call(&format!("{w}:12: names std::fs::read_to_string (read through aegis_core::vfs; user files through aegis_save)")), "a read in a writer is still refused");
    assert!(!writer_call(&format!("{w}:12: calls .is_file() (a std file system probe; use aegis_core::vfs)")), "a probe in a writer is still refused");
    assert!(!writer_call("engine/crates/aegis_ui/src/theme.rs:12: names std::fs::write (read through aegis_core::vfs; user files through aegis_save)"), "a write elsewhere is refused");
}
